//! Test-only owner. It owns real foreground children in the test host namespace.
//! It deliberately supports only DirectProcess cleanup, never Insula/cgroup
//! containment. Do not install this as the production Insula launcher.
#![allow(dead_code)]

use async_trait::async_trait;
use insula_supervisor::{
    model::*,
    owner_rpc::{
        read_frame, write_frame, OwnerReply, OwnerRequest, RpcRequest, RpcResponse,
        PROTOCOL_VERSION,
    },
    probes::LocalProbes,
    Error, ExecutionOwner, Result,
};
use rustix::{
    fd::OwnedFd,
    fs::{flock, FlockOperation},
    process::{pidfd_open, pidfd_send_signal, Pid, PidfdFlags, Signal},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::{
        fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        process::ExitStatusExt,
    },
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        atomic::{AtomicU64, AtomicU8, Ordering},
        Arc, Weak,
    },
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    net::{UnixListener, UnixStream},
    process::{Child, Command},
    sync::{oneshot, Mutex},
};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Reserved,
    Alive,
    Gone,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct SocketStamp {
    device: u64,
    inode: u64,
    uid: u32,
    gid: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TmuxEndpoint {
    path: PathBuf,
    parent_device: u64,
    parent_inode: u64,
    uid: u32,
    #[serde(default)]
    stamp: Option<SocketStamp>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Receipt {
    key: ServiceKey,
    generation: Generation,
    launch_operation: OperationId,
    launch_digest: String,
    phase: Phase,
    identity: Option<ProcessIdentity>,
    last_exit: Option<ExitInfo>,
    #[serde(default)]
    endpoint: Option<TmuxEndpoint>,
}

struct Entry {
    receipt: Receipt,
    child: Option<Child>,
    pidfd: Option<OwnedFd>,
    spec: Option<ServiceSpec>,
}

#[derive(Clone, Serialize, Deserialize)]
struct CompletedOperation {
    operation_id: OperationId,
    digest: String,
    observation: Observation,
}

#[derive(Clone, Serialize, Deserialize)]
struct PendingStop {
    operation_id: OperationId,
    digest: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Journal {
    epoch: u64,
    receipts: Vec<Receipt>,
    operations: Vec<CompletedOperation>,
    #[serde(default)]
    pending_stops: Vec<PendingStop>,
}

struct Gate {
    epoch: u64,
    ownership: Option<Ownership>,
    entries: BTreeMap<ServiceKey, Entry>,
    completed: BTreeMap<OperationId, CompletedOperation>,
    pending_stops: BTreeMap<OperationId, PendingStop>,
    environment: BTreeMap<String, String>,
}

pub struct FixtureOwner {
    self_ref: Weak<FixtureOwner>,
    authority: PathBuf,
    owner_instance: Uuid,
    _owner_lock: File,
    gate: Mutex<Gate>,
    probes: LocalProbes,
    launches: AtomicU64,
    fail_launch: AtomicU8,
    launch_delay_ms: AtomicU64,
}

impl FixtureOwner {
    pub fn new(authority: &Path) -> Result<Arc<Self>> {
        fs::create_dir_all(authority)?;
        fs::set_permissions(authority, fs::Permissions::from_mode(0o700))?;
        let authority = authority.canonicalize()?;
        let owner_lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(authority.join("owner.lock"))?;
        flock(&owner_lock, FlockOperation::NonBlockingLockExclusive)
            .map_err(|_| Error::Ownership)?;
        let owner_id_path = authority.join("owner-instance");
        let owner_instance = match fs::read_to_string(&owner_id_path) {
            Ok(value) => Uuid::parse_str(value.trim()).map_err(|_| Error::Storage)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let id = Uuid::new_v4();
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .open(owner_id_path)?;
                file.write_all(id.to_string().as_bytes())?;
                file.sync_all()?;
                File::open(&authority)?.sync_all()?;
                id
            }
            Err(_) => return Err(Error::Storage),
        };
        let journal: Journal = match fs::read(authority.join("receipts.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| Error::Storage)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Journal::default(),
            Err(_) => return Err(Error::Storage),
        };
        let probes = LocalProbes::new(4)?;
        Ok(Arc::new_cyclic(move |self_ref| Self {
            self_ref: self_ref.clone(),
            authority,
            owner_instance,
            _owner_lock: owner_lock,
            gate: Mutex::new(Gate {
                epoch: journal.epoch,
                ownership: None,
                // A restarted fixture has no live child handles. Persisted
                // Alive/Reserved entries remain Unknown, never reconstructed
                // from their numeric PID and never silently relaunched.
                entries: journal
                    .receipts
                    .into_iter()
                    .map(|receipt| {
                        (
                            receipt.key.clone(),
                            Entry {
                                receipt,
                                child: None,
                                pidfd: None,
                                spec: None,
                            },
                        )
                    })
                    .collect(),
                completed: journal
                    .operations
                    .into_iter()
                    .map(|operation| (operation.operation_id, operation))
                    .collect(),
                pending_stops: journal
                    .pending_stops
                    .into_iter()
                    .map(|operation| (operation.operation_id, operation))
                    .collect(),
                environment: BTreeMap::new(),
            }),
            probes,
            launches: AtomicU64::new(0),
            fail_launch: AtomicU8::new(0),
            launch_delay_ms: AtomicU64::new(0),
        }))
    }

    pub fn authority(&self) -> &Path {
        &self.authority
    }
    pub fn total_launches(&self) -> u64 {
        self.launches.load(Ordering::Acquire)
    }
    pub fn fail_next_launch_after_reservation(&self) {
        self.fail_launch.store(1, Ordering::Release);
    }
    pub fn fail_next_launch_after_spawn(&self) {
        self.fail_launch.store(2, Ordering::Release);
    }
    pub fn delay_next_launch(&self, duration: Duration) {
        self.launch_delay_ms
            .store(duration.as_millis() as u64, Ordering::Release);
    }
    pub async fn set_environment_reference(&self, reference: &str, value: &str) {
        self.gate
            .lock()
            .await
            .environment
            .insert(reference.into(), value.into());
    }

    pub async fn fence_controller(&self) {
        let mut gate = self.gate.lock().await;
        gate.ownership = None;
    }

    fn validate_gate(&self, gate: &Gate, ownership: &Ownership) -> Result<()> {
        if gate.ownership.as_ref() != Some(ownership)
            || ownership.owner_instance != self.owner_instance
        {
            return Err(Error::Ownership);
        }
        Ok(())
    }

    fn persist(&self, gate: &Gate) -> Result<()> {
        let journal = Journal {
            epoch: gate.epoch,
            receipts: gate
                .entries
                .values()
                .map(|entry| entry.receipt.clone())
                .collect(),
            operations: gate.completed.values().cloned().collect(),
            pending_stops: gate.pending_stops.values().cloned().collect(),
        };
        let bytes = serde_json::to_vec(&journal).map_err(|_| Error::Storage)?;
        let temp = self.authority.join("receipts.new");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(temp, self.authority.join("receipts.json"))?;
        File::open(&self.authority)?.sync_all()?;
        Ok(())
    }

    fn refresh(entry: &mut Entry) -> Result<()> {
        if let Some(child) = entry.child.as_mut() {
            if let Some(exit) = child.try_wait()? {
                entry.receipt.phase = Phase::Gone;
                entry.receipt.last_exit = Some(ExitInfo {
                    code: exit.code(),
                    signal: exit.signal(),
                    observed_ms: now_ms(),
                });
                entry.pidfd = None;
            }
        }
        Ok(())
    }

    fn endpoint_parent(endpoint: &TmuxEndpoint) -> Result<()> {
        let parent = endpoint.path.parent().ok_or(Error::Unknown)?;
        let metadata = fs::symlink_metadata(parent)?;
        if !metadata.is_dir()
            || metadata.uid() != endpoint.uid
            || metadata.mode() & 0o077 != 0
            || metadata.dev() != endpoint.parent_device
            || metadata.ino() != endpoint.parent_inode
        {
            return Err(Error::Unknown);
        }
        Ok(())
    }

    fn socket_stamp(endpoint: &TmuxEndpoint) -> Result<Option<SocketStamp>> {
        Self::endpoint_parent(endpoint)?;
        let metadata = match fs::symlink_metadata(&endpoint.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Error::Unknown),
        };
        if !metadata.file_type().is_socket()
            || metadata.uid() != endpoint.uid
            || metadata.mode() & 0o077 != 0
        {
            return Err(Error::Unknown);
        }
        Ok(Some(SocketStamp {
            device: metadata.dev(),
            inode: metadata.ino(),
            uid: metadata.uid(),
            gid: metadata.gid(),
        }))
    }

    /// Capture the socket inode only while its connected peer is the owned live
    /// generation. An arbitrary file left at the configured path is not proof.
    async fn capture_endpoint(entry: &mut Entry) -> Result<()> {
        let Some(endpoint) = entry.receipt.endpoint.as_ref() else {
            return Ok(());
        };
        if entry.receipt.phase != Phase::Alive || entry.pidfd.is_none() {
            return Err(Error::Unknown);
        }
        let identity = entry.receipt.identity.as_ref().ok_or(Error::Unknown)?;
        let before = Self::socket_stamp(endpoint)?.ok_or(Error::Unknown)?;
        let stream = tokio::time::timeout(
            Duration::from_millis(200),
            UnixStream::connect(&endpoint.path),
        )
        .await
        .map_err(|_| Error::Unknown)?
        .map_err(|_| Error::Unknown)?;
        let credentials = stream.peer_cred()?;
        if credentials.pid() != Some(identity.pid as i32)
            || credentials.uid() != before.uid
            || credentials.gid() != before.gid
            || !insula_supervisor::probes::verify_process_identity(identity)?
            || Self::socket_stamp(endpoint)? != Some(before.clone())
        {
            return Err(Error::Unknown);
        }
        entry.receipt.endpoint.as_mut().unwrap().stamp = Some(before);
        Ok(())
    }

    /// The fixture may remove only its own captured stale inode after the exact
    /// child has been reaped. A reachable, replaced, symlinked, or unrecorded
    /// endpoint blocks cleanup. The private parent and lifecycle gate protect
    /// the final metadata-check/unlink interval from other admitted owners.
    async fn cleanup_endpoint(entry: &mut Entry) -> Result<()> {
        let Some(endpoint) = entry.receipt.endpoint.as_ref() else {
            return Ok(());
        };
        if entry.receipt.phase != Phase::Gone || entry.pidfd.is_some() {
            return Err(Error::Unknown);
        }
        let current = Self::socket_stamp(endpoint)?;
        if current.is_none() {
            entry.receipt.endpoint.as_mut().unwrap().stamp = None;
            return Ok(());
        }
        let expected = endpoint.stamp.as_ref().ok_or(Error::Unknown)?;
        if current.as_ref() != Some(expected) {
            return Err(Error::Unknown);
        }
        match tokio::time::timeout(
            Duration::from_millis(200),
            UnixStream::connect(&endpoint.path),
        )
        .await
        {
            Ok(Err(error))
                if error.kind() == std::io::ErrorKind::ConnectionRefused
                    || error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err(Error::Unknown),
        }
        if Self::socket_stamp(endpoint)?.as_ref() != Some(expected) {
            return Err(Error::Unknown);
        }
        fs::remove_file(&endpoint.path)?;
        entry.receipt.endpoint.as_mut().unwrap().stamp = None;
        Ok(())
    }

    fn observation(entry: &Entry) -> Observation {
        match entry.receipt.phase {
            Phase::Alive if entry.pidfd.is_some() && entry.child.is_some() => {
                match &entry.receipt.identity {
                    Some(identity) => Observation::Alive {
                        identity: identity.clone(),
                    },
                    None => Observation::Unknown,
                }
            }
            Phase::Gone => Observation::Gone {
                generation: Some(entry.receipt.generation),
                last_exit: entry.receipt.last_exit.clone(),
                boundary_empty: true,
                launch_resolved: true,
            },
            _ => Observation::Unknown,
        }
    }

    fn environment(gate: &Gate, command: &CommandSpec) -> Result<BTreeMap<String, String>> {
        command
            .env_refs
            .iter()
            .map(|(name, reference)| {
                gate.environment
                    .get(reference)
                    .map(|value| (name.clone(), value.clone()))
                    .ok_or(Error::Unavailable)
            })
            .collect()
    }

    fn command(spec: &CommandSpec, environment: &BTreeMap<String, String>) -> Command {
        let mut command = Command::new(&spec.executable);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .env_clear()
            .envs(environment)
            .uid(spec.identity.uid)
            .gid(spec.identity.gid)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        command
    }

    fn signal(entry: &Entry, signal: Signal) -> Result<()> {
        let pidfd = entry.pidfd.as_ref().ok_or(Error::Unknown)?;
        match pidfd_send_signal(pidfd, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(_) => Err(Error::Unavailable),
        }
    }

    /// Cleanup owned test children at test teardown. This is owner teardown,
    /// distinct from controller release. There is no global waitpid reaper.
    pub async fn shutdown_all(&self) -> Result<()> {
        let mut gate = self.gate.lock().await;
        for entry in gate.entries.values_mut() {
            Self::refresh(entry)?;
            if entry.receipt.phase == Phase::Alive {
                Self::signal(entry, Signal::TERM)?;
                if let Some(child) = entry.child.as_mut() {
                    if tokio::time::timeout(Duration::from_millis(500), child.wait())
                        .await
                        .is_err()
                    {
                        child.kill().await?;
                    }
                }
                Self::refresh(entry)?;
            }
        }
        self.persist(&gate)
    }
}

#[async_trait]
impl ExecutionOwner for FixtureOwner {
    async fn capabilities(&self, root_insula_id: &str) -> Result<OwnerCapabilities> {
        if root_insula_id != "root" {
            return Err(Error::Unsupported(
                "fixture serves only the test root".into(),
            ));
        }
        Ok(OwnerCapabilities {
            owner_instance: self.owner_instance,
            root_insula_id: root_insula_id.into(),
            canonical_authority_dir: self.authority.clone(),
            generation_fencing: true,
            controller_fencing: true,
            durable_launch_reservations: true,
            service_boundary_cleanup: false,
            bounded_probes: true,
            bounded_logs: true,
            survives_controller_exit: true,
            details: vec!["TEST FIXTURE: direct host processes only; no Insula namespace or cgroup containment".into(), "DirectProcess cleanup proves only the owned foreground process was reaped; descendants are outside this boundary".into(), "Child stdout/stderr are discarded; no raw output is stored".into(), "Services survive controller loss only while this fixture owner and host remain alive".into()],
        })
    }

    async fn claim(&self, root_insula_id: &str, controller_id: Uuid) -> Result<Ownership> {
        if root_insula_id != "root" {
            return Err(Error::Unsupported("fixture serves only root".into()));
        }
        let mut gate = self.gate.lock().await;
        if gate.ownership.is_some() {
            return Err(Error::Ownership);
        }
        gate.epoch = gate.epoch.checked_add(1).ok_or(Error::Ownership)?;
        let ownership = Ownership {
            root_insula_id: root_insula_id.into(),
            owner_instance: self.owner_instance,
            controller_id,
            epoch: gate.epoch,
        };
        gate.ownership = Some(ownership.clone());
        self.persist(&gate)?;
        Ok(ownership)
    }

    async fn validate(&self, ownership: &Ownership) -> Result<()> {
        self.validate_gate(&*self.gate.lock().await, ownership)
    }

    async fn observe(
        &self,
        ownership: &Ownership,
        key: &ServiceKey,
        generation: Option<Generation>,
    ) -> Result<Observation> {
        let mut gate = self.gate.lock().await;
        self.validate_gate(&gate, ownership)?;
        let Some(entry) = gate.entries.get_mut(key) else {
            return Ok(Observation::Gone {
                generation,
                last_exit: None,
                boundary_empty: true,
                launch_resolved: true,
            });
        };
        if generation.is_some_and(|generation| generation != entry.receipt.generation) {
            return Err(Error::StaleGeneration);
        }
        Self::refresh(entry)?;
        let observation = Self::observation(entry);
        self.persist(&gate)?;
        Ok(observation)
    }

    async fn launch(&self, ownership: &Ownership, request: LaunchRequest) -> Result<Observation> {
        let delay = self.launch_delay_ms.swap(0, Ordering::AcqRel);
        if delay != 0 {
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
        let mut gate = self.gate.lock().await;
        // This check is at the effect, after any scheduling/transport delay.
        self.validate_gate(&gate, ownership)?;
        request.spec.validate()?;
        if request.spec.cleanup_boundary != CleanupBoundary::DirectProcess
            || request.spec.key.insula_id != "root"
        {
            return Err(Error::Unsupported(
                "fixture requires explicit DirectProcess boundary in root".into(),
            ));
        }
        let fingerprint = digest(&request)?;
        if gate.pending_stops.contains_key(&request.operation_id) {
            return Err(Error::Conflict("operation ID was reused".into()));
        }
        if let Some(completed) = gate.completed.get(&request.operation_id).cloned() {
            if completed.digest != fingerprint {
                return Err(Error::Conflict("operation ID was reused".into()));
            }
            if matches!(completed.observation, Observation::Alive { .. }) {
                if let Some(entry) = gate.entries.get_mut(&request.spec.key) {
                    if entry.receipt.generation == request.generation {
                        Self::refresh(entry)?;
                        let observation = Self::observation(entry);
                        self.persist(&gate)?;
                        return Ok(observation);
                    }
                    // A later admitted generation is evidence that this old
                    // generation was already proved gone by the same owner.
                    return Ok(Observation::Gone {
                        generation: Some(request.generation),
                        last_exit: None,
                        boundary_empty: true,
                        launch_resolved: true,
                    });
                }
                return Ok(Observation::Unknown);
            }
            return Ok(completed.observation.clone());
        }
        if gate
            .completed
            .values()
            .any(|completed| match &completed.observation {
                Observation::Alive { identity } => identity.generation == request.generation,
                Observation::Gone { generation, .. } => *generation == Some(request.generation),
                Observation::Unknown => false,
            })
        {
            return Err(Error::Conflict("generation has already been used".into()));
        }
        if let Some(entry) = gate.entries.get_mut(&request.spec.key) {
            Self::refresh(entry)?;
            if entry.receipt.generation == request.generation {
                if entry.receipt.launch_operation != request.operation_id
                    || entry.receipt.launch_digest != fingerprint
                {
                    return Err(Error::Conflict("generation was already reserved".into()));
                }
                return Ok(Self::observation(entry));
            }
            if entry.receipt.phase != Phase::Gone {
                return Err(Error::Conflict(
                    "prior generation is not proven gone".into(),
                ));
            }
            Self::cleanup_endpoint(entry).await?;
        }
        let environment = Self::environment(&gate, &request.spec.command)?;
        let mut command = Self::command(&request.spec.command, &environment);
        let endpoint = if let ServiceAdapter::Tmux { socket } = &request.spec.adapter {
            let parent = socket
                .parent()
                .ok_or_else(|| Error::Invalid("tmux socket has no parent".into()))?;
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(parent)?;
            let parent = parent.canonicalize()?;
            let metadata = fs::symlink_metadata(&parent)?;
            let endpoint = TmuxEndpoint {
                path: parent.join(socket.file_name().ok_or(Error::Unknown)?),
                parent_device: metadata.dev(),
                parent_inode: metadata.ino(),
                uid: request.spec.command.identity.uid,
                stamp: None,
            };
            Self::endpoint_parent(&endpoint)?;
            if gate.entries.iter().any(|(key, entry)| {
                key != &request.spec.key
                    && entry
                        .receipt
                        .endpoint
                        .as_ref()
                        .is_some_and(|existing| existing.path == endpoint.path)
            }) {
                return Err(Error::Conflict(
                    "canonical tmux endpoint is reserved by another service".into(),
                ));
            }
            match fs::symlink_metadata(&endpoint.path) {
                Ok(_) => return Err(Error::Conflict("tmux endpoint is not proven empty".into())),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(Error::Unknown),
            }
            command
                .args(["-D", "-f", "/dev/null", "-S"])
                .arg(&endpoint.path);
            Some(endpoint)
        } else {
            None
        };
        let key = request.spec.key.clone();
        gate.entries.insert(
            key.clone(),
            Entry {
                receipt: Receipt {
                    key: key.clone(),
                    generation: request.generation,
                    launch_operation: request.operation_id,
                    launch_digest: fingerprint.clone(),
                    phase: Phase::Reserved,
                    identity: None,
                    last_exit: None,
                    endpoint,
                },
                child: None,
                pidfd: None,
                spec: Some(request.spec.clone()),
            },
        );
        // Reservation reaches disk before spawn. An owner crash from here until
        // its live receipt commits is deliberately Unknown on owner recovery.
        self.persist(&gate)?;
        let failure = self.fail_launch.swap(0, Ordering::AcqRel);
        if failure == 1 {
            return Err(Error::Unknown);
        }
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                let entry = gate.entries.get_mut(&key).unwrap();
                entry.receipt.phase = Phase::Gone;
                let observation = Self::observation(entry);
                gate.completed.insert(
                    request.operation_id,
                    CompletedOperation {
                        operation_id: request.operation_id,
                        digest: fingerprint,
                        observation,
                    },
                );
                self.persist(&gate)?;
                return Err(Error::Unavailable);
            }
        };
        self.launches.fetch_add(1, Ordering::AcqRel);
        let pid = child.id().ok_or(Error::Unknown)?;
        // Child is unreaped until after the pidfd is opened, eliminating a
        // numeric-PID reuse race. Signals use this FD, never the saved PID.
        let pidfd = match pidfd_open(
            Pid::from_raw(pid as i32).ok_or(Error::Unknown)?,
            PidfdFlags::empty(),
        ) {
            Ok(pidfd) => pidfd,
            Err(_) => {
                child.kill().await?;
                return Err(Error::Unsupported("pidfds unavailable on test host".into()));
            }
        };
        let identity = insula_supervisor::probes::read_process_identity(
            pid,
            self.owner_instance,
            request.generation,
        )?;
        let entry = gate.entries.get_mut(&key).unwrap();
        entry.receipt.identity = Some(identity.clone());
        entry.receipt.phase = Phase::Alive;
        entry.child = Some(child);
        entry.pidfd = Some(pidfd);
        let observation = Observation::Alive { identity };
        gate.completed.insert(
            request.operation_id,
            CompletedOperation {
                operation_id: request.operation_id,
                digest: fingerprint,
                observation: observation.clone(),
            },
        );
        self.persist(&gate)?;
        if failure == 2 {
            return Err(Error::Unknown);
        }
        Ok(observation)
    }

    async fn stop(&self, ownership: &Ownership, request: StopRequest) -> Result<Observation> {
        let this = self.self_ref.upgrade().ok_or(Error::Unavailable)?;
        let ownership = ownership.clone();
        // The owner retains admitted stop work even if an in-process caller or
        // an RPC client abandons its wait. In particular a graceful helper must
        // finish or be explicitly killed/reaped before its gate is released.
        tokio::spawn(async move { this.stop_fenced(&ownership, request).await })
            .await
            .map_err(|_| Error::Unknown)?
    }

    async fn probe(&self, ownership: &Ownership, request: ProbeRequest) -> Result<ProbeResult> {
        let (identity, environment) = {
            let mut gate = self.gate.lock().await;
            self.validate_gate(&gate, ownership)?;
            let entry = gate.entries.get_mut(&request.key).ok_or(Error::NotFound)?;
            if entry.receipt.generation != request.generation {
                return Err(Error::StaleGeneration);
            }
            Self::refresh(entry)?;
            let identity = match Self::observation(entry) {
                Observation::Alive { identity } => identity,
                Observation::Gone { .. } => {
                    return Ok(ProbeResult {
                        generation: request.generation,
                        class: ProbeClass::ServiceFailure,
                        code: "process_exited".into(),
                        expectations_met: None,
                        panes: vec![],
                        observed_ms: now_ms(),
                    })
                }
                Observation::Unknown => {
                    return Ok(ProbeResult {
                        generation: request.generation,
                        class: ProbeClass::TargetUnavailable,
                        code: "identity_unknown".into(),
                        expectations_met: None,
                        panes: vec![],
                        observed_ms: now_ms(),
                    })
                }
            };
            let command = match &request.spec.probe {
                ProbeKind::Command { command } => command.clone(),
                _ => entry.spec.as_ref().ok_or(Error::Unknown)?.command.clone(),
            };
            (identity, Self::environment(&gate, &command)?)
        };
        // The independent probe helper cannot hold the lifecycle gate hostage.
        let result = self.probes.probe(&identity, &request, environment).await;
        let mut gate = self.gate.lock().await;
        self.validate_gate(&gate, ownership)?;
        if result.class == ProbeClass::Success
            && matches!(request.spec.probe, ProbeKind::Tmux { .. })
        {
            let entry = gate.entries.get_mut(&request.key).ok_or(Error::NotFound)?;
            if entry.receipt.generation != request.generation {
                return Err(Error::StaleGeneration);
            }
            Self::capture_endpoint(entry).await?;
            self.persist(&gate)?;
        }
        Ok(result)
    }

    async fn release(&self, ownership: &Ownership) -> Result<()> {
        let mut gate = self.gate.lock().await;
        if gate.ownership.as_ref() == Some(ownership) {
            gate.ownership = None;
        }
        // Releasing an already-fenced epoch is harmless, and never releases a
        // newer controller. The same gate serializes release with actual effects.
        Ok(())
    }
}

impl FixtureOwner {
    async fn stop_fenced(
        &self,
        ownership: &Ownership,
        request: StopRequest,
    ) -> Result<Observation> {
        let fingerprint = digest(&request)?;
        let deadline =
            tokio::time::Instant::now() + Duration::from_millis(request.policy.deadline_ms);
        {
            let mut gate = self.gate.lock().await;
            self.validate_gate(&gate, ownership)?;
            if let Some(completed) = gate.completed.get(&request.operation_id) {
                if completed.digest != fingerprint {
                    return Err(Error::Conflict("operation ID was reused".into()));
                }
                return Ok(completed.observation.clone());
            }
            let resume = match gate.pending_stops.get(&request.operation_id) {
                Some(pending) if pending.digest == fingerprint => true,
                Some(_) => return Err(Error::Conflict("operation ID was reused".into())),
                None => false,
            };
            let entry = gate.entries.get_mut(&request.key).ok_or(Error::NotFound)?;
            if entry.receipt.generation != request.expected_generation {
                return Err(Error::StaleGeneration);
            }
            if entry
                .spec
                .as_ref()
                .is_some_and(|spec| spec.stop != request.policy)
            {
                return Err(Error::Conflict(
                    "stop policy differs from the declaration".into(),
                ));
            }
            Self::refresh(entry)?;
            let initial = Self::observation(entry);
            if matches!(initial, Observation::Alive { .. }) && !resume {
                Self::capture_endpoint(entry).await?;
            }
            if !resume {
                gate.pending_stops.insert(
                    request.operation_id,
                    PendingStop {
                        operation_id: request.operation_id,
                        digest: fingerprint.clone(),
                    },
                );
                self.persist(&gate)?;
            }
            let entry = gate.entries.get_mut(&request.key).unwrap();
            match initial {
                Observation::Unknown => return Err(Error::Unknown),
                Observation::Gone { .. } => {}
                Observation::Alive { .. } if !resume => {
                    let service_command =
                        entry.spec.as_ref().ok_or(Error::Unknown)?.command.clone();
                    match &request.policy.graceful {
                        GracefulStop::Terminate => Self::signal(entry, Signal::TERM)?,
                        GracefulStop::Command { command } => {
                            let environment = Self::environment(&gate, command)?;
                            let mut helper = Self::command(command, &environment).spawn()?;
                            // Keep the gate until this external effect is
                            // finished or canceled and reaped. EOF cannot admit
                            // a new epoch while a graceful helper can still act.
                            if tokio::time::timeout_at(deadline, helper.wait())
                                .await
                                .is_err()
                            {
                                helper.kill().await?;
                            }
                        }
                        GracefulStop::Tmux { executable, socket } => {
                            let environment = Self::environment(&gate, &service_command)?;
                            let mut helper_spec = service_command;
                            helper_spec.executable = executable.clone();
                            helper_spec.args = vec![
                                "-N".into(),
                                "-f".into(),
                                "/dev/null".into(),
                                "-S".into(),
                                socket.to_string_lossy().into_owned(),
                                "kill-server".into(),
                            ];
                            let mut helper = Self::command(&helper_spec, &environment).spawn()?;
                            if tokio::time::timeout_at(deadline, helper.wait())
                                .await
                                .is_err()
                            {
                                helper.kill().await?;
                            }
                        }
                    }
                }
                Observation::Alive { .. } => {}
            }
        }
        loop {
            {
                let mut gate = self.gate.lock().await;
                self.validate_gate(&gate, ownership)?;
                let entry = gate.entries.get_mut(&request.key).ok_or(Error::NotFound)?;
                if entry.receipt.generation != request.expected_generation {
                    return Err(Error::StaleGeneration);
                }
                Self::refresh(entry)?;
                if entry.receipt.phase == Phase::Gone {
                    Self::cleanup_endpoint(entry).await?;
                    let observation = Self::observation(entry);
                    gate.completed.insert(
                        request.operation_id,
                        CompletedOperation {
                            operation_id: request.operation_id,
                            digest: fingerprint,
                            observation: observation.clone(),
                        },
                    );
                    gate.pending_stops.remove(&request.operation_id);
                    self.persist(&gate)?;
                    return Ok(observation);
                }
                if tokio::time::Instant::now() >= deadline {
                    if !request.policy.force {
                        return Err(Error::Unknown);
                    }
                    // Revalidate ownership and generation at the forced effect,
                    // not just when the grace deadline was scheduled.
                    Self::signal(entry, Signal::KILL)?;
                    if let Some(child) = entry.child.as_mut() {
                        tokio::time::timeout(Duration::from_secs(2), child.wait())
                            .await
                            .map_err(|_| Error::Unknown)??;
                    }
                    Self::refresh(entry)?;
                    Self::cleanup_endpoint(entry).await?;
                    let observation = Self::observation(entry);
                    gate.completed.insert(
                        request.operation_id,
                        CompletedOperation {
                            operation_id: request.operation_id,
                            digest: fingerprint,
                            observation: observation.clone(),
                        },
                    );
                    gate.pending_stops.remove(&request.operation_id);
                    self.persist(&gate)?;
                    return Ok(observation);
                }
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }
}

pub struct FixtureRpcServer {
    socket: PathBuf,
    shutdown: Option<oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

impl FixtureRpcServer {
    pub fn start(owner: Arc<FixtureOwner>, socket: &Path) -> Result<Self> {
        let listener = UnixListener::bind(socket)?;
        fs::set_permissions(socket, fs::Permissions::from_mode(0o600))?;
        let (shutdown, cancel) = oneshot::channel();
        let task = tokio::spawn(async move {
            tokio::pin!(cancel);
            let mut connections = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut cancel => break,
                    Ok((stream, _)) = listener.accept() => {
                        let owner = Arc::clone(&owner);
                        connections.spawn(async move { let _ = serve_connection(owner, stream).await; });
                    }
                    Some(_) = connections.join_next(), if !connections.is_empty() => {}
                }
            }
            owner.fence_controller().await;
            connections.abort_all();
            while connections.join_next().await.is_some() {}
        });
        Ok(Self {
            socket: socket.into(),
            shutdown: Some(shutdown),
            task,
        })
    }
    pub fn socket(&self) -> &Path {
        &self.socket
    }
}

impl Drop for FixtureRpcServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        // The accept task drains and fences on the next scheduler turn.
        // We intentionally do not abort it before that ownership transition.
        let _ = fs::remove_file(&self.socket);
    }
}

