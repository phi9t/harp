//! Closed-world model-free native fixture. Every executed RPC publishes new
//! independent invocation/effect IDs; no provider or deduplication is hidden here.
use crate::{ErrorCode, episode_runtime::*, gateway::*, supervisor::Supervisor};
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;
pub struct NativeOsRuntime {
    pub root: PathBuf,
    pub identity: RuntimeIdentity,
    pub supervisor: Supervisor,
    pub service: String,
    pub lose_reply: bool,
    pub hold_reply: bool,
}
fn durable(root: &Path, path: &Path, value: &impl serde::Serialize) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(&serde_json::to_vec(value)?)?;
    file.sync_all()?;
    std::fs::File::open(root)?.sync_all()
}
impl EpisodeRuntime for NativeOsRuntime {
    fn closed_world_os(&self) -> Option<ClosedWorldOs> {
        Some(ClosedWorldOs {
            workspace_root: self.supervisor.workspace_root().to_path_buf(),
            native_root: self.root.clone(),
        })
    }

    fn identity(&self) -> RuntimeIdentity {
        self.identity.clone()
    }
    fn capabilities(&self) -> CapabilityProfile {
        CapabilityProfile {
            create: true,
            start: true,
            observe: true,
            cancel: true,
            authoritative_effect_inventory: true,
            native_deduplication: false,
            ..Default::default()
        }
    }
    fn start<'a>(
        &'a mut self,
        request: &'a PreparedStart,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EpisodeReply> + Send + 'a>> {
        Box::pin(async move {
            let invocation = Uuid::new_v4();
            if durable(&self.root,&self.root.join(format!("{invocation}.invocation")),&serde_json::json!({"id":invocation,"operation":request.start.id,"episode":request.start.episode,"execution":request.start.execution,"input_packet":request.input_packet,"input_digest":request.input_digest,"input":request.input,"argv":request.start.argv})).is_err() {return EpisodeReply::Unknown("native invocation publication incomplete".into());}
            match self
                .supervisor
                .start(
                    &self.service,
                    request.start.execution,
                    request.start.argv.clone(),
                    30,
                )
                .await
            {
                Ok(()) => {}
                Err(error)
                    if matches!(
                        error.code,
                        ErrorCode::InvalidArgument
                            | ErrorCode::Unauthorized
                            | ErrorCode::StaleGeneration
                            | ErrorCode::StaleRevision
                            | ErrorCode::DependencyUnmet
                    ) =>
                {
                    return EpisodeReply::FailedKnown(
                        "current public supervisor rejected before launch".into(),
                    );
                }
                Err(_) => {
                    return EpisodeReply::Unknown("supervisor launch completion uncertain".into());
                }
            }
            let turn = NativeTurn {
                thread: format!("thread-{}", Uuid::new_v4()),
                turn: format!("turn-{}", Uuid::new_v4()),
                effect: format!("effect-{}", Uuid::new_v4()),
            };
            let record = serde_json::json!({"operation":request.start.id,"execution":request.start.execution,"episode":request.start.episode,"runtime":self.identity,"input_packet":request.input_packet,"input_digest":request.input_digest,"argv":request.start.argv,"turn":turn,"usage":{"profile":"inprocess-model-free-v1","model_requests":0,"cost_microusd":0}});
            if durable(
                &self.root,
                &self.root.join(format!("{}.effect", Uuid::new_v4())),
                &record,
            )
            .is_err()
            {
                return EpisodeReply::Unknown("native effect publication incomplete".into());
            }
            if self.hold_reply {
                std::future::pending::<()>().await;
            }
            if self.lose_reply {
                EpisodeReply::Unknown(
                    "actual native Start effect persisted; reply deliberately lost".into(),
                )
            } else {
                EpisodeReply::ConfirmedMeasured(
                    turn,
                    receipt_usage(
                        &record,
                        &serde_json::to_vec(&record).expect("native JSON record serializes"),
                    ),
                )
            }
        })
    }
    fn observe<'a>(
        &'a mut self,
        request: &'a PreparedStart,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EpisodeObservation> + Send + 'a>> {
        Box::pin(async move {
            let Ok(directory) = open_directory(&self.root) else {
                return EpisodeObservation::Inconclusive("native inventory root unproven".into());
            };
            observe_directory(&directory, &self.identity, request)
        })
    }
}

