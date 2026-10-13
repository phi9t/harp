//! Shared command/projection seam. UI claims never substitute for observed effects.
use crate::*;

pub(super) fn read(controller: &Controller, request: Value) -> Result<Value> {
    let parsed: Request = serde_json::from_value(request.clone()).map_err(|e| e.to_string())?;
    if parsed.command.is_some() {
        return Err("control-center query only".into());
    }
    let (mut value, role) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &parsed.auth, &parsed.project)?;
        let role = p.role.clone();
        (
            query::run(
                &c,
                &p,
                &parsed.project,
                parsed.query.ok_or("control-center query required")?,
            )?,
            role,
        )
    };
    if role != "attempt" {
        let inventory=controller.handle(json!({"auth":request["auth"],"project":parsed.project,"query":{"kind":"workspace_inventory","args":{"after":request["query"]["args"]["cursors"]["workspaces"],"history_after":request["query"]["args"]["cursors"]["git_history"],"limit":request["query"]["args"]["limit"]}}}));
        value["filesystem"] = match inventory {
            Ok(inventory) => inventory,
            Err(error) => json!({"status":"unknown","error":error}),
        };
    } else {
        value["filesystem"] = json!({"status":"inspect own runtime workspace identity; controller-only inventory","runtime":value["runtime_bindings"]});
    }
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    authenticate(&c, &parsed.auth, &parsed.project)?;
    let revision: i64 = db(c.query_row(
        "SELECT revision FROM projects WHERE id=?1",
        [&parsed.project],
        |r| r.get(0),
    ))?;
    value["projection_consistency"] = json!({"controller_revision":value["revision"],"revision_after_filesystem_read":revision,"refresh_required":value["revision"].as_i64()!=Some(revision),"filesystem_observation":"read outside the controller writer; current at observation, may subsequently change"});
    Ok(value)
}

