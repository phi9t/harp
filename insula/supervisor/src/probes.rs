//! Bounded Linux probe execution for an `ExecutionOwner` adapter.
//!
//! The supervisor never calls this module to enter or launch a service. The
//! authoritative owner must authenticate/fence the request and arrange the
//! target's namespaces and runtime identity before calling this local helper.
//! A probe child is owned and reaped here; service children are not reaped here.
//! This helper contains direct probe children, not arbitrary descendants. Owners
//! allowing commands that fork must supply their own verified containment.

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    net::{IpAddr, SocketAddr},
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::Path,
    process::{ExitStatus, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use rustix::{
    event::{poll, PollFd, PollFlags, Timespec},
    fd::OwnedFd,
    process::{pidfd_open, pidfd_send_signal, Pid, PidfdFlags, Signal},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, UnixStream},
    process::{Child, Command},
    sync::{oneshot, Semaphore},
    time::{sleep_until, timeout_at, Instant},
};
use uuid::Uuid;

use crate::{model::*, Error, Result};

const MAX_PROBES: usize = 64;
const MAX_OUTPUT: usize = 65_536;

/// Reuse one instance per owner; clones share its concurrency limit.
#[derive(Clone)]
pub struct LocalProbes {
    slots: Arc<Semaphore>,
}

impl LocalProbes {
    pub fn new(max_concurrent: usize) -> Result<Self> {
        if !(1..=MAX_PROBES).contains(&max_concurrent) {
            return Err(Error::Invalid("probe concurrency must be in 1..=64".into()));
        }
        Ok(Self {
            slots: Arc::new(Semaphore::new(max_concurrent)),
        })
    }

    /// `resolved_env` contains bytes resolved inside the owner. It is never
    /// logged, persisted, returned, or inherited from the supervisor process.
    /// Command probes require every declared reference to have been resolved.
    /// For tmux, the owner can supply the service's resolved runtime environment.
    ///
    /// Cancellation notifies a dedicated worker instead of dropping its child.
    /// The worker kills and waits for that direct child, retaining the semaphore
    /// permit until reaping finishes. Thus even an uninterruptible child cannot
    /// create unbounded replacement probe tasks or block a response deadline.
    pub async fn probe(
        &self,
        identity: &ProcessIdentity,
        request: &ProbeRequest,
        resolved_env: BTreeMap<String, String>,
    ) -> ProbeResult {
        let generation = request.generation;
        if generation != identity.generation {
            return result(
                generation,
                ProbeClass::TargetUnavailable,
                "stale_generation",
            );
        }
        if !valid_probe(&request.spec) || !valid_environment(&resolved_env) {
            return result(
                generation,
                ProbeClass::MechanismFailure,
                "invalid_probe_configuration",
            );
        }
        let deadline = Instant::now() + Duration::from_millis(request.spec.timeout_ms);
        let permit = match timeout_at(deadline, self.slots.clone().acquire_owned()).await {
            Ok(Ok(permit)) => permit,
            _ => {
                return result(
                    generation,
                    ProbeClass::MechanismFailure,
                    "probe_capacity_timeout",
                )
            }
        };
        let (reply, receive) = oneshot::channel();
        let (cancel, cancelled) = oneshot::channel();
        let _cancel_on_drop = CancelOnDrop(Some(cancel));
        let request = request.clone();
        let identity = identity.clone();
        tokio::spawn(async move {
            let mut context = Context {
                deadline,
                cancelled,
                cleanup: Vec::new(),
                output_count: Arc::new(AtomicUsize::new(0)),
            };
            let observation = execute(&mut context, &identity, &request, &resolved_env).await;
            // Return the bounded observation before waiting on a killed child.
            // Reaping retains the concurrency slot and has no second owner.
            let _ = reply.send(observation);
            for mut child in context.cleanup {
                let _ = child.wait().await;
            }
            drop(permit);
        });
        receive.await.unwrap_or_else(|_| {
            result(
                generation,
                ProbeClass::MechanismFailure,
                "probe_worker_unavailable",
            )
        })
    }
}

struct CancelOnDrop(Option<oneshot::Sender<()>>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if let Some(cancel) = self.0.take() {
            let _ = cancel.send(());
        }
    }
}

