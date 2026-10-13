use corenius_sqlite_controller::Controller;
use serde_json::{Value, json};
use std::sync::{Arc, Barrier};
use tempfile::TempDir;

struct Fixture {
    _dir: TempDir,
    controller: Arc<Controller>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let controller =
            Arc::new(Controller::open(dir.path().join("state.db"), "operator-secret").unwrap());
        let f = Self {
            _dir: dir,
            controller,
        };
        f.send(
            "create_project",
            0,
            "project",
            json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
        )
        .unwrap();
        f
    }
    fn send(&self, kind: &str, revision: u64, key: &str, args: Value) -> Result<Value, String> {
        command(
            &self.controller,
            "operator",
            "operator-secret",
            kind,
            revision,
            key,
            args,
        )
    }
    fn revision(&self) -> u64 {
        self.query("snapshot", json!({}))["revision"]
            .as_u64()
            .unwrap()
    }
    fn query(&self, kind: &str, args: Value) -> Value {
        self.controller.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":kind,"args":args}})).unwrap()
    }
    fn task(&self, name: &str) {
        self.send("create_task", self.revision(), name, json!({"task":name,"work_ref":{"deployment":"local","project":"kata-p","item":name},"source_revision":"r1","brief":"do work"})).unwrap();
    }
    fn admit(&self, task: &str, key: &str) -> Result<Value, String> {
        self.send("admit", self.revision(), key, json!({"task":task}))
    }
}
fn command(
    c: &Controller,
    principal: &str,
    token: &str,
    kind: &str,
    revision: u64,
    key: &str,
    args: Value,
) -> Result<Value, String> {
    c.handle(json!({"auth":{"principal":principal,"token":token},"project":"p","command":{"kind":kind,"expected_revision":revision,"idempotency_key":key,"args":args}}))
}

#[test]
fn two_slots_atomic_admission_and_scoped_replay() {
    let f = Fixture::new();
    for task in ["a", "b", "c"] {
        f.task(task);
    }
    let revision = f.revision();
    let args = json!({"task":"a"});
    let first = f.send("admit", revision, "start-a", args.clone()).unwrap();
    assert_eq!(
        first,
        f.send("admit", revision, "start-a", args.clone()).unwrap()
    );
    assert!(
        f.send("admit", revision, "start-a", json!({"task":"b"}))
            .unwrap_err()
            .contains("idempotency")
    );
    assert!(
        command(
            &f.controller,
            "operator",
            "bad",
            "admit",
            revision,
            "start-a",
            args
        )
        .unwrap_err()
        .contains("authentication")
    );
    assert!(
        f.send("admit", revision, "stale", json!({"task":"b"}))
            .unwrap_err()
            .contains("revision")
    );
    f.admit("b", "start-b").unwrap();
    assert!(f.admit("c", "start-c").unwrap_err().contains("capacity"));
    let snapshot = f.query("snapshot", json!({}));
    assert_eq!(snapshot["attempts"].as_array().unwrap().len(), 2);
    assert_eq!(snapshot["operations"].as_array().unwrap().len(), 2);
    assert_eq!(snapshot["occupied_slots"], 2);
    let events = f.query("events", json!({"after":first["event_seq"],"limit":50}));
    assert_eq!(events.as_array().unwrap().len(), 1);
}

#[test]
fn same_task_racing_commands_admit_once() {
    let f = Fixture::new();
    f.task("a");
    let revision = f.revision();
    let barrier = Arc::new(Barrier::new(3));
    let joins: Vec<_> = ["race-a", "race-b"]
        .into_iter()
        .map(|key| {
            let c = f.controller.clone();
            let b = barrier.clone();
            std::thread::spawn(move || {
                b.wait();
                command(
                    &c,
                    "operator",
                    "operator-secret",
                    "admit",
                    revision,
                    key,
                    json!({"task":"a"}),
                )
            })
        })
        .collect();
    barrier.wait();
    assert_eq!(
        joins
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(Result::is_ok)
            .count(),
        1
    );
    assert_eq!(f.query("snapshot", json!({}))["occupied_slots"], 1);
}

