use corenius_sqlite_controller::Controller;
use serde_json::{Value, json};
use tempfile::TempDir;

struct F {
    dir: TempDir,
    c: Controller,
}
impl F {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let c = Controller::open(dir.path().join("state.db"), "operator-secret").unwrap();
        let f = Self { dir, c };
        f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"create_project","idempotency_key":"project","expected_revision":0,"args":{"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}}})).unwrap();
        f
    }
    fn query(&self, kind: &str) -> Value {
        self.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":kind,"args":{}}})).unwrap()
    }
    fn cmd(&self, kind: &str, key: &str, args: Value) -> Value {
        self.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":kind,"idempotency_key":key,"expected_revision":self.query("snapshot")["revision"],"args":args}})).unwrap()
    }
    fn task(&self, name: &str) {
        self.cmd("create_task", name, json!({"task":name,"brief":"original immutable intent","work_ref":{"deployment":"local","project":"p","item":name},"source_revision":"r1"}));
    }
}

#[test]
fn repair2_runtime_context_refuses_missing_ambiguous_and_foreign_journals() {
    for variant in ["missing", "duplicate-start", "duplicate-cancel", "foreign"] {
        let f = F::new();
        f.task("a");
        let grant = f.cmd("admit", "admit", json!({"task":"a"}));
        let id = grant["result"]["attempt"].as_str().unwrap();
        let start = grant["result"]["operation"].as_str().unwrap();
        let lead = f.cmd("replace_lead", "lead", json!({}));
        let current = f.query("runtime_context");
        assert_eq!(current["attempts"].as_array().unwrap().len(), 1);
        assert_eq!(current["operations"][0]["id"], start);
        for actor in [&grant["result"], &lead["result"]] {
            assert!(f.c.handle(json!({"auth":{"principal":actor["principal"],"token":actor["token"]},"project":"p","query":{"kind":"runtime_context","args":{}}})).is_err());
        }
        let db = rusqlite::Connection::open(f.dir.path().join("state.db")).unwrap();
        match variant {
            "missing" => {
                db.execute("DELETE FROM operations WHERE id=?1", [start])
                    .unwrap();
            }
            "duplicate-start" => {
                db.execute("INSERT INTO operations SELECT 'duplicate',project,attempt,kind,state,payload,observation FROM operations WHERE id=?1",[start]).unwrap();
            }
            "duplicate-cancel" => {
                f.cmd("cancel_attempt", "cancel", json!({"attempt":id}));
                db.execute("INSERT INTO operations SELECT 'duplicate',project,attempt,kind,state,payload,observation FROM operations WHERE attempt=?1 AND kind='cancel'",[id]).unwrap();
            }
            "foreign" => {
                db.execute("INSERT INTO projects VALUES('foreign','base',1,0,0)", [])
                    .unwrap();
                db.execute("INSERT INTO operations SELECT 'foreign-op','foreign',attempt,kind,state,payload,observation FROM operations WHERE id=?1",[start]).unwrap();
            }
            _ => unreachable!(),
        }
        let before: (i64,i64,i64) = db.query_row("SELECT (SELECT COUNT(*) FROM operations),(SELECT COUNT(*) FROM receipts),(SELECT COUNT(*) FROM events)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        assert!(f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"runtime_context","args":{}}})).is_err(),"{variant}");
        let after = db.query_row("SELECT (SELECT COUNT(*) FROM operations),(SELECT COUNT(*) FROM receipts),(SELECT COUNT(*) FROM events)",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?))).unwrap();
        assert_eq!(before, after);
        println!(
            "actual_runtime_context_{variant}_refused_without_journal_receipt_event_effect=true"
        );
    }
}

