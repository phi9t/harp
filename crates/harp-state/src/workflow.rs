//! Durable v2 facts and guarded transitions. Scheduling and backend authority stay in Engine.
use crate::{StateError, StateResult, StateStore};
use harp_contracts::{ArtifactRef, RunId, TaskId, WorkflowV2};
use rusqlite::{params, Connection, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionRequest {
    pub namespace: String,
    pub submission_key: String,
    pub workflow: WorkflowV2,
    pub schemas_sha256: String,
    /// Pins the complete Engine-verified schemas, backend and environment authority receipt.
    pub execution_authority: ArtifactRef,
    pub trial_limit: u64,
    pub trial_bindings: BTreeMap<TaskId, String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionRecord {
    pub schema_version: u8,
    pub run_id: RunId,
    pub request_sha256: String,
    pub request: AdmissionRequest,
    pub revision: u64,
    pub owner_epoch: u64,
    pub cancellation_requested: bool,
    pub created_at: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunGuard {
    pub revision: u64,
    pub epoch: u64,
}
impl AdmissionRecord {
    pub fn guard(&self) -> RunGuard {
        RunGuard {
            revision: self.revision,
            epoch: self.owner_epoch,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceVector {
    pub agent_tokens: u64,
    pub cpu_seconds: u64,
    pub gpu_seconds: u64,
    pub wall_seconds: u64,
    pub storage_bytes: u64,
    pub trials: u64,
}
impl ResourceVector {
    fn values(self) -> [u64; 6] {
        [
            self.agent_tokens,
            self.cpu_seconds,
            self.gpu_seconds,
            self.wall_seconds,
            self.storage_bytes,
            self.trials,
        ]
    }
    fn from_values(v: [u64; 6]) -> Self {
        Self {
            agent_tokens: v[0],
            cpu_seconds: v[1],
            gpu_seconds: v[2],
            wall_seconds: v[3],
            storage_bytes: v[4],
            trials: v[5],
        }
    }
    fn checked_add(self, other: Self) -> StateResult<Self> {
        let mut v = self.values();
        for (a, b) in v.iter_mut().zip(other.values()) {
            *a = a
                .checked_add(b)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or_else(|| {
                    StateError::invalid("resource arithmetic exceeds durable integer range")
                })?;
        }
        Ok(Self::from_values(v))
    }
    fn within(self, ceiling: Self) -> StateResult<()> {
        for ((attempted, limit), dimension) in
            self.values().into_iter().zip(ceiling.values()).zip([
                "agent_tokens",
                "cpu_seconds",
                "gpu_seconds",
                "wall_seconds",
                "storage_bytes",
                "trials",
            ])
        {
            if attempted > limit {
                return Err(StateError::BudgetExceeded {
                    dimension: dimension.into(),
                    limit,
                    attempted,
                });
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowAccounting {
    pub limit: ResourceVector,
    pub reserved: ResourceVector,
    pub settled: ResourceVector,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Prepared,
    SubmissionUnknown,
    Running,
    Terminal,
    Collecting,
    Collected,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobConnectivity {
    Unknown,
    Reachable,
    Unreachable,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobObservation {
    pub connectivity: JobConnectivity,
    pub observed_at: i64,
    pub evidence: Option<ArtifactRef>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum JobTermination {
    Exit { code: i32, receipt: ArtifactRef },
    Signal { signal: u32, receipt: ArtifactRef },
    Lost { receipt: ArtifactRef },
    Absent { receipt: ArtifactRef },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum JobQuiescence {
    ProcessTree { receipt: ArtifactRef },
    AuthoritativeAbsence { receipt: ArtifactRef },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceReservation {
    pub reserved: ResourceVector,
    pub settled: Option<ResourceVector>,
    pub estimated: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobRecord {
    pub schema_version: u8,
    pub run_id: RunId,
    pub job_id: String,
    pub task_id: TaskId,
    pub attempt: u32,
    pub trial_id: Option<String>,
    pub submission_key: String,
    pub state: JobState,
    pub cancel_requested: bool,
    pub backend_job_id: Option<String>,
    /// Immutable full backend binding (request, backend, host/boot, owner and job identity).
    /// Engine validates these artifact bytes before admitting observations.
    pub binding_receipt: Option<ArtifactRef>,
    pub termination: Option<JobTermination>,
    pub quiescence: Option<JobQuiescence>,
    pub collected_manifest: Option<ArtifactRef>,
    pub observation: JobObservation,
    pub reservation: ResourceReservation,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobPreparation {
    pub job_id: String,
    pub task_id: TaskId,
    pub trial_id: Option<String>,
    pub resources: ResourceVector,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum JobTransition {
    Submit,
    Bind {
        backend_job_id: String,
        binding_receipt: ArtifactRef,
    },
    /// Absence must include exclusion of delayed submission, attested by the backend receipt.
    ConfirmAbsent {
        receipt: ArtifactRef,
    },
    Terminate {
        termination: JobTermination,
        quiescence: JobQuiescence,
    },
    BeginCollection,
    FinishCollection {
        manifest: ArtifactRef,
    },
    Cancel,
    /// None charges reserved maximum and records estimated accounting.
    Settle {
        measured: Option<ResourceVector>,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentState {
    Planned,
    Preparing,
    Verified,
    Failed,
    Unknown,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentRecord {
    pub schema_version: u8,
    pub run_id: RunId,
    pub environment_sha256: String,
    pub state: EnvironmentState,
    pub receipt: Option<ArtifactRef>,
    pub updated_at: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobDecision {
    pub decision_id: String,
    pub expected: RunGuard,
    pub job_id: String,
    pub transition: JobTransition,
    pub reason: String,
    pub evidence: Vec<ArtifactRef>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecord {
    pub schema_version: u8,
    pub run_id: RunId,
    pub decision: JobDecision,
    pub resulting_guard: RunGuard,
    pub applied_at: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowEvent {
    pub sequence: u64,
    pub run_id: RunId,
    pub revision: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: i64,
}

impl StateStore {
    /// Engine must perform semantic graph/backend qualification before admission.
    /// The state boundary additionally validates shape and pins complete immutable authoring bytes.
    pub fn admit_workflow(
        &mut self,
        request: &AdmissionRequest,
        now: i64,
    ) -> StateResult<AdmissionRecord> {
        label(&request.namespace)?;
        label(&request.submission_key)?;
        digest(&request.schemas_sha256)?;
        request
            .workflow
            .canonical_bytes()
            .map_err(StateError::invalid_contract)?;
        request
            .execution_authority
            .validate()
            .map_err(StateError::invalid_contract)?;
        if request.trial_bindings.len() > 64 {
            return Err(StateError::invalid("trial bindings exceed graph bound"));
        }
        for (task, trial) in &request.trial_bindings {
            label(trial)?;
            if !request.workflow.tasks.iter().any(|t| &t.task_id == task) {
                return Err(StateError::invalid("trial binding task is absent"));
            }
        }
        let limits = limits(request);
        ResourceVector::default().checked_add(limits)?;
        let bytes = encode(request)?;
        let hash = hash(&bytes);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin admission"))?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT run_id FROM workflow_admissions WHERE namespace=?1 AND submission_key=?2",
                params![request.namespace, request.submission_key],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql("lookup submission"))?;
        if let Some(id) = existing {
            let id = id.parse().map_err(StateError::invalid_contract)?;
            let record = load_admission(&tx, &id)?
                .ok_or_else(|| StateError::integrity("admission disappeared"))?;
            if record.request_sha256 != hash {
                return Err(conflict(
                    "submission key",
                    "identical request",
                    "different request",
                ));
            }
            tx.commit().map_err(sql("commit duplicate admission"))?;
            return Ok(record);
        }
        let run_id = RunId::new();
        tx.execute("INSERT INTO workflow_admissions(run_id,namespace,submission_key,request_sha256,request_json,revision,owner_epoch,cancel_requested,created_at) VALUES(?1,?2,?3,?4,?5,0,0,0,?6)",params![run_id.to_string(),request.namespace,request.submission_key,hash,bytes,now]).map_err(sql("insert admission"))?;
        for identity in request
            .workflow
            .tasks
            .iter()
            .map(|t| &t.environment_sha256)
            .collect::<BTreeSet<_>>()
        {
            let record = EnvironmentRecord {
                schema_version: 1,
                run_id: run_id.clone(),
                environment_sha256: identity.clone(),
                state: EnvironmentState::Planned,
                receipt: None,
                updated_at: now,
            };
            tx.execute(
                "INSERT INTO workflow_environments VALUES(?1,?2,?3)",
                params![run_id.to_string(), identity, encode(&record)?],
            )
            .map_err(sql("insert environment intent"))?;
        }
        event(
            &tx,
            &run_id,
            0,
            "admitted",
            &serde_json::json!({"request_sha256":hash}),
            now,
        )?;
        let record = load_admission(&tx, &run_id)?
            .ok_or_else(|| StateError::integrity("admission missing after insert"))?;
        tx.commit().map_err(sql("commit admission"))?;
        Ok(record)
    }
    pub fn get_admission(&self, run_id: &RunId) -> StateResult<Option<AdmissionRecord>> {
        load_admission(&self.connection, run_id)
    }
    /// Fences ledger writes only. This grants no authority over an existing backend process.
    pub fn acquire_controller(
        &mut self,
        run_id: &RunId,
        expected_revision: u64,
        now: i64,
    ) -> StateResult<RunGuard> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin controller epoch"))?;
        let record = required_admission(&tx, run_id)?;
        if record.revision != expected_revision {
            return Err(conflict("run revision", expected_revision, record.revision));
        }
        let epoch = increment(record.owner_epoch)?;
        let next = RunGuard {
            revision: increment(record.revision)?,
            epoch,
        };
        tx.execute(
            "UPDATE workflow_admissions SET revision=?2, owner_epoch=?3 WHERE run_id=?1",
            params![run_id.to_string(), integer(next.revision)?, integer(epoch)?],
        )
        .map_err(sql("acquire controller epoch"))?;
        event(
            &tx,
            run_id,
            next.revision,
            "controller_acquired",
            &next,
            now,
        )?;
        tx.commit().map_err(sql("commit controller epoch"))?;
        Ok(next)
    }
    pub fn prepare_job(
        &mut self,
        run_id: &RunId,
        guard: RunGuard,
        preparation: &JobPreparation,
        now: i64,
    ) -> StateResult<RunGuard> {
        label(&preparation.job_id)?;
        if let Some(trial) = &preparation.trial_id {
            label(trial)?;
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin job reservation"))?;
        let run = authorize(&tx, run_id, guard)?;
        if run.cancellation_requested {
            return Err(StateError::invalid("cancelled run cannot prepare jobs"));
        }
        let task = run
            .request
            .workflow
            .tasks
            .iter()
            .find(|t| t.task_id == preparation.task_id)
            .ok_or_else(|| StateError::invalid("job task is not admitted"))?;
        let jobs = load_jobs(&tx, run_id)?;
        if jobs.iter().any(|j| j.job_id == preparation.job_id) {
            return Err(StateError::invalid("job identity already exists"));
        }
        let prior: Vec<_> = jobs
            .iter()
            .filter(|j| j.task_id == preparation.task_id)
            .collect();
        if prior.len() > usize::from(task.max_retries) {
            return Err(StateError::invalid("task retry ceiling exhausted"));
        }
        if prior.iter().any(|j| {
            j.cancel_requested
                || j.state != JobState::Collected
                || j.reservation.settled.is_none()
                || j.trial_id != preparation.trial_id
                || matches!(j.termination, Some(JobTermination::Exit { code: 0, .. }))
        }) {
            return Err(StateError::invalid(
                "retry requires settled collected failure of the same trial",
            ));
        }
        if run.request.trial_bindings.get(&preparation.task_id) != preparation.trial_id.as_ref() {
            return Err(StateError::invalid(
                "job trial differs from admitted task binding",
            ));
        }
        let first_trial = preparation.trial_id.is_some()
            && !jobs.iter().any(|j| j.trial_id == preparation.trial_id);
        let expected = ResourceVector {
            agent_tokens: task.limits.agent_tokens,
            cpu_seconds: task.limits.cpu_seconds,
            gpu_seconds: task.limits.gpu_seconds,
            wall_seconds: task.limits.wall_seconds,
            storage_bytes: task.limits.storage_bytes,
            trials: u64::from(first_trial),
        };
        if preparation.resources != expected {
            return Err(StateError::invalid(
                "reservation must equal admitted task maximum and trial exposure",
            ));
        }
        let account = accounting(&run, &jobs)?;
        account
            .settled
            .checked_add(account.reserved)?
            .checked_add(preparation.resources)?
            .within(account.limit)?;
        let attempt =
            u32::try_from(prior.len() + 1).map_err(|_| StateError::invalid("attempt overflow"))?;
        let record = JobRecord {
            schema_version: 1,
            run_id: run_id.clone(),
            job_id: preparation.job_id.clone(),
            task_id: preparation.task_id.clone(),
            attempt,
            trial_id: preparation.trial_id.clone(),
            submission_key: format!("{run_id}/{}/{attempt}", preparation.task_id),
            state: JobState::Prepared,
            cancel_requested: false,
            backend_job_id: None,
            binding_receipt: None,
            termination: None,
            quiescence: None,
            collected_manifest: None,
            observation: JobObservation {
                connectivity: JobConnectivity::Unknown,
                observed_at: now,
                evidence: None,
            },
            reservation: ResourceReservation {
                reserved: preparation.resources,
                settled: None,
                estimated: false,
            },
            created_at: now,
            updated_at: now,
        };
        tx.execute(
            "INSERT INTO workflow_jobs VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                record.job_id,
                run_id.to_string(),
                record.task_id.to_string(),
                record.attempt,
                record.trial_id,
                encode(&record)?
            ],
        )
        .map_err(sql("insert prepared job"))?;
        let next = advance(&tx, run_id, guard)?;
        event(&tx, run_id, next.revision, "job_prepared", &record, now)?;
        tx.commit().map_err(sql("commit job reservation"))?;
        Ok(next)
    }
    pub fn get_job(&self, run_id: &RunId, job_id: &str) -> StateResult<Option<JobRecord>> {
        load_job(&self.connection, run_id, job_id)
    }
    pub fn workflow_jobs(&self, run_id: &RunId) -> StateResult<Vec<JobRecord>> {
        load_jobs(&self.connection, run_id)
    }
    pub fn workflow_accounting(&mut self, run_id: &RunId) -> StateResult<WorkflowAccounting> {
        let tx = self
            .connection
            .transaction()
            .map_err(sql("begin accounting snapshot"))?;
        let result = accounting(&required_admission(&tx, run_id)?, &load_jobs(&tx, run_id)?)?;
        tx.commit().map_err(sql("end accounting snapshot"))?;
        Ok(result)
    }
    pub fn transition_job(
        &mut self,
        run_id: &RunId,
        guard: RunGuard,
        job_id: &str,
        transition: &JobTransition,
        now: i64,
    ) -> StateResult<RunGuard> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin job transition"))?;
        let next = change_job(&tx, run_id, guard, job_id, transition, now)?;
        tx.commit().map_err(sql("commit job transition"))?;
        Ok(next)
    }
    pub fn observe_job(
        &mut self,
        run_id: &RunId,
        guard: RunGuard,
        job_id: &str,
        observation: &JobObservation,
    ) -> StateResult<RunGuard> {
        if let Some(e) = &observation.evidence {
            e.validate().map_err(StateError::invalid_contract)?;
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin connectivity observation"))?;
        authorize(&tx, run_id, guard)?;
        let mut job = required_job(&tx, run_id, job_id)?;
        job.observation = observation.clone();
        save_job(&tx, &job)?;
        let next = advance(&tx, run_id, guard)?;
        event(
            &tx,
            run_id,
            next.revision,
            "job_observed",
            &serde_json::json!({"job_id":job_id,"observation":observation}),
            observation.observed_at,
        )?;
        tx.commit()
            .map_err(sql("commit connectivity observation"))?;
        Ok(next)
    }
    pub fn cancel_workflow(
        &mut self,
        run_id: &RunId,
        guard: RunGuard,
        now: i64,
    ) -> StateResult<RunGuard> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin workflow cancellation"))?;
        authorize(&tx, run_id, guard)?;
        tx.execute(
            "UPDATE workflow_admissions SET cancel_requested=1 WHERE run_id=?1",
            [run_id.to_string()],
        )
        .map_err(sql("request workflow cancellation"))?;
        let next = advance(&tx, run_id, guard)?;
        event(
            &tx,
            run_id,
            next.revision,
            "cancellation_requested",
            &(),
            now,
        )?;
        tx.commit().map_err(sql("commit workflow cancellation"))?;
        Ok(next)
    }
    pub fn workflow_environments(&self, run_id: &RunId) -> StateResult<Vec<EnvironmentRecord>> {
        let mut statement=self.connection.prepare("SELECT record_json FROM workflow_environments WHERE run_id=?1 ORDER BY environment_sha256").map_err(sql("read environments"))?;
        let rows = statement
            .query_map([run_id.to_string()], |r| r.get::<_, Vec<u8>>(0))
            .map_err(sql("query environments"))?;
        rows.map(|r| decode(&r.map_err(sql("read environment"))?))
            .collect()
    }
    pub fn transition_environment(
        &mut self,
        run_id: &RunId,
        guard: RunGuard,
        identity: &str,
        state: EnvironmentState,
        receipt: Option<&ArtifactRef>,
        now: i64,
    ) -> StateResult<RunGuard> {
        if let Some(r) = receipt {
            r.validate().map_err(StateError::invalid_contract)?;
        }
        if matches!(state, EnvironmentState::Verified | EnvironmentState::Failed)
            && receipt.is_none()
        {
            return Err(StateError::invalid(
                "terminal environment needs evidence receipt",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin environment transition"))?;
        authorize(&tx, run_id, guard)?;
        let raw:Vec<u8>=tx.query_row("SELECT record_json FROM workflow_environments WHERE run_id=?1 AND environment_sha256=?2",params![run_id.to_string(),identity],|r|r.get(0)).map_err(sql("load environment"))?;
        let mut record: EnvironmentRecord = decode(&raw)?;
        if !matches!(
            (record.state, state),
            (EnvironmentState::Planned, EnvironmentState::Preparing)
                | (
                    EnvironmentState::Preparing,
                    EnvironmentState::Verified
                        | EnvironmentState::Failed
                        | EnvironmentState::Unknown
                )
                | (
                    EnvironmentState::Unknown,
                    EnvironmentState::Preparing
                        | EnvironmentState::Verified
                        | EnvironmentState::Failed
                )
        ) {
            return Err(StateError::invalid("invalid environment transition"));
        }
        record.state = state;
        record.receipt = receipt.cloned();
        record.updated_at = now;
        tx.execute("UPDATE workflow_environments SET record_json=?3 WHERE run_id=?1 AND environment_sha256=?2",params![run_id.to_string(),identity,encode(&record)?]).map_err(sql("update environment"))?;
        let next = advance(&tx, run_id, guard)?;
        event(
            &tx,
            run_id,
            next.revision,
            "environment_changed",
            &record,
            now,
        )?;
        tx.commit().map_err(sql("commit environment transition"))?;
        Ok(next)
    }
    pub fn apply_job_decision(
        &mut self,
        run_id: &RunId,
        decision: &JobDecision,
        now: i64,
    ) -> StateResult<DecisionRecord> {
        label(&decision.decision_id)?;
        label(&decision.reason)?;
        if decision.evidence.len() > 64 {
            return Err(StateError::invalid(
                "decision evidence exceeds 64 references",
            ));
        }
        for e in &decision.evidence {
            e.validate().map_err(StateError::invalid_contract)?;
        }
        let digest = hash(&encode(decision)?);
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql("begin decision"))?;
        let prior:Option<(String,Vec<u8>)>=tx.query_row("SELECT request_sha256,record_json FROM workflow_decisions WHERE run_id=?1 AND decision_id=?2",params![run_id.to_string(),decision.decision_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(sql("read decision"))?;
        if let Some((previous, raw)) = prior {
            if previous != digest {
                return Err(StateError::invalid(
                    "decision identity reused with changed content",
                ));
            }
            return decode(&raw);
        }
        let resulting_guard = change_job(
            &tx,
            run_id,
            decision.expected,
            &decision.job_id,
            &decision.transition,
            now,
        )?;
        let record = DecisionRecord {
            schema_version: 1,
            run_id: run_id.clone(),
            decision: decision.clone(),
            resulting_guard,
            applied_at: now,
        };
        tx.execute(
            "INSERT INTO workflow_decisions VALUES(?1,?2,?3,?4)",
            params![
                run_id.to_string(),
                decision.decision_id,
                digest,
                encode(&record)?
            ],
        )
        .map_err(sql("insert applied decision"))?;
        event(
            &tx,
            run_id,
            resulting_guard.revision,
            "decision_applied",
            &record,
            now,
        )?;
        tx.commit().map_err(sql("commit decision"))?;
        Ok(record)
    }
    pub fn workflow_events(
        &self,
        run_id: &RunId,
        after: u64,
        limit: usize,
    ) -> StateResult<Vec<WorkflowEvent>> {
        if !(1..=1000).contains(&limit) {
            return Err(StateError::invalid("event limit must be 1 through 1000"));
        }
        let mut statement=self.connection.prepare("SELECT sequence,revision,event_type,payload_json,created_at FROM workflow_events WHERE run_id=?1 AND sequence>?2 ORDER BY sequence LIMIT ?3").map_err(sql("prepare workflow events"))?;
        let rows = statement
            .query_map(
                params![run_id.to_string(), integer(after)?, limit as i64],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Vec<u8>>(3)?,
                        r.get::<_, i64>(4)?,
                    ))
                },
            )
            .map_err(sql("query workflow events"))?;
        rows.map(|r| {
            let (sequence, revision, event_type, payload, created_at) =
                r.map_err(sql("read workflow event"))?;
            Ok(WorkflowEvent {
                sequence: unsigned(sequence)?,
                run_id: run_id.clone(),
                revision: unsigned(revision)?,
                event_type,
                payload: decode(&payload)?,
                created_at,
            })
        })
        .collect()
    }
}

fn change_job(
    tx: &Transaction<'_>,
    run_id: &RunId,
    guard: RunGuard,
    job_id: &str,
    transition: &JobTransition,
    now: i64,
) -> StateResult<RunGuard> {
    let run = authorize(tx, run_id, guard)?;
    let mut job = required_job(tx, run_id, job_id)?;
    match transition {
        JobTransition::Submit
            if job.state == JobState::Prepared
                && !run.cancellation_requested
                && !job.cancel_requested =>
        {
            job.state = JobState::SubmissionUnknown;
        }
        JobTransition::Bind {
            backend_job_id,
            binding_receipt,
        } if job.state == JobState::SubmissionUnknown => {
            label(backend_job_id)?;
            binding_receipt
                .validate()
                .map_err(StateError::invalid_contract)?;
            job.backend_job_id = Some(backend_job_id.clone());
            job.binding_receipt = Some(binding_receipt.clone());
            job.state = JobState::Running;
        }
        JobTransition::ConfirmAbsent { receipt } if job.state == JobState::SubmissionUnknown => {
            receipt.validate().map_err(StateError::invalid_contract)?;
            // The same durable job/key may be submitted again only after authoritative exclusion.
            job.state = JobState::Prepared;
        }
        JobTransition::Terminate {
            termination,
            quiescence,
        } if matches!(
            job.state,
            JobState::Running | JobState::SubmissionUnknown | JobState::Prepared
        ) =>
        {
            match termination {
                JobTermination::Exit { receipt, .. }
                | JobTermination::Lost { receipt }
                | JobTermination::Absent { receipt } => {
                    receipt.validate().map_err(StateError::invalid_contract)?
                }
                JobTermination::Signal { signal, receipt } => {
                    if *signal == 0 {
                        return Err(StateError::invalid("terminal signal must be positive"));
                    }
                    receipt.validate().map_err(StateError::invalid_contract)?;
                }
            }
            if job.state == JobState::Prepared
                && !matches!(termination, JobTermination::Absent { .. })
            {
                return Err(StateError::invalid(
                    "unsubmitted job can only terminate with authoritative absence",
                ));
            }
            if job.state == JobState::SubmissionUnknown
                && matches!(
                    termination,
                    JobTermination::Exit { .. } | JobTermination::Signal { .. }
                )
            {
                return Err(StateError::invalid(
                    "exit or signal requires a bound running backend job",
                ));
            }
            if job.state == JobState::Running
                && matches!(termination, JobTermination::Absent { .. })
            {
                return Err(StateError::invalid(
                    "bound running job requires termination or authoritative loss",
                ));
            }
            match quiescence {
                JobQuiescence::ProcessTree { receipt }
                | JobQuiescence::AuthoritativeAbsence { receipt } => {
                    receipt.validate().map_err(StateError::invalid_contract)?
                }
            }
            if matches!(
                termination,
                JobTermination::Exit { .. } | JobTermination::Signal { .. }
            ) && !matches!(quiescence, JobQuiescence::ProcessTree { .. })
            {
                return Err(StateError::invalid(
                    "process exit requires process-tree quiescence",
                ));
            }
            if matches!(termination, JobTermination::Absent { .. })
                && !matches!(quiescence, JobQuiescence::AuthoritativeAbsence { .. })
            {
                return Err(StateError::invalid(
                    "absence requires exclusion of delayed creation",
                ));
            }
            job.state = JobState::Terminal;
            job.termination = Some(termination.clone());
            job.quiescence = Some(quiescence.clone());
        }
        JobTransition::BeginCollection if job.state == JobState::Terminal => {
            job.state = JobState::Collecting;
        }
        JobTransition::FinishCollection { manifest } if job.state == JobState::Collecting => {
            manifest.validate().map_err(StateError::invalid_contract)?;
            job.collected_manifest = Some(manifest.clone());
            job.state = JobState::Collected;
        }
        JobTransition::Cancel => {
            job.cancel_requested = true;
        }
        JobTransition::Settle { measured }
            if job.state == JobState::Collected && job.reservation.settled.is_none() =>
        {
            let usage = measured.unwrap_or(job.reservation.reserved);
            usage.within(job.reservation.reserved)?;
            if usage.trials != job.reservation.reserved.trials {
                return Err(StateError::invalid(
                    "settlement cannot refund admitted trial identity",
                ));
            }
            job.reservation.settled = Some(usage);
            job.reservation.estimated = measured.is_none();
        }
        _ => {
            return Err(StateError::invalid(
                "job transition is not permitted from persisted state",
            ))
        }
    }
    job.updated_at = now;
    save_job(tx, &job)?;
    let next = advance(tx, run_id, guard)?;
    event(
        tx,
        run_id,
        next.revision,
        "job_transition",
        &serde_json::json!({"job_id":job_id,"transition":transition}),
        now,
    )?;
    Ok(next)
}
fn load_admission(c: &Connection, id: &RunId) -> StateResult<Option<AdmissionRecord>> {
    let raw=c.query_row("SELECT request_sha256,request_json,revision,owner_epoch,cancel_requested,created_at FROM workflow_admissions WHERE run_id=?1",[id.to_string()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Vec<u8>>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,bool>(4)?,r.get::<_,i64>(5)?))).optional().map_err(sql("read admission"))?;
    raw.map(
        |(request_sha256, bytes, revision, owner_epoch, cancellation_requested, created_at)| {
            if hash(&bytes) != request_sha256 {
                return Err(StateError::integrity(
                    "admission digest differs from pinned request",
                ));
            }
            let request: AdmissionRequest = decode(&bytes)?;
            request
                .workflow
                .validate_shape()
                .map_err(StateError::invalid_contract)?;
            Ok(AdmissionRecord {
                schema_version: 1,
                run_id: id.clone(),
                request_sha256,
                request,
                revision: unsigned(revision)?,
                owner_epoch: unsigned(owner_epoch)?,
                cancellation_requested,
                created_at,
            })
        },
    )
    .transpose()
}
fn required_admission(c: &Connection, id: &RunId) -> StateResult<AdmissionRecord> {
    load_admission(c, id)?.ok_or_else(|| StateError::invalid("workflow admission does not exist"))
}
fn authorize(c: &Connection, id: &RunId, guard: RunGuard) -> StateResult<AdmissionRecord> {
    let r = required_admission(c, id)?;
    if guard.epoch == 0 || r.guard() != guard {
        return Err(conflict(
            "workflow ownership",
            format!("{guard:?}"),
            format!("{:?}", r.guard()),
        ));
    }
    Ok(r)
}
fn advance(c: &Connection, id: &RunId, guard: RunGuard) -> StateResult<RunGuard> {
    let next = RunGuard {
        revision: increment(guard.revision)?,
        epoch: guard.epoch,
    };
    let changed=c.execute("UPDATE workflow_admissions SET revision=?2 WHERE run_id=?1 AND revision=?3 AND owner_epoch=?4",params![id.to_string(),integer(next.revision)?,integer(guard.revision)?,integer(guard.epoch)?]).map_err(sql("advance workflow revision"))?;
    if changed != 1 {
        return Err(StateError::integrity(
            "guarded revision update did not change one row",
        ));
    }
    Ok(next)
}
fn load_job(c: &Connection, run: &RunId, id: &str) -> StateResult<Option<JobRecord>> {
    let raw: Option<(Vec<u8>,String,u32,Option<String>)> = c
        .query_row(
            "SELECT record_json,task_id,attempt,trial_id FROM workflow_jobs WHERE run_id=?1 AND job_id=?2",
            params![run.to_string(), id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)),
        )
        .optional()
        .map_err(sql("read job"))?;
    raw.map(|(b, task_id, attempt, trial_id)| {
        let j: JobRecord = decode(&b)?;
        if j.schema_version != 1
            || j.run_id != *run
            || j.job_id != id
            || j.task_id.to_string() != task_id
            || j.attempt != attempt
            || j.trial_id != trial_id
            || j.submission_key != format!("{run}/{task_id}/{attempt}")
        {
            return Err(StateError::integrity(
                "job row identity or version mismatch",
            ));
        }
        validate_job(&j)?;
        Ok(j)
    })
    .transpose()
}
fn validate_job(job: &JobRecord) -> StateResult<()> {
    let terminal = matches!(
        job.state,
        JobState::Terminal | JobState::Collecting | JobState::Collected
    );
    if terminal != job.termination.is_some()
        || terminal != job.quiescence.is_some()
        || (job.state == JobState::Collected) != job.collected_manifest.is_some()
        || job.backend_job_id.is_some() != job.binding_receipt.is_some()
        || (job.state == JobState::Running && job.backend_job_id.is_none())
        || (matches!(job.state, JobState::Prepared | JobState::SubmissionUnknown)
            && job.backend_job_id.is_some())
    {
        return Err(StateError::integrity(
            "persisted job state and evidence disagree",
        ));
    }
    ResourceVector::default().checked_add(job.reservation.reserved)?;
    if let Some(usage) = job.reservation.settled {
        if job.state != JobState::Collected || usage.trials != job.reservation.reserved.trials {
            return Err(StateError::integrity(
                "uncollected job has settlement or refunded trial",
            ));
        }
        usage.within(job.reservation.reserved)?;
    } else if job.reservation.estimated {
        return Err(StateError::integrity(
            "unsettled job has estimated accounting",
        ));
    }
    for artifact in [
        &job.binding_receipt,
        &job.collected_manifest,
        &job.observation.evidence,
    ]
    .into_iter()
    .flatten()
    {
        artifact.validate().map_err(StateError::invalid_contract)?;
    }
    Ok(())
}
fn required_job(c: &Connection, run: &RunId, id: &str) -> StateResult<JobRecord> {
    load_job(c, run, id)?.ok_or_else(|| StateError::invalid("job does not exist"))
}
fn load_jobs(c: &Connection, run: &RunId) -> StateResult<Vec<JobRecord>> {
    let mut stmt = c
        .prepare(
            "SELECT job_id FROM workflow_jobs WHERE run_id=?1 ORDER BY task_id,attempt LIMIT 257",
        )
        .map_err(sql("prepare jobs"))?;
    let ids = stmt
        .query_map([run.to_string()], |r| r.get::<_, String>(0))
        .map_err(sql("query jobs"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql("read job ids"))?;
    if ids.len() > 256 {
        return Err(StateError::integrity("jobs exceed static attempt bound"));
    }
    ids.into_iter()
        .map(|id| required_job(c, run, &id))
        .collect()
}
fn save_job(c: &Connection, j: &JobRecord) -> StateResult<()> {
    validate_job(j)?;
    c.execute(
        "UPDATE workflow_jobs SET record_json=?3 WHERE run_id=?1 AND job_id=?2",
        params![j.run_id.to_string(), j.job_id, encode(j)?],
    )
    .map_err(sql("save job"))?;
    Ok(())
}
fn accounting(run: &AdmissionRecord, jobs: &[JobRecord]) -> StateResult<WorkflowAccounting> {
    let mut result = WorkflowAccounting {
        limit: limits(&run.request),
        reserved: ResourceVector::default(),
        settled: ResourceVector::default(),
    };
    for j in jobs {
        if let Some(s) = j.reservation.settled {
            result.settled = result.settled.checked_add(s)?;
        } else {
            result.reserved = result.reserved.checked_add(j.reservation.reserved)?;
        }
    }
    result
        .reserved
        .checked_add(result.settled)?
        .within(result.limit)?;
    Ok(result)
}
fn limits(r: &AdmissionRequest) -> ResourceVector {
    let l = &r.workflow.limits;
    ResourceVector {
        agent_tokens: l.agent_tokens,
        cpu_seconds: l.cpu_seconds,
        gpu_seconds: l.gpu_seconds,
        wall_seconds: l.wall_seconds,
        storage_bytes: l.storage_bytes,
        trials: r.trial_limit,
    }
}
fn event<T: Serialize>(
    c: &Connection,
    run: &RunId,
    revision: u64,
    kind: &str,
    payload: &T,
    now: i64,
) -> StateResult<()> {
    c.execute("INSERT INTO workflow_events(run_id,revision,event_type,payload_json,created_at) VALUES(?1,?2,?3,?4,?5)",params![run.to_string(),integer(revision)?,kind,encode(payload)?,now]).map_err(sql("append workflow event"))?;
    Ok(())
}
fn encode<T: Serialize>(value: &T) -> StateResult<Vec<u8>> {
    let value = serde_json::to_value(value).map_err(|source| StateError::Serialization {
        context: "workflow record".into(),
        source,
    })?;
    let bytes = serde_json::to_vec(&value).map_err(|source| StateError::Serialization {
        context: "workflow record".into(),
        source,
    })?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(StateError::invalid("workflow record exceeds byte limit"));
    }
    Ok(bytes)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> StateResult<T> {
    let value = harp_contracts::decode_strict_json(bytes, 2 * 1024 * 1024)
        .map_err(StateError::invalid_contract)?;
    serde_json::from_value(value).map_err(|source| StateError::Serialization {
        context: "persisted workflow record".into(),
        source,
    })
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn label(value: &str) -> StateResult<()> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(StateError::invalid(
            "workflow label must be 1 through 256 bytes without controls",
        ));
    }
    Ok(())
}
fn digest(value: &str) -> StateResult<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(StateError::invalid("expected lowercase SHA-256"));
    }
    Ok(())
}
fn integer(v: u64) -> StateResult<i64> {
    i64::try_from(v).map_err(|_| StateError::invalid("workflow integer exceeds durable range"))
}
fn increment(v: u64) -> StateResult<u64> {
    v.checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .ok_or_else(|| StateError::invalid("workflow revision overflow"))
}
fn conflict(entity: impl ToString, expected: impl ToString, actual: impl ToString) -> StateError {
    StateError::Conflict {
        entity: entity.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}
fn sql(context: &'static str) -> impl Fn(rusqlite::Error) -> StateError {
    move |source| StateError::sqlite(context, source)
}

fn unsigned(v: i64) -> StateResult<u64> {
    u64::try_from(v).map_err(|_| StateError::integrity("negative workflow counter"))
}
