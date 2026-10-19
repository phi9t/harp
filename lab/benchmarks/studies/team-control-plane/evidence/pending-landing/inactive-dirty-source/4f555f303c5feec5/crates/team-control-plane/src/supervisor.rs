//! Durable local Git workspaces and independently supervised PID namespaces.
//! No cgroup enforcement is claimed. Unprovable membership holds capacity.
use crate::{auth, storage, *};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tokio_postgres::Transaction;
use uuid::Uuid;
fn unavailable() -> Error {
    crate::domain::err(ErrorCode::TemporarilyUnavailable)
}
fn invalid() -> Error {
    crate::domain::err(ErrorCode::InvalidArgument)
}
#[derive(Clone, Copy, Debug)]
pub enum CreateFault {
    None,
    AfterRegister,
    AfterCreate,
    DuringGit,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    File,
    Directory,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteScope {
    pub path: String,
    pub kind: ScopeKind,
}
#[derive(Clone, Copy, Debug)]
pub enum StartFault {
    None,
    BeforeMonitor,
}
#[derive(Clone)]
pub struct MonitorGate {
    pub claimed: std::sync::Arc<tokio::sync::Notify>,
    pub resume: std::sync::Arc<tokio::sync::Notify>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Registered,
    Preparing,
    Ready,
    Starting,
    Running,
    Cancelling,
    Stopped,
    Quarantined,
    Deleting,
    Released,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionHandle {
    pub id: Uuid,
    pub path: PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub start_ticks: u64,
    pub boot_id: String,
    pub namespace: String,
    #[serde(default)]
    pub namespace_pid: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub store: PathBuf,
    pub path: PathBuf,
    pub base: String,
    pub attempt: Option<Uuid>,
    pub task: Option<Uuid>,
    #[serde(default)]
    pub episode: Option<Uuid>,
    #[serde(default)]
    pub mission: Option<Uuid>,
    pub generation: i64,
    pub recovery_epoch: Uuid,
    pub spec_digest: String,
    pub write_scope: Vec<String>,
    pub scopes: Vec<WriteScope>,
    pub log_path: PathBuf,
    #[serde(default)]
    pub candidate: Option<crate::artifacts::Candidate>,
    /// Captured under the allocation lock; missing historical bindings cannot
    /// establish a successful gate under a subsequently configured policy.
    #[serde(default)]
    pub verification_policy: Option<VerificationPolicy>,
    #[serde(default)]
    pub research_verification: Option<crate::contracts::lifecycle::ResearchVerification>,
    #[serde(default)]
    pub input: Option<EpisodeInput>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub suite: String,
    pub environment: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodeInput {
    pub id: Uuid,
    pub digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionView {
    pub state: ExecutionState,
    pub members: Vec<ProcessIdentity>,
    pub controller: Option<ProcessIdentity>,
    pub launcher: Option<ProcessIdentity>,
    pub quarantine_reason: Option<String>,
    pub manifest: WorkspaceManifest,
}
/// Trusted monitor observation of the launcher wait status. Namespace shutdown
/// and successful workload completion are separate facts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution: Uuid,
    pub argv: Vec<String>,
    pub candidate: Option<crate::artifacts::Candidate>,
    pub recovery_epoch: Uuid,
    pub controller: ProcessIdentity,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub namespace_stopped: bool,
    pub logs: String,
    #[serde(default)]
    pub completion: CompletionProvenance,
}
/// Original wait status and recovered shutdown are distinct evidence.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompletionProvenance {
    #[default]
    MonitorWait,
    RecoveredStop {
        finalizer: ProcessIdentity,
        recovery_epoch: Uuid,
        stop_observation: Uuid,
        init_process: ProcessIdentity,
    },
}
#[derive(Clone, Debug)]
pub struct Supervisor {
    broker: Broker,
    root: PathBuf,
    monitor: PathBuf,
}
pub(crate) async fn migrate(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('supervisor_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        tx.batch_execute(include_str!("../migrations/0005.sql"))
            .await?;
    }
    schema(tx).await
}
async fn schema(tx: &Transaction<'_>) -> Result<()> {
    let exists: bool = tx
        .query_one(
            "SELECT to_regclass('supervisor_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get(0);
    if !exists {
        return Err(crate::domain::err(ErrorCode::IncompatibleSchema));
    }
    let rows = tx
        .query("SELECT version FROM supervisor_schema_version", &[])
        .await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
        return Err(crate::domain::err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
fn json<T: Serialize>(v: &T) -> Result<serde_json::Value> {
    serde_json::to_value(v).map_err(|_| unavailable())
}
fn decode<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> Result<T> {
    serde_json::from_value(v).map_err(|_| unavailable())
}
pub(crate) async fn authorize(
    tx: &Transaction<'_>,
    broker: &Broker,
    token: &str,
    id: Uuid,
    mutation: bool,
) -> Result<ExecutionView> {
    schema(tx).await?;
    crate::kernel::schema(tx).await?;
    storage::active(tx, broker.epoch).await?;
    let actor = auth::authenticate(tx, token, broker.epoch).await?;
    storage::validate_actor_generation(tx, &actor).await?;
    if let Some(row)=tx.query_opt("SELECT w.state,w.members,w.controller,w.manifest,x.mission_id,x.task_id,x.principal_id,x.role,x.recovery_epoch,w.launcher,w.quarantine_reason FROM supervised_workspaces w JOIN episode_executions x ON x.execution_id=w.id WHERE w.id=$1 FOR UPDATE OF w", &[&id]).await? {
        actor.scope(row.get(4),row.get(5))?;
        if actor.role!=Role::Owner && (actor.principal!=row.get::<_,Uuid>(6) || serde_json::to_value(&actor.role).map_err(|_|invalid())? != row.get::<_,String>(7)) { return Err(crate::domain::err(ErrorCode::Unauthorized)); }
        if actor.role!=Role::Owner && !execution_current(tx,id,broker.epoch).await? { return Err(crate::domain::err(ErrorCode::StaleGeneration)); }
        return Ok(ExecutionView { state:decode(serde_json::Value::String(row.get(0)))?,members:decode(row.get(1))?,controller:row.get::<_,Option<serde_json::Value>>(2).map(decode).transpose()?,manifest:decode(row.get(3))?,launcher:row.get::<_,Option<serde_json::Value>>(9).map(decode).transpose()?,quarantine_reason:row.get(10) });
    }
    let row=tx.query_opt("SELECT w.state,w.members,w.controller,w.manifest,t.mission_id,a.generation,a.recovery_epoch,t.generation,w.launcher,a.worker_id,w.quarantine_reason FROM supervised_workspaces w JOIN attempts a ON a.id=w.attempt_id JOIN tasks t ON t.id=a.task_id WHERE w.id=$1 FOR UPDATE OF w",&[&id]).await?.ok_or_else(||crate::domain::err(ErrorCode::NotFound))?;
    let manifest: WorkspaceManifest = decode(row.get(3))?;
    actor.scope(row.get(4), manifest.task)?;
    if actor.role == Role::Worker && actor.principal != row.get::<_, Uuid>(9) {
        return Err(crate::domain::err(ErrorCode::Unauthorized));
    }
    if mutation && actor.role == Role::Verifier {
        return Err(crate::domain::err(ErrorCode::Unauthorized));
    }
    if actor.role != Role::Owner
        && (row.get::<_, i64>(5) != row.get::<_, i64>(7) || row.get::<_, Uuid>(6) != broker.epoch)
    {
        return Err(crate::domain::err(ErrorCode::StaleGeneration));
    }
    Ok(ExecutionView {
        quarantine_reason: row.get(10),
        state: decode(serde_json::Value::String(row.get(0)))?,
        members: decode(row.get(1))?,
        launcher: row
            .get::<_, Option<serde_json::Value>>(8)
            .map(decode)
            .transpose()?,
        controller: row
            .get::<_, Option<serde_json::Value>>(2)
            .map(decode)
            .transpose()?,
        manifest,
    })
}
pub(crate) async fn execution_current(tx: &Transaction<'_>, id: Uuid, epoch: Uuid) -> Result<bool> {
    crate::kernel::schema(tx).await?;
    if let Some(row)=tx.query_opt("SELECT x.recovery_epoch,e.task_generation,t.generation,e.lead_generation,coalesce(l.generation,0),x.principal_id,l.principal_id,x.role FROM episode_executions x JOIN mailbox_episodes e ON e.id=x.episode_id LEFT JOIN tasks t ON t.id=x.task_id LEFT JOIN mission_leads l ON l.mission_id=x.mission_id WHERE x.execution_id=$1", &[&id]).await? {
        let credential_current:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_executions x JOIN credentials c ON c.id=x.credential_id JOIN mailbox_episodes e ON e.id=x.episode_id WHERE x.execution_id=$1 AND c.principal_id=x.principal_id AND c.role=x.role AND c.mission_id=x.mission_id AND c.task_id IS NOT DISTINCT FROM x.task_id AND c.generation IS NOT DISTINCT FROM e.task_generation AND c.recovery_epoch=$2 AND c.expires_at>clock_timestamp() AND (c.role<>'lead' OR c.lead_generation=e.lead_generation))", &[&id,&epoch]).await?.get(0);
        if !credential_current {return Ok(false);}
        return Ok(row.get::<_,Uuid>(0)==epoch && row.get::<_,Option<i64>>(1)==row.get::<_,Option<i64>>(2) && row.get::<_,Option<i64>>(3).is_none_or(|g|g==row.get::<_,i64>(4)) && (row.get::<_,String>(7)!="lead" || row.get::<_,Option<Uuid>>(6).is_none_or(|p|p==row.get::<_,Uuid>(5))));
    }
    Ok(tx.query_one("SELECT a.generation=t.generation AND a.recovery_epoch=$2 FROM supervised_workspaces w JOIN attempts a ON a.id=w.attempt_id JOIN tasks t ON t.id=a.task_id WHERE w.id=$1", &[&id,&epoch]).await?.get(0))
}
async fn bind_attempt_reservation(
    tx: &Transaction<'_>,
    attempt: Uuid,
    execution: Uuid,
) -> Result<()> {
    let bound = tx.execute("UPDATE reservations SET execution_id=$2 WHERE attempt_id=$1 AND state='reserved' AND execution_id IS NULL", &[&attempt,&execution]).await?;
    if bound != 1 {
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    Ok(())
}
async fn execution_reservation_current(tx: &Transaction<'_>, id: Uuid) -> Result<bool> {
    // Additional episodes own a separately charged physical reservation.
    if let Some(row) = tx
        .query_opt(
            "SELECT owns_capacity,capacity_released FROM episode_executions WHERE execution_id=$1",
            &[&id],
        )
        .await?
        && row.get::<_, bool>(0)
    {
        return Ok(!row.get::<_, bool>(1));
    }
    Ok(tx.query_one("SELECT EXISTS(SELECT 1 FROM reservations z WHERE z.state='reserved' AND z.execution_id=$1)", &[&id]).await?.get(0))
}
// Cancellation and terminal tasks retain inspection/cleanup authority but cannot dispatch work.
pub(crate) async fn execution_dispatch_current(
    tx: &Transaction<'_>,
    id: Uuid,
    epoch: Uuid,
) -> Result<bool> {
    if !execution_current(tx, id, epoch).await? {
        return Ok(false);
    }
    let cancelled:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM attempts a WHERE a.desired_cancel AND (a.id=(SELECT attempt_id FROM supervised_workspaces WHERE id=$1) OR a.id=(SELECT attempt_id FROM episode_executions WHERE execution_id=$1)))", &[&id]).await?.get(0);
    if cancelled {
        return Ok(false);
    }
    let target: Option<Uuid> = tx.query_one("SELECT coalesce(x.task_id,a.task_id) FROM supervised_workspaces w LEFT JOIN episode_executions x ON x.execution_id=w.id LEFT JOIN attempts a ON a.id=w.attempt_id WHERE w.id=$1", &[&id]).await?.get(0);
    let manifest: WorkspaceManifest = decode(
        tx.query_one(
            "SELECT manifest FROM supervised_workspaces WHERE id=$1",
            &[&id],
        )
        .await?
        .get(0),
    )?;
    if let Some(research) = &manifest.research_verification
        && crate::contracts::lifecycle::research_verification(tx, target.ok_or_else(invalid)?)
            .await?
            .as_ref()
            != Some(research)
    {
        return Ok(false);
    }
    let terminal = if let Some(task) = target {
        crate::contracts::lifecycle::current(tx, task)
            .await?
            .terminal()
    } else {
        false
    };
    let task: Option<Uuid> = tx.query_one(
        "SELECT CASE WHEN x.role='worker' THEN x.task_id WHEN x.execution_id IS NULL THEN a.task_id END FROM supervised_workspaces w LEFT JOIN episode_executions x ON x.execution_id=w.id LEFT JOIN attempts a ON a.id=w.attempt_id WHERE w.id=$1",
        &[&id],
    ).await?.get(0);
    if let Some(task) = task {
        match crate::artifacts::implementation_enabled(tx, task, epoch).await {
            Ok(()) => {}
            Err(error) if error.code == ErrorCode::DependencyUnmet => return Ok(false),
            Err(error) => return Err(error),
        }
    }
    Ok(!terminal && execution_reservation_current(tx, id).await?)
}
// AssignAttempt already reserved the first imported Worker episode. Additional
// episodes charge independently and cannot spend that reservation while an
// earlier task-bound native operation has unobserved usage.
async fn physical_usage_known(
    tx: &Transaction<'_>,
    id: Uuid,
    first_argv: Option<&[String]>,
) -> Result<()> {
    if let Some(row) = tx.query_opt("SELECT x.attempt_id,e.owned_capacity FROM episode_executions x JOIN mailbox_episodes e ON e.id=x.episode_id WHERE x.execution_id=$1", &[&id]).await?
        && row.get::<_, bool>(1)
            && let Some(attempt) = row.get::<_, Option<Uuid>>(0)
                && crate::kernel::legacy_usage_unknown(tx, attempt).await? {
                    return Err(crate::domain::err(ErrorCode::DependencyUnmet));
                }
    let attempt: Option<Uuid> = tx.query_one(
        "SELECT coalesce(x.attempt_id,w.attempt_id) FROM supervised_workspaces w LEFT JOIN episode_executions x ON x.execution_id=w.id WHERE w.id=$1", &[&id],
    ).await?.get(0);
    let uncertain = tx.query(
        "SELECT o.id,o.execution_id,o.state,o.request,o.controller,o.recovery_epoch FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE (o.execution_id=$2 OR ($1::uuid IS NOT NULL AND x.attempt_id=$1)) AND (o.state IN ('dispatching','outcome_unknown','quarantined') OR (o.state IN ('confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero'))", &[&attempt, &id],
    ).await?;
    for operation in uncertain {
        if operation.get::<_, Uuid>(1) != id {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let operation_id: Uuid = operation.get(0);
        let prepared: crate::episode_runtime::PreparedStart = decode(operation.get(3))?;
        let controller: Uuid = operation.get(4);
        let epoch: Uuid = operation.get(5);
        storage::active(tx, epoch).await?;
        let row = tx
            .query_one(
                "SELECT manifest,argv,state FROM supervised_workspaces WHERE id=$1",
                &[&id],
            )
            .await?;
        let manifest: WorkspaceManifest = decode(row.get(0))?;
        let input = manifest
            .input
            .as_ref()
            .ok_or_else(|| crate::domain::err(ErrorCode::DependencyUnmet))?;
        if prepared.start.execution != id
            || manifest.episode != Some(prepared.start.episode)
            || input.id != prepared.input_packet
            || input.digest != prepared.input_digest
            || manifest.recovery_epoch != epoch
        {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let transport = crate::model_free_runtime::transport(tx, controller).await?;
        use std::os::unix::fs::MetadataExt;
        if manifest.path.parent() != Some(transport.workspace_root.as_path()) {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        for (root, pinned) in [
            (&transport.native_root, &transport.native_directory),
            (&transport.workspace_root, &transport.workspace_directory),
        ] {
            let directory = crate::model_free_runtime::open_directory(root)?;
            let metadata = directory.metadata().map_err(|_| unavailable())?;
            if metadata.dev() != pinned.device
                || metadata.ino() != pinned.inode
                || metadata.uid() != pinned.owner
            {
                return Err(crate::domain::err(ErrorCode::DependencyUnmet));
            }
        }
        let current_store: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_stores WHERE identity=$1 AND controller=$2 AND runtime_identity=$3)", &[&transport.store,&controller,&json(&prepared.start.runtime)?]).await?.get(0);
        if transport.epoch != epoch
            || transport.identity != prepared.start.runtime
            || !current_store
        {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let argv: Vec<String> = if let Some(argv) = first_argv {
            argv.to_vec()
        } else {
            decode(row.get(1))?
        };
        if argv != prepared.start.argv {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let certificate = serde_json::json!({"kind":"original_launch_admitted","request":prepared,"controller":controller,"recovery_epoch":epoch,"execution":id,"argv":argv,"registered_process":transport.process});
        if first_argv.is_some() {
            if operation.get::<_, String>(2) != "dispatching"
                || row.get::<_, String>(2) != "ready"
                || local_process_identity().map_err(|_| unavailable())? != transport.process
            {
                return Err(crate::domain::err(ErrorCode::DependencyUnmet));
            }
            tx.execute(
                "INSERT INTO episode_runtime_evidence(id,operation_id,evidence) VALUES($1,$2,$3)",
                &[&Uuid::new_v4(), &operation_id, &certificate],
            )
            .await?;
        } else {
            let admitted: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_runtime_evidence WHERE operation_id=$1 AND evidence=$2)", &[&operation_id,&certificate]).await?.get(0);
            if row.get::<_, String>(2) != "starting" || !admitted {
                return Err(crate::domain::err(ErrorCode::DependencyUnmet));
            }
        }
    }
    Ok(())
}
impl Supervisor {
    pub async fn result(&self, token: &str, id: Uuid) -> Result<Option<ExecutionResult>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        authorize(&tx, &self.broker, token, id, false).await?;
        let result = tx
            .query_opt(
                "SELECT result FROM kernel_execution_results WHERE execution_id=$1",
                &[&id],
            )
            .await?
            .map(|r| decode(r.get(0)))
            .transpose()?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn open(broker: Broker, root: &Path, monitor: &str) -> Result<Self> {
        if !root.is_absolute() || !Path::new(monitor).is_absolute() {
            return Err(invalid());
        }
        let mut c = storage::client(&broker.dsn).await?;
        let tx = c.transaction().await?;
        schema(&tx).await?;
        crate::kernel::schema(&tx).await?;
        tx.commit().await?;
        fs::create_dir_all(root).map_err(|_| unavailable())?;
        let root = fs::canonicalize(root).map_err(|_| unavailable())?;
        fs::create_dir_all(root.join(".execution-logs")).map_err(|_| unavailable())?;
        Ok(Self {
            broker,
            root,
            monitor: PathBuf::from(monitor),
        })
    }
    /// Allocate physical ownership for a canonical episode without granting tool authority.
    /// Planning has no fabricated Task/Attempt; verification and implementation retain genuine links.
    pub async fn create_episode(
        &self,
        token: &str,
        episode: Uuid,
        requested: Vec<WriteScope>,
        fault: CreateFault,
    ) -> Result<ExecutionHandle> {
        self.create_episode_inner(token, episode, requested, fault, None)
            .await
    }
    /// Import only the currently submitted immutable candidate through scoped evidence reads.
    pub async fn create_episode_from_candidate(
        &self,
        token: &str,
        episode: Uuid,
        artifacts: &crate::artifacts::ArtifactService,
        fault: CreateFault,
    ) -> Result<ExecutionHandle> {
        let record = crate::mailbox::MailboxService::new(self.broker.clone())
            .record(token, crate::mailbox::RecordKind::Episode, episode)
            .await?;
        let task: Uuid =
            serde_json::from_value(record["task_id"].clone()).map_err(|_| invalid())?;
        let candidate = artifacts
            .snapshot(token, task)
            .await?
            .candidate
            .ok_or_else(|| crate::domain::err(ErrorCode::DependencyUnmet))?;
        let bytes = artifacts
            .read_evidence(token, task, &candidate.artifact)
            .await?;
        self.create_episode_inner(token, episode, vec![], fault, Some((candidate, bytes)))
            .await
    }
    async fn create_episode_inner(
        &self,
        token: &str,
        episode: Uuid,
        requested: Vec<WriteScope>,
        fault: CreateFault,
        source: Option<(crate::artifacts::Candidate, Vec<u8>)>,
    ) -> Result<ExecutionHandle> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        crate::kernel::schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, token, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        let row=tx.query_opt("SELECT e.mission_id,e.task_id,e.attempt_id,e.principal_id,e.role,e.recovery_epoch,e.input_digest,e.input_packet,e.state,e.deadline>clock_timestamp(),e.task_generation,t.generation,e.lead_generation,coalesce(l.generation,0),e.owned_capacity,coalesce(s.git_repository,m.git_repository),coalesce(s.git_commit,m.git_commit),coalesce(s.digest,m.input_digest),s.write_scope,a.worker_id FROM mailbox_episodes e JOIN missions m ON m.id=e.mission_id LEFT JOIN tasks t ON t.id=e.task_id LEFT JOIN attempts a ON a.id=e.attempt_id LEFT JOIN task_specs s ON s.task_id=t.id AND s.digest=a.spec_digest LEFT JOIN mission_leads l ON l.mission_id=e.mission_id WHERE e.id=$1 FOR UPDATE OF e", &[&episode]).await?.ok_or_else(||crate::domain::err(ErrorCode::NotFound))?;
        let mission: Uuid = row.get(0);
        let task: Option<Uuid> = row.get(1);
        let attempt: Option<Uuid> = row.get(2);
        let role: String = row.get(4);
        actor.scope(mission, task)?;
        if actor.principal != row.get::<_, Uuid>(3)
            || serde_json::to_value(&actor.role).map_err(|_| invalid())? != role
        {
            return Err(crate::domain::err(ErrorCode::Unauthorized));
        }
        if let Some(attempt) = attempt {
            crate::kernel::attempt_dispatch_enabled(&tx, attempt).await?;
        }
        if row.get::<_, Uuid>(5) != self.broker.epoch
            || row.get::<_, Option<i64>>(10) != row.get::<_, Option<i64>>(11)
            || row
                .get::<_, Option<i64>>(12)
                .is_some_and(|g| g != row.get::<_, i64>(13))
        {
            return Err(crate::domain::err(ErrorCode::StaleGeneration));
        }
        if row.get::<_, Option<Uuid>>(7).is_none()
            || !row.get::<_, bool>(9)
            || !matches!(row.get::<_, String>(8).as_str(), "admitted" | "active")
        {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        if row.get::<_, bool>(14)
            && let Some(attempt) = attempt
            && crate::kernel::legacy_usage_unknown(&tx, attempt).await?
        {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        if role == "worker" && row.get::<_, Option<Uuid>>(19) != Some(actor.principal) {
            return Err(crate::domain::err(ErrorCode::Unauthorized));
        }
        if let Some(task) = task {
            let enabled:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM kernel_verification_requirements WHERE task_id=$1 AND recovery_epoch=$2)", &[&task,&self.broker.epoch]).await?.get(0);
            if !enabled {
                return Err(crate::domain::err(ErrorCode::DependencyUnmet));
            }
        }
        let research_verification = if role == "verifier" && source.is_none() {
            let research =
                crate::contracts::lifecycle::research_verification(&tx, task.ok_or_else(invalid)?)
                    .await?
                    .ok_or_else(|| crate::domain::err(ErrorCode::DependencyUnmet))?;
            if research.verifier != actor.principal
                || row.get::<_, Option<Uuid>>(19) == Some(actor.principal)
            {
                return Err(crate::domain::err(ErrorCode::Unauthorized));
            }
            crate::contracts::lifecycle::research_input(
                &tx,
                row.get::<_, Option<Uuid>>(7).ok_or_else(invalid)?,
                &research,
            )
            .await?;
            Some(research)
        } else {
            None
        };
        let mut verification_policy = None;
        if let Some((candidate, bytes)) = &source {
            if !matches!(role.as_str(), "worker" | "verifier")
                || candidate.generation != row.get::<_, Option<i64>>(10).ok_or_else(invalid)?
                || candidate.spec_digest != row.get::<_, String>(17)
                || crate::artifacts::sha256(bytes) != candidate.artifact
            {
                return Err(invalid());
            }
            let current=tx.query_opt("SELECT c.candidate,p.base,p.suite,p.environment FROM artifact_current a JOIN artifact_candidates c ON c.id=a.candidate_id JOIN artifact_policies p ON p.task_id=a.task_id WHERE a.task_id=$1", &[&task]).await?.ok_or_else(||crate::domain::err(ErrorCode::DependencyUnmet))?;
            if current.get::<_, serde_json::Value>(0) != json(candidate)?
                || current.get::<_, String>(1) != candidate.base
            {
                return Err(crate::domain::err(ErrorCode::StaleGeneration));
            }
            if role == "verifier" {
                verification_policy = Some(VerificationPolicy {
                    suite: current.get(2),
                    environment: current.get(3),
                });
            }
        }
        if role == "worker" {
            crate::artifacts::implementation_enabled(
                &tx,
                task.ok_or_else(invalid)?,
                self.broker.epoch,
            )
            .await?;
        }
        if let Some(existing)=tx.query_opt("SELECT w.id,w.manifest FROM episode_executions x JOIN supervised_workspaces w ON w.id=x.execution_id WHERE x.episode_id=$1", &[&episode]).await? {
            let existing_id: Uuid = existing.get(0);
            if !execution_reservation_current(&tx, existing_id).await? {return Err(crate::domain::err(ErrorCode::DependencyUnmet));}
            let manifest:WorkspaceManifest=decode(existing.get(1))?;return Ok(ExecutionHandle{id:existing_id,path:manifest.path});
        }
        let paused:bool=tx.query_one("SELECT coalesce((SELECT paused FROM kernel_mission_control WHERE mission_id=$1),false)",&[&mission]).await?.get(0);
        if paused {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        if let Some(task) = task {
            crate::contracts::fresh_grant(&tx, task).await?;
        }
        let id = Uuid::new_v4();
        let path = self.root.join(id.to_string());
        let store = fs::canonicalize(row.get::<_, String>(15)).map_err(|_| invalid())?;
        let admitted_base: String = row.get(16);
        // Candidate bytes bind the current Owner-authorized integration base;
        // permitted paths still come from immutable admitted Task input.
        let base = source.as_ref().map_or_else(
            || admitted_base.clone(),
            |(candidate, _)| candidate.base.clone(),
        );
        let write_scope: Vec<String> = if role == "worker" {
            decode(
                row.get::<_, Option<serde_json::Value>>(18)
                    .ok_or_else(invalid)?,
            )?
        } else {
            vec![]
        };
        if write_scope
            .iter()
            .any(|p| !safe_relative(p) || p.split('/').any(|c| c == ".git"))
        {
            return Err(invalid());
        }
        let scopes = resolve_scopes(&store, &admitted_base, &write_scope, requested)?;
        let manifest = WorkspaceManifest {
            store,
            path: path.clone(),
            base,
            attempt,
            task,
            episode: Some(episode),
            mission: Some(mission),
            generation: row.get::<_, Option<i64>>(10).unwrap_or(0),
            recovery_epoch: self.broker.epoch,
            spec_digest: row.get(17),
            write_scope,
            scopes,
            log_path: self.root.join(".execution-logs").join(format!("{id}.log")),
            candidate: source.as_ref().map(|s| s.0.clone()),
            verification_policy,
            research_verification,
            input: Some(EpisodeInput {
                id: row.get::<_, Option<Uuid>>(7).ok_or_else(invalid)?,
                digest: row.get(6),
            }),
        };
        tx.execute(
            "INSERT INTO supervised_workspaces(id,manifest,state) VALUES($1,$2,'registered')",
            &[&id, &json(&manifest)?],
        )
        .await?;
        let credential: Uuid = tx
            .query_one(
                "SELECT id FROM credentials WHERE token_hash=$1",
                &[&token_hash(token)],
            )
            .await?
            .get(0);
        tx.execute("INSERT INTO episode_executions(execution_id,episode_id,mission_id,task_id,attempt_id,principal_id,role,recovery_epoch,input_digest,owns_capacity,credential_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)", &[&id,&episode,&mission,&task,&attempt,&actor.principal,&role,&self.broker.epoch,&row.get::<_,String>(6),&row.get::<_,bool>(14),&credential]).await?;
        if !row.get::<_, bool>(14) {
            bind_attempt_reservation(&tx, attempt.ok_or_else(invalid)?, id).await?;
        }
        tx.commit().await?;
        if matches!(fault, CreateFault::AfterRegister) {
            return Ok(ExecutionHandle { id, path });
        }
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let view = authorize(&tx, &self.broker, token, id, true).await?;
        if view.state != ExecutionState::Registered {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        tx.execute(
            "UPDATE supervised_workspaces SET state='preparing' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        let prepared = clone_workspace(&manifest, fault).and_then(|_| {
            if let Some((candidate, bytes)) = &source {
                let bundle = path.join(".git/kernel-candidate.bundle");
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&bundle)
                    .map_err(|_| unavailable())?;
                std::io::Write::write_all(&mut file, bytes).map_err(|_| unavailable())?;
                file.sync_all().map_err(|_| unavailable())?;
                fs::File::open(path.join(".git"))
                    .and_then(|f| f.sync_all())
                    .map_err(|_| unavailable())?;
                let bundle = bundle.to_str().ok_or_else(invalid)?;
                git_at(&path, &["bundle", "verify", bundle])?;
                git_at(&path, &["fetch", "--no-tags", bundle, &candidate.commit])?;
                if git_at(
                    &path,
                    &["rev-parse", &format!("{}^{{tree}}", candidate.commit)],
                )?
                .trim()
                    != candidate.tree
                    || git_at(&path, &["rev-parse", &format!("{}^1", candidate.commit)])?.trim()
                        != candidate.base
                    || candidate.base != manifest.base
                {
                    return Err(invalid());
                }
                git_at(
                    &path,
                    &["checkout", "--quiet", "--detach", &candidate.commit],
                )
                .map(|_| ())
            } else {
                git_at(&path, &["checkout", "--quiet", "--detach", &manifest.base]).map(|_| ())
            }
        });
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        authorize(&tx, &self.broker, token, id, true).await?;
        if prepared.is_err() || !execution_dispatch_current(&tx, id, self.broker.epoch).await? {
            tx.execute("UPDATE supervised_workspaces SET state='quarantined',quarantine_reason='episode Git preparation completion unknown' WHERE id=$1", &[&id]).await?;
            tx.commit().await?;
            return Err(unavailable());
        }
        for scope in &manifest.scopes {
            scope_path(&path, scope, false, true)?;
        }
        for scope in &manifest.scopes {
            scope_path(&path, scope, true, false)?;
        }
        fs::File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(|_| unavailable())?;
        tx.execute(
            "UPDATE supervised_workspaces SET state='ready' WHERE id=$1 AND state='preparing'",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        Ok(ExecutionHandle { id, path })
    }
    pub async fn create_attempt(
        &self,
        token: &str,
        attempt: Uuid,
        fault: CreateFault,
    ) -> Result<ExecutionHandle> {
        self.create_attempt_with_scopes(token, attempt, Vec::new(), fault)
            .await
    }
    /// Missing admitted paths require an explicit kind. Existing types come from exact Git bytes.
    pub async fn create_attempt_with_scopes(
        &self,
        token: &str,
        attempt: Uuid,
        requested: Vec<WriteScope>,
        fault: CreateFault,
    ) -> Result<ExecutionHandle> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, token, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        if !matches!(actor.role, Role::Owner | Role::Lead | Role::Worker) {
            return Err(crate::domain::err(ErrorCode::Unauthorized));
        }
        let r=tx.query_opt("SELECT a.task_id,a.generation,a.recovery_epoch,a.spec_digest,t.mission_id,t.generation,s.git_repository,s.git_commit,s.write_scope,a.worker_id FROM attempts a JOIN tasks t ON t.id=a.task_id JOIN task_specs s ON s.task_id=a.task_id AND s.digest=a.spec_digest JOIN reservations z ON z.attempt_id=a.id WHERE a.id=$1 AND z.state='reserved'",&[&attempt]).await?.ok_or_else(||crate::domain::err(ErrorCode::NotFound))?;
        let task = r.get(0);
        actor.scope(r.get(4), Some(task))?;
        if actor.role == Role::Worker && actor.principal != r.get::<_, Uuid>(9) {
            return Err(crate::domain::err(ErrorCode::Unauthorized));
        }
        if r.get::<_, i64>(1) != r.get::<_, i64>(5) || r.get::<_, Uuid>(2) != self.broker.epoch {
            return Err(crate::domain::err(ErrorCode::StaleGeneration));
        }
        crate::kernel::attempt_dispatch_enabled(&tx, attempt).await?;
        crate::contracts::fresh_grant(&tx, task).await?;
        crate::artifacts::implementation_enabled(&tx, task, self.broker.epoch).await?;
        if tx
            .query_opt(
                "SELECT id FROM supervised_workspaces WHERE attempt_id=$1",
                &[&attempt],
            )
            .await?
            .is_some()
        {
            return Err(crate::domain::err(ErrorCode::IdempotencyConflict));
        }
        let id = Uuid::new_v4();
        let path = self.root.join(id.to_string());
        let store = fs::canonicalize(r.get::<_, String>(6)).map_err(|_| invalid())?;
        let write_scope: Vec<String> = decode(r.get(8))?;
        if write_scope
            .iter()
            .any(|p| !safe_relative(p) || p.split('/').any(|c| c == ".git"))
        {
            return Err(invalid());
        }
        let base: String = r.get(7);
        let scopes = resolve_scopes(&store, &base, &write_scope, requested)?;
        let manifest = WorkspaceManifest {
            store,
            path: path.clone(),
            base,
            attempt: Some(attempt),
            task: Some(task),
            episode: None,
            mission: Some(r.get(4)),
            generation: r.get(1),
            recovery_epoch: self.broker.epoch,
            spec_digest: r.get(3),
            write_scope,
            scopes,
            log_path: self.root.join(".execution-logs").join(format!("{id}.log")),
            candidate: None,
            verification_policy: None,
            research_verification: None,
            input: None,
        };
        tx.execute("INSERT INTO supervised_workspaces(id,attempt_id,manifest,state) VALUES($1,$2,$3,'registered')",&[&id,&attempt,&json(&manifest)?]).await?;
        bind_attempt_reservation(&tx, attempt, id).await?;
        tx.commit().await?;
        if matches!(fault, CreateFault::AfterRegister) {
            return Ok(ExecutionHandle { id, path });
        }
        // Serialize filesystem preparation with deletion/launch reservation. A crash keeps registered state.
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if v.state != ExecutionState::Registered {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        tx.execute(
            "UPDATE supervised_workspaces SET state='preparing' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        // Git is an external preparation workload. Its durable unknown state precedes spawn.
        let prepared = clone_workspace(&manifest, fault).and_then(|_| {
            git_at(&path, &["checkout", "--quiet", "--detach", &manifest.base]).map(|_| ())
        });
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        let current = tx
            .query_one(
                "SELECT generation FROM tasks WHERE id=$1",
                &[&manifest.task],
            )
            .await?
            .get::<_, i64>(0);
        if prepared.is_err()
            || v.state != ExecutionState::Preparing
            || manifest.generation != current
            || manifest.recovery_epoch != self.broker.epoch
        {
            tx.execute(
                "UPDATE supervised_workspaces SET state='quarantined' WHERE id=$1",
                &[&id],
            )
            .await?;
            tx.commit().await?;
            return Err(unavailable());
        }
        // Validate every existing component before creating any missing scope.
        for scope in &manifest.scopes {
            scope_path(&path, scope, false, true)?;
        }
        for scope in &manifest.scopes {
            scope_path(&path, scope, true, false)?;
        }
        fs::File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(|_| unavailable())?;
        if matches!(fault, CreateFault::AfterCreate) {
            tx.execute(
                "UPDATE supervised_workspaces SET state='registered' WHERE id=$1",
                &[&id],
            )
            .await?;
            tx.commit().await?;
            return Ok(ExecutionHandle { id, path });
        }
        tx.execute(
            "UPDATE supervised_workspaces SET state='ready' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        Ok(ExecutionHandle { id, path })
    }
    pub async fn inspect(&self, token: &str, id: Uuid) -> Result<ExecutionView> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        let view = authorize(&tx, &self.broker, token, id, false).await?;
        tx.commit().await?;
        Ok(view)
    }
    pub async fn start(
        &self,
        token: &str,
        id: Uuid,
        argv: Vec<String>,
        seconds: u64,
    ) -> Result<()> {
        self.start_with_fault(token, id, argv, seconds, StartFault::None)
            .await
    }
    pub async fn start_with_fault(
        &self,
        token: &str,
        id: Uuid,
        argv: Vec<String>,
        seconds: u64,
        fault: StartFault,
    ) -> Result<()> {
        if argv.is_empty()
            || argv.iter().any(|s| s.contains('\0'))
            || seconds == 0
            || seconds > 86400
        {
            return Err(invalid());
        }
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if !execution_dispatch_current(&tx, id, self.broker.epoch).await? {
            return Err(crate::domain::err(ErrorCode::StaleGeneration));
        }
        physical_usage_known(&tx, id, Some(&argv)).await?;
        let mission: Uuid = tx
            .query_one(
                "SELECT coalesce($1,(SELECT mission_id FROM tasks WHERE id=$2))",
                &[&v.manifest.mission, &v.manifest.task],
            )
            .await?
            .get(0);
        crate::kernel::dispatch_enabled(&tx, mission).await?;
        if v.state != ExecutionState::Ready {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let limit:i64=tx.query_one("SELECT m.deadline_seconds FROM missions m WHERE m.id=coalesce($1,(SELECT mission_id FROM tasks WHERE id=$2))",&[&v.manifest.mission,&v.manifest.task]).await?.get(0);
        if let Some(episode) = v.manifest.episode {
            let eligible:bool=tx.query_one("SELECT state IN ('admitted','active') AND deadline>clock_timestamp() FROM mailbox_episodes WHERE id=$1",&[&episode]).await?.get(0);
            if !eligible {
                return Err(crate::domain::err(ErrorCode::DependencyUnmet));
            }
        }
        let seconds = i64::try_from(seconds).map_err(|_| invalid())?.min(limit);
        tx.execute("UPDATE supervised_workspaces SET state='starting',argv=$2,deadline=least(clock_timestamp()+$3::bigint*interval '1 second',coalesce((SELECT e.deadline FROM episode_executions x JOIN mailbox_episodes e ON e.id=x.episode_id WHERE x.execution_id=$1),clock_timestamp()+$3::bigint*interval '1 second')) WHERE id=$1",&[&id,&json(&argv)?,&seconds]).await?;
        tx.commit().await?;
        if matches!(fault, StartFault::BeforeMonitor) {
            return Ok(());
        }
        use std::os::unix::process::CommandExt;
        let mut command = Command::new(&self.monitor);
        command
            .arg("supervisor-monitor")
            .arg(id.to_string())
            .env_clear()
            .env("TEAM_CONTROL_PLANE_DSN", &self.broker.dsn)
            .env(
                "TEAM_CONTROL_PLANE_RECOVERY_EPOCH",
                self.broker.epoch.to_string(),
            )
            .env("PATH", "/usr/bin:/bin")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        command.process_group(0);
        let mut child = command.spawn().map_err(|_| unavailable())?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
    pub async fn cancel(&self, token: &str, id: Uuid) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if matches!(v.state, ExecutionState::Released | ExecutionState::Deleting) {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        tx.execute("UPDATE supervised_workspaces SET desired_cancel=true,state=CASE WHEN state IN ('running','starting') THEN 'cancelling' ELSE state END WHERE id=$1",&[&id]).await?;
        tx.commit().await?;
        self.reconcile(token, id).await?;
        Ok(())
    }
    pub async fn reconcile(&self, token: &str, id: Uuid) -> Result<ExecutionView> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        authorize(&tx, &self.broker, token, id, true).await?;
        reconcile_locked(&tx, id).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        match recover_result(&tx, &self.broker, id, &v).await {
            Ok(()) => {}
            Err(e) if e.code == ErrorCode::DependencyUnmet => {}
            Err(e) => return Err(e),
        }
        tx.commit().await?;
        self.inspect(token, id).await
    }
    /// Retained independently of workspace release. Authenticate every log read.
    pub async fn logs(&self, token: &str, id: Uuid) -> Result<Vec<u8>> {
        let view = self.inspect(token, id).await?;
        fs::read(view.manifest.log_path).map_err(|_| unavailable())
    }
    pub async fn history(&self, token: &str, id: Uuid) -> Result<Vec<ExecutionView>> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        let v = authorize(&tx, &self.broker, token, id, false).await?;
        let rows=tx.query("SELECT state,members FROM supervised_observations WHERE execution_id=$1 ORDER BY observed_at,id",&[&id]).await?;
        let mut result = vec![];
        for r in rows {
            result.push(ExecutionView {
                state: decode(serde_json::Value::String(r.get(0)))?,
                members: decode(r.get(1))?,
                controller: v.controller.clone(),
                launcher: v.launcher.clone(),
                quarantine_reason: v.quarantine_reason.clone(),
                manifest: v.manifest.clone(),
            });
        }
        tx.commit().await?;
        Ok(result)
    }
    pub async fn release(&self, token: &str, id: Uuid) -> Result<()> {
        let mut c = storage::client(&self.broker.dsn).await?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        let v = authorize(&tx, &self.broker, token, id, true).await?;
        if !matches!(v.state, ExecutionState::Stopped | ExecutionState::Deleting) {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        recover_result(&tx, &self.broker, id, &v).await?;
        // Namespace Stop can precede launcher reaping and immutable result
        // persistence. A launched episode retains its workspace until both facts
        // are durable; an allocation cancelled before launch has no result.
        if v.manifest.episode.is_some()
            && tx.query_one(
                "SELECT init_process IS NOT NULL AND NOT EXISTS(SELECT 1 FROM kernel_execution_results WHERE execution_id=$1) FROM supervised_workspaces WHERE id=$1",
                &[&id],
            ).await?.get::<_, bool>(0)
        {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        runtime_release_known(&tx, &v.manifest).await?;
        // Deletion reservation commits before filesystem mutation; starting cannot cross it.
        tx.execute(
            "UPDATE supervised_workspaces SET state='deleting' WHERE id=$1",
            &[&id],
        )
        .await?;
        tx.commit().await?;
        if v.manifest.path.exists() {
            fs::remove_dir_all(&v.manifest.path).map_err(|_| unavailable())?;
        }
        fs::File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(|_| unavailable())?;
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        authorize(&tx, &self.broker, token, id, true).await?;
        runtime_release_known(&tx, &v.manifest).await?;
        tx.execute(
            "UPDATE supervised_workspaces SET state='released' WHERE id=$1 AND state='deleting'",
            &[&id],
        )
        .await?;
        // A released execution has independently established shutdown and known
        // effects/usage. Retire only its expired live episode; predeadline
        // checkpoints and yielded waits remain authoritative. Budgets stay consumed.
        if let Some(episode) = v.manifest.episode {
            tx.execute("UPDATE mailbox_episodes e SET state=CASE WHEN w.desired_cancel THEN 'cancelled' ELSE 'completed' END FROM episode_executions x JOIN supervised_workspaces w ON w.id=x.execution_id WHERE e.id=$1 AND x.episode_id=e.id AND x.execution_id=$2 AND w.state='released' AND e.state IN ('admitted','active') AND e.deadline<=clock_timestamp()", &[&episode,&id]).await?;
        }
        // Only physically proven stop plus completed deletion refunds the concurrent slot.
        // Attempt/episode admission counters remain consumed.
        if v.manifest.episode.is_some() {
            tx.execute("WITH released AS (UPDATE episode_executions SET capacity_released=true WHERE execution_id=$1 AND owns_capacity AND NOT capacity_released RETURNING mission_id) UPDATE missions SET used_capacity=used_capacity-1 WHERE id IN (SELECT mission_id FROM released)", &[&id]).await?;
        }
        if v.manifest.episode.is_none()
            || !tx
                .query_one(
                    "SELECT owns_capacity FROM episode_executions WHERE execution_id=$1",
                    &[&id],
                )
                .await?
                .get::<_, bool>(0)
        {
            tx.execute("WITH released AS (UPDATE reservations SET state='released' WHERE attempt_id=$1 AND execution_id=$2 AND state='reserved' RETURNING mission_id) UPDATE missions SET used_capacity=used_capacity-1 WHERE id IN (SELECT mission_id FROM released)",&[&v.manifest.attempt,&id]).await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
// Never overwrite an immutable winner, including a recovered unknown exit.
async fn persist_result(tx: &Transaction<'_>, result: &ExecutionResult) -> Result<ExecutionResult> {
    tx.execute("INSERT INTO kernel_execution_results(execution_id,result) VALUES($1,$2) ON CONFLICT(execution_id) DO NOTHING", &[&result.execution,&json(result)?]).await?;
    decode(
        tx.query_one(
            "SELECT result FROM kernel_execution_results WHERE execution_id=$1",
            &[&result.execution],
        )
        .await?
        .get(0),
    )
}
fn log_digest(manifest: &WorkspaceManifest, id: Uuid) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::{io::Read, os::unix::fs::MetadataExt};
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    let root = manifest.path.parent().ok_or_else(rejected)?;
    if manifest.log_path != root.join(".execution-logs").join(format!("{id}.log")) {
        return Err(rejected());
    }
    let directory = crate::model_free_runtime::open_directory(&root.join(".execution-logs"))?;
    let name = manifest.log_path.file_name().ok_or_else(rejected)?;
    let mut file = crate::model_free_runtime::secure_file(&directory, name)?;
    let stamp = |m: fs::Metadata| {
        (
            m.dev(),
            m.ino(),
            m.uid(),
            m.mode(),
            m.nlink(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    };
    let before = stamp(file.metadata().map_err(|_| rejected())?);
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|_| rejected())?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if before != stamp(file.metadata().map_err(|_| rejected())?)
        || before
            != stamp(
                crate::model_free_runtime::secure_file(&directory, name)?
                    .metadata()
                    .map_err(|_| rejected())?,
            )
    {
        return Err(rejected());
    }
    // The pathname must still denote the pinned registered directory incarnation.
    if crate::model_free_runtime::directory_identity(&root.join(".execution-logs"))? != {
        let m = directory.metadata().map_err(|_| rejected())?;
        crate::model_free_runtime::DirectoryIdentity {
            device: m.dev(),
            inode: m.ino(),
            owner: m.uid(),
        }
    } {
        return Err(rejected());
    }
    Ok(format!("{:x}", digest.finalize()))
}
async fn recover_result(
    tx: &Transaction<'_>,
    broker: &Broker,
    id: Uuid,
    v: &ExecutionView,
) -> Result<()> {
    if v.manifest.episode.is_none()
        || tx
            .query_opt(
                "SELECT result FROM kernel_execution_results WHERE execution_id=$1",
                &[&id],
            )
            .await?
            .is_some()
    {
        return Ok(());
    }
    let row = tx
        .query_one(
            "SELECT init_process,argv FROM supervised_workspaces WHERE id=$1",
            &[&id],
        )
        .await?;
    let Some(init) = row.get::<_, Option<serde_json::Value>>(0) else {
        return Ok(());
    };
    let init: ProcessIdentity = decode(init)?;
    let rejected = || crate::domain::err(ErrorCode::DependencyUnmet);
    if v.state != ExecutionState::Stopped || !v.members.is_empty() {
        return Err(rejected());
    }
    let controller = v.controller.clone().ok_or_else(rejected)?;
    let launcher = v.launcher.as_ref().ok_or_else(rejected)?;
    observer_frame(Some(&controller)).map_err(|_| rejected())?;
    if !process_finalized(&controller).map_err(|_| rejected())?
        || !process_finalized(launcher).map_err(|_| rejected())?
        || !finalized(&init).map_err(|_| rejected())?
    {
        return Err(rejected());
    }
    // This append-only stop came from the complete observer for the launched
    // immutable PID1 incarnation. A mutable stopped string is insufficient.
    let observation=tx.query_opt("SELECT s.id FROM supervised_observations s WHERE s.execution_id=$1 AND s.state='stopped' AND s.members='[]'::jsonb AND EXISTS(SELECT 1 FROM supervised_observations r WHERE r.execution_id=s.execution_id AND r.state IN ('running','cancelling') AND r.members @> $2::jsonb AND r.observed_at<=s.observed_at) ORDER BY s.observed_at,s.id LIMIT 1", &[&id,&json(&vec![init.clone()])?]).await?.ok_or_else(rejected)?;
    runtime_release_known(tx, &v.manifest).await?;
    let result = ExecutionResult {
        execution: id,
        argv: decode(row.get(1))?,
        candidate: v.manifest.candidate.clone(),
        recovery_epoch: v.manifest.recovery_epoch,
        controller,
        exit_code: None,
        signal: None,
        namespace_stopped: true,
        logs: log_digest(&v.manifest, id)?,
        completion: CompletionProvenance::RecoveredStop {
            finalizer: identity(std::process::id()).map_err(|_| rejected())?,
            recovery_epoch: broker.epoch,
            stop_observation: observation.get(0),
            init_process: init,
        },
    };
    let _winner = persist_result(tx, &result).await?;
    Ok(())
}
/// Admission may encounter an episode released before its deadline. Reconcile
/// only the expired, current caller's exact registered completion; expiry alone
/// and a mutable released flag cannot establish shutdown or refund a claim.
/// An immutable Broker amendment can fence a completed historical generation;
/// its exact task/Intent/event binding admits retirement only, never execution.
pub(crate) async fn retire_expired_released_episodes(
    tx: &Transaction<'_>,
    actor: &auth::Actor,
    mission: Uuid,
    epoch: Uuid,
) -> Result<()> {
    let role = json(&actor.role)?;
    let role = role.as_str().ok_or_else(invalid)?;
    let rows = tx
        .query(
            "SELECT e.id,w.id,w.manifest,r.result
         FROM mailbox_episodes e
         JOIN episode_executions x ON x.episode_id=e.id
         JOIN supervised_workspaces w ON w.id=x.execution_id
         JOIN kernel_execution_results r ON r.execution_id=w.id
         LEFT JOIN tasks t ON t.id=e.task_id
         LEFT JOIN mission_leads l ON l.mission_id=e.mission_id
         WHERE e.principal_id=$1 AND x.principal_id=$1
           AND e.mission_id=$2 AND x.mission_id=$2
           AND e.role=$3 AND x.role=$3
           AND e.recovery_epoch=$4 AND x.recovery_epoch=$4
           AND ($5::uuid IS NULL OR e.task_id=$5)
           AND e.task_id IS NOT DISTINCT FROM x.task_id
           AND (e.task_generation IS NOT DISTINCT FROM t.generation OR
                (e.task_generation<t.generation AND EXISTS(
                  SELECT 1 FROM events v JOIN contract_intents i
                    ON i.id=v.payload->>'intent'
                  JOIN contract_work c ON c.id=i.work_id AND c.task_id=e.task_id
                  WHERE v.aggregate_kind='task' AND v.aggregate_id=e.task_id
                    AND v.kind='task.intent.amend' AND v.version=1
                    AND v.sequence>e.admitted_task_revision
                    AND v.principal_id=i.principal_id
                    AND v.payload->>'by_principal'=v.principal_id::text
                    AND i.recovery_epoch=$4
                    AND v.payload->>'recovery_epoch'=$4::text
                    AND v.payload->>'previous_generation'=e.task_generation::text
                    AND v.payload->>'generation'=(e.task_generation+1)::text)))
           AND (e.lead_generation IS NULL OR
                (e.lead_generation=coalesce(l.generation,0)
                 AND (l.principal_id IS NULL OR l.principal_id=$1)))
           AND e.state IN ('admitted','active') AND e.deadline<=clock_timestamp()
           AND w.state='released' AND w.members='[]'::jsonb
           AND w.init_process IS NOT NULL
           AND r.result->>'namespace_stopped'='true'
           AND ((x.owns_capacity AND x.capacity_released) OR
                (NOT x.owns_capacity AND EXISTS(SELECT 1 FROM reservations z
                 WHERE z.execution_id=w.id AND z.state='released')))
           AND EXISTS(SELECT 1 FROM supervised_observations s
               WHERE s.execution_id=w.id AND s.state='stopped'
                 AND s.members='[]'::jsonb
                 AND EXISTS(SELECT 1 FROM supervised_observations p
                     WHERE p.execution_id=w.id AND p.state IN ('running','cancelling')
                       AND p.members @> jsonb_build_array(w.init_process)
                       AND p.observed_at<=s.observed_at))",
            &[&actor.principal, &mission, &role, &epoch, &actor.task],
        )
        .await?;
    for row in rows {
        let episode: Uuid = row.get(0);
        let execution: Uuid = row.get(1);
        let manifest: WorkspaceManifest = decode(row.get(2))?;
        let result: ExecutionResult = decode(row.get(3))?;
        if manifest.episode != Some(episode)
            || manifest.mission != Some(mission)
            || manifest.recovery_epoch != epoch
            || result.execution != execution
            || result.recovery_epoch != epoch
            || !result.namespace_stopped
            || manifest.path.exists()
            || log_digest(&manifest, execution)? != result.logs
        {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        // Release is historical evidence: recheck current operation/usage
        // uncertainty, including every task-bound sibling, before retiring.
        runtime_release_known(tx, &manifest).await?;
        tx.execute("UPDATE mailbox_episodes e SET state=CASE WHEN w.desired_cancel THEN 'cancelled' ELSE 'completed' END FROM supervised_workspaces w WHERE e.id=$1 AND w.id=$2 AND e.state IN ('admitted','active') AND e.deadline<=clock_timestamp()", &[&episode,&execution]).await?;
    }
    Ok(())
}
async fn runtime_release_known(tx: &Transaction<'_>, manifest: &WorkspaceManifest) -> Result<()> {
    // Namespace shutdown proves local membership only. It cannot resolve a
    // separately dispatched provider operation whose reply/effect is unknown.
    if let Some(attempt) = manifest.attempt {
        if crate::kernel::legacy_usage_unknown(tx, attempt).await? {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        let unknown:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_operations WHERE attempt_id=$1 AND state IN ('dispatching','outcome_unknown','quarantined'))", &[&attempt]).await?.get(0);
        if unknown {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
    }
    if let Some(episode) = manifest.episode {
        let unknown:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE (o.episode_id=$1 OR ($2::uuid IS NOT NULL AND x.attempt_id=$2)) AND (o.state IN ('dispatching','outcome_unknown','quarantined') OR (o.state IN ('confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero')))", &[&episode,&manifest.attempt]).await?.get(0);
        if unknown {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
    }
    Ok(())
}
fn safe_relative(p: &str) -> bool {
    !p.is_empty()
        && !p.contains('\0')
        && p.split('/').all(|c| !c.is_empty() && c != "." && c != "..")
}
fn resolve_scopes(
    store: &Path,
    base: &str,
    paths: &[String],
    requested: Vec<WriteScope>,
) -> Result<Vec<WriteScope>> {
    let mut explicit = std::collections::HashMap::new();
    for scope in requested {
        if !paths.contains(&scope.path) || explicit.insert(scope.path, scope.kind).is_some() {
            return Err(invalid());
        }
    }
    let mut result = Vec::new();
    for path in paths {
        let mut prefix = String::new();
        let parts: Vec<_> = path.split('/').collect();
        let mut existing = None;
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                prefix.push('/');
            }
            prefix.push_str(part);
            let entry = git_at(store, &["ls-tree", "-z", base, "--", &prefix])?;
            let mode = entry.split_whitespace().next();
            let kind = match mode {
                Some("040000") => Some(ScopeKind::Directory),
                Some("100644" | "100755") => Some(ScopeKind::File),
                None => None,
                _ => return Err(invalid()),
            };
            if i + 1 < parts.len() && kind == Some(ScopeKind::File) {
                return Err(invalid());
            }
            if i + 1 == parts.len() {
                existing = kind;
            }
        }
        let declared = explicit.remove(path);
        let kind = match (existing, declared) {
            (Some(actual), Some(declared)) if actual != declared => return Err(invalid()),
            (Some(actual), _) => actual,
            (None, Some(declared)) => declared,
            (None, None) => return Err(invalid()),
        };
        result.push(WriteScope {
            path: path.clone(),
            kind,
        });
    }
    Ok(result)
}
/// Descriptor-relative traversal: neither validation nor creation follows symlinks.
/// Missing parent directories are prepared by the trusted supervisor, never mounted writable.
fn scope_path(root: &Path, scope: &WriteScope, create: bool, allow_missing: bool) -> Result<()> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::OpenOptionsExt,
        },
    };
    if !safe_relative(&scope.path) {
        return Err(invalid());
    }
    let mut directory = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(root)
        .map_err(|_| invalid())?;
    let parts: Vec<_> = scope.path.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        let name = CString::new(*part).map_err(|_| invalid())?;
        let final_path = i + 1 == parts.len();
        let is_directory = !final_path || scope.kind == ScopeKind::Directory;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if is_directory { libc::O_DIRECTORY } else { 0 };
        // directory is an owned, open directory descriptor; name has no slash or NUL.
        let mut fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::NotFound {
                return Err(invalid());
            }
            if allow_missing {
                return Ok(());
            }
            if !create {
                return Err(invalid());
            }
            if is_directory {
                // Creation remains relative to the validated parent descriptor.
                if unsafe { libc::mkdirat(directory.as_raw_fd(), name.as_ptr(), 0o700) } < 0 {
                    return Err(invalid());
                }
                fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
            } else {
                fd = unsafe {
                    libc::openat(
                        directory.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_WRONLY
                            | libc::O_CREAT
                            | libc::O_EXCL
                            | libc::O_NOFOLLOW
                            | libc::O_CLOEXEC,
                        0o600,
                    )
                };
            }
            if fd < 0 {
                return Err(invalid());
            }
            directory.sync_all().map_err(|_| unavailable())?;
        }
        // fd is fresh and nonnegative; this File exclusively owns and closes it.
        let file = unsafe { fs::File::from_raw_fd(fd) };
        let metadata = file.metadata().map_err(|_| invalid())?;
        if is_directory && !metadata.is_dir() || !is_directory && !metadata.is_file() {
            return Err(invalid());
        }
        if final_path && create {
            file.sync_all().map_err(|_| unavailable())?;
        }
        directory = file;
    }
    Ok(())
}
fn clone_workspace(manifest: &WorkspaceManifest, fault: CreateFault) -> Result<()> {
    let mut command = Command::new("/usr/bin/git");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(["clone", "--no-hardlinks", "--no-local", "--quiet"])
        .arg(&manifest.store)
        .arg(&manifest.path);
    if matches!(fault, CreateFault::DuringGit) {
        use std::os::unix::ffi::OsStrExt;
        let trace = manifest.log_path.with_extension("prepare-trace");
        let name = std::ffi::CString::new(trace.as_os_str().as_bytes()).map_err(|_| invalid())?;
        // A real Git process blocks opening its trace FIFO before any pack/helper work.
        if unsafe { libc::mkfifo(name.as_ptr(), 0o600) } < 0 {
            return Err(unavailable());
        }
        command
            .env("GIT_TRACE2_EVENT", trace)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        let mut child = command.spawn().map_err(|_| unavailable())?;
        let marker = manifest.log_path.with_extension("prepare-identity");
        let record = json(&(
            ExecutionHandle {
                id: Uuid::parse_str(
                    manifest
                        .path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .ok_or_else(invalid)?,
                )
                .map_err(|_| invalid())?,
                path: manifest.path.clone(),
            },
            identity(child.id()).map_err(|_| unavailable())?,
        ))?;
        let file = fs::File::create(marker).map_err(|_| unavailable())?;
        serde_json::to_writer(&file, &record).map_err(|_| unavailable())?;
        file.sync_all().map_err(|_| unavailable())?;
        if !child.wait().map_err(|_| unavailable())?.success() {
            return Err(unavailable());
        }
        return Ok(());
    }
    if !command
        .output()
        .map_err(|_| unavailable())?
        .status
        .success()
    {
        return Err(unavailable());
    }
    Ok(())
}
fn git(args: &[&str]) -> Result<String> {
    let o = Command::new("/usr/bin/git")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args(args)
        .output()
        .map_err(|_| unavailable())?;
    if !o.status.success() {
        return Err(unavailable());
    }
    String::from_utf8(o.stdout).map_err(|_| unavailable())
}
fn git_at(path: &Path, args: &[&str]) -> Result<String> {
    let mut a = vec!["-C", path.to_str().ok_or_else(invalid)?];
    a.extend_from_slice(args);
    git(&a)
}
#[derive(Debug)]
struct MembershipObservationError {
    pid: u32,
    operation: &'static str,
    source: std::io::Error,
}
impl std::fmt::Display for MembershipObservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "process {} {} membership observation failed: {}",
            self.pid, self.operation, self.source
        )
    }
}
impl std::error::Error for MembershipObservationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
fn membership_error(pid: u32, operation: &'static str, source: std::io::Error) -> std::io::Error {
    if process_disappeared(&source) {
        return source;
    }
    std::io::Error::new(
        source.kind(),
        MembershipObservationError {
            pid,
            operation,
            source,
        },
    )
}
fn identity(pid: u32) -> std::io::Result<ProcessIdentity> {
    let p = PathBuf::from(format!("/proc/{pid}"));
    let stat = fs::read_to_string(p.join("stat")).map_err(|e| membership_error(pid, "stat", e))?;
    let tail = stat
        .rsplit_once(')')
        .ok_or_else(|| std::io::Error::other("malformed stat"))?
        .1;
    let fields: Vec<_> = tail.split_whitespace().collect();
    let start_ticks = fields
        .get(19)
        .ok_or_else(|| std::io::Error::other("malformed stat"))?
        .parse()
        .map_err(|_| std::io::Error::other("malformed ticks"))?;
    Ok(ProcessIdentity {
        pid,
        start_ticks,
        boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")?
            .trim()
            .into(),
        namespace: fs::read_link(p.join("ns/pid"))
            .map_err(|e| membership_error(pid, "ns/pid", e))?
            .to_string_lossy()
            .into(),
        namespace_pid: Some(namespace_pid(
            &fs::read_to_string(p.join("status"))
                .map_err(|e| membership_error(pid, "status", e))?,
        )?),
    })
}
fn namespace_pid(status: &str) -> std::io::Result<u32> {
    status
        .lines()
        .find_map(|l| l.strip_prefix("NSpid:"))
        .and_then(|l| l.split_whitespace().last())
        .and_then(|p| p.parse().ok())
        .ok_or_else(|| std::io::Error::other("malformed namespace PID"))
}
fn host_start_ticks(stat: &str) -> std::io::Result<u64> {
    let (head, tail) = stat
        .rsplit_once(')')
        .ok_or_else(|| std::io::Error::other("malformed host stat"))?;
    let pid = head
        .split_once('(')
        .and_then(|(pid, _)| pid.trim().parse::<u32>().ok());
    let fields: Vec<_> = tail.split_whitespace().collect();
    if pid.is_none()
        || !fields.first().is_some_and(|state| {
            matches!(
                *state,
                "R" | "S" | "D" | "Z" | "T" | "t" | "X" | "x" | "K" | "W" | "P" | "I"
            )
        })
    {
        return Err(std::io::Error::other("malformed host stat state"));
    }
    fields
        .get(19)
        .and_then(|ticks| ticks.parse().ok())
        .ok_or_else(|| std::io::Error::other("malformed host stat ticks"))
}
fn validate_observer_frame(
    controller: Option<&ProcessIdentity>,
    stat: &str,
    namespace: &str,
    boot: &str,
    procfs: bool,
) -> std::io::Result<()> {
    let controller =
        controller.ok_or_else(|| std::io::Error::other("missing observer frame certificate"))?;
    let pid = stat
        .split_once('(')
        .and_then(|(pid, _)| pid.trim().parse::<u32>().ok());
    if !procfs
        || pid != Some(std::process::id())
        || namespace != controller.namespace
        || boot.trim() != controller.boot_id
    {
        return Err(std::io::Error::other(
            "original observer PID frame cannot be established",
        ));
    }
    host_start_ticks(stat)?;
    Ok(())
}
fn observer_frame(controller: Option<&ProcessIdentity>) -> std::io::Result<()> {
    let mut statfs = std::mem::MaybeUninit::<libc::statfs>::uninit();
    if unsafe { libc::statfs(c"/proc".as_ptr(), statfs.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let procfs = unsafe { statfs.assume_init() }.f_type == libc::PROC_SUPER_MAGIC;
    validate_observer_frame(
        controller,
        &fs::read_to_string("/proc/self/stat")?,
        &fs::read_link("/proc/self/ns/pid")?.to_string_lossy(),
        &fs::read_to_string("/proc/sys/kernel/random/boot_id")?,
        procfs,
    )
}
// Only the certified original PID1 can establish namespace-wide finalization.
// A missing namespace link or leader zombie state is deliberately insufficient.
#[cfg(test)]
fn observe_finalized(
    init: &ProcessIdentity,
    boot: std::io::Result<String>,
    read_stat: impl Fn(u32) -> std::io::Result<String>,
    group_ready: impl FnOnce(u32) -> std::io::Result<bool>,
) -> std::io::Result<bool> {
    if init.namespace_pid != Some(1) {
        return Ok(false);
    }
    observe_thread_group_finalized(init, boot, read_stat, group_ready)
}
fn observe_thread_group_finalized(
    init: &ProcessIdentity,
    boot: std::io::Result<String>,
    read_stat: impl Fn(u32) -> std::io::Result<String>,
    group_ready: impl FnOnce(u32) -> std::io::Result<bool>,
) -> std::io::Result<bool> {
    if boot?.trim() != init.boot_id {
        return Err(std::io::Error::other("original PID1 boot fence differs"));
    }
    let same = |stat: std::io::Result<String>| -> std::io::Result<bool> {
        match stat {
            Ok(stat) => Ok(host_start_ticks(&stat)? == init.start_ticks),
            Err(e) if process_disappeared(&e) => Ok(false),
            Err(e) => Err(e),
        }
    };
    if !same(read_stat(init.pid))? {
        return Ok(true);
    }
    let ready = match group_ready(init.pid) {
        Ok(ready) => ready,
        Err(e) if process_disappeared(&e) => {
            return Ok(!same(read_stat(init.pid))?);
        }
        Err(e) => return Err(e),
    };
    // Fence pidfd_open against numeric PID replacement before trusting readiness.
    if !same(read_stat(init.pid))? {
        return Ok(true);
    }
    Ok(ready)
}
fn finalized(init: &ProcessIdentity) -> std::io::Result<bool> {
    if init.namespace_pid != Some(1) {
        return Ok(false);
    }
    process_finalized(init)
}
fn process_finalized(init: &ProcessIdentity) -> std::io::Result<bool> {
    use std::os::fd::FromRawFd;
    observe_thread_group_finalized(
        init,
        fs::read_to_string("/proc/sys/kernel/random/boot_id"),
        |pid| fs::read_to_string(format!("/proc/{pid}/stat")),
        |pid| {
            // No PIDFD_THREAD: readiness covers the entire thread group.
            let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
            if raw < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let _fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) };
            let mut poll = libc::pollfd {
                fd: raw,
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut poll, 1, 0) } < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if poll.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
                return Err(std::io::Error::other("pidfd readiness observation failed"));
            }
            Ok(poll.revents & libc::POLLIN != 0)
        },
    )
}
fn alive(p: &ProcessIdentity) -> std::io::Result<bool> {
    observe_alive(p, identity, |pid| {
        fs::read_to_string(format!("/proc/{pid}/stat"))
    })
}
fn observe_alive(
    p: &ProcessIdentity,
    read_identity: impl FnOnce(u32) -> std::io::Result<ProcessIdentity>,
    read_stat: impl FnOnce(u32) -> std::io::Result<String>,
) -> std::io::Result<bool> {
    let current = match read_identity(p.pid) {
        Ok(current) => current,
        Err(error) if process_disappeared(&error) => return Ok(false),
        Err(error) => return Err(error),
    };
    if current != *p {
        return Ok(false);
    }
    let stat = match read_stat(p.pid) {
        Ok(stat) => stat,
        Err(error) if process_disappeared(&error) => return Ok(false),
        Err(error) => return Err(error),
    };
    let state = stat
        .rsplit_once(')')
        .and_then(|(_, tail)| tail.split_whitespace().next())
        .ok_or_else(|| std::io::Error::other("malformed process state"))?;
    if !matches!(
        state,
        "R" | "S" | "D" | "Z" | "T" | "t" | "X" | "x" | "K" | "W" | "P" | "I"
    ) {
        return Err(std::io::Error::other("unknown process state"));
    }
    Ok(state != "Z")
}
// The monitor keeps this descriptor until shutdown proof is complete. Namespace
// inode numbers otherwise may be reused immediately after short-lived PID1 exits.
fn pin_namespace(init: &ProcessIdentity) -> std::io::Result<fs::File> {
    use std::os::unix::fs::MetadataExt;
    let path = format!("/proc/{}/ns/pid", init.pid);
    let pin = fs::File::open(&path)?;
    let pinned = pin.metadata()?;
    let current = fs::metadata(&path)?;
    if pinned.dev() != current.dev()
        || pinned.ino() != current.ino()
        || init.namespace != format!("pid:[{}]", pinned.ino())
        || identity(init.pid)? != *init
        || !alive(init)?
    {
        return Err(std::io::Error::other("namespace pin identity mismatch"));
    }
    Ok(pin)
}
fn process_disappeared(error: &std::io::Error) -> bool {
    // /proc task reads can return ESRCH after enumeration while the inode remains.
    // Both errors prove disappearance at that read, unlike permission or other failures.
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(libc::ESRCH)
}
fn inventory(namespace: &str) -> std::io::Result<Vec<ProcessIdentity>> {
    use std::os::unix::fs::MetadataExt;
    let mut members = vec![];
    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) if process_disappeared(&e) => continue,
            Err(e) => {
                return Err(membership_error(pid, "metadata", e));
            }
        };
        // All descendants retain this host uid: user namespace is fixed and caps dropped.
        if meta.uid() != unsafe { libc::geteuid() } {
            continue;
        }
        // NSpid is scoped to the host /proc mount. A single component proves the
        // process is outside every child PID namespace, even if ptrace hides ns links.
        let status = match fs::read_to_string(entry.path().join("status")) {
            Ok(s) => s,
            Err(e) if process_disappeared(&e) => continue,
            Err(e) => {
                return Err(membership_error(pid, "status", e));
            }
        };
        let nspid = status.lines().find_map(|l| l.strip_prefix("NSpid:"));
        if nspid.is_some_and(|n| n.split_whitespace().count() == 1) {
            continue;
        }
        if nspid.is_none() {
            return Err(std::io::Error::other(format!(
                "process {pid} lacks readable NSpid"
            )));
        }

        match identity(pid) {
            Ok(p) if p.namespace == namespace => {
                if alive(&p)? {
                    members.push(p);
                }
            }
            Ok(_) => {}
            Err(e) if process_disappeared(&e) => {}
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(members)
}
fn signal(p: &ProcessIdentity) -> Result<()> {
    use std::os::fd::{AsRawFd, FromRawFd};
    if !alive(p).map_err(|_| unavailable())? {
        return Ok(());
    }
    // Open then re-check: PID reuse between observation and pidfd_open cannot target a replacement.
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, p.pid, 0) } as i32;
    if raw < 0 {
        return if !alive(p).map_err(|_| unavailable())? {
            Ok(())
        } else {
            Err(unavailable())
        };
    }
    let fd = unsafe { std::os::fd::OwnedFd::from_raw_fd(raw) };
    if !alive(p).map_err(|_| unavailable())? {
        return Ok(());
    }
    if unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd.as_raw_fd(),
            libc::SIGKILL,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    } < 0
        && alive(p).map_err(|_| unavailable())?
    {
        return Err(unavailable());
    }
    Ok(())
}
trait NamespaceObserver {
    type Pin;
    fn frame(&self, controller: Option<&ProcessIdentity>) -> std::io::Result<()>;
    fn alive(&self, process: &ProcessIdentity) -> std::io::Result<bool>;
    fn pin(&self, process: &ProcessIdentity) -> std::io::Result<Self::Pin>;
    fn finalized(&self, process: &ProcessIdentity) -> std::io::Result<bool>;
    fn inventory(&self, namespace: &str) -> std::io::Result<Vec<ProcessIdentity>>;
    fn signal(&self, process: &ProcessIdentity) -> std::io::Result<()>;
}
struct ProcObserver;
impl NamespaceObserver for ProcObserver {
    type Pin = fs::File;
    fn frame(&self, controller: Option<&ProcessIdentity>) -> std::io::Result<()> {
        observer_frame(controller)
    }
    fn alive(&self, process: &ProcessIdentity) -> std::io::Result<bool> {
        alive(process)
    }
    fn pin(&self, process: &ProcessIdentity) -> std::io::Result<Self::Pin> {
        pin_namespace(process)
    }
    fn finalized(&self, process: &ProcessIdentity) -> std::io::Result<bool> {
        finalized(process)
    }
    fn inventory(&self, namespace: &str) -> std::io::Result<Vec<ProcessIdentity>> {
        inventory(namespace)
    }
    fn signal(&self, process: &ProcessIdentity) -> std::io::Result<()> {
        signal(process)
            .map_err(|_| std::io::Error::other("pidfd signal of recorded membership failed"))
    }
}
fn decide_namespace<O: NamespaceObserver>(
    init: &ProcessIdentity,
    prior: &[ProcessIdentity],
    controller: Option<&ProcessIdentity>,
    cancel: bool,
    held_pin: Option<&O::Pin>,
    observer: &O,
) -> std::io::Result<(&'static str, Vec<ProcessIdentity>)> {
    observer.frame(controller)?;
    if observer.finalized(init)? {
        return Ok(("stopped", vec![]));
    }
    let mut owned_pin = None;
    if held_pin.is_none() {
        for process in std::iter::once(init)
            .chain(prior.iter())
            .filter(|p| p.namespace == init.namespace)
        {
            if observer.alive(process)? {
                match observer.pin(process) {
                    Ok(pin) => {
                        owned_pin = Some(pin);
                        break;
                    }
                    Err(e) if process_disappeared(&e) => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }
    let controller_lost = match controller {
        Some(controller) => !observer.alive(controller)?,
        None => true,
    };
    if held_pin.is_none() && owned_pin.is_none() {
        if observer.finalized(init)? {
            return Ok(("stopped", vec![]));
        }
        if !controller_lost {
            // Defer without inventory, stop classification or signals. The monitor
            // may finish using its explicitly passed gated-launch descriptor.
            return Ok(("running", prior.to_vec()));
        }
        return Err(std::io::Error::other(
            "namespace incarnation lacks certified finalization or verified pin",
        ));
    }
    // Both the owned external descriptor and borrowed monitor descriptor outlive
    // inventory and every signal; a cached identity boolean is never authority.
    // An incomplete scan contributes no membership or signal authority. Keep
    // the same verified descriptor while retrying complete scans only. The
    // window limits starting another attempt, not synchronous syscall runtime.
    let retry_started = std::time::Instant::now();
    let retry_window = std::time::Duration::from_millis(20);
    let mut scans = 0;
    let members = loop {
        scans += 1;
        match observer.inventory(&init.namespace) {
            Ok(members) => break members,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::PermissionDenied
                    || scans >= 3
                    || retry_started.elapsed() >= retry_window
                {
                    return Err(error);
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
                if retry_started.elapsed() >= retry_window {
                    return Err(error);
                }
                observer.frame(controller)?;
            }
        }
    };
    if members.is_empty() {
        return Ok(("stopped", members));
    }
    if cancel || controller_lost {
        for member in &members {
            observer.signal(member)?;
        }
        Ok(("cancelling", members))
    } else {
        Ok(("running", members))
    }
}
async fn reconcile_locked(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    reconcile_with_pin(tx, id, None).await
}
async fn reconcile_with_pin(
    tx: &Transaction<'_>,
    id: Uuid,
    held_pin: Option<&fs::File>,
) -> Result<()> {
    let r=tx.query_one("SELECT state,desired_cancel OR deadline<=clock_timestamp(),init_process,members,controller FROM supervised_workspaces WHERE id=$1 FOR UPDATE",&[&id]).await?;
    let state: String = r.get(0);
    let active_epoch: Uuid = tx
        .query_one("SELECT active_epoch FROM deployment WHERE singleton", &[])
        .await?
        .get(0);
    let cancel = r.get::<_, Option<bool>>(1).unwrap_or(false)
        || !execution_dispatch_current(tx, id, active_epoch).await?;
    if matches!(
        state.as_str(),
        "released" | "deleting" | "stopped" | "quarantined"
    ) {
        return Ok(());
    }
    let init = r
        .get::<_, Option<serde_json::Value>>(2)
        .map(decode::<ProcessIdentity>)
        .transpose()?;
    let prior: Vec<ProcessIdentity> = decode(r.get(3))?;
    let controller = r
        .get::<_, Option<serde_json::Value>>(4)
        .map(decode::<ProcessIdentity>)
        .transpose()?;
    let mut reason = None;
    let (next, members) = if let Some(init) = init {
        if let Some(pin) = held_pin {
            use std::os::unix::fs::MetadataExt;
            if format!("pid:[{}]", pin.metadata().map_err(|_| unavailable())?.ino())
                != init.namespace
            {
                return Err(unavailable());
            }
        }
        match decide_namespace(
            &init,
            &prior,
            controller.as_ref(),
            cancel,
            held_pin,
            &ProcObserver,
        ) {
            Ok(decision) => decision,
            Err(error) => {
                reason = Some(error.to_string());
                ("quarantined", prior)
            }
        }
    } else if state == "registered" || state == "ready" {
        if cancel || state == "registered" {
            ("stopped", vec![])
        } else {
            ("ready", vec![])
        }
    } else {
        reason = Some(
            if state == "preparing" {
                "Git preparation completion and full membership are unknown"
            } else {
                "launch identity and full membership are unknown"
            }
            .into(),
        );
        ("quarantined", prior)
    };
    if next != state || json(&members)? != r.get::<_, serde_json::Value>(3) {
        tx.execute("INSERT INTO supervised_observations(id,execution_id,state,members) VALUES($1,$2,$3,$4)",&[&Uuid::new_v4(),&id,&next,&json(&members)?]).await?;
    }
    tx.execute("UPDATE supervised_workspaces SET state=$2,members=$3,desired_cancel=desired_cancel OR $4,quarantine_reason=$5 WHERE id=$1",&[&id,&next,&json(&members)?,&cancel,&reason]).await?;
    Ok(())
}
/// Trusted local operator entry point. DSN comes from the environment, never argv/logs.
/// Workload launch is held by a pipe until its complete namespace identity is durable.
async fn monitor(broker: Broker, id: Uuid, gate: Option<MonitorGate>) -> Result<()> {
    use std::{
        io::Write,
        os::fd::AsRawFd,
        os::unix::{fs::OpenOptionsExt, process::CommandExt},
        process::Stdio,
    };
    let mut c = storage::client(&broker.dsn).await?;
    let tx = c.transaction().await?;
    storage::lock(&tx).await?;
    schema(&tx).await?;
    storage::active(&tx, broker.epoch).await?;
    let r=tx.query_one("SELECT w.state,w.manifest,w.argv,w.desired_cancel,w.controller FROM supervised_workspaces w WHERE w.id=$1 FOR UPDATE OF w",&[&id]).await?;
    if r.get::<_, String>(0) != "starting"
        || r.get::<_, bool>(3)
        || r.get::<_, Option<serde_json::Value>>(4).is_some()
        || !execution_dispatch_current(&tx, id, broker.epoch).await?
    {
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    let manifest: WorkspaceManifest = decode(r.get(1))?;
    let argv: Vec<String> = decode(r.get(2))?;
    let controller = identity(std::process::id()).map_err(|_| unavailable())?;
    observer_frame(Some(&controller)).map_err(|_| unavailable())?;
    tx.execute(
        "UPDATE supervised_workspaces SET controller=$2 WHERE id=$1",
        &[&id, &json(&controller)?],
    )
    .await?;
    tx.commit().await?;
    if let Some(gate) = gate {
        gate.claimed.notify_one();
        gate.resume.notified().await;
    }
    // Durable controller claim precedes bwrap spawn. A repeated monitor cannot relaunch.
    let tx = c.transaction().await?;
    storage::lock(&tx).await?;
    schema(&tx).await?;
    storage::active(&tx, broker.epoch).await?;
    let row=tx.query_one("SELECT w.state,w.desired_cancel,w.deadline>clock_timestamp() FROM supervised_workspaces w WHERE w.id=$1 FOR UPDATE OF w",&[&id]).await?;
    if row.get::<_, String>(0) != "starting"
        || row.get::<_, bool>(1)
        || !row.get::<_, Option<bool>>(2).unwrap_or(false)
        || !execution_dispatch_current(&tx, id, broker.epoch).await?
    {
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    physical_usage_known(&tx, id, None).await?;
    let mission: Uuid = tx
        .query_one(
            "SELECT coalesce($1,(SELECT mission_id FROM tasks WHERE id=$2))",
            &[&manifest.mission, &manifest.task],
        )
        .await?
        .get(0);
    crate::kernel::dispatch_enabled(&tx, mission).await?;
    let packet_fd = if let Some(input) = &manifest.input {
        let packet:serde_json::Value=tx.query_opt("SELECT p.packet FROM episode_executions x JOIN mailbox_episodes e ON e.id=x.episode_id JOIN mailbox_inputs p ON p.id=e.input_packet WHERE x.execution_id=$1 AND p.id=$2 AND e.input_digest=$3 AND x.input_digest=$3", &[&id,&input.id,&input.digest]).await?.ok_or_else(||crate::domain::err(ErrorCode::DependencyUnmet))?.get(0);
        let bytes = serde_json::to_vec(&packet).map_err(|_| invalid())?;
        if bytes.len() > 524288 || crate::artifacts::sha256(&bytes) != input.digest {
            return Err(crate::domain::err(ErrorCode::DependencyUnmet));
        }
        Some(sealed_packet(&bytes)?)
    } else {
        None
    };
    let info_path = manifest.log_path.with_extension("info");
    let info = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&info_path)
        .map_err(|_| unavailable())?;
    // The status descriptor and optional sealed captured-input descriptor cross
    // exec. Neither exposes credentials or a writable parent filesystem.
    if unsafe { libc::fcntl(info.as_raw_fd(), libc::F_SETFD, 0) } < 0 {
        return Err(unavailable());
    }
    let log = fs::OpenOptions::new()
        .append(true)
        .create_new(true)
        .mode(0o600)
        .open(&manifest.log_path)
        .map_err(|_| unavailable())?;
    let mut command = Command::new("/usr/bin/bwrap");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .args([
            "--unshare-user",
            "--unshare-pid",
            "--unshare-net",
            "--unshare-ipc",
            "--unshare-uts",
            "--disable-userns",
            "--new-session",
            "--die-with-parent",
            "--cap-drop",
            "ALL",
            "--ro-bind",
            "/usr",
            "/usr",
            "--symlink",
            "usr/bin",
            "/bin",
            "--symlink",
            "usr/lib",
            "/lib",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/tmp",
            "--ro-bind",
        ])
        .arg(&manifest.path)
        .arg("/workspace");
    if let Some(packet) = &packet_fd {
        command
            .args(["--perms", "0400", "--ro-bind-data"])
            .arg(packet.as_raw_fd().to_string())
            .arg("/episode-input.json");
    }
    for scope in &manifest.scopes {
        scope_path(&manifest.path, scope, false, false)?;
        command
            .arg("--bind")
            .arg(manifest.path.join(&scope.path))
            .arg(format!("/workspace/{}", scope.path));
    }
    command
        .args(["--chdir", "/workspace", "--block-fd", "0", "--info-fd"])
        .arg(info.as_raw_fd().to_string())
        .arg("--")
        .args(&argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::from(log.try_clone().map_err(|_| unavailable())?))
        .stderr(Stdio::from(log));
    command.process_group(0);
    let mut child = command.spawn().map_err(|_| unavailable())?;
    // Close descriptor in the parent; the child inherited its duplicate.
    drop(info);
    drop(packet_fd);
    let mut init = None;
    for _ in 0..100 {
        if let Ok(bytes) = fs::read(&info_path)
            && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && let Some(pid) = v["child-pid"].as_u64()
        {
            init = Some(identity(pid as u32).map_err(|_| unavailable())?);
            break;
        }
        if child.try_wait().map_err(|_| unavailable())?.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let Some(init) = init else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(unavailable());
    };
    if init.namespace == controller.namespace || init.namespace_pid != Some(1) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(unavailable());
    }
    // Acquire and verify the incarnation pin before unblocking any workload.
    let namespace_pin = match pin_namespace(&init) {
        Ok(pin) => pin,
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(unavailable());
        }
    };
    // Time can expire while bwrap prepares its blocked namespace. Recheck before eligibility.
    let live:bool=tx.query_one("SELECT NOT desired_cancel AND deadline>clock_timestamp() FROM supervised_workspaces WHERE id=$1",&[&id]).await?.get(0);
    if !live {
        let _ = child.kill();
        let _ = child.wait();
        return Err(crate::domain::err(ErrorCode::DependencyUnmet));
    }
    tx.execute(
        "UPDATE supervised_workspaces SET init_process=$2,members=$3,launcher=$4,state='running' WHERE id=$1",
        &[&id, &json(&init)?, &json(&vec![init.clone()])?, &json(&identity(child.id()).map_err(|_|unavailable())?)?],
    )
    .await?;
    // Keep the exact launch incarnation attributable even if a fast workload
    // exits before the first changed inventory observation.
    tx.execute("INSERT INTO supervised_observations(id,execution_id,state,members) VALUES($1,$2,'running',$3)", &[&Uuid::new_v4(),&id,&json(&vec![init.clone()])?]).await?;
    tx.commit().await?;
    // The workload can only start after this commit. Pipe closure/failure cannot declare stop.
    child
        .stdin
        .take()
        .ok_or_else(unavailable)?
        .write_all(b"1")
        .map_err(|_| unavailable())?;
    let mut observed_exit = None;
    let final_state = loop {
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        // Recovery epoch rotation uses the same cancellation path.
        if storage::active(&tx, broker.epoch).await.is_err() {
            tx.execute(
                "UPDATE supervised_workspaces SET desired_cancel=true WHERE id=$1",
                &[&id],
            )
            .await?;
        }
        reconcile_with_pin(&tx, id, Some(&namespace_pin)).await?;
        let state: String = tx
            .query_one(
                "SELECT state FROM supervised_workspaces WHERE id=$1",
                &[&id],
            )
            .await?
            .get(0);
        tx.commit().await?;
        if state == "stopped" || state == "quarantined" {
            break state;
        }
        if observed_exit.is_none() {
            observed_exit = child.try_wait().map_err(|_| unavailable())?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    };
    // Reap launcher before releasing the verified namespace incarnation pin.
    // A proved empty namespace is not an exit code. Allow the trusted bwrap
    // launcher to publish its actual wait result; forced termination remains a signal.
    let exit_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while observed_exit.is_none()
        && final_state == "stopped"
        && std::time::Instant::now() < exit_deadline
    {
        observed_exit = child.try_wait().map_err(|_| unavailable())?;
        if observed_exit.is_none() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
    if observed_exit.is_none() {
        child.kill().map_err(|_| unavailable())?;
    }
    let exit = child.wait().map_err(|_| unavailable())?;
    drop(namespace_pin);
    fs::File::open(&manifest.log_path)
        .and_then(|f| f.sync_all())
        .map_err(|_| unavailable())?;
    fs::File::open(&manifest.path)
        .and_then(|f| f.sync_all())
        .map_err(|_| unavailable())?;
    if manifest.episode.is_some() {
        use std::os::unix::process::ExitStatusExt;
        let result = ExecutionResult {
            execution: id,
            argv,
            candidate: manifest.candidate.clone(),
            recovery_epoch: broker.epoch,
            controller,
            exit_code: exit.code(),
            signal: exit.signal(),
            namespace_stopped: final_state == "stopped",
            logs: log_digest(&manifest, id)?,
            completion: CompletionProvenance::MonitorWait,
        };
        let tx = c.transaction().await?;
        storage::lock(&tx).await?;
        crate::kernel::schema(&tx).await?;
        let _winner = persist_result(&tx, &result).await?;
        tx.commit().await?;
    }
    Ok(())
}

fn sealed_packet(bytes: &[u8]) -> Result<fs::File> {
    use std::{
        io::{Seek, SeekFrom, Write},
        os::fd::{AsRawFd, FromRawFd},
    };
    // Read-only bind-data receives an immutable copy, closing a path/content race
    // between final digest verification and bubblewrap's mount setup.
    let fd =
        unsafe { libc::memfd_create(c"corenius-episode-input".as_ptr(), libc::MFD_ALLOW_SEALING) };
    if fd < 0 {
        return Err(unavailable());
    }
    let mut file = unsafe { fs::File::from_raw_fd(fd) };
    file.write_all(bytes)
        .and_then(|_| file.seek(SeekFrom::Start(0)).map(|_| ()))
        .map_err(|_| unavailable())?;
    let seals = libc::F_SEAL_WRITE | libc::F_SEAL_GROW | libc::F_SEAL_SHRINK | libc::F_SEAL_SEAL;
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_ADD_SEALS, seals) } < 0 {
        return Err(unavailable());
    }
    Ok(file)
}
pub async fn run_monitor(broker: Broker, id: Uuid) -> Result<()> {
    monitor(broker, id, None).await
}
pub async fn run_monitor_with_gate(broker: Broker, id: Uuid, gate: MonitorGate) -> Result<()> {
    monitor(broker, id, Some(gate)).await
}

// Assembly observes the service process through the same established observer
// frame parser; this does not turn a generic process into a certified PID1.
pub(crate) fn local_process_identity() -> std::io::Result<ProcessIdentity> {
    let process = identity(std::process::id())?;
    observer_frame(Some(&process))?;
    Ok(process)
}
pub(crate) fn validate_process_frame(process: &ProcessIdentity) -> std::io::Result<()> {
    observer_frame(Some(process))
}
impl Supervisor {
    pub(crate) fn workspace_root(&self) -> &Path {
        &self.root
    }
}
pub(crate) fn transport_alive(process: &ProcessIdentity) -> std::io::Result<bool> {
    alive(process)
}

/// Reconciliation observes effects; usage reservations are deliberately unchanged.
pub(crate) async fn reconcile_attempt_cancel(
    tx: &Transaction<'_>,
    attempt: Uuid,
) -> Result<String> {
    let ids=tx.query("SELECT w.id FROM supervised_workspaces w WHERE w.attempt_id=$1 OR EXISTS(SELECT 1 FROM episode_executions x WHERE x.execution_id=w.id AND x.attempt_id=$1) ORDER BY w.id", &[&attempt]).await?;
    for row in &ids {
        reconcile_locked(tx, row.get(0)).await?;
    }
    // Native confirmation can precede monitor launch publication. Re-observe the
    // registered pinned receipt only after reconciliation has inspected physical
    // state; no waiting under this lock and no native mutation/retry occurs.
    let native=tx.query("SELECT o.request,o.controller,o.recovery_epoch FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.attempt_id=$1 AND o.state='confirmed' AND NOT EXISTS(SELECT 1 FROM episode_runtime_evidence e WHERE e.operation_id=o.id AND e.evidence->>'kind'='registered_native_execution_bound')", &[&attempt]).await?;
    for row in native {
        let prepared: crate::episode_runtime::PreparedStart = serde_json::from_value(row.get(0))
            .map_err(|_| crate::domain::err(ErrorCode::IncompatibleSchema))?;
        let operation = tx
            .query_one(
                "SELECT thread_id,turn_id,effect_id FROM episode_runtime_operations WHERE id=$1",
                &[&prepared.start.id],
            )
            .await?;
        let turn = crate::episode_runtime::NativeTurn {
            thread: operation
                .get::<_, Option<String>>(0)
                .ok_or_else(unavailable)?,
            turn: operation
                .get::<_, Option<String>>(1)
                .ok_or_else(unavailable)?,
            effect: operation
                .get::<_, Option<String>>(2)
                .ok_or_else(unavailable)?,
        };
        match crate::episode_runtime::record_effect_binding(
            tx,
            &prepared,
            &turn,
            row.get(1),
            row.get(2),
        )
        .await
        {
            Ok(()) => {}
            Err(error) if error.code == ErrorCode::DependencyUnmet => {}
            Err(error) => return Err(error),
        }
    }
    let uncertain:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM supervised_workspaces w WHERE (w.attempt_id=$1 OR EXISTS(SELECT 1 FROM episode_executions x WHERE x.execution_id=w.id AND x.attempt_id=$1)) AND w.state='quarantined') OR EXISTS(SELECT 1 FROM gateway_operations WHERE attempt_id=$1 AND state IN ('dispatching','outcome_unknown','quarantined')) OR EXISTS(SELECT 1 FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.attempt_id=$1 AND o.state IN ('dispatching','outcome_unknown','quarantined'))", &[&attempt]).await?.get(0);
    let stopped:bool=tx.query_one("SELECT NOT EXISTS(SELECT 1 FROM supervised_workspaces w WHERE (w.attempt_id=$1 OR EXISTS(SELECT 1 FROM episode_executions x WHERE x.execution_id=w.id AND x.attempt_id=$1)) AND w.state NOT IN ('stopped','released'))", &[&attempt]).await?.get(0);
    // A confirmed RPC proves its native effect exists, not that it is stopped.
    // Only an immutable registered-transport observation independently binds the
    // original operation/effect to this execution. Caller arguments alone do
    // not establish that correspondence or permit borrowing its Stop proof.
    let unbound_native:bool=tx.query_one("SELECT EXISTS(SELECT 1 FROM gateway_operations o WHERE o.attempt_id=$1 AND o.kind='start' AND o.state='confirmed' AND NOT EXISTS(SELECT 1 FROM supervised_workspaces w LEFT JOIN episode_executions x ON x.execution_id=w.id WHERE w.id::text=o.request->'arguments'->>'execution' AND (w.attempt_id=o.attempt_id OR x.attempt_id=o.attempt_id) AND EXISTS(SELECT 1 FROM kernel_gateway_usage_evidence e WHERE e.operation_id=o.id AND e.evidence->>'kind'='legacy_native_usage_observed' AND e.evidence->'intent'=o.request AND e.evidence->>'controller'=o.controller::text AND e.evidence->>'recovery_epoch'=o.recovery_epoch::text AND e.evidence->'usage'=o.usage AND e.evidence->'usage'->>'kind'='measured_zero')))", &[&attempt]).await?.get(0);
    let unbound_episode: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id JOIN supervised_workspaces w ON w.id=x.execution_id WHERE x.attempt_id=$1 AND o.state='confirmed' AND NOT EXISTS(SELECT 1 FROM episode_runtime_evidence e WHERE e.operation_id=o.id AND e.evidence->>'kind'='registered_native_execution_bound' AND e.evidence->'request'=o.request AND e.evidence->>'controller'=o.controller::text AND e.evidence->>'recovery_epoch'=o.recovery_epoch::text AND e.evidence->'init_process'=w.init_process AND e.evidence->'turn'->>'thread'=o.thread_id AND e.evidence->'turn'->>'turn'=o.turn_id AND e.evidence->'turn'->>'effect'=o.effect_id))", &[&attempt]).await?.get(0);
    // An unallocated Yield may retain unresolved external execution references.
    // Only an exact empty checkpoint proves a prior refund; conflicting or
    // malformed checkpoint evidence must expose uncertainty as well as hold it.
    let uncertain_checkpoint: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM mailbox_episodes e JOIN mailbox_checkpoints c ON c.episode_id=e.id WHERE e.attempt_id=$1 AND e.start_operation IS NULL AND NOT EXISTS(SELECT 1 FROM episode_executions x WHERE x.episode_id=e.id) AND (c.principal_id<>e.principal_id OR c.executions<>'[]'::jsonb))", &[&attempt]).await?.get(0);
    let disposition = if uncertain || unbound_native || unbound_episode || uncertain_checkpoint {
        "quarantined"
    } else if stopped {
        "cancelled"
    } else {
        "cancellation_requested"
    };
    tx.execute(
        "UPDATE attempts SET status=$2 WHERE id=$1 AND desired_cancel",
        &[&attempt, &disposition],
    )
    .await?;
    if disposition == "cancelled" {
        tx.execute("UPDATE mailbox_episodes SET state='cancelled' WHERE attempt_id=$1 AND state='cancellation_requested'", &[&attempt]).await?;
    }
    if disposition == "cancelled" {
        // Old public Yield atomically refunded unused capacity but left its claim
        // set. The immutable exact-principal checkpoint proves that refund; a
        // cancelled state alone cannot distinguish it from an outstanding claim.
        tx.execute("UPDATE mailbox_episodes e SET owned_capacity=false WHERE e.attempt_id=$1 AND e.owned_capacity AND e.start_operation IS NULL AND NOT EXISTS(SELECT 1 FROM episode_executions x WHERE x.episode_id=e.id) AND EXISTS(SELECT 1 FROM mailbox_checkpoints c WHERE c.episode_id=e.id AND c.principal_id=e.principal_id AND c.executions='[]'::jsonb) AND NOT EXISTS(SELECT 1 FROM mailbox_checkpoints c WHERE c.episode_id=e.id AND (c.principal_id<>e.principal_id OR c.executions<>'[]'::jsonb))", &[&attempt]).await?;
        // Refund only unallocated claims with no conflicting checkpoint or start.
        // Allocated siblings retain their separate execution release contract.
        tx.execute("WITH released AS (UPDATE mailbox_episodes e SET owned_capacity=false WHERE e.attempt_id=$1 AND e.owned_capacity AND e.state='cancelled' AND e.start_operation IS NULL AND NOT EXISTS(SELECT 1 FROM episode_executions x WHERE x.episode_id=e.id) AND NOT EXISTS(SELECT 1 FROM mailbox_checkpoints c WHERE c.episode_id=e.id) RETURNING mission_id) UPDATE missions m SET used_capacity=m.used_capacity-r.n FROM (SELECT mission_id,count(*) AS n FROM released GROUP BY mission_id) r WHERE m.id=r.mission_id", &[&attempt]).await?;
    }
    if disposition == "cancelled" {
        // Independently owned continuation history is not the original claim.
        // Require no original physical binding and stopped, known effects and
        // usage under the admission lock; cumulative budgets remain consumed.
        let unused: bool = tx.query_one(
            "SELECT NOT EXISTS(SELECT 1 FROM supervised_workspaces WHERE attempt_id=$1) AND NOT EXISTS(SELECT 1 FROM episode_executions WHERE attempt_id=$1 AND NOT owns_capacity) AND NOT EXISTS(SELECT 1 FROM gateway_operations WHERE attempt_id=$1 AND state<>'prepared') AND NOT EXISTS(SELECT 1 FROM operations WHERE attempt_id=$1 AND (kind<>'prepare_attempt' OR state<>'prepared'))",
            &[&attempt],
        ).await?.get(0);
        let unknown_usage = crate::kernel::legacy_usage_unknown(tx, attempt).await?
            || tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_runtime_operations o JOIN episode_executions x ON x.execution_id=o.execution_id WHERE x.attempt_id=$1 AND o.state IN ('confirmed','failed_known') AND COALESCE(o.usage->>'kind','unknown')<>'measured_zero')", &[&attempt]).await?.get::<_, bool>(0);
        if unused && !unknown_usage {
            tx.execute("WITH released AS (UPDATE reservations SET state='released' WHERE attempt_id=$1 AND execution_id IS NULL AND state='reserved' RETURNING mission_id) UPDATE missions SET used_capacity=used_capacity-1 WHERE id IN (SELECT mission_id FROM released)", &[&attempt]).await?;
        }
    }
    Ok(disposition.into())
}

