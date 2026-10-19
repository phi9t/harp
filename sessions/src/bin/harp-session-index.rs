use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use harp_session_index::detect::detect_dialect;
use harp_session_index::index::{index_rollout, IndexOptions, SessionSummary};
use harp_session_index::store::{default_index_root, session_dir};
use harp_session_index::verify::{verify_session, VerifyOptions, VerifyVerdict};

#[derive(Parser, Debug)]
#[command(
    name = "harp-session-index",
    about = "Index TraeCLI Execution sessions from raw rollouts"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Detect the Trace dialect of a rollout JSONL.
    Detect { rollout: PathBuf },
    /// Build a Session index under the index root (default: XDG harp/session-index).
    Index {
        rollout: PathBuf,
        #[arg(long)]
        index_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 2 * 1024 * 1024)]
        oversize_line_bytes: usize,
    },
    /// Print a previously built Session summary JSON.
    Show {
        session_id: String,
        #[arg(long)]
        index_root: Option<PathBuf>,
    },
    /// Re-index a rollout and diff against the stored Session summary.
    Verify {
        rollout: PathBuf,
        session_id: String,
        #[arg(long)]
        index_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 2 * 1024 * 1024)]
        oversize_line_bytes: usize,
        /// Write verify-report.json beside the claimed summary.
        #[arg(long, default_value_t = true)]
        write_report: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Detect { rollout } => detect_dialect(&rollout).map(|d| {
            println!("{}", d.as_str());
        }),
        Commands::Index {
            rollout,
            index_root,
            artifacts_dir,
            oversize_line_bytes,
        } => {
            let root = index_root.unwrap_or_else(default_index_root);
            index_rollout(
                &rollout,
                &root,
                IndexOptions {
                    oversize_line_bytes,
                    artifacts_dir,
                },
            )
            .map(|summary| {
                println!(
                    "indexed {} dialect={} digest={}",
                    summary.session_id, summary.dialect, summary.index_digest
                );
                println!("index_root={}", root.display());
            })
        }
        Commands::Show {
            session_id,
            index_root,
        } => {
            let root = index_root.unwrap_or_else(default_index_root);
            (|| -> harp_session_index::SessionIndexResult<()> {
                let path = session_dir(&root, &session_id)?.join("summary.json");
                let raw = fs::read_to_string(&path).map_err(|source| {
                    harp_session_index::SessionIndexError::Io {
                        context: format!("read {}", path.display()),
                        source,
                    }
                })?;
                let summary: SessionSummary = serde_json::from_str(&raw).map_err(|source| {
                    harp_session_index::SessionIndexError::InvalidStoredSummary {
                        path: path.clone(),
                        source,
                    }
                })?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary).map_err(|source| {
                        harp_session_index::SessionIndexError::Serialization {
                            artifact: "session summary stdout",
                            source,
                        }
                    })?
                );
                Ok(())
            })()
        }
        Commands::Verify {
            rollout,
            session_id,
            index_root,
            artifacts_dir,
            oversize_line_bytes,
            write_report,
        } => {
            let root = index_root.unwrap_or_else(default_index_root);
            match verify_session(
                &rollout,
                &session_id,
                VerifyOptions {
                    index_root: root,
                    artifacts_dir,
                    oversize_line_bytes,
                    write_report,
                },
            ) {
                Ok(report) => {
                    println!(
                        "verdict={:?} checks={}/{} claimed={} observed={}",
                        report.verdict,
                        report.checks_passed,
                        report.checks_total,
                        report.claimed_index_digest,
                        report.observed_index_digest
                    );
                    for item in &report.feedback {
                        println!("[{:?}] {}: {}", item.severity, item.id, item.text);
                    }
                    if report.verdict == VerifyVerdict::Fail {
                        std::process::exit(2);
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            }
        }
    };
    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