#[test]
fn shared_control_projection_preserves_receipts_intent_and_replacement_context() {
    let f = F::new();
    f.cmd("set_mission", "mission", json!({"objective":"Ship two bounded changes","plan":"a then b","constraints":"never broaden acceptance","intent":"pinned-spec-sha256"}));
    for t in ["a", "b"] {
        f.task(t);
        f.cmd("admit", &format!("admit-{t}"), json!({"task":t}));
    }
    let revision = f.query("snapshot")["revision"].clone();
    let request = json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"pause_dispatch","idempotency_key":"pause","expected_revision":revision,"args":{"paused":true}}});
    let receipt = f.c.handle(request.clone()).unwrap();
    assert_eq!(receipt, f.c.handle(request).unwrap());
    let proposal = f.cmd("revise_brief", "proposal", json!({"task":"a","body":"proposed change","source_revision":"r2","rationale":"source amendment"}));
    let center = f.query("control_center");
    assert_eq!(center["mission"]["objective"], "Ship two bounded changes");
    assert_eq!(center["workers"].as_array().unwrap().len(), 2);
    assert_eq!(center["tasks"][0]["body"], "original immutable intent");
    assert_eq!(
        center["brief_proposals"][0]["id"],
        proposal["result"]["proposal"]
    );
    assert_eq!(center["occupied_slots"], 2);
    let lead = f.cmd("replace_lead", "lead", json!({}));
    let reconstructed = f.c.handle(json!({"auth":{"principal":lead["result"]["principal"],"token":lead["result"]["token"]},"project":"p","query":{"kind":"control_center","args":{}}})).unwrap();
    assert_eq!(reconstructed["mission"], center["mission"]);
    assert_eq!(reconstructed["occupied_slots"], 2);
    assert!(reconstructed["next_decisions"].is_array());
    assert!(reconstructed["unknown_effects"].is_array());
    let old = reconstructed["lead_epoch"].clone();
    f.cmd("replace_lead", "replacement", json!({}));
    assert!(f.c.handle(json!({"auth":{"principal":lead["result"]["principal"],"token":lead["result"]["token"]},"project":"p","query":{"kind":"control_center","args":{}}})).is_err());
    assert!(f.query("control_center")["lead_epoch"].as_i64().unwrap() > old.as_i64().unwrap());
}

#[test]
fn question_transport_is_separate_from_explicit_ack_and_answer() {
    let f = F::new();
    f.task("a");
    let worker = f.cmd("admit", "admit", json!({"task":"a"}));
    let lead = f.cmd("replace_lead", "lead", json!({}));
    let auth = json!({"principal":worker["result"]["principal"],"token":worker["result"]["token"]});
    let q = f.c.handle(json!({"auth":auth,"project":"p","command":{"kind":"ask_question","idempotency_key":"question","expected_revision":f.query("snapshot")["revision"],"args":{"task":"a","text":"What exact criterion?"}}})).unwrap();
    let center = f.query("control_center");
    assert_eq!(center["questions"][0]["delivery_state"], "persisted");
    assert_eq!(
        center["questions"][0]["attempt"],
        worker["result"]["attempt"]
    );
    let l_auth = json!({"principal":lead["result"]["principal"],"token":lead["result"]["token"]});
    let early = f.c.handle(json!({"auth":l_auth,"project":"p","command":{"kind":"ack_question","idempotency_key":"early","expected_revision":f.query("snapshot")["revision"],"args":{"question":q["result"]["question"],"delivery":"invented"}}}));
    assert!(
        early.is_err(),
        "ack must cite a prepared delivery, not arbitrary text"
    );
}

