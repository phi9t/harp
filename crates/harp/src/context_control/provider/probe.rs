use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use tempfile::NamedTempFile;
use wait_timeout::ChildExt;

use crate::context_control::{ProviderId, PROVIDER_CAPABILITIES_SCHEMA};
use crate::AppError;

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const OUTPUT_POLL_INTERVAL: Duration = Duration::from_millis(10);
const TERMINATION_GRACE: Duration = Duration::from_millis(100);
const MAX_OUTPUT_BYTES: u64 = 1024 * 1024;
const MAX_VERSION_BYTES: usize = 128;
const MAX_VERSION_IDENTIFIER_BYTES: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCapabilities {
    pub schema_version: String,
    pub provider: ProviderId,
    pub executable: PathBuf,
    pub version: String,
    pub exec_json: bool,
    pub output_last_message: bool,
    pub working_directory: bool,
    pub model: bool,
    pub profile: bool,
    pub sandbox: bool,
    pub approval_config: bool,
    pub resume_json: bool,
    pub app_server_schema: bool,
    pub capability_sha256: String,
}

#[derive(Serialize)]
struct CapabilityDigest<'a> {
    schema_version: &'a str,
    provider: ProviderId,
    executable: &'static str,
    version: &'a str,
    exec_json: bool,
    output_last_message: bool,
    working_directory: bool,
    model: bool,
    profile: bool,
    sandbox: bool,
    approval_config: bool,
    resume_json: bool,
    app_server_schema: bool,
}

#[derive(Debug)]
struct ProbeOutput {
    stdout: Vec<u8>,
}

pub fn probe(provider: ProviderId, path: &Path) -> Result<ProviderCapabilities, AppError> {
    probe_with_timeout(provider, path, PROBE_TIMEOUT)
}

fn probe_with_timeout(
    provider: ProviderId,
    path: &Path,
    timeout: Duration,
) -> Result<ProviderCapabilities, AppError> {
    validate_executable(path)?;

    let version_output = run_probe(path, &["--version"], timeout)?;
    let version = parse_version(provider, &version_output.stdout)?;
    let exec_help = parse_help(
        provider,
        &version,
        run_probe(path, &["exec", "--help"], timeout)?,
    )?;
    let resume_help = parse_optional_help(
        provider,
        &version,
        run_probe(path, &["exec", "resume", "--help"], timeout),
    )?;
    let schema_help = parse_optional_help(
        provider,
        &version,
        run_probe(
            path,
            &["app-server", "generate-json-schema", "--help"],
            timeout,
        ),
    )?;

    let exec_json = has_token(&exec_help, "--json");
    let output_last_message = has_token(&exec_help, "--output-last-message");
    let working_directory = has_token(&exec_help, "--cd");
    let model = has_token(&exec_help, "--model");
    let profile = has_token(&exec_help, "--profile");
    let sandbox = has_token(&exec_help, "--sandbox");
    let approval_config = has_token(&exec_help, "--config");
    let resume_json = has_token(&resume_help, "--json");
    let app_server_schema = has_token(&schema_help, "--out");

    for (supported, name) in [
        (exec_json, "--json"),
        (output_last_message, "--output-last-message"),
        (working_directory, "--cd"),
        (approval_config, "--config"),
    ] {
        if !supported {
            return Err(AppError::external(
                "provider.missing_capability",
                format!("{provider} exec help is missing required capability {name}"),
            ));
        }
    }

    let mut capabilities = ProviderCapabilities {
        schema_version: PROVIDER_CAPABILITIES_SCHEMA.to_owned(),
        provider,
        executable: path.to_owned(),
        version,
        exec_json,
        output_last_message,
        working_directory,
        model,
        profile,
        sandbox,
        approval_config,
        resume_json,
        app_server_schema,
        capability_sha256: String::new(),
    };
    capabilities.capability_sha256 = capability_digest(&capabilities)?;
    Ok(capabilities)
}

pub fn locate(provider: ProviderId, path_env: &OsStr) -> Result<PathBuf, AppError> {
    let entries = env::split_paths(path_env).collect::<Vec<_>>();
    if entries.is_empty() {
        return Err(AppError::external(
            "provider.path",
            "PATH contains no search directories",
        ));
    }

    let mut unusable = None;
    for directory in entries {
        if directory.as_os_str().is_empty() || !directory.is_absolute() {
            return Err(AppError::external(
                "provider.path",
                "PATH entries used for provider discovery must be nonempty absolute paths",
            ));
        }

        let candidate = directory.join(executable_name(provider));
        match executable_status(&candidate) {
            ExecutableStatus::Ready => return Ok(candidate),
            ExecutableStatus::Missing => {}
            ExecutableStatus::NotExecutable => {
                unusable.get_or_insert(candidate);
            }
        }
    }

    if let Some(path) = unusable {
        return Err(AppError::external(
            "provider.not_executable",
            format!(
                "provider executable is not a regular executable file: {}",
                path.display()
            ),
        ));
    }
    Err(AppError::external(
        "provider.missing",
        format!(
            "{} was not found in PATH",
            executable_name(provider).to_string_lossy()
        ),
    ))
}

fn validate_executable(path: &Path) -> Result<(), AppError> {
    match executable_status(path) {
        ExecutableStatus::Ready => Ok(()),
        ExecutableStatus::Missing => Err(AppError::external(
            "provider.missing",
            format!("provider executable does not exist: {}", path.display()),
        )),
        ExecutableStatus::NotExecutable => Err(AppError::external(
            "provider.not_executable",
            format!(
                "provider executable is not a regular executable file: {}",
                path.display()
            ),
        )),
    }
}

enum ExecutableStatus {
    Ready,
    Missing,
    NotExecutable,
}

fn executable_status(path: &Path) -> ExecutableStatus {
    match fs::metadata(path) {
        Ok(metadata) if regular_executable(&metadata) => ExecutableStatus::Ready,
        Ok(_) => ExecutableStatus::NotExecutable,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ExecutableStatus::Missing,
        Err(_) => ExecutableStatus::NotExecutable,
    }
}

