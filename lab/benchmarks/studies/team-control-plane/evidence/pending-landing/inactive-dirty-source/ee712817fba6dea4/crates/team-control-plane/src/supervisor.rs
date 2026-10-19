//! Durable local Git workspaces and independently supervised PID namespaces.
//! No cgroup enforcement is claimed. Unprovable membership holds capacity.
use crate::{auth, storage, *};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tokio_postgres::Transaction;
use uuid::Uuid;
fn unavailable() -> Error {
    crate::domain::err(ErrorCode::TemporarilyUnavailable)
}
fn invalid() -> Error {
    crate::domain::err(ErrorCode::InvalidArgument)
}
#[derive(Clone, Copy, Debug)]
pub enum CreateFault {
    None,
    AfterRegister,
    AfterCreate,
    DuringGit,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    File,
    Directory,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteScope {
    pub path: String,
    pub kind: ScopeKind,
}
#[derive(Clone, Copy, Debug)]
pub enum StartFault {
    None,
    BeforeMonitor,
}
#[derive(Clone)]
pub struct MonitorGate {
    pub claimed: std::sync::Arc<tokio::sync::Notify>,
    pub resume: std::sync::Arc<tokio::sync::Notify>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Registered,
    Preparing,
    Ready,
    Starting,
    Running,
    Cancelling,
    Stopped,
    Quarantined,
    Deleting,
    Released,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionHandle {
    pub id: Uuid,
    pub path: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub start_ticks: u64,
    pub boot_id: String,
    pub namespace: String,
    #[serde(default)]
    pub namespace_pid: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub store: PathBuf,
    pub path: PathBuf,
    pub base: String,
    pub attempt: Uuid,
    pub task: Uuid,
    pub generation: i64,
    pub recovery_epoch: Uuid,
    pub spec_digest: String,
    pub write_scope: Vec<String>,
    pub scopes: Vec<WriteScope>,
    pub log_path: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionView {
    pub state: ExecutionState,
    pub members: Vec<ProcessIdentity>,
    pub controller: Option<ProcessIdentity>,
    pub launcher: Option<ProcessIdentity>,
    pub quarantine_reason: Option<String>,
    pub manifest: WorkspaceManifest,
}
#[derive(Clone, Debug)]
pub struct Supervisor {
    broker: Broker,
    root: PathBuf,
    monitor: PathBuf,
}
pub(crate) async fn migrate(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('supervisor_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        tx.batch_execute(include_str!("../migrations/0005.sql"))
            .await?;
    }
    schema(tx).await
}
async fn schema(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('supervisor_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        return Err(crate::domain::err(ErrorCode::IncompatibleSchema));
    }
    let rows = tx
        .query("SELECT version FROM supervisor_schema_version", &[])
        .await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
        return Err(crate::domain::err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
fn json<T: Serialize>(v: &T) -> Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|_| unavailable())
}
fn decode<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> Result<T> {
    serde_json::from_value(v).map_err(|_| unavailable())
}
async fn authorize(
    tx: &Transaction<'_>,
    broker: &Broker,
    token: &str,
    id: Uuid,
    mutation: bool,
) -> Result<ExecutionView> {
    schema(tx).await?;
    storage::active(tx, broker.epoch).await?;
    let actor = auth::authenticate(tx, token, broker.epoch).await?;
    storage::validate_actor_generation(tx, &actor).await?;
    let row=tx.query_opt("SELECT w.state,w.members,w.controller,w.manifest,t.mission_id,a.generation,a.recovery_epoch,t.generation,w.launcher,a.worker_id,w.quarantine_reason FROM supervised_workspaces w JOIN attempts a ON a.id=w.attempt_id JOIN tasks t ON t.id=a.task_id WHERE w.id=$1 FOR UPDATE OF w",&[&id]).await?.ok_or_else(||crate::domain::err(ErrorCode::NotFound))?;
    let manifest: WorkspaceManifest = decode(row.get(3))?;
    actor.scope(row.get(4), Some(manifest.task))?;
    if actor.role == Role::Worker && actor.principal != row.get::<_, Uuid>(9) {
        return Err(crate::domain::err(ErrorCode::Unauthorized));
    }
    if mutation && actor.role == Role::Verifier {
        return Err(crate::domain::err(ErrorCode::Unauthorized));
    }
    if actor.role != Role::Owner
        && (row.get::<_, i64>(5) != row.get::<_, i64>(7) || row.get::<_, Uuid>(6) != broker.epoch)
    {
        return Err(crate::domain::err(ErrorCode::StaleGeneration));
    }
    Ok(ExecutionView {
        quarantine_reason: row.get(10),
        state: decode(serde_json::Value::String(row.get(0)))?,
        members: decode(row.get(1))?,
        launcher: row
            .get::<_, Option<serde_json::Value>>(8)
            .map(decode)
            .transpose()?,
        controller: row
            .get::<_, Option<serde_json::Value>>(2)
            .map(decode)
            .transpose()?,
        manifest,
    })
}
impl Supervisor {
    pub async fn open(broker: Broker, root: &Path, monitor: &str) -> Result<Self> {
        if !root.is_absolute() || !Path::new(monitor).is_absolute() {
            return Err(invalid());
        }
        let mut c = storage::client(&broker.dsn).await?;
        let tx = c.transaction().await?;
        schema(&tx).await?;
        tx.commit().await?;
        fs::create_dir_all(root).map_err(|_| unavailable())?;
        let root = fs::canonicalize(root).map_err(|_| unavailable())?;
        fs::create_dir_all(root.join(".execution-logs")).map_err(|_| unavailable())?;
        Ok(Self {
            broker,
            root,
            monitor: PathBuf::from(monitor),
        })
    }
    pub async fn create_attempt(
        &self,
        token: &str,
        attempt: Uuid,
        fault: CreateFault,
    ) -> Result<ExecutionHandle> {
        self.create_attempt_with_scopes(token, attempt, Vec::new(), fault)
            .await
    }
    /// Missing admitted paths require an explicit kind. Existing types come from exact Git bytes.
    pub async fn create_attempt_with_scopes(
        &self,
        token: &str,
        attempt: Uuid,
        requested: Vec<WriteScope>,
        fault: CreateFault,
    ) -> Result<ExecutionHandle> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, token, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        if !matches!(actor.role, Role::Owner | Role::Lead | Role::Worker) {
            return Err(crate::domain::err(ErrorCode::Unauthorized));
        }
        let r=tx.query_opt("SELECT a.task_id,a.generation,a.recovery_epoch,a.spec_digest,t.mission_id,t.generation,s.git_repository,s.git_commit,s.write_scope,a.worker_id FROM attempts a JOIN tasks t ON t.id=a.task_id JOIN task_specs s ON s.task_id=a.task_id AND s.digest=a.spec_digest JOIN reservations z ON z.attempt_id=a.id WHERE a.id=$1 AND z.state='reserved'",&[&attempt]).await?.ok_or_else(||crate::domain::err(ErrorCode::NotFound))?;
        let task = r.get(0);
        actor.scope(r.get(4), Some(task))?;
        if actor.role == Role::Worker && actor.principal != r.get::<_, Uuid>(9) {
            return Err(crate::domain::err(ErrorCode::Unauthorized));
        }
        if r.get::<_, i64>(1) != r.get::<_, i64>(5) || r.get::<_, Uuid>(2) != self.broker.epoch {
            return Err(crate::domain::err(ErrorCode::StaleGeneration));
        }
        if tx
            .query_opt(
                "SELECT id FROM supervised_workspaces WHERE attempt_id=$1",
                &[&attempt],
            )
            .await?
            .is_some()
        {
            return Err(crate::domain::err(ErrorCode::IdempotencyConflict));
        }
        let id = Uuid::new_v4();
        let path = self.root.join(id.to_string());
        let store = fs::canonicalize(r.get::<_, String>(6)).map_err(|_| invalid())?;
        let write_scope: Vec<String> = decode(r.get(8))?;
        if write_scope
            .iter()
            .any(|p| !safe_relative(p) || p.split('/').any(|c| c == ".git"))
        {
            return Err(invalid());
        }
        let base: String = r.get(7);
        let scopes = resolve_scopes(&store, &base, &write_scope, requested)?;
        let manifest = WorkspaceManifest {
            store,
            path: path.clone(),
            base,
            attempt,
            task,
            generation: r.get(1),
            recovery_epoch: self.broker.epoch,
            spec_digest: r.get(3),
            write_scope,
            scopes,
            log_path: self.root.join(".execution-logs").join(format!("{id}.log")),
        };
        tx.execute("INSERT INTO supervised_workspaces(id,attempt_id,manifest,state) VALUES($1,$2,$3,'registered')",&[&id,&attempt,&json(&manifest)?]).await?;
        tx.commit().await?;
        if matches!(fault, CreateFault::AfterRegister) {
            return Ok(ExecutionHandle { id, path });
        }
        // Serialize filesystem preparation with deletion/launch reservation. A crash keeps registered state.
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if v.state != ExecutionState::Registered {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        tx.execute(
            "UPDATE supervised_workspaces SET state='preparing' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        // Git is an external preparation workload. Its durable unknown state precedes spawn.
        let prepared = clone_workspace(&manifest, fault).and_then(|_| {
            git_at(&path, &["checkout", "--quiet", "--detach", &manifest.base]).map(|_| ())
        });
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        let current = tx
            .query_one(
                "SELECT generation FROM tasks WHERE id=$1",
                &[&manifest.task],
            )
            .await?
            .get::<_, i64>(0);
        if prepared.is_err()
            || v.state != ExecutionState::Preparing
            || manifest.generation != current
            || manifest.recovery_epoch != self.broker.epoch
        {
            tx.execute(
                "UPDATE supervised_workspaces SET state='quarantined' WHERE id=$1",
                &[&id],
            )
            .await?;
            tx.commit().await?;
            return Err(unavailable());
        }
        // Validate every existing component before creating any missing scope.
        for scope in &manifest.scopes {
            scope_path(&path, scope, false, true)?;
        }
        for scope in &manifest.scopes {
            scope_path(&path, scope, true, false)?;
        }
        fs::File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(|_| unavailable())?;
        if matches!(fault, CreateFault::AfterCreate) {
            tx.execute(
                "UPDATE supervised_workspaces SET state='registered' WHERE id=$1",
                &[&id],
            )
            .await?;
            tx.commit().await?;
            return Ok(ExecutionHandle { id, path });
        }
        tx.execute(
            "UPDATE supervised_workspaces SET state='ready' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        Ok(ExecutionHandle { id, path })
    }
    pub async fn inspect(&self, token: &str, id: Uuid) -> Result<ExecutionView> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        let view = authorize(&tx, &self.broker, token, id, false).await?;
        tx.commit().await?;
        Ok(view)
    }
    pub async fn start(
        &self,
        token: &str,
        id: Uuid,
        argv: Vec<String>,
        seconds: u64,
    ) -> Result<()> {
        self.start_with_fault(token, id, argv, seconds, StartFault::None)
            .await
    }
    pub async fn start_with_fault(
        &self,
        token: &str,
        id: Uuid,
        argv: Vec<String>,
        seconds: u64,
        fault: StartFault,
    ) -> Result<()> {
        if argv.is_empty()
            || argv.iter().any(|s| s.contains('\0'))
            || seconds == 0
            || seconds > 86400
        {
            return Err(invalid());
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        let current = tx
            .query_one(
                "SELECT generation FROM tasks WHERE id=$1",
                &[&v.manifest.task],
            )
            .await?
            .get::<_, i64>(0);
        if v.manifest.generation != current || v.manifest.recovery_epoch != self.broker.epoch {
            return Err(crate::domain::err(ErrorCode::StaleGeneration));
        }
        if v.state != ExecutionState::Ready {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let limit:i64=tx.query_one("SELECT m.deadline_seconds FROM tasks t JOIN missions m ON m.id=t.mission_id WHERE t.id=$1",&[&v.manifest.task]).await?.get(0);
        let seconds = i64::try_from(seconds).map_err(|_| invalid())?.min(limit);
        tx.execute("UPDATE supervised_workspaces SET state='starting',argv=$2,deadline=clock_timestamp()+$3::bigint*interval '1 second' WHERE id=$1",&[&id,&json(&argv)?,&seconds]).await?;
        tx.commit().await?;
        if matches!(fault, StartFault::BeforeMonitor) {
            return Ok(());
        }
        use std::os::unix::process::CommandExt;
        let mut command = Command::new(&self.monitor);
        command
            .arg("supervisor-monitor")
            .arg(id.to_string())
            .env_clear()
            .env("TEAM_CONTROL_PLANE_DSN", &self.broker.dsn)
            .env(
                "TEAM_CONTROL_PLANE_RECOVERY_EPOCH",
                self.broker.epoch.to_string(),
            )
            .env("PATH", "/usr/bin:/bin")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        command.process_group(0);
        let mut child = command.spawn().map_err(|_| unavailable())?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
    pub async fn cancel(&self, token: &str, id: Uuid) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if matches!(v.state, ExecutionState::Released | ExecutionState::Deleting) {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        tx.execute("UPDATE supervised_workspaces SET desired_cancel=true,state=CASE WHEN state IN ('running','starting') THEN 'cancelling' ELSE state END WHERE id=$1",&[&id]).await?;
        tx.commit().await?;
        self.reconcile(token, id).await?;
        Ok(())
    }
    pub async fn reconcile(&self, token: &str, id: Uuid) -> Result<ExecutionView> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        authorize(&tx, &self.broker, token, id, true).await?;
        reconcile_locked(&tx, id).await?;
        tx.commit().await?;
        self.inspect(token, id).await
    }
    /// Retained independently of workspace release. Authenticate every log read.
    pub async fn logs(&self, token: &str, id: Uuid) -> Result<Vec<u8>> {
        let view = self.inspect(token, id).await?;
        fs::read(view.manifest.log_path).map_err(|_| unavailable())
    }
    pub async fn history(&self, token: &str, id: Uuid) -> Result<Vec<ExecutionView>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        let v = authorize(&tx, &self.broker, token, id, false).await?;
        let rows=tx.query("SELECT state,members FROM supervised_observations WHERE execution_id=$1 ORDER BY observed_at,id",&[&id]).await?;
        let mut result = vec![];
        for r in rows {
            result.push(ExecutionView {
                state: decode(serde_json::Value::String(r.get(0)))?,
                members: decode(r.get(1))?,
                controller: v.controller.clone(),
                launcher: v.launcher.clone(),
                quarantine_reason: v.quarantine_reason.clone(),
                manifest: v.manifest.clone(),
            });
        }
        tx.commit().await?;
        Ok(result)
    }
    pub async fn release(&self, token: &str, id: Uuid) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if !matches!(v.state, ExecutionState::Stopped | ExecutionState::Deleting) {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        // Deletion reservation commits before filesystem mutation; starting cannot cross it.
        tx.execute(
            "UPDATE supervised_workspaces SET state='deleting' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        if v.manifest.path.exists() {
            fs::remove_dir_all(&v.manifest.path).map_err(|_| unavailable())?;
        }
        fs::File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(|_| unavailable())?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        authorize(&tx, &self.broker, token, id, true).await?;
        tx.execute(
            "UPDATE supervised_workspaces SET state='released' WHERE id=$1 AND state='deleting'",
            &[&id],
        )
        .await?;
        // Only physically proven stop plus completed deletion refunds the concurrent slot.
        // Attempt/episode admission counters remain consumed.
        tx.execute("WITH released AS (UPDATE reservations SET state='released' WHERE attempt_id=$1 AND state='reserved' RETURNING mission_id) UPDATE missions SET used_capacity=used_capacity-1 WHERE id IN (SELECT mission_id FROM released)",&[&v.manifest.attempt]).await?;
        tx.commit().await?;
        Ok(())
    }
}
fn safe_relative(p: &str) -> bool {
    !p.is_empty()
        && !p.contains('\0')
        && p.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
}
fn resolve_scopes(
    store: &Path,
    base: &str,
    paths: &[String],
    requested: Vec<WriteScope>,
) -> Result<Vec<WriteScope>> {
    let mut explicit = std::collections::HashMap::new();
    for scope in requested {
        if !paths.contains(&scope.path) || explicit.insert(scope.path, scope.kind).is_some() {
            return Err(invalid());
        }
    }
    let mut result = Vec::new();
    for path in paths {
        let mut prefix = String::new();
        let parts: Vec<_> = path.split('/').collect();
        let mut existing = None;
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                prefix.push('/');
            }
            prefix.push_str(part);
            let entry = git_at(store, &["ls-tree", "-z", base, "--", &prefix])?;
            let mode = entry.split_whitespace().next();
            let kind = match mode {
                Some("040000") => Some(ScopeKind::Directory),
                Some("100644" | "100755") => Some(ScopeKind::File),
                None => None,
                _ => return Err(invalid()),
            };
            if i + 1 < parts.len() && kind == Some(ScopeKind::File) {
                return Err(invalid());
            }
            if i + 1 == parts.len() {
                existing = kind;
            }
        }
        let declared = explicit.remove(path);
        let kind = match (existing, declared) {
            (Some(actual), Some(declared)) if actual != declared => return Err(invalid()),
            (Some(actual), _) => actual,
            (None, Some(declared)) => declared,
            (None, None) => return Err(invalid()),
        };
        result.push(WriteScope {
            path: path.clone(),
            kind,
        });
    }
    Ok(result)
}
/// Descriptor-relative traversal: neither validation nor creation follows symlinks.
/// Missing parent directories are prepared by the trusted supervisor, never mounted writable.
fn scope_path(root: &Path, scope: &WriteScope, create: bool, allow_missing: bool) -> Result<()> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::OpenOptionsExt,
        },
    };
    if !safe_relative(&scope.path) {
        return Err(invalid());
    }
    let mut directory = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(root)
        .map_err(|_| invalid())?;
    let parts: Vec<_> = scope.path.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        let name = CString::new(*part).map_err(|_| invalid())?;
        let final_path = i + 1 == parts.len();
        let is_directory = !final_path || scope.kind == ScopeKind::Directory;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if is_directory { libc::O_DIRECTORY } else { 0 };
        // directory is an owned, open directory descriptor; name has no slash or NUL.
        let mut fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(invalid());
            }
            if allow_missing {
                return Ok(());
            }
            if !create {
                return Err(invalid());
            }
            if is_directory {
                // Creation remains relative to the validated parent descriptor.
                if unsafe { libc::mkdirat(directory.as_raw_fd(), name.as_ptr(), 0o700) } < 0 {
                    return Err(invalid());
                }
                fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
            } else {
                fd = unsafe {
                    libc::openat(
                        directory.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_WRONLY
                            | libc::O_CREAT
                            | libc::O_EXCL
                            | libc::O_NOFOLLOW
                            | libc::O_CLOEXEC,
                        0o600,
                    )
                };
            }
            if fd < 0 {
                return Err(invalid());
            }
            directory.sync_all().map_err(|_| unavailable())?;
        }
        // fd is fresh and nonnegative; this File exclusively owns and closes it.
        let file = unsafe { fs::File::from_raw_fd(fd) };
        let metadata = file.metadata().map_err(|_| invalid())?;
        if is_directory && !metadata.is_dir() || !is_directory && !metadata.is_file() {
            return Err(invalid());
        }
        if final_path && create {
            file.sync_all().map_err(|_| unavailable())?;
        }
        directory = file;
    }
    Ok(())
}
fn clone_workspace(manifest: &WorkspaceManifest, fault: CreateFault) -> Result<()> {
    let mut command = Command::new("/usr/bin/git");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["clone", "--no-hardlinks", "--no-local", "--quiet"])
        .arg(&manifest.store)
        .arg(&manifest.path);
    if matches!(fault, CreateFault::DuringGit) {
        use std::os::unix::ffi::OsStrExt;
        let trace = manifest.log_path.with_extension("prepare-trace");
        let name = std::ffi::CString::new(trace.as_os_str().as_bytes()).map_err(|_| invalid())?;
        // A real Git process blocks opening its trace FIFO before any pack/helper work.
        if unsafe { libc::mkfifo(name.as_ptr(), 0o600) } < 0 {
            return Err(unavailable());
        }
        command
            .env("GIT_TRACE2_EVENT", trace)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let mut child = command.spawn().map_err(|_| unavailable())?;
        let marker = manifest.log_path.with_extension("prepare-identity");
        let record = json(&(
            ExecutionHandle {
                id: Uuid::parse_str(
                    manifest
                        .path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(invalid)?,
                )
                .map_err(|_| invalid())?,
                path: manifest.path.clone(),
            },
            identity(child.id()).map_err(|_| unavailable())?,
        ))?;
        let file = fs::File::create(marker).map_err(|_| unavailable())?;
        serde_json::to_writer(&file, &record).map_err(|_| unavailable())?;
        file.sync_all().map_err(|_| unavailable())?;
        if !child.wait().map_err(|_| unavailable())?.success() {
            return Err(unavailable());
        }
        return Ok(());
    }
    if !command
        .output()
        .map_err(|_| unavailable())?
        .status
        .success()
    {
        return Err(unavailable());
    }
    Ok(())
}
fn git(args: &[&str]) -> Result<String> {
    let o = Command::new("/usr/bin/git")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(args)
        .output()
        .map_err(|_| unavailable())?;
    if !o.status.success() {
        return Err(unavailable());
    }
    String::from_utf8(o.stdout).map_err(|_| unavailable())
}
fn git_at(path: &Path, args: &[&str]) -> Result<String> {
    let mut a = vec!["-C", path.to_str().ok_or_else(invalid)?];
    a.extend_from_slice(args);
    git(&a)
}
fn identity(pid: u32) -> std::io::Result<ProcessIdentity> {
    let p = PathBuf::from(format!("/proc/{pid}"));
    let stat = fs::read_to_string(p.join("stat"))?;
    let tail = stat
        .rsplit_once(')')
        .ok_or_else(|| std::io::Error::other("malformed stat"))?
        .1;
    let fields: Vec<_> = tail.split_whitespace().collect();
    let start_ticks = fields
        .get(19)
        .ok_or_else(|| std::io::Error::other("malformed stat"))?
        .parse()
        .map_err(|_| std::io::Error::other("malformed ticks"))?;
    Ok(ProcessIdentity {
        pid,
        start_ticks,
        boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")?
            .trim()
            .into(),
        namespace: fs::read_link(p.join("ns/pid"))?.to_string_lossy().into(),
        namespace_pid: Some(namespace_pid(&fs::read_to_string(p.join("status"))?)?),
    })
}
fn namespace_pid(status: &str) -> std::io::Result<u32> {
    status
        .lines()
        .find_map(|l| l.strip_prefix("NSpid:"))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|p| p.parse().ok())
        .ok_or_else(|| std::io::Error::other("malformed namespace PID"))
}
fn host_start_ticks(stat: &str) -> std::io::Result<u64> {
    let (head, tail) = stat
        .rsplit_once(')')
        .ok_or_else(|| std::io::Error::other("malformed host stat"))?;
    let pid = head
        .split_once('(')
        .and_then(|(pid, _)| pid.trim().parse::<u32>().ok());
    let fields: Vec<_> = tail.split_whitespace().collect();
    if pid.is_none()
        || !fields.first().is_some_and(|state| {
            matches!(
                *state,
                "R" | "S" | "D" | "Z" | "T" | "t" | "X" | "x" | "K" | "W" | "P" | "I"
            )
        })
    {
        return Err(std::io::Error::other("malformed host stat state"));
    }
    fields
        .get(19)
        .and_then(|ticks| ticks.parse().ok())
        .ok_or_else(|| std::io::Error::other("malformed host stat ticks"))
}
fn validate_observer_frame(
    controller: Option<&ProcessIdentity>,
    stat: &str,
    namespace: &str,
    boot: &str,
    procfs: bool,
) -> std::io::Result<()> {
    let controller =
        controller.ok_or_else(|| std::io::Error::other("missing observer frame certificate"))?;
    let pid = stat
        .split_once('(')
        .and_then(|(pid, _)| pid.trim().parse::<u32>().ok());
    if !procfs
        || pid != Some(std::process::id())
        || namespace != controller.namespace
        || boot.trim() != controller.boot_id
    {
        return Err(std::io::Error::other(
            "original observer PID frame cannot be established",
        ));
    }
    host_start_ticks(stat)?;
    Ok(())
}
fn observer_frame(controller: Option<&ProcessIdentity>) -> std::io::Result<()> {
    let mut statfs = std::mem::MaybeUninit::<libc::statfs>::uninit();
    if unsafe { libc::statfs(c"/proc".as_ptr(), statfs.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let procfs = unsafe { statfs.assume_init() }.f_type == libc::PROC_SUPER_MAGIC;
    validate_observer_frame(
        controller,
        &fs::read_to_string("/proc/self/stat")?,
        &fs::read_link("/proc/self/ns/pid")?.to_string_lossy(),
        &fs::read_to_string("/proc/sys/kernel/random/boot_id")?,
        procfs,
    )
}
// Only the certified original PID1 can establish namespace-wide finalization.
// A missing namespace link or leader zombie state is deliberately insufficient.
fn observe_finalized(
    init: &ProcessIdentity,
    boot: std::io::Result<String>,
    read_stat: impl Fn(u32) -> std::io::Result<String>,
    group_ready: impl FnOnce(u32) -> std::io::Result<bool>,
) -> std::io::Result<bool> {
    if init.namespace_pid != Some(1) {
        return Ok(false);
    }
    if boot?.trim() != init.boot_id {
        return Err(std::io::Error::other("original PID1 boot fence differs"));
    }
    let same = |stat: std::io::Result<String>| -> std::io::Result<bool> {
        match stat {
            Ok(stat) => Ok(host_start_ticks(&stat)? == init.start_ticks),
            Err(e) if process_disappeared(&e) => Ok(false),
            Err(e) => Err(e),
        }
    };
    if !same(read_stat(init.pid))? {
        return Ok(true);
    }
    let ready = match group_ready(init.pid) {
        Ok(ready) => ready,
        Err(e) if process_disappeared(&e) => {
            return Ok(!same(read_stat(init.pid))?);
        }
        Err(e) => return Err(e),
    };
    // Fence pidfd_open against numeric PID replacement before trusting readiness.
    if !same(read_stat(init.pid))? {
        return Ok(true);
    }
    Ok(ready)
}
fn finalized(init: &ProcessIdentity) -> std::io::Result<bool> {
    use std::os::fd::FromRawFd;
    observe_finalized(
        init,
        fs::read_to_string("/proc/sys/kernel/random/boot_id"),
        |pid| fs::read_to_string(format!("/proc/{pid}/stat")),
        |pid| {
            // No PIDFD_THREAD: readiness covers the entire thread group.
            let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
            if raw < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let _fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) };
            let mut poll = libc::pollfd {
                fd: raw,
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut poll, 1, 0) } < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if poll.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
                return Err(std::io::Error::other("pidfd readiness observation failed"));
            }
            Ok(poll.revents & libc::POLLIN != 0)
        },
    )
}
fn alive(p: &ProcessIdentity) -> std::io::Result<bool> {
    observe_alive(p, identity, |pid| {
        fs::read_to_string(format!("/proc/{pid}/stat"))
    })
}
fn observe_alive(
    p: &ProcessIdentity,
    read_identity: impl FnOnce(u32) -> std::io::Result<ProcessIdentity>,
    read_stat: impl FnOnce(u32) -> std::io::Result<String>,
) -> std::io::Result<bool> {
    let current = match read_identity(p.pid) {
        Ok(current) => current,
        Err(error) if process_disappeared(&error) => return Ok(false),
        Err(error) => return Err(error),
    };
    if current != *p {
        return Ok(false);
    }
    let stat = match read_stat(p.pid) {
        Ok(stat) => stat,
        Err(error) if process_disappeared(&error) => return Ok(false),
        Err(error) => return Err(error),
    };
    let state = stat
        .rsplit_once(')')
        .and_then(|(_, tail)| tail.split_whitespace().next())
        .ok_or_else(|| std::io::Error::other("malformed process state"))?;
    if !matches!(
        state,
        "R" | "S" | "D" | "Z" | "T" | "t" | "X" | "x" | "K" | "W" | "P" | "I"
    ) {
        return Err(std::io::Error::other("unknown process state"));
    }
    Ok(state != "Z")
}
// The monitor keeps this descriptor until shutdown proof is complete. Namespace
// inode numbers otherwise may be reused immediately after short-lived PID1 exits.
fn pin_namespace(init: &ProcessIdentity) -> std::io::Result<fs::File> {
    use std::os::unix::fs::MetadataExt;
    let path = format!("/proc/{}/ns/pid", init.pid);
    let pin = fs::File::open(&path)?;
    let pinned = pin.metadata()?;
    let current = fs::metadata(&path)?;
    if pinned.dev() != current.dev()
        || pinned.ino() != current.ino()
        || init.namespace != format!("pid:[{}]", pinned.ino())
        || identity(init.pid)? != *init
        || !alive(init)?
    {
        return Err(std::io::Error::other("namespace pin identity mismatch"));
    }
    Ok(pin)
}
fn process_disappeared(error: &std::io::Error) -> bool {
    // /proc task reads can return ESRCH after enumeration while the inode remains.
    // Both errors prove disappearance at that read, unlike permission or other failures.
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH)
}
fn inventory(namespace: &str) -> std::io::Result<Vec<ProcessIdentity>> {
    use std::os::unix::fs::MetadataExt;
    let mut members = vec![];
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) if process_disappeared(&e) => continue,
            Err(e) => {
                return Err(std::io::Error::other(format!(
                    "process {pid} membership observation failed: {e}"
                )));
            }
        };
        // All descendants retain this host uid: user namespace is fixed and caps dropped.
        if meta.uid() != unsafe { libc::geteuid() } {
            continue;
        }
        // NSpid is scoped to the host /proc mount. A single component proves the
        // process is outside every child PID namespace, even if ptrace hides ns links.
        let status = match fs::read_to_string(entry.path().join("status")) {
            Ok(s) => s,
            Err(e) if process_disappeared(&e) => continue,
            Err(e) => {
                return Err(std::io::Error::other(format!(
                    "process {pid} membership observation failed: {e}"
                )));
            }
        };
        let nspid = status.lines().find_map(|l| l.strip_prefix("NSpid:"));
        if nspid.is_some_and(|n| n.split_whitespace().count() == 1) {
            continue;
        }
        if nspid.is_none() {
            return Err(std::io::Error::other(format!(
                "process {pid} lacks readable NSpid"
            )));
        }

        match identity(pid) {
            Ok(p) if p.namespace == namespace => {
                if alive(&p)? {
                    members.push(p);
                }
            }
            Ok(_) => {}
            Err(e) if process_disappeared(&e) => {}
            Err(e) => {
                return Err(std::io::Error::other(format!(
                    "process {pid} membership observation failed: {e}"
                )));
            }
        }
    }
    Ok(members)
}
fn signal(p: &ProcessIdentity) -> Result<()> {
    use std::os::fd::{AsRawFd, FromRawFd};
    if !alive(p).map_err(|_| unavailable())? {
        return Ok(());
    }
    // Open then re-check: PID reuse between observation and pidfd_open cannot target a replacement.
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, p.pid, 0) } as i32;
    if raw < 0 {
        return if !alive(p).map_err(|_| unavailable())? {
            Ok(())
        } else {
            Err(unavailable())
        };
    }
    let fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) };
    if !alive(p).map_err(|_| unavailable())? {
        return Ok(());
    }
    if unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd.as_raw_fd(),
            libc::SIGKILL,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    } < 0
        && alive(p).map_err(|_| unavailable())?
    {
        return Err(unavailable());
    }
    Ok(())
}
trait NamespaceObserver {
    type Pin;
    fn frame(&self, controller: Option<&ProcessIdentity>) -> std::io::Result<()>;
    fn alive(&self, process: &ProcessIdentity) -> std::io::Result<bool>;
    fn pin(&self, process: &ProcessIdentity) -> std::io::Result<Self::Pin>;
    fn finalized(&self, process: &ProcessIdentity) -> std::io::Result<bool>;
    fn inventory(&self, namespace: &str) -> std::io::Result<Vec<ProcessIdentity>>;
    fn signal(&self, process: &ProcessIdentity) -> std::io::Result<()>;
}
struct ProcObserver;
impl NamespaceObserver for ProcObserver {
    type Pin = fs::File;
    fn frame(&self, controller: Option<&ProcessIdentity>) -> std::io::Result<()> {
        observer_frame(controller)
    }
    fn alive(&self, process: &ProcessIdentity) -> std::io::Result<bool> {
        alive(process)
    }
    fn pin(&self, process: &ProcessIdentity) -> std::io::Result<Self::Pin> {
        pin_namespace(process)
    }
    fn finalized(&self, process: &ProcessIdentity) -> std::io::Result<bool> {
        finalized(process)
    }
    fn inventory(&self, namespace: &str) -> std::io::Result<Vec<ProcessIdentity>> {
        inventory(namespace)
    }
    fn signal(&self, process: &ProcessIdentity) -> std::io::Result<()> {
        signal(process)
            .map_err(|_| std::io::Error::other("pidfd signal of recorded membership failed"))
    }
}
fn decide_namespace<O: NamespaceObserver>(
    init: &ProcessIdentity,
    prior: &[ProcessIdentity],
    controller: Option<&ProcessIdentity>,
    cancel: bool,
    held_pin: Option<&O::Pin>,
    observer: &O,
) -> std::io::Result<(&'static str, Vec<ProcessIdentity>)> {
    observer.frame(controller)?;
    if observer.finalized(init)? {
        return Ok(("stopped", vec![]));
    }
    let mut owned_pin = None;
    if held_pin.is_none() {
        for process in std::iter::once(init)
            .chain(prior.iter())
            .filter(|p| p.namespace == init.namespace)
        {
            if observer.alive(process)? {
                match observer.pin(process) {
                    Ok(pin) => {
                        owned_pin = Some(pin);
                        break;
                    }
                    Err(e) if process_disappeared(&e) => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }
    let controller_lost = match controller {
        Some(controller) => !observer.alive(controller)?,
        None => true,
    };
    if held_pin.is_none() && owned_pin.is_none() {
        if observer.finalized(init)? {
            return Ok(("stopped", vec![]));
        }
        if !controller_lost {
            // Defer without inventory, stop classification or signals. The monitor
            // may finish using its explicitly passed gated-launch descriptor.
            return Ok(("running", prior.to_vec()));
        }
        return Err(std::io::Error::other(
            "namespace incarnation lacks certified finalization or verified pin",
        ));
    }
    // Both the owned external descriptor and borrowed monitor descriptor outlive
    // inventory and every signal; a cached identity boolean is never authority.
    let members = observer.inventory(&init.namespace)?;
    if members.is_empty() {
        return Ok(("stopped", members));
    }
    if cancel || controller_lost {
        for member in &members {
            observer.signal(member)?;
        }
        Ok(("cancelling", members))
    } else {
        Ok(("running", members))
    }
}
async fn reconcile_locked(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    reconcile_with_pin(tx, id, None).await
}
async fn reconcile_with_pin(
    tx: &Transaction<'_>,
    id: Uuid,
    held_pin: Option<&fs::File>,
) -> Result<()> {
    let r=tx.query_one("SELECT state,desired_cancel OR deadline<=clock_timestamp(),init_process,members,controller FROM supervised_workspaces WHERE id=$1 FOR UPDATE",&[&id]).await?;
    let state: String = r.get(0);
    let cancel=r.get::<_,Option<bool>>(1).unwrap_or(false)||tx.query_one("SELECT a.generation<>t.generation OR a.recovery_epoch<>d.active_epoch FROM supervised_workspaces w JOIN attempts a ON a.id=w.attempt_id JOIN tasks t ON t.id=a.task_id CROSS JOIN deployment d WHERE w.id=$1",&[&id]).await?.get::<_,bool>(0);
    if matches!(
        state.as_str(),
        "released" | "deleting" | "stopped" | "quarantined"
    ) {
        return Ok(());
    }
    let init = r
        .get::<_, Option<serde_json::Value>>(2)
        .map(decode::<ProcessIdentity>)
        .transpose()?;
    let prior: Vec<ProcessIdentity> = decode(r.get(3))?;
    let controller = r
        .get::<_, Option<serde_json::Value>>(4)
        .map(decode::<ProcessIdentity>)
        .transpose()?;
    let mut reason = None;
    let (next, members) = if let Some(init) = init {
        if let Some(pin) = held_pin {
            use std::os::unix::fs::MetadataExt;
            if format!("pid:[{}]", pin.metadata().map_err(|_| unavailable())?.ino())
                != init.namespace
            {
                return Err(unavailable());
            }
        }
        match decide_namespace(
            &init,
            &prior,
            controller.as_ref(),
            cancel,
            held_pin,
            &ProcObserver,
        ) {
            Ok(decision) => decision,
            Err(error) => {
                reason = Some(error.to_string());
                ("quarantined", prior)
            }
        }
    } else if state == "registered" || state == "ready" {
        if cancel || state == "registered" {
            ("stopped", vec![])
        } else {
            ("ready", vec![])
        }
    } else {
        reason = Some(
            if state == "preparing" {
                "Git preparation completion and full membership are unknown"
            } else {
                "launch identity and full membership are unknown"
            }
            .into(),
        );
        ("quarantined", prior)
    };
    if next != state || json(&members)? != r.get::<_, serde_json::Value>(3) {
        tx.execute("INSERT INTO supervised_observations(id,execution_id,state,members) VALUES($1,$2,$3,$4)",&[&Uuid::new_v4(),&id,&next,&json(&members)?]).await?;
    }
    tx.execute("UPDATE supervised_workspaces SET state=$2,members=$3,desired_cancel=desired_cancel OR $4,quarantine_reason=$5 WHERE id=$1",&[&id,&next,&json(&members)?,&cancel,&reason]).await?;
    Ok(())
}
/// Trusted local operator entry point. DSN comes from the environment, never argv/logs.
/// Workload launch is held by a pipe until its complete namespace identity is durable.
async fn monitor(broker: Broker, id: Uuid, gate: Option<MonitorGate>) -> Result<()> {
    use std::{
        io::Write,
        os::fd::AsRawFd,
        os::unix::{fs::OpenOptionsExt, process::CommandExt},
        process::Stdio,
    };
    let mut c = storage::client(&broker.dsn).await?;
    let tx = c.transaction().await?;
    storage::lock(&tx).await?;
    schema(&tx).await?;
    storage::active(&tx, broker.epoch).await?;
    let r=tx.query_one("SELECT w.state,w.manifest,w.argv,w.desired_cancel,w.controller,a.generation,t.generation,a.recovery_epoch FROM supervised_workspaces w JOIN attempts a ON a.id=w.attempt_id JOIN tasks t ON t.id=a.task_id WHERE w.id=$1 FOR UPDATE OF w",&[&id]).await?;
    if r.get::<_, String>(0) != "starting"
        || r.get::<_, bool>(3)
        || r.get::<_, Option<serde_json::Value>>(4).is_some()
        || r.get::<_, i64>(5) != r.get::<_, i64>(6)
        || r.get::<_, Uuid>(7) != broker.epoch
    {
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    let manifest: WorkspaceManifest = decode(r.get(1))?;
    let argv: Vec<String> = decode(r.get(2))?;
    let controller = identity(std::process::id()).map_err(|_| unavailable())?;
    observer_frame(Some(&controller)).map_err(|_| unavailable())?;
    tx.execute(
        "UPDATE supervised_workspaces SET controller=$2 WHERE id=$1",
        &[&id, &json(&controller)?],
    )
    .await?;
    tx.commit().await?;
    if let Some(gate) = gate {
        gate.claimed.notify_one();
        gate.resume.notified().await;
    }
    // Durable controller claim precedes bwrap spawn. A repeated monitor cannot relaunch.
    let tx = c.transaction().await?;
    storage::lock(&tx).await?;
    schema(&tx).await?;
    storage::active(&tx, broker.epoch).await?;
    let row=tx.query_one("SELECT w.state,w.desired_cancel,w.deadline>clock_timestamp(),a.generation,t.generation,a.recovery_epoch FROM supervised_workspaces w JOIN attempts a ON a.id=w.attempt_id JOIN tasks t ON t.id=a.task_id WHERE w.id=$1 FOR UPDATE OF w",&[&id]).await?;
    if row.get::<_, String>(0) != "starting"
        || row.get::<_, bool>(1)
        || !row.get::<_, Option<bool>>(2).unwrap_or(false)
        || row.get::<_, i64>(3) != row.get::<_, i64>(4)
        || row.get::<_, Uuid>(5) != broker.epoch
    {
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    let info_path = manifest.log_path.with_extension("info");
    let info = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&info_path)
        .map_err(|_| unavailable())?;
    // Only this descriptor crosses exec. File is outside all admitted writable mounts.
    if unsafe { libc::fcntl(info.as_raw_fd(), libc::F_SETFD, 0) } < 0 {
        return Err(unavailable());
    }
    let log = fs::OpenOptions::new()
        .append(true)
        .create_new(true)
        .mode(0o600)
        .open(&manifest.log_path)
        .map_err(|_| unavailable())?;
    let mut command = Command::new("/usr/bin/bwrap");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .args([
            "--unshare-user",
            "--unshare-pid",
            "--unshare-net",
            "--unshare-ipc",
            "--unshare-uts",
            "--disable-userns",
            "--new-session",
            "--die-with-parent",
            "--cap-drop",
            "ALL",
            "--ro-bind",
            "/usr",
            "/usr",
            "--symlink",
            "usr/bin",
            "/bin",
            "--symlink",
            "usr/lib",
            "/lib",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/tmp",
            "--ro-bind",
        ])
        .arg(&manifest.path)
        .arg("/workspace");
    for scope in &manifest.scopes {
        scope_path(&manifest.path, scope, false, false)?;
        command
            .arg("--bind")
            .arg(manifest.path.join(&scope.path))
            .arg(format!("/workspace/{}", scope.path));
    }
    command
        .args(["--chdir", "/workspace", "--block-fd", "0", "--info-fd"])
        .arg(info.as_raw_fd().to_string())
        .arg("--")
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(log.try_clone().map_err(|_| unavailable())?))
        .stderr(Stdio::from(log));
    command.process_group(0);
    let mut child = command.spawn().map_err(|_| unavailable())?;
    // Close descriptor in the parent; the child inherited its duplicate.
    drop(info);
    let mut init = None;
    for _ in 0..100 {
        if let Ok(bytes) = fs::read(&info_path)
            && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && let Some(pid) = v["child-pid"].as_u64()
        {
            init = Some(identity(pid as u32).map_err(|_| unavailable())?);
            break;
        }
        if child.try_wait().map_err(|_| unavailable())?.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let Some(init) = init else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(unavailable());
    };
    if init.namespace == controller.namespace || init.namespace_pid != Some(1) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(unavailable());
    }
    // Acquire and verify the incarnation pin before unblocking any workload.
    let namespace_pin = match pin_namespace(&init) {
        Ok(pin) => pin,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(unavailable());
        }
    };
    // Time can expire while bwrap prepares its blocked namespace. Recheck before eligibility.
    let live:bool=tx.query_one("SELECT NOT desired_cancel AND deadline>clock_timestamp() FROM supervised_workspaces WHERE id=$1",&[&id]).await?.get(0);
    if !live {
        let _ = child.kill();
        let _ = child.wait();
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    tx.execute(
        "UPDATE supervised_workspaces SET init_process=$2,members=$3,launcher=$4,state='running' WHERE id=$1",
        &[&id, &json(&init)?, &json(&vec![init.clone()])?, &json(&identity(child.id()).map_err(|_|unavailable())?)?],
    )
    .await?;
    tx.commit().await?;
    // The workload can only start after this commit. Pipe closure/failure cannot declare stop.
    child
        .stdin
        .take()
        .ok_or_else(unavailable)?
        .write_all(b"1")
        .map_err(|_| unavailable())?;
    loop {
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        // Recovery epoch rotation uses the same cancellation path.
        if storage::active(&tx, broker.epoch).await.is_err() {
            tx.execute(
                "UPDATE supervised_workspaces SET desired_cancel=true WHERE id=$1",
                &[&id],
            )
            .await?;
        }
        reconcile_with_pin(&tx, id, Some(&namespace_pin)).await?;
        let state: String = tx
            .query_one(
                "SELECT state FROM supervised_workspaces WHERE id=$1",
                &[&id],
            )
            .await?
            .get(0);
        tx.commit().await?;
        if state == "stopped" || state == "quarantined" {
            break;
        }
        let _ = child.try_wait();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    // Reap launcher before releasing the verified namespace incarnation pin.
    let _ = child.kill();
    let _ = child.wait();
    drop(namespace_pin);
    fs::File::open(&manifest.log_path)
        .and_then(|f| f.sync_all())
        .map_err(|_| unavailable())?;
    fs::File::open(&manifest.path)
        .and_then(|f| f.sync_all())
        .map_err(|_| unavailable())?;
    Ok(())
}

pub async fn run_monitor(broker: Broker, id: Uuid) -> Result<()> {
    monitor(broker, id, None).await
}
pub async fn run_monitor_with_gate(broker: Broker, id: Uuid, gate: MonitorGate) -> Result<()> {
    monitor(broker, id, Some(gate)).await
}

#[cfg(test)]
mod observation_tests {
    use super::*;
    #[test]
    fn incarnation_replacement_between_proof_and_inventory_never_signals_new_members() {
        use std::cell::Cell;
        struct Replacement {
            newer: ProcessIdentity,
            replaced: Cell<bool>,
            signals: Cell<usize>,
        }
        impl NamespaceObserver for Replacement {
            type Pin = ();
            fn frame(&self, _: Option<&ProcessIdentity>) -> std::io::Result<()> {
                Ok(())
            }
            fn alive(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(true)
            }
            fn pin(&self, _: &ProcessIdentity) -> std::io::Result<()> {
                // Original init and sole monitor pin disappear before acquisition.
                self.replaced.set(true);
                Err(std::io::Error::other(
                    "original namespace incarnation replaced",
                ))
            }
            fn finalized(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(false)
            }
            fn inventory(&self, _: &str) -> std::io::Result<Vec<ProcessIdentity>> {
                self.replaced.set(true);
                Ok(vec![self.newer.clone()])
            }
            fn signal(&self, _: &ProcessIdentity) -> std::io::Result<()> {
                assert!(self.replaced.get());
                self.signals.set(self.signals.get() + 1);
                Ok(())
            }
        }
        let old = identity(std::process::id()).unwrap();
        let mut newer = old.clone();
        newer.pid += 1;
        newer.start_ticks += 1;
        let observer = Replacement {
            newer,
            replaced: Cell::new(false),
            signals: Cell::new(0),
        };
        let decision = decide_namespace(
            &old,
            std::slice::from_ref(&old),
            None,
            true,
            None,
            &observer,
        );
        assert_eq!(
            observer.signals.get(),
            0,
            "replacement must receive no signal"
        );
        assert!(
            decision.is_err(),
            "unproven original incarnation must fail closed"
        );
    }
    #[test]
    fn foreign_or_unproven_observer_frames_cannot_classify_original_pids() {
        let controller = identity(std::process::id()).unwrap();
        let stat = fs::read_to_string("/proc/self/stat").unwrap();
        let check = |stat: &str, ns: &str, boot: &str, procfs| {
            validate_observer_frame(Some(&controller), stat, ns, boot, procfs)
        };
        assert!(check(&stat, &controller.namespace, &controller.boot_id, true).is_ok());
        assert!(check(&stat, "pid:[different]", &controller.boot_id, true).is_err());
        assert!(
            check(
                "1 (foreign) S",
                &controller.namespace,
                &controller.boot_id,
                true
            )
            .is_err()
        );
        assert!(check(&stat, &controller.namespace, "different-boot", true).is_err());
        assert!(check(&stat, &controller.namespace, &controller.boot_id, false).is_err());
        assert!(
            validate_observer_frame(
                None,
                &stat,
                &controller.namespace,
                &controller.boot_id,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn only_certified_original_whole_group_finalization_proves_exit() {
        let mut init = identity(std::process::id()).unwrap();
        init.namespace_pid = Some(1);
        let stat = fs::read_to_string(format!("/proc/{}/stat", init.pid)).unwrap();
        let boot = || Ok(init.boot_id.clone());
        for state in ["S", "D", "Z"] {
            let (head, tail) = stat.rsplit_once(')').unwrap();
            let mut fields: Vec<_> = tail.split_whitespace().map(str::to_owned).collect();
            fields[0] = state.into();
            let observed = format!("{head}) {}", fields.join(" "));
            assert!(
                !observe_finalized(&init, boot(), |_| Ok(observed.clone()), |_| Ok(false)).unwrap()
            );
        }
        for errno in [libc::EACCES, libc::EIO] {
            assert!(
                observe_finalized(
                    &init,
                    boot(),
                    |_| Err(std::io::Error::from_raw_os_error(errno)),
                    |_| unreachable!()
                )
                .is_err()
            );
            assert!(
                observe_finalized(
                    &init,
                    boot(),
                    |_| Ok(stat.clone()),
                    |_| Err(std::io::Error::from_raw_os_error(errno))
                )
                .is_err()
            );
        }
        assert!(
            observe_finalized(
                &init,
                boot(),
                |_| Ok("malformed".into()),
                |_| unreachable!()
            )
            .is_err()
        );
        assert!(
            observe_finalized(
                &init,
                Ok("other-boot".into()),
                |_| unreachable!(),
                |_| unreachable!()
            )
            .is_err()
        );
        assert!(
            observe_finalized(
                &init,
                boot(),
                |_| Err(std::io::Error::from_raw_os_error(libc::ENOENT)),
                |_| unreachable!()
            )
            .unwrap()
        );
        assert!(observe_finalized(&init, boot(), |_| Ok(stat.clone()), |_| Ok(true)).unwrap());
        let (head, tail) = stat.rsplit_once(')').unwrap();
        let mut fields: Vec<_> = tail.split_whitespace().map(str::to_owned).collect();
        fields[19] = (init.start_ticks + 1).to_string();
        let replacement = format!("{head}) {}", fields.join(" "));
        assert!(
            observe_finalized(
                &init,
                boot(),
                |_| Ok(replacement.clone()),
                |_| unreachable!()
            )
            .unwrap()
        );
        let mut missing_certificate = init.clone();
        missing_certificate.namespace_pid = None;
        assert!(
            !observe_finalized(
                &missing_certificate,
                boot(),
                |_| unreachable!(),
                |_| unreachable!()
            )
            .unwrap()
        );
    }
    #[test]
    fn observation_uncertainty_is_never_proof_of_absence() {
        let p = identity(std::process::id()).unwrap();
        for errno in [libc::EACCES, libc::EIO] {
            assert!(
                observe_alive(
                    &p,
                    |_| Err(std::io::Error::from_raw_os_error(errno)),
                    |_| unreachable!()
                )
                .is_err()
            );
            assert!(
                observe_alive(
                    &p,
                    |_| Ok(p.clone()),
                    |_| Err(std::io::Error::from_raw_os_error(errno))
                )
                .is_err()
            );
        }
        assert!(observe_alive(&p, |_| Ok(p.clone()), |_| Ok("malformed".into())).is_err());
        for errno in [libc::ENOENT, libc::ESRCH] {
            assert!(
                !observe_alive(
                    &p,
                    |_| Err(std::io::Error::from_raw_os_error(errno)),
                    |_| unreachable!()
                )
                .unwrap()
            );
            assert!(
                !observe_alive(
                    &p,
                    |_| Ok(p.clone()),
                    |_| Err(std::io::Error::from_raw_os_error(errno))
                )
                .unwrap()
            );
        }
        assert!(!observe_alive(&p, |_| Ok(p.clone()), |_| Ok("1 (zombie) Z".into())).unwrap());
        let mut replacement = p.clone();
        replacement.start_ticks += 1;
        assert!(!observe_alive(&p, |_| Ok(replacement), |_| unreachable!()).unwrap());
    }
}
