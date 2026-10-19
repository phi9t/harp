use harp_contracts::{RunId, TaskId};

use crate::{
    AttemptRecord, CliAttemptRecord, EventPage, OperationRecord, RunExecutionRecord, StateError,
    StateResult, StateStore, TaskRecord,
};

#[derive(Debug)]
pub struct AttemptInspection {
    pub attempt: AttemptRecord,
    pub cli: Option<CliAttemptRecord>,
    pub operations: Vec<OperationRecord>,
}

#[derive(Debug)]
pub struct RunInspection {
    pub execution: RunExecutionRecord,
    pub tasks: Vec<TaskRecord>,
    pub attempts: Vec<AttemptInspection>,
    pub events: EventPage,
    pub observed_at: i64,
}

impl StateStore {
    /// Read all explanatory facts in one SQLite snapshot. Never advance recovery.
    pub fn inspect_run(
        &mut self,
        run_id: &RunId,
        task_id: Option<&TaskId>,
        after_sequence: Option<i64>,
        limit: usize,
    ) -> StateResult<Option<RunInspection>> {
        self.connection
            .execute_batch("BEGIN DEFERRED")
            .map_err(|source| StateError::sqlite("begin inspection snapshot", source))?;
        let result = self.read_inspection(run_id, task_id, after_sequence, limit);
        self.connection
            .execute_batch("ROLLBACK")
            .map_err(|source| StateError::sqlite("end inspection snapshot", source))?;
        result
    }

    fn read_inspection(
        &mut self,
        run_id: &RunId,
        task_id: Option<&TaskId>,
        after_sequence: Option<i64>,
        limit: usize,
    ) -> StateResult<Option<RunInspection>> {
        let Some(execution) = self.run_execution(run_id)? else {
            return Ok(None);
        };
        let observed_at = self.lease_clock.unix_seconds()?;
        let tasks = self.tasks(run_id)?;
        if task_id.is_some_and(|id| !tasks.iter().any(|task| &task.task_id == id)) {
            return Err(StateError::invalid(
                "inspection task does not belong to this run",
            ));
        }
        // Validate accepted-result lineage rather than trusting task labels alone.
        self.accepted_results(run_id)?;
        let mut attempts = Vec::new();
        for attempt in self.attempts(run_id)? {
            let cli = self.get_cli_attempt(&attempt.attempt_id)?;
            let operations = self.operations_for_attempt(&attempt.attempt_id)?;
            attempts.push(AttemptInspection {
                attempt,
                cli,
                operations,
            });
        }
        let events = self.events_page_for_task(run_id, task_id, after_sequence, limit)?;
        Ok(Some(RunInspection {
            execution,
            tasks,
            attempts,
            events,
            observed_at,
        }))
    }
}
