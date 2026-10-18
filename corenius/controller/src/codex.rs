//! Controller-owned native runtime registry. Journal commits precede external effects.
use crate::{
    Controller, db,
    native_ws::NativeWs,
    supervisor::{self, Binding},
};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};
use uuid::Uuid;
const SCHEMA: &str = "e587a3d5ffab12e528589e14d385b04d1192b9ae6914ce008cd3ea507e5aa455";
#[path = "native_ui.rs"]
mod ui;
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    schema_migration: String,
    topology: String,
    transport_root: PathBuf,
    servers: Vec<Server>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}
// Exact installed, visible model slugs observed for the retained native profile.
fn selected_model(model: Option<&str>) -> Result<&str, String> {
    let model = model.unwrap_or("gpt-6-sol");
    if !matches!(
        model,
        "gpt-6.1-sol"
            | "gpt-6-astra"
            | "gpt-6-sol"
            | "gpt-6-luna"
            | "gpt-5.6-sol"
            | "gpt-5.6-terra"
            | "gpt-5.6-luna"
            | "gpt-5.5"
    ) {
        return Err("native selected model is not in the installed profile".into());
    }
    Ok(model)
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Server {
    id: String,
    supervisor_socket: PathBuf,
    auth_ref: String,
    transport_root: PathBuf,
    worker_supervisor_socket: PathBuf,
    worker_auth_ref: String,
    worker_transport_root: PathBuf,
    schema_digest: String,
}
struct Connected {
    incarnation: String,
    ws: NativeWs,
    subscribed: std::collections::HashSet<String>,
}
struct NativeInputs<'a> {
    app: &'a Value,
    executor: &'a Value,
    worker: &'a Binding,
    body: &'a str,
}
pub struct CodexRuntime {
    config: Config,
    lifecycle: Mutex<()>,
    connections: Mutex<HashMap<String, Connected>>,
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .ok_or_else(|| format!("native {key} missing"))
}
fn secret(reference: &str) -> Result<String, String> {
    if reference.is_empty()
        || !reference
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b == b'_' || b.is_ascii_digit())
    {
        return Err("native auth reference refused".into());
    }
    let value = std::env::var(reference).map_err(|_| "native auth reference unavailable")?;
    if value.len() < 12 {
        return Err("native private credential required".into());
    }
    Ok(value)
}
#[cfg(feature = "fault-injection")]
fn crash_boundary(point: &str, observation: Value) -> Result<(), String> {
    if std::env::var("CORENIUS_TEST_CRASH_POINT").as_deref() == Ok(point)
        && let Ok(path) = std::env::var("CORENIUS_TEST_CRASH_FILE")
        && Path::new(&path).is_file()
    {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
        fs::write(format!("{path}.observed"), observation.to_string())
            .map_err(|e| e.to_string())?;
        std::process::exit(79);
    }
    Ok(())
}
#[cfg(feature = "fault-injection")]
fn adoption_fixture_defer(worker: &Binding) -> Result<(), String> {
    let Ok(path) = std::env::var("CORENIUS_TEST_ADOPT_EVENTS") else {
        return Ok(());
    };
    if !Path::new(&path).is_file() {
        return Ok(());
    }
    let actors: Value =
        serde_json::from_slice(&fs::read(format!("{path}.actors")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if actors["b"] != worker.attempt {
        return Ok(());
    }
    if Path::new(&format!("{path}.disarmed")).is_file() {
        fs::write(format!("{path}.retry-B"), worker.attempt.as_bytes())
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    let phase = if Path::new(&format!("{path}.projection")).is_file() {
        fs::write(format!("{path}.disarmed"), worker.attempt.as_bytes())
            .map_err(|e| e.to_string())?;
        "once after actual A projection; disarmed for next pass"
    } else {
        "until actual A projection"
    };
    use std::io::Write;
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{path}.deferrals"))
        .map_err(|e| e.to_string())?;
    writeln!(
        log,
        "{}",
        json!({"attempt":worker.attempt,"phase":phase,"native_rpc_performed":false})
    )
    .map_err(|e| e.to_string())?;
    Err("declared exact-B adoption fixture scheduling deferral; no native RPC".into())
}
#[cfg(feature = "fault-injection")]
fn adoption_fixture_snapshot_path(worker: &Binding) -> Result<Option<(String, String)>, String> {
    let Ok(path) = std::env::var("CORENIUS_TEST_ADOPT_EVENTS") else {
        return Ok(None);
    };
    if !Path::new(&path).is_file() {
        return Ok(None);
    }
    let actors: Value =
        serde_json::from_slice(&fs::read(format!("{path}.actors")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let suffix = if actors["a"] == worker.attempt {
        "projection"
    } else if actors["b"] == worker.attempt {
        "peer-projection"
    } else {
        return Ok(None);
    };
    Ok(Some((path.clone(), format!("{path}.{suffix}"))))
}
#[cfg(feature = "fault-injection")]
fn adoption_fixture_snapshot(worker: &Binding, document: &Value) -> Result<(), String> {
    let Some((path, snapshot)) = adoption_fixture_snapshot_path(worker)? else {
        return Ok(());
    };
    if document["ui_input_ready"] != true
        || document["thread_id"].as_str().is_none()
        || document["turn_id"].as_str().is_none()
        || document["environment_id"].as_str().is_none()
    {
        return Ok(());
    }
    if !Path::new(&snapshot).exists() {
        fs::write(snapshot, document.to_string()).map_err(|e| e.to_string())?;
    }
    // Only the fixture's two exact, independently qualified actors permit global freeze.
    if Path::new(&format!("{path}.projection")).is_file()
        && Path::new(&format!("{path}.peer-projection")).is_file()
    {
        let hold = std::env::var("CORENIUS_TEST_HOLD_CODEX").map_err(|_| "fixture hold missing")?;
        fs::write(
            hold,
            b"declared freeze after both real qualified actor projections",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

impl CodexRuntime {
    pub fn requires_configuration(controller: &Controller) -> Result<bool, String> {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let version: i64 = db(c.pragma_query_value(None, "user_version", |r| r.get(0)))?;
        if version != 2 {
            return Ok(false);
        }
        let raw: Option<String> = db(c
            .query_row(
                "SELECT document FROM codex_configuration WHERE id=1",
                [],
                |r| r.get(0),
            )
            .optional())?;
        let Some(raw) = raw else {
            return Ok(false);
        };
        let config: Config = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        Ok(!config.servers.is_empty())
    }
    pub fn open(path: &Path, controller: &Controller) -> Result<Self, String> {
        let config: Config = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        selected_model(config.model.as_deref())?;
        if config.schema_migration != "1-to-2"
            || !matches!(config.topology.as_str(), "shared" | "separate")
        {
            return Err("explicit Codex schema migration/topology required".into());
        }
        if !config.transport_root.is_absolute() || config.transport_root.starts_with("/usr") {
            return Err("private native transport root required".into());
        }
        fs::create_dir_all(&config.transport_root).map_err(|e| e.to_string())?;
        if config
            .transport_root
            .canonicalize()
            .map_err(|e| e.to_string())?
            != config.transport_root
        {
            return Err("native transport aliases refused".into());
        }
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config.transport_root, fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
        if config.servers.len() > 1 {
            return Err("one configured native profile required".into());
        }
        for s in &config.servers {
            if s.id.is_empty()
                || s.schema_digest != SCHEMA
                || s.transport_root != config.transport_root
                || !s.supervisor_socket.is_absolute()
                || !s.worker_supervisor_socket.is_absolute()
                || !s.worker_transport_root.is_absolute()
            {
                return Err("native registered profile refused".into());
            }
            secret(&s.auth_ref)?;
            secret(&s.worker_auth_ref)?;
            if std::env::var_os("CORENIUS_SUPERVISOR_SOCKET").map(PathBuf::from)
                != Some(s.worker_supervisor_socket.clone())
                || s.worker_auth_ref != "CORENIUS_SUPERVISOR_TOKEN"
            {
                return Err("native worker effect owner mismatch".into());
            }
        }
        let mut c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let version: i64 = db(c.pragma_query_value(None, "user_version", |r| r.get(0)))?;
        if version == 1 {
            let tx = db(c.transaction())?;
            db(tx.execute_batch(include_str!("codex_schema.sql")))?;
            db(tx.commit())?
        } else if version != 2 {
            return Err("Codex registry schema unsupported".into());
        }
        let expected = serde_json::to_string(&config).map_err(|e| e.to_string())?;
        let retained: Option<String> = db(c
            .query_row(
                "SELECT document FROM codex_configuration WHERE id=1",
                [],
                |r| r.get(0),
            )
            .optional())?;
        if let Some(retained) = retained {
            if retained != expected {
                return Err(
                    "durable native configured identity mismatch; explicit migration required"
                        .into(),
                );
            }
        } else {
            db(c.execute("INSERT INTO codex_configuration VALUES(1,?1)", [&expected]))?;
        }
        Ok(Self {
            config,
            lifecycle: Mutex::new(()),
            connections: Mutex::new(HashMap::new()),
        })
    }
    pub fn drain(&self, controller: &Controller) -> Result<(), String> {
        let _lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| "native lifecycle gate poisoned")?;
        self.drain_inner(controller)
    }
    fn drain_inner(&self, controller: &Controller) -> Result<(), String> {
        #[cfg(feature = "fault-injection")]
        if let Ok(path) = std::env::var("CORENIUS_TEST_HOLD_CODEX")
            && Path::new(&path).is_file()
        {
            return Ok(());
        }
        let Some(server) = self.config.servers.first() else {
            return Ok(());
        };
        self.retire_revoked(controller, server)?;
        let rows = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt=db(c.prepare("SELECT a.id,a.project,a.scope_epoch,a.base,a.brief,a.cancel_requested,a.state,o.id,b.body FROM attempts a JOIN operations o ON o.attempt=a.id AND o.kind='start' JOIN briefs b ON b.id=a.brief WHERE a.state<>'exited'"))?;
            let rows = db(stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                    r.get::<_, String>(8)?,
                ))
            }))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?
        };
        for (attempt, project, epoch, base, brief, cancelled, state, operation, body) in rows {
            if cancelled != 0 || state != "running" {
                continue;
            }
            let binding = Binding {
                project: project.clone(),
                attempt: attempt.clone(),
                operation,
                scope_epoch: epoch,
                base,
                brief,
            };
            if let Err(error) = self.bind_actor(
                controller,
                server,
                &binding,
                &body,
                "attempt",
                &format!("worker:{}:{}", binding.attempt, binding.scope_epoch),
            ) {
                let id = format!("worker:{attempt}:{epoch}");
                self.block_binding(controller, &id)?;
                eprintln!("native reconcile {attempt}: {error}");
            }
        }
        self.drain_leads(controller, server)?;
        Ok(())
    }
    fn retire_revoked(&self, controller: &Controller, server: &Server) -> Result<(), String> {
        let rows = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt=db(c.prepare("SELECT b.id,b.project,b.role,b.document FROM codex_bindings b JOIN projects p ON p.id=b.project LEFT JOIN attempts a ON a.id=b.attempt WHERE b.state<>'revoked' AND ((b.role='lead' AND b.authority_epoch<>p.lead_epoch) OR (b.role='attempt' AND (a.cancel_requested=1 OR a.state='exited' OR a.scope_epoch<>b.authority_epoch)))"))?;
            let rows = db(stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                ))
            }))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?
        };
        for (id, project, role, raw) in rows {
            let mut doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            doc["ui_input_ready"] = json!(false);
            doc["authority_state"] = json!("revoked");
            self.save_binding(controller, &id, &doc, "revoking")?;
            {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.execute("UPDATE codex_operations SET state='failed',observation=?2 WHERE binding=?1 AND state='prepared'",params![id,json!({"error":"actor authority revoked before dispatch"}).to_string()]))?;
            }
            if doc["thread_id"].as_str().is_some() {
                let observed_stop = match self
                    .shutdown_turn(controller, &id, &project, &doc, server)
                {
                    Ok(value) => value,
                    Err(error) => {
                        json!({"state":"unknown","reason":error,"thread_id":doc["thread_id"],"turn_id":doc["turn_id"],"terminal_receipt":false})
                    }
                };
                if doc["turn_shutdown"]["terminal_receipt"] == true {
                    // A later observation cannot erase a qualified terminal receipt.
                    if observed_stop["terminal_receipt"] != true {
                        doc["cleanup_stop_uncertainty"] = observed_stop.clone();
                    }
                    doc["cleanup_stop_observation"] = observed_stop;
                } else {
                    doc["turn_shutdown"] = observed_stop;
                }
                self.save_binding(controller, &id, &doc, "revoking")?;
                #[cfg(feature = "fault-injection")]
                if let Ok(path) = std::env::var("CORENIUS_TEST_CLEANUP_HISTORY")
                    && Path::new(&format!("{path}.observed")).is_file()
                    && !Path::new(&format!("{path}.paused")).exists()
                {
                    fs::write(
                        format!("{path}.paused"),
                        b"one declared observation barrier",
                    )
                    .map_err(|e| e.to_string())?;
                    let hold = std::env::var("CORENIUS_TEST_HOLD_CODEX")
                        .map_err(|_| "fixture hold missing")?;
                    fs::write(
                        hold,
                        b"declared pause after cleanup history observation loss",
                    )
                    .map_err(|e| e.to_string())?;
                }
                match self.retire_subscription(controller, &id, &project, &doc, server) {
                    Ok(value) => doc["subscription_retirement"] = value,
                    Err(error) => {
                        doc["subscription_retirement"] = json!({"state":"unknown","reason":error});
                        self.save_binding(controller, &id, &doc, "revoking")?;
                        continue;
                    }
                }
                self.save_binding(controller, &id, &doc, "revoking")?;
            }
            if role == "lead" {
                let effect: Binding =
                    serde_json::from_value(doc["executor_effect_binding"].clone())
                        .map_err(|e| e.to_string())?;
                let stopped = supervisor::call(
                    &server.worker_supervisor_socket,
                    &secret(&server.worker_auth_ref)?,
                    "cancel",
                    &effect,
                )?
                .ok_or("revoked lead cancellation missing")?;
                doc["executor_execution"] = json!(stopped);
                if stopped.state != "exited" {
                    self.save_binding(controller, &id, &doc, "unknown")?;
                    continue;
                }
            }
            if self.config.topology == "separate"
                && let Some(server_id) = doc["server_id"].as_str()
            {
                let raw: Option<String> = {
                    let c = controller
                        .connection
                        .lock()
                        .map_err(|_| "controller lock poisoned")?;
                    db(c.query_row(
                        "SELECT document FROM codex_servers WHERE id=?1",
                        [server_id],
                        |r| r.get(0),
                    )
                    .optional())?
                };
                if let Some(raw) = raw {
                    let mut server_doc: Value =
                        serde_json::from_str(&raw).map_err(|e| e.to_string())?;
                    let effect: Binding =
                        serde_json::from_value(server_doc["effect_binding"].clone())
                            .map_err(|e| e.to_string())?;
                    let stopped = supervisor::call(
                        &server.supervisor_socket,
                        &secret(&server.auth_ref)?,
                        "cancel",
                        &effect,
                    )?
                    .ok_or("revoked app cancellation missing")?;
                    server_doc["execution"] = json!(stopped);
                    let c = controller
                        .connection
                        .lock()
                        .map_err(|_| "controller lock poisoned")?;
                    db(c.execute(
                        "UPDATE codex_servers SET document=?2,state=?3 WHERE id=?1",
                        params![
                            server_id,
                            server_doc.to_string(),
                            if stopped.state == "exited" {
                                "retired"
                            } else {
                                "unknown"
                            }
                        ],
                    ))?;
                    if stopped.state != "exited" {
                        drop(c);
                        self.save_binding(controller, &id, &doc, "unknown")?;
                        continue;
                    }
                }
            }
            self.save_binding(controller, &id, &doc, "revoked")?;
            #[cfg(feature = "fault-injection")]
            if role == "attempt"
                && let Ok(path) = std::env::var("CORENIUS_TEST_RETIRED_EVENTS")
                && Path::new(&path).is_file()
            {
                fs::write(format!("{path}.observed"), doc.to_string())
                    .map_err(|e| e.to_string())?;
                let deadline = std::time::Instant::now() + Duration::from_secs(20);
                while Path::new(&path).is_file() {
                    if std::time::Instant::now() >= deadline {
                        return Err("owned retired native event setup deadline".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        Ok(())
    }
    fn active_threads(
        &self,
        controller: &Controller,
        server: &str,
    ) -> Result<std::collections::HashSet<String>, String> {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let mut stmt = db(c.prepare("SELECT document FROM codex_bindings WHERE state<>'revoked' AND json_extract(document,'$.server_id')=?1 AND COALESCE(json_extract(document,'$.authority_state'),'current')<>'revoked' LIMIT 1025"))?;
        let rows = db(stmt.query_map([server], |row| row.get::<_, String>(0)))?;
        let mut threads = std::collections::HashSet::new();
        for (count, raw) in rows.enumerate() {
            if count >= 1024 {
                return Err("native active binding identity bound".into());
            }
            let document: Value = serde_json::from_str(&db(raw)?).map_err(|e| e.to_string())?;
            if let Some(thread) = document["thread_id"].as_str() {
                threads.insert(thread.into());
            }
        }
        if threads.len() > 1024 {
            return Err("native active thread identity bound".into());
        }
        Ok(threads)
    }
    /// Subscription cleanup is a connection-scoped idempotent native effect, after stop disposition.
    fn retire_subscription(
        &self,
        controller: &Controller,
        id: &str,
        project: &str,
        doc: &Value,
        profile: &Server,
    ) -> Result<Value, String> {
        let server = text(doc, "server_id")?;
        let thread = text(doc, "thread_id")?;
        let retained = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt = db(c.prepare("SELECT id,state,intent,observation FROM codex_operations WHERE binding=?1 AND kind='thread/unsubscribe' LIMIT 2"))?;
            let rows = db(stmt.query_map([id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                ))
            }))?;
            let rows = rows
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?;
            if rows.len() > 1 {
                return Err("cleanup logical operation identity ambiguous; unresolved".into());
            }
            rows.into_iter().next()
        };
        if let Some((_, _, intent, _)) = &retained {
            let intent: Value = serde_json::from_str(intent).map_err(|e| e.to_string())?;
            if intent["threadId"] != thread
                || intent["server_id"] != server
                || intent["server_store"] != doc["server_store"]
            {
                return Err("cleanup retained operation binding/store mismatch; unresolved".into());
            }
        }
        let mut connections = self
            .connections
            .lock()
            .map_err(|_| "native connection lock poisoned")?;
        let Some(connected) = connections.get_mut(server) else {
            return match &retained {
                Some((_, state, _, observation)) if state == "confirmed" => {
                    serde_json::from_str(observation).map_err(|e| e.to_string())
                }
                Some(_) => {
                    Err("cleanup connection absent with pending unsubscribe; unresolved".into())
                }
                None => Ok(json!({"state":"connection_absent","thread_id":thread})),
            };
        };
        connected
            .ws
            .retain_threads(self.active_threads(controller, server)?)?;
        connected.subscribed.remove(thread);
        if let Some((_, state, _, observation)) = &retained
            && state == "confirmed"
        {
            return serde_json::from_str(observation).map_err(|e| e.to_string());
        }
        #[cfg(feature = "fault-injection")]
        if retained.is_none()
            && doc["turn_shutdown"]["terminal_receipt"] == true
            && let Ok(path) = std::env::var("CORENIUS_TEST_PRE_UNSUBSCRIBE")
            && Path::new(&path).is_file()
        {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            fs::write(format!("{path}.observed"), doc.to_string()).map_err(|e| e.to_string())?;
            fs::write(
                std::env::var("CORENIUS_TEST_HOLD_CODEX").map_err(|e| e.to_string())?,
                b"pre-unsubscribe barrier",
            )
            .map_err(|e| e.to_string())?;
            return Err("declared pre-unsubscribe observation barrier; unresolved".into());
        }
        // Current connection qualification is independent of later stop-observation failures.
        let record: Value = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let raw: String = db(c.query_row(
                "SELECT document FROM codex_servers WHERE id=?1",
                [server],
                |row| row.get(0),
            ))?;
            serde_json::from_str(&raw).map_err(|e| e.to_string())?
        };
        let effect: Binding =
            serde_json::from_value(record["effect_binding"].clone()).map_err(|e| e.to_string())?;
        let app = supervisor::native_lookup(
            &profile.supervisor_socket,
            &secret(&profile.auth_ref)?,
            &effect,
        )?;
        // The original stop receipt is immutable evidence. Only the first operation may
        // use this drain's fresh qualified terminal observation after a native restart.
        // Pending operations keep their original receipt/connection fencing below.
        let prior_intent = retained
            .as_ref()
            .map(|(_, _, raw, _)| serde_json::from_str::<Value>(raw).map_err(|e| e.to_string()))
            .transpose()?;
        let stop = if let Some(prior) = &prior_intent {
            prior
                .get("stop_qualification")
                .unwrap_or(&doc["turn_shutdown"])
        } else if doc["cleanup_stop_observation"].is_object() {
            &doc["cleanup_stop_observation"]
        } else {
            &doc["turn_shutdown"]
        };
        if app["runtime"]["store_id"] != doc["server_store"]
            || app["runtime"]["incarnation"] != connected.incarnation
            || stop["terminal_receipt"] != true
            || stop["turn_id"] != doc["turn_id"]
            || stop["server_store"] != doc["server_store"]
            || stop["server_incarnation"] != connected.incarnation
        {
            return Err(
                "cleanup current store/incarnation qualification unknown; unresolved".into(),
            );
        }
        let mut intent = json!({"threadId":thread,"server_id":server,"server_store":doc["server_store"],"server_incarnation":connected.incarnation,"connection_id":connected.ws.identity});
        // The first admitted operation retains its exact selected qualification.
        // Legacy pending operations keep their original shape and old receipt;
        // neither case substitutes a later observation or connection on retry.
        if prior_intent
            .as_ref()
            .is_none_or(|prior| prior.get("stop_qualification").is_some())
        {
            intent["stop_qualification"] = stop.clone();
        }
        let intent_raw = intent.to_string();
        if let Some((_, _, prior_intent, _)) = &retained
            && *prior_intent != intent_raw
        {
            return Err("cleanup pending connection/store/incarnation differs; unresolved".into());
        }
        #[cfg(feature = "fault-injection")]
        if let Ok(path) = std::env::var("CORENIUS_TEST_CLEANUP_HISTORY")
            && Path::new(&format!("{path}.observed")).is_file()
            && let Ok(hold) = std::env::var("CORENIUS_TEST_HOLD_CODEX")
            && Path::new(&hold).is_file()
        {
            return Err("declared pause after qualified cleanup observation".into());
        }
        let (operation, previous_state) = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            if let Some((operation, state, _, observation)) = retained {
                if state == "confirmed" {
                    return serde_json::from_str(&observation).map_err(|e| e.to_string());
                }
                db(c.execute(
                    "UPDATE codex_operations SET state='applying' WHERE id=?1",
                    [&operation],
                ))?;
                (operation, state)
            } else {
                let operation = Uuid::new_v4().to_string();
                db(c.execute("INSERT INTO codex_operations VALUES(?1,?2,?3,'thread/unsubscribe','applying',?4,NULL)", params![operation,project,id,intent_raw]))?;
                (operation, "prepared".into())
            }
        };
        let result = connected
            .ws
            .rpc("thread/unsubscribe", json!({"threadId":thread}))
            .and_then(|value| {
                if !matches!(
                    value["status"].as_str(),
                    Some("unsubscribed" | "notSubscribed" | "notLoaded")
                ) {
                    return Err("native unsubscribe status malformed; outcome unknown".into());
                }
                Ok(value)
            });
        #[cfg(feature = "fault-injection")]
        let result = if result.is_ok()
            && let Ok(path) = std::env::var("CORENIUS_TEST_LOSE_UNSUBSCRIBE_ACK")
            && Path::new(&path).is_file()
        {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            fs::write(
                format!("{path}.observed"),
                result.as_ref().unwrap().to_string(),
            )
            .map_err(|e| e.to_string())?;
            Err("actual native unsubscribe acknowledgement discarded by declared fixture".into())
        } else {
            result
        };
        let observation = match &result {
            Ok(value) => {
                json!({"state":"confirmed","native":value,"intent":intent,"previous_state":previous_state,"reconciliation":"connection_scoped_idempotent_unsubscribe"})
            }
            Err(error) => {
                json!({"state":"unknown","error":error,"intent":intent,"previous_state":previous_state})
            }
        };
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            db(c.execute(
                "UPDATE codex_operations SET state=?2,observation=?3 WHERE id=?1",
                params![
                    operation,
                    if result.is_ok() {
                        "confirmed"
                    } else {
                        "unknown"
                    },
                    observation.to_string()
                ],
            ))?;
        }
        result.map(|_| observation)
    }
    /// Withdrawn actor authority permits cleanup of this retained effect, never new turns.
    fn shutdown_turn(
        &self,
        controller: &Controller,
        id: &str,
        project: &str,
        doc: &Value,
        server: &Server,
    ) -> Result<Value, String> {
        let server_id = text(doc, "server_id")?;
        let record = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let raw: String = db(c.query_row(
                "SELECT document FROM codex_servers WHERE id=?1",
                [server_id],
                |r| r.get(0),
            ))?;
            serde_json::from_str::<Value>(&raw).map_err(|e| e.to_string())?
        };
        let effect: Binding =
            serde_json::from_value(record["effect_binding"].clone()).map_err(|e| e.to_string())?;
        let app = supervisor::native_lookup(
            &server.supervisor_socket,
            &secret(&server.auth_ref)?,
            &effect,
        )?;
        if doc["server_store"] != app["runtime"]["store_id"] {
            return Err("cleanup native store mismatch".into());
        }
        let incarnation = text(&app["runtime"], "incarnation")?;
        let mut connections = self
            .connections
            .lock()
            .map_err(|_| "native connection lock poisoned")?;
        if connections
            .get(server_id)
            .is_none_or(|v| v.incarnation != incarnation)
        {
            let mut ws = NativeWs::connect(
                Path::new(text(&app["runtime"], "endpoint")?),
                text(&app, "capability")?,
                text(&app["runtime"], "runtime_start")?,
            )?;
            ws.rpc("initialize",json!({"clientInfo":{"name":"corenius-controller-cleanup","version":"0.1.0"},"capabilities":{"experimentalApi":true}}))?;
            ws.notify("initialized", json!({}))?;
            ws.retain_threads(self.active_threads(controller, server_id)?)?;
            connections.insert(
                server_id.into(),
                Connected {
                    incarnation: incarnation.into(),
                    ws,
                    subscribed: std::collections::HashSet::new(),
                },
            );
        }
        let ws = &mut connections.get_mut(server_id).unwrap().ws;
        ws.allow_thread(text(doc, "thread_id")?)?;
        let observed = ws.rpc(
            "thread/read",
            json!({"threadId":doc["thread_id"],"includeTurns":true}),
        )?;
        #[cfg(feature = "fault-injection")]
        if doc["turn_shutdown"]["terminal_receipt"] == true
            && let Ok(path) = std::env::var("CORENIUS_TEST_CLEANUP_HISTORY")
            && Path::new(&path).is_file()
            && let Ok(lost) = std::env::var("CORENIUS_TEST_LOSE_UNSUBSCRIBE_ACK")
            && Path::new(&format!("{lost}.observed")).is_file()
        {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            fs::write(format!("{path}.observed"), json!({"prior_terminal":doc["turn_shutdown"],"actual_native_history":observed,"server_incarnation":incarnation,"server_store":app["runtime"]["store_id"]}).to_string()).map_err(|e| e.to_string())?;
            return Err(
                "actual native cleanup history acknowledgement discarded by declared fixture"
                    .into(),
            );
        }
        let turns = observed["thread"]["turns"]
            .as_array()
            .ok_or("cleanup native turns missing")?;
        let turn = turns.last().ok_or("cleanup native turn missing")?;
        if turn["id"] != doc["turn_id"] {
            return Err("cleanup current native turn not retained; unknown".into());
        }
        if matches!(
            turn["status"].as_str(),
            Some("completed" | "interrupted" | "failed")
        ) {
            return Ok(
                json!({"state":turn["status"],"turn_id":doc["turn_id"],"terminal_receipt":true,"source":"typed_native_history","server_incarnation":incarnation,"server_store":app["runtime"]["store_id"]}),
            );
        }
        if doc["turn_shutdown"]["terminal_receipt"] == true {
            return Err("cleanup history contradicts retained terminal receipt; unresolved".into());
        }
        self.mutation(
            controller,
            id,
            project,
            "turn/interrupt",
            &json!({"threadId":doc["thread_id"],"turnId":doc["turn_id"]}),
            ws,
        )?;
        let observed = ws.rpc(
            "thread/read",
            json!({"threadId":doc["thread_id"],"includeTurns":true}),
        )?;
        let terminal = observed["thread"]["turns"]
            .as_array()
            .and_then(|turns| turns.iter().find(|turn| turn["id"] == doc["turn_id"]))
            .and_then(|turn| turn["status"].as_str())
            .filter(|s| matches!(*s, "completed" | "interrupted" | "failed"));
        Ok(
            json!({"state":terminal.unwrap_or("interrupt_acknowledged"),"turn_id":doc["turn_id"],"interrupt_receipt":true,"terminal_receipt":terminal.is_some(),"source":"typed_native_reply_and_history","server_incarnation":incarnation,"server_store":app["runtime"]["store_id"]}),
        )
    }
    fn drain_leads(&self, controller: &Controller, server: &Server) -> Result<(), String> {
        let projects = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt =
                db(c.prepare("SELECT id,lead_epoch,base FROM projects WHERE lead_epoch>0"))?;
            let rows = db(stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            }))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?
        };
        for (project, epoch, base) in projects {
            let id = format!("lead:{project}:{epoch}");
            let binding = {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                let raw: Option<String> = db(c
                    .query_row(
                        "SELECT document FROM codex_bindings WHERE id=?1",
                        [&id],
                        |r| r.get(0),
                    )
                    .optional())?;
                if let Some(raw) = raw {
                    let doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
                    serde_json::from_value(doc["executor_effect_binding"].clone())
                        .map_err(|e| e.to_string())?
                } else {
                    let binding = Binding {
                        project: project.clone(),
                        attempt: Uuid::new_v4().to_string(),
                        operation: Uuid::new_v4().to_string(),
                        scope_epoch: epoch,
                        base,
                        brief: format!("lead-authority:{epoch}"),
                    };
                    let doc = json!({"executor_effect_binding":binding,"ui_input_ready":false});
                    db(c.execute(
                        "INSERT INTO codex_bindings VALUES(?1,?2,'lead',NULL,?3,?4,'prepared')",
                        params![id, project, epoch, doc.to_string()],
                    ))?;
                    binding
                }
            };
            let token = secret(&server.worker_auth_ref)?;
            if supervisor::call(&server.worker_supervisor_socket, &token, "lookup", &binding)?
                .is_none()
            {
                supervisor::call(&server.worker_supervisor_socket, &token, "start", &binding)?;
            }
            if let Err(error) = self.bind_actor(
                controller,
                server,
                &binding,
                &format!("You are the lead for project {project} at authority epoch {epoch}. Use the scoped controller tools for team coordination."),
                "lead",
                &id,
            ) {
                self.block_binding(controller,&id)?;
                eprintln!("native lead reconcile: {error}");
            }
        }
        Ok(())
    }
    fn bind_actor(
        &self,
        controller: &Controller,
        server: &Server,
        worker: &Binding,
        body: &str,
        role: &str,
        id: &str,
    ) -> Result<(), String> {
        #[cfg(feature = "fault-injection")]
        if let Some((_, snapshot)) = adoption_fixture_snapshot_path(worker)?
            && Path::new(&snapshot).is_file()
        {
            // Retain this actor's first observed fixture projection while its peer retries.
            return Ok(());
        }
        #[cfg(feature = "fault-injection")]
        adoption_fixture_defer(worker)?;
        let executor = supervisor::native_lookup(
            &server.worker_supervisor_socket,
            &secret(&server.worker_auth_ref)?,
            worker,
        )?;
        if executor["runtime"]["kind"] != "exec"
            || executor["runtime"]["endpoint"]
                != json!(
                    server
                        .worker_transport_root
                        .join(&worker.attempt)
                        .join("native.sock")
                )
        {
            return Err("registered executor endpoint mismatch".into());
        }
        let mut document = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let prior: Option<String> = db(c
                .query_row(
                    "SELECT document FROM codex_bindings WHERE id=?1",
                    [id],
                    |r| r.get(0),
                )
                .optional())?;
            match prior {
                Some(raw) => serde_json::from_str(&raw).map_err(|e| e.to_string())?,
                None => {
                    let value = json!({"server_id":if self.config.topology=="shared"{server.id.clone()}else{format!("{}:{}",server.id,worker.attempt)},"executor":executor["runtime"],"executor_execution":executor["execution"],"ui_input_ready":false});
                    db(c.execute(
                        "INSERT INTO codex_bindings VALUES(?1,?2,?6,?3,?4,?5,'preparing')",
                        params![
                            id,
                            worker.project,
                            if role == "attempt" {
                                Some(&worker.attempt)
                            } else {
                                None
                            },
                            worker.scope_epoch,
                            value.to_string(),
                            role
                        ],
                    ))?;
                    value
                }
            }
        };
        if document["executor_execution"].is_null() {
            document["executor_execution"] = executor["execution"].clone();
            document["executor"] = executor["runtime"].clone();
            document["server_id"] = json!(if self.config.topology == "shared" {
                server.id.clone()
            } else {
                format!("{}:{}", server.id, worker.attempt)
            });
            self.save_binding(controller, id, &document, "preparing")?;
        }
        #[cfg(feature = "fault-injection")]
        if role == "lead"
            && std::thread::current().name() == Some("runtime-reconcile")
            && let Ok(path) = std::env::var("CORENIUS_TEST_UI_DRAIN_BARRIER")
            && Path::new(&path).is_file()
        {
            fs::write(format!("{path}.observed"), document.to_string())
                .map_err(|e| e.to_string())?;
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while Path::new(&path).is_file() {
                if std::time::Instant::now() >= deadline {
                    return Err("declared UI/drain race barrier deadline".into());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        if document["executor_execution"]["handle"] != executor["execution"]["handle"]
            || document["executor"]["workspace_identity"]
                != executor["runtime"]["workspace_identity"]
        {
            return Err("durable executor binding mismatch".into());
        }
        let server_id = text(&document, "server_id")?.to_string();
        let app_binding = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let raw: Option<String> = db(c
                .query_row(
                    "SELECT document FROM codex_servers WHERE id=?1",
                    [&server_id],
                    |r| r.get(0),
                )
                .optional())?;
            if let Some(raw) = raw {
                let value: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
                serde_json::from_value(value["effect_binding"].clone())
                    .map_err(|e| e.to_string())?
            } else {
                let binding = Binding {
                    project: worker.project.clone(),
                    attempt: if self.config.topology == "shared" {
                        Uuid::new_v4().to_string()
                    } else {
                        worker.attempt.clone()
                    },
                    operation: Uuid::new_v4().to_string(),
                    scope_epoch: worker.scope_epoch,
                    base: worker.base.clone(),
                    brief: worker.brief.clone(),
                };
                let value = json!({"effect_binding":binding,"profile":server.id,"schema_digest":server.schema_digest,"auth_ref":server.auth_ref,"topology":self.config.topology});
                db(c.execute(
                    "INSERT INTO codex_servers VALUES(?1,?2,'prepared')",
                    params![server_id, value.to_string()],
                ))?;
                binding
            }
        };
        let token = secret(&server.auth_ref)?;
        let execution =
            match supervisor::call(&server.supervisor_socket, &token, "lookup", &app_binding)? {
                Some(e) => e,
                None => supervisor::call(&server.supervisor_socket, &token, "start", &app_binding)?
                    .ok_or("native server effect absent")?,
            };
        if execution.state != "running" {
            return Err("native server effect not live".into());
        }
        let app = supervisor::native_lookup(&server.supervisor_socket, &token, &app_binding)?;
        if app["runtime"]["kind"] != "app"
            || app["runtime"]["endpoint"]
                != json!(
                    server
                        .transport_root
                        .join(&app_binding.attempt)
                        .join("native.sock")
                )
            || app["runtime"]["own_executor_only"] != json!(self.config.topology == "separate")
        {
            return Err("native server profile/topology mismatch".into());
        }
        if !document["server_store"].is_null()
            && document["server_store"] != app["runtime"]["store_id"]
        {
            return Err("native persistent store generation mismatch".into());
        }
        let incarnation = text(&app["runtime"], "incarnation")?;
        self.adopt_initial_turn(controller, id, &mut document)?;
        if document["server_incarnation"] != incarnation {
            // A creation receipt from another incarnation cannot qualify an empty
            // retained thread. Reject before exposing reconciliation or native effects.
            if let Some(thread) = document["thread_id"].as_str()
                && document["turn_id"].is_null()
            {
                let receipt = self
                    .confirmed_creation(controller, id, "thread/start")?
                    .ok_or("empty native thread creation receipt missing")?;
                if receipt["thread"]["id"] != thread
                    || receipt["_controller_server_incarnation"] != incarnation
                {
                    return Err("empty native thread incarnation unknown; no replay".into());
                }
            }
            document["ui_input_ready"] = json!(false);
            self.save_binding(controller, id, &document, "reconciling")?;
        }
        let mut connections = self
            .connections
            .lock()
            .map_err(|_| "native connection lock poisoned")?;
        if connections
            .get(&server_id)
            .is_none_or(|v| v.incarnation != incarnation)
        {
            let mut ws = NativeWs::connect(
                Path::new(text(&app["runtime"], "endpoint")?),
                text(&app, "capability")?,
                text(&app["runtime"], "runtime_start")?,
            )?;
            let ready=ws.rpc("initialize",json!({"clientInfo":{"name":"corenius-controller","version":"0.1.0"},"capabilities":{"experimentalApi":true}}))?;
            if !text(&ready, "userAgent")?.contains("0.159.3") {
                return Err("native initialize version mismatch".into());
            }
            ws.notify("initialized", json!({}))?;
            ws.retain_threads(self.active_threads(controller, &server_id)?)?;
            connections.insert(
                server_id.clone(),
                Connected {
                    incarnation: incarnation.into(),
                    ws,
                    subscribed: std::collections::HashSet::new(),
                },
            );
        }
        let connected = connections.get_mut(&server_id).unwrap();
        if let Some(thread) = document["thread_id"].as_str() {
            connected.ws.allow_thread(thread)?;
        }
        if let Some(thread) = document["thread_id"].as_str()
            && !connected.subscribed.contains(thread)
        {
            // A persisted empty thread is loaded in the surviving native incarnation,
            // but has no rollout to resume. Validate native memory, never synthesize/recreate it.
            let empty_loaded = if document["turn_id"].is_null() {
                let receipt = self
                    .confirmed_creation(controller, id, "thread/start")?
                    .ok_or("empty native thread creation receipt missing")?;
                if receipt["thread"]["id"] != thread
                    || receipt["_controller_server_incarnation"] != incarnation
                {
                    return Err("empty native thread incarnation unknown; no replay".into());
                }
                if !connected.ws.thread_is_loaded(thread)? {
                    return Err("empty native thread no longer loaded; no replay".into());
                }
                true
            } else {
                false
            };
            if empty_loaded {
                connected.subscribed.insert(thread.into());
            } else {
                let restored = self.mutation(
                controller,
                id,
                &worker.project,
                "thread/resume",
                &json!({"threadId":thread,"approvalPolicy":"never","sandbox":"danger-full-access"}),
                &mut connected.ws,
            )?;
                if restored["thread"]["id"] != thread {
                    return Err("native resume thread mismatch".into());
                }
                connected.subscribed.insert(thread.into());
                document["recovery_state"] = json!("remote_registered_input_blocked");
                document["ui_input_ready"] = json!(false);
                document["resumed_environments"] = restored["thread"]["environments"].clone();
                self.save_binding(controller, id, &document, "reconciling")?;
            }
        }
        let result = self.configure_thread(
            controller,
            id,
            &mut document,
            &mut connected.ws,
            NativeInputs {
                app: &app,
                executor: &executor,
                worker,
                body,
            },
        );
        if let Some(thread) = document["thread_id"].as_str() {
            connected.subscribed.insert(thread.into());
        }
        if result.is_err() {
            connections.remove(&server_id);
        }
        result
    }
    fn adopt_initial_turn(
        &self,
        controller: &Controller,
        id: &str,
        document: &mut Value,
    ) -> Result<(), String> {
        if !document["turn_id"].is_null() || document["thread_id"].is_null() {
            return Ok(());
        }
        let receipts = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt = db(c.prepare("SELECT intent,observation FROM codex_operations WHERE binding=?1 AND kind='turn/start' AND state='confirmed'"))?;
            db(stmt.query_map([id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            }))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?
        };
        if receipts.len() > 1 {
            return Err("multiple initial native turn receipts; no replay".into());
        }
        if let Some((intent, observed)) = receipts.first() {
            let intent: Value = serde_json::from_str(intent).map_err(|e| e.to_string())?;
            let observed: Value = serde_json::from_str(observed).map_err(|e| e.to_string())?;
            if intent["threadId"] != document["thread_id"] {
                return Err("initial native turn receipt thread mismatch".into());
            }
            let turn_id = text(&observed["turn"], "id")?;
            document["turn_id"] = json!(turn_id);
            document["current_turn"] = json!({"id":turn_id,"native_status":observed["turn"]["status"],"state":"unknown","source":"retained_native_acknowledgement"});
            document["ui_input_ready"] = json!(false);
            self.save_binding(controller, id, document, "turn_bound")?;
        }
        Ok(())
    }
    fn confirmed_creation(
        &self,
        controller: &Controller,
        id: &str,
        kind: &str,
    ) -> Result<Option<Value>, String> {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let mut stmt=db(c.prepare("SELECT observation FROM codex_operations WHERE binding=?1 AND kind=?2 AND state='confirmed' ORDER BY rowid"))?;
        let rows = db(stmt.query_map(params![id, kind], |r| r.get::<_, String>(0)))?;
        let rows = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?;
        if rows.len() > 1 {
            return Err("multiple native creation receipts; identity unknown, no replay".into());
        }
        rows.first()
            .map(|raw| serde_json::from_str(raw).map_err(|e| e.to_string()))
            .transpose()
    }
    fn reconcile_turn_ack(
        &self,
        controller: &Controller,
        id: &str,
        document: &mut Value,
        observed: &Value,
    ) -> Result<(), String> {
        let operations = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt=db(c.prepare("SELECT id,kind,intent FROM codex_operations WHERE binding=?1 AND state IN ('unknown','applying') AND kind IN ('codex_turn','turn/start') ORDER BY rowid"))?;
            let rows = db(stmt.query_map([id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            }))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?
        };
        for (operation, kind, raw) in operations {
            let intent: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            let client = if kind == "codex_turn" {
                operation.as_str()
            } else {
                text(&intent, "clientUserMessageId")?
            };
            let expected = if kind == "codex_turn" {
                intent["text"].as_str()
            } else {
                intent["input"][0]["text"].as_str()
            };
            let turns = observed["thread"]["turns"]
                .as_array()
                .ok_or("native turn history missing")?;
            let matching: Vec<_> = turns
                .iter()
                .filter(|turn| {
                    turn["items"].as_array().is_some_and(|items| {
                        items.iter().any(|item| {
                            item["type"] == "userMessage"
                                && item["clientId"] == client
                                && item["content"].as_array().is_some_and(|content| {
                                    content.iter().any(|part| {
                                        part["type"] == "text" && part["text"].as_str() == expected
                                    })
                                })
                        })
                    })
                })
                .collect();
            if matching.len() != 1 {
                continue;
            }
            let turn_id = text(matching[0], "id")?;
            document["turn_id"] = json!(turn_id);
            if let Some(model) = intent["model"].as_str() {
                document["model"] = json!(model);
            }
            let status = matching[0]["status"].as_str();
            document["current_turn"] = json!({"id":turn_id,"native_status":status,"state":match status {Some("inProgress")=>"active",Some("completed"|"interrupted"|"failed")=>"terminal",_=>"unknown"},"source":"typed_native_history"});
            let mut c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let tx = db(c.transaction())?;
            // Present the pinned turn schema using actual matched ID/status/error.
            // Empty items are a history-derived response presentation, never a
            // claim about the lost ACK; full allowed items have separate typed refs.
            let start_response = json!({"turn":{"id":matching[0]["id"],"status":matching[0]["status"],"error":matching[0]["error"],"items":[]}});
            db(tx.execute("UPDATE codex_operations SET state='confirmed',observation=?2 WHERE id=?1 AND state IN ('unknown','applying')",params![operation,json!({"reconciled_from_native_history":true,"thread_id":document["thread_id"],"turn_id":turn_id,"client_user_message_id":client,"server_incarnation":document["server_incarnation"],"server_store":document["server_store"],"native_start_response":start_response,"response_presentation":"history-derived turn metadata; complete allowed items via public_history_reference"}).to_string()]))?;
            crate::control::delivery_result(
                &tx,
                &operation,
                "confirmed",
                &json!({"reconciled_from_native_history":true,"turn_id":turn_id}),
            )?;
            db(tx.execute(
                "UPDATE codex_bindings SET document=?2 WHERE id=?1",
                params![id, document.to_string()],
            ))?;
            db(tx.commit())?;
            #[cfg(feature = "fault-injection")]
            crash_boundary(
                "history_confirmation",
                json!({"operation":operation,"thread_id":document["thread_id"],"turn_id":turn_id}),
            )?;
            document["turn_id"] = json!(turn_id);
        }
        Ok(())
    }
    fn dispatch_prepared(
        &self,
        controller: &Controller,
        id: &str,
        document: &mut Value,
        ws: &mut NativeWs,
        worker: &Binding,
    ) -> Result<(), String> {
        let operations = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut stmt=db(c.prepare("SELECT id,kind,intent FROM codex_operations WHERE binding=?1 AND state='prepared' ORDER BY rowid"))?;
            let rows = db(stmt.query_map([id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            }))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|e| e.to_string())?
        };
        for (operation, kind, raw) in operations {
            let intent: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            #[cfg(feature = "fault-injection")]
            let race_path = if kind == "codex_steer" && intent["questions"].is_array() {
                std::env::var("CORENIUS_TEST_QUESTION_RACE")
                    .ok()
                    .filter(|p| Path::new(p).is_file())
            } else {
                None
            };
            #[cfg(feature = "fault-injection")]
            if let Some(path) = &race_path {
                fs::write(format!("{path}.observed"), operation.as_bytes())
                    .map_err(|e| e.to_string())?;
                let deadline = std::time::Instant::now() + Duration::from_secs(8);
                while Path::new(path).is_file() {
                    if std::time::Instant::now() >= deadline {
                        return Err("question race fixture deadline".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            {
                let mut c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                let allowed: i64 = if intent["role"] == "lead" {
                    db(c.query_row(
                        "SELECT COUNT(*) FROM projects WHERE id=?1 AND lead_epoch=?2",
                        params![worker.project, intent["authority_epoch"].as_i64()],
                        |r| r.get(0),
                    ))?
                } else {
                    db(c.query_row("SELECT COUNT(*) FROM attempts WHERE id=?1 AND project=?2 AND scope_epoch=?3 AND cancel_requested=0 AND state='running'",params![worker.attempt,worker.project,worker.scope_epoch],|r|r.get(0)))?
                };
                let principal_allowed:i64=db(c.query_row("SELECT COUNT(*) FROM principals r LEFT JOIN projects p ON p.id=r.project WHERE r.id=?1 AND r.epoch=?2 AND (r.role='operator' OR (r.role='lead' AND r.epoch=p.lead_epoch))",params![intent["principal"].as_str(),intent["principal_epoch"].as_i64()],|r|r.get(0)))?;
                if allowed != 1 || principal_allowed != 1 {
                    crate::control::fail_delivery(
                        &mut c,
                        &operation,
                        &json!({"error":"authority revoked"}),
                    )?;
                    continue;
                }
                // This rejection has no external effect. Keep the operation prepared
                // if its atomic domain transition aborts, so normal dispatch retries it.
                if kind != "codex_turn" && intent["expected_turn"] != document["turn_id"] {
                    crate::control::fail_delivery(
                        &mut c,
                        &operation,
                        &json!({"error":"stale turn"}),
                    )?;
                    continue;
                }
                if kind == "codex_turn" {
                    let ambiguous: i64 = db(c.query_row("SELECT COUNT(*) FROM codex_operations WHERE binding=?1 AND kind IN ('codex_turn','turn/start') AND state IN ('unknown','applying')", [id], |r| r.get(0)))?;
                    if ambiguous != 0 {
                        continue;
                    }
                }
                db(c.execute(
                    "UPDATE codex_operations SET state='applying' WHERE id=?1",
                    [&operation],
                ))?;
            }
            let native_kind = match kind.as_str() {
                "codex_turn" => "turn/start",
                "codex_steer" => "turn/steer",
                "codex_interrupt" => "turn/interrupt",
                _ => return Err("native operation kind refused".into()),
            };
            let mut params = json!({"threadId":document["thread_id"]});
            if kind != "codex_interrupt" {
                params["input"] = json!([{"type":"text","text":intent["text"],"text_elements":[]}]);
            }
            if kind == "codex_turn" {
                if let Some(model) = intent["model"].as_str() {
                    params["model"] = json!(model);
                }
                params["environments"] = json!([{"environmentId":document["environment_id"],"cwd":"/workspace","runtimeWorkspaceRoots":["/workspace"]}]);
                params["clientUserMessageId"] = json!(operation);
                params["approvalPolicy"] = json!("never");
                params["sandboxPolicy"] = json!({"type":"dangerFullAccess"});
            } else if kind == "codex_steer" {
                params["expectedTurnId"] = document["turn_id"].clone();
            } else {
                params["turnId"] = document["turn_id"].clone();
            }
            let mut handler = |call: &Value, observed: Option<&crate::native_ws::NativeTurn>| {
                crate::gateway::native_call(controller, id, document, call, observed)
            };
            let result = ws.rpc_with_tools(native_kind, params, &mut handler);
            #[cfg(feature = "fault-injection")]
            if result.is_ok()
                && intent["questions"].is_array()
                && let Ok(path) = std::env::var("CORENIUS_TEST_EARLY_QUESTION")
                && Path::new(&path).is_file()
            {
                // Exercise an actual actor acknowledgement while the durable question
                // remains prepared. The ordinary result transaction must preserve it.
                let deadline = std::time::Instant::now() + Duration::from_secs(4);
                loop {
                    let acknowledged = {
                        let c = controller
                            .connection
                            .lock()
                            .map_err(|_| "controller lock poisoned")?;
                        db(c.query_row("SELECT EXISTS(SELECT 1 FROM control_questions WHERE delivery=?1 AND state IN ('acknowledged_by_lead','resolved'))",[&operation],|r|r.get::<_,bool>(0)))?
                    };
                    if acknowledged {
                        fs::remove_file(&path).map_err(|e| e.to_string())?;
                        fs::write(format!("{path}.observed"), operation.as_bytes())
                            .map_err(|e| e.to_string())?;
                        break;
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err("actual early acknowledgement fixture deadline".into());
                    }
                    let mut handler =
                        |call: &Value, observed: Option<&crate::native_ws::NativeTurn>| {
                            crate::gateway::native_call(controller, id, document, call, observed)
                        };
                    ws.rpc_with_tools(
                        "thread/read",
                        json!({"threadId":document["thread_id"],"includeTurns":true}),
                        &mut handler,
                    )?;
                    for event in ws.take_events_for(text(document, "thread_id")?) {
                        if event["method"] == "item/tool/call" {
                            ws.reply_tool(
                                &event,
                                crate::gateway::native_call(
                                    controller,
                                    id,
                                    document,
                                    &event,
                                    ws.tool_turn(&event),
                                )?,
                            )?;
                        }
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            #[cfg(feature = "fault-injection")]
            let result = if kind == "codex_turn" && result.is_ok() {
                if let Ok(path) = std::env::var("CORENIUS_TEST_LOSE_NATIVE_ACK")
                    && Path::new(&path).is_file()
                {
                    fs::remove_file(&path).map_err(|e| e.to_string())?;
                    fs::write(
                        format!("{path}.observed"),
                        b"actual native acknowledgement discarded",
                    )
                    .map_err(|e| e.to_string())?;
                    if std::env::var("CORENIUS_TEST_ACK_FENCE_FIXTURE").as_deref() == Ok("1") {
                        let hold = std::env::var("CORENIUS_TEST_HOLD_CODEX")
                            .map_err(|_| "test hold path required")?;
                        fs::write(hold, b"declared pause after real acknowledgement loss")
                            .map_err(|e| e.to_string())?;
                    }
                    Err(
                        "native acknowledgement deliberately lost by test-only fault injection"
                            .into(),
                    )
                } else {
                    result
                }
            } else {
                result
            };

            {
                let mut c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                let tx = db(c.transaction())?;
                let (state, observation) = match &result {
                    Ok(v) => ("confirmed", v.clone()),
                    Err(e) if e.starts_with("native RPC refused:") => {
                        ("failed", json!({"error":e}))
                    }
                    Err(e) => ("unknown", json!({"error":e})),
                };
                crate::control::delivery_result(&tx, &operation, state, &observation)?;
                if kind == "codex_turn"
                    && let Ok(value) = &result
                {
                    let turn_id = text(&value["turn"], "id")?;
                    document["turn_id"] = json!(turn_id);
                    if let Some(model) = intent["model"].as_str() {
                        document["model"] = json!(model);
                    }
                    let status = value["turn"]["status"].as_str();
                    document["current_turn"] = json!({"id":turn_id,"native_status":status,"state":match status {Some("inProgress")=>"active",Some("completed"|"interrupted"|"failed")=>"terminal",_=>"unknown"},"source":"native_acknowledgement"});
                    db(tx.execute(
                        "UPDATE codex_bindings SET document=?2 WHERE id=?1",
                        params![id, document.to_string()],
                    ))?;
                }
                db(tx.execute(
                    "UPDATE codex_operations SET state=?2,observation=?3 WHERE id=?1",
                    params![operation, state, observation.to_string()],
                ))?;
                db(tx.commit())?;
            }
            #[cfg(feature = "fault-injection")]
            if let Some(path) = &race_path
                && result
                    .as_ref()
                    .is_err_and(|e| e.starts_with("native RPC refused:"))
            {
                fs::write(format!("{path}.failed"), operation.as_bytes())
                    .map_err(|e| e.to_string())?;
                let deadline = std::time::Instant::now() + Duration::from_secs(8);
                while !Path::new(&format!("{path}.continue")).exists() {
                    if std::time::Instant::now() >= deadline {
                        return Err("question race pending observation deadline".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            if kind == "codex_turn" {
                let value = result?;
                #[cfg(feature = "fault-injection")]
                crash_boundary(
                    "turn_confirmation",
                    json!({"operation":operation,"thread_id":document["thread_id"],"turn_id":value["turn"]["id"]}),
                )?;
                document["turn_id"] = value["turn"]["id"].clone();
            }
        }
        Ok(())
    }
    fn block_binding(&self, controller: &Controller, id: &str) -> Result<(), String> {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let raw: Option<String> = db(c
            .query_row(
                "SELECT document FROM codex_bindings WHERE id=?1",
                [id],
                |r| r.get(0),
            )
            .optional())?;
        if let Some(raw) = raw {
            let mut doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            doc["ui_input_ready"] = json!(false);
            doc["recovery_state"] = json!("qualification_unknown");
            db(c.execute(
                "UPDATE codex_bindings SET document=?2,state='unknown' WHERE id=?1",
                params![id, doc.to_string()],
            ))?;
        }
        Ok(())
    }
    fn save_binding(
        &self,
        controller: &Controller,
        id: &str,
        document: &Value,
        state: &str,
    ) -> Result<(), String> {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        db(c.execute(
            "UPDATE codex_bindings SET document=?2,state=?3 WHERE id=?1",
            params![id, document.to_string(), state],
        ))?;
        Ok(())
    }
    fn mutation(
        &self,
        controller: &Controller,
        id: &str,
        project: &str,
        kind: &str,
        intent: &Value,
        ws: &mut NativeWs,
    ) -> Result<Value, String> {
        let operation = Uuid::new_v4().to_string();
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            db(c.execute(
                "INSERT INTO codex_operations VALUES(?1,?2,?3,?4,'applying',?5,NULL)",
                params![operation, project, id, kind, intent.to_string()],
            ))?;
        }
        let result = ws.rpc(kind, intent.clone());
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let (state, observation) = match &result {
                Ok(value) => {
                    let mut observed = value.clone();
                    if kind == "thread/start" {
                        let raw: String = db(c.query_row(
                            "SELECT document FROM codex_bindings WHERE id=?1",
                            [id],
                            |r| r.get(0),
                        ))?;
                        let binding: Value =
                            serde_json::from_str(&raw).map_err(|e| e.to_string())?;
                        observed["_controller_server_incarnation"] =
                            binding["server_incarnation"].clone();
                    }
                    ("confirmed", observed)
                }
                Err(error) => ("unknown", json!({"error":error})),
            };
            db(c.execute(
                "UPDATE codex_operations SET state=?2,observation=?3 WHERE id=?1",
                params![operation, state, observation.to_string()],
            ))?;
        }
        #[cfg(feature = "fault-injection")]
        if kind == "turn/start"
            && let Ok(value) = &result
        {
            crash_boundary(
                "initial_turn_confirmation",
                json!({"operation":operation,"thread_id":intent["threadId"],"turn_id":value["turn"]["id"],"client_user_message_id":intent["clientUserMessageId"]}),
            )?;
        }
        #[cfg(feature = "fault-injection")]
        if kind == "thread/start"
            && result.is_ok()
            && let Ok(path) = std::env::var("CORENIUS_TEST_CRASH_AFTER_NATIVE_CONFIRM")
            && Path::new(&path).is_file()
        {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
            fs::write(
                format!("{path}.observed"),
                b"native thread confirmation journaled before binding adoption",
            )
            .map_err(|e| e.to_string())?;
            std::process::exit(79);
        }
        result
    }
    fn configure_thread(
        &self,
        controller: &Controller,
        id: &str,
        document: &mut Value,
        ws: &mut NativeWs,
        inputs: NativeInputs<'_>,
    ) -> Result<(), String> {
        let NativeInputs {
            app,
            executor,
            worker,
            body,
        } = inputs;
        let incarnation = text(&app["runtime"], "incarnation")?;
        if document["server_incarnation"] != incarnation
            || document["executor"]["incarnation"] != executor["runtime"]["incarnation"]
        {
            // Process identity alone does not prove its authenticated listener can accept
            // registration. A bounded real handshake fails closed before any environment effect.
            NativeWs::require_ready(
                Path::new(text(&executor["runtime"], "endpoint")?),
                text(executor, "capability")?,
                text(&executor["runtime"], "runtime_start")?,
            )?;
            let mut stream = UnixStream::connect(text(&app["runtime"], "control_endpoint")?)
                .map_err(|e| e.to_string())?;
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .map_err(|e| e.to_string())?;
            writeln!(stream,"{}",json!({"token":app["capability"],"action":"executor_bridge","attempt":worker.attempt})).map_err(|e|e.to_string())?;
            let mut reply = String::new();
            BufReader::new(stream)
                .read_line(&mut reply)
                .map_err(|e| e.to_string())?;
            let reply: Value = serde_json::from_str(&reply).map_err(|e| e.to_string())?;
            let url = format!(
                "{}/?corenius_start={}",
                text(&reply["ok"], "url")?,
                text(&executor["runtime"], "runtime_start")?
            );
            let environment_id = format!(
                "executor-{}-{}",
                worker.attempt,
                text(&executor["runtime"], "runtime_start")?
            );
            // Plaintext native capability remains transport-only, never enters intent/observation storage.
            let safe = json!({"environmentId":environment_id,"execServerUrl":url,"auth_ref":executor["runtime"]["auth_ref"],"executor_incarnation":executor["runtime"]["incarnation"]});
            let op = Uuid::new_v4().to_string();
            {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.execute("INSERT INTO codex_operations VALUES(?1,?2,?3,'environment/add','applying',?4,NULL)",params![op,worker.project,id,safe.to_string()]))?;
            }
            let added=ws.rpc("environment/add",json!({"environmentId":environment_id,"execServerUrl":url,"authBearerToken":executor["capability"],"connectTimeoutMs":1500}));
            {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.execute(
                    "UPDATE codex_operations SET state=?2,observation=?3 WHERE id=?1",
                    params![
                        op,
                        if added.is_ok() {
                            "confirmed"
                        } else {
                            "unknown"
                        },
                        json!({"registered":added.is_ok()}).to_string()
                    ],
                ))?;
            }
            added?;
            document["environment_id"] = json!(environment_id);
            document["server_incarnation"] = json!(incarnation);
            document["server_store"] = app["runtime"]["store_id"].clone();
            document["server_execution"] = app["execution"].clone();
            document["executor"] = executor["runtime"].clone();
            self.save_binding(controller, id, document, "remote_registered")?;
        }
        let status = ws.rpc(
            "environment/status",
            json!({"environmentId":document["environment_id"]}),
        )?;
        if status["status"] != "ready" {
            return Err(format!("remote executor not ready: {status}"));
        }
        let environments = json!([{"environmentId":document["environment_id"],"cwd":"/workspace","runtimeWorkspaceRoots":["/workspace"]}]);
        let model =
            selected_model(document["model"].as_str().or(self.config.model.as_deref()))?.to_owned();
        if document["thread_id"].is_null()
            && let Some(observed) = self.confirmed_creation(controller, id, "thread/start")?
        {
            if observed["_controller_server_incarnation"] != incarnation {
                return Err("unmaterialized native creation incarnation unknown; no replay".into());
            }
            document["thread_id"] = observed["thread"]["id"].clone();
            ws.allow_thread(text(document, "thread_id")?)?;
            // A newly created native thread is loaded but has no resumable rollout until its first turn.
            // Same-incarnation adoption uses that exact loaded identity, never another thread/start.
            self.save_binding(controller, id, document, "thread_bound")?;
        }
        if document["thread_id"].is_null() {
            let prior_unknown = {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.query_row("SELECT COUNT(*) FROM codex_operations WHERE binding=?1 AND kind='thread/start' AND state<>'confirmed'",[id],|r|r.get::<_,i64>(0)))?
            };
            if prior_unknown != 0 {
                return Err("thread creation acknowledgement unknown; no replay".into());
            }
            #[cfg(feature = "fault-injection")]
            if body.starts_with("TASK_A")
                && let Ok(path) = std::env::var("CORENIUS_TEST_PAGE_SETUP")
                && Path::new(&path).is_file()
            {
                fs::write(format!("{path}.observed"), document.to_string())
                    .map_err(|e| e.to_string())?;
                let deadline = std::time::Instant::now() + Duration::from_secs(120);
                while Path::new(&path).is_file() {
                    if std::time::Instant::now() >= deadline {
                        return Err("owned native pagination setup deadline".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            let result=self.mutation(controller,id,&worker.project,"thread/start",&json!({"model":model,"modelProvider":"corenius-relay","cwd":"/workspace","approvalPolicy":"never","sandbox":"danger-full-access","environments":environments,"dynamicTools":crate::gateway::specifications()}),ws)?;
            document["thread_id"] = result["thread"]["id"].clone();
            document["model"] = json!(model);
            if document["thread_id"].as_str().is_none() {
                return Err("native thread identity missing".into());
            }
            self.save_binding(controller, id, document, "thread_bound")?;
            #[cfg(feature = "fault-injection")]
            crash_boundary(
                "empty_thread_binding",
                json!({"thread_id":document["thread_id"],"server_incarnation":document["server_incarnation"]}),
            )?;
        }
        self.adopt_initial_turn(controller, id, document)?;
        if document["turn_id"].is_null() {
            let prior_unknown = {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.query_row("SELECT COUNT(*) FROM codex_operations WHERE binding=?1 AND kind='turn/start' AND state<>'confirmed'",[id],|r|r.get::<_,i64>(0)))?
            };
            if prior_unknown == 0 {
                let result=self.mutation(controller,id,&worker.project,"turn/start",&json!({"model":model,"threadId":document["thread_id"],"clientUserMessageId":Uuid::new_v4().to_string(),"input":[{"type":"text","text":body,"text_elements":[]}],"environments":environments}),ws)?;
                document["turn_id"] = result["turn"]["id"].clone();
                if document["turn_id"].as_str().is_none() {
                    return Err("native turn identity missing".into());
                }
                self.save_binding(controller, id, document, "turn_bound")?;
            }
        }
        #[cfg(feature = "fault-injection")]
        if body.starts_with("TASK_A")
            && let Ok(path) = std::env::var("CORENIUS_TEST_EVENT_BARRIER")
            && Path::new(&path).is_file()
        {
            fs::write(
                format!("{path}.observed"),
                document["thread_id"].to_string(),
            )
            .map_err(|e| e.to_string())?;
            let deadline = std::time::Instant::now() + Duration::from_secs(15);
            while Path::new(&path).is_file() {
                if std::time::Instant::now() >= deadline {
                    return Err("test event barrier deadline".into());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let observed = ws.rpc(
            "thread/read",
            json!({"threadId":document["thread_id"],"includeTurns":true}),
        )?;
        document["observed_environments"] = observed["thread"]["environments"].clone();
        document["native_turns"] = observed["thread"]["turns"].clone();
        let mut runtime_secrets = vec![
            text(app, "capability")?.to_owned(),
            text(executor, "capability")?.to_owned(),
        ];
        for server in &self.config.servers {
            runtime_secrets.push(secret(&server.auth_ref)?);
            runtime_secrets.push(secret(&server.worker_auth_ref)?);
        }
        crate::monitor::ingest_native(controller, id, document, &runtime_secrets)?;
        document["ui_input_ready"] = json!(observed["thread"]["environments"] == environments);
        if document["ui_input_ready"] == true {
            document["recovery_state"] = json!("remote_validated");
        }
        self.reconcile_turn_ack(controller, id, document, &observed)?;
        crate::control::prepare_delivery(controller, id, document)?;
        self.dispatch_prepared(controller, id, document, ws, worker)?;
        let turns = document["native_turns"].as_array();
        let turn =
            turns.and_then(|turns| turns.iter().find(|turn| turn["id"] == document["turn_id"]));
        let status = turn.and_then(|turn| turn["status"].as_str());
        document["current_turn"] = json!({"id":document["turn_id"],"native_status":status,"state":match status {Some("inProgress")=>"active",Some("completed"|"interrupted"|"failed")=>"terminal",_=>"unknown"},"source":"typed_native_history"});
        #[cfg(feature = "fault-injection")]
        if let Ok(path) = std::env::var("CORENIUS_TEST_EVENT_BARRIER")
            && Path::new(&format!("{path}.observed")).is_file()
        {
            let actor = if body.starts_with("TASK_A") { "A" } else { "B" };
            fs::write(
                format!("{path}.native-{actor}"),
                serde_json::to_vec(ws.buffered_events()).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        }
        let activity = ws.take_events_for(text(document, "thread_id")?);
        let mut visible = Vec::new();
        for event in activity {
            if event["method"] == "item/tool/call" {
                let result = crate::gateway::native_call(
                    controller,
                    id,
                    document,
                    &event,
                    ws.tool_turn(&event),
                )
                .unwrap_or_else(
                    |e| json!({"contentItems":[{"type":"inputText","text":e}],"success":false}),
                );
                ws.reply_tool(&event, result)?;
            } else if !event["method"]
                .as_str()
                .is_some_and(|m| m.contains("reasoning"))
            {
                visible.push(event);
            }
        }
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            for event in &visible {
                if event["method"].as_str().is_some_and(|m| {
                    !m.contains("delta") && !m.contains("reasoning") && !m.contains("tokenUsage")
                }) {
                    let raw = event.to_string();
                    if raw.len() <= 262144 {
                        db(c.execute("INSERT OR IGNORE INTO control_ui_events(project,binding,digest,document) VALUES(?1,?2,?3,?4)",params![worker.project,id,crate::hash(&raw),raw]))?;
                    }
                }
            }
        }
        document["native_activity"] = json!(visible);
        self.save_binding(controller, id, document, "bound")?;
        #[cfg(feature = "fault-injection")]
        adoption_fixture_snapshot(worker, document)?;
        #[cfg(feature = "fault-injection")]
        if body.starts_with("TASK_B")
            && let Ok(path) = std::env::var("CORENIUS_TEST_RETIRED_EVENTS")
            && Path::new(&format!("{path}.observed")).is_file()
        {
            let events = document["native_activity"]
                .as_array()
                .ok_or("fixture native activity absent")?;
            if events
                .iter()
                .any(|event| event["method"] == "thread/name/updated")
            {
                fs::write(format!("{path}.peer"), document.to_string())
                    .map_err(|e| e.to_string())?;
                let hold = std::env::var("CORENIUS_TEST_HOLD_CODEX")
                    .map_err(|_| "fixture hold missing")?;
                fs::write(
                    hold,
                    b"declared observation freeze after real peer notification",
                )
                .map_err(|e| e.to_string())?;
            }
        }
        #[cfg(feature = "fault-injection")]
        if body.starts_with("TASK_B")
            && let Ok(path) = std::env::var("CORENIUS_TEST_EVENT_BARRIER")
            && Path::new(&format!("{path}.observed")).is_file()
            && !Path::new(&format!("{path}.complete")).exists()
        {
            let hold =
                std::env::var("CORENIUS_TEST_HOLD_CODEX").map_err(|_| "test hold missing")?;
            fs::write(hold, b"declared event observation pause").map_err(|e| e.to_string())?;
            fs::write(
                format!("{path}.complete"),
                document["thread_id"].to_string(),
            )
            .map_err(|e| e.to_string())?;
        }
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let value = json!({"effect_binding":app["execution"]["binding"],"execution":app["execution"],"runtime":app["runtime"],"schema_digest":SCHEMA,"topology":self.config.topology,"capabilities":{"protocol_version":"0.159.3","explicit_remote_turn_environment":true,"controller_methods":["initialize","environment/add","environment/status","thread/start","thread/resume","thread/read","turn/start","turn/steer","turn/interrupt"],"raw_endpoint_controller_only":true,"scoped_ui_gateway":"controller-bound-0.159.3"}});
            db(c.execute(
                "UPDATE codex_servers SET document=?2,state='qualified' WHERE id=?1",
                params![document["server_id"].as_str(), value.to_string()],
            ))?;
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TurnCommand {
    attempt: Option<String>,
    role: Option<String>,
    text: Option<String>,
    expected_turn: Option<String>,
    expected_binding: Option<Value>,
    model: Option<String>,
}
pub(super) fn prepare(
    tx: &rusqlite::Transaction<'_>,
    p: &crate::Principal,
    project: &str,
    kind: &str,
    args: &Value,
) -> Result<Value, String> {
    let input: TurnCommand = serde_json::from_value(args.clone())
        .map_err(|e| format!("native command arguments: {e}"))?;
    let role = input.role.as_deref().unwrap_or("attempt");
    if !matches!(role, "attempt" | "lead") || p.role == "attempt" {
        return Err("native actor authority refused".into());
    }
    let (id, epoch, document): (String, i64, String) = if role == "lead" {
        if input.attempt.is_some() {
            return Err("lead attempt selector refused".into());
        }
        db(tx.query_row("SELECT b.id,b.authority_epoch,b.document FROM codex_bindings b JOIN projects p ON p.id=b.project AND p.lead_epoch=b.authority_epoch WHERE b.project=?1 AND b.role='lead'",[project],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?
    } else {
        let attempt = input.attempt.as_deref().ok_or("attempt required")?;
        db(tx.query_row("SELECT b.id,b.authority_epoch,b.document FROM codex_bindings b JOIN attempts a ON a.id=b.attempt AND a.scope_epoch=b.authority_epoch WHERE b.project=?1 AND b.attempt=?2 AND a.project=?1 AND a.cancel_requested=0 AND a.state='running'",params![project,attempt],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?
    };
    let document: Value = serde_json::from_str(&document).map_err(|e| e.to_string())?;
    if kind == "codex_turn" {
        let ambiguous: i64 = db(tx.query_row("SELECT COUNT(*) FROM codex_operations WHERE binding=?1 AND kind IN ('codex_turn','turn/start') AND state IN ('unknown','applying')", [&id], |r| r.get(0)))?;
        if ambiguous != 0 {
            return Err("native start acknowledgement unknown; reconcile before new turn".into());
        }
    }
    if let Some(expected) = input.expected_binding {
        let fields = expected
            .as_object()
            .ok_or("expected binding object required")?;
        for (key, value) in fields {
            if !matches!(
                key.as_str(),
                "thread_id" | "server_store" | "server_incarnation" | "environment_id"
            ) || &document[key] != value
            {
                return Err("native binding guard mismatch".into());
            }
        }
    }
    if kind != "codex_interrupt"
        && input
            .text
            .as_ref()
            .is_none_or(|t| t.is_empty() || t.len() > 65536)
    {
        return Err("native input text required".into());
    }
    if kind != "codex_turn" && input.expected_turn.as_deref() != document["turn_id"].as_str() {
        return Err("native expected turn mismatch".into());
    }
    if kind != "codex_turn" && input.model.is_some() {
        return Err("native model selection requires a new turn".into());
    }
    let operation = Uuid::new_v4().to_string();
    let mut intent = json!({"role":role,"authority_epoch":epoch,"text":input.text,"expected_turn":input.expected_turn,"principal":p.id,"principal_epoch":p.epoch});
    if kind == "codex_turn" {
        let configured: Option<String> = if input.model.is_none() && document["model"].is_null() {
            let raw: Option<String> = db(tx
                .query_row(
                    "SELECT document FROM codex_configuration WHERE id=1",
                    [],
                    |r| r.get(0),
                )
                .optional())?;
            raw.map(|raw| {
                serde_json::from_str::<Config>(&raw)
                    .map(|c| c.model)
                    .map_err(|e| e.to_string())
            })
            .transpose()?
            .flatten()
        } else {
            None
        };
        intent["model"] = json!(selected_model(
            input
                .model
                .as_deref()
                .or(document["model"].as_str())
                .or(configured.as_deref())
        )?);
    }
    db(tx.execute(
        "INSERT INTO codex_operations VALUES(?1,?2,?3,?4,'prepared',?5,NULL)",
        params![operation, project, id, kind, intent.to_string()],
    ))?;
    Ok(json!({"operation":operation,"binding":id,"state":"prepared"}))
}

// Caller-level recovery regression through the real dispatcher and on-disk SQLite.
#[cfg(test)]
mod repair1_dispatcher_tests {
    use super::*;
    fn command(c: &Controller, kind: &str, key: &str, args: Value) -> Value {
        let revision=c.handle(json!({"auth":{"principal":"operator","token":"standards-review-secret"},"project":"p","query":{"kind":"snapshot","args":{}}})).unwrap()["revision"].clone();
        c.handle(json!({"auth":{"principal":"operator","token":"standards-review-secret"},"project":"p","command":{"kind":kind,"idempotency_key":key,"expected_revision":revision,"args":args}})).unwrap()
    }
    #[test]
    fn repair1_dispatcher_local_refusal_remains_retryable_after_abort_reopen() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("state.db");
        let c = Controller::open(&path, "standards-review-secret").unwrap();
        c.handle(json!({"auth":{"principal":"operator","token":"standards-review-secret"},"project":"p","command":{"kind":"create_project","idempotency_key":"project","expected_revision":0,"args":{"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}}})).unwrap();
        command(
            &c,
            "create_task",
            "task",
            json!({"task":"task","brief":"bounded intent","work_ref":{"deployment":"fixture","project":"p","item":"task"},"source_revision":"r1"}),
        );
        let worker = command(&c, "admit", "worker", json!({"task":"task"}));
        command(&c, "replace_lead", "lead", json!({}));
        let q = command(
            &c,
            "ask_question",
            "question",
            json!({"task":"task","text":"question requiring durable delivery"}),
        )["result"]["question"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut doc = json!({"thread_id":"lead-thread","turn_id":"old-turn","ui_input_ready":true,"native_turns":[{"id":"old-turn","status":"inProgress","items":[]}]});
        {
            let db = c.connection.lock().unwrap();
            db.execute_batch(include_str!("codex_schema.sql")).unwrap();
            db.execute(
                "INSERT INTO codex_bindings VALUES('lead-binding','p','lead',NULL,1,?1,'bound')",
                [doc.to_string()],
            )
            .unwrap();
        }
        crate::control::prepare_delivery(&c, "lead-binding", &doc).unwrap();
        let operation: String = {
            let db = c.connection.lock().unwrap();
            db.query_row(
                "SELECT delivery FROM control_questions WHERE question=?1",
                [&q],
                |r| r.get(0),
            )
            .unwrap()
        };
        {
            let db = c.connection.lock().unwrap();
            db.execute_batch("CREATE TRIGGER review_injected_delivery_failure BEFORE UPDATE OF state ON control_questions WHEN OLD.state='prepared' AND NEW.state='persisted' BEGIN SELECT RAISE(ABORT,'review injected delivery write failure'); END;").unwrap();
        }
        doc["turn_id"] = json!("new-turn");
        doc["native_turns"] = json!([{"id":"new-turn","status":"completed","items":[]}]);
        let runtime = CodexRuntime {
            config: Config {
                schema_migration: "none".into(),
                topology: "shared".into(),
                transport_root: root.path().into(),
                servers: vec![],
                model: None,
            },
            connections: Mutex::new(HashMap::new()),
            lifecycle: Mutex::new(()),
        };
        let (mut ws, _peer) = NativeWs::owned_test_pair();
        let binding = Binding {
            project: "p".into(),
            attempt: worker["result"]["attempt"].as_str().unwrap().into(),
            operation: "review".into(),
            scope_epoch: 1,
            base: worker["result"]["base"].as_str().unwrap().into(),
            brief: worker["result"]["brief"].as_str().unwrap().into(),
        };
        let result = runtime.dispatch_prepared(&c, "lead-binding", &mut doc, &mut ws, &binding);
        assert!(
            result
                .as_ref()
                .unwrap_err()
                .contains("review injected delivery write failure")
        );
        {
            let db = c.connection.lock().unwrap();
            db.execute_batch("DROP TRIGGER review_injected_delivery_failure;")
                .unwrap();
        }
        drop(c);
        let reopened = Controller::open(&path, "standards-review-secret").unwrap();
        runtime
            .reconcile_turn_ack(
                &reopened,
                "lead-binding",
                &mut doc,
                &json!({"thread":{"turns":[]}}),
            )
            .unwrap();
        runtime
            .dispatch_prepared(&reopened, "lead-binding", &mut doc, &mut ws, &binding)
            .unwrap();
        crate::control::prepare_delivery(&reopened, "lead-binding", &doc).unwrap();
        let (operation_state, question_state, count): (String, String, i64) = {
            let db = reopened.connection.lock().unwrap();
            db.query_row("SELECT o.state,(SELECT state FROM control_questions WHERE question=?2),(SELECT COUNT(*) FROM codex_operations) FROM codex_operations o WHERE o.id=?1",params![operation,q],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap()
        };
        println!(
            "REPAIR1_LOCAL_REFUSAL {}",
            json!({"operation_state":operation_state,"question_state":question_state,"operations":count})
        );
        assert_eq!(operation_state, "failed");
        assert_eq!(question_state, "prepared");
        assert_eq!(count, 2);
        println!(
            "STANDARDS_DELIVERY_RECOVERY {}",
            json!({"fault":"declared SQLite domain-write abort at actual dispatcher; recovery retains pre-effect prepared state","dispatcher_error":result.unwrap_err(),"reopened_operation_state":operation_state,"reopened_question_state":question_state,"operations_after_reprepare":count,"native_processes_used":0})
        );
    }
}

#[cfg(test)]
mod selected_model_tests {
    use super::*;
    use std::io::Read;
    const TOKEN: &str = "selected-model-fixture-secret";
    fn command(
        c: &Controller,
        auth: Value,
        kind: &str,
        key: &str,
        args: Value,
    ) -> Result<Value, String> {
        let revision = c.handle(json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","query":{"kind":"snapshot","args":{}}}))?["revision"].clone();
        c.handle(json!({"auth":auth,"project":"p","command":{"kind":kind,"idempotency_key":key,"expected_revision":revision,"args":args}}))
    }
    pub(super) fn fixture() -> (
        tempfile::TempDir,
        Controller,
        Value,
        Value,
        Value,
        Binding,
        CodexRuntime,
    ) {
        let root = tempfile::tempdir().unwrap();
        let c = Controller::open(root.path().join("state.db"), TOKEN).unwrap();
        let auth = json!({"principal":"operator","token":TOKEN});
        c.handle(json!({"auth":auth,"project":"p","command":{"kind":"create_project","idempotency_key":"project","expected_revision":0,"args":{"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}}})).unwrap();
        command(&c, auth.clone(), "create_task", "task", json!({"task":"task","brief":"intent","work_ref":{"deployment":"fixture","project":"p","item":"task"},"source_revision":"r1"})).unwrap();
        let worker =
            command(&c, auth.clone(), "admit", "admit", json!({"task":"task"})).unwrap()["result"]
                .clone();
        let lead =
            command(&c, auth.clone(), "replace_lead", "lead", json!({})).unwrap()["result"].clone();
        let doc = json!({"thread_id":"lead-thread","server_store":"store","server_incarnation":"incarnation","environment_id":"env","turn_id":"old","current_turn":{"state":"terminal"},"ui_input_ready":true});
        {
            let db = c.connection.lock().unwrap();
            db.execute_batch(include_str!("codex_schema.sql")).unwrap();
            db.execute(
                "INSERT INTO codex_bindings VALUES('lead-binding','p','lead',NULL,1,?1,'bound')",
                [doc.to_string()],
            )
            .unwrap();
        }
        let binding = Binding {
            project: "p".into(),
            attempt: worker["attempt"].as_str().unwrap().into(),
            operation: "fixture".into(),
            scope_epoch: 1,
            base: worker["base"].as_str().unwrap().into(),
            brief: worker["brief"].as_str().unwrap().into(),
        };
        let runtime = CodexRuntime { config:serde_json::from_value(json!({"schema_migration":"1-to-2","topology":"shared","transport_root":"/run/corenius-tmp/model-fixture","servers":[]})).unwrap(), connections:Mutex::new(HashMap::new()), lifecycle:Mutex::new(()) };
        (root, c, lead, worker, doc, binding, runtime)
    }
    pub(super) fn native_peer(mut peer: UnixStream) -> std::thread::JoinHandle<Value> {
        std::thread::spawn(move || {
            let mut head = [0u8; 2];
            peer.read_exact(&mut head).unwrap();
            let mut length = (head[1] & 127) as usize;
            if length == 126 {
                let mut bytes = [0u8; 2];
                peer.read_exact(&mut bytes).unwrap();
                length = u16::from_be_bytes(bytes) as usize;
            }
            if length == 127 {
                let mut bytes = [0u8; 8];
                peer.read_exact(&mut bytes).unwrap();
                length = u64::from_be_bytes(bytes) as usize;
            }
            let mut mask = [0u8; 4];
            peer.read_exact(&mut mask).unwrap();
            let mut body = vec![0; length];
            peer.read_exact(&mut body).unwrap();
            for (i, b) in body.iter_mut().enumerate() {
                *b ^= mask[i % 4];
            }
            let request: Value = serde_json::from_slice(&body).unwrap();
            let body=json!({"id":request["id"],"result":{"turn":{"id":"model-turn","status":"completed","items":[]}}}).to_string().into_bytes();
            let mut frame = vec![0x81];
            if body.len() < 126 {
                frame.push(body.len() as u8);
            } else {
                frame.push(126);
                frame.extend((body.len() as u16).to_be_bytes());
            }
            frame.extend(body);
            peer.write_all(&frame).unwrap();
            request
        })
    }
    #[test]
    fn selected_model_core_command_preserves_role_and_idempotency_fences() {
        let (_root, c, lead, worker, _, _, _) = fixture();
        let args = json!({"role":"lead","text":"selected","model":"gpt-6.1-sol","expected_binding":{"thread_id":"lead-thread"}});
        let auth = json!({"principal":lead["principal"],"token":lead["token"]});
        let revision=c.handle(json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","query":{"kind":"snapshot","args":{}}})).unwrap()["revision"].clone();
        let request = json!({"auth":auth,"project":"p","command":{"kind":"codex_turn","idempotency_key":"selected-core","expected_revision":revision,"args":args}});
        let receipt = c.handle(request.clone()).unwrap();
        assert_eq!(c.handle(request.clone()).unwrap(), receipt);
        let mut changed = request.clone();
        changed["command"]["args"]["model"] = json!("gpt-6-sol");
        assert!(
            c.handle(changed)
                .unwrap_err()
                .contains("idempotency argument mismatch")
        );
        let mut foreign = args.clone();
        foreign["expected_binding"]["thread_id"] = json!("foreign");
        assert!(
            command(&c, auth.clone(), "codex_turn", "foreign", foreign)
                .unwrap_err()
                .contains("binding guard")
        );
        let worker_auth = json!({"principal":worker["principal"],"token":worker["token"]});
        let operations_before: i64 = c
            .connection
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM codex_operations", [], |r| r.get(0))
            .unwrap();
        assert!(
            command(&c, worker_auth, "codex_turn", "worker-forbidden", args)
                .unwrap_err()
                .contains("authorization")
        );
        let operations_after: i64 = c
            .connection
            .lock()
            .unwrap()
            .query_row("SELECT COUNT(*) FROM codex_operations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            operations_after, operations_before,
            "denied worker creates no dispatch intent"
        );
        assert!(
            command(
                &c,
                auth.clone(),
                "codex_turn",
                "unsupported",
                json!({"role":"lead","text":"selected","model":"not-installed"})
            )
            .is_err()
        );
        for kind in ["codex_steer", "codex_interrupt"] {
            assert!(command(&c,auth.clone(),kind,kind,json!({"role":"lead","text":"selected","expected_turn":"old","model":"gpt-6.1-sol"})).is_err());
        }
        for (key, expected) in [
            ("legacy-default", "gpt-6-sol"),
            ("configured-default", "gpt-6.1-sol"),
            ("bound-default", "gpt-6-astra"),
        ] {
            if key == "configured-default" {
                c.connection.lock().unwrap().execute("INSERT INTO codex_configuration VALUES(1,?1)",[json!({"schema_migration":"1-to-2","topology":"shared","transport_root":"/run/corenius-tmp/model-fixture","servers":[],"model":"gpt-6.1-sol"}).to_string()]).unwrap();
            } else if key == "bound-default" {
                c.connection.lock().unwrap().execute("UPDATE codex_bindings SET document=json_set(document,'$.model','gpt-6-astra') WHERE id='lead-binding'",[]).unwrap();
            }
            let receipt = command(
                &c,
                auth.clone(),
                "codex_turn",
                key,
                json!({"role":"lead","text":"omitted model"}),
            )
            .unwrap();
            let intent: Value = serde_json::from_str(
                &c.connection
                    .lock()
                    .unwrap()
                    .query_row(
                        "SELECT intent FROM codex_operations WHERE id=?1",
                        [receipt["result"]["operation"].as_str().unwrap()],
                        |r| r.get::<_, String>(0),
                    )
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(intent["model"], expected);
        }
    }
    #[test]
    fn selected_model_worker_initial_turn_and_thread_intents_match_configuration() {
        let (_root, c, _, worker, _, binding, mut runtime) = fixture();
        runtime.config=serde_json::from_value(json!({"schema_migration":"1-to-2","topology":"shared","transport_root":"/run/corenius-tmp/model-fixture","servers":[],"model":"gpt-6.1-sol"})).unwrap();
        let mut doc = json!({"server_id":"fixture-server","server_store":"fixture-store","server_incarnation":"incarnation","environment_id":"env","executor":{"incarnation":"executor"}});
        {
            let db = c.connection.lock().unwrap();
            db.execute(
                "INSERT INTO codex_bindings VALUES('worker-binding','p','attempt',?1,1,?2,'bound')",
                params![worker["attempt"].as_str(), doc.to_string()],
            )
            .unwrap();
        }
        let (mut ws, mut peer) = NativeWs::owned_test_pair();
        let capture = std::thread::spawn(move || {
            let mut calls = vec![];
            for expected in [
                "environment/status",
                "thread/start",
                "turn/start",
                "thread/read",
            ] {
                let mut head = [0u8; 2];
                peer.read_exact(&mut head).unwrap();
                let mut n = (head[1] & 127) as usize;
                if n == 126 {
                    let mut bytes = [0u8; 2];
                    peer.read_exact(&mut bytes).unwrap();
                    n = u16::from_be_bytes(bytes) as usize;
                }
                if n == 127 {
                    let mut bytes = [0u8; 8];
                    peer.read_exact(&mut bytes).unwrap();
                    n = u64::from_be_bytes(bytes) as usize;
                }
                let mut mask = [0u8; 4];
                peer.read_exact(&mut mask).unwrap();
                let mut body = vec![0; n];
                peer.read_exact(&mut body).unwrap();
                for (i, b) in body.iter_mut().enumerate() {
                    *b ^= mask[i % 4];
                }
                let call: Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(call["method"], expected);
                let result = match expected {
                    "environment/status" => json!({"status":"ready"}),
                    "thread/start" => json!({"thread":{"id":"worker-thread"}}),
                    "turn/start" => {
                        json!({"turn":{"id":"worker-turn","status":"completed","items":[]}})
                    }
                    _ => {
                        json!({"thread":{"id":"worker-thread","environments":[{"environmentId":"env","cwd":"/workspace","runtimeWorkspaceRoots":["/workspace"]}],"turns":[{"id":"worker-turn","status":"completed","items":[]}]}})
                    }
                };
                let body = json!({"id":call["id"],"result":result})
                    .to_string()
                    .into_bytes();
                let mut frame = vec![0x81];
                if body.len() < 126 {
                    frame.push(body.len() as u8);
                } else {
                    frame.push(126);
                    frame.extend((body.len() as u16).to_be_bytes());
                }
                frame.extend(body);
                peer.write_all(&frame).unwrap();
                calls.push(call);
            }
            calls
        });
        let app =
            json!({"runtime":{"incarnation":"incarnation"},"capability":"fixture-app-secret"});
        let executor =
            json!({"runtime":{"incarnation":"executor"},"capability":"fixture-worker-secret"});
        runtime
            .configure_thread(
                &c,
                "worker-binding",
                &mut doc,
                &mut ws,
                NativeInputs {
                    app: &app,
                    executor: &executor,
                    worker: &binding,
                    body: "useful worker intent",
                },
            )
            .unwrap();
        let sent = capture.join().unwrap();
        for call in sent
            .iter()
            .filter(|c| matches!(c["method"].as_str(), Some("thread/start" | "turn/start")))
        {
            assert_eq!(call["params"]["model"], "gpt-6.1-sol");
        }
        let db = c.connection.lock().unwrap();
        let mut statement=db.prepare("SELECT intent FROM codex_operations WHERE binding='worker-binding' AND kind IN ('thread/start','turn/start')").unwrap();
        let intents = statement
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| serde_json::from_str::<Value>(&r.unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(intents.len(), 2);
        assert!(intents.iter().all(|i| i["model"] == "gpt-6.1-sol"));
    }
}
