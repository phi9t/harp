use crate::{auth::Actor, domain::*, validation};
use tokio_postgres::Transaction;
use uuid::Uuid;
pub(crate) async fn apply(
    tx: &Transaction<'_>,
    actor: &Actor,
    envelope: &CommandEnvelope,
    digest: String,
) -> Result<(CommandResult, Option<Uuid>)> {
    let mut result = CommandResult {
        schema_version: 1,
        disposition: "applied".into(),
        revision: 1,
        generation: 0,
        recovery_epoch: envelope.recovery_epoch,
        attempt_id: None,
        reservation_id: None,
        operation_id: None,
    };
    let mission = match &envelope.command {
        Command::CreateMission {
            id,
            objective,
            input,
            policy,
        } => {
            actor.owner()?;
            revision(envelope, 0)?;
            if tx
                .query_opt("SELECT id FROM missions WHERE id=$1", &[id])
                .await?
                .is_some()
            {
                return Err(err(ErrorCode::StaleRevision));
            }
            tx.execute("INSERT INTO missions(id,objective,git_repository,git_commit,git_path,input_digest,revision,capacity,max_attempts,max_episodes,max_repairs,deadline_seconds) VALUES($1,$2,$3,$4,$5,$6,1,$7,$8,$9,$10,$11)",&[id,objective,&input.repository,&input.commit,&input.path,&input.digest,&policy.capacity,&policy.max_attempts,&policy.max_episodes,&policy.max_repairs,&policy.episode_deadline_seconds]).await?;
            Some(*id)
        }
        Command::CreateTask {
            id,
            mission_id,
            spec,
        } => {
            actor.owner()?;
            revision(envelope, 0)?;
            if tx
                .query_opt("SELECT id FROM missions WHERE id=$1", &[mission_id])
                .await?
                .is_none()
            {
                return Err(err(ErrorCode::NotFound));
            }
            if tx
                .query_opt("SELECT id FROM tasks WHERE id=$1", &[id])
                .await?
                .is_some()
            {
                return Err(err(ErrorCode::StaleRevision));
            }
            let mut spec = spec.clone();
            spec.dependencies.sort();
            validate_dependencies(tx, *id, *mission_id, &spec.dependencies).await?;
            let spec_digest = validation::digest(&spec)?;
            tx.execute("INSERT INTO tasks(id,mission_id,revision,status,spec_digest) VALUES($1,$2,1,'proposed',$3)",&[id,mission_id,&spec_digest]).await?;
            save_spec(tx, *id, *mission_id, &spec).await?;
            project_dependencies(tx, *id, *mission_id, &spec.dependencies).await?;
            Some(*mission_id)
        }
        Command::GrantCredential {
            credential_id,
            principal_id,
            role,
            mission_id,
            task_id,
            generation,
            token_hash,
            expires_in_seconds,
        } => {
            actor.owner()?;
            revision(envelope, 0)?;
            match role {
                Role::Owner => {
                    if mission_id.is_some() || task_id.is_some() || generation.is_some() {
                        return Err(err(ErrorCode::InvalidArgument));
                    }
                }
                Role::Lead => {
                    if mission_id.is_none() || generation.is_some() {
                        return Err(err(ErrorCode::InvalidArgument));
                    }
                }
                Role::Worker | Role::Verifier => {
                    if mission_id.is_none()
                        || task_id.is_none()
                        || generation.is_none_or(|g| g <= 0)
                    {
                        return Err(err(ErrorCode::InvalidArgument));
                    }
                }
            }
            if let Some(mission) = mission_id
                && tx
                    .query_opt("SELECT id FROM missions WHERE id=$1", &[mission])
                    .await?
                    .is_none()
            {
                return Err(err(ErrorCode::NotFound));
            }
            if let Some(task) = task_id {
                let row = tx
                    .query_opt(
                        "SELECT mission_id,generation FROM tasks WHERE id=$1",
                        &[task],
                    )
                    .await?
                    .ok_or_else(|| err(ErrorCode::NotFound))?;
                if Some(row.get::<_, Uuid>(0)) != *mission_id {
                    return Err(err(ErrorCode::Unauthorized));
                }
                if let Some(g) = generation {
                    let current: i64 = row.get(1);
                    if *g < current || *g > current + 1 {
                        return Err(err(ErrorCode::StaleGeneration));
                    }
                }
            }
            if *role == Role::Worker {
                tx.execute(
                    "INSERT INTO workers(id) VALUES($1) ON CONFLICT DO NOTHING",
                    &[principal_id],
                )
                .await?;
            }
            let lead_generation = if *role == Role::Lead {
                if let Some(current) = tx
                    .query_opt(
                        "SELECT generation,principal_id FROM mission_leads WHERE mission_id=$1",
                        &[mission_id],
                    )
                    .await?
                {
                    if current.get::<_, Uuid>(1) != *principal_id {
                        return Err(err(ErrorCode::StaleGeneration));
                    }
                    current.get::<_, i64>(0)
                } else {
                    0
                }
            } else {
                0
            };
            let role = match role {
                Role::Owner => "owner",
                Role::Lead => "lead",
                Role::Worker => "worker",
                Role::Verifier => "verifier",
            };
            tx.execute("INSERT INTO credentials(id,principal_id,role,mission_id,task_id,generation,recovery_epoch,token_hash,expires_at,lead_generation) VALUES($1,$2,$3,$4,$5,$6,$7,$8,clock_timestamp()+($9::bigint * interval '1 second'),$10)",&[credential_id,principal_id,&role,mission_id,task_id,generation,&envelope.recovery_epoch,token_hash,expires_in_seconds,&lead_generation]).await?;
            *mission_id
        }
        Command::SetDependencies {
            task_id,
            dependencies,
        } => {
            actor.owner()?;
            let task = load_task(tx, *task_id).await?;
            actor.scope(task.mission, Some(*task_id))?;
            revision(envelope, task.revision)?;
            generation(envelope, task.generation)?;
            if task.status != "proposed" {
                return Err(err(ErrorCode::InvalidArgument));
            }
            validate_dependencies(tx, *task_id, task.mission, dependencies).await?;
            let mut spec = load_spec(tx, *task_id, &task.digest).await?;
            spec.dependencies = dependencies.clone();
            spec.dependencies.sort();
            let spec_digest = save_spec(tx, *task_id, task.mission, &spec).await?;
            project_dependencies(tx, *task_id, task.mission, &spec.dependencies).await?;
            result.revision = task.revision + 1;
            result.generation = task.generation;
            tx.execute(
                "UPDATE tasks SET revision=$2,spec_digest=$3 WHERE id=$1",
                &[task_id, &result.revision, &spec_digest],
            )
            .await?;
            Some(task.mission)
        }
        Command::AdmitTask {
            task_id,
            spec_digest,
        } => {
            actor.owner()?;
            let task = load_task(tx, *task_id).await?;
            revision(envelope, task.revision)?;
            generation(envelope, task.generation)?;
            if task.status != "proposed" || task.digest != *spec_digest {
                return Err(err(ErrorCode::InvalidArgument));
            }
            result.revision = task.revision + 1;
            result.generation = task.generation;
            tx.execute(
                "UPDATE tasks SET status='ready',approved_digest=$2,revision=$3 WHERE id=$1",
                &[task_id, spec_digest, &result.revision],
            )
            .await?;
            Some(task.mission)
        }
        Command::CancelAttempt {
            task_id,
            attempt_id,
            ..
        }
        | Command::ReconcileAttempt {
            task_id,
            attempt_id,
        } => {
            let task = load_task(tx, *task_id).await?;
            revision(envelope, task.revision)?;
            generation(envelope, task.generation)?;
            if task.status == "accepted" {
                return Err(err(ErrorCode::DependencyUnmet));
            }
            let attempt = tx
                .query_opt(
                    "SELECT task_id,desired_cancel FROM attempts WHERE id=$1 FOR UPDATE",
                    &[attempt_id],
                )
                .await?
                .ok_or_else(|| err(ErrorCode::NotFound))?;
            if attempt.get::<_, Uuid>(0) != *task_id {
                return Err(err(ErrorCode::Unauthorized));
            }
            if matches!(&envelope.command, Command::CancelAttempt { .. }) {
                if attempt.get::<_, bool>(1) {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                tx.execute("UPDATE attempts SET desired_cancel=true,status='cancellation_requested' WHERE id=$1", &[attempt_id]).await?;
                tx.execute("UPDATE supervised_workspaces w SET desired_cancel=true,state=CASE WHEN state IN ('running','starting') THEN 'cancelling' ELSE state END WHERE w.attempt_id=$1 OR EXISTS(SELECT 1 FROM episode_executions x WHERE x.execution_id=w.id AND x.attempt_id=$1)", &[attempt_id]).await?;
                tx.execute("UPDATE mailbox_episodes SET state='cancellation_requested' WHERE attempt_id=$1 AND state IN ('admitted','active','yielded')", &[attempt_id]).await?;
                result.disposition = "cancellation_requested".into();
            } else {
                if !attempt.get::<_, bool>(1) {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                result.disposition =
                    crate::supervisor::reconcile_attempt_cancel(tx, *attempt_id).await?;
            }
            result.revision = task.revision + 1;
            result.generation = task.generation;
            result.attempt_id = Some(*attempt_id);
            tx.execute("UPDATE tasks SET revision=$2,status=CASE WHEN $3='cancelled' THEN 'cancelled' ELSE status END WHERE id=$1", &[task_id,&result.revision,&result.disposition]).await?;
            Some(task.mission)
        }
        Command::AssignAttempt { task_id, worker_id } => {
            let task = load_task(tx, *task_id).await?;
            actor.scope(task.mission, Some(*task_id))?;
            if !matches!(actor.role, Role::Owner | Role::Lead) {
                return Err(err(ErrorCode::Unauthorized));
            }
            revision(envelope, task.revision)?;
            generation(envelope, task.generation)?;
            if task.status != "ready" {
                return Err(err(ErrorCode::DependencyUnmet));
            }
            let has_dependencies: bool = tx
                .query_one(
                    "SELECT EXISTS(SELECT 1 FROM dependencies WHERE task_id=$1)",
                    &[task_id],
                )
                .await?
                .get(0);
            if has_dependencies {
                return Err(err(ErrorCode::DependencyUnmet));
            }
            let next = task.generation + 1;
            if tx.query_opt("SELECT id FROM credentials WHERE principal_id=$1 AND role='worker' AND mission_id=$2 AND task_id=$3 AND generation=$4 AND recovery_epoch=$5 AND expires_at>clock_timestamp()",&[worker_id,&task.mission,task_id,&next,&envelope.recovery_epoch]).await?.is_none(){return Err(err(ErrorCode::Unauthorized));}
            let m=tx.query_one("SELECT capacity,used_capacity,max_attempts,admitted_attempts,max_episodes,admitted_episodes FROM missions WHERE id=$1 FOR UPDATE",&[&task.mission]).await?;
            if m.get::<_, i64>("used_capacity") >= m.get::<_, i64>("capacity") {
                return Err(err(ErrorCode::CapacityExhausted));
            }
            if m.get::<_, i64>("admitted_attempts") >= m.get::<_, i64>("max_attempts")
                || m.get::<_, i64>("admitted_episodes") >= m.get::<_, i64>("max_episodes")
            {
                return Err(err(ErrorCode::BudgetExhausted));
            }
            let attempt = Uuid::new_v4();
            let reservation = Uuid::new_v4();
            let operation = Uuid::new_v4();
            tx.execute("UPDATE missions SET used_capacity=used_capacity+1,admitted_attempts=admitted_attempts+1,admitted_episodes=admitted_episodes+1,revision=revision+1 WHERE id=$1",&[&task.mission]).await?;
            tx.execute("INSERT INTO attempts(id,task_id,worker_id,spec_digest,generation,recovery_epoch,status) VALUES($1,$2,$3,$4,$5,$6,'reserved')",&[&attempt,task_id,worker_id,&task.digest,&next,&envelope.recovery_epoch]).await?;
            tx.execute("INSERT INTO reservations(id,attempt_id,mission_id,attempts,episodes,state) VALUES($1,$2,$3,1,1,'reserved')",&[&reservation,&attempt,&task.mission]).await?;
            tx.execute("INSERT INTO operations(id,attempt_id,kind,state,argument_digest,generation,recovery_epoch) VALUES($1,$2,'prepare_attempt','prepared',$3,$4,$5)",&[&operation,&attempt,&digest,&next,&envelope.recovery_epoch]).await?;
            result.revision = task.revision + 1;
            result.generation = next;
            result.attempt_id = Some(attempt);
            result.reservation_id = Some(reservation);
            result.operation_id = Some(operation);
            tx.execute(
                "UPDATE tasks SET status='running',generation=$2,revision=$3 WHERE id=$1",
                &[task_id, &next, &result.revision],
            )
            .await?;
            Some(task.mission)
        }
    };
    let event = Uuid::new_v4();
    let aggregate = envelope.command.aggregate();
    let spec_digest: Option<String> = if envelope.command.aggregate_kind() == "task" {
        tx.query_opt("SELECT spec_digest FROM tasks WHERE id=$1", &[&aggregate])
            .await?
            .map(|row| row.get(0))
    } else {
        None
    };
    let payload = serde_json::to_value(
        serde_json::json!({"result":result,"command":envelope.command,"spec_digest":spec_digest}),
    )
    .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
    tx.execute("INSERT INTO events(id,aggregate_kind,aggregate_id,sequence,version,principal_id,transaction_id,kind,payload) VALUES($1,$2,$3,$4,1,$5,txid_current(),$6,$7)",&[&event,&envelope.command.aggregate_kind(),&aggregate,&result.revision,&actor.principal,&envelope.command.family(),&payload]).await?;
    if let Some(operation) = result.operation_id {
        tx.execute("INSERT INTO outbox(id,event_id,operation_id,version,state) VALUES($1,$2,$3,1,'pending')",&[&Uuid::new_v4(),&event,&operation]).await?;
    }
    Ok((result, mission))
}
fn revision(envelope: &CommandEnvelope, current: i64) -> Result<()> {
    if envelope.expected_revision != current {
        Err(err(ErrorCode::StaleRevision))
    } else {
        Ok(())
    }
}
fn generation(envelope: &CommandEnvelope, current: i64) -> Result<()> {
    if envelope.expected_generation.is_some_and(|g| g != current) {
        Err(err(ErrorCode::StaleGeneration))
    } else {
        Ok(())
    }
}
struct Task {
    mission: Uuid,
    revision: i64,
    generation: i64,
    status: String,
    digest: String,
}
async fn load_task(tx: &Transaction<'_>, id: Uuid) -> Result<Task> {
    let row=tx.query_opt("SELECT mission_id,revision,generation,status,spec_digest FROM tasks WHERE id=$1 FOR UPDATE",&[&id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
    Ok(Task {
        mission: row.get(0),
        revision: row.get(1),
        generation: row.get(2),
        status: row.get(3),
        digest: row.get(4),
    })
}

async fn validate_dependencies(
    tx: &Transaction<'_>,
    task_id: Uuid,
    mission_id: Uuid,
    dependencies: &[Uuid],
) -> Result<()> {
    let mut unique = dependencies.to_vec();
    unique.sort();
    unique.dedup();
    if unique.len() != dependencies.len() {
        return Err(err(ErrorCode::InvalidArgument));
    }
    for dependency in dependencies {
        if *dependency == task_id {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let other = load_task(tx, *dependency).await?;
        if other.mission != mission_id {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        let cycle:bool=tx.query_one("WITH RECURSIVE reachable(id) AS (SELECT dependency_id FROM dependencies WHERE task_id=$1 UNION SELECT d.dependency_id FROM dependencies d JOIN reachable r ON d.task_id=r.id) SELECT EXISTS(SELECT 1 FROM reachable WHERE id=$2)",&[dependency,&task_id]).await?.get(0);
        if cycle {
            return Err(err(ErrorCode::DependencyUnmet));
        }
    }
    Ok(())
}
async fn load_spec(tx: &Transaction<'_>, task_id: Uuid, digest: &str) -> Result<TaskSpec> {
    let row = tx
        .query_one(
            "SELECT * FROM task_specs WHERE task_id=$1 AND digest=$2",
            &[&task_id, &digest],
        )
        .await?;
    let vec = |name: &str| {
        serde_json::from_value(row.get(name)).map_err(|_| err(ErrorCode::TemporarilyUnavailable))
    };
    Ok(TaskSpec {
        objective: row.get("objective"),
        input: GitInput {
            repository: row.get("git_repository"),
            commit: row.get("git_commit"),
            path: row.get("git_path"),
            digest: row.get("input_digest"),
        },
        acceptance: vec("acceptance")?,
        deliverables: vec("deliverables")?,
        write_scope: vec("write_scope")?,
        dependencies: serde_json::from_value(row.get("dependencies"))
            .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?,
    })
}
async fn save_spec(
    tx: &Transaction<'_>,
    task_id: Uuid,
    mission_id: Uuid,
    spec: &TaskSpec,
) -> Result<String> {
    let digest = validation::digest(spec)?;
    let inserted=tx.execute("INSERT INTO task_specs(task_id,digest,objective,git_repository,git_commit,git_path,input_digest,acceptance,deliverables,write_scope,dependencies) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) ON CONFLICT(task_id,digest) DO NOTHING",&[&task_id,&digest,&spec.objective,&spec.input.repository,&spec.input.commit,&spec.input.path,&spec.input.digest,&serde_json::json!(spec.acceptance),&serde_json::json!(spec.deliverables),&serde_json::json!(spec.write_scope),&serde_json::json!(spec.dependencies)]).await?;
    if inserted == 1 {
        for dependency in &spec.dependencies {
            tx.execute("INSERT INTO spec_dependencies(task_id,spec_digest,dependency_id,mission_id) VALUES($1,$2,$3,$4)",&[&task_id,&digest,dependency,&mission_id]).await?;
        }
    }
    Ok(digest)
}
async fn project_dependencies(
    tx: &Transaction<'_>,
    task_id: Uuid,
    mission_id: Uuid,
    dependencies: &[Uuid],
) -> Result<()> {
    tx.execute("DELETE FROM dependencies WHERE task_id=$1", &[&task_id])
        .await?;
    for dependency in dependencies {
        tx.execute(
            "INSERT INTO dependencies(task_id,dependency_id,mission_id) VALUES($1,$2,$3)",
            &[&task_id, dependency, &mission_id],
        )
        .await?;
    }
    Ok(())
}
