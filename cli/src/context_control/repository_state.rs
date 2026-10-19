use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest as _, Sha256};

use super::REPOSITORY_SNAPSHOT_SCHEMA;
use crate::AppError;

const MAX_GIT_OUTPUT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RepositorySnapshot {
    pub schema_version: String,
    pub repository_id: String,
    pub head: String,
    pub status_sha256: String,
    pub tracked_changes_sha256: String,
    pub tracked_count: u64,
    pub untracked_count: u64,
    pub dirty: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositorySnapshotWire {
    schema_version: String,
    repository_id: String,
    head: String,
    status_sha256: String,
    tracked_changes_sha256: String,
    tracked_count: u64,
    untracked_count: u64,
    dirty: bool,
}

impl<'de> Deserialize<'de> for RepositorySnapshot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RepositorySnapshotWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl RepositorySnapshot {
    pub fn capture(repository_root: &Path) -> Result<Self, AppError> {
        capture_with_runner_and_limit(
            repository_root,
            &ProcessGitRunner::git(),
            MAX_GIT_OUTPUT_BYTES,
        )
    }

    fn from_wire(wire: RepositorySnapshotWire) -> Result<Self, AppError> {
        let snapshot = Self {
            schema_version: wire.schema_version,
            repository_id: wire.repository_id,
            head: wire.head,
            status_sha256: wire.status_sha256,
            tracked_changes_sha256: wire.tracked_changes_sha256,
            tracked_count: wire.tracked_count,
            untracked_count: wire.untracked_count,
            dirty: wire.dirty,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != REPOSITORY_SNAPSHOT_SCHEMA {
            return Err(invalid_snapshot("invalid repository snapshot schema"));
        }
        if !is_sha256_id(&self.repository_id)
            || !is_sha256_id(&self.status_sha256)
            || !is_sha256_id(&self.tracked_changes_sha256)
        {
            return Err(invalid_snapshot(
                "repository snapshot digests must be lowercase SHA-256 IDs",
            ));
        }
        if !valid_object_id(self.head.as_bytes()) {
            return Err(invalid_snapshot(
                "repository snapshot HEAD must be a full object ID",
            ));
        }
        if self.dirty != (self.tracked_count != 0 || self.untracked_count != 0) {
            return Err(invalid_snapshot(
                "repository snapshot dirty flag does not match change counts",
            ));
        }
        Ok(())
    }
}

pub fn resolve_worktree_root(start: &Path) -> Result<PathBuf, AppError> {
    resolve_worktree_root_with_runner(start, &ProcessGitRunner::git())
}

fn resolve_worktree_root_with_runner(
    start: &Path,
    runner: &impl GitRunner,
) -> Result<PathBuf, AppError> {
    let output = run_hardened_git(
        start,
        runner,
        MAX_GIT_OUTPUT_BYTES,
        ["rev-parse", "--path-format=absolute", "--show-toplevel"],
    )?;
    let root = single_line(&output, "repository.root")?;
    let root = std::str::from_utf8(root).map_err(|_| {
        AppError::invalid_input("repository.root", "Git worktree root is not UTF-8")
    })?;
    validate_worktree_root(Path::new(root))
}

fn validate_worktree_root(root: &Path) -> Result<PathBuf, AppError> {
    if !root.is_absolute() {
        return Err(AppError::invalid_input(
            "repository.root",
            "Git worktree root must be absolute",
        ));
    }
    let metadata = fs::symlink_metadata(root).map_err(|error| {
        AppError::io(
            "repository.root",
            "could not inspect Git worktree root",
            error,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AppError::invalid_input(
            "repository.root",
            "Git worktree root must be a non-symlink directory",
        ));
    }
    let canonical = fs::canonicalize(root).map_err(|error| {
        AppError::io(
            "repository.root",
            "could not canonicalize Git worktree root",
            error,
        )
    })?;
    if canonical != root {
        return Err(AppError::invalid_input(
            "repository.root",
            "Git worktree root must use its canonical absolute spelling",
        ));
    }
    Ok(canonical)
}

pub(crate) fn repository_id(repository_root: &Path) -> Result<String, AppError> {
    repository_id_with_runner_and_limit(
        repository_root,
        &ProcessGitRunner::git(),
        MAX_GIT_OUTPUT_BYTES,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GitInvocation {
    current_dir: PathBuf,
    arguments: Vec<String>,
    environment: BTreeMap<String, String>,
}

impl GitInvocation {
    fn hardened(repository_root: &Path, arguments: impl IntoIterator<Item = String>) -> Self {
        Self {
            current_dir: repository_root.to_path_buf(),
            arguments: arguments.into_iter().collect(),
            environment: BTreeMap::from([
                ("GIT_CONFIG".to_owned(), "/dev/null".to_owned()),
                ("GIT_CONFIG_GLOBAL".to_owned(), "/dev/null".to_owned()),
                ("GIT_CONFIG_NOSYSTEM".to_owned(), "1".to_owned()),
                ("GIT_NO_REPLACE_OBJECTS".to_owned(), "1".to_owned()),
                ("GIT_OPTIONAL_LOCKS".to_owned(), "0".to_owned()),
            ]),
        }
    }

    fn local_config_keys(repository_root: &Path) -> Self {
        let mut invocation = Self::hardened(
            repository_root,
            ["config", "--null", "--name-only", "--list", "--includes"].map(str::to_owned),
        );
        // GIT_CONFIG is equivalent to `git config --file` and conflicts with
        // reading the repository scopes. This command reads bounded key names
        // only; it never evaluates configured commands or values.
        invocation.environment.remove("GIT_CONFIG");
        invocation
    }

    fn with_config(mut self, config: &[(String, String)]) -> Self {
        self.environment
            .insert("GIT_CONFIG_COUNT".to_owned(), config.len().to_string());
        for (index, (key, value)) in config.iter().enumerate() {
            self.environment
                .insert(format!("GIT_CONFIG_KEY_{index}"), key.clone());
            self.environment
                .insert(format!("GIT_CONFIG_VALUE_{index}"), value.clone());
        }
        self
    }
}

#[derive(Debug)]
struct GitOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    success: bool,
}

trait GitRunner {
    fn run(&self, invocation: &GitInvocation, limit: usize) -> Result<GitOutput, AppError>;
}

struct ProcessGitRunner {
    executable: PathBuf,
}

impl ProcessGitRunner {
    fn git() -> Self {
        Self::for_executable("git")
    }

    fn for_executable(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
        }
    }
}

impl GitRunner for ProcessGitRunner {
    fn run(&self, invocation: &GitInvocation, limit: usize) -> Result<GitOutput, AppError> {
        let mut command = Command::new(&self.executable);
        command
            .current_dir(&invocation.current_dir)
            .args(&invocation.arguments)
            .env_clear()
            .envs(invocation.environment.iter())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|error| AppError::io("repository.git_spawn", "could not start Git", error))?;
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => return cleanup_spawned_child(&mut child, "Git stdout pipe was unavailable"),
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => return cleanup_spawned_child(&mut child, "Git stderr pipe was unavailable"),
        };

        let (signals, receiver) = mpsc::channel();
        let stdout_signals = signals.clone();
        let stdout_reader = thread::spawn(move || {
            let result = read_bounded(stdout, limit);
            let _ = stdout_signals.send(reader_is_adverse(&result));
            result
        });
        let stderr_reader = thread::spawn(move || {
            let result = read_bounded(stderr, limit);
            let _ = signals.send(reader_is_adverse(&result));
            result
        });

        let mut status = None;
        let mut reader_failed = false;
        while status.is_none() && !reader_failed {
            status = match child.try_wait() {
                Ok(status) => status,
                Err(error) => {
                    cleanup_process_and_readers(&mut child, stdout_reader, stderr_reader);
                    return Err(AppError::io(
                        "repository.git_wait",
                        "could not wait for Git",
                        error,
                    ));
                }
            };
            if status.is_some() {
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(adverse) => reader_failed = adverse,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {}
            }
        }

        if reader_failed {
            let _ = child.kill();
        }
        let waited = match status {
            Some(status) => Ok(status),
            None => child.wait(),
        };
        let stdout = join_reader_result(stdout_reader, "stdout");
        let stderr = join_reader_result(stderr_reader, "stderr");
        let status = waited
            .map_err(|error| AppError::io("repository.git_wait", "could not wait for Git", error));

        let stdout = stdout?;
        let stderr = stderr?;
        Ok(GitOutput {
            stdout,
            stderr,
            success: status?.success(),
        })
    }
}

enum BoundedRead {
    Bytes(Vec<u8>),
    LimitExceeded,
}

fn read_bounded(mut stream: impl Read, limit: usize) -> io::Result<BoundedRead> {
    let mut bytes = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 8192];
    loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Ok(BoundedRead::Bytes(bytes));
        }
        let remaining = limit.saturating_sub(bytes.len());
        if read > remaining {
            return Ok(BoundedRead::LimitExceeded);
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
}

