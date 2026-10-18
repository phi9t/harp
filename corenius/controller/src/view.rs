//! Scoped, bounded public read-model pages. Runtime uses its separate occupied-slot context.
use crate::*;

const PAGE_BYTES: usize = 32_768;
const INLINE_BYTES: usize = 2048;

fn selection(collection: &str) -> Result<(&'static str, &'static str, &'static [&'static str])> {
    let selection = match collection {
        "tasks" => (
            "tasks",
            "SELECT t.rowid AS cursor,t.id,t.state,b.id AS brief,b.work_ref,b.source_revision,b.body,b.digest FROM tasks t JOIN briefs b ON b.id=t.brief WHERE t.project=?1 AND (?2 IS NULL OR t.id=?2)",
            &[][..],
        ),
        "attempts" => (
            "attempts",
            "SELECT rowid AS cursor,id,task,brief,base,state,scope_epoch,cancel_requested,observation FROM attempts WHERE project=?1 AND (?3 IS NULL OR id=?3)",
            &[][..],
        ),
        "operations" | "unknown_effects" => (
            "operations",
            "SELECT rowid AS cursor,id,attempt,kind,state,payload,observation FROM operations WHERE project=?1 AND (?3 IS NULL OR attempt=?3)",
            &[][..],
        ),
        "questions" | "pending_questions" => (
            "questions",
            "SELECT x.rowid AS cursor,x.id,x.task,x.sender,x.text,x.answer,x.answered_by,q.attempt,q.state AS delivery_state,q.delivery,q.lead_epoch,q.acknowledgement,q.history FROM questions x LEFT JOIN control_questions q ON q.question=x.id WHERE x.project=?1 AND (?2 IS NULL OR x.task=?2)",
            &[][..],
        ),
        "messages" => (
            "messages",
            "SELECT rowid AS cursor,id,sender,recipient,text,acknowledged FROM messages WHERE project=?1 AND (?3 IS NULL OR sender=?4 OR recipient=?4)",
            &[][..],
        ),
        "progress_reports" => (
            "events",
            "SELECT e.seq AS cursor,e.task,p.attempt,CASE WHEN p.role='attempt' THEN 'attempt' ELSE 'task' END AS report_scope,p.id AS principal,e.body FROM events e JOIN principals p ON p.id=json_extract(e.body,'$.principal') WHERE e.project=?1 AND e.kind='report_progress' AND (?2 IS NULL OR e.task=?2) AND (?3 IS NULL OR p.attempt=?3 OR p.role!='attempt')",
            &["body"][..],
        ),
        "decisions" => (
            "decisions",
            "SELECT rowid AS cursor,id,author,text FROM decisions WHERE project=?1",
            &[][..],
        ),
        "mission" => (
            "control_missions",
            "SELECT rowid AS cursor,document FROM control_missions WHERE project=?1",
            &["document"][..],
        ),
        "brief_proposals" => (
            "control_brief_proposals",
            "SELECT p.rowid AS cursor,p.id,p.task,p.brief,p.author,p.rationale,p.state,b.body,b.digest,b.source_revision FROM control_brief_proposals p JOIN briefs b ON b.id=p.brief WHERE p.project=?1 AND (?2 IS NULL OR p.task=?2)",
            &[][..],
        ),
        "repair_history" => (
            "control_repairs",
            "SELECT rowid AS cursor,id,task,candidate,author,rationale FROM control_repairs WHERE project=?1 AND (?2 IS NULL OR task=?2)",
            &[][..],
        ),
        "candidates" => (
            "candidates",
            "SELECT x.rowid AS cursor,x.* FROM candidates x JOIN attempts a ON a.id=x.attempt WHERE a.project=?1 AND (?2 IS NULL OR a.task=?2)",
            &[][..],
        ),
        "verification" => (
            "verification_reports",
            "SELECT r.rowid AS cursor,r.* FROM verification_reports r JOIN candidates x ON x.id=r.candidate JOIN attempts a ON a.id=x.attempt WHERE a.project=?1 AND (?2 IS NULL OR a.task=?2)",
            &[][..],
        ),
        "acceptance" => (
            "acceptance_decisions",
            "SELECT d.rowid AS cursor,d.* FROM acceptance_decisions d JOIN candidates x ON x.id=d.candidate JOIN attempts a ON a.id=x.attempt WHERE a.project=?1 AND (?2 IS NULL OR a.task=?2)",
            &[][..],
        ),
        "landing_queue" => (
            "candidates",
            "SELECT x.rowid AS cursor,t.id AS task,t.state,x.id AS candidate,x.commit_id,x.tree_id,x.base FROM tasks t JOIN attempts a ON a.project=t.project AND a.task=t.id JOIN candidates x ON x.attempt=a.id WHERE t.project=?1 AND (?2 IS NULL OR t.id=?2) AND t.state IN ('submitted','accepted','integration-required') AND (NOT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='git_workspaces') OR x.id=(SELECT current_candidate FROM git_workspaces WHERE attempt=a.id))",
            &[][..],
        ),
        "runtime_bindings" => (
            "codex_bindings",
            "SELECT b.rowid AS cursor,b.id,b.role,b.attempt,b.authority_epoch,b.state,json_remove(b.document,'$.native_turns','$.native_activity') AS document,s.ingested AS retained_lead_items,s.gap AS retained_lead_gap FROM codex_bindings b LEFT JOIN control_lead_streams s ON s.binding=b.id WHERE b.project=?1 AND (?3 IS NULL OR b.attempt=?3)",
            &["document"][..],
        ),
        "runtime_operations" | "runtime_uncertainty" => (
            "codex_operations",
            "SELECT o.rowid AS cursor,o.id,o.binding,o.kind,o.state,o.observation FROM codex_operations o JOIN codex_bindings b ON b.id=o.binding WHERE o.project=?1 AND (?3 IS NULL OR b.attempt=?3)",
            &["observation"][..],
        ),
        "runtime_public_history" => (
            "monitor_native_inputs",
            "SELECT e.rowid AS cursor,b.id AS binding,s.id AS stream,s.attempt,s.epoch,s.source,e.item,e.ordinal,e.document,n.artifact AS input_digest,m.bytes AS input_bytes,n.redaction FROM monitor_events e JOIN monitor_streams s ON s.id=e.stream JOIN monitor_native_inputs n ON n.stream=e.stream AND n.item=e.item JOIN artifact_manifests m ON m.digest=n.artifact JOIN codex_bindings b ON b.project=s.project AND b.attempt=s.attempt AND b.authority_epoch=s.epoch WHERE s.project=?1 AND (?3 IS NULL OR s.attempt=?3)",
            &["source", "document", "redaction"][..],
        ),
        "lead_public_history" => (
            "control_lead_inputs",
            "SELECT n.seq AS cursor,n.binding,s.epoch,n.item,n.turn,n.kind,n.source,n.artifact AS input_digest,n.bytes AS input_bytes,n.redaction FROM control_lead_inputs n JOIN control_lead_streams s ON s.binding=n.binding WHERE s.project=?1 AND ?3 IS NULL",
            &["source", "redaction"][..],
        ),
        "tool_invocations" => (
            "control_tool_calls",
            "SELECT rowid AS cursor,identity,binding,epoch,method,arguments,response IS NOT NULL AS response_retained FROM control_tool_calls WHERE project=?1 AND (?3 IS NULL OR binding IN (SELECT id FROM codex_bindings WHERE attempt=?3))",
            &["arguments"][..],
        ),
        "sources" => (
            "kata_sources",
            "SELECT s.rowid AS cursor,s.task,a.body AS admitted,v.body AS current,s.acknowledged_revision AS acknowledged_kata_revision,s.drift FROM kata_sources s JOIN kata_versions a ON a.id=s.admitted JOIN kata_versions v ON v.id=s.current WHERE s.project=?1 AND (?2 IS NULL OR s.task=?2)",
            &["admitted", "current"][..],
        ),
        "source_proposals" => (
            "kata_proposals",
            "SELECT p.rowid AS cursor,p.id,p.task,p.version,v.body FROM kata_proposals p JOIN kata_versions v ON v.id=p.version WHERE p.project=?1 AND (?2 IS NULL OR p.task=?2)",
            &["body"][..],
        ),
        "followup_proposals" => (
            "followup_proposals",
            "SELECT rowid AS cursor,id,task,author,text FROM followup_proposals WHERE project=?1 AND (?2 IS NULL OR task=?2)",
            &[][..],
        ),
        "synchronization_effects" => (
            "kata_effects",
            "SELECT rowid AS cursor,id,task,event_seq,controller_revision,kind,state,payload,observation,acknowledged_revision FROM kata_effects WHERE project=?1 AND (?2 IS NULL OR task=?2)",
            &[][..],
        ),
        "synchronization_history" => (
            "kata_effect_history",
            "SELECT h.seq AS cursor,h.seq,h.effect,h.state,h.observation FROM kata_effect_history h JOIN kata_effects e ON e.id=h.effect WHERE e.project=?1 AND (?2 IS NULL OR e.task=?2)",
            &[][..],
        ),
        "monitor_streams" => (
            "monitor_streams",
            "SELECT rowid AS cursor,id,attempt,epoch,source,ingested,byte_offset,previous_digest,reviewed_history,reviewed_live,segment,gap,last_ingested_at,ingested-reviewed_history AS backlog,reviewed_history<reviewed_live AS history_gap,CASE WHEN ingested=0 THEN NULL ELSE CAST(MAX(0,(julianday('now')-julianday(last_ingested_at))*86400) AS INTEGER) END AS age_seconds FROM monitor_streams WHERE project=?1 AND (?3 IS NULL OR attempt=?3)",
            &["source"][..],
        ),
        "monitor_batches" => (
            "monitor_batches",
            "SELECT b.rowid AS cursor,b.id,b.stream,b.lane,b.predecessor,b.through,b.input_digest,json_extract(b.intent,'$.brief') AS brief,json_extract(b.intent,'$.digest') AS intent_digest,b.boundary,b.state FROM monitor_batches b JOIN monitor_streams s ON s.id=b.stream WHERE s.project=?1 AND (?3 IS NULL OR s.attempt=?3)",
            &[][..],
        ),
        "monitor_assessments" => (
            "monitor_assessments",
            "SELECT a.rowid AS cursor,a.id,a.batch,a.analyst,a.version,a.output_digest FROM monitor_assessments a JOIN monitor_batches b ON b.id=a.batch JOIN monitor_streams s ON s.id=b.stream WHERE s.project=?1 AND (?3 IS NULL OR s.attempt=?3)",
            &[][..],
        ),
        "monitor_findings" => (
            "monitor_findings",
            "SELECT rowid AS cursor,* FROM monitor_findings WHERE project=?1 AND (?3 IS NULL OR attempt=?3)",
            &[][..],
        ),
        "monitor_corrections" => (
            "monitor_corrections",
            "SELECT rowid AS cursor,* FROM monitor_corrections WHERE project=?1 AND (?3 IS NULL OR attempt=?3)",
            &[][..],
        ),
        "monitor_hypotheses" => (
            "monitor_assessments",
            "SELECT a.rowid AS cursor,s.attempt,s.epoch,a.id AS assessment,json_extract(a.report,'$.steps[#-1].criterion') AS criterion,substr(json_extract(a.report,'$.steps[#-1].hypothesis'),1,1024) AS hypothesis,json_extract(a.report,'$.steps[#-1].assessment') AS disposition,substr(json_extract(a.report,'$.steps[#-1].next_test'),1,1024) AS next_test FROM monitor_assessments a JOIN monitor_batches b ON b.id=a.batch JOIN monitor_streams s ON s.id=b.stream WHERE s.project=?1 AND (?3 IS NULL OR s.attempt=?3)",
            &[][..],
        ),
        "events" => (
            "events",
            "SELECT e.seq AS cursor,e.seq,e.task,e.kind,e.body FROM events e WHERE e.project=?1 AND (?2 IS NULL OR (e.kind IN ('send_message','ack_message') AND EXISTS(SELECT 1 FROM messages m WHERE m.project=e.project AND m.id=json_extract(e.body,'$.result.message') AND (?4=m.sender OR ?4=m.recipient))) OR (e.kind NOT IN ('send_message','ack_message') AND (e.task=?2 OR e.audience=?4 OR e.kind IN ('record_decision','pause_dispatch'))))",
            &["body"][..],
        ),
        _ => return Err("control record collection refused".into()),
    };
    Ok(selection)
}

