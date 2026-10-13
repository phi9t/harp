//! Independent, authenticated effect owner. Records identify effects, never schedule team work.
//! A restarted supervisor refuses to reconstruct ownership from PIDs or replay uncertain starts.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File, OpenOptions},
    io::Write,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    os::unix::{
        fs::{FileTypeExt, OpenOptionsExt, PermissionsExt},
        net::{UnixListener, UnixStream},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use uuid::Uuid;

type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub project: String,
    pub attempt: String,
    pub operation: String,
    pub scope_epoch: i64,
    pub base: String,
    pub brief: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    pub binding: Binding,
    pub handle: String,
    pub incarnation: String,
    pub boot_id: String,
    pub pid: Option<u32>,
    pub start_ticks: Option<String>,
    pub namespace_init: Option<u32>,
    pub init_start_ticks: Option<String>,
    pub pid_namespace: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_store: Option<String>,
    pub state: String,
    pub outcome: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    root: PathBuf,
    profile: Profile,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    executable: String,
    args: Vec<String>,
    codex: Option<crate::native_profile::NativeProfile>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    token: String,
    action: String,
    binding: Binding,
}

/// Trusted workspace seam for the private Git backend. These are newly created, private,
/// controller-generated attempt directories, never caller-selected host mounts. A Git preparer
/// may populate `workspace` before dispatch, but must preserve the same isolation contract.
pub struct Workspace {
    pub workspace: PathBuf,
    pub home: PathBuf,
    pub scratch: PathBuf,
}
pub trait WorkspacePreparer {
    fn prepare(&self, root: &Path, binding: &Binding) -> Result<Workspace>;
}
pub struct EmptyWorkspace;
impl WorkspacePreparer for EmptyWorkspace {
    fn prepare(&self, root: &Path, binding: &Binding) -> Result<Workspace> {
        prepare_workspace(root, &binding.attempt)
    }
}
pub fn prepare_workspace(root: &Path, attempt: &str) -> Result<Workspace> {
    valid_id(attempt)?;
    canonical_directory(root)?;
    let attempt_root = root.join(attempt);
    fs::create_dir(&attempt_root).map_err(|e| format!("new attempt root: {e}"))?;
    private_directory(&attempt_root)?;
    let paths = Workspace {
        workspace: attempt_root.join("workspace"),
        home: attempt_root.join("home"),
        scratch: attempt_root.join("scratch"),
    };
    for path in [&paths.workspace, &paths.home, &paths.scratch] {
        fs::create_dir(path).map_err(|e| e.to_string())?;
        private_directory(path)?;
    }
    Ok(paths)
}
fn valid_id(value: &str) -> Result<()> {
    if Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value) {
        Ok(())
    } else {
        Err("generated canonical UUID required".into())
    }
}
fn canonical_directory(path: &Path) -> Result<()> {
    if !path.is_absolute()
        || fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        || path.canonicalize().map_err(|e| e.to_string())? != path
    {
        return Err("directory aliases/symlinks refused".into());
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<()> {
    canonical_directory(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())
}
fn private_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| e.to_string())
}
fn nix_store_package(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    if components.next()?.as_os_str() != "/" {
        return None;
    }
    if components.next()?.as_os_str() != "nix" {
        return None;
    }
    if components.next()?.as_os_str() != "store" {
        return None;
    }
    let package = components.next()?.as_os_str();
    Some(Path::new("/nix").join("store").join(package))
}
fn executable_interpreter(binary: &Path) -> Result<Option<PathBuf>> {
    let bytes = fs::read(binary).map_err(|e| e.to_string())?;
    let mut offset = 0;
    while let Some(relative) = bytes[offset..].iter().position(|byte| *byte == b'/') {
        let start = offset + relative;
        let end = bytes[start..]
            .iter()
            .position(|byte| *byte == 0)
            .map(|end| start + end)
            .unwrap_or(bytes.len());
        let candidate = &bytes[start..end];
        offset = start + 1;
        if !candidate
            .windows(b"ld-linux".len())
            .any(|w| w == b"ld-linux")
        {
            continue;
        }
        if let Ok(text) = std::str::from_utf8(candidate) {
            let path = PathBuf::from(text);
            if path.is_absolute() && path.exists() {
                return Ok(Some(path));
            }
        }
    }
    Ok(None)
}
fn add_host_runtime_mount(mounts: &mut BTreeSet<PathBuf>, path: PathBuf) -> Result<()> {
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    if path.starts_with("/usr") || path.starts_with("/lib") || path.starts_with("/lib64") {
        return Ok(());
    }
    if let Some(package) = nix_store_package(&path) {
        mounts.insert(package);
    } else {
        mounts.insert(path);
    }
    Ok(())
}
fn add_gate_runtime_mounts(command: &mut Command, gate_executable: &Path) -> Result<()> {
    let mut mounts = BTreeSet::new();
    if let Some(interpreter) = executable_interpreter(gate_executable)? {
        add_host_runtime_mount(&mut mounts, interpreter)?;
    }
    if Path::new("/usr/bin/ldd").is_file() {
        let output = Command::new("/usr/bin/ldd")
            .arg(gate_executable)
            .output()
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&output.stdout);
        for token in text.split_whitespace() {
            let path = token.trim_end_matches([',', ':', ';']);
            if !path.starts_with('/') {
                continue;
            }
            let path = PathBuf::from(path);
            if path.is_file() {
                add_host_runtime_mount(&mut mounts, path)?;
            }
        }
    }
    for path in mounts {
        command.arg("--ro-bind").arg(&path).arg(&path);
    }
    Ok(())
}
fn persist(root: &Path, execution: &Execution) -> Result<()> {
    let temporary = root.join(format!(".{}.tmp", execution.binding.operation));
    let mut file = private_file(&temporary)?;
    file.set_len(0).map_err(|e| e.to_string())?;
    file.write_all(
        serde_json::to_string(execution)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(
        &temporary,
        root.join(format!("{}.json", execution.binding.operation)),
    )
    .map_err(|e| e.to_string())?;
    File::open(root)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
fn process_identity(pid: u32) -> Result<(u32, String)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).map_err(|e| e.to_string())?;
    let (_, tail) = stat
        .rsplit_once(") ")
        .ok_or("invalid kernel process identity")?;
    let fields: Vec<_> = tail.split_whitespace().collect();
    let parent = fields
        .get(1)
        .ok_or("missing process parent")?
        .parse::<u32>()
        .map_err(|e| e.to_string())?;
    let start = fields
        .get(19)
        .ok_or("missing process start identity")?
        .to_string();
    Ok((parent, start))
}
fn start_ticks(pid: u32) -> Result<String> {
    Ok(process_identity(pid)?.1)
}
// A pidfd is provisional until kernel identity and ownership lineage are qualified.
// Never signal it on a failed check: its process may belong to someone else.
fn qualify_init(
    fd: &OwnedFd,
    init: u32,
    namespace: u64,
    monitor: u32,
    monitor_start: &str,
) -> Result<String> {
    let fdinfo = fs::read_to_string(format!("/proc/self/fdinfo/{}", fd.as_raw_fd()))
        .map_err(|e| e.to_string())?;
    let target = fdinfo
        .lines()
        .find_map(|line| line.strip_prefix("Pid:"))
        .ok_or("pidfd kernel target missing")?
        .trim();
    if target != init.to_string() {
        return Err("provisional pidfd kernel target mismatch".into());
    }
    if fs::read_link(format!("/proc/{init}/ns/pid"))
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        != format!("pid:[{namespace}]")
    {
        return Err("namespace kernel identity mismatch".into());
    }
    let status = fs::read_to_string(format!("/proc/{init}/status")).map_err(|e| e.to_string())?;
    if status
        .lines()
        .find_map(|line| line.strip_prefix("NSpid:"))
        .and_then(|ids| ids.split_whitespace().last())
        != Some("1")
    {
        return Err("namespace init must be PID1".into());
    }
    let (parent, init_start) = process_identity(init)?;
    if parent != monitor || start_ticks(monitor)? != monitor_start {
        return Err("namespace init owned monitor lineage mismatch".into());
    }
    // If init exited/PID was reused during proc sampling, this exact process fd is ready.
    // Keep the still-live descriptor pinned before granting cancellation authority.
    if pidfd_terminated(fd)? {
        return Err("namespace init terminated before identity qualification".into());
    }
    Ok(init_start)
}
fn pidfd_open(pid: u32) -> Result<OwnedFd> {
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
    if fd < 0 {
        return Err(format!(
            "pidfd ownership: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}
fn pidfd_terminated(fd: &OwnedFd) -> Result<bool> {
    let mut poll = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    if unsafe { libc::poll(&mut poll, 1, 0) } < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    if poll.revents & libc::POLLNVAL != 0 {
        return Err("invalid retained pidfd".into());
    }
    Ok(poll.revents & libc::POLLIN != 0)
}
fn duplicate_descriptor(fd: i32) -> Result<OwnedFd> {
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 10) };
    if duplicate < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
}
struct OwnedExecution {
    child: Child,
    namespace_init: Option<OwnedFd>,
    gate: Option<UnixStream>,
}
impl OwnedExecution {
    fn terminate(&mut self) -> Result<()> {
        // Closing an unreleased gate prevents workload exec even on setup failure.
        self.gate.take();
        if let Some(init) = &self.namespace_init {
            let result = unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    init.as_raw_fd(),
                    libc::SIGKILL,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                )
            };
            if result < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err(std::io::Error::last_os_error().to_string());
            }
        }
        // Child ownership prevents PID reuse until this exact child is reaped.
        if self.child.try_wait().map_err(|e| e.to_string())?.is_none() {
            self.child.kill().map_err(|e| e.to_string())?;
        }
        let deadline = Instant::now() + Duration::from_millis(300);
        loop {
            if self.child.try_wait().map_err(|e| e.to_string())?.is_some() {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err("owned monitor termination unconfirmed before deadline".into());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeStoreReceipt {
    execution: String,
    generation: String,
    filesystem: String,
}
struct Supervisor {
    config: Config,
    incarnation: String,
    boot_id: String,
    records: HashMap<String, Execution>,
    children: HashMap<String, OwnedExecution>,
    _ownership: File,
}
impl Supervisor {
    fn open(config: &Path) -> Result<Self> {
        let config: Config = serde_json::from_slice(&fs::read(config).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        private_directory(&config.root)?;
        if config.root.starts_with("/usr") {
            return Err("execution roots must be outside the toolchain mount".into());
        }
        if !config.profile.executable.starts_with("/usr/bin/")
            || config.profile.executable.contains("/../")
            || !Path::new(&config.profile.executable).is_file()
        {
            return Err("profile executable must be an operator-configured /usr/bin tool".into());
        }
        if let Some(native) = &config.profile.codex {
            native.validate()?;
        }
        let ownership = private_file(&config.root.join("supervisor-lock"))?;
        ownership
            .try_lock()
            .map_err(|e| format!("supervisor ownership unavailable: {e}"))?;
        let mut records = HashMap::new();
        let mut native_receipts = Vec::new();
        for entry in fs::read_dir(&config.root).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "json") {
                if fs::symlink_metadata(&path)
                    .map_err(|e| e.to_string())?
                    .file_type()
                    .is_symlink()
                {
                    return Err("record symlink refused".into());
                }
                if let Some(operation) = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.strip_prefix("native-store-"))
                {
                    valid_id(operation)?;
                    let receipt: NativeStoreReceipt =
                        serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
                            .map_err(|e| e.to_string())?;
                    valid_id(&receipt.execution)?;
                    valid_id(&receipt.generation)?;
                    let parts: Vec<_> = receipt.filesystem.split(':').collect();
                    if parts.len() != 2
                        || parts.iter().any(|part| {
                            part.is_empty()
                                || !part.bytes().all(|b| b.is_ascii_digit())
                                || part.parse::<u64>().is_err()
                        })
                    {
                        return Err("native store filesystem identity malformed".into());
                    }
                    native_receipts.push((operation.to_owned(), receipt));
                    continue;
                }
                let mut execution: Execution =
                    serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                valid_id(&execution.binding.operation)?;
                valid_id(&execution.binding.attempt)?;
                if path.file_stem().and_then(|s| s.to_str()) != Some(&execution.binding.operation) {
                    return Err("record identity mismatch".into());
                }
                if execution.state != "exited" {
                    execution.state = "unknown".into();
                    execution.outcome =
                        Some("supervisor ownership lost; PID reconstruction refused".into());
                    persist(&config.root, &execution)?;
                }
                records.insert(execution.binding.operation.clone(), execution);
            }
        }
        for (operation, receipt) in native_receipts {
            let execution = records
                .get(&operation)
                .ok_or("native store receipt execution absent")?;
            if execution.handle != receipt.execution
                || execution.native_store.as_deref() != Some(&receipt.generation)
            {
                return Err("native store receipt execution/generation mismatch".into());
            }
        }
        Ok(Self {
            config,
            incarnation: Uuid::new_v4().to_string(),
            boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .map_err(|e| e.to_string())?
                .trim()
                .into(),
            records,
            children: HashMap::new(),
            _ownership: ownership,
        })
    }
    fn observe(&mut self, operation: &str) -> Result<()> {
        let Some(owned) = self.children.get_mut(operation) else {
            return Ok(());
        };
        let execution = self
            .records
            .get_mut(operation)
            .ok_or("missing owned record")?;
        if let Some(status) = owned.child.try_wait().map_err(|e| e.to_string())? {
            let deadline = Instant::now() + Duration::from_millis(300);
            let Some(init) = owned.namespace_init.as_ref() else {
                execution.state = "unknown".into();
                execution.outcome =
                    Some("monitor exited without retained whole-process identity".into());
                persist(&self.config.root, execution)?;
                return Ok(());
            };
            while !pidfd_terminated(init)? {
                if Instant::now() >= deadline {
                    execution.state = "unknown".into();
                    execution.outcome =
                        Some("monitor exited; namespace process termination unconfirmed".into());
                    persist(&self.config.root, execution)?;
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            execution.state = "exited".into();
            execution.outcome = Some(format!(
                "owned namespace process pidfd confirmed termination: {status}"
            ));
            persist(&self.config.root, execution)?;
            self.children.remove(operation);
        } else if execution.incarnation != self.incarnation
            || execution.boot_id != self.boot_id
            || execution.pid != Some(owned.child.id())
            || execution.start_ticks.as_ref() != start_ticks(owned.child.id()).ok().as_ref()
        {
            execution.state = "unknown".into();
            execution.outcome = Some("live kernel ownership identity mismatch".into());
            persist(&self.config.root, execution)?;
        }
        Ok(())
    }
    fn handle(&mut self, request: Request, preparer: &dyn WorkspacePreparer) -> Result<Value> {
        let binding = request.binding;
        valid_id(&binding.operation)?;
        valid_id(&binding.attempt)?;
        if binding.project.is_empty() || binding.scope_epoch < 1 {
            return Err("invalid binding".into());
        }
        if let Some(record) = self.records.get(&binding.operation)
            && record.binding != binding
        {
            return Err("operation binding mismatch".into());
        }
        self.observe(&binding.operation)?;
        match request.action.as_str() {
            "lookup" => Ok(self
                .records
                .get(&binding.operation)
                .map(|e| json!(e))
                .unwrap_or(Value::Null)),
            "native_lookup" => {
                let record = self
                    .records
                    .get(&binding.operation)
                    .ok_or("native owned execution missing")?;
                let owned = self
                    .children
                    .get(&binding.operation)
                    .ok_or("native ownership lost")?;
                if record.state != "running"
                    || record.incarnation != self.incarnation
                    || pidfd_terminated(
                        owned
                            .namespace_init
                            .as_ref()
                            .ok_or("native init pidfd missing")?,
                    )?
                {
                    return Err("native execution not qualified live".into());
                }
                let profile = self
                    .config
                    .profile
                    .codex
                    .as_ref()
                    .ok_or("native profile not configured")?;
                let init = record
                    .namespace_init
                    .ok_or("native init identity missing")?;
                let info: Value = serde_json::from_slice(
                    &fs::read(
                        self.config
                            .root
                            .join(&binding.attempt)
                            .join("scratch/native-runtime.json"),
                    )
                    .map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let ns_pid = info["pid"].as_u64().ok_or("native namespace PID missing")?;
                let mut qualified = None;
                let children = fs::read_to_string(format!("/proc/{init}/task/{init}/children"))
                    .map_err(|e| e.to_string())?;
                for child in children.split_whitespace() {
                    let pid = child.parse::<u32>().map_err(|e| e.to_string())?;
                    let fd = match pidfd_open(pid) {
                        Ok(fd) => fd,
                        Err(_) => continue,
                    };
                    let status = fs::read_to_string(format!("/proc/{pid}/status"))
                        .map_err(|e| e.to_string())?;
                    if status
                        .lines()
                        .find_map(|line| line.strip_prefix("NSpid:"))
                        .and_then(|ids| ids.split_whitespace().last())
                        .and_then(|id| id.parse::<u64>().ok())
                        != Some(ns_pid)
                    {
                        continue;
                    }
                    let (parent, start) = process_identity(pid)?;
                    if parent != init || info["start_ticks"].as_str() != Some(&start) {
                        return Err("native child lineage/start mismatch".into());
                    }
                    if fs::read_link(format!("/proc/{pid}/ns/pid"))
                        .map_err(|e| e.to_string())?
                        .to_string_lossy()
                        != format!(
                            "pid:[{}]",
                            record.pid_namespace.ok_or("native namespace missing")?
                        )
                    {
                        return Err("native child namespace mismatch".into());
                    }
                    use std::os::unix::fs::MetadataExt;
                    let exe =
                        fs::metadata(format!("/proc/{pid}/exe")).map_err(|e| e.to_string())?;
                    let expected = fs::metadata(format!("/proc/{init}/root/run/codex/codex"))
                        .map_err(|e| e.to_string())?;
                    if exe.dev() != expected.dev() || exe.ino() != expected.ino() {
                        return Err("native pinned executable mismatch".into());
                    }
                    let command =
                        fs::read(format!("/proc/{pid}/cmdline")).map_err(|e| e.to_string())?;
                    let words: Vec<_> = command.split(|byte| *byte == 0).collect();
                    let method = if profile.kind == "app" {
                        b"app-server".as_slice()
                    } else {
                        b"exec-server".as_slice()
                    };
                    if words.get(1).copied() != Some(method) {
                        return Err("native service method mismatch".into());
                    }
                    for cap in ["CapEff:", "CapPrm:", "CapInh:", "CapAmb:"] {
                        if status
                            .lines()
                            .find_map(|line| line.strip_prefix(cap))
                            .map(str::trim)
                            != Some("0000000000000000")
                        {
                            return Err("native capabilities not dropped".into());
                        }
                    }
                    if status
                        .lines()
                        .find_map(|line| line.strip_prefix("NoNewPrivs:"))
                        .map(str::trim)
                        != Some("1")
                    {
                        return Err("native privilege growth refused".into());
                    }
                    if pidfd_terminated(&fd)? {
                        return Err("native child exited during qualification".into());
                    }
                    let workspace = fs::metadata(format!("/proc/{init}/root/workspace"))
                        .map_err(|e| e.to_string())?;
                    let store = fs::metadata(format!("/proc/{init}/root/home/worker/.codex"))
                        .map_err(|e| e.to_string())?;
                    let store_identity = format!("{}:{}", store.dev(), store.ino());
                    let store_receipt = self
                        .config
                        .root
                        .join(format!("native-store-{}.json", binding.operation));
                    let expected = json!({"execution":record.handle,"generation":record.native_store,"filesystem":store_identity});
                    if store_receipt.exists() {
                        let mut receipt = private_file(&store_receipt)?;
                        use std::io::Read;
                        let mut contents = String::new();
                        receipt
                            .read_to_string(&mut contents)
                            .map_err(|e| e.to_string())?;
                        let retained: Value =
                            serde_json::from_str(&contents).map_err(|e| e.to_string())?;
                        if retained != expected {
                            return Err(
                                "native persistent store filesystem identity changed".into()
                            );
                        }
                    } else {
                        let mut receipt = OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .mode(0o600)
                            .custom_flags(libc::O_NOFOLLOW)
                            .open(&store_receipt)
                            .map_err(|e| e.to_string())?;
                        receipt
                            .write_all(expected.to_string().as_bytes())
                            .map_err(|e| e.to_string())?;
                        receipt.sync_all().map_err(|e| e.to_string())?;
                        File::open(&self.config.root)
                            .and_then(|f| f.sync_all())
                            .map_err(|e| e.to_string())?;
                    }
                    if pidfd_terminated(&fd)? {
                        return Err("native child exited during store qualification".into());
                    }
                    #[cfg(feature = "fault-injection")]
                    if profile.kind == "exec"
                        && binding.brief.starts_with("lead-authority:")
                        && let Ok(path) = std::env::var("CORENIUS_TEST_PAUSE_EXEC_READY")
                        && Path::new(&path).is_file()
                    {
                        fs::remove_file(&path).map_err(|e| e.to_string())?;
                        if unsafe {
                            libc::syscall(
                                libc::SYS_pidfd_send_signal,
                                fd.as_raw_fd(),
                                libc::SIGSTOP,
                                std::ptr::null::<libc::siginfo_t>(),
                                0,
                            )
                        } != 0
                        {
                            return Err(std::io::Error::last_os_error().to_string());
                        }
                        fs::write(format!("{path}.observed"), json!({"runtime_pid":pid,"runtime_start":start,"pid_namespace":record.pid_namespace,"attempt":binding.attempt,"execution":record.handle}).to_string()).map_err(|e|e.to_string())?;
                    }
                    let transport = profile.transport_root.join(&binding.attempt);
                    use sha2::{Digest, Sha256};
                    let incarnation = format!(
                        "{:x}",
                        Sha256::digest(format!(
                            "{}:{}:{pid}:{start}",
                            record.boot_id, record.handle
                        ))
                    );
                    qualified = Some(
                        json!({"kind":profile.kind,"runtime_pid":pid,"runtime_start":start,"incarnation":incarnation,
                        "store_id":record.native_store,"store_identity":store_identity,"workspace_identity":format!("{}:{}",workspace.dev(),workspace.ino()),
                        "workspace_generation":record.handle,"pid_namespace":record.pid_namespace,
                        "endpoint":transport.join("native.sock"),"control_endpoint":transport.join("control.sock"),
                        "auth_ref":format!("native:{}",record.handle),"controlplane_cwd":"/workspace",
                        "pins":profile,"own_executor_only":profile.own_executor_only}),
                    );
                    break;
                }
                let runtime = qualified.ok_or("native child not yet qualified")?;
                let capability = fs::read_to_string(
                    self.config
                        .root
                        .join("native-authority")
                        .join(&binding.attempt),
                )
                .map_err(|e| e.to_string())?;
                Ok(json!({"execution":record,"runtime":runtime,"capability":capability}))
            }
            "start" => {
                if let Some(record) = self.records.get(&binding.operation) {
                    return Ok(json!(record));
                }
                if self
                    .records
                    .values()
                    .any(|e| e.binding.attempt == binding.attempt)
                {
                    return Err("attempt already has an execution identity".into());
                }
                let mut execution = Execution {
                    binding: binding.clone(),
                    handle: Uuid::new_v4().to_string(),
                    incarnation: self.incarnation.clone(),
                    boot_id: self.boot_id.clone(),
                    pid: None,
                    start_ticks: None,
                    namespace_init: None,
                    init_start_ticks: None,
                    pid_namespace: None,
                    native_store: self
                        .config
                        .profile
                        .codex
                        .as_ref()
                        .map(|_| Uuid::new_v4().to_string()),
                    state: "unknown".into(),
                    outcome: Some("start intent persisted; effect unconfirmed".into()),
                };
                // The uncertain intent exists before workspace creation or process effects.
                persist(&self.config.root, &execution)?;
                self.records
                    .insert(binding.operation.clone(), execution.clone());
                let workspace = preparer.prepare(&self.config.root, &binding)?;
                for (path, name) in [
                    (&workspace.workspace, "workspace"),
                    (&workspace.home, "home"),
                    (&workspace.scratch, "scratch"),
                ] {
                    if path != &self.config.root.join(&binding.attempt).join(name) {
                        return Err("workspace preparer returned an alias/unscoped mount".into());
                    }
                    private_directory(path)?;
                }
                let log =
                    private_file(&self.config.root.join(&binding.attempt).join("output.log"))?;
                let info_path = self
                    .config
                    .root
                    .join(&binding.attempt)
                    .join("namespace-info.json");
                let info = private_file(&info_path)?;
                let (gate_read, gate_write) = UnixStream::pair().map_err(|e| e.to_string())?;
                let info_descriptor = duplicate_descriptor(info.as_raw_fd())?;
                let info_fd = info_descriptor.as_raw_fd();
                let gate_executable = std::env::current_exe()
                    .map_err(|e| e.to_string())?
                    .parent()
                    .ok_or("supervisor executable directory missing")?
                    .join("corenius-attempt-gate");
                let mut command = Command::new("/usr/bin/bwrap");
                // The private info descriptor reaches bwrap, which closes it before exec.
                unsafe {
                    command.pre_exec(move || {
                        if libc::dup2(info_fd, 4) == -1 {
                            return Err(std::io::Error::last_os_error());
                        }
                        Ok(())
                    });
                }
                command
                    .env_clear()
                    .process_group(0)
                    .args([
                        "--unshare-all",
                        "--new-session",
                        "--die-with-parent",
                        "--as-pid-1",
                        "--clearenv",
                        "--ro-bind",
                        "/usr",
                        "/usr",
                        "--symlink",
                        "usr/bin",
                        "/bin",
                        "--symlink",
                        "usr/lib",
                        "/lib",
                        "--symlink",
                        "usr/lib64",
                        "/lib64",
                        "--proc",
                        "/proc",
                        "--dev",
                        "/dev",
                        "--tmpfs",
                        "/run/corenius-tmp",
                        "--setenv",
                        "TMPDIR",
                        "/run/corenius-tmp",
                        "--dir",
                        "/home",
                        "--setenv",
                        "HOME",
                        "/home/worker",
                        "--setenv",
                        "PATH",
                        "/usr/bin:/bin",
                    ])
                    .arg("--bind")
                    .arg(&workspace.home)
                    .arg("/home/worker")
                    .arg("--bind")
                    .arg(&workspace.workspace)
                    .arg("/workspace")
                    .arg("--bind")
                    .arg(&workspace.scratch)
                    .arg("/scratch")
                    .arg("--ro-bind")
                    .arg(&gate_executable)
                    .arg("/run/attempt-gate")
                    .args(["--info-fd", "4"]);
                add_gate_runtime_mounts(&mut command, &gate_executable)?;
                let _native_mounts = if let Some(native) = &self.config.profile.codex {
                    {
                        let secrets = self.config.root.join("native-authority");
                        if !secrets.exists() {
                            fs::create_dir(&secrets).map_err(|e| e.to_string())?;
                        }
                        private_directory(&secrets)?;
                        let token = Uuid::new_v4().to_string() + &Uuid::new_v4().to_string();
                        let mut secret = private_file(&secrets.join(&binding.attempt))?;
                        secret.set_len(0).map_err(|e| e.to_string())?;
                        secret
                            .write_all(token.as_bytes())
                            .map_err(|e| e.to_string())?;
                        secret.sync_all().map_err(|e| e.to_string())?;
                        use sha2::{Digest, Sha256};
                        Some(native.mount(
                            &mut command,
                            &binding.attempt,
                            &format!("{:x}", Sha256::digest(token.as_bytes())),
                        )?)
                    }
                } else {
                    None
                };
                command
                    .args(["--chdir", "/workspace", "--", "/run/attempt-gate"])
                    .arg(&self.config.profile.executable)
                    .args(&self.config.profile.args)
                    .stdin(Stdio::from(OwnedFd::from(gate_read)))
                    .stdout(log.try_clone().map_err(|e| e.to_string())?)
                    .stderr(log);
                let child = command
                    .spawn()
                    .map_err(|e| format!("bwrap dispatch: {e}"))?;
                execution.pid = Some(child.id());
                self.children.insert(
                    binding.operation.clone(),
                    OwnedExecution {
                        child,
                        namespace_init: None,
                        gate: Some(gate_write),
                    },
                );
                let setup = (|| -> Result<()> {
                    execution.start_ticks = Some(start_ticks(execution.pid.unwrap())?);
                    #[cfg(feature = "fault-injection")]
                    {
                        if let Ok(delay) = std::env::var("CORENIUS_TEST_IDENTITY_DELAY_MS") {
                            fs::write(
                                self.config
                                    .root
                                    .join(&binding.attempt)
                                    .join("sampling-delay"),
                                "before sampling",
                            )
                            .map_err(|e| e.to_string())?;
                            std::thread::sleep(Duration::from_millis(
                                delay.parse().map_err(|_| "bad test delay")?,
                            ));
                        }
                        if std::env::var("CORENIUS_TEST_SETUP_ERROR").as_deref() == Ok("1") {
                            std::thread::sleep(Duration::from_millis(80));
                            return Err("injected postspawn setup failure".into());
                        }
                    }
                    // bwrap's private setup acknowledgement identifies the actual namespace init,
                    // not only its outer monitor. Validate it against the independent kernel view.
                    let deadline = Instant::now() + Duration::from_millis(300);
                    let info: Value = loop {
                        if let Ok(bytes) = fs::read(&info_path)
                            && let Ok(value) = serde_json::from_slice::<Value>(&bytes)
                        {
                            break value;
                        }
                        if Instant::now() >= deadline {
                            return Err("namespace setup acknowledgement missing".into());
                        }
                        std::thread::sleep(Duration::from_millis(5));
                    };
                    #[cfg(feature = "fault-injection")]
                    let info = match std::env::var("CORENIUS_TEST_INIT_ACK") {
                        Ok(value) => {
                            serde_json::from_str::<Value>(&value).map_err(|e| e.to_string())?
                        }
                        Err(_) => info,
                    };
                    let init =
                        u32::try_from(info["child-pid"].as_u64().ok_or("namespace init missing")?)
                            .map_err(|e| e.to_string())?;
                    let namespace = info["pid-namespace"]
                        .as_u64()
                        .ok_or("PID namespace missing")?;
                    #[cfg(feature = "fault-injection")]
                    let descriptor_target = match std::env::var("CORENIUS_TEST_INIT_PIDFD_TARGET") {
                        Ok(value) => value.parse::<u32>().map_err(|e| e.to_string())?,
                        Err(_) => init,
                    };
                    #[cfg(not(feature = "fault-injection"))]
                    let descriptor_target = init;
                    let init_pidfd = pidfd_open(descriptor_target)?;
                    let owned = self
                        .children
                        .get_mut(&binding.operation)
                        .ok_or("owned child missing")?;
                    if owned.child.try_wait().map_err(|e| e.to_string())?.is_some() {
                        return Err("owned monitor exited before identity qualification".into());
                    }
                    let init_start = qualify_init(
                        &init_pidfd,
                        init,
                        namespace,
                        execution.pid.ok_or("owned monitor identity missing")?,
                        execution
                            .start_ticks
                            .as_deref()
                            .ok_or("owned monitor start missing")?,
                    )?;
                    self.children
                        .get_mut(&binding.operation)
                        .ok_or("owned child missing")?
                        .namespace_init = Some(init_pidfd);
                    execution.namespace_init = Some(init);
                    execution.pid_namespace = Some(namespace);
                    execution.init_start_ticks = Some(init_start);
                    execution.state = "running".into();
                    execution.outcome = None;
                    persist(&self.config.root, &execution)?;
                    self.records
                        .insert(binding.operation.clone(), execution.clone());
                    // Identity is durable before the PID1 gate execs the configured workload.
                    self.children
                        .get_mut(&binding.operation)
                        .ok_or("owned child missing")?
                        .gate
                        .as_mut()
                        .ok_or("dispatch gate missing")?
                        .write_all(b"R")
                        .map_err(|e| e.to_string())?;
                    self.children
                        .get_mut(&binding.operation)
                        .unwrap()
                        .gate
                        .take();
                    Ok(())
                })();
                if let Err(error) = setup {
                    execution.state = "unknown".into();
                    execution.outcome = Some(format!(
                        "setup failed; owned effect termination requested: {error}"
                    ));
                    // Preserve ownership even when durable record/identity setup failed.
                    self.records
                        .insert(binding.operation.clone(), execution.clone());
                    let cleanup = self
                        .children
                        .get_mut(&binding.operation)
                        .ok_or("owned child missing")?
                        .terminate();
                    persist(&self.config.root, &execution)?;
                    cleanup?;
                    self.observe(&binding.operation)?;
                    return Err(error);
                }
                Ok(json!(execution))
            }
            "cancel" => {
                if !self.records.contains_key(&binding.operation) {
                    // This is an identity-bound absence proof under the sole effect-owner lock.
                    // Persist a tombstone before replying, so a delayed start cannot overtake cancel.
                    let execution = Execution {
                        binding: binding.clone(),
                        handle: Uuid::new_v4().to_string(),
                        incarnation: self.incarnation.clone(),
                        boot_id: self.boot_id.clone(),
                        pid: None,
                        start_ticks: None,
                        namespace_init: None,
                        init_start_ticks: None,
                        pid_namespace: None,
                        native_store: None,
                        state: "exited".into(),
                        outcome: Some("cancel tombstone: effect never started".into()),
                    };
                    persist(&self.config.root, &execution)?;
                    self.records.insert(binding.operation.clone(), execution);
                } else if self.children.contains_key(&binding.operation) {
                    // Real owned Child/pidfd authority is independent of projected record state.
                    self.children
                        .get_mut(&binding.operation)
                        .unwrap()
                        .terminate()?;
                    self.observe(&binding.operation)?;
                }
                Ok(json!(self.records[&binding.operation]))
            }
            _ => Err("unknown supervisor action".into()),
        }
    }
}

/// Bounded private local transport. Neither socket nor its secret is mounted into attempts.
pub fn call(
    socket: &Path,
    token: &str,
    action: &str,
    binding: &Binding,
) -> Result<Option<Execution>> {
    let mut stream =
        UnixStream::connect(socket).map_err(|e| format!("supervisor unavailable: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    writeln!(
        stream,
        "{}",
        json!({"token":token,"action":action,"binding":binding})
    )
    .map_err(|e| e.to_string())?;
    let response = read_line(stream)?;
    if let Some(error) = response.get("error") {
        return Err(format!("supervisor: {error}"));
    }
    let execution: Option<Execution> = serde_json::from_value(
        response
            .get("ok")
            .ok_or("missing supervisor reply")?
            .clone(),
    )
    .map_err(|e| e.to_string())?;
    if let Some(e) = &execution {
        if &e.binding != binding
            || Uuid::parse_str(&e.handle).is_err()
            || Uuid::parse_str(&e.incarnation).is_err()
            || Uuid::parse_str(&e.boot_id).is_err()
            || !matches!(e.state.as_str(), "running" | "unknown" | "exited")
        {
            return Err("supervisor identity validation refused".into());
        }
        if e.state == "running"
            && (e.pid.is_none()
                || e.start_ticks
                    .as_ref()
                    .is_none_or(|s| s.parse::<u64>().is_err()))
        {
            return Err("supervisor kernel identity missing".into());
        }
    }
    Ok(execution)
}
pub fn native_lookup(socket: &Path, token: &str, binding: &Binding) -> Result<Value> {
    let mut stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    writeln!(
        stream,
        "{}",
        json!({"token":token,"action":"native_lookup","binding":binding})
    )
    .map_err(|e| e.to_string())?;
    let response = read_line(stream)?;
    if let Some(error) = response.get("error") {
        return Err(format!("native ownership lookup: {error}"));
    }
    let value = response.get("ok").ok_or("native lookup missing")?.clone();
    let execution: Execution =
        serde_json::from_value(value["execution"].clone()).map_err(|e| e.to_string())?;
    if execution.binding != *binding
        || execution.state != "running"
        || value["runtime"]["kind"].is_null()
        || value["capability"].as_str().is_none_or(|v| v.len() < 32)
    {
        return Err("native lookup identity refused".into());
    }
    Ok(value)
}
fn read_line(mut stream: UnixStream) -> Result<Value> {
    let bytes = crate::service::read_request(&mut stream, Instant::now() + Duration::from_secs(1))?
        .ok_or("empty supervisor request")?;
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
pub fn serve(config: impl AsRef<Path>, socket: impl AsRef<Path>, token: &str) -> Result<()> {
    serve_with_workspace(config, socket, token, &EmptyWorkspace)
}
pub fn serve_with_workspace(
    config: impl AsRef<Path>,
    socket: impl AsRef<Path>,
    token: &str,
    preparer: &dyn WorkspacePreparer,
) -> Result<()> {
    if token.len() < 12 {
        return Err("supervisor secret must have at least 12 bytes".into());
    }
    let mut supervisor = Supervisor::open(config.as_ref())?;
    let socket = socket.as_ref();
    if !socket.is_absolute() || socket.starts_with("/usr") {
        return Err("private supervisor socket outside toolchain mount required".into());
    }
    if let Ok(metadata) = fs::symlink_metadata(socket) {
        if !metadata.file_type().is_socket() {
            return Err("supervisor socket alias/non-socket refused".into());
        }
        match UnixStream::connect(socket) {
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                fs::remove_file(socket).map_err(|e| e.to_string())?
            }
            _ => return Err("supervisor socket already owned or ambiguous".into()),
        }
    }
    let listener = UnixListener::bind(socket).map_err(|e| e.to_string())?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    #[cfg(feature = "fault-injection")]
    let mut drop_first_lookup =
        std::env::var("CORENIUS_TEST_DROP_FIRST_LOOKUP").as_deref() == Ok("1");
    for stream in listener.incoming() {
        let mut stream = stream.map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(1)))
            .map_err(|e| e.to_string())?;
        #[cfg(feature = "fault-injection")]
        if drop_first_lookup {
            let value = read_line(stream.try_clone().map_err(|e| e.to_string())?);
            if let Ok(value) = value {
                let request: Request = serde_json::from_value(value).map_err(|e| e.to_string())?;
                if request.token == token && request.action == "lookup" {
                    drop_first_lookup = false;
                    continue;
                }
                let response = if request.token != token {
                    json!({"error":"supervisor authentication failed"})
                } else {
                    match supervisor.handle(request, preparer) {
                        Ok(value) => json!({"ok":value}),
                        Err(error) => json!({"error":error}),
                    }
                };
                let _ = writeln!(stream, "{response}");
                continue;
            } else {
                continue;
            }
        }
        let response = read_line(stream.try_clone().map_err(|e| e.to_string())?)
            .and_then(|value| serde_json::from_value::<Request>(value).map_err(|e| e.to_string()))
            .and_then(|request| {
                if request.token != token {
                    Err("supervisor authentication failed".into())
                } else {
                    supervisor.handle(request, preparer)
                }
            });
        let response = match response {
            Ok(value) => json!({"ok":value}),
            Err(error) => json!({"error":error}),
        };
        let _ = writeln!(stream, "{response}"); // A lost reply leaves the same durable operation identity.
    }
    Ok(())
}
