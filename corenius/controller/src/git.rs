//! Candidate publication, verifier evidence, acceptance and physical ref effects are
//! separate authorities. A single physical gate journals before any ref/cleanup effect.
use crate::{
    git_workspace::{GitBackend, RepositoryState, digest, retain},
    supervisor::{Binding, Execution},
    *,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Arc};
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS git_configuration(singleton INTEGER PRIMARY KEY CHECK(singleton=1), identity TEXT NOT NULL, document TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS git_workspaces(attempt TEXT PRIMARY KEY REFERENCES attempts(id), path TEXT NOT NULL, filesystem TEXT NOT NULL, base TEXT NOT NULL, current_candidate TEXT REFERENCES candidates(id), disposition TEXT NOT NULL, retention TEXT);
CREATE TABLE IF NOT EXISTS git_candidate_details(candidate TEXT PRIMARY KEY REFERENCES candidates(id), brief TEXT NOT NULL, bundle TEXT NOT NULL, manifest TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS git_verification_details(report TEXT PRIMARY KEY REFERENCES verification_reports(id), document TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS git_requests(project TEXT NOT NULL, principal TEXT NOT NULL, epoch INTEGER NOT NULL, key TEXT NOT NULL, arguments TEXT NOT NULL, operation TEXT NOT NULL REFERENCES operations(id), PRIMARY KEY(project,principal,epoch,key));
CREATE TABLE IF NOT EXISTS git_effect_history(seq INTEGER PRIMARY KEY AUTOINCREMENT, operation TEXT NOT NULL REFERENCES operations(id), state TEXT NOT NULL, observation TEXT NOT NULL);
CREATE TRIGGER IF NOT EXISTS git_candidate_details_no_update BEFORE UPDATE ON git_candidate_details BEGIN SELECT RAISE(ABORT,'immutable candidate detail'); END;
CREATE TRIGGER IF NOT EXISTS git_candidate_details_no_delete BEFORE DELETE ON git_candidate_details BEGIN SELECT RAISE(ABORT,'immutable candidate detail'); END;
CREATE TRIGGER IF NOT EXISTS git_verification_details_no_update BEFORE UPDATE ON git_verification_details BEGIN SELECT RAISE(ABORT,'immutable verification detail'); END;
CREATE TRIGGER IF NOT EXISTS git_verification_details_no_delete BEFORE DELETE ON git_verification_details BEGIN SELECT RAISE(ABORT,'immutable verification detail'); END;
CREATE TRIGGER IF NOT EXISTS git_history_no_update BEFORE UPDATE ON git_effect_history BEGIN SELECT RAISE(ABORT,'append-only Git effect history'); END;
CREATE TRIGGER IF NOT EXISTS git_history_no_delete BEFORE DELETE ON git_effect_history BEGIN SELECT RAISE(ABORT,'append-only Git effect history'); END;
"#;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureArgs {
    report: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateArgs {
    candidate: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AcceptArgs {
    candidate: String,
    verification: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LandArgs {
    candidate: String,
    expected_trunk: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetireArgs {
    attempt: String,
    retire_refs: Vec<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CleanupAdmission {
    attempt: String,
    binding: Binding,
    owner: crate::git_workspace::Ownership,
    retention: String,
    retire_refs: Vec<String>,
    process: Execution,
    root_identity: String,
    manifest: String,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct Candidate {
    id: String,
    attempt: String,
    task: String,
    brief: String,
    brief_digest: String,
    base: String,
    commit: String,
    tree: String,
    bundle: String,
    report: String,
    manifest: String,
}
#[derive(Deserialize, Serialize)]
struct CaptureManifest {
    attempt: String,
    brief: String,
    brief_digest: String,
    base: String,
    commit: String,
    tree: String,
    bundle: String,
    report: String,
    state: RepositoryState,
    policy: String,
}
fn decode<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T> {
    serde_json::from_value(v.clone()).map_err(|e| format!("invalid Git arguments: {e}"))
}
fn configured(controller: &Controller) -> Result<Option<Arc<GitBackend>>> {
    Ok(controller
        .git
        .lock()
        .map_err(|_| "Git configuration lock poisoned")?
        .clone())
}
pub fn handles(request: &Value) -> bool {
    matches!(
        request["command"]["kind"].as_str(),
        Some(
            "capture_candidate"
                | "verify_candidate"
                | "accept_candidate"
                | "land_candidate"
                | "retire_workspace"
                | "compare_candidates"
                | "reconcile_git"
        )
    ) || matches!(
        request["query"]["kind"].as_str(),
        Some("workspace_inventory" | "candidate_evidence")
    )
}
impl Controller {
    /// Operator configuration is file-backed and pins tools, suite bytes, roots and
    /// protected ref. It is never accepted from a worker/lead request.
    pub fn configure_git(&self, path: &Path) -> Result<()> {
        let backend = Arc::new(GitBackend::open(path)?);
        let mut c = self
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
        db(tx.execute_batch(SCHEMA))?;
        let previous: Option<String> = db(tx
            .query_row(
                "SELECT identity FROM git_configuration WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .optional())?;
        if previous.as_deref().is_some_and(|p| p != backend.identity) {
            return Err(
                "durable Git policy identity changed; explicit fresh project required".into(),
            );
        }
        db(tx.execute(
            "INSERT OR IGNORE INTO git_configuration VALUES(1,?1,?2)",
            params![
                backend.identity,
                serde_json::to_string(&backend.policy).map_err(|e| e.to_string())?
            ],
        ))?;
        db(tx.commit())?;
        drop(c);
        *self
            .git
            .lock()
            .map_err(|_| "Git configuration lock poisoned")? = Some(backend);
        self.recover_git()
    }
    pub fn git_requires_configuration(&self) -> Result<bool> {
        let c = self
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        db(c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='git_configuration')",
            [],
            |r| r.get(0),
        ))
    }
    pub fn recover_git(&self) -> Result<()> {
        let Some(g) = configured(self)? else {
            return Ok(());
        };
        let _gate = self
            .git_effect_gate
            .lock()
            .map_err(|_| "Git effect gate poisoned")?;
        recover(self, &g)
    }
}
fn binding(c: &Connection, project: &str, attempt: &str) -> Result<Binding> {
    db(c.query_row("SELECT o.id,a.scope_epoch,a.base,a.brief FROM attempts a JOIN operations o ON o.attempt=a.id AND o.kind='start' WHERE a.project=?1 AND a.id=?2",params![project,attempt],|r|Ok(Binding{project:project.into(),attempt:attempt.into(),operation:r.get(0)?,scope_epoch:r.get(1)?,base:r.get(2)?,brief:r.get(3)?})))
}
// Workspace ownership remains tied to the admitted epoch (the start operation),
// including after actor authority is revoked. Never substitute the current epoch.
fn owned_binding(c: &Connection, project: &str, attempt: &str) -> Result<Binding> {
    let raw: String = db(c.query_row(
        "SELECT payload FROM operations WHERE project=?1 AND attempt=?2 AND kind='start'",
        params![project, attempt],
        |r| r.get(0),
    ))?;
    let payload: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let mut b = binding(c, project, attempt)?;
    b.scope_epoch = payload["scope_epoch"]
        .as_i64()
        .ok_or("admitted authority epoch missing")?;
    Ok(b)
}
fn candidate(c: &Connection, project: &str, id: &str) -> Result<Candidate> {
    db(c.query_row("SELECT c.id,c.attempt,a.task,d.brief,b.digest,c.base,c.commit_id,c.tree_id,d.bundle,c.report_digest,d.manifest FROM candidates c JOIN attempts a ON a.id=c.attempt JOIN git_candidate_details d ON d.candidate=c.id JOIN briefs b ON b.id=d.brief WHERE c.id=?1 AND a.project=?2",params![id,project],|r|Ok(Candidate{id:r.get(0)?,attempt:r.get(1)?,task:r.get(2)?,brief:r.get(3)?,brief_digest:r.get(4)?,base:r.get(5)?,commit:r.get(6)?,tree:r.get(7)?,bundle:r.get(8)?,report:r.get(9)?,manifest:r.get(10)?})))
}
fn current(c: &Connection, v: &Candidate) -> Result<()> {
    let (current,task_state,brief):(Option<String>,String,String)=db(c.query_row("SELECT w.current_candidate,t.state,t.brief FROM git_workspaces w JOIN attempts a ON a.id=w.attempt JOIN tasks t ON t.project=a.project AND t.id=a.task WHERE w.attempt=?1",[&v.attempt],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))))?;
    let later:bool=db(c.query_row("SELECT EXISTS(SELECT 1 FROM attempts WHERE project=(SELECT project FROM attempts WHERE id=?1) AND task=?2 AND rowid>(SELECT rowid FROM attempts WHERE id=?1))",params![v.attempt,v.task],|r|r.get(0)))?;
    if current.as_deref() != Some(&v.id)
        || later
        || brief != v.brief
        || matches!(
            task_state.as_str(),
            "aborted" | "repair-required" | "integration-required" | "landed"
        )
    {
        return Err("candidate is superseded or task authority withdrawn".into());
    }
    Ok(())
}
fn evidence(g: &GitBackend, v: &Candidate) -> Result<()> {
    let manifest: CaptureManifest =
        serde_json::from_slice(&g.bytes(&v.manifest)?).map_err(|e| e.to_string())?;
    if manifest.attempt != v.attempt
        || manifest.base != v.base
        || manifest.commit != v.commit
        || manifest.tree != v.tree
        || manifest.brief != v.brief
        || manifest.brief_digest != v.brief_digest
        || manifest.bundle != v.bundle
        || manifest.report != v.report
        || manifest.policy != g.identity
    {
        return Err("candidate retained manifest binding differs".into());
    }
    g.bytes(&v.report)?;
    g.import(&v.bundle, &v.commit, &v.tree)
}
fn record_artifact(tx: &Transaction<'_>, g: &GitBackend, digest: &str) -> Result<()> {
    let bytes = g.bytes(digest)?;
    db(tx.execute(
        "INSERT OR IGNORE INTO artifact_manifests VALUES(?1,?2,?3)",
        params![
            digest,
            g.policy
                .root
                .join("artifacts")
                .join(digest)
                .to_string_lossy(),
            i64::try_from(bytes.len()).map_err(|_| "artifact too large")?
        ],
    ))?;
    Ok(())
}
fn transition(c: &Connection, operation: &str, state: &str, observation: &Value) -> Result<()> {
    db(c.execute(
        "UPDATE operations SET state=?2,observation=?3 WHERE id=?1",
        params![operation, state, observation.to_string()],
    ))?;
    db(c.execute(
        "INSERT INTO git_effect_history(operation,state,observation) VALUES(?1,?2,?3)",
        params![operation, state, observation.to_string()],
    ))?;
    Ok(())
}
fn trusted_proof(c: &Connection, g: &GitBackend, v: &Candidate, report: &str) -> Result<Value> {
    current(c, v)?;
    let (latest,suite,environment,artifact,verdict):(String,String,String,String,String)=db(c.query_row("SELECT id,suite,environment,artifact_digest,verdict FROM verification_reports WHERE candidate=?1 ORDER BY rowid DESC LIMIT 1",[&v.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))))?;
    let required = g.suite(&v.task, &v.base)?;
    let latest_operation:String=db(c.query_row("SELECT id FROM operations WHERE kind='git-verify_candidate' AND json_extract(payload,'$.candidate')=?1 ORDER BY rowid DESC LIMIT 1",[&v.id],|r|r.get(0)))?;
    if latest_operation != report
        || latest != report
        || verdict != "pass"
        || suite != digest(&serde_json::to_vec(required).map_err(|e| e.to_string())?)
    {
        return Err("passing current controller-required proof missing".into());
    }
    let raw: String = db(c.query_row(
        "SELECT document FROM git_verification_details WHERE report=?1",
        [report],
        |r| r.get(0),
    ))?;
    let doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if doc["candidate"] != v.id
        || doc["commit"] != v.commit
        || doc["tree"] != v.tree
        || doc["base"] != v.base
        || doc["brief_digest"] != v.brief_digest
        || doc["environment"] != environment
        || doc["suite"] != suite
        || doc["logs"] != artifact
        || doc["policy"] != g.identity
    {
        return Err("verification provenance binding differs".into());
    }
    let runtime = g.verifier_runtime_document()?;
    if doc["runtime"] != runtime {
        return Err("verifier runtime proof binding differs".into());
    }
    g.bytes(&artifact)?;
    check_suite(g, required)?;
    Ok(doc)
}
fn check_suite(g: &GitBackend, s: &crate::git_workspace::Suite) -> Result<()> {
    crate::git_workspace::check_tool(&s.script)?;
    crate::git_workspace::check_tool(&g.policy.python)?;
    crate::git_workspace::check_tool(&g.policy.bubblewrap)?;
    g.verifier_runtime_evidence()?;
    Ok(())
}

pub fn dispatch(controller: &Controller, request: Value) -> Result<Value> {
    let request: Request =
        serde_json::from_value(request).map_err(|e| format!("invalid request: {e}"))?;
    let _gate = controller
        .git_effect_gate
        .lock()
        .map_err(|_| "Git effect gate poisoned")?;
    let backend = configured(controller)?;
    if let (None, Some(query)) = (&request.command, &request.query) {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &request.auth, &request.project)?;
        if !matches!(p.role.as_str(), "operator" | "lead") {
            return Err("authorization: controller Git evidence query required".into());
        }
        if query.kind == "workspace_inventory" && backend.is_none() {
            return Ok(
                json!({"configured":false,"backend":"git","jj_qualified":false,"workspaces":[]}),
            );
        }
        let g = backend.as_ref().ok_or("Git backend not configured")?;
        if request.project != g.policy.project {
            return Err("Git project scope mismatch".into());
        }
        if query.kind == "candidate_evidence" {
            let v = candidate(&c, &request.project, text(&query.args, "candidate")?)?;
            drop(c);
            evidence(g, &v)?;
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            return Ok(
                json!({"candidate":v,"verification":query::rows(&c,"SELECT r.*,d.document FROM verification_reports r JOIN git_verification_details d ON d.report=r.id WHERE r.candidate=?1 ORDER BY r.rowid",&[&v.id])?,"acceptance":query::rows(&c,"SELECT * FROM acceptance_decisions WHERE candidate=?1 ORDER BY rowid",&[&v.id])?}),
            );
        }
        if let Some(input) = query.args["input"].as_str() {
            let bytes: i64 = db(c.query_row(
                "SELECT bytes FROM control_inventory_inputs WHERE project=?1 AND digest=?2",
                params![request.project, input],
                |r| r.get(0),
            ))?;
            if query.args["bytes"].as_i64() != Some(bytes) {
                return Err("inventory input exact reference changed".into());
            }
            drop(c);
            let raw = g.bytes(input)?;
            if raw.len() as i64 != bytes {
                return Err("inventory input byte identity changed".into());
            }
            let content = std::str::from_utf8(&raw).map_err(|e| e.to_string())?;
            let offset = usize::try_from(query.args["offset"].as_u64().unwrap_or(0))
                .map_err(|_| "inventory input offset bound")?;
            if offset > content.len() || !content.is_char_boundary(offset) {
                return Err("inventory input UTF8 offset refused".into());
            }
            let mut end = content.len().min(offset.saturating_add(65536));
            while !content.is_char_boundary(end) {
                end -= 1;
            }
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            authenticate(&c, &request.auth, &request.project)?;
            return Ok(
                json!({"sha256":input,"bytes":bytes,"offset":offset,"text":&content[offset..end],"next_offset":if end<content.len(){Some(end)}else{None}}),
            );
        }
        drop(c);
        return inventory(controller, g, &request.project, &query.args);
    }
    if request.query.is_some() {
        return Err("exactly one command or query required".into());
    }
    let command = request.command.ok_or("command required")?;
    if command.idempotency_key.is_empty() || command.idempotency_key.len() > 200 {
        return Err("invalid idempotency key".into());
    }
    let g = backend.ok_or("Git backend not configured")?;
    if request.project != g.policy.project {
        return Err("Git project scope mismatch".into());
    }
    let arguments=json!({"kind":command.kind,"expected_revision":command.expected_revision,"args":command.args}).to_string();
    let (p, operation) = {
        let mut c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let p = authenticate(&c, &request.auth, &request.project)?;
        if (command.kind == "capture_candidate" && p.role != "attempt")
            || (command.kind != "capture_candidate"
                && !matches!(p.role.as_str(), "operator" | "lead"))
        {
            return Err("authorization: Git authority scope".into());
        }
        if command.kind == "accept_candidate" && p.role != "operator" && p.role != "lead" {
            return Err("authorization: acceptance decision authority".into());
        }
        let prior:Option<(String,String)>=db(c.query_row("SELECT arguments,response FROM receipts WHERE project=?1 AND principal=?2 AND epoch=?3 AND key=?4",params![request.project,p.id,p.epoch,command.idempotency_key],|r|Ok((r.get(0)?,r.get(1)?))).optional())?;
        if let Some((saved, response)) = prior {
            if saved != arguments {
                return Err("idempotency argument mismatch".into());
            }
            return serde_json::from_str(&response).map_err(|e| e.to_string());
        }
        let existing:Option<(String,String)>=db(c.query_row("SELECT arguments,operation FROM git_requests WHERE project=?1 AND principal=?2 AND epoch=?3 AND key=?4",params![request.project,p.id,p.epoch,command.idempotency_key],|r|Ok((r.get(0)?,r.get(1)?))).optional())?;
        let operation = if let Some((saved, operation)) = existing {
            if saved != arguments {
                return Err("idempotency argument mismatch".into());
            }
            operation
        } else {
            let revision: i64 = db(c.query_row(
                "SELECT revision FROM projects WHERE id=?1",
                [&request.project],
                |r| r.get(0),
            ))?;
            if i64::try_from(command.expected_revision).ok() != Some(revision) {
                return Err(format!("revision conflict: current {revision}"));
            }
            // Parse the complete allowed payload before journaling any effect.
            match command.kind.as_str() {
                "capture_candidate" => {
                    let a: CaptureArgs = decode(&command.args)?;
                    if a.report.is_empty() || a.report.len() > 1_048_576 {
                        return Err("bounded candidate report required".into());
                    }
                }
                "verify_candidate" | "compare_candidates" => {
                    let _: CandidateArgs = decode(&command.args)?;
                }
                "accept_candidate" => {
                    let _: AcceptArgs = decode(&command.args)?;
                }
                "land_candidate" => {
                    let a: LandArgs = decode(&command.args)?;
                    commit_oid(&a.expected_trunk)?;
                }
                "retire_workspace" => {
                    let _: RetireArgs = decode(&command.args)?;
                }
                "reconcile_git" => {
                    if command.args != json!({}) {
                        return Err("reconcile accepts no selectors".into());
                    }
                }
                _ => return Err("unknown Git command".into()),
            }
            let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
            if command.kind == "verify_candidate" {
                // A refused fresh verifier must never become the latest operation
                // and invalidate the proof already bound to accepted evidence.
                let args: CandidateArgs = decode(&command.args)?;
                let v = candidate(&tx, &request.project, &args.candidate)?;
                current(&tx, &v)?;
                let state: String = db(tx.query_row(
                    "SELECT state FROM tasks WHERE project=?1 AND id=?2",
                    params![request.project, v.task],
                    |r| r.get(0),
                ))?;
                if state != "submitted" {
                    return Err("fresh verification requires submitted task; accepted/terminal evidence preserved".into());
                }
            }
            let operation = id();
            let attempt = p.attempt.as_deref();
            db(tx.execute("INSERT INTO operations(id,project,attempt,kind,state,payload) VALUES(?1,?2,?3,?4,'prepared',?5)",params![operation,request.project,attempt,format!("git-{}",command.kind),command.args.to_string()]))?;
            db(tx.execute(
                "INSERT INTO git_requests VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    request.project,
                    p.id,
                    p.epoch,
                    command.idempotency_key,
                    arguments,
                    operation
                ],
            ))?;
            transition(
                &tx,
                &operation,
                "applying",
                &json!({"intent_committed_before_effect":true}),
            )?;
            db(tx.commit())?;
            operation
        };
        (p, operation)
    };
    let retained_outcome = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let (state, observation): (String, Option<String>) = db(c.query_row(
            "SELECT state,observation FROM operations WHERE id=?1",
            [&operation],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ))?;
        if state == "confirmed" {
            observation
                .map(|raw| serde_json::from_str::<Value>(&raw).map_err(|e| e.to_string()))
                .transpose()?
        } else {
            None
        }
    };
    let outcome = match retained_outcome {
        Some(outcome) => Ok(outcome),
        None => execute(controller, &g, &request.project, &p, &command, &operation),
    };
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    // A fully admitted landing publishes in its confirmation transaction, even
    // after requester replacement. Other effects still require current authority.
    if let Some(response) = retained_response(&c, &operation)? {
        return Ok(response);
    }
    // Current actor authority is rechecked after every external effect, before publication.
    let authenticated = authenticate(&c, &request.auth, &request.project);
    let result = match outcome {
        Ok(value) => {
            authenticated?;
            value
        }
        Err(error) => {
            let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
            let payload: String = db(tx.query_row(
                "SELECT payload FROM operations WHERE id=?1",
                [&operation],
                |r| r.get(0),
            ))?;
            let payload: Value = serde_json::from_str(&payload).map_err(|e| e.to_string())?;
            let uncertain = command.kind == "land_candidate" && payload["commit"].is_string();
            transition(
                &tx,
                &operation,
                if uncertain { "unknown" } else { "failed" },
                &json!({"error":error}),
            )?;
            db(tx.commit())?;
            return Err(error);
        }
    };
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    let response = publish(&tx, &request.project, &operation, &result)?;
    db(tx.commit())?;
    Ok(response)
}
fn retained_response(c: &Connection, operation: &str) -> Result<Option<Value>> {
    let raw:Option<String>=db(c.query_row("SELECT r.response FROM receipts r JOIN git_requests g ON g.project=r.project AND g.principal=r.principal AND g.epoch=r.epoch AND g.key=r.key WHERE g.operation=?1",[operation],|r|r.get(0)).optional())?;
    raw.map(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
        .transpose()
}
/// Publish a broker's retained decision exactly once, independently of a later
/// actor replacement. The originating request and authority remain in the journal.
fn publish(tx: &Transaction<'_>, project: &str, operation: &str, result: &Value) -> Result<Value> {
    if let Some(response) = retained_response(tx, operation)? {
        return Ok(response);
    }
    let (principal, epoch, key, arguments): (String, i64, String, String) = db(tx.query_row(
        "SELECT principal,epoch,key,arguments FROM git_requests WHERE project=?1 AND operation=?2",
        params![project, operation],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ))?;
    let request: Value = serde_json::from_str(&arguments).map_err(|e| e.to_string())?;
    let kind = text(&request, "kind")?;
    let task = result.get("task").and_then(Value::as_str);
    let previous: i64 = db(tx.query_row(
        "SELECT revision FROM projects WHERE id=?1",
        [project],
        |r| r.get(0),
    ))?;
    let revision = previous + 1;
    db(tx.execute(
        "UPDATE projects SET revision=?2 WHERE id=?1",
        params![project, revision],
    ))?;
    if result["state"] != "unknown" && result["state"] != "stale_base" {
        transition(tx, operation, "confirmed", result)?;
    }
    db(tx.execute("INSERT INTO events(project,task,kind,body) VALUES(?1,?2,?3,?4)",params![project,task,kind,json!({"principal":principal,"revision":revision,"args":request["args"],"result":result}).to_string()]))?;
    let event = tx.last_insert_rowid();
    kata::queue_for_event(tx, project, task, kind, revision, event)?;
    let response = json!({"receipt":id(),"event_seq":event,"revision":revision,"result":result});
    db(tx.execute(
        "INSERT INTO receipts VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            project,
            principal,
            epoch,
            key,
            arguments,
            response.to_string()
        ],
    ))?;
    Ok(response)
}

fn execute(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    p: &Principal,
    command: &Command,
    operation: &str,
) -> Result<Value> {
    match command.kind.as_str() {
        "capture_candidate" => capture_candidate(
            controller,
            g,
            project,
            p,
            &decode(&command.args)?,
            operation,
        ),
        "verify_candidate" => {
            verify_candidate(controller, g, project, &decode(&command.args)?, operation)
        }
        "accept_candidate" => {
            let a: AcceptArgs = decode(&command.args)?;
            let mut c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            guard_actor(&c, p, project)?;
            let task_state:String=db(c.query_row("SELECT t.state FROM tasks t JOIN attempts a ON a.project=t.project AND a.task=t.id JOIN candidates v ON v.attempt=a.id WHERE v.id=?1 AND t.project=?2",params![a.candidate,project],|r|r.get(0)))?;
            if task_state != "submitted" {
                return Err(
                    "acceptance requires submitted task; accepted/terminal state preserved".into(),
                );
            }
            let v = candidate(&c, project, &a.candidate)?;
            evidence(g, &v)?;
            trusted_proof(&c, g, &v, &a.verification)?;
            let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
            let acceptance = operation.to_owned();
            db(tx.execute(
                "INSERT OR IGNORE INTO acceptance_decisions VALUES(?1,?2,?3,?4,'accepted')",
                params![acceptance, v.id, a.verification, p.id],
            ))?;
            db(tx.execute(
                "UPDATE tasks SET state='accepted' WHERE project=?1 AND id=?2",
                params![project, v.task],
            ))?;
            db(tx.commit())?;
            Ok(
                json!({"acceptance":acceptance,"candidate":v.id,"verification":a.verification,"task":v.task}),
            )
        }
        "compare_candidates" => {
            let a: CandidateArgs = decode(&command.args)?;
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            let v = candidate(&c, project, &a.candidate)?;
            drop(c);
            evidence(g, &v)?;
            Ok(
                json!({"candidate":v.id,"base":v.base,"commit":v.commit,"diff":g.compare(&v.base,&v.commit)?}),
            )
        }
        "land_candidate" => land_candidate(
            controller,
            g,
            project,
            p,
            &decode(&command.args)?,
            operation,
        ),
        "retire_workspace" => {
            retire_workspace(controller, g, project, &decode(&command.args)?, operation)
        }
        "reconcile_git" => {
            recover(controller, g)?;
            Ok(json!({"state":"reconciled","protected_ref":g.head()?}))
        }
        _ => Err("unknown Git command".into()),
    }
}
fn capture_candidate(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    p: &Principal,
    a: &CaptureArgs,
    operation: &str,
) -> Result<Value> {
    let attempt = p.attempt.as_deref().ok_or("own attempt required")?;
    let (binding, task, brief_digest) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let b = owned_binding(&c, project, attempt)?;
        let (state, cancel, epoch): (String, bool, i64) = db(c.query_row(
            "SELECT state,cancel_requested,scope_epoch FROM attempts WHERE id=?1",
            [attempt],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        ))?;
        if state != "running" || cancel || epoch != p.epoch {
            return Err("candidate submission requires current live attempt grant".into());
        }
        let (task, brief_digest): (String, String) = db(c.query_row(
            "SELECT a.task,b.digest FROM attempts a JOIN briefs b ON b.id=a.brief WHERE a.id=?1",
            [attempt],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ))?;
        (b, task, brief_digest)
    };
    let (repo, owner) = g.owned(&binding)?;
    let scratch = g.policy.root.join("captures").join(operation);
    if !scratch.exists() {
        fs::create_dir(&scratch).map_err(|e| e.to_string())?;
    }
    let saved = scratch.join("manifest.json");
    let (manifest, manifest_digest) = if saved.exists() {
        let bytes = crate::git_workspace::read_regular(&saved, 32 * 1024 * 1024)?;
        let m: CaptureManifest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if m.report != digest(a.report.as_bytes())
            || m.attempt != attempt
            || m.brief != binding.brief
            || m.base != binding.base
            || m.brief_digest != brief_digest
        {
            return Err("retained capture identity mismatch".into());
        }
        (m, g.artifact(&bytes)?)
    } else {
        let (state, bytes) = g.capture_candidate(&binding, &scratch)?;
        let bundle = g.artifact(&bytes)?;
        let report = g.artifact(a.report.as_bytes())?;
        g.import(&bundle, &state.commit, &state.tree)?;
        if !g.ancestor(&g.store(), &binding.base, &state.commit)? {
            return Err("candidate base is not retained full ancestry".into());
        }
        let m = CaptureManifest {
            attempt: attempt.into(),
            brief: binding.brief.clone(),
            brief_digest: brief_digest.clone(),
            base: binding.base.clone(),
            commit: state.commit.clone(),
            tree: state.tree.clone(),
            bundle,
            report,
            state,
            policy: g.identity.clone(),
        };
        let bytes = serde_json::to_vec(&m).map_err(|e| e.to_string())?;
        let d = g.artifact(&bytes)?;
        retain(&saved, &bytes)?;
        (m, d)
    };
    #[cfg(feature = "fault-injection")]
    if let Ok(path) = std::env::var("CORENIUS_TEST_GIT_CAPTURE_BARRIER")
        && Path::new(&path).is_file()
    {
        fs::write(
            format!("{path}.observed"),
            json!({"operation":operation,"manifest":manifest_digest,"bundle":manifest.bundle})
                .to_string(),
        )
        .map_err(|e| e.to_string())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while Path::new(&path).is_file() {
            if std::time::Instant::now() > deadline {
                return Err("declared capture publication barrier deadline".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    g.bytes(&manifest.report)?;
    g.import(&manifest.bundle, &manifest.commit, &manifest.tree)?;
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let (epoch, state, cancel): (i64, String, bool) = db(c.query_row(
        "SELECT scope_epoch,state,cancel_requested FROM attempts WHERE id=?1",
        [attempt],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ))?;
    let task_state: String = db(c.query_row(
        "SELECT state FROM tasks WHERE project=?1 AND id=?2",
        params![project, task],
        |r| r.get(0),
    ))?;
    if epoch != p.epoch
        || state != "running"
        || cancel
        || !matches!(task_state.as_str(), "active" | "submitted")
    {
        return Err(
            "candidate publication refused: current authority changed during capture".into(),
        );
    }
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    let candidate = operation.to_owned();
    db(tx.execute(
        "INSERT OR IGNORE INTO candidates VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            candidate,
            attempt,
            manifest.commit,
            manifest.tree,
            manifest.base,
            manifest.report
        ],
    ))?;
    db(tx.execute(
        "INSERT OR IGNORE INTO git_candidate_details VALUES(?1,?2,?3,?4)",
        params![candidate, manifest.brief, manifest.bundle, manifest_digest],
    ))?;
    for d in [&manifest.bundle, &manifest.report, &manifest_digest] {
        record_artifact(&tx, g, d)?;
    }
    db(tx.execute("INSERT INTO git_workspaces VALUES(?1,?2,?3,?4,?5,'retained',NULL) ON CONFLICT(attempt) DO UPDATE SET current_candidate=excluded.current_candidate",params![attempt,repo.to_string_lossy(),owner.workspace_identity,binding.base,candidate]))?;
    db(tx.execute(
        "UPDATE tasks SET state='submitted' WHERE project=?1 AND id=?2 AND state<>'aborted'",
        params![project, task],
    ))?;
    db(tx.commit())?;
    Ok(
        json!({"candidate":candidate,"task":task,"commit":manifest.commit,"tree":manifest.tree,"base":manifest.base,"bundle":manifest.bundle,"manifest":manifest_digest,"report":manifest.report}),
    )
}
fn verify_candidate(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    a: &CandidateArgs,
    operation: &str,
) -> Result<Value> {
    let v = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let v = candidate(&c, project, &a.candidate)?;
        current(&c, &v)?;
        v
    };
    evidence(g, &v)?;
    let retained: Option<String> = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        db(c.query_row(
            "SELECT document FROM git_verification_details WHERE report=?1",
            [operation],
            |r| r.get(0),
        )
        .optional())?
    };
    if let Some(raw) = retained {
        let doc: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        if doc["candidate"] != v.id
            || doc["commit"] != v.commit
            || doc["tree"] != v.tree
            || doc["base"] != v.base
            || doc["policy"] != g.identity
        {
            return Err("retained verifier operation binding differs".into());
        }
        g.bytes(text(&doc, "logs")?)?;
        let report = g.artifact(raw.as_bytes())?;
        return Ok(
            json!({"verification":operation,"candidate":v.id,"task":v.task,"verdict":doc["verdict"],"report":report,"document":doc}),
        );
    }
    let suite = g.suite(&v.task, &v.base)?;
    let repo = g.materialize_candidate(&v.bundle, &v.commit, &v.tree)?;
    let (pass, log, environment, runtime) = g.verify(&repo, suite)?;
    let logs = g.artifact(&log)?;
    let suite_digest = digest(&serde_json::to_vec(suite).map_err(|e| e.to_string())?);
    let doc = json!({"candidate":v.id,"commit":v.commit,"tree":v.tree,"base":v.base,"brief":v.brief,"brief_digest":v.brief_digest,"suite":suite_digest,"suite_id":suite.id,"environment":environment,"runtime":runtime,"logs":logs,"materialization":repo,"policy":g.identity,"verdict":if pass{"pass"}else{"fail"}});
    let report_bytes = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
    let report_artifact = g.artifact(&report_bytes)?;
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    current(&c, &v)?;
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    db(tx.execute(
        "INSERT INTO verification_reports VALUES(?1,?2,?3,?4,?5,?6)",
        params![
            operation,
            v.id,
            suite_digest,
            environment,
            logs,
            if pass { "pass" } else { "fail" }
        ],
    ))?;
    db(tx.execute(
        "INSERT INTO git_verification_details VALUES(?1,?2)",
        params![operation, doc.to_string()],
    ))?;
    record_artifact(&tx, g, &logs)?;
    record_artifact(&tx, g, &report_artifact)?;
    db(tx.commit())?;
    Ok(
        json!({"verification":operation,"candidate":v.id,"task":v.task,"verdict":if pass{"pass"}else{"fail"},"report":report_artifact,"document":doc}),
    )
}
fn guard_actor(c: &Connection, p: &Principal, project: &str) -> Result<()> {
    let (role, epoch): (String, i64) = db(c.query_row(
        "SELECT role,epoch FROM principals WHERE id=?1",
        [&p.id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    if role != p.role || epoch != p.epoch {
        return Err("authentication: actor identity changed before Git decision".into());
    }
    if p.role == "lead" {
        let epoch: i64 = db(c.query_row(
            "SELECT lead_epoch FROM projects WHERE id=?1",
            [project],
            |r| r.get(0),
        ))?;
        if epoch != p.epoch {
            return Err("authentication: superseded lead authority before Git decision".into());
        }
    }
    Ok(())
}
fn accepted(c: &Connection, g: &GitBackend, v: &Candidate) -> Result<String> {
    let report:String=db(c.query_row("SELECT verification FROM acceptance_decisions WHERE candidate=?1 AND decision='accepted' ORDER BY rowid DESC LIMIT 1",[&v.id],|r|r.get(0)))?;
    trusted_proof(c, g, v, &report)?;
    Ok(report)
}
fn land_candidate(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    p: &Principal,
    a: &LandArgs,
    operation: &str,
) -> Result<Value> {
    let v = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let v = candidate(&c, project, &a.candidate)?;
        accepted(&c, g, &v)?;
        let unknown:bool=db(c.query_row("SELECT EXISTS(SELECT 1 FROM operations WHERE project=?1 AND kind='git-land_candidate' AND id<>?2 AND state IN ('prepared','applying','unknown'))",params![project,operation],|r|r.get(0)))?;
        if unknown {
            return Err("unresolved landing blocks conflicting ref effects".into());
        }
        v
    };
    evidence(g, &v)?;
    let actual = g.head()?;
    barrier("before_land_admission", operation)?;
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let base: String = db(
        c.query_row("SELECT base FROM projects WHERE id=?1", [project], |r| {
            r.get(0)
        }),
    )?;
    if actual != base {
        transition(
            &c,
            operation,
            "unknown",
            &json!({"actual":actual,"project_base":base,"reason":"protected ref/project drift"}),
        )?;
        return Ok(json!({"state":"unknown","candidate":v.id,"task":v.task,"actual":actual}));
    }
    if a.expected_trunk != base || v.base != base {
        let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
        transition(
            &tx,
            operation,
            "failed",
            &json!({"reason":"stale base","actual":actual,"candidate_base":v.base}),
        )?;
        db(tx.execute(
            "UPDATE tasks SET state='integration-required' WHERE project=?1 AND id=?2",
            params![project, v.task],
        ))?;
        db(tx.commit())?;
        return Ok(
            json!({"state":"stale_base","candidate":v.id,"task":v.task,"actual":actual,"requires_new_attempt":true}),
        );
    }
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    guard_actor(&tx, p, project)?;
    let admitted = candidate(&tx, project, &a.candidate)?;
    let verification = accepted(&tx, g, &admitted)?;
    let state: String = db(tx.query_row(
        "SELECT state FROM tasks WHERE project=?1 AND id=?2",
        params![project, v.task],
        |r| r.get(0),
    ))?;
    if state != "accepted"
        || admitted.commit != v.commit
        || admitted.tree != v.tree
        || admitted.base != base
    {
        return Err("landing admission requires current accepted candidate/base".into());
    }
    let payload = json!({"candidate":v.id,"commit":v.commit,"tree":v.tree,"base":base,"task":v.task,"protected_ref":g.policy.protected_ref,"policy":g.identity,"principal":p.id,"epoch":p.epoch,"verification":verification,"authorized_admission":true});
    db(tx.execute(
        "UPDATE operations SET payload=?2 WHERE id=?1",
        params![operation, payload.to_string()],
    ))?;
    transition(
        &tx,
        operation,
        "applying",
        &json!({"intent":payload,"before_ref_effect":true}),
    )?;
    db(tx.commit())?;
    drop(c);
    barrier("after_land_admission", operation)?;
    fault("before_ref_update", operation)?;
    g.land(&v.commit, &base)?;
    fault("after_ref_update", operation)?;
    if g.head()? != v.commit {
        return Err("protected ref changed after CAS; unresolved".into());
    }
    confirm_landing(controller, operation, project, &v, &base)?;
    Ok(
        json!({"state":"landed","candidate":v.id,"task":v.task,"commit":v.commit,"base":base,"operation":operation}),
    )
}
fn confirm_landing(
    controller: &Controller,
    operation: &str,
    project: &str,
    v: &Candidate,
    expected: &str,
) -> Result<()> {
    let mut c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
    let base: String = db(
        tx.query_row("SELECT base FROM projects WHERE id=?1", [project], |r| {
            r.get(0)
        }),
    )?;
    if base != expected && base != v.commit {
        return Err("project lineage differs at landing confirmation".into());
    }
    db(tx.execute(
        "UPDATE projects SET base=?2 WHERE id=?1",
        params![project, v.commit],
    ))?;
    db(tx.execute(
        "UPDATE tasks SET state='landed' WHERE project=?1 AND id=?2",
        params![project, v.task],
    ))?;
    publish(
        &tx,
        project,
        operation,
        &json!({"state":"landed","candidate":v.id,"task":v.task,"commit":v.commit,"base":expected,"observed_ref":true,"operation":operation}),
    )?;
    db(tx.commit())?;
    Ok(())
}
fn barrier(point: &str, operation: &str) -> Result<()> {
    #[cfg(feature = "fault-injection")]
    if std::env::var("CORENIUS_TEST_GIT_BARRIER_POINT").as_deref() == Ok(point)
        && let Ok(path) = std::env::var("CORENIUS_TEST_GIT_BARRIER_FILE")
        && Path::new(&path).is_file()
    {
        fs::write(
            format!("{path}.observed"),
            json!({"point":point,"operation":operation}).to_string(),
        )
        .map_err(|e| e.to_string())?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while Path::new(&path).is_file() {
            if std::time::Instant::now() > deadline {
                return Err("declared Git scheduling barrier deadline".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    let _ = (point, operation);
    Ok(())
}
fn fault(point: &str, operation: &str) -> Result<()> {
    #[cfg(feature = "fault-injection")]
    if std::env::var("CORENIUS_TEST_GIT_CRASH_POINT").as_deref() == Ok(point)
        && let Ok(marker) = std::env::var("CORENIUS_TEST_GIT_CRASH_MARKER")
        && Path::new(&marker).exists()
    {
        fs::remove_file(&marker).map_err(|e| e.to_string())?;
        fs::write(
            format!("{marker}.observed"),
            json!({"point":point,"operation":operation}).to_string(),
        )
        .map_err(|e| e.to_string())?;
        std::process::exit(81);
    }
    let _ = (point, operation);
    Ok(())
}
fn recover(controller: &Controller, g: &GitBackend) -> Result<()> {
    let operations = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        query::rows(
            &c,
            "SELECT id,project,payload FROM operations WHERE kind='git-land_candidate' AND state IN ('prepared','applying','unknown') ORDER BY rowid",
            &[],
        )?
    };
    for operation in operations.as_array().ok_or("landing journal invalid")? {
        let op = text(operation, "id")?;
        let project = text(operation, "project")?;
        let payload: Value =
            serde_json::from_str(text(operation, "payload")?).map_err(|e| e.to_string())?;
        if payload["policy"] != g.identity
            || payload["protected_ref"] != g.policy.protected_ref
            || payload["commit"].as_str().is_none()
            || payload["authorized_admission"] != true
        {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            transition(
                &c,
                op,
                if payload["commit"].is_string() {
                    "unknown"
                } else {
                    "failed"
                },
                &json!({"reason":"landing ref effect was not prepared or policy changed"}),
            )?;
            continue;
        }
        let v = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            candidate(&c, project, text(&payload, "candidate")?)?
        };
        if payload["commit"] != v.commit || payload["tree"] != v.tree || payload["base"] != v.base {
            return Err("retained landing admission candidate provenance differs".into());
        }
        evidence(g, &v)?;
        let actual = g.head()?;
        let expected = text(&payload, "base")?;
        if actual == v.commit {
            confirm_landing(controller, op, project, &v, expected)?;
            continue;
        }
        if actual == expected {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            accepted(&c, g, &v)?;
            drop(c);
            g.land(&v.commit, expected)?;
            confirm_landing(controller, op, project, &v, expected)?;
            continue;
        }
        // An ancestry relation alone is insufficient. Require a complete retained
        // chain of confirmed protected-ref operations to explain the later head.
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let known = query::rows(
            &c,
            "SELECT payload FROM operations WHERE project=?1 AND kind='git-land_candidate' AND state='confirmed' ORDER BY rowid",
            &[&project],
        )?;
        let mut cursor = v.commit.clone();
        for row in known.as_array().ok_or("landing lineage invalid")? {
            let p: Value =
                serde_json::from_str(text(row, "payload")?).map_err(|e| e.to_string())?;
            if p["base"] == cursor
                && p["policy"] == g.identity
                && p["protected_ref"] == g.policy.protected_ref
            {
                cursor = text(&p, "commit")?.to_owned();
            }
        }
        drop(c);
        let explained = cursor == actual && g.ancestor(&g.policy.canonical, &v.commit, &actual)?;
        let mut c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        if explained {
            let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
            publish(
                &tx,
                project,
                op,
                &json!({"state":"landed","task":v.task,"candidate":v.id,"commit":v.commit,"base":expected,"actual":actual,"retained_later_lineage":true,"operation":op}),
            )?;
            db(tx.commit())?;
        } else {
            transition(
                &c,
                op,
                "unknown",
                &json!({"reason":"unexplained protected ref divergence","actual":actual,"expected":expected,"candidate":v.commit}),
            )?;
        }
    }
    Ok(())
}
fn stopped(c: &Connection, project: &str, attempt: &str, b: &Binding) -> Result<Execution> {
    let (state, observation): (String, Option<String>) = db(c.query_row(
        "SELECT state,observation FROM attempts WHERE project=?1 AND id=?2",
        params![project, attempt],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ))?;
    let observed: Execution = serde_json::from_str(
        observation
            .as_deref()
            .ok_or("cleanup process evidence absent")?,
    )
    .map_err(|_| "cleanup requires actual supervisor execution evidence")?;
    if state != "exited"
        || observed.state != "exited"
        || observed.binding != *b
        || observed.handle.is_empty()
        || observed.outcome.is_none()
    {
        return Err("cleanup live/unknown/identity mismatch; preserve workspace".into());
    }
    Ok(observed)
}
fn retire_workspace(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    a: &RetireArgs,
    operation: &str,
) -> Result<Value> {
    let (b, observed, v) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let b = owned_binding(&c, project, &a.attempt)?;
        let observed = stopped(&c, project, &a.attempt, &b)?;
        let current: String = db(c.query_row(
            "SELECT current_candidate FROM git_workspaces WHERE attempt=?1",
            [&a.attempt],
            |r| r.get(0),
        ))?;
        let v = candidate(&c, project, &current)?;
        (b, observed, v)
    };
    evidence(g, &v)?;
    let root = g.policy.attempts_root.join(&a.attempt);
    let retained: Option<CleanupAdmission> = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let raw:String=db(c.query_row("SELECT payload FROM operations WHERE id=?1 AND project=?2 AND kind='git-retire_workspace'",params![operation,project],|r|r.get(0)))?;
        let payload: Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        if payload.get("root_identity").is_some() {
            Some(decode(&payload)?)
        } else {
            None
        }
    };
    if let Some(admission) = retained {
        let (filesystem, retention, disposition): (String, String, String) = {
            let c = controller
                .connection
                .lock()
                .map_err(|_| "controller lock poisoned")?;
            db(c.query_row(
                "SELECT filesystem,retention,disposition FROM git_workspaces WHERE attempt=?1",
                [&a.attempt],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            ))?
        };
        if admission.attempt != a.attempt
            || admission.binding != b
            || admission.owner.binding != b
            || admission.owner.workspace_identity != filesystem
            || admission.retention != retention
            || admission.retire_refs != a.retire_refs
            || json!(admission.process) != json!(observed)
            || !matches!(disposition.as_str(), "cleanup-applying" | "retired")
        {
            return Err("cleanup original operation/owner/stop proof differs".into());
        }
        g.bytes(&admission.retention)?;
        g.bytes(&admission.manifest)?;
        let absent =
            fs::symlink_metadata(&root).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound);
        if !absent {
            g.retire_retained(&b, &admission.root_identity, &admission.manifest)?;
        }
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        db(c.execute(
            "UPDATE git_workspaces SET disposition='retired' WHERE attempt=?1",
            [&a.attempt],
        ))?;
        return Ok(
            json!({"state":"retired","attempt":a.attempt,"task":v.task,"retention":retention,"observed_execution":observed,"operation":operation,"lost_ack_reconciled":absent,"partial_cleanup_reconciled":!absent}),
        );
    }
    if fs::symlink_metadata(&root).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) {
        return Err(
            "cleanup missing root has no matching retained operation/owner/stop proof".into(),
        );
    }
    let (repo, owner) = g.owned(&b)?;
    let scratch = g.policy.root.join("captures").join(operation);
    if !scratch.exists() {
        fs::create_dir(&scratch).map_err(|e| e.to_string())?;
    }
    let (state, _) = g.inspect(&repo, &scratch, false)?;
    if !state.status.is_empty() {
        return Err("dirty/untracked workspace preserved; cleanup retryable".into());
    }
    if state.commit != v.commit
        || state
            .refs
            .iter()
            .any(|(r, oid)| r != "refs/heads/attempt" || oid != &v.commit)
        || a.retire_refs != ["refs/heads/attempt"]
    {
        return Err(
            "unowned/shared/changed refs preserved; explicit owned retirement required".into(),
        );
    }
    // Retain every private workspace/home/scratch/output byte before removal; tar
    // never follows worker symlinks. No Git config or hook is executed on the host.

    let retention = g.archive(&a.attempt)?;
    let root_identity = crate::git_workspace::identity(&root)?;
    let manifest = g.cleanup_manifest(&root)?;
    let admission = CleanupAdmission {
        attempt: a.attempt.clone(),
        binding: b.clone(),
        owner: owner.clone(),
        retention: retention.clone(),
        retire_refs: a.retire_refs.clone(),
        process: observed.clone(),
        root_identity: root_identity.clone(),
        manifest: manifest.clone(),
    };
    {
        let mut c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let tx = db(c.transaction_with_behavior(TransactionBehavior::Immediate))?;
        record_artifact(&tx, g, &retention)?;
        record_artifact(&tx, g, &manifest)?;
        db(tx.execute("UPDATE git_workspaces SET retention=?2,disposition='cleanup-applying' WHERE attempt=?1",params![a.attempt,retention]))?;
        db(tx.execute(
            "UPDATE operations SET payload=?2 WHERE id=?1",
            params![
                operation,
                serde_json::to_string(&admission).map_err(|e| e.to_string())?
            ],
        ))?;
        transition(
            &tx,
            operation,
            "applying",
            &json!({"retention":retention,"observed_stop":observed,"before_cleanup_effect":true}),
        )?;
        db(tx.commit())?;
    }
    fault("before_workspace_cleanup", operation)?;
    // Revalidate exact filesystem identities after retention and immediately before
    // deleting our exclusive root. Shared/live/unknown resources never reach here.
    g.owned(&b)?;
    #[cfg(feature = "fault-injection")]
    if std::env::var("CORENIUS_TEST_GIT_CRASH_POINT").as_deref() == Ok("partial_workspace_cleanup")
        && std::env::var("CORENIUS_TEST_GIT_CRASH_MARKER")
            .ok()
            .is_some_and(|p| Path::new(&p).exists())
    {
        // Real partial deletion under the admitted exclusive cleanup authority.
        fs::remove_dir_all(root.join("workspace/.git")).map_err(|e| e.to_string())?;
        fs::remove_file(root.join("workspace-owner.json")).map_err(|e| e.to_string())?;
        fault("partial_workspace_cleanup", operation)?;
    }
    g.retire_retained(&b, &root_identity, &manifest)?;
    fault("after_workspace_cleanup", operation)?;
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    db(c.execute(
        "UPDATE git_workspaces SET disposition='retired' WHERE attempt=?1",
        [&a.attempt],
    ))?;
    Ok(
        json!({"state":"retired","attempt":a.attempt,"task":v.task,"retention":retention,"observed_execution":observed,"operation":operation}),
    )
}
struct InventoryRecord {
    attempt: Value,
    binding: Binding,
    candidate: Option<String>,
    disposition: String,
    retention: Option<String>,
}
fn inventory_descriptor(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    value: Value,
) -> Result<Value> {
    let raw = value.to_string();
    if raw.len() <= 8192 {
        return Ok(value);
    }
    let digest = g.artifact(raw.as_bytes())?;
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    db(c.execute(
        "INSERT OR IGNORE INTO control_inventory_inputs VALUES(?1,?2,?3)",
        params![project, digest, raw.len() as i64],
    ))?;
    Ok(
        json!({"cursor":value["cursor"],"seq":value["seq"],"attempt":value["attempt"],"operation":value["operation"],"state":value["state"],"sha256":digest,"bytes":raw.len(),"input_reference":{"kind":"workspace_inventory","args":{"input":digest,"bytes":raw.len(),"offset":0}},"coverage":"complete observed record retained; follow exact JSON input reference"}),
    )
}
fn inventory(
    controller: &Controller,
    g: &GitBackend,
    project: &str,
    args: &Value,
) -> Result<Value> {
    let observed_at = std::time::Instant::now();
    let after =
        i64::try_from(args["after"].as_u64().unwrap_or(0)).map_err(|_| "inventory cursor bound")?;
    let history_after = i64::try_from(args["history_after"].as_u64().unwrap_or(0))
        .map_err(|_| "inventory history cursor bound")?;
    let limit = args["limit"].as_u64().unwrap_or(20).clamp(1, 20) as i64;
    // Retain a coherent database snapshot, then release the single connection
    // before running any untrusted repository observation.
    let (records, base, revision, mut history, more, workspace_count, history_count) = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        let mut attempts = query::rows(
            &c,
            "SELECT a.rowid AS cursor,a.* FROM attempts a WHERE a.project=?1 AND a.rowid>?2 ORDER BY a.rowid LIMIT ?3",
            &[&project, &after, &(limit + 1)],
        )?;
        let more = attempts.as_array().unwrap().len() > limit as usize;
        attempts.as_array_mut().unwrap().truncate(limit as usize);
        let mut records = Vec::new();
        for a in attempts.as_array().ok_or("attempt inventory invalid")? {
            let attempt = text(a, "id")?;
            let binding = owned_binding(&c, project, attempt)?;
            let saved:Option<(Option<String>,String,Option<String>)>=db(c.query_row("SELECT current_candidate,disposition,retention FROM git_workspaces WHERE attempt=?1",[attempt],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional())?;
            let (candidate, disposition, retention) =
                saved.unwrap_or((None, "not-captured".into(), None));
            records.push(InventoryRecord {
                attempt: a.clone(),
                binding,
                candidate,
                disposition,
                retention,
            });
        }
        let project_state: Option<(String, i64)> = db(c
            .query_row(
                "SELECT base,revision FROM projects WHERE id=?1",
                [project],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional())?;
        let history = query::rows(
            &c,
            "SELECT h.* FROM git_effect_history h JOIN operations o ON o.id=h.operation WHERE o.project=?1 AND h.seq>?2 ORDER BY h.seq LIMIT ?3",
            &[&project, &history_after, &(limit + 1)],
        )?;
        let workspace_count: i64 = db(c.query_row(
            "SELECT COUNT(*) FROM attempts WHERE project=?1",
            [project],
            |r| r.get(0),
        ))?;
        let history_count:i64=db(c.query_row("SELECT COUNT(*) FROM git_effect_history h JOIN operations o ON o.id=h.operation WHERE o.project=?1",[project],|r|r.get(0)))?;
        (
            records,
            project_state.as_ref().map(|v| v.0.clone()),
            project_state.map(|v| v.1),
            history,
            more,
            workspace_count,
            history_count,
        )
    };
    let history_more = history.as_array().unwrap().len() > limit as usize;
    history.as_array_mut().unwrap().truncate(limit as usize);
    let history_cursor = history
        .as_array()
        .unwrap()
        .last()
        .and_then(|h| h["seq"].as_i64())
        .unwrap_or(history_after);
    for entry in history.as_array_mut().unwrap() {
        *entry = inventory_descriptor(controller, g, project, entry.clone())?;
    }
    let mut workspaces = Vec::new();
    let mut workspace_cursor = after;
    for r in records {
        let a = &r.attempt;
        let b = &r.binding;
        let attempt = &b.attempt;
        let path = g.policy.attempts_root.join(attempt).join("workspace");
        let scratch = g
            .policy
            .root
            .join("captures")
            .join(format!("inventory-{}", id()));
        fs::create_dir(&scratch).map_err(|e| e.to_string())?;
        let inspection = if r.disposition == "retired" {
            Ok(None)
        } else {
            g.owned(b)
                .and_then(|(repo, _)| g.inspect(&repo, &scratch, false).map(|(state, _)| state))
                .map(Some)
        };
        let (state, drift) = match inspection {
            Ok(state) => (state, None),
            Err(e) => (None, Some(e)),
        };
        let process = a["observation"]
            .as_str()
            .and_then(|s| serde_json::from_str::<Value>(s).ok());
        workspace_cursor = a["cursor"]
            .as_i64()
            .ok_or("inventory workspace cursor missing")?;
        workspaces.push(inventory_descriptor(controller,g,project,json!({"cursor":workspace_cursor,"kind":"private-git-repository","path":path,"attempt":attempt,"base":b.base,"current_candidate":r.candidate,"refs":state.as_ref().map(|s|&s.refs),"dirty_untracked":state.as_ref().map(|s|&s.status),"commit":state.as_ref().map(|s|&s.commit),"inspection":state.as_ref().and_then(|s|s.inspection.as_ref()),"process":process,"observed_attempt_state":a["state"],"retention":r.retention,"cleanup_disposition":r.disposition,"ownership_drift":drift}))?);
    }
    let actual = g.head()?;
    let current_revision: Option<i64> = {
        let c = controller
            .connection
            .lock()
            .map_err(|_| "controller lock poisoned")?;
        db(c.query_row(
            "SELECT revision FROM projects WHERE id=?1",
            [project],
            |r| r.get(0),
        )
        .optional())?
    };
    Ok(
        json!({"configured":true,"backend":"git","jj_qualified":false,"policy":g.identity,"protected_ref":g.policy.protected_ref,"actual_trunk":actual,"project_base":base,"ref_drift":base.as_deref().is_some_and(|base|base!=actual),"snapshot_revision":revision,"current_revision":current_revision,"state_drift":revision!=current_revision,"observation_elapsed_ms":observed_at.elapsed().as_millis(),"workspaces":workspaces,"history":history,"coverage":{"workspace_records":workspace_count,"history_records":history_count,"physical_observations":workspaces.len(),"retention":"complete inputs retained; each page observes only its selected owned workspaces; unknown/drift never counts as clean"},"pagination":{"workspaces":{"after":workspace_cursor,"has_more":more},"history":{"after":history_cursor,"has_more":history_more},"continuation":{"kind":"workspace_inventory","args":{"after":workspace_cursor,"history_after":history_cursor,"limit":limit}}}}),
    )
}

/// Called within admission's short transaction; physical drift is checked outside
/// the transaction by the configured trusted backend before the request is applied.
pub(super) fn check_admission(c: &Connection, project: &str) -> Result<()> {
    let configured: bool = db(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='git_configuration')",
        [],
        |r| r.get(0),
    ))?;
    if configured {
        let unresolved:bool=db(c.query_row("SELECT EXISTS(SELECT 1 FROM operations WHERE project=?1 AND kind='git-land_candidate' AND state IN ('prepared','applying','unknown'))",[project],|r|r.get(0)))?;
        if unresolved {
            return Err("unresolved landing fences conflicting dispatch".into());
        }
    }
    Ok(())
}
pub(super) fn check_ref(controller: &Controller, request: &Value) -> Result<()> {
    if request["command"]["kind"] != "admit" {
        return Ok(());
    }
    let Some(g) = configured(controller)? else {
        return Ok(());
    };
    let project = request["project"].as_str().ok_or("project required")?;
    if project != g.policy.project {
        return Ok(());
    }
    let actual = g.head()?;
    let c = controller
        .connection
        .lock()
        .map_err(|_| "controller lock poisoned")?;
    let base: String = db(
        c.query_row("SELECT base FROM projects WHERE id=?1", [project], |r| {
            r.get(0)
        }),
    )?;
    if actual != base {
        return Err("protected ref/project base drift fences dispatch".into());
    }
    Ok(())
}
