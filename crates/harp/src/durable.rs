use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use harp::AppError;

fn repository_root(start: &Path) -> Result<PathBuf, AppError> {
    for root in start.ancestors() {
        if !root.join("Cargo.toml").is_file() {
            continue;
        }
        let package = root.join("durable");
        let files = [
            package.clone(),
            package.join("__init__.py"),
            package.join("__main__.py"),
        ];
        if files
            .iter()
            .all(|path| fs::symlink_metadata(path).is_ok_and(|info| !info.file_type().is_symlink()))
            && package.is_dir()
            && package.join("__init__.py").is_file()
            && package.join("__main__.py").is_file()
        {
            return fs::canonicalize(root)
                .map_err(|error| AppError::io("durable.root", "resolve repository root", error));
        }
    }
    Err(AppError::invalid_input(
        "durable.root",
        "run durable from a Harp checkout containing the durable package",
    ))
}

/// Replace this CLI process so signals, streams and exit status belong to the runtime.
pub fn exec(format: &str, args: &[OsString]) -> AppError {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => return AppError::io("durable.root", "read current directory", error),
    };
    let root = match repository_root(&cwd) {
        Ok(root) => root,
        Err(error) => return error,
    };
    let python_path = match env::join_paths([root.clone()]) {
        Ok(value) => value,
        Err(error) => return AppError::invalid_input("durable.root", error.to_string()),
    };
    let error = Command::new("python3")
        .args(["-m", "durable", "--format", format])
        .args(args)
        .current_dir(&root)
        .env("PYTHONPATH", python_path)
        .env("PYTHONNOUSERSITE", "1")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env_remove("PYTHONHOME")
        .env_remove("PYTHONSTARTUP")
        .env_remove("PYTHONINSPECT")
        .exec();
    AppError::io("durable.launch", "execute python3 durable runtime", error)
}