pub(crate) fn observe_directory(
    directory: &std::fs::File,
    identity: &RuntimeIdentity,
    request: &PreparedStart,
) -> EpisodeObservation {
    let mut matches = vec![];
    let Ok(names) = entries(directory) else {
        return EpisodeObservation::Inconclusive("native inventory incomplete or unbounded".into());
    };
    for name in names {
        let path = Path::new(&name);
        if path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .is_none()
        {
            return EpisodeObservation::Inconclusive("native inventory filename malformed".into());
        }
        let Some(extension) = path.extension().and_then(|s| s.to_str()) else {
            return EpisodeObservation::Inconclusive("native inventory filename malformed".into());
        };
        if !matches!(extension, "effect" | "invocation") {
            return EpisodeObservation::Inconclusive("foreign native inventory entry".into());
        }
        let Ok(bytes) = native_file(directory, &name) else {
            return EpisodeObservation::Inconclusive("native record unsafe or unreadable".into());
        };
        if extension == "invocation" {
            continue;
        }
        let Ok(record) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            return EpisodeObservation::Inconclusive("native effect malformed".into());
        };
        if record["operation"] == request.start.id.to_string()
            && record["execution"] == request.start.execution.to_string()
            && record["episode"] == request.start.episode.to_string()
        {
            if record["runtime"] != serde_json::json!(identity)
                || record["input_packet"] != request.input_packet.to_string()
                || record["input_digest"] != request.input_digest
                || record["argv"] != serde_json::json!(request.start.argv)
            {
                return EpisodeObservation::Inconclusive(
                    "native effect binding inconsistent".into(),
                );
            }
            let Ok(turn) = serde_json::from_value(record["turn"].clone()) else {
                return EpisodeObservation::Inconclusive("native binding malformed".into());
            };
            matches.push((turn, receipt_usage(&record, &bytes)));
        }
    }
    if matches.len() == 1 {
        let (turn, usage) = matches.pop().unwrap();
        EpisodeObservation::EffectMeasured(turn, usage)
    } else {
        EpisodeObservation::Inconclusive(
            "native effect missing or ambiguous; no absence inferred".into(),
        )
    }
}

fn receipt_usage(record: &serde_json::Value, bytes: &[u8]) -> UsageObservation {
    receipt_usage_digest(record, crate::artifacts::sha256(bytes))
}

fn receipt_usage_digest(record: &serde_json::Value, digest: String) -> UsageObservation {
    if record["usage"]
        == serde_json::json!({"profile":"inprocess-model-free-v1","model_requests":0,"cost_microusd":0})
    {
        UsageObservation::MeasuredZero {
            receipt_digest: digest,
        }
    } else {
        UsageObservation::Unknown
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub(crate) struct DirectoryIdentity {
    pub device: u64,
    pub inode: u64,
    pub owner: u32,
}
pub(crate) fn open_directory(path: &Path) -> crate::Result<std::fs::File> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::{MetadataExt, OpenOptionsExt},
        },
    };
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    if !path.is_absolute() {
        return Err(rejected());
    }
    let mut directory = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open("/")
        .map_err(|_| rejected())?;
    for component in path.components() {
        let std::path::Component::Normal(name) = component else {
            if matches!(component, std::path::Component::RootDir) {
                continue;
            }
            return Err(rejected());
        };
        use std::os::unix::ffi::OsStrExt;
        let name = CString::new(name.as_bytes()).map_err(|_| rejected())?;
        let raw = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(rejected());
        }
        directory = unsafe { std::fs::File::from_raw_fd(raw) };
    }
    let meta = directory.metadata().map_err(|_| rejected())?;
    if meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o022 != 0 {
        return Err(rejected());
    }
    Ok(directory)
}
pub(crate) fn directory_identity(path: &Path) -> crate::Result<DirectoryIdentity> {
    use std::os::unix::fs::MetadataExt;
    let metadata = open_directory(path)?
        .metadata()
        .map_err(|_| crate::domain::err(ErrorCode::DependencyUnmet))?;
    Ok(DirectoryIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner: metadata.uid(),
    })
}