#[cfg(unix)]
fn regular_executable(metadata: &fs::Metadata) -> bool {
    metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn regular_executable(metadata: &fs::Metadata) -> bool {
    metadata.is_file()
}

fn executable_name(provider: ProviderId) -> &'static OsStr {
    OsStr::new(match provider {
        ProviderId::Trae => "traecli",
        ProviderId::Codex => "codex",
    })
}

fn run_probe(
    executable: &Path,
    arguments: &[&str],
    timeout: Duration,
) -> Result<ProbeOutput, AppError> {
    run_probe_with_readiness(executable, arguments, timeout, None)
}

fn run_probe_with_readiness(
    executable: &Path,
    arguments: &[&str],
    timeout: Duration,
    readiness: Option<&Path>,
) -> Result<ProbeOutput, AppError> {
    let command_label = command_label(arguments);
    let mut stdout = private_temp_file("stdout")?;
    let mut stderr = private_temp_file("stderr")?;
    let stdout_child = stdout.as_file().try_clone().map_err(|error| {
        provider_io(
            format!("could not prepare stdout for {command_label}"),
            error,
        )
    })?;
    let stderr_child = stderr.as_file().try_clone().map_err(|error| {
        provider_io(
            format!("could not prepare stderr for {command_label}"),
            error,
        )
    })?;

    let mut command = Command::new(executable);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_child))
        .stderr(Stdio::from(stderr_child));
    configure_process_group(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| spawn_error(executable, &command_label, error))?;
    let process_group = ProbeProcessGroup::for_child(&child)?;
    if let Some(readiness) = readiness {
        await_probe_readiness(
            &mut child,
            process_group,
            readiness,
            &stdout,
            &stderr,
            &command_label,
        )?;
    }

    let status = wait_bounded(
        &mut child,
        process_group,
        &stdout,
        &stderr,
        timeout,
        &command_label,
    )?;
    terminate_and_reap(&mut child, process_group, true, &command_label)?;

    let stdout_bytes = read_temp_file(&mut stdout, "stdout", &command_label)?;
    let _stderr_bytes = read_temp_file(&mut stderr, "stderr", &command_label)?;
    require_success(status, &command_label)?;
    Ok(ProbeOutput {
        stdout: stdout_bytes,
    })
}

fn await_probe_readiness(
    child: &mut std::process::Child,
    process_group: ProbeProcessGroup,
    readiness: &Path,
    stdout: &NamedTempFile,
    stderr: &NamedTempFile,
    command_label: &str,
) -> Result<(), AppError> {
    let started = Instant::now();
    loop {
        if readiness
            .metadata()
            .is_ok_and(|metadata| metadata.len() > 0)
        {
            return Ok(());
        }
        if let Err(error) = check_output_limit(stdout, "stdout", command_label)
            .and_then(|_| check_output_limit(stderr, "stderr", command_label))
        {
            return Err(cleanup_preserving_primary(error, || {
                terminate_and_reap(child, process_group, false, command_label)
            }));
        }
        if child
            .try_wait()
            .map_err(|error| {
                provider_io(
                    format!("could not inspect provider command {command_label} for readiness"),
                    error,
                )
            })?
            .is_some()
        {
            terminate_and_reap(child, process_group, true, command_label)?;
            return Err(AppError::external(
                "provider.test_fixture",
                format!("provider command {command_label} exited before fixture readiness"),
            ));
        }
        if started.elapsed() >= PROBE_TIMEOUT {
            terminate_and_reap(child, process_group, false, command_label)?;
            return Err(AppError::external(
                "provider.test_fixture",
                format!("provider command {command_label} did not signal fixture readiness"),
            ));
        }
        thread::sleep(OUTPUT_POLL_INTERVAL);
    }
}

fn wait_bounded(
    child: &mut std::process::Child,
    process_group: ProbeProcessGroup,
    stdout: &NamedTempFile,
    stderr: &NamedTempFile,
    timeout: Duration,
    command_label: &str,
) -> Result<ExitStatus, AppError> {
    let started = Instant::now();
    loop {
        if let Err(error) = check_output_limit(stdout, "stdout", command_label)
            .and_then(|_| check_output_limit(stderr, "stderr", command_label))
        {
            return Err(cleanup_preserving_primary(error, || {
                terminate_and_reap(child, process_group, false, command_label)
            }));
        }

        let remaining = timeout.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            let timeout_error = AppError::external(
                "provider.timeout",
                format!(
                    "provider command {command_label} exceeded {} milliseconds",
                    timeout.as_millis()
                ),
            );
            return Err(cleanup_preserving_primary(timeout_error, || {
                terminate_and_reap(child, process_group, false, command_label)
            }));
        }
        let wait = remaining.min(OUTPUT_POLL_INTERVAL);
        if let Some(status) = child.wait_timeout(wait).map_err(|error| {
            provider_io(
                format!("could not wait for provider command {command_label}"),
                error,
            )
        })? {
            return Ok(status);
        }
    }
}

fn cleanup_preserving_primary(
    mut primary: AppError,
    cleanup: impl FnOnce() -> Result<(), AppError>,
) -> AppError {
    if let Err(cleanup_error) = cleanup() {
        primary.message.push_str("; cleanup also failed: ");
        primary.message.push_str(cleanup_error.code());
    }
    primary
}

#[cfg(unix)]
#[derive(Clone, Copy)]
struct ProbeProcessGroup(libc::pid_t);

#[cfg(unix)]
impl ProbeProcessGroup {
    fn for_child(child: &std::process::Child) -> Result<Self, AppError> {
        let pid = libc::pid_t::try_from(child.id()).map_err(|_| {
            AppError::external(
                "provider.process_group",
                "provider process ID does not fit the platform process-group type",
            )
        })?;
        Ok(Self(pid))
    }
}

#[cfg(not(unix))]
#[derive(Clone, Copy)]
struct ProbeProcessGroup;

