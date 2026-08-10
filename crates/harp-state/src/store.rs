use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use harp_contracts::{ArtifactRef, AttemptId, RunId, TaskGraph, TaskId, TaskNode};
use rusqlite::{params, OptionalExtension, Row, Transaction, TransactionBehavior};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::model::{
    checked_i64_to_u64, checked_u64_to_i64, node_kind_as_str, parse_id, parse_node_kind,
    AcceptedResultRecord, AttemptRecord, RunBudget, RunExecutionRecord, RunRecord, RunState,
    TaskRecord, TaskState,
};
use crate::{StateError, StateResult};

const MAX_GRAPH_BYTES: usize = 1024 * 1024;
const MAX_JSON_BYTES: usize = 256 * 1024;
const MAX_RECOVERY_ROWS: usize = 100_000;

#[derive(Debug)]
pub struct StateStore {
    pub(crate) connection: rusqlite::Connection,
    path: PathBuf,
    pub(crate) lease_clock: Arc<dyn crate::LeaseClock>,
}

impl StateStore {
    #[cfg(all(debug_assertions, unix))]
    #[doc(hidden)]
    pub fn test_only_forge_lease_from_visible(
        &self,
        attempt_id: harp_contracts::AttemptId,
        owner: &str,
        expires_at: i64,
    ) -> crate::LeaseToken {
        crate::LeaseToken::new(
            attempt_id,
            owner.to_owned(),
            expires_at,
            uuid::Uuid::new_v4().hyphenated().to_string(),
        )
    }

    #[cfg(all(debug_assertions, unix))]
    #[doc(hidden)]
    pub fn test_only_lease_with_capability(
        &self,
        attempt_id: harp_contracts::AttemptId,
        owner: &str,
        expires_at: i64,
        capability: &str,
    ) -> crate::LeaseToken {
        crate::LeaseToken::new(
            attempt_id,
            owner.to_owned(),
            expires_at,
            capability.to_owned(),
        )
    }

    pub fn open(path: &Path) -> StateResult<Self> {
        #[cfg(not(unix))]
        {
            let _ = path;
            return Err(StateError::Unsupported {
                context: "secure SQLite path opening is unavailable on non-Unix platforms"
                    .to_owned(),
            });
        }
        #[cfg(unix)]
        {
            let connection = crate::sqlite::open(path)?;
            Ok(Self {
                connection,
                path: path.to_path_buf(),
                lease_clock: crate::default_lease_clock(),
            })
        }
    }

    pub fn open_with_lease_clock(
        path: &Path,
        lease_clock: Arc<dyn crate::LeaseClock>,
    ) -> StateResult<Self> {
        #[cfg(not(unix))]
        {
            let _ = (path, lease_clock);
            return Err(StateError::Unsupported {
                context: "secure SQLite path opening is unavailable on non-Unix platforms"
                    .to_owned(),
            });
        }
        #[cfg(unix)]
        {
            lease_clock.unix_seconds()?;
            let connection = crate::sqlite::open(path)?;
            Ok(Self {
                connection,
                path: path.to_path_buf(),
                lease_clock,
            })
        }
    }

    pub(crate) fn lease_now(&self, _event_now: i64) -> StateResult<i64> {
        self.lease_clock.unix_seconds()
    }

    pub fn set_lease_clock(&mut self, lease_clock: Arc<dyn crate::LeaseClock>) -> StateResult<()> {
        lease_clock.unix_seconds()?;
        self.lease_clock = lease_clock;
        Ok(())
    }

