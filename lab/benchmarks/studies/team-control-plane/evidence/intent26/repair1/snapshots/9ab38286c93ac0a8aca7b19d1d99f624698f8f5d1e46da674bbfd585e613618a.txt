//! Canonical episode runtime journals; agent authorization stays separate from service control.
use crate::gateway::{OperationState, RuntimeIdentity};
use crate::{Broker, ErrorCode, Result, auth, domain::err, storage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartRequest {
    pub id: Uuid,
    pub episode: Uuid,
    pub execution: Uuid,
    pub runtime: RuntimeIdentity,
    pub argv: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeOperation {
    pub id: Uuid,
    pub episode: Uuid,
    pub execution: Uuid,
    pub state: OperationState,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub effect_id: Option<String>,
    pub usage: UsageObservation,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeOperationView {
    pub operation: EpisodeOperation,
    pub prepared: PreparedStart,
    pub evidence: Vec<serde_json::Value>,
    pub evidence_more: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedStart {
    pub start: StartRequest,
    pub input_packet: Uuid,
    pub input_digest: String,
    pub input: serde_json::Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativeTurn {
    pub thread: String,
    pub turn: String,
    pub effect: String,
}
/// Typed observations come only from the trusted local adapter, outside agent
/// commands/history. A native effect or namespace Stop is never a usage sample.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UsageObservation {
    Unknown,
    MeasuredZero { receipt_digest: String },
}
pub enum EpisodeReply {
    Confirmed(NativeTurn),
    ConfirmedMeasured(NativeTurn, UsageObservation),
    FailedKnown(String),
    Unknown(String),
}
pub enum EpisodeObservation {
    Effect(NativeTurn),
    EffectMeasured(NativeTurn, UsageObservation),
    Inconclusive(String),
}
#[derive(Clone, Debug)]
pub struct ClosedWorldOs {
    pub workspace_root: std::path::PathBuf,
    pub native_root: std::path::PathBuf,
}
pub trait EpisodeRuntime: Send {
    /// Trusted code capability, never deserialized from agent/history input.
    fn closed_world_os(&self) -> Option<ClosedWorldOs> {
        None
    }

    fn identity(&self) -> RuntimeIdentity;
    fn capabilities(&self) -> crate::gateway::CapabilityProfile;
    fn start<'a>(
        &'a mut self,
        request: &'a PreparedStart,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EpisodeReply> + Send + 'a>>;
    fn observe<'a>(
        &'a mut self,
        request: &'a PreparedStart,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EpisodeObservation> + Send + 'a>>;
}
#[derive(Clone)]
pub struct EpisodeGateway {
    broker: Broker,
    controller: Uuid,
    queue: std::sync::Arc<tokio::sync::Mutex<()>>,
}
impl std::fmt::Debug for EpisodeGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EpisodeGateway")
            .field("controller", &self.controller)
            .finish_non_exhaustive()
    }
}
impl EpisodeGateway {
    pub fn new(broker: Broker) -> Self {
        Self {
            broker,
            controller: Uuid::new_v4(),
            queue: Default::default(),
        }
    }
    pub fn controller(&self) -> Uuid {
        self.controller
    }
    /// Register the actual service incarnation before it can perform an effect.
    /// A replacement registration does not acquire an already-owned store.
    pub async fn register_runtime(
        &self,
        service_bearer: &str,
        runtime: &dyn EpisodeRuntime,
    ) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        auth::authenticate(&tx, service_bearer, self.broker.epoch)
            .await?
            .owner()?;
        let contract = runtime
            .closed_world_os()
            .ok_or_else(|| err(ErrorCode::DependencyUnmet))?;
        if !runtime.capabilities().authoritative_effect_inventory {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        crate::model_free_runtime::register_transport(
            &tx,
            self.controller,
            self.broker.epoch,
            runtime.identity(),
            contract,
            "inprocess-supervisor-os-v1",
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
    /// Independent Owner query of an existing effect. This neither adopts the
    /// transport nor sends a Start; missing evidence keeps its reservation held.
    pub async fn reconcile(&self, bearer: &str, id: Uuid) -> Result<EpisodeOperation> {
        use std::os::unix::fs::MetadataExt;
        let rejected = || err(ErrorCode::DependencyUnmet);
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        actor.owner()?;
        let operation = load(&tx, id).await?;
        crate::supervisor::authorize(&tx, &self.broker, bearer, operation.execution, false).await?;
        let unresolved = matches!(
            operation.state,
            OperationState::Dispatching | OperationState::OutcomeUnknown
        ) || (operation.state == OperationState::Confirmed
            && operation.usage == UsageObservation::Unknown);
        if !unresolved {
            tx.commit().await?;
            return Ok(operation);
        }
        let row=tx.query_one("SELECT request,controller,store,recovery_epoch FROM episode_runtime_operations WHERE id=$1", &[&id]).await?;
        let prepared: PreparedStart =
            serde_json::from_value(row.get(0)).map_err(|_| err(ErrorCode::IncompatibleSchema))?;
        let controller: Uuid = row.get::<_, Option<Uuid>>(1).ok_or_else(rejected)?;
        let store: String = row.get::<_, Option<String>>(2).ok_or_else(rejected)?;
        if row.get::<_, Uuid>(3) != self.broker.epoch {
            return Err(rejected());
        }
        let transport = crate::model_free_runtime::transport(&tx, controller).await?;
        let control=tx.query_one("SELECT controller,busy,quarantined,runtime_identity FROM gateway_stores WHERE identity=$1 FOR UPDATE", &[&store]).await?;
        if control.get::<_, Uuid>(0) != controller
            || (if operation.state == OperationState::Confirmed {
                control.get::<_, Option<Uuid>>(1).is_some()
            } else {
                control.get::<_, Option<Uuid>>(1) != Some(id)
            })
            || control.get::<_, bool>(2)
            || control.get::<_, serde_json::Value>(3) != serde_json::json!(transport.identity)
            || transport.identity != prepared.start.runtime
            || transport.store != store
            || transport.epoch != self.broker.epoch
            || !crate::model_free_runtime::exclusive_live_transport(
                &transport.process,
                transport.group,
            )
            .map_err(|_| rejected())?
        {
            return Err(rejected());
        }
        let directory = crate::model_free_runtime::open_directory(&transport.native_root)?;
        let meta = directory.metadata().map_err(|_| rejected())?;
        if meta.dev() != transport.native_directory.device
            || meta.ino() != transport.native_directory.inode
            || meta.uid() != transport.native_directory.owner
            || crate::model_free_runtime::directory_identity(&transport.workspace_root)?
                != transport.workspace_directory
        {
            return Err(rejected());
        }
        let observation = crate::model_free_runtime::observe_directory(
            &directory,
            &transport.identity,
            &prepared,
        );
        if !crate::model_free_runtime::exclusive_live_transport(&transport.process, transport.group)
            .map_err(|_| rejected())?
            || crate::model_free_runtime::directory_identity(&transport.native_root)?
                != transport.native_directory
            || crate::model_free_runtime::directory_identity(&transport.workspace_root)?
                != transport.workspace_directory
        {
            return Err(rejected());
        }
        let (observation, usage) = match observation {
            EpisodeObservation::EffectMeasured(turn, usage) => {
                (EpisodeObservation::Effect(turn), usage)
            }
            other => (other, UsageObservation::Unknown),
        };
        let kind = match observation {
            EpisodeObservation::Effect(turn) => {
                if !confirm_native(&tx, &prepared, &store, &turn, self.broker.epoch).await? {
                    return Err(rejected());
                }
                record_usage(&tx, &prepared, &usage, self.broker.epoch).await?;
                tx.execute(
                    "UPDATE episode_runtime_operations SET state='confirmed' WHERE id=$1",
                    &[&id],
                )
                .await?;
                tx.execute(
                    "UPDATE gateway_stores SET busy=NULL WHERE identity=$1 AND busy=$2",
                    &[&store, &id],
                )
                .await?;
                "owner_observed_existing_native_effect"
            }
            EpisodeObservation::Inconclusive(_) => "owner_native_observation_inconclusive",
            EpisodeObservation::EffectMeasured(..) => unreachable!("measurement normalized"),
        };
        tx.execute("INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&Uuid::new_v4(),&id,&serde_json::json!({"kind":kind,"by":actor.principal,"controller":controller,"native_directory":transport.native_directory})]).await?;
        let result = load(&tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }
    /// Exact authenticated lookup remains available beyond the bounded status page.
    /// Current Owner can inspect historical executions without adopting their controller.
    pub async fn inspect(&self, bearer: &str, id: Uuid) -> Result<EpisodeOperationView> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        let operation = load(&tx, id).await?;
        crate::supervisor::authorize(&tx, &self.broker, bearer, operation.execution, false).await?;
        let prepared: PreparedStart = serde_json::from_value(
            tx.query_one(
                "SELECT request FROM episode_runtime_operations WHERE id=$1",
                &[&id],
            )
            .await?
            .get(0),
        )
        .map_err(|_| err(ErrorCode::IncompatibleSchema))?;
        let rows=tx.query("SELECT evidence FROM episode_runtime_evidence WHERE operation_id=$1 ORDER BY id LIMIT 33", &[&id]).await?;
        let evidence_more = rows.len() > 32;
        let evidence = rows.into_iter().take(32).map(|r| r.get(0)).collect();
        tx.commit().await?;
        Ok(EpisodeOperationView {
            operation,
            prepared,
            evidence,
            evidence_more,
        })
    }
    async fn journal(
        &self,
        service_bearer: &str,
        workflow: &crate::temporal::WorkflowRef,
        episode: Uuid,
    ) -> Result<Option<EpisodeOperation>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let service = auth::authenticate(&tx, service_bearer, self.broker.epoch).await?;
        service.owner()?;
        crate::temporal::workflow_dispatch_guard(&tx, self.broker.epoch, workflow).await?;
        match workflow.kind {
            crate::temporal::AggregateKind::Mission => {
                service.scope(workflow.aggregate_id, None)?
            }
            crate::temporal::AggregateKind::Task => {
                let mission: Uuid = tx
                    .query_one(
                        "SELECT mission_id FROM tasks WHERE id=$1",
                        &[&workflow.aggregate_id],
                    )
                    .await?
                    .get(0);
                service.scope(mission, Some(workflow.aggregate_id))?;
            }
        }

        let Some(row)=tx.query_opt("SELECT o.id,x.mission_id,x.task_id FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE o.episode_id=$1", &[&episode]).await? else {tx.commit().await?;return Ok(None);};
        let mission: Uuid = row.get(1);
        let task: Option<Uuid> = row.get(2);
        service.scope(mission, task)?;
        crate::temporal::episode_dispatch_guard(&tx, self.broker.epoch, workflow, mission, task)
            .await?;
        let operation = load(&tx, row.get(0)).await?;
        tx.commit().await?;
        Ok(Some(operation))
    }
    /// Immutable current packet and physical binding are committed before any RPC.
    pub async fn prepare(&self, bearer: &str, request: StartRequest) -> Result<EpisodeOperation> {
        request.runtime.key()?;
        if request.argv.is_empty()
            || request.argv.len() > 1024
            || request.argv.iter().any(|s| s.contains('\0'))
            || serde_json::to_vec(&request)
                .map_err(|_| err(ErrorCode::InvalidArgument))?
                .len()
                > 65536
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        if !matches!(
            actor.role,
            crate::Role::Lead | crate::Role::Worker | crate::Role::Verifier
        ) {
            return Err(err(ErrorCode::Unauthorized));
        }
        let view = crate::supervisor::authorize(&tx, &self.broker, bearer, request.execution, true)
            .await?;
        let row=tx.query_opt("SELECT x.principal_id,x.role,x.credential_id,e.input_packet,e.input_digest,e.deadline>clock_timestamp(),e.state,x.mission_id,p.packet,x.attempt_id FROM episode_executions x JOIN mailbox_episodes e ON e.id=x.episode_id JOIN mailbox_inputs p ON p.id=e.input_packet WHERE x.execution_id=$1 AND x.episode_id=$2", &[&request.execution,&request.episode]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
        if actor.principal != row.get::<_, Uuid>(0)
            || serde_json::to_value(&actor.role).map_err(|_| err(ErrorCode::InvalidArgument))?
                != row.get::<_, String>(1)
        {
            return Err(err(ErrorCode::Unauthorized));
        }
        if !row.get::<_, bool>(5)
            || !matches!(row.get::<_, String>(6).as_str(), "admitted" | "active")
            || !crate::supervisor::execution_dispatch_current(
                &tx,
                request.execution,
                self.broker.epoch,
            )
            .await?
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let mission: Uuid = row.get(7);
        crate::kernel::dispatch_enabled(&tx, mission).await?;
        let prepared = serde_json::json!({"start":request,"input_packet":row.get::<_,Uuid>(3),"input_digest":row.get::<_,String>(4),"input":row.get::<_,serde_json::Value>(8)});
        let digest = crate::validation::digest(&prepared)?;
        if let Some(existing) = tx
            .query_opt(
                "SELECT id,digest FROM episode_runtime_operations WHERE id=$1 OR episode_id=$2",
                &[&request.id, &request.episode],
            )
            .await?
        {
            if existing.get::<_, Uuid>(0) != request.id || existing.get::<_, String>(1) != digest {
                return Err(err(ErrorCode::IdempotencyConflict));
            }
            let result = load(&tx, request.id).await?;
            tx.commit().await?;
            return Ok(result);
        }
        if view.state != crate::supervisor::ExecutionState::Ready {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let attempt: Option<Uuid> = row.get(9);
        if let Some(attempt) = attempt
            && crate::kernel::legacy_usage_unknown(&tx, attempt).await?
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let unknown:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_operations WHERE attempt_id=$1 AND state IN ('dispatching','outcome_unknown','quarantined')) OR EXISTS(SELECT 1 FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE (x.attempt_id=$1 OR x.principal_id=$2) AND (o.state IN ('dispatching','outcome_unknown','quarantined') OR (o.state IN ('confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero')))", &[&attempt,&actor.principal]).await?.get(0);
        if unknown {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        tx.execute("INSERT INTO episode_runtime_operations(id,episode_id,execution_id,principal_id,credential_id,recovery_epoch,digest,request,state) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'prepared')", &[&request.id,&request.episode,&request.execution,&actor.principal,&row.get::<_,Uuid>(2),&self.broker.epoch,&digest,&prepared]).await?;
        let result = load(&tx, request.id).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn dispatch(
        &self,
        service_bearer: &str,
        id: Uuid,
        runtime: &mut dyn EpisodeRuntime,
    ) -> Result<EpisodeOperation> {
        self.dispatch_guarded(service_bearer, id, runtime, None)
            .await
    }
    pub(crate) async fn dispatch_guarded(
        &self,
        service_bearer: &str,
        id: Uuid,
        runtime: &mut dyn EpisodeRuntime,
        workflow: Option<&crate::temporal::WorkflowRef>,
    ) -> Result<EpisodeOperation> {
        let _queue = self.queue.lock().await;
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let service = auth::authenticate(&tx, service_bearer, self.broker.epoch).await?;
        service.owner()?;
        let operation = load(&tx, id).await?;
        let prepared: PreparedStart = serde_json::from_value(
            tx.query_one(
                "SELECT request FROM episode_runtime_operations WHERE id=$1",
                &[&id],
            )
            .await?
            .get(0),
        )
        .map_err(|_| err(ErrorCode::IncompatibleSchema))?;
        let row=tx.query_one("SELECT x.mission_id,x.task_id,e.state,e.deadline>clock_timestamp(),e.input_packet,e.input_digest,p.packet,w.state,o.credential_id=x.credential_id,o.principal_id=x.principal_id,o.recovery_epoch,x.attempt_id FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id JOIN mailbox_episodes e ON e.id=x.episode_id JOIN mailbox_inputs p ON p.id=e.input_packet JOIN supervised_workspaces w ON w.id=x.execution_id WHERE o.id=$1", &[&id]).await?;
        let mission: Uuid = row.get(0);
        let task: Option<Uuid> = row.get(1);
        service.scope(mission, task)?;
        if let Some(workflow) = workflow {
            crate::temporal::episode_dispatch_guard(
                &tx,
                self.broker.epoch,
                workflow,
                mission,
                task,
            )
            .await?;
        }
        if matches!(
            operation.state,
            OperationState::Dispatching | OperationState::OutcomeUnknown
        ) {
            // Recovery observes the existing immutable request. No permission to
            // issue another Start follows from a lost reply or Activity retry.
            let identity = runtime.identity();
            if identity != prepared.start.runtime || !runtime.capabilities().observe {
                return Err(err(ErrorCode::DependencyUnmet));
            }
            let store = identity.key()?;
            let control=tx.query_opt("SELECT controller,busy,runtime_identity FROM gateway_stores WHERE identity=$1 FOR UPDATE", &[&store]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
            if control.get::<_, Uuid>(0) != self.controller
                || control.get::<_, Option<Uuid>>(1) != Some(id)
                || control.get::<_, serde_json::Value>(2) != serde_json::json!(identity)
            {
                return Err(err(ErrorCode::DependencyUnmet));
            }
            tx.commit().await?;
            let reply = match runtime.observe(&prepared).await {
                EpisodeObservation::Effect(turn) => EpisodeReply::Confirmed(turn),
                EpisodeObservation::EffectMeasured(turn, usage) => {
                    EpisodeReply::ConfirmedMeasured(turn, usage)
                }
                EpisodeObservation::Inconclusive(reason) => EpisodeReply::Unknown(reason),
            };
            return self.complete(&mut c, &prepared, &store, reply).await;
        }
        if operation.state != OperationState::Prepared {
            if let Some(workflow) = workflow {
                let identity = crate::supervisor::local_process_identity()
                    .map_err(|_| err(ErrorCode::DependencyUnmet))?;
                tx.execute("INSERT INTO kernel_transport_observations(controller,operation_id,evidence) SELECT $1,$2,$3 FROM kernel_transports t JOIN gateway_stores s ON s.identity=t.store WHERE t.controller=$1 AND s.controller=$1 AND t.process_identity=$4 AND t.recovery_epoch=$5 ON CONFLICT DO NOTHING", &[&self.controller,&id,&serde_json::json!({"kind":"sdk_observed_existing_operation","workflow":workflow,"state":operation.state,"thread":operation.thread_id,"turn":operation.turn_id,"effect":operation.effect_id}),&serde_json::json!(identity),&self.broker.epoch]).await?;
            }
            tx.commit().await?;
            return Ok(operation);
        }
        if row.get::<_, Uuid>(10) != self.broker.epoch
            || !row.get::<_, bool>(8)
            || !row.get::<_, bool>(9)
            || !crate::supervisor::execution_dispatch_current(
                &tx,
                operation.execution,
                self.broker.epoch,
            )
            .await?
        {
            return Err(err(ErrorCode::StaleGeneration));
        }
        if !row.get::<_, bool>(3)
            || !matches!(row.get::<_, String>(2).as_str(), "admitted" | "active")
            || row.get::<_, String>(7) != "ready"
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        if row.get::<_, Uuid>(4) != prepared.input_packet
            || row.get::<_, String>(5) != prepared.input_digest
            || row.get::<_, serde_json::Value>(6) != prepared.input
        {
            return Err(err(ErrorCode::StaleRevision));
        }
        crate::kernel::dispatch_enabled(&tx, mission).await?;
        let attempt: Option<Uuid> = row.get(11);
        if let Some(attempt) = attempt
            && crate::kernel::legacy_usage_unknown(&tx, attempt).await?
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let unknown:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_operations WHERE attempt_id=$1 AND state IN ('dispatching','outcome_unknown','quarantined')) OR EXISTS(SELECT 1 FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.attempt_id=$1 AND o.id<>$2 AND (o.state IN ('dispatching','outcome_unknown','quarantined') OR (o.state IN ('confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero')))", &[&attempt,&id]).await?.get(0);
        if unknown {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let identity = runtime.identity();
        let profile = runtime.capabilities();
        if identity != prepared.start.runtime || !profile.start {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let store = identity.key()?;
        // Shared with legacy Gateway: no per-module mutex can bypass this owner/busy claim.
        tx.execute("INSERT INTO gateway_stores(identity,controller,runtime_identity) VALUES($1,$2,$3) ON CONFLICT DO NOTHING", &[&store,&self.controller,&serde_json::json!(identity)]).await?;
        let control=tx.query_one("SELECT controller,runtime_identity,busy,quarantined FROM gateway_stores WHERE identity=$1 FOR UPDATE", &[&store]).await?;
        if control.get::<_, Uuid>(0) != self.controller
            || control.get::<_, serde_json::Value>(1) != serde_json::json!(identity)
            || control.get::<_, Option<Uuid>>(2).is_some()
            || control.get::<_, bool>(3)
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        tx.execute(
            "UPDATE gateway_stores SET busy=$2 WHERE identity=$1",
            &[&store, &id],
        )
        .await?;
        tx.execute("UPDATE episode_runtime_operations SET state='dispatching',controller=$2,store=$3 WHERE id=$1", &[&id,&self.controller,&store]).await?;
        tx.execute("INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&Uuid::new_v4(),&id,&serde_json::json!({"kind":"dispatch_claimed","controller":self.controller,"native_deduplication":profile.native_deduplication})]).await?;
        tx.commit().await?;
        let reply = runtime.start(&prepared).await;
        self.complete(&mut c, &prepared, &store, reply).await
    }
    async fn complete(
        &self,
        c: &mut tokio_postgres::Client,
        prepared: &PreparedStart,
        store: &str,
        reply: EpisodeReply,
    ) -> Result<EpisodeOperation> {
        let id = prepared.start.id;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        let current = tx
            .query_one(
                "SELECT controller,busy FROM gateway_stores WHERE identity=$1 FOR UPDATE",
                &[&store],
            )
            .await?;
        if current.get::<_, Uuid>(0) != self.controller
            || current.get::<_, Option<Uuid>>(1) != Some(id)
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let (reply, usage) = match reply {
            EpisodeReply::ConfirmedMeasured(turn, usage) => (EpisodeReply::Confirmed(turn), usage),
            other => (other, UsageObservation::Unknown),
        };
        let (state, evidence) = match reply {
            EpisodeReply::Confirmed(turn)
                if [&turn.thread, &turn.turn, &turn.effect]
                    .iter()
                    .all(|s| !s.is_empty() && s.len() <= 1024) =>
            {
                if confirm_native(&tx, prepared, store, &turn, self.broker.epoch).await? {
                    record_effect_binding(&tx, prepared, &turn, self.controller, self.broker.epoch)
                        .await?;
                    record_usage(&tx, prepared, &usage, self.broker.epoch).await?;
                    (
                        "confirmed",
                        serde_json::json!({"kind":"native_turn_confirmed","turn":turn}),
                    )
                } else {
                    (
                        "quarantined",
                        serde_json::json!({"kind":"native_thread_alias","observed":turn}),
                    )
                }
            }
            EpisodeReply::FailedKnown(_) => (
                "failed_known",
                serde_json::json!({"kind":"known_rejection"}),
            ),
            EpisodeReply::Unknown(_)
            | EpisodeReply::Confirmed(_)
            | EpisodeReply::ConfirmedMeasured(..) => (
                "outcome_unknown",
                serde_json::json!({"kind":"uncertain_effect"}),
            ),
        };
        tx.execute(
            "UPDATE episode_runtime_operations SET state=$2 WHERE id=$1",
            &[&id, &state],
        )
        .await?;
        tx.execute(
            "INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)",
            &[&Uuid::new_v4(), &id, &evidence],
        )
        .await?;
        if matches!(state, "confirmed" | "failed_known") {
            tx.execute(
                "UPDATE gateway_stores SET busy=NULL WHERE identity=$1 AND busy=$2",
                &[&store, &id],
            )
            .await?;
        }
        let operation = load(&tx, id).await?;
        tx.commit().await?;
        Ok(operation)
    }
}
/// Registered Activity driver; transports/service credentials never enter workflows.
pub struct EpisodeOperationDriver {
    gateway: EpisodeGateway,
    service_bearer: String,
    runtime: tokio::sync::Mutex<Box<dyn EpisodeRuntime>>,
    fallback: Option<std::sync::Arc<dyn crate::temporal::OperationDriver>>,
}
impl std::fmt::Debug for EpisodeOperationDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EpisodeOperationDriver")
            .field("gateway", &self.gateway)
            .finish_non_exhaustive()
    }
}
impl EpisodeOperationDriver {
    pub fn new(
        gateway: EpisodeGateway,
        service_bearer: String,
        runtime: Box<dyn EpisodeRuntime>,
        fallback: Option<std::sync::Arc<dyn crate::temporal::OperationDriver>>,
    ) -> Self {
        Self {
            gateway,
            service_bearer,
            runtime: tokio::sync::Mutex::new(runtime),
            fallback,
        }
    }
}
impl crate::temporal::OperationDriver for EpisodeOperationDriver {
    fn drive<'a>(
        &'a self,
        workflow: &'a crate::temporal::WorkflowRef,
        wait: crate::temporal::WaitRef,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<crate::temporal::OperationProgress>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let Some(episode) = wait.episode_id() else {
                return if let Some(fallback) = &self.fallback {
                    fallback.drive(workflow, wait).await
                } else {
                    Ok(crate::temporal::OperationProgress {
                        admission_id: wait.admission_id(),
                        episode_id: None,
                        journals: vec![],
                        limitation: Some("legacy_operation_driver_unconfigured".into()),
                    })
                };
            };
            let mut runtime = self.runtime.lock().await;
            let Some(journal) = self
                .gateway
                .journal(&self.service_bearer, workflow, episode)
                .await?
            else {
                return Ok(crate::temporal::OperationProgress {
                    admission_id: None,
                    episode_id: Some(episode),
                    journals: vec![],
                    limitation: Some("episode_journal_missing".into()),
                });
            };
            let mut limitation = None;
            let operation = match self
                .gateway
                .dispatch_guarded(
                    &self.service_bearer,
                    journal.id,
                    runtime.as_mut(),
                    Some(workflow),
                )
                .await
            {
                Ok(operation) => operation,
                Err(error)
                    if matches!(
                        error.code,
                        ErrorCode::DependencyUnmet | ErrorCode::StaleGeneration
                    ) =>
                {
                    // Execution revocation is separate from Workflow authority.
                    // Re-reading authenticates the current scoped WorkflowRef;
                    // superseded/closed/foreign references still fail here.
                    let current = self
                        .gateway
                        .journal(&self.service_bearer, workflow, episode)
                        .await?
                        .ok_or_else(|| err(ErrorCode::NotFound))?;
                    if current.state != OperationState::Prepared {
                        return Err(error);
                    }
                    limitation = Some("episode_dispatch_fenced".into());
                    current
                }
                Err(error) => return Err(error),
            };
            Ok(crate::temporal::OperationProgress {
                admission_id: None,
                episode_id: Some(episode),
                journals: vec![(operation.id, operation.state)],
                limitation,
            })
        })
    }
}
async fn load(tx: &tokio_postgres::Transaction<'_>, id: Uuid) -> Result<EpisodeOperation> {
    let row=tx.query_opt("SELECT id,episode_id,execution_id,state,thread_id,turn_id,effect_id,usage FROM episode_runtime_operations WHERE id=$1", &[&id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
    Ok(EpisodeOperation {
        id: row.get(0),
        episode: row.get(1),
        execution: row.get(2),
        state: serde_json::from_value(serde_json::Value::String(row.get(3)))
            .map_err(|_| err(ErrorCode::IncompatibleSchema))?,
        thread_id: row.get(4),
        turn_id: row.get(5),
        effect_id: row.get(6),
        usage: serde_json::from_value(row.get(7))
            .map_err(|_| err(ErrorCode::IncompatibleSchema))?,
    })
}

/// Effect correspondence is independent of usage certainty. A generic reply
/// cannot borrow a workspace Stop; only registered pinned native evidence can.
pub(crate) async fn record_effect_binding(
    tx: &tokio_postgres::Transaction<'_>,
    prepared: &PreparedStart,
    turn: &NativeTurn,
    controller: Uuid,
    epoch: Uuid,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    if !tx
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM kernel_transports WHERE controller=$1)",
            &[&controller],
        )
        .await?
        .get::<_, bool>(0)
    {
        return Ok(());
    }
    let transport = crate::model_free_runtime::transport(tx, controller).await?;
    if transport.epoch != epoch || transport.identity != prepared.start.runtime {
        return Ok(());
    }
    let directory = crate::model_free_runtime::open_directory(&transport.native_root)?;
    let metadata = directory
        .metadata()
        .map_err(|_| err(ErrorCode::DependencyUnmet))?;
    if metadata.dev() != transport.native_directory.device
        || metadata.ino() != transport.native_directory.inode
        || metadata.uid() != transport.native_directory.owner
    {
        return Ok(());
    }
    let observed = match crate::model_free_runtime::observe_directory(
        &directory,
        &transport.identity,
        prepared,
    ) {
        EpisodeObservation::Effect(native) | EpisodeObservation::EffectMeasured(native, _) => {
            native
        }
        EpisodeObservation::Inconclusive(_) => return Ok(()),
    };
    if observed.thread != turn.thread
        || observed.turn != turn.turn
        || observed.effect != turn.effect
    {
        return Ok(());
    }
    let row = tx
        .query_one(
            "SELECT init_process,manifest FROM supervised_workspaces WHERE id=$1",
            &[&prepared.start.execution],
        )
        .await?;
    let init: Option<serde_json::Value> = row.get(0);
    let manifest: crate::supervisor::WorkspaceManifest =
        serde_json::from_value(row.get(1)).map_err(|_| err(ErrorCode::IncompatibleSchema))?;
    if init.is_none()
        || manifest.episode != Some(prepared.start.episode)
        || manifest.recovery_epoch != epoch
        || manifest.path.parent() != Some(transport.workspace_root.as_path())
    {
        return Ok(());
    }
    tx.execute("INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&Uuid::new_v4(),&prepared.start.id,&serde_json::json!({"kind":"registered_native_execution_bound","request":prepared,"controller":controller,"recovery_epoch":epoch,"turn":turn,"init_process":init})]).await?;
    Ok(())
}

pub(crate) async fn record_usage(
    tx: &tokio_postgres::Transaction<'_>,
    prepared: &PreparedStart,
    usage: &UsageObservation,
    epoch: Uuid,
) -> Result<()> {
    let UsageObservation::MeasuredZero { receipt_digest } = usage else {
        return Ok(());
    };
    if receipt_digest.len() != 64
        || !receipt_digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    // A generic adapter's typed claim is insufficient: only a captured closed-world
    // transport and its independently read, exact pinned receipt can attest zero.
    let controller: Uuid = tx
        .query_one(
            "SELECT controller FROM episode_runtime_operations WHERE id=$1",
            &[&prepared.start.id],
        )
        .await?
        .get(0);
    if tx
        .query_opt(
            "SELECT controller FROM kernel_transports WHERE controller=$1",
            &[&controller],
        )
        .await?
        .is_none()
    {
        return Ok(());
    }
    let transport = crate::model_free_runtime::transport(tx, controller).await?;
    if transport.epoch != epoch || transport.identity != prepared.start.runtime {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let directory = crate::model_free_runtime::open_directory(&transport.native_root)?;
    use std::os::unix::fs::MetadataExt;
    let metadata = directory
        .metadata()
        .map_err(|_| err(ErrorCode::DependencyUnmet))?;
    if metadata.dev() != transport.native_directory.device
        || metadata.ino() != transport.native_directory.inode
        || metadata.uid() != transport.native_directory.owner
    {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let EpisodeObservation::EffectMeasured(turn, observed) =
        crate::model_free_runtime::observe_directory(&directory, &transport.identity, prepared)
    else {
        return Err(err(ErrorCode::DependencyUnmet));
    };
    if observed != *usage {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let native = tx
        .query_one(
            "SELECT thread_id,turn_id,effect_id FROM episode_runtime_operations WHERE id=$1",
            &[&prepared.start.id],
        )
        .await?;
    if native.get::<_, Option<String>>(0).as_deref() != Some(&turn.thread)
        || native.get::<_, Option<String>>(1).as_deref() != Some(&turn.turn)
        || native.get::<_, Option<String>>(2).as_deref() != Some(&turn.effect)
    {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let row=tx.query_one("SELECT usage,principal_id,credential_id,recovery_epoch FROM episode_runtime_operations WHERE id=$1", &[&prepared.start.id]).await?;
    if row.get::<_, Uuid>(3) != epoch {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let value = serde_json::json!(usage);
    let prior: UsageObservation =
        serde_json::from_value(row.get(0)).map_err(|_| err(ErrorCode::IncompatibleSchema))?;
    if prior != UsageObservation::Unknown {
        if prior != *usage {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        return Ok(());
    }
    tx.execute(
        "UPDATE episode_runtime_operations SET usage=$2 WHERE id=$1",
        &[&prepared.start.id, &value],
    )
    .await?;
    tx.execute("INSERT INTO kernel_usage_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&Uuid::new_v4(),&prepared.start.id,&serde_json::json!({"kind":"native_usage_observed","usage":usage,"runtime":prepared.start.runtime,"episode":prepared.start.episode,"execution":prepared.start.execution,"input_packet":prepared.input_packet,"input_digest":prepared.input_digest,"principal":row.get::<_,Uuid>(1),"credential":row.get::<_,Uuid>(2),"recovery_epoch":epoch})]).await?;
    Ok(())
}

pub(crate) async fn confirm_native(
    tx: &tokio_postgres::Transaction<'_>,
    prepared: &PreparedStart,
    store: &str,
    turn: &NativeTurn,
    epoch: Uuid,
) -> Result<bool> {
    let id = prepared.start.id;
    if let Some(existing)=tx.query_opt("SELECT operation_id,execution_id,thread_id,turn_id,input_packet,input_digest,recovery_epoch FROM episode_native_bindings WHERE episode_id=$1", &[&prepared.start.episode]).await? {
        return Ok(existing.get::<_,Uuid>(0)==id && existing.get::<_,Uuid>(1)==prepared.start.execution && existing.get::<_,String>(2)==turn.thread && existing.get::<_,String>(3)==turn.turn && existing.get::<_,Uuid>(4)==prepared.input_packet && existing.get::<_,String>(5)==prepared.input_digest && existing.get::<_,Uuid>(6)==epoch);
    }
    let alias:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_native_bindings WHERE store=$1 AND thread_id=$2) OR EXISTS(SELECT 1 FROM gateway_bindings WHERE store=$1 AND thread_id=$2)", &[&store,&turn.thread]).await?.get(0);
    if alias {
        return Ok(false);
    }
    tx.execute(
        "UPDATE episode_runtime_operations SET thread_id=$2,turn_id=$3,effect_id=$4 WHERE id=$1",
        &[&id, &turn.thread, &turn.turn, &turn.effect],
    )
    .await?;
    tx.execute("INSERT INTO episode_native_bindings(episode_id,operation_id,execution_id,store,thread_id,turn_id,input_packet,input_digest,recovery_epoch) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)", &[&prepared.start.episode,&id,&prepared.start.execution,&store,&turn.thread,&turn.turn,&prepared.input_packet,&prepared.input_digest,&epoch]).await?;
    tx.execute("UPDATE mailbox_episodes SET turn_id=$2,state=CASE WHEN state='admitted' THEN 'active' ELSE state END WHERE id=$1", &[&prepared.start.episode,&turn.turn]).await?;
    Ok(true)
}
