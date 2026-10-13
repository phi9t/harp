//! Supported pinned Kata API adapter. SQLite owns execution; Kata is a visible projection.
//! All external I/O occurs outside SQL locks/transactions. Ambiguous effects retain their IDs.
use crate::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    ffi::{CString, OsStr},
    fs::File,
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::PermissionsExt},
    },
    path::{Component, PathBuf},
    process::Command as ProcessCommand,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
const API_VERSION: &str = "0.24.0";
const SUMMARY_KEY: &str = "corenius.execution";
const MAX_HTTP: usize = 4 * 1024 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    schema_migration: String,
    controller_id: String,
    actor: String,
    api_schema_version: String,
    origin_instance_uid: String,
    origin: String,
    token_file: PathBuf,
    project_id: u64,
    project_uid: String,
    repositories: BTreeMap<String, PathBuf>,
}
pub struct KataRuntime {
    config: Configuration,
    token: String,
    address: SocketAddr,
}
pub(super) fn is_configured(c: &Connection) -> Result<bool> {
    db(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='kata_configuration')",
        [],
        |r| r.get(0),
    ))
}
pub fn requires_configuration(controller: &Controller) -> Result<bool> {
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    is_configured(&c)
}
fn protected(path: &Path) -> Result<Vec<u8>> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| format!("private Kata file: {e}"))?;
    if !meta.is_file() || meta.permissions().mode() & 0o077 != 0 || path.starts_with("/usr") {
        return Err(
            "Kata configuration/token must be private regular files outside worker toolchain"
                .into(),
        );
    }
    std::fs::read(path).map_err(|e| e.to_string())
}
impl Controller {
    /// Trusted service bootstrap only: participants cannot choose endpoint, credentials or paths.
    pub fn configure_kata(&self, path: &Path) -> Result<()> {
        let config: Configuration =
            serde_json::from_slice(&protected(path)?).map_err(|e| e.to_string())?;
        if config.schema_migration != "kata-v1"
            || config.api_schema_version != API_VERSION
            || config.controller_id.is_empty()
            || config.actor.is_empty()
            || config.origin_instance_uid.is_empty()
            || config.project_uid.is_empty()
            || config.project_id == 0
        {
            return Err("explicit kata-v1 extension and exact supported API/deployment/project pins required".into());
        }
        let address: SocketAddr = config
            .origin
            .strip_prefix("http://")
            .ok_or("Kata requires qualified loopback HTTP origin")?
            .parse()
            .map_err(|_| "invalid Kata origin")?;
        if !address.ip().is_loopback() || address.port() == 0 {
            return Err("Kata endpoint must be loopback with fixed port".into());
        }
        for (name, repository) in &config.repositories {
            if name.is_empty()
                || !repository.is_absolute()
                || repository.canonicalize().ok().as_ref() != Some(repository)
            {
                return Err(
                    "registered source repository must have stable canonical absolute identity"
                        .into(),
                );
            }
        }
        let token = String::from_utf8(protected(&config.token_file)?).map_err(|e| e.to_string())?;
        if token.trim().len() < 12 || token.trim().contains(['\r', '\n']) {
            return Err("invalid private Kata token".into());
        }
        let canonical = serde_json::to_string(&config).map_err(|e| e.to_string())?;
        let mut c = self
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
        if is_configured(&tx)? {
            let stored: String = db(tx.query_row(
                "SELECT body FROM kata_configuration WHERE singleton=1",
                [],
                |r| r.get(0),
            ))?;
            if stored != canonical {
                return Err("durable Kata deployment/project/source configuration mismatch".into());
            }
        } else {
            db(tx.execute_batch(include_str!("kata_schema.sql")))?;
            db(tx.execute("INSERT INTO kata_configuration VALUES(1,?1)", [&canonical]))?;
        }
        db(tx.commit())?;
        drop(c);
        let adapter = KataRuntime {
            config,
            token: token.trim().into(),
            address,
        };
        *self.kata.lock().map_err(|_| "Kata adapter lock poisoned")? = Some(Arc::new(adapter));
        Ok(())
    }
    pub fn drain_kata(&self) -> Result<Value> {
        let adapter = self
            .kata
            .lock()
            .map_err(|_| "Kata adapter lock poisoned")?
            .clone();
        match adapter {
            Some(adapter) => adapter.drain(self),
            None => Ok(json!({"configured":false})),
        }
    }
}
fn adapter(controller: &Controller) -> Result<Arc<KataRuntime>> {
    controller
        .kata
        .lock()
        .map_err(|_| "Kata adapter lock poisoned")?
        .clone()
        .ok_or("Kata adapter not configured".into())
}
fn safe_ref(value: &str) -> Result<&str> {
    if value.is_empty()
        || value.len() > 200
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("invalid pinned Kata item UID".into());
    }
    Ok(value)
}
fn now() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64)
}
impl KataRuntime {
    fn request(
        &self,
        method: &str,
        path: &str,
        value: Option<&Value>,
        headers: &[(&str, String)],
    ) -> Result<Value> {
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_millis(200))
            .map_err(|e| format!("Kata transport: {e}"))?;
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_millis(500)))
            .map_err(|e| e.to_string())?;
        let body = value.map(Value::to_string).unwrap_or_default();
        let mut request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
            self.address,
            self.token,
            body.len()
        );
        for (name, value) in headers {
            if value.contains(['\r', '\n']) {
                return Err("invalid Kata header".into());
            }
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str("\r\n");
        request.push_str(&body);
        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("Kata write: {e}"))?;
        let mut bytes = Vec::new();
        let deadline = Instant::now() + Duration::from_millis(500);
        let mut buffer = [0u8; 8192];
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or("Kata whole acknowledgement deadline")?;
            stream
                .set_read_timeout(Some(remaining))
                .map_err(|e| e.to_string())?;
            let count = stream
                .read(&mut buffer)
                .map_err(|e| format!("Kata acknowledgement: {e}"))?;
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..count]);
            if bytes.len() > MAX_HTTP {
                return Err("Kata response limit".into());
            }
        }
        if bytes.len() > MAX_HTTP {
            return Err("Kata response limit".into());
        }
        let split = bytes
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or("Kata lost/malformed acknowledgement")?;
        let header = std::str::from_utf8(&bytes[..split]).map_err(|e| e.to_string())?;
        let status = header
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .ok_or("Kata HTTP status missing")?;
        if status != "200" {
            return Err(format!(
                "Kata HTTP {status}; effect requires reconciliation"
            ));
        }
        let body = &bytes[split + 4..];
        let decoded;
        let body = if header
            .to_ascii_lowercase()
            .contains("transfer-encoding: chunked")
        {
            decoded = chunks(body)?;
            decoded.as_slice()
        } else {
            body
        };
        serde_json::from_slice(body).map_err(|e| format!("Kata JSON acknowledgement: {e}"))
    }
    fn project_path(&self) -> String {
        format!("/api/v1/projects/{}", self.config.project_id)
    }
    fn item_path(&self, item: &str) -> Result<String> {
        Ok(format!(
            "{}/issues/{}",
            self.project_path(),
            safe_ref(item)?
        ))
    }
    fn qualify(&self, item: &str) -> Result<()> {
        let health = self.request("GET", "/api/v1/health", None, &[])?;
        if health["api_schema_version"] != API_VERSION {
            return Err("Kata API version mismatch".into());
        }
        let identity = self.request("GET", "/api/v1/instance", None, &[])?;
        if identity["instance_uid"] != self.config.origin_instance_uid {
            return Err("Kata deployment identity mismatch".into());
        }
        if identity["auth"]["actor"] != self.config.actor {
            return Err("Kata authenticated controller actor mismatch".into());
        }
        let project = self.request("GET", &self.project_path(), None, &[])?;
        if project["project"]["uid"] != self.config.project_uid
            || project["project"]["id"] != self.config.project_id
        {
            return Err("Kata project identity mismatch".into());
        }
        let mut cursor = 0;
        for _ in 0..100 {
            let events = self.request(
                "GET",
                &format!("{}/events?after_id={cursor}", self.project_path()),
                None,
                &[],
            )?;
            let events = events["events"].as_array().ok_or("Kata events missing")?;
            for event in events {
                if event["type"] == "issue.created" && event["issue_uid"] == item {
                    if event["origin_instance_uid"] != self.config.origin_instance_uid {
                        return Err("Kata deployment identity mismatch".into());
                    }
                    return Ok(());
                }
                cursor = event["event_id"]
                    .as_u64()
                    .ok_or("Kata event cursor missing")?;
            }
            if events.is_empty() {
                break;
            }
        }
        Err("Kata creation origin could not be qualified; no effects authorized".into())
    }
    fn read_item(&self, item: &str) -> Result<Value> {
        self.qualify(item)?;
        let value = self.request("GET", &self.item_path(item)?, None, &[])?;
        if value["issue"]["uid"] != item || value["issue"]["project_id"] != self.config.project_id {
            return Err("Kata item identity mismatch".into());
        }
        Ok(value)
    }
    fn capture(&self, item: &str, files: &Value) -> Result<Value> {
        let remote = self.read_item(item)?;
        let mut intent = remote["issue"].clone();
        let object = intent.as_object_mut().ok_or("Kata issue missing")?;
        // These are ledger presentation/revision fields, not admitted human intent.
        for field in [
            "revision",
            "updated_at",
            "created_at",
            "web_url",
            "short_id",
            "id",
        ] {
            object.remove(field);
        }
        if let Some(metadata) = object.get_mut("metadata").and_then(Value::as_object_mut) {
            metadata.remove(SUMMARY_KEY);
        }
        let artifacts = self.capture_files(files)?;
        let work_ref = json!({"deployment":self.config.origin_instance_uid,"project":self.config.project_uid,"item":item});
        let contract =
            json!({"work_ref":work_ref,"intent":intent,"links":remote["links"],"files":artifacts});
        let digest = hash(&contract.to_string());
        Ok(
            json!({"work_ref":work_ref,"revision":remote["issue"]["revision"],"digest":digest,"intent":intent,"links":remote["links"],"files":artifacts,"projection":remote["issue"]["metadata"][SUMMARY_KEY]}),
        )
    }
    fn capture_files(&self, selection: &Value) -> Result<Value> {
        let storage = text(selection, "storage")?;
        let root = self
            .config
            .repositories
            .get(storage)
            .ok_or("unregistered source storage identity")?;
        let mode = text(selection, "mode")?;
        let paths = selection["paths"]
            .as_array()
            .filter(|p| !p.is_empty() && p.len() <= 100)
            .ok_or("all intent file paths required")?;
        let git = |args: &[&str]| -> Result<Vec<u8>> {
            let output = ProcessCommand::new("/usr/bin/git")
                .arg("--no-optional-locks")
                .args([
                    "-c",
                    "core.fsmonitor=false",
                    "-c",
                    "core.untrackedCache=false",
                ])
                .arg("-C")
                .arg(root)
                .args(args)
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .env_remove("GIT_INDEX_FILE")
                .output()
                .map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err("source Git object/status unavailable".into());
            }
            Ok(output.stdout)
        };
        let commit = if mode == "git" {
            commit_oid(text(selection, "commit")?)?.to_owned()
        } else if mode == "snapshot" {
            String::from_utf8(git(&["rev-parse", "HEAD"])?)
                .map_err(|e| e.to_string())?
                .trim()
                .to_owned()
        } else {
            return Err("source_files mode must be git or explicit immutable snapshot".into());
        };
        commit_oid(&commit)?;
        if git(&["cat-file", "-t", &commit])? != b"commit\n" {
            return Err("source version must be an exact Git commit object".into());
        }
        let dirty = !git(&["status", "--porcelain", "--untracked-files=all"])?.is_empty();
        let snapshot_root = (mode == "snapshot")
            .then(|| open_snapshot_root(root))
            .transpose()?;
        let mut artifacts = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for path in paths {
            let path = path.as_str().ok_or("source path must be text")?;
            if path.is_empty()
                || !Path::new(path)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                || !seen.insert(path)
            {
                return Err("source paths must be unique relative regular files".into());
            }
            let content = if mode == "git" {
                git(&["show", &format!("{commit}:{path}")])?
            } else {
                read_snapshot(snapshot_root.as_ref().ok_or("snapshot root missing")?, path)?
            };
            if content.len() > 512 * 1024 {
                return Err("source artifact limit".into());
            }
            artifacts.push(json!({"path":path,"sha256":format!("{:x}",Sha256::digest(&content)),"bytes":content.len(),"content_base64":STANDARD.encode(content)}));
        }
        // Exact git captures are independent of mutable worktree dirt. Snapshot explicitly retains it.
        let manifest = json!({"storage":storage,"mode":mode,"commit":commit,"dirty":mode=="snapshot" && dirty,"artifacts":artifacts});
        Ok(
            json!({"snapshot_digest":hash(&manifest.to_string()),"storage":storage,"mode":mode,"commit":commit,"dirty":mode=="snapshot" && dirty,"artifacts":manifest["artifacts"]}),
        )
    }
}
fn open_relative(directory: &File, name: &OsStr, flags: i32) -> Result<File> {
    let name = CString::new(name.as_bytes()).map_err(|_| "source path contains NUL")?;
    // The directory descriptor stays open across lookup; no symlink is followed.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(format!(
            "source snapshot descriptor lookup refused: {}",
            std::io::Error::last_os_error()
        ));
    }
    // openat returned this owned descriptor, so File is its sole owner.
    Ok(unsafe { File::from_raw_fd(fd) })
}
fn open_snapshot_root(root: &Path) -> Result<File> {
    let mut directory = File::open("/").map_err(|e| e.to_string())?;
    for component in root.components() {
        if let Component::Normal(name) = component {
            directory = open_relative(&directory, name, libc::O_RDONLY | libc::O_DIRECTORY)?;
        }
    }
    Ok(directory)
}
fn read_snapshot(root: &File, path: &str) -> Result<Vec<u8>> {
    let mut directory = root.try_clone().map_err(|e| e.to_string())?;
    let mut parts = Path::new(path).components().peekable();
    while let Some(Component::Normal(name)) = parts.next() {
        if parts.peek().is_some() {
            directory = open_relative(&directory, name, libc::O_RDONLY | libc::O_DIRECTORY)?;
        } else {
            #[cfg(feature = "fault-injection")]
            snapshot_barrier()?;
            let file = open_relative(&directory, name, libc::O_RDONLY | libc::O_NONBLOCK)?;
            if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                return Err("source snapshot regular file required".into());
            }
            let mut content = Vec::new();
            file.take(512 * 1024 + 1)
                .read_to_end(&mut content)
                .map_err(|e| e.to_string())?;
            return Ok(content);
        }
    }
    Err("source snapshot relative file required".into())
}
#[cfg(feature = "fault-injection")]
fn snapshot_barrier() -> Result<()> {
    if let Ok(path) = std::env::var("CORENIUS_TEST_SNAPSHOT_BARRIER")
        && Path::new(&path).is_file()
    {
        std::fs::write(format!("{path}.observed"), b"snapshot lookup boundary")
            .map_err(|e| e.to_string())?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while Path::new(&path).is_file() {
            if Instant::now() >= deadline {
                return Err("snapshot fixture barrier deadline".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    Ok(())
}
fn chunks(mut bytes: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    loop {
        let end = bytes
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("Kata chunk size missing")?;
        let line = std::str::from_utf8(&bytes[..end]).map_err(|e| e.to_string())?;
        let size = usize::from_str_radix(line.split(';').next().unwrap_or(""), 16)
            .map_err(|_| "Kata chunk size invalid")?;
        bytes = &bytes[end + 2..];
        if size == 0 {
            return Ok(result);
        }
        if size > bytes.len().saturating_sub(2) || &bytes[size..size + 2] != b"\r\n" {
            return Err("Kata incomplete chunk".into());
        }
        result.extend_from_slice(&bytes[..size]);
        bytes = &bytes[size + 2..];
    }
}

pub(super) fn prepare_request(controller: &Controller, request: &Request) -> Result<Option<Value>> {
    let Some(command) = &request.command else {
        return Ok(None);
    };
    if !matches!(
        command.kind.as_str(),
        "create_kata_task" | "refresh_kata_source" | "admit"
    ) {
        return Ok(None);
    }
    let source = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &request.auth, &request.project)?;
        authorize(&p, &command.kind, &command.args)?;
        // A repeated command has no external reads, even offline. Core rechecks authority and arguments.
        let exists: bool = db(c.query_row("SELECT EXISTS(SELECT 1 FROM receipts WHERE project=?1 AND principal=?2 AND epoch=?3 AND key=?4)",params![request.project,p.id,p.epoch,command.idempotency_key],|r|r.get(0)))?;
        if exists {
            return Ok(None);
        }
        if command.kind == "create_kata_task" {
            Some((
                text(&command.args, "item")?.to_owned(),
                command.args["source_files"].clone(),
            ))
        } else if is_configured(&c)? {
            let stored: Option<(String, String)> = db(c
                .query_row(
                    "SELECT item,source_files FROM kata_sources WHERE project=?1 AND task=?2",
                    params![request.project, text(&command.args, "task")?],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional())?;
            stored
                .map(|(item, files)| {
                    serde_json::from_str(&files)
                        .map(|files| (item, files))
                        .map_err(|e| e.to_string())
                })
                .transpose()?
        } else {
            None
        }
    };
    match source {
        Some((item, files)) => {
            let source = match adapter(controller)?.capture(&item, &files) {
                Ok(source) => source,
                Err(error) => {
                    record_source_failure(
                        controller,
                        &request.project,
                        text(&command.args, "task")?,
                        &error,
                    )?;
                    return Err(error);
                }
            };
            if command.kind == "admit" {
                let digest: String = {
                    let c = controller
                        .connection
                        .lock()
                        .map_err(|_| "controller lock poisoned")?;
                    db(c.query_row("SELECT v.digest FROM kata_sources s JOIN kata_versions v ON v.id=s.admitted WHERE s.project=?1 AND s.task=?2",params![request.project,text(&command.args,"task")?],|r|r.get(0)))?
                };
                if source["digest"] != digest || source["intent"]["status"] != "open" {
                    record_source_observation(
                        controller,
                        &request.project,
                        text(&command.args, "task")?,
                        &source,
                    )?;
                    return Err(
                        "Kata source amendment/closure retained as proposal; admission refused"
                            .into(),
                    );
                }
            }
            Ok(Some(source))
        }
        None if command.kind == "refresh_kata_source" => Err("linked Kata source missing".into()),
        None => Ok(None),
    }
}
fn record_source_failure(
    controller: &Controller,
    project: &str,
    task: &str,
    error: &str,
) -> Result<()> {
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let prior: Option<Option<String>> = db(c
        .query_row(
            "SELECT drift FROM kata_sources WHERE project=?1 AND task=?2",
            params![project, task],
            |r| r.get(0),
        )
        .optional())?;
    let Some(prior) = prior else {
        return Ok(());
    };
    let drift = format!("source observation unavailable: {error}");
    if prior.as_deref() == Some(drift.as_str()) {
        return Ok(());
    }
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    db(tx.execute(
        "UPDATE kata_sources SET drift=?3 WHERE project=?1 AND task=?2",
        params![project, task, drift],
    ))?;
    source_event(
        &tx,
        project,
        task,
        "kata_source_unavailable",
        json!({"error":error,"state":"unknown","admitted_intent_retained":true}),
    )?;
    db(tx.commit())
}
fn record_source_observation(
    controller: &Controller,
    project: &str,
    task: &str,
    source: &Value,
) -> Result<()> {
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let current: String = db(c.query_row("SELECT v.body FROM kata_sources s JOIN kata_versions v ON v.id=s.current WHERE s.project=?1 AND s.task=?2",params![project,task],|r|r.get(0)))?;
    let current: Value = serde_json::from_str(&current).map_err(|e| e.to_string())?;
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    if current == *source {
        let drift: Option<String> = db(tx.query_row(
            "SELECT drift FROM kata_sources WHERE project=?1 AND task=?2",
            params![project, task],
            |r| r.get(0),
        ))?;
        if drift.as_deref() == projection_drift(&tx, project, task, source)? {
            return Ok(());
        }
    }
    let result = observe_source(&tx, project, task, source)?;
    source_event(
        &tx,
        project,
        task,
        if current == *source {
            "kata_source_recovered"
        } else {
            "kata_source_observed"
        },
        result,
    )?;
    db(tx.commit())
}
fn source_event(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    kind: &str,
    observation: Value,
) -> Result<()> {
    db(tx.execute(
        "UPDATE projects SET revision=revision+1 WHERE id=?1",
        [project],
    ))?;
    let revision: i64 = db(tx.query_row(
        "SELECT revision FROM projects WHERE id=?1",
        [project],
        |r| r.get(0),
    ))?;
    db(tx.execute(
        "INSERT INTO events(project,task,kind,body) VALUES(?1,?2,?3,?4)",
        params![
            project,
            task,
            kind,
            json!({"revision":revision,"observation":observation}).to_string()
        ],
    ))?;
    Ok(())
}

fn retain_version(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    source: &Value,
) -> Result<String> {
    let body = source.to_string();
    let version = hash(&format!("{project}\n{task}\n{body}"));
    db(tx.execute("INSERT OR IGNORE INTO kata_versions(id,project,task,revision,digest,body) VALUES(?1,?2,?3,?4,?5,?6)",params![version,project,task,source["revision"].as_i64().ok_or("Kata source revision missing")?,text(source,"digest")?,body]))?;
    Ok(version)
}
pub(super) fn apply(
    tx: &Transaction<'_>,
    p: &Principal,
    project: &str,
    kind: &str,
    args: &Value,
    source: Option<&Value>,
) -> Result<Value> {
    if !is_configured(tx)? {
        return Err("Kata/participant extension not configured".into());
    }
    let task = text(args, "task")?;
    match kind {
        "create_kata_task" => {
            let source = source.ok_or("controller-owned source observation required")?;
            if args["expected_source_revision"] != source["revision"] {
                return Err("Kata source revision changed; admission refused".into());
            }
            if source["intent"]["status"] != "open" || source["intent"].get("deleted_at").is_some()
            {
                return Err("Kata task closed/revoked/deleted; admission refused".into());
            }
            let item = text(args, "item")?;
            let bound: bool = db(tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM kata_sources WHERE item=?1)",
                [item],
                |r| r.get(0),
            ))?;
            if bound {
                return Err("Kata WorkRef already bound to a controller task, including another local project".into());
            }
            let version = retain_version(tx, project, task, source)?;
            let mut brief_source = source.clone();
            if let Some(artifacts) = brief_source["files"]["artifacts"].as_array_mut() {
                for artifact in artifacts {
                    if let Some(object) = artifact.as_object_mut() {
                        object.remove("content_base64");
                    }
                }
            }
            brief_source["version"] = json!(version);
            let brief_args = json!({"task":task,"work_ref":source["work_ref"],"source_revision":format!("rev-{}",source["revision"]),"brief":brief_source.to_string()});
            let result = crate::apply(tx, p, project, "create_task", &brief_args, Some(source))?;
            db(tx.execute("INSERT INTO kata_sources(project,task,item,source_files,admitted,current) VALUES(?1,?2,?3,?4,?5,?5)",params![project,task,text(args,"item")?,args["source_files"].to_string(),version]))?;
            Ok(json!({"task":task,"brief":result["brief"],"source":source,"version":version}))
        }
        "refresh_kata_source" => {
            let source = source.ok_or("controller-owned source observation required")?;
            observe_source(tx, project, task, source)
        }
        "sync_kata_task" => {
            let exists: bool = db(tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM kata_sources WHERE project=?1 AND task=?2)",
                params![project, task],
                |r| r.get(0),
            ))?;
            if !exists {
                return Err("linked Kata source missing".into());
            }
            Ok(json!({"task":task,"disposition":"queued"}))
        }
        "propose_followup" => {
            let proposal = id();
            db(tx.execute("INSERT INTO followup_proposals(id,project,task,author,text) VALUES(?1,?2,?3,?4,?5)",params![proposal,project,task,p.id,text(args,"text")?]))?;
            Ok(json!({"task":task,"proposal":proposal,"disposition":"proposal-only"}))
        }
        _ => Err("unsupported Kata mutation".into()),
    }
}
/// Select a retained controller capture, never caller-authored linked intent.
pub(super) fn revised_brief(
    c: &Connection,
    project: &str,
    task: &str,
    args: &Value,
) -> Result<Option<String>> {
    if !is_configured(c)? {
        return Ok(None);
    }
    let current: Option<String> = db(c.query_row("SELECT v.body FROM kata_sources s JOIN kata_versions v ON v.id=s.current WHERE s.project=?1 AND s.task=?2", params![project,task], |r|r.get(0)).optional())?;
    let Some(current) = current else {
        return Ok(None);
    };
    let version = text(args, "source_version")?;
    let body: Option<String> = db(c
        .query_row(
            "SELECT body FROM kata_versions WHERE id=?1 AND project=?2 AND task=?3",
            params![version, project, task],
            |r| r.get(0),
        )
        .optional())?;
    let mut source: Value =
        serde_json::from_str(&body.ok_or("captured source version/task missing")?)
            .map_err(|e| e.to_string())?;
    let current: Value = serde_json::from_str(&current).map_err(|e| e.to_string())?;
    if source["digest"] != args["source_digest"]
        || source["digest"] != current["digest"]
        || args["source_revision"] != format!("rev-{}", source["revision"])
        || source["work_ref"] != current["work_ref"]
        || source["intent"]["status"] != "open"
        || source["intent"].get("deleted_at").is_some()
    {
        return Err("captured source revision/digest/WorkRef is mismatched or closed".into());
    }
    if let Some(artifacts) = source["files"]["artifacts"].as_array_mut() {
        for artifact in artifacts {
            artifact
                .as_object_mut()
                .ok_or("captured source artifact missing")?
                .remove("content_base64");
        }
    }
    source["version"] = json!(version);
    if let Some(body) = args.get("body") {
        let supplied: Value = serde_json::from_str(body.as_str().ok_or("source body type")?)
            .map_err(|_| "source body must match captured version")?;
        if supplied != source {
            return Err("source body must match captured version".into());
        }
    }
    Ok(Some(source.to_string()))
}