pub(super) fn apply(
    tx: &Transaction<'_>,
    p: &Principal,
    project: &str,
    kind: &str,
    a: &Value,
) -> Result<Value> {
    match kind {
        "set_mission" => {
            for field in ["objective", "plan", "constraints", "intent"] {
                text(a, field)?;
            }
            db(tx.execute("INSERT INTO control_missions VALUES(?1,?2) ON CONFLICT(project) DO UPDATE SET document=excluded.document", params![project,a.to_string()]))?;
            Ok(json!({"mission":a}))
        }
        "revise_brief" => {
            let task = text(a, "task")?;
            let old:(String,String)=db(tx.query_row("SELECT b.work_ref,t.state FROM tasks t JOIN briefs b ON b.id=t.brief WHERE t.project=?1 AND t.id=?2",params![project,task],|r|Ok((r.get(0)?,r.get(1)?))))?;
            let brief = id();
            let proposal = id();
            let captured = kata::revised_brief(tx, project, task, a)?;
            let body = match captured.as_deref() {
                Some(body) => body,
                None => text(a, "body")?,
            };
            db(tx.execute(
                "INSERT INTO briefs VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    brief,
                    project,
                    task,
                    old.0,
                    text(a, "source_revision")?,
                    body,
                    hash(body)
                ],
            ))?;
            db(tx.execute(
                "INSERT INTO control_brief_proposals VALUES(?1,?2,?3,?4,?5,?6,'proposed')",
                params![proposal, project, task, brief, p.id, text(a, "rationale")?],
            ))?;
            Ok(
                json!({"proposal":proposal,"brief":brief,"task":task,"disposition":"proposed; admitted brief unchanged"}),
            )
        }
        "approve_brief" => {
            let proposal = text(a, "proposal")?;
            let (task,brief):(String,String)=db(tx.query_row("SELECT task,brief FROM control_brief_proposals WHERE id=?1 AND project=?2 AND state='proposed'",params![proposal,project],|r|Ok((r.get(0)?,r.get(1)?))))?;
            let live:bool=db(tx.query_row("SELECT EXISTS(SELECT 1 FROM attempts WHERE project=?1 AND task=?2 AND state<>'exited')",params![project,task],|r|r.get(0)))?;
            if live {
                return Err(
                    "brief approval waits for observed termination; active admission immutable"
                        .into(),
                );
            }
            let changed=db(tx.execute("UPDATE tasks SET brief=?3 WHERE project=?1 AND id=?2 AND state IN ('created','repair-required','integration-required')",params![project,task,brief]))?;
            if changed != 1 {
                return Err("task not eligible for revised admission".into());
            }
            kata::approve_revised_brief(tx, project, &task, &brief)?;
            db(tx.execute(
                "UPDATE control_brief_proposals SET state='approved' WHERE id=?1",
                [proposal],
            ))?;
            Ok(json!({"proposal":proposal,"brief":brief,"task":task,"state":"approved"}))
        }
        "request_repair" => {
            let task = text(a, "task")?;
            let candidate = text(a, "candidate")?;
            let valid:bool=db(tx.query_row("SELECT EXISTS(SELECT 1 FROM candidates c JOIN attempts a ON a.id=c.attempt WHERE c.id=?1 AND a.project=?2 AND a.task=?3)",params![candidate,project,task],|r|r.get(0)))?;
            if !valid {
                return Err("repair candidate/task binding mismatch".into());
            }
            let landing:bool=db(tx.query_row("SELECT EXISTS(SELECT 1 FROM operations o JOIN candidates c ON c.id=json_extract(o.payload,'$.candidate') JOIN attempts a ON a.id=c.attempt WHERE o.project=?1 AND a.project=?1 AND (a.task=?2 OR c.id=?3) AND o.kind='git-land_candidate' AND o.state IN ('prepared','applying','unknown'))",params![project,task,candidate],|r|r.get(0)))?;
            if landing {
                return Err("repair waits for unresolved physical landing disposition".into());
            }
            let changed=db(tx.execute("UPDATE tasks SET state='repair-required' WHERE project=?1 AND id=?2 AND state IN ('submitted','accepted','repair-required')",params![project,task]))?;
            if changed != 1 {
                return Err("task cannot request repair".into());
            }
            let repair = id();
            db(tx.execute(
                "INSERT INTO control_repairs VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    repair,
                    project,
                    task,
                    candidate,
                    p.id,
                    text(a, "rationale")?
                ],
            ))?;
            Ok(json!({"repair":repair,"task":task,"candidate":candidate,"state":"repair-required"}))
        }
        "ack_question" => {
            let question = text(a, "question")?;
            let delivery = text(a, "delivery")?;
            let (state,operation,epoch):(String,Option<String>,Option<i64>)=db(tx.query_row("SELECT state,delivery,lead_epoch FROM control_questions q JOIN questions x ON x.id=q.question WHERE x.project=?1 AND q.question=?2",params![project,question],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?;
            let current: i64 = db(tx.query_row(
                "SELECT lead_epoch FROM projects WHERE id=?1",
                [project],
                |r| r.get(0),
            ))?;
            if operation.as_deref() != Some(delivery)
                || epoch != Some(current)
                || !matches!(
                    state.as_str(),
                    "prepared" | "submitted_to_runtime" | "acknowledged_by_lead"
                )
            {
                return Err("question acknowledgement delivery/authority mismatch".into());
            }
            if p.role != "lead" {
                return Err("explicit question acknowledgement requires lead".into());
            }
            db(tx.execute("UPDATE control_questions SET state='acknowledged_by_lead',acknowledgement=?2,history=json_insert(history,'$[#]',json(?3)) WHERE question=?1",params![question,p.id,json!({"state":"acknowledged_by_lead","delivery":delivery,"principal":p.id,"early":state=="prepared"}).to_string()]))?;
            Ok(json!({"question":question,"delivery":delivery,"state":"acknowledged_by_lead"}))
        }
        _ => crate::monitor::apply(tx, p, project, kind, a),
    }
}

