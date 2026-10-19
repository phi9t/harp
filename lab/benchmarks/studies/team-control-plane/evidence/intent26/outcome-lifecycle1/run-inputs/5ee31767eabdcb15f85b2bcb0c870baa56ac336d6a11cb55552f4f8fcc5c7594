use crate::domain::*;
use sha2::{Digest, Sha256};
use tokio_postgres::Transaction;
use uuid::Uuid;
pub fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
pub(crate) struct Actor {
    pub principal: Uuid,
    pub role: Role,
    pub mission: Option<Uuid>,
    pub task: Option<Uuid>,
    pub generation: Option<i64>,
}
pub(crate) async fn authenticate(tx: &Transaction<'_>, token: &str, epoch: Uuid) -> Result<Actor> {
    let row=tx.query_opt("SELECT principal_id,role,mission_id,task_id,generation,lead_generation FROM credentials WHERE token_hash=$1 AND recovery_epoch=$2 AND expires_at>clock_timestamp()", &[&token_hash(token),&epoch]).await?.ok_or_else(||err(ErrorCode::Unauthorized))?;
    let role: String = row.get(1);
    let role = match role.as_str() {
        "owner" => Role::Owner,
        "lead" => Role::Lead,
        "worker" => Role::Worker,
        "verifier" => Role::Verifier,
        _ => return Err(err(ErrorCode::Unauthorized)),
    };
    if role == Role::Lead {
        let mission: Option<Uuid> = row.get(2);
        let current = tx
            .query_opt(
                "SELECT generation,principal_id FROM mission_leads WHERE mission_id=$1",
                &[&mission],
            )
            .await?;
        let generation: i64 = row.get(5);
        if current.as_ref().map(|r| r.get::<_, i64>(0)).unwrap_or(0) != generation
            || current
                .as_ref()
                .is_some_and(|r| r.get::<_, Uuid>(1) != row.get::<_, Uuid>(0))
        {
            return Err(err(ErrorCode::StaleGeneration));
        }
    }
    Ok(Actor {
        principal: row.get(0),
        role,
        mission: row.get(2),
        task: row.get(3),
        generation: row.get(4),
    })
}
impl Actor {
    pub fn owner(&self) -> Result<()> {
        if self.role == Role::Owner {
            Ok(())
        } else {
            Err(err(ErrorCode::Unauthorized))
        }
    }
    pub fn scope(&self, mission: Uuid, task: Option<Uuid>) -> Result<()> {
        if self.role == Role::Owner
            || (self.mission == Some(mission) && (self.task.is_none() || self.task == task))
        {
            Ok(())
        } else {
            Err(err(ErrorCode::Unauthorized))
        }
    }
}

#[derive(Clone)]
pub(crate) enum RecordScope {
    Owner,
    Mission(Uuid),
    Task { mission: Uuid, task: Uuid },
}
impl RecordScope {
    pub fn columns(&self) -> (&'static str, Option<Uuid>, Option<Uuid>) {
        match self {
            Self::Owner => ("owner", None, None),
            Self::Mission(m) => ("mission", Some(*m), None),
            Self::Task { mission, task } => ("task", Some(*mission), Some(*task)),
        }
    }
    pub fn from_columns(kind: String, mission: Option<Uuid>, task: Option<Uuid>) -> Result<Self> {
        match (kind.as_str(), mission, task) {
            ("owner", None, None) => Ok(Self::Owner),
            ("mission", Some(m), None) => Ok(Self::Mission(m)),
            ("task", Some(m), Some(t)) => Ok(Self::Task {
                mission: m,
                task: t,
            }),
            _ => Err(err(ErrorCode::TemporarilyUnavailable)),
        }
    }
    pub fn authorize(&self, actor: &Actor) -> Result<()> {
        match self {
            Self::Owner => actor.owner(),
            Self::Mission(m) => actor.scope(*m, None),
            Self::Task { mission, task } => actor.scope(*mission, Some(*task)),
        }
    }
}
pub(crate) async fn authorize_command(
    tx: &Transaction<'_>,
    actor: &Actor,
    command: &Command,
) -> Result<RecordScope> {
    let task_id = match command {
        Command::CancelAttempt {
            task_id,
            attempt_id,
            ..
        }
        | Command::ReconcileAttempt {
            task_id,
            attempt_id,
        } => {
            let row=tx.query_opt("SELECT a.task_id,a.worker_id,a.generation,a.recovery_epoch FROM attempts a WHERE a.id=$1", &[attempt_id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
            if row.get::<_, Uuid>(0) != *task_id {
                return Err(err(ErrorCode::Unauthorized));
            }
            if actor.role != Role::Owner
                && (actor.role != Role::Worker
                    || actor.principal != row.get::<_, Uuid>(1)
                    || actor.generation != Some(row.get::<_, i64>(2)))
            {
                return Err(err(ErrorCode::Unauthorized));
            }
            // An Owner may reconcile historical cancellation under the current epoch.
            if actor.role != Role::Owner
                && row.get::<_, Uuid>(3)
                    != tx
                        .query_one("SELECT active_epoch FROM deployment WHERE singleton", &[])
                        .await?
                        .get::<_, Uuid>(0)
            {
                return Err(err(ErrorCode::Unauthorized));
            }
            Some(*task_id)
        }
        Command::AssignAttempt { task_id, .. } => {
            if !matches!(actor.role, Role::Owner | Role::Lead) {
                return Err(err(ErrorCode::Unauthorized));
            }
            Some(*task_id)
        }
        Command::SetDependencies { task_id, .. } | Command::AdmitTask { task_id, .. } => {
            actor.owner()?;
            Some(*task_id)
        }
        Command::CreateTask { id, mission_id, .. } => {
            actor.owner()?;
            return if tx
                .query_opt("SELECT id FROM missions WHERE id=$1", &[mission_id])
                .await?
                .is_some()
            {
                Ok(RecordScope::Task {
                    mission: *mission_id,
                    task: *id,
                })
            } else {
                Ok(RecordScope::Owner)
            };
        }
        _ => {
            actor.owner()?;
            None
        }
    };
    if let Some(task) = task_id {
        if let Some(row) = tx
            .query_opt("SELECT mission_id FROM tasks WHERE id=$1", &[&task])
            .await?
        {
            let scope = RecordScope::Task {
                mission: row.get(0),
                task,
            };
            scope.authorize(actor)?;
            return Ok(scope);
        }
        // An owner may request a missing resource, but the rejection keeps an
        // owner-only context rather than acquiring a future resource's scope.
        actor.owner()?;
    }
    Ok(RecordScope::Owner)
}
