//! Declarative Plan admission only; this module never evaluates or runs code.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub kind: String,
    pub depends_on: Vec<String>,
    pub join: String,
    pub effects: Vec<String>,
    pub max_iterations: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlanRevision {
    pub schema_version: u32,
    pub interpreter_version: String,
    pub intent: String,
    pub acceptance: String,
    pub allowed_effects: Vec<String>,
    pub resources: Resources,
    pub max_repairs: i64,
    pub steps: Vec<Step>,
}
pub(super) fn within(actual: &Resources, limit: &Resources) -> bool {
    actual.attempts > 0
        && actual.episodes > 0
        && actual.repairs >= 0
        && actual.execution_seconds > 0
        && actual.attempts <= limit.attempts
        && actual.episodes <= limit.episodes
        && actual.repairs <= limit.repairs
        && actual.execution_seconds <= limit.execution_seconds
}
pub(super) async fn current(
    tx: &Transaction<'_>,
    task: Uuid,
) -> Result<(String, String, IntentRevision)> {
    let row=tx.query_opt("SELECT i.id,i.acceptance_id,i.intent FROM contract_current c JOIN contract_intents i ON i.id=c.intent_id WHERE c.task_id=$1 AND c.admission_open",&[&task]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
    Ok((
        row.get(0),
        row.get(1),
        serde_json::from_value(row.get(2)).map_err(|_| err(ErrorCode::TemporarilyUnavailable))?,
    ))
}
fn validate(plan: &PlanRevision) -> Result<()> {
    if plan.schema_version != 1
        || plan.interpreter_version != "declarative-v1"
        || plan.steps.is_empty()
        || plan.steps.len() > 128
        || plan.allowed_effects.is_empty()
        || !unique(&plan.allowed_effects)
        || plan.max_repairs < 0
        || plan.max_repairs > plan.resources.repairs
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    let mut steps = BTreeMap::new();
    for step in &plan.steps {
        if !text(&step.id)
            || step.id.len() > 128
            || steps.insert(&step.id, step).is_some()
            || !["action", "join", "loop", "repair"].contains(&step.kind.as_str())
            || !["all", "any"].contains(&step.join.as_str())
            || !unique(&step.depends_on)
            || !unique(&step.effects)
            || step
                .effects
                .iter()
                .any(|e| !plan.allowed_effects.contains(e))
            || step.max_iterations <= 0
            || step.max_iterations > 128
            || (step.kind != "loop" && step.max_iterations != 1)
            || (step.kind == "repair" && plan.max_repairs == 0)
            || (step.kind == "join" && (step.depends_on.is_empty() || !step.effects.is_empty()))
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
    }
    let mut completed = BTreeSet::new();
    loop {
        let before = completed.len();
        for step in &plan.steps {
            if step.depends_on.iter().any(|id| !steps.contains_key(id)) {
                return Err(err(ErrorCode::InvalidArgument));
            }
            if step.depends_on.iter().all(|id| completed.contains(id)) {
                completed.insert(step.id.clone());
            }
        }
        if completed.len() == plan.steps.len() {
            return Ok(());
        }
        if completed.len() == before {
            return Err(err(ErrorCode::InvalidArgument));
        }
    }
}
pub(super) async fn admit(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    revision: i64,
    plan: &PlanRevision,
) -> Result<Value> {
    validate(plan)?;
    let (intent_id, acceptance_id, intent) = current(tx, envelope.task).await?;
    if plan.intent != intent_id || plan.acceptance != acceptance_id {
        return Err(err(ErrorCode::StaleRevision));
    }
    if plan
        .allowed_effects
        .iter()
        .any(|e| !intent.allowed_effects.contains(e))
    {
        return Err(err(ErrorCode::Unauthorized));
    }
    if !within(&plan.resources, &intent.resources) {
        return Err(err(ErrorCode::BudgetExhausted));
    }
    // The current strategy pointer may be cleared by source admission. Its
    // authority ceilings survive in immutable admitted history for every caller.
    // Plan admission cannot expand authority or cumulative budget envelopes.
    if let Some(row)=tx.query_opt("SELECT plan FROM contract_plans WHERE task_id=$1 ORDER BY admitted_version DESC LIMIT 1",&[&envelope.task]).await? {
        let previous:PlanRevision=serde_json::from_value(row.get(0)).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?;
        if !within(&plan.resources,&previous.resources) || plan.max_repairs>previous.max_repairs {return Err(err(ErrorCode::BudgetExhausted));}
        if plan.allowed_effects.iter().any(|e|!previous.allowed_effects.contains(e)) {return Err(err(ErrorCode::Unauthorized));}
    }
    // All earlier consumed steps survive strategy replacement. Their meaning
    // cannot depend on the latest plan or on a replaced interpreter.
    for row in tx
        .query(
            "SELECT step_id,step FROM contract_completed_steps WHERE task_id=$1",
            &[&envelope.task],
        )
        .await?
    {
        let id: String = row.get(0);
        let stored: Value = row.get(1);
        if plan
            .steps
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| serde_json::to_value(s).ok())
            .as_ref()
            != Some(&stored)
        {
            return Err(err(ErrorCode::StaleRevision));
        }
    }
    authority::check_limit(tx, actor, envelope, false).await?;
    let id = crate::validation::digest(&(envelope.task, plan))?;
    if tx
        .query_opt("SELECT id FROM contract_plans WHERE id=$1", &[&id])
        .await?
        .is_some()
    {
        return Err(err(ErrorCode::StaleRevision));
    }
    let version: i64 = tx
        .query_one(
            "SELECT count(*)+1 FROM contract_plans WHERE task_id=$1",
            &[&envelope.task],
        )
        .await?
        .get(0);
    tx.execute("INSERT INTO contract_plans(id,task_id,intent_id,acceptance_id,admitted_version,plan,principal_id,recovery_epoch) VALUES($1,$2,$3,$4,$5,$6,$7,$8)",&[&id,&envelope.task,&intent_id,&acceptance_id,&version,&serde_json::to_value(plan).map_err(|_|err(ErrorCode::InvalidArgument))?,&actor.principal,&envelope.recovery_epoch]).await?;
    tx.execute(
        "UPDATE contract_current SET revision=$2,plan_id=$3 WHERE task_id=$1",
        &[&envelope.task, &(revision + 1), &id],
    )
    .await?;
    Ok(
        json!({"schema_version":2,"disposition":"applied","revision":revision+1,"plan":id,"admitted_version":version}),
    )
}