#[test]
fn incremental_monitor_seals_retained_input_and_keeps_live_and_history_distinct() {
    let f = F::new();
    f.task("a");
    let worker = f.cmd("admit", "admit", json!({"task":"a"}));
    let stream = f.cmd("register_monitor_stream", "stream", json!({"attempt":worker["result"]["attempt"],"source":{"deployment":"runtime","store":"store","thread":"thread","segment":"seg1","parser":"visible-v1","identity":"immutable-fixture-export"},"live_baseline":1}));
    let sid = stream["result"]["stream"].clone();
    let events = json!([{"item":"one","turn":"turn-a","kind":"public_message","text":"hypothesis: immutable evidence","ordinal":1,"byte_end":70,"digest":"d1"},{"item":"two","turn":"turn-a","kind":"tool_completed","text":"actual result exit 1","ordinal":2,"byte_end":140,"digest":"d2"}]);
    f.cmd(
        "ingest_monitor_events",
        "ingest",
        json!({"stream":sid,"expected_ordinal":0,"previous_digest":"","events":events}),
    );
    let duplicate = f.cmd(
        "ingest_monitor_events",
        "duplicate",
        json!({"stream":sid,"expected_ordinal":2,"previous_digest":"d2","events":[]}),
    );
    assert_eq!(duplicate["result"]["ingested"], 2);
    let live = f.cmd(
        "seal_monitor_batch",
        "live",
        json!({"stream":sid,"lane":"live","through":2,"boundary":"handoff"}),
    );
    let batch = live["result"]["batch"].clone();
    let steps = json!([{"item":"two","criterion":"intent-1","rationale":"unknown","observation":"source-observed exit 1","assessment":"valid_learning","next_test":"change one input"}]);
    f.cmd("publish_monitor_assessment", "assessment", json!({"batch":batch,"analyst":"deterministic-fixture","version":"v1","steps":steps,"findings":[],"evidence":[]}));
    let monitoring = f.query("control_center")["monitoring"].clone();
    assert_eq!(monitoring["streams"][0]["reviewed_history"], 0);
    assert_eq!(monitoring["streams"][0]["reviewed_live"], 2);
    let retained=f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"monitor_batch","args":{"batch":batch}}})).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(retained["inputs"].as_str().unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(monitoring["streams"][0]["history_gap"].as_bool().unwrap());
    let original = monitoring["assessments"][0]["id"].clone();
    let rerun=f.cmd("publish_monitor_assessment","new-analyzer-version",json!({"batch":batch,"analyst":"deterministic-fixture","version":"v2","steps":[{"item":"two","criterion":"intent-1","rationale":"unknown","observation":"source-observed exit 1","assessment":"valid_learning","next_test":"change one input"}],"findings":[],"evidence":[]}));
    assert_ne!(rerun["result"]["assessment"], original);
    let monitoring = f.query("control_center")["monitoring"].clone();
    assert_eq!(monitoring["assessments"].as_array().unwrap().len(), 2);
    assert_eq!(monitoring["streams"][0]["reviewed_history"], 0);
    assert_eq!(monitoring["streams"][0]["reviewed_live"], 2);
    let original_report=f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"monitor_assessment","args":{"assessment":original}}})).unwrap();
    let replay_args: Value =
        serde_json::from_str(original_report["report"].as_str().unwrap()).unwrap();
    let replay = f.cmd(
        "publish_monitor_assessment",
        "same-version-replay",
        replay_args,
    );
    assert_eq!(replay["result"]["assessment"], original);
    assert_eq!(
        f.query("control_center")["monitoring"]["assessments"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    drop(f.c);
    let reopened = Controller::open(f.dir.path().join("state.db"), "operator-secret").unwrap();
    let projection = reopened.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"control_center","args":{}}})).unwrap();
    let mut persisted = projection["monitoring"].clone();
    let mut expected = monitoring;
    for view in [&mut persisted, &mut expected] {
        for stream in view["streams"].as_array_mut().unwrap() {
            stream.as_object_mut().unwrap().remove("age_seconds");
        }
    }
    assert_eq!(persisted, expected);
}

