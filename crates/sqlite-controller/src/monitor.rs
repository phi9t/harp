//! Incremental visible-input receipts. Analysis is advisory and runs outside the writer.
use crate::*;
use serde::{Deserialize, Serialize};
#[cfg(test)]
thread_local! { static RETENTION_SHORT_WRITE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) }; }

fn retain_input(path: &Path, raw: &[u8]) -> Result<()> {
    use std::{
        io::Write,
        os::unix::fs::{MetadataExt, OpenOptionsExt},
    };
    if std::fs::symlink_metadata(path).is_ok() {
        if retained_bytes(path, 1_048_576)? != raw {
            return Err("retained visible input digest collision/change".into());
        }
        return Ok(());
    }
    let parent = path.parent().ok_or("visible input parent missing")?;
    let stage = parent.join(format!(".pending-{}", id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&stage)
        .map_err(|e| e.to_string())?;
    let owned = file.metadata().map_err(|e| e.to_string())?;
    let result = (|| {
        #[cfg(test)]
        {
            let fault = RETENTION_SHORT_WRITE.with(|fault| fault.replace(0));
            if fault != 0 {
                file.write_all(&raw[..raw.len() / 2])
                    .map_err(|e| e.to_string())?;
                file.sync_all().map_err(|e| e.to_string())?;
                if fault == 2 {
                    std::process::exit(86);
                }
                return Err("declared owned short-write failure".into());
            }
        }
        file.write_all(raw).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        let from = std::ffi::CString::new(stage.as_os_str().as_encoded_bytes())
            .map_err(|e| e.to_string())?;
        let to = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(|e| e.to_string())?;
        // Atomic no-replace publication never exposes partial bytes or replaces
        // unknown final content. Concurrent identical publishers share one artifact.
        if unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                from.as_ptr(),
                libc::AT_FDCWD,
                to.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        } != 0
        {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::EEXIST) {
                return Err(error.to_string());
            }
            if retained_bytes(path, 1_048_576)? != raw {
                return Err("retained visible input digest collision/change".into());
            }
        }
        std::fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    // Only this invocation's private stage identity may be removed. A crash
    // leaves an unreferenced stage; reingestion does not trust or remove it.
    if let Ok(named) = std::fs::symlink_metadata(&stage)
        && named.is_file()
        && named.dev() == owned.dev()
        && named.ino() == owned.ino()
    {
        std::fs::remove_file(&stage).map_err(|e| e.to_string())?;
        std::fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|e| e.to_string())?;
    }
    result
}

