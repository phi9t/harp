use super::*;
use rusqlite::types::ValueRef;

pub(super) fn rows(c: &Connection, sql: &str, values: &[&dyn rusqlite::ToSql]) -> Result<Value> {
    let mut stmt = db(c.prepare(sql))?;
    let names: Vec<String> = stmt.column_names().into_iter().map(str::to_owned).collect();
    let mapped = db(stmt.query_map(values, |row| {
        let mut object = serde_json::Map::new();
        for (i, name) in names.iter().enumerate() {
            let v = match row.get_ref(i)? {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(n)
                    if matches!(
                        name.as_str(),
                        "acknowledged" | "cancel_requested" | "paused"
                    ) =>
                {
                    json!(n != 0)
                }
                ValueRef::Integer(n) => json!(n),
                ValueRef::Real(n) => json!(n),
                ValueRef::Text(s) => json!(String::from_utf8_lossy(s)),
                ValueRef::Blob(_) => Value::Null,
            };
            object.insert(name.clone(), v);
        }
        Ok(Value::Object(object))
    }))?;
    let values: Vec<Value> = db(mapped.collect())?;
    Ok(Value::Array(values))
}

pub(super) fn run(c: &Connection, p: &Principal, project: &str, q: Query) -> Result<Value> {
    let monitor_reference = matches!(q.kind.as_str(), "control_input" | "control_records")
        && matches!(
            q.args["collection"].as_str(),
            Some(
                "monitor_streams"
                    | "monitor_batches"
                    | "monitor_assessments"
                    | "monitor_findings"
                    | "monitor_corrections"
                    | "monitor_hypotheses"
            )
        );
    if p.role == "observer"
        && !monitor_reference
        && !matches!(
            q.kind.as_str(),
            "monitor_context" | "monitor_batch" | "monitor_assessment" | "monitor_records"
        )
    {
        return Err("authorization: bounded monitoring read only".into());
    }
    match q.kind.as_str() {
        "question_input" => {
            let question = text(&q.args, "question")?;
            let (task,content):(String,String)=db(c.query_row("SELECT task,text FROM questions WHERE id=?1 AND project=?2 AND (?3 IS NULL OR task=?3)",params![question,project,p.task],|r|Ok((r.get(0)?,r.get(1)?))))?;
            Ok(
                json!({"question":question,"task":task,"sha256":hash(&content),"bytes":content.len(),"text":content}),
            )
        }
        "monitor_records" => crate::monitor::records(c, p, project, &q.args),
        "monitor_assessment" => {
            let assessment = text(&q.args, "assessment")?;
            let result = rows(
                c,
                "SELECT a.* FROM monitor_assessments a JOIN monitor_batches b ON b.id=a.batch JOIN monitor_streams s ON s.id=b.stream WHERE a.id=?1 AND s.project=?2 AND (?3 IS NULL OR s.attempt=?3)",
                &[&assessment, &project, &p.attempt],
            )?;
            if result.as_array().unwrap().len() != 1 {
                return Err("monitor assessment missing/scope".into());
            }
            Ok(result[0].clone())
        }
        "monitor_context" => crate::monitor::projection(c, p, project),
        "monitor_batch" => {
            let batch = text(&q.args, "batch")?;
            let result = rows(
                c,
                "SELECT b.* FROM monitor_batches b JOIN monitor_streams s ON s.id=b.stream WHERE b.id=?1 AND s.project=?2 AND (?3 IS NULL OR s.attempt=?3)",
                &[&batch, &project, &p.attempt],
            )?;
            if result.as_array().unwrap().len() != 1 {
                return Err("monitor batch scope/missing".into());
            }
            Ok(result[0].clone())
        }
        "control_center" | "lead_context" => crate::control::projection(c, p, project, &q.args),
        "participant_context" => kata::context(c, p, project),
        "codex_registry" => {
            if p.role != "operator" {
                return Err("authorization: operator required".into());
            }
            let version: i64 = db(c.pragma_query_value(None, "user_version", |r| r.get(0)))?;
            if version == 1 {
                return Ok(json!({"configured":false,"servers":[],"bindings":[],"operations":[]}));
            }
            let mut servers = rows(
                c,
                "SELECT id,document,state FROM codex_servers ORDER BY rowid",
                &[],
            )?;
            let mut bindings = rows(
                c,
                "SELECT id,role,attempt,authority_epoch,document,state FROM codex_bindings WHERE project=?1 ORDER BY rowid",
                &[&project],
            )?;
            let mut operations = rows(
                c,
                "SELECT id,binding,kind,state,intent,observation FROM codex_operations WHERE project=?1 ORDER BY rowid",
                &[&project],
            )?;
            for collection in [&mut servers, &mut bindings, &mut operations] {
                for item in collection.as_array_mut().unwrap() {
                    for field in ["document", "intent", "observation"] {
                        if let Some(raw) = item[field].as_str() {
                            item[field] = serde_json::from_str(raw).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
            Ok(
                json!({"configured":true,"servers":servers,"bindings":bindings,"operations":operations}),
            )
        }
        "runtime_projects" => {
            if p.role != "operator" {
                return Err("authorization: operator required".into());
            }
            rows(c, "SELECT id FROM projects ORDER BY rowid", &[])
        }
        "storage_configuration" => {
            if p.role != "operator" {
                return Err("authorization: operator required".into());
            }
            Ok(
                json!({"journal_mode":db(c.pragma_query_value(None,"journal_mode",|r|r.get::<_,String>(0)))?,
                "synchronous":db(c.pragma_query_value(None,"synchronous",|r|r.get::<_,i64>(0)))?,
                "foreign_keys":db(c.pragma_query_value(None,"foreign_keys",|r|r.get::<_,i64>(0)))?,
                "schema_version":db(c.pragma_query_value(None,"user_version",|r|r.get::<_,i64>(0)))?}),
            )
        }
        "events" => Ok(crate::view::page(c, p, project, "events", &q.args)?["items"].clone()),
        "event_page" => crate::view::page(c, p, project, "events", &q.args),
        "control_records" => {
            crate::view::page(c, p, project, text(&q.args, "collection")?, &q.args)
        }
        "control_input" => crate::view::input(c, p, project, &q.args),
        "dispatch_context" => {
            if p.role != "operator" {
                return Err("authorization: operator required".into());
            }
            let attempt = text(&q.args, "attempt")?;
            let paused: bool = db(c.query_row(
                "SELECT paused FROM projects WHERE id=?1",
                [project],
                |r| r.get(0),
            ))?;
            if paused {
                return Err("dispatch paused".into());
            }
            let (epoch,state,cancel):(i64,String,bool)=db(c.query_row("SELECT scope_epoch,state,cancel_requested FROM attempts WHERE project=?1 AND id=?2",params![project,attempt],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?;
            if state == "exited" || cancel {
                return Err("dispatch grant revoked".into());
            }
            let receipts = rows(
                c,
                "SELECT response FROM receipts WHERE project=?1 ORDER BY rowid",
                &[&project],
            )?;
            for receipt in receipts.as_array().unwrap() {
                let response: Value = serde_json::from_str(receipt["response"].as_str().unwrap())
                    .map_err(|e| e.to_string())?;
                let grant = &response["result"];
                if grant["attempt"].as_str() == Some(attempt)
                    && grant["scope_epoch"].as_i64() == Some(epoch)
                    && grant["token"].is_string()
                {
                    return Ok(grant.clone());
                }
            }
            Err("retained dispatch grant missing".into())
        }
        "runtime_context" => {
            if p.role != "operator" {
                return Err("authorization: operator required".into());
            }
            let requested = q.args.get("attempt").and_then(Value::as_str);
            let project_row = rows(
                c,
                "SELECT id,base,revision,lead_epoch,paused FROM projects WHERE id=?1",
                &[&project],
            )?;
            let project_row = project_row
                .as_array()
                .unwrap()
                .first()
                .ok_or("runtime project missing")?;
            let attempts = rows(
                c,
                "SELECT a.id,a.task,a.brief,a.base,a.state,a.scope_epoch,a.cancel_requested,a.observation FROM attempts a JOIN slots s ON s.attempt=a.id WHERE a.project=?1 AND (?2 IS NULL OR a.id=?2) ORDER BY s.id LIMIT 2",
                &[&project, &requested],
            )?;
            if requested.is_some() && attempts.as_array().unwrap().len() != 1 {
                return Err("runtime attempt is no longer a current occupied slot".into());
            }
            let mut operations = Vec::new();
            for attempt in attempts.as_array().unwrap() {
                let journals = rows(
                    c,
                    "SELECT id,project,attempt,kind,state,payload,observation FROM operations WHERE attempt=?1 AND kind IN ('start','cancel') ORDER BY rowid LIMIT 3",
                    &[&attempt["id"].as_str()],
                )?;
                let journals = journals.as_array().unwrap();
                if journals.iter().any(|o| o["project"] != project)
                    || journals.iter().filter(|o| o["kind"] == "start").count() != 1
                    || journals.iter().filter(|o| o["kind"] == "cancel").count() > 1
                    || (attempt["cancel_requested"] == true
                        && !journals.iter().any(|o| o["kind"] == "cancel"))
                {
                    return Err("runtime start/cancel journal missing or ambiguous".into());
                }
                operations.extend(journals.iter().cloned());
            }
            Ok(
                json!({"project":project_row,"revision":project_row["revision"],"attempts":attempts,"operations":operations}),
            )
        }
        "pending_operations" => {
            if p.role != "operator" {
                return Err("authorization: operator required".into());
            }
            rows(
                c,
                "SELECT id,attempt,kind,state,payload,observation FROM operations WHERE project=?1 AND state IN ('prepared','applying','unknown') ORDER BY rowid",
                &[&project],
            )
        }
        "snapshot" => crate::view::snapshot(c, p, project, &q.args),
        _ => Err(format!("unknown query: {}", q.kind)),
    }
}
