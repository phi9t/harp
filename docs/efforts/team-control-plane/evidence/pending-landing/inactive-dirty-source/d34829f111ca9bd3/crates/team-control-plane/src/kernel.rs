//! Authenticated operator views are independent of runtime/execution ownership.
use crate::{Broker, ErrorCode, Result, auth, domain::err, storage};
use serde_json::{Value, json};

#[derive(Clone, Debug)]
pub struct Kernel {
    broker: Broker,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TakeoverRequest {
    pub operation: uuid::Uuid,
    pub expected_controller: uuid::Uuid,
    pub replacement: uuid::Uuid,
    pub key: String,
    pub expected_control_revision: i64,
}
impl Kernel {
    /// Owner transfers a paused physical store only after independent closed-world proof.
    pub async fn takeover(&self, bearer: &str, request: TakeoverRequest) -> Result<Value> {
        use crate::model_free_runtime as os;
        let rejected = || err(ErrorCode::DependencyUnmet);
        if request.key.is_empty()
            || request.key.len() > 256
            || request.expected_control_revision < 0
            || request.expected_controller == request.replacement
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        actor.owner()?;
        let row=tx.query_opt("SELECT x.mission_id,x.task_id,o.store FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE o.id=$1", &[&request.operation]).await?.ok_or_else(rejected)?;
        let mission: uuid::Uuid = row.get(0);
        actor.scope(mission, row.get(1))?;
        let store: Option<String> = row.get(2);
        let store = store.ok_or_else(rejected)?;
        let digest = crate::validation::digest(
            &json!({"request":request,"recovery_epoch":self.broker.epoch}),
        )?;
        let control = tx
            .query_opt(
                "SELECT paused,revision FROM kernel_mission_control WHERE mission_id=$1",
                &[&mission],
            )
            .await?
            .ok_or_else(rejected)?;
        if !control.get::<_, bool>(0) {
            return Err(rejected());
        }
        if control.get::<_, i64>(1) != request.expected_control_revision {
            return Err(err(ErrorCode::StaleRevision));
        }
        let owner=tx.query_one("SELECT controller,runtime_identity,quarantined FROM gateway_stores WHERE identity=$1 FOR UPDATE", &[&store]).await?;
        let actual_controller: uuid::Uuid = owner.get(0);
        if ![request.expected_controller, request.replacement].contains(&actual_controller) {
            return Err(err(ErrorCode::StaleRevision));
        }
        if owner.get::<_, bool>(2) {
            return Err(rejected());
        }
        let old = os::transport(&tx, request.expected_controller).await?;
        let replacement = os::transport(&tx, request.replacement).await?;
        if old.store != store
            || replacement.store != store
            || old.identity != replacement.identity
            || owner.get::<_, Value>(1) != json!(old.identity)
            || old.native_directory != replacement.native_directory
            || old.workspace_directory != replacement.workspace_directory
            || old.native_root != replacement.native_root
            || old.workspace_root != replacement.workspace_root
            || replacement.epoch != self.broker.epoch
        {
            return Err(rejected());
        }
        if !os::transport_finalized(&old.process, old.group).map_err(|_| rejected())?
            || !os::exclusive_live_transport(&replacement.process, replacement.group)
                .map_err(|_| rejected())?
        {
            return Err(rejected());
        }
        let inventory = os::inventory(&tx, &old).await?;
        // The PG lock excludes all admitted claims. Independent directory handles
        // remain held while incarnation/group/roots are revalidated at transfer.
        if !os::transport_finalized(&old.process, old.group).map_err(|_| rejected())?
            || !os::exclusive_live_transport(&replacement.process, replacement.group)
                .map_err(|_| rejected())?
            || os::directory_identity(&old.native_root)? != old.native_directory
            || os::directory_identity(&old.workspace_root)? != old.workspace_directory
        {
            return Err(rejected());
        }
        if let Some(cached)=tx.query_opt("SELECT argument_digest,result FROM kernel_commands WHERE principal_id=$1 AND family='takeover' AND key=$2", &[&actor.principal,&request.key]).await? {
            if cached.get::<_,String>(0)!=digest{return Err(err(ErrorCode::IdempotencyConflict));}
            if actual_controller!=request.replacement{return Err(err(ErrorCode::StaleRevision));}
            let result:Value=cached.get(1);
            if result["inventory_digest"]!=inventory.digest{return Err(err(ErrorCode::StaleRevision));}
            tx.commit().await?;drop(inventory.native);drop(inventory.workspaces);return Ok(result);
        }
        if actual_controller != request.expected_controller {
            return Err(err(ErrorCode::StaleRevision));
        }
        for (prepared, turn, usage) in &inventory.effects {
            let operation = tx
                .query_one(
                    "SELECT state,recovery_epoch FROM episode_runtime_operations WHERE id=$1",
                    &[&prepared.start.id],
                )
                .await?;
            let state: String = operation.get(0);
            let epoch: uuid::Uuid = operation.get(1);
            if !crate::episode_runtime::confirm_native(&tx, prepared, &store, turn, epoch).await? {
                return Err(rejected());
            }
            crate::episode_runtime::record_usage(&tx, prepared, usage, epoch).await?;
            if state != "confirmed" {
                tx.execute(
                    "UPDATE episode_runtime_operations SET state='confirmed' WHERE id=$1",
                    &[&prepared.start.id],
                )
                .await?;
                tx.execute("INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&uuid::Uuid::new_v4(),&prepared.start.id,&json!({"kind":"takeover_observed_existing_effect","turn":turn,"inventory_digest":inventory.digest})]).await?;
            }
        }
        tx.execute(
            "UPDATE gateway_stores SET controller=$2,busy=NULL WHERE identity=$1 AND controller=$3",
            &[&store, &request.replacement, &request.expected_controller],
        )
        .await?;
        let result = json!({"controller":request.replacement,"previous_controller":request.expected_controller,"mission":mission,"operation":request.operation,"inventory_digest":inventory.digest,"dispatch_paused":true,"control_revision":request.expected_control_revision});
        tx.execute("INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&uuid::Uuid::new_v4(),&request.operation,&json!({"kind":"exclusive_takeover","old_process":old.process,"old_group":old.group,"replacement_process":replacement.process,"replacement_group":replacement.group,"native_directory":old.native_directory,"workspace_directory":old.workspace_directory,"result":result})]).await?;
        tx.execute("INSERT INTO kernel_control_events(id,mission_id,principal_id,recovery_epoch,kind,argument_digest) VALUES($1,$2,$3,$4,'takeover',$5)", &[&uuid::Uuid::new_v4(),&mission,&actor.principal,&self.broker.epoch,&digest]).await?;
        tx.execute("INSERT INTO kernel_commands(principal_id,family,key,mission_id,argument_digest,result,recovery_epoch) VALUES($1,'takeover',$2,$3,$4,$5,$6)", &[&actor.principal,&request.key,&mission,&digest,&result,&self.broker.epoch]).await?;
        // Keep the pins alive through commit, rather than inferring their presence.
        tx.commit().await?;
        drop(inventory.native);
        drop(inventory.workspaces);
        Ok(result)
    }
    /// Owner activates irreversible execution-backed acceptance before assembled dispatch.
    pub async fn require_supervised_verification(
        &self,
        bearer: &str,
        task: uuid::Uuid,
    ) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        actor.owner()?;
        let mission: uuid::Uuid = tx
            .query_opt(
                "SELECT mission_id FROM tasks WHERE id=$1 AND status IN ('ready','running')",
                &[&task],
            )
            .await?
            .ok_or_else(|| err(ErrorCode::DependencyUnmet))?
            .get(0);
        actor.scope(mission, Some(task))?;
        tx.execute("INSERT INTO kernel_verification_requirements(task_id,recovery_epoch,by_principal) VALUES($1,$2,$3) ON CONFLICT DO NOTHING", &[&task,&self.broker.epoch,&actor.principal]).await?;
        tx.commit().await?;
        Ok(())
    }
    pub fn new(broker: Broker) -> Self {
        Self { broker }
    }
    /// Operator control is journaled under the same authenticated broker/PG lock.
    pub async fn pause(
        &self,
        bearer: &str,
        mission: uuid::Uuid,
        key: &str,
        expected_revision: i64,
        paused: bool,
        rationale: &str,
    ) -> Result<Value> {
        use crate::{ErrorCode, domain::err};
        if key.is_empty()
            || key.len() > 256
            || rationale.is_empty()
            || rationale.len() > 4096
            || expected_revision < 0
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut client = storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        actor.owner()?;
        if tx
            .query_opt("SELECT id FROM missions WHERE id=$1", &[&mission])
            .await?
            .is_none()
        {
            return Err(err(ErrorCode::NotFound));
        }
        let digest = crate::validation::digest(
            &json!({"mission":mission,"expected_revision":expected_revision,"paused":paused,"rationale":rationale,"recovery_epoch":self.broker.epoch}),
        )?;
        if let Some(row)=tx.query_opt("SELECT argument_digest,result FROM kernel_commands WHERE principal_id=$1 AND family='pause' AND key=$2",&[&actor.principal,&key]).await? {
            if row.get::<_,String>(0)!=digest {return Err(err(ErrorCode::IdempotencyConflict));}
            return Ok(row.get(1));
        }
        let revision = tx
            .query_opt(
                "SELECT revision FROM kernel_mission_control WHERE mission_id=$1",
                &[&mission],
            )
            .await?
            .map(|r| r.get::<_, i64>(0))
            .unwrap_or(0);
        if revision != expected_revision {
            return Err(err(ErrorCode::StaleRevision));
        }
        let next = revision
            .checked_add(1)
            .ok_or_else(|| err(ErrorCode::InvalidArgument))?;
        tx.execute("INSERT INTO kernel_mission_control(mission_id,paused,revision) VALUES($1,$2,$3) ON CONFLICT(mission_id) DO UPDATE SET paused=EXCLUDED.paused,revision=EXCLUDED.revision",&[&mission,&paused,&next]).await?;
        tx.execute("INSERT INTO kernel_control_events(id,mission_id,principal_id,recovery_epoch,kind,argument_digest) VALUES($1,$2,$3,$4,'pause',$5)",&[&uuid::Uuid::new_v4(),&mission,&actor.principal,&self.broker.epoch,&digest]).await?;
        let result = json!({"mission":mission,"paused":paused,"revision":next,"recovery_epoch":self.broker.epoch});
        tx.execute("INSERT INTO kernel_commands(principal_id,family,key,mission_id,argument_digest,result,recovery_epoch) VALUES($1,'pause',$2,$3,$4,$5,$6)",&[&actor.principal,&key,&mission,&digest,&result,&self.broker.epoch]).await?;
        tx.commit().await?;
        Ok(result)
    }
    /// Bounded current views; credentials and private runtime configuration stay outside output.
    pub async fn status(&self, bearer: &str) -> Result<Value> {
        let mut client = storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        let owner = actor.role == crate::Role::Owner;
        let missions = tx.query("SELECT jsonb_build_object('id',m.id,'revision',m.revision,'capacity',m.capacity,'used_capacity',m.used_capacity,'admitted_attempts',m.admitted_attempts,'admitted_episodes',m.admitted_episodes,'max_attempts',m.max_attempts,'max_episodes',m.max_episodes,'max_repairs',m.max_repairs,'consumed_repairs',(SELECT count(*) FROM kernel_repair_charges r WHERE r.mission_id=m.id),'deadline_seconds',m.deadline_seconds,'dispatch_paused',coalesce((SELECT paused FROM kernel_mission_control WHERE mission_id=m.id),false)) FROM missions m WHERE ($1 OR m.id=$2) ORDER BY m.id LIMIT 33", &[&owner,&actor.mission]).await?;
        let more = missions.len() > 32;
        let missions: Vec<Value> = missions.into_iter().take(32).map(|r| r.get(0)).collect();
        let execution_rows=tx.query("SELECT jsonb_build_object('id',w.id,'mission',x.mission_id,'task',x.task_id,'attempt',x.attempt_id,'episode',x.episode_id,'role',x.role,'observed_state',w.state,'desired_cancel',w.desired_cancel,'deadline',w.deadline,'quarantine_reason',w.quarantine_reason,'candidate_artifact',w.manifest->'candidate'->'artifact','capacity_released',x.capacity_released) FROM supervised_workspaces w JOIN episode_executions x ON x.execution_id=w.id WHERE ($1 OR (x.mission_id=$2 AND ($3::uuid IS NULL OR x.task_id=$3) AND x.principal_id=$4)) ORDER BY w.id LIMIT 33", &[&owner,&actor.mission,&actor.task,&actor.principal]).await?;
        let more_executions = execution_rows.len() > 32;
        let executions: Vec<Value> = execution_rows
            .into_iter()
            .take(32)
            .map(|r| r.get(0))
            .collect();
        let operation_rows=tx.query("SELECT jsonb_build_object('id',o.id,'episode',o.episode_id,'execution',o.execution_id,'state',o.state,'thread_id',o.thread_id,'turn_id',o.turn_id,'effect_id',o.effect_id,'usage',o.usage) FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE ($1 OR (x.mission_id=$2 AND ($3::uuid IS NULL OR x.task_id=$3) AND x.principal_id=$4)) AND (o.state IN ('prepared','dispatching','outcome_unknown','quarantined') OR (o.state IN ('confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero')) ORDER BY o.id LIMIT 33", &[&owner,&actor.mission,&actor.task,&actor.principal]).await?;
        let more_operations = operation_rows.len() > 32;
        let unresolved_operations: Vec<Value> = operation_rows
            .into_iter()
            .take(32)
            .map(|r| r.get(0))
            .collect();
        let result = json!({"schema_version":1,"recovery_epoch":self.broker.epoch,"missions":missions,"more_missions":more,"executions":executions,"more_executions":more_executions,"unresolved_operations":unresolved_operations,"more_operations":more_operations});
        tx.commit().await?;
        Ok(result)
    }
}

pub(crate) async fn migrate(tx: &tokio_postgres::Transaction<'_>) -> Result<()> {
    if !tx
        .query_one(
            "SELECT to_regclass('kernel_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0)
    {
        tx.batch_execute(include_str!("../migrations/0007.sql"))
            .await?;
    }
    schema(tx).await
}
pub(crate) async fn schema(tx: &tokio_postgres::Transaction<'_>) -> Result<()> {
    if !tx
        .query_one(
            "SELECT to_regclass('kernel_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0)
    {
        return Err(crate::domain::err(crate::ErrorCode::IncompatibleSchema));
    }
    let rows = tx
        .query("SELECT version FROM kernel_schema_version", &[])
        .await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
        return Err(crate::domain::err(crate::ErrorCode::IncompatibleSchema));
    }
    Ok(())
}

pub(crate) async fn dispatch_enabled(
    tx: &tokio_postgres::Transaction<'_>,
    mission: uuid::Uuid,
) -> Result<()> {
    schema(tx).await?;
    if tx.query_one("SELECT coalesce((SELECT paused FROM kernel_mission_control WHERE mission_id=$1),false)",&[&mission]).await?.get::<_,bool>(0) {return Err(crate::domain::err(crate::ErrorCode::DependencyUnmet));}
    Ok(())
}

/// Accepted standalone gateway behavior remains separate from an explicitly
/// assembled task. The durable task policy covers all its genuine attempts,
/// even if a legacy request omits Episode/Execution metadata.
pub(crate) async fn legacy_usage_unknown(
    tx: &tokio_postgres::Transaction<'_>,
    attempt: uuid::Uuid,
) -> Result<bool> {
    Ok(tx.query_one(
        "SELECT EXISTS(SELECT 1 FROM gateway_operations o JOIN attempts a ON a.id=o.attempt_id JOIN kernel_verification_requirements r ON r.task_id=a.task_id WHERE o.attempt_id=$1 AND o.state IN ('dispatching','outcome_unknown','quarantined','confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero')",
        &[&attempt],
    ).await?.get(0))
}