fn reader_is_adverse(result: &io::Result<BoundedRead>) -> bool {
    !matches!(result, Ok(BoundedRead::Bytes(_)))
}

fn cleanup_spawned_child(
    child: &mut std::process::Child,
    message: &'static str,
) -> Result<GitOutput, AppError> {
    let _ = child.kill();
    let _ = child.wait();
    Err(AppError::external("repository.git_io", message))
}

fn cleanup_process_and_readers(
    child: &mut std::process::Child,
    stdout: thread::JoinHandle<io::Result<BoundedRead>>,
    stderr: thread::JoinHandle<io::Result<BoundedRead>>,
) {
    let _ = child.kill();
    let _ = child.wait();
    let _ = stdout.join();
    let _ = stderr.join();
}

fn join_reader_result(
    reader: thread::JoinHandle<io::Result<BoundedRead>>,
    stream: &str,
) -> Result<Vec<u8>, AppError> {
    match reader.join() {
        Ok(Ok(BoundedRead::Bytes(bytes))) => Ok(bytes),
        Ok(Ok(BoundedRead::LimitExceeded)) => Err(AppError::invalid_input(
            "repository.output_limit",
            format!("Git {stream} exceeds the configured output limit"),
        )),
        Ok(Err(error)) => Err(AppError::io(
            "repository.git_io",
            &format!("could not read Git {stream}"),
            error,
        )),
        Err(_) => Err(AppError::external(
            "repository.git_io",
            format!("Git {stream} reader failed"),
        )),
    }
}

