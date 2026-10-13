use team_control_plane::{Broker, Error, ErrorCode, migrate};
use uuid::Uuid;
#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!(
            "{}",
            serde_json::to_string(&error).unwrap_or_else(|_| "operation failed".into())
        );
        std::process::exit(1);
    }
}
async fn run() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    let invalid = || Error {
        code: ErrorCode::InvalidArgument,
    };
    let dsn = std::env::var("TEAM_CONTROL_PLANE_DSN").map_err(|_| invalid())?;
    match args.get(1).map(String::as_str) {
        Some("migrate") => migrate(&dsn).await,
        Some("supervisor-monitor") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let id = Uuid::parse_str(args.get(2).ok_or_else(invalid)?).map_err(|_| invalid())?;
            team_control_plane::supervisor::run_monitor(Broker::connect(&dsn, epoch).await?, id)
                .await
        }
        Some("bootstrap") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let b = Broker::connect(&dsn, epoch).await?;
            println!("{}", b.bootstrap_owner().await?);
            Ok(())
        }
        Some("rotate-epoch") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let new = Uuid::parse_str(args.get(2).ok_or_else(invalid)?).map_err(|_| invalid())?;
            let token = std::env::var("TEAM_CONTROL_PLANE_OWNER_TOKEN").map_err(|_| invalid())?;
            let b = Broker::connect(&dsn, epoch).await?;
            println!("{}", b.rotate_epoch(&token, new).await?);
            Ok(())
        }
        Some("takeover-episode-store") => {
            if args.len() != 7 {
                return Err(invalid());
            }
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let request = team_control_plane::kernel::TakeoverRequest {
                operation: Uuid::parse_str(&args[2]).map_err(|_| invalid())?,
                replacement: Uuid::parse_str(&args[3]).map_err(|_| invalid())?,
                expected_controller: Uuid::parse_str(&args[4]).map_err(|_| invalid())?,
                key: args[5].clone(),
                expected_control_revision: args[6].parse().map_err(|_| invalid())?,
            };
            let result =
                team_control_plane::kernel::Kernel::new(Broker::connect(&dsn, epoch).await?)
                    .takeover(&bearer, request)
                    .await?;
            println!("{result}");
            Ok(())
        }
        Some("reconcile-gateway-usage") => {
            if args.len() != 3 {
                return Err(invalid());
            }
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let id = Uuid::parse_str(&args[2]).map_err(|_| invalid())?;
            let result =
                team_control_plane::gateway::Gateway::new(Broker::connect(&dsn, epoch).await?)
                    .reconcile_usage(&bearer, id)
                    .await?;
            println!("{}", serde_json::to_value(result).map_err(|_| invalid())?);
            Ok(())
        }
        Some(command @ ("inspect-episode-operation" | "reconcile-episode-operation")) => {
            if args.len() != 3 {
                return Err(invalid());
            }
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let id = Uuid::parse_str(&args[2]).map_err(|_| invalid())?;
            let gateway = team_control_plane::episode_runtime::EpisodeGateway::new(
                Broker::connect(&dsn, epoch).await?,
            );
            let result = if command == "inspect-episode-operation" {
                serde_json::to_value(gateway.inspect(&bearer, id).await?).map_err(|_| invalid())?
            } else {
                serde_json::to_value(gateway.reconcile(&bearer, id).await?)
                    .map_err(|_| invalid())?
            };
            println!("{result}");
            Ok(())
        }
        Some("prepare-episode-start") => {
            if args.len() != 3 || args[2].len() > 65536 {
                return Err(invalid());
            }
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let request = serde_json::from_str(&args[2]).map_err(|_| invalid())?;
            let gateway = team_control_plane::episode_runtime::EpisodeGateway::new(
                Broker::connect(&dsn, epoch).await?,
            );
            println!(
                "{}",
                serde_json::to_string(&gateway.prepare(&bearer, request).await?)
                    .map_err(|_| invalid())?
            );
            Ok(())
        }
        Some(
            command @ ("inspect-execution"
            | "cancel-execution"
            | "reconcile-execution"
            | "release-execution"),
        ) => {
            if args.len() != 3 {
                return Err(invalid());
            }
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let id = Uuid::parse_str(&args[2]).map_err(|_| invalid())?;
            let broker = Broker::connect(&dsn, epoch).await?;
            team_control_plane::kernel::Kernel::new(broker.clone())
                .status(&bearer)
                .await?;
            let root = std::path::PathBuf::from(
                std::env::var_os("TEAM_CONTROL_PLANE_WORKSPACE_ROOT").ok_or_else(invalid)?,
            );
            let monitor = std::env::current_exe().map_err(|_| invalid())?;
            let supervisor = team_control_plane::supervisor::Supervisor::open(
                broker,
                &root,
                monitor.to_str().ok_or_else(invalid)?,
            )
            .await?;
            let result = match command {
                "inspect-execution" => serde_json::to_value(supervisor.inspect(&bearer, id).await?)
                    .map_err(|_| invalid())?,
                "cancel-execution" => {
                    supervisor.cancel(&bearer, id).await?;
                    serde_json::to_value(supervisor.inspect(&bearer, id).await?)
                        .map_err(|_| invalid())?
                }
                "reconcile-execution" => {
                    serde_json::to_value(supervisor.reconcile(&bearer, id).await?)
                        .map_err(|_| invalid())?
                }
                "release-execution" => {
                    supervisor.release(&bearer, id).await?;
                    serde_json::to_value(supervisor.inspect(&bearer, id).await?)
                        .map_err(|_| invalid())?
                }
                _ => return Err(invalid()),
            };
            println!("{result}");
            Ok(())
        }
        Some("start-execution") => {
            if args.len() != 5 {
                return Err(invalid());
            }
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let id = Uuid::parse_str(&args[2]).map_err(|_| invalid())?;
            let seconds = args[3].parse().map_err(|_| invalid())?;
            if args[4].len() > 65536 {
                return Err(invalid());
            }
            let argv: Vec<String> = serde_json::from_str(&args[4]).map_err(|_| invalid())?;
            let broker = Broker::connect(&dsn, epoch).await?;
            team_control_plane::kernel::Kernel::new(broker.clone())
                .status(&bearer)
                .await?;
            let root = std::path::PathBuf::from(
                std::env::var_os("TEAM_CONTROL_PLANE_WORKSPACE_ROOT").ok_or_else(invalid)?,
            );
            let monitor = std::env::current_exe().map_err(|_| invalid())?;
            let supervisor = team_control_plane::supervisor::Supervisor::open(
                broker,
                &root,
                monitor.to_str().ok_or_else(invalid)?,
            )
            .await?;
            supervisor.start(&bearer, id, argv, seconds).await?;
            println!("{}", serde_json::to_string(&()).map_err(|_| invalid())?);
            Ok(())
        }
        Some("allocate-episode") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let root = std::path::PathBuf::from(
                std::env::var_os("TEAM_CONTROL_PLANE_WORKSPACE_ROOT").ok_or_else(invalid)?,
            );
            let id = Uuid::parse_str(args.get(2).ok_or_else(invalid)?).map_err(|_| invalid())?;
            let monitor = std::env::current_exe().map_err(|_| invalid())?;
            let broker = Broker::connect(&dsn, epoch).await?;
            team_control_plane::kernel::Kernel::new(broker.clone())
                .status(&bearer)
                .await?;
            let supervisor = team_control_plane::supervisor::Supervisor::open(
                broker,
                &root,
                monitor.to_str().ok_or_else(invalid)?,
            )
            .await?;
            let handle = supervisor
                .create_episode(
                    &bearer,
                    id,
                    vec![],
                    team_control_plane::supervisor::CreateFault::None,
                )
                .await?;
            println!("{}", serde_json::to_string(&handle).map_err(|_| invalid())?);
            Ok(())
        }
        Some("pause") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let mission =
                Uuid::parse_str(args.get(2).ok_or_else(invalid)?).map_err(|_| invalid())?;
            let key = args.get(3).ok_or_else(invalid)?;
            let revision = args
                .get(4)
                .ok_or_else(invalid)?
                .parse()
                .map_err(|_| invalid())?;
            let paused = args
                .get(5)
                .ok_or_else(invalid)?
                .parse()
                .map_err(|_| invalid())?;
            let rationale = args.get(6).ok_or_else(invalid)?;
            let result =
                team_control_plane::kernel::Kernel::new(Broker::connect(&dsn, epoch).await?)
                    .pause(&bearer, mission, key, revision, paused, rationale)
                    .await?;
            println!("{}", result);
            Ok(())
        }
        Some("status") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let bearer = std::env::var("TEAM_CONTROL_PLANE_TOKEN").map_err(|_| invalid())?;
            let broker = Broker::connect(&dsn, epoch).await?;
            let view = team_control_plane::kernel::Kernel::new(broker)
                .status(&bearer)
                .await?;
            println!("{}", view);
            Ok(())
        }
        Some("serve") => {
            let epoch = Uuid::parse_str(
                &std::env::var("TEAM_CONTROL_PLANE_RECOVERY_EPOCH").map_err(|_| invalid())?,
            )
            .map_err(|_| invalid())?;
            let b = Broker::connect(&dsn, epoch).await?;
            let bind = args.get(2).map(String::as_str).unwrap_or("127.0.0.1:8080");
            let address: std::net::SocketAddr = bind.parse().map_err(|_| invalid())?;
            if !address.ip().is_loopback() {
                return Err(invalid());
            }
            let router = if let Some(root) = std::env::var_os("TEAM_CONTROL_PLANE_ARTIFACT_ROOT") {
                let root = std::path::PathBuf::from(root);
                if !root.is_absolute() {
                    return Err(invalid());
                }
                let artifacts =
                    team_control_plane::artifacts::ArtifactService::open(b.clone(), &root).await?;
                team_control_plane::http::router_with_artifacts(b, artifacts)
            } else {
                team_control_plane::http::router(b)
            };
            let listener = tokio::net::TcpListener::bind(address)
                .await
                .map_err(|_| Error {
                    code: ErrorCode::TemporarilyUnavailable,
                })?;
            axum::serve(listener, router).await.map_err(|_| Error {
                code: ErrorCode::TemporarilyUnavailable,
            })
        }
        _ => Err(invalid()),
    }
}