#[derive(Clone, Copy)]
enum Failure {
    Timeout,
    Cancelled,
    OutputLimit,
    Mechanism(&'static str),
    Service(&'static str),
    Target(&'static str),
}

impl Failure {
    fn observation(self, generation: Generation) -> ProbeResult {
        let (class, code) = match self {
            Self::Timeout => (ProbeClass::ServiceFailure, "probe_timeout"),
            Self::Cancelled => (ProbeClass::MechanismFailure, "probe_cancelled"),
            Self::OutputLimit => (ProbeClass::MechanismFailure, "probe_output_limit"),
            Self::Mechanism(code) => (ProbeClass::MechanismFailure, code),
            Self::Service(code) => (ProbeClass::ServiceFailure, code),
            Self::Target(code) => (ProbeClass::TargetUnavailable, code),
        };
        result(generation, class, code)
    }
}

struct Context {
    deadline: Instant,
    cancelled: oneshot::Receiver<()>,
    cleanup: Vec<Child>,
    output_count: Arc<AtomicUsize>,
}

impl Context {
    async fn bounded<T>(
        &mut self,
        future: impl std::future::Future<Output = std::result::Result<T, Failure>>,
    ) -> std::result::Result<T, Failure> {
        tokio::select! {
            biased;
            _ = &mut self.cancelled => Err(Failure::Cancelled),
            _ = sleep_until(self.deadline) => Err(Failure::Timeout),
            value = future => value,
        }
    }
}

fn result(generation: Generation, class: ProbeClass, code: &'static str) -> ProbeResult {
    ProbeResult {
        generation,
        class,
        code: code.into(),
        panes: Vec::new(),
        expectations_met: None,
        observed_ms: now_ms(),
    }
}

fn valid_probe(spec: &ProbeSpec) -> bool {
    if spec.timeout_ms == 0
        || spec.timeout_ms > 60_000
        || spec.max_output_bytes == 0
        || spec.max_output_bytes > MAX_OUTPUT
    {
        return false;
    }
    match &spec.probe {
        ProbeKind::Process => true,
        ProbeKind::Command { command } => validate_command(command).is_ok(),
        ProbeKind::UnixSocket { path } => path.is_absolute() && path.as_os_str().len() <= 107,
        ProbeKind::Http {
            address,
            port,
            path,
            expected_status,
        } => {
            address.parse::<IpAddr>().is_ok()
                && *port != 0
                && path.starts_with('/')
                && path.len() <= 8192
                && !path.bytes().any(|c| c <= b' ' || c == 127)
                && (100..600).contains(expected_status)
        }
        ProbeKind::Tmux {
            executable,
            socket,
            expected_sessions,
            expected_panes,
        } => {
            executable.is_absolute()
                && socket.is_absolute()
                && socket.as_os_str().len() <= 107
                && expected_sessions.len() <= 128
                && expected_panes.len() <= 1024
                && expected_sessions
                    .iter()
                    .all(|x| !x.is_empty() && x.len() <= 128 && !x.chars().any(char::is_control))
                && expected_panes.iter().all(|x| valid_tmux_id(x, '%'))
        }
    }
}

fn valid_environment(env: &BTreeMap<String, String>) -> bool {
    env.len() <= 256
        && env.iter().all(|(key, value)| {
            valid_id(key) && !key.contains('=') && !value.contains('\0') && value.len() <= 65_536
        })
}

async fn execute(
    context: &mut Context,
    identity: &ProcessIdentity,
    request: &ProbeRequest,
    env: &BTreeMap<String, String>,
) -> ProbeResult {
    let verified = match VerifiedProcess::open(identity) {
        Ok(verified) => verified,
        Err(error) => return identity_failure(error).observation(request.generation),
    };
    let observation = match &request.spec.probe {
        ProbeKind::Process => Ok(result(
            request.generation,
            ProbeClass::Success,
            "process_alive",
        )),
        ProbeKind::Command { command } => {
            if command.env_refs.keys().any(|key| !env.contains_key(key)) {
                Err(Failure::Mechanism("environment_reference_unresolved"))
            } else if command.identity.uid != rustix::process::geteuid().as_raw()
                || command.identity.gid != rustix::process::getegid().as_raw()
            {
                Err(Failure::Mechanism("runtime_identity_unavailable"))
            } else {
                let mut cmd = Command::new(&command.executable);
                cmd.args(&command.args).current_dir(&command.cwd);
                match run_command(context, cmd, identity, env, request.spec.max_output_bytes).await
                {
                    Ok(output) if output.status.success() => Ok(result(
                        request.generation,
                        ProbeClass::Success,
                        "command_success",
                    )),
                    Ok(_) => Err(Failure::Service("command_exit_nonzero")),
                    Err(failure) => Err(failure),
                }
            }
        }
        ProbeKind::UnixSocket { path } => {
            socket_identity(context, path, identity, &verified, false)
                .await
                .map(|_| {
                    result(
                        request.generation,
                        ProbeClass::Success,
                        "unix_socket_connected",
                    )
                })
        }
        ProbeKind::Http {
            address,
            port,
            path,
            expected_status,
        } => http(
            context,
            address,
            *port,
            path,
            *expected_status,
            request.spec.max_output_bytes,
        )
        .await
        .map(|_| {
            result(
                request.generation,
                ProbeClass::Success,
                "http_status_expected",
            )
        }),
        ProbeKind::Tmux {
            executable,
            socket,
            expected_sessions,
            expected_panes,
        } => {
            tmux(
                context,
                identity,
                &verified,
                request,
                executable,
                socket,
                expected_sessions,
                expected_panes,
                env,
            )
            .await
        }
    };
    // A probe may have raced target exit, even if an unrelated endpoint replied.
    match verified.alive() {
        Ok(true) => observation.unwrap_or_else(|failure| failure.observation(request.generation)),
        Ok(false) => Failure::Service("process_exited").observation(request.generation),
        Err(error) => identity_failure(error).observation(request.generation),
    }
}

fn identity_failure(error: Error) -> Failure {
    match error {
        Error::NotFound => Failure::Service("process_exited"),
        Error::StaleGeneration => Failure::Target("process_identity_mismatch"),
        Error::Unsupported(_) => Failure::Mechanism("pidfd_unavailable"),
        _ => Failure::Mechanism("process_identity_unverifiable"),
    }
}

/// Read kernel identity in the owner's PID namespace. Passing the owner's UUID
/// labels an observation; it does not establish authority to control that PID.
/// A saved PID alone is never accepted by `verify_process_identity`.
pub fn read_process_identity(
    pid: u32,
    owner_instance: Uuid,
    generation: Generation,
) -> Result<ProcessIdentity> {
    if pid == 0 || pid > i32::MAX as u32 {
        return Err(Error::Invalid("invalid process id".into()));
    }
    let boot_id = read_small("/proc/sys/kernel/random/boot_id", 128)?
        .trim()
        .to_owned();
    if Uuid::parse_str(&boot_id).is_err() {
        return Err(Error::Io);
    }
    let (start_time_ticks, _) = proc_stat(pid)?;
    Ok(ProcessIdentity {
        owner_instance,
        boot_id,
        pid,
        start_time_ticks,
        generation,
    })
}

/// Reopens a pidfd and compares boot ID and `/proc` start time while holding it.
/// Returns false for an exited or different process, and an error when this host
/// cannot establish identity. This performs no signalling and no reaping.
pub fn verify_process_identity(identity: &ProcessIdentity) -> Result<bool> {
    match VerifiedProcess::open(identity) {
        Ok(verified) => verified.alive(),
        Err(Error::NotFound | Error::StaleGeneration) => Ok(false),
        Err(error) => Err(error),
    }
}

struct VerifiedProcess {
    pidfd: OwnedFd,
    uid: u32,
    gid: u32,
}
impl VerifiedProcess {
    fn open(identity: &ProcessIdentity) -> Result<Self> {
        let pid = i32::try_from(identity.pid)
            .ok()
            .and_then(Pid::from_raw)
            .ok_or(Error::StaleGeneration)?;
        let pidfd = pidfd_open(pid, PidfdFlags::empty()).map_err(|error| match error {
            rustix::io::Errno::SRCH => Error::NotFound,
            rustix::io::Errno::NOSYS | rustix::io::Errno::INVAL => {
                Error::Unsupported("pidfd".into())
            }
            _ => Error::Io,
        })?;
        let observed =
            read_process_identity(identity.pid, identity.owner_instance, identity.generation)?;
        if observed.boot_id != identity.boot_id
            || observed.start_time_ticks != identity.start_time_ticks
        {
            return Err(Error::StaleGeneration);
        }
        let metadata = std::fs::metadata(format!("/proc/{}", identity.pid)).map_err(proc_error)?;
        let this = Self {
            pidfd,
            uid: metadata.uid(),
            gid: metadata.gid(),
        };
        if !this.alive()? {
            return Err(Error::NotFound);
        }
        Ok(this)
    }
    fn alive(&self) -> Result<bool> {
        let mut fds = [PollFd::new(&self.pidfd, PollFlags::IN)];
        let timeout = Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let count = poll(&mut fds, Some(&timeout)).map_err(|_| Error::Io)?;
        Ok(count == 0)
    }
}

fn proc_error(error: std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::NotFound
    } else {
        Error::Io
    }
}
fn read_small(path: impl AsRef<Path>, max: usize) -> Result<String> {
    let mut value = String::new();
    std::fs::File::open(path)
        .map_err(proc_error)?
        .take((max + 1) as u64)
        .read_to_string(&mut value)
        .map_err(proc_error)?;
    if value.len() > max {
        return Err(Error::Io);
    }
    Ok(value)
}
fn proc_stat(pid: u32) -> Result<(u64, char)> {
    let stat = read_small(format!("/proc/{pid}/stat"), 8192)?;
    // comm is parenthesized and may itself contain spaces or parentheses.
    let (_, fields) = stat.rsplit_once(')').ok_or(Error::Io)?;
    let fields: Vec<_> = fields.split_whitespace().collect();
    let state = fields
        .first()
        .and_then(|x| x.chars().next())
        .ok_or(Error::Io)?;
    let start = fields
        .get(19)
        .ok_or(Error::Io)?
        .parse()
        .map_err(|_| Error::Io)?;
    Ok((start, state))
}

struct CommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
}

async fn run_command(
    context: &mut Context,
    mut command: Command,
    service: &ProcessIdentity,
    env: &BTreeMap<String, String>,
    max_output: usize,
) -> std::result::Result<CommandOutput, Failure> {
    if Instant::now() >= context.deadline {
        return Err(Failure::Timeout);
    }
    if !matches!(
        context.cancelled.try_recv(),
        Err(oneshot::error::TryRecvError::Empty)
    ) {
        return Err(Failure::Cancelled);
    }
    command
        .env_clear()
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|_| Failure::Mechanism("probe_spawn_failed"))?;
    let pid = child
        .id()
        .and_then(|pid| i32::try_from(pid).ok())
        .and_then(Pid::from_raw);
    // A child cannot have its PID recycled before it is reaped by this owner.
    // Keep a pidfd too, so cleanup never sends a signal to a reused numeric PID.
    let child_fd = pid.and_then(|pid| pidfd_open(pid, PidfdFlags::empty()).ok());
    let child_identity = child.id().and_then(|pid| {
        read_process_identity(pid, service.owner_instance, service.generation).ok()
    });
    if child_fd.is_none() || child_identity.is_none() {
        let _ = child.start_kill();
        context.cleanup.push(child);
        return Err(Failure::Mechanism("probe_identity_unverifiable"));
    }
    let (Some(mut stdout), Some(mut stderr)) = (child.stdout.take(), child.stderr.take()) else {
        if let Some(fd) = child_fd {
            let _ = pidfd_send_signal(fd, Signal::KILL);
        }
        context.cleanup.push(child);
        return Err(Failure::Mechanism("probe_pipe_unavailable"));
    };
    let total = context.output_count.clone();
    let collected = context
        .bounded(async {
            let (status, stdout, _) = tokio::try_join!(
                async {
                    child
                        .wait()
                        .await
                        .map_err(|_| Failure::Mechanism("probe_wait_failed"))
                },
                drain(&mut stdout, max_output, &total, true),
                drain(&mut stderr, max_output, &total, false),
            )?;
            Ok(CommandOutput { status, stdout })
        })
        .await;
    drop(stdout);
    drop(stderr);
    if collected.is_err() {
        if let Some(fd) = child_fd {
            let _ = pidfd_send_signal(fd, Signal::KILL);
        }
        context.cleanup.push(child);
    }
    collected
}

