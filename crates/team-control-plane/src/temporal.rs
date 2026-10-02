//! Durable command bridge. PostgreSQL owns every domain transition and disposition;
//! Temporal owns deterministic waits. Signals are hints, never commands or authority.
use crate::{Broker, auth, domain::*, storage, validation};
use serde::{Deserialize, Serialize};
use tokio_postgres::Transaction;
use uuid::Uuid;

pub(crate) async fn migrate(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('temporal_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        tx.batch_execute(include_str!("../migrations/0004.sql"))
            .await?;
    }
    let versions = tx
        .query("SELECT version FROM temporal_schema_version", &[])
        .await?;
    if versions.len() != 1 || versions[0].get::<_, i32>(0) != 1 {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AggregateKind {
    Mission,
    Task,
}
impl AggregateKind {
    fn name(&self) -> &'static str {
        match self {
            Self::Mission => "mission",
            Self::Task => "task",
        }
    }
}
/// Workflow generation differs from an attempt generation. No bearer is serialized.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowRef {
    pub kind: AggregateKind,
    pub aggregate_id: Uuid,
    pub generation: i64,
    pub recovery_epoch: Uuid,
    pub workflow_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowInput {
    pub reference: WorkflowRef,
    pub cursor: i64,
    pub waits: Vec<Uuid>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowLifecycle {
    Running,
    Completed,
    Failed,
    Canceled,
    Terminated,
    ContinuedAsNew,
    TimedOut,
    Paused,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservedExecution {
    pub run_id: String,
    pub incarnation: i64,
    pub lifecycle: WorkflowLifecycle,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Progress {
    pub cursor: i64,
    pub closed: bool,
    pub canonical_revision: i64,
    pub waits: Vec<Uuid>,
    pub operations: Vec<OperationProgress>,
    #[serde(default)]
    pub mailbox_attention: Vec<crate::mailbox::WaitAttention>,
    #[serde(default)]
    pub observed: Option<ObservedExecution>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueuedCommand {
    pub sequence: i64,
    pub principal_id: Uuid,
    pub disposition: Option<String>,
    pub result: Option<serde_json::Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wakeup {
    pub id: Uuid,
    pub reference: WorkflowRef,
    pub sequence: i64,
}
#[derive(Clone, Debug)]
pub struct Bridge {
    broker: Broker,
}
impl Bridge {
    pub fn new(broker: Broker) -> Self {
        Self { broker }
    }
    async fn check(
        &self,
        tx: &Transaction<'_>,
        bearer: &str,
        r: &WorkflowRef,
    ) -> Result<auth::Actor> {
        storage::active(tx, self.broker.epoch).await?;
        if r.recovery_epoch != self.broker.epoch
            || r.workflow_id != workflow_id(&r.kind, r.aggregate_id, r.generation, r.recovery_epoch)
        {
            return Err(err(ErrorCode::Unauthorized));
        }
        let actor = auth::authenticate(tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(tx, &actor).await?;
        self.scope(tx, &actor, &r.kind, r.aggregate_id).await?;
        let row=tx.query_opt("SELECT generation,recovery_epoch FROM workflow_registry WHERE aggregate_kind=$1 AND aggregate_id=$2 ORDER BY generation DESC LIMIT 1",&[&r.kind.name(),&r.aggregate_id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
        if row.get::<_, i64>(0) != r.generation || row.get::<_, Uuid>(1) != r.recovery_epoch {
            return Err(err(ErrorCode::StaleGeneration));
        }
        Ok(actor)
    }
    async fn scope(
        &self,
        tx: &Transaction<'_>,
        actor: &auth::Actor,
        kind: &AggregateKind,
        id: Uuid,
    ) -> Result<()> {
        match kind {
            AggregateKind::Mission => {
                if tx
                    .query_opt("SELECT id FROM missions WHERE id=$1", &[&id])
                    .await?
                    .is_none()
                {
                    return Err(err(ErrorCode::NotFound));
                }
                actor.scope(id, None)
            }
            AggregateKind::Task => {
                let row = tx
                    .query_opt("SELECT mission_id FROM tasks WHERE id=$1", &[&id])
                    .await?
                    .ok_or_else(|| err(ErrorCode::NotFound))?;
                actor.scope(row.get(0), Some(id))
            }
        }
    }
    /// Explicit Owner allocation; same generation is idempotent. A closed generation
    /// never restarts implicitly. New allocation requires the exact previous generation.
    pub async fn register(
        &self,
        bearer: &str,
        kind: AggregateKind,
        id: Uuid,
        previous: i64,
    ) -> Result<WorkflowRef> {
        if previous < 0 {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        actor.owner()?;
        self.scope(&tx, &actor, &kind, id).await?;
        let generation = previous
            .checked_add(1)
            .ok_or_else(|| err(ErrorCode::InvalidArgument))?;
        let r = WorkflowRef {
            workflow_id: workflow_id(&kind, id, generation, self.broker.epoch),
            kind,
            aggregate_id: id,
            generation,
            recovery_epoch: self.broker.epoch,
        };
        if let Some(row)=tx.query_opt("SELECT generation,closed,recovery_epoch FROM workflow_registry WHERE aggregate_kind=$1 AND aggregate_id=$2 ORDER BY generation DESC LIMIT 1",&[&r.kind.name(),&id]).await? {
   let current:i64=row.get(0);
   if current==generation && row.get::<_,Uuid>(2)==self.broker.epoch {tx.commit().await?;return Ok(r);}
   if current!=previous || !row.get::<_,bool>(1) {return Err(err(ErrorCode::StaleGeneration));}
  } else if previous!=0 {return Err(err(ErrorCode::StaleGeneration));}
        tx.execute("INSERT INTO workflow_registry(aggregate_kind,aggregate_id,generation,recovery_epoch,workflow_id) VALUES($1,$2,$3,$4,$5)",&[&r.kind.name(),&id,&generation,&self.broker.epoch,&r.workflow_id]).await?;
        tx.execute(
            "INSERT INTO workflow_wakeups(id,workflow_id,sequence) VALUES($1,$2,0)",
            &[&Uuid::new_v4(), &r.workflow_id],
        )
        .await?;
        tx.commit().await?;
        Ok(r)
    }
    /// Read the current registry generation for a canonical aggregate. No caller
    /// needs to reconstruct a Workflow ID or infer a generation from runtime state.
    pub async fn lookup(&self, bearer: &str, kind: AggregateKind, id: Uuid) -> Result<WorkflowRef> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        self.scope(&tx, &actor, &kind, id).await?;
        let row=tx.query_opt("SELECT generation,recovery_epoch,workflow_id FROM workflow_registry WHERE aggregate_kind=$1 AND aggregate_id=$2 ORDER BY generation DESC LIMIT 1",&[&kind.name(),&id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
        let r = WorkflowRef {
            kind,
            aggregate_id: id,
            generation: row.get(0),
            recovery_epoch: row.get(1),
            workflow_id: row.get(2),
        };
        if r.recovery_epoch != self.broker.epoch {
            return Err(err(ErrorCode::StaleGeneration));
        }
        tx.commit().await?;
        Ok(r)
    }
    pub async fn enqueue(
        &self,
        bearer: &str,
        r: &WorkflowRef,
        mut envelope: CommandEnvelope,
    ) -> Result<QueuedCommand> {
        if envelope.recovery_epoch != r.recovery_epoch
            || envelope.schema_version != 1
            || envelope.key.is_empty()
            || envelope.key.len() > 256
            || envelope.command.aggregate() != r.aggregate_id
            || envelope.command.aggregate_kind() != r.kind.name()
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let actor = self.check(&tx, bearer, r).await?;
        auth::authorize_command(&tx, &actor, &envelope.command).await?;
        match &mut envelope.command {
            Command::SetDependencies { dependencies, .. } => dependencies.sort(),
            Command::CreateTask { spec, .. } => spec.dependencies.sort(),
            _ => {}
        }
        let family = envelope.command.family();
        let digest = validation::digest(&envelope.command)?;
        if let Some(row)=tx.query_opt("SELECT sequence,digest,disposition,result FROM workflow_commands WHERE workflow_id=$1 AND principal_id=$2 AND family=$3 AND command_key=$4",&[&r.workflow_id,&actor.principal,&family,&envelope.key]).await? {
   if row.get::<_,String>(1)!=digest{return Err(err(ErrorCode::IdempotencyConflict));}
   let q=QueuedCommand{sequence:row.get(0),principal_id:actor.principal,disposition:row.get(2),result:row.get(3)};tx.commit().await?;return Ok(q);
  }
        let row = tx
            .query_one(
                "SELECT next_sequence,closed FROM workflow_registry WHERE workflow_id=$1",
                &[&r.workflow_id],
            )
            .await?;
        let sequence: i64 = row.get(0);
        let closed: bool = row.get(1);
        let (disposition, result) = if closed {
            (
                Some("rejected"),
                Some(
                    serde_json::to_value(err(ErrorCode::StaleGeneration))
                        .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?,
                ),
            )
        } else {
            (None, None)
        };
        tx.execute("INSERT INTO workflow_commands(workflow_id,sequence,principal_id,family,command_key,digest,envelope,disposition,result) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",&[&r.workflow_id,&sequence,&actor.principal,&family,&envelope.key,&digest,&serde_json::to_value(&envelope).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?,&disposition,&result]).await?;
        tx.execute("UPDATE workflow_registry SET next_sequence=next_sequence+1,cursor=CASE WHEN closed THEN $2 ELSE cursor END WHERE workflow_id=$1",&[&r.workflow_id,&sequence]).await?;
        tx.execute(
            "INSERT INTO workflow_wakeups(id,workflow_id,sequence) VALUES($1,$2,$3)",
            &[&Uuid::new_v4(), &r.workflow_id, &sequence],
        )
        .await?;
        tx.commit().await?;
        Ok(QueuedCommand {
            sequence,
            principal_id: actor.principal,
            disposition: disposition.map(str::to_owned),
            result,
        })
    }
    /// A repeated Activity returns its recorded disposition. Gaps cannot advance the
    /// cursor. Live credential must belong to the original principal, even for Owner.
    pub async fn apply(
        &self,
        bearer: &str,
        r: &WorkflowRef,
        sequence: i64,
    ) -> Result<QueuedCommand> {
        self.apply_with_capability(bearer, Some(bearer), r, sequence)
            .await
    }
    /// Live scheduler authority may record a caller rejection; it never executes
    /// a command with the scheduler's privileges instead of the caller's.
    pub async fn apply_with_capability(
        &self,
        service_bearer: &str,
        caller_bearer: Option<&str>,
        r: &WorkflowRef,
        sequence: i64,
    ) -> Result<QueuedCommand> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let service = self.check(&tx, service_bearer, r).await?;
        if !matches!(service.role, Role::Owner | Role::Lead) {
            return Err(err(ErrorCode::Unauthorized));
        }
        let row=tx.query_opt("SELECT principal_id,envelope,disposition,result FROM workflow_commands WHERE workflow_id=$1 AND sequence=$2",&[&r.workflow_id,&sequence]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
        let principal: Uuid = row.get(0);
        let envelope: CommandEnvelope = serde_json::from_value(row.get(1))
            .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        let disposition: Option<String> = row.get(2);
        if disposition.is_some() {
            let q = QueuedCommand {
                sequence,
                principal_id: principal,
                disposition,
                result: row.get(3),
            };
            tx.commit().await?;
            return Ok(q);
        }
        let cursor: i64 = tx
            .query_one(
                "SELECT cursor FROM workflow_registry WHERE workflow_id=$1",
                &[&r.workflow_id],
            )
            .await?
            .get(0);
        if sequence != cursor + 1 {
            return Err(err(ErrorCode::StaleRevision));
        }
        let outcome = match caller_bearer {
            Some(bearer) => match auth::authenticate(&tx, bearer, self.broker.epoch).await {
                Ok(actor) if actor.principal == principal => {
                    self.broker
                        .execute_in_transaction(&tx, bearer, envelope)
                        .await
                }
                Ok(_) => Err(err(ErrorCode::Unauthorized)),
                Err(e) => Err(e),
            },
            None => Err(err(ErrorCode::Unauthorized)),
        };
        if matches!(&outcome,Err(e) if matches!(e.code,ErrorCode::TemporarilyUnavailable|ErrorCode::IncompatibleSchema))
        {
            return Err(outcome.unwrap_err());
        }
        let disposition = if outcome.is_ok() {
            "applied"
        } else {
            "rejected"
        }
        .to_string();
        let value = match outcome {
            Ok(v) => serde_json::to_value(v),
            Err(e) => serde_json::to_value(e),
        }
        .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        tx.execute("UPDATE workflow_commands SET disposition=$3,result=$4 WHERE workflow_id=$1 AND sequence=$2",&[&r.workflow_id,&sequence,&disposition,&value]).await?;
        tx.execute(
            "UPDATE workflow_registry SET cursor=$2 WHERE workflow_id=$1",
            &[&r.workflow_id, &sequence],
        )
        .await?;
        tx.commit().await?;
        Ok(QueuedCommand {
            sequence,
            principal_id: principal,
            disposition: Some(disposition),
            result: Some(value),
        })
    }
    pub async fn progress(&self, bearer: &str, r: &WorkflowRef) -> Result<Progress> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let actor = self.check(&tx, bearer, r).await?;
        let mailbox_attention =
            crate::mailbox::workflow_attention(&tx, self.broker.epoch, r, &actor).await?;
        let row = tx
            .query_one(
                "SELECT cursor,closed,observed_run_id,incarnation,lifecycle FROM workflow_registry WHERE workflow_id=$1",
                &[&r.workflow_id],
            )
            .await?;
        let revision: i64 = match r.kind {
            AggregateKind::Mission => tx
                .query_one(
                    "SELECT revision FROM missions WHERE id=$1",
                    &[&r.aggregate_id],
                )
                .await?
                .get(0),
            AggregateKind::Task => tx
                .query_one("SELECT revision FROM tasks WHERE id=$1", &[&r.aggregate_id])
                .await?
                .get(0),
        };
        let mut waits:Vec<Uuid>=match r.kind {AggregateKind::Mission=>Vec::new(),AggregateKind::Task=>tx.query("SELECT o.id FROM operations o JOIN attempts a ON a.id=o.attempt_id WHERE a.task_id=$1 ORDER BY o.id",&[&r.aggregate_id]).await?.iter().map(|v|v.get(0)).collect()};
        crate::kernel::schema(&tx).await?;
        // A canonical Mission owns planning; a genuine Task owns its Worker and
        // Verifier episodes. Unknown journals stay observable, never fresh RPC candidates.
        let episodes=match r.kind {
            AggregateKind::Mission=>tx.query("SELECT o.episode_id FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.mission_id=$1 AND x.task_id IS NULL ORDER BY o.episode_id", &[&r.aggregate_id]).await?,
            AggregateKind::Task=>tx.query("SELECT o.episode_id FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.task_id=$1 ORDER BY o.episode_id", &[&r.aggregate_id]).await?,
        };
        waits.extend(episodes.into_iter().map(|row| row.get::<_, Uuid>(0)));
        let p = Progress {
            cursor: row.get(0),
            closed: row.get(1),
            canonical_revision: revision,
            waits,
            operations: Vec::new(),
            mailbox_attention,
            observed: row
                .get::<_, Option<String>>(2)
                .map(|run_id| -> Result<ObservedExecution> {
                    Ok(ObservedExecution {
                        run_id,
                        incarnation: row.get(3),
                        lifecycle: serde_json::from_value(serde_json::Value::String(row.get(4)))
                            .map_err(|_| err(ErrorCode::IncompatibleSchema))?,
                    })
                })
                .transpose()?,
        };
        tx.commit().await?;
        Ok(p)
    }
    pub async fn next(&self, bearer: &str, r: &WorkflowRef) -> Result<Option<(i64, Uuid)>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.check(&tx, bearer, r).await?;
        let row=tx.query_opt("SELECT c.sequence,c.principal_id FROM workflow_commands c JOIN workflow_registry r USING(workflow_id) WHERE c.workflow_id=$1 AND c.sequence=r.cursor+1 AND c.disposition IS NULL",&[&r.workflow_id]).await?;
        let next = row.map(|v| (v.get(0), v.get(1)));
        tx.commit().await?;
        Ok(next)
    }
    /// Only Owner may end orchestration, and only after all queued commands have
    /// dispositions. This does not mutate task status or imply task acceptance.
    pub async fn close(&self, bearer: &str, r: &WorkflowRef) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.check(&tx, bearer, r).await?.owner()?;
        let row = tx
            .query_one(
                "SELECT cursor,next_sequence FROM workflow_registry WHERE workflow_id=$1",
                &[&r.workflow_id],
            )
            .await?;
        if row.get::<_, i64>(0) + 1 != row.get::<_, i64>(1) {
            return Err(err(ErrorCode::StaleRevision));
        }
        tx.execute(
            "UPDATE workflow_registry SET closed=true WHERE workflow_id=$1",
            &[&r.workflow_id],
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn pending_wakeups(&self, bearer: &str, r: &WorkflowRef) -> Result<Vec<Wakeup>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.check(&tx, bearer, r).await?;
        let values=tx.query("SELECT id,sequence FROM workflow_wakeups WHERE workflow_id=$1 AND NOT delivered ORDER BY sequence",&[&r.workflow_id]).await?.iter().map(|v|Wakeup{id:v.get(0),sequence:v.get(1),reference:r.clone()}).collect();
        tx.commit().await?;
        Ok(values)
    }
    pub async fn acknowledge_wakeup(&self, bearer: &str, r: &WorkflowRef, id: Uuid) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.scheduler(&tx, bearer, r).await?;
        if tx
            .execute(
                "UPDATE workflow_wakeups SET delivered=true WHERE workflow_id=$1 AND id=$2",
                &[&r.workflow_id, &id],
            )
            .await?
            != 1
        {
            return Err(err(ErrorCode::NotFound));
        }
        tx.commit().await?;
        Ok(())
    }
}
fn workflow_id(kind: &AggregateKind, id: Uuid, generation: i64, epoch: Uuid) -> String {
    format!("corenius-{}-{}-{}-g{}", epoch, kind.name(), id, generation)
}

pub use workflows::{BridgeActivities, MissionWorkflow, TaskWorkflow};
mod workflows {
    use super::{
        AggregateKind, Bridge, OperationDriver, OperationProgress, Progress, WorkflowInput,
    };
    use crate::domain::{Error, ErrorCode, err};
    use std::{collections::HashMap, sync::Arc, time::Duration};
    use temporalio_macros::{activities, workflow, workflow_methods};
    use temporalio_sdk::{
        ActivityOptions, ApplicationFailure, ContinueAsNewOptions, SyncWorkflowContext,
        WorkflowContext, WorkflowResult,
        activities::{ActivityContext, ActivityError},
    };
    use uuid::Uuid;
    /// Worker-local capabilities. Debug deliberately excludes bearer credentials.
    /// Provisioning is explicit; no role is synthesized from a command or workflow.
    pub struct BridgeActivities {
        bridge: Bridge,
        read_bearer: String,
        credentials: HashMap<Uuid, String>,
        driver: Option<Arc<dyn OperationDriver>>,
    }
    impl std::fmt::Debug for BridgeActivities {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BridgeActivities")
                .field("bridge", &self.bridge)
                .finish_non_exhaustive()
        }
    }
    impl BridgeActivities {
        pub fn new(
            bridge: Bridge,
            read_bearer: String,
            credentials: HashMap<Uuid, String>,
        ) -> Self {
            Self {
                bridge,
                read_bearer,
                credentials,
                driver: None,
            }
        }
    }
    impl BridgeActivities {
        /// Register the assembled kernel's trusted runtime binding. Without a driver,
        /// operation refs remain explicitly unconfigured, never reported completed.
        pub fn with_operation_driver(mut self, driver: Arc<dyn OperationDriver>) -> Self {
            self.driver = Some(driver);
            self
        }
    }
    fn activity_error(e: Error) -> ActivityError {
        let message = format!("broker {:?}", e.code);
        if e.code == ErrorCode::TemporarilyUnavailable {
            ActivityError::application(ApplicationFailure::new(message))
        } else {
            ActivityError::application(ApplicationFailure::non_retryable(message))
        }
    }
    #[activities]
    impl BridgeActivities {
        #[activity]
        pub async fn drain(
            self: Arc<Self>,
            ctx: ActivityContext,
            input: WorkflowInput,
        ) -> std::result::Result<Progress, ActivityError> {
            if ctx.is_cancelled() {
                return Err(ActivityError::cancelled());
            }
            self.bridge
                .observe_execution(
                    &self.read_bearer,
                    &input.reference,
                    &ctx.client(),
                    ctx.info().workflow_run_id.as_deref(),
                )
                .await
                .map_err(activity_error)?;
            let initial = self
                .bridge
                .progress(&self.read_bearer, &input.reference)
                .await
                .map_err(activity_error)?;
            if input.cursor > initial.cursor {
                return Err(activity_error(err(ErrorCode::StaleRevision)));
            }
            // Bounded activity; a later durable timer continues draining. Each application
            // is independently atomic and repeatable after an Activity reply is lost.
            for _ in 0..8 {
                if ctx.is_cancelled() {
                    return Err(ActivityError::cancelled());
                }
                let Some((sequence, principal)) = self
                    .bridge
                    .next(&self.read_bearer, &input.reference)
                    .await
                    .map_err(activity_error)?
                else {
                    break;
                };
                let bearer = self.credentials.get(&principal).map(String::as_str);
                let receipt = self
                    .bridge
                    .apply_with_capability(&self.read_bearer, bearer, &input.reference, sequence)
                    .await
                    .map_err(activity_error)?;
                ctx.record_heartbeat((input.reference.clone(), receipt.sequence))
                    .await?;
            }
            let mut progress = self
                .bridge
                .progress(&self.read_bearer, &input.reference)
                .await
                .map_err(activity_error)?;
            for admission in &progress.waits {
                if ctx.is_cancelled() {
                    return Err(ActivityError::cancelled());
                }
                let observed = if progress.closed {
                    OperationProgress {
                        admission_id: Some(*admission),
                        episode_id: None,
                        journals: Vec::new(),
                        limitation: Some("workflow_closed".into()),
                    }
                } else {
                    match &self.driver {
                        Some(driver) => driver
                            .drive(&input.reference, *admission)
                            .await
                            .map_err(activity_error)?,
                        None => OperationProgress {
                            admission_id: Some(*admission),
                            episode_id: None,
                            journals: Vec::new(),
                            limitation: Some("operation_driver_unconfigured".into()),
                        },
                    }
                };
                progress.operations.push(observed);
                ctx.record_heartbeat((input.reference.clone(), *admission))
                    .await?;
            }
            Ok(progress)
        }
    }
    fn activity_options() -> ActivityOptions {
        ActivityOptions::with_start_to_close_timeout(Duration::from_secs(10))
            .heartbeat_timeout(Duration::from_secs(3))
            .retry_policy(
                temporalio_common::RetryPolicy::builder()
                    .initial_interval(Duration::from_millis(100))
                    .maximum_attempts(3)
                    .build(),
            )
            .build()
    }
    #[workflow]
    #[derive(Default)]
    pub struct MissionWorkflow {
        hint: bool,
    }
    #[workflow_methods]
    impl MissionWorkflow {
        #[run]
        pub async fn run(
            ctx: &mut WorkflowContext<Self>,
            mut input: WorkflowInput,
        ) -> WorkflowResult<Progress> {
            if input.reference.kind != AggregateKind::Mission {
                return Err(ApplicationFailure::non_retryable("wrong aggregate kind").into());
            }
            for _ in 0..4 {
                let progress = ctx
                    .execute_activity(BridgeActivities::drain, input.clone(), activity_options())
                    .await?;
                input.cursor = progress.cursor;
                input.waits = progress.waits.clone();
                if progress.closed {
                    return Ok(progress);
                }
                ctx.state_mut(|s| s.hint = false);
                temporalio_sdk::workflows::select! { _ = ctx.timer(Duration::from_millis(500))=>{}, _ = ctx.wait_condition(|s|s.hint)=>{} }
            }
            ctx.continue_as_new(input, ContinueAsNewOptions::default())?;
            unreachable!("continue-as-new exits workflow")
        }
        #[signal]
        pub fn wake(&mut self, _ctx: &mut SyncWorkflowContext<Self>, _sequence: i64) {
            self.hint = true;
        }
    }
    #[workflow]
    #[derive(Default)]
    pub struct TaskWorkflow {
        hint: bool,
    }
    #[workflow_methods]
    impl TaskWorkflow {
        #[run]
        pub async fn run(
            ctx: &mut WorkflowContext<Self>,
            mut input: WorkflowInput,
        ) -> WorkflowResult<Progress> {
            if input.reference.kind != AggregateKind::Task {
                return Err(ApplicationFailure::non_retryable("wrong aggregate kind").into());
            }
            for _ in 0..4 {
                let progress = ctx
                    .execute_activity(BridgeActivities::drain, input.clone(), activity_options())
                    .await?;
                input.cursor = progress.cursor;
                input.waits = progress.waits.clone();
                if progress.closed {
                    return Ok(progress);
                }
                ctx.state_mut(|s| s.hint = false);
                temporalio_sdk::workflows::select! { _ = ctx.timer(Duration::from_millis(500))=>{}, _ = ctx.wait_condition(|s|s.hint)=>{} }
            }
            ctx.continue_as_new(input, ContinueAsNewOptions::default())?;
            unreachable!("continue-as-new exits workflow")
        }
        #[signal]
        pub fn wake(&mut self, _ctx: &mut SyncWorkflowContext<Self>, _sequence: i64) {
            self.hint = true;
        }
    }
}

impl Bridge {
    /// Deliver one durable wakeup. RPC failure leaves it pending. Stable workflow ID
    /// and RejectDuplicate prevent starting a second execution after a lost ack.
    /// Completed orchestration is explicitly rejected, never implicitly restarted.
    pub async fn deliver(
        &self,
        bearer: &str,
        client: &temporalio_client::Client,
        queue: &str,
        wakeup: &Wakeup,
    ) -> Result<()> {
        self.authorize_scheduler(bearer, &wakeup.reference).await?;
        let p = self.progress(bearer, &wakeup.reference).await?;
        if p.closed {
            if p.observed.is_some() {
                self.observe_execution(bearer, &wakeup.reference, client, None)
                    .await?;
            }

            self.acknowledge_wakeup(bearer, &wakeup.reference, wakeup.id)
                .await?;
            return Err(err(ErrorCode::StaleGeneration));
        }
        let input = WorkflowInput {
            reference: wakeup.reference.clone(),
            cursor: p.cursor,
            waits: p.waits,
        };
        let options =
            temporalio_client::WorkflowStartOptions::new(queue, &wakeup.reference.workflow_id)
                .id_reuse_policy(temporalio_client::WorkflowIdReusePolicy::RejectDuplicate)
                .id_conflict_policy(temporalio_client::WorkflowIdConflictPolicy::UseExisting)
                .build();
        let signal = temporalio_client::WorkflowSignalOptions::default();
        match wakeup.reference.kind {
            AggregateKind::Mission => {
                let handle = client
                    .start_workflow(MissionWorkflow::run, input, options)
                    .await
                    .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
                handle
                    .signal(MissionWorkflow::wake, wakeup.sequence, signal)
                    .await
                    .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
            }
            AggregateKind::Task => {
                let handle = client
                    .start_workflow(TaskWorkflow::run, input, options)
                    .await
                    .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
                handle
                    .signal(TaskWorkflow::wake, wakeup.sequence, signal)
                    .await
                    .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
            }
        }
        self.observe_execution(bearer, &wakeup.reference, client, None)
            .await?;
        self.acknowledge_wakeup(bearer, &wakeup.reference, wakeup.id)
            .await
    }
}

/// Small gateway milestones only; task state remains canonical PostgreSQL state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationProgress {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<Uuid>,
    pub journals: Vec<(Uuid, crate::gateway::OperationState)>,
    pub limitation: Option<String>,
}
/// Activity-side extension seam for assembled proof16. Implementations own adapter
/// binding and transport control; workflow code never sees a runtime or credential.
pub trait OperationDriver: Send + Sync {
    fn drive<'a>(
        &'a self,
        reference: &'a WorkflowRef,
        admission_id: Uuid,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<OperationProgress>> + Send + 'a>>;
}
impl Bridge {
    /// Stable admission operation -> exact prepared gateway journals, scoped and fenced.
    pub async fn operation_journal(
        &self,
        bearer: &str,
        r: &WorkflowRef,
        admission_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.check(&tx, bearer, r).await?;
        if tx
            .query_one(
                "SELECT closed FROM workflow_registry WHERE workflow_id=$1",
                &[&r.workflow_id],
            )
            .await?
            .get::<_, bool>(0)
        {
            return Err(err(ErrorCode::StaleGeneration));
        }

        let row=tx.query_opt("SELECT a.task_id,t.mission_id FROM operations o JOIN attempts a ON a.id=o.attempt_id JOIN tasks t ON t.id=a.task_id WHERE o.id=$1",&[&admission_id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
        let expected = match r.kind {
            AggregateKind::Task => row.get::<_, Uuid>(0),
            AggregateKind::Mission => row.get::<_, Uuid>(1),
        };
        if expected != r.aggregate_id {
            return Err(err(ErrorCode::Unauthorized));
        }
        let ids=tx.query("SELECT id FROM gateway_operations WHERE admission_id=$1 ORDER BY CASE kind WHEN 'create' THEN 0 WHEN 'recover' THEN 1 WHEN 'start' THEN 2 WHEN 'steer' THEN 3 ELSE 4 END,id",&[&admission_id]).await?.iter().map(|v|v.get(0)).collect();
        tx.commit().await?;
        Ok(ids)
    }
}
/// Usable driver for an explicitly prepared provider binding. It never invents a
/// Create/Start intent and never re-dispatches uncertain journals. Kernel16 prepares
/// bindings/intents using Gateway's public commands, then registers this driver.
pub struct GatewayOperationDriver {
    bridge: Bridge,
    gateway: crate::gateway::Gateway,
    bearer: String,
    runtime: std::sync::Arc<tokio::sync::Mutex<Box<dyn crate::gateway::RuntimeAdapter>>>,
    queued: std::sync::atomic::AtomicUsize,
}
impl std::fmt::Debug for GatewayOperationDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayOperationDriver")
            .field("bridge", &self.bridge)
            .field("gateway", &self.gateway)
            .finish_non_exhaustive()
    }
}
impl GatewayOperationDriver {
    pub fn new(
        bridge: Bridge,
        gateway: crate::gateway::Gateway,
        bearer: String,
        runtime: Box<dyn crate::gateway::RuntimeAdapter>,
    ) -> Self {
        Self {
            bridge,
            gateway,
            bearer,
            runtime: std::sync::Arc::new(tokio::sync::Mutex::new(runtime)),
            queued: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}
impl GatewayOperationDriver {
    /// Trusted runtime binding can share its transport gate with the local supervisor.
    pub fn with_shared_runtime(
        bridge: Bridge,
        gateway: crate::gateway::Gateway,
        bearer: String,
        runtime: std::sync::Arc<tokio::sync::Mutex<Box<dyn crate::gateway::RuntimeAdapter>>>,
    ) -> Self {
        Self {
            bridge,
            gateway,
            bearer,
            runtime,
            queued: std::sync::atomic::AtomicUsize::new(0),
        }
    }
    /// Local queue observability; contains no credential or runtime identity.
    pub fn queued(&self) -> usize {
        self.queued.load(std::sync::atomic::Ordering::SeqCst)
    }
}
struct RuntimeWaiter<'a>(&'a std::sync::atomic::AtomicUsize);
impl Drop for RuntimeWaiter<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}
impl OperationDriver for GatewayOperationDriver {
    fn drive<'a>(
        &'a self,
        r: &'a WorkflowRef,
        admission: Uuid,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<OperationProgress>> + Send + 'a>>
    {
        Box::pin(async move {
            let mut runtime = {
                self.queued
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let _waiting = RuntimeWaiter(&self.queued);
                self.runtime.lock().await
            };
            let ids = self
                .bridge
                .operation_journal(&self.bearer, r, admission)
                .await?;
            let mut progress = OperationProgress {
                admission_id: Some(admission),
                episode_id: None,
                journals: Vec::new(),
                limitation: if ids.is_empty() {
                    Some("gateway_binding_or_intent_missing".into())
                } else {
                    None
                },
            };
            for id in ids {
                let op = match self
                    .gateway
                    .dispatch_guarded(&self.bearer, id, runtime.as_mut(), Some(r))
                    .await
                {
                    Ok(op) => op,
                    Err(error) if error.code == ErrorCode::DependencyUnmet => {
                        // A terminal Task can revoke fresh effects while its
                        // current workflow still observes retained journals.
                        let op = self.gateway.read_guarded(&self.bearer, id, Some(r)).await?;
                        if op.state != crate::gateway::OperationState::Prepared {
                            return Err(error);
                        }
                        progress.limitation = Some("gateway_dispatch_fenced".into());
                        op
                    }
                    Err(error) => return Err(error),
                };
                progress.journals.push((id, op.state));
            }
            Ok(progress)
        })
    }
}

/// Called only while the gateway holds the broker's PG dispatch-claim lock.
/// Closing/replacing a workflow before this claim prevents physical dispatch;
/// closing after a committed claim does not revoke an already in-flight RPC.
pub(crate) async fn dispatch_guard(
    tx: &Transaction<'_>,
    epoch: Uuid,
    r: &WorkflowRef,
    admission: Uuid,
) -> Result<()> {
    workflow_dispatch_guard(tx, epoch, r).await?;
    let row=tx.query_opt("SELECT a.task_id,t.mission_id FROM operations o JOIN attempts a ON a.id=o.attempt_id JOIN tasks t ON t.id=a.task_id WHERE o.id=$1",&[&admission]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
    let aggregate = match r.kind {
        AggregateKind::Task => row.get::<_, Uuid>(0),
        AggregateKind::Mission => row.get::<_, Uuid>(1),
    };
    if aggregate != r.aggregate_id {
        return Err(err(ErrorCode::Unauthorized));
    }
    Ok(())
}
pub(crate) async fn episode_dispatch_guard(
    tx: &Transaction<'_>,
    epoch: Uuid,
    r: &WorkflowRef,
    mission: Uuid,
    task: Option<Uuid>,
) -> Result<()> {
    workflow_dispatch_guard(tx, epoch, r).await?;
    let expected = match r.kind {
        AggregateKind::Mission if task.is_none() => mission,
        AggregateKind::Task => task.ok_or_else(|| err(ErrorCode::Unauthorized))?,
        _ => return Err(err(ErrorCode::Unauthorized)),
    };
    if expected != r.aggregate_id {
        return Err(err(ErrorCode::Unauthorized));
    }
    Ok(())
}
async fn workflow_dispatch_guard(tx: &Transaction<'_>, epoch: Uuid, r: &WorkflowRef) -> Result<()> {
    storage::active(tx, epoch).await?;
    if r.recovery_epoch != epoch
        || r.workflow_id != workflow_id(&r.kind, r.aggregate_id, r.generation, epoch)
    {
        return Err(err(ErrorCode::Unauthorized));
    }
    let row=tx.query_opt("SELECT generation,recovery_epoch,workflow_id,closed FROM workflow_registry WHERE aggregate_kind=$1 AND aggregate_id=$2 ORDER BY generation DESC LIMIT 1 FOR UPDATE",&[&r.kind.name(),&r.aggregate_id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
    if row.get::<_, i64>(0) != r.generation
        || row.get::<_, Uuid>(1) != epoch
        || row.get::<_, String>(2) != r.workflow_id
        || row.get::<_, bool>(3)
    {
        return Err(err(ErrorCode::StaleGeneration));
    }
    Ok(())
}
impl Bridge {
    async fn scheduler(&self, tx: &Transaction<'_>, bearer: &str, r: &WorkflowRef) -> Result<()> {
        let actor = self.check(tx, bearer, r).await?;
        if !matches!(actor.role, Role::Owner | Role::Lead) {
            return Err(err(ErrorCode::Unauthorized));
        }
        Ok(())
    }
    async fn authorize_scheduler(&self, bearer: &str, r: &WorkflowRef) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.scheduler(&tx, bearer, r).await?;
        tx.commit().await?;
        Ok(())
    }
}
impl Bridge {
    /// Observe the actual Temporal execution, not a caller-asserted Run ID. Activity
    /// callers supply their SDK execution identity; an older run cannot overwrite a
    /// current incarnation. The authoritative parent chain must extend the last run.
    /// All RPC is Activity/service-side and credentials never enter workflow input.
    pub async fn observe_execution(
        &self,
        bearer: &str,
        r: &WorkflowRef,
        client: &temporalio_client::Client,
        expected_run: Option<&str>,
    ) -> Result<ObservedExecution> {
        use temporalio_client::{
            UntypedWorkflow, WorkflowDescribeOptions, WorkflowExecutionInfo,
            WorkflowExecutionStatus as Status, WorkflowFetchHistoryOptions,
        };
        self.authorize_scheduler(bearer, r).await?;
        let handle = client.get_workflow_handle::<UntypedWorkflow>(&r.workflow_id);
        let description = handle
            .describe(WorkflowDescribeOptions::default())
            .await
            .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        let run_id = description.run_id().to_owned();
        if expected_run.is_some_and(|run| run != run_id) {
            return Err(err(ErrorCode::StaleGeneration));
        }
        let expected_type = match r.kind {
            AggregateKind::Mission => "MissionWorkflow",
            AggregateKind::Task => "TaskWorkflow",
        };
        if description.id() != r.workflow_id
            || description.workflow_type() != expected_type
            || run_id.is_empty()
            || run_id.len() > 256
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let lifecycle = match description.status() {
            Status::Running => WorkflowLifecycle::Running,
            Status::Completed => WorkflowLifecycle::Completed,
            Status::Failed => WorkflowLifecycle::Failed,
            Status::Canceled => WorkflowLifecycle::Canceled,
            Status::Terminated => WorkflowLifecycle::Terminated,
            Status::ContinuedAsNew => WorkflowLifecycle::ContinuedAsNew,
            Status::TimedOut => WorkflowLifecycle::TimedOut,
            Status::Paused => WorkflowLifecycle::Paused,
            _ => return Err(err(ErrorCode::IncompatibleSchema)),
        };
        let bytes = WorkflowExecutionInfo::builder()
            .namespace(&client.options().namespace)
            .workflow_id(&r.workflow_id)
            .run_id(&run_id)
            .build()
            .bind_untyped(client.clone())
            .fetch_history(WorkflowFetchHistoryOptions::default())
            .to_json()
            .await
            .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        let history: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        let previous=history["events"][0]["attributes"]["WorkflowExecutionStartedEventAttributes"]["continued_execution_run_id"].as_str().ok_or_else(||err(ErrorCode::TemporarilyUnavailable))?;
        let previous = if previous.is_empty() {
            None
        } else {
            Some(previous.to_owned())
        };
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.scheduler(&tx, bearer, r).await?;
        let row=tx.query_one("SELECT observed_run_id,incarnation,lifecycle FROM workflow_registry WHERE workflow_id=$1 FOR UPDATE",&[&r.workflow_id]).await?;
        let current: Option<String> = row.get(0);
        let mut incarnation: i64 = row.get(1);
        let current_lifecycle: String = row.get(2);
        if current.as_deref() == Some(&run_id) {
            if !matches!(current_lifecycle.as_str(), "running" | "paused")
                && matches!(
                    lifecycle,
                    WorkflowLifecycle::Running | WorkflowLifecycle::Paused
                )
            {
                return Err(err(ErrorCode::StaleGeneration));
            }
        } else {
            if previous!=current || tx.query_opt("SELECT incarnation FROM workflow_incarnations WHERE workflow_id=$1 AND run_id=$2",&[&r.workflow_id,&run_id]).await?.is_some(){return Err(err(ErrorCode::StaleGeneration));}
            incarnation = incarnation
                .checked_add(1)
                .ok_or_else(|| err(ErrorCode::InvalidArgument))?;
            tx.execute("INSERT INTO workflow_incarnations(workflow_id,run_id,previous_run_id,incarnation) VALUES($1,$2,$3,$4)",&[&r.workflow_id,&run_id,&previous,&incarnation]).await?;
        }
        let name =
            serde_json::to_value(&lifecycle).map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        let name = name
            .as_str()
            .ok_or_else(|| err(ErrorCode::TemporarilyUnavailable))?;
        tx.execute("UPDATE workflow_registry SET observed_run_id=$2,incarnation=$3,lifecycle=$4 WHERE workflow_id=$1",&[&r.workflow_id,&run_id,&incarnation,&name]).await?;
        tx.commit().await?;
        Ok(ObservedExecution {
            run_id,
            incarnation,
            lifecycle,
        })
    }
}
