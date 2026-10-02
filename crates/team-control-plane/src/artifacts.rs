//! Immutable local artifact publication and exact candidate acceptance.
use crate::{Broker, ErrorCode, Result, domain::err};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn io_error(_: std::io::Error) -> crate::Error {
    err(ErrorCode::TemporarilyUnavailable)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub digest: String,
    pub bytes: u64,
    pub version: u32,
}
#[derive(Clone, Copy, Debug, Default)]
pub enum PublishFault {
    #[default]
    None,
    BeforeFinalize,
}
#[derive(Clone, Debug)]
pub struct ArtifactStore {
    root: PathBuf,
}
impl ArtifactStore {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root).map_err(io_error)?;
        let root = root.canonicalize().map_err(io_error)?;
        for dir in ["objects", "manifests", "staging"] {
            fs::create_dir_all(root.join(dir)).map_err(io_error)?;
            File::open(root.join(dir))
                .and_then(|f| f.sync_all())
                .map_err(io_error)?;
        }
        File::open(&root)
            .and_then(|f| f.sync_all())
            .map_err(io_error)?;
        if let Some(parent) = root.parent() {
            File::open(parent)
                .and_then(|f| f.sync_all())
                .map_err(io_error)?;
        }
        Ok(Self { root })
    }
    fn path(&self, area: &str, digest: &str) -> Result<PathBuf> {
        if !valid_digest(digest) {
            return Err(err(ErrorCode::InvalidArgument));
        }
        Ok(self.root.join(area).join(digest))
    }
    fn regular_read(path: &Path) -> Result<Vec<u8>> {
        if !fs::symlink_metadata(path)
            .map_err(io_error)?
            .file_type()
            .is_file()
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        fs::read(path).map_err(io_error)
    }
    fn finalize(&self, area: &str, digest: &str, bytes: &[u8]) -> Result<()> {
        let target = self.path(area, digest)?;
        if target.exists() {
            if Self::regular_read(&target)? != bytes {
                return Err(err(ErrorCode::InvalidArgument));
            }
            // A prior interrupted publisher may have linked but not synced its directory.
            File::open(&target)
                .and_then(|f| f.sync_all())
                .map_err(io_error)?;
            File::open(self.root.join(area))
                .and_then(|f| f.sync_all())
                .map_err(io_error)?;
            return Ok(());
        }
        let staging = self.root.join("staging").join(Uuid::new_v4().to_string());
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(io_error)?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(io_error)?;
        // Hard-link is atomic and refuses replacement, unlike rename.
        match fs::hard_link(&staging, &target) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if Self::regular_read(&target)? != bytes {
                    return Err(err(ErrorCode::InvalidArgument));
                }
            }
            Err(e) => return Err(io_error(e)),
        }
        File::open(self.root.join(area))
            .and_then(|f| f.sync_all())
            .map_err(io_error)?;
        fs::remove_file(staging).map_err(io_error)?;
        File::open(self.root.join("staging"))
            .and_then(|f| f.sync_all())
            .map_err(io_error)?;
        Ok(())
    }
    pub fn publish(&self, bytes: &[u8], digest: &str, fault: PublishFault) -> Result<Manifest> {
        if !valid_digest(digest) || sha256(bytes) != digest {
            return Err(err(ErrorCode::InvalidArgument));
        }
        self.finalize("objects", digest, bytes)?;
        if matches!(fault, PublishFault::BeforeFinalize) {
            return Err(err(ErrorCode::TemporarilyUnavailable));
        }
        let manifest = Manifest {
            digest: digest.into(),
            bytes: bytes.len() as u64,
            version: 1,
        };
        let manifest_bytes =
            serde_json::to_vec(&manifest).map_err(|_| err(ErrorCode::InvalidArgument))?;
        self.finalize("manifests", digest, &manifest_bytes)?;
        self.read(digest)?;
        Ok(manifest)
    }
    pub fn read(&self, digest: &str) -> Result<Vec<u8>> {
        let manifest: Manifest =
            serde_json::from_slice(&Self::regular_read(&self.path("manifests", digest)?)?)
                .map_err(|_| err(ErrorCode::InvalidArgument))?;
        let bytes = Self::regular_read(&self.path("objects", digest)?)?;
        if manifest.version != 1
            || manifest.digest != digest
            || manifest.bytes != bytes.len() as u64
            || sha256(&bytes) != digest
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub commit: String,
    pub tree: String,
    pub base: String,
    pub spec_digest: String,
    pub spec_revision: i64,
    pub generation: i64,
    pub artifact: String,
    pub deliverables: Vec<String>,
    pub risks: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub command: Vec<String>,
    pub exit_code: i32,
    pub logs: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub candidate: Candidate,
    pub suite: String,
    pub environment: String,
    pub checks: Vec<Check>,
    pub resolved_decisions: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArtifactCommand {
    Configure {
        verifier: Uuid,
        lead_acceptance: bool,
        base: String,
        suite: String,
        environment: String,
        mandatory_commands: Vec<Vec<String>>,
    },
    Submit {
        candidate: Candidate,
    },
    Verify {
        verification: Verification,
    },
    Accept {
        candidate_id: String,
        report_id: String,
    },
    Repair {
        candidate_id: String,
        report_id: String,
        reason: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactEnvelope {
    pub key: String,
    pub task: Uuid,
    pub expected_revision: i64,
    pub generation: i64,
    pub recovery_epoch: Uuid,
    pub command: ArtifactCommand,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactResult {
    pub revision: i64,
    pub candidate_id: Option<String>,
    pub report_id: Option<String>,
    pub accepted: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArtifactSnapshot {
    pub revision: i64,
    pub status: String,
    pub candidate: Option<Candidate>,
    pub candidate_id: Option<String>,
    pub reports: Vec<Verification>,
}
#[derive(Clone, Copy, Debug, Default)]
pub enum ReferenceFault {
    #[default]
    None,
    BeforeReference,
}
#[derive(Clone, Debug)]
pub struct ArtifactService {
    broker: Broker,
    store: ArtifactStore,
}
/// Install the additive artifact/credential schema after core migration.
/// Repeated application checks its independent version marker under the same lock.
pub async fn migrate(dsn: &str) -> Result<()> {
    let mut client = crate::storage::client(dsn).await?;
    let tx = client.transaction().await?;
    crate::storage::lock(&tx).await?;
    if tx
        .query_one(
            "SELECT to_regclass('artifact_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0)
    {
        let rows = tx
            .query("SELECT version FROM artifact_schema_version", &[])
            .await?;
        if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
            return Err(err(ErrorCode::IncompatibleSchema));
        }
    } else {
        tx.batch_execute(include_str!("../migrations/0002.sql"))
            .await?;
    }
    tx.commit().await?;
    Ok(())
}
impl ArtifactService {
    pub async fn open(broker: Broker, root: &Path) -> Result<Self> {
        migrate(&broker.dsn).await?;
        Ok(Self {
            broker,
            store: ArtifactStore::open(root)?,
        })
    }
    pub fn store(&self) -> &ArtifactStore {
        &self.store
    }
    /// Publish immutable internal evidence bytes under current task authority.
    /// This does not submit, verify, accept, or externally publish the task.
    pub async fn publish_evidence(
        &self,
        bearer: &str,
        task_id: Uuid,
        digest: &str,
        bytes: &[u8],
    ) -> Result<Manifest> {
        if bytes.len() > 1024 * 1024 || !valid_digest(digest) {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut client = crate::storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        crate::storage::lock(&tx).await?;
        artifact_schema(&tx).await?;
        crate::storage::active(&tx, self.broker.epoch).await?;
        let actor = crate::auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        crate::storage::validate_actor_generation(&tx, &actor).await?;
        let task = load_task(&tx, task_id).await?;
        actor.scope(task.mission, Some(task_id))?;
        let manifest = self.store.publish(bytes, digest, PublishFault::None)?;
        tx.commit().await?;
        Ok(manifest)
    }
    /// Addressing grants only the explicitly referenced immutable evidence.
    /// This grants the exact sender-authorized reference, never peer task/history scope.
    pub async fn read_mailbox_evidence(
        &self,
        bearer: &str,
        message: Uuid,
        digest: &str,
    ) -> Result<Vec<u8>> {
        let mut client = crate::storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        crate::storage::lock(&tx).await?;
        artifact_schema(&tx).await?;
        crate::storage::active(&tx, self.broker.epoch).await?;
        let actor = crate::auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        crate::storage::validate_actor_generation(&tx, &actor).await?;
        crate::mailbox::shared_artifact(&tx, &actor, message, digest).await?;
        let bytes = self.store.read(digest)?;
        tx.commit().await?;
        Ok(bytes)
    }
    /// Scoped evidence access; publishing an object does not grant a task permission.
    pub async fn read_evidence(
        &self,
        bearer: &str,
        task_id: Uuid,
        digest: &str,
    ) -> Result<Vec<u8>> {
        if !valid_digest(digest) {
            return Err(err(ErrorCode::InvalidArgument));
        }
        let mut client = crate::storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        crate::storage::lock(&tx).await?;
        artifact_schema(&tx).await?;
        crate::storage::active(&tx, self.broker.epoch).await?;
        let actor = crate::auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        crate::storage::validate_actor_generation(&tx, &actor).await?;
        let task = load_task(&tx, task_id).await?;
        actor.scope(task.mission, Some(task_id))?;
        let exists=tx.query_one("SELECT EXISTS(SELECT 1 FROM artifact_candidates c WHERE c.task_id=$1 AND c.artifact=$2 UNION ALL SELECT 1 FROM artifact_report_logs l JOIN artifact_reports r ON l.report_id=r.id JOIN artifact_candidates c ON r.candidate_id=c.id WHERE c.task_id=$1 AND l.digest=$2)",&[&task_id,&digest]).await?.get::<_,bool>(0);
        if !exists {
            return Err(err(ErrorCode::Unauthorized));
        }
        let bytes = self.store.read(digest)?;
        tx.commit().await?;
        Ok(bytes)
    }
    pub async fn execute(
        &self,
        bearer: &str,
        envelope: ArtifactEnvelope,
        fault: ReferenceFault,
    ) -> Result<ArtifactResult> {
        if envelope.key.is_empty() || envelope.key.len() > 256 {
            return Err(err(ErrorCode::InvalidArgument));
        }
        if envelope.recovery_epoch != self.broker.epoch {
            return Err(err(ErrorCode::Unauthorized));
        }
        let mut client = crate::storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        crate::storage::lock(&tx).await?;
        artifact_schema(&tx).await?;
        crate::storage::active(&tx, self.broker.epoch).await?;
        let actor = crate::auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        crate::storage::validate_actor_generation(&tx, &actor).await?;
        let task = load_task(&tx, envelope.task).await?;
        actor.scope(task.mission, Some(envelope.task))?;
        authorize(&tx, &actor, &envelope.command, &task, envelope.task).await?;
        crate::kernel::task_dispatch_enabled(&tx, envelope.task).await?;
        // An authenticated Owner may inspect its immutable Accept receipt;
        // cached Worker/Verifier work must not revive a terminal Task.
        if task.status == "accepted" && !matches!(envelope.command, ArtifactCommand::Accept { .. })
        {
            return Err(err(ErrorCode::StaleRevision));
        }
        let family = envelope.command.family();
        // Bind scope and concurrency preconditions, not only command payload.
        let digest = crate::validation::digest(&envelope)?;
        if let Some(row)=tx.query_opt("SELECT argument_digest, disposition, result, scope_kind, scope_mission, scope_task FROM commands WHERE principal_id=$1 AND family=$2 AND key=$3",&[&actor.principal,&family,&envelope.key]).await? {
   crate::auth::RecordScope::from_columns(row.get("scope_kind"),row.get("scope_mission"),row.get("scope_task"))?.authorize(&actor)?;
   if row.get::<_,String>(0)!=digest {return Err(err(ErrorCode::IdempotencyConflict));}
   let value:serde_json::Value=row.get(2);
   if row.get::<_,String>(1)=="rejected" {return Err(serde_json::from_value(value).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?);}
   let result:ArtifactResult=serde_json::from_value(value).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?;
   if let ArtifactCommand::Repair{candidate_id,report_id,..}=&envelope.command {
       crate::kernel::schema(&tx).await?;
       if envelope.generation!=task.generation {return Err(err(ErrorCode::StaleGeneration));}
       if task.revision!=result.revision || task.status!="repair_required" {return Err(err(ErrorCode::StaleRevision));}
       self.repair_evidence(&tx,&task,envelope.task,candidate_id,report_id).await?;
   }
   if result.accepted { self.verify_evidence(&tx,result.candidate_id.as_deref().ok_or_else(||err(ErrorCode::InvalidArgument))?,result.report_id.as_deref().ok_or_else(||err(ErrorCode::InvalidArgument))?).await?; }
   tx.commit().await?; return Ok(result);
  }
        tx.batch_execute("SAVEPOINT artifact_mutation").await?;
        let outcome = if envelope.expected_revision != task.revision {
            Err(err(ErrorCode::StaleRevision))
        } else if envelope.generation != task.generation {
            Err(err(ErrorCode::StaleGeneration))
        } else {
            self.apply(&tx, &actor, &envelope, &task, fault).await
        };
        let (disposition, value, result) = match outcome {
            Ok(result) => (
                "applied",
                serde_json::to_value(&result).map_err(|_| err(ErrorCode::InvalidArgument))?,
                Ok(result),
            ),
            Err(error) => {
                if matches!(
                    error.code,
                    ErrorCode::Unauthorized
                        | ErrorCode::TemporarilyUnavailable
                        | ErrorCode::IncompatibleSchema
                ) {
                    return Err(error);
                }
                tx.batch_execute("ROLLBACK TO SAVEPOINT artifact_mutation")
                    .await?;
                (
                    "rejected",
                    serde_json::to_value(&error).map_err(|_| err(ErrorCode::InvalidArgument))?,
                    Err(error),
                )
            }
        };
        tx.execute("INSERT INTO commands(principal_id,family,key,argument_digest,aggregate_id,scope_kind,scope_mission,scope_task,disposition,result) VALUES($1,$2,$3,$4,$5,'task',$6,$5,$7,$8)",&[&actor.principal,&family,&envelope.key,&digest,&envelope.task,&task.mission,&disposition,&value]).await?;
        tx.commit().await?;
        result
    }
    async fn reference(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        digest: &str,
        fault: ReferenceFault,
    ) -> Result<()> {
        let bytes = self.store.read(digest)?;
        if matches!(fault, ReferenceFault::BeforeReference) {
            return Err(err(ErrorCode::TemporarilyUnavailable));
        }
        let manifest = Manifest {
            digest: digest.into(),
            bytes: bytes.len() as u64,
            version: 1,
        };
        let value = serde_json::to_value(manifest).map_err(|_| err(ErrorCode::InvalidArgument))?;
        tx.execute(
            "INSERT INTO artifact_objects(digest,manifest) VALUES($1,$2) ON CONFLICT DO NOTHING",
            &[&digest, &value],
        )
        .await?;
        Ok(())
    }
    async fn repair_evidence(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        task: &TaskBinding,
        id: Uuid,
        candidate_id: &str,
        report_id: &str,
    ) -> Result<()> {
        let policy = load_policy(tx, id).await?;
        let (current_id, current_candidate, implementer) = current(tx, id).await?;
        if current_id != candidate_id {
            return Err(err(ErrorCode::StaleRevision));
        }
        let (candidate, report, verifier) =
            self.verify_evidence(tx, candidate_id, report_id).await?;
        if candidate != current_candidate || verifier != policy.verifier || verifier == implementer
        {
            return Err(err(ErrorCode::StaleRevision));
        }
        validate_candidate(&candidate, task, &policy)?;
        validate_report(&report, &policy)?;
        if report.resolved_decisions
            && candidate.risks.is_empty()
            && report.checks.iter().all(|c| c.exit_code == 0)
        {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        Ok(())
    }
    async fn verify_evidence(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        candidate_id: &str,
        report_id: &str,
    ) -> Result<(Candidate, Verification, Uuid)> {
        let row=tx.query_opt("SELECT c.candidate,r.verification,r.verifier,c.task_id FROM artifact_candidates c JOIN artifact_reports r ON r.candidate_id=c.id WHERE c.id=$1 AND r.id=$2",&[&candidate_id,&report_id]).await?.ok_or_else(||err(ErrorCode::NotFound))?;
        let candidate: Candidate = decode(row.get(0))?;
        let report: Verification = decode(row.get(1))?;
        if candidate != report.candidate {
            return Err(err(ErrorCode::StaleRevision));
        }
        self.store.read(&candidate.artifact)?;
        for check in &report.checks {
            self.store.read(&check.logs)?;
        }
        self.verify_execution_report(tx, row.get(3), row.get(2), &report)
            .await?;
        Ok((candidate, report, row.get(2)))
    }
    /// Legacy unassembled tasks retain ticket11's reporting contract. An
    /// irreversible assembled policy requires independent monitor observations.
    async fn verify_execution_report(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        task: Uuid,
        verifier: Uuid,
        report: &Verification,
    ) -> Result<()> {
        crate::kernel::schema(tx).await?;
        let Some(requirement) = tx
            .query_opt(
                "SELECT recovery_epoch FROM kernel_verification_requirements WHERE task_id=$1",
                &[&task],
            )
            .await?
        else {
            return Ok(());
        };
        if requirement.get::<_, Uuid>(0) != self.broker.epoch {
            return Err(err(ErrorCode::DependencyUnmet));
        }
        for check in &report.checks {
            let bytes = self.store.read(&check.logs)?;
            let evidence: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| err(ErrorCode::DependencyUnmet))?;
            let claimed = &evidence["trusted_monitor_result"];
            let execution: Uuid = serde_json::from_value(claimed["execution"].clone())
                .map_err(|_| err(ErrorCode::DependencyUnmet))?;
            let row=tx.query_opt("SELECT r.result,w.manifest FROM kernel_execution_results r JOIN supervised_workspaces w ON w.id=r.execution_id JOIN episode_executions x ON x.execution_id=w.id JOIN mailbox_episodes e ON e.id=x.episode_id JOIN credentials c ON c.id=x.credential_id JOIN tasks t ON t.id=x.task_id WHERE r.execution_id=$1 AND x.task_id=$2 AND x.principal_id=$3 AND x.role='verifier' AND x.recovery_epoch=$4 AND w.state IN ('stopped','deleting','released') AND e.task_generation=t.generation AND c.principal_id=x.principal_id AND c.role=x.role AND c.mission_id=x.mission_id AND c.task_id=x.task_id AND c.generation=e.task_generation AND c.recovery_epoch=$4 AND c.expires_at>clock_timestamp()", &[&execution,&task,&verifier,&self.broker.epoch]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
            let stored: serde_json::Value = row.get(0);
            if claimed != &stored {
                return Err(err(ErrorCode::DependencyUnmet));
            }
            let result: crate::supervisor::ExecutionResult = decode(stored)?;
            let manifest: crate::supervisor::WorkspaceManifest = decode(row.get(1))?;
            let output = evidence["output"]
                .as_str()
                .ok_or_else(|| err(ErrorCode::DependencyUnmet))?;
            if result.execution != execution
                || result.recovery_epoch != self.broker.epoch
                || result.candidate.as_ref() != Some(&report.candidate)
                || manifest.candidate.as_ref() != Some(&report.candidate)
                || result.argv != check.command
                || result.exit_code != Some(check.exit_code)
                || result.signal.is_some()
                || !result.namespace_stopped
                || result.logs != sha256(output.as_bytes())
            {
                return Err(err(ErrorCode::DependencyUnmet));
            }
        }
        Ok(())
    }
    async fn apply(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        actor: &crate::auth::Actor,
        envelope: &ArtifactEnvelope,
        task: &TaskBinding,
        fault: ReferenceFault,
    ) -> Result<ArtifactResult> {
        if task.status == "accepted" {
            return Err(err(ErrorCode::StaleRevision));
        }
        let mut result = ArtifactResult {
            revision: task.revision + 1,
            candidate_id: None,
            report_id: None,
            accepted: false,
        };
        let status = match &envelope.command {
            ArtifactCommand::Configure {
                verifier,
                lead_acceptance,
                base,
                suite,
                environment,
                mandatory_commands,
            } => {
                if !git_id(base)
                    || !valid_digest(suite)
                    || !valid_digest(environment)
                    || mandatory_commands.is_empty()
                    || mandatory_commands
                        .iter()
                        .any(|v| v.is_empty() || v.iter().any(|s| s.is_empty() || s.contains('\0')))
                {
                    return Err(err(ErrorCode::InvalidArgument));
                }
                let commands = serde_json::to_value(mandatory_commands)
                    .map_err(|_| err(ErrorCode::InvalidArgument))?;
                tx.execute("INSERT INTO artifact_policies(task_id,verifier,lead_acceptance,base,suite,environment,mandatory_commands) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(task_id) DO UPDATE SET verifier=$2,lead_acceptance=$3,base=$4,suite=$5,environment=$6,mandatory_commands=$7",&[&envelope.task,verifier,lead_acceptance,base,suite,environment,&commands]).await?;
                task.status.as_str()
            }
            ArtifactCommand::Submit { candidate } => {
                if !["running", "submitted", "verifying", "repair_required"]
                    .contains(&task.status.as_str())
                {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                let policy = load_policy(tx, envelope.task).await?;
                validate_candidate(candidate, task, &policy)?;
                let admitted=tx.query_opt("SELECT sequence FROM events WHERE aggregate_kind='task' AND aggregate_id=$1 AND kind='task.admit' AND payload->>'spec_digest'=$2 ORDER BY sequence DESC LIMIT 1",&[&envelope.task,&task.spec]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
                if candidate.spec_revision != admitted.get::<_, i64>(0) {
                    return Err(err(ErrorCode::StaleRevision));
                }
                let spec = tx
                    .query_one(
                        "SELECT deliverables FROM task_specs WHERE task_id=$1 AND digest=$2",
                        &[&envelope.task, &task.spec],
                    )
                    .await?;
                let required: Vec<String> = decode(spec.get(0))?;
                if required.iter().any(|d| !candidate.deliverables.contains(d)) {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                self.reference(tx, &candidate.artifact, fault).await?;
                let id = crate::validation::digest(&(envelope.task, candidate))?;
                let value =
                    serde_json::to_value(candidate).map_err(|_| err(ErrorCode::InvalidArgument))?;
                tx.execute("INSERT INTO artifact_candidates(id,task_id,implementer,candidate,artifact) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",&[&id,&envelope.task,&actor.principal,&value,&candidate.artifact]).await?;
                tx.execute("INSERT INTO artifact_current(task_id,candidate_id) VALUES($1,$2) ON CONFLICT(task_id) DO UPDATE SET candidate_id=$2,accepted_report=NULL",&[&envelope.task,&id]).await?;
                result.candidate_id = Some(id);
                "submitted"
            }
            ArtifactCommand::Verify { verification } => {
                if !["submitted", "verifying"].contains(&task.status.as_str()) {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                let policy = load_policy(tx, envelope.task).await?;
                let (id, candidate, implementer) = current(tx, envelope.task).await?;
                if actor.principal == implementer || actor.principal != policy.verifier {
                    return Err(err(ErrorCode::Unauthorized));
                }
                validate_candidate(&verification.candidate, task, &policy)?;
                if verification.candidate != candidate {
                    return Err(err(ErrorCode::StaleRevision));
                }
                validate_report(verification, &policy)?;
                self.verify_execution_report(tx, envelope.task, actor.principal, verification)
                    .await?;
                self.store.read(&candidate.artifact)?;
                for check in &verification.checks {
                    self.reference(tx, &check.logs, fault).await?;
                }
                let report_id =
                    crate::validation::digest(&(envelope.task, &actor.principal, verification))?;
                let value = serde_json::to_value(verification)
                    .map_err(|_| err(ErrorCode::InvalidArgument))?;
                tx.execute("INSERT INTO artifact_reports(id,candidate_id,verifier,verification) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING",&[&report_id,&id,&actor.principal,&value]).await?;
                for check in &verification.checks {
                    tx.execute("INSERT INTO artifact_report_logs(report_id,digest) VALUES($1,$2) ON CONFLICT DO NOTHING",&[&report_id,&check.logs]).await?;
                }
                result.candidate_id = Some(id);
                result.report_id = Some(report_id);
                "verifying"
            }
            ArtifactCommand::Repair {
                candidate_id,
                report_id,
                reason,
            } => {
                crate::kernel::schema(tx).await?;
                if reason.is_empty() || reason.len() > 8192 {
                    return Err(err(ErrorCode::InvalidArgument));
                }
                self.repair_evidence(tx, task, envelope.task, candidate_id, report_id)
                    .await?;
                let exhausted:bool=tx.query_one("SELECT (SELECT count(*) FROM kernel_repair_charges WHERE mission_id=$1)>=max_repairs FROM missions WHERE id=$1", &[&task.mission]).await?.get(0);
                if exhausted {
                    return Err(err(ErrorCode::BudgetExhausted));
                }
                if task.status!="verifying" || tx.query_one("SELECT EXISTS(SELECT 1 FROM kernel_repair_charges WHERE candidate_id=$1 AND report_id=$2)", &[candidate_id,report_id]).await?.get::<_,bool>(0) {return Err(err(ErrorCode::DependencyUnmet));}
                tx.execute("INSERT INTO kernel_repair_charges(id,mission_id,task_id,candidate_id,report_id,principal_id,recovery_epoch,generation,reason) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)", &[&Uuid::new_v4(),&task.mission,&envelope.task,candidate_id,report_id,&actor.principal,&self.broker.epoch,&task.generation,reason]).await?;
                result.candidate_id = Some(candidate_id.clone());
                result.report_id = Some(report_id.clone());
                "repair_required"
            }
            ArtifactCommand::Accept {
                candidate_id,
                report_id,
            } => {
                if task.status != "verifying" {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                let policy = load_policy(tx, envelope.task).await?;
                let (id, current_candidate, implementer) = current(tx, envelope.task).await?;
                if id != *candidate_id {
                    return Err(err(ErrorCode::StaleRevision));
                }
                let (candidate, report, verifier) =
                    self.verify_evidence(tx, candidate_id, report_id).await?;
                if candidate != current_candidate
                    || verifier != policy.verifier
                    || verifier == implementer
                {
                    return Err(err(ErrorCode::StaleRevision));
                }
                validate_candidate(&candidate, task, &policy)?;
                validate_report(&report, &policy)?;
                if !report.resolved_decisions
                    || !candidate.risks.is_empty()
                    || report.checks.iter().any(|c| c.exit_code != 0)
                {
                    return Err(err(ErrorCode::DependencyUnmet));
                }
                tx.execute(
                    "UPDATE artifact_current SET accepted_report=$2 WHERE task_id=$1",
                    &[&envelope.task, report_id],
                )
                .await?;
                result.candidate_id = Some(id);
                result.report_id = Some(report_id.clone());
                result.accepted = true;
                "accepted"
            }
        };
        tx.execute(
            "UPDATE tasks SET revision=$2,status=$3 WHERE id=$1",
            &[&envelope.task, &result.revision, &status],
        )
        .await?;
        let payload =
            serde_json::json!({"result":result,"command":envelope.command,"spec_digest":task.spec});
        tx.execute("INSERT INTO events(id,aggregate_kind,aggregate_id,sequence,version,principal_id,transaction_id,kind,payload) VALUES($1,'task',$2,$3,1,$4,txid_current(),$5,$6)",&[&Uuid::new_v4(),&envelope.task,&result.revision,&actor.principal,&envelope.command.family(),&payload]).await?;
        Ok(result)
    }
    pub async fn snapshot(&self, bearer: &str, task_id: Uuid) -> Result<ArtifactSnapshot> {
        let mut client = crate::storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        crate::storage::lock(&tx).await?;
        crate::storage::active(&tx, self.broker.epoch).await?;
        let actor = crate::auth::authenticate(&tx, bearer, self.broker.epoch).await?;
        crate::storage::validate_actor_generation(&tx, &actor).await?;
        let task = load_task(&tx, task_id).await?;
        actor.scope(task.mission, Some(task_id))?;
        let row=tx.query_opt("SELECT c.id,c.candidate FROM artifact_current a JOIN artifact_candidates c ON a.candidate_id=c.id WHERE a.task_id=$1",&[&task_id]).await?;
        let (candidate_id, candidate) = match row {
            Some(r) => (Some(r.get(0)), Some(decode(r.get(1))?)),
            None => (None, None),
        };
        let rows=tx.query("SELECT r.verification FROM artifact_reports r JOIN artifact_candidates c ON r.candidate_id=c.id WHERE c.task_id=$1 ORDER BY r.id",&[&task_id]).await?;
        let reports = rows
            .into_iter()
            .map(|r| decode(r.get(0)))
            .collect::<Result<Vec<_>>>()?;
        tx.commit().await?;
        Ok(ArtifactSnapshot {
            revision: task.revision,
            status: task.status,
            candidate,
            candidate_id,
            reports,
        })
    }
    /// Owner-only collection. The same global database lock excludes new references.
    pub async fn collect(&self, bearer: &str, grace: std::time::Duration) -> Result<Vec<String>> {
        let mut client = crate::storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        crate::storage::lock(&tx).await?;
        crate::storage::active(&tx, self.broker.epoch).await?;
        crate::auth::authenticate(&tx, bearer, self.broker.epoch)
            .await?
            .owner()?;
        let mut removed = vec![];
        for area in ["objects", "manifests", "staging"] {
            for entry in fs::read_dir(self.store.root.join(area)).map_err(io_error)? {
                let entry = entry.map_err(io_error)?;
                let name = entry.file_name().to_string_lossy().into_owned();
                let meta = fs::symlink_metadata(entry.path()).map_err(io_error)?;
                if !meta.file_type().is_file() {
                    continue;
                }
                let age = meta
                    .modified()
                    .map_err(io_error)?
                    .elapsed()
                    .unwrap_or_default();
                if age < grace {
                    continue;
                }
                if area != "staging"
                    && tx
                        .query_opt(
                            "SELECT digest FROM artifact_objects WHERE digest=$1",
                            &[&name],
                        )
                        .await?
                        .is_some()
                {
                    continue;
                }
                fs::remove_file(entry.path()).map_err(io_error)?;
                removed.push(format!("{area}/{name}"));
            }
            File::open(self.store.root.join(area))
                .and_then(|f| f.sync_all())
                .map_err(io_error)?;
        }
        tx.commit().await?;
        Ok(removed)
    }
}
fn decode<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    serde_json::from_value(value).map_err(|_| err(ErrorCode::InvalidArgument))
}
fn git_id(value: &str) -> bool {
    (value.len() == 40 || value.len() == 64)
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
struct TaskBinding {
    mission: Uuid,
    revision: i64,
    generation: i64,
    spec: String,
    status: String,
}
async fn load_task(tx: &tokio_postgres::Transaction<'_>, id: Uuid) -> Result<TaskBinding> {
    let r = tx
        .query_opt(
            "SELECT mission_id,revision,generation,spec_digest,status FROM tasks WHERE id=$1",
            &[&id],
        )
        .await?
        .ok_or_else(|| err(ErrorCode::NotFound))?;
    Ok(TaskBinding {
        mission: r.get(0),
        revision: r.get(1),
        generation: r.get(2),
        spec: r.get(3),
        status: r.get(4),
    })
}
struct AcceptancePolicy {
    verifier: Uuid,
    lead: bool,
    base: String,
    suite: String,
    environment: String,
    commands: Vec<Vec<String>>,
}
async fn load_policy(tx: &tokio_postgres::Transaction<'_>, task: Uuid) -> Result<AcceptancePolicy> {
    let r=tx.query_opt("SELECT verifier,lead_acceptance,base,suite,environment,mandatory_commands FROM artifact_policies WHERE task_id=$1",&[&task]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
    Ok(AcceptancePolicy {
        verifier: r.get(0),
        lead: r.get(1),
        base: r.get(2),
        suite: r.get(3),
        environment: r.get(4),
        commands: decode(r.get(5))?,
    })
}
async fn authorize(
    tx: &tokio_postgres::Transaction<'_>,
    actor: &crate::auth::Actor,
    command: &ArtifactCommand,
    task: &TaskBinding,
    id: Uuid,
) -> Result<()> {
    match command {
        ArtifactCommand::Configure { .. } => actor.owner(),
        ArtifactCommand::Submit { .. } => {
            if actor.role != crate::Role::Worker || actor.generation != Some(task.generation) {
                return Err(err(ErrorCode::Unauthorized));
            }
            let r=tx.query_opt("SELECT worker_id FROM attempts WHERE task_id=$1 AND generation=$2 AND recovery_epoch=(SELECT active_epoch FROM deployment)",&[&id,&task.generation]).await?.ok_or_else(||err(ErrorCode::StaleGeneration))?;
            if r.get::<_, Uuid>(0) != actor.principal {
                return Err(err(ErrorCode::Unauthorized));
            }
            Ok(())
        }
        ArtifactCommand::Verify { .. } => {
            let policy = load_policy(tx, id).await?;
            if actor.role != crate::Role::Verifier || actor.principal != policy.verifier {
                return Err(err(ErrorCode::Unauthorized));
            }
            Ok(())
        }
        ArtifactCommand::Accept { .. } | ArtifactCommand::Repair { .. } => {
            if actor.role == crate::Role::Owner {
                return Ok(());
            }
            let policy = load_policy(tx, id).await?;
            if actor.role != crate::Role::Lead || !policy.lead || actor.principal == policy.verifier
            {
                return Err(err(ErrorCode::Unauthorized));
            }
            Ok(())
        }
    }
}
async fn current(
    tx: &tokio_postgres::Transaction<'_>,
    task: Uuid,
) -> Result<(String, Candidate, Uuid)> {
    let r=tx.query_opt("SELECT c.id,c.candidate,c.implementer FROM artifact_current a JOIN artifact_candidates c ON c.id=a.candidate_id WHERE a.task_id=$1",&[&task]).await?.ok_or_else(||err(ErrorCode::DependencyUnmet))?;
    Ok((r.get(0), decode(r.get(1))?, r.get(2)))
}
fn validate_candidate(
    candidate: &Candidate,
    task: &TaskBinding,
    policy: &AcceptancePolicy,
) -> Result<()> {
    if !git_id(&candidate.commit)
        || !git_id(&candidate.tree)
        || !git_id(&candidate.base)
        || !valid_digest(&candidate.artifact)
        || !valid_digest(&candidate.spec_digest)
        || candidate.spec_revision <= 0
        || candidate.deliverables.is_empty()
        || candidate.deliverables.iter().any(|s| s.is_empty())
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    if candidate.spec_digest != task.spec || candidate.base != policy.base {
        return Err(err(ErrorCode::StaleRevision));
    }
    if candidate.generation != task.generation {
        return Err(err(ErrorCode::StaleGeneration));
    }
    Ok(())
}
fn validate_report(report: &Verification, policy: &AcceptancePolicy) -> Result<()> {
    if report.suite != policy.suite || report.environment != policy.environment {
        return Err(err(ErrorCode::StaleRevision));
    }
    if report.checks.is_empty()
        || report
            .checks
            .iter()
            .any(|c| c.command.is_empty() || !valid_digest(&c.logs))
        || policy
            .commands
            .iter()
            .any(|c| !report.checks.iter().any(|check| &check.command == c))
    {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    Ok(())
}
impl ArtifactCommand {
    fn family(&self) -> &'static str {
        match self {
            Self::Configure { .. } => "artifact.configure",
            Self::Submit { .. } => "result.submit",
            Self::Verify { .. } => "verification.submit",
            Self::Accept { .. } => "task.accept",
            Self::Repair { .. } => "task.repair",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationCandidate {
    pub commit: String,
    pub tree: String,
    pub base: String,
    pub source: String,
    pub checks: Vec<IntegrationCheck>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationCheck {
    pub command: Vec<String>,
    pub exit_code: i32,
    pub logs: Vec<u8>,
}
/// Local fixture-only preparation; no push, ref update, or mainline merge.
pub fn prepare_integration(
    repo: &Path,
    source: &str,
    base: &str,
    commands: &[Vec<String>],
) -> Result<IntegrationCandidate> {
    if !git_id(source)
        || !git_id(base)
        || commands.is_empty()
        || commands.iter().any(|c| c.is_empty() || c[0].is_empty())
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    let tree = git_output(repo, &["merge-tree", "--write-tree", base, source])?;
    let tree = tree
        .lines()
        .next()
        .ok_or_else(|| err(ErrorCode::InvalidArgument))?
        .to_string();
    if !git_id(&tree) {
        return Err(err(ErrorCode::InvalidArgument));
    }
    let commit_output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args([
            "commit-tree",
            &tree,
            "-p",
            base,
            "-p",
            source,
            "-m",
            "Fixture integration candidate",
        ])
        .env("GIT_AUTHOR_NAME", "Artifact fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Artifact fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .output()
        .map_err(io_error)?;
    if !commit_output.status.success() {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    let commit = String::from_utf8(commit_output.stdout)
        .map_err(|_| err(ErrorCode::InvalidArgument))?
        .trim()
        .to_string();
    let path = std::env::temp_dir().join(format!("artifact-check-{}", Uuid::new_v4()));
    let path_str = path
        .to_str()
        .ok_or_else(|| err(ErrorCode::InvalidArgument))?;
    git_output(repo, &["worktree", "add", "--detach", path_str, &commit])?;
    let check_result = (|| {
        let mut checks = vec![];
        for command in commands {
            let output = std::process::Command::new(&command[0])
                .args(&command[1..])
                .current_dir(&path)
                .output()
                .map_err(io_error)?;
            let mut logs = output.stdout;
            logs.extend_from_slice(b"\n--- stderr ---\n");
            logs.extend_from_slice(&output.stderr);
            checks.push(IntegrationCheck {
                command: command.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                logs,
            });
        }
        Ok(IntegrationCandidate {
            commit,
            tree,
            base: base.into(),
            source: source.into(),
            checks,
        })
    })();
    let cleanup = git_output(repo, &["worktree", "remove", "--force", path_str]);
    cleanup?;
    check_result
}
fn git_output(repo: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(io_error)?;
    if !output.status.success() {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_| err(ErrorCode::InvalidArgument))
}

async fn artifact_schema(tx: &tokio_postgres::Transaction<'_>) -> Result<()> {
    if !tx
        .query_one(
            "SELECT to_regclass('artifact_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0)
    {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    let rows = tx
        .query("SELECT version FROM artifact_schema_version", &[])
        .await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 1 {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