async fn serve_connection(owner: Arc<FixtureOwner>, mut stream: UnixStream) -> Result<()> {
    if stream.peer_cred()?.uid() != rustix::process::getuid().as_raw() {
        return Err(Error::Ownership);
    }
    let request: RpcRequest = tokio::time::timeout(Duration::from_secs(2), read_frame(&mut stream))
        .await
        .map_err(|_| Error::Unavailable)??;
    if request.version != PROTOCOL_VERSION {
        return Err(Error::Invalid("unsupported owner protocol".into()));
    }
    if let OwnerRequest::Claim {
        root_insula_id,
        controller_id,
    } = &request.request
    {
        let result = owner.claim(root_insula_id, *controller_id).await;
        let claim = result.as_ref().ok().cloned();
        let response = RpcResponse::for_request(&request, result.map(OwnerReply::Claimed));
        if write_frame(&mut stream, &response).await.is_ok() && claim.is_some() {
            let mut byte = [0];
            let _ = stream.read(&mut byte).await;
        }
        if let Some(claim) = claim {
            owner.release(&claim).await?;
        }
        return Ok(());
    }
    let result = match &request.request {
        OwnerRequest::Capabilities { root_insula_id } => owner
            .capabilities(root_insula_id)
            .await
            .map(OwnerReply::Capabilities),
        OwnerRequest::Validate { ownership } => owner
            .validate(ownership)
            .await
            .map(|_| OwnerReply::Acknowledged),
        OwnerRequest::Observe {
            ownership,
            key,
            generation,
        } => owner
            .observe(ownership, key, *generation)
            .await
            .map(OwnerReply::Observation),
        OwnerRequest::Launch { ownership, request } => owner
            .launch(ownership, *request.clone())
            .await
            .map(OwnerReply::Observation),
        OwnerRequest::Stop { ownership, request } => owner
            .stop(ownership, request.clone())
            .await
            .map(OwnerReply::Observation),
        OwnerRequest::Probe { ownership, request } => owner
            .probe(ownership, request.clone())
            .await
            .map(OwnerReply::Probe),
        OwnerRequest::Release { ownership } => owner
            .release(ownership)
            .await
            .map(|_| OwnerReply::Acknowledged),
        OwnerRequest::Claim { .. } => unreachable!(),
    };
    write_frame(&mut stream, &RpcResponse::for_request(&request, result)).await
}

