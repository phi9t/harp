use clap::{Parser, Subcommand};
use insula_supervisor::{
    owner_rpc::OwnerRpc,
    transport::{self, ControlAction, ControlReply, ControlServer},
    *,
};
use std::{io::Read, path::PathBuf, sync::Arc, time::Duration};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    version,
    about = "Observe and recover declared services through the authoritative Insula owner"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(long)]
        owner_socket: PathBuf,
        #[arg(long)]
        owner_uid: Option<u32>,
        #[arg(long)]
        root_id: String,
        #[arg(long)]
        state_dir: PathBuf,
    },
    Start {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        operation_id: Uuid,
    },
    Stop {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        insula: String,
        #[arg(long)]
        service: String,
        #[arg(long)]
        generation: Uuid,
        #[arg(long)]
        operation_id: Uuid,
    },
    Restart {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        insula: String,
        #[arg(long)]
        service: String,
        #[arg(long)]
        generation: Uuid,
        #[arg(long)]
        operation_id: Uuid,
    },
    Inspect {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        insula: String,
        #[arg(long)]
        service: String,
    },
    List {
        #[arg(long)]
        socket: PathBuf,
    },
    Operation {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long)]
        operation_id: Uuid,
    },
    WatchEvents {
        #[arg(long)]
        socket: PathBuf,
        #[arg(long, default_value_t = 0)]
        after: u64,
    },
    Capabilities {
        #[arg(long)]
        socket: PathBuf,
    },
    NewId,
}
#[tokio::main]
async fn main() {
    if let Err(e) = run(Cli::parse()).await {
        eprintln!(
            "{}",
            serde_json::to_string(&e).unwrap_or_else(|_| "\"error\"".into())
        );
        std::process::exit(1)
    }
}
async fn run(cli: Cli) -> Result<()> {
    let uid = rustix::process::geteuid().as_raw();
    let (socket, action) = match cli.command {
        Command::Serve {
            owner_socket,
            owner_uid,
            root_id,
            state_dir,
        } => return serve(owner_socket, owner_uid.unwrap_or(uid), root_id, state_dir).await,
        Command::NewId => {
            println!("{}", Uuid::new_v4());
            return Ok(());
        }
        Command::Start {
            socket,
            spec,
            operation_id,
        } => {
            let mut bytes = Vec::new();
            std::fs::File::open(spec)?
                .take(512 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() > 512 * 1024 {
                return Err(Error::Invalid("service declaration exceeds 512 KiB".into()));
            }
            let spec: ServiceSpec = serde_json::from_slice(&bytes)?;
            spec.validate()?;
            (
                socket,
                ControlAction::Mutate {
                    request: OperationRequest {
                        operation_id,
                        mutation: Mutation::Start {
                            spec: Box::new(spec),
                        },
                    },
                },
            )
        }
        Command::Stop {
            socket,
            insula,
            service,
            generation,
            operation_id,
        } => (
            socket,
            ControlAction::Mutate {
                request: OperationRequest {
                    operation_id,
                    mutation: Mutation::Stop {
                        key: ServiceKey {
                            insula_id: insula,
                            service_id: service,
                        },
                        expected_generation: generation,
                    },
                },
            },
        ),
        Command::Restart {
            socket,
            insula,
            service,
            generation,
            operation_id,
        } => (
            socket,
            ControlAction::Mutate {
                request: OperationRequest {
                    operation_id,
                    mutation: Mutation::Restart {
                        key: ServiceKey {
                            insula_id: insula,
                            service_id: service,
                        },
                        expected_generation: generation,
                    },
                },
            },
        ),
        Command::Inspect {
            socket,
            insula,
            service,
        } => (
            socket,
            ControlAction::Inspect {
                key: ServiceKey {
                    insula_id: insula,
                    service_id: service,
                },
            },
        ),
        Command::List { socket } => (socket, ControlAction::List),
        Command::Operation {
            socket,
            operation_id,
        } => (socket, ControlAction::Operation { operation_id }),
        Command::Capabilities { socket } => (socket, ControlAction::Capabilities),
        Command::WatchEvents { socket, mut after } => loop {
            let reply = transport::request(
                &socket,
                uid,
                ControlAction::WatchEvents {
                    after,
                    limit: 128,
                    wait_ms: 5000,
                },
                Duration::from_secs(8),
            )
            .await?;
            if let ControlReply::Events {
                events,
                history_gap,
            } = reply
            {
                if history_gap {
                    eprintln!("event history gap; inspect current status before interpreting older events")
                }
                for event in events {
                    after = event.sequence;
                    println!("{}", serde_json::to_string(&event)?)
                }
            }
        },
    };
    let reply = transport::request(&socket, uid, action, Duration::from_secs(65)).await?;
    println!("{}", serde_json::to_string_pretty(&reply)?);
    Ok(())
}
async fn serve(owner_socket: PathBuf, uid: u32, root_id: String, state_dir: PathBuf) -> Result<()> {
    let owner = Arc::new(OwnerRpc::new(owner_socket, uid, Duration::from_secs(65))?);
    let supervisor = Supervisor::open(owner, root_id, state_dir).await?;
    let control = ControlServer::bind(&supervisor)?;
    let (closing_tx, closing_rx) = tokio::sync::watch::channel(false);
    let server_supervisor = supervisor.clone();
    let mut server = tokio::spawn(control.run(server_supervisor, closing_rx));
    let ticking = supervisor.clone();
    let mut monitor = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(250));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if ticking.is_shutting_down() {
                return Ok(());
            }
            ticking.tick().await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), Error>(())
    });
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let service_result = tokio::select! {
        result=tokio::signal::ctrl_c()=>result.map_err(|_|Error::Io),
        _=terminate.recv()=>Ok(()),
        result=&mut server=>result.map_err(|_|Error::Unknown)?,
        result=&mut monitor=>result.map_err(|_|Error::Unknown)?,
    };
    let _ = closing_tx.send(true);
    let stop_result = supervisor.shutdown().await;
    // The monitoring loop contains observation only; all accepted mutations
    // were drained by shutdown's admission barrier.
    monitor.abort();
    server.abort();
    service_result.and(stop_result)
}
