use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use harp::{build_corpus, check_corpus, AppError, BuildMode};
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
    }
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