pub(super) fn approve_revised_brief(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    brief: &str,
) -> Result<()> {
    if !is_configured(tx)? {
        return Ok(());
    }
    let linked: bool = db(tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM kata_sources WHERE project=?1 AND task=?2)",
        params![project, task],
        |r| r.get(0),
    ))?;
    if !linked {
        return Ok(());
    }
    let (body, revision): (String, String) = db(tx.query_row(
        "SELECT body,source_revision FROM briefs WHERE id=?1 AND project=?2 AND task=?3",
        params![brief, project, task],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    let source: Value =
        serde_json::from_str(&body).map_err(|_| "linked brief lacks captured source version")?;
    revised_brief(tx, project, task, &json!({"source_version":source["version"],"source_digest":source["digest"],"source_revision":revision,"body":body}))?.ok_or("linked source version missing")?;
    db(tx.execute(
        "UPDATE kata_sources SET admitted=?3 WHERE project=?1 AND task=?2",
        params![project, task, text(&source, "version")?],
    ))?;
    Ok(())
}

fn observe_source(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    source: &Value,
) -> Result<Value> {
    let admitted: String = db(tx.query_row("SELECT v.digest FROM kata_sources s JOIN kata_versions v ON v.id=s.admitted WHERE s.project=?1 AND s.task=?2",params![project,task],|r|r.get(0)))?;
    let version = retain_version(tx, project, task, source)?;
    let proposed = source["digest"] != admitted;
    if proposed {
        db(tx.execute(
            "INSERT OR IGNORE INTO kata_proposals(id,project,task,version) VALUES(?1,?2,?3,?4)",
            params![id(), project, task, version],
        ))?;
    }
    let drift = projection_drift(tx, project, task, source)?;
    db(tx.execute(
        "UPDATE kata_sources SET current=?3,drift=?4 WHERE project=?1 AND task=?2",
        params![project, task, version, drift],
    ))?;
    Ok(
        json!({"task":task,"version":version,"disposition":if proposed {"proposed"} else {"unchanged"},"drift":drift}),
    )
}
fn projection_drift(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    source: &Value,
) -> Result<Option<&'static str>> {
    let acknowledged: Option<String> = db(tx.query_row(
        "SELECT acknowledged_summary FROM kata_sources WHERE project=?1 AND task=?2",
        params![project, task],
        |r| r.get(0),
    ))?;
    let expected = acknowledged
        .map(|s| serde_json::from_str::<Value>(&s))
        .transpose()
        .map_err(|e| e.to_string())?
        .unwrap_or(Value::Null);
    Ok((source["projection"] != expected).then_some("external execution projection drift"))
}
pub(super) fn check_admission(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    source: Option<&Value>,
) -> Result<()> {
    if !is_configured(tx)? {
        return Ok(());
    }
    let digest: Option<String> = db(tx.query_row("SELECT v.digest FROM kata_sources s JOIN kata_versions v ON v.id=s.admitted WHERE s.project=?1 AND s.task=?2",params![project,task],|r|r.get(0)).optional())?;
    let digest = digest.ok_or("qualified Kata source required; unlinked storage-fixture task cannot be admitted in Kata mode")?;
    {
        let source = source.ok_or("fresh qualified Kata source required for admission")?;
        if source["digest"] != digest || source["intent"]["status"] != "open" {
            return Err("Kata source amendment/closure: refresh proposal before admission; immutable brief retained".into());
        }
    }
    Ok(())
}