#[derive(Clone, Debug)]
pub(crate) struct HostProcess {
    pub pid: u32,
    pub start_ticks: u64,
    pub group: i32,
}
fn host_process(pid: u32) -> std::io::Result<Option<HostProcess>> {
    let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => stat,
        Err(e)
            if e.kind() == std::io::ErrorKind::NotFound
                || e.raw_os_error() == Some(libc::ESRCH) =>
        {
            return Ok(None);
        }
        Err(e) => return Err(e),
    };
    let invalid = || std::io::Error::other("host process observation malformed");
    let close = stat.rfind(')').ok_or_else(invalid)?;
    let fields: Vec<_> = stat[close + 1..].split_whitespace().collect();
    let observed_pid = stat
        .split_whitespace()
        .next()
        .ok_or_else(invalid)?
        .parse::<u32>()
        .map_err(|_| invalid())?;
    if observed_pid != pid {
        return Err(invalid());
    }
    let group = fields
        .get(2)
        .ok_or_else(invalid)?
        .parse::<i32>()
        .map_err(|_| invalid())?;
    let start_ticks = fields
        .get(19)
        .ok_or_else(invalid)?
        .parse::<u64>()
        .map_err(|_| invalid())?;
    if group < 0 || start_ticks == 0 {
        return Err(invalid());
    }
    Ok(Some(HostProcess {
        pid,
        start_ticks,
        group,
    }))
}
pub(crate) fn group_members(group: i32) -> std::io::Result<Vec<HostProcess>> {
    let mut members = vec![];
    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            return Err(std::io::Error::other("proc entry malformed"));
        };
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if let Some(process) = host_process(pid)?
            && process.group == group
        {
            members.push(process);
        }
    }
    Ok(members)
}
pub(crate) fn exclusive_live_transport(
    process: &crate::supervisor::ProcessIdentity,
    group: i32,
) -> std::io::Result<bool> {
    crate::supervisor::validate_process_frame(process)?;
    if !crate::supervisor::transport_alive(process)? || group as u32 != process.pid {
        return Ok(false);
    }
    let members = group_members(group)?;
    Ok(members.len() == 1
        && members[0].pid == process.pid
        && members[0].start_ticks == process.start_ticks)
}
pub(crate) fn transport_finalized(
    process: &crate::supervisor::ProcessIdentity,
    group: i32,
) -> std::io::Result<bool> {
    crate::supervisor::validate_process_frame(process)?;
    // This deliberately requires the original service process to be reaped or
    // replaced, rather than treating leader-Z or missing namespace links as exit.
    if host_process(process.pid)?.is_some_and(|p| p.start_ticks == process.start_ticks) {
        return Ok(false);
    }
    Ok(group_members(group)?.is_empty())
}

