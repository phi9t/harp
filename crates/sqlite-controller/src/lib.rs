//! A single local controller writer. All external participants use `handle` or the socket service.
//! Runtime and Git effects belong outside this module's short transactions.
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::Mutex,
    time::Duration,
};
use uuid::Uuid;

pub mod client;
pub mod codex;
mod control;
pub mod gateway;
pub mod git;
pub mod git_workspace;
pub mod kata;
mod monitor;
mod native_profile;
mod native_ws;
mod query;
pub mod runtime;
pub mod service;
pub mod supervisor;
pub mod ui_gateway;
mod view;

type Result<T> = std::result::Result<T, String>;
fn db<T>(r: rusqlite::Result<T>) -> Result<T> {
    r.map_err(|e| format!("storage: {e}"))
}
fn id() -> String {
    Uuid::new_v4().to_string()
}
fn hash(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
fn text<'a>(v: &'a Value, field: &str) -> Result<&'a str> {
    v.get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty() && s.len() <= 65_536)
        .ok_or_else(|| format!("invalid argument: {field}"))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Auth {
    principal: String,
    token: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    auth: Auth,
    project: String,
    command: Option<Command>,
    query: Option<Query>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Command {
    kind: String,
    expected_revision: u64,
    idempotency_key: String,
    args: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Query {
    kind: String,
    args: Value,
}
struct Principal {
    id: String,
    role: String,
    epoch: i64,
    attempt: Option<String>,
    task: Option<String>,
}

pub struct Controller {
    connection: Mutex<Connection>,
    visible_inputs: std::path::PathBuf,
    _ownership: File,
    kata: Mutex<Option<std::sync::Arc<kata::KataRuntime>>>,
    kata_effect_gate: Mutex<()>,
    git: Mutex<Option<std::sync::Arc<git_workspace::GitBackend>>>,
    git_effect_gate: Mutex<()>,
}
impl Controller {
    /// Open a supported local database, locking its canonical pathname for this controller lifetime.
    /// The bootstrap secret initializes the operator only on a fresh database; it never rotates it.
    pub fn open(path: impl AsRef<Path>, bootstrap_token: &str) -> Result<Self> {
        if bootstrap_token.len() < 12 {
            return Err("bootstrap token must have at least 12 bytes".into());
        }
        let path = path.as_ref();
        // A dangling alias has no stable canonical identity for the lifetime lock.
        if std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
            && !path.exists()
        {
            return Err("dangling database symlink is unsupported".into());
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let parent = parent
            .canonicalize()
            .map_err(|e| format!("database directory: {e}"))?;
        let path = if path.exists() {
            path.canonicalize().map_err(|e| e.to_string())?
        } else {
            parent.join(path.file_name().ok_or("database filename required")?)
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if path.exists() && path.metadata().map_err(|e| e.to_string())?.nlink() != 1 {
                return Err("database hard links are unsupported".into());
            }
        }
        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".controller-lock");
        let ownership =
            private_file(Path::new(&lock_path)).map_err(|e| format!("ownership lock: {e}"))?;
        ownership
            .try_lock()
            .map_err(|e| format!("controller ownership unavailable: {e}"))?;
        drop(private_file(&path)?);
        let mut connection = db(Connection::open(&path))?;
        db(connection.busy_timeout(Duration::from_millis(500)))?;
        db(connection.pragma_update(None, "journal_mode", "WAL"))?;
        db(connection.pragma_update(None, "synchronous", "FULL"))?;
        db(connection.pragma_update(None, "foreign_keys", "ON"))?;
        let mode: String = db(connection.pragma_query_value(None, "journal_mode", |r| r.get(0)))?;
        let sync: i64 = db(connection.pragma_query_value(None, "synchronous", |r| r.get(0)))?;
        let fk: i64 = db(connection.pragma_query_value(None, "foreign_keys", |r| r.get(0)))?;
        if mode != "wal" || sync != 2 || fk != 1 {
            return Err("effective SQLite durability configuration refused".into());
        }
        let version: i64 = db(connection.pragma_query_value(None, "user_version", |r| r.get(0)))?;
        if version == 0 {
            let objects: i64 = db(connection.query_row(
                "SELECT COUNT(*) FROM sqlite_schema WHERE name NOT GLOB 'sqlite_*'",
                [],
                |r| r.get(0),
            ))?;
            if objects != 0 {
                return Err("fresh database required; unversioned foreign schema refused".into());
            }
            let tx = db(connection.transaction_with_behavior(TransactionBehavior::Immediate))?;
            db(tx.execute_batch(include_str!("schema.sql")))?;
            db(tx.execute("INSERT INTO principals(id,token_hash,role,epoch) VALUES('operator',?1,'operator',1)",[hash(bootstrap_token)]))?;
            db(tx.commit())?;
        } else if version != 1 && version != 2 {
            return Err(format!(
                "unsupported schema version {version}; explicit migration required"
            ));
        }
        let stored: String = db(connection.query_row(
            "SELECT token_hash FROM principals WHERE id='operator'",
            [],
            |r| r.get(0),
        ))?;
        if stored != hash(bootstrap_token) {
            return Err("bootstrap secret does not match existing operator".into());
        }
        db(connection.execute_batch(include_str!("control_schema.sql")))?;
        let mut visible_inputs = path.as_os_str().to_os_string();
        visible_inputs.push(".visible-inputs");
        let visible_inputs = std::path::PathBuf::from(visible_inputs);
        if std::fs::symlink_metadata(&visible_inputs).is_ok_and(|m| m.file_type().is_symlink()) {
            return Err("visible input directory alias refused".into());
        }
        std::fs::create_dir_all(&visible_inputs).map_err(|e| e.to_string())?;
        File::open(&parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&visible_inputs, std::fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
            visible_inputs,
            _ownership: ownership,
            kata: Mutex::new(None),
            kata_effect_gate: Mutex::new(()),
            git: Mutex::new(None),
            git_effect_gate: Mutex::new(()),
        })
    }

    /// Authenticated JSON seam. A command returns a durable receipt only after COMMIT.
    /// Queries authenticate before selecting any scoped state. Errors never create receipts.
    pub fn handle(&self, request: Value) -> Result<Value> {
        if request.get("gateway").is_some() {
            return gateway::dispatch(self, request);
        }
        if request["query"]["kind"] == "monitor_input"
            || request["query"]["kind"] == "runtime_public_input"
        {
            return monitor::input(self, request);
        }
        if request["query"]["kind"] == "control_center"
            || request["query"]["kind"] == "lead_context"
        {
            return control::read(self, request);
        }
        if git::handles(&request) {
            return git::dispatch(self, request);
        }
        git::check_ref(self, &request)?;
        if request.get("kata").is_some() {
            return kata::reconcile_request(self, request);
        }
        let request: Request =
            serde_json::from_value(request).map_err(|e| format!("invalid request: {e}"))?;
        let monitor_check = monitor::validate_evidence(self, &request)?;
        if request.project.is_empty() || request.project.len() > 200 {
            return Err("invalid project".into());
        }
        // Source HTTP/filesystem reads never run inside the SQLite transaction or mutex.
        let source = kata::prepare_request(self, &request)?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let principal = authenticate(&connection, &request.auth, &request.project)?;
        match (request.command, request.query) {
            (Some(command), None) => self.command(
                &mut connection,
                &principal,
                &request.project,
                command,
                source.as_ref(),
                monitor_check.as_ref(),
            ),
            (None, Some(query)) => query::run(&connection, &principal, &request.project, query),
            _ => Err("exactly one command or query required".into()),
        }
    }

    fn command(
        &self,
        connection: &mut Connection,
        p: &Principal,
        project: &str,
        c: Command,
        source: Option<&Value>,
        monitor_check: Option<&Value>,
    ) -> Result<Value> {
        if c.idempotency_key.is_empty() || c.idempotency_key.len() > 200 {
            return Err("invalid idempotency key".into());
        }
        authorize(p, &c.kind, &c.args)?;
        let arguments =
            json!({"kind":c.kind,"expected_revision":c.expected_revision,"args":c.args})
                .to_string();
        let tx = db(connection.transaction_with_behavior(TransactionBehavior::Immediate))?;
        let prior: Option<(String,String)> = db(tx.query_row("SELECT arguments,response FROM receipts WHERE project=?1 AND principal=?2 AND epoch=?3 AND key=?4",params![project,p.id,p.epoch,c.idempotency_key],|r|Ok((r.get(0)?,r.get(1)?))).optional())?;
        if let Some((stored, receipt)) = prior {
            if arguments != stored {
                return Err("idempotency argument mismatch".into());
            }
            return serde_json::from_str(&receipt).map_err(|e| e.to_string());
        }
        let revision: i64 = db(tx
            .query_row(
                "SELECT revision FROM projects WHERE id=?1",
                [project],
                |r| r.get(0),
            )
            .optional())?
        .unwrap_or(0);
        if i64::try_from(c.expected_revision).ok() != Some(revision) {
            return Err(format!(
                "revision conflict: expected {}, current {revision}",
                c.expected_revision
            ));
        }
        if revision == 0 && c.kind != "create_project" {
            return Err("project does not exist".into());
        }
        if c.kind == "publish_monitor_assessment" {
            monitor::bind_checked(&tx, p, project, &c.args, monitor_check)?;
        }
        let result = apply(&tx, p, project, &c.kind, &c.args, source)?;
        let next = revision.checked_add(1).ok_or("revision exhausted")?;
        db(tx.execute(
            "UPDATE projects SET revision=?2 WHERE id=?1",
            params![project, next],
        ))?;
        let is_message = matches!(c.kind.as_str(), "send_message" | "ack_message");
        // Message audiences come from the retained participant record, never caller task metadata.
        let audience: Option<String> = if is_message {
            let message = text(&result, "message")?;
            let (sender, recipient): (String, String) = db(tx.query_row(
                "SELECT sender,recipient FROM messages WHERE project=?1 AND id=?2",
                params![project, message],
                |r| Ok((r.get(0)?, r.get(1)?)),
            ))?;
            Some(if c.kind == "ack_message" {
                sender
            } else {
                recipient
            })
        } else {
            None
        };
        let task: Option<String> = if is_message {
            None
        } else {
            match c.kind.as_str() {
                // Execution routing follows the retained identity, not caller task metadata.
                "admit" | "observe_attempt" | "cancel_attempt" => Some(db(tx.query_row(
                    "SELECT task FROM attempts WHERE project=?1 AND id=?2",
                    params![project,text(&result,"attempt")?], |r| r.get(0),
                ))?),
                "transition_operation" => db(tx.query_row(
                    "SELECT a.task FROM operations o LEFT JOIN attempts a ON a.id=o.attempt AND a.project=o.project WHERE o.project=?1 AND o.id=?2",
                    params![project,text(&result,"operation")?], |r| r.get(0),
                ))?,
                _ => result.get("task").and_then(Value::as_str)
                    .or(p.task.as_deref()).or_else(|| c.args.get("task").and_then(Value::as_str))
                    .map(str::to_owned),
            }
        };
        let event_body =
            json!({"principal":p.id,"revision":next,"args":c.args,"result":public_result(&result)});
        db(tx.execute(
            "INSERT INTO events(project,task,audience,kind,body) VALUES(?1,?2,?3,?4,?5)",
            params![project, task, audience, c.kind, event_body.to_string()],
        ))?;
        let seq = tx.last_insert_rowid();
        kata::queue_for_event(&tx, project, task.as_deref(), &c.kind, next, seq)?;
        let response = json!({"receipt":id(),"event_seq":seq,"revision":next,"result":result});
        db(tx.execute("INSERT INTO receipts(project,principal,epoch,key,arguments,response) VALUES(?1,?2,?3,?4,?5,?6)",params![project,p.id,p.epoch,c.idempotency_key,arguments,response.to_string()]))?;
        #[cfg(feature = "fault-injection")]
        if c.kind == "admit"
            && let Ok(marker) = std::env::var("CORENIUS_TEST_ADMISSION_BARRIER")
        {
            std::fs::write(marker, b"before commit").map_err(|e| e.to_string())?;
            loop {
                std::thread::park();
            }
        }
        db(tx.commit())?;
        Ok(response)
    }
}

fn authenticate(c: &Connection, auth: &Auth, project: &str) -> Result<Principal> {
    type StoredPrincipal = (String, Option<String>, String, i64, Option<String>);
    let row: Option<StoredPrincipal> = db(c
        .query_row(
            "SELECT token_hash,project,role,epoch,attempt FROM principals WHERE id=?1",
            [&auth.principal],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional())?;
    let (digest, scope, role, epoch, attempt) = row.ok_or("authentication failed")?;
    if digest != hash(&auth.token) {
        return Err("authentication failed".into());
    }
    if role != "operator" && scope.as_deref() != Some(project) {
        return Err("authorization: project scope".into());
    }
    let mut task = None;
    let observer: bool = db(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM monitor_observers WHERE principal=?1 AND active=1)",
        [&auth.principal],
        |r| r.get(0),
    ))?;
    if observer {
        return Ok(Principal {
            id: auth.principal.clone(),
            role: "observer".into(),
            epoch,
            attempt: None,
            task: None,
        });
    }
    if role == "lead" {
        let current: i64 = db(c.query_row(
            "SELECT lead_epoch FROM projects WHERE id=?1",
            [project],
            |r| r.get(0),
        ))?;
        if epoch != current {
            return Err("authentication: superseded lead authority".into());
        }
    }
    if role == "attempt" {
        let (current, owned, state): (i64, String, String) = db(c.query_row(
            "SELECT scope_epoch,task,state FROM attempts WHERE id=?1",
            [attempt.as_deref().ok_or("authentication failed")?],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ))?;
        if current != epoch || state == "exited" {
            return Err("authentication: revoked attempt authority".into());
        }
        task = Some(owned);
    }
    Ok(Principal {
        id: auth.principal.clone(),
        role,
        epoch,
        attempt,
        task,
    })
}

