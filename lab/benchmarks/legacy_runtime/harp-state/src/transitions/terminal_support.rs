use super::*;

pub(super) fn transition_task_state(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    task_id: &TaskId,
    expected: TaskState,
    next: TaskState,
    now: i64,
) -> StateResult<()> {
    let changed = transaction
        .execute(
            "UPDATE tasks
             SET state = ?4, updated_at = ?5
             WHERE run_id = ?1
               AND task_id = ?2
               AND state = ?3
               AND accepted_attempt_id IS NULL",
            params![
                run_id.to_string(),
                task_id.to_string(),
                expected.as_str(),
                next.as_str(),
                now
            ],
        )
        .map_err(|source| StateError::sqlite("transition task state", source))?;
    require_changed(
        changed,
        format!("task {run_id}/{task_id}"),
        expected.as_str(),
        "changed before task CAS",
    )
}

pub(super) fn fail_run(transaction: &Transaction<'_>, run_id: &RunId, now: i64) -> StateResult<()> {
    let mut operation_statement = transaction
        .prepare(
            "SELECT o.operation_id, o.state
             FROM operations o
             JOIN attempts a ON a.attempt_id = o.attempt_id
             WHERE a.run_id = ?1
               AND o.state IN ('prepared', 'dispatching')
             ORDER BY o.operation_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare failed run operations", source))?;
    let operation_rows = operation_statement
        .query_map([run_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StateError::sqlite("query failed run operations", source))?;
    let mut operations = Vec::new();
    for row in operation_rows {
        let (operation_id, state) =
            row.map_err(|source| StateError::sqlite("read failed run operation", source))?;
        operations.push((
            parse_id("failed run operation id", operation_id)?,
            OperationState::parse(&state)?,
        ));
        enforce_row_limit("failed run operations", operations.len())?;
    }
    drop(operation_statement);
    for (operation_id, state) in &operations {
        update_operation_state(
            transaction,
            operation_id,
            *state,
            OperationState::Failed,
            now,
        )?;
    }

    let attempts = query_attempt_states(
        transaction,
        "SELECT attempt_id, state
         FROM attempts
         WHERE run_id = ?1
           AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
         ORDER BY task_id, ordinal, attempt_id
         LIMIT 100001",
        [run_id.to_string()],
        "failed run attempts",
    )?;
    for (attempt_id, state) in &attempts {
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'failed',
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = ?2",
                params![attempt_id.to_string(), state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("terminalize failed run attempt", source))?;
        require_changed(
            changed,
            format!("failed run attempt {attempt_id}"),
            state.as_str(),
            "changed before failure CAS",
        )?;
        terminalize_cli_extension(
            transaction,
            attempt_id,
            CliAttemptState::Failed,
            CliActivityState::Failed,
            CliTerminalFailureClass::NonResumableProtocolFailure,
            now,
        )?;
    }

    let mut task_statement = transaction
        .prepare(
            "SELECT task_id, state
             FROM tasks
             WHERE run_id = ?1
               AND state IN ('pending', 'ready', 'running', 'result_published')
             ORDER BY task_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare failed run tasks", source))?;
    let task_rows = task_statement
        .query_map([run_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StateError::sqlite("query failed run tasks", source))?;
    let mut tasks: Vec<(TaskId, TaskState)> = Vec::new();
    for row in task_rows {
        let (task_id, state) =
            row.map_err(|source| StateError::sqlite("read failed run task", source))?;
        tasks.push((
            parse_id("failed run task id", task_id)?,
            TaskState::parse(&state)?,
        ));
        enforce_row_limit("failed run tasks", tasks.len())?;
    }
    drop(task_statement);
    for (task_id, state) in &tasks {
        let changed = transaction
            .execute(
                "UPDATE tasks
                 SET state = 'failed', updated_at = ?4
                 WHERE run_id = ?1 AND task_id = ?2 AND state = ?3",
                params![run_id.to_string(), task_id.to_string(), state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("terminalize failed run task", source))?;
        require_changed(
            changed,
            format!("failed run task {run_id}/{task_id}"),
            state.as_str(),
            "changed before failure CAS",
        )?;
    }

    let changed = transaction
        .execute(
            "UPDATE runs
             SET state = 'failed', updated_at = ?2
             WHERE run_id = ?1 AND state = 'active'",
            params![run_id.to_string(), now],
        )
        .map_err(|source| StateError::sqlite("fail run", source))?;
    require_changed(
        changed,
        format!("run {run_id}"),
        RunState::Active.as_str(),
        "changed before failure CAS",
    )?;
    insert_event(
        transaction,
        Some(run_id),
        None,
        None,
        "run_failed_terminalized",
        &json!({
            "tasks": tasks.len(),
            "attempts": attempts.len(),
            "operations": operations.len(),
        }),
        now,
    )
}

pub(super) fn retry_allowed(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    task_id: &TaskId,
) -> StateResult<bool> {
    let max_transient_attempts: i64 = transaction
        .query_row(
            "SELECT max_transient_attempts
             FROM tasks WHERE run_id = ?1 AND task_id = ?2",
            params![run_id.to_string(), task_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("load task retry policy", source))?;
    let max_transient_attempts =
        checked_i64_to_u64("task max transient attempts", max_transient_attempts)?;
    let mut statement = transaction
        .prepare(
            "SELECT ordinal, state
             FROM attempts
             WHERE run_id = ?1
               AND task_id = ?2
             ORDER BY ordinal
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare retry attempt count", source))?;
    let rows = statement
        .query_map(params![run_id.to_string(), task_id.to_string()], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StateError::sqlite("query retry attempt count", source))?;
    let mut count = 0u64;
    for row in rows {
        let (ordinal, state) =
            row.map_err(|source| StateError::sqlite("read retry attempt state", source))?;
        let ordinal = checked_i64_to_u64("retry attempt ordinal", ordinal)?;
        if ordinal != count {
            return Err(StateError::integrity(format!(
                "task {run_id}/{task_id} attempt ordinals are not contiguous: expected {count}, got {ordinal}"
            )));
        }
        let state = AttemptState::parse(&state)?;
        if !matches!(state, AttemptState::Failed | AttemptState::Indeterminate) {
            return Err(StateError::integrity(format!(
                "task {run_id}/{task_id} retry history contains nonterminal state {}",
                state.as_str()
            )));
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| StateError::LimitExceeded {
                context: "retry attempt count".to_owned(),
                limit: u64::MAX,
                actual: u64::MAX,
            })?;
        enforce_row_limit(
            "retry attempts",
            usize::try_from(count).unwrap_or(usize::MAX),
        )?;
    }
    Ok(count <= max_transient_attempts)
}

pub(super) fn quarantine_competing_attempts(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    task_id: &TaskId,
    accepted_attempt_id: &AttemptId,
    now: i64,
) -> StateResult<()> {
    let competing = query_attempt_states(
        transaction,
        "SELECT attempt_id, state
         FROM attempts
         WHERE run_id = ?1
           AND task_id = ?2
           AND attempt_id <> ?3
           AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
         ORDER BY ordinal, attempt_id
         LIMIT 100001",
        params![
            run_id.to_string(),
            task_id.to_string(),
            accepted_attempt_id.to_string()
        ],
        "competing attempts",
    )?;
    for (attempt_id, state) in competing {
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'indeterminate',
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = ?2",
                params![attempt_id.to_string(), state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("quarantine competing attempt", source))?;
        require_changed(
            changed,
            format!("competing attempt {attempt_id}"),
            state.as_str(),
            "changed before acceptance CAS",
        )?;
        transition_nonterminal_operations(
            transaction,
            &attempt_id,
            OperationState::Indeterminate,
            now,
        )?;
        terminalize_cli_extension(
            transaction,
            &attempt_id,
            CliAttemptState::Indeterminate,
            CliActivityState::Indeterminate,
            CliTerminalFailureClass::Indeterminate,
            now,
        )?;
    }
    Ok(())
}

pub(super) fn cancel_competing_attempts(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    task_id: &TaskId,
    cancelled_attempt_id: &AttemptId,
    now: i64,
) -> StateResult<()> {
    let competing = query_attempt_states(
        transaction,
        "SELECT attempt_id, state
         FROM attempts
         WHERE run_id = ?1
           AND task_id = ?2
           AND attempt_id <> ?3
           AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
         ORDER BY ordinal, attempt_id
         LIMIT 100001",
        params![
            run_id.to_string(),
            task_id.to_string(),
            cancelled_attempt_id.to_string()
        ],
        "competing cancellation attempts",
    )?;
    for (attempt_id, state) in competing {
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'cancelled',
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = ?2",
                params![attempt_id.to_string(), state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("cancel competing attempt", source))?;
        require_changed(
            changed,
            format!("competing attempt {attempt_id}"),
            state.as_str(),
            "changed before cancellation CAS",
        )?;
        transition_nonterminal_operations(
            transaction,
            &attempt_id,
            OperationState::Cancelled,
            now,
        )?;
        terminalize_cli_extension(
            transaction,
            &attempt_id,
            CliAttemptState::Cancelled,
            CliActivityState::Cancelled,
            CliTerminalFailureClass::Cancelled,
            now,
        )?;
    }
    Ok(())
}

pub(super) fn activate_ready_dependents(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    now: i64,
) -> StateResult<usize> {
    let mut statement = transaction
        .prepare(
            "SELECT t.task_id
             FROM tasks t
             JOIN runs r ON r.run_id = t.run_id
             WHERE t.run_id = ?1
               AND t.state = 'pending'
               AND r.state = 'active'
               AND r.cancellation_requested = 0
               AND NOT EXISTS (
                   SELECT 1
                   FROM task_dependencies d
                   JOIN tasks dependency
                     ON dependency.run_id = d.run_id
                    AND dependency.task_id = d.depends_on_task_id
                   WHERE d.run_id = t.run_id
                     AND d.task_id = t.task_id
                     AND dependency.state <> 'completed'
               )
             ORDER BY t.task_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare ready dependents", source))?;
    let rows = statement
        .query_map([run_id.to_string()], |row| row.get::<_, String>(0))
        .map_err(|source| StateError::sqlite("query ready dependents", source))?;
    let mut task_ids: Vec<TaskId> = Vec::new();
    for row in rows {
        task_ids.push(parse_id(
            "ready dependent task id",
            row.map_err(|source| StateError::sqlite("read ready dependent", source))?,
        )?);
        enforce_row_limit("ready dependents", task_ids.len())?;
    }
    drop(statement);
    for task_id in &task_ids {
        let changed = transaction
            .execute(
                "UPDATE tasks
                 SET state = 'ready', updated_at = ?3
                 WHERE run_id = ?1
                   AND task_id = ?2
                   AND state = 'pending'
                   AND EXISTS (
                       SELECT 1 FROM runs
                       WHERE run_id = ?1
                         AND state = 'active'
                         AND cancellation_requested = 0
                   )",
                params![run_id.to_string(), task_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("activate ready dependent", source))?;
        require_changed(
            changed,
            format!("task {run_id}/{task_id}"),
            TaskState::Pending.as_str(),
            "changed before readiness CAS",
        )?;
    }
    Ok(task_ids.len())
}

pub(super) fn complete_run_if_finished(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    now: i64,
) -> StateResult<()> {
    let unfinished: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE run_id = ?1 AND state <> 'completed'",
            [run_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("count unfinished run tasks", source))?;
    if unfinished != 0 {
        return Ok(());
    }
    let changed = transaction
        .execute(
            "UPDATE runs
             SET state = 'completed', updated_at = ?2
             WHERE run_id = ?1
               AND state = 'active'
               AND cancellation_requested = 0",
            params![run_id.to_string(), now],
        )
        .map_err(|source| StateError::sqlite("complete run", source))?;
    require_changed(
        changed,
        format!("run {run_id}"),
        "active noncancelled with all tasks completed",
        "changed before completion CAS",
    )
}

pub(super) fn cancel_nonrunning_tasks(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    now: i64,
) -> StateResult<()> {
    let mut statement = transaction
        .prepare(
            "SELECT task_id, state
             FROM tasks
             WHERE run_id = ?1 AND state IN ('pending', 'ready', 'result_published')
             ORDER BY task_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare nonrunning cancellation", source))?;
    let rows = statement
        .query_map([run_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| StateError::sqlite("query nonrunning cancellation", source))?;
    let mut tasks: Vec<(TaskId, TaskState)> = Vec::new();
    for row in rows {
        let (task_id, state) =
            row.map_err(|source| StateError::sqlite("read nonrunning cancellation", source))?;
        tasks.push((
            parse_id("cancelled task id", task_id)?,
            TaskState::parse(&state)?,
        ));
        enforce_row_limit("nonrunning cancellation", tasks.len())?;
    }
    drop(statement);
    for (task_id, state) in tasks {
        if state == TaskState::ResultPublished {
            let attempts = query_attempt_states(
                transaction,
                "SELECT attempt_id, state
                 FROM attempts
                 WHERE run_id = ?1
                   AND task_id = ?2
                   AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
                 ORDER BY ordinal, attempt_id
                 LIMIT 100001",
                params![run_id.to_string(), task_id.to_string()],
                "published cancellation attempts",
            )?;
            for (attempt_id, attempt_state) in attempts {
                cancel_nonterminal_attempt(
                    transaction,
                    &attempt_id,
                    attempt_state,
                    now,
                    "published attempt",
                )?;
            }
        }
        let changed = transaction
            .execute(
                "UPDATE tasks
                 SET state = 'cancelled', updated_at = ?4
                 WHERE run_id = ?1 AND task_id = ?2 AND state = ?3",
                params![run_id.to_string(), task_id.to_string(), state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("cancel nonrunning task", source))?;
        require_changed(
            changed,
            format!("task {run_id}/{task_id}"),
            state.as_str(),
            "changed before cancellation CAS",
        )?;
    }
    Ok(())
}

pub(super) fn cancel_nonterminal_attempt(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    attempt_state: AttemptState,
    now: i64,
    context: &str,
) -> StateResult<()> {
    if attempt_state.is_terminal() {
        return Ok(());
    }
    transition_nonterminal_operations(transaction, attempt_id, OperationState::Cancelled, now)?;
    let changed = transaction
        .execute(
            "UPDATE attempts
             SET state = 'cancelled',
                 lease_owner = NULL,
                 lease_expires_at = NULL,
                 updated_at = ?3
             WHERE attempt_id = ?1 AND state = ?2",
            params![attempt_id.to_string(), attempt_state.as_str(), now],
        )
        .map_err(|source| StateError::sqlite("cancel nonterminal attempt", source))?;
    require_changed(
        changed,
        format!("{context} {attempt_id}"),
        attempt_state.as_str(),
        "changed before cancellation CAS",
    )?;
    terminalize_cli_extension(
        transaction,
        attempt_id,
        CliAttemptState::Cancelled,
        CliActivityState::Cancelled,
        CliTerminalFailureClass::Cancelled,
        now,
    )
}

pub(super) fn terminalize_cli_extension(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    attempt_state: CliAttemptState,
    activity_state: CliActivityState,
    failure_class: CliTerminalFailureClass,
    now: i64,
) -> StateResult<()> {
    let current_attempt_state: Option<String> = transaction
        .query_row(
            "SELECT state FROM cli_attempts WHERE attempt_id = ?1",
            [attempt_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| StateError::sqlite("load CLI terminal extension", source))?;
    let Some(current_attempt_state) = current_attempt_state else {
        return Ok(());
    };
    let current_attempt_state = CliAttemptState::parse(&current_attempt_state)?;
    if current_attempt_state.is_terminal() {
        if current_attempt_state == attempt_state {
            return Ok(());
        }
        return Err(StateError::Conflict {
            entity: format!("CLI attempt {attempt_id} terminal state"),
            expected: attempt_state.as_str().to_owned(),
            actual: current_attempt_state.as_str().to_owned(),
        });
    }
    let changed = transaction
        .execute(
            "UPDATE cli_attempts
             SET state = ?3,
                 terminal_failure_class = ?4,
                 updated_at = ?5
             WHERE attempt_id = ?1 AND state = ?2",
            params![
                attempt_id.to_string(),
                current_attempt_state.as_str(),
                attempt_state.as_str(),
                failure_class.as_str(),
                now
            ],
        )
        .map_err(|source| StateError::sqlite("terminalize CLI attempt extension", source))?;
    require_changed(
        changed,
        format!("CLI attempt {attempt_id}"),
        current_attempt_state.as_str(),
        "changed before CLI terminal CAS",
    )?;

    let mut statement = transaction
        .prepare(
            "SELECT activity_id, kind, state, process_record_sha256
             FROM cli_activities
             WHERE attempt_id = ?1
               AND state IN ('prepared', 'dispatching', 'running', 'reconciling')
             ORDER BY kind, ordinal, activity_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare CLI terminal activities", source))?;
    let rows = statement
        .query_map([attempt_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|source| StateError::sqlite("query CLI terminal activities", source))?;
    let mut activities = Vec::new();
    for row in rows {
        let (activity_id, kind, state, process_record_sha256) =
            row.map_err(|source| StateError::sqlite("read CLI terminal activity", source))?;
        activities.push((
            parse_id::<OperationId>("CLI terminal activity id", activity_id)?,
            CliActivityKind::parse(&kind)?,
            CliActivityState::parse(&state)?,
            process_record_sha256,
        ));
        enforce_row_limit("CLI terminal activities", activities.len())?;
    }
    drop(statement);
    for (activity_id, kind, current_state, process_record_sha256) in activities {
        let next_activity_state = if activity_state == CliActivityState::Failed
            && ((matches!(
                kind,
                CliActivityKind::StartActivity | CliActivityKind::ContinueActivity
            ) && process_record_sha256.is_none())
                || kind == CliActivityKind::InterruptActivity)
        {
            CliActivityState::Indeterminate
        } else {
            activity_state
        };
        let next_failure_class = if next_activity_state == CliActivityState::Indeterminate {
            CliTerminalFailureClass::Indeterminate
        } else {
            failure_class
        };
        let changed = transaction
            .execute(
                "UPDATE cli_activities
                 SET state = ?3,
                     terminal_failure_class = ?4,
                     signal_stage = CASE
                        WHEN kind = 'interrupt_activity' AND ?3 = 'indeterminate'
                            THEN 'indeterminate'
                        WHEN kind = 'interrupt_activity' AND ?3 = 'cancelled'
                            THEN 'indeterminate'
                        ELSE signal_stage
                     END,
                     updated_at = ?5
                 WHERE activity_id = ?1 AND state = ?2",
                params![
                    activity_id.to_string(),
                    current_state.as_str(),
                    next_activity_state.as_str(),
                    next_failure_class.as_str(),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("terminalize CLI activity", source))?;
        require_changed(
            changed,
            format!("CLI activity {activity_id}"),
            current_state.as_str(),
            "changed before CLI activity terminal CAS",
        )?;
    }
    Ok(())
}

pub(super) fn cancel_run_if_finished(
    transaction: &Transaction<'_>,
    run_id: &RunId,
    now: i64,
) -> StateResult<()> {
    let nonterminal: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM tasks
             WHERE run_id = ?1
               AND state NOT IN ('completed', 'failed', 'cancelled')",
            [run_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|source| StateError::sqlite("count cancellation tasks", source))?;
    if nonterminal != 0 {
        return Ok(());
    }
    let changed = transaction
        .execute(
            "UPDATE runs
             SET state = 'cancelled', updated_at = ?2
             WHERE run_id = ?1
               AND state = 'active'",
            params![run_id.to_string(), now],
        )
        .map_err(|source| StateError::sqlite("finalize cancelled run", source))?;
    require_changed(
        changed,
        format!("run {run_id}"),
        "active with all tasks terminal after cancellation",
        "changed before cancellation CAS",
    )
}
