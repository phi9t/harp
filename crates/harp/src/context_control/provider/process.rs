use std::fs::File;
use std::io::{self, Read, Write};
use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::iterator::{Handle as SignalHandle, Signals};

use super::invocation::{
    canonical_raw_archive, canonical_raw_members_manifest, ClaimedProviderInvocation,
    ProviderRawMember, SealedProviderEvidence, MAX_FINAL_MESSAGE_BYTES, MAX_STDERR_BYTES,
    MAX_STDOUT_BYTES,
};
use super::BoundProviderInvocation;
use crate::AppError;

const MAX_STREAM_BYTES: usize = if MAX_STDOUT_BYTES < MAX_STDERR_BYTES {
    MAX_STDOUT_BYTES
} else {
    MAX_STDERR_BYTES
};
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(5);
#[cfg(test)]
const SIGNAL_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderProcessResult {
    pub exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
    pub capture_complete: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub final_message_sha256: Option<String>,
    pub raw_archive_sha256: String,
}

#[derive(Debug)]
pub struct ProviderExecution {
    pub result: ProviderProcessResult,
    pub evidence: SealedProviderEvidence,
}

pub fn execute(invocation: BoundProviderInvocation) -> Result<ProviderExecution, AppError> {
    execute_with_options(invocation, ProcessOptions::standard()?)
}

struct ProcessOptions {
    stream_limit: usize,
    signal_source: SignalSource,
    terminal_stdout: Box<dyn Write + Send>,
    terminal_stderr: Box<dyn Write + Send>,
}

impl ProcessOptions {
    fn standard() -> Result<Self, AppError> {
        Ok(Self {
            stream_limit: MAX_STREAM_BYTES,
            signal_source: SignalSource::Global(Signals::new([SIGINT, SIGTERM, SIGHUP]).map_err(
                |error| {
                    AppError::io(
                        "provider.signal",
                        "could not install provider signal forwarding",
                        error,
                    )
                },
            )?),
            terminal_stdout: Box::new(io::stdout()),
            terminal_stderr: Box::new(io::stderr()),
        })
    }

    #[cfg(test)]
    fn injected(stream_limit: usize, receiver: mpsc::Receiver<ForwardSignal>) -> Self {
        Self::injected_with_terminals(stream_limit, receiver, io::sink(), io::sink())
    }