#[test]
fn restart_retains_journal_questions_decisions_and_unknown_capacity() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let c = Controller::open(&db, "operator-secret").unwrap();
    command(
        &c,
        "operator",
        "operator-secret",
        "create_project",
        0,
        "project",
        json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
    )
    .unwrap();
    command(&c,"operator","operator-secret","create_task",1,"a",json!({"task":"a","work_ref":{"deployment":"d","project":"p","item":"a"},"source_revision":"r","brief":"brief"})).unwrap();
    let admission = command(
        &c,
        "operator",
        "operator-secret",
        "admit",
        2,
        "admit",
        json!({"task":"a"}),
    )
    .unwrap();
    command(
        &c,
        "operator",
        "operator-secret",
        "ask_question",
        3,
        "q",
        json!({"task":"a","text":"which approach?"}),
    )
    .unwrap();
    command(
        &c,
        "operator",
        "operator-secret",
        "record_decision",
        4,
        "d",
        json!({"text":"preserve unknown capacity"}),
    )
    .unwrap();
    command(
        &c,
        "operator",
        "operator-secret",
        "abort_task",
        5,
        "abort",
        json!({"task":"a"}),
    )
    .unwrap();
    drop(c);
    let c = Controller::open(&db, "operator-secret").unwrap();
    let snapshot = c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"snapshot","args":{}}})).unwrap();
    assert_eq!(snapshot["occupied_slots"], 1);
    assert_eq!(
        snapshot["operations"][0]["id"],
        admission["result"]["operation"]
    );
    assert_eq!(snapshot["operations"][0]["state"], "prepared");
    assert_eq!(snapshot["questions"][0]["text"], "which approach?");
    assert_eq!(
        snapshot["decisions"][0]["text"],
        "preserve unknown capacity"
    );
    assert_eq!(snapshot["tasks"][0]["state"], "aborted");
    let cfg = c.handle(json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"p","query":{"kind":"storage_configuration","args":{}}})).unwrap();
    assert_eq!(
        cfg,
        json!({"journal_mode":"wal","synchronous":2,"foreign_keys":1,"schema_version":1})
    );
}

#[test]
fn lead_replacement_does_not_revoke_attempt_and_attempt_cannot_read_other_scope() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
    }
    let a = f.admit("a", "start-a").unwrap()["result"].clone();
    let b = f.admit("b", "start-b").unwrap()["result"].clone();
    let lead = f
        .send("replace_lead", f.revision(), "lead", json!({}))
        .unwrap()["result"]
        .clone();
    let p = a["principal"].as_str().unwrap();
    let token = a["token"].as_str().unwrap();
    let req = |kind: &str, args: Value| json!({"auth":{"principal":p,"token":token},"project":"p","query":{"kind":kind,"args":args}});
    let own = f.controller.handle(req("snapshot", json!({}))).unwrap();
    assert_eq!(own["tasks"].as_array().unwrap().len(), 1);
    assert_eq!(own["tasks"][0]["id"], "a");
    assert!(
        f.controller
            .handle(req("pending_operations", json!({})))
            .unwrap_err()
            .contains("authorization")
    );
    assert!(
        command(
            &f.controller,
            p,
            token,
            "abort_task",
            f.revision(),
            "forbidden",
            json!({"task":"b"})
        )
        .unwrap_err()
        .contains("authorization")
    );
    let question = command(
        &f.controller,
        p,
        token,
        "ask_question",
        f.revision(),
        "question",
        json!({"task":"a","text":"question"}),
    )
    .unwrap();
    f.send("replace_lead", f.revision(), "replace", json!({}))
        .unwrap();
    assert!(
        command(
            &f.controller,
            lead["principal"].as_str().unwrap(),
            lead["token"].as_str().unwrap(),
            "pause_dispatch",
            f.revision(),
            "pause",
            json!({"paused":true})
        )
        .unwrap_err()
        .contains("authentication")
    );
    assert_eq!(
        question,
        command(
            &f.controller,
            p,
            token,
            "ask_question",
            question["revision"].as_u64().unwrap() - 1,
            "question",
            json!({"task":"a","text":"question"})
        )
        .unwrap()
    );
    f.send(
        "cancel_attempt",
        f.revision(),
        "cancel",
        json!({"attempt":a["attempt"]}),
    )
    .unwrap();
    assert!(
        command(
            &f.controller,
            p,
            token,
            "ask_question",
            0,
            "question",
            json!({"task":"a","text":"question"})
        )
        .unwrap_err()
        .contains("authentication")
    );
    assert_eq!(f.query("snapshot", json!({}))["occupied_slots"], 2);
    assert_ne!(a["principal"], b["principal"]);
}

