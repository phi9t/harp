use super::*;

impl StateStore {
    pub(super) fn immediate(&mut self, context: &str) -> StateResult<Transaction<'_>> {
        self.connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StateError::sqlite(context, source))
    }
}

pub(super) fn aggregate_run_reservations(
    transaction: &Transaction<'_>,
    run_id: &RunId,
) -> StateResult<(u128, u128, u128)> {
    let mut statement = transaction
        .prepare(
            "SELECT b.reserved_tokens, b.reserved_storage_bytes,
                    b.reserved_wall_seconds, b.observed_tokens,
                    b.observed_storage_bytes, b.observed_wall_seconds,
                    a.state
             FROM budget_reservations b
             JOIN attempts a ON a.attempt_id = b.attempt_id
             WHERE a.run_id = ?1
             ORDER BY a.attempt_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare run reservations", source))?;
    let rows = statement
        .query_map([run_id.to_string()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|source| StateError::sqlite("query run reservations", source))?;
    let mut tokens = 0u128;
    let mut storage = 0u128;
    let mut wall = 0u128;
    let mut count = 0usize;
    for row in rows {
        let (
            reserved_tokens,
            reserved_storage,
            reserved_wall,
            observed_tokens,
            observed_storage,
            observed_wall,
            state,
        ) = row.map_err(|source| StateError::sqlite("read run reservation", source))?;
        let state = AttemptState::parse(&state)?;
        let active = !state.is_terminal();
        tokens = checked_add_aggregate(
            "aggregate token reservation",
            tokens,
            u128::from(if active {
                checked_i64_to_u64("reserved tokens", reserved_tokens)?
                    .max(checked_i64_to_u64("observed tokens", observed_tokens)?)
            } else {
                checked_i64_to_u64("observed tokens", observed_tokens)?
            }),
        )?;
        storage = checked_add_aggregate(
            "aggregate storage reservation",
            storage,
            u128::from(if active {
                checked_i64_to_u64("reserved storage", reserved_storage)?
                    .max(checked_i64_to_u64("observed storage", observed_storage)?)
            } else {
                checked_i64_to_u64("observed storage", observed_storage)?
            }),
        )?;
        wall = checked_add_aggregate(
            "aggregate wall reservation",
            wall,
            u128::from(if active {
                checked_i64_to_u64("reserved wall seconds", reserved_wall)?
                    .max(checked_i64_to_u64("observed wall seconds", observed_wall)?)
            } else {
                checked_i64_to_u64("observed wall seconds", observed_wall)?
            }),
        )?;
        count += 1;
        enforce_row_limit("run reservations", count)?;
    }
    Ok((tokens, storage, wall))
}

pub(super) fn checked_add_aggregate(
    context: &str,
    current: u128,
    value: u128,
) -> StateResult<u128> {
    current
        .checked_add(value)
        .ok_or_else(|| StateError::LimitExceeded {
            context: context.to_owned(),
            limit: u64::MAX,
            actual: u64::MAX,
        })
}

pub(super) fn charge_observed_wall(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    now: i64,
) -> StateResult<()> {
    let observed_wall: i64 = transaction
        .query_row(
            "SELECT observed_wall_seconds FROM attempts WHERE attempt_id = ?1",
            [attempt_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("load attempt wall charge", source))?;
    transaction
        .execute(
            "UPDATE budget_reservations
             SET observed_wall_seconds = MAX(observed_wall_seconds, ?2),
                 updated_at = ?3
             WHERE attempt_id = ?1",
            params![attempt_id.to_string(), observed_wall, now],
        )
        .map_err(|source| StateError::sqlite("charge attempt wall usage", source))?;
    Ok(())
}

pub(super) fn aggregate_run_wall_usage(
    transaction: &Transaction<'_>,
    run_id: &RunId,
) -> StateResult<u128> {
    let mut statement = transaction
        .prepare(
            "SELECT b.observed_wall_seconds
             FROM budget_reservations b
             JOIN attempts a ON a.attempt_id = b.attempt_id
             WHERE a.run_id = ?1
             ORDER BY b.attempt_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare run wall usage", source))?;
    let rows = statement
        .query_map([run_id.to_string()], |row| row.get::<_, i64>(0))
        .map_err(|source| StateError::sqlite("query run wall usage", source))?;
    let mut total = 0u128;
    let mut count = 0usize;
    for row in rows {
        total = checked_add_aggregate(
            "aggregate run wall usage",
            total,
            u128::from(checked_i64_to_u64(
                "attempt observed wall seconds",
                row.map_err(|source| StateError::sqlite("read run wall usage", source))?,
            )?),
        )?;
        count += 1;
        enforce_row_limit("run wall usage", count)?;
    }
    Ok(total)
}

pub(super) fn transition_nonterminal_operations(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    next: OperationState,
    now: i64,
) -> StateResult<()> {
    let mut statement = transaction
        .prepare(
            "SELECT operation_id, state
             FROM operations
             WHERE attempt_id = ?1
               AND state IN ('prepared', 'dispatching')
             ORDER BY operation_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare nonterminal operations", source))?;
    let rows = statement
        .query_map([attempt_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StateError::sqlite("query nonterminal operations", source))?;
    let mut operations = Vec::new();
    for row in rows {
        let (operation_id, state) =
            row.map_err(|source| StateError::sqlite("read nonterminal operation", source))?;
        operations.push((
            parse_id("nonterminal operation id", operation_id)?,
            OperationState::parse(&state)?,
        ));
        enforce_row_limit("nonterminal operations", operations.len())?;
    }
    drop(statement);
    for (operation_id, state) in operations {
        update_operation_state(transaction, &operation_id, state, next, now)?;
    }
    Ok(())
}

pub(super) fn query_attempt_states<P>(
    transaction: &Transaction<'_>,
    sql: &str,
    parameters: P,
    context: &str,
) -> StateResult<Vec<(AttemptId, AttemptState)>>
where
    P: rusqlite::Params,
{
    let mut statement = transaction
        .prepare(sql)
        .map_err(|source| StateError::sqlite(format!("prepare {context}"), source))?;
    let rows = statement
        .query_map(parameters, |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StateError::sqlite(format!("query {context}"), source))?;
    let mut attempts = Vec::new();
    for row in rows {
        let (attempt_id, state) =
            row.map_err(|source| StateError::sqlite(format!("read {context}"), source))?;
        attempts.push((
            parse_id("attempt id", attempt_id)?,
            AttemptState::parse(&state)?,
        ));
        enforce_row_limit(context, attempts.len())?;
    }
    Ok(attempts)
}

pub(super) fn require_task(
    connection: &Connection,
    run_id: &RunId,
    task_id: &TaskId,
) -> StateResult<TaskRecord> {
    load_task(connection, run_id, task_id)?.ok_or_else(|| StateError::Conflict {
        entity: format!("task {run_id}/{task_id}"),
        expected: "existing task".to_owned(),
        actual: "missing".to_owned(),
    })
}

pub(super) fn load_task(
    connection: &Connection,
    run_id: &RunId,
    task_id: &TaskId,
) -> StateResult<Option<TaskRecord>> {
    let raw: Option<(String, String, String, String, Option<String>)> = connection
        .query_row(
            "SELECT run_id, task_id, kind, state, accepted_attempt_id
             FROM tasks WHERE run_id = ?1 AND task_id = ?2",
            params![run_id.to_string(), task_id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()
        .map_err(|source| StateError::sqlite("query task", source))?;
    raw.map(|(run_id, task_id, kind, state, accepted_attempt_id)| {
        Ok(TaskRecord {
            run_id: parse_id("task run id", run_id)?,
            task_id: parse_id("task id", task_id)?,
            kind: parse_node_kind(&kind)?,
            state: TaskState::parse(&state)?,
            accepted_attempt_id: accepted_attempt_id
                .map(|value| parse_id("accepted attempt id", value))
                .transpose()?,
        })
    })
    .transpose()
}

pub(super) fn parse_bounded_json(context: &str, bytes: &[u8]) -> StateResult<Value> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(StateError::LimitExceeded {
            context: context.to_owned(),
            limit: MAX_JSON_BYTES as u64,
            actual: bytes.len() as u64,
        });
    }
    serde_json::from_slice(bytes).map_err(|source| StateError::Serialization {
        context: context.to_owned(),
        source,
    })
}

pub(super) fn enforce_row_limit(context: &str, rows: usize) -> StateResult<()> {
    if rows > MAX_RECOVERY_ROWS {
        return Err(StateError::LimitExceeded {
            context: context.to_owned(),
            limit: MAX_RECOVERY_ROWS as u64,
            actual: rows as u64,
        });
    }
    Ok(())
}

pub(super) fn validate_operation_preparation(
    connection: &Connection,
    attempt: &AttemptRecord,
    kind: OperationKind,
) -> StateResult<()> {
    if kind == OperationKind::InterruptTurn && attempt.state == AttemptState::DispatchingTurn {
        return Err(StateError::Conflict {
            entity: format!("attempt {} interrupt preparation", attempt.attempt_id),
            expected: "turn dispatch reconciled before interruption".to_owned(),
            actual: "dispatching_turn".to_owned(),
        });
    }
    let expected = match kind {
        OperationKind::StartThread => &[AttemptState::Prepared][..],
        OperationKind::StartTurn => &[AttemptState::ThreadStarted][..],
        OperationKind::ContinueTurn => &[AttemptState::TurnStarted, AttemptState::Reconciling][..],
        OperationKind::InterruptTurn => &[AttemptState::TurnStarted, AttemptState::Reconciling][..],
        OperationKind::PublishResult => &[
            AttemptState::Prepared,
            AttemptState::TurnStarted,
            AttemptState::Reconciling,
        ][..],
        OperationKind::Evaluate => &[AttemptState::ResultPublished][..],
    };
    require_attempt_state(attempt, expected)?;
    let active: i64 = connection
        .query_row(
            "SELECT COUNT(*)
             FROM runs
             WHERE run_id = ?1
               AND state = 'active'
               AND (?2 = 1 OR cancellation_requested = 0)",
            params![
                attempt.run_id.to_string(),
                i64::from(kind == OperationKind::InterruptTurn)
            ],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("check run operation authority", source))?;
    if active != 1 {
        return Err(StateError::Conflict {
            entity: format!("run {}", attempt.run_id),
            expected: "active and not cancellation requested".to_owned(),
            actual: "inactive or cancellation requested".to_owned(),
        });
    }
    if kind == OperationKind::ContinueTurn && attempt.latest_turn_id.is_none() {
        return Err(StateError::Integrity {
            context: "continuation requires persisted turn identity".to_owned(),
            source: None,
        });
    }
    Ok(())
}

pub(super) fn operation_target(
    attempt: &AttemptRecord,
    kind: OperationKind,
) -> StateResult<Option<String>> {
    match kind {
        OperationKind::InterruptTurn => attempt
            .latest_turn_id
            .as_ref()
            .map(ToString::to_string)
            .map(Some)
            .ok_or_else(|| StateError::Conflict {
                entity: format!("attempt {} interrupt target", attempt.attempt_id),
                expected: "latest turn id".to_owned(),
                actual: "missing".to_owned(),
            }),
        _ => Ok(None),
    }
}

pub(super) fn parse_task_node(bytes: &[u8]) -> StateResult<TaskNode> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(StateError::LimitExceeded {
            context: "persisted task JSON".to_owned(),
            limit: MAX_JSON_BYTES as u64,
            actual: bytes.len() as u64,
        });
    }
    let node: TaskNode =
        serde_json::from_slice(bytes).map_err(|source| StateError::Serialization {
            context: "persisted task JSON".to_owned(),
            source,
        })?;
    TaskGraph {
        schema_version: 1,
        nodes: vec![node.clone()],
    }
    .validate_shape()
    .map_err(|source| StateError::Integrity {
        context: "persisted task JSON failed contract validation".to_owned(),
        source: Some(source),
    })?;
    Ok(node)
}

pub(super) fn validate_label(context: &str, value: &str) -> StateResult<()> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(StateError::invalid(format!(
            "{context} must contain 1..=256 non-control bytes"
        )));
    }
    Ok(())
}

pub(super) fn validate_sha256(context: &str, value: &str) -> StateResult<()> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        return Err(StateError::invalid(format!(
            "{context} must be lowercase SHA-256"
        )));
    }
    Ok(())
}

