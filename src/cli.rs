use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use tokio::sync::{mpsc, oneshot};

use crate::core::{
    ActiveTurn, ApprovalDecision, ApprovalRequest, ApprovalResponder, Coordinator, PolicyGate,
    ProviderEvent, ProviderEventSink, SandboxResult, TranscriptStore,
};
use crate::eval::{
    AimeDatasetSource, AimeEvalOptions, AimeExecutionMode, AimeRegradeOptions, AimeReplayOptions,
    AimeSanityOptions, AimeSelection, run_aime_eval, run_aime_regrade, run_aime_replay,
    run_aime_sanity,
};
use crate::insula::InsulaSandbox;
use crate::provider::acp::{
    AcpOptions, SupervisorSink, TraeCliAcpProvider, run_acp_prompt, run_acp_prompt_observed,
    run_acp_prompt_streaming,
};
use crate::provider::codex::CodexDirectProvider;
use crate::supervisor::ProcessSupervisor;
use crate::tools::{ShellRunner, ToolRegistry};
use crate::tui::{TuiState, render_rows};

#[derive(Parser, Debug)]
#[command(name = "corenius", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Run(WorkspaceArgs),
    Eval {
        #[command(subcommand)]
        command: EvalCommand,
    },
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Insula {
        #[command(subcommand)]
        command: InsulaCommand,
    },
}

#[derive(Args, Debug)]
struct WorkspaceArgs {
    #[arg(long)]
    workspace: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ProviderKind::Codex)]
    provider: ProviderKind,
    #[arg(long)]
    model: Option<String>,
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum ProviderKind {
    Codex,
    Traecli,
}

