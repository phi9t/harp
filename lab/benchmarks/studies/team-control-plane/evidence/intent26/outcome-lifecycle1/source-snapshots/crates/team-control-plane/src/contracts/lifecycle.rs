//! Explicit later work fences the accepted generation without rewriting its
//! evidence, attempts, reservations or cumulative accounting.
use super::*;

pub(super) async fn migrate(tx: &Transaction<'_>) -> Result<()> {
    let marker: bool = tx
        .query_one(
            "SELECT to_regclass('contract_attempt_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !marker {
        let legacy: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='attempts'::regclass AND conname='attempts_task_id_key' AND pg_get_constraintdef(oid)='UNIQUE (task_id)') AND NOT EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='attempts'::regclass AND conname='attempts_task_generation_key')", &[]).await?.get(0);
        if !legacy {
            return Err(err(ErrorCode::IncompatibleSchema));
        }
        tx.batch_execute(include_str!("../../migrations/0009.sql"))
            .await?;
    }
    schema(tx).await
}

pub(super) async fn schema(tx: &Transaction<'_>) -> Result<()> {
    let marker: bool = tx
        .query_one(
            "SELECT to_regclass('contract_attempt_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !marker {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    let rows = tx
        .query("SELECT version FROM contract_attempt_schema_version", &[])
        .await?;
    let constraints: bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='attempts'::regclass AND conname='attempts_task_generation_key' AND pg_get_constraintdef(oid)='UNIQUE (task_id, generation)') AND NOT EXISTS(SELECT 1 FROM pg_constraint WHERE conrelid='attempts'::regclass AND contype='u' AND pg_get_constraintdef(oid)='UNIQUE (task_id)')", &[]).await?.get(0);
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 || !constraints {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}

/// Closure of the exact admitted work. Legacy acceptance remains an inert
/// historical case; only the typed outcome supplies a v2 accepted subject.
#[derive(Serialize)]
pub(crate) struct CurrentWork {
    pub(crate) accepted: Option<Value>,
    pub(crate) legacy_accepted: bool,
    pub(crate) cancelled: bool,
}
impl CurrentWork {
    pub(crate) fn completed(&self) -> bool {
        self.accepted.is_some() || self.legacy_accepted
    }
    pub(crate) fn terminal(&self) -> bool {
        self.completed() || self.cancelled
    }
}
pub(crate) async fn current(tx: &Transaction<'_>, task: Uuid) -> Result<CurrentWork> {
    let row = tx.query_opt(
        "SELECT t.status, to_jsonb(o) FROM tasks t LEFT JOIN contract_current c ON c.task_id=t.id LEFT JOIN contract_outcomes o ON o.task_id=t.id AND o.intent_id=c.intent_id WHERE t.id=$1",
        &[&task],
    ).await?.ok_or_else(||err(ErrorCode::NotFound))?;
    let status: String = row.get(0);
    let accepted: Option<Value> = row.get(1);
    Ok(CurrentWork {
        legacy_accepted: status == "accepted" && accepted.is_none(),
        accepted,
        cancelled: status == "cancelled",
    })
}

pub(crate) async fn require_open(tx: &Transaction<'_>, task: Uuid) -> Result<()> {
    if current(tx, task).await?.terminal() {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    Ok(())
}

pub(super) async fn amend(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    next_intent: &str,
) -> Result<()> {
    let row = tx.query_opt(
        "SELECT t.status,t.revision,t.generation,t.spec_digest,c.intent_id,EXISTS(SELECT 1 FROM contract_outcomes o WHERE o.task_id=t.id AND o.intent_id=c.intent_id) FROM tasks t JOIN contract_current c ON c.task_id=t.id WHERE t.id=$1",
        &[&envelope.task],
    ).await?;
    let Some(row) = row else {
        // Mapping historical v1 acceptance alone admits no new execution.
        return Ok(());
    };
    if !current(tx, envelope.task).await?.completed() {
        return Ok(());
    }
    // A later explicit Owner amendment can start new work after a historical
    // mapping. A delegated lead needs an accepted v2 revision under its scope.
    if actor.role != Role::Owner && !row.get::<_, bool>(5) {
        return Err(err(ErrorCode::Unauthorized));
    }
    crate::kernel::task_dispatch_enabled(tx, envelope.task).await?;
    let previous_generation: i64 = row.get(2);
    let revision = row.get::<_, i64>(1) + 1;
    tx.execute(
        "UPDATE tasks SET status='ready',generation=generation+1,revision=$2 WHERE id=$1",
        &[&envelope.task, &revision],
    )
    .await?;
    let payload = json!({"by_principal":actor.principal,"recovery_epoch":envelope.recovery_epoch,"previous_intent":row.get::<_,String>(4),"intent":next_intent,"previous_generation":previous_generation,"generation":previous_generation+1,"spec_digest":row.get::<_,String>(3),"reason":"explicit admitted later work; accepted evidence remains historical"});
    tx.execute("INSERT INTO events(id,aggregate_kind,aggregate_id,sequence,version,principal_id,transaction_id,kind,payload) VALUES($1,'task',$2,$3,1,$4,txid_current(),'task.intent.amend',$5)",&[&Uuid::new_v4(),&envelope.task,&revision,&actor.principal,&payload]).await?;
    Ok(())
}

/// Exact research authority captured in the immutable input/manifest. It is not
/// a production candidate, report or production verification binding.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchVerification {
    pub intent: String,
    pub acceptance: String,
    pub verifier: Uuid,
    pub policy: String,
    pub suite: String,
    pub environment: String,
}
pub(crate) async fn research_verification(
    tx: &Transaction<'_>,
    task: Uuid,
) -> Result<Option<ResearchVerification>> {
    let Some(row) = tx.query_opt("SELECT c.intent_id,i.acceptance_id,i.intent FROM contract_current c JOIN contract_intents i ON i.id=c.intent_id WHERE c.task_id=$1", &[&task]).await? else {return Ok(None);};
    let intent: IntentRevision =
        serde_json::from_value(row.get(2)).map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
    if intent.acceptance.outcome != Outcome::InvestigationComplete {
        return Ok(None);
    }
    Ok(Some(ResearchVerification {
        intent: row.get(0),
        acceptance: row.get(1),
        verifier: intent.acceptance.verifier,
        policy: intent.acceptance.policy,
        suite: intent.acceptance.suite,
        environment: intent.acceptance.environment,
    }))
}
pub(crate) async fn research_input(
    tx: &Transaction<'_>,
    input: Uuid,
    expected: &ResearchVerification,
) -> Result<()> {
    let packet: Value = tx
        .query_one("SELECT packet FROM mailbox_inputs WHERE id=$1", &[&input])
        .await?
        .get(0);
    if packet["task"]["research_verification"]
        != serde_json::to_value(expected).map_err(|_| err(ErrorCode::InvalidArgument))?
    {
        return Err(err(ErrorCode::StaleRevision));
    }
    Ok(())
}