#[test]
fn gateway_lead_turn_admits_same_attempt_and_preserves_authority_and_start_fences() {
    // Dropping the gateway link prevents lead recovery; broadening it or
    // bypassing prepare would admit worker/operator or ambiguous native starts.
    let f = F::new();
    f.task("a");
    let worker = f.cmd("admit", "admit-a", json!({"task":"a"}));
    let lead = f.cmd("replace_lead", "lead", json!({}));
    let attempt = worker["result"]["attempt"].as_str().unwrap();
    let binding = format!("worker:{attempt}:1");
    let guard = json!({"thread_id":"retained-thread","server_store":"retained-store","server_incarnation":"retained-incarnation","environment_id":"retained-environment"});
    let db = rusqlite::Connection::open(f.dir.path().join("state.db")).unwrap();
    db.execute_batch(include_str!("../src/codex_schema.sql"))
        .unwrap();
    db.execute("UPDATE attempts SET state='running' WHERE id=?1", [attempt])
        .unwrap();
    db.execute(
        "INSERT INTO codex_bindings VALUES(?1,'p','attempt',?2,1,?3,'bound')",
        rusqlite::params![binding, attempt, guard.to_string()],
    )
    .unwrap();
    let invoke = |auth: Value, operation: &str, args: Value| {
        f.c.handle(json!({"auth":auth,"project":"p","gateway":{"tool":"corenius_command","arguments":{"kind":"codex_turn","operation_id":operation,"expected_revision":f.query("snapshot")["revision"],"args":args}}}))
    };
    let lead_auth =
        json!({"principal":lead["result"]["principal"],"token":lead["result"]["token"]});
    let args =
        json!({"attempt":attempt,"text":"Continue the same pinned task","expected_binding":guard});
    let before = f.query("snapshot");
    for auth in [
        json!({"principal":worker["result"]["principal"],"token":worker["result"]["token"]}),
        json!({"principal":"operator","token":"operator-secret"}),
    ] {
        assert!(invoke(auth, "forbidden", args.clone()).is_err());
    }
    let mut wrong = args.clone();
    wrong["expected_binding"]["server_incarnation"] = json!("foreign-incarnation");
    assert!(invoke(lead_auth.clone(), "wrong-binding", wrong).is_err());
    assert_eq!(before, f.query("snapshot"));
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM codex_operations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    db.execute("INSERT INTO codex_operations VALUES('unresolved-start','p',?1,'turn/start','unknown','{}',NULL)",[&binding]).unwrap();
    assert!(invoke(lead_auth.clone(), "unknown-start", args.clone()).is_err());
    assert_eq!(before, f.query("snapshot"));
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM codex_operations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    db.execute(
        "UPDATE codex_operations SET state='confirmed' WHERE id='unresolved-start'",
        [],
    )
    .unwrap();
    let admitted = invoke(lead_auth.clone(), "lead-recovery", args.clone()).unwrap();
    assert_eq!(admitted["result"]["binding"], binding);
    assert_eq!(admitted["result"]["state"], "prepared");
    let operation = admitted["result"]["operation"].as_str().unwrap();
    let row: (String, String, String) = db
        .query_row(
            "SELECT binding,kind,state FROM codex_operations WHERE id=?1",
            [operation],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(row, (binding, "codex_turn".into(), "prepared".into()));
    let revision = f.query("snapshot")["revision"].clone();
    let replay = f.c.handle(json!({"auth":lead_auth,"project":"p","gateway":{"tool":"corenius_command","arguments":{"kind":"codex_turn","operation_id":"lead-recovery","expected_revision":before["revision"],"args":args}}})).unwrap();
    assert_eq!(admitted, replay);
    assert_eq!(revision, f.query("snapshot")["revision"]);
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM codex_operations", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
}

#[test]
fn bound_tool_gateway_injects_identity_and_fences_raw_worker_escalation() {
    let f = F::new();
    f.task("a");
    f.task("b");
    let a = f.cmd("admit", "a-admit", json!({"task":"a"}));
    let b = f.cmd("admit", "b-admit", json!({"task":"b"}));
    let auth = json!({"principal":a["result"]["principal"],"token":a["result"]["token"]});
    let own=f.c.handle(json!({"auth":auth,"project":"p","gateway":{"tool":"corenius_query","arguments":{"kind":"control_center"}}})).unwrap();
    assert_eq!(own["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(own["tasks"][0]["id"], "a");
    let progress=f.c.handle(json!({"auth":auth,"project":"p","gateway":{"tool":"corenius_command","arguments":{"kind":"report_progress","operation_id":"own-progress","expected_revision":f.query("snapshot")["revision"],"args":{"text":"actual public checkpoint"}}}})).unwrap();
    assert!(progress["receipt"].is_string());
    for args in [
        json!({"kind":"accept_candidate","operation_id":"escalation","expected_revision":f.query("snapshot")["revision"],"args":{"candidate":"invented","verification":"invented"}}),
        json!({"kind":"report_progress","operation_id":"peer","expected_revision":f.query("snapshot")["revision"],"args":{"task":"b","text":"fake"}}),
        json!({"kind":"pause_dispatch","operation_id":"principal","principal":"operator","expected_revision":f.query("snapshot")["revision"],"args":{"paused":true}}),
    ] {
        assert!(f.c.handle(json!({"auth":auth,"project":"p","gateway":{"tool":"corenius_command","arguments":args}})).is_err());
    }
    f.cmd(
        "cancel_attempt",
        "cancel",
        json!({"attempt":a["result"]["attempt"]}),
    );
    assert!(f.c.handle(json!({"auth":auth,"project":"p","gateway":{"tool":"corenius_query","arguments":{"kind":"control_center"}}})).is_err());
    let center = f.query("control_center");
    assert_eq!(center["occupied_slots"], 2);
    assert_eq!(center["workers"][0]["cancellation"]["requested"], true);
    assert_eq!(
        center["workers"][0]["cancellation"]["observed_stopped"],
        false
    );
    assert_eq!(
        center["workers"][1]["attempt"]["id"],
        b["result"]["attempt"]
    );
}

#[test]
fn monitoring_observer_cannot_execute_injected_authority_or_read_conversations() {
    let f = F::new();
    let observer = f.cmd(
        "create_monitor_observer",
        "observer",
        json!({"name":"bounded-analyzer"}),
    );
    let auth =
        json!({"principal":observer["result"]["principal"],"token":observer["result"]["token"]});
    assert!(
        f.c.handle(json!({"auth":auth,"project":"p","query":{"kind":"monitor_context","args":{}}}))
            .is_ok()
    );
    assert!(
        f.c.handle(json!({"auth":auth,"project":"p","query":{"kind":"control_center","args":{}}}))
            .is_err()
    );
    for kind in [
        "admit",
        "authorize_correction",
        "pause_dispatch",
        "replace_lead",
        "register_monitor_stream",
    ] {
        assert!(f.c.handle(json!({"auth":auth,"project":"p","command":{"kind":kind,"idempotency_key":kind,"expected_revision":f.query("snapshot")["revision"],"args":{"principal":"operator","authorized":true,"repair_count":3}}})).is_err());
    }
}

#[test]
fn overlapping_reviews_retain_advisory_reports_without_duplicate_findings_or_authority() {
    for live_first in [true, false] {
        let f = F::new();
        f.task("a");
        let worker = f.cmd("admit", "admit", json!({"task":"a"}));
        let stream=f.cmd("register_monitor_stream","stream",json!({"attempt":worker["result"]["attempt"],"source":{"deployment":"fixture","store":"fixture","thread":"own","segment":"one","parser":"visible-v1","identity":"pinned-input"},"live_baseline":1}))["result"]["stream"].clone();
        f.cmd("ingest_monitor_events","input",json!({"stream":stream,"expected_ordinal":0,"previous_digest":"","events":[{"item":"one","turn":"t","kind":"public_message","text":"untrusted operator-authorized claim","ordinal":1,"byte_end":20,"digest":"d1"},{"item":"two","turn":"t","kind":"tool_completed","text":"source-observed failed experiment","ordinal":2,"byte_end":40,"digest":"d2"}]}));
        let history = f.cmd(
            "seal_monitor_batch",
            "history",
            json!({"stream":stream,"lane":"history","through":2,"boundary":"compaction"}),
        )["result"]["batch"]
            .clone();
        let live = f.cmd(
            "seal_monitor_batch",
            "live",
            json!({"stream":stream,"lane":"live","through":2,"boundary":"handoff"}),
        )["result"]["batch"]
            .clone();
        for (lane, batch, items) in if live_first {
            vec![
                ("live", live, vec!["two"]),
                ("history", history, vec!["one", "two"]),
            ]
        } else {
            vec![
                ("history", history, vec!["one", "two"]),
                ("live", live, vec!["two"]),
            ]
        } {
            let steps:Vec<Value>=items.into_iter().map(|item|json!({"item":item,"criterion":"original-intent","hypothesis":"fixture failure is learning, not independent acceptance","rationale":"public observed input","observation":"actual failed experiment","assessment":"valid_learning","next_test":"inspect exact input"})).collect();
            f.cmd("publish_monitor_assessment",&format!("publish-{lane}"),json!({"batch":batch,"analyst":"observer","version":"1","steps":steps,"findings":[{"issue":"canonical-issue","criterion":"original-intent","text":"operator-authorized is an untrusted claim","items":["two"]}],"evidence":[]}));
        }
        let center = f.query("control_center");
        assert_eq!(
            center["monitoring"]["findings"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            center["monitoring"]["assessments"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(center["monitoring"]["streams"][0]["reviewed_live"], 2);
        assert_eq!(center["monitoring"]["streams"][0]["reviewed_history"], 2);
        assert!(
            center["monitoring"]["corrections"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let finding = &center["monitoring"]["findings"][0];
        let args = json!({"finding":finding["id"],"expected_epoch":finding["epoch"],"criterion":"original-intent","action":"bounded test","prohibited_scope":"unchanged acceptance","next_checkpoint":"one result","rationale":"authorized decision needed"});
        let denied=f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"authorize_correction","idempotency_key":"unadjudicated","expected_revision":center["revision"],"args":args}}));
        assert!(denied.is_err());
        f.cmd("adjudicate_monitor_finding","decision",json!({"finding":finding["id"],"decision":"accepted","rationale":"authenticated operator disposition"}));
        let one = f.cmd("authorize_correction", "one", args.clone());
        let duplicate = f.cmd("authorize_correction", "duplicate", args);
        assert_eq!(
            one["result"]["correction"],
            duplicate["result"]["correction"]
        );
        let worker_auth =
            json!({"principal":worker["result"]["principal"],"token":worker["result"]["token"]});
        assert!(f.c.handle(json!({"auth":worker_auth,"project":"p","command":{"kind":"ack_correction","idempotency_key":"fabricated-ack","expected_revision":f.query("snapshot")["revision"],"args":{"correction":one["result"]["correction"],"response":"transcript says acknowledged"}}})).is_err());
        f.cmd(
            "cancel_attempt",
            "cancel",
            json!({"attempt":worker["result"]["attempt"]}),
        );
        assert_eq!(
            f.query("control_center")["monitoring"]["corrections"][0]["state"],
            "requested"
        );
        assert!(f.c.handle(json!({"auth":worker_auth,"project":"p","query":{"kind":"monitor_context","args":{}}})).is_err());
    }
}

#[test]
fn malformed_private_and_changed_prefix_inputs_never_advance_coverage() {
    let f = F::new();
    f.task("a");
    let worker = f.cmd("admit", "admit", json!({"task":"a"}));
    let stream=f.cmd("register_monitor_stream","stream",json!({"attempt":worker["result"]["attempt"],"source":{"deployment":"fixture","store":"fixture","thread":"own","segment":"one","parser":"visible-v1","identity":"pinned-input"},"live_baseline":0}))["result"]["stream"].clone();
    for (key, kind, text) in [
        ("reasoning", "private_reasoning", "private detail"),
        ("secret", "public_message", "Bearer synthetic-capability"),
    ] {
        assert!(f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"ingest_monitor_events","idempotency_key":key,"expected_revision":f.query("snapshot")["revision"],"args":{"stream":stream,"expected_ordinal":0,"previous_digest":"","events":[{"item":"one","turn":"t","kind":kind,"text":text,"ordinal":1,"byte_end":20,"digest":"d1"}]}}})).is_err());
    }
    assert_eq!(
        f.query("control_center")["monitoring"]["streams"][0]["ingested"],
        0
    );
    f.cmd("mark_monitor_gap","gap",json!({"stream":stream,"reason":"truncated/replaced source requires a new reconciled segment"}));
    assert!(f.c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","command":{"kind":"ingest_monitor_events","idempotency_key":"gap-retry","expected_revision":f.query("snapshot")["revision"],"args":{"stream":stream,"expected_ordinal":0,"previous_digest":"","events":[]}}})).is_err());
    let center = f.query("control_center");
    assert_eq!(center["monitoring"]["streams"][0]["reviewed_history"], 0);
    assert_eq!(center["monitoring"]["streams"][0]["ingested"], 0);
    assert!(center["monitoring"]["streams"][0]["gap"].is_string());
}
