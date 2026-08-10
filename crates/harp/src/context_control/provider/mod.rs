mod invocation;
mod probe;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod process;

pub(crate) use invocation::build_invocation_from_snapshot;
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
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use process::{execute, ProviderExecution, ProviderProcessResult};

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
