use super::*;

impl StateStore {
    pub fn rebuild_ready_tasks(&mut self, run_id: &RunId, now: i64) -> StateResult<usize> {
        let transaction = self.immediate("begin readiness rebuild")?;
        let activated = activate_ready_dependents(&transaction, run_id, now)?;
        insert_event(
            &transaction,
            Some(run_id),
            None,
            None,
            "ready_tasks_rebuilt",
            &json!({"activated": activated}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit readiness rebuild", source))?;
        Ok(activated)
    }

    pub fn nonterminal_attempts(&mut self, run_id: &RunId) -> StateResult<Vec<AttemptRecord>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT attempt_id
                 FROM attempts
                 WHERE run_id = ?1
                   AND state NOT IN ('succeeded', 'failed', 'indeterminate', 'cancelled')
                 ORDER BY task_id, ordinal, attempt_id
                 LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare nonterminal attempts", source))?;
        let rows = statement
            .query_map([run_id.to_string()], |row| row.get::<_, String>(0))
            .map_err(|source| StateError::sqlite("query nonterminal attempts", source))?;
        let mut attempt_ids = Vec::new();
        for row in rows {
            attempt_ids.push(parse_id(
                "nonterminal attempt id",
                row.map_err(|source| StateError::sqlite("read nonterminal attempt id", source))?,
            )?);
            enforce_row_limit("nonterminal attempts", attempt_ids.len())?;
        }
        drop(statement);
        attempt_ids
            .into_iter()
            .map(|attempt_id| {
                load_attempt(&self.connection, &attempt_id)?.ok_or_else(|| {
                    StateError::integrity("nonterminal attempt disappeared during query")
                })
            })
            .collect()
    }

    pub fn events_page(
        &mut self,
        run_id: &RunId,
        after_sequence: Option<i64>,
        limit: usize,
    ) -> StateResult<EventPage> {
        if !(1..=MAX_EVENT_PAGE_LIMIT).contains(&limit) {
            return Err(StateError::invalid(
                "event page limit must be between 1 and 1000",
            ));
        }
        if after_sequence.is_some_and(|sequence| sequence < 0) {
            return Err(StateError::invalid(
                "event page after_sequence must be nonnegative",
            ));
        }
        let query_limit = i64::try_from(limit + 1).expect("event page limit fits i64");
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence, run_id, task_id, attempt_id,
                        event_type, payload_json, timestamp
                 FROM events
                 WHERE run_id = ?1
                   AND sequence > ?2
                 ORDER BY sequence
                 LIMIT ?3",
            )
            .map_err(|source| StateError::sqlite("prepare run events", source))?;
        let rows = statement
            .query_map(
                params![run_id.to_string(), after_sequence.unwrap_or(0), query_limit],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Vec<u8>>(5)?,
                        row.get::<_, i64>(6)?,
                    ))
                },
            )
            .map_err(|source| StateError::sqlite("query run events", source))?;
        let mut events = Vec::new();
        let mut payload_bytes = 0usize;
        let mut has_more = false;
        for row in rows {
            let (sequence, run_id, task_id, attempt_id, event_type, payload, timestamp) =
                row.map_err(|source| StateError::sqlite("read run event", source))?;
            if events.len() == limit {
                has_more = true;
                break;
            }
            payload_bytes = payload_bytes.checked_add(payload.len()).ok_or_else(|| {
                StateError::LimitExceeded {
                    context: "event page payload bytes".to_owned(),
                    limit: MAX_EVENT_PAGE_PAYLOAD_BYTES as u64,
                    actual: u64::MAX,
                }
            })?;
            if payload_bytes > MAX_EVENT_PAGE_PAYLOAD_BYTES {
                return Err(StateError::LimitExceeded {
                    context: "event page payload bytes".to_owned(),
                    limit: MAX_EVENT_PAGE_PAYLOAD_BYTES as u64,
                    actual: payload_bytes as u64,
                });
            }
            events.push(EventRecord {
                sequence: checked_i64_to_u64("event sequence", sequence)?,
                run_id: run_id
                    .map(|value| parse_id("event run id", value))
                    .transpose()?,
                task_id: task_id
                    .map(|value| parse_id("event task id", value))
                    .transpose()?,
                attempt_id: attempt_id
                    .map(|value| parse_id("event attempt id", value))
                    .transpose()?,
                event_type,
                payload: parse_bounded_json("event payload", &payload)?,
                timestamp,
            });
        }
        let next_after_sequence = if has_more {
            events
                .last()
                .map(|event| i64::try_from(event.sequence))
                .transpose()
                .map_err(|_| StateError::integrity("event sequence exceeds i64"))?
        } else {
            None
        };
        Ok(EventPage {
            events,
            next_after_sequence,
        })
    }

    pub fn get_task(
        &mut self,
        run_id: &RunId,
        task_id: &TaskId,
    ) -> StateResult<Option<TaskRecord>> {
        load_task(&self.connection, run_id, task_id)
    }

    pub fn reservation_usage(&mut self, attempt_id: &AttemptId) -> StateResult<Option<(u64, u64)>> {
        self.connection
            .query_row(
                "SELECT observed_tokens, observed_storage_bytes
                 FROM budget_reservations WHERE attempt_id = ?1",
                [attempt_id.to_string()],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(|source| StateError::sqlite("query reservation usage", source))?
            .map(|(tokens, storage)| {
                Ok((
                    checked_i64_to_u64("reservation observed tokens", tokens)?,
                    checked_i64_to_u64("reservation observed storage", storage)?,
                ))
            })
            .transpose()
    }

    pub fn get_attempt(&mut self, attempt_id: &AttemptId) -> StateResult<Option<AttemptRecord>> {
        load_attempt(&self.connection, attempt_id)
    }

    pub fn get_cli_attempt(
        &mut self,
        attempt_id: &AttemptId,
    ) -> StateResult<Option<CliAttemptRecord>> {
        load_cli_attempt(&self.connection, attempt_id)
    }

    pub fn get_cli_activity(
        &mut self,
        activity_id: &OperationId,
    ) -> StateResult<Option<CliActivityRecord>> {
        load_cli_activity(&self.connection, activity_id)
    }

    pub fn recoverable_activities(
        &mut self,
        run_id: &RunId,
    ) -> StateResult<Vec<ActivityRecoveryRecord>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT ca.activity_id
                 FROM cli_activities ca
                 JOIN attempts a ON a.attempt_id = ca.attempt_id
                 JOIN cli_attempts cat ON cat.attempt_id = ca.attempt_id
                 WHERE a.run_id = ?1
                   AND cat.state IN ('prepared', 'running', 'reconciling')
                   AND ca.state IN ('prepared', 'dispatching', 'running', 'reconciling', 'completed')
                 ORDER BY a.task_id, a.ordinal, ca.kind, ca.ordinal, ca.activity_id
                 LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare recoverable CLI activities", source))?;
        let rows = statement
            .query_map([run_id.to_string()], |row| row.get::<_, String>(0))
            .map_err(|source| StateError::sqlite("query recoverable CLI activities", source))?;
        let mut activity_ids = Vec::new();
        for row in rows {
            activity_ids.push(parse_id(
                "recoverable CLI activity id",
                row.map_err(|source| {
                    StateError::sqlite("read recoverable CLI activity id", source)
                })?,
            )?);
            enforce_row_limit("recoverable CLI activities", activity_ids.len())?;
        }
        drop(statement);

        activity_ids
            .into_iter()
            .map(|activity_id| {
                let activity =
                    load_cli_activity(&self.connection, &activity_id)?.ok_or_else(|| {
                        StateError::integrity(
                            "recoverable CLI activity disappeared during recovery query",
                        )
                    })?;
                let cli_attempt = load_cli_attempt(&self.connection, &activity.attempt_id)?
                    .ok_or_else(|| {
                        StateError::integrity(
                            "recoverable CLI activity has no CLI attempt extension",
                        )
                    })?;
                let attempt =
                    load_attempt(&self.connection, &activity.attempt_id)?.ok_or_else(|| {
                        StateError::integrity("recoverable CLI activity has no legacy attempt")
                    })?;
                if attempt.run_id != *run_id {
                    return Err(StateError::integrity(
                        "recoverable CLI activity belongs to a different run",
                    ));
                }
                Ok(ActivityRecoveryRecord {
                    run_id: attempt.run_id,
                    task_id: attempt.task_id,
                    activity_id: activity.activity_id,
                    attempt_id: activity.attempt_id,
                    kind: activity.kind,
                    ordinal: activity.ordinal,
                    activity_state: activity.state,
                    attempt_state: cli_attempt.state,
                    logical_session_id: cli_attempt.logical_session_id,
                    logical_turn_id: activity.logical_turn_id,
                    external_session_id: cli_attempt.external_session_id,
                    continuation_count: cli_attempt.continuation_count,
                    activity_dir: activity.activity_dir,
                    invocation_sha256: activity.invocation_sha256,
                    process_record_sha256: activity.process_record_sha256,
                    terminal_failure_class: activity.terminal_failure_class,
                    interrupt_purpose: activity.interrupt_purpose,
                    target_process_record_sha256: activity.target_process_record_sha256,
                    signal_stage: activity.signal_stage,
                })
            })
            .collect()
    }

    pub fn get_operation(
        &mut self,
        operation_id: &OperationId,
    ) -> StateResult<Option<OperationRecord>> {
        load_operation(&self.connection, operation_id)
    }

    pub fn get_operation_by_marker(
        &mut self,
        operation_marker: &str,
    ) -> StateResult<Option<OperationRecord>> {
        validate_label("operation marker", operation_marker)?;
        let operation_id: Option<String> = self
            .connection
            .query_row(
                "SELECT operation_id
                 FROM operations
                 WHERE operation_marker = ?1",
                [operation_marker],
                |row| row.get(0),
            )
            .optional()
            .map_err(|source| StateError::sqlite("query operation marker", source))?;
        operation_id
            .map(|value| {
                let operation_id = parse_id("operation marker id", value)?;
                load_operation(&self.connection, &operation_id)?.ok_or_else(|| {
                    StateError::integrity("operation marker points to missing operation")
                })
            })
            .transpose()
    }
}
