use super::*;

impl StateStore {
    pub fn mark_dispatching_thread(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin thread dispatch")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        require_operation_kind(&operation, &[OperationKind::StartThread])?;
        require_operation_state(&operation, OperationState::Prepared)?;
        require_attempt_state(&attempt, &[AttemptState::Prepared])?;
        update_operation_state(
            &transaction,
            operation_id,
            OperationState::Prepared,
            OperationState::Dispatching,
            now,
        )?;
        update_attempt_state(
            &transaction,
            &attempt.attempt_id,
            AttemptState::Prepared,
            AttemptState::DispatchingThread,
            now,
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "thread_dispatching",
            &json!({"operationId": operation_id}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit thread dispatch", source))
    }

    pub fn record_thread_started(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        thread_id: &ThreadId,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin thread started recording")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        require_operation_kind(&operation, &[OperationKind::StartThread])?;
        if operation.state == OperationState::Completed
            && operation.external_id.as_deref() == Some(&thread_id.to_string())
            && attempt.state == AttemptState::ThreadStarted
            && attempt.thread_id.as_ref() == Some(thread_id)
        {
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit idempotent thread record", source))?;
            return Ok(());
        }
        require_operation_state(&operation, OperationState::Dispatching)?;
        require_attempt_state(&attempt, &[AttemptState::DispatchingThread])?;
        let changed = transaction
            .execute(
                "UPDATE operations
                 SET state = 'completed', external_id = ?2, updated_at = ?3
                 WHERE operation_id = ?1
                   AND state = 'dispatching'
                   AND external_id IS NULL",
                params![operation_id.to_string(), thread_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("record thread operation", source))?;
        require_changed(
            changed,
            format!("operation {operation_id}"),
            "dispatching with no external id",
            format!(
                "{} with {:?}",
                operation.state.as_str(),
                operation.external_id
            ),
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'thread_started', thread_id = ?2, updated_at = ?3
                 WHERE attempt_id = ?1
                   AND state = 'dispatching_thread'
                   AND thread_id IS NULL",
                params![attempt.attempt_id.to_string(), thread_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("record attempt thread", source))?;
        require_changed(
            changed,
            format!("attempt {}", attempt.attempt_id),
            "dispatching_thread with no thread id",
            format!("{} with {:?}", attempt.state.as_str(), attempt.thread_id),
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "thread_started",
            &json!({"operationId": operation_id, "threadId": thread_id}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit thread started", source))
    }

    pub fn mark_dispatching_turn(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        now: i64,
    ) -> StateResult<String> {
        let lease_now = self.lease_now(now)?;
        let operation_marker = format!("harp-operation:{operation_id}");
        let transaction = self.immediate("begin turn dispatch")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        require_operation_kind(
            &operation,
            &[OperationKind::StartTurn, OperationKind::ContinueTurn],
        )?;
        if operation.turn_intent.is_none() || operation.intent_sha256.is_none() {
            return Err(StateError::Integrity {
                context: "turn dispatch requires persisted exact turn intent".to_owned(),
                source: None,
            });
        }
        if operation.state == OperationState::Dispatching
            && operation.operation_marker.as_deref() == Some(operation_marker.as_str())
            && attempt.state == AttemptState::DispatchingTurn
            && attempt.latest_operation_marker.as_deref() == Some(operation_marker.as_str())
        {
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit idempotent turn dispatch", source))?;
            return Ok(operation_marker);
        }
        require_operation_state(&operation, OperationState::Prepared)?;
        let expected_attempts = match operation.kind {
            OperationKind::StartTurn => &[AttemptState::ThreadStarted][..],
            OperationKind::ContinueTurn => {
                &[AttemptState::TurnStarted, AttemptState::Reconciling][..]
            }
            _ => unreachable!("kind checked"),
        };
        require_attempt_state(&attempt, expected_attempts)?;
        if attempt.thread_id.is_none()
            || (operation.kind == OperationKind::ContinueTurn && attempt.latest_turn_id.is_none())
        {
            return Err(StateError::Integrity {
                context: "turn dispatch requires persisted thread/turn identity".to_owned(),
                source: None,
            });
        }
        if operation.kind == OperationKind::ContinueTurn
            && attempt.continuation_count <= u64::from(operation.ordinal)
        {
            return Err(StateError::Conflict {
                entity: format!("attempt {} continuation", attempt.attempt_id),
                expected: format!("count greater than operation ordinal {}", operation.ordinal),
                actual: attempt.continuation_count.to_string(),
            });
        }
        let changed = transaction
            .execute(
                "UPDATE operations
                 SET state = 'dispatching',
                     operation_marker = ?2,
                     updated_at = ?3
                 WHERE operation_id = ?1
                   AND state = 'prepared'
                   AND operation_marker IS NULL",
                params![operation_id.to_string(), operation_marker, now],
            )
            .map_err(|source| StateError::sqlite("mark turn operation dispatching", source))?;
        require_changed(
            changed,
            format!("operation {operation_id}"),
            "prepared with no operation marker",
            format!(
                "{} with {:?}",
                operation.state.as_str(),
                operation.operation_marker
            ),
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'dispatching_turn',
                     latest_operation_marker = ?2,
                     updated_at = ?3
                 WHERE attempt_id = ?1
                   AND state IN (?4, ?5)",
                params![
                    attempt.attempt_id.to_string(),
                    operation_marker,
                    now,
                    expected_attempts[0].as_str(),
                    expected_attempts
                        .get(1)
                        .copied()
                        .unwrap_or(expected_attempts[0])
                        .as_str()
                ],
            )
            .map_err(|source| StateError::sqlite("mark turn dispatching", source))?;
        require_changed(
            changed,
            format!("attempt {}", attempt.attempt_id),
            expected_attempts
                .iter()
                .map(|state| state.as_str())
                .collect::<Vec<_>>()
                .join("|"),
            attempt.state.as_str(),
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "turn_dispatching",
            &json!({
                "operationId": operation_id,
                "operationMarker": operation_marker,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit turn dispatch", source))?;
        Ok(operation_marker)
    }

    pub fn record_turn_started(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        turn_id: &TurnId,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin turn started recording")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        require_operation_kind(
            &operation,
            &[OperationKind::StartTurn, OperationKind::ContinueTurn],
        )?;
        if operation.state == OperationState::Completed
            && operation.external_id.as_deref() == Some(&turn_id.to_string())
            && attempt.state == AttemptState::TurnStarted
            && attempt.latest_turn_id.as_ref() == Some(turn_id)
        {
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit idempotent turn record", source))?;
            return Ok(());
        }
        require_operation_state(&operation, OperationState::Dispatching)?;
        require_attempt_state(&attempt, &[AttemptState::DispatchingTurn])?;
        let changed = transaction
            .execute(
                "UPDATE operations
                 SET state = 'completed', external_id = ?2, updated_at = ?3
                 WHERE operation_id = ?1
                   AND state = 'dispatching'
                   AND external_id IS NULL",
                params![operation_id.to_string(), turn_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("record turn operation", source))?;
        require_changed(
            changed,
            format!("operation {operation_id}"),
            "dispatching with no external id",
            format!(
                "{} with {:?}",
                operation.state.as_str(),
                operation.external_id
            ),
        )?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET state = 'turn_started',
                     latest_turn_id = ?2,
                 latest_turn_observed_tokens = 0,
                     updated_at = ?3
                 WHERE attempt_id = ?1
                   AND state = 'dispatching_turn'
                   AND latest_operation_marker = ?4",
                params![
                    attempt.attempt_id.to_string(),
                    turn_id.to_string(),
                    now,
                    operation.operation_marker.as_deref().ok_or_else(|| {
                        StateError::integrity("dispatching turn operation has no marker")
                    })?
                ],
            )
            .map_err(|source| StateError::sqlite("record attempt turn", source))?;
        require_changed(
            changed,
            format!("attempt {}", attempt.attempt_id),
            "dispatching_turn with matching latest operation marker",
            format!(
                "{} with {:?}",
                attempt.state.as_str(),
                attempt.latest_turn_id
            ),
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "turn_started",
            &json!({"operationId": operation_id, "turnId": turn_id}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit turn started", source))
    }

    pub fn increment_continuation(
        &mut self,
        lease: &LeaseToken,
        expected_count: u64,
        max_count: u64,
        now: i64,
    ) -> StateResult<u64> {
        let lease_now = self.lease_now(now)?;
        let expected = checked_u64_to_i64("expected continuation count", expected_count)?;
        checked_u64_to_i64("maximum continuation count", max_count)?;
        if expected_count >= max_count {
            return Err(StateError::LimitExceeded {
                context: "continuation count".to_owned(),
                limit: max_count,
                actual: expected_count + 1,
            });
        }
        let next_count =
            expected_count
                .checked_add(1)
                .ok_or_else(|| StateError::LimitExceeded {
                    context: "continuation count".to_owned(),
                    limit: max_count,
                    actual: u64::MAX,
                })?;
        let next = checked_u64_to_i64("next continuation count", next_count)?;
        let transaction = self.immediate("begin continuation increment")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET continuation_count = ?3, updated_at = ?4
                 WHERE attempt_id = ?1
                   AND continuation_count = ?2
                   AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
                   AND EXISTS (
                       SELECT 1 FROM runs
                       WHERE run_id = attempts.run_id
                         AND state = 'active'
                         AND cancellation_requested = 0
                   )",
                params![lease.attempt_id().to_string(), expected, next, now],
            )
            .map_err(|source| StateError::sqlite("increment continuation", source))?;
        require_changed(
            changed,
            format!("attempt {} continuation", lease.attempt_id()),
            expected_count.to_string(),
            attempt.continuation_count.to_string(),
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "continuation_incremented",
            &json!({"previousCount": expected_count, "newCount": next_count}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit continuation increment", source))?;
        Ok(next_count)
    }

    pub fn mark_reconciling(&mut self, lease: &LeaseToken, now: i64) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin reconciliation transition")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        require_attempt_state(&attempt, &[AttemptState::TurnStarted])?;
        update_attempt_state(
            &transaction,
            lease.attempt_id(),
            AttemptState::TurnStarted,
            AttemptState::Reconciling,
            now,
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "attempt_reconciling",
            &json!({}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit reconciliation transition", source))
    }
}