fn capture_with_runner_and_limit(
    repository_root: &Path,
    runner: &impl GitRunner,
    limit: usize,
) -> Result<RepositorySnapshot, AppError> {
    let repository_id = repository_id_with_runner_and_limit(repository_root, runner, limit)?;

    let head = run_hardened_git(
        repository_root,
        runner,
        limit,
        ["rev-parse", "--verify", "HEAD^{commit}"],
    )?;
    let head = single_line(&head, "repository.head")?;
    if !valid_object_id(head) {
        return Err(AppError::invalid_input(
            "repository.head",
            "Git HEAD is not a full lowercase hexadecimal object ID",
        ));
    }
    let head = std::str::from_utf8(head)
        .expect("validated hexadecimal Git object ID is UTF-8")
        .to_owned();

    let filter_overrides = repository_filter_overrides(repository_root, runner, limit)?;
    let safe_config = safe_worktree_config(&filter_overrides);
    let status = run_hardened_git_with_config(
        repository_root,
        runner,
        limit,
        &safe_config,
        ["status", "--porcelain=v2", "-z", "--untracked-files=normal"],
    )?;
    let tracked_changes = run_hardened_git_with_config(
        repository_root,
        runner,
        limit,
        &safe_config,
        [
            "diff".to_owned(),
            "--no-ext-diff".to_owned(),
            "--no-textconv".to_owned(),
            "--binary".to_owned(),
            head.clone(),
            "--".to_owned(),
        ],
    )?;
    let final_head = run_hardened_git(
        repository_root,
        runner,
        limit,
        ["rev-parse", "--verify", "HEAD^{commit}"],
    )?;
    let final_head = single_line(&final_head, "repository.head")?;
    if final_head != head.as_bytes() {
        return Err(AppError::invalid_input(
            "repository.concurrent_change",
            "repository HEAD changed during snapshot capture",
        ));
    }
    let (tracked_count, untracked_count) = status_counts(&status)?;

    let snapshot = RepositorySnapshot {
        schema_version: REPOSITORY_SNAPSHOT_SCHEMA.to_owned(),
        repository_id,
        head,
        status_sha256: sha256_id(&status),
        tracked_changes_sha256: sha256_id(&tracked_changes),
        tracked_count,
        untracked_count,
        dirty: tracked_count != 0 || untracked_count != 0,
    };
    snapshot.validate()?;
    Ok(snapshot)
}

