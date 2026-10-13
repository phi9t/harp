//! Controller-owned adapter. Every journal command commits before the next external effect.
//! Only this adapter translates authenticated supervisor identities into lifecycle observations.
use crate::{
    Controller,
    supervisor::{self, Binding, Execution},
};
use serde_json::{Value, json};
use std::path::PathBuf;
use uuid::Uuid;

pub struct Runtime {
    socket: PathBuf,
    token: String,
    operator_token: String,
}
impl Runtime {
    pub fn new(socket: PathBuf, token: String, operator_token: String) -> Result<Self, String> {
        if !socket.is_absolute() || token.len() < 12 || operator_token.len() < 12 {
            return Err("absolute supervisor socket and private secret required".into());
        }
        Ok(Self {
            socket,
            token,
            operator_token,
        })
    }
    /// The daemon drains its own journal under retained operator authority. CLI, lead and
    /// outer observers issue admission/cancellation only, never supervisor scheduling calls.
    pub fn drain(&self, controller: &Controller) -> Result<(), String> {
        let auth = json!({"principal":"operator","token":self.operator_token});
        let projects = controller.handle(
            json!({"auth":auth,"project":"runtime","query":{"kind":"runtime_projects","args":{}}}),
        )?;
        for project in projects.as_array().ok_or("project projection missing")? {
            if let Err(error) =
                self.reconcile(controller, &json!({"auth":auth,"project":project["id"]}))
            {
                eprintln!("runtime reconcile: {error}");
            }
        }
        Ok(())
    }
    /// Repeated reconciliation polls current, server-derived state; callers cannot supply
    /// an execution identity, principal, profile, host path or terminal observation.
    pub fn reconcile(&self, controller: &Controller, request: &Value) -> Result<Value, String> {
        let auth = request.get("auth").ok_or("authentication required")?;
        let project = request
            .get("project")
            .and_then(Value::as_str)
            .ok_or("project required")?;
        let query = |kind: &str, args: Value| {
            controller
                .handle(json!({"auth":auth,"project":project,"query":{"kind":kind,"args":args}}))
        };
        // Operator-only query authenticates current authority before any runtime access.
        let snapshot = query("runtime_context", json!({}))?;
        let mut observed = Vec::new();
        for attempt in snapshot["attempts"]
            .as_array()
            .ok_or("attempt projection missing")?
        {
            if attempt["state"] == "exited" {
                continue;
            }
            let attempt_id = attempt["id"].as_str().ok_or("attempt identity missing")?;
            let start = snapshot["operations"]
                .as_array()
                .ok_or("operation projection missing")?
                .iter()
                .find(|op| op["attempt"] == attempt_id && op["kind"] == "start")
                .ok_or("start journal missing")?;
            let payload: Value = serde_json::from_str(
                start["payload"]
                    .as_str()
                    .ok_or("operation payload missing")?,
            )
            .map_err(|e| e.to_string())?;
            let binding = Binding {
                project: project.into(),
                attempt: attempt_id.into(),
                operation: start["id"].as_str().ok_or("start identity missing")?.into(),
                scope_epoch: payload["scope_epoch"]
                    .as_i64()
                    .ok_or("authority epoch missing")?,
                base: payload["base"]
                    .as_str()
                    .ok_or("pinned base missing")?
                    .into(),
                brief: payload["brief"]
                    .as_str()
                    .ok_or("pinned brief missing")?
                    .into(),
            };
            let cancelled = attempt["cancel_requested"] == true;
            let lookup = if !cancelled {
                Some(supervisor::call(
                    &self.socket,
                    &self.token,
                    "lookup",
                    &binding,
                ))
            } else {
                None
            };
            if !cancelled
                && snapshot["project"]["paused"] == true
                && matches!(lookup, Some(Ok(None)))
            {
                observed.push(
                    json!({"attempt":attempt_id,"state":attempt["state"],"dispatch_paused":true}),
                );
                continue;
            }
            let cancellation = snapshot["operations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|op| op["attempt"] == attempt_id && op["kind"] == "cancel");
            let mut start_state = start["state"]
                .as_str()
                .ok_or("operation state missing")?
                .to_string();
            let mut cancel_state = cancellation
                .and_then(|op| op["state"].as_str())
                .unwrap_or("")
                .to_string();
            let outcome = (|| -> Result<Execution, String> {
                if cancelled {
                    let cancel = cancellation.ok_or("cancel intent missing")?;
                    if matches!(cancel_state.as_str(), "prepared" | "unknown") {
                        journal(
                            controller,
                            auth,
                            project,
                            "transition_operation",
                            json!({"operation":cancel["id"],"state":"applying","observation":"identity-bound supervisor cancellation pending"}),
                        )?;
                        cancel_state = "applying".into();
                    }
                    // Never dispatch a revoked grant. The effect owner creates an absence
                    // tombstone if cancellation overtakes a prepared start.
                    supervisor::call(&self.socket, &self.token, "cancel", &binding)?
                        .ok_or("cancellation identity absent".into())
                } else {
                    match lookup.ok_or("lookup missing")?? {
                        Some(execution) => {
                            let current = query("runtime_context", json!({"attempt":attempt_id}))?;
                            let current = current["attempts"]
                                .as_array()
                                .ok_or("attempt projection missing")?
                                .iter()
                                .find(|a| a["id"] == attempt_id)
                                .ok_or("attempt missing")?;
                            if current["cancel_requested"] == true
                                || current["scope_epoch"] != binding.scope_epoch
                            {
                                return Err("current observation authority revoked".into());
                            }
                            Ok(execution)
                        }
                        None => {
                            // The current dispatch check belongs to the actual absence branch;
                            // lookup errors never authorize a second lookup or a paused effect.
                            let grant = query("dispatch_context", json!({"attempt":attempt_id}))?;
                            if grant["scope_epoch"] != binding.scope_epoch {
                                return Err("current dispatch authority mismatch".into());
                            }
                            if start_state != "prepared"
                                && start_state != "applying"
                                && start_state != "unknown"
                            {
                                return Err(
                                    "acknowledged effect identity missing; dispatch refused".into(),
                                );
                            }
                            if start_state != "applying" {
                                journal(
                                    controller,
                                    auth,
                                    project,
                                    "transition_operation",
                                    json!({"operation":binding.operation,"state":"applying","observation":"durable start intent before supervisor effect"}),
                                )?;
                                start_state = "applying".into();
                            }
                            supervisor::call(&self.socket, &self.token, "start", &binding)?
                                .ok_or("start identity absent".into())
                        }
                    }
                }
            })();
            let (state, observation) = match outcome {
                Ok(execution) => {
                    // Once retained, an effect's immutable identity cannot be replaced by a
                    // new reply under the same operation. Preserve capacity on any mismatch.
                    for previous in [start.get("observation"), attempt.get("observation")]
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
                        if let Ok(retained) = serde_json::from_str::<Execution>(previous)
                            && (retained.binding != execution.binding
                                || retained.handle != execution.handle
                                || retained.incarnation != execution.incarnation
                                || retained.boot_id != execution.boot_id
                                || retained.pid != execution.pid
                                || retained.start_ticks != execution.start_ticks
                                || retained.namespace_init != execution.namespace_init
                                || retained.init_start_ticks != execution.init_start_ticks
                                || retained.pid_namespace != execution.pid_namespace
                                || retained.native_store != execution.native_store)
                        {
                            journal(
                                controller,
                                auth,
                                project,
                                "observe_attempt",
                                json!({"attempt":attempt_id,"state":"unknown","observation":"retained supervisor execution identity mismatch"}),
                            )?;
                            return Err(
                                "retained supervisor execution identity mismatch; capacity held"
                                    .into(),
                            );
                        }
                    }
                    let observation =
                        serde_json::to_string(&execution).map_err(|e| e.to_string())?;
                    if execution.state == "unknown" {
                        if matches!(start_state.as_str(), "prepared" | "applying") {
                            journal(
                                controller,
                                auth,
                                project,
                                "transition_operation",
                                json!({"operation":binding.operation,"state":"unknown","observation":observation}),
                            )?;
                        }
                        if let Some(cancel) = cancellation
                            && cancel_state == "applying"
                        {
                            journal(
                                controller,
                                auth,
                                project,
                                "transition_operation",
                                json!({"operation":cancel["id"],"state":"unknown","observation":observation}),
                            )?;
                        }
                    } else {
                        if matches!(start_state.as_str(), "prepared" | "unknown") {
                            journal(
                                controller,
                                auth,
                                project,
                                "transition_operation",
                                json!({"operation":binding.operation,"state":"applying","observation":"reconciled retained supervisor operation identity"}),
                            )?;
                            start_state = "applying".into();
                        }
                        if start_state == "applying" {
                            journal(
                                controller,
                                auth,
                                project,
                                "transition_operation",
                                json!({"operation":binding.operation,"state":"confirmed","observation":observation}),
                            )?;
                        }
                        if execution.state == "exited"
                            && let Some(cancel) = cancellation
                            && cancel_state == "applying"
                        {
                            journal(
                                controller,
                                auth,
                                project,
                                "transition_operation",
                                json!({"operation":cancel["id"],"state":"confirmed","observation":observation}),
                            )?;
                        }
                    }
                    (execution.state, observation)
                }
                Err(error) => {
                    if matches!(start_state.as_str(), "prepared" | "applying") {
                        journal(
                            controller,
                            auth,
                            project,
                            "transition_operation",
                            json!({"operation":binding.operation,"state":"unknown","observation":error}),
                        )?;
                    }
                    if let Some(cancel) = cancellation
                        && matches!(cancel_state.as_str(), "prepared" | "applying")
                    {
                        journal(
                            controller,
                            auth,
                            project,
                            "transition_operation",
                            json!({"operation":cancel["id"],"state":"unknown","observation":error}),
                        )?;
                    }
                    (
                        "unknown".into(),
                        format!("unconfirmed supervisor identity: {error}"),
                    )
                }
            };
            if attempt["state"] != state || attempt["observation"] != observation {
                journal(
                    controller,
                    auth,
                    project,
                    "observe_attempt",
                    json!({"attempt":attempt_id,"state":state,"observation":observation}),
                )?;
            }
            observed.push(json!({"attempt":attempt_id,"state":state,"observation":observation}));
        }
        Ok(json!({"observed":observed}))
    }
}
fn journal(
    controller: &Controller,
    auth: &Value,
    project: &str,
    kind: &str,
    args: Value,
) -> Result<Value, String> {
    let state = controller
        .handle(json!({"auth":auth,"project":project,"query":{"kind":"snapshot","args":{}}}))?;
    controller.handle(json!({"auth":auth,"project":project,"command":{"kind":kind,"expected_revision":state["revision"],"idempotency_key":Uuid::new_v4().to_string(),"args":args}}))
}