#[test]
fn messages_answers_and_operation_outcomes_are_retained() {
    let f = Fixture::new();
    f.task("a");
    let a = f.admit("a", "admit").unwrap()["result"].clone();
    let message = f
        .send(
            "send_message",
            f.revision(),
            "msg",
            json!({"recipient":a["principal"],"text":"hello"}),
        )
        .unwrap();
    let question = f
        .send(
            "ask_question",
            f.revision(),
            "q",
            json!({"task":"a","text":"how?"}),
        )
        .unwrap();
    f.send(
        "answer_question",
        f.revision(),
        "answer",
        json!({"question":question["result"]["question"],"text":"carefully"}),
    )
    .unwrap();
    command(
        &f.controller,
        a["principal"].as_str().unwrap(),
        a["token"].as_str().unwrap(),
        "ack_message",
        f.revision(),
        "ack",
        json!({"message":message["result"]["message"]}),
    )
    .unwrap();
    f.send(
        "transition_operation",
        f.revision(),
        "applying",
        json!({"operation":a["operation"],"state":"applying","observation":"dispatch intent"}),
    )
    .unwrap();
    f.send(
        "transition_operation",
        f.revision(),
        "unknown",
        json!({"operation":a["operation"],"state":"unknown","observation":"ack missing"}),
    )
    .unwrap();
    f.send(
        "observe_attempt",
        f.revision(),
        "uncertain",
        json!({"attempt":a["attempt"],"state":"unknown","observation":"unresolved"}),
    )
    .unwrap();
    let s = f.query("snapshot", json!({}));
    assert_eq!(s["questions"][0]["answer"], "carefully");
    assert_eq!(s["messages"][0]["acknowledged"], true);
    assert_eq!(s["attempts"][0]["state"], "unknown");
    assert_eq!(s["occupied_slots"], 1);
    f.send("observe_attempt",f.revision(),"exited",json!({"attempt":a["attempt"],"state":"exited","observation":"identity-bound supervisor confirmed exit"})).unwrap();
    assert_eq!(f.query("snapshot", json!({}))["occupied_slots"], 0);
}

#[test]
fn exclusive_lifetime_writer_lock_and_bootstrap_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("state.db");
    let c = Controller::open(&db, "operator-secret").unwrap();
    assert!(
        Controller::open(&db, "operator-secret")
            .err()
            .unwrap()
            .contains("ownership")
    );
    drop(c);
    assert!(
        Controller::open(&db, "replacement-secret")
            .err()
            .unwrap()
            .contains("bootstrap")
    );
    assert!(Controller::open(&db, "operator-secret").is_ok());
}

