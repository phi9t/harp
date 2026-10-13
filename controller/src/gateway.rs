//! Structured tools inject the bound sender. Native bearer credentials never cross this seam.
use crate::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    auth: Auth,
    project: String,
    gateway: Invocation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Invocation {
    tool: String,
    arguments: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolQuery {
    kind: String,
    #[serde(default)]
    args: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolCommand {
    kind: String,
    operation_id: String,
    expected_revision: u64,
    args: Value,
}

pub fn specifications() -> Value {
    json!([
    {"type":"function","name":"corenius_query","description":"Fetch durable controller state for your authenticated role. Workers see their task. question_input with args.question retrieves exact question text/digest/bytes for delivery references. control_records args.collection/after/limit follows retained history pages; control_input follows exact text_reference with UTF8 byte offset; event_page provides reconnect cursors. workspace_inventory exposes bounded physical coverage/history continuations; runtime_public_input retrieves exact retained typed-visible input references. Observers may fetch bounded monitor_context, monitor_batch, monitor_input, monitor_assessment, monitor_records using args. Activity is separate from verified evidence.","inputSchema":{"type":"object","properties":{"kind":{"type":"string","enum":["control_center","lead_context","own_assignment","questions","question_input","control_records","control_input","event_page","workspace_inventory","runtime_public_input","corrections","monitor_context","monitor_batch","monitor_input","monitor_assessment","monitor_records"]},"args":{"type":"object"}},"required":["kind"],"additionalProperties":false}},
        {"type":"function","name":"corenius_command","description":"Issue an authorized controller command. Keep operation_id AND expected_revision AND arguments unchanged on lost-response retry. Do not infer success from a runtime transcript.","inputSchema":{"type":"object","properties":{"kind":{"type":"string"},"operation_id":{"type":"string"},"expected_revision":{"type":"integer"},"args":{"type":"object"}},"required":["kind","operation_id","expected_revision","args"],"additionalProperties":false}}
    ])
}

pub(super) fn dispatch(controller: &Controller, request: Value) -> Result<Value> {
    let request: Envelope =
        serde_json::from_value(request).map_err(|e| format!("gateway request: {e}"))?;
    let (task, role) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &request.auth, &request.project)?;
        (p.task, p.role)
    };
    match request.gateway.tool.as_str() {
        "corenius_query" => {
            let q: ToolQuery = serde_json::from_value(request.gateway.arguments)
                .map_err(|e| format!("gateway query: {e}"))?;
            if !matches!(
                q.kind.as_str(),
                "control_center"
                    | "lead_context"
                    | "own_assignment"
                    | "questions"
                    | "question_input"
                    | "control_records"
                    | "control_input"
                    | "event_page"
                    | "workspace_inventory"
                    | "runtime_public_input"
                    | "corrections"
                    | "monitor_context"
                    | "monitor_batch"
                    | "monitor_input"
                    | "monitor_assessment"
                    | "monitor_records"
            ) {
                return Err("gateway query scope refused".into());
            }
            if q.kind.starts_with("monitor_")
                || matches!(
                    q.kind.as_str(),
                    "question_input"
                        | "control_records"
                        | "control_input"
                        | "event_page"
                        | "workspace_inventory"
                        | "runtime_public_input"
                )
            {
                return controller.handle(json!({"auth":{"principal":request.auth.principal,"token":request.auth.token},"project":request.project,"query":{"kind":q.kind,"args":if q.args.is_null(){json!({})}else{q.args}}}));
            }
            let result=controller.handle(json!({"auth":{"principal":request.auth.principal,"token":request.auth.token},"project":request.project,"query":{"kind":"control_center","args":if q.args.is_null(){json!({})}else{q.args}}}))?;
            Ok(match q.kind.as_str() {
                "questions" => result["questions"].clone(),
                "corrections" => result["monitoring"]["corrections"].clone(),
                _ => result,
            })
        }
        "corenius_command" => {
            let mut command: ToolCommand = serde_json::from_value(request.gateway.arguments)
                .map_err(|e| format!("gateway command: {e}"))?;
            if command.operation_id.is_empty() {
                return Err("stable operation_id required; do not generate on retry".into());
            }
            if role == "observer" && command.kind != "publish_monitor_assessment" {
                return Err("gateway observer advisory command only".into());
            }
            if command.kind == "codex_turn" && role != "lead" {
                return Err("gateway native turn requires lead authority".into());
            }
            if !matches!(
                command.kind.as_str(),
                "set_mission"
                    | "replace_lead"
                    | "publish_monitor_assessment"
                    | "create_kata_task"
                    | "admit"
                    | "pause_dispatch"
                    | "cancel_attempt"
                    | "answer_question"
                    | "ack_question"
                    | "revise_brief"
                    | "approve_brief"
                    | "request_repair"
                    | "accept_candidate"
                    | "verify_candidate"
                    | "land_candidate"
                    | "record_decision"
                    | "send_message"
                    | "ack_message"
                    | "ask_question"
                    | "report_progress"
                    | "propose_followup"
                    | "capture_candidate"
                    | "ack_correction"
                    | "adjudicate_monitor_finding"
                    | "authorize_correction"
                    | "resolve_correction"
                    | "codex_turn"
            ) {
                return Err("gateway command scope refused".into());
            }
            if role == "attempt"
                && matches!(
                    command.kind.as_str(),
                    "ask_question" | "report_progress" | "propose_followup"
                )
            {
                if let Some(supplied) = command.args.get("task")
                    && supplied.as_str() != task.as_deref()
                {
                    return Err("gateway own task only".into());
                }
                command
                    .args
                    .as_object_mut()
                    .ok_or("gateway args object")?
                    .insert("task".into(), json!(task));
            }
            for selector in [
                "principal",
                "auth",
                "project",
                "database",
                "server",
                "runtime",
                "cwd",
                "environment",
                "canonical_ref",
                "token",
                "sender",
            ] {
                if command.args.get(selector).is_some() {
                    return Err("gateway identity/environment selector refused".into());
                }
            }
            controller.handle(json!({"auth":{"principal":request.auth.principal,"token":request.auth.token},"project":request.project,"command":{"kind":command.kind,"idempotency_key":command.operation_id,"expected_revision":command.expected_revision,"args":command.args}}))
        }
        _ => Err("unknown scoped tool".into()),
    }
}