    #[cfg(all(debug_assertions, unix))]
    #[doc(hidden)]
    pub fn open_with_test_hook<F>(path: &Path, hook: F) -> StateResult<Self>
    where
        F: FnOnce(),
    {
        let connection = crate::sqlite::open_with_hook(path, || {
            hook();
            Ok(())
        })?;
        Ok(Self {
            connection,
            path: path.to_path_buf(),
            lease_clock: crate::default_lease_clock(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn schema_version(&self) -> StateResult<i64> {
        self.connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|source| StateError::sqlite("read schema version", source))
    }

    pub fn foreign_keys_enabled(&self) -> StateResult<bool> {
        self.connection
            .pragma_query_value(None, "foreign_keys", |row| row.get::<_, i64>(0))
            .map(|value| value == 1)
            .map_err(|source| StateError::sqlite("read foreign key mode", source))
    }

    pub fn journal_mode(&self) -> StateResult<String> {
        self.connection
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .map_err(|source| StateError::sqlite("read journal mode", source))
    }

    pub fn synchronous_mode(&self) -> StateResult<i64> {
        self.connection
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .map_err(|source| StateError::sqlite("read synchronous mode", source))
    }

    pub fn busy_timeout_millis(&self) -> StateResult<i64> {
        self.connection
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .map_err(|source| StateError::sqlite("read busy timeout", source))
    }

    pub fn create_run(
        &mut self,
        graph: &TaskGraph,
        provenance: &Value,
        budget: &RunBudget,
        now: i64,
    ) -> StateResult<RunRecord> {
        graph
            .validate_shape()
            .map_err(StateError::invalid_contract)?;
        validate_graph_references(graph)?;
        budget.validate()?;
        if !provenance.is_object() {
            return Err(StateError::invalid("run provenance must be a JSON object"));
        }
        let graph_json = bounded_json("task graph", graph, MAX_GRAPH_BYTES)?;
        let provenance_json = bounded_json("run provenance", provenance, MAX_JSON_BYTES)?;
        let graph_sha256 = format!("{:x}", Sha256::digest(&graph_json));
        let run_id = RunId::new();
        let run = RunRecord {
            run_id: run_id.clone(),
            graph_sha256: graph_sha256.clone(),
            state: RunState::Active,
            cancellation_requested: false,
            created_at: now,
            updated_at: now,
        };
        let max_tokens = checked_u64_to_i64("run max tokens", budget.max_tokens)?;
        let max_storage = checked_u64_to_i64("run max storage bytes", budget.max_storage_bytes)?;
        let max_wall = checked_u64_to_i64("run max wall seconds", budget.max_wall_seconds)?;

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|source| StateError::sqlite("begin create run", source))?;
        transaction
            .execute(
                "INSERT INTO runs (
                    run_id, graph_sha256, graph_json, provenance_json,
                    max_tokens, max_storage_bytes, max_wall_seconds,
                    state, cancellation_requested, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active', 0, ?8, ?8)",
                params![
                    run_id.to_string(),
                    graph_sha256,
                    graph_json,
                    provenance_json,
                    max_tokens,
                    max_storage,
                    max_wall,
                    now
                ],
            )
            .map_err(|source| StateError::sqlite("insert run", source))?;
        for node in &graph.nodes {
            let state = if node.dependencies.is_empty() {
                TaskState::Ready
            } else {
                TaskState::Pending
            };
            let task_json = bounded_json("task node", node, MAX_JSON_BYTES)?;
            transaction
                .execute(
                    "INSERT INTO tasks (
                        run_id, task_id, kind, state, accepted_attempt_id,
                        max_transient_attempts, task_json, created_at, updated_at
                     ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?7)",
                    params![
                        run_id.to_string(),
                        node.task_id.to_string(),
                        node_kind_as_str(node.kind),
                        state.as_str(),
                        i64::from(node.retry_policy.max_transient_attempts),
                        task_json,
                        now
                    ],
                )
                .map_err(|source| StateError::sqlite("insert task", source))?;
        }
        for node in &graph.nodes {
            for dependency in &node.dependencies {
                transaction
                    .execute(
                        "INSERT INTO task_dependencies (
                            run_id, task_id, depends_on_task_id
                         ) VALUES (?1, ?2, ?3)",
                        params![
                            run_id.to_string(),
                            node.task_id.to_string(),
                            dependency.to_string()
                        ],
                    )
                    .map_err(|source| StateError::sqlite("insert task dependency", source))?;
            }
        }
        insert_event(
            &transaction,
            Some(&run_id),
            None,
            None,
            "run_created",
            &json!({"graphSha256": run.graph_sha256}),
            now,
        )?;
        transaction
            .commit()
            .map_err(|source| StateError::sqlite("commit create run", source))?;
        Ok(run)
    }