pub(super) fn enforce_reservation_budget(
    dimension: &str,
    current: u128,
    requested: u128,
    limit: u128,
) -> StateResult<()> {
    let attempted = current
        .checked_add(requested)
        .ok_or_else(|| StateError::LimitExceeded {
            context: format!("{dimension} reservation sum"),
            limit: i64::MAX as u64,
            actual: u64::MAX,
        })?;
    if attempted > limit {
        let attempted = u64::try_from(attempted).map_err(|_| StateError::LimitExceeded {
            context: format!("{dimension} aggregate reservation"),
            limit: u64::MAX,
            actual: u64::MAX,
        })?;
        return Err(StateError::BudgetExceeded {
            dimension: dimension.to_owned(),
            limit: u64::try_from(limit).map_err(|_| StateError::LimitExceeded {
                context: format!("{dimension} reservation limit"),
                limit: u64::MAX,
                actual: u64::MAX,
            })?,
            attempted,
        });
    }
    Ok(())
}

pub(super) fn require_changed(
    changed: usize,
    entity: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> StateResult<()> {
    if changed == 1 {
        return Ok(());
    }
    Err(StateError::Conflict {
        entity: entity.into(),
        expected: expected.into(),
        actual: actual.into(),
    })
}

pub(super) fn require_attempt(
    connection: &Connection,
    attempt_id: &AttemptId,
) -> StateResult<AttemptRecord> {
    load_attempt(connection, attempt_id)?.ok_or_else(|| StateError::Conflict {
        entity: format!("attempt {attempt_id}"),
        expected: "existing attempt".to_owned(),
        actual: "missing".to_owned(),
    })
}

pub(super) fn require_operation(
    connection: &Connection,
    operation_id: &OperationId,
) -> StateResult<OperationRecord> {
    load_operation(connection, operation_id)?.ok_or_else(|| StateError::Conflict {
        entity: format!("operation {operation_id}"),
        expected: "existing operation".to_owned(),
        actual: "missing".to_owned(),
    })
}

pub(super) fn authorize_lease(
    transaction: &Transaction<'_>,
    lease: &LeaseToken,
    lease_now: i64,
    allow_cancellation_requested: bool,
) -> StateResult<AttemptRecord> {
    let attempt = require_attempt(transaction, lease.attempt_id())?;
    let stored_capability: Option<String> = transaction
        .query_row(
            "SELECT lease_capability_sha256 FROM attempts WHERE attempt_id = ?1",
            [lease.attempt_id().to_string()],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("load lease capability", source))?;
    let capability_sha256 = format!("{:x}", Sha256::digest(lease.capability().as_bytes()));
    if attempt.state.is_terminal() {
        return Err(StateError::Conflict {
            entity: format!("attempt {} authority", lease.attempt_id()),
            expected: "nonterminal attempt".to_owned(),
            actual: attempt.state.as_str().to_owned(),
        });
    }
    if attempt.lease_owner.as_deref() != Some(lease.owner())
        || attempt.lease_expires_at != Some(lease.expires_at())
        || lease.expires_at() <= lease_now
        || stored_capability.as_deref() != Some(capability_sha256.as_str())
    {
        return Err(StateError::Conflict {
            entity: format!("attempt {} lease", lease.attempt_id()),
            expected: format!("{}@{} and unexpired", lease.owner(), lease.expires_at()),
            actual: format!(
                "{:?}@{:?} at {lease_now}",
                attempt.lease_owner, attempt.lease_expires_at
            ),
        });
    }
    let run_authorized: i64 = transaction
        .query_row(
            "SELECT COUNT(*)
             FROM runs
             WHERE run_id = ?1
               AND state = 'active'
               AND (?2 = 1 OR cancellation_requested = 0)",
            params![
                attempt.run_id.to_string(),
                i64::from(allow_cancellation_requested)
            ],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("authorize lease run", source))?;
    if run_authorized != 1 {
        return Err(StateError::Conflict {
            entity: format!("run {} authority", attempt.run_id),
            expected: if allow_cancellation_requested {
                "active run".to_owned()
            } else {
                "active noncancelled run".to_owned()
            },
            actual: "inactive or cancellation requested".to_owned(),
        });
    }
    Ok(attempt)
}

pub(super) fn require_operation_attempt(
    operation: &OperationRecord,
    lease: &LeaseToken,
) -> StateResult<()> {
    if &operation.attempt_id == lease.attempt_id() {
        return Ok(());
    }
    Err(StateError::Conflict {
        entity: format!("operation {} attempt", operation.operation_id),
        expected: lease.attempt_id().to_string(),
        actual: operation.attempt_id.to_string(),
    })
}

pub(super) fn require_attempt_state(
    attempt: &AttemptRecord,
    expected: &[AttemptState],
) -> StateResult<()> {
    if expected.contains(&attempt.state) {
        return Ok(());
    }
    Err(StateError::Conflict {
        entity: format!("attempt {}", attempt.attempt_id),
        expected: expected
            .iter()
            .map(|state| state.as_str())
            .collect::<Vec<_>>()
            .join("|"),
        actual: attempt.state.as_str().to_owned(),
    })
}

pub(super) fn require_operation_state(
    operation: &OperationRecord,
    expected: OperationState,
) -> StateResult<()> {
    if operation.state == expected {
        return Ok(());
    }
    Err(StateError::Conflict {
        entity: format!("operation {}", operation.operation_id),
        expected: expected.as_str().to_owned(),
        actual: operation.state.as_str().to_owned(),
    })
}

pub(super) fn require_operation_kind(
    operation: &OperationRecord,
    expected: &[OperationKind],
) -> StateResult<()> {
    if expected.contains(&operation.kind) {
        return Ok(());
    }
    Err(StateError::Conflict {
        entity: format!("operation {} kind", operation.operation_id),
        expected: expected
            .iter()
            .map(|kind| kind.as_str())
            .collect::<Vec<_>>()
            .join("|"),
        actual: operation.kind.as_str().to_owned(),
    })
}

pub(super) fn update_operation_state(
    transaction: &Transaction<'_>,
    operation_id: &OperationId,
    expected: OperationState,
    next: OperationState,
    now: i64,
) -> StateResult<()> {
    let changed = transaction
        .execute(
            "UPDATE operations
             SET state = ?3, updated_at = ?4
             WHERE operation_id = ?1 AND state = ?2",
            params![
                operation_id.to_string(),
                expected.as_str(),
                next.as_str(),
                now
            ],
        )
        .map_err(|source| StateError::sqlite("update operation state", source))?;
    require_changed(
        changed,
        format!("operation {operation_id}"),
        expected.as_str(),
        "changed before CAS",
    )
}

pub(super) fn update_attempt_state(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    expected: AttemptState,
    next: AttemptState,
    now: i64,
) -> StateResult<()> {
    let changed = transaction
        .execute(
            "UPDATE attempts
             SET state = ?3, updated_at = ?4
             WHERE attempt_id = ?1 AND state = ?2",
            params![
                attempt_id.to_string(),
                expected.as_str(),
                next.as_str(),
                now
            ],
        )
        .map_err(|source| StateError::sqlite("update attempt state", source))?;
    require_changed(
        changed,
        format!("attempt {attempt_id}"),
        expected.as_str(),
        "changed before CAS",
    )
}

pub(super) fn load_attempt(
    connection: &Connection,
    attempt_id: &AttemptId,
) -> StateResult<Option<AttemptRecord>> {
    let raw = connection
        .query_row(
            "SELECT attempt_id, run_id, task_id, ordinal, state,
                    lease_owner, lease_expires_at, last_lease_expires_at,
                    thread_id, latest_turn_id,
                    latest_operation_marker, scratch_path, scratch_device, scratch_inode,
                    wall_started_at, wall_last_observed_at, observed_wall_seconds,
                    semantic_failure_class,
                    continuation_count, observed_tokens,
                    latest_turn_observed_tokens,
                    result_sha256, result_status, result_token_usage
             FROM attempts WHERE attempt_id = ?1",
            [attempt_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<i64>>(12)?,
                    row.get::<_, Option<i64>>(13)?,
                    row.get::<_, Option<i64>>(14)?,
                    row.get::<_, Option<i64>>(15)?,
                    row.get::<_, i64>(16)?,
                    row.get::<_, Option<String>>(17)?,
                    row.get::<_, i64>(18)?,
                    row.get::<_, i64>(19)?,
                    row.get::<_, i64>(20)?,
                    row.get::<_, Option<String>>(21)?,
                    row.get::<_, Option<String>>(22)?,
                    row.get::<_, Option<i64>>(23)?,
                ))
            },
        )
        .optional()
        .map_err(|source| StateError::sqlite("query attempt", source))?;
    raw.map(
        |(
            attempt_id,
            run_id,
            task_id,
            ordinal,
            state,
            lease_owner,
            lease_expires_at,
            last_lease_expires_at,
            thread_id,
            turn_id,
            operation_marker,
            scratch_path,
            scratch_device,
            scratch_inode,
            wall_started_at,
            wall_last_observed_at,
            observed_wall_seconds,
            semantic_failure_class,
            continuation_count,
            observed_tokens,
            latest_turn_observed_tokens,
            result_sha256,
            result_status,
            result_token_usage,
        )| {
            if lease_owner.is_some() != lease_expires_at.is_some() {
                return Err(StateError::integrity(
                    "attempt lease owner and expiry presence differ",
                ));
            }
            Ok(AttemptRecord {
                attempt_id: parse_id("attempt id", attempt_id)?,
                run_id: parse_id("attempt run id", run_id)?,
                task_id: parse_id("attempt task id", task_id)?,
                ordinal: checked_i64_to_u32("attempt ordinal", ordinal)?,
                state: AttemptState::parse(&state)?,
                lease_owner,
                lease_expires_at,
                last_lease_expires_at,
                thread_id: thread_id
                    .map(|value| {
                        ThreadId::from_str(&value)
                            .map_err(|source| StateError::integrity_contract("thread id", source))
                    })
                    .transpose()?,
                latest_turn_id: turn_id
                    .map(|value| {
                        TurnId::from_str(&value)
                            .map_err(|source| StateError::integrity_contract("turn id", source))
                    })
                    .transpose()?,
                latest_operation_marker: operation_marker,
                scratch_path,
                scratch_device: scratch_device
                    .map(|value| checked_i64_to_u64("attempt scratch device", value))
                    .transpose()?,
                scratch_inode: scratch_inode
                    .map(|value| checked_i64_to_u64("attempt scratch inode", value))
                    .transpose()?,
                wall_started_at,
                wall_last_observed_at,
                observed_wall_seconds: checked_i64_to_u64(
                    "attempt observed wall seconds",
                    observed_wall_seconds,
                )?,
                semantic_failure_class,
                continuation_count: checked_i64_to_u64(
                    "attempt continuation count",
                    continuation_count,
                )?,
                observed_tokens: checked_i64_to_u64("attempt observed tokens", observed_tokens)?,
                latest_turn_observed_tokens: checked_i64_to_u64(
                    "attempt latest turn observed tokens",
                    latest_turn_observed_tokens,
                )?,
                result_sha256,
                result_status: result_status
                    .map(|value| parse_result_status(&value))
                    .transpose()?,
                result_token_usage: result_token_usage
                    .map(|value| checked_i64_to_u64("attempt result token usage", value))
                    .transpose()?,
            })
        },
    )
    .transpose()
}

