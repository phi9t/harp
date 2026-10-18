//! Pinned native TUI allowlist. A native bearer is never a participant credential.
use super::*;
use crate::{Auth, authenticate, query};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UiEnvelope {
    auth: Auth,
    project: String,
    native_ui: UiRequest,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UiRequest {
    method: String,
    params: Value,
}
fn clean(v: &mut Value) {
    match v {
        Value::Object(o) => {
            for k in ["authBearerToken", "token", "capability"] {
                o.remove(k);
            }
            for v in o.values_mut() {
                clean(v);
            }
        }
        Value::Array(a) => {
            for v in a {
                clean(v)
            }
        }
        _ => {}
    }
}
impl CodexRuntime {
    pub fn ui_request(&self, controller: &Controller, request: Value) -> Result<Value, String> {
        let _lifecycle = self
            .lifecycle
            .lock()
            .map_err(|_| "native lifecycle gate poisoned")?;
        let r: UiEnvelope =
            serde_json::from_value(request).map_err(|e| format!("native UI envelope: {e}"))?;
        let (principal, binding, document, epoch) = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let p = authenticate(&c, &r.auth, &r.project)?;
            if !matches!(p.role.as_str(), "operator" | "lead" | "attempt") {
                return Err("native UI role refused".into());
            }
            let row: (String, String, i64) = if p.role == "attempt" {
                db(c.query_row("SELECT id,document,authority_epoch FROM codex_bindings WHERE project=?1 AND attempt=?2 AND authority_epoch=?3 AND state='bound'",params![r.project,p.attempt,p.epoch],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?
            } else {
                db(c.query_row("SELECT b.id,b.document,b.authority_epoch FROM codex_bindings b JOIN projects p ON p.id=b.project WHERE b.project=?1 AND b.role='lead' AND b.authority_epoch=p.lead_epoch AND b.state='bound'",[&r.project],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?
            };
            (
                p,
                row.0,
                serde_json::from_str::<Value>(&row.1).map_err(|e| e.to_string())?,
                row.2,
            )
        };
        if document["ui_input_ready"] != true || document["recovery_state"] != "remote_validated" {
            return Err("native attachment awaits validated remote environment restoration".into());
        }
        let method = r.native_ui.method.as_str();
        let input = &r.native_ui.params;
        if input.as_object().is_none() {
            return Err("native UI params object required".into());
        }
        let thread = text(&document, "thread_id")?;
        if input
            .get("threadId")
            .is_some_and(|id| !id.is_null() && id.as_str() != Some(thread))
            || input.get("history").is_some_and(|v| !v.is_null())
        {
            return Err("native UI exact thread/history scope refused".into());
        }
        if input.get("cursor").is_some_and(|c| !c.is_null()) {
            let cursor = input["cursor"]
                .as_str()
                .ok_or("native UI cursor type refused")?;
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let issued:bool=db(c.query_row("SELECT EXISTS(SELECT 1 FROM control_ui_cursors WHERE binding=?1 AND incarnation=?2 AND method=?3 AND cursor=?4)",params![binding,document["server_incarnation"].as_str(),method,cursor],|r|r.get(0)))?;
            if !issued {
                return Err("native UI unowned cursor refused".into());
            }
        }
        let allowed = matches!(
            method,
            "initialize"
                | "initialized"
                | "account/read"
                | "model/list"
                | "configRequirements/read"
                | "collaborationMode/list"
                | "hooks/list"
                | "skills/list"
                | "config/read"
                | "thread/list"
                | "thread/loaded/list"
                | "thread/read"
                | "thread/resume"
                | "thread/turns/list"
                | "thread/items/list"
                | "thread/goal/get"
                | "turn/start"
                | "turn/steer"
                | "turn/interrupt"
                | "corenius/events"
        );
        if !allowed {
            return Err("native UI method refused; helpers, raw tool responses and runtime mutation require controller authority".into());
        }
        let profile = self
            .config
            .servers
            .first()
            .ok_or("native profile unavailable")?;
        let app_binding: Binding =
            serde_json::from_value(document["server_execution"]["binding"].clone())
                .map_err(|e| e.to_string())?;
        let app = supervisor::native_lookup(
            &profile.supervisor_socket,
            &secret(&profile.auth_ref)?,
            &app_binding,
        )?;
        if app["runtime"]["store_id"] != document["server_store"]
            || app["runtime"]["incarnation"] != document["server_incarnation"]
        {
            return Err(
                "native UI incarnation/store changed; reconcile before attachment/input".into(),
            );
        }
        if method == "corenius/events" {
            let after = input["after"]
                .as_u64()
                .and_then(|n| i64::try_from(n).ok())
                .unwrap_or(0);
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let mut events = query::rows(
                &c,
                "SELECT seq,document FROM control_ui_events WHERE project=?1 AND binding=?2 AND seq>?3 ORDER BY seq LIMIT 20",
                &[&r.project, &binding, &after],
            )?;
            let mut bytes = 2;
            let mut retained = 0;
            for event in events.as_array_mut().unwrap() {
                let size = event["document"]
                    .as_str()
                    .ok_or("UI event text missing")?
                    .len()
                    + 128;
                if bytes + size > 524288 {
                    break;
                }
                event["document"] = serde_json::from_str(event["document"].as_str().unwrap())
                    .map_err(|e| e.to_string())?;
                bytes += size;
                retained += 1;
            }
            // The attachment continues from the last returned seq, leaving every
            // deferred event retained for the next poll instead of dropping it.
            events.as_array_mut().unwrap().truncate(retained);
            return Ok(events);
        }
        if matches!(method, "turn/start" | "turn/steer" | "turn/interrupt") {
            if principal.role == "attempt" {
                return Err(
                    "worker UI input requires an authorized lead controller command".into(),
                );
            }
            return self.ui_input(controller, &r, &binding, &document, epoch);
        }
        if method == "initialize" {
            return Ok(
                json!({"userAgent":"codex/0.159.3 corenius-scoped-ui","platformFamily":"unix","platformOs":"linux"}),
            );
        }
        if method == "initialized" {
            return Ok(json!({}));
        }
        let mut connections = self
            .connections
            .lock()
            .map_err(|_| "native connection lock poisoned")?;
        let connected = connections
            .get_mut(text(&document, "server_id")?)
            .ok_or("native UI controller connection unavailable")?;
        if connected.incarnation != document["server_incarnation"] {
            return Err("native UI stale controller connection".into());
        }
        // Check authority again after waiting for the effect transport.
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            authenticate(&c, &r.auth, &r.project)?;
        }
        let ws = &mut connected.ws;
        if input.get("path").is_some_and(|p| !p.is_null()) {
            let read = ws.rpc(
                "thread/read",
                json!({"threadId":thread,"includeTurns":false}),
            )?;
            if method != "thread/resume" || input["path"] != read["thread"]["path"] {
                return Err(
                    "native UI resume path does not match registered bound thread/store".into(),
                );
            }
        }
        let mut result=match method {
            "thread/loaded/list"=>json!({"data":[thread],"nextCursor":null}),
            "thread/list"=>{let read=ws.rpc("thread/read",json!({"threadId":thread,"includeTurns":false}))?;json!({"data":[read["thread"]],"nextCursor":null})},
            "thread/read"=>ws.rpc("thread/read",json!({"threadId":thread,"includeTurns":input["includeTurns"].as_bool().unwrap_or(true)}))?,
            "thread/resume"=>self.mutation(controller,&binding,&r.project,"thread/resume",&json!({"threadId":thread,"approvalPolicy":"never","sandbox":"danger-full-access","environments":[{"environmentId":document["environment_id"],"cwd":"/workspace","runtimeWorkspaceRoots":["/workspace"]}]}),ws)?,
            "thread/turns/list"|"thread/items/list"=>{
                let mut params=json!({"threadId":thread,"limit":input["limit"].as_u64().unwrap_or(100).clamp(1,100),"cursor":input["cursor"]});
                for key in ["itemsView","sortDirection"] {if !input[key].is_null(){params[key]=input[key].clone();}}
                if !input["turnId"].is_null(){let turn=text(input,"turnId")?;if !document["native_turns"].as_array().is_some_and(|turns|turns.iter().any(|t|t["id"]==turn)){return Err("native UI turn scope refused".into());}params["turnId"]=json!(turn);}
                ws.rpc(method,params)?
            },
            "thread/goal/get"=>ws.rpc(method,json!({"threadId":thread}))?,
            "account/read"=>json!({"account":null,"requiresOpenaiAuth":false}),
            "config/read"=>ws.rpc(method,json!({"cwd":"/workspace","includeLayers":false}))?,
            "skills/list"=>ws.rpc(method,json!({"cwds":["/workspace"],"forceReload":false}))?,
            "hooks/list"=>ws.rpc(method,json!({"cwd":"/workspace"}))?,
            "model/list"=>ws.rpc(method,json!({"limit":100,"cursor":null}))?,
            "configRequirements/read"|"collaborationMode/list"=>ws.rpc(method,json!({}))?,
            _=>return Err("native UI routing invariant".into()),
        };
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            for (key, endpoint) in [
                ("turnsBackwardsCursor", "thread/turns/list"),
                ("itemsBackwardsCursor", "thread/items/list"),
                ("nextCursor", method),
            ] {
                if let Some(cursor) = result[key].as_str() {
                    if cursor.len() > 4096 {
                        return Err("native UI cursor bound".into());
                    }
                    db(c.execute(
                        "INSERT OR IGNORE INTO control_ui_cursors VALUES(?1,?2,?3,?4)",
                        params![
                            binding,
                            document["server_incarnation"].as_str(),
                            endpoint,
                            cursor
                        ],
                    ))?;
                }
            }
        }
        clean(&mut result);
        if result.to_string().len() > 2_097_152 {
            return Err("native UI response bound".into());
        }
        Ok(result)
    }
    fn ui_input(
        &self,
        controller: &Controller,
        r: &UiEnvelope,
        binding: &str,
        document: &Value,
        epoch: i64,
    ) -> Result<Value, String> {
        let params = &r.native_ui.params;
        let serialized_params = params.to_string();
        let method = r.native_ui.method.as_str();
        let stable = if method == "turn/interrupt" {
            text(params, "turnId")?
        } else {
            params["clientUserMessageId"]
                .as_str()
                .or_else(|| params["operationId"].as_str())
                .filter(|s| !s.is_empty())
                .ok_or("native UI stable clientUserMessageId/operationId required")?
        };
        let key=crate::hash(&json!({"binding":binding,"epoch":epoch,"incarnation":document["server_incarnation"],"client":stable,"method":method}).to_string());
        let text = if method == "turn/interrupt" {
            None
        } else {
            let input = params["input"]
                .as_array()
                .ok_or("literal native UI input required")?;
            if input.is_empty() || input.len() > 20 || input.iter().any(|p| p["type"] != "text") {
                return Err("native UI only literal text input is admitted".into());
            }
            let text = input
                .iter()
                .map(|p| p["text"].as_str().ok_or("native UI text missing"))
                .collect::<Result<Vec<_>, _>>()?
                .join("\n");
            if text.is_empty() || text.len() > 65536 {
                return Err("native UI input bound".into());
            }
            Some(text)
        };
        let active = document["current_turn"]["state"] == "active";
        let kind = if method == "turn/interrupt" {
            "codex_interrupt"
        } else if method == "turn/steer" || active {
            "codex_steer"
        } else {
            "codex_turn"
        };
        let prior = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            db(c.query_row(
                "SELECT parameters,request,receipt FROM control_ui_commands WHERE identity=?1",
                [&key],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional())?
        };
        let unadmitted_retry = prior
            .as_ref()
            .is_some_and(|(_, _, receipt)| receipt.is_none());
        let request = if let Some((parameters, request, _)) = prior {
            if parameters != serialized_params {
                return Err("native UI stable input arguments changed".into());
            }
            serde_json::from_str(&request).map_err(|e| e.to_string())?
        } else {
            if method == "turn/steer" && params["expectedTurnId"] != document["turn_id"] {
                return Err("native UI expected turn mismatch".into());
            }
            if method == "turn/interrupt" && (params["turnId"] != document["turn_id"] || !active) {
                return Err("native UI interrupt current turn mismatch".into());
            }
            let revision = {
                let c = controller
                    .connection
                    .lock()
                    .map_err(|_| "controller lock poisoned")?;
                db(c.query_row(
                    "SELECT revision FROM projects WHERE id=?1",
                    [&r.project],
                    |r| r.get::<_, i64>(0),
                ))?
            };
            let mut args = json!({"role":"lead","text":text,"expected_binding":{"thread_id":document["thread_id"],"server_store":document["server_store"],"server_incarnation":document["server_incarnation"],"environment_id":document["environment_id"]}});
            if !params["model"].is_null() {
                if kind == "codex_turn" {
                    args["model"] = params["model"].clone();
                } else if params["model"].as_str()
                    != Some(selected_model(
                        document["model"].as_str().or(self.config.model.as_deref()),
                    )?)
                {
                    return Err("native UI active turn model cannot change".into());
                }
            }
            if kind != "codex_turn" {
                args["expected_turn"] = document["turn_id"].clone();
            }
            let req = json!({"auth":{"principal":r.auth.principal,"token":r.auth.token},"project":r.project,"command":{"kind":kind,"idempotency_key":key,"expected_revision":revision,"args":args}});
            let mut safe = req.clone();
            safe.as_object_mut().unwrap().remove("auth");
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            db(c.execute(
                "INSERT INTO control_ui_commands VALUES(?1,?2,?3,?4,?5,NULL)",
                params![
                    key,
                    r.project,
                    binding,
                    params.to_string(),
                    safe.to_string()
                ],
            ))?;
            safe
        };
        #[cfg(feature = "fault-injection")]
        if let Ok(path) = std::env::var("CORENIUS_TEST_UI_ADMISSION_BARRIER")
            && std::path::Path::new(&path).is_file()
        {
            std::fs::write(format!("{path}.observed"), &key).map_err(|e| e.to_string())?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while std::path::Path::new(&path).is_file() {
                if std::time::Instant::now() > deadline {
                    return Err("declared UI admission barrier deadline".into());
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        let mut request = request;
        // The saved command owns retry semantics even after runtime state changes.
        let saved_kind = crate::text(&request["command"], "kind")?.to_owned();
        request["auth"] = json!({"principal":r.auth.principal,"token":r.auth.token});
        let receipt = if unadmitted_retry {
            let mut c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let actor = authenticate(&c, &r.auth, &r.project)?;
            // A core receipt and its native operation commit in the same transaction.
            // Only an intent with neither saved nor core receipt can refresh its revision.
            let admitted: bool = db(c.query_row(
                "SELECT EXISTS(SELECT 1 FROM receipts WHERE project=?1 AND key=?2)",
                params![r.project, key],
                |row| row.get(0),
            ))?;
            if !admitted {
                request["command"]["expected_revision"] = json!(db(c.query_row(
                    "SELECT revision FROM projects WHERE id=?1",
                    [&r.project],
                    |row| row.get::<_, i64>(0)
                ))?);
                let mut safe = request.clone();
                safe.as_object_mut().unwrap().remove("auth");
                db(c.execute("UPDATE control_ui_commands SET request=?2 WHERE identity=?1 AND receipt IS NULL",params![key,safe.to_string()]))?;
            }
            let command =
                serde_json::from_value(request["command"].clone()).map_err(|e| e.to_string())?;
            // Hold the writer through revision selection and command admission. No runtime I/O.
            controller.command(&mut c, &actor, &r.project, command, None, None)?
        } else {
            controller.handle(request)?
        };
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            db(c.execute(
                "UPDATE control_ui_commands SET receipt=?2 WHERE identity=?1",
                params![key, receipt.to_string()],
            ))?;
        }
        self.drain_inner(controller)?;
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let (state, raw): (String, Option<String>) = db(c.query_row(
            "SELECT state,observation FROM codex_operations WHERE id=?1",
            [receipt["result"]["operation"]
                .as_str()
                .ok_or("UI operation missing")?],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ))?;
        if state != "confirmed" {
            return Err(format!(
                "native UI operation {} is {state}; retain same stable input and reconcile",
                receipt["result"]["operation"]
            ));
        }
        let observed: Value =
            serde_json::from_str(raw.as_deref().ok_or("UI observed result missing")?)
                .map_err(|e| e.to_string())?;
        if method == "turn/start"
            && saved_kind == "codex_turn"
            && observed["reconciled_from_native_history"] == true
        {
            let response = &observed["native_start_response"];
            if response["turn"]["id"] != observed["turn_id"]
                || !response["turn"]["id"].is_string()
                || observed["thread_id"] != document["thread_id"]
                || observed["server_store"] != document["server_store"]
                || observed["server_incarnation"] != document["server_incarnation"]
            {
                return Err("native UI reconciled start response provenance unavailable".into());
            }
            Ok(response.clone())
        } else if method == "turn/start" && saved_kind == "codex_steer" {
            Ok(
                json!({"turn":{"id":observed["turnId"],"status":"inProgress","items":[],"error":null}}),
            )
        } else {
            Ok(observed)
        }
    }
}

#[cfg(test)]
mod selected_model_ui_tests {
    use super::super::selected_model_tests::{fixture, native_peer};
    use super::*;
    #[test]
    fn selected_model_native_ui_records_identity_and_reaches_actual_adapter() {
        let (_root, c, lead, _, mut doc, binding, runtime) = fixture();
        let request = json!({"auth":{"principal":lead["principal"],"token":lead["token"]},"project":"p","native_ui":{"method":"turn/start","params":{"threadId":"lead-thread","clientUserMessageId":"selected-model-ui","model":"gpt-6.1-sol","input":[{"type":"text","text":"selected model","text_elements":[]}]}}});
        let error = runtime
            .ui_input(
                &c,
                &serde_json::from_value(request.clone()).unwrap(),
                "lead-binding",
                &doc,
                1,
            )
            .unwrap_err();
        assert!(error.contains("is prepared"), "{error}");
        let intent: Value = {
            let db = c.connection.lock().unwrap();
            serde_json::from_str(
                &db.query_row(
                    "SELECT intent FROM codex_operations WHERE kind='codex_turn'",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .unwrap(),
            )
            .unwrap()
        };
        assert_eq!(intent["model"], "gpt-6.1-sol");
        let mut changed = request.clone();
        changed["native_ui"]["params"]["model"] = json!("gpt-6-sol");
        assert!(
            runtime
                .ui_input(
                    &c,
                    &serde_json::from_value(changed).unwrap(),
                    "lead-binding",
                    &doc,
                    1
                )
                .unwrap_err()
                .contains("stable input arguments changed")
        );
        let (mut ws, peer) = NativeWs::owned_test_pair();
        ws.allow_thread("lead-thread").unwrap();
        let capture = native_peer(peer);
        runtime
            .dispatch_prepared(&c, "lead-binding", &mut doc, &mut ws, &binding)
            .unwrap();
        let sent = capture.join().unwrap();
        assert_eq!(sent["method"], "turn/start");
        assert_eq!(sent["params"]["model"], "gpt-6.1-sol");
        assert_eq!(
            sent["params"]["clientUserMessageId"]
                .as_str()
                .unwrap()
                .len(),
            36
        );
        assert_eq!(doc["model"], "gpt-6.1-sol");
        doc["current_turn"]["state"] = json!("active");
        let mut changed = request.clone();
        changed["native_ui"]["params"]["clientUserMessageId"] = json!("changed-active-model");
        changed["native_ui"]["params"]["model"] = json!("gpt-6-sol");
        assert!(
            runtime
                .ui_input(
                    &c,
                    &serde_json::from_value(changed).unwrap(),
                    "lead-binding",
                    &doc,
                    1
                )
                .unwrap_err()
                .contains("active turn model")
        );
    }
}
