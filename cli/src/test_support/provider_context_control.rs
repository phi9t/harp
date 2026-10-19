use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub struct ProviderFixtures {
    _root: TempDir,
    bin: PathBuf,
}

impl ProviderFixtures {
    pub fn supported() -> Self {
        let root = TempDir::new().expect("provider fixture root");
        let bin = root.path().join("bin");
        fs::create_dir(&bin).expect("provider fixture bin");
        write_provider(&bin, "traecli", "traecli 0.200.19");
        write_provider(&bin, "codex", "codex-cli 0.144.5");
        Self { _root: root, bin }
    }

    pub fn path(&self) -> &Path {
        &self.bin
    }
}

fn write_provider(bin: &Path, name: &str, version: &str) {
    let path = bin.join(name);
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
case "$*" in
  "--version")
    printf '%s\n' '{version}'
    ;;
  "exec --help")
    printf '%s\n' --json --output-last-message --cd --model --profile --sandbox --config
    ;;
  "exec resume --help")
    printf '%s\n' --json
    ;;
  "app-server generate-json-schema --help")
    printf '%s\n' --out
    ;;
  *)
    printf 'unexpected arguments: %s\n' "$*" >&2
    exit 64
    ;;
esac
"#
        ),
    )
    .expect("write provider fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
        .expect("make provider fixture executable");
}