pub fn foreground_spec(executable: &str, args: &[&str]) -> ServiceSpec {
    ServiceSpec {
        key: ServiceKey {
            insula_id: "root".into(),
            service_id: "example".into(),
        },
        revision: "test-v1".into(),
        command: CommandSpec {
            executable: executable.into(),
            args: args.iter().map(|arg| (*arg).into()).collect(),
            cwd: "/tmp".into(),
            env_refs: BTreeMap::new(),
            identity: RuntimeIdentity {
                uid: rustix::process::getuid().as_raw(),
                gid: rustix::process::getgid().as_raw(),
            },
        },
        adapter: ServiceAdapter::Foreground,
        readiness: ProbeSpec::default(),
        health: ProbeSpec::default(),
        startup_grace_ms: 0,
        restart: RestartPolicy::Never,
        replay_safe: false,
        stop: StopPolicy {
            graceful: GracefulStop::Terminate,
            deadline_ms: 100,
            force: true,
        },
        logs: LogPolicy {
            max_bytes: 0,
            retained_files: 0,
        },
        cleanup_boundary: CleanupBoundary::DirectProcess,
        controller_exit: ControllerExit::KeepServices,
    }
}

#[cfg(test)]
mod endpoint_cleanup_tests {
    use super::*;

    #[tokio::test]
    async fn cleanup_requires_a_recorded_stale_socket_and_preserves_foreign_paths() {
        let temp = tempfile::TempDir::new().unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let metadata = fs::symlink_metadata(temp.path()).unwrap();
        let path = temp.path().join("endpoint.sock");
        let listener = UnixListener::bind(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let mut endpoint = TmuxEndpoint {
            path: path.clone(),
            parent_device: metadata.dev(),
            parent_inode: metadata.ino(),
            uid: rustix::process::getuid().as_raw(),
            stamp: None,
        };
        endpoint.stamp = FixtureOwner::socket_stamp(&endpoint).unwrap();
        let mut entry = Entry {
            receipt: Receipt {
                key: ServiceKey {
                    insula_id: "root".into(),
                    service_id: "endpoint-check".into(),
                },
                generation: Uuid::new_v4(),
                launch_operation: Uuid::new_v4(),
                launch_digest: "test".into(),
                phase: Phase::Gone,
                identity: None,
                last_exit: None,
                endpoint: Some(endpoint),
            },
            child: None,
            pidfd: None,
            spec: None,
        };
        // A matching inode that still accepts connections is never unlinked.
        assert_eq!(
            FixtureOwner::cleanup_endpoint(&mut entry)
                .await
                .unwrap_err(),
            Error::Unknown
        );
        assert!(path.exists());
        drop(listener);
        // The recorded inode with no listener is the one cleanup may remove.
        FixtureOwner::cleanup_endpoint(&mut entry).await.unwrap();
        assert!(!path.exists());
        // A new stale socket at the same pathname is not covered by that receipt.
        let listener = UnixListener::bind(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        drop(listener);
        assert_eq!(
            FixtureOwner::cleanup_endpoint(&mut entry)
                .await
                .unwrap_err(),
            Error::Unknown
        );
        assert!(path.exists());
        // Nor is a replacement regular file, even when owned by the same UID.
        fs::remove_file(&path).unwrap();
        fs::write(&path, "preserve this unrelated file").unwrap();
        assert_eq!(
            FixtureOwner::cleanup_endpoint(&mut entry)
                .await
                .unwrap_err(),
            Error::Unknown
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "preserve this unrelated file"
        );
    }
}
