use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

pub struct RunProviderFixtures {
    _root: TempDir,
    bin: PathBuf,
    capture: PathBuf,
}

impl RunProviderFixtures {
    pub fn supported() -> Self {
        Self::new(None)
    }

    pub fn missing_required_capability(provider: &str) -> Self {
        Self::new(Some(provider))
    }

    fn new(missing_required_capability: Option<&str>) -> Self {
        let root = TempDir::new().expect("run provider fixture root");
        let bin = root.path().join("bin");
        let capture = root.path().join("capture");
        fs::create_dir(&bin).expect("run provider fixture bin");
        fs::create_dir(&capture).expect("run provider fixture capture");
        write_run_provider(
            &bin,
            "traecli",
            "trae",
            "traecli 0.200.19",
            missing_required_capability == Some("trae"),
        );
        write_run_provider(
            &bin,
            "codex",
            "codex",
            "codex-cli 0.144.5",
            missing_required_capability == Some("codex"),
        );
        Self {
            _root: root,
            bin,
            capture,
        }
    }

    pub fn path_env(&self) -> String {
        let ambient = std::env::var("PATH").expect("test PATH");
        format!("{}:{ambient}", self.bin.display())
    }

    pub fn capture(&self) -> &Path {
        &self.capture
    }

    pub fn prompt(&self, provider: &str) -> Vec<u8> {
        fs::read(self.capture.join(format!("{provider}.prompt"))).expect("captured provider prompt")
    }

    pub fn launches(&self) -> Vec<String> {
        fs::read_to_string(self.capture.join("launches"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    pub fn invocations(&self) -> Vec<String> {
        fs::read_to_string(self.capture.join("invocations"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

pub struct RunRepository {
    _root: TempDir,
    repository: PathBuf,
    state: PathBuf,
}

impl RunRepository {
    pub fn new() -> Self {
        let root = TempDir::new().expect("run repository root");
        let canonical = fs::canonicalize(root.path()).expect("canonical run repository root");
        let repository = canonical.join("repository");
        let state = canonical.join("state");
        fs::create_dir(&repository).expect("create run repository");
        git(&repository, &["init", "-q"]);
        git(
            &repository,
            &["config", "user.email", "harp@example.invalid"],
        );
        git(&repository, &["config", "user.name", "Harp Test"]);
        fs::write(repository.join("tracked.txt"), "initial\n").expect("seed repository");
        git(&repository, &["add", "tracked.txt"]);
        git(&repository, &["commit", "-q", "-m", "initial"]);
        Self {
            _root: root,
            repository,
            state,
        }
    }

    pub fn path(&self) -> &Path {
        &self.repository
    }

    pub fn state(&self) -> &Path {
        &self.state
    }

    pub fn write_policy(&self, policy: serde_json::Value) {
        let directory = self.repository.join(".harp");
        fs::create_dir_all(&directory).expect("create policy directory");
        fs::write(
            directory.join("context-control.json"),
            serde_json::to_vec_pretty(&policy).expect("serialize repository policy"),
        )
        .expect("write repository policy");
    }

    pub fn repository_state_directory(&self) -> PathBuf {
        let repositories = self.state.join("repositories");
        let entries = fs::read_dir(&repositories)
            .unwrap_or_else(|error| {
                panic!(
                    "read repository state directory {}: {error}",
                    repositories.display()
                )
            })
            .map(|entry| entry.expect("repository state entry").path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1, "one repository state namespace");
        entries[0].clone()
    }
}

fn write_run_provider(
    bin: &Path,
    executable: &str,
    provider: &str,
    version: &str,
    missing_required_capability: bool,
) {
    let path = bin.join(executable);
    let exec_help = if missing_required_capability {
        "--json --cd --model --profile --sandbox --config"
    } else {
        "--json --output-last-message --cd --model --profile --sandbox --config"
    };
    let script = format!(
        r#"#!/bin/sh
umask 077
printf '%s	%s\n' '{provider}' "$*" >> "$HARP_TEST_CAPTURE/invocations"
case "$*" in
  "--version")
    printf '%s\n' '{version}'
    exit 0
    ;;
  "exec --help")
    printf '%s\n' {exec_help}
    exit 0
    ;;
  "exec resume --help")
    printf '%s\n' --json
    exit 0
    ;;
  "app-server generate-json-schema --help")
    printf '%s\n' --out
    exit 0
    ;;
esac

if [ "$1" != "exec" ]; then
  printf 'unexpected provider command: %s\n' "$*" >&2
  exit 64
fi
shift
repository=
final_message=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --cd)
      repository=$2
      shift 2
      ;;
    --model|--profile|--sandbox|--config)
      shift 2
      ;;
    --output-last-message)
      final_message=$2
      shift 2
      ;;
    --json|-)
      shift
      ;;
    *)
      printf 'unexpected execution argument: %s\n' "$1" >&2
      exit 64
      ;;
  esac
done

if [ -z "$repository" ] || [ -z "$final_message" ]; then
  printf 'missing repository or final-message path\n' >&2
  exit 65
fi
episode_directory=$(dirname "$(dirname "$(dirname "$final_message")")")
if [ ! -f "$episode_directory/manifest.json" ] || [ ! -f "$episode_directory/context.json" ]; then
  printf 'episode manifest and context were not published before launch\n' >&2
  exit 66
fi
cat > "$HARP_TEST_CAPTURE/{provider}.prompt"
printf '%s\n' '{provider}' >> "$HARP_TEST_CAPTURE/launches"
printf '{{"provider":"%s","event":"complete"}}\n' '{provider}'
printf 'provider=%s lifecycle=complete\n' '{provider}' >&2
printf 'final provider=%s\n' '{provider}' > "$final_message"
exit "${{HARP_TEST_EXIT_CODE:-0}}"
"#
    );
    fs::write(&path, script).expect("write run provider fixture");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
        .expect("make run provider fixture executable");
}

fn git(repository: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .current_dir(repository)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .args(arguments)
        .status()
        .expect("run Git fixture command");
    assert!(status.success(), "git {} failed", arguments.join(" "));
}