pub(super) fn question_created(tx: &Transaction<'_>, question: &str, p: &Principal) -> Result<()> {
    db(tx.execute(
        "INSERT INTO control_questions(question,attempt) VALUES(?1,?2)",
        params![question, p.attempt],
    ))?;
    Ok(())
}
pub(super) fn question_answered(tx: &Transaction<'_>, question: &str, p: &Principal) -> Result<()> {
    let state: Option<String> = db(tx
        .query_row(
            "SELECT state FROM control_questions WHERE question=?1",
            [question],
            |r| r.get(0),
        )
        .optional())?;
    if p.role == "lead" && state.as_deref() != Some("acknowledged_by_lead") {
        return Err("lead answer requires explicit acknowledgement".into());
    }
    db(tx.execute("UPDATE control_questions SET state='resolved',history=json_insert(history,'$[#]',json(?2)) WHERE question=?1",params![question,json!({"state":"resolved","principal":p.id}).to_string()]))?;
    Ok(())
}

pub(super) fn prepare_delivery(
    controller: &Controller,
    binding: &str,
    document: &Value,
) -> Result<()> {
    if document["ui_input_ready"] != true {
        return Ok(());
    }
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let tx = db(c.transaction())?;
    let (project, role, epoch): (String, String, i64) = db(tx.query_row(
        "SELECT project,role,authority_epoch FROM codex_bindings WHERE id=?1",
        [binding],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ))?;
    if role == "attempt" {
        crate::monitor::prepare_correction(&tx, binding, document, &project, epoch)?;
        db(tx.commit())?;
        return Ok(());
    }
    let current: i64 = db(tx.query_row(
        "SELECT lead_epoch FROM projects WHERE id=?1",
        [&project],
        |r| r.get(0),
    ))?;
    if role != "lead" || epoch != current {
        return Ok(());
    }
    let ambiguous:i64=db(tx.query_row("SELECT COUNT(*) FROM codex_operations WHERE binding=?1 AND state IN ('prepared','applying','unknown') AND kind IN ('codex_turn','codex_steer','turn/start')",[binding],|r|r.get(0)))?;
    if ambiguous != 0 {
        return Ok(());
    }
    let mut qs = query::rows(
        &tx,
        "SELECT q.id,q.task,q.text,m.attempt,m.state,m.delivery,m.lead_epoch FROM questions q JOIN control_questions m ON m.question=q.id WHERE q.project=?1 AND q.answer IS NULL AND (m.state='persisted' OR m.lead_epoch<>?2) ORDER BY q.rowid LIMIT 20",
        &[&project, &epoch],
    )?;
    if qs.as_array().unwrap().is_empty() {
        return Ok(());
    }
    let turns = document["native_turns"].as_array();
    let status = turns
        .and_then(|ts| ts.last())
        .and_then(|t| t["status"].as_str());
    let kind = match status {
        Some("completed" | "interrupted" | "failed") => "codex_turn",
        Some("inProgress") => "codex_steer",
        _ => return Ok(()),
    };
    let operation = id();
    const INSTRUCTIONS: &str = "Use corenius_query question_input with the referenced question ID to retrieve exact full text/digest/bytes when text_reference is present; verify its digest. Then ack_question with question+delivery and answer_question. Runtime transport is not explicit acknowledgement.";
    let mut selected = Vec::new();
    for original in qs.as_array_mut().unwrap() {
        let content = original["text"].as_str().ok_or("question text missing")?;
        let mut q = json!({"id":original["id"],"attempt":original["attempt"],"state":original["state"],"delivery":operation,"lead_epoch":epoch,"text_digest":hash(content),"text_bytes":content.len()});
        if content.len() <= 8192 {
            q["text"] = json!(content);
        } else {
            q["text_reference"] = json!({"kind":"question_input","question":original["id"],"sha256":hash(content),"bytes":content.len()});
        }
        selected.push(q);
        if format!("CORENIUS_QUESTIONS\n{}\n{INSTRUCTIONS}", json!(selected)).len() > 65536 {
            selected.pop();
            break;
        }
    }
    if selected.is_empty() {
        return Err("question descriptor exceeds delivery bound".into());
    }
    let qs = json!(selected);
    let text = format!("CORENIUS_QUESTIONS\n{qs}\n{INSTRUCTIONS}");
    for q in qs.as_array().unwrap() {
        db(tx.execute("UPDATE control_questions SET state='prepared',delivery=?2,lead_epoch=?3,history=json_insert(history,'$[#]',json(?4)) WHERE question=?1",params![q["id"].as_str(),operation,epoch,json!({"state":"prepared","delivery":operation,"lead_epoch":epoch,"prior_state":q["state"],"text_digest":q["text_digest"],"text_bytes":q["text_bytes"]}).to_string()]))?;
    }
    let principal:String=db(tx.query_row("SELECT p.id FROM principals p WHERE p.project=?1 AND p.role='lead' AND p.epoch=?2 AND NOT EXISTS(SELECT 1 FROM monitor_observers m WHERE m.principal=p.id)",params![project,epoch],|r|r.get(0)))?;
    let intent = json!({"role":"lead","authority_epoch":epoch,"text":text,"expected_turn":document["turn_id"],"principal":principal,"principal_epoch":epoch,"questions":qs});
    db(tx.execute(
        "INSERT INTO codex_operations VALUES(?1,?2,?3,?4,'prepared',?5,NULL)",
        params![operation, project, binding, kind, intent.to_string()],
    ))?;
    db(tx.commit())?;
    Ok(())
}
pub(super) fn delivery_result(
    c: &Connection,
    operation: &str,
    state: &str,
    observation: &Value,
) -> Result<()> {
    crate::monitor::correction_result(c, operation, state, observation)?;
    let ids = query::rows(
        c,
        "SELECT question,state FROM control_questions WHERE delivery=?1",
        &[&operation],
    )?;
    for q in ids.as_array().unwrap() {
        let next = if state == "confirmed" && q["state"] == "prepared" {
            Some("submitted_to_runtime")
        } else if state == "failed" && q["state"] == "prepared" {
            Some("persisted")
        } else {
            None
        };
        if let Some(next) = next {
            db(c.execute("UPDATE control_questions SET state=?2,history=json_insert(history,'$[#]',json(?3)) WHERE question=?1",params![q["question"].as_str(),next,json!({"state":next,"delivery":operation,"transport_state":state,"observation":observation}).to_string()]))?;
        }
    }
    Ok(())
}
pub(super) fn fail_delivery(
    c: &mut Connection,
    operation: &str,
    observation: &Value,
) -> Result<()> {
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    db(tx.execute(
        "UPDATE codex_operations SET state='failed',observation=?2 WHERE id=?1",
        params![operation, observation.to_string()],
    ))?;
    delivery_result(&tx, operation, "failed", observation)?;
    db(tx.commit())
}

