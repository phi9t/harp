use corenius_sqlite_controller::{Controller, codex::CodexRuntime, runtime::Runtime, service};
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).is_some_and(|a| a == "--native-ui") {
        return corenius_sqlite_controller::ui_gateway::run(&args);
    }
    if args
        .get(1)
        .is_some_and(|a| matches!(a.as_str(), "--dashboard" | "--control" | "--mcp"))
    {
        return corenius_sqlite_controller::client::run(&args);
    }
    if args.len() != 3 {
        return Err(
            "usage: corenius-sqlite-controller DATABASE SOCKET; CORENIUS_OPERATOR_TOKEN required"
                .into(),
        );
    }
    let token =
        std::env::var("CORENIUS_OPERATOR_TOKEN").map_err(|_| "CORENIUS_OPERATOR_TOKEN required")?;
    let runtime = match (
        std::env::var("CORENIUS_SUPERVISOR_SOCKET"),
        std::env::var("CORENIUS_SUPERVISOR_TOKEN"),
    ) {
        (Ok(socket), Ok(supervisor_token)) => Some(Runtime::new(
            socket.into(),
            supervisor_token,
            token.clone(),
        )?),
        (Err(std::env::VarError::NotPresent), Err(std::env::VarError::NotPresent)) => None,
        _ => return Err("both supervisor socket and secret required".into()),
    };
    if runtime.is_some() {
        for path in [&args[1], &args[2]] {
            let configured = std::path::Path::new(path);
            if configured.exists() || std::fs::symlink_metadata(configured).is_ok() {
                let target = configured.canonicalize().map_err(|e| e.to_string())?;
                if target.starts_with("/usr") {
                    return Err(
                        "resolved controller storage/socket is inside worker toolchain mount"
                            .into(),
                    );
                }
            }
            let parent = std::path::Path::new(path)
                .parent()
                .ok_or("private controller directory required")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if parent.starts_with("/usr") {
                return Err("controller storage/socket must be outside the toolchain mount".into());
            }
        }
    }
    let controller = Controller::open(&args[1], &token)?;
    match std::env::var("CORENIUS_GIT_CONFIG") {
        Ok(path) => controller.configure_git(std::path::Path::new(&path))?,
        Err(std::env::VarError::NotPresent) if controller.git_requires_configuration()? => {
            return Err("durable Git policy requires CORENIUS_GIT_CONFIG".into());
        }
        Err(std::env::VarError::NotPresent) => {}
        Err(_) => return Err("invalid Git configuration path".into()),
    }
    match std::env::var("CORENIUS_KATA_CONFIG") {
        Ok(path) => controller.configure_kata(std::path::Path::new(&path))?,
        Err(std::env::VarError::NotPresent) => {
            if corenius_sqlite_controller::kata::requires_configuration(&controller)? {
                return Err("durable Kata bindings require CORENIUS_KATA_CONFIG".into());
            }
        }
        Err(_) => return Err("invalid Kata configuration path".into()),
    }
    let codex = match std::env::var("CORENIUS_CODEX_CONFIG") {
        Ok(path) => Some(CodexRuntime::open(
            std::path::Path::new(&path),
            &controller,
        )?),
        Err(std::env::VarError::NotPresent) => {
            if CodexRuntime::requires_configuration(&controller)? {
                return Err("durable native registry requires CORENIUS_CODEX_CONFIG".into());
            }
            None
        }
        Err(_) => return Err("invalid Codex configuration path".into()),
    };
    service::serve_with_codex(controller, &args[2], runtime, codex)
}