    #[cfg(test)]
    fn injected_with_terminals(
        stream_limit: usize,
        receiver: mpsc::Receiver<ForwardSignal>,
        terminal_stdout: impl Write + Send + 'static,
        terminal_stderr: impl Write + Send + 'static,
    ) -> Self {
        Self {
            stream_limit,
            signal_source: SignalSource::Injected(receiver),
            terminal_stdout: Box::new(terminal_stdout),
            terminal_stderr: Box::new(terminal_stderr),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ForwardSignal {
    Interrupt,
    Terminate,
    Hangup,
}

impl ForwardSignal {
    const fn number(self) -> i32 {
        match self {
            Self::Interrupt => SIGINT,
            Self::Terminate => SIGTERM,
            Self::Hangup => SIGHUP,
        }
    }

    fn from_number(signal: i32) -> Result<Self, AppError> {
        match signal {
            SIGINT => Ok(Self::Interrupt),
            SIGTERM => Ok(Self::Terminate),
            SIGHUP => Ok(Self::Hangup),
            _ => Err(AppError::invalid_input(
                "provider.signal",
                "signal control produced a signal outside the fixed forwarding set",
            )),
        }
    }
}

enum SignalSource {
    Global(Signals),
    #[cfg(test)]
    Injected(mpsc::Receiver<ForwardSignal>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkerKind {
    Stdin,
    Stdout,
    Stderr,
}

enum ProcessEvent {
    WorkerFinished(WorkerKind, Result<(), AppError>),
    SignalForwardingFailed(AppError),
}

struct SignalForwarder {
    stop: Arc<AtomicBool>,
    handle: Option<SignalHandle>,
    thread: Option<thread::JoinHandle<()>>,
}

impl SignalForwarder {
    fn start(
        source: SignalSource,
        process_group: i32,
        events: mpsc::Sender<ProcessEvent>,
    ) -> Result<Self, AppError> {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let (handle, thread) = match source {
            SignalSource::Global(mut signals) => {
                let handle = signals.handle();
                let thread = thread::Builder::new()
                    .name("harp-provider-signals".to_owned())
                    .spawn(move || {
                        for signal in signals.forever() {
                            if worker_stop.load(Ordering::Acquire) {
                                break;
                            }
                            let result = ForwardSignal::from_number(signal)
                                .and_then(|signal| forward_signal(process_group, signal));
                            if let Err(error) = result {
                                let _ = events.send(ProcessEvent::SignalForwardingFailed(error));
                                break;
                            }
                        }
                    })
                    .map_err(|error| {
                        AppError::io(
                            "provider.signal",
                            "could not start provider signal-forwarding thread",
                            error,
                        )
                    })?;
                (Some(handle), thread)
            }
            #[cfg(test)]
            SignalSource::Injected(receiver) => {
                let thread = thread::Builder::new()
                    .name("harp-provider-signals".to_owned())
                    .spawn(move || {
                        while !worker_stop.load(Ordering::Acquire) {
                            match receiver.recv_timeout(SIGNAL_POLL_INTERVAL) {
                                Ok(signal) => {
                                    if let Err(error) = forward_signal(process_group, signal) {
                                        let _ = events
                                            .send(ProcessEvent::SignalForwardingFailed(error));
                                        break;
                                    }
                                }
                                Err(mpsc::RecvTimeoutError::Timeout) => {}
                                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                            }
                        }
                    })
                    .map_err(|error| {
                        AppError::io(
                            "provider.signal",
                            "could not start provider signal-forwarding thread",
                            error,
                        )
                    })?;
                (None, thread)
            }
        };
        Ok(Self {
            stop,
            handle,
            thread: Some(thread),
        })
    }

    fn stop_and_join(mut self) -> Result<(), AppError> {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = &self.handle {
            handle.close();
        }
        self.thread
            .take()
            .expect("signal forwarding thread is present")
            .join()
            .map_err(|_| {
                AppError::external(
                    "provider.signal",
                    "provider signal-forwarding thread panicked",
                )
            })
    }
}

fn execute_with_options(
    invocation: BoundProviderInvocation,
    options: ProcessOptions,
) -> Result<ProviderExecution, AppError> {
    if options.stream_limit == 0 || options.stream_limit > MAX_STREAM_BYTES {
        return Err(AppError::invalid_input(
            "provider.output_limit",
            format!("provider stream limit must be between 1 and {MAX_STREAM_BYTES} bytes"),
        ));
    }

    let invocation = invocation.claim_launch()?;
    let stdout_file = invocation.create_raw_member(ProviderRawMember::StdoutJsonl)?;
    let stderr_file = invocation.create_raw_member(ProviderRawMember::StderrBin)?;
    let mut command = Command::new(invocation.executable());
    command
        .args(invocation.arguments())
        .current_dir(invocation.working_directory())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        });
    }

    // This check is deliberately adjacent to spawn. It authenticates the executable,
    // working directory, raw namespace, and absent final-message leaf one last time.
    invocation.verify_before_spawn()?;
    let mut child = command.spawn().map_err(|error| {
        AppError::io(
            "provider.spawn",
            "could not spawn provider in a new process group",
            error,
        )
    })?;
    let process_group = match i32::try_from(child.id()) {
        Ok(process_group) => process_group,
        Err(_) => {
            let error = AppError::external(
                "provider.process_group",
                "provider process ID does not fit a Unix process group",
            );
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };

    let child_stdin = match child.stdin.take() {
        Some(pipe) => pipe,
        None => {
            return Err(cleanup_spawned_child(
                &mut child,
                process_group,
                AppError::external("provider.stdin", "spawned provider has no stdin pipe"),
            ));
        }
    };
    let child_stdout = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            return Err(cleanup_spawned_child(
                &mut child,
                process_group,
                AppError::external("provider.stdout", "spawned provider has no stdout pipe"),
            ));
        }
    };
    let child_stderr = match child.stderr.take() {
        Some(pipe) => pipe,
        None => {
            return Err(cleanup_spawned_child(
                &mut child,
                process_group,
                AppError::external("provider.stderr", "spawned provider has no stderr pipe"),
            ));
        }
    };
    let (events, received_events) = mpsc::channel();
    let mut workers = Vec::with_capacity(3);
    match spawn_stdin_worker(
        child_stdin,
        invocation.stdin_bytes().to_vec(),
        events.clone(),
    ) {
        Ok(worker) => workers.push(worker),
        Err(error) => {
            return Err(abort_process_setup(
                &mut child,
                process_group,
                workers,
                error,
            ));
        }
    }
    match spawn_tee_worker(
        WorkerKind::Stdout,
        child_stdout,
        stdout_file,
        options.terminal_stdout,
        options.stream_limit,
        events.clone(),
    ) {
        Ok(worker) => workers.push(worker),
        Err(error) => {
            return Err(abort_process_setup(
                &mut child,
                process_group,
                workers,
                error,
            ));
        }
    }
    match spawn_tee_worker(
        WorkerKind::Stderr,
        child_stderr,
        stderr_file,
        options.terminal_stderr,
        options.stream_limit,
        events.clone(),
    ) {
        Ok(worker) => workers.push(worker),
        Err(error) => {
            return Err(abort_process_setup(
                &mut child,
                process_group,
                workers,
                error,
            ));
        }
    }
    let mut signal_forwarder =
        match SignalForwarder::start(options.signal_source, process_group, events.clone()) {
            Ok(forwarder) => Some(forwarder),
            Err(error) => {
                return Err(abort_process_setup(
                    &mut child,
                    process_group,
                    workers,
                    error,
                ));
            }
        };
    drop(events);

    let mut status = None;
    let mut leader_finished = false;
    let mut first_error = None;
    let mut group_terminated = false;
    while !leader_finished || workers.iter().any(|worker| !worker.is_finished()) {
        drain_process_events(&received_events, &mut first_error);
        if first_error.is_some() && !group_terminated {
            if let Err(error) = terminate_process_group(process_group) {
                first_error.get_or_insert(error);
                let _ = child.kill();
            }
            group_terminated = true;
        }
        if !leader_finished {
            match child_has_exited(child.id()) {
                Ok(true) => {
                    stop_signal_forwarding(&mut signal_forwarder, &mut first_error);
                    if !group_terminated {
                        if let Err(error) = terminate_group_after_leader_exit(process_group) {
                            first_error.get_or_insert(error);
                        }
                        group_terminated = true;
                    }
                    match child.wait() {
                        Ok(exit_status) => status = Some(exit_status),
                        Err(error) => {
                            first_error.get_or_insert_with(|| {
                                AppError::io(
                                    "provider.wait",
                                    "could not reap provider process",
                                    error,
                                )
                            });
                        }
                    }
                    leader_finished = true;
                }
                Ok(false) => {}
                Err(error) => {
                    first_error.get_or_insert_with(|| {
                        AppError::io(
                            "provider.wait",
                            "could not inspect provider process status",
                            error,
                        )
                    });
                    if !group_terminated {
                        if let Err(error) = terminate_process_group(process_group) {
                            first_error.get_or_insert(error);
                            let _ = child.kill();
                        }
                        group_terminated = true;
                    }
                    stop_signal_forwarding(&mut signal_forwarder, &mut first_error);
                    match child.wait() {
                        Ok(exit_status) => status = Some(exit_status),
                        Err(error) => {
                            first_error.get_or_insert_with(|| {
                                AppError::io(
                                    "provider.wait",
                                    "could not reap provider process",
                                    error,
                                )
                            });
                        }
                    }
                    leader_finished = true;
                }
            }
        }
        if !leader_finished || workers.iter().any(|worker| !worker.is_finished()) {
            thread::sleep(WORKER_POLL_INTERVAL);
        }
    }

    stop_signal_forwarding(&mut signal_forwarder, &mut first_error);
    drain_process_events(&received_events, &mut first_error);
    for worker in workers {
        if worker.join().is_err() {
            first_error.get_or_insert_with(|| {
                AppError::external("provider.capture", "provider I/O worker thread panicked")
            });
        }
    }
    drain_process_events(&received_events, &mut first_error);
    if let Some(error) = first_error {
        return Err(error);
    }
    let status = status.ok_or_else(|| {
        AppError::external(
            "provider.wait",
            "provider process ended without a wait status",
        )
    })?;

    let stdout = read_raw_member(
        &invocation,
        ProviderRawMember::StdoutJsonl,
        options.stream_limit,
    )?
    .ok_or_else(|| missing_raw_member(ProviderRawMember::StdoutJsonl))?;
    let stderr = read_raw_member(
        &invocation,
        ProviderRawMember::StderrBin,
        options.stream_limit,
    )?
    .ok_or_else(|| missing_raw_member(ProviderRawMember::StderrBin))?;
    let final_message = read_raw_member(
        &invocation,
        ProviderRawMember::FinalMessage,
        MAX_FINAL_MESSAGE_BYTES,
    )?;
    let archive = canonical_raw_archive(&stdout, &stderr, final_message.as_deref())?;
    write_raw_member(
        &invocation,
        ProviderRawMember::RawArchive,
        &archive,
        "could not write provider raw archive",
    )?;
    let manifest = canonical_raw_members_manifest(&stdout, &stderr, final_message.as_deref());
    write_raw_member(
        &invocation,
        ProviderRawMember::RawMembersManifest,
        &manifest,
        "could not write provider raw-members manifest",
    )?;
    invocation.sync_raw_directory()?;
    let evidence = invocation.seal(true)?;
    let summary = evidence.summary();
    let result = ProviderProcessResult {
        exit_code: status.code(),
        terminated_by_signal: status.signal(),
        capture_complete: summary.capture_complete,
        stdout_sha256: summary.stdout_sha256.clone(),
        stderr_sha256: summary.stderr_sha256.clone(),
        final_message_sha256: summary.final_message_sha256.clone(),
        raw_archive_sha256: summary.raw_archive_sha256.clone(),
    };
    Ok(ProviderExecution { result, evidence })
}

fn stop_signal_forwarding(
    forwarder: &mut Option<SignalForwarder>,
    first_error: &mut Option<AppError>,
) {
    if let Some(forwarder) = forwarder.take() {
        if let Err(error) = forwarder.stop_and_join() {
            first_error.get_or_insert(error);
        }
    }
}

fn drain_process_events(events: &mpsc::Receiver<ProcessEvent>, first_error: &mut Option<AppError>) {
    while let Ok(event) = events.try_recv() {
        match event {
            ProcessEvent::WorkerFinished(_kind, result) => {
                if let Err(error) = result {
                    first_error.get_or_insert(error);
                }
            }
            ProcessEvent::SignalForwardingFailed(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
}

fn cleanup_spawned_child(
    child: &mut std::process::Child,
    process_group: i32,
    original: AppError,
) -> AppError {
    if terminate_process_group(process_group).is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    original
}

fn abort_process_setup(
    child: &mut std::process::Child,
    process_group: i32,
    workers: Vec<thread::JoinHandle<()>>,
    original: AppError,
) -> AppError {
    let original = cleanup_spawned_child(child, process_group, original);
    for worker in workers {
        let _ = worker.join();
    }
    original
}

fn spawn_stdin_worker(
    mut stdin: std::process::ChildStdin,
    prompt: Vec<u8>,
    events: mpsc::Sender<ProcessEvent>,
) -> Result<thread::JoinHandle<()>, AppError> {
    thread::Builder::new()
        .name("harp-provider-stdin".to_owned())
        .spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| {
                stdin.write_all(&prompt).map_err(|error| {
                    AppError::io(
                        "provider.stdin",
                        "could not write the provider prompt",
                        error,
                    )
                })
            }))
            .unwrap_or_else(|_| {
                Err(AppError::external(
                    "provider.stdin",
                    "provider stdin worker panicked",
                ))
            });
            drop(stdin);
            let _ = events.send(ProcessEvent::WorkerFinished(WorkerKind::Stdin, result));
        })
        .map_err(|error| {
            AppError::io(
                "provider.capture",
                "could not start provider stdin worker",
                error,
            )
        })
}

