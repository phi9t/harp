#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Retained invocation and evidence contracts also support the test-only legacy executor."
    )
)]
mod invocation;
mod probe;
#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod process;

pub use invocation::{
    build_invocation, encode_provider_command, ApprovalPolicy, ProviderArgumentPlan,
    ProviderCapabilitySnapshot, ProviderInvocation, ProviderRunOptions, SandboxMode,
};
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use invocation::{
    BoundProviderInvocation, EpisodeProviderRawPath, EpisodeRawDirectoryAuthority,
    MaterializedProviderInvocation, SealedProviderEvidence,
};
pub use probe::{locate, probe, ProviderCapabilities};
#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
pub(crate) use process::execute;

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderProcessResult {
    pub exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
    pub capture_complete: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub final_message_sha256: Option<String>,
    pub raw_archive_sha256: String,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Debug)]
pub struct ProviderExecution {
    pub result: ProviderProcessResult,
    pub evidence: SealedProviderEvidence,
}

use std::path::Path;

use crate::context_control::ProviderId;
use crate::AppError;

pub fn probe_snapshot(
    provider: ProviderId,
    path: &Path,
) -> Result<ProviderCapabilitySnapshot, AppError> {
    let identity = ProviderCapabilitySnapshot::capture_executable_identity(provider, path)?;
    ProviderCapabilitySnapshot::from_probed(&probe::probe(provider, path)?, identity)
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::Path;

    use tempfile::TempDir;

    use super::probe_snapshot;
    use crate::context_control::ProviderId;

    #[test]
    fn probe_snapshot_rejects_executable_replacement_during_probe() {
        let root = TempDir::new().unwrap();
        let executable = root.path().join("traecli");
        let replacement = root.path().join("replacement");
        let replaced = root.path().join("replaced");
        write_executable(
            &replacement,
            "#!/bin/sh\ncase \"$*\" in\n\
             \"--version\") printf '%s\n' 'traecli 1.2.3' ;;\n\
             \"exec --help\") printf '%s\n' --json --output-last-message --cd --config ;;\n\
             *) exit 0 ;;\n\
             esac\n",
        );
        write_executable(
            &executable,
            &format!(
                "#!/bin/sh\ncase \"$*\" in\n\
                 \"--version\") mv '{}' '{}'; mv '{}' '{}'; printf '%s\n' 'traecli 1.2.3' ;;\n\
                 \"exec --help\") printf '%s\n' --json --output-last-message --cd --config ;;\n\
                 *) exit 0 ;;\n\
                 esac\n",
                executable.display(),
                replaced.display(),
                replacement.display(),
                executable.display(),
            ),
        );

        assert_eq!(
            probe_snapshot(ProviderId::Trae, &executable)
                .unwrap_err()
                .code(),
            "provider.executable_changed"
        );
    }

    fn write_executable(path: &Path, script: &str) {
        fs::write(path, script).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
}
