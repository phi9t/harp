//! Single-host, fail-closed runtime gateway. Persistent ownership never expires.
//! Database claims prevent repeated dispatch, but do not fence an external RPC.
use crate::{Broker, ErrorCode, Result, auth, domain::err, storage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Transaction;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIdentity {
    pub runtime_kind: String,
    pub runtime_version: String,
    pub deployment: String,
    pub thread_store: String,
    pub endpoint: String,
    pub environment: String,
}
impl RuntimeIdentity {
    fn key(&self) -> Result<String> {
        if [
            &self.runtime_kind,
            &self.runtime_version,
            &self.deployment,
            &self.thread_store,
            &self.endpoint,
            &self.environment,
        ]
        .iter()
        .any(|v| v.is_empty() || v.len() > 1024)
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        crate::validation::digest(&(&self.runtime_kind, &self.deployment, &self.thread_store))
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CapabilityProfile {
    pub create: bool,
    pub recover: bool,
    pub start: bool,
    pub observe: bool,
    pub steer: bool,
    pub cancel: bool,
    pub authoritative_effect_inventory: bool,
    /// Informational only: this gateway never retries a dispatched operation.
    pub native_deduplication: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Create,
    Recover,
    Start,
    Steer,
    Cancel,
}
impl OperationKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Recover => "recover",
            Self::Start => "start",
            Self::Steer => "steer",
            Self::Cancel => "cancel",
        }
    }
    fn supported(self, p: &CapabilityProfile) -> bool {
        match self {
            Self::Create => p.create,
            Self::Recover => p.recover,
            Self::Start => p.start,
            Self::Steer => p.steer,
            Self::Cancel => p.cancel,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Prepared,
    Dispatching,
    Confirmed,
    FailedKnown,
    OutcomeUnknown,
    Quarantined,
}
impl OperationState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Dispatching => "dispatching",
            Self::Confirmed => "confirmed",
            Self::FailedKnown => "failed_known",
            Self::OutcomeUnknown => "outcome_unknown",
            Self::Quarantined => "quarantined",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    pub id: Uuid,
    pub admission_id: Uuid,
    pub binding_id: Uuid,
    pub generation: i64,
    pub kind: OperationKind,
    pub arguments: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub intent: Intent,
    pub state: OperationState,
    pub thread_id: Option<String>,
    pub evidence: Vec<Evidence>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Evidence {
    IntentCommitted,
    DispatchClaimed {
        controller: Uuid,
    },
    ReplyConfirmed {
        thread_id: Option<String>,
        effect_id: String,
    },
    FailedKnown {
        reason: String,
    },
    OutcomeUnknown {
        reason: String,
        thread_id: Option<String>,
    },
    Quarantined {
        reason: String,
    },
    AuthoritativeEffect {
        effect_id: String,
        thread_id: Option<String>,
    },
    AuthoritativeAbsence {
        inventory_id: String,
    },
    Limitation {
        capability: String,
    },
}
#[derive(Clone, Debug)]
pub enum RuntimeReply {
    Confirmed {
        thread_id: Option<String>,
        effect_id: String,
    },
    FailedKnown(String),
    Unknown(String),
}
#[derive(Clone, Debug)]
pub enum Observation {
    Effect {
        thread_id: Option<String>,
        effect_id: String,
    },
    AuthoritativeAbsence {
        inventory_id: String,
    },
    Inconclusive(String),
}
/// Implementations must inspect exact runtime/store identity, not a thread ID alone.
/// Runtime adapters are trusted gateway-side components, never worker credentials.
pub trait RuntimeAdapter: Send {
    fn identity(&self) -> RuntimeIdentity;
    fn inspect(&self) -> CapabilityProfile;
    fn execute(&mut self, intent: &Intent, thread: Option<&str>) -> RuntimeReply;
    fn observe(&mut self, intent: &Intent, thread: Option<&str>) -> Observation;
}
#[derive(Clone, Debug)]
pub struct Gateway {
    broker: Broker,
    controller: Uuid,
    queue: std::sync::Arc<tokio::sync::Mutex<()>>,
}
impl Gateway {
    pub fn new(broker: Broker) -> Self {
        Self {
            broker,
            controller: Uuid::new_v4(),
            queue: std::sync::Arc::new(tokio::sync::Mutex::new(())),
        }
    }
    pub fn controller(&self) -> Uuid {
        self.controller
    }
    async fn authorize(
        &self,
        tx: &Transaction<'_>,
        bearer: &str,
        admission: Uuid,
        generation: i64,
        write: bool,
    ) -> Result<Uuid> {
        schema(tx).await?;
        storage::active(tx, self.broker.epoch).await?;
        let actor = auth::authenticate(tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(tx, &actor).await?;
        // No workflow/gateway grant exists in admissionv1; owner is the only dispatch authority.
        if write {
            actor.owner()?;
        }
        let row=tx.query_opt("SELECT a.id,a.task_id,t.mission_id,t.generation,a.generation,a.recovery_epoch,o.generation,o.recovery_epoch FROM operations o JOIN attempts a ON a.id=o.attempt_id JOIN tasks t ON t.id=a.task_id WHERE o.id=$1",&[&admission]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
        actor.scope(row.get(2), Some(row.get(1)))?;
        if row.get::<_, Uuid>(5) != self.broker.epoch || row.get::<_, Uuid>(7) != self.broker.epoch
        {
            return Err(err(ErrorCode::Unauthorized));
        }
        if row.get::<_, i64>(3) != generation
            || row.get::<_, i64>(4) != generation
            || row.get::<_, i64>(6) != generation
        {
            return Err(err(ErrorCode::StaleGeneration));
        }
        Ok(row.get(0))
    }
    /// Acquire a runtime-store controller and exclusive physical binding before intent.
    /// Store identity includes runtime/version/deployment/thread-store/endpoint/environment.
    pub async fn bind(
        &self,
        bearer: &str,
        id: Uuid,
        admission: Uuid,
        identity: &RuntimeIdentity,
        thread: Option<&str>,
        generation: i64,
    ) -> Result<()> {
        let store = identity.key()?;
        if thread.is_some_and(|t| !valid_thread_id(t)) {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let attempt = self
            .authorize(&tx, bearer, admission, generation, true)
            .await?;
        tx.execute(
            "INSERT INTO gateway_stores(identity,controller,runtime_identity) VALUES($1,$2,$3) ON CONFLICT DO NOTHING",
            &[&store, &self.controller, &serde_json::json!(identity)],
        )
        .await?;
        let owner = tx
            .query_one(
                "SELECT controller,quarantined,runtime_identity FROM gateway_stores WHERE identity=$1",
                &[&store],
            )
            .await?;
        if owner.get::<_, Value>(2) != serde_json::json!(identity) {
            return Err(err(ErrorCode::IdempotencyConflict));
        }
        if owner.get::<_, Uuid>(0) != self.controller || owner.get::<_, bool>(1) {
            tx.execute(
                "UPDATE gateway_stores SET quarantined=true WHERE identity=$1",
                &[&store],
            )
            .await?;
            tx.commit().await?;
            return Err(err(ErrorCode::TemporarilyUnavailable));
        }
        if let Some(row)=tx.query_opt("SELECT attempt_id,store,thread_id,controller FROM gateway_bindings WHERE id=$1 OR (store=$2 AND thread_id=$3)",&[&id,&store,&thread]).await? {
            if row.get::<_,Uuid>(0)==attempt && row.get::<_,String>(1)==store && row.get::<_,Option<String>>(2).as_deref()==thread && row.get::<_,Uuid>(3)==self.controller {
                // A physical thread has one canonical binding: aliases may not create another.
                if tx.query_opt("SELECT id FROM gateway_bindings WHERE id=$1",&[&id]).await?.is_some() {tx.commit().await?;return Ok(());}
            }
            return Err(err(ErrorCode::IdempotencyConflict));
        }
        tx.execute("INSERT INTO gateway_bindings(id,attempt_id,store,thread_id,controller) VALUES($1,$2,$3,$4,$5)",&[&id,&attempt,&store,&thread,&self.controller]).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn prepare(&self, bearer: &str, intent: Intent) -> Result<Operation> {
        let digest = crate::validation::digest(&intent)?;
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let attempt = self
            .authorize(&tx, bearer, intent.admission_id, intent.generation, true)
            .await?;
        if let Some(row) = tx
            .query_opt(
                "SELECT digest FROM gateway_operations WHERE id=$1",
                &[&intent.id],
            )
            .await?
        {
            let op = load(&tx, intent.id).await?;
            self.authorize(
                &tx,
                bearer,
                op.intent.admission_id,
                op.intent.generation,
                true,
            )
            .await?;
            if row.get::<_, String>(0) != digest {
                return Err(err(ErrorCode::IdempotencyConflict));
            }
            tx.commit().await?;
            return Ok(op);
        }
        let binding = tx
            .query_opt(
                "SELECT attempt_id,thread_id FROM gateway_bindings WHERE id=$1",
                &[&intent.binding_id],
            )
            .await?
            .ok_or_else(|| err(ErrorCode::NotFound))?;
        if binding.get::<_, Uuid>(0) != attempt {
            return Err(err(ErrorCode::Unauthorized));
        }
        let known = binding.get::<_, Option<String>>(1).is_some();
        if (intent.kind == OperationKind::Create && known)
            || (intent.kind != OperationKind::Create && !known)
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        if matches!(intent.kind, OperationKind::Create | OperationKind::Start)
            && tx
                .query_opt(
                    "SELECT id FROM gateway_operations WHERE admission_id=$1 AND kind=$2",
                    &[&intent.admission_id, &intent.kind.as_str()],
                )
                .await?
                .is_some()
        {
            return Err(err(ErrorCode::IdempotencyConflict));
        }
        tx.execute("INSERT INTO gateway_operations(id,admission_id,attempt_id,binding_id,kind,digest,request,generation,recovery_epoch,state) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,'prepared')",&[&intent.id,&intent.admission_id,&attempt,&intent.binding_id,&intent.kind.as_str(),&digest,&serde_json::json!(intent),&intent.generation,&self.broker.epoch]).await?;
        append(&tx, intent.id, Evidence::IntentCommitted).await?;
        let op = load(&tx, intent.id).await?;
        tx.commit().await?;
        Ok(op)
    }
    pub async fn read(&self, bearer: &str, id: Uuid) -> Result<Operation> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let op = load(&tx, id).await?;
        self.authorize(
            &tx,
            bearer,
            op.intent.admission_id,
            op.intent.generation,
            false,
        )
        .await?;
        tx.commit().await?;
        Ok(op)
    }
    /// Claim and commit before I/O. A duplicate dispatch only reads the original record.
    pub async fn dispatch(
        &self,
        bearer: &str,
        id: Uuid,
        runtime: &mut dyn RuntimeAdapter,
    ) -> Result<Operation> {
        self.dispatch_guarded(bearer, id, runtime, None).await
    }
    /// Optional workflow authority is validated inside the same serialized PG
    /// transaction that claims dispatch. Existing standalone dispatch has no guard.
    pub(crate) async fn dispatch_guarded(
        &self,
        bearer: &str,
        id: Uuid,
        runtime: &mut dyn RuntimeAdapter,
        guard: Option<&crate::temporal::WorkflowRef>,
    ) -> Result<Operation> {
        let _queue = self.queue.lock().await;
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let op = load(&tx, id).await?;
        self.authorize(
            &tx,
            bearer,
            op.intent.admission_id,
            op.intent.generation,
            true,
        )
        .await?;
        if let Some(reference) = guard {
            crate::temporal::dispatch_guard(
                &tx,
                self.broker.epoch,
                reference,
                op.intent.admission_id,
            )
            .await?;
        }
        if op.state != OperationState::Prepared {
            tx.commit().await?;
            return Ok(op);
        }
        let row=tx.query_one("SELECT s.identity,s.controller,s.busy,s.quarantined,b.controller,s.runtime_identity FROM gateway_stores s JOIN gateway_bindings b ON b.store=s.identity WHERE b.id=$1",&[&op.intent.binding_id]).await?;
        let profile = runtime.inspect();
        if row.get::<_, Value>(5) != serde_json::json!(runtime.identity()) {
            state(
                &tx,
                id,
                OperationState::Quarantined,
                Evidence::Quarantined {
                    reason: "runtime identity mismatch".into(),
                },
            )
            .await?;
            let result = load(&tx, id).await?;
            tx.commit().await?;
            return Ok(result);
        }
        if row.get::<_, Uuid>(1) != self.controller
            || row.get::<_, Uuid>(4) != self.controller
            || row.get::<_, bool>(3)
            || row.get::<_, Option<Uuid>>(2).is_some()
        {
            state(&tx,id,OperationState::Quarantined,Evidence::Quarantined {reason:"runtime control is retained, uncertain, or quarantined; transport fencing unproven".into()}).await?;
            let op = load(&tx, id).await?;
            tx.commit().await?;
            return Ok(op);
        }
        if !op.intent.kind.supported(&profile) {
            state(
                &tx,
                id,
                OperationState::FailedKnown,
                Evidence::Limitation {
                    capability: op.intent.kind.as_str().into(),
                },
            )
            .await?;
            let op = load(&tx, id).await?;
            tx.commit().await?;
            return Ok(op);
        }
        let store: String = row.get(0);
        tx.execute(
            "UPDATE gateway_stores SET busy=$2 WHERE identity=$1",
            &[&store, &id],
        )
        .await?;
        tx.execute(
            "UPDATE gateway_operations SET controller=$2 WHERE id=$1",
            &[&id, &self.controller],
        )
        .await?;
        state(
            &tx,
            id,
            OperationState::Dispatching,
            Evidence::DispatchClaimed {
                controller: self.controller,
            },
        )
        .await?;
        tx.commit().await?;
        // A controller remains the persistent owner while this synchronous RPC runs.
        // No replacement may dispatch, even if this process survives a crash elsewhere.
        let reply = runtime.execute(&op.intent, op.thread_id.as_deref());
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.authorize(
            &tx,
            bearer,
            op.intent.admission_id,
            op.intent.generation,
            true,
        )
        .await?;
        let current = load(&tx, id).await?;
        if current.state != OperationState::Dispatching {
            return Err(err(ErrorCode::TemporarilyUnavailable));
        }
        match reply {
            RuntimeReply::Confirmed {
                thread_id,
                effect_id,
            } => confirmed(&tx, &op, thread_id, effect_id, false).await?,
            RuntimeReply::FailedKnown(reason) => {
                state(
                    &tx,
                    id,
                    OperationState::FailedKnown,
                    Evidence::FailedKnown { reason },
                )
                .await?
            }
            RuntimeReply::Unknown(reason) => {
                state(
                    &tx,
                    id,
                    OperationState::OutcomeUnknown,
                    Evidence::OutcomeUnknown {
                        reason,
                        thread_id: op.thread_id,
                    },
                )
                .await?
            }
        }
        let op = load(&tx, id).await?;
        if matches!(
            op.state,
            OperationState::Confirmed | OperationState::FailedKnown
        ) {
            clear_busy(&tx, id).await?;
        }
        tx.commit().await?;
        Ok(op)
    }
    /// Authoritative observations may resolve an effect; lossy/silent history never proves absence.
    /// Reconciliation does not grant a new controller transport ownership.
    pub async fn reconcile(
        &self,
        bearer: &str,
        id: Uuid,
        runtime: &mut dyn RuntimeAdapter,
    ) -> Result<Operation> {
        let _queue = self.queue.lock().await;
        let op = self.read(bearer, id).await?;
        if !matches!(
            op.state,
            OperationState::Dispatching
                | OperationState::OutcomeUnknown
                | OperationState::Quarantined
        ) {
            return Ok(op);
        }
        let mut check = storage::client(&self.broker.dsn).await?;
        let check_tx = check.transaction().await?;
        storage::lock(&check_tx).await?;
        self.authorize(
            &check_tx,
            bearer,
            op.intent.admission_id,
            op.intent.generation,
            true,
        )
        .await?;
        let row=check_tx.query_one("SELECT s.runtime_identity,s.controller,s.identity FROM gateway_stores s JOIN gateway_bindings b ON b.store=s.identity WHERE b.id=$1",&[&op.intent.binding_id]).await?;
        let identity: Value = row.get(0);
        if row.get::<_, Uuid>(1) != self.controller {
            let store: String = row.get(2);
            check_tx
                .execute(
                    "UPDATE gateway_stores SET quarantined=true WHERE identity=$1",
                    &[&store],
                )
                .await?;
            state(&check_tx,id,OperationState::Quarantined,Evidence::Quarantined {reason:"previous control not confirmed inactive; absence while RPC may be in flight is insufficient".into()}).await?;
            let result = load(&check_tx, id).await?;
            check_tx.commit().await?;
            return Ok(result);
        }
        check_tx.commit().await?;
        let p = runtime.inspect();
        let observed = if identity != serde_json::json!(runtime.identity()) {
            Observation::Inconclusive("runtime identity mismatch".into())
        } else if p.observe && p.authoritative_effect_inventory {
            runtime.observe(&op.intent, op.thread_id.as_deref())
        } else {
            Observation::Inconclusive(
                "authoritative_effect_inventory capability unavailable".into(),
            )
        };
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        self.authorize(
            &tx,
            bearer,
            op.intent.admission_id,
            op.intent.generation,
            true,
        )
        .await?;
        let current = load(&tx, id).await?;
        if matches!(
            current.state,
            OperationState::Confirmed | OperationState::FailedKnown
        ) {
            tx.commit().await?;
            return Ok(current);
        }
        match observed {
            Observation::Effect {
                thread_id,
                effect_id,
            } => confirmed(&tx, &op, thread_id, effect_id, true).await?,
            Observation::AuthoritativeAbsence { inventory_id } => {
                state(
                    &tx,
                    id,
                    OperationState::FailedKnown,
                    Evidence::AuthoritativeAbsence { inventory_id },
                )
                .await?
            }
            Observation::Inconclusive(reason) => {
                state(
                    &tx,
                    id,
                    OperationState::Quarantined,
                    Evidence::Quarantined { reason },
                )
                .await?
            }
        }
        let result = load(&tx, id).await?;
        if matches!(
            result.state,
            OperationState::Confirmed | OperationState::FailedKnown
        ) {
            clear_busy(&tx, id).await?;
        }
        tx.commit().await?;
        Ok(result)
    }
}
async fn append(tx: &Transaction<'_>, id: Uuid, evidence: Evidence) -> Result<()> {
    tx.execute(
        "INSERT INTO gateway_evidence(id,operation_id,evidence) VALUES($1,$2,$3)",
        &[&Uuid::new_v4(), &id, &serde_json::json!(evidence)],
    )
    .await?;
    Ok(())
}
async fn state(tx: &Transaction<'_>, id: Uuid, s: OperationState, e: Evidence) -> Result<()> {
    tx.execute(
        "UPDATE gateway_operations SET state=$2 WHERE id=$1",
        &[&id, &s.as_str()],
    )
    .await?;
    append(tx, id, e).await
}
async fn clear_busy(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("UPDATE gateway_stores SET busy=NULL WHERE busy=$1", &[&id])
        .await?;
    Ok(())
}
fn valid_thread_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 1024
}
async fn confirmed(
    tx: &Transaction<'_>,
    op: &Operation,
    thread: Option<String>,
    effect: String,
    observation: bool,
) -> Result<()> {
    if effect.is_empty()
        || thread.as_deref().is_some_and(|id| !valid_thread_id(id))
        || (op.intent.kind == OperationKind::Create && thread.is_none())
        || (op.thread_id.is_some() && thread != op.thread_id)
    {
        return state(
            tx,
            op.id,
            OperationState::Quarantined,
            Evidence::Quarantined {
                reason: "incomplete or mismatched runtime identity".into(),
            },
        )
        .await;
    }
    if let Some(ref thread) = thread {
        let collision:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_bindings other JOIN gateway_bindings own ON own.store=other.store WHERE own.id=$1 AND other.thread_id=$2 AND other.id<>own.id)",&[&op.intent.binding_id,&thread]).await?.get(0);
        if collision {
            return state(
                tx,
                op.id,
                OperationState::Quarantined,
                Evidence::Quarantined {
                    reason: "physical thread already has a binding".into(),
                },
            )
            .await;
        }
        tx.execute(
            "UPDATE gateway_bindings SET thread_id=$2 WHERE id=$1",
            &[&op.intent.binding_id, &thread],
        )
        .await?;
    }
    let evidence = if observation {
        Evidence::AuthoritativeEffect {
            effect_id: effect,
            thread_id: thread,
        }
    } else {
        Evidence::ReplyConfirmed {
            effect_id: effect,
            thread_id: thread,
        }
    };
    state(tx, op.id, OperationState::Confirmed, evidence).await
}
async fn load(tx: &Transaction<'_>, id: Uuid) -> Result<Operation> {
    let row=tx.query_opt("SELECT o.request,o.state,b.thread_id FROM gateway_operations o JOIN gateway_bindings b ON b.id=o.binding_id WHERE o.id=$1",&[&id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
    let intent =
        serde_json::from_value(row.get(0)).map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
    let state = serde_json::from_value(Value::String(row.get(1)))
        .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
    let mut evidence = Vec::new();
    for row in tx
        .query(
            "SELECT evidence FROM gateway_evidence WHERE operation_id=$1 ORDER BY sequence",
            &[&id],
        )
        .await?
    {
        evidence.push(
            serde_json::from_value(row.get(0))
                .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?,
        );
    }
    Ok(Operation {
        id,
        intent,
        state,
        thread_id: row.get(2),
        evidence,
    })
}
pub(crate) async fn migrate(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('gateway_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        tx.batch_execute(include_str!("../migrations/0003.sql"))
            .await?;
    }
    schema(tx).await
}
async fn schema(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('gateway_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    let rows = tx
        .query("SELECT version FROM gateway_schema_version", &[])
        .await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