fn repository_id_with_runner_and_limit(
    repository_root: &Path,
    runner: &impl GitRunner,
    limit: usize,
) -> Result<String, AppError> {
    let common_dir = run_hardened_git(
        repository_root,
        runner,
        limit,
        ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    let common_dir = single_line(&common_dir, "repository.common_dir")?;
    Ok(sha256_id(common_dir))
}

fn run_hardened_git(
    repository_root: &Path,
    runner: &impl GitRunner,
    limit: usize,
    arguments: impl IntoIterator<Item = impl Into<String>>,
) -> Result<Vec<u8>, AppError> {
    let invocation =
        GitInvocation::hardened(repository_root, arguments.into_iter().map(Into::into));
    run_git(runner, limit, invocation)
}

fn run_hardened_git_with_config(
    repository_root: &Path,
    runner: &impl GitRunner,
    limit: usize,
    config: &[(String, String)],
    arguments: impl IntoIterator<Item = impl Into<String>>,
) -> Result<Vec<u8>, AppError> {
    let invocation =
        GitInvocation::hardened(repository_root, arguments.into_iter().map(Into::into))
            .with_config(config);
    run_git(runner, limit, invocation)
}

fn run_git(
    runner: &impl GitRunner,
    limit: usize,
    invocation: GitInvocation,
) -> Result<Vec<u8>, AppError> {
    let output = runner.run(&invocation, limit)?;
    if !output.success {
        return Err(AppError::external(
            "repository.git",
            format!("Git command failed: git {}", invocation.arguments.join(" ")),
        ));
    }
    // Stderr is captured and bounded to avoid inheriting a terminal or leaking it
    // into snapshot data. Successful plumbing commands do not use it as input.
    let _ = output.stderr;
    Ok(output.stdout)
}

fn repository_filter_overrides(
    repository_root: &Path,
    runner: &impl GitRunner,
    limit: usize,
) -> Result<Vec<String>, AppError> {
    let keys = run_git(
        runner,
        limit,
        GitInvocation::local_config_keys(repository_root),
    )?;
    let mut drivers = BTreeMap::<String, ()>::new();
    for key in keys
        .split(|byte| *byte == b'\0')
        .filter(|key| !key.is_empty())
    {
        let key = std::str::from_utf8(key).map_err(|_| {
            AppError::invalid_input(
                "repository.git_config",
                "repository Git config contains a non-UTF-8 key",
            )
        })?;
        let Some(driver) = filter_driver_from_key(key) else {
            continue;
        };
        drivers.insert(driver.to_owned(), ());
    }

    Ok(drivers.into_keys().collect())
}

fn filter_driver_from_key(key: &str) -> Option<&str> {
    let key = key.strip_prefix("filter.")?;
    [".clean", ".process", ".required"]
        .into_iter()
        .find_map(|suffix| key.strip_suffix(suffix))
        .filter(|driver| !driver.is_empty())
}

fn safe_worktree_config(filter_overrides: &[String]) -> Vec<(String, String)> {
    let mut config = Vec::with_capacity(1 + filter_overrides.len() * 3);
    config.push(("core.fsmonitor".to_owned(), "false".to_owned()));
    for driver in filter_overrides {
        config.extend([
            (format!("filter.{driver}.clean"), String::new()),
            (format!("filter.{driver}.process"), String::new()),
            (format!("filter.{driver}.required"), "false".to_owned()),
        ]);
    }
    config
}

fn single_line<'a>(bytes: &'a [u8], code: &'static str) -> Result<&'a [u8], AppError> {
    let line = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    if line.is_empty()
        || line
            .iter()
            .any(|byte| matches!(byte, b'\0' | b'\n' | b'\r'))
    {
        return Err(AppError::invalid_input(
            code,
            "Git returned a malformed single-line value",
        ));
    }
    Ok(line)
}

fn valid_object_id(value: &[u8]) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn is_sha256_id(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(is_lowercase_sha256)
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_id(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn invalid_snapshot(message: impl Into<String>) -> AppError {
    AppError::invalid_input("repository.snapshot", message)
}

fn status_counts(bytes: &[u8]) -> Result<(u64, u64), AppError> {
    let mut records = bytes.split(|byte| *byte == b'\0').peekable();
    let mut tracked = 0_u64;
    let mut untracked = 0_u64;

    while let Some(record) = records.next() {
        if record.is_empty() && records.peek().is_none() {
            break;
        }
        if record.starts_with(b"1 ") || record.starts_with(b"u ") {
            tracked = checked_increment(tracked)?;
        } else if record.starts_with(b"2 ") {
            tracked = checked_increment(tracked)?;
            let original_path = records.next().ok_or_else(malformed_status)?;
            if original_path.is_empty() {
                return Err(malformed_status());
            }
        } else if record.starts_with(b"? ") {
            untracked = checked_increment(untracked)?;
        } else {
            return Err(malformed_status());
        }
    }
    Ok((tracked, untracked))
}

fn checked_increment(value: u64) -> Result<u64, AppError> {
    value.checked_add(1).ok_or_else(malformed_status)
}

fn malformed_status() -> AppError {
    AppError::invalid_input(
        "repository.status",
        "Git returned malformed porcelain v2 status",
    )
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use tempfile::tempdir;

    use super::*;
    use crate::context_control::REPOSITORY_SNAPSHOT_SCHEMA;

    #[test]
    fn worktree_root_resolution_returns_the_canonical_git_root_from_a_nested_path() {
        let fixture = GitFixture::new();
        let nested = fixture.root().join("nested/directory");
        fs::create_dir_all(&nested).unwrap();

        let resolved = resolve_worktree_root(&nested).unwrap();

        assert_eq!(resolved, fs::canonicalize(fixture.root()).unwrap());
        assert!(resolved.is_absolute());
        assert!(!fs::symlink_metadata(&resolved)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn worktree_root_resolution_uses_one_hardened_git_command() {
        let fixture = GitFixture::new();
        let canonical = fs::canonicalize(fixture.root()).unwrap();
        let runner =
            FakeRunner::with_results([output(format!("{}\n", canonical.display()).as_bytes())]);

        let resolved = resolve_worktree_root_with_runner(fixture.root(), &runner).unwrap();

        assert_eq!(resolved, canonical);
        let invocations = runner.invocations.borrow();
        assert_eq!(invocations.len(), 1);
        assert_eq!(
            invocations[0].arguments,
            ["rev-parse", "--path-format=absolute", "--show-toplevel"].map(str::to_owned)
        );
        assert_eq!(
            invocations[0].environment.get("GIT_CONFIG_NOSYSTEM"),
            Some(&"1".to_owned())
        );
        assert_eq!(
            invocations[0].environment.get("GIT_CONFIG_GLOBAL"),
            Some(&"/dev/null".to_owned())
        );
    }

    #[test]
    fn linked_worktrees_share_repository_identity_and_dirty_content_changes_digests() {
        let fixture = GitFixture::new();
        let linked = fixture.linked_worktree();

        let repository = RepositorySnapshot::capture(fixture.root()).unwrap();
        let worktree = RepositorySnapshot::capture(&linked).unwrap();
        assert_eq!(repository.repository_id, worktree.repository_id);
        assert_eq!(repository.head, worktree.head);
        assert!(!repository.dirty);
        assert_eq!(repository.tracked_count, 0);
        assert_eq!(repository.untracked_count, 0);

        fs::write(linked.join("tracked.txt"), "changed tracked contents\n").unwrap();
        let dirty = RepositorySnapshot::capture(&linked).unwrap();
        assert!(dirty.dirty);
        assert_eq!(dirty.tracked_count, 1);
        assert_eq!(dirty.untracked_count, 0);
        assert_ne!(dirty.status_sha256, worktree.status_sha256);
        assert_ne!(
            dirty.tracked_changes_sha256,
            worktree.tracked_changes_sha256
        );
    }

    #[test]
    fn snapshot_counts_untracked_entries_without_exposing_names_or_contents() {
        let fixture = GitFixture::new();
        let secret_name = "private-untracked-name.txt";
        let secret_content = "private untracked contents";
        let remote = "https://credentials.example/private/repository.git";
        git(fixture.root(), &["remote", "add", "origin", remote]);
        fs::write(fixture.root().join(secret_name), secret_content).unwrap();

        let snapshot = RepositorySnapshot::capture(fixture.root()).unwrap();
        let serialized = serde_json::to_string(&snapshot).unwrap();

        assert_eq!(snapshot.schema_version, REPOSITORY_SNAPSHOT_SCHEMA);
        assert!(snapshot.dirty);
        assert_eq!(snapshot.tracked_count, 0);
        assert_eq!(snapshot.untracked_count, 1);
        assert!(!serialized.contains(fixture.root().to_string_lossy().as_ref()));
        assert!(!serialized.contains(remote));
        assert!(!serialized.contains(secret_name));
        assert!(!serialized.contains(secret_content));
        assert!(!serialized.contains("tracked.txt"));
        assert!(!serialized.contains("diff --git"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&serialized)
                .unwrap()
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "dirty",
                "head",
                "repository_id",
                "schema_version",
                "status_sha256",
                "tracked_changes_sha256",
                "tracked_count",
                "untracked_count",
            ]
        );
    }

    #[test]
    fn public_deserialization_validates_snapshot_invariants() {
        let valid = RepositorySnapshot {
            schema_version: REPOSITORY_SNAPSHOT_SCHEMA.to_owned(),
            repository_id: format!("sha256:{}", "a".repeat(64)),
            head: "b".repeat(40),
            status_sha256: format!("sha256:{}", "c".repeat(64)),
            tracked_changes_sha256: format!("sha256:{}", "d".repeat(64)),
            tracked_count: 1,
            untracked_count: 0,
            dirty: true,
        };
        let value = serde_json::to_value(&valid).unwrap();
        assert_eq!(
            serde_json::from_value::<RepositorySnapshot>(value.clone()).unwrap(),
            valid
        );

        for (field, invalid) in [
            ("schema_version", serde_json::json!("wrong")),
            ("repository_id", serde_json::json!("sha256:short")),
            ("head", serde_json::json!("not-an-object")),
            ("status_sha256", serde_json::json!("sha256:short")),
            ("tracked_changes_sha256", serde_json::json!("sha256:short")),
            ("dirty", serde_json::json!(false)),
        ] {
            let mut value = value.clone();
            value[field] = invalid;
            assert!(
                serde_json::from_value::<RepositorySnapshot>(value).is_err(),
                "invalid {field} must be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_suppresses_repository_fsmonitor_and_clean_filter_commands() {
        use std::os::unix::fs::PermissionsExt;

        let fixture = GitFixture::new();
        let fsmonitor_marker = fixture.workspace.path().join("fsmonitor-ran");
        let clean_marker = fixture.workspace.path().join("clean-filter-ran");
        let fsmonitor = fixture.workspace.path().join("fsmonitor.sh");
        let clean = fixture.workspace.path().join("clean.sh");
        fs::write(
            &fsmonitor,
            format!(
                "#!/bin/sh\ntouch '{}'\nprintf '/'\n",
                fsmonitor_marker.display()
            ),
        )
        .unwrap();
        fs::write(
            &clean,
            format!("#!/bin/sh\ntouch '{}'\ncat\n", clean_marker.display()),
        )
        .unwrap();
        fs::set_permissions(&fsmonitor, fs::Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(&clean, fs::Permissions::from_mode(0o700)).unwrap();

        fs::write(
            fixture.root().join(".gitattributes"),
            "tracked.txt filter=harp=unsafe\n",
        )
        .unwrap();
        git(fixture.root(), &["add", ".gitattributes"]);
        git(
            fixture.root(),
            &["commit", "-q", "-m", "add filter attributes"],
        );
        git(
            fixture.root(),
            &["config", "core.fsmonitor", fsmonitor.to_str().unwrap()],
        );
        git(
            fixture.root(),
            &[
                "config",
                "filter.harp=unsafe.clean",
                clean.to_str().unwrap(),
            ],
        );
        fs::remove_file(&fsmonitor_marker).ok();
        fs::remove_file(&clean_marker).ok();
        fs::write(fixture.root().join("tracked.txt"), "filtered change\n").unwrap();

        let index_before = fs::read(fixture.root().join(".git/index")).unwrap();
        let first = RepositorySnapshot::capture(fixture.root()).unwrap();
        let second = RepositorySnapshot::capture(fixture.root()).unwrap();
        let index_after = fs::read(fixture.root().join(".git/index")).unwrap();

        assert_eq!(first, second);
        assert!(first.dirty);
        assert_eq!(first.tracked_count, 1);
        assert_eq!(first.untracked_count, 0);
        assert_eq!(index_before, index_after);
        assert!(!fsmonitor_marker.exists());
        assert!(!clean_marker.exists());
    }

    #[test]
    fn snapshot_diffs_captured_head_and_rejects_head_change() {
        let first_head = "a".repeat(40);
        let second_head = "b".repeat(40);
        let runner = FakeRunner::with_results([
            output(b"/private/repository/.git\n"),
            output(format!("{first_head}\n").as_bytes()),
            output(b""),
            output(b""),
            output(b""),
            output(format!("{second_head}\n").as_bytes()),
        ]);

        let error = capture_with_runner_and_limit(Path::new("/target/repository"), &runner, 4096)
            .unwrap_err();

        assert_eq!(error.code(), "repository.concurrent_change");
        let invocations = runner.invocations.borrow();
        assert!(invocations[4].arguments.contains(&first_head));
        assert!(!invocations[4]
            .arguments
            .iter()
            .any(|argument| argument == "HEAD"));
        assert_eq!(
            invocations[5].arguments,
            ["rev-parse", "--verify", "HEAD^{commit}"].map(str::to_owned)
        );
    }

    #[cfg(unix)]
    #[test]
    fn real_output_limit_kills_and_reaps_process_with_both_readers_joined() {
        use std::os::unix::fs::PermissionsExt;
        use std::time::{Duration, Instant};

        let fixture = tempdir().unwrap();
        let executable = fixture.path().join("noisy-git.sh");
        let pid_file = fixture.path().join("pid");
        fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s' \"$$\" > \"$HARP_TEST_PID_FILE\"\nwhile :; do\n  printf '0123456789abcdef0123456789abcdef'\n  printf 'fedcba9876543210fedcba9876543210' >&2\ndone\n",
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        let mut invocation = GitInvocation::hardened(fixture.path(), ["ignored".to_owned()]);
        invocation.environment.insert(
            "HARP_TEST_PID_FILE".to_owned(),
            pid_file.to_string_lossy().into_owned(),
        );
        let runner = ProcessGitRunner::for_executable(executable);

        let started = Instant::now();
        let error = runner.run(&invocation, 128).unwrap_err();

        assert_eq!(error.code(), "repository.output_limit");
        assert!(started.elapsed() < Duration::from_secs(2));
        let pid = fs::read_to_string(pid_file)
            .unwrap()
            .parse::<i32>()
            .unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }

    #[test]
    fn porcelain_v2_parser_counts_rename_as_one_tracked_change() {
        let status = b"2 R. N... 100644 100644 100644 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb R100 new name\0old name\0? untracked\0";

        assert_eq!(status_counts(status).unwrap(), (1, 1));
    }

    #[test]
    fn capture_uses_exact_git_commands_and_hardened_environment() {
        let runner = FakeRunner::successful();

        let snapshot =
            capture_with_runner_and_limit(Path::new("/target/repository"), &runner, 4096).unwrap();

        assert_eq!(snapshot.tracked_count, 0);
        let invocations = runner.invocations.borrow();
        assert_eq!(invocations.len(), 6);
        assert_eq!(
            invocations
                .iter()
                .map(|invocation| invocation.arguments.as_slice())
                .collect::<Vec<_>>(),
            [
                ["rev-parse", "--path-format=absolute", "--git-common-dir"]
                    .map(str::to_owned)
                    .as_slice(),
                ["rev-parse", "--verify", "HEAD^{commit}"]
                    .map(str::to_owned)
                    .as_slice(),
                ["config", "--null", "--name-only", "--list", "--includes"]
                    .map(str::to_owned)
                    .as_slice(),
                ["status", "--porcelain=v2", "-z", "--untracked-files=normal"]
                    .map(str::to_owned)
                    .as_slice(),
                [
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--binary",
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "--"
                ]
                .map(str::to_owned)
                .as_slice(),
                ["rev-parse", "--verify", "HEAD^{commit}"]
                    .map(str::to_owned)
                    .as_slice(),
            ]
        );
        for (index, invocation) in invocations.iter().enumerate() {
            assert_eq!(invocation.current_dir, Path::new("/target/repository"));
            let mut expected = BTreeMap::from([
                ("GIT_CONFIG".to_owned(), "/dev/null".to_owned()),
                ("GIT_CONFIG_GLOBAL".to_owned(), "/dev/null".to_owned()),
                ("GIT_CONFIG_NOSYSTEM".to_owned(), "1".to_owned()),
                ("GIT_NO_REPLACE_OBJECTS".to_owned(), "1".to_owned()),
                ("GIT_OPTIONAL_LOCKS".to_owned(), "0".to_owned()),
            ]);
            if index == 2 {
                expected.remove("GIT_CONFIG");
            }
            if matches!(index, 3 | 4) {
                expected.extend([
                    ("GIT_CONFIG_COUNT".to_owned(), "1".to_owned()),
                    ("GIT_CONFIG_KEY_0".to_owned(), "core.fsmonitor".to_owned()),
                    ("GIT_CONFIG_VALUE_0".to_owned(), "false".to_owned()),
                ]);
            }
            assert_eq!(invocation.environment, expected);
        }
        assert_eq!(*runner.limits.borrow(), vec![4096; 6]);
    }

    #[test]
    fn capture_propagates_output_limit_from_bounded_runner() {
        let runner = FakeRunner::with_results([Err(AppError::invalid_input(
            "repository.output_limit",
            "injected output limit",
        ))]);

        let error =
            capture_with_runner_and_limit(Path::new("/target/repository"), &runner, 8).unwrap_err();

        assert_eq!(error.code(), "repository.output_limit");
        assert_eq!(*runner.limits.borrow(), vec![8]);
    }

    #[test]
    fn bounded_reader_stops_before_retaining_excess_output() {
        assert!(matches!(
            read_bounded(&b"123456789"[..], 8).unwrap(),
            BoundedRead::LimitExceeded
        ));
    }

    #[test]
    fn capture_rejects_failed_git_malformed_head_and_status() {
        let failed = FakeRunner::with_results([Ok(GitOutput {
            stdout: Vec::new(),
            stderr: b"not a repository".to_vec(),
            success: false,
        })]);
        assert_eq!(
            capture_with_runner_and_limit(Path::new("/target"), &failed, 4096)
                .unwrap_err()
                .code(),
            "repository.git"
        );

        let malformed_head =
            FakeRunner::with_results([output(b"/target/.git\n"), output(b"not-a-commit\n")]);
        assert_eq!(
            capture_with_runner_and_limit(Path::new("/target"), &malformed_head, 4096)
                .unwrap_err()
                .code(),
            "repository.head"
        );

        let malformed_status = FakeRunner::with_results([
            output(b"/target/.git\n"),
            output(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
            output(b""),
            output(b"x invalid\0"),
            output(b""),
            output(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
        ]);
        assert_eq!(
            capture_with_runner_and_limit(Path::new("/target"), &malformed_status, 4096)
                .unwrap_err()
                .code(),
            "repository.status"
        );
    }

    struct GitFixture {
        workspace: tempfile::TempDir,
        repository: PathBuf,
    }

    impl GitFixture {
        fn new() -> Self {
            let workspace = tempdir().unwrap();
            let repository = workspace.path().join("repository");
            fs::create_dir(&repository).unwrap();
            git(&repository, &["init", "-q"]);
            git(&repository, &["config", "user.name", "Harp Test"]);
            git(
                &repository,
                &["config", "user.email", "harp-test@example.invalid"],
            );
            fs::write(repository.join("tracked.txt"), "initial\n").unwrap();
            git(&repository, &["add", "tracked.txt"]);
            git(&repository, &["commit", "-q", "-m", "initial"]);
            Self {
                workspace,
                repository,
            }
        }

        fn root(&self) -> &Path {
            &self.repository
        }

        fn linked_worktree(&self) -> PathBuf {
            let linked = self.workspace.path().join("linked-worktree");
            git(
                self.root(),
                &[
                    "worktree",
                    "add",
                    "-q",
                    "-b",
                    "linked",
                    linked.to_str().unwrap(),
                    "HEAD",
                ],
            );
            linked
        }
    }

    #[test]
    fn repository_identity_reads_only_the_common_git_directory() {
        let runner = FakeRunner::with_results([output(b"/private/repository/.git\n")]);

        let repository_id =
            repository_id_with_runner_and_limit(Path::new("/target/repository"), &runner, 4096)
                .unwrap();

        assert_eq!(repository_id, sha256_id(b"/private/repository/.git"));
        let invocations = runner.invocations.borrow();
        assert_eq!(invocations.len(), 1);
        assert_eq!(
            invocations[0].arguments,
            ["rev-parse", "--path-format=absolute", "--git-common-dir"].map(str::to_owned)
        );
    }

    fn git(repository: &Path, arguments: &[&str]) {
        let status = Command::new("git")
            .current_dir(repository)
            .args(arguments)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .status()
            .unwrap();
        assert!(status.success(), "git {arguments:?} failed");
    }

    struct FakeRunner {
        results: RefCell<VecDeque<Result<GitOutput, AppError>>>,
        invocations: RefCell<Vec<GitInvocation>>,
        limits: RefCell<Vec<usize>>,
    }

    impl FakeRunner {
        fn successful() -> Self {
            Self::with_results([
                output(b"/private/repository/.git\n"),
                output(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
                output(b""),
                output(b""),
                output(b""),
                output(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"),
            ])
        }

        fn with_results(results: impl IntoIterator<Item = Result<GitOutput, AppError>>) -> Self {
            Self {
                results: RefCell::new(results.into_iter().collect()),
                invocations: RefCell::new(Vec::new()),
                limits: RefCell::new(Vec::new()),
            }
        }
    }

    impl GitRunner for FakeRunner {
        fn run(&self, invocation: &GitInvocation, limit: usize) -> Result<GitOutput, AppError> {
            self.invocations.borrow_mut().push(invocation.clone());
            self.limits.borrow_mut().push(limit);
            self.results
                .borrow_mut()
                .pop_front()
                .expect("fake result for invocation")
        }
    }

    fn output(stdout: &[u8]) -> Result<GitOutput, AppError> {
        Ok(GitOutput {
            stdout: stdout.to_vec(),
            stderr: Vec::new(),
            success: true,
        })
    }
}