async fn drain(
    reader: &mut (impl AsyncRead + Unpin),
    limit: usize,
    total: &AtomicUsize,
    keep: bool,
) -> std::result::Result<Vec<u8>, Failure> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|_| Failure::Mechanism("probe_output_read_failed"))?;
        if count == 0 {
            return Ok(bytes);
        }
        if total
            .fetch_add(count, Ordering::Relaxed)
            .saturating_add(count)
            > limit
        {
            return Err(Failure::OutputLimit);
        }
        if keep {
            bytes.extend_from_slice(&buffer[..count]);
        }
    }
}

fn socket_io(error: std::io::Error) -> Failure {
    use std::io::ErrorKind::*;
    match error.kind() {
        PermissionDenied => Failure::Mechanism("endpoint_access_denied"),
        NotFound => Failure::Service("endpoint_missing"),
        ConnectionRefused | ConnectionReset | ConnectionAborted | BrokenPipe | UnexpectedEof => {
            Failure::Service("endpoint_unresponsive")
        }
        NetworkUnreachable | HostUnreachable => Failure::Target("endpoint_unreachable"),
        _ => Failure::Mechanism("endpoint_probe_failed"),
    }
}

async fn socket_identity(
    context: &mut Context,
    path: &Path,
    identity: &ProcessIdentity,
    verified: &VerifiedProcess,
    private: bool,
) -> std::result::Result<(), Failure> {
    if private {
        let parent = path
            .parent()
            .ok_or(Failure::Mechanism("tmux_socket_not_private"))?;
        let dir = std::fs::symlink_metadata(parent).map_err(socket_io)?;
        let socket = std::fs::symlink_metadata(path).map_err(socket_io)?;
        if !dir.is_dir()
            || dir.file_type().is_symlink()
            || dir.uid() != verified.uid
            || dir.mode() & 0o077 != 0
            || !socket.file_type().is_socket()
            || socket.uid() != verified.uid
            || socket.mode() & 0o077 != 0
        {
            return Err(Failure::Mechanism("tmux_socket_not_private"));
        }
    }
    context
        .bounded(async {
            let stream = UnixStream::connect(path).await.map_err(socket_io)?;
            let cred = rustix::net::sockopt::socket_peercred(&stream)
                .map_err(|_| Failure::Mechanism("socket_peer_unverifiable"))?;
            if cred.pid.as_raw_pid() as u32 != identity.pid
                || cred.uid.as_raw() != verified.uid
                || cred.gid.as_raw() != verified.gid
            {
                return Err(Failure::Target("socket_peer_identity_mismatch"));
            }
            if !verified.alive().map_err(identity_failure)? {
                return Err(Failure::Service("process_exited"));
            }
            Ok(())
        })
        .await
}