struct RecordRange<'a> {
    cursor: i64,
    limit: i64,
    exact: bool,
    binding: Option<&'a str>,
}

fn raw(
    c: &Connection,
    p: &Principal,
    project: &str,
    collection: &str,
    range: RecordRange<'_>,
) -> Result<Value> {
    let RecordRange {
        cursor,
        limit,
        exact,
        binding,
    } = range;
    let (table, mut sql, fields) = selection(collection)?;
    if !db(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name=?1)",
        [table],
        |r| r.get::<_, bool>(0),
    ))? {
        return Ok(json!([]));
    }
    if collection == "landing_queue"
        && !db(c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='git_workspaces')",
            [],
            |r| r.get::<_, bool>(0),
        ))?
    {
        sql = "SELECT x.rowid AS cursor,t.id AS task,t.state,x.id AS candidate,x.commit_id,x.tree_id,x.base FROM tasks t JOIN attempts a ON a.project=t.project AND a.task=t.id JOIN candidates x ON x.attempt=a.id WHERE t.project=?1 AND (?2 IS NULL OR t.id=?2) AND t.state IN ('submitted','accepted','integration-required')";
    }
    if collection == "tool_invocations"
        && !db(c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='codex_bindings')",
            [],
            |r| r.get::<_, bool>(0),
        ))?
    {
        sql = "SELECT rowid AS cursor,identity,binding,epoch,method,arguments,response IS NOT NULL AS response_retained FROM control_tool_calls WHERE project=?1 AND ?3 IS NULL";
    }
    let filter = match collection {
        "pending_questions" => " AND answer IS NULL",
        "unknown_effects" | "runtime_uncertainty" => " AND state IN ('unknown','applying')",
        _ => "",
    };
    let binding_filter = if matches!(
        collection,
        "runtime_operations"
            | "runtime_uncertainty"
            | "runtime_public_history"
            | "lead_public_history"
    ) {
        " AND (?7 IS NULL OR binding=?7)"
    } else {
        ""
    };
    let sql = format!(
        "SELECT * FROM ({sql}) WHERE cursor{}?5{filter}{binding_filter} ORDER BY cursor LIMIT ?6",
        if exact { "=" } else { ">" }
    );
    let values: [&dyn rusqlite::ToSql; 7] = [
        &project, &p.task, &p.attempt, &p.id, &cursor, &limit, &binding,
    ];
    let mut result = query::rows(
        c,
        &sql,
        &values[..if binding_filter.is_empty() { 6 } else { 7 }],
    )?;
    for row in result.as_array_mut().unwrap() {
        if collection == "monitor_streams" {
            row["history_gap"] = json!(row["history_gap"] == 1);
        }
        if collection == "runtime_public_history" {
            row["input_reference"] = json!({"kind":"runtime_public_input","args":{"stream":row["stream"],"item":row["item"],"sha256":row["input_digest"],"bytes":row["input_bytes"]}});
        }
        if collection == "lead_public_history" {
            row["input_reference"] = json!({"kind":"runtime_public_input","args":{"binding":row["binding"],"item":row["item"],"sha256":row["input_digest"],"bytes":row["input_bytes"]}});
        }
        for field in fields {
            if let Some(s) = row[field].as_str()
                && s.len() <= 8192
                && let Ok(value) = serde_json::from_str(s)
            {
                row[field] = value;
            }
        }
        if collection == "runtime_bindings" {
            row["public_history_reference"] = json!({"kind":"control_records","args":{"collection":if row["role"]=="lead"{"lead_public_history"}else{"runtime_public_history"},"binding":row["id"],"after":0,"limit":20},"coverage":"retained typed public messages and completed/failed tool calls; unfinished/private reasoning excluded; no reviewed credit"});
            if row["role"] == "lead" {
                row["public_history_coverage"] = json!({"retained_items":row["retained_lead_items"],"gap":if row["retained_lead_items"].is_null(){json!("no typed visible receipt retained; historical coverage unavailable")}else{row["retained_lead_gap"].clone()},"worker_review_credit":false});
            }
            if row["role"] == "lead" || row["attempt"].as_str() == p.attempt.as_deref() {
                row["native_attachment_history_reference"] = json!({"native_ui":{"method":"thread/turns/list","params":{"threadId":row["document"]["thread_id"],"limit":20}},"authority":"current authenticated own attachment only; native store/incarnation and issued cursor are revalidated"});
                row["native_attachment_events_reference"] = json!({"native_ui":{"method":"corenius/events","params":{"after":0}},"item_limit":20,"byte_limit":524288,"continuation":"pass the last returned seq as after; deferred events stay retained; current own attachment only"});
            }
        }
    }
    Ok(result)
}

