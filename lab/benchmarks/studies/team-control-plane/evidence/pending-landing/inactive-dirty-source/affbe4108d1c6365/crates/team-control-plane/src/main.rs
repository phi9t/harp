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
            let listener = tokio::net::TcpListener::bind(address)
                .await
                .map_err(|_| Error {
                    code: ErrorCode::TemporarilyUnavailable,
                })?;
            axum::serve(listener, team_control_plane::http::router(b))
                .await
                .map_err(|_| Error {
                    code: ErrorCode::TemporarilyUnavailable,
                })
        }
        _ => Err(invalid()),
    }
}
