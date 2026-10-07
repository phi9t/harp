//! Private, complete Git repositories. Host Git only reads controller-owned stores;
//! worker metadata is inspected by fixed commands in a separate OS boundary.
use crate::{
    Result, commit_oid, hash,
    supervisor::{Binding, Workspace, WorkspacePreparer},
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::{
            fs::{MetadataExt, OpenOptionsExt},
            process::ExitStatusExt,
        },
    },
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub path: PathBuf,
    pub sha256: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub id: String,
    pub task: String,
    pub base: Option<String>,
    pub script: Tool,
    pub args: Vec<String>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifierRuntime {
    pub root: PathBuf,
    pub manifest: Tool,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub project: String,
    pub root: PathBuf,
    pub attempts_root: PathBuf,
    pub canonical: PathBuf,
    pub protected_ref: String,
    pub git: Tool,
    pub python: Tool,
    pub bubblewrap: Tool,
    pub verifier_runtime: Option<VerifierRuntime>,
    pub suites: Vec<Suite>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ownership {
    pub binding: Binding,
    pub workspace_identity: String,
    pub git_identity: String,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryState {
    pub commit: String,
    pub tree: String,
    pub refs: Vec<(String, String)>,
    pub status: String,
    pub capture_pid: u32,
    pub pid_namespace: String,
    #[serde(default)]
    pub inspection: Option<InspectionExecution>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct InspectionExecution {
    argv: Vec<String>,
    environment_cleared: bool,
    tools: Vec<Tool>,
    process: ProcessEvidence,
}
#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CleanupEntry {
    identity: String,
    kind: String,
    content: Option<String>,
}
fn cleanup_entries(root: &Path) -> Result<BTreeMap<String, CleanupEntry>> {
    directory(root)?;
    fn visit(
        root: &Path,
        path: &Path,
        entries: &mut BTreeMap<String, CleanupEntry>,
        remaining: &mut u64,
    ) -> Result<()> {
        if entries.len() >= 16384 {
            return Err("cleanup manifest entry bound exceeded".into());
        }
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        let relative = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("cleanup path encoding refused")?
            .to_owned();
        let (kind, content) = if metadata.is_dir() {
            ("directory", None)
        } else if metadata.is_file() {
            if metadata.len() > *remaining {
                return Err("cleanup manifest byte bound exceeded".into());
            }
            let bytes = read_regular(path, *remaining)?;
            *remaining -= bytes.len() as u64;
            ("regular", Some(digest(&bytes)))
        } else if metadata.file_type().is_symlink() {
            (
                "symlink",
                Some(
                    fs::read_link(path)
                        .map_err(|e| e.to_string())?
                        .to_str()
                        .ok_or("cleanup link encoding refused")?
                        .to_owned(),
                ),
            )
        } else {
            return Err("nonregular cleanup entry preserved".into());
        };
        entries.insert(
            relative,
            CleanupEntry {
                identity: format!("{}:{}", metadata.dev(), metadata.ino()),
                kind: kind.into(),
                content,
            },
        );
        if metadata.is_dir() {
            let mut children = fs::read_dir(path)
                .map_err(|e| e.to_string())?
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?;
            children.sort_by_key(|e| e.file_name());
            for child in children {
                visit(root, &child.path(), entries, remaining)?;
            }
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries, &mut (32 * 1024 * 1024))?;
    Ok(entries)
}
#[derive(Clone)]
pub struct GitBackend {
    pub policy: Policy,
    pub identity: String,
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// Retain the opened regular descriptor, refuse links/nonregular files and bound
/// allocation. No untrusted pathname is reopened between validation and reading.
pub fn read_regular(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| "retained regular file unavailable")?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > limit {
        return Err("retained file type/link/size refused".into());
    }
    let parent = path.parent().ok_or("retained parent missing")?;
    directory(parent)?;
    let named = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if named.dev() != metadata.dev() || named.ino() != metadata.ino() || !named.is_file() {
        return Err("retained file identity changed".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("retained read bound exceeded".into());
    }
    Ok(bytes)
}
#[derive(Deserialize)]
struct Inspection {
    state: RepositoryState,
    bundle_b64: Option<String>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeManifest {
    schema_version: String,
    entries: Vec<RuntimeManifestEntry>,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeManifestEntry {
    path: String,
    kind: String,
    sha256: Option<String>,
    bytes: Option<u64>,
    target: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RuntimeActualEntry {
    kind: String,
    sha256: Option<String>,
    bytes: Option<u64>,
    target: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct RuntimeEvidence {
    status: &'static str,
    root: PathBuf,
    manifest_path: PathBuf,
    manifest: String,
    entries: usize,
}

pub fn check_tool(tool: &Tool) -> Result<()> {
    if !tool.path.is_absolute()
        || digest(&fs::read(&tool.path).map_err(|e| e.to_string())?) != tool.sha256
    {
        return Err("pinned executable/suite identity changed".into());
    }
    Ok(())
}
pub fn directory(path: &Path) -> Result<()> {
    if !path.is_absolute()
        || path.canonicalize().map_err(|e| e.to_string())? != path
        || !fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .is_dir()
    {
        return Err("directory ownership alias refused".into());
    }
    Ok(())
}
fn manifest_relative(raw: &str) -> Result<PathBuf> {
    let path = Path::new(raw);
    if raw.is_empty() || path.is_absolute() {
        return Err("verifier runtime manifest path escaped snapshot".into());
    }
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => out.push(part),
            _ => return Err("verifier runtime manifest path escaped snapshot".into()),
        }
    }
    Ok(out)
}
fn link_target_inside(link: &Path, target: &str) -> Result<()> {
    let target_path = Path::new(target);
    if target.is_empty() || target_path.is_absolute() {
        return Err("verifier runtime symlink escaped snapshot".into());
    }
    let mut normalized = link
        .parent()
        .ok_or("verifier runtime symlink parent missing")?
        .to_path_buf();
    for component in target_path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::ParentDir => {
                return Err("verifier runtime symlink escaped snapshot".into());
            }
            _ => return Err("verifier runtime symlink escaped snapshot".into()),
        }
    }
    Ok(())
}
fn runtime_entries(root: &Path) -> Result<BTreeMap<String, RuntimeActualEntry>> {
    fn visit(
        root: &Path,
        path: &Path,
        entries: &mut BTreeMap<String, RuntimeActualEntry>,
    ) -> Result<()> {
        if entries.len() >= 100_000 {
            return Err("verifier runtime manifest entry bound exceeded".into());
        }
        let mut children = fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        children.sort_by_key(|e| e.file_name());
        for child in children {
            let path = child.path();
            let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("verifier runtime path encoding refused")?
                .to_owned();
            if metadata.is_dir() {
                visit(root, &path, entries)?;
            } else if metadata.is_file() {
                let bytes = read_regular(&path, 256 * 1024 * 1024)?;
                entries.insert(
                    relative,
                    RuntimeActualEntry {
                        kind: "regular".into(),
                        sha256: Some(digest(&bytes)),
                        bytes: Some(metadata.len()),
                        target: None,
                    },
                );
            } else if metadata.file_type().is_symlink() {
                let target = fs::read_link(&path).map_err(|e| e.to_string())?;
                let target = target
                    .to_str()
                    .ok_or("verifier runtime symlink encoding refused")?
                    .to_owned();
                link_target_inside(Path::new(&relative), &target)?;
                entries.insert(
                    relative,
                    RuntimeActualEntry {
                        kind: "symlink".into(),
                        sha256: None,
                        bytes: None,
                        target: Some(target),
                    },
                );
            } else {
                return Err("verifier runtime nonregular entry refused".into());
            }
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries)?;
    Ok(entries)
}
pub fn identity(path: &Path) -> Result<String> {
    directory(path)?;
    let m = fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(format!("{}:{}", m.dev(), m.ino()))
}
/// Publish once, sync bytes and directory, and verify any existing content.
pub fn retain(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.exists() {
        if fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
            || fs::read(path).map_err(|e| e.to_string())? != bytes
        {
            return Err("immutable retained bytes differ".into());
        }
        return Ok(());
    }
    let temporary = path.with_file_name(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    f.write_all(bytes)
        .and_then(|()| f.sync_all())
        .map_err(|e| e.to_string())?;
    fs::hard_link(&temporary, path).map_err(|e| e.to_string())?;
    fs::remove_file(temporary).map_err(|e| e.to_string())?;
    File::open(path.parent().ok_or("retained parent missing")?)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
}
#[derive(Clone, Deserialize, Serialize)]
struct ProcessEvidence {
    pid: u32,
    start_ticks: String,
    monitor_pid_namespace: String,
    exit: Option<i32>,
    signal: Option<i32>,
    waited: bool,
    stdout_limit: usize,
    stderr_limit: usize,
    limit: Option<String>,
}
struct ProcessOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    process: ProcessEvidence,
}
fn output(command: Command) -> Result<ProcessOutput> {
    output_limited(command, 24 * 1024 * 1024)
}
fn bounded_reader(
    reader: impl Read + Send + 'static,
    maximum: usize,
    exceeded: Arc<AtomicBool>,
) -> std::thread::JoinHandle<Result<Vec<u8>>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        reader
            .take(maximum as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > maximum {
            exceeded.store(true, Ordering::Release);
            bytes.truncate(maximum);
        }
        // Closing this pipe prevents further producer storage expansion.
        Ok(bytes)
    })
}
fn output_limited(mut command: Command, stdout_limit: usize) -> Result<ProcessOutput> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(feature = "fault-injection")]
    let await_exit = std::env::var("CORENIUS_TEST_GIT_FINAL_OUTPUT_WAIT").as_deref() == Ok("1")
        && command.get_args().any(|a| a == "/run/verification.py");
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, child.id(), 0) } as i32;
    if raw < 0 {
        child.kill().map_err(|e| e.to_string())?;
        child.wait().map_err(|e| e.to_string())?;
        return Err("owned command pidfd unavailable".into());
    }
    let pidfd = unsafe { OwnedFd::from_raw_fd(raw) };
    let identity = (|| {
        let stat =
            fs::read_to_string(format!("/proc/{}/stat", child.id())).map_err(|e| e.to_string())?;
        let start = stat
            .rsplit_once(") ")
            .and_then(|(_, tail)| tail.split_whitespace().nth(19))
            .ok_or("owned command kernel identity absent")?
            .to_owned();
        let namespace = fs::read_link(format!("/proc/{}/ns/pid", child.id()))
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "owned child already exited".into());
        Ok::<_, String>((start, namespace))
    })();
    let (start_ticks, monitor_pid_namespace) = match identity {
        Ok(v) => v,
        Err(e) => {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())?;
            return Err(e);
        }
    };
    let exceeded = Arc::new(AtomicBool::new(false));
    let stdout = bounded_reader(
        child.stdout.take().ok_or("owned stdout absent")?,
        stdout_limit,
        exceeded.clone(),
    );
    let stderr = bounded_reader(
        child.stderr.take().ok_or("owned stderr absent")?,
        1024 * 1024,
        exceeded.clone(),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut limit = None;
    #[cfg(feature = "fault-injection")]
    if await_exit {
        // Declared scheduling shim retains exit-before-observation semantics.
        while child.try_wait().map_err(|e| e.to_string())?.is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        let oversized = exceeded.load(Ordering::Acquire);
        if oversized || Instant::now() >= deadline {
            limit = Some(
                if oversized {
                    "output bound"
                } else {
                    "execution deadline"
                }
                .to_owned(),
            );
            if unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    pidfd.as_raw_fd(),
                    libc::SIGKILL,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                )
            } != 0
            {
                return Err("owned command pidfd stop failed".into());
            }
            break child.wait().map_err(|e| e.to_string())?;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let stdout = stdout.join().map_err(|_| "owned stdout reader failed")??;
    let stderr = stderr.join().map_err(|_| "owned stderr reader failed")??;
    // A successful fast exit cannot waive a limit observed by the readers.
    if exceeded.load(Ordering::Acquire) {
        limit = Some("output bound".into());
    }
    Ok(ProcessOutput {
        status,
        stdout,
        stderr,
        process: ProcessEvidence {
            pid: child.id(),
            start_ticks,
            monitor_pid_namespace,
            exit: status.code(),
            signal: status.signal(),
            waited: true,
            stdout_limit,
            stderr_limit: 1024 * 1024,
            limit,
        },
    })
}
const FIXED_GIT: &[&str] = &[
    "-c",
    "core.hooksPath=/dev/null",
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.untrackedCache=false",
    "-c",
    "core.sshCommand=/bin/false",
    "-c",
    "protocol.allow=never",
    "-c",
    "protocol.file.allow=always",
    "-c",
    "core.askPass=/bin/false",
    "-c",
    "credential.helper=",
    "-c",
    "uploadpack.allowFilter=false",
    "-c",
    "init.templateDir=",
    "-c",
    "transfer.fsckObjects=true",
    "-c",
    "fetch.fsckObjects=true",
];
impl GitBackend {
    pub fn open(path: &Path) -> Result<Self> {
        let policy: Policy = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for t in [&policy.git, &policy.python, &policy.bubblewrap] {
            check_tool(t)?;
        }
        for s in &policy.suites {
            check_tool(&s.script)?;
            if s.id.is_empty() || s.task.is_empty() {
                return Err("verification policy selector missing".into());
            }
        }
        for p in [&policy.root, &policy.attempts_root, &policy.canonical] {
            directory(p)?;
            if p.starts_with("/usr") {
                return Err("Git authority inside worker toolchain mount".into());
            }
        }
        if let Some(runtime) = &policy.verifier_runtime {
            directory(&runtime.root)?;
            if runtime.root.starts_with("/usr")
                || runtime.root.starts_with(&policy.attempts_root)
                || runtime.root.starts_with(&policy.canonical)
                || policy.attempts_root.starts_with(&runtime.root)
            {
                return Err("verifier runtime topology refused".into());
            }
        }
        if policy.root.starts_with(&policy.attempts_root)
            || policy.canonical.starts_with(&policy.attempts_root)
            || policy.attempts_root.starts_with(&policy.root)
            || policy.attempts_root.starts_with(&policy.canonical)
            || !policy.protected_ref.starts_with("refs/heads/")
        {
            return Err("Git authority/workspace topology refused".into());
        }
        let identity = hash(&serde_json::to_string(&policy).map_err(|e| e.to_string())?);
        let backend = Self { policy, identity };
        backend.git(
            &backend.policy.canonical,
            &["check-ref-format", &backend.policy.protected_ref],
        )?;
        for name in ["objects", "artifacts", "materializations", "captures"] {
            let p = backend.policy.root.join(name);
            if !p.exists() {
                fs::create_dir(&p).map_err(|e| e.to_string())?;
            }
            directory(&p)?;
        }
        let store = backend.store();
        if !store.exists() {
            backend.init(&store, true)?;
        }
        backend.verifier_runtime_evidence()?;
        backend.head()?;
        Ok(backend)
    }
    pub fn create_workspace(&self, root: &Path, binding: &Binding) -> Result<Workspace> {
        let g = self;
        if root != g.policy.attempts_root || binding.project != g.policy.project {
            return Err("trusted workspace policy/attempt root mismatch".into());
        }
        commit_oid(&binding.base)?;
        let workspace = crate::supervisor::prepare_workspace(root, &binding.attempt)?;
        let seed = root.join(format!("seed-{}.bundle", binding.operation));
        g.git(
            &g.policy.canonical,
            &[
                "bundle",
                "create",
                seed.to_str().ok_or("seed path encoding")?,
                "--all",
            ],
        )?;
        // The existing empty workspace is exclusively ours, before any process is started.
        g.init(&workspace.workspace, false)?;
        g.git(
            &workspace.workspace,
            &[
                "fetch",
                "--no-tags",
                seed.to_str().ok_or("seed path encoding")?,
                &binding.base,
            ],
        )?;
        g.git(
            &workspace.workspace,
            &["update-ref", "refs/heads/attempt", &binding.base],
        )?;
        g.git(
            &workspace.workspace,
            &["symbolic-ref", "HEAD", "refs/heads/attempt"],
        )?;
        g.git(&workspace.workspace, &["reset", "--hard", &binding.base])?;
        let owner = Ownership {
            binding: binding.clone(),
            workspace_identity: identity(&workspace.workspace)?,
            git_identity: identity(&workspace.workspace.join(".git"))?,
        };
        retain(
            &root.join(&binding.attempt).join("workspace-owner.json"),
            &serde_json::to_vec(&owner).map_err(|e| e.to_string())?,
        )?;
        Ok(workspace)
    }
    pub fn capture_candidate(
        &self,
        binding: &Binding,
        scratch: &Path,
    ) -> Result<(RepositoryState, Vec<u8>)> {
        let (repo, _) = self.owned(binding)?;
        let (state, bytes) = self.inspect(&repo, scratch, true)?;
        Ok((
            state,
            bytes.ok_or("retained candidate bundle stream absent")?,
        ))
    }
    pub fn cleanup_manifest(&self, root: &Path) -> Result<String> {
        self.artifact(&serde_json::to_vec(&cleanup_entries(root)?).map_err(|e| e.to_string())?)
    }
    /// Revalidate the original root and every surviving entry. Missing entries are
    /// expected after partial deletion; additions/replacements are preserved.
    pub fn retire_retained(
        &self,
        binding: &Binding,
        original_root: &str,
        manifest: &str,
    ) -> Result<()> {
        if binding.project != self.policy.project
            || uuid::Uuid::parse_str(&binding.attempt).is_err()
        {
            return Err("cleanup scope outside policy".into());
        }
        let root = self.policy.attempts_root.join(&binding.attempt);
        if identity(&root)? != original_root {
            return Err("cleanup root identity changed; preserve replacement".into());
        }
        let original: BTreeMap<String, CleanupEntry> =
            serde_json::from_slice(&self.bytes(manifest)?).map_err(|e| e.to_string())?;
        let remaining = cleanup_entries(&root)?;
        if remaining
            .iter()
            .any(|(path, entry)| original.get(path) != Some(entry))
        {
            return Err("cleanup unknown/changed remaining work preserved".into());
        }
        fs::remove_dir_all(root).map_err(|e| format!("owned cleanup failed; retryable: {e}"))
    }
    /// Stream a stopped owned workspace through bounded producer pipes. Never
    /// create an expanding archive file or wait with unbounded diagnostics.
    pub fn archive(&self, attempt: &str) -> Result<String> {
        if uuid::Uuid::parse_str(attempt).is_err() {
            return Err("archive attempt identity invalid".into());
        }
        let mut command = Command::new("/usr/bin/tar");
        command
            .env_clear()
            .args(["--format=posix", "-cf", "-", "-C"])
            .arg(&self.policy.attempts_root)
            .arg(attempt);
        let archive_tool = Tool {
            path: "/usr/bin/tar".into(),
            sha256: digest(&fs::read("/usr/bin/tar").map_err(|e| e.to_string())?),
        };
        let o = output_limited(command, 32 * 1024 * 1024)?;
        let diagnostics=self.artifact(&serde_json::to_vec(&serde_json::json!({"tool":archive_tool,"argv":["--format=posix","-cf","-","-C",self.policy.attempts_root.to_str().ok_or("archive root encoding")?,attempt],"process":o.process,"stderr":String::from_utf8_lossy(&o.stderr),"retained_archive_bytes":o.stdout.len(),"archive_limit":32*1024*1024})).map_err(|e|e.to_string())?)?;
        if !o.status.success() || o.process.limit.is_some() {
            return Err(format!(
                "workspace retention failed; cleanup retryable: {}; diagnostics {diagnostics}",
                o.process.limit.as_deref().unwrap_or("producer exit")
            ));
        }
        self.artifact(&o.stdout)
    }
    pub fn store(&self) -> PathBuf {
        self.policy.root.join("objects/store.git")
    }
    /// Only call on newly-created materializations or trusted controller repositories.
    pub fn git(&self, repo: &Path, args: &[&str]) -> Result<String> {
        check_tool(&self.policy.git)?;
        let mut c = Command::new(&self.policy.git.path);
        c.env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(FIXED_GIT)
            .arg("-C")
            .arg(repo)
            .args(args);
        let o = output(c)?;
        if !o.status.success() || o.process.limit.is_some() {
            return Err(format!(
                "trusted Git failed: {}",
                String::from_utf8_lossy(&o.stderr)
            ));
        }
        String::from_utf8(o.stdout)
            .map(|s| s.trim_end().to_owned())
            .map_err(|e| e.to_string())
    }
    pub fn init(&self, path: &Path, bare: bool) -> Result<()> {
        let format = self.git(
            &self.policy.canonical,
            &["rev-parse", "--show-object-format"],
        )?;
        let parent = path.parent().ok_or("repo parent missing")?;
        let mut args = vec!["init", "--template=", "--object-format", format.as_str()];
        if bare {
            args.push("--bare");
        }
        args.push(path.to_str().ok_or("repo path encoding")?);
        self.git(parent, &args)?;
        Ok(())
    }
    pub fn head(&self) -> Result<String> {
        self.git(
            &self.policy.canonical,
            &[
                "rev-parse",
                "--verify",
                &format!("{}^{{commit}}", self.policy.protected_ref),
            ],
        )
    }
    pub fn suite<'a>(&'a self, task: &str, base: &str) -> Result<&'a Suite> {
        let exact: Vec<_> = self
            .policy
            .suites
            .iter()
            .filter(|s| s.task == task && s.base.as_deref() == Some(base))
            .collect();
        let fallback: Vec<_> = self
            .policy
            .suites
            .iter()
            .filter(|s| s.task == task && s.base.is_none())
            .collect();
        let choices = if exact.is_empty() { fallback } else { exact };
        if choices.len() != 1 {
            return Err("fixed verification policy missing/ambiguous for admitted contract".into());
        }
        Ok(choices[0])
    }
    pub fn artifact(&self, bytes: &[u8]) -> Result<String> {
        let digest = digest(bytes);
        retain(&self.policy.root.join("artifacts").join(&digest), bytes)?;
        Ok(digest)
    }
    pub fn bytes(&self, digest_value: &str) -> Result<Vec<u8>> {
        if digest_value.len() != 64 || !digest_value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("artifact identity invalid".into());
        }
        let path = self.policy.root.join("artifacts").join(digest_value);
        if fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err("artifact alias refused".into());
        }
        let bytes = read_regular(&path, 32 * 1024 * 1024)?;
        if digest(&bytes) != digest_value {
            return Err("retained artifact digest mismatch".into());
        }
        Ok(bytes)
    }
    pub fn owned(&self, binding: &Binding) -> Result<(PathBuf, Ownership)> {
        if binding.project != self.policy.project
            || uuid::Uuid::parse_str(&binding.attempt).is_err()
        {
            return Err("workspace ownership outside policy".into());
        }
        let root = self.policy.attempts_root.join(&binding.attempt);
        directory(&root)?;
        let owner: Ownership = serde_json::from_slice(
            &fs::read(root.join("workspace-owner.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let repo = root.join("workspace");
        if owner.binding != *binding
            || owner.workspace_identity != identity(&repo)?
            || owner.git_identity != identity(&repo.join(".git"))?
        {
            return Err("workspace ownership/filesystem drift refused".into());
        }
        Ok((repo, owner))
    }
    pub fn verifier_runtime_evidence(&self) -> Result<Option<RuntimeEvidence>> {
        let Some(runtime) = &self.policy.verifier_runtime else {
            return Ok(None);
        };
        directory(&runtime.root)?;
        let manifest_bytes = read_regular(&runtime.manifest.path, 32 * 1024 * 1024)
            .map_err(|e| format!("verifier runtime manifest unavailable: {e}"))?;
        if digest(&manifest_bytes) != runtime.manifest.sha256 {
            return Err("verifier runtime manifest digest mismatch".into());
        }
        let manifest: RuntimeManifest =
            serde_json::from_slice(&manifest_bytes).map_err(|e| e.to_string())?;
        if manifest.schema_version != "corenius.verifier-runtime.v1" {
            return Err("verifier runtime manifest schema unsupported".into());
        }
        let mut expected = BTreeMap::new();
        for entry in manifest.entries {
            let relative = manifest_relative(&entry.path)?;
            let key = relative
                .to_str()
                .ok_or("verifier runtime manifest path encoding refused")?
                .to_owned();
            if expected.contains_key(&key) {
                return Err("verifier runtime manifest duplicate entry".into());
            }
            let actual = match entry.kind.as_str() {
                "regular" => RuntimeActualEntry {
                    kind: entry.kind,
                    sha256: Some(
                        entry
                            .sha256
                            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                            .ok_or("verifier runtime manifest digest invalid")?,
                    ),
                    bytes: Some(
                        entry
                            .bytes
                            .ok_or("verifier runtime manifest size missing")?,
                    ),
                    target: None,
                },
                "symlink" => {
                    let target = entry
                        .target
                        .ok_or("verifier runtime manifest link target missing")?;
                    link_target_inside(&relative, &target)?;
                    RuntimeActualEntry {
                        kind: entry.kind,
                        sha256: None,
                        bytes: None,
                        target: Some(target),
                    }
                }
                _ => return Err("verifier runtime manifest kind unsupported".into()),
            };
            expected.insert(key, actual);
        }
        let actual = runtime_entries(&runtime.root)?;
        if actual != expected {
            return Err("verifier runtime manifest mismatch".into());
        }
        Ok(Some(RuntimeEvidence {
            status: "pinned",
            root: runtime.root.clone(),
            manifest_path: runtime.manifest.path.clone(),
            manifest: runtime.manifest.sha256.clone(),
            entries: actual.len(),
        }))
    }
    pub fn verifier_runtime_document(&self) -> Result<serde_json::Value> {
        match self.verifier_runtime_evidence()? {
            Some(evidence) => serde_json::to_value(evidence).map_err(|e| e.to_string()),
            None => Ok(serde_json::Value::Null),
        }
    }
    fn boundary(
        &self,
        repo: &Path,
        _scratch: &Path,
        runtime: Option<&RuntimeEvidence>,
    ) -> Result<Command> {
        check_tool(&self.policy.bubblewrap)?;
        let mut c = Command::new(&self.policy.bubblewrap.path);
        c.env_clear()
            .args([
                "--unshare-all",
                "--new-session",
                "--die-with-parent",
                "--as-pid-1",
                "--clearenv",
                "--ro-bind",
            ])
            .arg(
                runtime
                    .map(|r| r.root.as_path())
                    .unwrap_or(Path::new("/usr")),
            )
            .arg("/usr")
            .args([
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
                "--dir",
                "/home/worker",
                "--setenv",
                "HOME",
                "/home/worker",
                "--setenv",
                "PATH",
                "/usr/bin:/bin",
                "--setenv",
                "GIT_CONFIG_NOSYSTEM",
                "1",
                "--setenv",
                "GIT_CONFIG_GLOBAL",
                "/dev/null",
                "--setenv",
                "GIT_NO_REPLACE_OBJECTS",
                "1",
                "--setenv",
                "GIT_TERMINAL_PROMPT",
                "0",
            ])
            .arg("--bind")
            .arg(repo)
            .arg("/workspace")
            .args(["--tmpfs", "/scratch", "--chdir", "/workspace"]);
        Ok(c)
    }
    /// Capture and inspect untrusted .git ONLY inside the OS boundary. Fixed argv,
    /// disabled hooks/fsmonitor/replacements, no worker upload-pack on the host.
    pub fn inspect(
        &self,
        repo: &Path,
        scratch: &Path,
        bundle: bool,
    ) -> Result<(RepositoryState, Option<Vec<u8>>)> {
        let script = r#"import subprocess,json,os,base64,stat
fixed=['-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-c','core.untrackedCache=false','-c','core.sshCommand=/bin/false','-c','credential.helper=']
def git(*args):return subprocess.check_output(['/run/git']+fixed+['-C','/workspace']+list(args),text=True).rstrip('\n')
assert os.path.isdir('/workspace/.git') and not os.path.islink('/workspace/.git')
assert not os.path.exists('/workspace/.git/commondir') and not os.path.exists('/workspace/.git/gitdir'),'linked metadata refused'
assert os.path.isdir('/workspace/.git/objects') and not os.path.islink('/workspace/.git/objects')
assert not os.path.exists('/workspace/.git/objects/info/alternates'),'alternate objects refused'
for parent,dirs,files in os.walk('/workspace/.git/objects'):
 for name in dirs+files:
  entry=os.lstat(os.path.join(parent,name));assert not stat.S_ISLNK(entry.st_mode)
  if stat.S_ISREG(entry.st_mode):assert entry.st_nlink==1,'shared object hardlink refused'
assert not os.path.exists('/workspace/.git/info/grafts'),'grafts refused'
commit=git('rev-parse','--verify','HEAD^{commit}');tree=git('rev-parse','--verify',commit+'^{tree}')
refs=[line.split(' ',1) for line in git('for-each-ref','--format=%(refname) %(objectname)').splitlines()]
status=git('status','--porcelain=v1','--untracked-files=all')
encoded=None
if os.environ.get('CAPTURE_BUNDLE')=='1':
 git('bundle','create','/scratch/candidate.bundle',commit,'--all')
 fd=os.open('/scratch/candidate.bundle',os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
 with os.fdopen(fd,'rb') as stream:
  retained=os.fstat(stream.fileno());named=os.lstat('/scratch/candidate.bundle')
  assert stat.S_ISREG(retained.st_mode) and retained.st_nlink==1 and retained.st_size<=16777216
  assert retained.st_dev==named.st_dev and retained.st_ino==named.st_ino and stat.S_ISREG(named.st_mode)
  data=stream.read(16777217);assert len(data)<=16777216;encoded=base64.b64encode(data).decode()
print(json.dumps({'state':{'commit':commit,'tree':tree,'refs':refs,'status':status,'capture_pid':os.getpid(),'pid_namespace':os.readlink('/proc/self/ns/pid')},'bundle_b64':encoded}))
"#;
        let mut c = self.boundary(repo, scratch, None)?;
        c.arg("--ro-bind")
            .arg(&self.policy.git.path)
            .arg("/run/git")
            .args(["--setenv", "CAPTURE_BUNDLE", if bundle { "1" } else { "0" }]);
        c.arg(&self.policy.python.path).args(["-I", "-c", script]);
        let argv = std::iter::once(c.get_program())
            .chain(c.get_args())
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();
        let result = output(c)?;
        if !result.status.success() || result.process.limit.is_some() {
            return Err(format!(
                "isolated worker Git refused: {}",
                String::from_utf8_lossy(&result.stderr)
            ));
        }
        let mut inspection: Inspection = serde_json::from_slice(&result.stdout)
            .map_err(|e| format!("isolated Git metadata malformed: {e}"))?;
        inspection.state.inspection = Some(InspectionExecution {
            argv,
            environment_cleared: true,
            tools: vec![
                self.policy.python.clone(),
                self.policy.git.clone(),
                self.policy.bubblewrap.clone(),
            ],
            process: result.process,
        });
        let bytes = inspection
            .bundle_b64
            .map(|s| {
                base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .map_err(|_| "bundle stream malformed".to_owned())
            })
            .transpose()?;
        Ok((inspection.state, bytes))
    }
    pub fn import(&self, bundle_digest: &str, commit: &str, tree: &str) -> Result<()> {
        commit_oid(commit)?;
        commit_oid(tree)?;
        self.bytes(bundle_digest)?;
        let path = self.policy.root.join("artifacts").join(bundle_digest);
        self.git(
            &self.store(),
            &[
                "bundle",
                "verify",
                path.to_str().ok_or("bundle path encoding")?,
            ],
        )?;
        self.git(
            &self.store(),
            &[
                "fetch",
                "--no-tags",
                path.to_str().ok_or("bundle path encoding")?,
                commit,
            ],
        )?;
        let actual = self.git(
            &self.store(),
            &["rev-parse", "--verify", &format!("{commit}^{{tree}}")],
        )?;
        if actual != tree {
            return Err("candidate commit/tree differs from retained bundle".into());
        }
        self.git(
            &self.store(),
            &["update-ref", &format!("refs/candidates/{commit}"), commit],
        )?;
        Ok(())
    }
    pub fn materialize_candidate(&self, bundle: &str, commit: &str, tree: &str) -> Result<PathBuf> {
        self.import(bundle, commit, tree)?;
        let root = self
            .policy
            .root
            .join("materializations")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir(&root).map_err(|e| e.to_string())?;
        let repo = root.join("workspace");
        self.init(&repo, false)?;
        self.git(
            &repo,
            &[
                "fetch",
                "--no-tags",
                self.policy
                    .root
                    .join("artifacts")
                    .join(bundle)
                    .to_str()
                    .ok_or("path encoding")?,
                commit,
            ],
        )?;
        self.git(&repo, &["checkout", "--detach", commit])?;
        fs::create_dir(root.join("scratch")).map_err(|e| e.to_string())?;
        Ok(repo)
    }
    pub fn verify(
        &self,
        repo: &Path,
        suite: &Suite,
    ) -> Result<(bool, Vec<u8>, String, serde_json::Value)> {
        check_tool(&suite.script)?;
        check_tool(&self.policy.python)?;
        let runtime = self.verifier_runtime_evidence()?;
        let runtime_doc = match &runtime {
            Some(evidence) => serde_json::to_value(evidence).map_err(|e| e.to_string())?,
            None => serde_json::Value::Null,
        };
        let environment = digest(
            &serde_json::to_vec(&(
                &self.policy.python,
                &self.policy.bubblewrap,
                &runtime_doc,
                "unshare-all/no-controller-mounts/PATH=/usr/bin:/bin",
            ))
            .map_err(|e| e.to_string())?,
        );
        let mut c = self.boundary(
            repo,
            &repo
                .parent()
                .ok_or("materialization parent")?
                .join("scratch"),
            runtime.as_ref(),
        )?;
        c.arg("--ro-bind")
            .arg(&suite.script.path)
            .arg("/run/verification.py")
            .arg(&self.policy.python.path)
            .arg("/run/verification.py")
            .args(&suite.args);
        let o = output(c)?;
        let log=serde_json::to_vec(&serde_json::json!({"argv":[self.policy.python.path.to_string_lossy(),"/run/verification.py"],"args":suite.args,"exit":o.status.code(),"stdout":String::from_utf8_lossy(&o.stdout),"stderr":String::from_utf8_lossy(&o.stderr),"suite":suite,"environment":environment,"runtime":runtime_doc,"os_isolated":true,"process":o.process})).map_err(|e|e.to_string())?;
        Ok((
            o.status.success() && o.process.limit.is_none(),
            log,
            environment,
            runtime_doc,
        ))
    }
    pub fn compare(&self, base: &str, candidate: &str) -> Result<String> {
        self.git(
            &self.store(),
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                base,
                candidate,
                "--",
            ],
        )
    }
    pub fn ancestor(&self, repo: &Path, base: &str, candidate: &str) -> Result<bool> {
        match self.git(repo, &["merge-base", "--is-ancestor", base, candidate]) {
            Ok(_) => Ok(true),
            Err(e) if e.starts_with("trusted Git failed:") => Ok(false),
            Err(e) => Err(e),
        }
    }
    pub fn land(&self, commit: &str, expected: &str) -> Result<()> {
        if !self.ancestor(&self.store(), expected, commit)? {
            return Err("candidate does not fast-forward expected base".into());
        }
        self.git(
            &self.policy.canonical,
            &[
                "fetch",
                "--no-tags",
                self.store().to_str().ok_or("store path encoding")?,
                commit,
            ],
        )?;
        self.git(
            &self.policy.canonical,
            &["update-ref", &self.policy.protected_ref, commit, expected],
        )?;
        Ok(())
    }
}
/// Used by the actual supervisor, including native Codex executor profiles.
pub struct TrustedWorkspacePreparer(pub GitBackend);
impl WorkspacePreparer for TrustedWorkspacePreparer {
    fn prepare(&self, root: &Path, binding: &Binding) -> Result<Workspace> {
        self.0.create_workspace(root, binding)
    }
}
