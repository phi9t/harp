use super::*;

impl StateStore {
    pub fn start_wall_tracking(
        &mut self,
        lease: &LeaseToken,
        wall_started_at: i64,
        now: i64,
    ) -> StateResult<AttemptRecord> {
        let lease_now = self.lease_now(now)?;
        if wall_started_at < 0 {
            return Err(StateError::invalid(
                "wall tracking start must be a nonnegative Unix timestamp",
            ));
        }
        let transaction = self.immediate("begin wall tracking")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        if let Some(existing) = attempt.wall_started_at {
            if existing != wall_started_at {
                return Err(StateError::Conflict {
                    entity: format!("attempt {} wall tracking start", attempt.attempt_id),
                    expected: existing.to_string(),
                    actual: wall_started_at.to_string(),
                });
            }
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit idempotent wall tracking", source))?;
            return Ok(attempt);
        }
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET wall_started_at = ?2,
                     wall_last_observed_at = ?2,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND wall_started_at IS NULL",
                params![attempt.attempt_id.to_string(), wall_started_at, now],
            )
            .map_err(|source| StateError::sqlite("persist wall tracking start", source))?;
        require_changed(
            changed,
            format!("attempt {} wall tracking start", attempt.attempt_id),
            "unstarted",
            "changed before wall tracking CAS",
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "wall_tracking_started",
            &json!({"wallStartedAt": wall_started_at}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit wall tracking start", source))?;
        Ok(AttemptRecord {
            wall_started_at: Some(wall_started_at),
            wall_last_observed_at: Some(wall_started_at),
            ..attempt
        })
    }

    pub fn reconcile_wall_usage(
        &mut self,
        lease: &LeaseToken,
        expected_previous_wall_seconds: u64,
        new_wall_seconds: u64,
        wall_observed_at: i64,
        now: i64,
    ) -> StateResult<WallUsageOutcome> {
        if wall_observed_at < 0 {
            return Err(StateError::invalid(
                "wall observation must be a nonnegative Unix timestamp",
            ));
        }
        let lease_now = self.lease_now(now)?;
        if new_wall_seconds < expected_previous_wall_seconds {
            return Err(StateError::invalid(
                "new wall usage must not decrease from expected usage",
            ));
        }
        let expected = checked_u64_to_i64(
            "expected previous wall seconds",
            expected_previous_wall_seconds,
        )?;
        let new_wall = checked_u64_to_i64("new wall seconds", new_wall_seconds)?;
        let transaction = self.immediate("begin wall usage reconciliation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        if attempt.observed_wall_seconds != expected_previous_wall_seconds {
            return Err(StateError::Conflict {
                entity: format!("attempt {} wall usage", attempt.attempt_id),
                expected: expected_previous_wall_seconds.to_string(),
                actual: attempt.observed_wall_seconds.to_string(),
            });
        }
        if attempt
            .wall_last_observed_at
            .is_some_and(|previous| wall_observed_at < previous)
        {
            return Err(StateError::Clock {
                context: format!(
                    "wall clock moved backward for attempt {}",
                    attempt.attempt_id
                ),
                source: None,
            });
        }
        let reserved_wall: i64 = transaction
            .query_row(
                "SELECT reserved_wall_seconds
                 FROM budget_reservations
                 WHERE attempt_id = ?1 AND observed_wall_seconds = ?2",
                params![lease.attempt_id().to_string(), expected],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("load wall usage reservation", source))?;
        if new_wall_seconds > expected_previous_wall_seconds {
            let changed = transaction
                .execute(
                    "UPDATE budget_reservations
                     SET observed_wall_seconds = ?3, updated_at = ?4
                     WHERE attempt_id = ?1 AND observed_wall_seconds = ?2",
                    params![lease.attempt_id().to_string(), expected, new_wall, now],
                )
                .map_err(|source| StateError::sqlite("update wall usage reservation", source))?;
            require_changed(
                changed,
                format!("attempt {} wall reservation", lease.attempt_id()),
                expected_previous_wall_seconds.to_string(),
                "changed before wall usage CAS",
            )?;
            let changed = transaction
                .execute(
                    "UPDATE attempts
                     SET observed_wall_seconds = ?3,
                         wall_last_observed_at = ?4,
                         updated_at = ?5
                     WHERE attempt_id = ?1 AND observed_wall_seconds = ?2",
                    params![
                        lease.attempt_id().to_string(),
                        expected,
                        new_wall,
                        wall_observed_at,
                        now
                    ],
                )
                .map_err(|source| StateError::sqlite("update attempt wall usage", source))?;
            require_changed(
                changed,
                format!("attempt {} wall usage", lease.attempt_id()),
                expected_previous_wall_seconds.to_string(),
                "changed before wall usage CAS",
            )?;
        }
        if new_wall_seconds == expected_previous_wall_seconds {
            transaction
                .execute(
                    "UPDATE attempts
                     SET wall_last_observed_at = ?2, updated_at = ?3
                     WHERE attempt_id = ?1
                       AND (wall_last_observed_at IS NULL OR wall_last_observed_at <= ?2)",
                    params![lease.attempt_id().to_string(), wall_observed_at, now],
                )
                .map_err(|source| StateError::sqlite("update wall observation", source))?;
        }
        let max_run_wall: i64 = transaction
            .query_row(
                "SELECT max_wall_seconds FROM runs WHERE run_id = ?1",
                [attempt.run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("load run wall limit", source))?;
        let observed_run_wall = aggregate_run_wall_usage(&transaction, &attempt.run_id)?;
        let reserved_wall = checked_i64_to_u64("reserved wall seconds", reserved_wall)?;
        let max_run_wall = checked_i64_to_u64("run max wall seconds", max_run_wall)?;
        let outcome = WallUsageOutcome {
            observed_attempt_wall_seconds: new_wall_seconds,
            observed_run_wall_seconds: observed_run_wall,
            attempt_limit_exceeded: new_wall_seconds > reserved_wall,
            run_limit_exceeded: observed_run_wall > u128::from(max_run_wall),
        };
        if new_wall_seconds > expected_previous_wall_seconds {
            insert_event(
                &transaction,
                Some(&attempt.run_id),
                Some(&attempt.task_id),
                Some(lease.attempt_id()),
                "wall_usage_reconciled",
                &json!({
                    "previousWallSeconds": expected_previous_wall_seconds,
                    "wallSeconds": new_wall_seconds,
                    "reservedWallSeconds": reserved_wall,
                    "observedRunWallSeconds": observed_run_wall.to_string(),
                    "maxRunWallSeconds": max_run_wall,
                }),
                now,
            )?;
        }
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit wall usage reconciliation", source))?;
        Ok(outcome)
    }

    pub fn register_artifact(&mut self, artifact: &ArtifactRef, now: i64) -> StateResult<()> {
        artifact.validate().map_err(StateError::invalid_contract)?;
        let size_bytes = checked_u64_to_i64("artifact size", artifact.size_bytes)?;
        let metadata_json = bounded_json("artifact metadata", artifact, MAX_JSON_BYTES)?;
        let transaction = self.immediate("begin artifact registration")?;
        register_artifact_metadata(&transaction, artifact, size_bytes, &metadata_json, now)?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit artifact registration", source))
    }

    pub fn reconcile_usage(
        &mut self,
        lease: &LeaseToken,
        expected_previous_tokens: u64,
        new_total_tokens: u64,
        new_storage_bytes: u64,
        now: i64,
    ) -> StateResult<UsageOutcome> {
        let lease_now = self.lease_now(now)?;
        if new_total_tokens < expected_previous_tokens {
            return Err(StateError::invalid(
                "new total tokens must not decrease from expected usage",
            ));
        }
        let expected = checked_u64_to_i64("expected previous tokens", expected_previous_tokens)?;
        let new_tokens = checked_u64_to_i64("new total tokens", new_total_tokens)?;
        let new_storage = checked_u64_to_i64("new storage bytes", new_storage_bytes)?;
        let transaction = self.immediate("begin usage reconciliation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let previous_storage: i64 = transaction
            .query_row(
                "SELECT observed_storage_bytes
                 FROM budget_reservations WHERE attempt_id = ?1",
                [lease.attempt_id().to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("load previous storage usage", source))?;
        if new_storage < previous_storage {
            return Err(StateError::invalid("new storage usage must not decrease"));
        }
        let changed = transaction
            .execute(
                "UPDATE budget_reservations
                 SET observed_tokens = ?3,
                     observed_storage_bytes = ?4,
                     updated_at = ?5
                 WHERE attempt_id = ?1
                   AND observed_tokens = ?2
                   AND observed_storage_bytes <= ?4",
                params![
                    lease.attempt_id().to_string(),
                    expected,
                    new_tokens,
                    new_storage,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("update usage reservation", source))?;
        require_changed(
            changed,
            format!("attempt {} usage", lease.attempt_id()),
            expected_previous_tokens.to_string(),
            attempt.observed_tokens.to_string(),
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET observed_tokens = ?3, updated_at = ?4
                 WHERE attempt_id = ?1
                   AND observed_tokens = ?2",
                params![lease.attempt_id().to_string(), expected, new_tokens, now],
            )
            .map_err(|source| StateError::sqlite("update attempt usage", source))?;
        require_changed(
            changed,
            format!("attempt {} token usage", lease.attempt_id()),
            expected_previous_tokens.to_string(),
            attempt.observed_tokens.to_string(),
        )?;
        let (max_tokens, max_storage): (i64, i64) = transaction
            .query_row(
                "SELECT max_tokens, max_storage_bytes
                 FROM runs WHERE run_id = ?1",
                [attempt.run_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|source| StateError::sqlite("load run usage limits", source))?;
        let max_tokens = checked_i64_to_u64("run max tokens", max_tokens)?;
        let max_storage = checked_i64_to_u64("run max storage", max_storage)?;
        let (observed_tokens, observed_storage) =
            aggregate_run_usage(&transaction, &attempt.run_id)?;
        let outcome = if observed_tokens > u128::from(max_tokens)
            || observed_storage > u128::from(max_storage)
        {
            insert_event(
                &transaction,
                Some(&attempt.run_id),
                Some(&attempt.task_id),
                Some(lease.attempt_id()),
                "budget_exceeded",
                &json!({
                    "maxTokens": max_tokens,
                    "observedTokens": observed_tokens.to_string(),
                    "maxStorageBytes": max_storage,
                    "observedStorageBytes": observed_storage.to_string(),
                }),
                now,
            )?;
            UsageOutcome::Exceeded {
                max_tokens,
                observed_tokens,
                max_storage_bytes: max_storage,
                observed_storage_bytes: observed_storage,
            }
        } else {
            UsageOutcome::WithinBudget
        };
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "usage_reconciled",
            &json!({
                "previousTokens": expected_previous_tokens,
                "totalTokens": new_total_tokens,
                "storageBytes": new_storage_bytes,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit usage reconciliation", source))?;
        Ok(outcome)
    }

    pub fn reconcile_turn_usage(
        &mut self,
        lease: &LeaseToken,
        expected_previous_turn_tokens: u64,
        new_turn_tokens: u64,
        new_storage_bytes: u64,
        now: i64,
    ) -> StateResult<(u64, UsageOutcome)> {
        let lease_now = self.lease_now(now)?;
        if new_turn_tokens < expected_previous_turn_tokens {
            return Err(StateError::invalid(
                "new turn token usage must not decrease",
            ));
        }
        let expected_turn = checked_u64_to_i64(
            "expected previous turn tokens",
            expected_previous_turn_tokens,
        )?;
        let new_turn = checked_u64_to_i64("new turn tokens", new_turn_tokens)?;
        let transaction = self.immediate("begin turn usage reconciliation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        if attempt.latest_turn_observed_tokens != expected_previous_turn_tokens {
            return Err(StateError::Conflict {
                entity: format!("attempt {} turn token usage", attempt.attempt_id),
                expected: expected_previous_turn_tokens.to_string(),
                actual: attempt.latest_turn_observed_tokens.to_string(),
            });
        }
        let delta = new_turn_tokens
            .checked_sub(expected_previous_turn_tokens)
            .expect("monotonic turn usage checked");
        let new_total_tokens = attempt.observed_tokens.checked_add(delta).ok_or_else(|| {
            StateError::LimitExceeded {
                context: "attempt observed token total".to_owned(),
                limit: i64::MAX as u64,
                actual: u64::MAX,
            }
        })?;
        let new_total = checked_u64_to_i64("attempt observed token total", new_total_tokens)?;
        let new_storage = checked_u64_to_i64("new storage bytes", new_storage_bytes)?;
        let previous_storage: i64 = transaction
            .query_row(
                "SELECT observed_storage_bytes
                 FROM budget_reservations WHERE attempt_id = ?1",
                [lease.attempt_id().to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("load previous storage usage", source))?;
        if new_storage < previous_storage {
            return Err(StateError::invalid("new storage usage must not decrease"));
        }
        let changed = transaction
            .execute(
                "UPDATE budget_reservations
                 SET observed_tokens = ?2,
                     observed_storage_bytes = ?3,
                     updated_at = ?4
                 WHERE attempt_id = ?1
                   AND observed_tokens = ?5
                   AND observed_storage_bytes <= ?3",
                params![
                    lease.attempt_id().to_string(),
                    new_total,
                    new_storage,
                    now,
                    checked_u64_to_i64(
                        "expected attempt observed tokens",
                        attempt.observed_tokens,
                    )?,
                ],
            )
            .map_err(|source| StateError::sqlite("update turn usage reservation", source))?;
        require_changed(
            changed,
            format!("attempt {} turn usage reservation", lease.attempt_id()),
            attempt.observed_tokens.to_string(),
            "changed before turn usage CAS",
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET observed_tokens = ?3,
                     latest_turn_observed_tokens = ?4,
                     updated_at = ?5
                 WHERE attempt_id = ?1
                   AND observed_tokens = ?2
                   AND latest_turn_observed_tokens = ?6",
                params![
                    lease.attempt_id().to_string(),
                    checked_u64_to_i64(
                        "expected attempt observed tokens",
                        attempt.observed_tokens,
                    )?,
                    new_total,
                    new_turn,
                    now,
                    expected_turn,
                ],
            )
            .map_err(|source| StateError::sqlite("update attempt turn usage", source))?;
        require_changed(
            changed,
            format!("attempt {} turn usage", lease.attempt_id()),
            format!(
                "total={} turn={}",
                attempt.observed_tokens, expected_previous_turn_tokens
            ),
            "changed before turn usage CAS",
        )?;
        let (max_tokens, max_storage): (i64, i64) = transaction
            .query_row(
                "SELECT max_tokens, max_storage_bytes
                 FROM runs WHERE run_id = ?1",
                [attempt.run_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|source| StateError::sqlite("load run turn usage limits", source))?;
        let max_tokens = checked_i64_to_u64("run max tokens", max_tokens)?;
        let max_storage = checked_i64_to_u64("run max storage", max_storage)?;
        let (observed_tokens, observed_storage) =
            aggregate_run_usage(&transaction, &attempt.run_id)?;
        let outcome = if observed_tokens > u128::from(max_tokens)
            || observed_storage > u128::from(max_storage)
        {
            insert_event(
                &transaction,
                Some(&attempt.run_id),
                Some(&attempt.task_id),
                Some(lease.attempt_id()),
                "budget_exceeded",
                &json!({
                    "maxTokens": max_tokens,
                    "observedTokens": observed_tokens.to_string(),
                    "maxStorageBytes": max_storage,
                    "observedStorageBytes": observed_storage.to_string(),
                }),
                now,
            )?;
            UsageOutcome::Exceeded {
                max_tokens,
                observed_tokens,
                max_storage_bytes: max_storage,
                observed_storage_bytes: observed_storage,
            }
        } else {
            UsageOutcome::WithinBudget
        };
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "turn_usage_reconciled",
            &json!({
                "previousTurnTokens": expected_previous_turn_tokens,
                "turnTokens": new_turn_tokens,
                "totalTokens": new_total_tokens,
                "storageBytes": new_storage_bytes,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit turn usage reconciliation", source))?;
        Ok((new_total_tokens, outcome))
    }

    pub fn record_result_published(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        result: &ResultEnvelope,
        result_ref: &ArtifactRef,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        result.validate().map_err(StateError::invalid_contract)?;
        result_ref
            .validate()
            .map_err(StateError::invalid_contract)?;
        let result_json = bounded_json("result envelope", result, MAX_JSON_BYTES)?;
        let expected_digest = format!("{:x}", Sha256::digest(&result_json));

        let transaction = self.immediate("begin result publication")?;
        let cancelled_result = result.status == ResultStatus::Cancelled;
        let attempt = authorize_lease(&transaction, lease, lease_now, cancelled_result)?;
        if cancelled_result {
            let cancellation_requested: i64 = transaction
                .query_row(
                    "SELECT cancellation_requested
                     FROM runs
                     WHERE run_id = ?1 AND state = 'active'",
                    [attempt.run_id.to_string()],
                    |row| row.get(0),
                )
                .map_err(|source| {
                    StateError::sqlite("authorize cancelled result publication", source)
                })?;
            if cancellation_requested != 1 {
                return Err(StateError::Conflict {
                    entity: format!("run {} cancelled result", attempt.run_id),
                    expected: "cancellation requested".to_owned(),
                    actual: "cancellation not requested".to_owned(),
                });
            }
        }
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        require_operation_kind(&operation, &[OperationKind::PublishResult])?;
        require_operation_state(&operation, OperationState::Completed)?;
        if operation.consumed_at.is_some() {
            return Err(StateError::Conflict {
                entity: format!("operation {operation_id} result"),
                expected: "unconsumed completed operation".to_owned(),
                actual: "already consumed".to_owned(),
            });
        }
        if result.task_id != attempt.task_id {
            return Err(StateError::invalid(format!(
                "result task {} does not match attempt task {}",
                result.task_id, attempt.task_id
            )));
        }
        if result_ref.sha256 != expected_digest
            || result_ref.size_bytes
                != u64::try_from(result_json.len()).expect("usize fits into u64")
        {
            return Err(StateError::integrity(
                "result artifact digest or size does not match the result envelope",
            ));
        }
        if result.token_usage != attempt.observed_tokens {
            return Err(StateError::Conflict {
                entity: format!("attempt {} result token usage", attempt.attempt_id),
                expected: attempt.observed_tokens.to_string(),
                actual: result.token_usage.to_string(),
            });
        }
        require_attempt_state(&attempt, &[AttemptState::Reconciling])?;
        let cli_attempt = load_cli_attempt(&transaction, &attempt.attempt_id)?;
        if let Some(cli_attempt) = &cli_attempt {
            if cli_attempt.state != CliAttemptState::Reconciling {
                return Err(StateError::Conflict {
                    entity: format!("CLI attempt {} result publication", attempt.attempt_id),
                    expected: CliAttemptState::Reconciling.as_str().to_owned(),
                    actual: cli_attempt.state.as_str().to_owned(),
                });
            }
        }
        validate_result_artifacts_registered(&transaction, result, result_ref)?;
        let result_tokens = checked_u64_to_i64("result token usage", result.token_usage)?;
        let next_attempt_state = match result.status {
            ResultStatus::Success => AttemptState::ResultPublished,
            ResultStatus::Partial => AttemptState::Reconciling,
            ResultStatus::Failed => AttemptState::Failed,
            ResultStatus::Cancelled => AttemptState::Cancelled,
        };
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = ?3,
                     result_sha256 = ?4,
                     result_status = ?5,
                     result_token_usage = ?6,
                     lease_owner = CASE WHEN ?3 IN ('failed', 'cancelled') THEN NULL ELSE lease_owner END,
                     lease_expires_at = CASE WHEN ?3 IN ('failed', 'cancelled') THEN NULL ELSE lease_expires_at END,
                     updated_at = ?7
                 WHERE attempt_id = ?1
                   AND state = ?2
                   AND lease_owner = ?8
                   AND lease_expires_at = ?9",
                params![
                    lease.attempt_id().to_string(),
                    attempt.state.as_str(),
                    next_attempt_state.as_str(),
                    result_ref.sha256,
                    result_status_as_str(result.status),
                    result_tokens,
                    now,
                    lease.owner(),
                    lease.expires_at()
                ],
            )
            .map_err(|source| StateError::sqlite("publish attempt result", source))?;
        require_changed(
            changed,
            format!("attempt {} result", lease.attempt_id()),
            format!("{} with matching lease", attempt.state.as_str()),
            "changed before result publication CAS",
        )?;
        let changed = transaction
            .execute(
                "UPDATE operations
                 SET consumed_at = ?2, updated_at = ?2
                 WHERE operation_id = ?1
                   AND state = 'completed'
                   AND consumed_at IS NULL",
                params![operation_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("consume result operation", source))?;
        require_changed(
            changed,
            format!("operation {operation_id} result"),
            "completed and unconsumed",
            "changed before consume CAS",
        )?;
        match result.status {
            ResultStatus::Success => {
                transition_task_state(
                    &transaction,
                    &attempt.run_id,
                    &attempt.task_id,
                    TaskState::Running,
                    TaskState::ResultPublished,
                    now,
                )?;
            }
            ResultStatus::Partial => {}
            ResultStatus::Failed => {
                terminalize_cli_extension(
                    &transaction,
                    &attempt.attempt_id,
                    CliAttemptState::Failed,
                    CliActivityState::Failed,
                    CliTerminalFailureClass::NonResumableProtocolFailure,
                    now,
                )?;
                transition_task_state(
                    &transaction,
                    &attempt.run_id,
                    &attempt.task_id,
                    TaskState::Running,
                    TaskState::Failed,
                    now,
                )?;
                fail_run(&transaction, &attempt.run_id, now)?;
            }
            ResultStatus::Cancelled => {
                terminalize_cli_extension(
                    &transaction,
                    &attempt.attempt_id,
                    CliAttemptState::Cancelled,
                    CliActivityState::Cancelled,
                    CliTerminalFailureClass::Cancelled,
                    now,
                )?;
                transition_nonterminal_operations(
                    &transaction,
                    &attempt.attempt_id,
                    OperationState::Cancelled,
                    now,
                )?;
                cancel_competing_attempts(
                    &transaction,
                    &attempt.run_id,
                    &attempt.task_id,
                    &attempt.attempt_id,
                    now,
                )?;
                transition_task_state(
                    &transaction,
                    &attempt.run_id,
                    &attempt.task_id,
                    TaskState::Running,
                    TaskState::Cancelled,
                    now,
                )?;
                cancel_nonrunning_tasks(&transaction, &attempt.run_id, now)?;
                cancel_run_if_finished(&transaction, &attempt.run_id, now)?;
            }
        }
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "result_published",
            &json!({
                "operationId": operation_id,
                "sha256": result_ref.sha256,
                "status": result_status_as_str(result.status),
                "tokenUsage": result.token_usage,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit result publication", source))
    }

    /// Promotes one durably published successful result under controller authority.
    ///
    /// This transition is intentionally not bound to a worker [`LeaseToken`].
    /// The worker must already have completed publication while authorized;
    /// promotion instead requires an active, non-cancelled run, matching
    /// run/task/attempt identity, registered result metadata, persisted
    /// `success` status and token parity, and a compare-and-swap from
    /// `accepted_attempt_id IS NULL`.
    pub fn accept_result(
        &mut self,
        run_id: &RunId,
        task_id: &TaskId,
        attempt_id: &AttemptId,
        now: i64,
    ) -> StateResult<AcceptResult> {
        let transaction = self.immediate("begin result acceptance")?;
        let task = require_task(&transaction, run_id, task_id)?;
        if let Some(accepted_attempt_id) = task.accepted_attempt_id {
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit already accepted result", source))?;
            return Ok(AcceptResult::AlreadyAccepted {
                attempt_id: accepted_attempt_id,
            });
        }
        let promotable_run: i64 = transaction
            .query_row(
                "SELECT COUNT(*)
                 FROM runs
                 WHERE run_id = ?1
                   AND state = 'active'
                   AND cancellation_requested = 0",
                [run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("authorize result acceptance", source))?;
        if promotable_run != 1 {
            return Err(StateError::Conflict {
                entity: format!("run {run_id} result promotion"),
                expected: "active noncancelled run".to_owned(),
                actual: "inactive or cancellation requested".to_owned(),
            });
        }
        if task.state != TaskState::ResultPublished {
            return Err(StateError::Conflict {
                entity: format!("task {run_id}/{task_id}"),
                expected: TaskState::ResultPublished.as_str().to_owned(),
                actual: task.state.as_str().to_owned(),
            });
        }
        let attempt = require_attempt(&transaction, attempt_id)?;
        if attempt.run_id != *run_id || attempt.task_id != *task_id {
            return Err(StateError::Integrity {
                context: "accepted attempt does not belong to the target task".to_owned(),
                source: None,
            });
        }
        require_attempt_state(&attempt, &[AttemptState::ResultPublished])?;
        let cli_attempt = load_cli_attempt(&transaction, attempt_id)?;
        if let Some(cli_attempt) = &cli_attempt {
            if cli_attempt.state != CliAttemptState::Reconciling {
                return Err(StateError::Conflict {
                    entity: format!("CLI attempt {attempt_id} result acceptance"),
                    expected: CliAttemptState::Reconciling.as_str().to_owned(),
                    actual: cli_attempt.state.as_str().to_owned(),
                });
            }
        }
        charge_observed_wall(&transaction, &attempt.attempt_id, now)?;
        let (reserved_wall, max_run_wall): (i64, i64) = transaction
            .query_row(
                "SELECT b.reserved_wall_seconds, r.max_wall_seconds
                 FROM budget_reservations b
                 JOIN attempts a ON a.attempt_id = b.attempt_id
                 JOIN runs r ON r.run_id = a.run_id
                 WHERE b.attempt_id = ?1",
                [attempt_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|source| StateError::sqlite("load acceptance wall limits", source))?;
        let reserved_wall = checked_i64_to_u64("accepted reserved wall seconds", reserved_wall)?;
        let max_run_wall = checked_i64_to_u64("accepted run wall seconds", max_run_wall)?;
        let observed_run_wall = aggregate_run_wall_usage(&transaction, run_id)?;
        if attempt.observed_wall_seconds > reserved_wall
            || observed_run_wall > u128::from(max_run_wall)
        {
            return Err(StateError::Conflict {
                entity: format!("attempt {attempt_id} accepted wall usage"),
                expected: format!("attempt <= {reserved_wall} and run <= {max_run_wall} seconds"),
                actual: format!(
                    "attempt={} run={observed_run_wall}",
                    attempt.observed_wall_seconds
                ),
            });
        }
        if attempt.result_status != Some(ResultStatus::Success) {
            return Err(StateError::Conflict {
                entity: format!("attempt {attempt_id} result status"),
                expected: result_status_as_str(ResultStatus::Success).to_owned(),
                actual: attempt
                    .result_status
                    .map(result_status_as_str)
                    .unwrap_or("missing")
                    .to_owned(),
            });
        }
        if attempt.result_token_usage != Some(attempt.observed_tokens) {
            return Err(StateError::Conflict {
                entity: format!("attempt {attempt_id} accepted usage"),
                expected: attempt.observed_tokens.to_string(),
                actual: attempt
                    .result_token_usage
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "missing".to_owned()),
            });
        }
        ensure_acceptance_operations_terminal(&transaction, attempt_id)?;
        if cli_attempt.is_some() {
            let nonterminal_cli_activities: i64 = transaction
                .query_row(
                    "SELECT COUNT(*)
                     FROM cli_activities
                     WHERE attempt_id = ?1
                       AND state IN ('prepared', 'dispatching', 'running', 'reconciling')",
                    [attempt_id.to_string()],
                    |row| row.get(0),
                )
                .map_err(|source| {
                    StateError::sqlite("verify accepted CLI activities terminal", source)
                })?;
            if nonterminal_cli_activities != 0 {
                return Err(StateError::Conflict {
                    entity: format!("CLI attempt {attempt_id} activity completion"),
                    expected: "no nonterminal CLI activities".to_owned(),
                    actual: format!("{nonterminal_cli_activities} nonterminal activity row(s)"),
                });
            }
        }
        let result_sha256 = attempt
            .result_sha256
            .as_ref()
            .ok_or_else(|| StateError::integrity("published attempt has no result digest"))?;
        let artifact_count: i64 = transaction
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE sha256 = ?1",
                [result_sha256],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("verify accepted result artifact", source))?;
        if artifact_count != 1 {
            return Err(StateError::integrity(
                "published result digest has no registered artifact metadata",
            ));
        }
        let changed = transaction
            .execute(
                "UPDATE tasks
                 SET accepted_attempt_id = ?3,
                     state = 'completed',
                     updated_at = ?4
                 WHERE run_id = ?1
                   AND task_id = ?2
                   AND state = 'result_published'
                   AND accepted_attempt_id IS NULL",
                params![
                    run_id.to_string(),
                    task_id.to_string(),
                    attempt_id.to_string(),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("accept task result", source))?;
        require_changed(
            changed,
            format!("task {run_id}/{task_id}"),
            "result_published with no accepted attempt",
            "changed before acceptance CAS",
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'succeeded',
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = ?2",
                params![
                    attempt_id.to_string(),
                    AttemptState::ResultPublished.as_str(),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("succeed accepted attempt", source))?;
        require_changed(
            changed,
            format!("attempt {attempt_id}"),
            AttemptState::ResultPublished.as_str(),
            attempt.state.as_str(),
        )?;
        if cli_attempt.is_some() {
            update_cli_attempt_state(
                &transaction,
                attempt_id,
                CliAttemptState::Reconciling,
                CliAttemptState::Succeeded,
                now,
            )?;
        }
        quarantine_competing_attempts(&transaction, run_id, task_id, attempt_id, now)?;
        activate_ready_dependents(&transaction, run_id, now)?;
        complete_run_if_finished(&transaction, run_id, now)?;
        insert_event(
            &transaction,
            Some(run_id),
            Some(task_id),
            Some(attempt_id),
            "result_accepted",
            &json!({"sha256": result_sha256}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit result acceptance", source))?;
        Ok(AcceptResult::Accepted)
    }
}
