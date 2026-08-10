use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use clap::{Parser, Subcommand, ValueEnum};
use harp::context_control::{ProviderId, WorkflowChoice, WorkflowId};
use harp::{build_corpus, check_corpus, AppError, BuildMode};
use harp_artifacts::ArtifactStore;
use harp_cli_process::{CliProcessRuntime, ProcessRuntimeConfig};
use harp_contracts::{RunId, TaskGraph};
use harp_engine::{
    role_name, validate_graph, workspace_mode_name, Engine, EngineConfig, GraphPolicy,
    ProjectionPolicy, RunExecutionSpec, RunSummary,
};
use harp_runtime::fake::{FakeCodexRuntime, FakeStep};
use harp_runtime::ActivityRuntime;
use harp_state::StateStore;
use serde::Serialize;
use serde_json::Value;

#[derive(Parser)]
#[command(name = "harp", version, about = "Standalone RSI technical atlas")]
struct Cli {
    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    format: OutputFormat,
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Subcommand)]
enum Command {
    /// Validate the canonical corpus and evidence contracts.
    Check,
    /// Compile canonical Markdown into the Atlas payload.
    Build {
        #[arg(long)]
        check: bool,
        #[arg(long, default_value = "atlas/src/content/generated/corpus.json")]
        output: PathBuf,
    },
    /// Manage the local full-text index.
    Search {
        #[command(subcommand)]
        command: SearchCommand,
    },
    /// Verify or materialize public sources.
    Sources {
        #[command(subcommand)]
        command: SourcesCommand,
    },
    /// Verify repository import and generated-artifact contracts.
    Repository {
        #[command(subcommand)]
        command: RepositoryCommand,
    },
    /// Run and recover durable RLM task graphs.
    Rlm {
        #[command(subcommand)]
        command: RlmCommand,
    },
    /// Inspect local agent CLI provider capabilities.
    Providers {
        #[command(subcommand)]
        command: ProvidersCommand,
    },
    /// Manage immutable context-control releases.
    Releases {
        #[command(subcommand)]
        command: ReleasesCommand,
    },
    /// Run a task through the context-control surface.
    Run {
        #[arg(long, value_enum)]
        provider: ProviderArg,
        #[arg(long, value_enum, default_value_t = WorkflowArg::Auto)]
        workflow: WorkflowArg,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, value_enum)]
        sandbox: Option<SandboxArg>,
        #[arg(long, value_enum)]
        approval: Option<ApprovalArg>,
        #[arg(last = true, required = true, num_args = 1..)]
        task: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SearchCommand {
    Refresh,
    Status,
    Query {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum SourcesCommand {
    Verify,
    Materialize {
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand)]
enum RepositoryCommand {
    Verify,
}

#[derive(Subcommand)]
enum RlmCommand {
    Run {
        #[arg(long)]
        benchmark: PathBuf,
        #[arg(long, value_enum, default_value_t = RuntimeSelection::Codex)]
        runtime: RuntimeSelection,
        #[arg(long, default_value = ".harp/rlm")]
        state_dir: PathBuf,
        #[arg(long)]
        runtime_executable: Option<PathBuf>,
    },
    Resume {
        run_id: Option<String>,
        #[arg(long)]
        all_incomplete: bool,
        #[arg(long, value_enum, default_value_t = RuntimeSelection::Codex)]
        runtime: RuntimeSelection,
        #[arg(long, default_value = ".harp/rlm")]
        state_dir: PathBuf,
        #[arg(long)]
        runtime_executable: Option<PathBuf>,
    },
    Status {
        run_id: String,
        #[arg(long, default_value = ".harp/rlm")]
        state_dir: PathBuf,
    },
    Checkpoint {
        #[arg(long, default_value = ".harp/rlm")]
        state_dir: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum RuntimeSelection {
    Fake,
    Codex,
    Traecli,
}

#[derive(Subcommand)]
enum ProvidersCommand {
    Doctor {
        #[arg(long, value_enum)]
        provider: Option<ProviderArg>,
    },
}

#[derive(Subcommand)]
enum ReleasesCommand {
    Compile,
    List,
    Inspect { release_id: String },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProviderArg {
    Trae,
    Codex,
}

impl From<ProviderArg> for ProviderId {
    fn from(provider: ProviderArg) -> Self {
        match provider {
            ProviderArg::Trae => Self::Trae,
            ProviderArg::Codex => Self::Codex,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum WorkflowArg {
    #[default]
    Auto,
    #[value(name = "ci_repair")]
    CiRepair,
    #[value(name = "code_review")]
    CodeReview,
    #[value(name = "dependency_update")]
    DependencyUpdate,
    #[value(name = "general_coding")]
    GeneralCoding,
}

impl From<WorkflowArg> for WorkflowChoice {
    fn from(workflow: WorkflowArg) -> Self {
        match workflow {
            WorkflowArg::Auto => Self::Auto,
            WorkflowArg::CiRepair => Self::Workflow(WorkflowId::CiRepair),
            WorkflowArg::CodeReview => Self::Workflow(WorkflowId::CodeReview),
            WorkflowArg::DependencyUpdate => Self::Workflow(WorkflowId::DependencyUpdate),
            WorkflowArg::GeneralCoding => Self::Workflow(WorkflowId::GeneralCoding),
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum SandboxArg {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ApprovalArg {
    Untrusted,
    OnRequest,
    Never,
}

#[derive(Serialize)]
struct Envelope<T> {
    schema_version: u8,
    command: &'static str,
    status: &'static str,
    warnings: Vec<String>,
    data: T,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok((command, message, data)) => {
            emit_success(cli.format, command, &message, data);
            ExitCode::SUCCESS
        }
        Err(error) => {
            emit_error(cli.format, &error);
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(&'static str, String, Value), AppError> {
    let root = Path::new(".");
    match &cli.command {
        Command::Check => {
            let summary = check_corpus(root)?;
            Ok((
                "check",
                format!(
                    "Harp corpus checked: {} documents, {} concepts",
                    summary.canonical_documents, summary.retained_concepts
                ),
                serde_json::to_value(summary).expect("summary serializes"),
            ))
        }
        Command::Build { check, output } => {
            let result = build_corpus(
                root,
                Some(output),
                if *check {
                    BuildMode::Check
                } else {
                    BuildMode::Write
                },
            )?;
            let message = if *check {
                format!("{} matches canonical inputs", result.output.display())
            } else if result.matched {
                format!(
                    "{} already matches canonical inputs",
                    result.output.display()
                )
            } else {
                format!("wrote {}", result.output.display())
            };
            Ok((
                "build",
                message,
                serde_json::to_value(result).expect("build result serializes"),
            ))
        }
        Command::Search { command } => match command {
            SearchCommand::Refresh => {
                let receipt = harp::search::refresh(root)?;
                Ok((
                    "search.refresh",
                    format!(
                        "search index refreshed: {} documents",
                        receipt.document_count
                    ),
                    serde_json::to_value(receipt).expect("search receipt serializes"),
                ))
            }
            SearchCommand::Status => {
                let receipt = harp::search::status(root)?;
                Ok((
                    "search.status",
                    format!("search index current: {} documents", receipt.document_count),
                    serde_json::to_value(receipt).expect("search receipt serializes"),
                ))
            }
            SearchCommand::Query { query, limit } => {
                let results = harp::search::query(root, query, *limit)?;
                Ok((
                    "search.query",
                    format!("search returned {} results", results.len()),
                    serde_json::to_value(results).expect("search results serialize"),
                ))
            }
        },
        Command::Sources { command } => match command {
            SourcesCommand::Verify => {
                let report = harp::sources::verify(root)?;
                Ok((
                    "sources.verify",
                    format!(
                        "sources verified: {} evidence artifacts, {} snapshot files",
                        report.evidence_artifacts, report.snapshot_files
                    ),
                    serde_json::to_value(report).expect("source report serializes"),
                ))
            }
            SourcesCommand::Materialize { source, all } => {
                let report = harp::sources::materialize(root, source.as_deref(), *all)?;
                Ok((
                    "sources.materialize",
                    format!("materialized {} public sources", report.sources.len()),
                    serde_json::to_value(report).expect("materialize report serializes"),
                ))
            }
        },
        Command::Repository { command } => match command {
            RepositoryCommand::Verify => {
                let report = harp::repository::verify(root)?;
                Ok((
                    "repository.verify",
                    format!("repository verified: {} import rows", report.import_rows),
                    serde_json::to_value(report).expect("repository report serializes"),
                ))
            }
        },
        Command::Rlm { command } => run_rlm(root, command),
        Command::Providers { command } => match command {
            ProvidersCommand::Doctor { provider } => {
                let _provider = provider.map(ProviderId::from);
                Err(AppError::external(
                    "provider.not_implemented",
                    "provider doctor is not implemented",
                ))
            }
        },
        Command::Releases { command } => match command {
            ReleasesCommand::Compile => {
                let release = harp::context_control::release::compile_and_publish_baseline()?;
                Ok((
                    "releases.compile",
                    format!("compiled context release {}", release.release_id),
                    serde_json::to_value(release).expect("release summary serializes"),
                ))
            }
            ReleasesCommand::List => {
                let releases = harp::context_control::release::list_releases()?;
                let count = releases.releases.len();
                Ok((
                    "releases.list",
                    format!("found {count} context releases"),
                    serde_json::to_value(releases).expect("release list serializes"),
                ))
            }
            ReleasesCommand::Inspect { release_id } => {
                let release = harp::context_control::release::inspect_release(release_id)?;
                Ok((
                    "releases.inspect",
                    format!("validated context release {release_id}"),
                    serde_json::to_value(release).expect("release inspection serializes"),
                ))
            }
        },
        Command::Run {
            provider,
            workflow,
            model,
            profile,
            sandbox,
            approval,
            task,
        } => {
            let _request = (
                ProviderId::from(*provider),
                WorkflowChoice::from(*workflow),
                model,
                profile,
                sandbox,
                approval,
                task,
            );
            Err(AppError::external(
                "run.not_implemented",
                "context-control run is not implemented",
            ))
        }
    }
}

fn run_rlm(root: &Path, command: &RlmCommand) -> Result<(&'static str, String, Value), AppError> {
    let runtime = tokio::runtime::Runtime::new().map_err(|error| {
        AppError::external(
            "rlm.runtime",
            format!("could not start async runtime: {error}"),
        )
    })?;
    runtime.block_on(async {
        match command {
            RlmCommand::Run {
                benchmark,
                runtime,
                state_dir,
                runtime_executable,
            } => {
                let mut env = RlmEnvironment::open(root, state_dir)?;
                let graph = read_task_graph(benchmark)?;
                let (graph_policy, projection_policy) = default_policies(&graph, &env)?;
                let mut runtime =
                    build_activity_runtime(*runtime, runtime_executable.as_deref(), &env).await?;
                let provenance = runtime.provenance().map_err(app_runtime_error)?;
                let validated =
                    validate_graph(graph, &graph_policy, &projection_policy).map_err(|error| {
                        AppError::invalid_input(
                            "rlm.graph",
                            format!("benchmark graph validation failed: {error}"),
                        )
                    })?;
                let spec = RunExecutionSpec::new(
                    validated,
                    graph_policy,
                    projection_policy,
                    &env.artifacts,
                    &provenance,
                )
                .map_err(app_engine_error)?;
                let mut engine = Engine::new(default_engine_config()).map_err(app_engine_error)?;
                let summary = engine
                    .execute_run(&mut env.state, &env.artifacts, runtime.as_mut(), &spec)
                    .await
                    .map_err(app_engine_error)?;
                let value = run_summary_value(&summary);
                Ok((
                    "rlm.run",
                    format!("rlm run {} completed", summary.run_id),
                    value,
                ))
            }
            RlmCommand::Resume {
                run_id,
                all_incomplete,
                runtime,
                state_dir,
                runtime_executable,
            } => {
                if run_id.is_some() == *all_incomplete {
                    return Err(AppError::invalid_input(
                        "rlm.resume.selection",
                        "provide exactly one run id or --all-incomplete",
                    ));
                }
                let mut env = RlmEnvironment::open(root, state_dir)?;
                let mut runtime =
                    build_activity_runtime(*runtime, runtime_executable.as_deref(), &env).await?;
                let mut engine = Engine::new(default_engine_config()).map_err(app_engine_error)?;
                let summaries = if *all_incomplete {
                    let run_ids = env
                        .state
                        .incomplete_runs()
                        .map_err(app_state_error)?
                        .into_iter()
                        .map(|run| run.run_id)
                        .collect::<Vec<_>>();
                    let mut summaries = Vec::new();
                    for run_id in run_ids {
                        summaries.push(
                            engine
                                .resume_run(
                                    &mut env.state,
                                    &env.artifacts,
                                    runtime.as_mut(),
                                    &run_id,
                                )
                                .await
                                .map_err(app_engine_error)?,
                        );
                    }
                    summaries
                } else {
                    let run_id = parse_run_id(run_id.as_deref().expect("checked run id"))?;
                    vec![
                        engine
                            .resume_run(&mut env.state, &env.artifacts, runtime.as_mut(), &run_id)
                            .await
                            .map_err(app_engine_error)?,
                    ]
                };
                Ok((
                    "rlm.resume",
                    format!("resumed {} rlm run(s)", summaries.len()),
                    serde_json::json!({
                        "runs": summaries.iter().map(run_summary_value).collect::<Vec<_>>()
                    }),
                ))
            }
            RlmCommand::Status { run_id, state_dir } => {
                let mut env = RlmEnvironment::open(root, state_dir)?;
                let run_id = parse_run_id(run_id)?;
                let run = env
                    .state
                    .get_run(&run_id)
                    .map_err(app_state_error)?
                    .ok_or_else(|| {
                        AppError::invalid_input("rlm.run_missing", "run does not exist")
                    })?;
                let tasks = env.state.tasks(&run_id).map_err(app_state_error)?;
                let attempts = env.state.attempts(&run_id).map_err(app_state_error)?;
                let accepted = env.state.accepted_results(&run_id).map_err(app_state_error)?;
                Ok((
                    "rlm.status",
                    format!("rlm run {} is {}", run_id, run.state.as_str()),
                    serde_json::json!({
                        "run_id": run_id.to_string(),
                        "state": run.state.as_str(),
                        "cancellation_requested": run.cancellation_requested,
                        "graph_sha256": run.graph_sha256,
                        "created_at": run.created_at,
                        "updated_at": run.updated_at,
                        "tasks": tasks.iter().map(|task| serde_json::json!({
                            "task_id": task.task_id.to_string(),
                            "kind": format!("{:?}", task.kind),
                            "state": task.state.as_str(),
                            "accepted_attempt_id": task.accepted_attempt_id.as_ref().map(ToString::to_string),
                        })).collect::<Vec<_>>(),
                        "attempts": attempts.iter().map(|attempt| serde_json::json!({
                            "attempt_id": attempt.attempt_id.to_string(),
                            "task_id": attempt.task_id.to_string(),
                            "ordinal": attempt.ordinal,
                            "state": attempt.state.as_str(),
                            "result_sha256": attempt.result_sha256,
                        })).collect::<Vec<_>>(),
                        "accepted_results": accepted.len(),
                    }),
                ))
            }
            RlmCommand::Checkpoint { state_dir } => {
                let env = RlmEnvironment::open(root, state_dir)?;
                Ok((
                    "rlm.checkpoint",
                    format!("rlm checkpoint root {}", env.root.display()),
                    serde_json::json!({
                        "state_dir": env.root,
                        "state_path": env.state_path,
                        "artifact_root": env.artifact_root,
                        "runtime_home": env.runtime_home,
                    }),
                ))
            }
        }
    })
}

fn emit_success(format: OutputFormat, command: &'static str, message: &str, data: Value) {
    match format {
        OutputFormat::Text => println!("{message}"),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string(&Envelope {
                schema_version: 1,
                command,
                status: "ok",
                warnings: Vec::new(),
                data,
            })
            .expect("envelope serializes")
        ),
    }
}

fn emit_error(format: OutputFormat, error: &AppError) {
    match format {
        OutputFormat::Text => eprintln!("{}", error.message),
        OutputFormat::Json => eprintln!(
            "{}",
            serde_json::to_string(&Envelope {
                schema_version: 1,
                command: "error",
                status: "error",
                warnings: Vec::new(),
                data: serde_json::json!({
                    "code": error.code,
                    "message": error.message,
                }),
            })
            .expect("error envelope serializes")
        ),
    }
}

struct RlmEnvironment {
    root: PathBuf,
    state_path: PathBuf,
    artifact_root: PathBuf,
    runtime_home: PathBuf,
    state: StateStore,
    artifacts: ArtifactStore,
}

impl RlmEnvironment {
    fn open(repo_root: &Path, state_dir: &Path) -> Result<Self, AppError> {
        let root = repo_root.join(state_dir);
        create_private_dir(&root)?;
        let root = root
            .canonicalize()
            .map_err(|error| AppError::io("rlm.path", "canonicalize RLM directory", error))?;
        let artifact_root = root.join("artifacts");
        create_private_dir(&artifact_root)?;
        let runtime_home = root.join("runtime-home");
        create_private_dir(&runtime_home)?;
        let state_path = root.join("state.sqlite");
        let state = StateStore::open(&state_path).map_err(app_state_error)?;
        let artifacts = ArtifactStore::open(&artifact_root).map_err(app_artifact_error)?;
        Ok(Self {
            root,
            state_path,
            artifact_root,
            runtime_home,
            state,
            artifacts,
        })
    }
}

fn create_private_dir(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        if !path.is_dir() {
            return Err(AppError::invalid_input(
                "rlm.path",
                format!("{} must be a directory", path.display()),
            ));
        }
    } else {
        fs::create_dir_all(path)
            .map_err(|error| AppError::io("rlm.mkdir", "create RLM directory", error))?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|error| AppError::io("rlm.permissions", "set RLM directory permissions", error))
}

fn read_task_graph(path: &Path) -> Result<TaskGraph, AppError> {
    let bytes = fs::read(path)
        .map_err(|error| AppError::io("rlm.benchmark", "read benchmark graph", error))?;
    if bytes.len() > 1024 * 1024 {
        return Err(AppError::invalid_input(
            "rlm.benchmark_size",
            "benchmark graph exceeds 1 MiB",
        ));
    }
    let graph: TaskGraph = serde_json::from_slice(&bytes).map_err(|error| {
        AppError::invalid_input(
            "rlm.benchmark_json",
            format!("invalid benchmark JSON: {error}"),
        )
    })?;
    graph.validate_shape().map_err(|error| {
        AppError::invalid_input(
            "rlm.benchmark_contract",
            format!("invalid benchmark graph: {error}"),
        )
    })?;
    Ok(graph)
}

fn default_policies(
    graph: &TaskGraph,
    env: &RlmEnvironment,
) -> Result<(GraphPolicy, ProjectionPolicy), AppError> {
    let mut allowed_roles = BTreeSet::new();
    let mut allowed_output_schemas = BTreeSet::new();
    let mut allowed_model_policies = BTreeSet::new();
    let mut allowed_permission_profiles = BTreeSet::new();
    let mut allowed_workspace_modes = BTreeSet::new();
    let mut scratch_paths = BTreeMap::new();
    let mut max_node_tokens = 1_u64;
    let mut max_node_storage_bytes = 1_u64;
    let mut max_node_timeout_seconds = 1_u64;
    let mut total_tokens = 0_u64;
    let mut total_storage = 0_u64;
    let mut total_timeout = 0_u64;
    for node in &graph.nodes {
        allowed_roles.insert(role_name(node.role).to_owned());
        allowed_output_schemas.insert(node.output_schema.clone());
        allowed_model_policies.insert(node.model_policy.clone());
        allowed_permission_profiles.insert(node.permission_profile.clone());
        allowed_workspace_modes.insert(workspace_mode_name(node.workspace_mode).to_owned());
        let scratch = env
            .root
            .join("scratch")
            .join(node.task_id.to_string())
            .to_str()
            .ok_or_else(|| {
                AppError::invalid_input("rlm.scratch_path", "scratch path is not UTF-8")
            })?
            .to_owned();
        scratch_paths.insert(node.task_id.clone(), scratch);
        max_node_tokens = max_node_tokens.max(node.budget.max_tokens);
        max_node_storage_bytes = max_node_storage_bytes.max(node.budget.max_storage_bytes);
        max_node_timeout_seconds = max_node_timeout_seconds.max(node.budget.timeout_seconds);
        total_tokens = total_tokens
            .checked_add(node.budget.max_tokens)
            .ok_or_else(|| AppError::invalid_input("rlm.budget", "token budget overflow"))?;
        total_storage = total_storage
            .checked_add(node.budget.max_storage_bytes)
            .ok_or_else(|| AppError::invalid_input("rlm.budget", "storage budget overflow"))?;
        total_timeout = total_timeout
            .checked_add(node.budget.timeout_seconds)
            .ok_or_else(|| AppError::invalid_input("rlm.budget", "timeout budget overflow"))?;
    }
    allowed_roles.insert("reduce".to_owned());
    let graph_policy = GraphPolicy {
        max_nodes: graph.nodes.len().max(1),
        max_concurrency: graph.nodes.len().clamp(1, 4),
        max_total_tokens: total_tokens.max(max_node_tokens),
        max_total_storage_bytes: total_storage.max(max_node_storage_bytes),
        max_total_timeout_seconds: total_timeout.max(max_node_timeout_seconds),
        max_node_tokens,
        max_node_storage_bytes,
        max_node_timeout_seconds,
        max_projected_prompt_bytes: 1024 * 1024,
        max_recursion_depth: 1,
        allowed_roles,
        allowed_output_schemas,
        allowed_model_policies,
        allowed_permission_profiles,
        allowed_workspace_modes,
        approved_artifacts: BTreeMap::new(),
    };
    let projection_policy = ProjectionPolicy {
        base_instructions: "Execute the assigned Harp RLM task and return only the required JSON."
            .to_owned(),
        role_instructions: [
            (
                "explore".to_owned(),
                "Investigate the assigned task.".to_owned(),
            ),
            (
                "classify".to_owned(),
                "Classify the assigned evidence.".to_owned(),
            ),
            (
                "implement".to_owned(),
                "Implement the requested change.".to_owned(),
            ),
            (
                "verify".to_owned(),
                "Verify the assigned result.".to_owned(),
            ),
            (
                "critic".to_owned(),
                "Review the assigned result.".to_owned(),
            ),
            (
                "reduce".to_owned(),
                "Synthesize accepted child results.".to_owned(),
            ),
        ]
        .into_iter()
        .collect(),
        checkpoint_instructions: "Write checkpoints only when useful for durable recovery."
            .to_owned(),
        max_bytes: 1024 * 1024,
        scratch_paths,
    };
    Ok((graph_policy, projection_policy))
}

async fn build_activity_runtime(
    selection: RuntimeSelection,
    executable: Option<&Path>,
    env: &RlmEnvironment,
) -> Result<Box<dyn ActivityRuntime>, AppError> {
    match selection {
        RuntimeSelection::Fake => Ok(Box::new(
            FakeCodexRuntime::new(Vec::<FakeStep>::new()).map_err(|error| {
                AppError::external("rlm.fake_runtime", format!("invalid fake runtime: {error}"))
            })?,
        )),
        RuntimeSelection::Codex => {
            let executable = match executable {
                Some(path) => path.to_path_buf(),
                None => find_on_path("codex").ok_or_else(|| {
                    AppError::invalid_input(
                        "rlm.codex_missing",
                        "codex executable not found; pass --runtime-executable",
                    )
                })?,
            };
            let config = with_test_cli_env(
                ProcessRuntimeConfig::builder(executable, &env.runtime_home)
                    .exec_args(["exec", "--json"])
                    .build()
                    .map_err(app_runtime_error)?,
            )?;
            Ok(Box::new(
                CliProcessRuntime::spawn(config)
                    .await
                    .map_err(app_runtime_error)?,
            ))
        }
        RuntimeSelection::Traecli => Err(AppError::invalid_input(
            "rlm.runtime",
            "traecli runtime is not implemented in this adapter slice",
        )),
    }
}

#[cfg(feature = "test-cli-fixture")]
fn with_test_cli_env(mut config: ProcessRuntimeConfig) -> Result<ProcessRuntimeConfig, AppError> {
    for (key, value) in std::env::vars() {
        if key.starts_with("HARP_FAKE_CLI_") {
            config = config.with_env(key, value).map_err(app_runtime_error)?;
        }
    }
    Ok(config)
}

#[cfg(not(feature = "test-cli-fixture"))]
fn with_test_cli_env(config: ProcessRuntimeConfig) -> Result<ProcessRuntimeConfig, AppError> {
    Ok(config)
}

fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

fn default_engine_config() -> EngineConfig {
    let lease_seconds = test_lease_seconds().unwrap_or(60);
    EngineConfig {
        worker_id: format!("harp-cli-{}", std::process::id()),
        lease_seconds,
        lease_renewal_threshold_seconds: (lease_seconds / 3).max(1).min(lease_seconds - 1),
        runtime_operation_timeout_seconds: 30,
        max_concurrency: 4,
        logical_time: 1,
        crash_point: None,
    }
}

#[cfg(feature = "test-cli-fixture")]
fn test_lease_seconds() -> Option<i64> {
    std::env::var("HARP_RLM_TEST_LEASE_SECONDS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 1)
}

#[cfg(not(feature = "test-cli-fixture"))]
fn test_lease_seconds() -> Option<i64> {
    None
}

fn parse_run_id(value: &str) -> Result<RunId, AppError> {
    RunId::from_str(value)
        .map_err(|error| AppError::invalid_input("rlm.run_id", format!("invalid run id: {error}")))
}

fn run_summary_value(summary: &RunSummary) -> Value {
    serde_json::json!({
        "run_id": summary.run_id.to_string(),
        "completed_tasks": summary.completed_tasks,
        "indeterminate_attempts": summary.indeterminate_attempts,
    })
}

fn app_engine_error(error: harp_engine::EngineError) -> AppError {
    AppError::external("rlm.engine", error.to_string())
}

fn app_runtime_error(error: harp_runtime::RuntimeError) -> AppError {
    AppError::external("rlm.runtime", error.to_string())
}

fn app_state_error(error: harp_state::StateError) -> AppError {
    AppError::external("rlm.state", error.to_string())
}

fn app_artifact_error(error: harp_artifacts::ArtifactError) -> AppError {
    AppError::external("rlm.artifact", error.to_string())
}