#[test]
fn dispatch_credentials_and_receipts_are_not_in_public_events_or_other_attempts() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
    }
    let a = f.admit("a", "admit-a").unwrap()["result"].clone();
    let b = f.admit("b", "admit-b").unwrap()["result"].clone();
    let grant = f.query("dispatch_context", json!({"attempt":a["attempt"]}));
    assert_eq!(grant["token"], a["token"]);
    let events = f.query("events", json!({"after":0})).to_string();
    assert!(!events.contains(a["token"].as_str().unwrap()));
    assert!(!events.contains(b["token"].as_str().unwrap()));
    let query = |kind: &str, args: Value| json!({"auth":{"principal":a["principal"],"token":a["token"]},"project":"p","query":{"kind":kind,"args":args}});
    assert!(
        f.controller
            .handle(query("dispatch_context", json!({"attempt":b["attempt"]})))
            .unwrap_err()
            .contains("authorization")
    );
    let own_events = f
        .controller
        .handle(query("events", json!({"after":0})))
        .unwrap()
        .to_string();
    assert!(!own_events.contains(b["attempt"].as_str().unwrap()));
    let own_replay = command(
        &f.controller,
        a["principal"].as_str().unwrap(),
        a["token"].as_str().unwrap(),
        "ask_question",
        f.revision(),
        "admit-b",
        json!({"task":"a","text":"scoped key reuse"}),
    )
    .unwrap();
    assert_ne!(own_replay["result"], b);
}

#[test]
fn answers_reconnect_to_own_attempt_and_storage_busy_is_bounded() {
    let f = Fixture::new();
    f.task("a");
    let a = f.admit("a", "start").unwrap()["result"].clone();
    let q = f
        .send(
            "ask_question",
            f.revision(),
            "q",
            json!({"task":"a","text":"need answer"}),
        )
        .unwrap();
    f.send(
        "answer_question",
        f.revision(),
        "answer",
        json!({"question":q["result"]["question"],"text":"yes"}),
    )
    .unwrap();
    let events=f.controller.handle(json!({"auth":{"principal":a["principal"],"token":a["token"]},"project":"p","query":{"kind":"events","args":{"after":q["event_seq"]}}})).unwrap();
    assert_eq!(events[0]["kind"], "answer_question");
    let conn = rusqlite::Connection::open(f._dir.path().join("state.db")).unwrap();
    conn.execute_batch("BEGIN IMMEDIATE").unwrap();
    let begin = std::time::Instant::now();
    let blocked = f.send(
        "record_decision",
        f.revision(),
        "blocked",
        json!({"text":"must not hang"}),
    );
    assert!(blocked.unwrap_err().contains("locked"));
    assert!(begin.elapsed() < std::time::Duration::from_secs(2));
    conn.execute_batch("ROLLBACK").unwrap();
    f.send(
        "record_decision",
        f.revision(),
        "blocked",
        json!({"text":"must not hang"}),
    )
    .unwrap();
}

#[test]
fn controller_files_have_private_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    for name in [
        "state.db",
        "state.db-wal",
        "state.db-shm",
        "state.db.controller-lock",
    ] {
        let mode = std::fs::metadata(f._dir.path().join(name))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "{name}");
    }
}

#[test]
fn canonical_work_ref_cannot_be_admitted_under_two_aliases() {
    let f = Fixture::new();
    f.task("a");
    let args = json!({"task":"alias-a","work_ref":{"deployment":"local","project":"kata-p","item":"a"},"source_revision":"r1","brief":"same logical task"});
    let revision = f.revision();
    assert!(f.send("create_task", revision, "alias", args).is_err());
    assert_eq!(f.revision(), revision);
    assert_eq!(
        f.query("snapshot", json!({}))["tasks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn project_scopes_and_two_global_slots_hold_across_projects() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
        f.admit(task, &format!("start-{task}")).unwrap();
    }
    let lead = f
        .send("replace_lead", f.revision(), "lead", json!({}))
        .unwrap()["result"]
        .clone();
    let request = |body: Value| json!({"auth":{"principal":"operator","token":"operator-secret"},"project":"q","command":body});
    f.controller.handle(request(json!({"kind":"create_project","expected_revision":0,"idempotency_key":"p","args":{"name":"q","base":"0123456789abcdef0123456789abcdef01234567"}}))).unwrap();
    f.controller.handle(request(json!({"kind":"create_task","expected_revision":1,"idempotency_key":"q-task","args":{"task":"c","work_ref":{"deployment":"local","project":"q","item":"c"},"source_revision":"r","brief":"brief"}}))).unwrap();
    assert!(f.controller.handle(request(json!({"kind":"admit","expected_revision":2,"idempotency_key":"third","args":{"task":"c"}}))).unwrap_err().contains("capacity"));
    assert!(f.controller.handle(json!({"auth":{"principal":lead["principal"],"token":lead["token"]},"project":"q","query":{"kind":"snapshot","args":{}}})).unwrap_err().contains("authorization"));
}

#[test]
fn foreign_or_future_database_is_refused_without_import() {
    let dir = tempfile::tempdir().unwrap();
    let foreign = dir.path().join("foreign.db");
    let connection = rusqlite::Connection::open(&foreign).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE unrelated(value TEXT); INSERT INTO unrelated VALUES('preserve');",
        )
        .unwrap();
    drop(connection);
    assert!(
        Controller::open(&foreign, "operator-secret")
            .err()
            .unwrap()
            .contains("fresh")
    );
    let connection = rusqlite::Connection::open(&foreign).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT value FROM unrelated", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "preserve"
    );
    let future = dir.path().join("future.db");
    let connection = rusqlite::Connection::open(&future).unwrap();
    connection.execute_batch("PRAGMA user_version=47").unwrap();
    drop(connection);
    assert!(
        Controller::open(&future, "operator-secret")
            .err()
            .unwrap()
            .contains("schema version")
    );
}

