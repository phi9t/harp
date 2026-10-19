use super::*;

impl StateStore {
    pub fn prepare_turn_operation(
        &mut self,
        lease: &LeaseToken,
        preparation: &TurnOperationPreparation,
        now: i64,
    ) -> StateResult<OperationRecord> {
        let lease_now = self.lease_now(now)?;
        let operation_id = &preparation.operation_id;
        let kind = preparation.kind;
        let ordinal = preparation.ordinal;
        let intent = &preparation.intent;
        let checkpoint_sha256 = preparation.checkpoint_sha256.as_deref();
        if !matches!(kind, OperationKind::StartTurn | OperationKind::ContinueTurn) {
            return Err(StateError::invalid(
                "turn operation kind must be start_turn or continue_turn",
            ));
        }
        intent.validate().map_err(StateError::invalid_contract)?;
        if &intent.operation_marker != operation_id {
            return Err(StateError::invalid(
                "turn intent marker must equal operation id",
            ));
        }
        if let Some(checkpoint_sha256) = checkpoint_sha256 {
            validate_sha256("checkpoint digest", checkpoint_sha256)?;
        }
        if kind == OperationKind::StartTurn && checkpoint_sha256.is_some() {
            return Err(StateError::invalid(
                "initial turn operation cannot bind a checkpoint",
            ));
        }
        if kind == OperationKind::ContinueTurn && checkpoint_sha256.is_none() {
            return Err(StateError::invalid(
                "continuation operation requires a checkpoint digest",
            ));
        }
        let intent_json = bounded_json("turn operation intent", intent, MAX_JSON_BYTES)?;
        let intent_sha256 = format!("{:x}", Sha256::digest(&intent_json));
        let transaction = self.immediate("begin turn operation preparation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        reject_legacy_operation_for_cli_attempt(&transaction, lease.attempt_id(), kind)?;
        if let Some(existing) =
            load_operation_by_key(&transaction, lease.attempt_id(), kind, ordinal)?
        {
            if existing.turn_intent.as_ref() != Some(intent)
                || existing.intent_sha256.as_deref() != Some(intent_sha256.as_str())
                || existing.checkpoint_sha256.as_deref() != checkpoint_sha256
            {
                return Err(StateError::Conflict {
                    entity: format!("operation {} turn intent", existing.operation_id),
                    expected: intent_sha256,
                    actual: existing
                        .intent_sha256
                        .unwrap_or_else(|| "missing".to_owned()),
                });
            }
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit idempotent turn preparation", source)
            })?;
            return Ok(existing);
        }
        validate_operation_preparation(&transaction, &attempt, kind)?;
        transaction
            .execute(
                "INSERT INTO operations (
                    operation_id, attempt_id, kind, ordinal, state,
                    intent_json, intent_sha256, checkpoint_sha256,
                    created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, 'prepared', ?5, ?6, ?7, ?8, ?8)",
                params![
                    operation_id.to_string(),
                    lease.attempt_id().to_string(),
                    kind.as_str(),
                    i64::from(ordinal),
                    intent_json,
                    intent_sha256,
                    checkpoint_sha256,
                    now,
                ],
            )
            .map_err(|source| StateError::sqlite("insert prepared turn operation", source))?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "turn_operation_prepared",
            &json!({
                "operationId": operation_id,
                "kind": kind.as_str(),
                "ordinal": ordinal,
                "intentSha256": intent_sha256,
                "checkpointSha256": checkpoint_sha256,
            }),
            now,
        )?;
        let record = OperationRecord {
            operation_id: operation_id.clone(),
            attempt_id: lease.attempt_id().clone(),
            kind,
            ordinal,
            state: OperationState::Prepared,
            external_id: None,
            target_external_id: None,
            operation_marker: None,
            turn_intent: Some(intent.clone()),
            interrupt_intent: None,
            intent_sha256: Some(intent_sha256),
            checkpoint_sha256: checkpoint_sha256.map(str::to_owned),
            consumed_at: None,
            created_at: now,
            updated_at: now,
        };
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit turn operation preparation", source))?;
        Ok(record)
    }

    pub fn prepare_interrupt_operation(
        &mut self,
        lease: &LeaseToken,
        ordinal: u32,
        intent: &InterruptIntent,
        now: i64,
    ) -> StateResult<OperationRecord> {
        intent.validate()?;
        let lease_now = self.lease_now(now)?;
        let intent_json = bounded_json("interrupt operation intent", intent, MAX_JSON_BYTES)?;
        let intent_sha256 = format!("{:x}", Sha256::digest(&intent_json));
        let transaction = self.immediate("begin interrupt operation preparation")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, true)?;
        reject_legacy_operation_for_cli_attempt(
            &transaction,
            lease.attempt_id(),
            OperationKind::InterruptTurn,
        )?;
        if let Some(existing) = load_operation_by_key(
            &transaction,
            lease.attempt_id(),
            OperationKind::InterruptTurn,
            ordinal,
        )? {
            if existing.interrupt_intent.as_ref() != Some(intent)
                || existing.intent_sha256.as_deref() != Some(intent_sha256.as_str())
            {
                return Err(StateError::Conflict {
                    entity: format!("operation {} interrupt intent", existing.operation_id),
                    expected: intent_sha256,
                    actual: existing
                        .intent_sha256
                        .unwrap_or_else(|| "missing".to_owned()),
                });
            }
            transaction.commit().map_err(|source| {
                StateError::sqlite("commit idempotent interrupt preparation", source)
            })?;
            return Ok(existing);
        }
        validate_operation_preparation(&transaction, &attempt, OperationKind::InterruptTurn)?;
        let cancellation_requested: i64 = transaction
            .query_row(
                "SELECT cancellation_requested FROM runs WHERE run_id = ?1 AND state = 'active'",
                [attempt.run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| {
                StateError::sqlite("load interrupt cancellation authority", source)
            })?;
        let cancellation_intent = intent.purpose == crate::InterruptPurpose::Cancellation;
        if cancellation_intent != (cancellation_requested == 1) {
            return Err(StateError::Conflict {
                entity: format!("run {} interrupt purpose", attempt.run_id),
                expected: if cancellation_requested == 1 {
                    "cancellation".to_owned()
                } else {
                    "budget or scanner_integrity".to_owned()
                },
                actual: format!("{:?}", intent.purpose),
            });
        }
        let target_external_id = operation_target(&attempt, OperationKind::InterruptTurn)?;
        let operation_id = OperationId::new();
        transaction
            .execute(
                "INSERT INTO operations (
                    operation_id, attempt_id, kind, ordinal, state,
                    target_external_id, intent_json, intent_sha256,
                    created_at, updated_at
                 ) VALUES (?1, ?2, 'interrupt_turn', ?3, 'prepared', ?4, ?5, ?6, ?7, ?7)",
                params![
                    operation_id.to_string(),
                    lease.attempt_id().to_string(),
                    i64::from(ordinal),
                    target_external_id,
                    intent_json,
                    intent_sha256,
                    now,
                ],
            )
            .map_err(|source| StateError::sqlite("insert prepared interrupt operation", source))?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "interrupt_operation_prepared",
            &json!({
                "operationId": operation_id,
                "ordinal": ordinal,
                "intentSha256": intent_sha256,
                "purpose": intent.purpose,
                "reasonCode": intent.reason_code,
                "budgetDimension": intent.budget_dimension,
            }),
            now,
        )?;
        let record = OperationRecord {
            operation_id,
            attempt_id: lease.attempt_id().clone(),
            kind: OperationKind::InterruptTurn,
            ordinal,
            state: OperationState::Prepared,
            external_id: None,
            target_external_id,
            operation_marker: None,
            turn_intent: None,
            interrupt_intent: Some(intent.clone()),
            intent_sha256: Some(intent_sha256),
            checkpoint_sha256: None,
            consumed_at: None,
            created_at: now,
            updated_at: now,
        };
        transaction.commit().map_err(|source| {
            StateError::sqlite("commit interrupt operation preparation", source)
        })?;
        Ok(record)
    }

    pub fn prepare_operation(
        &mut self,
        lease: &LeaseToken,
        kind: OperationKind,
        ordinal: u32,
        now: i64,
    ) -> StateResult<OperationRecord> {
        let lease_now = self.lease_now(now)?;
        if kind == OperationKind::InterruptTurn {
            return Err(StateError::invalid(
                "interrupt operations require prepare_interrupt_operation",
            ));
        }
        let transaction = self.immediate("begin operation preparation")?;
        let attempt = authorize_lease(
            &transaction,
            lease,
            lease_now,
            kind == OperationKind::InterruptTurn,
        )?;
        if kind == OperationKind::PublishResult
            && load_cli_attempt(&transaction, lease.attempt_id())?.is_some()
        {
            return Err(StateError::Conflict {
                entity: format!("CLI attempt {} result publication", lease.attempt_id()),
                expected: "completion bridge-owned publish_result ordinal 0".to_owned(),
                actual: format!("generic publish_result ordinal {ordinal}"),
            });
        }
        reject_legacy_operation_for_cli_attempt(&transaction, lease.attempt_id(), kind)?;
        if let Some(existing) =
            load_operation_by_key(&transaction, lease.attempt_id(), kind, ordinal)?
        {
            let expected_target = operation_target(&attempt, kind)?;
            if existing.target_external_id != expected_target {
                return Err(StateError::Conflict {
                    entity: format!("operation {} target", existing.operation_id),
                    expected: expected_target.unwrap_or_else(|| "none".to_owned()),
                    actual: existing
                        .target_external_id
                        .clone()
                        .unwrap_or_else(|| "none".to_owned()),
                });
            }
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit idempotent preparation", source))?;
            return Ok(existing);
        }
        validate_operation_preparation(&transaction, &attempt, kind)?;
        let operation_id = OperationId::new();
        let record = insert_prepared_plain_operation(
            &transaction,
            &attempt,
            &operation_id,
            kind,
            ordinal,
            now,
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "operation_prepared",
            &json!({
                "operationId": operation_id,
                "kind": kind.as_str(),
                "ordinal": ordinal,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit operation preparation", source))?;
        Ok(record)
    }

    pub fn mark_operation_dispatching(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin generic operation dispatch")?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        let attempt = authorize_lease(
            &transaction,
            lease,
            lease_now,
            operation.kind == OperationKind::InterruptTurn,
        )?;
        require_operation_state(&operation, OperationState::Prepared)?;
        let next_attempt_state = match operation.kind {
            OperationKind::InterruptTurn | OperationKind::PublishResult => {
                require_attempt_state(
                    &attempt,
                    &[AttemptState::TurnStarted, AttemptState::Reconciling],
                )?;
                Some(AttemptState::Reconciling)
            }
            OperationKind::Evaluate => {
                require_attempt_state(
                    &attempt,
                    &[AttemptState::ResultPublished, AttemptState::Succeeded],
                )?;
                None
            }
            other => {
                return Err(StateError::Conflict {
                    entity: format!("operation {operation_id} kind"),
                    expected: "interrupt_turn|publish_result|evaluate".to_owned(),
                    actual: other.as_str().to_owned(),
                });
            }
        };
        update_operation_state(
            &transaction,
            operation_id,
            OperationState::Prepared,
            OperationState::Dispatching,
            now,
        )?;
        if let Some(next) = next_attempt_state {
            if attempt.state != next {
                update_attempt_state(&transaction, &attempt.attempt_id, attempt.state, next, now)?;
            }
        }
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "operation_dispatching",
            &json!({"operationId": operation_id, "kind": operation.kind.as_str()}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit generic operation dispatch", source))
    }

    pub fn complete_operation(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        external_id: Option<&str>,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        if let Some(external_id) = external_id {
            validate_label("operation external id", external_id)?;
        }
        let transaction = self.immediate("begin generic operation completion")?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        let attempt = authorize_lease(
            &transaction,
            lease,
            lease_now,
            operation.kind == OperationKind::InterruptTurn,
        )?;
        require_operation_kind(
            &operation,
            &[
                OperationKind::InterruptTurn,
                OperationKind::PublishResult,
                OperationKind::Evaluate,
            ],
        )?;
        require_operation_state(&operation, OperationState::Dispatching)?;
        let changed = transaction
            .execute(
                "UPDATE operations
                 SET state = 'completed',
                     external_id = ?2,
                     updated_at = ?3
                 WHERE operation_id = ?1
                   AND state = 'dispatching'
                   AND external_id IS NULL",
                params![operation_id.to_string(), external_id, now],
            )
            .map_err(|source| StateError::sqlite("complete generic operation", source))?;
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
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "operation_completed",
            &json!({
                "operationId": operation_id,
                "kind": operation.kind.as_str(),
                "externalId": external_id,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit generic operation completion", source))
    }

    pub fn mark_operation_failed(
        &mut self,
        lease: &LeaseToken,
        operation_id: &OperationId,
        transient: bool,
        failure_class: &str,
        now: i64,
    ) -> StateResult<()> {
        let lease_now = self.lease_now(now)?;
        validate_label("failure class", failure_class)?;
        let transaction = self.immediate("begin operation failure")?;
        let operation = require_operation(&transaction, operation_id)?;
        require_operation_attempt(&operation, lease)?;
        let attempt = authorize_lease(
            &transaction,
            lease,
            lease_now,
            operation.kind == OperationKind::InterruptTurn,
        )?;
        require_operation_state(&operation, OperationState::Dispatching)?;
        if attempt.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("attempt {}", attempt.attempt_id),
                expected: "nonterminal state".to_owned(),
                actual: attempt.state.as_str().to_owned(),
            });
        }
        charge_observed_wall(&transaction, &attempt.attempt_id, now)?;
        let changed = transaction
            .execute(
                "UPDATE operations
                 SET state = 'failed',
                     transient = ?2,
                     failure_class = ?3,
                     updated_at = ?4
                 WHERE operation_id = ?1 AND state = 'dispatching'",
                params![
                    operation_id.to_string(),
                    i64::from(transient),
                    failure_class,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("mark operation failed", source))?;
        require_changed(
            changed,
            format!("operation {operation_id}"),
            OperationState::Dispatching.as_str(),
            operation.state.as_str(),
        )?;
        match operation.kind {
            OperationKind::InterruptTurn | OperationKind::Evaluate => {}
            OperationKind::StartThread
            | OperationKind::StartTurn
            | OperationKind::ContinueTurn
            | OperationKind::PublishResult => {
                let changed = transaction
                    .execute(
                        "UPDATE attempts
                         SET state = 'failed',
                             lease_owner = NULL,
                             lease_expires_at = NULL,
                             updated_at = ?3
                         WHERE attempt_id = ?1 AND state = ?2",
                        params![attempt.attempt_id.to_string(), attempt.state.as_str(), now],
                    )
                    .map_err(|source| {
                        StateError::sqlite("mark operation attempt failed", source)
                    })?;
                require_changed(
                    changed,
                    format!("attempt {}", attempt.attempt_id),
                    attempt.state.as_str(),
                    "changed before failure CAS",
                )?;
                terminalize_cli_extension(
                    &transaction,
                    &attempt.attempt_id,
                    CliAttemptState::Failed,
                    CliActivityState::Failed,
                    CliTerminalFailureClass::NonResumableProtocolFailure,
                    now,
                )?;
                if transient && retry_allowed(&transaction, &attempt.run_id, &attempt.task_id)? {
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
        }
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(&attempt.attempt_id),
            "operation_failed",
            &json!({
                "operationId": operation_id,
                "transient": transient,
                "failureClass": failure_class,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit operation failure", source))
    }
}

pub(super) fn insert_prepared_plain_operation(
    transaction: &Transaction<'_>,
    attempt: &AttemptRecord,
    operation_id: &OperationId,
    kind: OperationKind,
    ordinal: u32,
    now: i64,
) -> StateResult<OperationRecord> {
    if matches!(
        kind,
        OperationKind::StartTurn | OperationKind::ContinueTurn | OperationKind::InterruptTurn
    ) {
        return Err(StateError::invalid(
            "plain operation insert does not accept intent-bearing kinds",
        ));
    }
    validate_operation_preparation(transaction, attempt, kind)?;
    let target_external_id = operation_target(attempt, kind)?;
    transaction
        .execute(
            "INSERT INTO operations (
                operation_id, attempt_id, kind, ordinal, state,
                target_external_id, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, 'prepared', ?5, ?6, ?6)",
            params![
                operation_id.to_string(),
                attempt.attempt_id.to_string(),
                kind.as_str(),
                i64::from(ordinal),
                target_external_id,
                now
            ],
        )
        .map_err(|source| StateError::sqlite("insert prepared operation", source))?;
    Ok(OperationRecord {
        operation_id: operation_id.clone(),
        attempt_id: attempt.attempt_id.clone(),
        kind,
        ordinal,
        state: OperationState::Prepared,
        external_id: None,
        target_external_id,
        operation_marker: None,
        turn_intent: None,
        interrupt_intent: None,
        intent_sha256: None,
        checkpoint_sha256: None,
        consumed_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn reject_legacy_operation_for_cli_attempt(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
    kind: OperationKind,
) -> StateResult<()> {
    if matches!(kind, OperationKind::PublishResult | OperationKind::Evaluate) {
        return Ok(());
    }
    if load_cli_attempt(transaction, attempt_id)?.is_some() {
        return Err(StateError::Conflict {
            entity: format!("CLI attempt {attempt_id} legacy operation"),
            expected: "CLI activity lifecycle".to_owned(),
            actual: kind.as_str().to_owned(),
        });
    }
    Ok(())
}