#[cfg(not(unix))]
impl ProbeProcessGroup {
    fn for_child(_child: &std::process::Child) -> Result<Self, AppError> {
        Ok(Self)
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_and_reap(
    child: &mut std::process::Child,
    process_group: ProbeProcessGroup,
    mut direct_child_reaped: bool,
    command_label: &str,
) -> Result<(), AppError> {
    if signal_process_group(process_group, libc::SIGTERM, command_label)? {
        let started = Instant::now();
        if !direct_child_reaped
            && child
                .wait_timeout(TERMINATION_GRACE)
                .map_err(|error| {
                    provider_io(
                        format!(
                            "could not wait for provider command {command_label} during termination"
                        ),
                        error,
                    )
                })?
                .is_some()
        {
            direct_child_reaped = true;
        }
        thread::sleep(TERMINATION_GRACE.saturating_sub(started.elapsed()));
        signal_process_group(process_group, libc::SIGKILL, command_label)?;
    }
    if direct_child_reaped {
        Ok(())
    } else {
        reap_direct_child(child, command_label)
    }
}

#[cfg(not(unix))]
fn terminate_and_reap(
    child: &mut std::process::Child,
    _process_group: ProbeProcessGroup,
    direct_child_reaped: bool,
    command_label: &str,
) -> Result<(), AppError> {
    if direct_child_reaped {
        return Ok(());
    }
    if child
        .try_wait()
        .map_err(|error| {
            provider_io(
                format!("could not inspect provider command {command_label}"),
                error,
            )
        })?
        .is_none()
    {
        child.kill().map_err(|error| {
            provider_io(
                format!("could not kill provider command {command_label}"),
                error,
            )
        })?;
    }
    reap_direct_child(child, command_label)
}

fn reap_direct_child(child: &mut std::process::Child, command_label: &str) -> Result<(), AppError> {
    child.wait().map_err(|error| {
        provider_io(
            format!("could not reap provider command {command_label}"),
            error,
        )
    })?;
    Ok(())
}

#[cfg(unix)]
fn signal_process_group(
    process_group: ProbeProcessGroup,
    signal: libc::c_int,
    command_label: &str,
) -> Result<bool, AppError> {
    // SAFETY: the child is launched as leader of this dedicated process group.
    let result = unsafe { libc::killpg(process_group.0, signal) };
    if result == 0 {
        return Ok(true);
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ESRCH) {
        return Ok(false);
    }
    Err(provider_io(
        format!("could not send signal {signal} to provider process group for {command_label}"),
        error,
    ))
}

fn private_temp_file(stream: &str) -> Result<NamedTempFile, AppError> {
    let file = NamedTempFile::new().map_err(|error| {
        provider_io(
            format!("could not create private provider {stream} file"),
            error,
        )
    })?;
    secure_temp_file(&file, stream)?;
    Ok(file)
}

#[cfg(unix)]
fn secure_temp_file(file: &NamedTempFile, stream: &str) -> Result<(), AppError> {
    fs::set_permissions(file.path(), fs::Permissions::from_mode(0o600)).map_err(|error| {
        provider_io(
            format!("could not secure private provider {stream} file"),
            error,
        )
    })
}

#[cfg(not(unix))]
fn secure_temp_file(_file: &NamedTempFile, _stream: &str) -> Result<(), AppError> {
    Ok(())
}

fn check_output_limit(
    file: &NamedTempFile,
    stream: &str,
    command_label: &str,
) -> Result<(), AppError> {
    let length = file
        .as_file()
        .metadata()
        .map_err(|error| {
            provider_io(
                format!("could not inspect provider {stream} for {command_label}"),
                error,
            )
        })?
        .len();
    if length > MAX_OUTPUT_BYTES {
        return Err(output_limit(stream, command_label));
    }
    Ok(())
}

fn output_limit(stream: &str, command_label: &str) -> AppError {
    AppError::external(
        "provider.output_limit",
        format!("provider {stream} for {command_label} exceeds {MAX_OUTPUT_BYTES} bytes"),
    )
}

fn read_temp_file(
    file: &mut NamedTempFile,
    stream: &str,
    command_label: &str,
) -> Result<Vec<u8>, AppError> {
    file.as_file_mut()
        .seek(SeekFrom::Start(0))
        .map_err(|error| {
            provider_io(
                format!("could not seek provider {stream} for {command_label}"),
                error,
            )
        })?;
    let mut bytes = Vec::new();
    file.as_file_mut()
        .take(MAX_OUTPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            provider_io(
                format!("could not read provider {stream} for {command_label}"),
                error,
            )
        })?;
    if bytes.len() as u64 > MAX_OUTPUT_BYTES {
        return Err(output_limit(stream, command_label));
    }
    Ok(bytes)
}

fn require_success(status: ExitStatus, command_label: &str) -> Result<(), AppError> {
    if status.success() {
        return Ok(());
    }
    let status = status
        .code()
        .map_or_else(|| "signal".to_owned(), |code| code.to_string());
    Err(AppError::external(
        "provider.nonzero",
        format!("provider command {command_label} exited with status {status}"),
    ))
}

fn parse_version(provider: ProviderId, bytes: &[u8]) -> Result<String, AppError> {
    let output = std::str::from_utf8(bytes).map_err(|_| {
        AppError::external(
            "provider.malformed",
            format!("{provider} --version output is not UTF-8"),
        )
    })?;
    let output = output
        .strip_suffix("\r\n")
        .or_else(|| output.strip_suffix('\n'))
        .unwrap_or(output);
    let expected_name = match provider {
        ProviderId::Trae => "traecli",
        ProviderId::Codex => "codex-cli",
    };
    let Some(version) = output
        .strip_prefix(expected_name)
        .and_then(|value| value.strip_prefix(' '))
    else {
        return Err(malformed_version(provider, expected_name));
    };
    let version = match provider {
        ProviderId::Trae => version
            .strip_suffix("(internal edition)")
            .unwrap_or(version),
        ProviderId::Codex => version,
    };
    if !valid_version(version) {
        return Err(malformed_version(provider, expected_name));
    }
    Ok(version.to_owned())
}

fn malformed_version(provider: ProviderId, expected_name: &str) -> AppError {
    AppError::external(
        "provider.malformed",
        format!("{provider} --version must have the form `{expected_name} <major.minor.patch>`"),
    )
}

fn valid_version(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_VERSION_BYTES {
        return false;
    }
    let (without_build, build) = match value.split_once('+') {
        Some((version, build)) if !build.contains('+') => (version, Some(build)),
        Some(_) => return false,
        None => (value, None),
    };
    let (core, prerelease) = match without_build.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (without_build, None),
    };
    let mut core_components = core.split('.');
    let core_valid = (0..3).all(|_| core_components.next().is_some_and(valid_numeric_identifier))
        && core_components.next().is_none();