    pub fn ready_tasks(&mut self, run_id: &RunId) -> StateResult<Vec<TaskRecord>> {
        self.tasks_with_state(run_id, TaskState::Ready)
    }

    fn tasks_with_state(
        &mut self,
        run_id: &RunId,
        state: TaskState,
    ) -> StateResult<Vec<TaskRecord>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT run_id, task_id, kind, state, accepted_attempt_id
                 FROM tasks
                 WHERE run_id = ?1 AND state = ?2
                 ORDER BY task_id
                 LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare task query", source))?;
        let rows = statement
            .query_map(params![run_id.to_string(), state.as_str()], row_to_task)
            .map_err(|source| StateError::sqlite("query tasks", source))?;
        collect_rows(rows, "task query")
    }

    pub fn get_run(&mut self, run_id: &RunId) -> StateResult<Option<RunRecord>> {
        self.connection
            .query_row(
                "SELECT run_id, graph_sha256, state, cancellation_requested, created_at, updated_at
                 FROM runs WHERE run_id = ?1",
                [run_id.to_string()],
                row_to_run,
            )
            .optional()
            .map_err(|source| StateError::sqlite("query run", source))?
            .transpose()
    }

    pub fn incomplete_runs(&mut self) -> StateResult<Vec<RunRecord>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT run_id, graph_sha256, state, cancellation_requested, created_at, updated_at
                 FROM runs WHERE state = 'active'
                 ORDER BY created_at, run_id
                 LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare incomplete runs", source))?;
        let rows = statement
            .query_map([], row_to_run)
            .map_err(|source| StateError::sqlite("query incomplete runs", source))?;
        collect_rows(rows, "incomplete runs")
    }

    pub fn run_execution(&mut self, run_id: &RunId) -> StateResult<Option<RunExecutionRecord>> {
        let raw = self
            .connection
            .query_row(
                "SELECT run_id, graph_sha256, graph_json, provenance_json,
                        max_tokens, max_storage_bytes, max_wall_seconds,
                        state, cancellation_requested, created_at, updated_at
                 FROM runs WHERE run_id = ?1",
                [run_id.to_string()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                        row.get::<_, i64>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, i64>(6)?,
                        row.get::<_, String>(7)?,
                        row.get::<_, i64>(8)?,
                        row.get::<_, i64>(9)?,
                        row.get::<_, i64>(10)?,
                    ))
                },
            )
            .optional()
            .map_err(|source| StateError::sqlite("query run execution", source))?;
        raw.map(
            |(
                stored_run_id,
                graph_sha256,
                graph_json,
                provenance_json,
                max_tokens,
                max_storage_bytes,
                max_wall_seconds,
                state,
                cancellation_requested,
                created_at,
                updated_at,
            )| {
                let stored_run_id = parse_id("run execution id", stored_run_id)?;
                if stored_run_id != *run_id {
                    return Err(StateError::integrity(
                        "run execution row identity does not match query",
                    ));
                }
                if graph_json.len() > MAX_GRAPH_BYTES || provenance_json.len() > MAX_JSON_BYTES {
                    return Err(StateError::integrity(
                        "persisted run JSON exceeds schema bounds",
                    ));
                }
                let verified_graph_sha256 = format!("{:x}", Sha256::digest(&graph_json));
                if verified_graph_sha256 != graph_sha256 {
                    return Err(StateError::integrity(
                        "persisted task graph digest does not match graph bytes",
                    ));
                }
                let graph: TaskGraph = serde_json::from_slice(&graph_json).map_err(|source| {
                    StateError::Serialization {
                        context: "persisted task graph".to_owned(),
                        source,
                    }
                })?;
                graph.validate_shape().map_err(|source| {
                    StateError::integrity_contract(
                        "persisted task graph failed contract validation",
                        source,
                    )
                })?;
                validate_graph_references(&graph)?;
                let provenance: Value =
                    serde_json::from_slice(&provenance_json).map_err(|source| {
                        StateError::Serialization {
                            context: "persisted run provenance".to_owned(),
                            source,
                        }
                    })?;
                if !provenance.is_object() {
                    return Err(StateError::integrity(
                        "persisted run provenance is not an object",
                    ));
                }
                if cancellation_requested != 0 && cancellation_requested != 1 {
                    return Err(StateError::integrity(
                        "run cancellation flag is not boolean",
                    ));
                }
                let budget = RunBudget::new(
                    checked_i64_to_u64("run max tokens", max_tokens)?,
                    checked_i64_to_u64("run max storage bytes", max_storage_bytes)?,
                    checked_i64_to_u64("run max wall seconds", max_wall_seconds)?,
                )?;
                Ok(RunExecutionRecord {
                    run: RunRecord {
                        run_id: stored_run_id,
                        graph_sha256: graph_sha256.clone(),
                        state: RunState::parse(&state)?,
                        cancellation_requested: cancellation_requested == 1,
                        created_at,
                        updated_at,
                    },
                    graph,
                    verified_graph_sha256,
                    provenance,
                    budget,
                })
            },
        )
        .transpose()
    }

    pub fn task_node(&mut self, run_id: &RunId, task_id: &TaskId) -> StateResult<Option<TaskNode>> {
        let raw = self
            .connection
            .query_row(
                "SELECT task_json, kind
                 FROM tasks WHERE run_id = ?1 AND task_id = ?2",
                params![run_id.to_string(), task_id.to_string()],
                |row| Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|source| StateError::sqlite("query task node", source))?;
        raw.map(|(bytes, kind)| {
            if bytes.len() > MAX_JSON_BYTES {
                return Err(StateError::integrity(
                    "persisted task node exceeds schema bounds",
                ));
            }
            let node: TaskNode =
                serde_json::from_slice(&bytes).map_err(|source| StateError::Serialization {
                    context: "persisted task node".to_owned(),
                    source,
                })?;
            let singleton = TaskGraph {
                schema_version: 1,
                nodes: vec![node.clone()],
            };
            singleton.validate_shape().map_err(|source| {
                StateError::integrity_contract(
                    "persisted task node failed contract validation",
                    source,
                )
            })?;
            if node.task_id != *task_id {
                return Err(StateError::integrity(
                    "persisted task JSON identity does not match task row",
                ));
            }
            if node.kind != parse_node_kind(&kind)? {
                return Err(StateError::integrity(
                    "persisted task JSON kind does not match task row",
                ));
            }
            Ok(node)
        })
        .transpose()
    }

    pub fn tasks(&mut self, run_id: &RunId) -> StateResult<Vec<TaskRecord>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT run_id, task_id, kind, state, accepted_attempt_id
                 FROM tasks WHERE run_id = ?1
                 ORDER BY task_id LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare run tasks", source))?;
        let rows = statement
            .query_map([run_id.to_string()], row_to_task)
            .map_err(|source| StateError::sqlite("query run tasks", source))?;
        collect_rows(rows, "run tasks")
    }

    pub fn attempts(&mut self, run_id: &RunId) -> StateResult<Vec<AttemptRecord>> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT attempt_id FROM attempts WHERE run_id = ?1
                 ORDER BY task_id, ordinal, attempt_id LIMIT 100001",
            )
            .map_err(|source| StateError::sqlite("prepare run attempts", source))?;
        let rows = statement
            .query_map([run_id.to_string()], |row| row.get::<_, String>(0))
            .map_err(|source| StateError::sqlite("query run attempts", source))?;
        let mut attempt_ids = Vec::new();
        for row in rows {
            attempt_ids.push(parse_id(
                "run attempt id",
                row.map_err(|source| StateError::sqlite("read run attempt id", source))?,
            )?);
            enforce_row_limit("run attempts", attempt_ids.len())?;
        }
        drop(statement);
        attempt_ids
            .into_iter()
            .map(|attempt_id| {
                crate::transitions::load_attempt_record(&self.connection, &attempt_id)?
                    .ok_or_else(|| StateError::integrity("run attempt disappeared during query"))
            })
            .collect()
    }

    pub fn operations_for_attempt(
        &mut self,
        attempt_id: &AttemptId,
    ) -> StateResult<Vec<crate::OperationRecord>> {
        crate::transitions::load_operations_for_attempt(&self.connection, attempt_id)
    }

    pub fn accepted_results(&mut self, run_id: &RunId) -> StateResult<Vec<AcceptedResultRecord>> {
        let tasks = self.tasks(run_id)?;
        let mut accepted = Vec::new();
        for task in tasks {
            let Some(attempt_id) = task.accepted_attempt_id.clone() else {
                continue;
            };
            let attempt = crate::transitions::load_attempt_record(&self.connection, &attempt_id)?
                .ok_or_else(|| StateError::integrity("accepted attempt is missing"))?;
            if attempt.run_id != *run_id
                || attempt.task_id != task.task_id
                || attempt.state != crate::AttemptState::Succeeded
                || attempt.result_sha256.is_none()
            {
                return Err(StateError::integrity(
                    "accepted result lineage is inconsistent",
                ));
            }
            accepted.push(AcceptedResultRecord { task, attempt });
            enforce_row_limit("accepted results", accepted.len())?;
        }
        Ok(accepted)
    }

    pub fn artifact(&mut self, sha256: &str) -> StateResult<Option<ArtifactRef>> {
        let bytes = self
            .connection
            .query_row(
                "SELECT metadata_json FROM artifacts WHERE sha256 = ?1",
                [sha256],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()
            .map_err(|source| StateError::sqlite("query artifact metadata", source))?;
        bytes
            .map(|bytes| {
                if bytes.len() > MAX_JSON_BYTES {
                    return Err(StateError::integrity(
                        "persisted artifact metadata exceeds schema bounds",
                    ));
                }
                let artifact: ArtifactRef =
                    serde_json::from_slice(&bytes).map_err(|source| StateError::Serialization {
                        context: "persisted artifact metadata".to_owned(),
                        source,
                    })?;
                artifact.validate().map_err(|source| {
                    StateError::integrity_contract(
                        "persisted artifact metadata failed contract validation",
                        source,
                    )
                })?;
                if artifact.sha256 != sha256 {
                    return Err(StateError::integrity(
                        "persisted artifact metadata identity does not match artifact row",
                    ));
                }
                Ok(artifact)
            })
            .transpose()
    }
}