#[derive(Subcommand, Debug)]
enum EvalCommand {
    Aime {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, num_args = 1.., default_values_t = default_aime_years())]
        years: Vec<String>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long)]
        filter_id: Vec<String>,
        #[arg(long, default_value_t = 300_000)]
        timeout_ms: u64,
        #[arg(long)]
        stream: bool,
        #[arg(long)]
        refresh_data: bool,
    },
    AimeSanity {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long, default_value = "evals/aime_sanity.jsonl")]
        dataset: PathBuf,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 180_000)]
        timeout_ms: u64,
        #[arg(long)]
        stream: bool,
        #[arg(long)]
        include_timeout_probe: bool,
        #[arg(long)]
        filter_id: Vec<String>,
    },
    AimeReplay {
        #[arg(long)]
        from_run: PathBuf,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value_t = 300_000)]
        timeout_ms: u64,
        #[arg(long)]
        stream: bool,
        #[arg(long, default_value_t = true)]
        failed_only: bool,
        #[arg(long)]
        filter_id: Vec<String>,
    },
    AimeRegrade {
        #[arg(long)]
        from_run: PathBuf,
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum ProviderCommand {
    Smoke {
        #[command(subcommand)]
        command: ProviderSmokeCommand,
    },
    Supervise {
        #[command(subcommand)]
        command: ProviderSuperviseCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ProviderSmokeCommand {
    Traecli {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long, default_value = "reply with exactly the word: ok")]
        prompt: String,
        #[arg(long, default_value_t = 120_000)]
        timeout_ms: u64,
        #[arg(long)]
        stream: bool,
        #[arg(long)]
        supervise: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ProviderSuperviseCommand {
    Traecli {
        #[arg(long)]
        pid: u32,
    },
}

#[derive(Subcommand, Debug)]
enum InsulaCommand {
    Scan {
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Select {
        #[arg(long)]
        rootfs: PathBuf,
        #[arg(long)]
        store_root: PathBuf,
        #[arg(long)]
        workspace: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        workspace: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
}

pub async fn run() -> anyhow::Result<()> {
    run_from(Cli::parse()).await
}

async fn run_from(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Run(args) => {
            let workspace = workspace_or_current(args.workspace)?;
            run_interactive(workspace, args.provider, args.model).await
        }
        Command::Eval { command } => match command {
            EvalCommand::Aime {
                workspace,
                run_dir,
                model,
                years,
                limit,
                offset,
                filter_id,
                timeout_ms,
                stream,
                refresh_data,
            } => {
                let workspace = workspace_or_current(workspace)?;
                let run_dir = run_dir.unwrap_or_else(|| default_aime_run_dir(&workspace));
                let summary = run_aime_eval(AimeEvalOptions {
                    workspace: workspace.clone(),
                    source: AimeDatasetSource::Gair {
                        years: years.clone(),
                        data_dir: workspace.join(".scratch/aime-benchmark/data"),
                        refresh_data,
                    },
                    run_dir,
                    model,
                    timeout_ms,
                    stream,
                    selection: AimeSelection {
                        years,
                        offset,
                        limit,
                        filter_id,
                        include_timeout_probe: true,
                    },
                    execution_mode: AimeExecutionMode::SharedProvider,
                })
                .await?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
                Ok(())
            }
            EvalCommand::AimeSanity {
                workspace,
                dataset,
                run_dir,
                model,
                timeout_ms,
                stream,
                include_timeout_probe,
                filter_id,
            } => {
                let workspace = workspace_or_current(workspace)?;
                let run_dir =
                    run_dir.unwrap_or_else(|| workspace.join(".scratch/aime-sanity/latest"));
                let summary = run_aime_sanity(AimeSanityOptions {
                    workspace,
                    dataset,
                    run_dir,
                    model,
                    timeout_ms,
                    stream,
                    include_timeout_probe,
                    filter_id,
                })
                .await?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
                Ok(())
            }
            EvalCommand::AimeReplay {
                from_run,
                workspace,
                run_dir,
                model,
                timeout_ms,
                stream,
                failed_only,
                filter_id,
            } => {
                let workspace = workspace_or_current(workspace)?;
                let run_dir = run_dir.unwrap_or_else(|| default_aime_replay_run_dir(&workspace));
                let summary = run_aime_replay(AimeReplayOptions {
                    workspace,
                    from_run,
                    run_dir,
                    model,
                    timeout_ms,
                    stream,
                    failed_only,
                    filter_id,
                })
                .await?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
                Ok(())
            }
            EvalCommand::AimeRegrade {
                from_run,
                workspace,
                run_dir,
            } => {
                let workspace = workspace_or_current(workspace)?;
                let run_dir = run_dir.unwrap_or_else(|| default_aime_regrade_run_dir(&workspace));
                let summary = run_aime_regrade(AimeRegradeOptions {
                    workspace,
                    from_run,
                    run_dir,
                })
                .await?;
                println!("{}", serde_json::to_string_pretty(&summary)?);
                Ok(())
            }
        },
        Command::Provider { command } => match command {
            ProviderCommand::Smoke { command } => match command {
                ProviderSmokeCommand::Traecli {
                    workspace,
                    model,
                    prompt,
                    timeout_ms,
                    stream,
                    supervise,
                } => {
                    let workspace = workspace_or_current(workspace)?;
                    let provider = TraeCliAcpProvider::new(workspace.clone(), model);
                    let command = provider.command.clone();
                    let args = provider.args();
                    let options = AcpOptions {
                        command,
                        args,
                        cwd: workspace,
                        model: provider.model,
                        prompt,
                        timeout_ms,
                    };
                    let supervisor_sink: Option<SupervisorSink> = if supervise {
                        Some(Arc::new(|snapshot| {
                            println!(
                                "supervisor: {}",
                                serde_json::to_string(&snapshot)
                                    .context("serialize supervisor snapshot")?
                            );
                            Ok(())
                        }))
                    } else {
                        None
                    };
                    let output = if stream {
                        let sink: ProviderEventSink = Arc::new(|event| {
                            Box::pin(async move {
                                if let ProviderEvent::AssistantDelta { delta } = event {
                                    println!("delta: {delta}");
                                }
                                Ok(())
                            })
                                as Pin<
                                    Box<
                                        dyn std::future::Future<Output = anyhow::Result<()>> + Send,
                                    >,
                                >
                        });
                        if supervise {
                            run_acp_prompt_observed(options, Some(sink), supervisor_sink).await?
                        } else {
                            run_acp_prompt_streaming(options, Some(sink)).await?
                        }
                    } else if supervise {
                        run_acp_prompt_observed(options, None, supervisor_sink).await?
                    } else {
                        run_acp_prompt(options).await?
                    };
                    println!("provider: traecli-acp");
                    println!("session_id: {}", output.session_id);
                    if let Some(stop_reason) = output.stop_reason {
                        println!("stop_reason: {stop_reason}");
                    }
                    if let Some(usage) = output.usage {
                        println!(
                            "usage: input={} output={} cached_read={} cached_write={}",
                            usage.input_tokens,
                            usage.output_tokens,
                            usage.cache_read_tokens,
                            usage.cache_write_tokens
                        );
                    }
                    println!("text: {}", output.text.trim());
                    Ok(())
                }
            },
            ProviderCommand::Supervise { command } => match command {
                ProviderSuperviseCommand::Traecli { pid } => {
                    let snapshot = ProcessSupervisor::system().snapshot(pid)?;
                    println!("{}", serde_json::to_string_pretty(&snapshot)?);
                    Ok(())
                }
            },
        },
        Command::Insula { command } => match command {
            InsulaCommand::Scan {
                store_root,
                workspace,
            } => {
                let workspace = workspace_or_current(workspace)?;
                for selection in InsulaSandbox::scan(&store_root, &workspace)? {
                    println!(
                        "{}\t{}\t{}",
                        selection.rootfs.display(),
                        selection.marker.digest,
                        selection.marker.node_path
                    );
                }
                Ok(())
            }
            InsulaCommand::Select {
                rootfs,
                store_root,
                workspace,
            } => {
                let workspace = workspace_or_current(workspace)?;
                let selection = InsulaSandbox::write_selection(&workspace, &rootfs, &store_root)?;
                println!("selected {}", selection.rootfs.display());
                Ok(())
            }
            InsulaCommand::Doctor { workspace, dry_run } => {
                let workspace = workspace_or_current(workspace)?;
                let sandbox = InsulaSandbox::load_selection(&workspace)?;
                for line in sandbox.doctor(dry_run)? {
                    println!("{line}");
                }
                Ok(())
            }
        },
    }
}

fn workspace_or_current(workspace: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    Ok(match workspace {
        Some(path) => path,
        None => std::env::current_dir()?,
    })
}

fn default_aime_years() -> Vec<String> {
    ["2024", "2025-I", "2025-II"]
        .iter()
        .map(|year| (*year).to_string())
        .collect()
}

fn default_aime_run_dir(workspace: &std::path::Path) -> PathBuf {
    let timestamp = time::OffsetDateTime::now_utc()
        .format(time::macros::format_description!(
            "[year][month][day]T[hour][minute][second][subsecond digits:3]Z"
        ))
        .expect("format UTC timestamp");
    workspace.join(format!(".scratch/aime-benchmark/run-{timestamp}"))
}

fn default_aime_replay_run_dir(workspace: &std::path::Path) -> PathBuf {
    let timestamp = time::OffsetDateTime::now_utc()
        .format(time::macros::format_description!(
            "[year][month][day]T[hour][minute][second][subsecond digits:3]Z"
        ))
        .expect("format UTC timestamp");
    workspace.join(format!(".scratch/aime-benchmark/replay-{timestamp}"))
}

fn default_aime_regrade_run_dir(workspace: &Path) -> PathBuf {
    let timestamp = time::OffsetDateTime::now_utc()
        .format(time::macros::format_description!(
            "[year][month][day]T[hour][minute][second][subsecond digits:3]Z"
        ))
        .expect("format UTC timestamp");
    workspace.join(format!(".scratch/aime-benchmark/regrade-{timestamp}"))
}

struct InsulaShellRunner {
    sandbox: InsulaSandbox,
}

#[async_trait]
impl ShellRunner for InsulaShellRunner {
    async fn run_shell(
        &self,
        command: String,
        cwd: PathBuf,
        timeout_ms: u64,
    ) -> anyhow::Result<SandboxResult> {
        self.sandbox.run(&command, &cwd, &[], timeout_ms).await
    }
}

struct PendingApproval {
    response: oneshot::Sender<ApprovalDecision>,
}

struct TuiApproval {
    requests: mpsc::UnboundedSender<PendingApproval>,
}

#[async_trait]
impl ApprovalResponder for TuiApproval {
    async fn request_approval(&self, request: ApprovalRequest) -> anyhow::Result<ApprovalDecision> {
        let _ = request;
        let (tx, rx) = oneshot::channel();
        if self
            .requests
            .send(PendingApproval { response: tx })
            .is_err()
        {
            return Ok(ApprovalDecision::Denied);
        }
        Ok(rx.await.unwrap_or(ApprovalDecision::Denied))
    }
}

async fn run_interactive(
    workspace: PathBuf,
    provider_kind: ProviderKind,
    model: Option<String>,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(&workspace).await?;
    let transcript = TranscriptStore::load(workspace.join(".corenius/transcript.jsonl")).await?;
    let (approval_tx, mut approval_rx) = mpsc::unbounded_channel();
    let sandbox = InsulaSandbox::load_selection(&workspace)?;
    let tools = ToolRegistry::default_for_workspace(
        &workspace,
        Arc::new(InsulaShellRunner {
            sandbox: sandbox.clone(),
        }),
    );
    let provider: Arc<dyn crate::core::ProviderSession> = match provider_kind {
        ProviderKind::Codex => Arc::new(CodexDirectProvider::new(
            "You are Corenius, a concise terminal coding agent. Use tools when useful.",
        )),
        ProviderKind::Traecli => Arc::new(TraeCliAcpProvider::new(workspace.clone(), model)),
    };
    let coordinator = Coordinator::new(
        provider,
        Arc::new(tools),
        PolicyGate::new(),
        transcript,
        Some(Arc::new(TuiApproval {
            requests: approval_tx,
        })),
    );
    let _terminal = TerminalGuard::enter()?;
    let mut state = TuiState::default();
    let mut input = String::new();
    let mut active: Option<ActiveTurn> = None;
    let mut approval_response: Option<oneshot::Sender<ApprovalDecision>> = None;
    loop {
        draw_tui(&state, &input)?;
        while let Ok(pending) = approval_rx.try_recv() {
            approval_response = Some(pending.response);
        }
        if let Some(turn) = &mut active {
            loop {
                match turn.events.try_recv() {
                    Ok(event) => state.apply(event),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        active = None;
                        break;
                    }
                }
            }
        }
        while let Ok(pending) = approval_rx.try_recv() {
            approval_response = Some(pending.response);
        }
        if let Some(key) = poll_key(Duration::from_millis(50)).await?
            && handle_key(
                key,
                &coordinator,
                &mut state,
                &mut input,
                &mut active,
                &mut approval_response,
            )
            .await?
        {
            break;
        }
    }
    Ok(())
}

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

async fn poll_key(timeout: Duration) -> anyhow::Result<Option<KeyEvent>> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<Option<KeyEvent>> {
        if event::poll(timeout)? {
            return Ok(match event::read()? {
                Event::Key(key) => Some(key),
                _ => None,
            });
        }
        Ok(None)
    })
    .await?
}

async fn handle_key(
    key: KeyEvent,
    coordinator: &Coordinator,
    state: &mut TuiState,
    input: &mut String,
    active: &mut Option<ActiveTurn>,
    approval_response: &mut Option<oneshot::Sender<ApprovalDecision>>,
) -> anyhow::Result<bool> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        if let Some(turn) = active.take() {
            turn.abort();
        }
        if let Some(response) = approval_response.take() {
            let _ = response.send(ApprovalDecision::Denied);
        }
        return Ok(true);
    }
    if state.approval.is_some() {
        if approval_response.is_none() {
            return Ok(false);
        }
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(response) = approval_response.take() {
                    let _ = response.send(ApprovalDecision::Approved);
                }
                state.approval = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                if let Some(response) = approval_response.take() {
                    let _ = response.send(ApprovalDecision::Denied);
                }
                state.approval = None;
            }
            _ => {}
        }
        return Ok(false);
    }
    match key.code {
        KeyCode::Enter => {
            let text = input.trim().to_string();
            input.clear();
            if text == "/quit" || text == "/exit" {
                return Ok(true);
            }
            if !text.is_empty() && active.is_none() {
                *active = Some(coordinator.start_user_turn(&text).await?);
            }
        }
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Char(c) if active.is_none() => {
            input.push(c);
        }
        KeyCode::Esc => return Ok(true),
        _ => {}
    }
    Ok(false)
}