pub(super) fn projection(
    c: &Connection,
    p: &Principal,
    project: &str,
    args: &Value,
) -> Result<Value> {
    crate::view::context(c, p, project, args)
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    #[test]
    fn failed_delivery_write_boundary_reopens_atomically_and_preserves_early_ack() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.db");
        let controller = Controller::open(&path, "operator-secret").unwrap();
        let auth = json!({"principal":"operator","token":"operator-secret"});
        let command = |kind: &str, key: &str, args: Value| {
            let revision = if kind == "create_project" {
                json!(0)
            } else {
                controller
                    .handle(
                        json!({"auth":auth,"project":"p","query":{"kind":"snapshot","args":{}}}),
                    )
                    .unwrap()["revision"]
                    .clone()
            };
            controller.handle(json!({"auth":auth,"project":"p","command":{"kind":kind,"idempotency_key":key,"expected_revision":revision,"args":args}})).unwrap()
        };
        command(
            "create_project",
            "project",
            json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
        );
        command(
            "create_task",
            "task",
            json!({"task":"a","work_ref":{"deployment":"fixture","project":"p","item":"a"},"source_revision":"r1","brief":"retained exact delivery contract"}),
        );
        let worker = command("admit", "admit", json!({"task":"a"}));
        let q = command(
            "ask_question",
            "question",
            json!({"task":"a","text":"durable delivery?"}),
        );
        {
            let mut c = controller.connection.lock().unwrap();
            c.execute_batch(include_str!("codex_schema.sql")).unwrap();
            c.execute(
                "INSERT INTO codex_bindings VALUES('binding','p','lead',NULL,1,'{}','bound')",
                [],
            )
            .unwrap();
            c.execute("INSERT INTO codex_operations VALUES('delivery','p','binding','codex_turn','prepared','{}',NULL)",[]).unwrap();
            c.execute("UPDATE control_questions SET state='prepared',delivery='delivery',lead_epoch=1 WHERE question=?1",[q["result"]["question"].as_str()]).unwrap();
            // Declared prepared native delivery fixture, with real SQLite foreign keys.
            c.execute("INSERT INTO monitor_streams(id,project,attempt,epoch,source,segment) VALUES('stream','p',?1,1,'{}','segment')",[worker["result"]["attempt"].as_str()]).unwrap();
            c.execute("INSERT INTO monitor_batches VALUES('batch','stream','history',0,1,'input','[]','{}','manual','review-pending')",[]).unwrap();
            c.execute("INSERT INTO monitor_assessments VALUES('assessment','batch','fixture','1','{}','output')",[]).unwrap();
            c.execute("INSERT INTO monitor_findings(id,project,attempt,epoch,issue,criterion,document,assessment) VALUES('finding','p',?1,1,'issue','criterion','{}','assessment')",[worker["result"]["attempt"].as_str()]).unwrap();
            c.execute("INSERT INTO monitor_corrections(id,finding,project,attempt,epoch,decision,document,state,operation) VALUES('correction','finding','p',?1,1,'fixture-decision','{}','prepared','delivery')",[worker["result"]["attempt"].as_str()]).unwrap();
            c.execute_batch("CREATE TRIGGER injected_delivery_failure BEFORE UPDATE ON control_questions BEGIN SELECT RAISE(ABORT,'declared second write failure'); END;").unwrap();
            assert!(
                fail_delivery(&mut c, "delivery", &json!({"error":"authority revoked"})).is_err()
            );
        }
        drop(controller);
        let controller = Controller::open(&path, "operator-secret").unwrap();
        let mut c = controller.connection.lock().unwrap();
        let states = |c: &Connection| {
            (
                c.query_row(
                    "SELECT state FROM codex_operations WHERE id='delivery'",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .unwrap(),
                c.query_row("SELECT state FROM control_questions", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap(),
                c.query_row("SELECT state FROM monitor_corrections", [], |r| {
                    r.get::<_, String>(0)
                })
                .unwrap(),
            )
        };
        assert_eq!(
            states(&c),
            ("prepared".into(), "prepared".into(), "prepared".into())
        );
        c.execute_batch("DROP TRIGGER injected_delivery_failure")
            .unwrap();
        fail_delivery(&mut c, "delivery", &json!({"error":"authority revoked"})).unwrap();
        assert_eq!(
            states(&c),
            ("failed".into(), "persisted".into(), "requested".into())
        );
        c.execute_batch("UPDATE codex_operations SET state='prepared'; UPDATE control_questions SET state='acknowledged_by_lead',acknowledgement='retained exact ack'; UPDATE monitor_corrections SET state='acknowledged',acknowledgement='retained addressed ack';").unwrap();
        fail_delivery(&mut c, "delivery", &json!({"error":"stale turn"})).unwrap();
        assert_eq!(
            states(&c),
            (
                "failed".into(),
                "acknowledged_by_lead".into(),
                "acknowledged".into()
            )
        );
        let history: String = c
            .query_row("SELECT history FROM control_questions", [], |r| r.get(0))
            .unwrap();
        fail_delivery(&mut c, "delivery", &json!({"error":"stale turn"})).unwrap();
        assert_eq!(
            history,
            c.query_row("SELECT history FROM control_questions", [], |r| r
                .get::<_, String>(0))
                .unwrap()
        );
    }

    #[test]
    fn repair1_new_question_batches_progress_with_large_individual_and_later_question() {
        let dir = tempfile::tempdir().unwrap();
        let controller = Controller::open(dir.path().join("state.db"), "operator-secret").unwrap();
        let auth = json!({"principal":"operator","token":"operator-secret"});
        let command = |kind: &str, key: &str, args: Value| {
            let revision = if kind == "create_project" {
                json!(0)
            } else {
                controller
                    .handle(
                        json!({"auth":auth,"project":"p","query":{"kind":"snapshot","args":{}}}),
                    )
                    .unwrap()["revision"]
                    .clone()
            };
            controller.handle(json!({"auth":auth,"project":"p","command":{"kind":kind,"idempotency_key":key,"expected_revision":revision,"args":args}})).unwrap()
        };
        command(
            "create_project",
            "project",
            json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
        );
        command(
            "create_task",
            "task",
            json!({"task":"a","work_ref":{"deployment":"fixture","project":"p","item":"a"},"source_revision":"r1","brief":"question byte fairness"}),
        );
        command("replace_lead", "lead", json!({}));
        let mut ids = Vec::new();
        for n in 0..10 {
            let content = if n == 0 {
                "L".repeat(65536)
            } else if n == 9 {
                "late question".into()
            } else {
                "M".repeat(16000)
            };
            ids.push(
                command(
                    "ask_question",
                    &format!("question-{n}"),
                    json!({"task":"a","text":content}),
                )["result"]["question"]
                    .clone(),
            );
        }
        let doc = json!({"thread_id":"lead-thread","turn_id":"old-turn","ui_input_ready":true,"native_turns":[{"id":"old-turn","status":"completed","items":[]}]});
        {
            let c = controller.connection.lock().unwrap();
            c.execute_batch(include_str!("codex_schema.sql")).unwrap();
            c.execute(
                "INSERT INTO codex_bindings VALUES('binding','p','lead',NULL,1,?1,'bound')",
                [doc.to_string()],
            )
            .unwrap();
        }
        for _ in 0..10 {
            let result = prepare_delivery(&controller, "binding", &doc);
            assert!(
                result.is_ok(),
                "valid questions cannot starve behind aggregate bound: {result:?}"
            );
            let mut c = controller.connection.lock().unwrap();
            let tx = c.transaction().unwrap();
            let rows = query::rows(
                &tx,
                "SELECT id,intent FROM codex_operations WHERE state='prepared'",
                &[],
            )
            .unwrap();
            for row in rows.as_array().unwrap() {
                let intent: Value = serde_json::from_str(row["intent"].as_str().unwrap()).unwrap();
                assert!(intent["text"].as_str().unwrap().len() <= 65536);
                tx.execute(
                    "UPDATE codex_operations SET state='confirmed' WHERE id=?1",
                    [row["id"].as_str()],
                )
                .unwrap();
                delivery_result(
                    &tx,
                    row["id"].as_str().unwrap(),
                    "confirmed",
                    &json!({"fixture":"bounded native offer"}),
                )
                .unwrap();
            }
            tx.commit().unwrap();
        }
        let c = controller.connection.lock().unwrap();
        let remaining: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM control_questions WHERE state='persisted'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0);
        let large: String = c
            .query_row(
                "SELECT text FROM questions WHERE id=?1",
                [ids[0].as_str()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(large.len(), 65536);
    }
}