pub(super) fn load_cli_attempt(
    connection: &Connection,
    attempt_id: &AttemptId,
) -> StateResult<Option<CliAttemptRecord>> {
    let raw = connection
        .query_row(
            "SELECT attempt_id, state, logical_session_id, external_session_id,
                    continuation_count, terminal_failure_class, created_at, updated_at
             FROM cli_attempts WHERE attempt_id = ?1",
            [attempt_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            },
        )
        .optional()
        .map_err(|source| StateError::sqlite("query CLI attempt", source))?;
    raw.map(
        |(
            stored_attempt_id,
            state,
            logical_session_id,
            external_session_id,
            continuation_count,
            terminal_failure_class,
            created_at,
            updated_at,
        )| {
            let stored_attempt_id = parse_id("CLI attempt id", stored_attempt_id)?;
            if stored_attempt_id != *attempt_id {
                return Err(StateError::integrity(
                    "CLI attempt identity does not match query",
                ));
            }
            let state = CliAttemptState::parse(&state)?;
            let terminal_failure_class = terminal_failure_class
                .map(|value| CliTerminalFailureClass::parse(&value))
                .transpose()?;
            validate_cli_attempt_terminal_failure(state, terminal_failure_class)?;
            let continuation_count =
                checked_i64_to_u32("CLI attempt continuation count", continuation_count)?;
            if continuation_count > 1 {
                return Err(StateError::integrity(
                    "CLI attempt continuation count exceeds one",
                ));
            }
            Ok(CliAttemptRecord {
                attempt_id: stored_attempt_id,
                state,
                logical_session_id: ThreadId::from_str(&logical_session_id).map_err(|source| {
                    StateError::integrity_contract("CLI logical session id", source)
                })?,
                external_session_id: external_session_id
                    .map(|value| {
                        ExternalSessionId::from_str(&value).map_err(|source| {
                            StateError::integrity_contract("CLI external session id", source)
                        })
                    })
                    .transpose()?,
                continuation_count,
                terminal_failure_class,
                created_at,
                updated_at,
            })
        },
    )
    .transpose()
}

