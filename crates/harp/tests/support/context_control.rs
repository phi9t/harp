use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

pub const PROCESS_CONTEXT: &str = "Harp process test context.";
pub const PROCESS_TASK: &str = "Capture the provider process exactly.";
pub const PROCESS_STDOUT: &[u8] = b"{\"event\":1,\"raw\":\"\x80\"}\n{\"event\":2}\n";
pub const PROCESS_STDERR: &[u8] = b"provider diagnostic \xff\n";
pub const PROCESS_FINAL_MESSAGE: &[u8] = b"final \xfe message\n";

#[derive(Clone, Copy, Debug)]
pub enum ProcessProviderBehavior {
    Complete { exit_code: u8 },
    CompleteWithoutFinalMessage,
    CompleteWithRepositoryEdit,
    CompleteWithRepositoryCommit,
    Overflow,
    WaitForTerm,
}

pub struct ProcessProviderFixture {
    _root: TempDir,
    executable: PathBuf,
    launches: PathBuf,
}

impl ProcessProviderFixture {
    pub fn new(behavior: ProcessProviderBehavior) -> Self {
        let root = TempDir::new().expect("process provider fixture root");
        let executable = root.path().join("traecli");
        let launches = root.path().join("launches");
        write_process_provider(&executable, &launches, behavior);
        Self {
            _root: root,
            executable,
            launches,
        }
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn launches(&self) -> &Path {
        &self.launches
    }
}

fn write_process_provider(path: &Path, launches: &Path, behavior: ProcessProviderBehavior) {
    let execution = match behavior {
        ProcessProviderBehavior::Complete { exit_code } => format!(
            r#"
printf '{{"event":1,"raw":"\200"}}\n{{"event":2}}\n'
printf 'provider diagnostic \377\n' >&2
printf 'final \376 message\n' > "$final_message"
exit {exit_code}
"#
        ),
        ProcessProviderBehavior::CompleteWithoutFinalMessage => r#"
printf '{"event":1,"raw":"\200"}\n{"event":2}\n'
printf 'provider diagnostic \377\n' >&2
exit 0
"#
        .to_owned(),
        ProcessProviderBehavior::CompleteWithRepositoryEdit => r#"
printf '{"event":1,"raw":"\200"}\n{"event":2}\n'
printf 'provider diagnostic \377\n' >&2
printf 'final \376 message\n' > "$final_message"
printf 'provider-created\n' > provider-created.txt
exit 0
"#
        .to_owned(),
        ProcessProviderBehavior::CompleteWithRepositoryCommit => r#"
printf '{"event":1,"raw":"\200"}\n{"event":2}\n'
printf 'provider diagnostic \377\n' >&2
printf 'final \376 message\n' > "$final_message"
printf 'provider-committed\n' >> tracked.txt
git add tracked.txt
git -c core.hooksPath=/dev/null commit --quiet -m 'provider commit'
exit 0
"#
        .to_owned(),
        ProcessProviderBehavior::Overflow => r#"
sleep 30 &
printf '%s\n' "$!" > "$final_message"
i=0
while [ "$i" -lt 4096 ]; do
  printf '0123456789abcdef0123456789abcdef'
  i=$((i + 1))
done
exit 0
"#
        .to_owned(),
        ProcessProviderBehavior::WaitForTerm => r#"
trap 'printf "provider trapped TERM\n" >&2; printf "term-trapped\n" > "$final_message"; exit 143' TERM
printf 'ready\n' > "$final_message"
printf 'provider ready\n' >&2
while :; do
  sleep 1
done
"#
        .to_owned(),
    };

    let script = format!(
        r#"#!/bin/sh
umask 077
case "$*" in
  "--version")
    printf '%s\n' 'traecli 0.200.19'
    exit 0
    ;;
  "exec --help")
    printf '%s\n' --json --output-last-message --cd --model --profile --sandbox --config
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
final_message=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --cd|--model|--profile|--sandbox|--config)
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
if [ -z "$final_message" ]; then
  printf 'missing final-message member\n' >&2
  exit 65
fi
prompt=$(cat)
case "$prompt" in
  *"{PROCESS_CONTEXT}"*"{PROCESS_TASK}"*) ;;
  *)
    printf 'prompt did not contain Harp context and original task\n' >&2
    exit 66
    ;;
esac
printf 'launched\n' >> {launches}
{execution}
"#,
        launches = shell_quote(launches),
    );
    fs::write(path, script).expect("write process provider fixture");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .expect("make process provider fixture executable");
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\"'\"'"))
}
