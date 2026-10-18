//! Exact prerequisite proofs suspend fresh grants only on Broker admission.
//! Previously recorded operations retain their receipts and reconciliation seam.
use super::*;
pub(super) async fn validate(
    tx: &Transaction<'_>,
    task: Uuid,
    dependencies: &[Dependency],
) -> Result<()> {
    if dependencies.len() > 128 {
        return Err(err(ErrorCode::InvalidArgument));
    }
    let mut seen = std::collections::BTreeSet::new();
    for dependency in dependencies {
        validate_work(&dependency.work)?;
        let work = crate::validation::digest(&dependency.work)?;
        if !seen.insert(work.clone())
            || !digest(&dependency.intent)
            || !digest(&dependency.accepted_subject)
            || !text(&dependency.policy)
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let row=tx.query_opt("SELECT w.task_id,o.intent_id,o.policy,c.intent_id FROM contract_work w JOIN contract_outcomes o ON o.task_id=w.task_id JOIN contract_current c ON c.task_id=w.task_id WHERE w.id=$1 AND o.subject=$2",&[&work,&dependency.accepted_subject]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
        let prerequisite: Uuid = row.get(0);
        if prerequisite == task
            || row.get::<_, String>(1) != dependency.intent
            || row.get::<_, String>(3) != dependency.intent
            || row.get::<_, String>(2) != dependency.policy
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let cycle:bool=tx.query_one("WITH RECURSIVE reachable(task_id) AS (SELECT $1::uuid UNION SELECT w.task_id FROM reachable r JOIN contract_dependencies d ON d.task_id=r.task_id JOIN contract_work w ON w.id=d.prerequisite_work) SELECT EXISTS(SELECT 1 FROM reachable WHERE task_id=$2)",&[&prerequisite,&task]).await?.get(0);
        if cycle {
            return Err(err(ErrorCode::InvalidArgument));
        }
    }
    Ok(())
}
pub(super) async fn replace(
    tx: &Transaction<'_>,
    task: Uuid,
    dependencies: &[Dependency],
) -> Result<()> {
    tx.execute(
        "DELETE FROM contract_dependencies WHERE task_id=$1",
        &[&task],
    )
    .await?;
    for dependency in dependencies {
        let work = crate::validation::digest(&dependency.work)?;
        tx.execute("INSERT INTO contract_dependencies(task_id,prerequisite_work,intent_id,accepted_subject,policy,disposition) VALUES($1,$2,$3,$4,$5,'active')",&[&task,&work,&dependency.intent,&dependency.accepted_subject,&dependency.policy]).await?;
    }
    Ok(())
}
pub(super) async fn supersede(tx: &Transaction<'_>, work: &str, intent: &str) -> Result<()> {
    // No source event/proposal calls this. Historical outcome and receipt rows
    // stay immutable, even while their dependency projections become suspended.
    let changed=tx.query("UPDATE contract_dependencies SET disposition='suspended' WHERE prerequisite_work=$1 AND intent_id<>$2 AND disposition='active' RETURNING task_id",&[&work,&intent]).await?;
    for row in changed {
        tx.execute(
            "UPDATE contract_current SET revision=revision+1 WHERE task_id=$1",
            &[&row.get::<_, Uuid>(0)],
        )
        .await?;
    }
    Ok(())
}
pub(super) async fn revalidate(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    envelope: &Envelope,
    revision: i64,
    intent: &str,
    dependencies: &[Dependency],
) -> Result<Value> {
    let (current, _, _) = strategy::current(tx, envelope.task).await?;
    if current != intent {
        return Err(err(ErrorCode::StaleRevision));
    }
    let old = tx
        .query(
            "SELECT prerequisite_work,policy FROM contract_dependencies WHERE task_id=$1",
            &[&envelope.task],
        )
        .await?;
    if old.len() != dependencies.len() {
        return Err(err(ErrorCode::InvalidArgument));
    }
    for dependency in dependencies {
        let work = crate::validation::digest(&dependency.work)?;
        if !old
            .iter()
            .any(|r| r.get::<_, String>(0) == work && r.get::<_, String>(1) == dependency.policy)
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
    }
    validate(tx, envelope.task, dependencies).await?;
    let id = crate::validation::digest(&(envelope.task, intent, dependencies, revision))?;
    tx.execute("INSERT INTO contract_dependency_revalidations(id,task_id,intent_id,dependencies,principal_id,recovery_epoch) VALUES($1,$2,$3,$4,$5,$6)",&[&id,&envelope.task,&intent,&serde_json::to_value(dependencies).map_err(|_|err(ErrorCode::InvalidArgument))?,&actor.principal,&envelope.recovery_epoch]).await?;
    replace(tx, envelope.task, dependencies).await?;
    tx.execute(
        "UPDATE contract_current SET revision=$2 WHERE task_id=$1",
        &[&envelope.task, &(revision + 1)],
    )
    .await?;
    Ok(json!({"schema_version":2,"disposition":"applied","revision":revision+1,"revalidation":id}))
}
pub(crate) async fn fresh_grant(tx: &Transaction<'_>, task: Uuid) -> Result<()> {
    let Some(row)=tx.query_opt("SELECT c.admission_open,i.intent,t.spec_digest FROM contract_current c JOIN contract_intents i ON i.id=c.intent_id JOIN tasks t ON t.id=c.task_id WHERE c.task_id=$1",&[&task]).await? else {return Ok(());};
    let intent: IntentRevision =
        serde_json::from_value(row.get(1)).map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
    if !row.get::<_, bool>(0) || row.get::<_, String>(2) != intent.legacy_spec_digest {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let closed:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM contract_dependencies d JOIN contract_work w ON w.id=d.prerequisite_work JOIN contract_current c ON c.task_id=w.task_id WHERE d.task_id=$1 AND (d.disposition<>'active' OR d.intent_id<>c.intent_id)) OR EXISTS(SELECT 1 FROM contract_outcomes o JOIN contract_current c ON c.intent_id=o.intent_id WHERE c.task_id=$1)",&[&task]).await?.get(0);
    if closed {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    Ok(())
}
