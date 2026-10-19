use std::str::FromStr;
use std::sync::Arc;

use harp_contracts::{
    ArtifactRef, AttemptId, Budget, ExternalSessionId, OperationId, ResultEnvelope, ResultStatus,
    RunId, TaskGraph, TaskId, TaskNode, ThreadId, TurnId, TurnSpec,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::model::{
    checked_i64_to_u32, checked_i64_to_u64, checked_u64_to_i64, parse_id, parse_node_kind,
    parse_result_status, result_status_as_str, AcceptResult, ActivityPreparation,
    ActivityRecoveryRecord, AttemptRecord, AttemptState, CliActivityKind, CliActivityRecord,
    CliActivityState, CliAttemptRecord, CliAttemptState, CliSignalStage, CliTerminalFailureClass,
    EventPage, EventRecord, InterruptIntent, LeaseCapability, LeaseToken, OperationKind,
    OperationRecord, OperationState, RunState, TaskClaim, TaskRecord, TaskState,
    TurnOperationPreparation, UsageOutcome, WallUsageOutcome,
};
use crate::store::{bounded_json, insert_event};
use crate::{StateError, StateResult, StateStore};
use zeroize::Zeroizing;

const MAX_JSON_BYTES: usize = 256 * 1024;
const MAX_RECOVERY_ROWS: usize = 100_000;
const MAX_EVENT_PAGE_LIMIT: usize = 1_000;
const MAX_EVENT_PAGE_PAYLOAD_BYTES: usize = 8 * 1024 * 1024;

mod activities;
mod lease;
mod operations;
mod recovery;
mod results;
mod results_support;
mod scratch;
mod support;
mod terminal;
mod terminal_support;
mod turns;

use activities::update_cli_attempt_state;
use results_support::*;
use support::*;
use terminal_support::*;

pub(crate) fn load_attempt_record(
    connection: &Connection,
    attempt_id: &AttemptId,
) -> StateResult<Option<AttemptRecord>> {
    load_attempt(connection, attempt_id)
}

pub(crate) fn load_operations_for_attempt(
    connection: &Connection,
    attempt_id: &AttemptId,
) -> StateResult<Vec<OperationRecord>> {
    let mut statement = connection
        .prepare(
            "SELECT operation_id
             FROM operations
             WHERE attempt_id = ?1
             ORDER BY ordinal, operation_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare attempt operations", source))?;
    let rows = statement
        .query_map([attempt_id.to_string()], |row| row.get::<_, String>(0))
        .map_err(|source| StateError::sqlite("query attempt operations", source))?;
    let mut operation_ids = Vec::new();
    for row in rows {
        operation_ids.push(parse_id(
            "attempt operation id",
            row.map_err(|source| StateError::sqlite("read attempt operation id", source))?,
        )?);
        enforce_row_limit("attempt operations", operation_ids.len())?;
    }
    drop(statement);
    operation_ids
        .into_iter()
        .map(|operation_id| {
            load_operation(connection, &operation_id)?
                .ok_or_else(|| StateError::integrity("attempt operation disappeared during query"))
        })
        .collect()
}