fn authorize(p: &Principal, kind: &str, args: &Value) -> Result<()> {
    if p.role == "observer" && kind != "publish_monitor_assessment" {
        return Err("authorization: observer is advisory only".into());
    }
    if matches!(
        kind,
        "create_project"
            | "replace_lead"
            | "observe_attempt"
            | "transition_operation"
            | "create_monitor_observer"
    ) && p.role != "operator"
    {
        return Err("authorization: operator required".into());
    }
    if p.role == "attempt" {
        if !matches!(
            kind,
            "ask_question"
                | "send_message"
                | "ack_message"
                | "report_progress"
                | "propose_followup"
                | "ack_correction"
        ) {
            return Err("authorization: attempt command scope".into());
        }
        if matches!(
            kind,
            "ask_question" | "report_progress" | "propose_followup"
        ) && Some(text(args, "task")?) != p.task.as_deref()
        {
            return Err("authorization: own task only".into());
        }
    }
    Ok(())
}
fn public_result(result: &Value) -> Value {
    let mut value = result.clone();
    if let Some(object) = value.as_object_mut() {
        object.remove("token");
    }
    value
}

fn apply(
    tx: &Transaction<'_>,
    p: &Principal,
    project: &str,
    kind: &str,
    a: &Value,
    source: Option<&Value>,
) -> Result<Value> {
    match kind {
        "create_kata_task" | "refresh_kata_source" | "sync_kata_task" | "propose_followup" => {
            kata::apply(tx, p, project, kind, a, source)
        }
        "codex_turn" | "codex_steer" | "codex_interrupt" => codex::prepare(tx, p, project, kind, a),
        "create_project" => {
            if text(a, "name")? != project {
                return Err("project name mismatch".into());
            }
            db(tx.execute(
                "INSERT INTO projects(id,base,revision) VALUES(?1,?2,1)",
                params![project, commit_oid(text(a, "base")?)?],
            ))?;
            Ok(json!({"project":project}))
        }
        "create_task" => {
            if kata::is_configured(tx)? && source.is_none() {
                return Err("qualified Kata source capture required; use create_kata_task".into());
            }
            let task = text(a, "task")?;
            let brief = id();
            let body = text(a, "brief")?;
            let wr = a.get("work_ref").ok_or("work_ref required")?;
            for field in ["deployment", "project", "item"] {
                text(wr, field)?;
            }
            let canonical_ref=json!({"deployment":text(wr,"deployment")?,"project":text(wr,"project")?,"item":text(wr,"item")?}).to_string();
            let digest = hash(body);
            if let Some(expected) = a.get("brief_digest")
                && expected.as_str() != Some(&digest)
            {
                return Err("brief digest mismatch".into());
            }
            db(tx.execute("INSERT INTO briefs(id,project,task,work_ref,source_revision,body,digest) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![brief,project,task,canonical_ref,text(a,"source_revision")?,body,digest]))?;
            db(tx.execute(
                "INSERT INTO tasks(project,id,brief,work_ref,state) VALUES(?1,?2,?3,?4,'created')",
                params![project, task, brief, canonical_ref],
            ))?;
            Ok(json!({"task":task,"brief":brief,"brief_digest":digest}))
        }
        "admit" => {
            git::check_admission(tx, project)?;
            let paused: bool = db(tx.query_row(
                "SELECT paused FROM projects WHERE id=?1",
                [project],
                |r| r.get(0),
            ))?;
            if paused {
                return Err("dispatch paused".into());
            }
            let task = text(a, "task")?;
            kata::check_admission(tx, project, task, source)?;
            let (state, brief): (String, String) = db(tx.query_row(
                "SELECT state,brief FROM tasks WHERE project=?1 AND id=?2",
                params![project, task],
                |r| Ok((r.get(0)?, r.get(1)?)),
            ))?;
            if !matches!(
                state.as_str(),
                "created" | "repair-required" | "integration-required"
            ) {
                return Err("task is not eligible for admission".into());
            }
            let slot: Option<i64> = db(tx
                .query_row(
                    "SELECT id FROM slots WHERE attempt IS NULL ORDER BY id LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional())?;
            let slot = slot.ok_or("capacity: both worker slots occupied")?;
            let attempt = id();
            let operation = id();
            let principal = format!("attempt:{attempt}");
            let token = id();
            let base: String = db(tx.query_row(
                "SELECT base FROM projects WHERE id=?1",
                [project],
                |r| r.get(0),
            ))?;
            commit_oid(&base)?;
            db(tx.execute("INSERT INTO attempts(id,project,task,brief,base,state) VALUES(?1,?2,?3,?4,?5,'starting')",params![attempt,project,task,brief,base]))?;
            db(tx.execute(
                "UPDATE slots SET attempt=?2 WHERE id=?1 AND attempt IS NULL",
                params![slot, attempt],
            ))?;
            db(tx.execute(
                "UPDATE tasks SET state='active' WHERE project=?1 AND id=?2",
                params![project, task],
            ))?;
            db(tx.execute("INSERT INTO operations(id,project,attempt,kind,state,payload) VALUES(?1,?2,?3,'start','prepared',?4)",params![operation,project,attempt,json!({"attempt":attempt,"base":base,"brief":brief,"scope_epoch":1}).to_string()]))?;
            db(tx.execute("INSERT INTO principals(id,token_hash,project,role,epoch,attempt) VALUES(?1,?2,?3,'attempt',1,?4)",params![principal,hash(&token),project,attempt]))?;
            Ok(
                json!({"attempt":attempt,"operation":operation,"slot":slot,"principal":principal,"token":token,"scope_epoch":1,"base":base,"brief":brief}),
            )
        }
        "replace_lead" => {
            db(tx.execute(
                "UPDATE projects SET lead_epoch=lead_epoch+1 WHERE id=?1",
                [project],
            ))?;
            let epoch: i64 = db(tx.query_row(
                "SELECT lead_epoch FROM projects WHERE id=?1",
                [project],
                |r| r.get(0),
            ))?;
            let principal = format!("lead:{}", id());
            let token = id();
            db(tx.execute("INSERT INTO principals(id,token_hash,project,role,epoch) VALUES(?1,?2,?3,'lead',?4)",params![principal,hash(&token),project,epoch]))?;
            Ok(json!({"principal":principal,"token":token,"role_epoch":epoch}))
        }
        "pause_dispatch" => {
            let paused = a
                .get("paused")
                .and_then(Value::as_bool)
                .ok_or("paused must be boolean")?;
            db(tx.execute(
                "UPDATE projects SET paused=?2 WHERE id=?1",
                params![project, paused],
            ))?;
            Ok(json!({"paused":paused}))
        }
        "abort_task" => {
            let task = text(a, "task")?;
            let n=db(tx.execute("UPDATE tasks SET state='aborted' WHERE project=?1 AND id=?2 AND state NOT IN ('landed','accepted','aborted')",params![project,task]))?;
            if n != 1 {
                return Err("task cannot be aborted".into());
            }
            // Task outcome is independent of external execution lifetime and slot occupancy.
            Ok(json!({"task":task}))
        }
        "cancel_attempt" => {
            let attempt = text(a, "attempt")?;
            require_attempt(tx, project, attempt)?;
            let operation = id();
            let n=db(tx.execute("UPDATE attempts SET cancel_requested=1,scope_epoch=scope_epoch+1 WHERE id=?1 AND state<>'exited' AND cancel_requested=0",[attempt]))?;
            if n != 1 {
                return Err("attempt cannot be cancelled".into());
            }
            db(tx.execute("INSERT INTO operations(id,project,attempt,kind,state,payload) VALUES(?1,?2,?3,'cancel','prepared',?4)",params![operation,project,attempt,json!({"attempt":attempt}).to_string()]))?;
            Ok(json!({"attempt":attempt,"operation":operation,"cancel_requested":true}))
        }
        "observe_attempt" => {
            let attempt = text(a, "attempt")?;
            require_attempt(tx, project, attempt)?;
            let state = text(a, "state")?;
            let observation = text(a, "observation")?;
            let old: String = db(tx.query_row(
                "SELECT state FROM attempts WHERE id=?1",
                [attempt],
                |r| r.get(0),
            ))?;
            if old == "exited" || !matches!(state, "running" | "unknown" | "exited") {
                return Err("invalid attempt transition".into());
            }
            db(tx.execute("UPDATE attempts SET state=?2,observation=?3,scope_epoch=scope_epoch+CASE WHEN ?2='exited' THEN 1 ELSE 0 END WHERE id=?1",params![attempt,state,observation]))?;
            if state == "exited" {
                db(tx.execute("UPDATE slots SET attempt=NULL WHERE attempt=?1", [attempt]))?;
            }
            Ok(json!({"attempt":attempt,"state":state}))
        }
        "transition_operation" => {
            let operation = text(a, "operation")?;
            let state = text(a, "state")?;
            let old: String = db(tx.query_row(
                "SELECT state FROM operations WHERE id=?1 AND project=?2",
                params![operation, project],
                |r| r.get(0),
            ))?;
            let allowed = match old.as_str() {
                "prepared" => matches!(state, "applying" | "failed" | "unknown"),
                "applying" => matches!(state, "confirmed" | "failed" | "unknown"),
                "unknown" => matches!(state, "applying" | "confirmed" | "failed"),
                _ => false,
            };
            if !allowed {
                return Err("invalid operation transition".into());
            }
            db(tx.execute(
                "UPDATE operations SET state=?2,observation=?3 WHERE id=?1",
                params![operation, state, text(a, "observation")?],
            ))?;
            Ok(json!({"operation":operation,"state":state}))
        }
        "ask_question" => {
            let question = id();
            let task = text(a, "task")?;
            db(tx.execute(
                "INSERT INTO questions(id,project,task,sender,text) VALUES(?1,?2,?3,?4,?5)",
                params![question, project, task, p.id, text(a, "text")?],
            ))?;
            control::question_created(tx, &question, p)?;
            Ok(json!({"question":question}))
        }
        "answer_question" => {
            let question = text(a, "question")?;
            control::question_answered(tx, question, p)?;
            let n=db(tx.execute("UPDATE questions SET answer=?3,answered_by=?4 WHERE project=?1 AND id=?2 AND answer IS NULL",params![project,question,text(a,"text")?,p.id]))?;
            if n != 1 {
                return Err("question missing or already answered".into());
            }
            let task: String = db(tx.query_row(
                "SELECT task FROM questions WHERE id=?1",
                [question],
                |r| r.get(0),
            ))?;
            Ok(json!({"question":question,"task":task}))
        }
        "record_decision" => {
            let decision = id();
            db(tx.execute(
                "INSERT INTO decisions(id,project,author,text) VALUES(?1,?2,?3,?4)",
                params![decision, project, p.id, text(a, "text")?],
            ))?;
            Ok(json!({"decision":decision}))
        }
        "send_message" => {
            if a.get("sender").is_some() || a.get("principal").is_some() || a.get("actor").is_some()
            {
                return Err("sender is derived from authenticated principal".into());
            }
            let recipient = text(a, "recipient")?;
            let scope: Option<Option<String>> = db(tx
                .query_row(
                    "SELECT project FROM principals WHERE id=?1",
                    [recipient],
                    |r| r.get(0),
                )
                .optional())?;
            if !matches!(scope, Some(None))
                && scope.as_ref().and_then(|s| s.as_deref()) != Some(project)
            {
                return Err("authorization: recipient project scope".into());
            }
            let message = id();
            db(tx.execute(
                "INSERT INTO messages(id,project,sender,recipient,text) VALUES(?1,?2,?3,?4,?5)",
                params![message, project, p.id, recipient, text(a, "text")?],
            ))?;
            Ok(json!({"message":message}))
        }
        "ack_message" => {
            let message = text(a, "message")?;
            let n=db(tx.execute("UPDATE messages SET acknowledged=1 WHERE id=?1 AND project=?2 AND recipient=?3 AND acknowledged=0",params![message,project,p.id]))?;
            if n != 1 {
                return Err(
                    "authorization: message missing, acknowledged, or wrong recipient".into(),
                );
            }
            Ok(json!({"message":message}))
        }
        "report_progress" => Ok(json!({"task":text(a,"task")?,"text":text(a,"text")?})),
        _ => control::apply(tx, p, project, kind, a),
    }
}
fn require_attempt(tx: &Transaction<'_>, project: &str, attempt: &str) -> Result<()> {
    let exists: bool = db(tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM attempts WHERE project=?1 AND id=?2)",
        params![project, attempt],
        |r| r.get(0),
    ))?;
    if exists {
        Ok(())
    } else {
        Err("attempt outside project or missing".into())
    }
}

fn private_file(path: &Path) -> Result<File> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
        .map_err(|e| e.to_string())?;
    Ok(file)
}

/// Validate an exact Git object identity. Repository object retention/lookup belongs to ticket04.
fn commit_oid(value: &str) -> Result<&str> {
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid base: full 40- or 64-hex Git commit OID required".into());
    }
    Ok(value)
}