fn spawn_tee_worker<R>(
    kind: WorkerKind,
    reader: R,
    raw: File,
    terminal: Box<dyn Write + Send>,
    limit: usize,
    events: mpsc::Sender<ProcessEvent>,
) -> Result<thread::JoinHandle<()>, AppError>
where
    R: Read + Send + 'static,
{
    thread::Builder::new()
        .name(format!("harp-provider-{}", worker_stream_name(kind)))
        .spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| {
                tee_bounded(reader, raw, terminal, limit, kind)
            }))
            .unwrap_or_else(|_| {
                Err(AppError::external(
                    capture_error_code(kind),
                    format!("provider {} worker panicked", worker_stream_name(kind)),
                ))
            });
            let _ = events.send(ProcessEvent::WorkerFinished(kind, result));
        })
        .map_err(|error| {
            AppError::io(
                "provider.capture",
                "could not start provider output worker",
                error,
            )
        })
}

fn tee_bounded(
    mut reader: impl Read,
    mut raw: File,
    mut terminal: Box<dyn Write + Send>,
    limit: usize,
    kind: WorkerKind,
) -> Result<(), AppError> {
    let mut total = 0usize;
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(|error| {
            capture_io_error(kind, "could not read provider output pipe", error)
        })?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(total);
        let accepted = read.min(remaining);
        if accepted != 0 {
            raw.write_all(&buffer[..accepted]).map_err(|error| {
                capture_io_error(kind, "could not write provider raw output", error)
            })?;
            terminal.write_all(&buffer[..accepted]).map_err(|error| {
                capture_io_error(kind, "could not tee provider output to terminal", error)
            })?;
            total += accepted;
        }
        if accepted != read {
            raw.sync_all().map_err(|error| {
                capture_io_error(kind, "could not sync limited provider output", error)
            })?;
            return Err(AppError::invalid_input(
                "provider.output_limit",
                format!(
                    "provider {} exceeded the configured {limit}-byte limit",
                    worker_stream_name(kind)
                ),
            ));
        }
    }
    raw.sync_all()
        .map_err(|error| capture_io_error(kind, "could not sync provider raw output", error))?;
    terminal
        .flush()
        .map_err(|error| capture_io_error(kind, "could not flush provider terminal tee", error))
}

