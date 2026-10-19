//! Exact admitted work and strategy, independent of mutable planning sources.
//! PostgreSQL serializes authorization, revision checking, replay and mutation.
use crate::{Broker, ErrorCode, Result, Role, auth, domain::err, storage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio_postgres::Transaction;
use uuid::Uuid;
mod authority;
mod dependencies;
pub(crate) use dependencies::fresh_grant;
mod outcomes;
mod strategy;
pub use authority::Delegation;
pub use strategy::{PlanRevision, Step};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkRef {
    pub ledger: String,
    pub deployment: String,
    pub tenant: String,
    pub project: String,
    pub item: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceSnapshot {
    pub adapter: String,
    pub locator: String,
    pub revision: String,
    pub availability: String,
    pub snapshot: Vec<u8>,
    pub digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub attempts: i64,
    pub episodes: i64,
    pub repairs: i64,
    pub execution_seconds: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Production,
    InvestigationComplete,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AcceptanceContract {
    pub outcome: Outcome,
    pub allow_negative: bool,
    pub criteria: Vec<String>,
    pub verifier: Uuid,
    pub policy: String,
    pub suite: String,
    pub environment: String,
    pub mandatory_commands: Vec<Vec<String>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub work: WorkRef,
    pub intent: String,
    pub accepted_subject: String,
    pub policy: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IntentRevision {
    pub objective: String,
    pub constraints: Vec<String>,
    pub allowed_effects: Vec<String>,
    pub resources: Resources,
    pub acceptance: AcceptanceContract,
    pub dependencies: Vec<Dependency>,
    pub legacy_spec_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Command {
    #[serde(rename = "dependency.revalidate")]
    RevalidateDependency {
        intent: String,
        dependencies: Vec<Dependency>,
    },
    #[serde(rename = "outcome.propose")]
    ProposeOutcome {
        intent: String,
        evidence: String,
        negative: bool,
        evaluator_change: bool,
    },
    #[serde(rename = "verification.bind")]
    BindVerification {
        episode: Uuid,
        intent: String,
        acceptance: String,
        policy: String,
        environment: String,
    },
    #[serde(rename = "production.verify")]
    VerifyProduction {
        binding: String,
        candidate: String,
        report: String,
    },
    #[serde(rename = "research.verify")]
    VerifyResearch {
        intent: String,
        acceptance: String,
        proposal: String,
        policy: String,
        environment: String,
    },
    #[serde(rename = "outcome.accept")]
    AcceptOutcome { proof: String },
    #[serde(rename = "delegation.admit")]
    AdmitDelegation { delegation: Delegation },
    #[serde(rename = "plan.admit")]
    AdmitPlan { plan: PlanRevision },
    #[serde(rename = "intent.admit")]
    AdmitIntent {
        work: WorkRef,
        source: SourceSnapshot,
        intent: IntentRevision,
    },
    #[serde(rename = "source.propose")]
    ProposeSource {
        work: WorkRef,
        source: SourceSnapshot,
    },
}
impl Command {
    fn family(&self) -> &'static str {
        match self {
            Self::RevalidateDependency { .. } => "dependency.revalidate",
            Self::ProposeOutcome { .. } => "outcome.propose",
            Self::BindVerification { .. } => "verification.bind",
            Self::VerifyProduction { .. } => "production.verify",
            Self::VerifyResearch { .. } => "research.verify",
            Self::AcceptOutcome { .. } => "outcome.accept",
            Self::AdmitDelegation { .. } => "delegation.admit",
            Self::AdmitPlan { .. } => "plan.admit",
            Self::AdmitIntent { .. } => "intent.admit",
            Self::ProposeSource { .. } => "source.propose",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub schema_version: u32,
    pub key: String,
    pub expected_revision: i64,
    pub recovery_epoch: Uuid,
    pub task: Uuid,
    pub command: Command,
}
pub(crate) async fn migrate(tx: &Transaction<'_>) -> Result<()> {
    if !tx
        .query_one(
            "SELECT to_regclass('contract_schema_version') IS NOT NULL",
            &[],
        )
        .await?
        .get::<_, bool>(0)
    {
        tx.batch_execute(include_str!("../migrations/0008.sql"))
            .await?;
    }
    schema(tx).await
}
pub(crate) async fn schema(tx: &Transaction<'_>) -> Result<()> {
    let rows = tx
        .query("SELECT version FROM contract_schema_version", &[])
        .await?;
    if rows.len() != 1 || rows[0].get::<_, i32>(0) != 2 {
        return Err(err(ErrorCode::IncompatibleSchema));
    }
    Ok(())
}
pub(crate) async fn legacy_acceptance(tx: &Transaction<'_>, task: Uuid) -> Result<()> {
    if tx
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM contract_current WHERE task_id=$1)",
            &[&task],
        )
        .await?
        .get::<_, bool>(0)
    {
        return Err(err(ErrorCode::DependencyUnmet));
    }
    Ok(())
}
#[derive(Clone)]
pub struct ContractService {
    broker: Broker,
}
impl ContractService {
    pub fn new(broker: Broker) -> Self {
        Self { broker }
    }
    pub async fn execute(
        &self,
        token: &str,
        envelope: Envelope,
        artifacts: Option<&crate::artifacts::ArtifactService>,
    ) -> Result<Value> {
        if envelope.schema_version != 2
            || envelope.key.is_empty()
            || envelope.key.len() > 256
            || envelope.expected_revision < 0
        {
            return Err(err(ErrorCode::InvalidArgument));
        }
        if envelope.recovery_epoch != self.broker.epoch {
            return Err(err(ErrorCode::Unauthorized));
        }
        let mut client = storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, token, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        let mission: Uuid = tx
            .query_opt(
                "SELECT mission_id FROM tasks WHERE id=$1",
                &[&envelope.task],
            )
            .await?
            .ok_or_else(|| err(ErrorCode::NotFound))?
            .get(0);
        actor.scope(mission, Some(envelope.task))?;
        authority::authorize(&tx, &actor, &envelope, mission).await?;
        outcomes::mutation_enabled(&tx, &envelope).await?;
        let family = envelope.command.family();
        let digest = crate::validation::digest(&envelope)?;
        if let Some(row)=tx.query_opt("SELECT argument_digest,disposition,result,task_id FROM contract_commands WHERE principal_id=$1 AND family=$2 AND key=$3",&[&actor.principal,&family,&envelope.key]).await? {
            if row.get::<_,String>(0)!=digest || row.get::<_,Uuid>(3)!=envelope.task { return Err(err(ErrorCode::IdempotencyConflict)); }
            let value:Value=row.get(2);
            if row.get::<_,String>(1)=="rejected" { return Err(serde_json::from_value(value).map_err(|_|err(ErrorCode::TemporarilyUnavailable))?); }
            if let Command::RevalidateDependency{intent,dependencies}=&envelope.command {
                if strategy::current(&tx,envelope.task).await?.0!=*intent {return Err(err(ErrorCode::StaleRevision));}
                dependencies::validate(&tx,envelope.task,dependencies).await?;
            }
            outcomes::replay(&tx,&envelope,&value,artifacts).await?;
            tx.commit().await?;
            return Ok(value);
        }
        tx.batch_execute("SAVEPOINT contract_mutation").await?;
        let revision: i64 = tx
            .query_opt(
                "SELECT revision FROM contract_current WHERE task_id=$1",
                &[&envelope.task],
            )
            .await?
            .map_or(0, |r| r.get(0));
        let outcome = if envelope.expected_revision != revision {
            Err(err(ErrorCode::StaleRevision))
        } else {
            self.apply(&tx, &actor, &envelope, mission, revision, artifacts)
                .await
        };
        let (disposition, value, result) = match outcome {
            Ok(value) => ("applied", value.clone(), Ok(value)),
            Err(error) => {
                if matches!(
                    error.code,
                    ErrorCode::Unauthorized
                        | ErrorCode::TemporarilyUnavailable
                        | ErrorCode::IncompatibleSchema
                ) {
                    return Err(error);
                }
                tx.batch_execute("ROLLBACK TO SAVEPOINT contract_mutation")
                    .await?;
                (
                    "rejected",
                    serde_json::to_value(&error).map_err(|_| err(ErrorCode::InvalidArgument))?,
                    Err(error),
                )
            }
        };
        tx.execute("INSERT INTO contract_commands(principal_id,family,key,argument_digest,task_id,disposition,result) VALUES($1,$2,$3,$4,$5,$6,$7)",&[&actor.principal,&family,&envelope.key,&digest,&envelope.task,&disposition,&value]).await?;
        tx.commit().await?;
        result
    }
    async fn apply(
        &self,
        tx: &Transaction<'_>,
        actor: &auth::Actor,
        envelope: &Envelope,
        mission: Uuid,
        revision: i64,
        artifacts: Option<&crate::artifacts::ArtifactService>,
    ) -> Result<Value> {
        match &envelope.command {
            Command::RevalidateDependency {
                intent,
                dependencies,
            } => {
                dependencies::revalidate(tx, actor, envelope, revision, intent, dependencies).await
            }
            Command::ProposeOutcome { .. }
            | Command::BindVerification { .. }
            | Command::VerifyProduction { .. }
            | Command::VerifyResearch { .. }
            | Command::AcceptOutcome { .. } => {
                outcomes::apply(tx, actor, envelope, revision, artifacts).await
            }
            Command::AdmitDelegation { delegation } => {
                authority::admit(tx, actor, envelope, mission, revision, delegation).await
            }
            Command::AdmitPlan { plan } => {
                strategy::admit(tx, actor, envelope, revision, plan).await
            }
            Command::AdmitIntent {
                work,
                source,
                intent,
            } => {
                validate_work(work)?;
                validate_source(source)?;
                validate_intent(intent)?;
                if source.adapter != work.ledger {
                    return Err(err(ErrorCode::InvalidArgument));
                }
                let legacy = tx
                    .query_one(
                        "SELECT spec_digest FROM tasks WHERE id=$1",
                        &[&envelope.task],
                    )
                    .await?
                    .get::<_, String>(0);
                if intent.legacy_spec_digest != legacy {
                    return Err(err(ErrorCode::StaleRevision));
                }
                let policy=tx.query_one("SELECT max_attempts,max_episodes,max_repairs,deadline_seconds FROM missions WHERE id=$1",&[&mission]).await?;
                if intent.resources.attempts > policy.get::<_, i64>(0)
                    || intent.resources.episodes > policy.get::<_, i64>(1)
                    || intent.resources.repairs > policy.get::<_, i64>(2)
                    || intent.resources.execution_seconds > policy.get::<_, i64>(3)
                {
                    return Err(err(ErrorCode::BudgetExhausted));
                }
                dependencies::validate(tx, envelope.task, &intent.dependencies).await?;
                authority::check_limit(tx, actor, envelope, true).await?;
                let work_id = crate::validation::digest(work)?;
                if let Some(row) = tx
                    .query_opt(
                        "SELECT id FROM contract_work WHERE task_id=$1",
                        &[&envelope.task],
                    )
                    .await?
                {
                    if row.get::<_, String>(0) != work_id {
                        return Err(err(ErrorCode::IdempotencyConflict));
                    }
                }
                if let Some(row) = tx
                    .query_opt("SELECT task_id FROM contract_work WHERE id=$1", &[&work_id])
                    .await?
                {
                    if row.get::<_, Uuid>(0) != envelope.task {
                        return Err(err(ErrorCode::IdempotencyConflict));
                    }
                }
                tx.execute("INSERT INTO contract_work(id,task_id,work,mission_id) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING",&[&work_id,&envelope.task,&serde_json::to_value(work).map_err(|_|err(ErrorCode::InvalidArgument))?,&mission]).await?;
                let source_id = crate::validation::digest(&(&work_id, source))?;
                if let Some(row) = tx
                    .query_opt(
                        "SELECT id FROM contract_sources WHERE work_id=$1 AND source_revision=$2",
                        &[&work_id, &source.revision],
                    )
                    .await?
                {
                    if row.get::<_, String>(0) != source_id {
                        return Err(err(ErrorCode::IdempotencyConflict));
                    }
                }
                tx.execute("INSERT INTO contract_sources(id,work_id,source_revision,source) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING",&[&source_id,&work_id,&source.revision,&serde_json::to_value(source).map_err(|_|err(ErrorCode::InvalidArgument))?]).await?;
                let acceptance_id = crate::validation::digest(&intent.acceptance)?;
                tx.execute("INSERT INTO contract_acceptance(id,contract) VALUES($1,$2) ON CONFLICT DO NOTHING",&[&acceptance_id,&serde_json::to_value(&intent.acceptance).map_err(|_|err(ErrorCode::InvalidArgument))?]).await?;
                let id = crate::validation::digest(&(&work_id, &source_id, intent))?;
                if tx
                    .query_opt("SELECT id FROM contract_intents WHERE id=$1", &[&id])
                    .await?
                    .is_some()
                {
                    return Err(err(ErrorCode::StaleRevision));
                }
                let admitted_version: i64 = tx
                    .query_one(
                        "SELECT count(*)+1 FROM contract_intents WHERE work_id=$1",
                        &[&work_id],
                    )
                    .await?
                    .get(0);
                tx.execute("INSERT INTO contract_intents(id,work_id,source_id,acceptance_id,admitted_version,intent,principal_id,recovery_epoch) VALUES($1,$2,$3,$4,$5,$6,$7,$8)",&[&id,&work_id,&source_id,&acceptance_id,&admitted_version,&serde_json::to_value(intent).map_err(|_|err(ErrorCode::InvalidArgument))?,&actor.principal,&self.broker.epoch]).await?;
                dependencies::replace(tx, envelope.task, &intent.dependencies).await?;
                dependencies::supersede(tx, &work_id, &id).await?;
                let next = revision + 1;
                tx.execute("INSERT INTO contract_current(task_id,revision,intent_id,admission_open) VALUES($1,$2,$3,true) ON CONFLICT(task_id) DO UPDATE SET revision=$2,intent_id=$3,plan_id=NULL,admission_open=true",&[&envelope.task,&next,&id]).await?;
                Ok(
                    json!({"schema_version":2,"disposition":"applied","revision":next,"intent":id,"acceptance":acceptance_id,"work":work_id,"admitted_version":admitted_version}),
                )
            }
            Command::ProposeSource { work, source } => {
                if !matches!(actor.role, Role::Owner | Role::Lead | Role::Worker) {
                    return Err(err(ErrorCode::Unauthorized));
                }
                validate_work(work)?;
                let proposal = json!({"work":work,"source":source});
                let id = crate::validation::digest(&(envelope.task, actor.principal, &proposal))?;
                tx.execute("INSERT INTO contract_proposals(id,task_id,principal_id,proposal,recovery_epoch) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",&[&id,&envelope.task,&actor.principal,&proposal,&self.broker.epoch]).await?;
                Ok(
                    json!({"schema_version":2,"disposition":"proposed","revision":revision,"proposal":id}),
                )
            }
        }
    }
    pub async fn snapshot(&self, token: &str, task: Uuid) -> Result<Value> {
        let mut client = storage::client(&self.broker.dsn).await?;
        let tx = client.transaction().await?;
        storage::lock(&tx).await?;
        schema(&tx).await?;
        storage::active(&tx, self.broker.epoch).await?;
        let actor = auth::authenticate(&tx, token, self.broker.epoch).await?;
        storage::validate_actor_generation(&tx, &actor).await?;
        let mission: Uuid = tx
            .query_opt("SELECT mission_id FROM tasks WHERE id=$1", &[&task])
            .await?
            .ok_or_else(|| err(ErrorCode::NotFound))?
            .get(0);
        actor.scope(mission, Some(task))?;
        let current=tx.query_opt("SELECT revision,intent_id,plan_id,admission_open FROM contract_current WHERE task_id=$1",&[&task]).await?;
        let intents=tx.query("SELECT jsonb_build_object('id',i.id,'admitted_version',i.admitted_version,'source',i.source_id,'acceptance',i.acceptance_id,'intent',i.intent) FROM contract_intents i JOIN contract_work w ON w.id=i.work_id WHERE w.task_id=$1 ORDER BY i.admitted_version",&[&task]).await?.into_iter().map(|r|r.get::<_,Value>(0)).collect::<Vec<_>>();
        let sources=tx.query("SELECT s.source FROM contract_sources s JOIN contract_work w ON w.id=s.work_id WHERE w.task_id=$1 ORDER BY s.source_revision",&[&task]).await?.into_iter().map(|r|r.get::<_,Value>(0)).collect::<Vec<_>>();
        let plans=tx.query("SELECT jsonb_build_object('id',id,'admitted_version',admitted_version,'plan',plan) FROM contract_plans WHERE task_id=$1 ORDER BY admitted_version",&[&task]).await?.into_iter().map(|r|r.get::<_,Value>(0)).collect::<Vec<_>>();
        let accepted=tx.query("SELECT to_jsonb(o) FROM contract_outcomes o WHERE task_id=$1 ORDER BY created_at,subject",&[&task]).await?.into_iter().map(|r|r.get::<_,Value>(0)).collect::<Vec<_>>();
        let dependencies=tx.query("SELECT jsonb_build_object('work',w.work,'intent',d.intent_id,'accepted_subject',d.accepted_subject,'policy',d.policy,'disposition',d.disposition) FROM contract_dependencies d JOIN contract_work w ON w.id=d.prerequisite_work WHERE d.task_id=$1 ORDER BY d.prerequisite_work",&[&task]).await?.into_iter().map(|r|r.get::<_,Value>(0)).collect::<Vec<_>>();
        let work = tx
            .query_opt("SELECT work FROM contract_work WHERE task_id=$1", &[&task])
            .await?
            .map(|r| r.get::<_, Value>(0));
        let time: String = tx
            .query_one("SELECT clock_timestamp()::text", &[])
            .await?
            .get(0);
        let result = json!({"schema_version":2,"snapshot_time":time,"revision":current.as_ref().map_or(0,|r|r.get::<_,i64>(0)),"current_intent":current.as_ref().map(|r|r.get::<_,String>(1)),"current_plan":current.as_ref().and_then(|r|r.get::<_,Option<String>>(2)),"admission_open":current.as_ref().is_some_and(|r|r.get::<_,bool>(3)),"intents":intents,"sources":sources,"plans":plans,"accepted":accepted,"dependencies":dependencies,"work":work});
        tx.commit().await?;
        Ok(result)
    }
}
fn text(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 8192 && !s.contains('\0')
}
fn digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn unique(items: &[String]) -> bool {
    let mut values = std::collections::BTreeSet::new();
    items.iter().all(|s| text(s) && values.insert(s))
}
fn validate_work(work: &WorkRef) -> Result<()> {
    if ![
        &work.ledger,
        &work.deployment,
        &work.tenant,
        &work.project,
        &work.item,
    ]
    .into_iter()
    .all(|s| text(s))
        || !["git", "kata"].contains(&work.ledger.as_str())
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    Ok(())
}
fn validate_source(source: &SourceSnapshot) -> Result<()> {
    if source.availability != "available" || source.snapshot.is_empty() {
        return Err(err(ErrorCode::ArtifactMissing));
    }
    if source.snapshot.len() > 1024 * 1024
        || !text(&source.locator)
        || !text(&source.revision)
        || !digest(&source.digest)
        || crate::artifacts::sha256(&source.snapshot) != source.digest
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    Ok(())
}
fn validate_intent(intent: &IntentRevision) -> Result<()> {
    let acceptance = &intent.acceptance;
    if !text(&intent.objective)
        || !unique(&intent.constraints)
        || intent.allowed_effects.is_empty()
        || !unique(&intent.allowed_effects)
        || intent
            .allowed_effects
            .iter()
            .any(|e| !["read", "write", "execute", "submit", "verify"].contains(&e.as_str()))
        || !digest(&intent.legacy_spec_digest)
        || intent.resources.attempts <= 0
        || intent.resources.episodes <= 0
        || intent.resources.repairs < 0
        || intent.resources.execution_seconds <= 0
        || acceptance.criteria.is_empty()
        || !unique(&acceptance.criteria)
        || !text(&acceptance.policy)
        || !digest(&acceptance.suite)
        || !digest(&acceptance.environment)
        || (acceptance.outcome == Outcome::Production && acceptance.allow_negative)
        || acceptance.mandatory_commands.is_empty()
        || acceptance
            .mandatory_commands
            .iter()
            .any(|c| c.is_empty() || c.iter().any(|s| !text(s)))
    {
        return Err(err(ErrorCode::InvalidArgument));
    }
    Ok(())
}