#[test]
fn project_base_must_be_a_full_commit_oid_and_failure_is_atomic() {
    let f = Fixture::new();
    let invalid = [
        "HEAD",
        "main",
        "0123456",
        "",
        &"a".repeat(39),
        &"a".repeat(41),
        &"g".repeat(40),
        &"a".repeat(63),
        &"a".repeat(65),
    ];
    for (i, base) in invalid.iter().enumerate() {
        let project = format!("invalid-{i}");
        let request = |args: Value| json!({"auth":{"principal":"operator","token":"operator-secret"},"project":project,"command":{"kind":"create_project","expected_revision":0,"idempotency_key":"create","args":args}});
        assert!(
            f.controller
                .handle(request(json!({"name":project,"base":base})))
                .is_err(),
            "accepted nonexact base: {base}"
        );
        let query = |kind: &str| json!({"auth":{"principal":"operator","token":"operator-secret"},"project":project,"query":{"kind":kind,"args":{}}});
        assert!(f.controller.handle(query("snapshot")).is_err());
        assert!(
            f.controller
                .handle(query("events"))
                .unwrap()
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(f.query("snapshot", json!({}))["occupied_slots"], 0);
        let receipt = f
            .controller
            .handle(request(
                json!({"name":project,"base":"0123456789abcdef0123456789abcdef01234567"}),
            ))
            .unwrap();
        assert_eq!(
            receipt["revision"], 1,
            "failed command must not consume revision or replay key"
        );
    }
    for base in [
        "0123456789abcdef0123456789abcdef01234567".to_owned(),
        "a".repeat(64),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let c = Controller::open(dir.path().join("state.db"), "operator-secret").unwrap();
        command(
            &c,
            "operator",
            "operator-secret",
            "create_project",
            0,
            "project",
            json!({"name":"p","base":base}),
        )
        .unwrap();
        command(&c,"operator","operator-secret","create_task",1,"task",json!({"task":"a","work_ref":{"deployment":"d","project":"p","item":"a"},"source_revision":"r","brief":"brief"})).unwrap();
        let admission = command(
            &c,
            "operator",
            "operator-secret",
            "admit",
            2,
            "admit",
            json!({"task":"a"}),
        )
        .unwrap();
        assert_eq!(admission["result"]["base"], base);
    }
}

fn attempt_query(f: &Fixture, grant: &Value, kind: &str, args: Value) -> Value {
    f.controller.handle(json!({"auth":{"principal":grant["principal"],"token":grant["token"]},"project":"p","query":{"kind":kind,"args":args}})).unwrap()
}
#[test]
fn message_events_ignore_forged_task_and_preserve_peer_participants() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
    }
    let a = f.admit("a", "start-a").unwrap()["result"].clone();
    let b = f.admit("b", "start-b").unwrap()["result"].clone();
    let private = command(
        &f.controller,
        a["principal"].as_str().unwrap(),
        a["token"].as_str().unwrap(),
        "send_message",
        f.revision(),
        "private",
        json!({"recipient":"operator","text":"private A-to-operator content","task":"b"}),
    )
    .unwrap();
    for kind in ["snapshot", "events"] {
        assert!(
            !attempt_query(&f, &b, kind, json!({"after":0}))
                .to_string()
                .contains("private A-to-operator content"),
            "{kind} leaked private message to B"
        );
    }
    assert!(
        attempt_query(
            &f,
            &a,
            "events",
            json!({"after":private["event_seq"].as_i64().unwrap()-1})
        )
        .to_string()
        .contains("private A-to-operator content")
    );
    assert!(
        f.query(
            "events",
            json!({"after":private["event_seq"].as_i64().unwrap()-1})
        )
        .to_string()
        .contains("private A-to-operator content")
    );
    let peer = command(
        &f.controller,
        a["principal"].as_str().unwrap(),
        a["token"].as_str().unwrap(),
        "send_message",
        f.revision(),
        "peer",
        json!({"recipient":b["principal"],"text":"A-to-B peer content","task":"unrelated"}),
    )
    .unwrap();
    for grant in [&a, &b] {
        assert!(
            attempt_query(&f, grant, "snapshot", json!({}))
                .to_string()
                .contains("A-to-B peer content")
        );
        assert!(
            attempt_query(
                &f,
                grant,
                "events",
                json!({"after":peer["event_seq"].as_i64().unwrap()-1})
            )
            .to_string()
            .contains("A-to-B peer content")
        );
    }
}