pub(super) fn load_cli_activity(
    connection: &Connection,
    activity_id: &OperationId,
) -> StateResult<Option<CliActivityRecord>> {
    let raw = connection
        .query_row(
            "SELECT activity_id, attempt_id, kind, ordinal, state,
                    logical_turn_id, activity_dir, invocation_sha256,
                    process_record_sha256, terminal_failure_class,
                    interrupt_purpose, target_process_record_sha256, signal_stage,
                    created_at, updated_at
             FROM cli_activities WHERE activity_id = ?1",
            [activity_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, i64>(13)?,
                    row.get::<_, i64>(14)?,
                ))
            },
        )
        .optional()
        .map_err(|source| StateError::sqlite("query CLI activity", source))?;
    raw.map(
        |(
            stored_activity_id,
            attempt_id,
            kind,
            ordinal,
            state,
            logical_turn_id,
            activity_dir,
            invocation_sha256,
            process_record_sha256,
            terminal_failure_class,
            interrupt_purpose,
            target_process_record_sha256,
            signal_stage,
            created_at,
            updated_at,
        )| {
            let stored_activity_id: OperationId = parse_id("CLI activity id", stored_activity_id)?;
            if stored_activity_id != *activity_id {
                return Err(StateError::integrity(
                    "CLI activity identity does not match query",
                ));
            }
            let logical_turn_id = TurnId::from_str(&logical_turn_id)
                .map_err(|source| StateError::integrity_contract("CLI logical turn id", source))?;
            if logical_turn_id.to_string() != stored_activity_id.to_string() {
                return Err(StateError::integrity(
                    "CLI activity logical turn id differs from activity id",
                ));
            }
            validate_persisted_activity_dir(&activity_dir)?;
            validate_persisted_sha256("CLI invocation digest", &invocation_sha256)?;
            if let Some(value) = process_record_sha256.as_deref() {
                validate_persisted_sha256("CLI process record digest", value)?;
            }
            if let Some(value) = target_process_record_sha256.as_deref() {
                validate_persisted_sha256("CLI target process record digest", value)?;
            }
            let kind = CliActivityKind::parse(&kind)?;
            let state = CliActivityState::parse(&state)?;
            let terminal_failure_class = terminal_failure_class
                .map(|value| CliTerminalFailureClass::parse(&value))
                .transpose()?;
            let interrupt_purpose = interrupt_purpose
                .map(|value| crate::InterruptPurpose::parse(&value))
                .transpose()?;
            let signal_stage = signal_stage
                .map(|value| CliSignalStage::parse(&value))
                .transpose()?;
            validate_cli_activity_fields(
                kind,
                state,
                process_record_sha256.as_deref(),
                terminal_failure_class,
                interrupt_purpose,
                target_process_record_sha256.as_deref(),
                signal_stage,
            )?;
            Ok(CliActivityRecord {
                activity_id: stored_activity_id,
                attempt_id: parse_id("CLI activity attempt id", attempt_id)?,
                kind,
                ordinal: checked_i64_to_u32("CLI activity ordinal", ordinal)?,
                state,
                logical_turn_id,
                activity_dir,
                invocation_sha256,
                process_record_sha256,
                terminal_failure_class,
                interrupt_purpose,
                target_process_record_sha256,
                signal_stage,
                created_at,
                updated_at,
            })
        },
    )
    .transpose()
}

