//! Trace-dialect interpretation, isolated from indexing and publication.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::index::DivergenceCounts;

#[derive(Default)]
pub(crate) struct IndexState {
    pub(crate) session_id: Option<String>,
    pub(crate) top_type_counts: BTreeMap<String, u64>,
    pub(crate) event_msg_counts: BTreeMap<String, u64>,
    pub(crate) hm_item_types: BTreeMap<String, u64>,
    pub(crate) hm_append: u64,
    pub(crate) hm_replace: u64,
    pub(crate) turns_started: u64,
    pub(crate) turns_completed: u64,
    pub(crate) turns_aborted: u64,
    pub(crate) exec_command_end: u64,
    pub(crate) collab_spawn_end: u64,
    pub(crate) divergence: DivergenceCounts,
    pub(crate) hm_call_ids: BTreeSet<String>,
    pub(crate) exec_call_ids: BTreeSet<String>,
}

pub(crate) fn ingest_traecli_dual_stream(record: &Value, state: &mut IndexState) {
    let top = record
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    *state.top_type_counts.entry(top.to_owned()).or_insert(0) += 1;
    let payload = record.get("payload").unwrap_or(&Value::Null);

    match top {
        "session_meta" => {
            if state.session_id.is_none() {
                state.session_id = payload.get("id").and_then(Value::as_str).map(str::to_owned);
            }
        }
        "event_msg" => ingest_event(payload, state),
        "history_mutation" => ingest_history_mutation(payload, state),
        _ => {}
    }
}

fn ingest_event(payload: &Value, state: &mut IndexState) {
    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    *state
        .event_msg_counts
        .entry(event_type.to_owned())
        .or_insert(0) += 1;
    match event_type {
        "task_started" => state.turns_started += 1,
        "task_complete" => state.turns_completed += 1,
        "turn_aborted" => state.turns_aborted += 1,
        "exec_command_end" => {
            state.exec_command_end += 1;
            if let Some(call_id) = payload.get("call_id").and_then(Value::as_str) {
                state.exec_call_ids.insert(call_id.to_owned());
            }
            let exit = payload
                .get("exit_code")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let empty = ["aggregated_output", "stdout", "stderr"].iter().all(|key| {
                payload
                    .get(*key)
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
            });
            if exit != 0 && empty {
                state.divergence.exec_empty_output_nonzero += 1;
            }
        }
        "collab_agent_spawn_end" => {
            state.collab_spawn_end += 1;
            record_missing_turn_id(payload, state);
        }
        "collab_waiting_end" | "collab_close_end" | "collab_agent_interaction_end" => {
            record_missing_turn_id(payload, state);
        }
        _ => {}
    }
}

fn record_missing_turn_id(payload: &Value, state: &mut IndexState) {
    if payload.get("turn_id").and_then(Value::as_str).is_none() {
        state.divergence.collab_events_missing_turn_id += 1;
    }
}

fn ingest_history_mutation(payload: &Value, state: &mut IndexState) {
    match payload.get("operation").and_then(Value::as_str) {
        Some("append") => state.hm_append += 1,
        Some("replace") => state.hm_replace += 1,
        _ => {}
    }
    if let Some(items) = payload.get("items").and_then(Value::as_array) {
        for item in items {
            let item_type = item
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            *state.hm_item_types.entry(item_type.to_owned()).or_insert(0) += 1;
            if item_type == "function_call" {
                if let Some(call_id) = item.get("call_id").and_then(Value::as_str) {
                    state.hm_call_ids.insert(call_id.to_owned());
                }
            }
        }
    }
}