fn enforce_row_limit(context: &str, count: usize) -> StateResult<()> {
    if count > MAX_RECOVERY_ROWS {
        return Err(StateError::LimitExceeded {
            context: context.to_owned(),
            limit: MAX_RECOVERY_ROWS as u64,
            actual: count as u64,
        });
    }
    Ok(())
}

fn validate_graph_references(graph: &TaskGraph) -> StateResult<()> {
    let mut task_ids = HashSet::with_capacity(graph.nodes.len());
    for node in &graph.nodes {
        if !task_ids.insert(node.task_id.to_string()) {
            return Err(StateError::invalid(format!(
                "duplicate task id {}",
                node.task_id
            )));
        }
    }
    for node in &graph.nodes {
        let mut dependencies = HashSet::with_capacity(node.dependencies.len());
        for dependency in &node.dependencies {
            if dependency == &node.task_id {
                return Err(StateError::invalid(format!(
                    "task {} cannot depend on itself",
                    node.task_id
                )));
            }
            if !task_ids.contains(&dependency.to_string()) {
                return Err(StateError::invalid(format!(
                    "task {} depends on unknown task {}",
                    node.task_id, dependency
                )));
            }
            if !dependencies.insert(dependency.to_string()) {
                return Err(StateError::invalid(format!(
                    "task {} repeats dependency {}",
                    node.task_id, dependency
                )));
            }
        }
    }
    Ok(())
}