fn validate_cli_attempt_terminal_failure(
    state: CliAttemptState,
    failure: Option<CliTerminalFailureClass>,
) -> StateResult<()> {
    let valid = match state {
        CliAttemptState::Prepared
        | CliAttemptState::Running
        | CliAttemptState::Reconciling
        | CliAttemptState::Succeeded => failure.is_none(),
        CliAttemptState::Failed => matches!(
            failure,
            Some(
                CliTerminalFailureClass::NonResumableProtocolFailure
                    | CliTerminalFailureClass::ContinuationExhausted
            )
        ),
        CliAttemptState::Indeterminate => failure == Some(CliTerminalFailureClass::Indeterminate),
        CliAttemptState::Cancelled => failure == Some(CliTerminalFailureClass::Cancelled),
    };
    if valid {
        Ok(())
    } else {
        Err(StateError::integrity(
            "CLI attempt state and terminal failure class disagree",
        ))
    }
}

fn validate_cli_activity_fields(
    kind: CliActivityKind,
    state: CliActivityState,
    process_record_sha256: Option<&str>,
    failure: Option<CliTerminalFailureClass>,
    interrupt_purpose: Option<crate::InterruptPurpose>,
    target_process_record_sha256: Option<&str>,
    signal_stage: Option<CliSignalStage>,
) -> StateResult<()> {
    let terminal_valid = match state {
        CliActivityState::Prepared
        | CliActivityState::Dispatching
        | CliActivityState::Running
        | CliActivityState::Reconciling
        | CliActivityState::Completed => failure.is_none(),
        CliActivityState::Failed => matches!(
            failure,
            Some(
                CliTerminalFailureClass::ResumableInterrupted
                    | CliTerminalFailureClass::ResumableCliFailure
                    | CliTerminalFailureClass::NonResumableProtocolFailure
                    | CliTerminalFailureClass::ContinuationExhausted
            )
        ),
        CliActivityState::Indeterminate => failure == Some(CliTerminalFailureClass::Indeterminate),
        CliActivityState::Cancelled => failure == Some(CliTerminalFailureClass::Cancelled),
    };
    if !terminal_valid {
        return Err(StateError::integrity(
            "CLI activity state and terminal failure class disagree",
        ));
    }
    match kind {
        CliActivityKind::StartActivity | CliActivityKind::ContinueActivity => {
            if interrupt_purpose.is_some()
                || target_process_record_sha256.is_some()
                || signal_stage.is_some()
            {
                return Err(StateError::integrity(
                    "non-interrupt CLI activity carries interrupt-only fields",
                ));
            }
            if matches!(
                state,
                CliActivityState::Running
                    | CliActivityState::Reconciling
                    | CliActivityState::Completed
                    | CliActivityState::Failed
            ) && process_record_sha256.is_none()
            {
                return Err(StateError::integrity(
                    "spawned CLI activity is missing its process record digest",
                ));
            }
        }
        CliActivityKind::InterruptActivity => {
            if process_record_sha256.is_some()
                || interrupt_purpose.is_none()
                || target_process_record_sha256.is_none()
                || signal_stage.is_none()
            {
                return Err(StateError::integrity(
                    "interrupt CLI activity fields are incomplete",
                ));
            }
            let signal_stage = signal_stage.expect("checked above");
            let valid_pair = match state {
                CliActivityState::Prepared => matches!(
                    signal_stage,
                    CliSignalStage::Prepared
                        | CliSignalStage::SigintPrepared
                        | CliSignalStage::SigintSent
                        | CliSignalStage::SigtermPrepared
                        | CliSignalStage::SigtermSent
                        | CliSignalStage::SigkillPrepared
                        | CliSignalStage::SigkillSent
                ),
                CliActivityState::Completed => signal_stage == CliSignalStage::Quiescent,
                CliActivityState::Indeterminate | CliActivityState::Cancelled => {
                    signal_stage == CliSignalStage::Indeterminate
                }
                CliActivityState::Dispatching
                | CliActivityState::Running
                | CliActivityState::Reconciling
                | CliActivityState::Failed => false,
            };
            if !valid_pair {
                return Err(StateError::integrity(
                    "interrupt CLI activity state and signal stage disagree",
                ));
            }
        }
    }
    Ok(())
}