fn compact(value: &mut Value, collection: &str, cursor: i64, pointer: &str) {
    match value {
        Value::String(s)
            if s.len()
                > if matches!(pointer, "/history" | "/acknowledgement") {
                    8192
                } else {
                    INLINE_BYTES
                } =>
        {
            let digest = hash(s);
            let bytes = s.len();
            *value = json!({"sha256":digest,"bytes":bytes,"text_reference":{"kind":"control_input","collection":collection,"cursor":cursor,"field":pointer,"sha256":digest,"bytes":bytes}});
        }
        Value::Object(object) => {
            for (name, child) in object {
                compact(
                    child,
                    collection,
                    cursor,
                    &format!("{pointer}/{}", name.replace('~', "~0").replace('/', "~1")),
                );
            }
        }
        Value::Array(array) => {
            for (index, child) in array.iter_mut().enumerate() {
                compact(child, collection, cursor, &format!("{pointer}/{index}"));
            }
        }
        _ => {}
    }
}

pub(super) fn page(
    c: &Connection,
    p: &Principal,
    project: &str,
    collection: &str,
    args: &Value,
) -> Result<Value> {
    if collection == "lead_public_history" && !matches!(p.role.as_str(), "operator" | "lead") {
        return Err("authorization: retained lead history scope refused".into());
    }
    if let Some(binding) = args["binding"].as_str() {
        if binding.is_empty() || binding.len() > 200 {
            return Err("control binding bound".into());
        }
        let valid:bool=db(c.query_row("SELECT EXISTS(SELECT 1 FROM codex_bindings WHERE id=?1 AND project=?2 AND (?3 IS NULL OR attempt=?3))",params![binding,project,p.attempt],|r|r.get(0)))?;
        if !valid {
            return Err("control history binding scope refused".into());
        }
    }
    let after =
        i64::try_from(args["after"].as_u64().unwrap_or(0)).map_err(|_| "control cursor bound")?;
    let limit = args["limit"].as_u64().unwrap_or(20).clamp(1, 20) as usize;
    let rows = raw(
        c,
        p,
        project,
        collection,
        RecordRange {
            cursor: after,
            limit: (limit + 1) as i64,
            exact: false,
            binding: args["binding"].as_str(),
        },
    )?;
    let mut items = Vec::new();
    let mut bytes = 2;
    let mut cursor = after;
    let mut more = false;
    for original in rows.as_array().unwrap() {
        let mut row = original.clone();
        let position = row["cursor"].as_i64().ok_or("control cursor missing")?;
        if (collection == "questions" || collection == "pending_questions")
            && let Some(text) = row["text"].as_str()
            && text.len() > INLINE_BYTES
        {
            let digest = hash(text);
            let size = text.len();
            row["text_reference"] =
                json!({"kind":"question_input","question":row["id"],"sha256":digest,"bytes":size});
            row.as_object_mut().unwrap().remove("text");
        }
        compact(&mut row, collection, position, "");
        let size = row.to_string().len() + usize::from(!items.is_empty());
        if items.len() == limit || bytes + size > PAGE_BYTES - 1024 {
            if items.is_empty() {
                return Err("single control descriptor exceeds page budget".into());
            }
            more = true;
            break;
        }
        bytes += size;
        cursor = position;
        items.push(row);
    }
    Ok(
        json!({"items":items,"page":{"collection":collection,"after":cursor,"has_more":more,"item_limit":limit,"byte_limit":PAGE_BYTES,"continuation":{"kind":"control_records","args":{"collection":collection,"binding":args["binding"],"after":cursor,"limit":limit}}}}),
    )
}