pub(crate) fn bounded_json<T: serde::Serialize>(
    context: &str,
    value: &T,
    max_bytes: usize,
) -> StateResult<Vec<u8>> {
    let bytes = serde_json::to_vec(value).map_err(|source| StateError::Serialization {
        context: context.to_owned(),
        source,
    })?;
    if bytes.len() > max_bytes {
        return Err(StateError::LimitExceeded {
            context: context.to_owned(),
            limit: max_bytes as u64,
            actual: bytes.len() as u64,
        });
    }
    Ok(bytes)
}

pub(crate) fn insert_event(
    transaction: &Transaction<'_>,
    run_id: Option<&RunId>,
    task_id: Option<&harp_contracts::TaskId>,
    attempt_id: Option<&harp_contracts::AttemptId>,
    event_type: &str,
    payload: &Value,
    now: i64,
) -> StateResult<()> {
    if event_type.is_empty() || event_type.len() > 128 || event_type.chars().any(char::is_control) {
        return Err(StateError::invalid("invalid event type"));
    }
    let payload = bounded_json("event payload", payload, MAX_JSON_BYTES)?;
    transaction
        .execute(
            "INSERT INTO events (
                run_id, task_id, attempt_id, event_type, payload_json, timestamp
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                run_id.map(ToString::to_string),
                task_id.map(ToString::to_string),
                attempt_id.map(ToString::to_string),
                event_type,
                payload,
                now
            ],
        )
        .map_err(|source| StateError::sqlite("insert event", source))?;
    Ok(())
}