    core_valid
        && prerelease.is_none_or(|value| valid_identifiers(value, true))
        && build.is_none_or(|value| valid_identifiers(value, false))
}

fn valid_numeric_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_VERSION_IDENTIFIER_BYTES
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn valid_identifiers(value: &str, reject_numeric_leading_zero: bool) -> bool {
    !value.is_empty()
        && value.split('.').all(|identifier| {
            !identifier.is_empty()
                && identifier.len() <= MAX_VERSION_IDENTIFIER_BYTES
                && identifier
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && (!reject_numeric_leading_zero
                    || !identifier.bytes().all(|byte| byte.is_ascii_digit())
                    || identifier == "0"
                    || !identifier.starts_with('0'))
        })
}

fn parse_help(
    provider: ProviderId,
    version: &str,
    output: ProbeOutput,
) -> Result<Vec<String>, AppError> {
    let help = String::from_utf8(output.stdout).map_err(|_| {
        AppError::external("provider.malformed", "provider help output is not UTF-8")
    })?;
    Ok(HelpDialect::for_provider(provider, version).parse_option_rows(&help))
}

fn parse_optional_help(
    provider: ProviderId,
    version: &str,
    output: Result<ProbeOutput, AppError>,
) -> Result<Vec<String>, AppError> {
    match output {
        Ok(output) => parse_help(provider, version, output),
        Err(error) if error.code() == "provider.nonzero" => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

enum HelpDialect {
    Trae,
    Codex,
}

impl HelpDialect {
    fn for_provider(provider: ProviderId, version: &str) -> Self {
        debug_assert!(valid_version(version));
        match provider {
            ProviderId::Trae => Self::Trae,
            ProviderId::Codex => Self::Codex,
        }
    }

    fn parse_option_rows(&self, help: &str) -> Vec<String> {
        help.lines()
            .filter_map(|line| match self {
                Self::Trae => declared_trae_long_option(line),
                Self::Codex => declared_codex_long_option(line),
            })
            .collect()
    }
}

fn declared_trae_long_option(line: &str) -> Option<String> {
    declared_long_option(line)
}

fn declared_codex_long_option(line: &str) -> Option<String> {
    declared_long_option(line)
}

fn declared_long_option(line: &str) -> Option<String> {
    let declaration = line.trim_start();
    let declaration = strip_short_option_alias(declaration).unwrap_or(declaration);
    parse_long_option_declaration(declaration)
}

fn strip_short_option_alias(declaration: &str) -> Option<&str> {
    let bytes = declaration.as_bytes();
    if bytes.len() < 5
        || bytes[0] != b'-'
        || bytes[1] == b'-'
        || !bytes[1].is_ascii_alphanumeric()
        || bytes[2] != b','
    {
        return None;
    }
    declaration[3..]
        .trim_start()
        .starts_with("--")
        .then(|| declaration[3..].trim_start())
}

fn parse_long_option_declaration(declaration: &str) -> Option<String> {
    if !declaration.starts_with("--") {
        return None;
    }
    let token_end = declaration
        .find(|character: char| {
            character.is_ascii_whitespace() || character == '=' || character == ','
        })
        .unwrap_or(declaration.len());
    let option = &declaration[..token_end];
    if option.len() <= 2
        || !option[2..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return None;
    }
    let suffix = &declaration[token_end..];
    let declaration_suffix = suffix.is_empty()
        || suffix.starts_with("  ")
        || suffix.starts_with('\t')
        || suffix.starts_with(" <")
        || suffix.starts_with(" [")
        || suffix.starts_with('=')
        || suffix.starts_with(',');
    declaration_suffix.then(|| option.to_owned())
}

fn has_token(help: &[String], token: &str) -> bool {
    help.iter().any(|candidate| candidate == token)
}

fn capability_digest(capabilities: &ProviderCapabilities) -> Result<String, AppError> {
    let payload = CapabilityDigest {
        schema_version: &capabilities.schema_version,
        provider: capabilities.provider,
        executable: executable_name(capabilities.provider)
            .to_str()
            .expect("provider executable names are ASCII"),
        version: &capabilities.version,
        exec_json: capabilities.exec_json,
        output_last_message: capabilities.output_last_message,
        working_directory: capabilities.working_directory,
        model: capabilities.model,
        profile: capabilities.profile,
        sandbox: capabilities.sandbox,
        approval_config: capabilities.approval_config,
        resume_json: capabilities.resume_json,
        app_server_schema: capabilities.app_server_schema,
    };
    let canonical = serde_json::to_vec(&payload).map_err(|error| {
        AppError::external(
            "provider.serialization",
            format!("could not serialize provider capabilities: {error}"),
        )
    })?;
    Ok(format!("sha256:{:x}", Sha256::digest(canonical)))
}

fn command_label(arguments: &[&str]) -> String {
    arguments.join(" ")
}

fn spawn_error(executable: &Path, command_label: &str, error: std::io::Error) -> AppError {
    if error.kind() == std::io::ErrorKind::NotFound {
        AppError::external(
            "provider.missing",
            format!(
                "provider executable disappeared before {command_label}: {}",
                executable.display()
            ),
        )
    } else {
        provider_io(
            format!(
                "could not start provider command {} {command_label}",
                executable.display()
            ),
            error,
        )
    }
}

fn provider_io(context: String, error: std::io::Error) -> AppError {
    AppError::io("provider.io", &context, error)
}

#[cfg(all(test, unix))]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::io::{Seek, SeekFrom};
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, Instant};

    use tempfile::TempDir;

    use super::{
        cleanup_preserving_primary, locate, parse_optional_help, parse_version, probe,
        probe_with_timeout, read_temp_file, run_probe_with_readiness, HelpDialect,
        MAX_OUTPUT_BYTES,
    };
    use crate::context_control::{ProviderId, PROVIDER_CAPABILITIES_SCHEMA};

    const EXEC_HELP: &str =
        "--json\n--output-last-message\n--cd\n--model\n--profile\n--sandbox\n--config\n";
    const CAPTURED_TRAE_EXEC_HELP: &str = r#"Run TRAE CLI non-interactively

Usage: traecli exec [OPTIONS] [PROMPT]

Options:
  -c, --config <key=value>
          Override a configuration value

  -m, --model <MODEL>
          Model the agent should use

  -p, --profile <CONFIG_PROFILE>
          Layer a named configuration profile

  -s, --sandbox <SANDBOX_MODE>
          Select the sandbox policy

  -C, --cd <DIR>
          Tell the agent to use the specified directory as its working root

      --json
          Print events to stdout as JSONL

  -o, --output-last-message <FILE>
          Specifies file where the last message should be written
"#;
    const CAPTURED_TRAE_RESUME_HELP: &str = r#"Resume a previous session

Usage: traecli exec resume [OPTIONS] [SESSION_ID] [PROMPT]

Options:
  -c, --config <key=value>
          Override a configuration value

      --json
          Print events to stdout as JSONL

  -o, --output-last-message <FILE>
          Specifies file where the last message should be written
"#;
    const CAPTURED_TRAE_SCHEMA_HELP: &str = r#"[experimental] Generate JSON Schema

Usage: traecli app-server generate-json-schema [OPTIONS] --out <DIR>

Options:
  -c, --config <key=value>
          Override a configuration value

  -o, --out <DIR>
          Output directory where the schema bundle will be written
"#;

    #[test]
    fn probe_reports_supported_trae_capabilities() {
        let fixture = ProviderFixture::supported(ProviderId::Trae);
        let capabilities = probe(ProviderId::Trae, &fixture.executable).unwrap();

        assert_eq!(capabilities.schema_version, PROVIDER_CAPABILITIES_SCHEMA);
        assert_eq!(capabilities.provider, ProviderId::Trae);
        assert_eq!(capabilities.executable, fixture.executable);
        assert_eq!(capabilities.version, "0.200.19");
        assert!(capabilities.exec_json);
        assert!(capabilities.output_last_message);
        assert!(capabilities.working_directory);
        assert!(capabilities.model);
        assert!(capabilities.profile);
        assert!(capabilities.sandbox);
        assert!(capabilities.approval_config);
        assert!(capabilities.resume_json);
        assert!(capabilities.app_server_schema);
        assert_eq!(
            capabilities.capability_sha256,
            "sha256:b954641724ae8b3c9d0a072d5c1acf965e849bb119c2d912529c6e564ee5031e"
        );
        assert_eq!(
            fixture.invocations(),
            [
                "--version",
                "exec --help",
                "exec resume --help",
                "app-server generate-json-schema --help",
            ]
        );
    }

    #[test]
    fn probe_accepts_the_captured_trae_internal_edition_version() {
        let fixture = ProviderFixture::supported(ProviderId::Trae);
        let capabilities = probe(ProviderId::Trae, &fixture.executable).unwrap();

        assert_eq!(capabilities.version, "0.200.19");
    }

    #[test]
    fn probe_parses_the_exact_codex_version_prefix() {
        let fixture = ProviderFixture::supported(ProviderId::Codex);
        let capabilities = probe(ProviderId::Codex, &fixture.executable).unwrap();
        assert_eq!(capabilities.version, "0.144.5");
        assert_eq!(
            capabilities.capability_sha256,
            "sha256:80df74127b9fdd0299426c4c3fd7b806a64e4e5e442d7e8d5383478d95e22f5a"
        );
    }

    #[test]
    fn probe_accepts_newer_well_formed_versions_when_capabilities_match() {
        let fixture = ProviderFixture::new(
            ProviderId::Trae,
            FixtureBehavior::Custom {
                version: "traecli 9.8.7",
                exec_help: EXEC_HELP,
                resume_help: "--json\n",
                schema_help: "--out\n",
            },
        );

        let capabilities = probe(ProviderId::Trae, &fixture.executable).unwrap();
        assert_eq!(capabilities.version, "9.8.7");
        assert!(capabilities.exec_json);
        assert!(capabilities.output_last_message);
        assert!(capabilities.working_directory);
        assert!(capabilities.approval_config);
    }

    #[test]
    fn probe_accepts_bounded_semver_prerelease_and_build_identifiers() {
        for (provider, output, expected) in [
            (
                ProviderId::Trae,
                "traecli 0.145.0-beta.1+build.7(internal edition)",
                "0.145.0-beta.1+build.7",
            ),
            (
                ProviderId::Codex,
                "codex-cli 0.145.0-beta.1",
                "0.145.0-beta.1",
            ),
            (
                ProviderId::Codex,
                "codex-cli 0.145.0+build.7",
                "0.145.0+build.7",
            ),
        ] {
            assert_eq!(
                parse_version(provider, output.as_bytes()).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn probe_keeps_optional_capabilities_false() {
        let fixture = ProviderFixture::new(
            ProviderId::Trae,
            FixtureBehavior::Custom {
                version: "traecli 0.200.19",
                exec_help: "--json\n--output-last-message\n--cd\n--config\n",
                resume_help: "resume command\n",
                schema_help: "schema command\n",
            },
        );
        let capabilities = probe(ProviderId::Trae, &fixture.executable).unwrap();

        assert!(!capabilities.model);
        assert!(!capabilities.profile);
        assert!(!capabilities.sandbox);
        assert!(!capabilities.resume_json);
        assert!(!capabilities.app_server_schema);
    }

    #[test]
    fn probe_treats_nonzero_optional_help_as_unsupported() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::OptionalHelpNonzero);
        let capabilities = probe(ProviderId::Trae, &fixture.executable).unwrap();

        assert!(!capabilities.resume_json);
        assert!(!capabilities.app_server_schema);
    }

    #[test]
    fn optional_help_timeout_and_overflow_remain_failures() {
        let timeout = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::OptionalHelpTimeout);
        assert_eq!(
            probe_with_timeout(
                ProviderId::Trae,
                &timeout.executable,
                Duration::from_millis(500),
            )
            .unwrap_err()
            .code(),
            "provider.timeout"
        );

        let overflow =
            ProviderFixture::new(ProviderId::Trae, FixtureBehavior::OptionalHelpOverLimit);
        assert_eq!(
            probe(ProviderId::Trae, &overflow.executable)
                .unwrap_err()
                .code(),
            "provider.output_limit"
        );
    }

    #[test]
    fn optional_help_io_error_remains_a_typed_failure() {
        let error = crate::AppError::io(
            "provider.io",
            "optional help fixture",
            std::io::Error::other("fixture I/O failure"),
        );

        assert_eq!(
            parse_optional_help(ProviderId::Trae, "0.200.19", Err(error))
                .unwrap_err()
                .code(),
            "provider.io"
        );
    }

    #[test]
    fn probe_rejects_each_missing_required_capability() {
        for missing in ["--json", "--output-last-message", "--cd", "--config"] {
            let help = EXEC_HELP
                .lines()
                .filter(|line| *line != missing)
                .collect::<Vec<_>>()
                .join("\n");
            let fixture = ProviderFixture::new(
                ProviderId::Trae,
                FixtureBehavior::Custom {
                    version: "traecli 0.200.19",
                    exec_help: Box::leak(help.into_boxed_str()),
                    resume_help: "--json\n",
                    schema_help: "--out\n",
                },
            );

            let error = probe(ProviderId::Trae, &fixture.executable).unwrap_err();
            assert_eq!(error.code(), "provider.missing_capability", "{missing}");
            assert!(
                error.message.contains(missing),
                "{missing}: {}",
                error.message
            );
        }
    }

    #[test]
    fn probe_ignores_flags_outside_help_declaration_rows() {
        let false_positive_help = [
            "Use --json with --output-last-message, --cd, and --config for automation.\n",
            "Example: traecli exec --json --output-last-message result --cd repo --config x=y\n",
            "This build does not support --json, --output-last-message, --cd, or --config.\n",
        ];

        for help in false_positive_help {
            let fixture = ProviderFixture::new(
                ProviderId::Trae,
                FixtureBehavior::Custom {
                    version: "traecli 0.200.19",
                    exec_help: help,
                    resume_help: "Resume does not support --json.\n",
                    schema_help: "Example: generate-json-schema --out schema.json\n",
                },
            );

            assert_eq!(
                probe(ProviderId::Trae, &fixture.executable)
                    .unwrap_err()
                    .code(),
                "provider.missing_capability",
                "{help:?}"
            );
        }
    }

    #[test]
    fn provider_help_parsers_accept_only_option_declaration_rows() {
        let trae = HelpDialect::for_provider(ProviderId::Trae, "9.8.7").parse_option_rows(
            "Options:\n  -c, --config <key=value>  Override configuration\n\
               -C, --cd <DIR>  Set working directory\n\
               -o, --output-last-message <FILE>  Store final response\n\
                   --json  Emit JSON\n\
             Example: traecli exec --sandbox read-only\n\
             This build does not support --profile.\n",
        );
        assert_eq!(
            trae,
            ["--config", "--cd", "--output-last-message", "--json"]
        );

        let codex = HelpDialect::for_provider(ProviderId::Codex, "0.144.5").parse_option_rows(
            "Options:\n  -c, --config <key=value>  Override configuration\n\
             This build does not support --sandbox.\n",
        );
        assert_eq!(codex, ["--config"]);
    }

    #[test]
    fn probe_rejects_malformed_version_output() {
        let oversized_identifier = format!("traecli 1.2.3-{}", "a".repeat(65));
        let oversized_version = format!("traecli 1.2.3+{}", "a.".repeat(61) + "a");
        let versions = [
            "trae 0.200.19 extra".to_owned(),
            "traecli 0.200.19(internal edition) extra".to_owned(),
            "traecli 0.200.19(beta edition)".to_owned(),
            "traecli 0.200.19 internal edition".to_owned(),
            "traecli 01.2.3".to_owned(),
            "traecli 1.02.3".to_owned(),
            "traecli 1.2.03".to_owned(),
            "traecli 1.2.3-01".to_owned(),
            "traecli 1.2.3-beta..1".to_owned(),
            "traecli 1.2.3-beta_1".to_owned(),
            "traecli 1.2.3+".to_owned(),
            oversized_identifier,
            oversized_version,
        ];
        for version in &versions {
            let fixture = ProviderFixture::new(
                ProviderId::Trae,
                FixtureBehavior::Custom {
                    version: Box::leak(version.clone().into_boxed_str()),
                    exec_help: EXEC_HELP,
                    resume_help: "--json\n",
                    schema_help: "--out\n",
                },
            );

            assert_eq!(
                probe(ProviderId::Trae, &fixture.executable)
                    .unwrap_err()
                    .code(),
                "provider.malformed",
                "{version:?}"
            );
        }
    }

    #[test]
    fn probe_rejects_nonzero_commands() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::Nonzero);
        let error = probe(ProviderId::Trae, &fixture.executable).unwrap_err();
        assert_eq!(error.code(), "provider.nonzero");
        assert!(error.message.contains("--version"));
        assert!(error.message.contains("23"));
    }

    #[test]
    fn probe_rejects_stdout_over_the_limit() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::OverLimit);
        let error = probe(ProviderId::Trae, &fixture.executable).unwrap_err();
        assert_eq!(error.code(), "provider.output_limit");
        assert!(error.message.contains("stdout"));
    }

    #[test]
    fn probe_rejects_stderr_over_the_limit() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::StderrOverLimit);
        let error = probe(ProviderId::Trae, &fixture.executable).unwrap_err();
        assert_eq!(error.code(), "provider.output_limit");
        assert!(error.message.contains("stderr"));
    }

    #[test]
    fn bounded_capture_read_never_allocates_past_limit_plus_one() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.as_file_mut().set_len(MAX_OUTPUT_BYTES * 2).unwrap();
        file.as_file_mut().seek(SeekFrom::Start(0)).unwrap();

        let error = read_temp_file(&mut file, "stdout", "--version").unwrap_err();
        assert_eq!(error.code(), "provider.output_limit");
    }

    #[test]
    fn cleanup_failure_preserves_timeout_primary_error_and_attempts_cleanup() {
        assert_cleanup_failure_preserves_primary("provider.timeout");
    }

    #[test]
    fn cleanup_failure_preserves_output_limit_primary_error_and_attempts_cleanup() {
        assert_cleanup_failure_preserves_primary("provider.output_limit");
    }

    #[cfg(unix)]
    #[test]
    fn output_limit_terminates_term_resistant_descendants_before_returning() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::PersistentWriter);
        let error = run_probe_with_readiness(
            &fixture.executable,
            &["--version"],
            Duration::from_secs(1),
            Some(&fixture.ready_file),
        )
        .unwrap_err();
        assert_eq!(error.code(), "provider.output_limit");
        assert_descendant_stopped(&fixture);
    }

    #[cfg(unix)]
    #[test]
    fn timeout_terminates_term_resistant_descendants_before_returning() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::PersistentTimeout);
        let error = run_probe_with_readiness(
            &fixture.executable,
            &["--version"],
            Duration::from_millis(50),
            Some(&fixture.ready_file),
        )
        .unwrap_err();
        assert_eq!(error.code(), "provider.timeout", "{}", error.message);
        assert_descendant_stopped(&fixture);
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_exit_terminates_term_resistant_descendants_before_returning() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::PersistentNonzero);
        let error = run_probe_with_readiness(
            &fixture.executable,
            &["--version"],
            Duration::from_secs(1),
            Some(&fixture.ready_file),
        )
        .unwrap_err();
        assert_eq!(error.code(), "provider.nonzero", "{}", error.message);
        assert_descendant_stopped(&fixture);
    }

    #[test]
    fn probe_kills_and_waits_for_a_timed_out_command() {
        let fixture = ProviderFixture::new(ProviderId::Trae, FixtureBehavior::Timeout);
        let started = Instant::now();
        let error = probe_with_timeout(
            ProviderId::Trae,
            &fixture.executable,
            Duration::from_millis(30),
        )
        .unwrap_err();

        assert_eq!(error.code(), "provider.timeout", "{}", error.message);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn probe_rejects_a_missing_executable() {
        let missing = Path::new("/definitely/missing/traecli");
        assert_eq!(
            probe(ProviderId::Trae, missing).unwrap_err().code(),
            "provider.missing"
        );
    }

    #[test]
    fn capability_digest_excludes_the_absolute_executable_location() {
        let first = ProviderFixture::supported(ProviderId::Trae);
        let second = ProviderFixture::supported(ProviderId::Trae);
        let first = probe(ProviderId::Trae, &first.executable).unwrap();
        let second = probe(ProviderId::Trae, &second.executable).unwrap();

        assert_ne!(first.executable, second.executable);
        assert_eq!(first.capability_sha256, second.capability_sha256);
    }

    #[test]
    fn locate_finds_a_regular_executable_in_path_order() {
        let first = TempDir::new().unwrap();
        let second = ProviderFixture::supported(ProviderId::Trae);
        let path = std::env::join_paths([first.path(), second.bin.as_path()]).unwrap();

        assert_eq!(locate(ProviderId::Trae, &path).unwrap(), second.executable);
    }

    #[test]
    fn locate_rejects_relative_and_empty_path_entries() {
        for path in [OsString::from("relative/bin"), OsString::from(":/usr/bin")] {
            assert_eq!(
                locate(ProviderId::Trae, &path).unwrap_err().code(),
                "provider.path"
            );
        }
    }

    #[test]
    fn locate_rejects_non_executable_candidates_and_accepts_executable_symlinks() {
        let root = TempDir::new().unwrap();
        let candidate = root.path().join("traecli");
        fs::write(&candidate, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            locate(ProviderId::Trae, root.path().as_os_str())
                .unwrap_err()
                .code(),
            "provider.not_executable"
        );

        let target = root.path().join("target");
        fs::write(&target, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_file(&candidate).unwrap();
        symlink(&target, &candidate).unwrap();
        assert_eq!(
            locate(ProviderId::Trae, root.path().as_os_str()).unwrap(),
            candidate
        );
    }

    enum FixtureBehavior {
        Custom {
            version: &'static str,
            exec_help: &'static str,
            resume_help: &'static str,
            schema_help: &'static str,
        },
        Nonzero,
        OptionalHelpNonzero,
        OptionalHelpOverLimit,
        OptionalHelpTimeout,
        OverLimit,
        StderrOverLimit,
        PersistentNonzero,
        PersistentTimeout,
        PersistentWriter,
        Timeout,
    }

    struct ProviderFixture {
        _root: TempDir,
        bin: PathBuf,
        executable: PathBuf,
        log: PathBuf,
        marker: PathBuf,
        pid_file: PathBuf,
        ready_file: PathBuf,
    }

    impl ProviderFixture {
        fn supported(provider: ProviderId) -> Self {
            let behavior = match provider {
                ProviderId::Trae => FixtureBehavior::Custom {
                    version: "traecli 0.200.19(internal edition)",
                    exec_help: CAPTURED_TRAE_EXEC_HELP,
                    resume_help: CAPTURED_TRAE_RESUME_HELP,
                    schema_help: CAPTURED_TRAE_SCHEMA_HELP,
                },
                ProviderId::Codex => FixtureBehavior::Custom {
                    version: "codex-cli 0.144.5",
                    exec_help: EXEC_HELP,
                    resume_help: "--json\n",
                    schema_help: "--out\n",
                },
            };
            Self::new(provider, behavior)
        }

        fn new(provider: ProviderId, behavior: FixtureBehavior) -> Self {
            let root = TempDir::new().unwrap();
            let bin = root.path().join("bin");
            fs::create_dir(&bin).unwrap();
            let name = match provider {
                ProviderId::Trae => "traecli",
                ProviderId::Codex => "codex",
            };
            let executable = bin.join(name);
            let log = root.path().join("invocations");
            let marker = root.path().join("descendant-marker");
            let pid_file = root.path().join("descendant-pid");
            let ready_file = root.path().join("descendant-ready");
            let script = match behavior {
                FixtureBehavior::Custom {
                    version,
                    exec_help,
                    resume_help,
                    schema_help,
                } => custom_script(&log, version, exec_help, resume_help, schema_help),
                FixtureBehavior::Nonzero => "#!/bin/sh\nexit 23\n".to_owned(),
                FixtureBehavior::OptionalHelpNonzero => optional_help_script("exit 23"),
                FixtureBehavior::OptionalHelpOverLimit => {
                    optional_help_script("dd if=/dev/zero bs=1048577 count=1 2>/dev/null")
                }
                FixtureBehavior::OptionalHelpTimeout => optional_help_script("sleep 10"),
                FixtureBehavior::OverLimit => "#!/bin/sh\nyes x | head -c 1048577\n".to_owned(),
                FixtureBehavior::StderrOverLimit => {
                    "#!/bin/sh\nyes x | head -c 1048577 >&2\n".to_owned()
                }
                FixtureBehavior::PersistentNonzero => {
                    persistent_descendant_script(&marker, &pid_file, &ready_file, "exit 23", false)
                }
                FixtureBehavior::PersistentTimeout => {
                    persistent_descendant_script(&marker, &pid_file, &ready_file, "wait", false)
                }
                FixtureBehavior::PersistentWriter => {
                    persistent_writer_script(&marker, &pid_file, &ready_file)
                }
                FixtureBehavior::Timeout => "#!/bin/sh\nsleep 10\n".to_owned(),
            };
            write_executable(&executable, &script);
            Self {
                _root: root,
                bin,
                executable,
                log,
                marker,
                pid_file,
                ready_file,
            }
        }

        fn invocations(&self) -> Vec<String> {
            fs::read_to_string(&self.log)
                .unwrap()
                .lines()
                .map(str::to_owned)
                .collect()
        }
    }

    fn custom_script(
        log: &Path,
        version: &str,
        exec_help: &str,
        resume_help: &str,
        schema_help: &str,
    ) -> String {
        format!(
            r#"#!/bin/sh
printf '%s\n' "$*" >> '{}'
case "$*" in
  "--version") printf '%s\n' '{}' ;;
  "exec --help") printf '%s' '{}' ;;
  "exec resume --help") printf '%s' '{}' ;;
  "app-server generate-json-schema --help") printf '%s' '{}' ;;
  *) exit 64 ;;