fn capture_io_error(kind: WorkerKind, context: &str, error: io::Error) -> AppError {
    AppError::io(capture_error_code(kind), context, error)
}

const fn capture_error_code(kind: WorkerKind) -> &'static str {
    match kind {
        WorkerKind::Stdout => "provider.stdout",
        WorkerKind::Stderr => "provider.stderr",
        WorkerKind::Stdin => "provider.stdin",
    }
}

const fn worker_stream_name(kind: WorkerKind) -> &'static str {
    match kind {
        WorkerKind::Stdout => "stdout",
        WorkerKind::Stderr => "stderr",
        WorkerKind::Stdin => "stdin",
    }
}

fn forward_signal(process_group: i32, signal: ForwardSignal) -> Result<(), AppError> {
    signal_process_group(process_group, signal.number(), true).map_err(|error| {
        AppError::io(
            "provider.signal",
            "could not forward signal to provider process group",
            error,
        )
    })
}

fn terminate_process_group(process_group: i32) -> Result<(), AppError> {
    signal_process_group(process_group, libc::SIGKILL, true).map_err(|error| {
        AppError::io(
            "provider.terminate",
            "could not terminate provider process group after capture failure",
            error,
        )
    })
}

fn terminate_group_after_leader_exit(process_group: i32) -> Result<(), AppError> {
    match signal_process_group(process_group, libc::SIGKILL, true) {
        Ok(()) => Ok(()),
        // On macOS, kill(2) reports EPERM when the group contains only the
        // unreaped leader. WNOWAIT still pins that leader and therefore the
        // numeric PGID, so this cannot refer to an unrelated process group.
        Err(error) if error.raw_os_error() == Some(libc::EPERM) => Ok(()),
        Err(error) => Err(AppError::io(
            "provider.terminate",
            "could not terminate provider process group after leader exit",
            error,
        )),
    }
}

