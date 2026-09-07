//! Explanations of persisted facts, without runtime observation or recovery effects.
use harp_contracts::{AttemptId, RunId, TaskId};
use harp_state::{RunInspection, RunState, TaskState};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionAction {
    None,
    Wait,
    Resume,
    ReviewFailure,
}

#[derive(Debug, Serialize)]
pub struct TaskExplanation {
    pub task_id: TaskId,
    pub state: String,
    pub accepted_attempt_id: Option<AttemptId>,
    pub blocked_by: Vec<TaskId>,
    pub action: InspectionAction,
    pub reason: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AttemptExplanation {
    pub attempt_id: AttemptId,
    pub task_id: TaskId,
    pub ordinal: u32,
    pub state: String,
    pub semantic_failure_class: Option<String>,
    pub cli_failure_class: Option<String>,
    pub lease_expires_at: Option<i64>,
    pub observed_tokens: u64,
    pub observed_wall_seconds: u64,
    pub continuation_count: u64,
    pub result_sha256: Option<String>,
    pub checkpoints: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectionEvent {
    pub sequence: u64,
    pub task_id: Option<TaskId>,
    pub attempt_id: Option<AttemptId>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowInspection {
    pub schema_version: u8,
    pub run_id: RunId,
    pub state: String,
    pub cancellation_requested: bool,
    pub graph_sha256: String,
    pub observed_at: i64,
    pub recovery_precondition: &'static str,
    pub tasks: Vec<TaskExplanation>,
    pub attempts: Vec<AttemptExplanation>,
    pub events: Vec<InspectionEvent>,
    pub next_after_sequence: Option<i64>,
}

/// This report is advisory. Only Engine::resume_run can admit recovery effects.
pub fn explain_run(snapshot: RunInspection, task_id: Option<&TaskId>) -> WorkflowInspection {
    let run = &snapshot.execution.run;
    // Resume currently waits for any live recoverable activity owner in the run.
    // An inspection can conservatively wait for any nonterminal attempt lease.
    let live_lease = snapshot.attempts.iter().any(|entry| {
        !entry.attempt.state.is_terminal()
            && entry
                .attempt
                .lease_expires_at
                .is_some_and(|expiry| expiry > snapshot.observed_at)
    });
    let tasks = snapshot
        .tasks
        .iter()
        .filter(|task| task_id.is_none_or(|id| id == &task.task_id))
        .map(|task| {
            let blocked_by = snapshot
                .execution
                .graph
                .nodes
                .iter()
                .find(|node| node.task_id == task.task_id)
                .into_iter()
                .flat_map(|node| &node.dependencies)
                .filter(|dependency| {
                    !snapshot.tasks.iter().any(|other| {
                        &other.task_id == *dependency && other.state == TaskState::Completed
                    })
                })
                .cloned()
                .collect::<Vec<_>>();
            let (action, reason) = if task.state == TaskState::Completed {
                (InspectionAction::None, "result.accepted")
            } else if run.state != RunState::Active {
                (InspectionAction::ReviewFailure, "run.terminal")
            } else if live_lease {
                (InspectionAction::Wait, "recovery.live_lease")
            } else if run.cancellation_requested {
                (InspectionAction::Resume, "recovery.cancellation_pending")
            } else {
                match task.state {
                    TaskState::Failed | TaskState::Cancelled => {
                        (InspectionAction::ReviewFailure, "task.terminal")
                    }
                    TaskState::Pending if !blocked_by.is_empty() => {
                        (InspectionAction::Wait, "task.dependencies_pending")
                    }
                    TaskState::Pending | TaskState::Ready => {
                        (InspectionAction::Resume, "recovery.schedule_pending")
                    }
                    TaskState::Running | TaskState::ResultPublished => {
                        (InspectionAction::Resume, "recovery.reconciliation_required")
                    }
                    TaskState::Completed => (InspectionAction::None, "result.accepted"),
                }
            };
            TaskExplanation {
                task_id: task.task_id.clone(),
                state: task.state.as_str().to_owned(),
                accepted_attempt_id: task.accepted_attempt_id.clone(),
                blocked_by,
                action,
                reason,
            }
        })
        .collect();
    let attempts = snapshot
        .attempts
        .into_iter()
        .filter(|entry| task_id.is_none_or(|id| id == &entry.attempt.task_id))
        .map(|entry| {
            let attempt = entry.attempt;
            let continuation_count = entry
                .cli
                .as_ref()
                .map_or(attempt.continuation_count, |cli| {
                    u64::from(cli.continuation_count)
                });
            let mut checkpoints = entry
                .operations
                .into_iter()
                .filter_map(|operation| operation.checkpoint_sha256)
                .collect::<Vec<_>>();
            checkpoints.sort();
            checkpoints.dedup();
            AttemptExplanation {
                attempt_id: attempt.attempt_id,
                task_id: attempt.task_id,
                ordinal: attempt.ordinal,
                state: attempt.state.as_str().to_owned(),
                semantic_failure_class: attempt.semantic_failure_class,
                cli_failure_class: entry
                    .cli
                    .and_then(|cli| cli.terminal_failure_class)
                    .map(|class| class.as_str().to_owned()),
                lease_expires_at: attempt.lease_expires_at,
                observed_tokens: attempt.observed_tokens,
                observed_wall_seconds: attempt.observed_wall_seconds,
                continuation_count,
                result_sha256: attempt.result_sha256,
                checkpoints,
            }
        })
        .collect();
    WorkflowInspection {
        schema_version: 1, run_id: run.run_id.clone(), state: run.state.as_str().to_owned(),
        cancellation_requested: run.cancellation_requested,
        graph_sha256: run.graph_sha256.clone(), observed_at: snapshot.observed_at,
        recovery_precondition: "Resume must revalidate pinned authorities, ownership, budgets and process state. Inspection does not authorize replay or verify artifact bytes.",
        tasks, attempts,
        events: snapshot.events.events.into_iter().map(|event| InspectionEvent {
            sequence: event.sequence, task_id: event.task_id, attempt_id: event.attempt_id,
            event_type: event.event_type, payload: event.payload, timestamp: event.timestamp,
        }).collect(),
        next_after_sequence: snapshot.events.next_after_sequence,
    }
}

impl WorkflowInspection {
    pub fn render_text(&self) -> String {
        use std::fmt::Write;
        let mut text = format!("Run {}: {}\n", self.run_id, self.state);
        for task in &self.tasks {
            let action = match task.action {
                InspectionAction::None => "none",
                InspectionAction::Wait => "wait",
                InspectionAction::Resume => "resume",
                InspectionAction::ReviewFailure => "review_failure",
            };
            let _ = writeln!(
                text,
                "{}: {}; next={action}; reason={}",
                task.task_id, task.state, task.reason
            );
            if !task.blocked_by.is_empty() {
                let _ = writeln!(
                    text,
                    "  blocked by: {}",
                    task.blocked_by
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            for attempt in self
                .attempts
                .iter()
                .filter(|attempt| attempt.task_id == task.task_id)
            {
                let _ = writeln!(
                    text,
                    "  attempt {} ({}): {}; semantic_failure={}; cli_failure={}",
                    attempt.ordinal,
                    attempt.attempt_id,
                    attempt.state,
                    attempt
                        .semantic_failure_class
                        .as_deref()
                        .unwrap_or("unrecorded"),
                    attempt.cli_failure_class.as_deref().unwrap_or("unrecorded")
                );
                if let Some(result) = &attempt.result_sha256 {
                    let _ = writeln!(text, "    result: {result}");
                }
            }
        }
        for event in &self.events {
            // JSON escaping keeps provider-derived control characters out of the terminal.
            let payload = event.payload.to_string();
            let mut end = payload.len().min(512);
            while !payload.is_char_boundary(end) {
                end -= 1;
            }
            let suffix = if end < payload.len() {
                "... [full payload in JSON]"
            } else {
                ""
            };
            let _ = writeln!(
                text,
                "event {}: {} {}{suffix}",
                event.sequence,
                event.event_type,
                &payload[..end]
            );
        }
        if let Some(cursor) = self.next_after_sequence {
            let _ = writeln!(text, "More evidence: --after-sequence {cursor}");
        }
        text.push_str(self.recovery_precondition);
        text
    }
}
