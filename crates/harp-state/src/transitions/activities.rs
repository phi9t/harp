use super::*;

impl StateStore {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_activity(
        &mut self,
        lease: &LeaseToken,
        activity_id: OperationId,
        attempt_id: &AttemptId,
        kind: CliActivityKind,
        logical_turn_id: TurnId,
        preparation: &ActivityPreparation,
        now: i64,
    ) -> StateResult<CliActivityRecord> {
        if attempt_id != lease.attempt_id() {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} attempt"),
                expected: lease.attempt_id().to_string(),
                actual: attempt_id.to_string(),
            });
        }
        if logical_turn_id.to_string() != activity_id.to_string() {
            return Err(StateError::invalid(
                "CLI logical turn id must equal activity id",
            ));
        }
        if kind != CliActivityKind::StartActivity {
            return Err(StateError::invalid(match kind {
                CliActivityKind::ContinueActivity => {
                    "continuations require prepare_continuation_after_failure"
                }
                CliActivityKind::InterruptActivity => {
                    "interrupt activities require prepare_interrupt_activity"
                }
                CliActivityKind::StartActivity => unreachable!("checked above"),
            }));
        }
        preparation.validate()?;
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI activity preparation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let cli_attempt = require_cli_attempt(&transaction, attempt_id)?;
        let ordinal = 0;

        if let Some(existing) = load_cli_activity_by_key(&transaction, attempt_id, kind, ordinal)? {
            if existing.activity_id != activity_id
                || existing.logical_turn_id != logical_turn_id
                || existing.activity_dir != preparation.activity_dir
                || existing.invocation_sha256 != preparation.invocation_sha256
            {
                return Err(StateError::Conflict {
                    entity: format!(
                        "CLI activity {attempt_id}/{} idempotency key",
                        kind.as_str()
                    ),
                    expected: activity_id.to_string(),
                    actual: existing.activity_id.to_string(),
                });
            }
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit idempotent CLI activity preparation", source)
            })?;
            return Ok(existing);
        }
        require_cli_attempt_state(&cli_attempt, &[CliAttemptState::Prepared])?;
        if load_cli_activity(&transaction, &activity_id)?.is_some() {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} identity"),
                expected: format!("{attempt_id}/{} ordinal {ordinal}", kind.as_str()),
                actual: "already used by another activity key".to_owned(),
            });
        }

        transaction
            .execute(
                "INSERT INTO cli_activities (
                    activity_id, attempt_id, kind, ordinal, state,
                    logical_turn_id, activity_dir, invocation_sha256,
                    created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, 'prepared', ?1, ?5, ?6, ?7, ?7)",
                params![
                    activity_id.to_string(),
                    attempt_id.to_string(),
                    kind.as_str(),
                    i64::from(ordinal),
                    preparation.activity_dir,
                    preparation.invocation_sha256,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("insert prepared CLI activity", source))?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(attempt_id),
            "cli_activity_prepared",
            &json!({
                "activityId": activity_id,
                "kind": kind.as_str(),
                "ordinal": ordinal,
                "activityDir": preparation.activity_dir,
                "invocationSha256": preparation.invocation_sha256,
            }),
            now,
        )?;
        let record = CliActivityRecord {
            activity_id,
            attempt_id: attempt_id.clone(),
            kind,
            ordinal,
            state: CliActivityState::Prepared,
            logical_turn_id,
            activity_dir: preparation.activity_dir.clone(),
            invocation_sha256: preparation.invocation_sha256.clone(),
            process_record_sha256: None,
            terminal_failure_class: None,
            interrupt_purpose: None,
            target_process_record_sha256: None,
            signal_stage: None,
            created_at: now,
            updated_at: now,
        };
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI activity preparation", source))?;
        Ok(record)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_continuation_after_failure(
        &mut self,
        lease: &LeaseToken,
        failed_activity_id: &OperationId,
        failure_class: CliTerminalFailureClass,
        continuation_activity_id: OperationId,
        preparation: &ActivityPreparation,
        now: i64,
    ) -> StateResult<CliActivityRecord> {
        if !failure_class.is_resumable() {
            return Err(StateError::invalid(
                "continuation preparation requires a resumable CLI failure class",
            ));
        }
        preparation.validate()?;
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI continuation preparation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let failed_activity = require_cli_activity(&transaction, failed_activity_id)?;
        require_activity_attempt(&failed_activity, lease)?;
        if !matches!(
            failed_activity.kind,
            CliActivityKind::StartActivity | CliActivityKind::ContinueActivity
        ) {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {failed_activity_id} kind"),
                expected: "start_activity|continue_activity".to_owned(),
                actual: failed_activity.kind.as_str().to_owned(),
            });
        }
        let cli_attempt = require_cli_attempt(&transaction, lease.attempt_id())?;

        let existing_continuation = load_cli_activity_by_key(
            &transaction,
            lease.attempt_id(),
            CliActivityKind::ContinueActivity,
            0,
        )?;
        let replay = failed_activity.state == CliActivityState::Failed
            && failed_activity.terminal_failure_class == Some(failure_class)
            && cli_attempt.continuation_count == 1
            && existing_continuation.as_ref().is_some_and(|existing| {
                existing.activity_id == continuation_activity_id
                    && existing.activity_dir == preparation.activity_dir
                    && existing.invocation_sha256 == preparation.invocation_sha256
            });
        if replay {
            let continuation = existing_continuation.expect("checked above");
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit replayed CLI continuation preparation", source)
            })?;
            return Ok(continuation);
        }
        require_cli_activity_state(&failed_activity, CliActivityState::Reconciling)?;
        require_cli_attempt_state(&cli_attempt, &[CliAttemptState::Running])?;
        if cli_attempt.continuation_count != 0 {
            return Err(StateError::Conflict {
                entity: format!("CLI attempt {} continuation count", attempt.attempt_id),
                expected: "0".to_owned(),
                actual: cli_attempt.continuation_count.to_string(),
            });
        }
        if cli_attempt.external_session_id.is_none() {
            return Err(StateError::Conflict {
                entity: format!("CLI attempt {} continuation session", attempt.attempt_id),
                expected: "known external session".to_owned(),
                actual: "missing".to_owned(),
            });
        }
        if existing_continuation.is_some()
            || load_cli_activity(&transaction, &continuation_activity_id)?.is_some()
        {
            return Err(StateError::Conflict {
                entity: format!("CLI continuation {}", continuation_activity_id),
                expected: "unused continuation idempotency key and activity id".to_owned(),
                actual: "already exists".to_owned(),
            });
        }

        let changed = transaction
            .execute(
                "UPDATE cli_activities
                 SET state = 'failed',
                     terminal_failure_class = ?3,
                     updated_at = ?4
                 WHERE activity_id = ?1
                   AND state = ?2
                   AND terminal_failure_class IS NULL",
                params![
                    failed_activity_id.to_string(),
                    CliActivityState::Reconciling.as_str(),
                    failure_class.as_str(),
                    now
                ],
            )
            .map_err(|source| {
                StateError::sqlite("record resumable CLI activity failure", source)
            })?;
        require_changed(
            changed,
            format!("CLI activity {failed_activity_id}"),
            CliActivityState::Reconciling.as_str(),
            "changed before resumable failure CAS",
        )?;
        update_cli_attempt_state(
            &transaction,
            lease.attempt_id(),
            CliAttemptState::Running,
            CliAttemptState::Reconciling,
            now,
        )?;
        transaction
            .execute(
                "INSERT INTO cli_activities (
                    activity_id, attempt_id, kind, ordinal, state,
                    logical_turn_id, activity_dir, invocation_sha256,
                    created_at, updated_at
                 ) VALUES (
                    ?1, ?2, 'continue_activity', 0, 'prepared',
                    ?1, ?3, ?4, ?5, ?5
                 )",
                params![
                    continuation_activity_id.to_string(),
                    lease.attempt_id().to_string(),
                    preparation.activity_dir,
                    preparation.invocation_sha256,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("insert prepared CLI continuation", source))?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET latest_turn_observed_tokens = 0,
                     updated_at = ?2
                 WHERE attempt_id = ?1
                   AND latest_turn_observed_tokens != 0",
                params![lease.attempt_id().to_string(), now],
            )
            .map_err(|source| StateError::sqlite("reset CLI continuation turn usage", source))?;
        if changed > 1 {
            return Err(StateError::integrity(
                "reset CLI continuation turn usage changed multiple attempts",
            ));
        }
        let changed = transaction
            .execute(
                "UPDATE cli_attempts
                 SET continuation_count = 1, updated_at = ?2
                 WHERE attempt_id = ?1
                   AND state = 'reconciling'
                   AND continuation_count = 0",
                params![lease.attempt_id().to_string(), now],
            )
            .map_err(|source| StateError::sqlite("reserve CLI continuation allowance", source))?;
        require_changed(
            changed,
            format!("CLI attempt {} continuation", attempt.attempt_id),
            "reconciling with continuation count 0",
            "changed before continuation-count CAS",
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_continuation_prepared",
            &json!({
                "failedActivityId": failed_activity_id,
                "failureClass": failure_class.as_str(),
                "activityId": continuation_activity_id,
                "ordinal": 0,
                "invocationSha256": preparation.invocation_sha256,
            }),
            now,
        )?;
        let continuation = load_cli_activity(&transaction, &continuation_activity_id)?
            .ok_or_else(|| StateError::integrity("inserted CLI continuation disappeared"))?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI continuation preparation", source))?;
        Ok(continuation)
    }

    pub fn record_activity_terminal_outcome(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        failure_class: CliTerminalFailureClass,
        now: i64,
    ) -> StateResult<()> {
        if failure_class.is_resumable() {
            return Err(StateError::invalid(
                "resumable CLI failures require atomic continuation preparation",
            ));
        }
        let lease_now = self.lease_now(now)?;
        let allow_cancellation = failure_class == CliTerminalFailureClass::Cancelled;
        let transaction = self.immediate("begin CLI terminal outcome")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, allow_cancellation)?;
        require_attempt_state(&attempt, &[AttemptState::Prepared])?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        if activity.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id}"),
                expected: "nonterminal activity".to_owned(),
                actual: activity.state.as_str().to_owned(),
            });
        }
        if activity.kind != CliActivityKind::InterruptActivity
            && matches!(
                failure_class,
                CliTerminalFailureClass::NonResumableProtocolFailure
                    | CliTerminalFailureClass::ContinuationExhausted
            )
            && activity.process_record_sha256.is_none()
        {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} process record"),
                expected: "persisted process record before failed outcome".to_owned(),
                actual: "missing".to_owned(),
            });
        }
        if activity.kind == CliActivityKind::InterruptActivity
            && matches!(
                failure_class,
                CliTerminalFailureClass::NonResumableProtocolFailure
                    | CliTerminalFailureClass::ContinuationExhausted
            )
        {
            return Err(StateError::Conflict {
                entity: format!("CLI interrupt activity {activity_id} outcome"),
                expected: "quiescent, indeterminate, or cancelled".to_owned(),
                actual: failure_class.as_str().to_owned(),
            });
        }
        let cli_attempt = require_cli_attempt(&transaction, lease.attempt_id())?;
        if cli_attempt.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("CLI attempt {}", attempt.attempt_id),
                expected: "nonterminal CLI attempt".to_owned(),
                actual: cli_attempt.state.as_str().to_owned(),
            });
        }
        let (legacy_state, cli_state, activity_state) = match failure_class {
            CliTerminalFailureClass::NonResumableProtocolFailure
            | CliTerminalFailureClass::ContinuationExhausted => (
                AttemptState::Failed,
                CliAttemptState::Failed,
                CliActivityState::Failed,
            ),
            CliTerminalFailureClass::Indeterminate => (
                AttemptState::Indeterminate,
                CliAttemptState::Indeterminate,
                CliActivityState::Indeterminate,
            ),
            CliTerminalFailureClass::Cancelled => (
                AttemptState::Cancelled,
                CliAttemptState::Cancelled,
                CliActivityState::Cancelled,
            ),
            CliTerminalFailureClass::ResumableInterrupted
            | CliTerminalFailureClass::ResumableCliFailure => {
                unreachable!("validated above")
            }
        };
        if failure_class == CliTerminalFailureClass::Cancelled {
            let cancellation_requested: i64 = transaction
                .query_row(
                    "SELECT cancellation_requested
                     FROM runs
                     WHERE run_id = ?1 AND state = 'active'",
                    [attempt.run_id.to_string()],
                    |row| row.get(0),
                )
                .map_err(|source| {
                    StateError::sqlite("authorize CLI cancellation outcome", source)
                })?;
            if cancellation_requested != 1 {
                return Err(StateError::Conflict {
                    entity: format!("run {} CLI cancellation", attempt.run_id),
                    expected: "cancellation requested".to_owned(),
                    actual: "cancellation not requested".to_owned(),
                });
            }
        }

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
                    activity.state.as_str(),
                    activity_state.as_str(),
                    failure_class.as_str(),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("terminalize selected CLI activity", source))?;
        require_changed(
            changed,
            format!("CLI activity {activity_id}"),
            activity.state.as_str(),
            "changed before CLI outcome CAS",
        )?;
        terminalize_cli_extension(
            &transaction,
            &attempt.attempt_id,
            cli_state,
            activity_state,
            failure_class,
            now,
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = ?3,
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?4
                 WHERE attempt_id = ?1 AND state = ?2",
                params![
                    attempt.attempt_id.to_string(),
                    AttemptState::Prepared.as_str(),
                    legacy_state.as_str(),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("terminalize legacy CLI attempt", source))?;
        require_changed(
            changed,
            format!("attempt {}", attempt.attempt_id),
            AttemptState::Prepared.as_str(),
            "changed before legacy CLI terminal CAS",
        )?;

        match failure_class {
            CliTerminalFailureClass::NonResumableProtocolFailure
            | CliTerminalFailureClass::ContinuationExhausted => {
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
            CliTerminalFailureClass::Indeterminate => {
                if retry_allowed(&transaction, &attempt.run_id, &attempt.task_id)? {
                    transition_task_state(
                        &transaction,
                        &attempt.run_id,
                        &attempt.task_id,
                        TaskState::Running,
                        TaskState::Ready,
                        now,
                    )?;
                } else {
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
            }
            CliTerminalFailureClass::Cancelled => {
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
            CliTerminalFailureClass::ResumableInterrupted
            | CliTerminalFailureClass::ResumableCliFailure => {
                unreachable!("validated above")
            }
        }
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_activity_terminal",
            &json!({
                "activityId": activity_id,
                "state": activity_state.as_str(),
                "failureClass": failure_class.as_str(),
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI terminal outcome", source))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_interrupt_activity(
        &mut self,
        lease: &LeaseToken,
        activity_id: OperationId,
        ordinal: u32,
        preparation: &ActivityPreparation,
        target_process_record_sha256: &str,
        purpose: crate::InterruptPurpose,
        now: i64,
    ) -> StateResult<CliActivityRecord> {
        preparation.validate()?;
        validate_sha256_input("target process record digest", target_process_record_sha256)?;
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI interrupt preparation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        let cli_attempt = require_cli_attempt(&transaction, lease.attempt_id())?;
        require_cli_attempt_state(
            &cli_attempt,
            &[CliAttemptState::Running, CliAttemptState::Reconciling],
        )?;
        validate_cli_interrupt_purpose(&transaction, &attempt, purpose)?;
        if let Some(existing) = load_cli_activity_by_key(
            &transaction,
            lease.attempt_id(),
            CliActivityKind::InterruptActivity,
            ordinal,
        )? {
            if existing.activity_id != activity_id
                || existing.activity_dir != preparation.activity_dir
                || existing.invocation_sha256 != preparation.invocation_sha256
                || existing.target_process_record_sha256.as_deref()
                    != Some(target_process_record_sha256)
                || existing.interrupt_purpose != Some(purpose)
            {
                return Err(StateError::Conflict {
                    entity: format!("CLI interrupt activity {activity_id} idempotency"),
                    expected: "matching immutable interrupt intent".to_owned(),
                    actual: "different persisted interrupt intent".to_owned(),
                });
            }
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit idempotent CLI interrupt preparation", source)
            })?;
            return Ok(existing);
        }
        transaction
            .execute(
                "INSERT INTO cli_activities (
                    activity_id, attempt_id, kind, ordinal, state,
                    logical_turn_id, activity_dir, invocation_sha256,
                    interrupt_purpose, target_process_record_sha256, signal_stage,
                    created_at, updated_at
                 ) VALUES (
                    ?1, ?2, 'interrupt_activity', ?3, 'prepared',
                    ?1, ?4, ?5, ?6, ?7, 'prepared', ?8, ?8
                 )",
                params![
                    activity_id.to_string(),
                    lease.attempt_id().to_string(),
                    i64::from(ordinal),
                    preparation.activity_dir,
                    preparation.invocation_sha256,
                    purpose.as_str(),
                    target_process_record_sha256,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("insert prepared CLI interrupt", source))?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_interrupt_prepared",
            &json!({
                "activityId": activity_id,
                "ordinal": ordinal,
                "purpose": purpose.as_str(),
                "targetProcessRecordSha256": target_process_record_sha256,
            }),
            now,
        )?;
        let record = load_cli_activity(&transaction, &activity_id)?
            .ok_or_else(|| StateError::integrity("inserted CLI interrupt disappeared"))?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI interrupt preparation", source))?;
        Ok(record)
    }

    pub fn mark_activity_dispatching(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI activity dispatch")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        require_cli_activity_state(&activity, CliActivityState::Prepared)?;
        if activity.kind == CliActivityKind::InterruptActivity {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} kind"),
                expected: "start_activity|continue_activity".to_owned(),
                actual: activity.kind.as_str().to_owned(),
            });
        }
        let cli_attempt = require_cli_attempt(&transaction, lease.attempt_id())?;
        let expected_attempt_state = match activity.kind {
            CliActivityKind::StartActivity => CliAttemptState::Prepared,
            CliActivityKind::ContinueActivity => CliAttemptState::Reconciling,
            CliActivityKind::InterruptActivity => unreachable!("checked above"),
        };
        update_cli_activity_state(
            &transaction,
            activity_id,
            CliActivityState::Prepared,
            CliActivityState::Dispatching,
            now,
        )?;
        update_cli_attempt_state(
            &transaction,
            &cli_attempt.attempt_id,
            expected_attempt_state,
            CliAttemptState::Running,
            now,
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_activity_dispatching",
            &json!({"activityId": activity_id, "kind": activity.kind.as_str()}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI activity dispatch", source))
    }

    pub fn record_process(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        process_record_sha256: &str,
        now: i64,
    ) -> StateResult<()> {
        validate_sha256_input("process record digest", process_record_sha256)?;
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI process record")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        require_cli_activity_state(&activity, CliActivityState::Dispatching)?;
        if activity.kind == CliActivityKind::InterruptActivity {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} process record"),
                expected: "start or continue activity".to_owned(),
                actual: "interrupt activity".to_owned(),
            });
        }
        if let Some(existing) = activity.process_record_sha256 {
            if existing != process_record_sha256 {
                return Err(StateError::Conflict {
                    entity: format!("CLI activity {activity_id} process record"),
                    expected: existing,
                    actual: process_record_sha256.to_owned(),
                });
            }
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit replayed CLI process record", source)
            })?;
            return Ok(());
        }
        let changed = transaction
            .execute(
                "UPDATE cli_activities
                 SET process_record_sha256 = ?2, updated_at = ?3
                 WHERE activity_id = ?1
                   AND state = 'dispatching'
                   AND process_record_sha256 IS NULL",
                params![activity_id.to_string(), process_record_sha256, now],
            )
            .map_err(|source| StateError::sqlite("record CLI process digest", source))?;
        require_changed(
            changed,
            format!("CLI activity {activity_id} process record"),
            "dispatching with no process digest",
            "changed before process-record CAS",
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_process_recorded",
            &json!({
                "activityId": activity_id,
                "processRecordSha256": process_record_sha256,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI process record", source))
    }

    pub fn mark_activity_running(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        now: i64,
    ) -> StateResult<()> {
        self.transition_cli_activity_with_lease(
            lease,
            activity_id,
            CliActivityState::Dispatching,
            CliActivityState::Running,
            true,
            "cli_activity_running",
            now,
        )
    }

    pub fn mark_activity_reconciling(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        now: i64,
    ) -> StateResult<()> {
        self.transition_cli_activity_with_lease(
            lease,
            activity_id,
            CliActivityState::Running,
            CliActivityState::Reconciling,
            true,
            "cli_activity_reconciling",
            now,
        )
    }

    pub fn record_cli_external_session(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        value: ExternalSessionId,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI external session assignment")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        let cli_attempt = require_cli_attempt(&transaction, lease.attempt_id())?;
        if let Some(existing) = cli_attempt.external_session_id {
            if existing != value {
                return Err(StateError::Conflict {
                    entity: format!("CLI attempt {} external session", attempt.attempt_id),
                    expected: existing.to_string(),
                    actual: value.to_string(),
                });
            }
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit replayed CLI external session", source)
            })?;
            return Ok(());
        }
        let changed = transaction
            .execute(
                "UPDATE cli_attempts
                 SET external_session_id = ?2, updated_at = ?3
                 WHERE attempt_id = ?1 AND external_session_id IS NULL",
                params![attempt.attempt_id.to_string(), value.to_string(), now],
            )
            .map_err(|source| match source {
                rusqlite::Error::SqliteFailure(error, _)
                    if error.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE =>
                {
                    StateError::Conflict {
                        entity: format!("CLI external session {value}"),
                        expected: "unused external session id".to_owned(),
                        actual: "already assigned to another attempt".to_owned(),
                    }
                }
                other => StateError::sqlite("record CLI external session", other),
            })?;
        require_changed(
            changed,
            format!("CLI attempt {} external session", attempt.attempt_id),
            "unassigned",
            "changed before external-session CAS",
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_external_session_recorded",
            &json!({"activityId": activity_id, "externalSessionId": value}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI external session", source))
    }

    pub fn advance_cli_signal_stage(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        expected: CliSignalStage,
        next: CliSignalStage,
        now: i64,
    ) -> StateResult<()> {
        if !valid_signal_transition(expected, next) {
            return Err(StateError::invalid(
                "invalid CLI interrupt signal-stage transition",
            ));
        }
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI signal-stage transition")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        if activity.kind != CliActivityKind::InterruptActivity {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} kind"),
                expected: CliActivityKind::InterruptActivity.as_str().to_owned(),
                actual: activity.kind.as_str().to_owned(),
            });
        }
        if activity.signal_stage != Some(expected) {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} signal stage"),
                expected: expected.as_str().to_owned(),
                actual: activity
                    .signal_stage
                    .map(CliSignalStage::as_str)
                    .unwrap_or("missing")
                    .to_owned(),
            });
        }
        validate_cli_interrupt_purpose(
            &transaction,
            &attempt,
            activity
                .interrupt_purpose
                .ok_or_else(|| StateError::integrity("CLI interrupt activity lacks purpose"))?,
        )?;
        let next_activity_state = match next {
            CliSignalStage::Quiescent => Some(CliActivityState::Completed),
            CliSignalStage::Indeterminate => Some(CliActivityState::Indeterminate),
            _ => None,
        };
        let terminal_failure = (next == CliSignalStage::Indeterminate)
            .then_some(CliTerminalFailureClass::Indeterminate);
        let changed = transaction
            .execute(
                "UPDATE cli_activities
                 SET signal_stage = ?3,
                     state = COALESCE(?4, state),
                     terminal_failure_class = ?5,
                     updated_at = ?6
                 WHERE activity_id = ?1
                   AND kind = 'interrupt_activity'
                   AND signal_stage = ?2
                   AND state = 'prepared'",
                params![
                    activity_id.to_string(),
                    expected.as_str(),
                    next.as_str(),
                    next_activity_state.map(CliActivityState::as_str),
                    terminal_failure.map(CliTerminalFailureClass::as_str),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("advance CLI signal stage", source))?;
        require_changed(
            changed,
            format!("CLI activity {activity_id} signal stage"),
            expected.as_str(),
            "changed before signal-stage CAS",
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_signal_stage_advanced",
            &json!({
                "activityId": activity_id,
                "previous": expected.as_str(),
                "signalStage": next.as_str(),
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI signal stage", source))
    }

    pub fn bridge_cli_completion(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        publish_operation_id: &OperationId,
        now: i64,
    ) -> StateResult<OperationRecord> {
        if activity_id == publish_operation_id {
            return Err(StateError::invalid(
                "CLI activity id must never be inserted as a legacy operation id",
            ));
        }
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI completion bridge")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        if activity.kind == CliActivityKind::InterruptActivity {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} kind"),
                expected: "start_activity|continue_activity".to_owned(),
                actual: activity.kind.as_str().to_owned(),
            });
        }
        let cli_attempt = require_cli_attempt(&transaction, lease.attempt_id())?;
        let existing_publish = load_operation_by_key(
            &transaction,
            lease.attempt_id(),
            OperationKind::PublishResult,
            0,
        )?;
        let publish_operation_count: i64 = transaction
            .query_row(
                "SELECT COUNT(*)
                 FROM operations
                 WHERE attempt_id = ?1 AND kind = 'publish_result'",
                [lease.attempt_id().to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("count bridge publish operations", source))?;

        let first_execution = activity.state == CliActivityState::Reconciling
            && cli_attempt.state == CliAttemptState::Running
            && attempt.state == AttemptState::Prepared
            && publish_operation_count == 0;
        let replay = activity.state == CliActivityState::Completed
            && cli_attempt.state == CliAttemptState::Reconciling
            && attempt.state == AttemptState::Reconciling
            && publish_operation_count == 1
            && existing_publish.as_ref().is_some_and(|operation| {
                operation.operation_id == *publish_operation_id
                    && operation.kind == OperationKind::PublishResult
                    && operation.ordinal == 0
            });

        if replay {
            let operation = existing_publish.expect("checked above");
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit replayed CLI bridge", source))?;
            return Ok(operation);
        }
        if !first_execution {
            return Err(StateError::Conflict {
                entity: format!("CLI completion bridge {activity_id}"),
                expected: "reconciling/running/prepared without publish operation or exact replay"
                    .to_owned(),
                actual: format!(
                    "{}/{}/{} with publish {}",
                    activity.state.as_str(),
                    cli_attempt.state.as_str(),
                    attempt.state.as_str(),
                    existing_publish
                        .as_ref()
                        .map(|operation| operation.operation_id.to_string())
                        .unwrap_or_else(|| "missing".to_owned())
                ),
            });
        }
        if load_operation(&transaction, publish_operation_id)?.is_some() {
            return Err(StateError::Conflict {
                entity: format!("publish operation {publish_operation_id}"),
                expected: "unused operation id".to_owned(),
                actual: "already exists".to_owned(),
            });
        }

        update_cli_activity_state(
            &transaction,
            activity_id,
            CliActivityState::Reconciling,
            CliActivityState::Completed,
            now,
        )?;
        update_cli_attempt_state(
            &transaction,
            lease.attempt_id(),
            CliAttemptState::Running,
            CliAttemptState::Reconciling,
            now,
        )?;
        update_attempt_state(
            &transaction,
            lease.attempt_id(),
            AttemptState::Prepared,
            AttemptState::Reconciling,
            now,
        )?;
        let publish = super::operations::insert_prepared_plain_operation(
            &transaction,
            &AttemptRecord {
                state: AttemptState::Reconciling,
                ..attempt.clone()
            },
            publish_operation_id,
            OperationKind::PublishResult,
            0,
            now,
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "cli_completion_bridged",
            &json!({
                "activityId": activity_id,
                "publishOperationId": publish_operation_id,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI completion bridge", source))?;
        Ok(publish)
    }

    #[allow(clippy::too_many_arguments)]
    fn transition_cli_activity_with_lease(
        &mut self,
        lease: &LeaseToken,
        activity_id: &OperationId,
        expected: CliActivityState,
        next: CliActivityState,
        require_process_record: bool,
        event_type: &str,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin CLI activity transition")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let activity = require_cli_activity(&transaction, activity_id)?;
        require_activity_attempt(&activity, lease)?;
        require_cli_activity_state(&activity, expected)?;
        if require_process_record && activity.process_record_sha256.is_none() {
            return Err(StateError::Conflict {
                entity: format!("CLI activity {activity_id} process record"),
                expected: "persisted process record digest".to_owned(),
                actual: "missing".to_owned(),
            });
        }
        update_cli_activity_state(&transaction, activity_id, expected, next, now)?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            event_type,
            &json!({"activityId": activity_id}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit CLI activity transition", source))
    }
}

fn require_cli_attempt(
    connection: &Connection,
    attempt_id: &AttemptId,
) -> StateResult<CliAttemptRecord> {
    load_cli_attempt(connection, attempt_id)?.ok_or_else(|| StateError::Conflict {
        entity: format!("CLI attempt {attempt_id}"),
        expected: "existing CLI attempt extension".to_owned(),
        actual: "legacy-only attempt".to_owned(),
    })
}

fn require_cli_activity(
    connection: &Connection,
    activity_id: &OperationId,
) -> StateResult<CliActivityRecord> {
    load_cli_activity(connection, activity_id)?.ok_or_else(|| StateError::Conflict {
        entity: format!("CLI activity {activity_id}"),
        expected: "existing CLI activity".to_owned(),
        actual: "missing".to_owned(),
    })
}

fn load_cli_activity_by_key(
    connection: &Connection,
    attempt_id: &AttemptId,
    kind: CliActivityKind,
    ordinal: u32,
) -> StateResult<Option<CliActivityRecord>> {
    let activity_id: Option<String> = connection
        .query_row(
            "SELECT activity_id
             FROM cli_activities
             WHERE attempt_id = ?1 AND kind = ?2 AND ordinal = ?3",
            params![attempt_id.to_string(), kind.as_str(), i64::from(ordinal)],
            |row| row.get(0),
        )
        .optional()
        .map_err(|source| StateError::sqlite("query CLI activity idempotency key", source))?;
    activity_id
        .map(|value| {
            let activity_id = parse_id("CLI activity idempotency id", value)?;
            load_cli_activity(connection, &activity_id)?.ok_or_else(|| {
                StateError::integrity("CLI activity idempotency key points to missing row")
            })
        })
        .transpose()
}

fn require_activity_attempt(activity: &CliActivityRecord, lease: &LeaseToken) -> StateResult<()> {
    if &activity.attempt_id == lease.attempt_id() {
        Ok(())
    } else {
        Err(StateError::Conflict {
            entity: format!("CLI activity {} attempt", activity.activity_id),
            expected: lease.attempt_id().to_string(),
            actual: activity.attempt_id.to_string(),
        })
    }
}

fn require_cli_attempt_state(
    attempt: &CliAttemptRecord,
    expected: &[CliAttemptState],
) -> StateResult<()> {
    if expected.contains(&attempt.state) {
        return Ok(());
    }
    Err(StateError::Conflict {
        entity: format!("CLI attempt {}", attempt.attempt_id),
        expected: expected
            .iter()
            .map(|state| state.as_str())
            .collect::<Vec<_>>()
            .join("|"),
        actual: attempt.state.as_str().to_owned(),
    })
}

fn require_cli_activity_state(
    activity: &CliActivityRecord,
    expected: CliActivityState,
) -> StateResult<()> {
    if activity.state == expected {
        Ok(())
    } else {
        Err(StateError::Conflict {
            entity: format!("CLI activity {}", activity.activity_id),
            expected: expected.as_str().to_owned(),
            actual: activity.state.as_str().to_owned(),
        })
    }
}

pub(super) fn update_cli_attempt_state(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    expected: CliAttemptState,
    next: CliAttemptState,
    now: i64,
) -> StateResult<()> {
    let changed = transaction
        .execute(
            "UPDATE cli_attempts
             SET state = ?3, terminal_failure_class = NULL, updated_at = ?4
             WHERE attempt_id = ?1 AND state = ?2",
            params![
                attempt_id.to_string(),
                expected.as_str(),
                next.as_str(),
                now
            ],
        )
        .map_err(|source| StateError::sqlite("update CLI attempt state", source))?;
    require_changed(
        changed,
        format!("CLI attempt {attempt_id}"),
        expected.as_str(),
        "changed before CLI attempt CAS",
    )
}

pub(super) fn update_cli_activity_state(
    transaction: &Transaction<'_>,
    activity_id: &OperationId,
    expected: CliActivityState,
    next: CliActivityState,
    now: i64,
) -> StateResult<()> {
    let changed = transaction
        .execute(
            "UPDATE cli_activities
             SET state = ?3, updated_at = ?4
             WHERE activity_id = ?1 AND state = ?2",
            params![
                activity_id.to_string(),
                expected.as_str(),
                next.as_str(),
                now
            ],
        )
        .map_err(|source| StateError::sqlite("update CLI activity state", source))?;
    require_changed(
        changed,
        format!("CLI activity {activity_id}"),
        expected.as_str(),
        "changed before CLI activity CAS",
    )
}

fn validate_sha256_input(context: &str, value: &str) -> StateResult<()> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(StateError::invalid(format!(
            "{context} must be lowercase SHA-256"
        )))
    }
}

fn valid_signal_transition(expected: CliSignalStage, next: CliSignalStage) -> bool {
    matches!(
        (expected, next),
        (CliSignalStage::Prepared, CliSignalStage::SigintPrepared)
            | (CliSignalStage::Prepared, CliSignalStage::Indeterminate)
            | (CliSignalStage::SigintPrepared, CliSignalStage::SigintSent)
            | (
                CliSignalStage::SigintPrepared,
                CliSignalStage::Indeterminate
            )
            | (CliSignalStage::SigintSent, CliSignalStage::SigtermPrepared)
            | (CliSignalStage::SigtermPrepared, CliSignalStage::SigtermSent)
            | (
                CliSignalStage::SigtermPrepared,
                CliSignalStage::Indeterminate
            )
            | (CliSignalStage::SigtermSent, CliSignalStage::SigkillPrepared)
            | (CliSignalStage::SigkillPrepared, CliSignalStage::SigkillSent)
            | (
                CliSignalStage::SigkillPrepared,
                CliSignalStage::Indeterminate
            )
            | (
                CliSignalStage::SigintSent
                    | CliSignalStage::SigtermSent
                    | CliSignalStage::SigkillSent,
                CliSignalStage::Quiescent | CliSignalStage::Indeterminate
            )
    )
}

fn validate_cli_interrupt_purpose(
    transaction: &Transaction<'_>,
    attempt: &AttemptRecord,
    purpose: crate::InterruptPurpose,
) -> StateResult<()> {
    let cancellation_requested: i64 = transaction
        .query_row(
            "SELECT cancellation_requested
             FROM runs
             WHERE run_id = ?1 AND state = 'active'",
            [attempt.run_id.to_string()],
            |row| row.get(0),
        )
        .map_err(|source| {
            StateError::sqlite("load CLI interrupt cancellation authority", source)
        })?;
    let cancellation_purpose = purpose == crate::InterruptPurpose::Cancellation;
    if cancellation_purpose != (cancellation_requested == 1) {
        return Err(StateError::Conflict {
            entity: format!("run {} CLI interrupt purpose", attempt.run_id),
            expected: if cancellation_requested == 1 {
                "cancellation".to_owned()
            } else {
                "budget or scanner_integrity".to_owned()
            },
            actual: format!("{purpose:?}"),
        });
    }
    Ok(())
}