#[test]
fn message_ack_reconnect_reaches_original_sender_and_recipient() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
    }
    let a = f.admit("a", "start-a").unwrap()["result"].clone();
    let b = f.admit("b", "start-b").unwrap()["result"].clone();
    let message = command(
        &f.controller,
        a["principal"].as_str().unwrap(),
        a["token"].as_str().unwrap(),
        "send_message",
        f.revision(),
        "peer",
        json!({"recipient":b["principal"],"text":"peer"}),
    )
    .unwrap();
    let ack = command(
        &f.controller,
        b["principal"].as_str().unwrap(),
        b["token"].as_str().unwrap(),
        "ack_message",
        f.revision(),
        "ack",
        json!({"message":message["result"]["message"]}),
    )
    .unwrap();
    for grant in [&a, &b] {
        let events = attempt_query(&f, grant, "events", json!({"after":message["event_seq"]}));
        assert_eq!(
            events[0]["seq"], ack["event_seq"],
            "original message participant cannot replay ack"
        );
        assert_eq!(events[0]["kind"], "ack_message");
    }
}

#[test]
fn own_attempt_events_use_retained_task_identity() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
    }
    let a = f.admit("a", "start-a").unwrap()["result"].clone();
    let b = f.admit("b", "start-b").unwrap()["result"].clone();
    for (i, args) in [
        json!({"attempt":a["attempt"],"state":"running","observation":"A is running"}),
        json!({"attempt":a["attempt"],"state":"unknown","observation":"A unresolved","task":"b"}),
    ]
    .into_iter()
    .enumerate()
    {
        let r = f
            .send(
                "observe_attempt",
                f.revision(),
                &format!("observe-{i}"),
                args,
            )
            .unwrap();
        let cursor = json!({"after":r["event_seq"].as_i64().unwrap()-1});
        let own = attempt_query(&f, &a, "events", cursor.clone());
        assert_eq!(
            own[0]["seq"], r["event_seq"],
            "own attempt observation missing from reconnect"
        );
        assert_eq!(own[0]["task"], "a");
        assert!(
            attempt_query(&f, &b, "events", cursor)
                .as_array()
                .unwrap()
                .is_empty(),
            "other worker sees A's observation"
        );
    }
    let cancel = f
        .send(
            "cancel_attempt",
            f.revision(),
            "cancel-a",
            json!({"attempt":a["attempt"],"task":"b"}),
        )
        .unwrap();
    let events = f.query(
        "events",
        json!({"after":cancel["event_seq"].as_i64().unwrap()-1}),
    );
    assert_eq!(events[0]["task"], "a");
    assert!(
        attempt_query(
            &f,
            &b,
            "events",
            json!({"after":cancel["event_seq"].as_i64().unwrap()-1})
        )
        .as_array()
        .unwrap()
        .is_empty()
    );
}