async fn http(
    context: &mut Context,
    address: &str,
    port: u16,
    path: &str,
    expected: u16,
    limit: usize,
) -> std::result::Result<(), Failure> {
    let ip = address
        .parse::<IpAddr>()
        .map_err(|_| Failure::Mechanism("http_numeric_address_required"))?;
    let address = SocketAddr::new(ip, port);
    context
        .bounded(async {
            let mut stream = TcpStream::connect(address).await.map_err(socket_io)?;
            let request =
                format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
            stream
                .write_all(request.as_bytes())
                .await
                .map_err(socket_io)?;
            let mut line = Vec::new();
            let mut buffer = [0; 1024];
            loop {
                let count = stream.read(&mut buffer).await.map_err(socket_io)?;
                if count == 0 {
                    return Err(Failure::Service("http_response_incomplete"));
                }
                for &byte in &buffer[..count] {
                    if line.len() >= limit {
                        return Err(Failure::OutputLimit);
                    }
                    line.push(byte);
                    if byte == b'\n' {
                        let text = std::str::from_utf8(&line)
                            .map_err(|_| Failure::Service("http_status_malformed"))?;
                        let fields: Vec<_> = text.split_whitespace().collect();
                        if fields.len() < 2
                            || !matches!(fields[0], "HTTP/1.0" | "HTTP/1.1")
                            || fields[1].len() != 3
                        {
                            return Err(Failure::Service("http_status_malformed"));
                        }
                        let status = fields[1]
                            .parse::<u16>()
                            .map_err(|_| Failure::Service("http_status_malformed"))?;
                        return if status == expected {
                            Ok(())
                        } else {
                            Err(Failure::Service("http_status_unexpected"))
                        };
                    }
                }
            }
        })
        .await
}