pub(super) fn input(c: &Connection, p: &Principal, project: &str, args: &Value) -> Result<Value> {
    let collection = text(args, "collection")?;
    let cursor = args["cursor"]
        .as_i64()
        .filter(|n| *n >= 0)
        .ok_or("control input cursor bound")?;
    let rows = raw(
        c,
        p,
        project,
        collection,
        RecordRange {
            cursor,
            limit: 1,
            exact: true,
            binding: None,
        },
    )?;
    let row = rows
        .as_array()
        .unwrap()
        .first()
        .ok_or("control input missing/scope")?;
    let field = text(args, "field")?;
    let content = row
        .pointer(field)
        .and_then(Value::as_str)
        .ok_or("control input exact text field missing")?;
    let digest = hash(content);
    if args["sha256"] != digest || args["bytes"].as_u64() != Some(content.len() as u64) {
        return Err("control input changed; refresh exact reference".into());
    }
    let offset = usize::try_from(args["offset"].as_u64().unwrap_or(0))
        .map_err(|_| "control input offset bound")?;
    if offset > content.len() || !content.is_char_boundary(offset) {
        return Err("control input UTF8 offset refused".into());
    }
    let mut end = content.len().min(offset.saturating_add(65536));
    while !content.is_char_boundary(end) {
        end -= 1;
    }
    Ok(
        json!({"collection":collection,"cursor":cursor,"field":field,"sha256":digest,"bytes":content.len(),"offset":offset,"text":&content[offset..end],"next_offset":if end<content.len(){Some(end)}else{None}}),
    )
}