esac
"#,
            shell_quote(log.as_os_str()),
            shell_quote(OsStr::new(version)),
            shell_quote(OsStr::new(exec_help)),
            shell_quote(OsStr::new(resume_help)),
            shell_quote(OsStr::new(schema_help)),
        )
    }

    fn shell_quote(value: &OsStr) -> String {
        value.to_string_lossy().replace('\'', "'\\''")
    }

    fn optional_help_script(optional_action: &str) -> String {
        format!(
            r#"#!/bin/sh
case "$*" in
  "--version") printf '%s\n' 'traecli 0.200.19' ;;
  "exec --help") printf '%s\n' --json --output-last-message --cd --config ;;
  "exec resume --help"|"app-server generate-json-schema --help") {optional_action} ;;
  *) exit 64 ;;
esac
"#
        )
    }

    fn persistent_writer_script(marker: &Path, pid_file: &Path, ready_file: &Path) -> String {
        persistent_descendant_script(marker, pid_file, ready_file, "wait", true)
    }

    fn persistent_descendant_script(
        marker: &Path,
        pid_file: &Path,
        ready_file: &Path,
        parent_action: &str,
        write_stdout: bool,
    ) -> String {
        let stdout_write = if write_stdout {
            "  dd if=/dev/zero bs=1048577 count=1 2>/dev/null\n"
        } else {
            ""
        };
        format!(
            r#"#!/bin/sh
(
  trap '' TERM
  : > '{}'
  while [ ! -s '{}' ]; do
    sleep 0.01
  done
{}
  printf ready > '{}'
  while :; do
    printf x >> '{}'
  done
) &
printf '%s\n' "$!" > '{}'
while [ ! -s '{}' ]; do
  sleep 0.01
done
{}
"#,
            shell_quote(marker.as_os_str()),
            shell_quote(pid_file.as_os_str()),
            stdout_write,
            shell_quote(ready_file.as_os_str()),
            shell_quote(marker.as_os_str()),
            shell_quote(pid_file.as_os_str()),
            shell_quote(ready_file.as_os_str()),
            parent_action,
        )
    }

    fn assert_descendant_stopped(fixture: &ProviderFixture) {
        let descendant_pid = wait_for_pid(&fixture.pid_file);
        let _guard = ProcessGuard(descendant_pid);
        assert_process_gone(descendant_pid);

        let marker_len = fs::metadata(&fixture.marker).unwrap().len();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(fs::metadata(&fixture.marker).unwrap().len(), marker_len);
    }

    fn wait_for_pid(path: &Path) -> libc::pid_t {
        let started = Instant::now();
        loop {
            if let Ok(value) = fs::read_to_string(path) {
                return value.trim().parse().unwrap();
            }
            assert!(started.elapsed() < Duration::from_secs(2));
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn assert_cleanup_failure_preserves_primary(primary_code: &'static str) {
        let mut cleanup_attempts = 0;
        let primary = crate::AppError::external(primary_code, "primary provider failure");
        let error = cleanup_preserving_primary(primary, || {
            cleanup_attempts += 1;
            Err(crate::AppError::external(
                "provider.io",
                "sensitive cleanup detail",
            ))
        });

        assert_eq!(cleanup_attempts, 1);
        assert_eq!(error.code(), primary_code);
        assert!(error.message.contains("cleanup also failed: provider.io"));
        assert!(!error.message.contains("sensitive cleanup detail"));
    }

    #[cfg(unix)]
    fn assert_process_gone(pid: libc::pid_t) {
        let started = Instant::now();
        loop {
            // SAFETY: signal zero does not mutate the target process.
            let result = unsafe { libc::kill(pid, 0) };
            if result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                return;
            }
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "descendant {pid} survived probe cleanup"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[cfg(unix)]
    struct ProcessGuard(libc::pid_t);

    #[cfg(unix)]
    impl Drop for ProcessGuard {
        fn drop(&mut self) {
            // SAFETY: test cleanup targets the PID written by the fixture.
            unsafe {
                libc::kill(self.0, libc::SIGKILL);
            }
        }
    }

    fn write_executable(path: &Path, script: &str) {
        fs::write(path, script).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
}