fn row_to_run(row: &Row<'_>) -> rusqlite::Result<StateResult<RunRecord>> {
    let run_id: String = row.get(0)?;
    let graph_sha256: String = row.get(1)?;
    let state: String = row.get(2)?;
    let cancellation_requested: i64 = row.get(3)?;
    let created_at: i64 = row.get(4)?;
    let updated_at: i64 = row.get(5)?;
    Ok((|| {
        if cancellation_requested != 0 && cancellation_requested != 1 {
            return Err(StateError::integrity(
                "run cancellation flag is not boolean",
            ));
        }
        Ok(RunRecord {
            run_id: parse_id("run id", run_id)?,
            graph_sha256,
            state: RunState::parse(&state)?,
            cancellation_requested: cancellation_requested == 1,
            created_at,
            updated_at,
        })
    })())
}

fn row_to_task(row: &Row<'_>) -> rusqlite::Result<StateResult<TaskRecord>> {
    let run_id: String = row.get(0)?;
    let task_id: String = row.get(1)?;
    let kind: String = row.get(2)?;
    let state: String = row.get(3)?;
    let accepted_attempt_id: Option<String> = row.get(4)?;
    Ok((|| {
        Ok(TaskRecord {
            run_id: parse_id("task run id", run_id)?,
            task_id: parse_id("task id", task_id)?,
            kind: parse_node_kind(&kind)?,
            state: TaskState::parse(&state)?,
            accepted_attempt_id: accepted_attempt_id
                .map(|value| parse_id("accepted attempt id", value))
                .transpose()?,
        })
    })())
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&Row<'_>) -> rusqlite::Result<StateResult<T>>>,
    context: &str,
) -> StateResult<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row.map_err(|source| StateError::sqlite(context, source))??);
        if values.len() > MAX_RECOVERY_ROWS {
            return Err(StateError::LimitExceeded {
                context: context.to_owned(),
                limit: MAX_RECOVERY_ROWS as u64,
                actual: values.len() as u64,
            });
        }
    }
    Ok(values)
}
