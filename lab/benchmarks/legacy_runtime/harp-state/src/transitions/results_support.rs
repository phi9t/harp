use super::*;

pub(super) fn register_artifact_metadata(
    transaction: &Transaction<'_>,
    artifact: &ArtifactRef,
    size_bytes: i64,
    metadata_json: &[u8],
    now: i64,
) -> StateResult<()> {
    if transaction
        .query_row(
            "SELECT media_type, size_bytes, metadata_json
             FROM artifacts WHERE sha256 = ?1",
            [artifact.sha256.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            },
        )
        .optional()
        .map_err(|source| StateError::sqlite("query artifact metadata", source))?
        .is_some_and(|(media_type, stored_size, stored_metadata)| {
            media_type != artifact.media_type
                || stored_size != size_bytes
                || stored_metadata != metadata_json
        })
    {
        return Err(StateError::integrity(
            "artifact digest is already registered with conflicting metadata",
        ));
    }
    transaction
        .execute(
            "INSERT OR IGNORE INTO artifacts (
                sha256, media_type, size_bytes, metadata_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                artifact.sha256,
                artifact.media_type,
                size_bytes,
                metadata_json,
                now
            ],
        )
        .map_err(|source| StateError::sqlite("register artifact metadata", source))?;
    validate_existing_artifact(transaction, artifact)
}

pub(super) fn validate_result_artifacts_registered(
    connection: &Connection,
    result: &ResultEnvelope,
    result_ref: &ArtifactRef,
) -> StateResult<()> {
    validate_existing_artifact(connection, result_ref)?;
    if let Some(answer) = &result.answer_ref {
        validate_existing_artifact(connection, answer)?;
    }
    for evidence in &result.evidence {
        validate_existing_artifact(connection, evidence)?;
    }
    validate_existing_artifact(connection, &result.trace_ref)
}

pub(super) fn ensure_acceptance_operations_terminal(
    transaction: &Transaction<'_>,
    attempt_id: &AttemptId,
) -> StateResult<()> {
    let mut statement = transaction
        .prepare(
            "SELECT operation_id, kind, ordinal, state
             FROM operations
             WHERE attempt_id = ?1
             ORDER BY kind, ordinal, operation_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare acceptance operations", source))?;
    let rows = statement
        .query_map([attempt_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|source| StateError::sqlite("query acceptance operations", source))?;
    let mut count = 0usize;
    let mut latest_evaluate: Option<(u32, OperationId, OperationState)> = None;
    for row in rows {
        let (operation_id, kind, ordinal, state) =
            row.map_err(|source| StateError::sqlite("read acceptance operation", source))?;
        let operation_id = parse_id("acceptance operation id", operation_id)?;
        let kind = OperationKind::parse(&kind)?;
        let ordinal = checked_i64_to_u32("acceptance operation ordinal", ordinal)?;
        let state = OperationState::parse(&state)?;
        if matches!(
            state,
            OperationState::Prepared | OperationState::Dispatching
        ) {
            return Err(StateError::Conflict {
                entity: format!("attempt {attempt_id} operation barrier"),
                expected: "all operations terminal".to_owned(),
                actual: format!("{} {} is {}", kind.as_str(), operation_id, state.as_str()),
            });
        }
        if kind == OperationKind::Evaluate
            && latest_evaluate
                .as_ref()
                .is_none_or(|(latest, _, _)| ordinal > *latest)
        {
            latest_evaluate = Some((ordinal, operation_id, state));
        }
        count += 1;
        enforce_row_limit("acceptance operations", count)?;
    }
    if let Some((ordinal, operation_id, state)) = latest_evaluate {
        if state != OperationState::Completed {
            return Err(StateError::Conflict {
                entity: format!("attempt {attempt_id} evaluation"),
                expected: "latest evaluation completed".to_owned(),
                actual: format!(
                    "ordinal {ordinal} operation {operation_id} is {}",
                    state.as_str()
                ),
            });
        }
    }
    Ok(())
}

pub(super) fn validate_existing_artifact(
    connection: &Connection,
    artifact: &ArtifactRef,
) -> StateResult<()> {
    let existing: Option<(String, i64, Vec<u8>)> = connection
        .query_row(
            "SELECT media_type, size_bytes, metadata_json
             FROM artifacts WHERE sha256 = ?1",
            [artifact.sha256.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|source| StateError::sqlite("validate artifact metadata", source))?;
    let Some((media_type, size_bytes, metadata_json)) = existing else {
        return Err(StateError::integrity(
            "published artifact metadata is missing",
        ));
    };
    let expected_size = checked_u64_to_i64("artifact size", artifact.size_bytes)?;
    let expected_metadata = bounded_json("artifact metadata", artifact, MAX_JSON_BYTES)?;
    if media_type != artifact.media_type
        || size_bytes != expected_size
        || metadata_json != expected_metadata
    {
        return Err(StateError::integrity(
            "artifact digest is registered with conflicting metadata",
        ));
    }
    Ok(())
}

pub(super) fn aggregate_run_usage(
    transaction: &Transaction<'_>,
    run_id: &RunId,
) -> StateResult<(u128, u128)> {
    let mut statement = transaction
        .prepare(
            "SELECT b.observed_tokens, b.observed_storage_bytes
             FROM budget_reservations b
             JOIN attempts a ON a.attempt_id = b.attempt_id
             WHERE a.run_id = ?1
             ORDER BY a.attempt_id
             LIMIT 100001",
        )
        .map_err(|source| StateError::sqlite("prepare aggregate run usage", source))?;
    let rows = statement
        .query_map([run_id.to_string()], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|source| StateError::sqlite("query aggregate run usage", source))?;
    let mut observed_tokens = 0u128;
    let mut observed_storage = 0u128;
    let mut count = 0usize;
    for row in rows {
        let (tokens, storage) =
            row.map_err(|source| StateError::sqlite("read aggregate run usage", source))?;
        observed_tokens = observed_tokens
            .checked_add(u128::from(checked_i64_to_u64(
                "reservation observed tokens",
                tokens,
            )?))
            .ok_or_else(|| StateError::LimitExceeded {
                context: "aggregate observed tokens".to_owned(),
                limit: u64::MAX,
                actual: u64::MAX,
            })?;
        observed_storage = observed_storage
            .checked_add(u128::from(checked_i64_to_u64(
                "reservation observed storage",
                storage,
            )?))
            .ok_or_else(|| StateError::LimitExceeded {
                context: "aggregate observed storage".to_owned(),
                limit: u64::MAX,
                actual: u64::MAX,
            })?;
        count += 1;
        enforce_row_limit("aggregate run usage", count)?;
    }
    Ok((observed_tokens, observed_storage))
}
