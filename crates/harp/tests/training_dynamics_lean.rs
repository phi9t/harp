use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::TempDir;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

#[test]
fn missing_lake_is_actionable() {
    let empty_path = TempDir::new().expect("empty PATH directory");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_training_dynamics_lean.sh")
        .env("PATH", empty_path.path())
        .assert()
        .failure()
        .stderr("Training Dynamics Lean toolchain is unavailable\n");
}