fn child_has_exited(process: u32) -> io::Result<bool> {
    let process = libc::id_t::try_from(process).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "provider process ID does not fit waitid",
        )
    })?;
    loop {
        // Keep the leader unreaped so its PID continues to own the process-group
        // identity while any residual descendants are terminated.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                process,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result == 0 {
            return Ok(unsafe { info.si_pid() } != 0);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

fn signal_process_group(process_group: i32, signal: i32, ignore_missing: bool) -> io::Result<()> {
    if process_group <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "provider process group must be positive",
        ));
    }
    if unsafe { libc::kill(-process_group, signal) } == 0 {
        return Ok(());
    }
    let error = io::Error::last_os_error();
    if ignore_missing && error.raw_os_error() == Some(libc::ESRCH) {
        Ok(())
    } else {
        Err(error)
    }
}

fn read_raw_member(
    invocation: &ClaimedProviderInvocation,
    member: ProviderRawMember,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, AppError> {
    invocation.read_raw_member_bounded(member, max_bytes)
}

fn write_raw_member(
    invocation: &ClaimedProviderInvocation,
    member: ProviderRawMember,
    bytes: &[u8],
    context: &'static str,
) -> Result<(), AppError> {
    let mut file = invocation.create_raw_member(member)?;
    file.write_all(bytes)
        .map_err(|error| AppError::io(raw_member_error_code(member), context, error))?;
    file.sync_all()
        .map_err(|error| AppError::io(raw_member_error_code(member), context, error))
}

const fn raw_member_name(member: ProviderRawMember) -> &'static str {
    match member {
        ProviderRawMember::StdoutJsonl => "stdout.jsonl",
        ProviderRawMember::StderrBin => "stderr.bin",
        ProviderRawMember::FinalMessage => "final_message.bin",
        ProviderRawMember::RawArchive => "raw_traecli_run.tar.gz",
        ProviderRawMember::RawMembersManifest => "raw_members.tsv",
        ProviderRawMember::LaunchClaim => "launch.json",
    }
}

const fn raw_member_error_code(member: ProviderRawMember) -> &'static str {
    match member {
        ProviderRawMember::FinalMessage => "provider.final_message",
        ProviderRawMember::LaunchClaim => "provider.launch_claim",
        ProviderRawMember::RawArchive => "provider.raw_archive",
        ProviderRawMember::RawMembersManifest => "provider.raw_members",
        ProviderRawMember::StdoutJsonl | ProviderRawMember::StderrBin => "provider.raw_member",
    }
}