fn attach(
    c: &Connection,
    p: &Principal,
    project: &str,
    value: &mut Value,
    collection: &str,
    args: &Value,
) -> Result<()> {
    let request = json!({"after":args["cursors"][collection].as_u64().unwrap_or(0),"limit":args["limit"].as_u64().unwrap_or(20)});
    let result = page(c, p, project, collection, &request)?;
    value[collection] = result["items"].clone();
    value["pagination"][collection] = result["page"].clone();
    Ok(())
}

pub(super) fn snapshot(
    c: &Connection,
    p: &Principal,
    project: &str,
    args: &Value,
) -> Result<Value> {
    let project_row = query::rows(
        c,
        "SELECT id,base,revision,lead_epoch,paused FROM projects WHERE id=?1",
        &[&project],
    )?;
    let project_row = project_row
        .as_array()
        .unwrap()
        .first()
        .ok_or("project missing")?;
    let occupied: i64 = db(c.query_row(
        "SELECT COUNT(*) FROM slots WHERE attempt IS NOT NULL",
        [],
        |r| r.get(0),
    ))?;
    let mut value = json!({"project":project_row,"revision":project_row["revision"],"occupied_slots":occupied,"pagination":{}});
    for collection in [
        "tasks",
        "attempts",
        "operations",
        "questions",
        "messages",
        "decisions",
    ] {
        attach(c, p, project, &mut value, collection, args)?;
    }
    Ok(value)
}

