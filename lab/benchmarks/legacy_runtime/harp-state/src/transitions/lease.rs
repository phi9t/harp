use super::*;

impl StateStore {
    pub fn restore_task_claim(&mut self, lease: &LeaseToken, now: i64) -> StateResult<TaskClaim> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin task claim restoration")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let (reserved_tokens, reserved_storage, reserved_wall): (i64, i64, i64) = transaction
            .query_row(
                "SELECT reserved_tokens, reserved_storage_bytes, reserved_wall_seconds
                 FROM budget_reservations WHERE attempt_id = ?1",
                [attempt.attempt_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|source| StateError::sqlite("load restored task budget", source))?;
        let claim = TaskClaim {
            run_id: attempt.run_id,
            task_id: attempt.task_id,
            attempt_id: attempt.attempt_id,
            lease_owner: lease.owner().to_owned(),
            lease_expires_at: lease.expires_at(),
            budget: Budget::new(
                checked_i64_to_u64("restored reserved tokens", reserved_tokens)?,
                checked_i64_to_u64("restored reserved wall seconds", reserved_wall)?,
                checked_i64_to_u64("restored reserved storage", reserved_storage)?,
            ),
            lease_capability: Arc::clone(&lease.capability),
        };
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit task claim restoration", source))?;
        Ok(claim)
    }

    pub fn claim_ready_task(
        &mut self,
        run_id: &RunId,
        worker: &str,
        now: i64,
        lease_until: i64,
    ) -> StateResult<Option<TaskClaim>> {
        self.claim_ready_task_internal(run_id, worker, now, lease_until, None)
    }

    pub fn claim_ready_cli_task(
        &mut self,
        run_id: &RunId,
        worker: &str,
        now: i64,
        lease_until: i64,
        logical_session_id: ThreadId,
    ) -> StateResult<Option<TaskClaim>> {
        self.claim_ready_task_internal(run_id, worker, now, lease_until, Some(&logical_session_id))
    }