fn validate_persisted_activity_dir(value: &str) -> StateResult<()> {
    if value.is_empty()
        || value.len() > 4096
        || !value.starts_with('/')
        || value.as_bytes().contains(&0)
    {
        return Err(StateError::integrity(
            "persisted CLI activity directory is not bounded and absolute",
        ));
    }
    Ok(())
}

fn validate_persisted_sha256(context: &str, value: &str) -> StateResult<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(StateError::integrity(format!(
            "persisted {context} is not lowercase SHA-256"
        )))
    }
}

pub(super) fn load_operation(
    connection: &Connection,
    operation_id: &OperationId,
) -> StateResult<Option<OperationRecord>> {
    let raw = connection
        .query_row(
            "SELECT operation_id, attempt_id, kind, ordinal, state,
                    external_id, target_external_id, operation_marker,
                    intent_json, intent_sha256, checkpoint_sha256,
                    consumed_at, created_at, updated_at
             FROM operations WHERE operation_id = ?1",
            [operation_id.to_string()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<Vec<u8>>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, Option<String>>(10)?,
                    row.get::<_, Option<i64>>(11)?,
                    row.get::<_, i64>(12)?,
                    row.get::<_, i64>(13)?,
                ))
            },
        )
        .optional()
        .map_err(|source| StateError::sqlite("query operation", source))?;
    raw.map(
        |(
            operation_id,
            attempt_id,
            kind,
            ordinal,
            state,
            external_id,
            target_external_id,
            operation_marker,
            intent_json,
            intent_sha256,
            checkpoint_sha256,
            consumed_at,
            created_at,
            updated_at,
        )| {
            let kind = OperationKind::parse(&kind)?;
            let (turn_intent, interrupt_intent) = match (kind, intent_json) {
                (OperationKind::StartTurn | OperationKind::ContinueTurn, Some(bytes)) => (
                    Some(
                        serde_json::from_slice::<TurnSpec>(&parse_bounded_intent_bytes(
                            "turn operation intent",
                            bytes,
                        )?)
                        .map_err(|source| StateError::Serialization {
                            context: "turn operation intent".to_owned(),
                            source,
                        })?,
                    ),
                    None,
                ),
                (OperationKind::InterruptTurn, Some(bytes)) => {
                    let intent = serde_json::from_slice::<InterruptIntent>(
                        &parse_bounded_intent_bytes("interrupt operation intent", bytes)?,
                    )
                    .map_err(|source| StateError::Serialization {
                        context: "interrupt operation intent".to_owned(),
                        source,
                    })?;
                    intent.validate()?;
                    (None, Some(intent))
                }
                (_, None) => (None, None),
                (_, Some(_)) => {
                    return Err(StateError::integrity(
                        "operation kind cannot carry persisted intent",
                    ));
                }
            };
            let canonical_intent = match (&turn_intent, &interrupt_intent) {
                (Some(intent), None) => Some(bounded_json(
                    "turn operation intent",
                    intent,
                    MAX_JSON_BYTES,
                )?),
                (None, Some(intent)) => Some(bounded_json(
                    "interrupt operation intent",
                    intent,
                    MAX_JSON_BYTES,
                )?),
                (None, None) => None,
                (Some(_), Some(_)) => {
                    return Err(StateError::integrity(
                        "operation cannot carry multiple intent types",
                    ));
                }
            };
            if let (Some(bytes), Some(expected)) = (&canonical_intent, &intent_sha256) {
                let actual = format!("{:x}", Sha256::digest(bytes));
                if &actual != expected {
                    return Err(StateError::integrity(
                        "operation intent digest does not match persisted bytes",
                    ));
                }
            } else if canonical_intent.is_some() != intent_sha256.is_some() {
                return Err(StateError::integrity(
                    "operation intent bytes and digest presence differ",
                ));
            }
            let operation_state = OperationState::parse(&state)?;
            if matches!(kind, OperationKind::StartTurn | OperationKind::ContinueTurn)
                && canonical_intent.is_none()
            {
                return Err(StateError::integrity(
                    "intent-bearing operation is missing persisted intent",
                ));
            }
            if kind == OperationKind::InterruptTurn
                && matches!(
                    operation_state,
                    OperationState::Prepared | OperationState::Dispatching
                )
                && interrupt_intent.is_none()
            {
                return Err(StateError::integrity(
                    "nonterminal interrupt operation is missing persisted intent",
                ));
            }
            Ok(OperationRecord {
                operation_id: parse_id("operation id", operation_id)?,
                attempt_id: parse_id("operation attempt id", attempt_id)?,
                kind,
                ordinal: checked_i64_to_u32("operation ordinal", ordinal)?,
                state: operation_state,
                external_id,
                target_external_id,
                operation_marker,
                turn_intent,
                interrupt_intent,
                intent_sha256,
                checkpoint_sha256,
                consumed_at,
                created_at,
                updated_at,
            })
        },
    )
    .transpose()
}

fn parse_bounded_intent_bytes(context: &str, bytes: Vec<u8>) -> StateResult<Vec<u8>> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err(StateError::LimitExceeded {
            context: context.to_owned(),
            limit: MAX_JSON_BYTES as u64,
            actual: bytes.len() as u64,
        });
    }
    Ok(bytes)
}

pub(super) fn load_operation_by_key(
    connection: &Connection,
    attempt_id: &AttemptId,
    kind: OperationKind,
    ordinal: u32,
) -> StateResult<Option<OperationRecord>> {
    let operation_id: Option<String> = connection
        .query_row(
            "SELECT operation_id
             FROM operations
             WHERE attempt_id = ?1 AND kind = ?2 AND ordinal = ?3",
            params![attempt_id.to_string(), kind.as_str(), i64::from(ordinal)],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| StateError::sqlite("query operation idempotency key", source))?;
    operation_id
        .map(|value| {
            let operation_id = parse_id("operation idempotency id", value)?;
            load_operation(connection, &operation_id)?.ok_or_else(|| {
                StateError::integrity("operation idempotency key points to missing operation")
            })
        })
        .transpose()
}