#[derive(Clone, Debug)]
pub(crate) struct Transport {
    pub store: String,
    pub identity: RuntimeIdentity,
    pub process: crate::supervisor::ProcessIdentity,
    pub group: i32,
    pub workspace_root: PathBuf,
    pub native_root: PathBuf,
    pub native_directory: DirectoryIdentity,
    pub workspace_directory: DirectoryIdentity,
    pub epoch: Uuid,
}
pub(crate) async fn transport(
    tx: &tokio_postgres::Transaction<'_>,
    controller: Uuid,
) -> crate::Result<Transport> {
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    let row=tx.query_opt("SELECT store,runtime_identity,process_identity,group_id,workspace_root,native_root,native_directory,workspace_directory,recovery_epoch,contract FROM kernel_transports WHERE controller=$1", &[&controller]).await?.ok_or_else(rejected)?;
    if row.get::<_, String>(9) != "inprocess-supervisor-os-v1" {
        return Err(rejected());
    }
    Ok(Transport {
        store: row.get(0),
        identity: serde_json::from_value(row.get(1)).map_err(|_| rejected())?,
        process: serde_json::from_value(row.get(2)).map_err(|_| rejected())?,
        group: row.get(3),
        workspace_root: PathBuf::from(row.get::<_, String>(4)),
        native_root: PathBuf::from(row.get::<_, String>(5)),
        native_directory: serde_json::from_value(row.get(6)).map_err(|_| rejected())?,
        workspace_directory: serde_json::from_value(row.get(7)).map_err(|_| rejected())?,
        epoch: row.get(8),
    })
}
pub(crate) struct NativeInventory {
    pub digest: String,
    pub effects: Vec<(PreparedStart, NativeTurn, UsageObservation)>,
    // Owned directory handles pin the exact registered roots through transfer.
    pub native: std::fs::File,
    pub workspaces: std::fs::File,
}
pub(crate) fn entries(directory: &std::fs::File) -> crate::Result<Vec<std::ffi::OsString>> {
    use std::os::fd::AsRawFd;
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    let entries: Vec<_> = std::fs::read_dir(format!("/proc/self/fd/{}", directory.as_raw_fd()))
        .map_err(|_| rejected())?
        .take(4097)
        .map(|entry| entry.map(|e| e.file_name()).map_err(|_| rejected()))
        .collect::<crate::Result<_>>()?;
    if entries.len() > 4096 {
        return Err(rejected());
    }
    Ok(entries)
}
pub(crate) fn secure_file(
    directory: &std::fs::File,
    name: &std::ffi::OsStr,
) -> crate::Result<std::fs::File> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{ffi::OsStrExt, fs::MetadataExt},
        },
    };
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    if name.as_bytes().contains(&b'/') || name.as_bytes().is_empty() || name == "." || name == ".."
    {
        return Err(rejected());
    }
    let name = CString::new(name.as_bytes()).map_err(|_| rejected())?;
    let raw = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err(rejected());
    }
    let file = unsafe { std::fs::File::from_raw_fd(raw) };
    let metadata = file.metadata().map_err(|_| rejected())?;
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.nlink() != 1
        || metadata.mode() & 0o022 != 0
    {
        return Err(rejected());
    }
    Ok(file)
}
pub(crate) fn native_file(
    directory: &std::fs::File,
    name: &std::ffi::OsStr,
) -> crate::Result<Vec<u8>> {
    use std::io::Read;
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    let file = secure_file(directory, name)?;
    if file.metadata().map_err(|_| rejected())?.len() > 1_048_576 {
        return Err(rejected());
    }
    let mut bytes = vec![];
    file.take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|_| rejected())?;
    if bytes.len() > 1_048_576 {
        return Err(rejected());
    }
    Ok(bytes)
}
/// Complete inventory exists only for the pinned, declared in-process adapter.
/// Every file is classified, every invocation is bound, and every physical
/// workspace under the declared root must have a durable stopped/released row.
pub(crate) async fn inventory(
    tx: &tokio_postgres::Transaction<'_>,
    transport: &Transport,
) -> crate::Result<NativeInventory> {
    use std::collections::BTreeMap;
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    if directory_identity(&transport.native_root)? != transport.native_directory
        || directory_identity(&transport.workspace_root)? != transport.workspace_directory
    {
        return Err(rejected());
    }
    let native = open_directory(&transport.native_root)?;
    let workspaces = open_directory(&transport.workspace_root)?;
    let mut invocations: BTreeMap<Uuid, serde_json::Value> = BTreeMap::new();
    let mut effects: BTreeMap<Uuid, serde_json::Value> = BTreeMap::new();
    let mut effect_digests = BTreeMap::new();
    let mut hashes = BTreeMap::new();
    for name in entries(&native)? {
        let name_string = name.to_str().ok_or_else(rejected)?;
        let path = Path::new(&name);
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(rejected)?;
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(rejected)?;
        Uuid::parse_str(filename).map_err(|_| rejected())?;
        if !matches!(extension, "invocation" | "effect") {
            return Err(rejected());
        }
        let bytes = native_file(&native, &name)?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| rejected())?;
        let operation = value["operation"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(rejected)?;
        if extension == "invocation" {
            if value["id"] != filename || invocations.insert(operation, value).is_some() {
                return Err(rejected());
            }
        } else {
            if effects.insert(operation, value).is_some() {
                return Err(rejected());
            }
            effect_digests.insert(operation, crate::artifacts::sha256(&bytes));
        }
        hashes.insert(name_string.to_owned(), crate::artifacts::sha256(&bytes));
    }
    let journals=tx.query("SELECT id FROM episode_runtime_operations WHERE store=$1 AND state IN ('confirmed','dispatching','outcome_unknown','quarantined') ORDER BY id LIMIT 4097", &[&transport.store]).await?;
    if journals.len() > 4096
        || journals
            .iter()
            .any(|r| !invocations.contains_key(&r.get::<_, Uuid>(0)))
    {
        return Err(rejected());
    }
    let legacy_unknown:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_operations o JOIN gateway_bindings b ON b.id=o.binding_id WHERE b.store=$1 AND o.state IN ('dispatching','outcome_unknown','quarantined'))", &[&transport.store]).await?.get(0);
    if legacy_unknown {
        return Err(rejected());
    }
    let mut observed = vec![];
    for (operation, invocation) in &invocations {
        let row=tx.query_opt("SELECT request,state,store,thread_id,turn_id,effect_id FROM episode_runtime_operations WHERE id=$1", &[operation]).await?.ok_or_else(rejected)?;
        let prepared: PreparedStart = serde_json::from_value(row.get(0)).map_err(|_| rejected())?;
        if prepared.start.runtime != transport.identity
            || row.get::<_, Option<String>>(2).as_deref() != Some(&transport.store)
            || invocation["episode"] != prepared.start.episode.to_string()
            || invocation["execution"] != prepared.start.execution.to_string()
            || invocation["input_packet"] != prepared.input_packet.to_string()
            || invocation["input_digest"] != prepared.input_digest
            || invocation["input"] != prepared.input
            || invocation["argv"] != serde_json::json!(prepared.start.argv)
        {
            return Err(rejected());
        }
        let state: String = row.get(1);
        if state == "failed_known" && !effects.contains_key(operation) {
            continue;
        }
        if !matches!(
            state.as_str(),
            "confirmed" | "dispatching" | "outcome_unknown"
        ) {
            return Err(rejected());
        }
        let effect = effects.remove(operation).ok_or_else(rejected)?;
        if effect["episode"] != prepared.start.episode.to_string()
            || effect["execution"] != prepared.start.execution.to_string()
            || effect["runtime"] != serde_json::json!(transport.identity)
            || effect["input_packet"] != prepared.input_packet.to_string()
            || effect["input_digest"] != prepared.input_digest
            || effect["argv"] != serde_json::json!(prepared.start.argv)
        {
            return Err(rejected());
        }
        let turn: NativeTurn =
            serde_json::from_value(effect["turn"].clone()).map_err(|_| rejected())?;
        for (value, prefix) in [
            (&turn.thread, "thread-"),
            (&turn.turn, "turn-"),
            (&turn.effect, "effect-"),
        ] {
            Uuid::parse_str(value.strip_prefix(prefix).ok_or_else(rejected)?)
                .map_err(|_| rejected())?;
        }
        if state == "confirmed"
            && (row.get::<_, Option<String>>(3).as_deref() != Some(&turn.thread)
                || row.get::<_, Option<String>>(4).as_deref() != Some(&turn.turn)
                || row.get::<_, Option<String>>(5).as_deref() != Some(&turn.effect))
        {
            return Err(rejected());
        }
        let usage = receipt_usage_digest(
            &effect,
            effect_digests
                .remove(&prepared.start.id)
                .ok_or_else(rejected)?,
        );
        observed.push((prepared, turn, usage));
    }
    if !effects.is_empty() {
        return Err(rejected());
    }
    let prefix = format!(
        "{}/",
        transport.workspace_root.to_str().ok_or_else(rejected)?
    );
    let rows=tx.query("SELECT id,manifest,state FROM supervised_workspaces WHERE left(manifest->>'path',length($1))=$1 ORDER BY id LIMIT 4097", &[&prefix]).await?;
    if rows.len() > 4096 {
        return Err(rejected());
    }
    let mut paths = BTreeMap::new();
    for row in rows {
        let id: Uuid = row.get(0);
        let manifest: crate::supervisor::WorkspaceManifest =
            serde_json::from_value(row.get(1)).map_err(|_| rejected())?;
        if manifest.path.parent() == Some(&transport.workspace_root) {
            let state: String = row.get(2);
            if !matches!(state.as_str(), "stopped" | "released") {
                return Err(rejected());
            }
            if manifest.path.file_name().and_then(|s| s.to_str()) != Some(id.to_string().as_str()) {
                return Err(rejected());
            }
            paths.insert(id.to_string(), state);
        }
    }
    for name in entries(&workspaces)? {
        let name = name.to_str().ok_or_else(rejected)?;
        if name == ".execution-logs" {
            let logs = open_directory(&transport.workspace_root.join(name))?;
            for log in entries(&logs)? {
                let path = Path::new(&log);
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(rejected)?;
                if !paths.contains_key(stem)
                    || !matches!(
                        path.extension().and_then(|s| s.to_str()),
                        Some("log" | "info")
                    )
                {
                    return Err(rejected());
                }
                // Raw retained output is securely classified without materializing
                // contents or borrowing the native JSON receipt size bound.
                if path.extension().is_some_and(|s| s == "log") {
                    secure_file(&logs, &log)?;
                } else {
                    native_file(&logs, &log)?;
                }
            }
            continue;
        }
        let state = paths.get(name).ok_or_else(rejected)?;
        if state == "released" {
            return Err(rejected());
        }
        open_directory(&transport.workspace_root.join(name))?;
    }
    for (name, state) in &paths {
        if state == "stopped" && !transport.workspace_root.join(name).exists() {
            return Err(rejected());
        }
    }
    let digest = crate::validation::digest(
        &serde_json::json!({"files":hashes,"workspaces":paths,"native_directory":transport.native_directory,"workspace_directory":transport.workspace_directory}),
    )?;
    Ok(NativeInventory {
        digest,
        effects: observed,
        native,
        workspaces,
    })
}