#[allow(clippy::too_many_arguments)]
async fn tmux(
    context: &mut Context,
    identity: &ProcessIdentity,
    verified: &VerifiedProcess,
    request: &ProbeRequest,
    executable: &Path,
    socket: &Path,
    expected_sessions: &[String],
    expected_panes: &[String],
    env: &BTreeMap<String, String>,
) -> std::result::Result<ProbeResult, Failure> {
    socket_identity(context, socket, identity, verified, true).await?;
    if verified.uid != rustix::process::geteuid().as_raw()
        || verified.gid != rustix::process::getegid().as_raw()
    {
        return Err(Failure::Mechanism("runtime_identity_unavailable"));
    }
    let pid = tmux_query(
        context,
        executable,
        socket,
        &["display-message", "-p", "#{pid}"],
        identity,
        env,
        request.spec.max_output_bytes,
    )
    .await?;
    if pid.trim().parse::<u32>().ok() != Some(identity.pid) {
        return Err(Failure::Target("tmux_server_identity_mismatch"));
    }
    // tmux sanitizes literal control characters in arguments; use printable
    // delimiters. split_once preserves a pipe inside the session name itself.
    let listed_sessions = tmux_query(
        context,
        executable,
        socket,
        &["list-sessions", "-F", "#{session_id}|#{session_name}"],
        identity,
        env,
        request.spec.max_output_bytes,
    )
    .await?;
    let mut sessions = BTreeSet::new();
    for line in listed_sessions.lines() {
        let (id, name) = line
            .split_once('|')
            .ok_or(Failure::Mechanism("tmux_response_malformed"))?;
        if !valid_tmux_id(id, '$')
            || name.is_empty()
            || name.len() > 128
            || name.chars().any(char::is_control)
        {
            return Err(Failure::Mechanism("tmux_response_malformed"));
        }
        sessions.insert(id.to_owned());
        sessions.insert(name.to_owned());
    }
    let listed_panes = if sessions.is_empty() {
        String::new()
    } else {
        tmux_query(
            context,
            executable,
            socket,
            &[
                "list-panes",
                "-a",
                "-F",
                "#{pane_id}|#{pane_dead}|#{pane_dead_status}",
            ],
            identity,
            env,
            request.spec.max_output_bytes,
        )
        .await?
    };
    let mut panes = Vec::new();
    for line in listed_panes.lines() {
        let fields: Vec<_> = line.split('|').collect();
        if fields.len() != 3
            || !valid_tmux_id(fields[0], '%')
            || !matches!(fields[1], "0" | "1")
            || panes.len() >= 1024
        {
            return Err(Failure::Mechanism("tmux_response_malformed"));
        }
        let exit_code = if fields[2].is_empty() {
            None
        } else {
            Some(
                fields[2]
                    .parse::<i32>()
                    .map_err(|_| Failure::Mechanism("tmux_response_malformed"))?,
            )
        };
        panes.push(PaneStatus {
            pane_id: fields[0].into(),
            dead: fields[1] == "1",
            exit_code,
        });
    }
    // Recheck the private endpoint after the separate read-only tmux clients.
    // Its directory must also be outside workload-writable mounts; mode 0700
    // cannot defend a directory from an attacker running with the same UID.
    socket_identity(context, socket, identity, verified, true).await?;
    let expectations_met = expected_sessions.iter().all(|x| sessions.contains(x))
        && expected_panes
            .iter()
            .all(|id| panes.iter().any(|pane| &pane.pane_id == id && !pane.dead));
    let mut observation = result(
        request.generation,
        ProbeClass::Success,
        "tmux_server_healthy",
    );
    observation.expectations_met = Some(expectations_met);
    observation.panes = panes;
    Ok(observation)
}

fn valid_tmux_id(value: &str, prefix: char) -> bool {
    value.len() >= 2
        && value.len() <= 20
        && value.starts_with(prefix)
        && value[1..].bytes().all(|c| c.is_ascii_digit())
}

#[allow(clippy::too_many_arguments)]
async fn tmux_query(
    context: &mut Context,
    executable: &Path,
    socket: &Path,
    args: &[&str],
    identity: &ProcessIdentity,
    env: &BTreeMap<String, String>,
    limit: usize,
) -> std::result::Result<String, Failure> {
    let mut command = Command::new(executable);
    // -N is mandatory even for inspection: no missing server may be created.
    command
        .args(["-N", "-f", "/dev/null", "-S"])
        .arg(socket)
        .args(args)
        .current_dir("/");
    let output = run_command(context, command, identity, env, limit).await?;
    if !output.status.success() {
        return Err(Failure::Service("tmux_command_failed"));
    }
    String::from_utf8(output.stdout).map_err(|_| Failure::Mechanism("tmux_response_malformed"))
}
