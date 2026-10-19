mod durable;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use harp::context_control::provider::{locate, probe};
use harp::context_control::ProviderId;
use harp::{
    build_corpus, check_corpus, publish_crouzeix_textbook, AppError, BuildMode, TextbookPublishMode,
};
use serde::Serialize;
use serde_json::Value;

#[derive(Parser)]
#[command(name = "harp", version, about = "Local knowledge and agent runtime")]
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
    /// Run the project-scoped lightweight durable runtime.
    #[command(disable_help_flag = true)]
    Durable {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Validate the canonical corpus and evidence contracts.
    Check,
    /// Compile canonical Markdown into the headless corpus.
    Build {
        #[arg(long)]
        check: bool,
        #[arg(long, default_value = "knowledge/generated/corpus.json")]
        output: PathBuf,
    },
    /// Check or publish the verified Crouzeix textbook ledgers.
    CrouzeixTextbook {
        #[command(subcommand)]
        command: CrouzeixTextbookCommand,
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
    /// Retired task-graph commands; use harp durable.
    Rlm {
        #[command(subcommand)]
        command: RlmCommand,
    },
    /// Retired workflow commands; use harp durable.
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommand,
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
    /// Retired direct provider wrapper; use harp durable.
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
    /// Sanitize and scrub PII, machine identifiable information, and proprietary data.
    Sanitize {
        #[command(subcommand)]
        command: SanitizeCommand,
    },
}

#[derive(Subcommand)]
enum SanitizeCommand {
    /// Scrub text from a file or standard input.
    Text {
        #[arg(long)]
        input: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Check a file or directory for unredacted PII, machine info, or proprietary terms.
    Check {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Scrub a file in place.
    Scrub { path: PathBuf },
}

#[derive(Subcommand)]
enum CrouzeixTextbookCommand {
    Check {
        #[arg(long, required = true)]
        receipt: PathBuf,
    },
    Publish {
        #[arg(long, required = true)]
        receipt: PathBuf,
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

#[derive(Subcommand)]
enum WorkflowCommand {
    Run {
        #[arg(long)]
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = RuntimeSelection::Codex)]
        runtime: RuntimeSelection,
        #[arg(long, default_value = ".harp/workflow")]
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
        #[arg(long, default_value = ".harp/workflow")]
        state_dir: PathBuf,
        #[arg(long)]
        runtime_executable: Option<PathBuf>,
    },
    Status {
        run_id: String,
        #[arg(long, default_value = ".harp/workflow")]
        state_dir: PathBuf,
    },
    /// Inspect persisted evidence without starting a runtime or resuming work.
    Inspect {
        run_id: String,
        #[arg(long, default_value = ".harp/workflow")]
        state_dir: PathBuf,
        #[arg(long)]
        after_sequence: Option<i64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Explain one task using persisted failures, dependencies and result references.
    Explain {
        run_id: String,
        #[arg(long)]
        task: String,
        #[arg(long, default_value = ".harp/workflow")]
        state_dir: PathBuf,
        #[arg(long)]
        after_sequence: Option<i64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
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

fn parse_cli() -> Cli {
    let argv: Vec<_> = env::args_os().collect();
    // Parse only Harp's prefix: backend flags (including --format and --)
    // must remain untouched once the durable subcommand has been selected.
    if let Some(index) = argv.iter().position(|arg| arg == "durable") {
        if let Ok(mut cli) = Cli::try_parse_from(&argv[..=index]) {
            if let Command::Durable { args } = &mut cli.command {
                *args = argv[index + 1..].to_vec();
                return cli;
            }
        }
    }
    Cli::parse_from(argv)
}

fn main() -> ExitCode {
    let cli = parse_cli();
    if let Command::Durable { args } = &cli.command {
        let format = match cli.format {
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
        };
        let error = durable::exec(format, args);
        emit_error(cli.format, &error);
        return ExitCode::FAILURE;
    }

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
        Command::Durable { .. } => unreachable!("durable runtime replaces the CLI process"),
        Command::Check => {
            let summary = check_corpus(root)?;
            Ok((
                "check",
                format!(
                    "Harp corpus checked: {} documents, {} reader routes",
                    summary.canonical_documents, summary.reader_routes
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
        Command::CrouzeixTextbook { command } => {
            let (receipt, mode, command_name) = match command {
                CrouzeixTextbookCommand::Check { receipt } => (
                    receipt,
                    TextbookPublishMode::Check,
                    "crouzeix-textbook.check",
                ),
                CrouzeixTextbookCommand::Publish { receipt } => (
                    receipt,
                    TextbookPublishMode::Write,
                    "crouzeix-textbook.publish",
                ),
            };
            let result = publish_crouzeix_textbook(root, receipt, mode)?;
            if mode == TextbookPublishMode::Check && !result.matched {
                return Err(AppError::invalid_input(
                    "crouzeix-textbook.publication.stale",
                    "Crouzeix textbook publication does not match canonical inputs",
                ));
            }
            let message = match (mode, result.matched) {
                (TextbookPublishMode::Check, true) => {
                    "Crouzeix textbook publication matches canonical inputs".to_owned()
                }
                (TextbookPublishMode::Write, true) => {
                    "Crouzeix textbook publication already matches canonical inputs".to_owned()
                }
                (TextbookPublishMode::Write, false) => format!(
                    "published {} Crouzeix textbook ledgers",
                    result.outputs.len()
                ),
                (TextbookPublishMode::Check, false) => unreachable!("stale check returned above"),
            };
            Ok((
                command_name,
                message,
                serde_json::to_value(result).expect("textbook publication result serializes"),
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
        Command::Rlm { .. } => Err(retired("rlm", "rlm.retired")),
        Command::Workflow { .. } => Err(retired("workflow", "workflow.retired")),
        Command::Providers { command } => match command {
            ProvidersCommand::Doctor { provider } => {
                let path_env = env::var_os("PATH").ok_or_else(|| {
                    AppError::external("provider.path", "PATH is required for provider discovery")
                })?;
                let providers = match provider {
                    Some(provider) => vec![ProviderId::from(*provider)],
                    None => vec![ProviderId::Trae, ProviderId::Codex],
                };
                let snapshots = providers
                    .into_iter()
                    .map(|provider| {
                        let executable = locate(provider, &path_env)?;
                        probe(provider, &executable)
                    })
                    .collect::<Result<Vec<_>, AppError>>()?;
                let message = snapshots
                    .iter()
                    .map(|snapshot| {
                        format!(
                            "{} executable={} version={} capabilities=ready",
                            snapshot.provider,
                            snapshot.executable.display(),
                            snapshot.version
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok((
                    "providers.doctor",
                    message,
                    serde_json::to_value(snapshots).expect("provider snapshots serialize"),
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
        Command::Sanitize { command } => match command {
            SanitizeCommand::Text { input, output } => {
                let content = if let Some(path) = input {
                    fs::read_to_string(path)
                        .map_err(|e| AppError::io("sanitize.text", "read input", e))?
                } else {
                    use std::io::Read;
                    let mut buffer = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buffer)
                        .map_err(|e| AppError::io("sanitize.text", "read stdin", e))?;
                    buffer
                };
                let sanitizer = harp::sanitize::Sanitizer::with_default_config();
                let scrubbed = sanitizer.scrub_str(&content);
                if let Some(path) = output {
                    fs::write(path, &scrubbed)
                        .map_err(|e| AppError::io("sanitize.text", "write output", e))?;
                    Ok((
                        "sanitize.text",
                        format!("scrubbed content written to {}", path.display()),
                        serde_json::json!({ "output": path.display().to_string() }),
                    ))
                } else {
                    Ok((
                        "sanitize.text",
                        scrubbed.clone(),
                        serde_json::json!({ "scrubbed": scrubbed }),
                    ))
                }
            }
            SanitizeCommand::Check { path } => {
                let sanitizer = harp::sanitize::Sanitizer::with_default_config();
                let report = sanitizer.scan_path(path)?;
                if !report.clean {
                    let mut summary = format!(
                        "Sanitization check failed: {} violation(s) across {} file(s)\n",
                        report.total_findings,
                        report.file_reports.len()
                    );
                    for file_report in &report.file_reports {
                        summary.push_str(&format!("  {}:\n", file_report.path.display()));
                        for finding in &file_report.findings {
                            summary.push_str(&format!(
                                "    line {}:{}: [{}] {} ({})\n",
                                finding.line_number,
                                finding.start_col,
                                finding.category,
                                finding.rule_name,
                                finding.snippet
                            ));
                        }
                    }
                    Err(AppError::invalid_input("sanitize.check", summary))
                } else {
                    Ok((
                        "sanitize.check",
                        format!(
                            "Sanitization check passed: {} file(s) clean",
                            report.scanned_files
                        ),
                        serde_json::to_value(&report).expect("report serializes"),
                    ))
                }
            }
            SanitizeCommand::Scrub { path } => {
                let sanitizer = harp::sanitize::Sanitizer::with_default_config();
                let count = sanitizer.scrub_path(path)?;
                Ok((
                    "sanitize.scrub",
                    format!("scrubbed {} file(s) under {}", count, path.display()),
                    serde_json::json!({ "path": path.display().to_string(), "modified_files": count }),
                ))
            }
        },
        Command::Run { .. } => Err(retired("run", "run.retired")),
    }
}

#[allow(clippy::too_many_arguments)]
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

fn retired(command: &str, code: &'static str) -> AppError {
    AppError::invalid_input(
        code,
        format!(
            "harp {command} is retired. {}",
            harp::context_control::run::EXECUTION_REPLACEMENT
        ),
    )
}