fn missing_raw_member(member: ProviderRawMember) -> AppError {
    AppError::invalid_input(
        raw_member_error_code(member),
        format!(
            "provider raw member {} disappeared during capture",
            raw_member_name(member)
        ),
    )
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Read as _;
    use std::os::fd::OwnedFd;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::AtomicUsize;
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use flate2::read::GzDecoder;
    use sha2::{Digest as _, Sha256};
    use tar::Archive;
    use tempfile::TempDir;

    use super::*;
    use crate::context_control::canonical::sha256_hex;
    use crate::context_control::context::ContextBundle;
    use crate::context_control::provider::{
        build_invocation, probe_snapshot, EpisodeProviderRawPath, EpisodeRawDirectoryAuthority,
        ProviderRunOptions,
    };
    use crate::context_control::routing::RouteDecision;
    use crate::context_control::RepositorySnapshot;
    use crate::context_control::{ProviderId, WorkflowId, CONTEXT_BUNDLE_SCHEMA};

    #[allow(dead_code)]
    mod process_support {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/context_control.rs"
        ));
    }
    use process_support::{
        ProcessProviderBehavior, ProcessProviderFixture, PROCESS_CONTEXT, PROCESS_FINAL_MESSAGE,
        PROCESS_STDERR, PROCESS_STDOUT, PROCESS_TASK,
    };

    const EPISODE_ID: &str = "ep-0123456789abcdef-01234567-89abcdef";

    #[test]
    fn provider_process_captures_exact_bytes_nonzero_exit_and_deterministic_archive() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::Complete { exit_code: 7 });
        let terminal_stdout = CapturedWriter::default();
        let terminal_stderr = CapturedWriter::default();
        let execution = execute_with_options(
            fixture.bound(),
            ProcessOptions::injected_with_terminals(
                64 * 1024 * 1024,
                mpsc::channel().1,
                terminal_stdout.clone(),
                terminal_stderr.clone(),
            ),
        )
        .unwrap();
        let result = execution.result;

        assert_eq!(
            fs::read(fixture.raw.join("stdout.jsonl")).unwrap(),
            PROCESS_STDOUT
        );
        assert_eq!(
            fs::read(fixture.raw.join("stderr.bin")).unwrap(),
            PROCESS_STDERR
        );
        assert_eq!(terminal_stdout.bytes(), PROCESS_STDOUT);
        assert_eq!(terminal_stderr.bytes(), PROCESS_STDERR);
        assert_eq!(
            fs::read(fixture.raw.join("final_message.bin")).unwrap(),
            PROCESS_FINAL_MESSAGE
        );
        assert_eq!(result.exit_code, Some(7));
        assert_eq!(result.terminated_by_signal, None);
        assert!(result.capture_complete);
        assert_eq!(result.stdout_sha256, digest(PROCESS_STDOUT));
        assert_eq!(result.stderr_sha256, digest(PROCESS_STDERR));
        assert_eq!(
            result.final_message_sha256,
            Some(digest(PROCESS_FINAL_MESSAGE))
        );

        assert_archive(&fixture.raw);
        assert_eq!(
            result.raw_archive_sha256,
            digest(&fs::read(fixture.raw.join("raw_traecli_run.tar.gz")).unwrap())
        );
        assert_eq!(
            fs::read_to_string(fixture.raw.join("raw_members.tsv")).unwrap(),
            format!(
                "{}\t{}\tfinal_message.bin\n{}\t{}\tstderr.bin\n{}\t{}\tstdout.jsonl\n",
                digest(PROCESS_FINAL_MESSAGE),
                PROCESS_FINAL_MESSAGE.len(),
                digest(PROCESS_STDERR),
                PROCESS_STDERR.len(),
                digest(PROCESS_STDOUT),
                PROCESS_STDOUT.len(),
            )
        );
    }

    #[test]
    fn provider_process_seals_evidence_after_editing_the_repository() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::CompleteWithRepositoryEdit);

        let execution = execute_with_options(
            fixture.bound(),
            ProcessOptions::injected(64 * 1024 * 1024, mpsc::channel().1),
        )
        .unwrap();

        assert_eq!(execution.result.exit_code, Some(0));
        assert!(execution.result.capture_complete);
        assert_eq!(
            fs::read(fixture.repository.join("provider-created.txt")).unwrap(),
            b"provider-created\n"
        );
        assert_eq!(
            RepositorySnapshot::capture(&fixture.repository)
                .unwrap()
                .repository_id,
            execution.evidence.repository().repository_id
        );
    }

    #[test]
    fn provider_process_seals_evidence_after_committing_repository_changes() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::CompleteWithRepositoryCommit);

        let execution = execute_with_options(
            fixture.bound(),
            ProcessOptions::injected(MAX_STREAM_BYTES, mpsc::channel().1),
        )
        .unwrap();

        assert_eq!(execution.result.exit_code, Some(0));
        assert!(execution.result.capture_complete);
        assert!(std::process::Command::new("git")
            .args(["log", "-1", "--format=%s"])
            .current_dir(&fixture.repository)
            .output()
            .is_ok_and(|output| output.status.success() && output.stdout == b"provider commit\n"));
        assert_eq!(
            RepositorySnapshot::capture(&fixture.repository)
                .unwrap()
                .repository_id,
            execution.evidence.repository().repository_id
        );
    }

    #[test]
    fn provider_process_terminates_group_on_stream_limit() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::Overflow);
        let dropped_terminals = Arc::new(AtomicUsize::new(0));

        let error = execute_with_options(
            fixture.bound(),
            ProcessOptions::injected_with_terminals(
                1024,
                mpsc::channel().1,
                DropTrackedWriter::new(Arc::clone(&dropped_terminals)),
                DropTrackedWriter::new(Arc::clone(&dropped_terminals)),
            ),
        )
        .unwrap_err();

        assert_eq!(error.code(), "provider.output_limit");
        assert_eq!(dropped_terminals.load(Ordering::Acquire), 2);
        assert_eq!(
            fs::metadata(fixture.raw.join("stdout.jsonl"))
                .unwrap()
                .len(),
            1024
        );
        let descendant: i32 = fs::read_to_string(fixture.raw.join("final_message.bin"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert_process_gone(descendant);
    }

    #[test]
    fn provider_process_forwards_injected_term_to_the_process_group() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::WaitForTerm);
        let final_message = fixture.raw.join("final_message.bin");
        let (signals, receiver) = mpsc::channel();

        let sender = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline {
                if fs::read(&final_message).is_ok_and(|bytes| bytes == b"ready\n") {
                    signals.send(ForwardSignal::Terminate).unwrap();
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
            panic!("provider did not become ready for injected TERM");
        });
        let execution = execute_with_options(
            fixture.bound(),
            ProcessOptions::injected(64 * 1024 * 1024, receiver),
        )
        .unwrap();
        let result = execution.result;
        sender.join().unwrap();

        assert_eq!(result.exit_code, Some(143));
        assert_eq!(result.terminated_by_signal, None);
        assert!(fs::read(fixture.raw.join("stderr.bin"))
            .unwrap()
            .ends_with(b"provider trapped TERM\n"));
        assert_eq!(
            fs::read(fixture.raw.join("final_message.bin")).unwrap(),
            b"term-trapped\n"
        );
    }

    #[test]
    fn provider_process_stops_signal_forwarding_before_slow_capture_finishes() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::Complete { exit_code: 0 });
        let final_message = fixture.raw.join("final_message.bin");
        let (signals, receiver) = mpsc::channel();
        let (entered, entered_receiver) = mpsc::channel();
        let (release, release_receiver) = mpsc::channel();
        let bound = fixture.bound();

        let execution = thread::spawn(move || {
            execute_with_options(
                bound,
                ProcessOptions::injected_with_terminals(
                    64 * 1024 * 1024,
                    receiver,
                    BlockingWriter {
                        entered: Some(entered),
                        release: release_receiver,
                    },
                    io::sink(),
                ),
            )
        });
        entered_receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("stdout tee reached the blocking terminal");
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && !final_message.exists() {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(final_message.exists(), "provider did not finish its output");
        thread::sleep(Duration::from_millis(100));

        assert!(
            signals.send(ForwardSignal::Terminate).is_err(),
            "signal receiver survived after the provider leader exited"
        );
        release.send(()).unwrap();
        execution.join().unwrap().unwrap();
    }

    #[test]
    fn provider_process_signal_control_accepts_only_the_forwarded_signal_set() {
        assert_eq!(
            [
                ForwardSignal::Interrupt.number(),
                ForwardSignal::Terminate.number(),
                ForwardSignal::Hangup.number(),
            ],
            [libc::SIGINT, libc::SIGTERM, libc::SIGHUP]
        );
        assert_eq!(
            ForwardSignal::from_number(libc::SIGUSR1)
                .unwrap_err()
                .code(),
            "provider.signal"
        );
    }

    #[test]
    fn provider_process_claim_allows_only_one_launch() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::Complete { exit_code: 0 });
        let command_sha256 = fixture
            .invocation
            .materialize(&fixture.raw_path)
            .unwrap()
            .command_sha256()
            .to_owned();
        let prompt_sha256 = fixture.invocation.prompt_sha256().to_owned();
        let first = fixture.bound();
        let second = fixture.bound();
        execute_with_options(
            first,
            ProcessOptions::injected(64 * 1024 * 1024, mpsc::channel().1),
        )
        .unwrap();

        let error = execute_with_options(
            second,
            ProcessOptions::injected(64 * 1024 * 1024, mpsc::channel().1),
        )
        .unwrap_err();

        assert_eq!(error.code(), "provider.output_exists");
        assert_eq!(
            fs::read_to_string(fixture.provider.launches()).unwrap(),
            "launched\n"
        );
        let claim: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.raw.join("launch.json")).unwrap()).unwrap();
        assert_eq!(claim["provider"], "trae");
        assert_eq!(claim["command_sha256"], command_sha256);
        assert_eq!(claim["prompt_sha256"], prompt_sha256);
    }

    #[test]
    fn provider_process_archive_bytes_are_deterministic_across_episodes() {
        let first = ProcessFixture::new(ProcessProviderBehavior::Complete { exit_code: 7 });
        let second = ProcessFixture::new(ProcessProviderBehavior::Complete { exit_code: 7 });
        for fixture in [&first, &second] {
            execute_with_options(
                fixture.bound(),
                ProcessOptions::injected(64 * 1024 * 1024, mpsc::channel().1),
            )
            .unwrap();
        }

        assert_eq!(
            fs::read(first.raw.join("raw_traecli_run.tar.gz")).unwrap(),
            fs::read(second.raw.join("raw_traecli_run.tar.gz")).unwrap()
        );
    }

    #[test]
    fn provider_process_omits_absent_final_message_from_archive_and_manifest() {
        let fixture = ProcessFixture::new(ProcessProviderBehavior::CompleteWithoutFinalMessage);
        let execution = execute_with_options(
            fixture.bound(),
            ProcessOptions::injected(64 * 1024 * 1024, mpsc::channel().1),
        )
        .unwrap();
        let result = execution.result;

        assert_eq!(result.final_message_sha256, None);
        assert!(!fixture.raw.join("final_message.bin").exists());
        let manifest = fs::read_to_string(fixture.raw.join("raw_members.tsv")).unwrap();
        assert!(!manifest.contains("final_message.bin"));

        let archive_file = File::open(fixture.raw.join("raw_traecli_run.tar.gz")).unwrap();
        let mut archive = Archive::new(GzDecoder::new(archive_file));
        let names = archive
            .entries()
            .unwrap()
            .map(|entry| entry.unwrap().path().unwrap().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [PathBuf::from("stderr.bin"), PathBuf::from("stdout.jsonl"),]
        );
    }

    fn assert_archive(raw: &Path) {
        let archive_file = File::open(raw.join("raw_traecli_run.tar.gz")).unwrap();
        let decoder = GzDecoder::new(archive_file);
        let gzip = decoder.header().unwrap();
        assert_eq!(gzip.mtime(), 0);
        assert_eq!(gzip.operating_system(), 255);
        assert_eq!(gzip.extra(), None);
        assert_eq!(gzip.filename(), None);
        assert_eq!(gzip.comment(), None);
        let mut archive = Archive::new(decoder);
        let mut names = Vec::new();
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let header = entry.header().clone();
            let name = entry.path().unwrap().into_owned();
            names.push(name.clone());
            assert_eq!(header.mtime().unwrap(), 0);
            assert_eq!(header.uid().unwrap(), 0);
            assert_eq!(header.gid().unwrap(), 0);
            assert_eq!(header.mode().unwrap(), 0o644);
            assert_eq!(header.username().unwrap(), None);
            assert_eq!(header.groupname().unwrap(), None);
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            let expected = match name.to_str().unwrap() {
                "final_message.bin" => PROCESS_FINAL_MESSAGE,
                "stderr.bin" => PROCESS_STDERR,
                "stdout.jsonl" => PROCESS_STDOUT,
                other => panic!("unexpected archive member {other}"),
            };
            assert_eq!(bytes, expected);
        }
        assert_eq!(
            names,
            [
                PathBuf::from("final_message.bin"),
                PathBuf::from("stderr.bin"),
                PathBuf::from("stdout.jsonl"),
            ]
        );
    }

    fn digest(bytes: &[u8]) -> String {
        format!("sha256:{:x}", Sha256::digest(bytes))
    }

    fn assert_process_gone(process: i32) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let result = unsafe { libc::kill(process, 0) };
            if result == -1 && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("provider descendant {process} survived process-group termination");
    }

    #[derive(Clone, Default)]
    struct CapturedWriter {
        bytes: Arc<Mutex<Vec<u8>>>,
    }

    impl CapturedWriter {
        fn bytes(&self) -> Vec<u8> {
            self.bytes.lock().unwrap().clone()
        }
    }

    impl Write for CapturedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct DropTrackedWriter {
        dropped: Arc<AtomicUsize>,
    }

    impl DropTrackedWriter {
        fn new(dropped: Arc<AtomicUsize>) -> Self {
            Self { dropped }
        }
    }

    impl Write for DropTrackedWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Drop for DropTrackedWriter {
        fn drop(&mut self) {
            self.dropped.fetch_add(1, Ordering::Release);
        }
    }

    struct BlockingWriter {
        entered: Option<mpsc::Sender<()>>,
        release: mpsc::Receiver<()>,
    }

    impl Write for BlockingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if let Some(entered) = self.entered.take() {
                entered.send(()).unwrap();
                let _ = self.release.recv();
            }
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct ProcessFixture {
        _temporary: TempDir,
        provider: ProcessProviderFixture,
        repository: PathBuf,
        raw: PathBuf,
        invocation: crate::context_control::provider::ProviderInvocation,
        raw_path: EpisodeProviderRawPath,
    }

    impl ProcessFixture {
        fn new(behavior: ProcessProviderBehavior) -> Self {
            let temporary = TempDir::new().unwrap();
            let root = fs::canonicalize(temporary.path()).unwrap();
            let repository = root.join("repository");
            fs::create_dir(&repository).unwrap();
            fs::set_permissions(&repository, fs::Permissions::from_mode(0o700)).unwrap();
            initialize_repository(&repository);
            let repository_snapshot = RepositorySnapshot::capture(&repository).unwrap();
            let provider = ProcessProviderFixture::new(behavior);
            let capabilities = probe_snapshot(ProviderId::Trae, provider.executable()).unwrap();
            let context = context();
            let invocation = build_invocation(
                &capabilities,
                &repository,
                &context,
                PROCESS_TASK,
                &ProviderRunOptions::default(),
            )
            .unwrap();
            let relative = Path::new("repositories")
                .join(&repository_snapshot.repository_id)
                .join("episodes")
                .join(EPISODE_ID)
                .join("raw")
                .join("trae");
            let raw = root.join(&relative);
            fs::create_dir_all(&raw).unwrap();
            fs::set_permissions(&raw, fs::Permissions::from_mode(0o700)).unwrap();
            let raw_path =
                EpisodeProviderRawPath::new(ProviderId::Trae, relative, raw.clone()).unwrap();
            Self {
                _temporary: temporary,
                provider,
                repository,
                raw,
                invocation,
                raw_path,
            }
        }

        fn bound(&self) -> BoundProviderInvocation {
            let directory: OwnedFd = File::open(&self.raw).unwrap().into();
            let authority =
                EpisodeRawDirectoryAuthority::new(self.raw_path.clone(), directory).unwrap();
            self.invocation
                .materialize(&self.raw_path)
                .unwrap()
                .bind(authority)
                .unwrap()
        }
    }

    fn initialize_repository(repository: &Path) {
        for arguments in [
            &["init", "--quiet"][..],
            &["config", "user.email", "harp-process@example.invalid"],
            &["config", "user.name", "Harp Process Test"],
        ] {
            assert!(std::process::Command::new("git")
                .args(arguments)
                .current_dir(repository)
                .status()
                .unwrap()
                .success());
        }
        fs::write(
            repository.join("tracked.txt"),
            b"provider process fixture\n",
        )
        .unwrap();
        assert!(std::process::Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(repository)
            .status()
            .unwrap()
            .success());
        assert!(std::process::Command::new("git")
            .args(["commit", "--quiet", "-m", "fixture"])
            .current_dir(repository)
            .status()
            .unwrap()
            .success());
    }

    fn context() -> ContextBundle {
        ContextBundle {
            schema_version: CONTEXT_BUNDLE_SCHEMA.to_owned(),
            release_id: "sha256-1111111111111111111111111111111111111111111111111111111111111111"
                .to_owned(),
            workflow: WorkflowId::GeneralCoding,
            repository_invariants: Vec::new(),
            workflow_steps: Vec::new(),
            relevant_patterns: Vec::new(),
            anti_patterns: Vec::new(),
            verification_expectations: Vec::new(),
            selected_item_ids: Vec::new(),
            rejected_items: Vec::new(),
            routing_trace: RouteDecision {
                selected: WorkflowId::GeneralCoding,
                explicit: true,
                considered: Vec::new(),
            },
            estimated_tokens: 8,
            rendered_context_sha256: sha256_hex(PROCESS_CONTEXT.as_bytes()),
            rendered_markdown: PROCESS_CONTEXT.to_owned(),
        }
    }
}