/// Publish only controller-derived state, never a participant-selected Kata status.
/// Kata's own open/closed field is intentionally independent of execution acceptance.
fn execution_summary(
    tx: &Transaction<'_>,
    project: &str,
    task: &str,
    revision: i64,
    event: i64,
) -> Result<Value> {
    let (state, brief): (String, String) = db(tx.query_row(
        "SELECT state,brief FROM tasks WHERE project=?1 AND id=?2",
        params![project, task],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    let attempt: Option<(String,String,bool)> = db(tx.query_row("SELECT id,state,cancel_requested FROM attempts WHERE project=?1 AND task=?2 ORDER BY rowid DESC LIMIT 1",params![project,task],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional())?;
    let unanswered: i64 = db(tx.query_row(
        "SELECT COUNT(*) FROM questions WHERE project=?1 AND task=?2 AND answer IS NULL",
        params![project, task],
        |r| r.get(0),
    ))?;
    let visible = match state.as_str() {
        "created" => "unassigned",
        "submitted" => "submitted",
        "accepted" => "accepted",
        "landed" => "landed",
        "aborted" => "aborted",
        "repair-required" | "integration-required" => "blocked",
        "active"
            if unanswered > 0
                || attempt
                    .as_ref()
                    .is_some_and(|(_, state, cancel)| *cancel || state == "unknown") =>
        {
            "blocked"
        }
        "active"
            if attempt
                .as_ref()
                .is_some_and(|(_, state, _)| state == "running") =>
        {
            "running"
        }
        "active" => "assigned",
        _ => return Err("unsupported execution mapping".into()),
    };
    let config: String = db(tx.query_row(
        "SELECT body FROM kata_configuration WHERE singleton=1",
        [],
        |r| r.get(0),
    ))?;
    let config: Value = serde_json::from_str(&config).map_err(|e| e.to_string())?;
    let admitted: String = db(tx.query_row(
        "SELECT admitted FROM kata_sources WHERE project=?1 AND task=?2",
        params![project, task],
        |r| r.get(0),
    ))?;
    let progress: Option<String> = db(tx.query_row("SELECT body FROM events WHERE project=?1 AND task=?2 AND kind='report_progress' ORDER BY seq DESC LIMIT 1",params![project,task],|r|r.get(0)).optional())?;
    let progress = progress
        .map(|s| serde_json::from_str::<Value>(&s))
        .transpose()
        .map_err(|e| e.to_string())?;
    let full_progress = progress.as_ref().and_then(|p| p["args"]["text"].as_str());
    let visible_progress = full_progress.map(|p| excerpt(p, 4096));
    Ok(
        json!({"controller":config["controller_id"],"project":project,"task":task,"controller_revision":revision,"event_seq":event,"state":visible,"task_state":state,"brief":brief,"admitted_version":admitted,"attempt":attempt.as_ref().map(|a|&a.0),"observed_attempt_state":attempt.as_ref().map(|a|&a.1),"cancel_requested":attempt.as_ref().is_some_and(|a|a.2),"unanswered_questions":unanswered,"reported_progress":visible_progress,"reported_progress_bytes":full_progress.map(str::len),"reported_progress_truncated":full_progress.zip(visible_progress).is_some_and(|(full,shown)|full!=shown),"runtime_cleanup_qualified":true}),
    )
}
fn excerpt(value: &str, limit: usize) -> &str {
    let mut end = value.len().min(limit);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}
pub(super) fn queue_for_event(
    tx: &Transaction<'_>,
    project: &str,
    task: Option<&str>,
    kind: &str,
    revision: i64,
    event: i64,
) -> Result<()> {
    if !is_configured(tx)?
        || !matches!(
            kind,
            "admit"
                | "approve_brief"
                | "observe_attempt"
                | "abort_task"
                | "cancel_attempt"
                | "report_progress"
                | "ask_question"
                | "answer_question"
                | "sync_kata_task"
                | "capture_candidate"
                | "accept_candidate"
                | "land_candidate"
                | "retire_workspace"
        )
    {
        return Ok(());
    }
    let Some(task) = task else {
        return Ok(());
    };
    let linked: bool = db(tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM kata_sources WHERE project=?1 AND task=?2)",
        params![project, task],
        |r| r.get(0),
    ))?;
    if !linked {
        return Ok(());
    }
    let summary = execution_summary(tx, project, task, revision, event)?;
    for kind in ["metadata", "comment"] {
        let operation = id();
        let mut summary = summary.clone();
        summary["operation"] = json!(operation);
        let payload = if kind == "metadata" {
            summary
        } else {
            let body = format!(
                "Corenius execution: {}. Task {task}; admitted version {}; controller event {event}, revision {revision}. Observed attempt: {}; reported progress: {}. Runtime cleanup mechanism qualified; resource disposition follows observed controller evidence.",
                summary["state"].as_str().unwrap_or("unknown"),
                summary["admitted_version"].as_str().unwrap_or("unknown"),
                summary["observed_attempt_state"],
                summary["reported_progress"].as_str().unwrap_or("none")
            );
            json!({"body":format!("{} [corenius operation {operation}]",excerpt(&body,8192))})
        };
        db(tx.execute("INSERT INTO kata_effects(id,project,task,event_seq,controller_revision,kind,state,payload,prepared_at) VALUES(?1,?2,?3,?4,?5,?6,'prepared',?7,?8)",params![operation,project,task,event,revision,kind,payload.to_string(),now()?]))?;
        db(tx.execute("INSERT INTO kata_effect_history(effect,state,observation) VALUES(?1,'prepared','intent committed before any HTTP effect')",[&operation]))?;
    }
    Ok(())
}
pub(super) fn reconcile_request(controller: &Controller, request: Value) -> Result<Value> {
    let object = request
        .as_object()
        .ok_or("invalid Kata reconcile request")?;
    if object.len() != 3 || request["kata"] != json!({"kind":"reconcile"}) {
        return Err("Kata accepts only authenticated project reconciliation".into());
    }
    let auth: Auth = serde_json::from_value(request["auth"].clone()).map_err(|e| e.to_string())?;
    let project = text(&request, "project")?;
    {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &auth, project)?;
        if p.role != "operator" {
            return Err("authorization: operator required for Kata reconciliation".into());
        }
    }
    adapter(controller)?.drain_project(controller, Some(project))
}
struct Effect {
    id: String,
    project: String,
    task: String,
    item: String,
    kind: String,
    payload: Value,
    prepared_at: i64,
    prior: Option<Value>,
}
fn effect_state(
    controller: &Controller,
    effect: &Effect,
    state: &str,
    observation: Value,
    revision: Option<i64>,
) -> Result<()> {
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    db(tx.execute(
        "UPDATE kata_effects SET state=?2,observation=?3,acknowledged_revision=?4 WHERE id=?1",
        params![effect.id, state, observation.to_string(), revision],
    ))?;
    db(tx.execute(
        "INSERT INTO kata_effect_history(effect,state,observation) VALUES(?1,?2,?3)",
        params![effect.id, state, observation.to_string()],
    ))?;
    if state == "confirmed" {
        db(tx.execute(
            "UPDATE kata_sources SET acknowledged_revision=?3 WHERE project=?1 AND task=?2",
            params![effect.project, effect.task, revision],
        ))?;
        if effect.kind == "metadata" {
            db(tx.execute("UPDATE kata_sources SET acknowledged_summary=?3,drift=NULL WHERE project=?1 AND task=?2",params![effect.project,effect.task,effect.payload.to_string()]))?;
        }
    }
    db(tx.execute(
        "UPDATE projects SET revision=revision+1 WHERE id=?1",
        [&effect.project],
    ))?;
    let project_revision: i64 = db(tx.query_row(
        "SELECT revision FROM projects WHERE id=?1",
        [&effect.project],
        |r| r.get(0),
    ))?;
    db(tx.execute("INSERT INTO events(project,task,kind,body) VALUES(?1,?2,'kata_effect_observed',?3)",params![effect.project,effect.task,json!({"revision":project_revision,"effect":effect.id,"state":state,"observation":observation,"acknowledged_kata_revision":revision}).to_string()]))?;
    db(tx.commit())
}
impl KataRuntime {
    fn drain(&self, controller: &Controller) -> Result<Value> {
        self.drain_project(controller, None)
    }
    fn drain_project(&self, controller: &Controller, project: Option<&str>) -> Result<Value> {
        let _gate = controller
            .kata_effect_gate
            .lock()
            .map_err(|_| "Kata effect gate poisoned")?;
        let mut blocked = std::collections::BTreeSet::new();
        let mut results = Vec::new();
        // Consider only the earliest pending effect for each task. Recent failed
        // heads rotate behind untouched heads, including across bounded batches.
        for _ in 0..100 {
            let effects = {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                let mut statement = db(c.prepare("SELECT e.id,e.project,e.task,s.item,e.kind,e.payload,e.prepared_at,s.acknowledged_summary FROM kata_effects e JOIN kata_sources s ON s.project=e.project AND s.task=e.task WHERE e.state<>'confirmed' AND (?1 IS NULL OR e.project=?1) AND NOT EXISTS(SELECT 1 FROM kata_effects earlier WHERE earlier.project=e.project AND earlier.task=e.task AND earlier.state<>'confirmed' AND earlier.rowid<e.rowid) ORDER BY (SELECT MAX(h.rowid) FROM kata_effect_history h WHERE h.effect=e.id AND h.state<>'prepared') NULLS FIRST,e.rowid LIMIT 100"))?;
                let rows = db(statement.query_map([project], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, i64>(6)?,
                        r.get::<_, Option<String>>(7)?,
                    ))
                }))?;
                db(rows.collect::<rusqlite::Result<Vec<_>>>())?
            };
            let Some((id, project, task, item, kind, payload, prepared_at, _)) = effects
                .into_iter()
                .find(|row| !blocked.contains(&(row.1.clone(), row.2.clone())))
            else {
                break;
            };
            // Read current acknowledged guard after the previous effect has committed.
            let prior = {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.query_row(
                    "SELECT acknowledged_summary FROM kata_sources WHERE project=?1 AND task=?2",
                    params![project, task],
                    |r| r.get::<_, Option<String>>(0),
                ))?
            };
            let effect = Effect {
                id,
                project,
                task,
                item,
                kind,
                payload: serde_json::from_str(&payload).map_err(|e| e.to_string())?,
                prepared_at,
                prior: prior
                    .map(|p| serde_json::from_str(&p))
                    .transpose()
                    .map_err(|e| e.to_string())?,
            };
            effect_state(
                controller,
                &effect,
                "applying",
                json!({"reason":"reconcile readback before retry"}),
                None,
            )?;
            let outcome = self.apply_effect(&effect);
            match outcome {
                Ok((revision, receipt)) => {
                    effect_state(controller, &effect, "confirmed", receipt, Some(revision))?;
                    results.push(json!({"effect":effect.id,"state":"confirmed","acknowledged_kata_revision":revision}));
                }
                Err(error) => {
                    effect_state(
                        controller,
                        &effect,
                        "unknown",
                        json!({"error":error,"retry":"readback required; no acceptance or stop inferred"}),
                        None,
                    )?;
                    results.push(json!({"effect":effect.id,"state":"unknown","error":error}));
                    blocked.insert((effect.project.clone(), effect.task.clone()));
                }
            }
        }
        // Poll pinned human source as proposals independently from execution publication.
        self.refresh_sources(controller, project)?;
        Ok(json!({"configured":true,"effects":results}))
    }
    fn apply_effect(&self, effect: &Effect) -> Result<(i64, Value)> {
        let remote = self.read_item(&effect.item)?;
        let revision = remote["issue"]["revision"]
            .as_i64()
            .ok_or("Kata revision missing")?;
        let path = self.item_path(&effect.item)?;
        if effect.kind == "metadata" {
            let current = &remote["issue"]["metadata"][SUMMARY_KEY];
            if *current == effect.payload {
                return Ok((
                    revision,
                    json!({"method":"metadata-readback","operation":effect.id,"observed_revision":revision}),
                ));
            }
            if *current != effect.prior.clone().unwrap_or(Value::Null) {
                return Err(
                    "Kata external execution projection drift; guarded publication withheld".into(),
                );
            }
            let guard = if current.is_null() {
                json!({"key":SUMMARY_KEY,"if_absent":true})
            } else {
                json!({"key":SUMMARY_KEY,"if_value":current.to_string()})
            };
            let body = json!({"patch":{SUMMARY_KEY:effect.payload},"guard":guard});
            let reply = self.request(
                "POST",
                &format!("{path}/metadata"),
                Some(&body),
                &[("If-Match", format!("\"rev-{revision}\""))],
            )?;
            let observed = self.read_item(&effect.item)?;
            if observed["issue"]["metadata"][SUMMARY_KEY] != effect.payload {
                return Err(
                    "Kata metadata acknowledgement does not establish requested value".into(),
                );
            }
            let revision = observed["issue"]["revision"]
                .as_i64()
                .ok_or("Kata acknowledged revision missing")?;
            Ok((
                revision,
                json!({"method":"guarded-metadata-and-readback","operation":effect.id,"reply":reply,"observed_revision":revision}),
            ))
        } else {
            let body = text(&effect.payload, "body")?;
            let comments = remote["comments"]
                .as_array()
                .ok_or("Kata comments missing")?;
            let matches: Vec<_> = comments
                .iter()
                .filter(|c| c["body"] == body && c["author"] == self.config.actor)
                .collect();
            if matches.len() == 1 {
                return Ok((
                    revision,
                    json!({"method":"comment-readback","operation":effect.id,"comment":matches[0],"observed_revision":revision}),
                ));
            }
            if !matches.is_empty() {
                return Err(
                    "Kata duplicate visible effect detected; reconciliation required".into(),
                );
            }
            if now()? - effect.prepared_at >= 7 * 24 * 60 * 60 {
                return Err("Kata comment idempotency retention expired; absent effect is ambiguous and not retried".into());
            }
            let reply = self.request(
                "POST",
                &format!("{path}/comments"),
                Some(&effect.payload),
                &[("Idempotency-Key", format!("corenius-{}", effect.id))],
            )?;
            let observed = self.read_item(&effect.item)?;
            let count = observed["comments"]
                .as_array()
                .ok_or("Kata comments missing")?
                .iter()
                .filter(|c| c["body"] == body && c["author"] == self.config.actor)
                .count();
            if count != 1 {
                return Err("Kata comment acknowledgement needs exact unique readback".into());
            }
            let revision = observed["issue"]["revision"]
                .as_i64()
                .ok_or("Kata acknowledged revision missing")?;
            Ok((
                revision,
                json!({"method":"idempotent-comment-and-readback","operation":effect.id,"reply":reply,"observed_revision":revision}),
            ))
        }
    }
    fn refresh_sources(&self, controller: &Controller, project: Option<&str>) -> Result<()> {
        let sources = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut statement = db(c.prepare("SELECT project,task,item,source_files FROM kata_sources WHERE ?1 IS NULL OR project=?1 ORDER BY rowid"))?;
            let rows = db(statement.query_map([project], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            }))?;
            db(rows.collect::<rusqlite::Result<Vec<_>>>())?
        };
        for (project, task, item, files) in sources {
            let files: Value = serde_json::from_str(&files).map_err(|e| e.to_string())?;
            match self.capture(&item, &files) {
                Ok(source) => record_source_observation(controller, &project, &task, &source)?,
                Err(error) => record_source_failure(controller, &project, &task, &error)?,
            }
        }
        Ok(())
    }
}

pub(super) fn context(c: &Connection, p: &Principal, project: &str) -> Result<Value> {
    if !is_configured(c)? {
        return Err("participant extension not configured".into());
    }
    crate::view::context(c, p, project, &json!({}))
}