pub(super) fn context(c: &Connection, p: &Principal, project: &str, args: &Value) -> Result<Value> {
    let mut value = snapshot(c, p, project, args)?;
    for collection in [
        "brief_proposals",
        "repair_history",
        "progress_reports",
        "candidates",
        "verification",
        "acceptance",
        "landing_queue",
        "pending_questions",
        "unknown_effects",
        "runtime_bindings",
        "tool_invocations",
        "runtime_uncertainty",
        "sources",
        "source_proposals",
        "followup_proposals",
        "synchronization_effects",
        "synchronization_history",
    ] {
        attach(c, p, project, &mut value, collection, args)?;
    }
    let mission = page(c, p, project, "mission", &json!({}))?;
    value["mission"] = mission["items"]
        .as_array()
        .unwrap()
        .first()
        .map(|m| m["document"].clone())
        .unwrap_or(json!({"objective":null,"plan":null,"constraints":null,"intent":null}));
    value["pagination"]["mission"] = mission["page"].clone();
    value["lead_epoch"] = value["project"]["lead_epoch"].clone();
    value["event_cursor"] = json!(db(c.query_row(
        "SELECT COALESCE(MAX(seq),0) FROM events WHERE project=?1",
        [project],
        |r| r.get::<_, i64>(0)
    ))?);
    let mut monitoring = json!({"pagination":{},"continuous_runner":"not implemented/qualified; ticket06","authority":"advisory analyzer; controller authenticates adjudication and delivery"});
    for (collection, name) in [
        ("monitor_streams", "streams"),
        ("monitor_batches", "batches"),
        ("monitor_assessments", "assessments"),
        ("monitor_findings", "findings"),
        ("monitor_corrections", "corrections"),
        ("monitor_hypotheses", "hypotheses"),
    ] {
        let result = page(
            c,
            p,
            project,
            collection,
            &json!({"after":args["cursors"][collection].as_u64().unwrap_or(0)}),
        )?;
        monitoring[name] = result["items"].clone();
        monitoring["pagination"][name] = result["page"].clone();
    }
    value["monitoring"] = monitoring;
    value["synchronization"] = json!({"authority":"SQLite/controller","effects":value["synchronization_effects"],"history":value["synchronization_history"],"pagination":{"effects":value["pagination"]["synchronization_effects"],"history":value["pagination"]["synchronization_history"]},"mapping":{"created":"unassigned","active_starting":"assigned","active_running":"running","active_question_unknown_cancel":"blocked","submitted":"submitted","repair-required":"blocked","integration-required":"blocked","accepted":"accepted","landed":"landed","aborted":"aborted"}});
    let runtime_operations = page(
        c,
        p,
        project,
        "runtime_operations",
        &json!({"after":args["cursors"]["runtime_operations"].as_u64().unwrap_or(0)}),
    )?;
    value["native_runtime"] = json!({"bindings":value["runtime_bindings"],"operations":runtime_operations["items"],"pagination":{"bindings":value["pagination"]["runtime_bindings"],"operations":runtime_operations["page"]}});
    let proposals = value["source_proposals"].clone();
    let proposal_continuation = value["pagination"]["source_proposals"]["continuation"].clone();
    for source in value["sources"].as_array_mut().unwrap() {
        source["proposed"] = json!(
            proposals
                .as_array()
                .unwrap()
                .iter()
                .filter(|q| q["task"] == source["task"])
                .cloned()
                .collect::<Vec<_>>()
        );
        source["proposed_continuation"] = proposal_continuation.clone();
    }
    value["runtime_cleanup"] = json!({"qualified":true,"disposition":"per_binding_observation"});
    value["linked_tasks"] = json!(
        value["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| json!({"task":s["task"],"links":s["admitted"]["links"]}))
            .collect::<Vec<_>>()
    );
    if !kata::is_configured(c)? {
        value["synchronization"] =
            json!({"configured":false,"uncertainty":"Kata source not configured"});
    }
    let active = query::rows(
        c,
        "SELECT a.id,a.task,a.brief,a.base,a.state,a.scope_epoch,a.cancel_requested,a.observation FROM attempts a JOIN slots s ON s.attempt=a.id WHERE a.project=?1 AND (?2 IS NULL OR a.id=?2) ORDER BY s.id LIMIT 2",
        &[&project, &p.attempt],
    )?;
    let mut workers = Vec::new();
    for attempt in active.as_array().unwrap() {
        let report:Option<(i64,String)>=db(c.query_row("SELECT e.seq,e.body FROM events e JOIN principals p ON p.id=json_extract(e.body,'$.principal') WHERE e.project=?1 AND e.task=?2 AND e.kind='report_progress' AND p.role='attempt' AND p.attempt=?3 ORDER BY e.seq DESC LIMIT 1",params![project,attempt["task"].as_str(),attempt["id"].as_str()],|r|Ok((r.get(0)?,r.get(1)?))).optional())?;
        let reported = if let Some((cursor, raw)) = report {
            let mut body: Value = if raw.len() > 8192 {
                json!(raw)
            } else {
                serde_json::from_str(&raw).map_err(|e| e.to_string())?
            };
            compact(&mut body, "events", cursor, "/body");
            let attribution =
                json!({"scope":"attempt","attempt":attempt["id"],"event_cursor":cursor});
            if let Some(fields) = body.as_object_mut() {
                fields.insert("attribution".into(), attribution);
            } else {
                body = json!({"body":body,"attribution":attribution});
            }
            body
        } else {
            Value::Null
        };
        let runtime_cursor: Option<i64> = if db(c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='codex_bindings')",
            [],
            |r| r.get::<_, bool>(0),
        ))? {
            db(c.query_row("SELECT rowid FROM codex_bindings WHERE project=?1 AND attempt=?2 ORDER BY authority_epoch DESC LIMIT 1",params![project,attempt["id"].as_str()],|r|r.get(0)).optional())?
        } else {
            None
        };
        let runtime = if let Some(cursor) = runtime_cursor {
            let mut row = raw(
                c,
                p,
                project,
                "runtime_bindings",
                RecordRange {
                    cursor,
                    limit: 1,
                    exact: true,
                    binding: None,
                },
            )?[0]
                .clone();
            compact(&mut row, "runtime_bindings", cursor, "");
            row
        } else {
            Value::Null
        };
        let evidence_cursor: Option<i64> = db(c
            .query_row(
                "SELECT rowid FROM candidates WHERE attempt=?1 ORDER BY rowid DESC LIMIT 1",
                [attempt["id"].as_str()],
                |r| r.get(0),
            )
            .optional())?;
        let material_evidence = if let Some(cursor) = evidence_cursor {
            let mut row = raw(
                c,
                p,
                project,
                "candidates",
                RecordRange {
                    cursor,
                    limit: 1,
                    exact: true,
                    binding: None,
                },
            )?[0]
                .clone();
            compact(&mut row, "candidates", cursor, "");
            row
        } else {
            Value::Null
        };
        let last_visible:Option<i64>=db(c.query_row("SELECT e.rowid FROM monitor_events e JOIN monitor_streams s ON s.id=e.stream JOIN monitor_native_inputs n ON n.stream=e.stream AND n.item=e.item WHERE s.project=?1 AND s.attempt=?2 ORDER BY e.rowid DESC LIMIT 1",params![project,attempt["id"].as_str()],|r|r.get(0)).optional())?;
        let activity = if let Some(cursor) = last_visible {
            let mut item = raw(
                c,
                p,
                project,
                "runtime_public_history",
                RecordRange {
                    cursor,
                    limit: 1,
                    exact: true,
                    binding: None,
                },
            )?[0]
                .clone();
            compact(&mut item, "runtime_public_history", cursor, "");
            item
        } else {
            Value::Null
        };
        workers.push(json!({"attempt":attempt,"observed":{"state":attempt["state"],"observation":attempt["observation"],"stop_observed":attempt["state"]=="exited"},"reported_progress":reported,"runtime":runtime,"last_activity":{"visible_stream":activity,"read_exact_input":"control_records runtime_public_history then runtime_public_input","activity_is_verified_progress":false},"material_evidence":material_evidence,"cancellation":{"requested":attempt["cancel_requested"],"observed_stopped":attempt["state"]=="exited"},"staleness":"no continuous liveness guarantee; inspect observation/event cursor"}));
    }
    value["workers"] = json!(workers);
    value["next_decisions"] = json!([{"questions":db(c.query_row("SELECT COUNT(*) FROM questions WHERE project=?1 AND answer IS NULL AND (?2 IS NULL OR task=?2)",params![project,p.task],|r|r.get::<_,i64>(0)))?,"candidate_acceptance":value["landing_queue"],"brief_amendments":value["brief_proposals"],"unknown_effects":value["unknown_effects"],"pagination":value["pagination"]}]);
    Ok(value)
}