fn draw_tui(state: &TuiState, input: &str) -> anyhow::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.draw(|frame| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Length(3),
            ])
            .split(frame.area());
        let transcript = state
            .transcript
            .iter()
            .rev()
            .take(14)
            .rev()
            .map(|(role, content)| ListItem::new(format!("{role}: {content}")))
            .collect::<Vec<_>>();
        frame.render_widget(
            List::new(transcript).block(Block::default().title("Transcript").borders(Borders::ALL)),
            layout[0],
        );
        let status = if state.status.is_empty() {
            "idle"
        } else {
            &state.status
        };
        frame.render_widget(
            Paragraph::new(format!(
                "status: {status}{}",
                state
                    .sandbox_status
                    .as_ref()
                    .map(|sandbox| format!(" | sandbox: {sandbox}"))
                    .unwrap_or_default()
            ))
            .block(Block::default().title("Status").borders(Borders::ALL)),
            layout[1],
        );
        frame.render_widget(
            Paragraph::new(render_rows(state).join("\n"))
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title("Tools / Approval")
                        .borders(Borders::ALL),
                ),
            layout[2],
        );
        let prompt = if state.approval.is_some() {
            "approval: press y to approve, n to deny".to_string()
        } else {
            format!("> {input}")
        };
        frame.render_widget(
            Paragraph::new(prompt).block(Block::default().title("Input").borders(Borders::ALL)),
            layout[3],
        );
    })?;
    Ok(())
}