#[cfg(test)]
mod observation_tests {
    use super::*;
    #[test]
    fn pinned_inventory_transient_permission_error_requires_fresh_complete_scan() {
        use std::cell::Cell;
        struct Transient {
            scans: Cell<usize>,
            signals: Cell<usize>,
        }
        impl NamespaceObserver for Transient {
            type Pin = ();
            fn frame(&self, _: Option<&ProcessIdentity>) -> std::io::Result<()> {
                Ok(())
            }
            fn alive(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(true)
            }
            fn pin(&self, _: &ProcessIdentity) -> std::io::Result<()> {
                Ok(())
            }
            fn finalized(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(false)
            }
            fn inventory(&self, _: &str) -> std::io::Result<Vec<ProcessIdentity>> {
                let scans = self.scans.get();
                self.scans.set(scans + 1);
                if scans == 0 {
                    Err(std::io::Error::from_raw_os_error(libc::EACCES))
                } else {
                    Ok(vec![])
                }
            }
            fn signal(&self, _: &ProcessIdentity) -> std::io::Result<()> {
                self.signals.set(self.signals.get() + 1);
                Ok(())
            }
        }
        let original = identity(std::process::id()).unwrap();
        let observer = Transient {
            scans: Cell::new(0),
            signals: Cell::new(0),
        };
        let held_pin = ();
        let (state, members) = decide_namespace(
            &original,
            std::slice::from_ref(&original),
            None,
            true,
            Some(&held_pin),
            &observer,
        )
        .unwrap();
        assert_eq!(state, "stopped");
        assert!(members.is_empty());
        assert_eq!(observer.scans.get(), 2);
        assert_eq!(observer.signals.get(), 0);
    }
    #[test]
    fn whole_scan_retries_preserve_pin_bounds_and_fresh_signal_authority() {
        use std::{
            cell::{Cell, RefCell},
            collections::VecDeque,
            rc::Rc,
            time::Duration,
        };
        struct Pin(Rc<Cell<bool>>);
        impl Drop for Pin {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        struct Script {
            held: Rc<Cell<bool>>,
            scans: Cell<usize>,
            signals: RefCell<Vec<ProcessIdentity>>,
            results: RefCell<VecDeque<std::io::Result<Vec<ProcessIdentity>>>>,
            frame_valid: bool,
            pin_valid: bool,
            process_alive: bool,
            finalized: bool,
            delay: Duration,
        }
        impl NamespaceObserver for Script {
            type Pin = Pin;
            fn frame(&self, _: Option<&ProcessIdentity>) -> std::io::Result<()> {
                if self.frame_valid {
                    Ok(())
                } else {
                    Err(std::io::Error::other("unproven observer frame"))
                }
            }
            fn alive(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(self.process_alive)
            }
            fn pin(&self, _: &ProcessIdentity) -> std::io::Result<Pin> {
                if !self.pin_valid {
                    return Err(std::io::Error::other("missing verified pin"));
                }
                self.held.set(true);
                Ok(Pin(self.held.clone()))
            }
            fn finalized(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(self.finalized)
            }
            fn inventory(&self, _: &str) -> std::io::Result<Vec<ProcessIdentity>> {
                assert!(
                    self.held.get(),
                    "every entire scan retains the verified descriptor"
                );
                self.scans.set(self.scans.get() + 1);
                std::thread::sleep(self.delay);
                self.results
                    .borrow_mut()
                    .pop_front()
                    .expect("unexpected extra scan")
            }
            fn signal(&self, p: &ProcessIdentity) -> std::io::Result<()> {
                assert!(self.held.get());
                self.signals.borrow_mut().push(p.clone());
                Ok(())
            }
        }
        let original = identity(std::process::id()).unwrap();
        let error = |errno| Err(std::io::Error::from_raw_os_error(errno));
        let script = |results: Vec<std::io::Result<Vec<ProcessIdentity>>>| Script {
            held: Rc::new(Cell::new(false)),
            scans: Cell::new(0),
            signals: RefCell::new(vec![]),
            results: RefCell::new(results.into()),
            frame_valid: true,
            pin_valid: true,
            process_alive: true,
            finalized: false,
            delay: Duration::ZERO,
        };
        let mut stale = original.clone();
        stale.pid += 1;
        let mut fresh = stale.clone();
        fresh.start_ticks += 1;
        let observer = script(vec![error(libc::EACCES), Ok(vec![fresh.clone()])]);
        let (state, members) = decide_namespace(
            &original,
            std::slice::from_ref(&stale),
            None,
            true,
            None,
            &observer,
        )
        .unwrap();
        assert_eq!(state, "cancelling");
        assert_eq!(members, vec![fresh.clone()]);
        assert_eq!(*observer.signals.borrow(), vec![fresh]);
        assert_eq!(observer.scans.get(), 2);
        assert!(!observer.held.get());
        let observer = script(vec![
            error(libc::EACCES),
            error(libc::EACCES),
            error(libc::EACCES),
        ]);
        assert_eq!(
            decide_namespace(&original, &[], None, true, None, &observer)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(observer.scans.get(), 3);
        assert!(observer.signals.borrow().is_empty());
        assert!(!observer.held.get());
        let observer = script(vec![error(libc::EIO)]);
        assert!(decide_namespace(&original, &[], None, true, None, &observer).is_err());
        assert_eq!(observer.scans.get(), 1);
        assert!(observer.signals.borrow().is_empty());
        for boundary in ["frame", "pin", "missing-pin"] {
            let mut observer = script(vec![]);
            match boundary {
                "frame" => observer.frame_valid = false,
                "pin" => observer.pin_valid = false,
                _ => observer.process_alive = false,
            }
            assert!(decide_namespace(&original, &[], None, true, None, &observer).is_err());
            assert_eq!(observer.scans.get(), 0);
            assert!(observer.signals.borrow().is_empty());
        }
        let mut observer = script(vec![error(libc::EACCES)]);
        observer.delay = Duration::from_millis(30);
        assert!(decide_namespace(&original, &[], None, true, None, &observer).is_err());
        assert_eq!(observer.scans.get(), 1);
        assert!(observer.signals.borrow().is_empty());
        let mut observer = script(vec![]);
        observer.finalized = true;
        assert_eq!(
            decide_namespace(&original, &[], None, true, None, &observer).unwrap(),
            ("stopped", vec![])
        );
        assert_eq!(observer.scans.get(), 0);
    }
    #[test]
    fn membership_errors_retain_structured_provenance_and_disappearance_semantics() {
        let error = membership_error(
            123,
            "ns/pid",
            std::io::Error::from_raw_os_error(libc::EACCES),
        );
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        let provenance = error
            .get_ref()
            .unwrap()
            .downcast_ref::<MembershipObservationError>()
            .unwrap();
        assert_eq!(provenance.pid, 123);
        assert_eq!(provenance.operation, "ns/pid");
        assert_eq!(provenance.source.raw_os_error(), Some(libc::EACCES));
        for errno in [libc::ENOENT, libc::ESRCH] {
            let error = membership_error(123, "stat", std::io::Error::from_raw_os_error(errno));
            assert!(process_disappeared(&error));
            assert_eq!(error.raw_os_error(), Some(errno));
        }
    }
    #[test]
    fn incarnation_replacement_between_proof_and_inventory_never_signals_new_members() {
        use std::cell::Cell;
        struct Replacement {
            newer: ProcessIdentity,
            replaced: Cell<bool>,
            signals: Cell<usize>,
        }
        impl NamespaceObserver for Replacement {
            type Pin = ();
            fn frame(&self, _: Option<&ProcessIdentity>) -> std::io::Result<()> {
                Ok(())
            }
            fn alive(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(true)
            }
            fn pin(&self, _: &ProcessIdentity) -> std::io::Result<()> {
                // Original init and sole monitor pin disappear before acquisition.
                self.replaced.set(true);
                Err(std::io::Error::other(
                    "original namespace incarnation replaced",
                ))
            }
            fn finalized(&self, _: &ProcessIdentity) -> std::io::Result<bool> {
                Ok(false)
            }
            fn inventory(&self, _: &str) -> std::io::Result<Vec<ProcessIdentity>> {
                self.replaced.set(true);
                Ok(vec![self.newer.clone()])
            }
            fn signal(&self, _: &ProcessIdentity) -> std::io::Result<()> {
                assert!(self.replaced.get());
                self.signals.set(self.signals.get() + 1);
                Ok(())
            }
        }
        let old = identity(std::process::id()).unwrap();
        let mut newer = old.clone();
        newer.pid += 1;
        newer.start_ticks += 1;
        let observer = Replacement {
            newer,
            replaced: Cell::new(false),
            signals: Cell::new(0),
        };
        let decision = decide_namespace(
            &old,
            std::slice::from_ref(&old),
            None,
            true,
            None,
            &observer,
        );
        assert_eq!(
            observer.signals.get(),
            0,
            "replacement must receive no signal"
        );
        assert!(
            decision.is_err(),
            "unproven original incarnation must fail closed"
        );
    }
    #[test]
    fn foreign_or_unproven_observer_frames_cannot_classify_original_pids() {
        let controller = identity(std::process::id()).unwrap();
        let stat = fs::read_to_string("/proc/self/stat").unwrap();
        let check = |stat: &str, ns: &str, boot: &str, procfs| {
            validate_observer_frame(Some(&controller), stat, ns, boot, procfs)
        };
        assert!(check(&stat, &controller.namespace, &controller.boot_id, true).is_ok());
        assert!(check(&stat, "pid:[different]", &controller.boot_id, true).is_err());
        assert!(
            check(
                "1 (foreign) S",
                &controller.namespace,
                &controller.boot_id,
                true
            )
            .is_err()
        );
        assert!(check(&stat, &controller.namespace, "different-boot", true).is_err());
        assert!(check(&stat, &controller.namespace, &controller.boot_id, false).is_err());
        assert!(
            validate_observer_frame(
                None,
                &stat,
                &controller.namespace,
                &controller.boot_id,
                true
            )
            .is_err()
        );
    }
    #[test]
    fn only_certified_original_whole_group_finalization_proves_exit() {
        let mut init = identity(std::process::id()).unwrap();
        init.namespace_pid = Some(1);
        let stat = fs::read_to_string(format!("/proc/{}/stat", init.pid)).unwrap();
        let boot = || Ok(init.boot_id.clone());
        for state in ["S", "D", "Z"] {
            let (head, tail) = stat.rsplit_once(')').unwrap();
            let mut fields: Vec<_> = tail.split_whitespace().map(str::to_owned).collect();
            fields[0] = state.into();
            let observed = format!("{head}) {}", fields.join(" "));
            assert!(
                !observe_finalized(&init, boot(), |_| Ok(observed.clone()), |_| Ok(false)).unwrap()
            );
        }
        for errno in [libc::EACCES, libc::EIO] {
            assert!(
                observe_finalized(
                    &init,
                    boot(),
                    |_| Err(std::io::Error::from_raw_os_error(errno)),
                    |_| unreachable!()
                )
                .is_err()
            );
            assert!(
                observe_finalized(
                    &init,
                    boot(),
                    |_| Ok(stat.clone()),
                    |_| Err(std::io::Error::from_raw_os_error(errno))
                )
                .is_err()
            );
        }
        assert!(
            observe_finalized(
                &init,
                boot(),
                |_| Ok("malformed".into()),
                |_| unreachable!()
            )
            .is_err()
        );
        assert!(
            observe_finalized(
                &init,
                Ok("other-boot".into()),
                |_| unreachable!(),
                |_| unreachable!()
            )
            .is_err()
        );
        assert!(
            observe_finalized(
                &init,
                boot(),
                |_| Err(std::io::Error::from_raw_os_error(libc::ENOENT)),
                |_| unreachable!()
            )
            .unwrap()
        );
        assert!(observe_finalized(&init, boot(), |_| Ok(stat.clone()), |_| Ok(true)).unwrap());
        let (head, tail) = stat.rsplit_once(')').unwrap();
        let mut fields: Vec<_> = tail.split_whitespace().map(str::to_owned).collect();
        fields[19] = (init.start_ticks + 1).to_string();
        let replacement = format!("{head}) {}", fields.join(" "));
        assert!(
            observe_finalized(
                &init,
                boot(),
                |_| Ok(replacement.clone()),
                |_| unreachable!()
            )
            .unwrap()
        );
        let mut missing_certificate = init.clone();
        missing_certificate.namespace_pid = None;
        assert!(
            !observe_finalized(
                &missing_certificate,
                boot(),
                |_| unreachable!(),
                |_| unreachable!()
            )
            .unwrap()
        );
    }
    #[test]
    fn observation_uncertainty_is_never_proof_of_absence() {
        let p = identity(std::process::id()).unwrap();
        for errno in [libc::EACCES, libc::EIO] {
            assert!(
                observe_alive(
                    &p,
                    |_| Err(std::io::Error::from_raw_os_error(errno)),
                    |_| unreachable!()
                )
                .is_err()
            );
            assert!(
                observe_alive(
                    &p,
                    |_| Ok(p.clone()),
                    |_| Err(std::io::Error::from_raw_os_error(errno))
                )
                .is_err()
            );
        }
        assert!(observe_alive(&p, |_| Ok(p.clone()), |_| Ok("malformed".into())).is_err());
        for errno in [libc::ENOENT, libc::ESRCH] {
            assert!(
                !observe_alive(
                    &p,
                    |_| Err(std::io::Error::from_raw_os_error(errno)),
                    |_| unreachable!()
                )
                .unwrap()
            );
            assert!(
                !observe_alive(
                    &p,
                    |_| Ok(p.clone()),
                    |_| Err(std::io::Error::from_raw_os_error(errno))
                )
                .unwrap()
            );
        }
        assert!(!observe_alive(&p, |_| Ok(p.clone()), |_| Ok("1 (zombie) Z".into())).unwrap());
        let mut replacement = p.clone();
        replacement.start_ticks += 1;
        assert!(!observe_alive(&p, |_| Ok(replacement), |_| unreachable!()).unwrap());
    }
}
