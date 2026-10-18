fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err(
            "usage: corenius-attempt-supervisor CONFIG SOCKET; CORENIUS_SUPERVISOR_TOKEN required"
                .into(),
        );
    }
    let token = std::env::var("CORENIUS_SUPERVISOR_TOKEN")
        .map_err(|_| "CORENIUS_SUPERVISOR_TOKEN required")?;
    match std::env::var("CORENIUS_GIT_CONFIG") {
        Ok(path) => {
            let backend = corenius_sqlite_controller::git_workspace::GitBackend::open(
                std::path::Path::new(&path),
            )?;
            let preparer =
                corenius_sqlite_controller::git_workspace::TrustedWorkspacePreparer(backend);
            corenius_sqlite_controller::supervisor::serve_with_workspace(
                &args[1], &args[2], &token, &preparer,
            )
        }
        Err(std::env::VarError::NotPresent) => {
            corenius_sqlite_controller::supervisor::serve(&args[1], &args[2], &token)
        }
        Err(_) => Err("invalid Git configuration path".into()),
    }
}