#[test]
fn own_operation_events_use_retained_attempt_identity() {
    let f = Fixture::new();
    for task in ["a", "b"] {
        f.task(task);
    }
    let a = f.admit("a", "start-a").unwrap()["result"].clone();
    let b = f.admit("b", "start-b").unwrap()["result"].clone();
    for (i,args) in [json!({"operation":a["operation"],"state":"applying","observation":"A dispatch journal"}),json!({"operation":a["operation"],"state":"unknown","observation":"A unknown journal","task":"b"})].into_iter().enumerate(){
        let r=f.send("transition_operation",f.revision(),&format!("operation-{i}"),args).unwrap();
        let cursor=json!({"after":r["event_seq"].as_i64().unwrap()-1});
        let own=attempt_query(&f,&a,"events",cursor.clone());
        assert_eq!(own[0]["seq"],r["event_seq"],"own operation transition missing from reconnect");
        assert_eq!(own[0]["task"],"a");
        assert!(attempt_query(&f,&b,"events",cursor).as_array().unwrap().is_empty(),"other worker sees A's operation");
    }
}

#[test]
fn sqlite_like_user_table_is_foreign_and_keeps_its_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("foreign.db");
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute_batch(
        "CREATE TABLE sqlitecache(value TEXT); INSERT INTO sqlitecache VALUES('preserve me');",
    )
    .unwrap();
    drop(c);
    let result = Controller::open(&path, "operator-secret");
    assert!(result.is_err(), "foreign sqlitecache table was imported");
    assert!(result.err().unwrap().contains("fresh"));
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        c.query_row("SELECT value FROM sqlitecache", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "preserve me"
    );
}

#[test]
fn dangling_database_symlink_cannot_create_two_controller_owners() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.db");
    let alias = dir.path().join("alias.db");
    symlink(&target, &alias).unwrap();
    match Controller::open(&alias, "operator-secret") {
        Ok(first) => {
            let via_alias = Controller::open(&alias, "operator-secret").is_ok();
            let via_target = Controller::open(&target, "operator-secret").is_ok();
            assert!(
                !via_alias && !via_target,
                "dangling alias permits competing owners: alias={via_alias}, target={via_target}"
            );
            drop(first);
            panic!("dangling database symlinks must be refused before creating their target");
        }
        Err(error) => assert!(error.contains("dangling database symlink"), "{error}"),
    }
    assert!(!target.exists(), "refused open created the symlink target");
    let owner = Controller::open(&target, "operator-secret").unwrap();
    for path in [&alias, &target] {
        assert!(
            Controller::open(path, "operator-secret")
                .err()
                .unwrap()
                .contains("ownership")
        );
    }
    drop(owner);
    let owner = Controller::open(&alias, "operator-secret").unwrap();
    assert!(
        Controller::open(&target, "operator-secret")
            .err()
            .unwrap()
            .contains("ownership")
    );
    drop(owner);
    assert!(Controller::open(&target, "operator-secret").is_ok());
}

#[test]
fn hardlinked_database_is_refused_through_each_name() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.db");
    let alias = dir.path().join("alias.db");
    drop(Controller::open(&target, "operator-secret").unwrap());
    std::fs::hard_link(&target, &alias).unwrap();
    for path in [&alias, &target] {
        assert!(
            Controller::open(path, "operator-secret")
                .err()
                .unwrap()
                .contains("hard links")
        );
    }
    std::fs::remove_file(alias).unwrap();
    assert!(Controller::open(&target, "operator-secret").is_ok());
}
