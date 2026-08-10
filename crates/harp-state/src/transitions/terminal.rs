use super::*;

impl StateStore {
    pub fn fail_terminal_semantic(
        &mut self,
        lease: &LeaseToken,
        failure_class: &str,
        now: i64,
    ) -> StateResult<()> {
        validate_label("semantic failure class", failure_class)?;
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin terminal semantic failure")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        if attempt.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("attempt {} semantic failure", attempt.attempt_id),
                expected: "nonterminal attempt".to_owned(),
                actual: attempt.state.as_str().to_owned(),
            });
        }
        update_attempt_state(
            &transaction,
            &attempt.attempt_id,
            attempt.state,
            AttemptState::Failed,
            now,
        )?;
        transaction
            .execute(
                "UPDATE attempts
                 SET semantic_failure_class = ?2,
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = 'failed'",
                params![attempt.attempt_id.to_string(), failure_class, now],
            )
            .map_err(|source| StateError::sqlite("record semantic failure class", source))?;
        transition_nonterminal_operations(
            &transaction,
            &attempt.attempt_id,
            OperationState::Failed,
            now,
        )?;
        terminalize_cli_extension(
            &transaction,
            &attempt.attempt_id,
            CliAttemptState::Failed,
            CliActivityState::Failed,
            CliTerminalFailureClass::NonResumableProtocolFailure,
            now,
        )?;
        let task = require_task(&transaction, &attempt.run_id, &attempt.task_id)?;
        if matches!(task.state, TaskState::Running | TaskState::ResultPublished) {
            transition_task_state(
                &transaction,
                &attempt.run_id,
                &attempt.task_id,
                task.state,
                TaskState::Failed,
                now,
            )?;
        }
        fail_run(&transaction, &attempt.run_id, now)?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "terminal_semantic_failure",
            &json!({"failureClass": failure_class}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit terminal semantic failure", source))
    }

    pub fn fail_terminal_budget_exhausted(
        &mut self,
        lease: &LeaseToken,
        reason_code: &str,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        validate_label("reason code", reason_code)?;
        let transaction = self.immediate("begin terminal budget exhaustion")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        if load_cli_attempt(&transaction, lease.attempt_id())?.is_some() {
            require_attempt_state(
                &attempt,
                &[
                    AttemptState::Prepared,
                    AttemptState::Reconciling,
                    AttemptState::ResultPublished,
                ],
            )?;
        } else {
            require_attempt_state(
                &attempt,
                &[
                    AttemptState::TurnStarted,
                    AttemptState::Reconciling,
                    AttemptState::ResultPublished,
                ],
            )?;
        }
        charge_observed_wall(&transaction, &attempt.attempt_id, now)?;
        update_attempt_state(
            &transaction,
            &attempt.attempt_id,
            attempt.state,
            AttemptState::Failed,
            now,
        )?;
        transaction
            .execute(
                "UPDATE attempts
                 SET lease_owner = NULL, lease_expires_at = NULL, updated_at = ?2
                 WHERE attempt_id = ?1 AND state = 'failed'",
                params![attempt.attempt_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("release budget-exhausted lease", source))?;
        transition_nonterminal_operations(
            &transaction,
            &attempt.attempt_id,
            OperationState::Failed,
            now,
        )?;
        terminalize_cli_extension(
            &transaction,
            &attempt.attempt_id,
            CliAttemptState::Failed,
            CliActivityState::Failed,
            CliTerminalFailureClass::ContinuationExhausted,
            now,
        )?;
        let task = require_task(&transaction, &attempt.run_id, &attempt.task_id)?;
        if !matches!(task.state, TaskState::Running | TaskState::ResultPublished) {
            return Err(StateError::Conflict {
                entity: format!("task {}/{}", attempt.run_id, attempt.task_id),
                expected: "running|result_published".to_owned(),
                actual: task.state.as_str().to_owned(),
            });
        }
        transition_task_state(
            &transaction,
            &attempt.run_id,
            &attempt.task_id,
            task.state,
            TaskState::Failed,
            now,
        )?;
        fail_run(&transaction, &attempt.run_id, now)?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "terminal_budget_exhausted",
            &json!({"reasonCode": reason_code}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit terminal budget exhaustion", source))
    }

    pub fn mark_dispatching_turn_indeterminate(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        reason_code: &str,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        validate_label("reason code", reason_code)?;
        let transaction = self.immediate("begin dispatching turn indeterminate transition")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        require_attempt_state(&attempt, &[AttemptState::DispatchingTurn])?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        require_operation_kind(
            &operation,
            &[OperationKind::StartTurn, OperationKind::ContinueTurn],
        )?;
        require_operation_state(&operation, OperationState::Dispatching)?;
        update_operation_state(
            &transaction,
            operation_id,
            OperationState::Dispatching,
            OperationState::Indeterminate,
            now,
        )?;
        update_attempt_state(
            &transaction,
            &attempt.attempt_id,
            AttemptState::DispatchingTurn,
            AttemptState::Indeterminate,
            now,
        )?;
        transaction
            .execute(
                "UPDATE attempts
                 SET lease_owner = NULL, lease_expires_at = NULL, updated_at = ?2
                 WHERE attempt_id = ?1 AND state = 'indeterminate'",
                params![attempt.attempt_id.to_string(), now],
            )
            .map_err(|source| {
                StateError::sqlite("release indeterminate dispatching turn lease", source)
            })?;
        let cancellation_requested: i64 = transaction
            .query_row(
                "SELECT cancellation_requested FROM runs WHERE run_id = ?1",
                [attempt.run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("load dispatch ambiguity authority", source))?;
        if cancellation_requested == 1 {
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
        } else if retry_allowed(&transaction, &attempt.run_id, &attempt.task_id)? {
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
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "dispatching_turn_indeterminate",
            &json!({"operationId": operation_id, "reasonCode": reason_code}),
            now,
        )?;
        transaction.commit().map_err(|source| {
            StateError::sqlite("commit dispatching turn indeterminate transition", source)
        })
    }

    pub fn mark_indeterminate(
        &mut self,
        lease: &LeaseToken,
        reason_code: &str,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        validate_label("reason code", reason_code)?;
        let transaction = self.immediate("begin indeterminate transition")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        let cancellation_requested: i64 = transaction
            .query_row(
                "SELECT cancellation_requested
                 FROM runs
                 WHERE run_id = ?1 AND state = 'active'",
                [attempt.run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| {
                StateError::sqlite("load indeterminate cancellation authority", source)
            })?;
        if attempt.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("attempt {}", lease.attempt_id()),
                expected: "nonterminal state".to_owned(),
                actual: attempt.state.as_str().to_owned(),
            });
        }
        charge_observed_wall(&transaction, &attempt.attempt_id, now)?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'indeterminate',
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = ?2",
                params![lease.attempt_id().to_string(), attempt.state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("mark attempt indeterminate", source))?;
        require_changed(
            changed,
            format!("attempt {}", lease.attempt_id()),
            attempt.state.as_str(),
            "changed before indeterminate CAS",
        )?;
        let operation_state = if cancellation_requested == 1 {
            OperationState::Cancelled
        } else {
            OperationState::Indeterminate
        };
        transition_nonterminal_operations(&transaction, lease.attempt_id(), operation_state, now)?;
        terminalize_cli_extension(
            &transaction,
            lease.attempt_id(),
            CliAttemptState::Indeterminate,
            CliActivityState::Indeterminate,
            CliTerminalFailureClass::Indeterminate,
            now,
        )?;
        if cancellation_requested == 1 {
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
        } else if retry_allowed(&transaction, &attempt.run_id, &attempt.task_id)? {
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
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "attempt_indeterminate",
            &json!({"reasonCode": reason_code}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit indeterminate transition", source))
    }

    pub fn request_run_cancellation(&mut self, run_id: &RunId, now: i64) -> StateResult<bool> {
        let transaction = self.immediate("begin cancellation request")?;
        let run: Option<(String, i64)> = transaction
            .query_row(
                "SELECT state, cancellation_requested FROM runs WHERE run_id = ?1",
                [run_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|source| StateError::sqlite("load cancellation run", source))?;
        let Some((state, requested)) = run else {
            return Err(StateError::Conflict {
                entity: format!("run {run_id}"),
                expected: "existing active run".to_owned(),
                actual: "missing".to_owned(),
            });
        };
        let state = RunState::parse(&state)?;
        if requested == 1 {
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit repeated cancellation", source))?;
            return Ok(false);
        }
        if requested != 0 || state != RunState::Active {
            return Err(StateError::Conflict {
                entity: format!("run {run_id}"),
                expected: "active with cancellation_requested=false".to_owned(),
                actual: format!("{} with cancellation_requested={requested}", state.as_str()),
            });
        }
        let changed = transaction
            .execute(
                "UPDATE runs
                 SET cancellation_requested = 1, updated_at = ?2
                 WHERE run_id = ?1
                   AND state = 'active'
                   AND cancellation_requested = 0",
                params![run_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("request run cancellation", source))?;
        require_changed(
            changed,
            format!("run {run_id}"),
            "active with cancellation_requested=false",
            "changed before cancellation CAS",
        )?;
        cancel_nonrunning_tasks(&transaction, run_id, now)?;
        cancel_run_if_finished(&transaction, run_id, now)?;
        insert_event(
            &transaction,
            Some(run_id),
            None,
            None,
            "run_cancellation_requested",
            &json!({}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit cancellation request", source))?;
        Ok(true)
    }

    pub fn finalize_task_cancelled(&mut self, lease: &LeaseToken, now: i64) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin cancellation finalization")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        if attempt.state == AttemptState::DispatchingTurn {
            return Err(StateError::Conflict {
                entity: format!("attempt {} cancellation", lease.attempt_id()),
                expected: "dispatching turn requires reconciliation or indeterminate transition"
                    .to_owned(),
                actual: AttemptState::DispatchingTurn.as_str().to_owned(),
            });
        }
        charge_observed_wall(&transaction, &attempt.attempt_id, now)?;
        let dispatching_turn_operations: i64 = transaction
            .query_row(
                "SELECT COUNT(*)
                 FROM operations
                 WHERE attempt_id = ?1
                   AND kind IN ('start_turn', 'continue_turn')
                   AND state = 'dispatching'",
                [lease.attempt_id().to_string()],
                |row| row.get(0),
            )
            .map_err(|source| {
                StateError::sqlite("verify unresolved turn dispatch cancellation", source)
            })?;
        if dispatching_turn_operations != 0 {
            return Err(StateError::Conflict {
                entity: format!("attempt {} cancellation", lease.attempt_id()),
                expected: "dispatching turn requires reconciliation or indeterminate transition"
                    .to_owned(),
                actual: "dispatching start_turn or continue_turn operation".to_owned(),
            });
        }
        if attempt.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("attempt {}", lease.attempt_id()),
                expected: "nonterminal state".to_owned(),
                actual: attempt.state.as_str().to_owned(),
            });
        }
        let cancellation_requested: i64 = transaction
            .query_row(
                "SELECT cancellation_requested
                 FROM runs
                 WHERE run_id = ?1 AND state = 'active'",
                [attempt.run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("verify cancellation request", source))?;
        if cancellation_requested != 1 {
            return Err(StateError::Conflict {
                entity: format!("run {}", attempt.run_id),
                expected: "cancellation requested".to_owned(),
                actual: "cancellation not requested".to_owned(),
            });
        }
        let ambiguous_cli_processes: i64 = transaction
            .query_row(
                "SELECT COUNT(*)
                 FROM cli_activities
                 WHERE attempt_id = ?1
                   AND kind IN ('start_activity', 'continue_activity')
                   AND state = 'dispatching'
                   AND process_record_sha256 IS NULL",
                [lease.attempt_id().to_string()],
                |row| row.get(0),
            )
            .map_err(|source| {
                StateError::sqlite("verify ambiguous CLI cancellation process identity", source)
            })?;
        if ambiguous_cli_processes != 0 {
            return Err(StateError::Conflict {
                entity: format!("attempt {} CLI cancellation", lease.attempt_id()),
                expected: "indeterminate reconciliation before cancellation finalization"
                    .to_owned(),
                actual: format!("{ambiguous_cli_processes} dispatched activity record(s) without process identity"),
            });
        }
        let unresolved_cli_processes: i64 = transaction
            .query_row(
                "SELECT COUNT(*)
                 FROM cli_activities target
                 WHERE target.attempt_id = ?1
                   AND target.kind IN ('start_activity', 'continue_activity')
                   AND target.state IN ('dispatching', 'running', 'reconciling')
                   AND target.process_record_sha256 IS NOT NULL
                   AND NOT EXISTS (
                       SELECT 1
                       FROM cli_activities interrupt
                       WHERE interrupt.attempt_id = target.attempt_id
                         AND interrupt.kind = 'interrupt_activity'
                         AND interrupt.interrupt_purpose = 'cancellation'
                         AND interrupt.target_process_record_sha256 =
                             target.process_record_sha256
                         AND (
                             (
                                 interrupt.state = 'completed'
                                 AND interrupt.signal_stage = 'quiescent'
                             )
                             OR (
                                 interrupt.state = 'indeterminate'
                                 AND interrupt.signal_stage = 'indeterminate'
                             )
                         )
                   )",
                [lease.attempt_id().to_string()],
                |row| row.get(0),
            )
            .map_err(|source| {
                StateError::sqlite("verify terminal CLI cancellation interrupt", source)
            })?;
        if unresolved_cli_processes != 0 {
            return Err(StateError::Conflict {
                entity: format!("attempt {} CLI cancellation", lease.attempt_id()),
                expected: "terminal cancellation interrupt for every attributable CLI process"
                    .to_owned(),
                actual: format!("{unresolved_cli_processes} unresolved process record(s)"),
            });
        }
        if let Some(latest_turn_id) = &attempt.latest_turn_id {
            let completed_interrupts: i64 = transaction
                .query_row(
                    "SELECT COUNT(*)
                     FROM operations
                     WHERE attempt_id = ?1
                       AND kind = 'interrupt_turn'
                       AND target_external_id = ?2
                       AND state = 'completed'",
                    params![lease.attempt_id().to_string(), latest_turn_id.to_string()],
                    |row| row.get(0),
                )
                .map_err(|source| {
                    StateError::sqlite("verify completed cancellation interrupt", source)
                })?;
            if completed_interrupts == 0 {
                return Err(StateError::Conflict {
                    entity: format!("attempt {} cancellation", lease.attempt_id()),
                    expected: format!("completed interrupt_turn targeting {latest_turn_id}"),
                    actual: "missing".to_owned(),
                });
            }
        }
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'cancelled',
                     lease_owner = NULL,
                     lease_expires_at = NULL,
                     updated_at = ?3
                 WHERE attempt_id = ?1 AND state = ?2",
                params![lease.attempt_id().to_string(), attempt.state.as_str(), now],
            )
            .map_err(|source| StateError::sqlite("cancel attempt", source))?;
        require_changed(
            changed,
            format!("attempt {}", lease.attempt_id()),
            attempt.state.as_str(),
            "changed before cancellation CAS",
        )?;
        transition_nonterminal_operations(
            &transaction,
            lease.attempt_id(),
            OperationState::Cancelled,
            now,
        )?;
        terminalize_cli_extension(
            &transaction,
            lease.attempt_id(),
            CliAttemptState::Cancelled,
            CliActivityState::Cancelled,
            CliTerminalFailureClass::Cancelled,
            now,
        )?;
        cancel_competing_attempts(
            &transaction,
            &attempt.run_id,
            &attempt.task_id,
            lease.attempt_id(),
            now,
        )?;
        let task = require_task(&transaction, &attempt.run_id, &attempt.task_id)?;
        if !matches!(task.state, TaskState::Running | TaskState::ResultPublished) {
            return Err(StateError::Conflict {
                entity: format!("task {}/{}", attempt.run_id, attempt.task_id),
                expected: "running|result_published".to_owned(),
                actual: task.state.as_str().to_owned(),
            });
        }
        let changed = transaction
            .execute(
                "UPDATE tasks
                 SET state = 'cancelled', updated_at = ?4
                 WHERE run_id = ?1 AND task_id = ?2 AND state = ?3",
                params![
                    attempt.run_id.to_string(),
                    attempt.task_id.to_string(),
                    task.state.as_str(),
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("cancel task", source))?;
        require_changed(
            changed,
            format!("task {}/{}", attempt.run_id, attempt.task_id),
            task.state.as_str(),
            "changed before cancellation CAS",
        )?;
        cancel_nonrunning_tasks(&transaction, &attempt.run_id, now)?;
        cancel_run_if_finished(&transaction, &attempt.run_id, now)?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "task_cancelled",
            &json!({}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit cancellation finalization", source))
    }
}