fn retained_bytes(path: &std::path::Path, limit: u64) -> Result<Vec<u8>> {
    use std::{
        io::Read,
        os::unix::fs::{MetadataExt, OpenOptionsExt},
    };
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|e| format!("retained input unavailable; coverage unavailable: {e}"))?;
    let before = file.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() || before.nlink() != 1 || before.len() > limit {
        return Err("retained input type/link/size refused; coverage unavailable".into());
    }
    crate::git_workspace::directory(path.parent().ok_or("retained input parent missing")?)?;
    let identity = |m: &std::fs::Metadata| {
        (
            m.dev(),
            m.ino(),
            m.nlink(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    };
    let named = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !named.is_file() || identity(&named) != identity(&before) {
        return Err("retained input identity changed; coverage unavailable".into());
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let after = file.metadata().map_err(|e| e.to_string())?;
    let named_after = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit
        || !named_after.is_file()
        || identity(&before) != identity(&after)
        || identity(&after) != identity(&named_after)
    {
        return Err("retained input changed or exceeded byte bound; coverage unavailable".into());
    }
    Ok(bytes)
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct VisibleEvent {
    item: String,
    turn: String,
    kind: String,
    text: String,
    ordinal: u64,
    byte_end: u64,
    digest: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Step {
    item: String,
    criterion: String,
    #[serde(default)]
    hypothesis: Option<String>,
    rationale: String,
    observation: String,
    assessment: String,
    next_test: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Finding {
    issue: String,
    criterion: String,
    text: String,
    items: Vec<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Assessment {
    batch: String,
    analyst: String,
    version: String,
    steps: Vec<Step>,
    findings: Vec<Finding>,
    evidence: Vec<String>,
}
fn decode<T: serde::de::DeserializeOwned>(a: &Value) -> Result<T> {
    serde_json::from_value(a.clone()).map_err(|e| format!("monitor schema: {e}"))
}
fn number(a: &Value, k: &str) -> Result<i64> {
    a[k].as_u64()
        .and_then(|n| i64::try_from(n).ok())
        .ok_or_else(|| format!("monitor {k} required"))
}
fn own_stream(c: &Connection, p: &Principal, project: &str, stream: &str) -> Result<()> {
    let owner: (String, String) = db(c.query_row(
        "SELECT project,attempt FROM monitor_streams WHERE id=?1",
        [stream],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    if owner.0 != project || p.attempt.as_ref().is_some_and(|a| a != &owner.1) {
        return Err("monitor stream scope refused".into());
    }
    Ok(())
}
fn current(c: &Connection, attempt: &str, epoch: i64) -> Result<()> {
    let valid:bool=db(c.query_row("SELECT EXISTS(SELECT 1 FROM attempts WHERE id=?1 AND scope_epoch=?2 AND cancel_requested=0 AND state<>'exited')",params![attempt,epoch],|r|r.get(0)))?;
    if !valid {
        return Err("monitor target authority superseded".into());
    }
    Ok(())
}

// The adapter supplies typed history only after validating the owned runtime binding.
// Filesystem retention precedes the short writer transaction. Native reasoning items
// and unfinished tool calls are excluded; neither source activity nor ingestion is credit.
pub(super) fn ingest_native(
    controller: &Controller,
    binding: &str,
    document: &Value,
    runtime_secrets: &[String],
) -> Result<()> {
    let (project, role, attempt, epoch, secrets) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let (project, role, attempt, epoch): (String, String, Option<String>, i64) = db(c
            .query_row(
                "SELECT project,role,attempt,authority_epoch FROM codex_bindings WHERE id=?1",
                [binding],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            ))?;
        if !matches!(role.as_str(), "attempt" | "lead") {
            return Ok(());
        }
        // Revocation stops ingestion under the old authority without claiming process exit.
        if role == "attempt"
            && current(
                &c,
                attempt.as_deref().ok_or("native monitor attempt missing")?,
                epoch,
            )
            .is_err()
        {
            return Ok(());
        }
        if role == "lead"
            && !db(c.query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id=?1 AND lead_epoch=?2)",
                params![project, epoch],
                |r| r.get::<_, bool>(0),
            ))?
        {
            return Ok(());
        }
        let mut secrets = runtime_secrets.to_vec();
        let rows = query::rows(
            &c,
            "SELECT response FROM receipts WHERE project=?1",
            &[&project],
        )?;
        for row in rows.as_array().unwrap() {
            let value: Value = serde_json::from_str(row["response"].as_str().unwrap())
                .map_err(|e| e.to_string())?;
            collect_secrets(&value, &mut secrets);
        }
        for name in [
            "CORENIUS_OPERATOR_TOKEN",
            "CORENIUS_CLIENT_TOKEN",
            "CORENIUS_UI_TOKEN",
        ] {
            if let Ok(value) = std::env::var(name) {
                secrets.push(value);
            }
        }
        (project, role, attempt, epoch, secrets)
    };
    let source = json!({"deployment":text(document,"server_id")?,"store":text(document,"server_store")?,"thread":text(document,"thread_id")?,"segment":text(document,"server_incarnation")?,"parser":"codex-0.159.3-typed-visible-v1","identity":hash(&format!("{binding}:{epoch}"))});
    let mut retained = Vec::new();
    let mut gap = None;
    for turn in document["native_turns"].as_array().into_iter().flatten() {
        let Some(turn_id) = turn["id"].as_str() else {
            continue;
        };
        for item in turn["items"].as_array().into_iter().flatten() {
            let kind = match item["type"].as_str() {
                Some("agentMessage" | "userMessage") => "public_message",
                Some("commandExecution" | "fileChange" | "mcpToolCall" | "dynamicToolCall")
                    if item["status"] == "completed" || item["status"] == "failed" =>
                {
                    "tool_completed"
                }
                Some("contextCompaction") => "compaction",
                _ => continue,
            };
            let Some(item_id) = item["id"].as_str() else {
                gap = Some("native visible item missing stable identity".to_string());
                continue;
            };
            let identity = format!("{turn_id}:{item_id}");
            let mut visible = item.clone();
            redact(&mut visible, &secrets);
            let raw = visible.to_string();
            if raw.len() > 1_048_576 {
                gap = Some(format!(
                    "visible item {identity} exceeds retention bound; coverage incomplete"
                ));
                continue;
            }
            let digest = hash(&raw);
            let path = controller.visible_inputs.join(&digest);
            retain_input(&path, raw.as_bytes())?;
            retained.push((
                identity,
                turn_id.to_owned(),
                kind.to_owned(),
                raw,
                digest,
                path,
                hash(&item.to_string()),
            ));
        }
    }
    std::fs::File::open(&controller.visible_inputs)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| format!("visible input directory sync: {e}"))?;
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    if role == "lead" {
        if !db(tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM projects WHERE id=?1 AND lead_epoch=?2)",
            params![project, epoch],
            |r| r.get::<_, bool>(0),
        ))? {
            return Ok(());
        }
        db(tx.execute("INSERT OR IGNORE INTO control_lead_streams(binding,project,epoch,source) VALUES(?1,?2,?3,?4)",params![binding,project,epoch,source.to_string()]))?;
        for (item, turn, kind, raw, digest, path, original_digest) in retained {
            let prior: Option<String> = db(tx
                .query_row(
                    "SELECT artifact FROM control_lead_inputs WHERE binding=?1 AND item=?2",
                    params![binding, item],
                    |r| r.get(0),
                )
                .optional())?;
            if let Some(prior) = prior {
                if prior != digest {
                    gap = Some(format!(
                        "retained lead item {item} changed; coverage incomplete"
                    ));
                }
                continue;
            }
            db(tx.execute(
                "INSERT OR IGNORE INTO artifact_manifests VALUES(?1,?2,?3)",
                params![digest, path.to_string_lossy(), raw.len() as i64],
            ))?;
            let redaction = json!({"original_source_digest":original_digest,"redacted_input_digest":digest,"policy":"typed-visible-v1; known capabilities, credential fields, bearer values redacted; reasoning excluded","retention":"complete allowed input; no automatic expiry"});
            db(tx.execute("INSERT INTO control_lead_inputs(binding,item,turn,kind,source,artifact,bytes,redaction) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![binding,item,turn,kind,source.to_string(),digest,raw.len() as i64,redaction.to_string()]))?;
        }
        db(tx.execute("UPDATE control_lead_streams SET ingested=(SELECT COUNT(*) FROM control_lead_inputs WHERE binding=?1),gap=COALESCE(?2,gap) WHERE binding=?1",params![binding,gap]))?;
        db(tx.commit())?;
        return Ok(());
    }
    let attempt = attempt.ok_or("native monitor attempt missing")?;
    if current(&tx, &attempt, epoch).is_err() {
        return Ok(());
    }
    let source_raw = source.to_string();
    let existing:Option<String>=db(tx.query_row("SELECT id FROM monitor_streams WHERE project=?1 AND attempt=?2 AND epoch=?3 AND source=?4",params![project,attempt,epoch,source_raw],|r|r.get(0)).optional())?;
    let stream = if let Some(stream) = existing {
        stream
    } else {
        let stream = id();
        db(tx.execute("INSERT INTO monitor_streams(id,project,attempt,epoch,source,segment) VALUES(?1,?2,?3,?4,?5,?6)",params![stream,project,attempt,epoch,source_raw,source["segment"].as_str()]))?;
        stream
    };
    let (mut ordinal, mut bytes): (i64, i64) = db(tx.query_row(
        "SELECT ingested,byte_offset FROM monitor_streams WHERE id=?1",
        [&stream],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    let mut previous = None;
    for (item, turn, kind, raw, digest, path, original_digest) in retained {
        let prior: Option<String> = db(tx
            .query_row(
                "SELECT artifact FROM monitor_native_inputs WHERE stream=?1 AND item=?2",
                params![stream, item],
                |r| r.get(0),
            )
            .optional())?;
        if let Some(prior) = prior {
            if prior != digest {
                gap = Some(format!(
                    "retained native item {item} changed; explicit reconciliation required"
                ));
            }
            continue;
        }
        ordinal += 1;
        bytes = bytes
            .checked_add(i64::try_from(raw.len()).map_err(|_| "visible input byte count")?)
            .ok_or("visible input offset overflow")?;
        let text = if raw.len() <= 32_768 {
            raw.clone()
        } else {
            json!({"complete_visible_input_artifact":digest,"bytes":raw.len(),"presentation":"retrieve monitor_input; full input retained without truncation"}).to_string()
        };
        let event = VisibleEvent {
            item: item.clone(),
            turn,
            kind,
            text,
            ordinal: u64::try_from(ordinal).map_err(|_| "visible ordinal")?,
            byte_end: u64::try_from(bytes).map_err(|_| "visible offset")?,
            digest: digest.clone(),
        };
        let event_raw = serde_json::to_string(&event).map_err(|e| e.to_string())?;
        db(tx.execute(
            "INSERT OR IGNORE INTO artifact_manifests VALUES(?1,?2,?3)",
            params![digest, path.to_string_lossy(), raw.len() as i64],
        ))?;
        db(tx.execute(
            "INSERT INTO monitor_events VALUES(?1,?2,?3,?4,?5)",
            params![stream, item, ordinal, event_raw, hash(&event_raw)],
        ))?;
        db(tx.execute("INSERT INTO monitor_native_inputs VALUES(?1,?2,?3,?4)",params![stream,item,digest,json!({"original_source_digest":original_digest,"redacted_input_digest":digest,"policy":"typed-visible-v1; known capabilities, credential fields, bearer values redacted; reasoning excluded","retention":"complete allowed input; no automatic expiry"}).to_string()]))?;
        previous = Some(digest);
    }
    db(tx.execute("UPDATE monitor_streams SET last_ingested_at=CASE WHEN ingested<?2 THEN CURRENT_TIMESTAMP ELSE last_ingested_at END,ingested=?2,byte_offset=?3,previous_digest=COALESCE(?4,previous_digest),gap=COALESCE(?5,gap) WHERE id=?1",params![stream,ordinal,bytes,previous,gap]))?;
    db(tx.commit())?;
    Ok(())
}
fn collect_secrets(value: &Value, secrets: &mut Vec<String>) {
    match value {
        Value::Object(o) => {
            for (k, v) in o {
                if k == "token" {
                    if let Some(s) = v.as_str() {
                        secrets.push(s.to_string());
                    }
                } else {
                    collect_secrets(v, secrets);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                collect_secrets(v, secrets);
            }
        }
        _ => {}
    }
}
fn redact(value: &mut Value, secrets: &[String]) {
    match value {
        Value::Object(o) => {
            for (k, v) in o {
                if matches!(
                    k.as_str(),
                    "token"
                        | "auth"
                        | "authorization"
                        | "authBearerToken"
                        | "capability"
                        | "api_key"
                ) {
                    *v = json!("[credential-redacted]");
                } else {
                    redact(v, secrets);
                }
            }
        }
        Value::Array(a) => {
            for v in a {
                redact(v, secrets);
            }
        }
        Value::String(s) => {
            for secret in secrets {
                if !secret.is_empty() {
                    *s = s.replace(secret, "[credential-redacted]");
                }
            }
            while let Some(start) = s.find("Bearer ") {
                let end = s[start + 7..]
                    .find(|c: char| c.is_whitespace() || matches!(c, '\"' | '\'' | '\\'))
                    .map_or(s.len(), |n| start + 7 + n);
                s.replace_range(start..end, "[bearer-redacted]");
            }
            if s.contains("BEGIN PRIVATE KEY") {
                *s = "[private-key-redacted]".into();
            }
        }
        _ => {}
    }
}

pub(super) fn input(controller: &Controller, request: Value) -> Result<Value> {
    let request: Request = serde_json::from_value(request).map_err(|e| e.to_string())?;
    if request.command.is_some() {
        return Err("monitor input query only".into());
    }
    let query = request.query.ok_or("monitor input query missing")?;
    let public_history = query.kind == "runtime_public_input";
    let args = query.args;
    let item = text(&args, "item")?;
    let (path, digest, bytes, redaction) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &request.auth, &request.project)?;
        let selector = if public_history {
            if !matches!(p.role.as_str(), "operator" | "lead" | "attempt") {
                return Err("authorization: public runtime input scope refused".into());
            }
            if args["binding"].is_string() {
                text(&args, "binding")?
            } else {
                text(&args, "stream")?
            }
        } else {
            text(&args, "batch")?
        };
        let sql = if public_history && args["binding"].is_string() {
            if p.role == "attempt" {
                return Err("authorization: retained lead input scope refused".into());
            }
            "SELECT a.path,a.digest,a.bytes,n.redaction FROM control_lead_inputs n JOIN control_lead_streams s ON s.binding=n.binding JOIN artifact_manifests a ON a.digest=n.artifact WHERE s.binding=?1 AND n.item=?2 AND s.project=?3 AND ?4 IS NULL"
        } else if public_history {
            "SELECT a.path,a.digest,a.bytes,n.redaction FROM monitor_native_inputs n JOIN monitor_streams s ON s.id=n.stream JOIN artifact_manifests a ON a.digest=n.artifact WHERE s.id=?1 AND n.item=?2 AND s.project=?3 AND (?4 IS NULL OR s.attempt=?4)"
        } else {
            "SELECT a.path,a.digest,a.bytes,n.redaction FROM monitor_batches b JOIN monitor_streams s ON s.id=b.stream JOIN monitor_events e ON e.stream=s.id AND e.ordinal>b.predecessor AND e.ordinal<=b.through JOIN monitor_native_inputs n ON n.stream=e.stream AND n.item=e.item JOIN artifact_manifests a ON a.digest=n.artifact WHERE b.id=?1 AND e.item=?2 AND s.project=?3 AND (?4 IS NULL OR s.attempt=?4)"
        };
        db(c.query_row(
            sql,
            params![selector, item, request.project, p.attempt],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        ))?
    };
    if public_history && (args["sha256"] != digest || args["bytes"].as_i64() != Some(bytes)) {
        return Err("public runtime input exact reference changed".into());
    }
    if !(0..=1_048_576).contains(&bytes) {
        return Err("retained monitor input byte bound; coverage unavailable".into());
    }
    let raw = retained_bytes(std::path::Path::new(&path), 1_048_576)?;
    if raw.len() as i64 != bytes || format!("{:x}", Sha256::digest(&raw)) != digest {
        return Err("retained monitor input changed; coverage unavailable".into());
    }
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    authenticate(&c, &request.auth, &request.project)?;
    Ok(
        json!({"digest":digest,"bytes":bytes,"redaction":serde_json::from_str::<Value>(&redaction).map_err(|e|e.to_string())?,"input":serde_json::from_slice::<Value>(&raw).map_err(|e|e.to_string())?}),
    )
}
pub(super) fn apply(
    tx: &Transaction<'_>,
    p: &Principal,
    project: &str,
    kind: &str,
    a: &Value,
) -> Result<Value> {
    match kind {
        "create_monitor_observer" => {
            let principal = format!("observer:{}", id());
            let token = id();
            db(tx.execute("INSERT INTO principals(id,token_hash,project,role,epoch) VALUES(?1,?2,?3,'lead',1)",params![principal,hash(&token),project]))?;
            db(tx.execute(
                "INSERT INTO monitor_observers VALUES(?1,?2,1)",
                params![principal, text(a, "name")?],
            ))?;
            Ok(
                json!({"principal":principal,"token":token,"role":"observer","scope":"read monitoring and publish advisory typed assessments only"}),
            )
        }
        "register_monitor_stream" => {
            let attempt = text(a, "attempt")?;
            require_attempt(tx, project, attempt)?;
            let epoch: i64 = db(tx.query_row(
                "SELECT scope_epoch FROM attempts WHERE id=?1",
                [attempt],
                |r| r.get(0),
            ))?;
            current(tx, attempt, epoch)?;
            let source = &a["source"];
            for field in [
                "deployment",
                "store",
                "thread",
                "segment",
                "parser",
                "identity",
            ] {
                text(source, field)?;
            }
            if source.as_object().is_none_or(|o| o.len() != 6) {
                return Err("monitor source fields refused".into());
            }
            let source_raw = source.to_string();
            let existing:Option<String>=db(tx.query_row("SELECT id FROM monitor_streams WHERE project=?1 AND attempt=?2 AND epoch=?3 AND source=?4",params![project,attempt,epoch,source_raw],|r|r.get(0)).optional())?;
            if let Some(stream) = existing {
                return Ok(json!({"stream":stream,"existing":true}));
            }
            let stream = id();
            let live = number(a, "live_baseline")?;
            db(tx.execute("INSERT INTO monitor_streams(id,project,attempt,epoch,source,reviewed_live,segment) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![stream,project,attempt,epoch,source_raw,live,text(source,"segment")?]))?;
            Ok(
                json!({"stream":stream,"historical_frontier":0,"live_baseline":live,"history_uncovered":live>0}),
            )
        }
        "mark_monitor_gap" => {
            let stream = text(a, "stream")?;
            own_stream(tx, p, project, stream)?;
            db(tx.execute(
                "UPDATE monitor_streams SET gap=?2 WHERE id=?1",
                params![stream, text(a, "reason")?],
            ))?;
            Ok(json!({"stream":stream,"state":"source-gap; cursor unchanged"}))
        }
        "ingest_monitor_events" => {
            let stream = text(a, "stream")?;
            own_stream(tx, p, project, stream)?;
            let (mut cursor,mut offset,mut previous,gap):(i64,i64,String,Option<String>)=db(tx.query_row("SELECT ingested,byte_offset,previous_digest,gap FROM monitor_streams WHERE id=?1",[stream],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))))?;
            if gap.is_some() {
                return Err("monitor source gap requires reconciliation".into());
            }
            if number(a, "expected_ordinal")? != cursor
                || a["previous_digest"].as_str() != Some(previous.as_str())
            {
                return Err("monitor cursor/prefix mismatch; source gap".into());
            }
            let events: Vec<VisibleEvent> = decode(&a["events"])?;
            if events.len() > 200 || a.to_string().len() > 524_288 {
                return Err("monitor ingestion bound".into());
            }
            for e in events {
                if e.item.is_empty()
                    || e.turn.is_empty()
                    || e.text.len() > 32_768
                    || e.digest.is_empty()
                    || !matches!(
                        e.kind.as_str(),
                        "tool_completed"
                            | "public_message"
                            | "peer_message"
                            | "compaction"
                            | "handoff"
                            | "pending_observation"
                            | "turn_completed"
                    )
                {
                    return Err("monitor visible-item schema/private reasoning refused".into());
                }
                if e.text.contains("Bearer ") || e.text.contains("BEGIN PRIVATE KEY") {
                    return Err("monitor credential input must be redacted before ingestion".into());
                }
                let raw = serde_json::to_string(&e).map_err(|e| e.to_string())?;
                let digest = hash(&raw);
                let duplicate: Option<String> = db(tx
                    .query_row(
                        "SELECT input_digest FROM monitor_events WHERE stream=?1 AND item=?2",
                        params![stream, e.item],
                        |r| r.get(0),
                    )
                    .optional())?;
                if let Some(prior) = duplicate {
                    if prior != digest {
                        return Err("canonical item changed; source gap".into());
                    }
                    continue;
                }
                if i64::try_from(e.ordinal).ok() != cursor.checked_add(1)
                    || i64::try_from(e.byte_end)
                        .ok()
                        .is_none_or(|end| end <= offset)
                {
                    return Err("noncontiguous complete input; source gap".into());
                }
                cursor += 1;
                offset = i64::try_from(e.byte_end).map_err(|_| "monitor byte offset")?;
                previous = e.digest;
                db(tx.execute(
                    "INSERT INTO monitor_events VALUES(?1,?2,?3,?4,?5)",
                    params![stream, e.item, cursor, raw, digest],
                ))?;
            }
            db(tx.execute("UPDATE monitor_streams SET last_ingested_at=CASE WHEN ingested<?2 THEN CURRENT_TIMESTAMP ELSE last_ingested_at END,ingested=?2,byte_offset=?3,previous_digest=?4 WHERE id=?1",params![stream,cursor,offset,previous]))?;
            Ok(
                json!({"stream":stream,"ingested":cursor,"byte_offset":offset,"previous_digest":previous}),
            )
        }
        "seal_monitor_batch" => {
            let stream = text(a, "stream")?;
            own_stream(tx, p, project, stream)?;
            let lane = text(a, "lane")?;
            if !matches!(lane, "history" | "live") {
                return Err("monitor lane refused".into());
            }
            let (ingested,history,live,attempt):(i64,i64,i64,String)=db(tx.query_row("SELECT ingested,reviewed_history,reviewed_live,attempt FROM monitor_streams WHERE id=?1",[stream],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))))?;
            let predecessor = if lane == "history" { history } else { live };
            let through = number(a, "through")?;
            if through <= predecessor || through > ingested || through - predecessor > 20 {
                return Err("monitor immutable batch range bound (1..20)".into());
            }
            let boundary = text(a, "boundary")?;
            if !matches!(
                boundary,
                "20-actions"
                    | "5-turns"
                    | "10-minutes"
                    | "compaction"
                    | "assignment"
                    | "handoff"
                    | "manual"
            ) {
                return Err("monitor boundary refused".into());
            }
            let existing:Option<String>=db(tx.query_row("SELECT id FROM monitor_batches WHERE stream=?1 AND lane=?2 AND predecessor=?3 AND through=?4",params![stream,lane,predecessor,through],|r|r.get(0)).optional())?;
            if let Some(batch) = existing {
                return Ok(json!({"batch":batch,"existing":true}));
            }
            let rows = crate::query::rows(
                tx,
                "SELECT document FROM monitor_events WHERE stream=?1 AND ordinal>?2 AND ordinal<=?3 ORDER BY ordinal",
                &[&stream, &predecessor, &through],
            )?;
            let inputs: Vec<Value> = rows
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    serde_json::from_str(r["document"].as_str().unwrap()).map_err(|e| e.to_string())
                })
                .collect::<Result<_>>()?;
            if inputs.len()
                != usize::try_from(through - predecessor).map_err(|_| "monitor range")?
            {
                return Err("monitor coverage hole".into());
            }
            let intent:(String,String,String)=db(tx.query_row("SELECT a.brief,b.digest,b.body FROM attempts a JOIN briefs b ON b.id=a.brief WHERE a.id=?1",[attempt],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?;
            let batch = id();
            let raw = json!(inputs).to_string();
            let digest = hash(&raw);
            db(tx.execute("INSERT INTO monitor_batches(id,stream,lane,predecessor,through,input_digest,inputs,intent,boundary) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![batch,stream,lane,predecessor,through,digest,raw,json!({"brief":intent.0,"digest":intent.1,"body":intent.2}).to_string(),boundary]))?;
            Ok(
                json!({"batch":batch,"input_digest":digest,"state":"review-pending","analysis_authority":"advisory only"}),
            )
        }
        "publish_monitor_assessment" => {
            let report: Assessment = decode(a)?;
            let report_raw = a.to_string();
            if report.analyst.is_empty()
                || report.version.is_empty()
                || a.to_string().len() > 131_072
            {
                return Err("monitor report bound".into());
            }
            let (stream,lane,predecessor,through,raw,state):(String,String,i64,i64,String,String)=db(tx.query_row("SELECT stream,lane,predecessor,through,inputs,state FROM monitor_batches WHERE id=?1",[&report.batch],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))))?;
            own_stream(tx, p, project, &stream)?;
            let prior:Option<(String,String)>=db(tx.query_row("SELECT id,report FROM monitor_assessments WHERE batch=?1 AND analyst=?2 AND version=?3",params![report.batch,report.analyst,report.version],|r|Ok((r.get(0)?,r.get(1)?))).optional())?;
            if let Some((assessment, stored)) = prior {
                if stored != report_raw {
                    return Err("assessment version arguments changed".into());
                }
                return Ok(json!({"assessment":assessment,"state":"published","replayed":true}));
            }
            if !matches!(state.as_str(), "review-pending" | "reviewed") {
                return Err("batch state refuses assessment publication".into());
            }
            let inputs: Vec<VisibleEvent> =
                serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            let items: std::collections::BTreeSet<_> =
                inputs.iter().map(|e| e.item.as_str()).collect();
            let reviewed: std::collections::BTreeSet<_> =
                report.steps.iter().map(|s| s.item.as_str()).collect();
            if reviewed != items || reviewed.len() != report.steps.len() {
                return Err("every sealed item requires one explicit step disposition".into());
            }
            for s in &report.steps {
                if s.criterion.is_empty()
                    || s.rationale.is_empty()
                    || s.observation.is_empty()
                    || s.next_test.is_empty()
                    || !matches!(
                        s.assessment.as_str(),
                        "progress"
                            | "valid_learning"
                            | "necessary_setup"
                            | "waiting"
                            | "repetition"
                            | "scope_concern"
                            | "contradicted_claim"
                            | "insufficient_evidence"
                    )
                {
                    return Err("monitor step disposition schema".into());
                }
            }
            for f in &report.findings {
                if f.issue.is_empty()
                    || f.criterion.is_empty()
                    || f.text.is_empty()
                    || f.items.is_empty()
                    || f.items.iter().any(|i| !items.contains(i.as_str()))
                {
                    return Err("finding lacks cited immutable input".into());
                }
            }
            // Only artifacts already persisted by the trusted controller can receive credit.
            for evidence in &report.evidence {
                let present:bool=db(tx.query_row("SELECT EXISTS(SELECT 1 FROM artifact_manifests m JOIN verification_reports r ON r.artifact_digest=m.digest JOIN candidates c ON c.id=r.candidate JOIN attempts a ON a.id=c.attempt WHERE m.digest=?1 AND a.project=?2 AND r.verdict='pass')",params![evidence,project],|r|r.get(0)))?;
                if !present {
                    return Err("unbound/unverified monitor evidence; no progress credit".into());
                }
            }
            let cursor: i64 = db(tx.query_row(
                if lane == "history" {
                    "SELECT reviewed_history FROM monitor_streams WHERE id=?1"
                } else {
                    "SELECT reviewed_live FROM monitor_streams WHERE id=?1"
                },
                [&stream],
                |r| r.get(0),
            ))?;
            if state == "review-pending" && cursor != predecessor {
                return Err("monitor review predecessor conflict".into());
            }
            if state == "reviewed" && cursor < through {
                return Err("monitor rerun lacks original committed coverage".into());
            }
            let assessment = id();
            db(tx.execute(
                "INSERT INTO monitor_assessments VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    assessment,
                    report.batch,
                    report.analyst,
                    report.version,
                    a.to_string(),
                    hash(&a.to_string())
                ],
            ))?;
            let (attempt, epoch): (String, i64) = db(tx.query_row(
                "SELECT attempt,epoch FROM monitor_streams WHERE id=?1",
                [&stream],
                |r| Ok((r.get(0)?, r.get(1)?)),
            ))?;
            for finding in &report.findings {
                db(tx.execute("INSERT OR IGNORE INTO monitor_findings(id,project,attempt,epoch,issue,criterion,document,assessment) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![id(),project,attempt,epoch,finding.issue,finding.criterion,serde_json::to_string(finding).map_err(|e|e.to_string())?,assessment]))?;
            }
            for s in &report.steps {
                if s.assessment == "progress" {
                    for evidence in &report.evidence {
                        db(tx.execute(
                            "INSERT OR IGNORE INTO monitor_credits VALUES(?1,?2,?3,?4)",
                            params![project, s.criterion, evidence, assessment],
                        ))?;
                    }
                }
            }
            if state == "review-pending" {
                db(tx.execute(if lane=="history"{"UPDATE monitor_streams SET reviewed_history=?2 WHERE id=?1 AND reviewed_history=?3"}else{"UPDATE monitor_streams SET reviewed_live=?2 WHERE id=?1 AND reviewed_live=?3"},params![stream,through,predecessor]))?;
            }
            db(tx.execute(
                "UPDATE monitor_batches SET state='reviewed' WHERE id=?1",
                [&report.batch],
            ))?;
            Ok(
                json!({"assessment":assessment,"batch":report.batch,"state":"published","authority":"advisory; no execution command admitted"}),
            )
        }
        "adjudicate_monitor_finding" => {
            let finding = text(a, "finding")?;
            let decision = text(a, "decision")?;
            if !matches!(decision, "accepted" | "dismissed") {
                return Err("finding decision refused".into());
            }
            let n=db(tx.execute("UPDATE monitor_findings SET disposition=?3,adjudication=?4 WHERE id=?1 AND project=?2",params![finding,project,decision,json!({"principal":p.id,"epoch":p.epoch,"rationale":text(a,"rationale")?}).to_string()]))?;
            if n != 1 {
                return Err("finding missing".into());
            }
            Ok(json!({"finding":finding,"decision":decision}))
        }
        "authorize_correction" => {
            let finding = text(a, "finding")?;
            let (attempt,epoch,disposition,criterion):(String,i64,String,String)=db(tx.query_row("SELECT attempt,epoch,disposition,criterion FROM monitor_findings WHERE id=?1 AND project=?2",params![finding,project],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))))?;
            current(tx, &attempt, epoch)?;
            if disposition != "accepted"
                || number(a, "expected_epoch")? != epoch
                || text(a, "criterion")? != criterion
            {
                return Err(
                    "correction requires authenticated adjudication/current intent and epoch"
                        .into(),
                );
            }
            for k in ["action", "prohibited_scope", "next_checkpoint", "rationale"] {
                text(a, k)?;
            }
            let existing: Option<(String, String, String, Option<String>, String)> = db(tx
                .query_row(
                    "SELECT m.id,m.decision,m.state,m.operation,d.author FROM monitor_corrections m JOIN decisions d ON d.id=m.decision WHERE m.finding=?1 AND m.epoch=?2",
                    params![finding, epoch],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                ).optional())?;
            if let Some((correction, old_decision, state, operation, author)) = existing {
                if correction_authority(tx, project, &author)?.is_some()
                    || !matches!(state.as_str(), "requested" | "authorization-stale")
                {
                    return Ok(json!({"correction":correction,"existing":true}));
                }
                // A fresh grant never retries an effect whose outcome is unresolved or applied.
                if let Some(operation) = &operation {
                    let outcome: Option<String> = db(tx
                        .query_row(
                            "SELECT state FROM codex_operations WHERE id=?1",
                            [operation],
                            |r| r.get(0),
                        )
                        .optional())?;
                    if outcome.as_deref() != Some("failed") {
                        return Err(
                            "correction authorization requires reconciliation of prior operation"
                                .into(),
                        );
                    }
                }
                let decision = id();
                db(tx.execute("INSERT INTO decisions VALUES(?1,?2,?3,?4)",params![decision,project,p.id,json!({"finding":finding,"authorization":a,"supersedes":old_decision,"prior_correction":{"state":state,"operation":operation,"disposition":"authorization-stale"}}).to_string()]))?;
                db(tx.execute("UPDATE monitor_corrections SET decision=?2,document=?3,state='requested',operation=NULL WHERE id=?1",params![correction,decision,a.to_string()]))?;
                return Ok(
                    json!({"correction":correction,"decision":decision,"state":"requested","supersedes":old_decision}),
                );
            }
            let correction = id();
            let decision = id();
            db(tx.execute(
                "INSERT INTO decisions VALUES(?1,?2,?3,?4)",
                params![
                    decision,
                    project,
                    p.id,
                    json!({"finding":finding,"authorization":a}).to_string()
                ],
            ))?;
            db(tx.execute("INSERT INTO monitor_corrections(id,finding,project,attempt,epoch,decision,document) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![correction,finding,project,attempt,epoch,decision,a.to_string()]))?;
            Ok(json!({"correction":correction,"decision":decision,"state":"requested"}))
        }
        "ack_correction" => {
            if p.role != "attempt" {
                return Err("correction acknowledgement requires addressed actor".into());
            }
            let correction = text(a, "correction")?;
            let (attempt, epoch, state): (String, i64, String) = db(tx.query_row(
                "SELECT attempt,epoch,state FROM monitor_corrections WHERE id=?1 AND project=?2",
                params![correction, project],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ))?;
            current(tx, &attempt, epoch)?;
            if p.attempt.as_deref() != Some(&attempt)
                || p.epoch != epoch
                || !matches!(state.as_str(), "presented" | "prepared" | "acknowledged")
            {
                return Err("correction ack target/delivery mismatch".into());
            }
            db(tx.execute("UPDATE monitor_corrections SET state='acknowledged',acknowledgement=?2 WHERE id=?1",params![correction,json!({"principal":p.id,"epoch":p.epoch,"response":text(a,"response")?}).to_string()]))?;
            Ok(json!({"correction":correction,"state":"acknowledged"}))
        }
        "resolve_correction" => {
            let correction = text(a, "correction")?;
            let valid:bool=db(tx.query_row("SELECT EXISTS(SELECT 1 FROM monitor_corrections c JOIN monitor_assessments a ON a.id=?3 JOIN monitor_batches b ON b.id=a.batch JOIN monitor_streams s ON s.id=b.stream WHERE c.id=?1 AND c.project=?2 AND s.project=c.project AND s.attempt=c.attempt AND s.epoch=c.epoch)",params![correction,project,text(a,"evidence")?],|r|r.get(0)))?;
            if !valid {
                return Err(
                    "correction resolution requires retained same-target assessment".into(),
                );
            }
            let n=db(tx.execute("UPDATE monitor_corrections SET state='resolved',resolution=?3 WHERE id=?1 AND project=?2 AND state='acknowledged'",params![correction,project,json!({"principal":p.id,"evidence":text(a,"evidence")?,"rationale":text(a,"rationale")?}).to_string()]))?;
            if n != 1 {
                return Err(
                    "correction requires authenticated acknowledgement before resolution".into(),
                );
            }
            Ok(json!({"correction":correction,"state":"resolved"}))
        }
        _ => Err(format!("unknown command: {kind}")),
    }
}

pub(super) fn projection(c: &Connection, p: &Principal, project: &str) -> Result<Value> {
    let mut result = json!({"view_limits":{"collection_byte_limit":32768,"byte_limit":200000,"item_limit":20},"continuous_runner":"not implemented/qualified; ticket06","authority":"advisory analyzer; controller authenticates adjudication and delivery"});
    for name in [
        "streams",
        "batches",
        "assessments",
        "findings",
        "corrections",
        "hypotheses",
    ] {
        let page = records(c, p, project, &json!({"collection":name}))?;
        result[name] = page["items"].clone();
        result["pagination"][name] = page["page"].clone();
    }
    Ok(result)
}

pub(super) fn records(c: &Connection, p: &Principal, project: &str, args: &Value) -> Result<Value> {
    let collection = match text(args, "collection")? {
        "streams" => "monitor_streams",
        "batches" => "monitor_batches",
        "assessments" => "monitor_assessments",
        "findings" => "monitor_findings",
        "corrections" => "monitor_corrections",
        "hypotheses" => "monitor_hypotheses",
        _ => return Err("monitor collection refused".into()),
    };
    let mut result = crate::view::page(c, p, project, collection, args)?;
    result["page"]["continuation"]["kind"] = json!("monitor_records");
    result["page"]["continuation"]["args"]["collection"] = args["collection"].clone();
    Ok(result)
}

fn publication_source(
    c: &Connection,
    p: &Principal,
    project: &str,
    report: &Assessment,
) -> Result<Value> {
    let batch = query::rows(
        c,
        "SELECT b.*,s.source,s.epoch,s.attempt FROM monitor_batches b JOIN monitor_streams s ON s.id=b.stream WHERE b.id=?1 AND s.project=?2",
        &[&report.batch, &project],
    )?;
    let batch = batch
        .as_array()
        .unwrap()
        .first()
        .ok_or("monitor batch unavailable")?;
    own_stream(c, p, project, text(batch, "stream")?)?;
    let dependencies = query::rows(
        c,
        "SELECT e.item,e.document,n.artifact,m.path,m.bytes FROM monitor_batches b JOIN monitor_events e ON e.stream=b.stream AND e.ordinal>b.predecessor AND e.ordinal<=b.through JOIN monitor_native_inputs n ON n.stream=e.stream AND n.item=e.item LEFT JOIN artifact_manifests m ON m.digest=n.artifact WHERE b.id=?1 ORDER BY e.ordinal",
        &[&report.batch],
    )?;
    let inputs: Vec<VisibleEvent> = serde_json::from_str(
        batch["inputs"]
            .as_str()
            .ok_or("sealed input document missing")?,
    )
    .map_err(|e| e.to_string())?;
    let native_source: Value =
        serde_json::from_str(batch["source"].as_str().ok_or("monitor source missing")?)
            .map_err(|e| e.to_string())?;
    let required_native = native_source["parser"] == "codex-0.159.3-typed-visible-v1";
    for input in inputs {
        let native = dependencies
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["item"] == input.item);
        let descriptor = serde_json::from_str::<Value>(&input.text)
            .ok()
            .is_some_and(|v| v.get("complete_visible_input_artifact").is_some());
        if (descriptor || required_native) && native.is_none() {
            return Err("sealed full input dependency missing; coverage unavailable".into());
        }
        if let Some(native) = native
            && (native["artifact"] != input.digest
                || !native["path"].is_string()
                || !native["bytes"].is_i64())
        {
            return Err("sealed full input provenance unavailable; no reviewed coverage".into());
        }
    }
    let mut evidence = Vec::new();
    for digest in &report.evidence {
        let rows = query::rows(
            c,
            "SELECT m.digest,m.path,m.bytes,r.id AS report,r.candidate FROM artifact_manifests m JOIN verification_reports r ON r.artifact_digest=m.digest JOIN candidates v ON v.id=r.candidate JOIN attempts a ON a.id=v.attempt WHERE m.digest=?1 AND a.project=?2 AND r.verdict='pass' ORDER BY r.rowid LIMIT 1",
            &[digest, &project],
        )?;
        evidence.push(
            rows.as_array()
                .unwrap()
                .first()
                .cloned()
                .ok_or("unbound monitor evidence")?,
        );
    }
    Ok(json!({"batch":batch,"dependencies":dependencies,"evidence":evidence}))
}

pub(super) fn bind_checked(
    c: &Connection,
    p: &Principal,
    project: &str,
    args: &Value,
    checked: Option<&Value>,
) -> Result<()> {
    let report: Assessment = decode(args)?;
    let prior: Option<String> = db(c
        .query_row(
            "SELECT report FROM monitor_assessments WHERE batch=?1 AND analyst=?2 AND version=?3",
            params![report.batch, report.analyst, report.version],
            |r| r.get(0),
        )
        .optional())?;
    if prior.as_deref() == Some(&args.to_string()) {
        return Ok(());
    }
    if checked != Some(&publication_source(c, p, project, &report)?) {
        return Err("checked sealed input/source binding changed; no reviewed coverage".into());
    }
    Ok(())
}

pub(super) fn validate_evidence(
    controller: &Controller,
    request: &Request,
) -> Result<Option<Value>> {
    let Some(command) = &request.command else {
        return Ok(None);
    };
    if command.kind != "publish_monitor_assessment" {
        return Ok(None);
    }
    if command.args.to_string().len() > 131_072 {
        return Err("monitor report bound".into());
    }
    let report: Assessment = decode(&command.args)?;
    let checked = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &request.auth, &request.project)?;
        authorize(&p, &command.kind, &command.args)?;
        let prior:Option<String>=db(c.query_row("SELECT arguments FROM receipts WHERE project=?1 AND principal=?2 AND epoch=?3 AND key=?4",params![request.project,p.id,p.epoch,command.idempotency_key],|r|r.get(0)).optional())?;
        if let Some(prior) = prior {
            let arguments=json!({"kind":command.kind,"expected_revision":command.expected_revision,"args":command.args}).to_string();
            if prior != arguments {
                return Err("idempotency argument mismatch".into());
            }
            return Ok(None);
        }
        let prior:Option<String>=db(c.query_row("SELECT report FROM monitor_assessments WHERE batch=?1 AND analyst=?2 AND version=?3",params![report.batch,report.analyst,report.version],|r|r.get(0)).optional())?;
        if prior.as_deref() == Some(&command.args.to_string()) {
            return Ok(None);
        }
        publication_source(&c, &p, &request.project, &report)?
    };
    for (collection, limit, digest_field) in [
        ("dependencies", 1_048_576, "artifact"),
        ("evidence", 25_165_824, "digest"),
    ] {
        for artifact in checked[collection].as_array().unwrap() {
            let size = artifact["bytes"]
                .as_i64()
                .ok_or("sealed artifact byte count missing")?;
            if size < 0 || size as u64 > limit {
                return Err("sealed artifact byte bound; coverage unavailable".into());
            }
            let bytes = retained_bytes(Path::new(text(artifact, "path")?), limit)?;
            if bytes.len() as i64 != size
                || format!("{:x}", Sha256::digest(&bytes)) != text(artifact, digest_field)?
            {
                return Err(
                    "sealed artifact digest/bytes changed; no reviewed coverage or credit".into(),
                );
            }
        }
    }
    Ok(Some(checked))
}

fn correction_authority(c: &Connection, project: &str, principal: &str) -> Result<Option<i64>> {
    let (role, epoch): (String, i64) = db(c.query_row(
        "SELECT role,epoch FROM principals WHERE id=?1",
        [principal],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    let lead_epoch: i64 = db(c.query_row(
        "SELECT lead_epoch FROM projects WHERE id=?1",
        [project],
        |r| r.get(0),
    ))?;
    Ok((role == "operator" || (role == "lead" && epoch == lead_epoch)).then_some(epoch))
}

pub(super) fn prepare_correction(
    tx: &Transaction<'_>,
    binding: &str,
    document: &Value,
    project: &str,
    epoch: i64,
) -> Result<()> {
    let attempt: String = db(tx.query_row(
        "SELECT attempt FROM codex_bindings WHERE id=?1",
        [binding],
        |r| r.get(0),
    ))?;
    current(tx, &attempt, epoch)?;
    let pending:i64=db(tx.query_row("SELECT COUNT(*) FROM codex_operations WHERE binding=?1 AND kind IN ('codex_turn','codex_steer','turn/start') AND state IN ('prepared','applying','unknown')",[binding],|r|r.get(0)))?;
    if pending != 0 {
        return Ok(());
    }
    let (correction, raw, principal, principal_epoch) = loop {
        let row:Option<(String,String,String,Option<String>)>=db(tx.query_row("SELECT m.id,m.document,d.author,m.operation FROM monitor_corrections m JOIN decisions d ON d.id=m.decision WHERE m.project=?1 AND m.attempt=?2 AND m.epoch=?3 AND m.state='requested' ORDER BY m.rowid LIMIT 1",params![project,attempt,epoch],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional())?;
        let Some((correction, raw, principal, operation)) = row else {
            return Ok(());
        };
        if let Some(principal_epoch) = correction_authority(tx, project, &principal)? {
            break (correction, raw, principal, principal_epoch);
        }
        if let Some(operation) = operation {
            let outcome: Option<String> = db(tx
                .query_row(
                    "SELECT state FROM codex_operations WHERE id=?1",
                    [operation],
                    |r| r.get(0),
                )
                .optional())?;
            if outcome.as_deref() != Some("failed") {
                return Ok(());
            }
        }
        // Retain the withdrawn grant, visibly disposition it, then consider the next request.
        db(tx.execute("UPDATE monitor_corrections SET state='authorization-stale' WHERE id=?1 AND state='requested'",[correction]))?;
    };
    let status = document["native_turns"]
        .as_array()
        .and_then(|ts| ts.last())
        .and_then(|t| t["status"].as_str());
    let kind = match status {
        Some("inProgress") => "codex_steer",
        Some("completed" | "interrupted" | "failed") => "codex_turn",
        _ => return Ok(()),
    };
    let operation = id();
    let text = format!(
        "CORENIUS_CORRECTION {correction}\n{raw}\nPreserve admitted acceptance. Acknowledge only through corenius_command ack_correction with this ID and a public response; transcript text is not acknowledgement."
    );
    let intent = json!({"role":"attempt","authority_epoch":epoch,"principal":principal,"principal_epoch":principal_epoch,"text":text,"expected_turn":document["turn_id"],"correction":correction});
    db(tx.execute(
        "INSERT INTO codex_operations VALUES(?1,?2,?3,?4,'prepared',?5,NULL)",
        params![operation, project, binding, kind, intent.to_string()],
    ))?;
    db(tx.execute("UPDATE monitor_corrections SET state='prepared',operation=?2 WHERE id=?1 AND state='requested'",params![correction,operation]))?;
    Ok(())
}

pub(super) fn correction_result(
    c: &Connection,
    operation: &str,
    state: &str,
    observation: &Value,
) -> Result<()> {
    let next = match state {
        "confirmed" => "presented",
        "failed" => "requested",
        _ => return Ok(()),
    };
    db(c.execute(
        "UPDATE monitor_corrections SET state=?2 WHERE operation=?1 AND state='prepared'",
        params![operation, next],
    ))?;
    // Native operation history retains presentation/refusal independently of the acknowledgement.
    let _ = observation;
    Ok(())
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use std::{
        io::Write,
        os::unix::fs::OpenOptionsExt,
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };
    fn fixture() -> (
        tempfile::TempDir,
        Arc<Controller>,
        Value,
        std::path::PathBuf,
        Vec<u8>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let controller =
            Arc::new(Controller::open(dir.path().join("state.db"), "operator-secret").unwrap());
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
            json!({"task":"a","work_ref":{"deployment":"fixture","project":"p","item":"a"},"source_revision":"r1","brief":"retained input boundary"}),
        );
        let worker = command("admit", "admit", json!({"task":"a"}));
        let document = json!({"server_id":"owned-server","server_store":"owned-store","thread_id":"owned-thread","server_incarnation":"owned-incarnation","native_turns":[{"id":"turn","items":[{"id":"item","type":"userMessage","content":[{"type":"text","text":"complete allowed fixture input synthetic-native-app-secret synthetic-native-executor-secret"}]}]}]});
        {
            let c = controller.connection.lock().unwrap();
            c.execute_batch(include_str!("codex_schema.sql")).unwrap();
            c.execute(
                "INSERT INTO codex_bindings VALUES('owned-binding','p','attempt',?1,1,?2,'bound')",
                params![worker["result"]["attempt"].as_str(), document.to_string()],
            )
            .unwrap();
        }
        ingest_native(
            &controller,
            "owned-binding",
            &document,
            &[
                "synthetic-native-app-secret".into(),
                "synthetic-native-executor-secret".into(),
            ],
        )
        .unwrap();
        let stream: String = controller
            .connection
            .lock()
            .unwrap()
            .query_row("SELECT id FROM monitor_streams", [], |r| r.get(0))
            .unwrap();
        let sealed = command(
            "seal_monitor_batch",
            "seal",
            json!({"stream":stream,"lane":"history","through":1,"boundary":"manual"}),
        );
        let request = json!({"auth":auth,"project":"p","query":{"kind":"monitor_input","args":{"batch":sealed["result"]["batch"],"item":"turn:item"}}});
        let path: String = controller
            .connection
            .lock()
            .unwrap()
            .query_row("SELECT path FROM artifact_manifests", [], |r| r.get(0))
            .unwrap();
        let path = std::path::PathBuf::from(path);
        let raw = std::fs::read(&path).unwrap();
        assert!(
            !String::from_utf8_lossy(&raw).contains("synthetic-native-app-secret")
                && !String::from_utf8_lossy(&raw).contains("synthetic-native-executor-secret")
        );
        assert!(controller.handle(request.clone()).is_ok());
        (dir, controller, request, path, raw)
    }
    #[test]
    fn public_retained_monitor_input_refuses_fifo_promptly_without_credit() {
        let (_dir, controller, request, path, raw) = fixture();
        std::fs::remove_file(&path).unwrap();
        let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        let (send, receive) = mpsc::channel();
        let reader = controller.clone();
        let started = Instant::now();
        let task = std::thread::spawn(move || send.send(reader.handle(request)).unwrap());
        let first = receive.recv_timeout(Duration::from_millis(300));
        let blocked = first.is_err();
        let result = match first {
            Ok(result) => result,
            Err(_) => {
                let mut writer = OpenOptions::new()
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(&path)
                    .unwrap();
                writer.write_all(&raw).unwrap();
                drop(writer);
                receive.recv_timeout(Duration::from_secs(2)).unwrap()
            }
        };
        task.join().unwrap();
        assert!(
            !blocked,
            "FIFO read blocked until a fixture writer supplied matching retained bytes: {result:?}"
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_millis(300));
        let c = controller.connection.lock().unwrap();
        assert_eq!(
            c.query_row("SELECT reviewed_history FROM monitor_streams", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM monitor_credits", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn retained_lead_public_inputs_exclude_private_data_and_preserve_worker_credit() {
        let (_dir, controller, _, _, _) = fixture();
        let auth = json!({"principal":"operator","token":"operator-secret"});
        let revision = controller
            .handle(json!({"auth":auth,"project":"p","query":{"kind":"snapshot","args":{}}}))
            .unwrap()["revision"]
            .clone();
        let lead=controller.handle(json!({"auth":auth,"project":"p","command":{"kind":"replace_lead","idempotency_key":"lead-visible","expected_revision":revision,"args":{}}})).unwrap()["result"].clone();
        let before = controller
            .handle(json!({"auth":auth,"project":"p","query":{"kind":"monitor_context","args":{}}}))
            .unwrap();
        let document = json!({"server_id":"retained-lead-server","server_store":"retained-lead-store","thread_id":"retained-lead-thread","server_incarnation":"retained-lead-incarnation","native_turns":[{"id":"retained-turn","items":[{"id":"private","type":"reasoning","text":"fixture-private-reasoning"},{"id":"unfinished","type":"dynamicToolCall","status":"inProgress","arguments":"unfinished"},{"id":"public","type":"agentMessage","text":"visible synthetic-runtime-capability Bearer fixture-bearer", "token":"credential-field"},{"id":"tool","type":"dynamicToolCall","status":"completed","result":"complete allowed tool result"}]}]});
        {
            let c = controller.connection.lock().unwrap();
            c.execute(
                "INSERT INTO codex_bindings VALUES('retained-lead','p','lead',NULL,?1,?2,'bound')",
                params![lead["role_epoch"].as_i64(), document.to_string()],
            )
            .unwrap();
        }
        for _ in 0..2 {
            ingest_native(
                &controller,
                "retained-lead",
                &document,
                &["synthetic-runtime-capability".into()],
            )
            .unwrap();
        }
        let page=controller.handle(json!({"auth":auth,"project":"p","query":{"kind":"control_records","args":{"collection":"lead_public_history","binding":"retained-lead"}}})).unwrap();
        assert_eq!(page["items"].as_array().unwrap().len(), 2);
        let mut public_reference = Value::Null;
        for item in page["items"].as_array().unwrap() {
            let reference = item["input_reference"].clone();
            let full = controller
                .handle(json!({"auth":auth,"project":"p","query":reference}))
                .unwrap();
            let raw = full.to_string();
            for private in [
                "fixture-private-reasoning",
                "synthetic-runtime-capability",
                "fixture-bearer",
                "credential-field",
                "unfinished",
            ] {
                assert!(
                    !raw.contains(private),
                    "private fixture data reached public input"
                );
            }
            if full["input"]["type"] == "agentMessage" {
                public_reference = item["input_reference"].clone();
            }
            assert_eq!(full["digest"], item["input_digest"]);
            assert_eq!(item["source"]["store"], "retained-lead-store");
        }
        assert!(public_reference.is_object());
        let after = controller
            .handle(json!({"auth":auth,"project":"p","query":{"kind":"monitor_context","args":{}}}))
            .unwrap();
        assert_eq!(before["streams"], after["streams"]);
        assert_eq!(before["assessments"], after["assessments"]);
        let c = controller.connection.lock().unwrap();
        assert!(
            c.execute("UPDATE control_lead_inputs SET kind='modified'", [])
                .is_err()
        );
        assert!(c.execute("DELETE FROM control_lead_inputs", []).is_err());
        drop(c);
        let lead_auth = json!({"principal":lead["principal"],"token":lead["token"]});
        assert!(
            controller
                .handle(json!({"auth":lead_auth,"project":"p","query":public_reference}))
                .is_ok()
        );
        println!(
            "actual_control_lead_read_ledger_filtered_immutable_deduplicated_without_worker_credit=true; synthetic_typed_negative_fixture=true"
        );
    }
    #[test]
    fn public_retained_monitor_input_refuses_symlink_even_with_matching_bytes() {
        let (dir, controller, request, path, raw) = fixture();
        let target = dir.path().join("replacement");
        std::fs::write(&target, raw).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(target, path).unwrap();
        assert!(
            controller.handle(request).is_err(),
            "matching bytes through a symlink do not admit a retained input"
        );
    }
    #[test]
    fn public_retained_monitor_input_refuses_oversize_replaced_and_missing_without_credit() {
        let (_dir, controller, request, path, _raw) = fixture();
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(4 * 1_048_576).unwrap();
        drop(file);
        let started = Instant::now();
        assert!(controller.handle(request.clone()).is_err());
        assert!(started.elapsed() < Duration::from_millis(300));
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, b"replacement bytes").unwrap();
        assert!(controller.handle(request.clone()).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(controller.handle(request).is_err());
        let c = controller.connection.lock().unwrap();
        assert_eq!(
            c.query_row("SELECT reviewed_history FROM monitor_streams", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM monitor_credits", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn repair1_missing_full_sealed_input_cannot_advance_coverage() {
        let (_dir, controller, request, _small_path, _raw) = fixture();
        let document = json!({"server_id":"owned-server","server_store":"owned-store","thread_id":"owned-thread","server_incarnation":"owned-incarnation","native_turns":[{"id":"turn","items":[{"id":"large","type":"userMessage","content":[{"type":"text","text":"X".repeat(40_000)}]}]}]});
        ingest_native(&controller, "owned-binding", &document, &[]).unwrap();
        let auth = request["auth"].clone();
        let query = |kind: &str, args: Value| {
            controller
                .handle(json!({"auth":auth,"project":"p","query":{"kind":kind,"args":args}}))
                .unwrap()
        };
        let stream = query("monitor_context", json!({}))["streams"][0]["id"].clone();
        let seal=controller.handle(json!({"auth":auth,"project":"p","command":{"kind":"seal_monitor_batch","idempotency_key":"review-large-seal","expected_revision":query("snapshot",json!({}))["revision"],"args":{"stream":stream,"lane":"history","through":2,"boundary":"manual"}}})).unwrap();
        let batch = seal["result"]["batch"].clone();
        let row = query("monitor_batch", json!({"batch":batch}));
        let inputs: Vec<Value> = serde_json::from_str(row["inputs"].as_str().unwrap()).unwrap();
        assert!(
            inputs.iter().find(|i| i["item"] == "turn:large").unwrap()["text"]
                .as_str()
                .unwrap()
                .contains("complete_visible_input_artifact")
        );
        let path:String=controller.connection.lock().unwrap().query_row("SELECT m.path FROM monitor_native_inputs n JOIN artifact_manifests m ON m.digest=n.artifact WHERE n.item='turn:large'",[],|r|r.get(0)).unwrap();
        let path = std::path::PathBuf::from(path);
        let full = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let unavailable=controller.handle(json!({"auth":auth,"project":"p","query":{"kind":"monitor_input","args":{"batch":batch,"item":"turn:large"}}}));
        assert!(unavailable.is_err());
        let steps:Vec<Value>=inputs.iter().map(|i|json!({"item":i["item"],"criterion":"admitted-input-retention","rationale":"claimed inspected full input","observation":"claimed complete","assessment":"progress","next_test":"none needed"})).collect();
        let outcome=controller.handle(json!({"auth":auth,"project":"p","command":{"kind":"publish_monitor_assessment","idempotency_key":"review-missing-publish","expected_revision":query("snapshot",json!({}))["revision"],"args":{"batch":batch,"analyst":"independent-negative-probe","version":"v1","steps":steps,"findings":[],"evidence":[]}}}));
        let after = query("monitor_context", json!({}));
        println!(
            "REVIEW_MISSING_FULL_INPUT {}",
            json!({"unavailable":unavailable.err(),"publish":outcome,"stream":after["streams"][0]})
        );
        assert!(
            outcome.is_err(),
            "missing full native input must not acquire reviewed coverage"
        );
        assert_eq!(after["streams"][0]["reviewed_history"], 0);
        let before = query("snapshot", json!({}))["revision"].clone();
        for mode in ["corrupt", "fifo", "symlink", "oversized", "incomplete"] {
            match mode {
                "corrupt" => std::fs::write(&path, b"wrong complete bytes").unwrap(),
                "fifo" => {
                    let name = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).unwrap();
                    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
                }
                "symlink" => {
                    std::fs::write(path.with_extension("owned-real"), &full).unwrap();
                    std::os::unix::fs::symlink(path.with_extension("owned-real"), &path).unwrap();
                }
                "oversized" => std::fs::File::create(&path)
                    .unwrap()
                    .set_len(2_000_000)
                    .unwrap(),
                _ => std::fs::write(&path, &full[..full.len() / 2]).unwrap(),
            }
            let started = std::time::Instant::now();
            let request = json!({"auth":auth,"project":"p","command":{"kind":"publish_monitor_assessment","idempotency_key":format!("invalid-{mode}"),"expected_revision":before,"args":{"batch":batch,"analyst":"independent-negative-probe","version":"v1","steps":steps,"findings":[],"evidence":[]}}});
            let result = controller.handle(request);
            assert!(result.is_err(), "{mode} must not acquire coverage");
            assert!(started.elapsed() < std::time::Duration::from_millis(300));
            assert_eq!(query("snapshot", json!({}))["revision"], before);
            assert_eq!(
                query("monitor_context", json!({}))["streams"][0]["reviewed_history"],
                0
            );
            std::fs::remove_file(&path).unwrap();
        }
        std::fs::write(&path, &full).unwrap();
        let request = json!({"auth":auth,"project":"p","command":{"kind":"publish_monitor_assessment","idempotency_key":"valid-full-input","expected_revision":before,"args":{"batch":batch,"analyst":"independent-negative-probe","version":"v1","steps":steps,"findings":[],"evidence":[]}}});
        let receipt = controller.handle(request.clone()).unwrap();
        assert_eq!(
            query("monitor_context", json!({}))["streams"][0]["reviewed_history"],
            2
        );
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            controller.handle(request).unwrap(),
            receipt,
            "exact durable replay has no new review credit"
        );
    }

    #[test]
    fn repair1_new_short_write_does_not_publish_final_and_reingestion_recovers() {
        let (dir, controller, _request, _path, _raw) = fixture();
        let document = json!({"server_id":"owned-server","server_store":"owned-store","thread_id":"owned-thread","server_incarnation":"owned-incarnation","native_turns":[{"id":"turn","items":[{"id":"short-write","type":"userMessage","content":[{"type":"text","text":"actual owned short write input"}]}]}]});
        let expected = document["native_turns"][0]["items"][0].to_string();
        let final_path = controller.visible_inputs.join(hash(&expected));
        RETENTION_SHORT_WRITE.with(|fault| fault.set(1));
        assert!(
            ingest_native(&controller, "owned-binding", &document, &[])
                .unwrap_err()
                .contains("short-write")
        );
        println!(
            "REPAIR1_SHORT_WRITE {}",
            json!({"final_exists_before_reopen":final_path.exists(),"final_size":std::fs::metadata(&final_path).ok().map(|m|m.len())})
        );
        drop(controller);
        let reopened = Controller::open(dir.path().join("state.db"), "operator-secret").unwrap();
        let result = ingest_native(&reopened, "owned-binding", &document, &[]);
        assert!(
            result.is_ok(),
            "actual short write must permit reingestion from full native input: {result:?}"
        );
        assert_eq!(
            retained_bytes(&final_path, 1_048_576).unwrap(),
            expected.as_bytes()
        );
    }

    fn crash_document() -> Value {
        json!({"server_id":"owned-server","server_store":"owned-store","thread_id":"owned-thread","server_incarnation":"owned-incarnation","native_turns":[{"id":"turn","items":[{"id":"crash-retention","type":"userMessage","content":[{"type":"text","text":"full allowed bytes after owned process exit boundary"}]}]}]})
    }
    #[test]
    fn repair1_retention_crash_child() {
        let Ok(root) = std::env::var("CORENIUS_TEST_RETENTION_CRASH_ROOT") else {
            return;
        };
        let controller = Controller::open(
            std::path::Path::new(&root).join("state.db"),
            "operator-secret",
        )
        .unwrap();
        RETENTION_SHORT_WRITE.with(|fault| fault.set(2));
        ingest_native(&controller, "owned-binding", &crash_document(), &[]).unwrap();
        panic!("declared crash boundary must exit owned child");
    }
    #[test]
    fn repair1_owned_process_exit_before_artifact_publication_reingests_and_concurrent_publish_dedups()
     {
        let (dir, controller, _request, _path, _raw) = fixture();
        let doc = crash_document();
        let raw = doc["native_turns"][0]["items"][0].to_string();
        let final_path = controller.visible_inputs.join(hash(&raw));
        drop(controller);
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "monitor::input_tests::repair1_retention_crash_child",
                "--nocapture",
            ])
            .env("CORENIUS_TEST_RETENTION_CRASH_ROOT", dir.path())
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(86));
        assert!(
            !final_path.exists(),
            "partial bytes must never be published at final digest path"
        );
        let stages: Vec<_> = std::fs::read_dir(final_path.parent().unwrap())
            .unwrap()
            .map(|x| x.unwrap().path())
            .filter(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(".pending-")
            })
            .collect();
        assert_eq!(stages.len(), 1);
        let orphan = std::fs::read(&stages[0]).unwrap();
        assert_eq!(orphan, raw.as_bytes()[..raw.len() / 2]);
        let controller =
            Arc::new(Controller::open(dir.path().join("state.db"), "operator-secret").unwrap());
        let joins: Vec<_> = (0..2)
            .map(|_| {
                let c = Arc::clone(&controller);
                let d = doc.clone();
                std::thread::spawn(move || ingest_native(&c, "owned-binding", &d, &[]))
            })
            .collect();
        for join in joins {
            join.join().unwrap().unwrap();
        }
        assert_eq!(
            retained_bytes(&final_path, 1_048_576).unwrap(),
            raw.as_bytes()
        );
        assert_eq!(
            std::fs::read(&stages[0]).unwrap(),
            orphan,
            "recovery must preserve unreferenced unknown stage"
        );
        let count: i64 = controller
            .connection
            .lock()
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM monitor_native_inputs WHERE item='turn:crash-retention'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
        println!(
            "REPAIR1_RETENTION_EXIT {}",
            json!({"child_exit":86,"partial_final_present":false,"concurrent_ingestions":2,"native_input_identities":count,"orphan_preserved":true})
        );
    }
}
