//! Explicit current-lead delegation. Authentication alone conveys no admission.
use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delegation {
    pub id: Uuid,
    pub principal: Uuid,
    pub role_epoch: i64,
    pub work: WorkRef,
    pub families: Vec<String>,
    pub effects: Vec<String>,
    pub max_intents: i64,
    pub max_plans: i64,
    pub resources: Resources,
    pub expires_in_seconds: i64,
}
async fn admitted(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    mission: Uuid,
) -> Result<Delegation> {
    let family = envelope.command.family();
    let rows=tx.query("SELECT d.delegation FROM contract_delegations d JOIN mission_leads l ON l.mission_id=d.mission_id AND l.principal_id=d.principal_id AND l.generation=d.role_epoch JOIN contract_work w ON w.id=d.work_id WHERE d.task_id=$1 AND d.mission_id=$2 AND d.principal_id=$3 AND d.recovery_epoch=$4 AND d.expires_at>clock_timestamp() AND w.task_id=$1 ORDER BY d.expires_at DESC",&[&envelope.task,&mission,&actor.principal,&envelope.recovery_epoch]).await?;
    for row in rows {
        let delegation: Delegation = serde_json::from_value(row.get(0))
            .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
        if delegation.families.iter().any(|f| f == family) {
            return Ok(delegation);
        }
    }
    Err(err(ErrorCode::Unauthorized))
}
pub(super) async fn authorize(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    mission: Uuid,
) -> Result<()> {
    if matches!(
        envelope.command,
        Command::ProposeOutcome { .. }
            | Command::BindVerification { .. }
            | Command::VerifyProduction { .. }
            | Command::VerifyResearch { .. }
    ) {
        return outcomes::authorize(tx, actor, envelope).await;
    }
    if matches!(envelope.command, Command::ProposeSource { .. }) {
        return Ok(());
    }
    if actor.role == Role::Owner {
        return Ok(());
    }
    if actor.role != Role::Lead || matches!(envelope.command, Command::AdmitDelegation { .. }) {
        return Err(err(ErrorCode::Unauthorized));
    }
    let delegation = admitted(tx, actor, envelope, mission).await?;
    let (_, _, current) = strategy::current(tx, envelope.task).await?;
    let counts=tx.query_one("SELECT (SELECT count(*) FROM contract_intents i JOIN contract_work w ON w.id=i.work_id WHERE w.task_id=$1),(SELECT count(*) FROM contract_plans WHERE task_id=$1)",&[&envelope.task]).await?;
    // A replay is still a current authority decision. Owner amendments may
    // exceed an earlier delegation, but they cannot silently enlarge it.
    if counts.get::<_, i64>(0) > delegation.max_intents
        || counts.get::<_, i64>(1) > delegation.max_plans
        || !strategy::within(&current.resources, &delegation.resources)
        || current
            .allowed_effects
            .iter()
            .any(|e| !delegation.effects.contains(e))
    {
        return Err(err(ErrorCode::Unauthorized));
    }
    match &envelope.command {
        Command::AdmitIntent { work, intent, .. } => {
            // Outcome, acceptance and authority expansion always need Owner.
            if work != &delegation.work
                || intent.objective != current.objective
                || intent.acceptance != current.acceptance
                || intent.legacy_spec_digest != current.legacy_spec_digest
                || current
                    .constraints
                    .iter()
                    .any(|c| !intent.constraints.contains(c))
                || intent.allowed_effects.iter().any(|e| {
                    !current.allowed_effects.contains(e) || !delegation.effects.contains(e)
                })
                || !strategy::within(&intent.resources, &current.resources)
                || !strategy::within(&intent.resources, &delegation.resources)
            {
                return Err(err(ErrorCode::Unauthorized));
            }
            // General intent delegation cannot discard or replace prerequisites.
            // Use the current projection so an exact explicit revalidation is
            // preserved even when the immutable original Intent predates it.
            dependencies::preserve(tx, envelope.task, &intent.dependencies).await?;
        }
        Command::AdmitPlan { plan }
            if plan
                .allowed_effects
                .iter()
                .any(|e| !delegation.effects.contains(e))
                || !strategy::within(&plan.resources, &delegation.resources) =>
        {
            return Err(err(ErrorCode::Unauthorized));
        }
        _ => {}
    }
    Ok(())
}
pub(super) async fn check_limit(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    intent: bool,
) -> Result<()> {
    if actor.role == Role::Owner {
        return Ok(());
    }
    let mission: Uuid = tx
        .query_one(
            "SELECT mission_id FROM tasks WHERE id=$1",
            &[&envelope.task],
        )
        .await?
        .get(0);
    let delegation = admitted(tx, actor, envelope, mission).await?;
    let count: i64 = if intent {
        tx.query_one("SELECT count(*) FROM contract_intents i JOIN contract_work w ON w.id=i.work_id WHERE w.task_id=$1",&[&envelope.task]).await?.get(0)
    } else {
        tx.query_one(
            "SELECT count(*) FROM contract_plans WHERE task_id=$1",
            &[&envelope.task],
        )
        .await?
        .get(0)
    };
    if count
        >= if intent {
            delegation.max_intents
        } else {
            delegation.max_plans
        }
    {
        return Err(err(ErrorCode::BudgetExhausted));
    }
    Ok(())
}
pub(super) async fn admit(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    mission: Uuid,
    revision: i64,
    delegation: &Delegation,
) -> Result<Value> {
    actor.owner()?;
    let (_, _, intent) = strategy::current(tx, envelope.task).await?;
    validate_work(&delegation.work)?;
    let work_id = crate::validation::digest(&delegation.work)?;
    if delegation.role_epoch <= 0
        || delegation.max_intents < 1
        || delegation.max_plans < 1
        || !(1..=86400).contains(&delegation.expires_in_seconds)
        || delegation.families.is_empty()
        || !unique(&delegation.families)
        || delegation.families.iter().any(|f| {
            ![
                "intent.admit",
                "plan.admit",
                "dependency.revalidate",
                "outcome.accept",
            ]
            .contains(&f.as_str())
        })
        || !unique(&delegation.effects)
        || delegation
            .effects
            .iter()
            .any(|e| !intent.allowed_effects.contains(e))
        || !strategy::within(&delegation.resources, &intent.resources)
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    if !tx.query_one("SELECT EXISTS(SELECT 1 FROM contract_work w JOIN mission_leads l ON l.mission_id=w.mission_id WHERE w.id=$1 AND w.task_id=$2 AND l.principal_id=$3 AND l.generation=$4)",&[&work_id,&envelope.task,&delegation.principal,&delegation.role_epoch]).await?.get::<_,bool>(0) {return Err(err(ErrorCode::StaleGeneration));}
    if tx
        .query_opt(
            "SELECT id FROM contract_delegations WHERE id=$1",
            &[&delegation.id],
        )
        .await?
        .is_some()
    {
        return Err(err(ErrorCode::IdempotencyConflict));
    }
    tx.execute("INSERT INTO contract_delegations(id,task_id,mission_id,principal_id,role_epoch,work_id,delegation,by_principal,recovery_epoch,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,clock_timestamp()+($10::bigint*interval '1 second'))",&[&delegation.id,&envelope.task,&mission,&delegation.principal,&delegation.role_epoch,&work_id,&serde_json::to_value(delegation).map_err(|_|err(ErrorCode::InvalidArgument))?,&actor.principal,&envelope.recovery_epoch,&delegation.expires_in_seconds]).await?;
    tx.execute(
        "UPDATE contract_current SET revision=$2 WHERE task_id=$1",
        &[&envelope.task, &(revision + 1)],
    )
    .await?;
    Ok(
        json!({"schema_version":2,"disposition":"applied","revision":revision+1,"delegation":delegation.id}),
    )
}
