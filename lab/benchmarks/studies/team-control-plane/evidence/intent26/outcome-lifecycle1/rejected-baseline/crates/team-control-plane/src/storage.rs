use crate::{
    auth::{self, Actor},
    domain::*,
    validation,
};
use tokio_postgres::{Client, NoTls, Transaction};
use uuid::Uuid;
/// PostgreSQL broker. Debug deliberately excludes DSN and all bearer material.
#[derive(Clone)]
pub struct Broker {
    pub(crate) dsn: String,
    pub(crate) epoch: Uuid,
}
impl std::fmt::Debug for Broker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Broker")
            .field("recovery_epoch", &self.epoch)
            .finish_non_exhaustive()
    }
}
pub(crate) async fn client(dsn: &str) -> Result<Client> {
    let mut config: tokio_postgres::Config =
        dsn.parse().map_err(|_| err(ErrorCode::InvalidArgument))?;
    if config.get_hosts().is_empty()
        || config.get_hostaddrs().iter().any(|ip| !ip.is_loopback())
        || config.get_hosts().iter().any(|host| match host {
            tokio_postgres::config::Host::Tcp(host) => {
                host != "localhost"
                    && host
                        .parse::<std::net::IpAddr>()
                        .ok()
                        .is_none_or(|ip| !ip.is_loopback())
            }
            #[cfg(unix)]
            tokio_postgres::config::Host::Unix(_) => false,
        })
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    config.connect_timeout(std::time::Duration::from_secs(5));
    let (client, connection) = config.connect(NoTls).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok(client)
}
pub async fn migrate(dsn: &str) -> Result<()> {
    let mut client = client(dsn).await?;
    let tx = client.transaction().await?;
    lock(&tx).await?;
    let exists: bool = tx
        .query_one("SELECT to_regclass('schema_version') IS NOT NULL", &[])
        .await?
        .get(0);
    if exists {
        schema(&tx).await?;
    } else {
        tx.batch_execute(include_str!("../migrations/0001.sql"))
            .await?;
    }
    tx.commit().await?;
    crate::artifacts::migrate(dsn).await?;
    let tx = client.transaction().await?;
    lock(&tx).await?;
    crate::gateway::migrate(&tx).await?;
    crate::temporal::migrate(&tx).await?;
    crate::supervisor::migrate(&tx).await?;
    crate::mailbox::migrate(&tx).await?;
    crate::kernel::migrate(&tx).await?;
    crate::contracts::migrate(&tx).await?;
    tx.commit().await?;
    Ok(())
}
async fn schema(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one("SELECT to_regclass('schema_version') IS NOT NULL", &[])
        .await?
        .get(0);
    if !exists {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    let rows = tx.query("SELECT version FROM schema_version", &[]).await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
pub(crate) async fn lock(tx: &Transaction<'_>) -> Result<()> {
    tx.batch_execute("SET LOCAL lock_timeout='5s'; SET LOCAL statement_timeout='10s'; SELECT pg_advisory_xact_lock(784092110)").await?;
    Ok(())
}
pub(crate) async fn active(tx: &Transaction<'_>, epoch: Uuid) -> Result<()> {
    let row = tx
        .query_opt(
            "SELECT active_epoch FROM deployment WHERE singleton=true",
            &[],
        )
        .await?;
    if row.map(|r| r.get::<_, Uuid>(0)) != Some(epoch) {
        return Err(err(ErrorCode::Unauthorized));
    }
    Ok(())
}
impl Broker {
    pub async fn connect(dsn: &str, recovery_epoch: Uuid) -> Result<Self> {
        let mut c = client(dsn).await?;
        let tx = c.transaction().await?;
        schema(&tx).await?;
        crate::mailbox::schema(&tx).await?;
        crate::kernel::schema(&tx).await?;
        crate::contracts::schema(&tx).await?;
        tx.commit().await?;
        Ok(Self {
            dsn: dsn.into(),
            epoch: recovery_epoch,
        })
    }
    /// Privileged local bootstrap; only an empty deployment may be initialized.
    pub async fn bootstrap_owner(&self) -> Result<String> {
        let mut c = client(&self.dsn).await?;
        let tx = c.transaction().await?;
        lock(&tx).await?;
        schema(&tx).await?;
        if tx
            .query_opt("SELECT active_epoch FROM deployment", &[])
            .await?
            .is_some()
        {
            return Err(err(ErrorCode::Unauthorized));
        }
        tx.execute(
            "INSERT INTO deployment_epochs(epoch) VALUES($1)",
            &[&self.epoch],
        )
        .await?;
        tx.execute(
            "INSERT INTO deployment(active_epoch) VALUES($1)",
            &[&self.epoch],
        )
        .await?;
        let token = owner_token(&tx, self.epoch).await?;
        tx.commit().await?;
        Ok(token)
    }
    /// Explicit privileged epoch rotation invalidates every existing grant.
    pub async fn rotate_epoch(&self, bearer: &str, new_epoch: Uuid) -> Result<String> {
        if new_epoch == self.epoch {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut c = client(&self.dsn).await?;
        let tx = c.transaction().await?;
        lock(&tx).await?;
        active(&tx, self.epoch).await?;
        auth::authenticate(&tx, bearer, self.epoch).await?.owner()?;
        if tx
            .query_opt(
                "SELECT epoch FROM deployment_epochs WHERE epoch=$1",
                &[&new_epoch],
            )
            .await?
            .is_some()
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        tx.execute(
            "INSERT INTO deployment_epochs(epoch) VALUES($1)",
            &[&new_epoch],
        )
        .await?;
        tx.execute(
            "UPDATE deployment SET active_epoch=$1 WHERE singleton=true",
            &[&new_epoch],
        )
        .await?;
        let token = owner_token(&tx, new_epoch).await?;
        tx.commit().await?;
        Ok(token)
    }
    pub async fn execute(&self, bearer: &str, envelope: CommandEnvelope) -> Result<CommandResult> {
        let mut c = client(&self.dsn).await?;
        let tx = c.transaction().await?;
        lock(&tx).await?;
        schema(&tx).await?;
        active(&tx, self.epoch).await?;
        let result = self.execute_in_transaction(&tx, bearer, envelope).await;
        // Domain rejections are journaled; infrastructure/auth failures roll back.
        if !matches!(&result, Err(e) if matches!(e.code, ErrorCode::TemporarilyUnavailable | ErrorCode::Unauthorized | ErrorCode::IncompatibleSchema))
        {
            tx.commit().await?;
        }
        result
    }
    pub(crate) async fn execute_in_transaction(
        &self,
        tx: &Transaction<'_>,
        bearer: &str,
        mut envelope: CommandEnvelope,
    ) -> Result<CommandResult> {
        if envelope.schema_version != 1 || envelope.key.is_empty() || envelope.key.len() > 256 {
            return Err(err(ErrorCode::InvalidArgument));
        }
        if envelope.recovery_epoch != self.epoch {
            return Err(err(ErrorCode::Unauthorized));
        }
        let actor = auth::authenticate(tx, bearer, self.epoch).await?;
        validate_actor_generation(tx, &actor).await?;
        let command_scope = auth::authorize_command(tx, &actor, &envelope.command).await?;
        let (scope_kind, scope_mission, scope_task) = command_scope.columns();
        match &mut envelope.command {
            Command::SetDependencies { dependencies, .. } => dependencies.sort(),
            Command::CreateTask { spec, .. } => spec.dependencies.sort(),
            _ => {}
        }
        let family = envelope.command.family();
        let digest = validation::digest(&envelope.command)?;
        let recorded = tx.query_opt(
            "SELECT argument_digest,result,disposition,scope_kind,scope_mission,scope_task FROM commands WHERE principal_id=$1 AND family=$2 AND key=$3",
            &[&actor.principal, &family, &envelope.key],
        ).await?;
        if let Some(row) = recorded {
            auth::RecordScope::from_columns(
                row.get("scope_kind"),
                row.get("scope_mission"),
                row.get("scope_task"),
            )?
            .authorize(&actor)?;
            if row.get::<_, String>(0) != digest {
                return Err(err(ErrorCode::IdempotencyConflict));
            }
            let value: serde_json::Value = row.get(1);
            if row.get::<_, String>(2) == "rejected" {
                let rejection: Error = serde_json::from_value(value)
                    .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
                return Err(rejection);
            }
            let result = serde_json::from_value(value)
                .map_err(|_| err(ErrorCode::TemporarilyUnavailable))?;
            return Ok(result);
        }
        tx.batch_execute("SAVEPOINT command_mutation").await?;
        let outcome = match validation::validate(&envelope.command) {
            Ok(()) => crate::admission::apply(tx, &actor, &envelope, digest.clone()).await,
            Err(error) => Err(error),
        };
        let (result, _mission) = match outcome {
            Ok(applied) => applied,
            Err(rejection) => {
                if matches!(
                    rejection.code,
                    ErrorCode::TemporarilyUnavailable
                        | ErrorCode::Unauthorized
                        | ErrorCode::IncompatibleSchema
                ) {
                    // Temporal may persist an authorization rejection in this same
                    // transaction. Never leave partial mutation behind it.
                    if rejection.code == ErrorCode::Unauthorized {
                        tx.batch_execute("ROLLBACK TO SAVEPOINT command_mutation")
                            .await?;
                    }
                    return Err(rejection);
                }
                tx.batch_execute("ROLLBACK TO SAVEPOINT command_mutation")
                    .await?;
                let aggregate = envelope.command.aggregate();
                tx.execute("INSERT INTO commands(principal_id,family,key,argument_digest,aggregate_id,scope_kind,scope_mission,scope_task,disposition,result) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'rejected',$9)",&[&actor.principal,&family,&envelope.key,&digest,&aggregate,&scope_kind,&scope_mission,&scope_task,&serde_json::to_value(&rejection).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?]).await?;
                return Err(rejection);
            }
        };
        let aggregate = envelope.command.aggregate();
        tx.execute("INSERT INTO commands(principal_id,family,key,argument_digest,aggregate_id,scope_kind,scope_mission,scope_task,disposition,result) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'applied',$9)",&[&actor.principal,&family,&envelope.key,&digest,&aggregate,&scope_kind,&scope_mission,&scope_task,&serde_json::to_value(&result).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?]).await?;
        Ok(result)
    }
    pub async fn snapshot(&self, bearer: &str, query: SnapshotQuery) -> Result<Snapshot> {
        let mut c = client(&self.dsn).await?;
        let tx = c.transaction().await?;
        lock(&tx).await?;
        schema(&tx).await?;
        active(&tx, self.epoch).await?;
        let actor = auth::authenticate(&tx, bearer, self.epoch).await?;
        validate_actor_generation(&tx, &actor).await?;
        let (revision, data) = match query {
            SnapshotQuery::Mission(id) => {
                actor.scope(id, None)?;
                let row=tx.query_opt("SELECT revision,jsonb_build_object('id',id,'objective',objective,'input_digest',input_digest,'git_repository',git_repository,'git_commit',git_commit,'git_path',git_path,'max_repairs',max_repairs,'episode_deadline_seconds',deadline_seconds,'capacity',capacity,'used_capacity',used_capacity,'admitted_attempts',admitted_attempts,'admitted_episodes',admitted_episodes,'max_attempts',max_attempts,'max_episodes',max_episodes,'dispatch_paused',coalesce((SELECT paused FROM kernel_mission_control WHERE mission_id=missions.id),false),'control_revision',coalesce((SELECT revision FROM kernel_mission_control WHERE mission_id=missions.id),0)) FROM missions WHERE id=$1",&[&id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
                (row.get(0), row.get(1))
            }
            SnapshotQuery::Task(id) => {
                let row=tx.query_opt("SELECT mission_id,revision,jsonb_build_object('id',t.id,'mission_id',mission_id,'status',status,'generation',generation,'spec_digest',spec_digest,'approved_digest',approved_digest,'spec',(SELECT to_jsonb(s) FROM task_specs s WHERE s.task_id=t.id AND s.digest=t.spec_digest),'spec_history',(SELECT coalesce(jsonb_agg(to_jsonb(s) ORDER BY s.created_at,s.digest),'[]') FROM task_specs s WHERE s.task_id=t.id),'events',(SELECT coalesce(jsonb_agg(to_jsonb(e) ORDER BY e.sequence),'[]') FROM events e WHERE e.aggregate_kind='task' AND e.aggregate_id=t.id),'dependencies',(SELECT coalesce(jsonb_agg(dependency_id),'[]') FROM dependencies WHERE task_id=t.id),'attempt_count',(SELECT count(*) FROM attempts WHERE task_id=t.id),'reservation_count',(SELECT count(*) FROM reservations r JOIN attempts a ON a.id=r.attempt_id WHERE a.task_id=t.id),'operation_count',(SELECT count(*) FROM operations o JOIN attempts a ON a.id=o.attempt_id WHERE a.task_id=t.id),'attempts',(SELECT coalesce(jsonb_agg(to_jsonb(a)),'[]') FROM attempts a WHERE a.task_id=t.id),'reservations',(SELECT coalesce(jsonb_agg(to_jsonb(r)),'[]') FROM reservations r JOIN attempts a ON a.id=r.attempt_id WHERE a.task_id=t.id),'operations',(SELECT coalesce(jsonb_agg(to_jsonb(o)),'[]') FROM operations o JOIN attempts a ON a.id=o.attempt_id WHERE a.task_id=t.id),'outbox_count',(SELECT count(*) FROM outbox o JOIN operations op ON op.id=o.operation_id JOIN attempts a ON a.id=op.attempt_id WHERE a.task_id=t.id)) FROM tasks t WHERE id=$1",&[&id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
                actor.scope(row.get(0), Some(id))?;
                (row.get(1), row.get(2))
            }
            SnapshotQuery::Command(principal, family, key) => {
                if actor.role != Role::Owner && actor.principal != principal {
                    return Err(err(ErrorCode::Unauthorized));
                }
                let row=tx.query_opt("SELECT scope_kind,scope_mission,scope_task,result FROM commands WHERE principal_id=$1 AND family=$2 AND key=$3",&[&principal,&family,&key]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
                auth::RecordScope::from_columns(
                    row.get("scope_kind"),
                    row.get("scope_mission"),
                    row.get("scope_task"),
                )?
                .authorize(&actor)?;
                let value: serde_json::Value = row.get("result");
                (value["revision"].as_i64().unwrap_or(0), value)
            }
        };
        let time: String = tx
            .query_one("SELECT clock_timestamp()::text", &[])
            .await?
            .get(0);
        tx.commit().await?;
        Ok(Snapshot {
            schema_version: 1,
            revision,
            snapshot_time: time,
            data,
        })
    }
}
async fn owner_token(tx: &Transaction<'_>, epoch: Uuid) -> Result<String> {
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = auth::token_hash(&token);
    tx.execute("INSERT INTO credentials(id,principal_id,role,recovery_epoch,token_hash,expires_at) VALUES($1,$2,'owner',$3,$4,clock_timestamp()+interval '24 hours')",&[&Uuid::new_v4(),&Uuid::new_v4(),&epoch,&hash]).await?;
    Ok(token)
}
pub(crate) async fn validate_actor_generation(tx: &Transaction<'_>, actor: &Actor) -> Result<()> {
    if let Some(task) = actor.task {
        let row = tx
            .query_opt("SELECT generation FROM tasks WHERE id=$1", &[&task])
            .await?
            .ok_or_else(|| err(ErrorCode::Unauthorized))?;
        if matches!(actor.role, Role::Worker | Role::Verifier)
            && actor.generation != Some(row.get::<_, i64>(0))
        {
            return Err(err(ErrorCode::StaleGeneration));
        }
    }
    Ok(())
}
