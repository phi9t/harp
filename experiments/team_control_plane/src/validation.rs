use crate::*;
use sha2::{Digest, Sha256};
use std::path::{Component, Path};
pub(crate) fn digest(value: &impl serde::Serialize) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|_| domain::err(ErrorCode::InvalidArgument))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
pub(crate) fn hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub(crate) fn path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\\', '\0'])
        && Path::new(value)
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
        && !value
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
}
pub(crate) fn input(value: &GitInput) -> Result<()> {
    if value.repository.trim().is_empty()
        || value.repository.contains(['\n', '\r', '\0'])
        || !(hex(&value.commit, 40) || hex(&value.commit, 64))
        || !hex(&value.digest, 64)
        || !path(&value.path)
    {
        Err(domain::err(ErrorCode::InvalidArgument))
    } else {
        Ok(())
    }
}
pub(crate) fn validate(command: &Command) -> Result<()> {
    let invalid = || domain::err(ErrorCode::InvalidArgument);
    match command {
        Command::CreateMission {
            objective,
            input: i,
            policy,
            ..
        } => {
            input(i)?;
            if objective.trim().is_empty()
                || policy.capacity <= 0
                || policy.max_attempts <= 0
                || policy.max_episodes <= 0
                || policy.max_repairs < 0
                || policy.episode_deadline_seconds <= 0
                || policy.episode_deadline_seconds > 86400
            {
                return Err(invalid());
            }
        }
        Command::CreateTask { spec, .. } => {
            input(&spec.input)?;
            if spec.objective.trim().is_empty()
                || spec.acceptance.is_empty()
                || spec.deliverables.is_empty()
                || spec.write_scope.is_empty()
                || spec
                    .acceptance
                    .iter()
                    .chain(&spec.deliverables)
                    .any(|s| s.trim().is_empty())
                || spec.write_scope.iter().any(|s| !path(s))
            {
                return Err(invalid());
            }
        }
        Command::AdmitTask { spec_digest, .. } if !hex(spec_digest, 64) => {
            return Err(invalid());
        }
        Command::GrantCredential {
            token_hash,
            expires_in_seconds,
            ..
        } if (!hex(token_hash, 64) || *expires_in_seconds <= 0 || *expires_in_seconds > 86400) => {
            return Err(invalid());
        }
        Command::CancelAttempt { reason, .. }
            if reason.trim().is_empty() || reason.len() > 4096 =>
        {
            return Err(invalid());
        }
        _ => {}
    }
    Ok(())
}
impl Command {
    pub(crate) fn family(&self) -> &'static str {
        match self {
            Self::CreateMission { .. } => "mission.create",
            Self::CreateTask { .. } => "task.create",
            Self::SetDependencies { .. } => "task.dependencies",
            Self::AdmitTask { .. } => "task.admit",
            Self::GrantCredential { .. } => "credential.grant",
            Self::AssignAttempt { .. } => "attempt.assign",
            Self::CancelAttempt { .. } => "attempt.cancel",
            Self::ReconcileAttempt { .. } => "attempt.reconcile",
        }
    }
    pub(crate) fn aggregate_kind(&self) -> &'static str {
        match self {
            Self::CreateMission { .. } => "mission",
            Self::GrantCredential { .. } => "credential",
            _ => "task",
        }
    }
    pub(crate) fn aggregate(&self) -> uuid::Uuid {
        match self {
            Self::CreateMission { id, .. } | Self::CreateTask { id, .. } => *id,
            Self::SetDependencies { task_id, .. }
            | Self::AdmitTask { task_id, .. }
            | Self::AssignAttempt { task_id, .. }
            | Self::CancelAttempt { task_id, .. }
            | Self::ReconcileAttempt { task_id, .. } => *task_id,
            Self::GrantCredential { credential_id, .. } => *credential_id,
        }
    }
}