/// Capture an actual local service incarnation before effects. Registration
/// proves this process/root shape; it never adopts a retained store controller.
pub(crate) async fn register_transport(
    tx: &tokio_postgres::Transaction<'_>,
    controller: Uuid,
    epoch: Uuid,
    identity: RuntimeIdentity,
    roots: ClosedWorldOs,
    contract: &str,
) -> crate::Result<()> {
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    let expected_kind = match contract {
        "inprocess-supervisor-os-v1" => "kernel16-inprocess-native",
        "legacy-supervisor-os-v1" => "kernel16-owned-os",
        _ => return Err(rejected()),
    };
    if identity.runtime_kind != expected_kind
        || identity.runtime_version != "1"
        || identity.deployment != epoch.to_string()
    {
        return Err(rejected());
    }
    let process = crate::supervisor::local_process_identity().map_err(|_| rejected())?;
    let group = unsafe { libc::getpgrp() };
    if group <= 0 || !exclusive_live_transport(&process, group).map_err(|_| rejected())? {
        return Err(rejected());
    }
    let store = identity.key()?;
    let native_root = roots.native_root.to_str().ok_or_else(rejected)?;
    let workspace_root = roots.workspace_root.to_str().ok_or_else(rejected)?;
    if native_root != identity.thread_store {
        return Err(rejected());
    }
    let native = directory_identity(&roots.native_root)?;
    let workspaces = directory_identity(&roots.workspace_root)?;
    tx.execute("INSERT INTO gateway_stores(identity,controller,runtime_identity) VALUES($1,$2,$3) ON CONFLICT DO NOTHING", &[&store,&controller,&serde_json::json!(identity)]).await?;
    if tx
        .query_one(
            "SELECT runtime_identity FROM gateway_stores WHERE identity=$1",
            &[&store],
        )
        .await?
        .get::<_, serde_json::Value>(0)
        != serde_json::json!(identity)
    {
        return Err(rejected());
    }
    tx.execute("INSERT INTO kernel_transports(controller,store,runtime_identity,process_identity,group_id,workspace_root,contract,recovery_epoch,native_root,native_directory,workspace_directory) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)", &[&controller,&store,&serde_json::json!(identity),&serde_json::json!(process),&group,&workspace_root,&contract,&epoch,&native_root,&serde_json::json!(native),&serde_json::json!(workspaces)]).await?;
    Ok(())
}