    fn claim_ready_task_internal(
        &mut self,
        run_id: &RunId,
        worker: &str,
        now: i64,
        lease_until: i64,
        logical_session_id: Option<&ThreadId>,
    ) -> StateResult<Option<TaskClaim>> {
        validate_label("worker", worker)?;
        let lease_now = self.lease_now(now)?;
        if lease_until <= lease_now {
            return Err(StateError::invalid("lease_until must be greater than now"));
        }

        let transaction = self.immediate("begin task claim")?;
        let selected: Option<(String, Vec<u8>, i64, i64, i64)> = transaction
            .query_row(
                "SELECT t.task_id, t.task_json,
                        r.max_tokens, r.max_storage_bytes, r.max_wall_seconds
                 FROM tasks t
                 JOIN runs r ON r.run_id = t.run_id
                 WHERE t.run_id = ?1
                   AND t.state = 'ready'
                   AND r.state = 'active'
                   AND r.cancellation_requested = 0
                 ORDER BY t.task_id
                 LIMIT 1",
                [run_id.to_string()],
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
            .map_err(|source| StateError::sqlite("select ready task", source))?;
        let Some((task_id_text, task_json, max_tokens, max_storage, max_wall)) = selected else {
            transaction
                .commit()
                .map_err(|source| StateError::sqlite("commit empty task claim", source))?;
            return Ok(None);
        };
        let task_id = parse_id("claimed task id", task_id_text)?;
        let node = parse_task_node(&task_json)?;
        if node.task_id != task_id {
            return Err(StateError::integrity(
                "persisted task JSON identity does not match task row",
            ));
        }
        let (used_tokens, used_storage, used_wall) =
            aggregate_run_reservations(&transaction, run_id)?;
        let remaining_tokens = u128::from(checked_i64_to_u64("run token limit", max_tokens)?)
            .saturating_sub(used_tokens);
        let remaining_storage = u128::from(checked_i64_to_u64("run storage limit", max_storage)?)
            .saturating_sub(used_storage);
        let remaining_wall =
            u128::from(checked_i64_to_u64("run wall limit", max_wall)?).saturating_sub(used_wall);
        let claim_budget = Budget::new(
            node.budget
                .max_tokens
                .min(u64::try_from(remaining_tokens).unwrap_or(u64::MAX)),
            node.budget
                .timeout_seconds
                .min(u64::try_from(remaining_wall).unwrap_or(u64::MAX)),
            node.budget
                .max_storage_bytes
                .min(u64::try_from(remaining_storage).unwrap_or(u64::MAX)),
        );
        if claim_budget.max_tokens == 0
            || claim_budget.max_storage_bytes == 0
            || claim_budget.timeout_seconds == 0
        {
            return Err(StateError::BudgetExceeded {
                dimension: "remaining_run_budget".to_owned(),
                limit: 0,
                attempted: 1,
            });
        }
        let reserved_tokens = checked_u64_to_i64("task reserved tokens", claim_budget.max_tokens)?;
        let reserved_storage =
            checked_u64_to_i64("task reserved storage", claim_budget.max_storage_bytes)?;
        let reserved_wall =
            checked_u64_to_i64("task reserved wall seconds", claim_budget.timeout_seconds)?;
        enforce_reservation_budget(
            "tokens",
            used_tokens,
            u128::from(checked_i64_to_u64(
                "requested token reservation",
                reserved_tokens,
            )?),
            u128::from(checked_i64_to_u64("run token limit", max_tokens)?),
        )?;
        enforce_reservation_budget(
            "storage_bytes",
            used_storage,
            u128::from(checked_i64_to_u64(
                "requested storage reservation",
                reserved_storage,
            )?),
            u128::from(checked_i64_to_u64("run storage limit", max_storage)?),
        )?;
        enforce_reservation_budget(
            "wall_seconds",
            used_wall,
            u128::from(checked_i64_to_u64(
                "requested wall reservation",
                reserved_wall,
            )?),
            u128::from(checked_i64_to_u64("run wall limit", max_wall)?),
        )?;

        let changed = transaction
            .execute(
                "UPDATE tasks
                 SET state = 'running', updated_at = ?3
                 WHERE run_id = ?1
                   AND task_id = ?2
                   AND state = 'ready'
                   AND EXISTS (
                       SELECT 1 FROM runs
                       WHERE run_id = ?1
                         AND state = 'active'
                         AND cancellation_requested = 0
                   )",
                params![run_id.to_string(), task_id.to_string(), now],
            )
            .map_err(|source| StateError::sqlite("claim ready task", source))?;
        require_changed(
            changed,
            format!("task {run_id}/{task_id}"),
            "ready in active noncancelled run",
            "changed before claim",
        )?;

        let previous_ordinal: i64 = transaction
            .query_row(
                "SELECT COALESCE(MAX(ordinal), -1)
                 FROM attempts WHERE run_id = ?1 AND task_id = ?2",
                params![run_id.to_string(), task_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("find attempt ordinal", source))?;
        let ordinal_i64 =
            previous_ordinal
                .checked_add(1)
                .ok_or_else(|| StateError::LimitExceeded {
                    context: "attempt ordinal".to_owned(),
                    limit: u32::MAX as u64,
                    actual: u64::MAX,
                })?;
        let ordinal = checked_i64_to_u32("attempt ordinal", ordinal_i64)?;
        let attempt_id = AttemptId::new();
        let lease_capability = uuid::Uuid::new_v4().hyphenated().to_string();
        let lease_capability_sha256 = format!("{:x}", Sha256::digest(lease_capability.as_bytes()));
        transaction
            .execute(
                "INSERT INTO attempts (
                    attempt_id, run_id, task_id, ordinal, state,
                    lease_owner, lease_expires_at, last_lease_expires_at,
                    lease_capability_sha256, continuation_count,
                    observed_tokens, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, 'prepared', ?5, ?6, ?6, ?7, 0, 0, ?8, ?8)",
                params![
                    attempt_id.to_string(),
                    run_id.to_string(),
                    task_id.to_string(),
                    ordinal_i64,
                    worker,
                    lease_until,
                    lease_capability_sha256,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("insert claimed attempt", source))?;
        if let Some(logical_session_id) = logical_session_id {
            transaction
                .execute(
                    "INSERT INTO cli_attempts (
                        attempt_id, state, logical_session_id,
                        continuation_count, created_at, updated_at
                     ) VALUES (?1, 'prepared', ?2, 0, ?3, ?3)",
                    params![attempt_id.to_string(), logical_session_id.to_string(), now],
                )
                .map_err(|source| StateError::sqlite("insert claimed CLI attempt", source))?;
        }
        transaction
            .execute(
                "INSERT INTO budget_reservations (
                    attempt_id, reserved_tokens, reserved_storage_bytes,
                    reserved_wall_seconds, observed_tokens,
                    observed_storage_bytes, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, 0, 0, ?5, ?5)",
                params![
                    attempt_id.to_string(),
                    reserved_tokens,
                    reserved_storage,
                    reserved_wall,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("reserve attempt budget", source))?;
        insert_event(
            &transaction,
            Some(run_id),
            Some(&task_id),
            Some(&attempt_id),
            "task_claimed",
            &json!({
                "attemptId": attempt_id,
                "leaseOwner": worker,
                "leaseExpiresAt": lease_until,
                "ordinal": ordinal,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit task claim", source))?;
        Ok(Some(TaskClaim {
            run_id: run_id.clone(),
            task_id,
            attempt_id,
            lease_owner: worker.to_owned(),
            lease_expires_at: lease_until,
            budget: claim_budget,
            lease_capability: Arc::new(LeaseCapability(Zeroizing::new(lease_capability))),
        }))
    }

    pub fn renew_lease(
        &mut self,
        lease: &LeaseToken,
        new_expires_at: i64,
        now: i64,
    ) -> StateResult<LeaseToken> {
        let lease_now = self.lease_now(now)?;
        if new_expires_at <= lease.expires_at() || new_expires_at <= lease_now {
            return Err(StateError::invalid(
                "new lease expiry must exceed current expiry and now",
            ));
        }
        let transaction = self.immediate("begin lease renewal")?;
        let attempt = authorize_lease(&transaction, lease, lease_now, false)?;
        let new_capability = uuid::Uuid::new_v4().hyphenated().to_string();
        let new_capability_sha256 = format!("{:x}", Sha256::digest(new_capability.as_bytes()));
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET lease_expires_at = ?4,
                     lease_capability_sha256 = ?6,
                     updated_at = ?5
                 WHERE attempt_id = ?1
                   AND lease_owner = ?2
                   AND lease_expires_at = ?3
                   AND lease_expires_at > ?7
                   AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
                   AND EXISTS (
                       SELECT 1 FROM runs
                       WHERE run_id = attempts.run_id
                         AND state = 'active'
                         AND cancellation_requested = 0
                   )",
                params![
                    lease.attempt_id().to_string(),
                    lease.owner(),
                    lease.expires_at(),
                    new_expires_at,
                    now,
                    new_capability_sha256,
                    lease_now
                ],
            )
            .map_err(|source| StateError::sqlite("renew lease", source))?;
        require_changed(
            changed,
            format!("attempt {} lease", lease.attempt_id()),
            format!(
                "{}@{} unexpired in active noncancelled run",
                lease.owner(),
                lease.expires_at()
            ),
            format!(
                "{:?}@{:?}, state {}",
                attempt.lease_owner,
                attempt.lease_expires_at,
                attempt.state.as_str()
            ),
        )?;
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(lease.attempt_id()),
            "lease_renewed",
            &json!({
                "previousExpiresAt": lease.expires_at(),
                "newExpiresAt": new_expires_at,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit lease renewal", source))?;
        Ok(LeaseToken::new(
            lease.attempt_id().clone(),
            lease.owner().to_owned(),
            new_expires_at,
            new_capability,
        ))
    }

    pub fn reclaim_attempt(
        &mut self,
        attempt_id: &AttemptId,
        new_owner: &str,
        expected_expired_at: i64,
        now: i64,
        new_expires_at: i64,
    ) -> StateResult<LeaseToken> {
        validate_label("worker", new_owner)?;
        let lease_now = self.lease_now(now)?;
        if expected_expired_at > lease_now || new_expires_at <= lease_now {
            return Err(StateError::invalid(
                "reclaim requires an expired lease and a future new expiry",
            ));
        }
        let transaction = self.immediate("begin attempt reclaim")?;
        let attempt = require_attempt(&transaction, attempt_id)?;
        if attempt.state.is_terminal() {
            return Err(StateError::Conflict {
                entity: format!("attempt {attempt_id}"),
                expected: "nonterminal attempt".to_owned(),
                actual: attempt.state.as_str().to_owned(),
            });
        }
        let task = require_task(&transaction, &attempt.run_id, &attempt.task_id)?;
        let cancellation_requested: i64 = transaction
            .query_row(
                "SELECT cancellation_requested
                 FROM runs
                 WHERE run_id = ?1 AND state = 'active'",
                [attempt.run_id.to_string()],
                |row| row.get(0),
            )
            .map_err(|source| StateError::sqlite("load reclaim run authority", source))?;
        let task_reclaimable = match cancellation_requested {
            0 => matches!(
                task.state,
                TaskState::Running | TaskState::Ready | TaskState::ResultPublished
            ),
            1 => task.state == TaskState::Running,
            _ => {
                return Err(StateError::integrity(
                    "run cancellation flag is not boolean during reclaim",
                ));
            }
        };
        if !task_reclaimable {
            return Err(StateError::Conflict {
                entity: format!("task {}/{}", attempt.run_id, attempt.task_id),
                expected: if cancellation_requested == 1 {
                    "running under cancellation".to_owned()
                } else {
                    "running|ready".to_owned()
                },
                actual: task.state.as_str().to_owned(),
            });
        }
        let new_capability = uuid::Uuid::new_v4().hyphenated().to_string();
        let new_capability_sha256 = format!("{:x}", Sha256::digest(new_capability.as_bytes()));
        let changed = transaction
            .execute(
                "UPDATE attempts
                 SET lease_owner = ?4,
                     lease_expires_at = ?5,
                     lease_capability_sha256 = ?6,
                     last_lease_expires_at = ?3,
                     updated_at = ?2
                 WHERE attempt_id = ?1
                   AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
                   AND (
                       (
                           lease_owner IS NULL
                           AND lease_expires_at IS NULL
                           AND last_lease_expires_at = ?3
                       )
                       OR lease_expires_at = ?3
                   )
                   AND (?3 <= ?7)
                   AND EXISTS (
                       SELECT 1 FROM tasks
                       WHERE run_id = attempts.run_id
                         AND task_id = attempts.task_id
                         AND (
                             state = 'running'
                             OR state = 'result_published'
                             OR (
                                 state = 'ready'
                                 AND EXISTS (
                                     SELECT 1 FROM runs
                                     WHERE run_id = attempts.run_id
                                       AND state = 'active'
                                       AND cancellation_requested = 0
                                 )
                             )
                         )
                   )
                   AND EXISTS (
                       SELECT 1 FROM runs
                       WHERE run_id = attempts.run_id
                         AND state = 'active'
                   )",
                params![
                    attempt_id.to_string(),
                    now,
                    expected_expired_at,
                    new_owner,
                    new_expires_at,
                    new_capability_sha256,
                    lease_now
                ],
            )
            .map_err(|source| StateError::sqlite("reclaim attempt", source))?;
        require_changed(
            changed,
            format!("attempt {attempt_id} lease"),
            format!("expired at {expected_expired_at}"),
            format!("{:?}@{:?}", attempt.lease_owner, attempt.lease_expires_at),
        )?;
        if task.state == TaskState::Ready && cancellation_requested == 0 {
            let changed = transaction
                .execute(
                    "UPDATE tasks
                     SET state = 'running', updated_at = ?3
                     WHERE run_id = ?1 AND task_id = ?2 AND state = 'ready'",
                    params![attempt.run_id.to_string(), attempt.task_id.to_string(), now],
                )
                .map_err(|source| StateError::sqlite("reactivate reclaimed task", source))?;
            require_changed(
                changed,
                format!("task {}/{}", attempt.run_id, attempt.task_id),
                TaskState::Ready.as_str(),
                "changed before reclaim CAS",
            )?;
        }
        insert_event(
            &transaction,
            Some(&attempt.run_id),
            Some(&attempt.task_id),
            Some(attempt_id),
            "attempt_reclaimed",
            &json!({
                "leaseOwner": new_owner,
                "previousExpiresAt": expected_expired_at,
                "leaseExpiresAt": new_expires_at,
            }),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit attempt reclaim", source))?;
        Ok(LeaseToken::new(
            attempt_id.clone(),
            new_owner.to_owned(),
            new_expires_at,
            new_capability,
        ))
    }

    pub fn expire_leases(&mut self, run_id: &RunId, now: i64) -> StateResult<usize> {
        let lease_now = self.lease_now(now)?;
        let transaction = self.immediate("begin lease expiry")?;
        let mut statement = transaction
            .prepare(
                "SELECT attempt_id, task_id, lease_owner, lease_expires_at
                 FROM attempts
                 WHERE run_id = ?1
                   AND lease_expires_at IS NOT NULL
                   AND lease_expires_at <= ?2
                 ORDER BY attempt_id
                 LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare expired leases", source))?;
        let rows = statement
            .query_map(params![run_id.to_string(), lease_now], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|source| StateError::sqlite("query expired leases", source))?;
        let mut expired = Vec::new();
        for row in rows {
            expired.push(row.map_err(|source| StateError::sqlite("read expired lease", source))?);
            if expired.len() > MAX_RECOVERY_ROWS {
                return Err(StateError::LimitExceeded {
                    context: "expired leases".to_owned(),
                    limit: MAX_RECOVERY_ROWS as u64,
                    actual: expired.len() as u64,
                });
            }
        }
        drop(statement);

        for (attempt_text, task_text, owner, expiry) in &expired {
            let attempt_id: AttemptId = parse_id("expired attempt id", attempt_text.clone())?;
            let task_id: TaskId = parse_id("expired task id", task_text.clone())?;
            let changed = transaction
                .execute(
                    "UPDATE attempts
                     SET last_lease_expires_at = lease_expires_at,
                         lease_owner = NULL,
                         lease_expires_at = NULL,
                         updated_at = ?4
                     WHERE attempt_id = ?1
                       AND lease_owner = ?2
                       AND lease_expires_at = ?3",
                    params![attempt_text, owner, expiry, now],
                )
                .map_err(|source| StateError::sqlite("expire lease", source))?;
            require_changed(
                changed,
                format!("attempt {attempt_id} lease"),
                format!("{owner}@{expiry}"),
                "changed during expiry scan",
            )?;
            insert_event(
                &transaction,
                Some(run_id),
                Some(&task_id),
                Some(&attempt_id),
                "lease_expired",
                &json!({"leaseOwner": owner, "leaseExpiresAt": expiry}),
                now,
            )?;
        }
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit lease expiry", source))?;
        Ok(expired.len())
    }
}