pub(super) fn native_call(
    controller: &Controller,
    binding: &str,
    document: &Value,
    call: &Value,
    observed: Option<&crate::native_ws::NativeTurn>,
) -> Result<Value> {
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let (project, role, attempt, epoch, state): (String, String, Option<String>, i64, String) =
        db(c.query_row(
            "SELECT project,role,attempt,authority_epoch,state FROM codex_bindings WHERE id=?1",
            [binding],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        ))?;
    if matches!(state.as_str(), "revoked" | "revoking" | "unknown")
        || call["method"] != "item/tool/call"
        || call["params"]["threadId"] != document["thread_id"]
        || call["params"]["turnId"].as_str().is_none()
    {
        return Err("native tool current binding refused".into());
    }
    let principal: String = if role == "lead" {
        db(c.query_row("SELECT p.id FROM principals p WHERE p.project=?1 AND p.role='lead' AND p.epoch=?2 AND NOT EXISTS(SELECT 1 FROM monitor_observers m WHERE m.principal=p.id)",params![project,epoch],|r|r.get(0)))?
    } else {
        format!("attempt:{}", attempt.as_deref().ok_or("attempt binding")?)
    };
    let current: i64 = if role == "lead" {
        db(c.query_row(
            "SELECT lead_epoch FROM projects WHERE id=?1",
            [&project],
            |r| r.get(0),
        ))?
    } else {
        db(c.query_row(
            "SELECT scope_epoch FROM attempts WHERE id=?1",
            [attempt.as_deref().unwrap()],
            |r| r.get(0),
        ))?
    };
    if current != epoch {
        return Err("native tool superseded authority".into());
    }
    let mut response: Option<String> = None;
    let mut receipts =
        db(c.prepare("SELECT response FROM receipts WHERE project=?1 ORDER BY rowid DESC"))?;
    for raw in db(receipts.query_map([&project], |r| r.get::<_, String>(0)))? {
        let raw = db(raw)?;
        let receipt: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        if receipt["result"]["principal"].as_str() == Some(&principal) {
            response = receipt["result"]["token"].as_str().map(str::to_owned);
            break;
        }
    }
    drop(receipts);
    let token = response.ok_or("native scoped credential receipt missing")?;
    let identity=hash(&json!({"binding":binding,"epoch":epoch,"server":document["server_id"],"incarnation":document["server_incarnation"],"store":document["server_store"],"thread":call["params"]["threadId"],"turn":call["params"]["turnId"],"call":call["params"]["callId"],"request":call["id"]}).to_string());
    let invocation = json!({"tool":call["params"]["tool"],"arguments":call["params"]["arguments"]});
    let invocation_raw = invocation.to_string();
    let prior: Option<(String, Option<String>)> = db(c
        .query_row(
            "SELECT arguments,response FROM control_tool_calls WHERE identity=?1",
            [&identity],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional())?;
    if let Some((args, response)) = prior {
        if args != invocation_raw {
            return Err("native replay argument mismatch".into());
        }
        if let Some(response) = response {
            return serde_json::from_str(&response).map_err(|e| e.to_string());
        }
    }
    let observed = observed.ok_or("native tool active turn observation missing")?;
    if !observed.active
        || call["params"]["threadId"] != observed.thread
        || call["params"]["turnId"] != observed.turn
    {
        return Err("native tool stale or unrelated turn refused".into());
    }
    let in_flight = if let Some(client) = &observed.client_user_message_id {
        db(c.query_row("SELECT EXISTS(SELECT 1 FROM codex_operations WHERE binding=?1 AND state IN ('applying','unknown') AND ((kind='codex_turn' AND id=?2) OR (kind='turn/start' AND json_extract(intent,'$.clientUserMessageId')=?2)))",params![binding,client],|r|r.get::<_,bool>(0)))?
    } else {
        false
    };
    if document["turn_id"] != observed.turn && !in_flight {
        return Err("native tool turn lacks current or admitted in-flight correlation".into());
    }
    db(c.execute(
        "INSERT OR IGNORE INTO control_tool_calls VALUES(?1,?2,?3,?4,?5,?6,NULL)",
        params![
            identity,
            project,
            binding,
            epoch,
            call["params"]["tool"].as_str(),
            invocation.to_string()
        ],
    ))?;
    drop(c);
    let result = dispatch(
        controller,
        json!({"auth":{"principal":principal,"token":token},"project":project,"gateway":invocation}),
    );
    let reply = match result {
        Ok(value) => {
            json!({"contentItems":[{"type":"inputText","text":value.to_string()}],"success":true})
        }
        Err(error) => json!({"contentItems":[{"type":"inputText","text":error}],"success":false}),
    };
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    db(c.execute(
        "UPDATE control_tool_calls SET response=?2 WHERE identity=?1",
        params![identity, reply.to_string()],
    ))?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct NativeFixture {
        _dir: tempfile::TempDir,
        controller: Controller,
        document: Value,
    }
    impl NativeFixture {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let controller =
                Controller::open(dir.path().join("state.db"), "operator-secret").unwrap();
            controller.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"create_project","idempotency_key":"project","expected_revision":0,"args":{"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}}})).unwrap();
            controller.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"replace_lead","idempotency_key":"lead","expected_revision":1,"args":{}}})).unwrap();
            let document = json!({"thread_id":"owned-thread","server_id":"owned-server","server_incarnation":"owned-incarnation","server_store":"owned-store","turn_id":"current-turn","current_turn":{"id":"current-turn","state":"active","native_status":"inProgress"}});
            {
                let c = controller.connection.lock().unwrap();
                c.execute_batch(include_str!("codex_schema.sql")).unwrap();
                c.execute(
                    "INSERT INTO codex_bindings VALUES('lead:p:1','p','lead',NULL,1,?1,'bound')",
                    [document.to_string()],
                )
                .unwrap();
            }
            Self {
                _dir: dir,
                controller,
                document,
            }
        }
        fn call(&self, turn: &str, operation: &str, objective: &str) -> Value {
            json!({"id":17,"method":"item/tool/call","params":{"threadId":"owned-thread","turnId":turn,"callId":"owned-call","tool":"corenius_command","arguments":{"kind":"set_mission","operation_id":operation,"expected_revision":2,"args":{"objective":objective,"plan":"bounded","constraints":"same authority","intent":"source-pinned"}}}})
        }
        fn observed(&self) -> crate::native_ws::NativeTurn {
            crate::native_ws::NativeTurn {
                thread: "owned-thread".into(),
                turn: "current-turn".into(),
                active: true,
                client_user_message_id: None,
            }
        }
        fn effects(&self) -> Value {
            let c = self.controller.connection.lock().unwrap();
            json!({"revision":c.query_row("SELECT revision FROM projects WHERE id='p'",[],|r|r.get::<_,i64>(0)).unwrap(),"events":c.query_row("SELECT COUNT(*) FROM events",[],|r|r.get::<_,i64>(0)).unwrap(),"receipts":c.query_row("SELECT COUNT(*) FROM receipts",[],|r|r.get::<_,i64>(0)).unwrap(),"tool_calls":c.query_row("SELECT COUNT(*) FROM control_tool_calls",[],|r|r.get::<_,i64>(0)).unwrap()})
        }
        fn pending(&self, call: &Value) -> (String, String) {
            let identity=hash(&json!({"binding":"lead:p:1","epoch":1,"server":self.document["server_id"],"incarnation":self.document["server_incarnation"],"store":self.document["server_store"],"thread":call["params"]["threadId"],"turn":call["params"]["turnId"],"call":call["params"]["callId"],"request":call["id"]}).to_string());
            let arguments =
                json!({"tool":call["params"]["tool"],"arguments":call["params"]["arguments"]})
                    .to_string();
            self.controller.connection.lock().unwrap().execute("INSERT INTO control_tool_calls VALUES(?1,'p','lead:p:1',1,'corenius_command',?2,NULL)",params![identity,arguments]).unwrap();
            (identity, arguments)
        }
    }
    // Declared controller/transport fixtures; actual native qualification remains separate.
    #[test]
    fn pending_native_identity_rejects_changed_arguments_without_effect() {
        let f = NativeFixture::new();
        let original = f.call("current-turn", "original-operation", "original objective");
        let (identity, arguments) = f.pending(&original);
        let before = f.effects();
        let changed = f.call("current-turn", "changed-operation", "changed objective");
        let result = native_call(
            &f.controller,
            "lead:p:1",
            &f.document,
            &changed,
            Some(&f.observed()),
        );
        assert!(
            result.is_err(),
            "pending identity must compare arguments before dispatch: {result:?}"
        );
        assert_eq!(before, f.effects());
        let row: (String, Option<String>) = f
            .controller
            .connection
            .lock()
            .unwrap()
            .query_row(
                "SELECT arguments,response FROM control_tool_calls WHERE identity=?1",
                [identity],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(row, (arguments, None));
    }
    #[test]
    fn stale_unrelated_native_turn_cannot_create_effect_or_call_journal() {
        let f = NativeFixture::new();
        let before = f.effects();
        let result = native_call(
            &f.controller,
            "lead:p:1",
            &f.document,
            &f.call("unrelated-completed-turn", "stale", "forbidden objective"),
            Some(&f.observed()),
        );
        assert!(
            result.is_err(),
            "unrelated turn must not admit a new command: {result:?}"
        );
        assert_eq!(before, f.effects());
    }
    #[test]
    fn same_pending_payload_recovers_and_completed_turn_replays_exact_reply_once() {
        let f = NativeFixture::new();
        let original = f.call("current-turn", "original-operation", "original objective");
        f.pending(&original);
        let reply = native_call(
            &f.controller,
            "lead:p:1",
            &f.document,
            &original,
            Some(&f.observed()),
        )
        .unwrap();
        assert_eq!(reply["success"], true);
        let before = f.effects();
        let mut completed = f.document.clone();
        completed["current_turn"]["state"] = json!("terminal");
        completed["current_turn"]["native_status"] = json!("completed");
        assert_eq!(
            reply,
            native_call(&f.controller, "lead:p:1", &completed, &original, None).unwrap()
        );
        assert_eq!(before, f.effects());
    }
    #[test]
    fn admitted_in_flight_turn_can_admit_early_tool_while_completed_turn_cannot() {
        let f = NativeFixture::new();
        let mut origin = f.observed();
        origin.turn = "new-turn".into();
        origin.client_user_message_id = Some("admitted-turn-operation".into());
        let before = f.effects();
        let call = f.call("new-turn", "early-mission", "correlated objective");
        assert!(native_call(&f.controller, "lead:p:1", &f.document, &call, Some(&origin)).is_err());
        assert_eq!(before, f.effects());
        f.controller.connection.lock().unwrap().execute("INSERT INTO codex_operations VALUES('admitted-turn-operation','p','lead:p:1','codex_turn','applying','{}',NULL)",[]).unwrap();
        assert_eq!(
            native_call(&f.controller, "lead:p:1", &f.document, &call, Some(&origin)).unwrap()["success"],
            true
        );
        let before = f.effects();
        let mut completed = f.observed();
        completed.active = false;
        let mut new_call = f.call("current-turn", "late-mission", "forbidden after completion");
        new_call["id"] = json!(18);
        assert!(
            native_call(
                &f.controller,
                "lead:p:1",
                &f.document,
                &new_call,
                Some(&completed)
            )
            .is_err()
        );
        assert_eq!(before, f.effects());
    }
}
