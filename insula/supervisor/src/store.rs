use crate::{model::*, Error, Result};
use rusqlite::{params, Connection, OptionalExtension};
use rustix::fs::{flock, FlockOperation, OFlags};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Mutex,
};

/// Locks are never removed: unlinking would permit a second locked inode.
pub struct AuthorityLock {
    file: File,
    path: PathBuf,
}
impl AuthorityLock {
    pub fn acquire(dir: &Path, name: &str, create_dir: bool) -> Result<Self> {
        private_directory(dir, create_dir)?;
        let path = dir.join(name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags((OFlags::NOFOLLOW | OFlags::CLOEXEC).bits() as i32)
            .open(&path)?;
        let m = file.metadata()?;
        if !m.is_file()
            || m.uid() != rustix::process::geteuid().as_raw()
            || m.mode() & 0o077 != 0
            || m.nlink() != 1
        {
            return Err(Error::Ownership);
        }
        flock(&file, FlockOperation::NonBlockingLockExclusive).map_err(|_| Error::Ownership)?;
        let result = Self { file, path };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<()> {
        let named = fs::symlink_metadata(&self.path).map_err(|_| Error::Ownership)?;
        let held = self.file.metadata().map_err(|_| Error::Ownership)?;
        if !named.is_file()
            || named.dev() != held.dev()
            || named.ino() != held.ino()
            || held.nlink() != 1
            || named.uid() != rustix::process::geteuid().as_raw()
            || named.mode() & 0o077 != 0
        {
            return Err(Error::Ownership);
        }
        private_directory(self.path.parent().ok_or(Error::Ownership)?, false)
    }
}

pub fn private_directory(path: &Path, create: bool) -> Result<()> {
    if !path.is_absolute() {
        return Err(Error::Invalid(
            "state and authority directories must be absolute".into(),
        ));
    }
    if create && !path.exists() {
        DirBuilder::new().recursive(true).mode(0o700).create(path)?;
    }
    // Reject symlink aliases; checking the directory alone misses symlink ancestors.
    let mut prefix = PathBuf::new();
    for c in path.components() {
        if matches!(c, std::path::Component::ParentDir) {
            return Err(Error::Ownership);
        }
        prefix.push(c);
        if fs::symlink_metadata(&prefix)?.file_type().is_symlink() {
            return Err(Error::Ownership);
        }
    }
    let m = fs::symlink_metadata(path)?;
    if !m.is_dir()
        || m.uid() != rustix::process::geteuid().as_raw()
        || m.permissions().mode() & 0o077 != 0
    {
        return Err(Error::Ownership);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    Launch,
    Stop,
    RestartStop,
    RestartLaunch,
    AutomaticStop,
    AutomaticLaunch,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transition {
    pub request: OperationRequest,
    pub phase: Phase,
    pub replacement: Option<Generation>,
    /// Separate immutable owner operations for each restart phase. Reusing one
    /// ID for both stop and launch would conflict at an idempotent owner.
    pub owner_launch_id: OperationId,
    pub owner_stop_id: OperationId,
    pub controller_epoch: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredService {
    pub spec: ServiceSpec,
    pub status: ServiceStatus,
    pub pending: Option<Transition>,
    pub readiness_failures: u32,
    pub health_failures: u32,
    pub readiness_sequence: u64,
    pub health_sequence: u64,
    pub next_readiness_ms: u64,
    pub next_health_ms: u64,
}
impl StoredService {
    pub fn new(spec: ServiceSpec) -> Result<Self> {
        let status = ServiceStatus {
            key: spec.key.clone(),
            revision: spec.revision.clone(),
            config_digest: digest(&spec)?,
            generation: None,
            desired: DesiredState::Stopped,
            lifecycle: Lifecycle::Absent,
            ready: false,
            health: Health::Unknown,
            identity: None,
            last_exit: None,
            last_readiness: None,
            last_probe: None,
            next_recovery: RecoveryAction::None,
            automatic_restarts: 0,
            started_ms: None,
            pending_operation: None,
        };
        Ok(Self {
            spec,
            status,
            pending: None,
            readiness_failures: 0,
            health_failures: 0,
            readiness_sequence: 0,
            health_sequence: 0,
            next_readiness_ms: 0,
            next_health_ms: 0,
        })
    }
}

pub struct Store {
    conn: Mutex<Connection>,
    _lock: AuthorityLock,
}
impl Store {
    pub fn open(dir: &Path, root: &str, owner_instance: uuid::Uuid) -> Result<Self> {
        let lock = AuthorityLock::acquire(dir, "store.lock", true)?;
        let path = dir.join("supervisor.sqlite3");
        // Precreate safely before SQLite opens it. Parent is owner-only and locked.
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags((OFlags::NOFOLLOW | OFlags::CLOEXEC).bits() as i32)
            .open(&path)?;
        if file.metadata()?.nlink() != 1 {
            return Err(Error::Ownership);
        }
        for suffix in ["-wal", "-shm", "-journal"] {
            let p = dir.join(format!("supervisor.sqlite3{suffix}"));
            if let Ok(m) = fs::symlink_metadata(p) {
                if !m.is_file() || m.nlink() != 1 || m.uid() != rustix::process::geteuid().as_raw()
                {
                    return Err(Error::Ownership);
                }
            }
        }
        let conn = Connection::open(&path)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS services(insula_id TEXT NOT NULL, service_id TEXT NOT NULL, body TEXT NOT NULL, PRIMARY KEY(insula_id,service_id));
            CREATE TABLE IF NOT EXISTS operations(id TEXT PRIMARY KEY, body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS events(sequence INTEGER PRIMARY KEY AUTOINCREMENT, body TEXT NOT NULL);")?;
        for (key, value) in [
            ("schema_version", "1".to_owned()),
            ("root_insula_id", root.to_owned()),
            ("owner_instance", owner_instance.to_string()),
        ] {
            let old: Option<String> = conn
                .query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                    r.get(0)
                })
                .optional()?;
            match old {
                Some(old) if old != value => {
                    return Err(if key == "schema_version" {
                        Error::Unsupported("database schema version".into())
                    } else {
                        Error::Ownership
                    })
                }
                None => {
                    conn.execute(
                        "INSERT INTO metadata(key,value) VALUES(?1,?2)",
                        params![key, value],
                    )?;
                }
                _ => {}
            }
        }
        // WAL inherits SQLite's database mode; enforce the DB mode explicitly too.
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(Self {
            conn: Mutex::new(conn),
            _lock: lock,
        })
    }
    pub fn get(&self, key: &ServiceKey) -> Result<Option<StoredService>> {
        let c = self.conn.lock().map_err(|_| Error::Storage)?;
        let s: Option<String> = c
            .query_row(
                "SELECT body FROM services WHERE insula_id=?1 AND service_id=?2",
                params![key.insula_id, key.service_id],
                |r| r.get(0),
            )
            .optional()?;
        s.map(|s| serde_json::from_str(&s).map_err(|_| Error::Storage))
            .transpose()
    }
    pub fn all(&self) -> Result<Vec<StoredService>> {
        let c = self.conn.lock().map_err(|_| Error::Storage)?;
        let mut q = c.prepare("SELECT body FROM services ORDER BY insula_id,service_id")?;
        let rows = q
            .query_map([], |r| r.get::<_, String>(0))?
            .map(|s| serde_json::from_str(&s?).map_err(|_| Error::Storage))
            .collect();
        rows
    }
    pub fn operation(&self, id: OperationId) -> Result<Option<OperationRecord>> {
        let c = self.conn.lock().map_err(|_| Error::Storage)?;
        let s: Option<String> = c
            .query_row(
                "SELECT body FROM operations WHERE id=?1",
                [id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        s.map(|s| serde_json::from_str(&s).map_err(|_| Error::Storage))
            .transpose()
    }
    /// State, operation outcome and its event commit as one durable transaction.
    /// Operation IDs are immutable bindings, including across distinct services.
    pub fn commit(
        &self,
        row: &StoredService,
        op: Option<&OperationRecord>,
        kind: &str,
    ) -> Result<Event> {
        self._lock.validate()?;
        let mut c = self.conn.lock().map_err(|_| Error::Storage)?;
        let tx = c.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if let Some(op) = op {
            let old: Option<String> = tx
                .query_row(
                    "SELECT body FROM operations WHERE id=?1",
                    [op.operation_id.to_string()],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(old) = old {
                let old: OperationRecord =
                    serde_json::from_str(&old).map_err(|_| Error::Storage)?;
                if old.request_digest != op.request_digest || old.key != op.key {
                    return Err(Error::Conflict(
                        "operation ID reused with a different request".into(),
                    ));
                }
            }
            tx.execute("INSERT INTO operations(id,body) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET body=excluded.body",params![op.operation_id.to_string(),serde_json::to_string(op)?])?;
        }
        tx.execute("INSERT INTO services(insula_id,service_id,body) VALUES(?1,?2,?3) ON CONFLICT(insula_id,service_id) DO UPDATE SET body=excluded.body",params![row.spec.key.insula_id,row.spec.key.service_id,serde_json::to_string(row)?])?;
        let mut event = Event {
            sequence: 0,
            at_ms: now_ms(),
            key: row.spec.key.clone(),
            generation: row.status.generation,
            operation_id: op.map(|o| o.operation_id),
            kind: kind.to_owned(),
            status: row.status.clone(),
        };
        tx.execute(
            "INSERT INTO events(body) VALUES(?1)",
            [serde_json::to_string(&event)?],
        )?;
        event.sequence = tx.last_insert_rowid() as u64;
        // Bounded event retention. Operation idempotency records deliberately persist.
        tx.execute(
            "DELETE FROM events WHERE sequence<=?1",
            [event.sequence.saturating_sub(4096) as i64],
        )?;
        tx.commit()?;
        Ok(event)
    }
    pub fn events(&self, after: u64, limit: usize) -> Result<Vec<Event>> {
        let c = self.conn.lock().map_err(|_| Error::Storage)?;
        let mut q = c.prepare(
            "SELECT sequence,body FROM events WHERE sequence>?1 ORDER BY sequence LIMIT ?2",
        )?;
        if after > i64::MAX as u64 {
            return Ok(vec![]);
        }
        let rows = q
            .query_map(params![after as i64, limit.min(1024) as i64], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .map(|r| {
                let (seq, s) = r?;
                let mut e: Event = serde_json::from_str(&s).map_err(|_| Error::Storage)?;
                e.sequence = seq as u64;
                Ok(e)
            })
            .collect();
        rows
    }
}
