use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::fs::File;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::io::{Read as _, Write as _};
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::fd::{AsRawFd as _, FromRawFd as _, IntoRawFd as _, OwnedFd};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt as _;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt as _;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::path::Component;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ProviderCapabilities;
use crate::context_control::canonical::{json_bytes, sha256_hex};
use crate::context_control::context::ContextBundle;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use crate::context_control::repository_state::repository_id;
use crate::context_control::{
    ProviderId, RepositorySnapshot, CONTEXT_BUNDLE_SCHEMA, PROVIDER_CAPABILITIES_SCHEMA,
    PROVIDER_INVOCATION_SCHEMA,
};
use crate::AppError;

const MAX_OPTION_BYTES: usize = 256;
const FINAL_MESSAGE_NAME: &str = "final_message.bin";
const STDOUT_NAME: &str = "stdout.jsonl";
const STDERR_NAME: &str = "stderr.bin";
const RAW_ARCHIVE_NAME: &str = "raw_traecli_run.tar.gz";
const RAW_MEMBERS_MANIFEST_NAME: &str = "raw_members.tsv";
const LAUNCH_CLAIM_NAME: &str = "launch.json";
const LAUNCH_CLAIM_SCHEMA: &str = "harp-provider-launch/v1";
const COMMAND_ENCODING_MAGIC: &[u8] = b"harp-provider-command-v1\0";
const COMMAND_PLAN_ENCODING_MAGIC: &[u8] = b"harp-provider-command-plan-v1\0";
const PROMPT_ENCODING_MAGIC: &[u8] = b"harp-provider-prompt-v1\n";
const MAX_VERSION_BYTES: usize = 128;
const MAX_VERSION_IDENTIFIER_BYTES: usize = 64;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) const MAX_STDOUT_BYTES: usize = 64 * 1024 * 1024;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) const MAX_STDERR_BYTES: usize = 64 * 1024 * 1024;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) const MAX_FINAL_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
#[cfg(any(target_os = "macos", target_os = "linux"))]
const TAR_BLOCK_BYTES: usize = 512;
#[cfg(any(target_os = "macos", target_os = "linux"))]
const TAR_END_BYTES: usize = 2 * TAR_BLOCK_BYTES;
#[cfg(any(target_os = "macos", target_os = "linux"))]
const DEFLATE_STORED_BLOCK_BYTES: usize = 16 * 1024;
#[cfg(any(target_os = "macos", target_os = "linux"))]
const DEFLATE_STORED_BLOCK_OVERHEAD: usize = 5;
#[cfg(any(target_os = "macos", target_os = "linux"))]
const GZIP_WRAPPER_BYTES: usize = 18;
#[cfg(any(target_os = "macos", target_os = "linux"))]
const MAX_RAW_TAR_BYTES: usize = tar_member_bound(MAX_STDOUT_BYTES)
    + tar_member_bound(MAX_STDERR_BYTES)
    + tar_member_bound(MAX_FINAL_MESSAGE_BYTES)
    + TAR_END_BYTES;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) const MAX_RAW_ARCHIVE_BYTES: usize = MAX_RAW_TAR_BYTES
    + div_ceil(MAX_RAW_TAR_BYTES, DEFLATE_STORED_BLOCK_BYTES) * DEFLATE_STORED_BLOCK_OVERHEAD
    + GZIP_WRAPPER_BYTES;

#[cfg(any(target_os = "macos", target_os = "linux"))]
const fn div_ceil(value: usize, divisor: usize) -> usize {
    value.div_ceil(divisor)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const fn tar_member_bound(member_bytes: usize) -> usize {
    TAR_BLOCK_BYTES + div_ceil(member_bytes, TAR_BLOCK_BYTES) * TAR_BLOCK_BYTES
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl SandboxMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

impl fmt::Display for SandboxMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    Untrusted,
    OnRequest,
    Never,
}

impl ApprovalPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::OnRequest => "on-request",
            Self::Never => "never",
        }
    }
}

impl fmt::Display for ApprovalPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderRunOptions {
    pub model: Option<String>,
    pub profile: Option<String>,
    pub sandbox: Option<SandboxMode>,
    pub approval: Option<ApprovalPolicy>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderArgumentPlan {
    Literal(OsString),
    OutputMember(PathBuf),
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProviderRawMember {
    StdoutJsonl,
    StderrBin,
    FinalMessage,
    RawArchive,
    RawMembersManifest,
    LaunchClaim,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl ProviderRawMember {
    const fn name(self) -> &'static str {
        match self {
            Self::StdoutJsonl => STDOUT_NAME,
            Self::StderrBin => STDERR_NAME,
            Self::FinalMessage => FINAL_MESSAGE_NAME,
            Self::RawArchive => RAW_ARCHIVE_NAME,
            Self::RawMembersManifest => RAW_MEMBERS_MANIFEST_NAME,
            Self::LaunchClaim => LAUNCH_CLAIM_NAME,
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const PROVIDER_RAW_MEMBERS: [ProviderRawMember; 6] = [
    ProviderRawMember::StdoutJsonl,
    ProviderRawMember::StderrBin,
    ProviderRawMember::FinalMessage,
    ProviderRawMember::RawArchive,
    ProviderRawMember::RawMembersManifest,
    ProviderRawMember::LaunchClaim,
];

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug, Eq, PartialEq)]
struct RawMemberEvidence {
    member: ProviderRawMember,
    bytes: Vec<u8>,
    sha256: String,
    version: RawFileVersion,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RawFileVersion {
    identity: FileIdentity,
    length: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
struct DirectoryStream {
    directory: Option<std::ptr::NonNull<libc::DIR>>,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl DirectoryStream {
    fn open(directory_fd: OwnedFd) -> Result<Self, AppError> {
        let raw_fd = directory_fd.into_raw_fd();
        let directory = unsafe { libc::fdopendir(raw_fd) };
        let Some(directory) = std::ptr::NonNull::new(directory) else {
            let error = std::io::Error::last_os_error();
            unsafe {
                drop(OwnedFd::from_raw_fd(raw_fd));
            }
            return Err(AppError::io(
                "provider.raw_member",
                "could not open provider raw directory stream",
                error,
            ));
        };
        Ok(Self {
            directory: Some(directory),
        })
    }

    fn as_ptr(&self) -> *mut libc::DIR {
        self.directory
            .expect("open directory stream retains its pointer")
            .as_ptr()
    }

    fn close(mut self) -> Result<(), AppError> {
        let directory = self
            .directory
            .take()
            .expect("open directory stream closes exactly once");
        if unsafe { libc::closedir(directory.as_ptr()) } != 0 {
            return Err(AppError::io(
                "provider.raw_member",
                "could not close provider raw directory stream",
                std::io::Error::last_os_error(),
            ));
        }
        Ok(())
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl Drop for DirectoryStream {
    fn drop(&mut self) {
        if let Some(directory) = self.directory.take() {
            unsafe {
                libc::closedir(directory.as_ptr());
            }
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RawEvidenceSummary {
    pub(super) capture_complete: bool,
    pub(super) stdout_sha256: String,
    pub(super) stderr_sha256: String,
    pub(super) final_message_sha256: Option<String>,
    pub(super) raw_archive_sha256: String,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Serialize)]
struct LaunchClaim<'a> {
    schema_version: &'static str,
    repository_id: &'a str,
    episode_id: &'a str,
    provider: ProviderId,
    provider_version: &'a str,
    provider_capabilities_sha256: &'a str,
    command_sha256: &'a str,
    prompt_sha256: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ExecutableIdentity {
    device: u64,
    inode: u64,
    length: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderCapabilitySnapshot {
    schema_version: String,
    provider: ProviderId,
    executable: PathBuf,
    executable_identity: ExecutableIdentity,
    version: String,
    exec_json: bool,
    output_last_message: bool,
    working_directory: bool,
    model: bool,
    profile: bool,
    sandbox: bool,
    approval_config: bool,
    resume_json: bool,
    app_server_schema: bool,
    capability_sha256: String,
}

#[derive(Serialize)]
struct LegacyCapabilityDigest<'a> {
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

impl ProviderCapabilitySnapshot {
    pub(super) fn capture_executable_identity(
        provider: ProviderId,
        executable: &Path,
    ) -> Result<ExecutableIdentity, AppError> {
        validate_executable_provider(provider, executable)?;
        executable_identity(executable)
    }

    pub(super) fn from_probed(
        capabilities: &ProviderCapabilities,
        probed_identity: ExecutableIdentity,
    ) -> Result<Self, AppError> {
        validate_raw_capabilities(capabilities)?;
        let executable_identity = executable_identity(&capabilities.executable)?;
        if executable_identity != probed_identity {
            return Err(AppError::invalid_input(
                "provider.executable_changed",
                "provider executable identity changed during capability probing",
            ));
        }
        let mut snapshot = Self {
            schema_version: capabilities.schema_version.clone(),
            provider: capabilities.provider,
            executable: capabilities.executable.clone(),
            executable_identity,
            version: capabilities.version.clone(),
            exec_json: capabilities.exec_json,
            output_last_message: capabilities.output_last_message,
            working_directory: capabilities.working_directory,
            model: capabilities.model,
            profile: capabilities.profile,
            sandbox: capabilities.sandbox,
            approval_config: capabilities.approval_config,
            resume_json: capabilities.resume_json,
            app_server_schema: capabilities.app_server_schema,
            capability_sha256: String::new(),
        };
        snapshot.capability_sha256 = sha256_prefixed(&snapshot.canonical_bytes());
        Ok(snapshot)
    }

    pub fn provider(&self) -> ProviderId {
        self.provider
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn capability_sha256(&self) -> &str {
        &self.capability_sha256
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"harp-provider-capability-snapshot-v1\0");
        encode_utf8_field(&mut bytes, b's', &self.schema_version);
        bytes.push(provider_tag(self.provider));
        encode_os_field(&mut bytes, b'e', self.executable.as_os_str());
        encode_executable_identity(&mut bytes, self.executable_identity);
        encode_utf8_field(&mut bytes, b'v', &self.version);
        for value in [
            self.exec_json,
            self.output_last_message,
            self.working_directory,
            self.model,
            self.profile,
            self.sandbox,
            self.approval_config,
            self.resume_json,
            self.app_server_schema,
        ] {
            bytes.push(u8::from(value));
        }
        bytes
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != PROVIDER_CAPABILITIES_SCHEMA
            || !valid_version(&self.version)
            || self.capability_sha256 != sha256_prefixed(&self.canonical_bytes())
        {
            return Err(invalid_capabilities(
                "provider capability snapshot identity is invalid",
            ));
        }
        validate_executable_provider(self.provider, &self.executable)?;
        let current = executable_identity(&self.executable)?;
        if current != self.executable_identity {
            return Err(AppError::invalid_input(
                "provider.executable_changed",
                "provider executable identity changed after capability probing",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderInvocation {
    schema_version: String,
    capabilities: ProviderCapabilitySnapshot,
    arguments: Vec<ProviderArgumentPlan>,
    working_directory: PathBuf,
    repository: RepositorySnapshot,
    stdin_bytes: Vec<u8>,
    command_plan_bytes: Vec<u8>,
    command_plan_sha256: String,
    prompt_sha256: String,
}

pub fn build_invocation(
    capabilities: &ProviderCapabilitySnapshot,
    repository_root: &Path,
    context: &ContextBundle,
    task: &str,
    options: &ProviderRunOptions,
) -> Result<ProviderInvocation, AppError> {
    validate_invocation_path(capabilities.executable(), "provider executable")?;
    validate_repository_root(repository_root)?;
    let repository = RepositorySnapshot::capture(repository_root)?;
    build_invocation_from_snapshot(
        capabilities,
        repository_root,
        &repository,
        context,
        task,
        options,
    )
}

pub(crate) fn build_invocation_from_snapshot(
    capabilities: &ProviderCapabilitySnapshot,
    repository_root: &Path,
    repository: &RepositorySnapshot,
    context: &ContextBundle,
    task: &str,
    options: &ProviderRunOptions,
) -> Result<ProviderInvocation, AppError> {
    validate_invocation_path(capabilities.executable(), "provider executable")?;
    validate_repository_root(repository_root)?;
    validate_repository_association(repository_root, repository)?;
    validate_capabilities(capabilities, options)?;
    validate_context(context)?;
    validate_task(task)?;
    validate_options(options)?;

    let mut arguments = vec![
        ProviderArgumentPlan::Literal(OsString::from("exec")),
        ProviderArgumentPlan::Literal(OsString::from("--cd")),
        ProviderArgumentPlan::Literal(repository_root.as_os_str().to_owned()),
        ProviderArgumentPlan::Literal(OsString::from("--json")),
        ProviderArgumentPlan::Literal(OsString::from("--output-last-message")),
        ProviderArgumentPlan::OutputMember(PathBuf::from(FINAL_MESSAGE_NAME)),
    ];
    if let Some(model) = options.model.as_deref() {
        arguments.push(ProviderArgumentPlan::Literal(OsString::from("--model")));
        arguments.push(ProviderArgumentPlan::Literal(OsString::from(model)));
    }
    if let Some(profile) = options.profile.as_deref() {
        arguments.push(ProviderArgumentPlan::Literal(OsString::from("--profile")));
        arguments.push(ProviderArgumentPlan::Literal(OsString::from(profile)));
    }
    if let Some(sandbox) = options.sandbox {
        arguments.push(ProviderArgumentPlan::Literal(OsString::from("--sandbox")));
        arguments.push(ProviderArgumentPlan::Literal(OsString::from(
            sandbox.as_str(),
        )));
    }
    if let Some(approval) = options.approval {
        arguments.push(ProviderArgumentPlan::Literal(OsString::from("--config")));
        arguments.push(ProviderArgumentPlan::Literal(OsString::from(format!(
            "approval_policy=\"{}\"",
            approval.as_str()
        ))));
    }
    arguments.push(ProviderArgumentPlan::Literal(OsString::from("-")));

    let stdin_bytes = render_prompt(context, task);
    let command_plan_bytes =
        encode_provider_command_plan(capabilities.executable().as_os_str(), &arguments);
    let command_plan_sha256 = sha256_prefixed(&command_plan_bytes);
    let prompt_sha256 = sha256_prefixed(&stdin_bytes);

    Ok(ProviderInvocation {
        schema_version: PROVIDER_INVOCATION_SCHEMA.to_owned(),
        capabilities: capabilities.clone(),
        arguments,
        working_directory: repository_root.to_owned(),
        repository: repository.clone(),
        stdin_bytes,
        command_plan_bytes,
        command_plan_sha256,
        prompt_sha256,
    })
}

impl ProviderInvocation {
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    pub fn provider(&self) -> ProviderId {
        self.capabilities.provider
    }

    #[cfg(test)]
    pub(crate) fn provider_version(&self) -> &str {
        self.capabilities.version()
    }

    #[cfg(test)]
    pub(crate) fn provider_capabilities_sha256(&self) -> &str {
        self.capabilities.capability_sha256()
    }

    pub fn executable(&self) -> &Path {
        self.capabilities.executable()
    }

    pub fn argument_plan(&self) -> &[ProviderArgumentPlan] {
        &self.arguments
    }

    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    pub fn repository(&self) -> &RepositorySnapshot {
        &self.repository
    }

    pub fn stdin_bytes(&self) -> &[u8] {
        &self.stdin_bytes
    }

    pub fn command_plan_bytes(&self) -> &[u8] {
        &self.command_plan_bytes
    }

    pub fn command_plan_sha256(&self) -> &str {
        &self.command_plan_sha256
    }

    pub fn prompt_sha256(&self) -> &str {
        &self.prompt_sha256
    }

    pub fn final_message_member(&self) -> &Path {
        Path::new(FINAL_MESSAGE_NAME)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[allow(dead_code)]
    pub(crate) fn materialize(
        &self,
        raw_path: &EpisodeProviderRawPath,
    ) -> Result<MaterializedProviderInvocation, AppError> {
        if raw_path.provider != self.provider() {
            return Err(invocation_path(
                "provider raw-directory path does not match invocation provider",
            ));
        }
        self.capabilities.validate()?;
        validate_repository_root(&self.working_directory)?;
        validate_repository_association(&self.working_directory, &self.repository)?;
        raw_path.validate()?;
        let final_message_path = raw_path.path.join(FINAL_MESSAGE_NAME);
        let arguments = self
            .arguments
            .iter()
            .map(|argument| match argument {
                ProviderArgumentPlan::Literal(value) => Ok(value.clone()),
                ProviderArgumentPlan::OutputMember(member)
                    if member == Path::new(FINAL_MESSAGE_NAME) =>
                {
                    Ok(final_message_path.as_os_str().to_owned())
                }
                ProviderArgumentPlan::OutputMember(_) => Err(invocation_path(
                    "invocation contains an unknown output member",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let command_bytes = encode_provider_command(self.executable().as_os_str(), &arguments);
        let command_sha256 = sha256_prefixed(&command_bytes);
        let launch_claim_bytes = json_bytes(&LaunchClaim {
            schema_version: LAUNCH_CLAIM_SCHEMA,
            repository_id: raw_path.repository_id()?,
            episode_id: raw_path.episode_id()?,
            provider: self.provider(),
            provider_version: self.capabilities.version(),
            provider_capabilities_sha256: self.capabilities.capability_sha256(),
            command_sha256: &command_sha256,
            prompt_sha256: self.prompt_sha256(),
        })?;
        Ok(MaterializedProviderInvocation {
            plan: self.clone(),
            raw_path: raw_path.clone(),
            arguments,
            final_message_path,
            command_bytes,
            command_sha256,
            launch_claim_bytes,
        })
    }
}

/// Deterministic provider raw path known before episode publication.
///
/// This type contains no filesystem authority. Execution requires combining a
/// materialized invocation with `EpisodeRawDirectoryAuthority` after episode
/// publication.
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpisodeProviderRawPath {
    provider: ProviderId,
    relative_path: PathBuf,
    path: PathBuf,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl EpisodeProviderRawPath {
    #[allow(dead_code)]
    pub(crate) fn new(
        provider: ProviderId,
        relative_path: PathBuf,
        path: PathBuf,
    ) -> Result<Self, AppError> {
        let raw_path = Self {
            provider,
            relative_path,
            path,
        };
        raw_path.validate()?;
        Ok(raw_path)
    }

    #[allow(dead_code)]
    pub(crate) fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    fn validate(&self) -> Result<(), AppError> {
        validate_provider_raw_relative_path(self.provider, &self.relative_path)?;
        validate_provider_raw_path(self.provider, &self.relative_path, &self.path)?;
        validate_existing_directory_ancestors(&self.path)
    }

    fn repository_id(&self) -> Result<&str, AppError> {
        self.relative_path
            .components()
            .nth(1)
            .and_then(|component| component.as_os_str().to_str())
            .ok_or_else(|| invocation_path("provider raw path has no repository identity"))
    }

    fn episode_id(&self) -> Result<&str, AppError> {
        self.relative_path
            .components()
            .nth(3)
            .and_then(|component| component.as_os_str().to_str())
            .ok_or_else(|| invocation_path("provider raw path has no episode identity"))
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaterializedProviderInvocation {
    plan: ProviderInvocation,
    raw_path: EpisodeProviderRawPath,
    arguments: Vec<OsString>,
    final_message_path: PathBuf,
    command_bytes: Vec<u8>,
    command_sha256: String,
    launch_claim_bytes: Vec<u8>,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl MaterializedProviderInvocation {
    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    pub fn final_message_path(&self) -> &Path {
        &self.final_message_path
    }

    pub fn command_bytes(&self) -> &[u8] {
        &self.command_bytes
    }

    pub fn command_sha256(&self) -> &str {
        &self.command_sha256
    }

    pub fn command_plan_sha256(&self) -> &str {
        self.plan.command_plan_sha256()
    }

    pub fn prompt_sha256(&self) -> &str {
        self.plan.prompt_sha256()
    }

    pub(crate) fn provider(&self) -> ProviderId {
        self.plan.provider()
    }

    pub(crate) fn provider_version(&self) -> &str {
        self.plan.capabilities.version()
    }

    pub(crate) fn provider_capabilities_sha256(&self) -> &str {
        self.plan.capabilities.capability_sha256()
    }

    pub(crate) fn repository(&self) -> &RepositorySnapshot {
        self.plan.repository()
    }

    pub(crate) fn raw_path(&self) -> &EpisodeProviderRawPath {
        &self.raw_path
    }

    pub fn bind(
        self,
        authority: EpisodeRawDirectoryAuthority,
    ) -> Result<BoundProviderInvocation, AppError> {
        if authority.raw_path != self.raw_path {
            return Err(invocation_path(
                "provider raw-directory authority does not match materialized invocation",
            ));
        }
        self.plan.capabilities.validate()?;
        let authority = HeldProviderRawDirectory::from_episode_directory(authority)?;
        authority.verify_before_spawn()?;
        Ok(BoundProviderInvocation {
            invocation: self,
            authority,
        })
    }
}

/// Opaque authority for one episode's provider raw directory.
///
/// Episode publication creates this token from the directory descriptor held
/// by `StateRoot`. Invocation binding consumes it; callers cannot construct one
/// from an arbitrary pathname.
#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Debug)]
pub struct EpisodeRawDirectoryAuthority {
    raw_path: EpisodeProviderRawPath,
    directory: OwnedFd,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl EpisodeRawDirectoryAuthority {
    #[allow(dead_code)]
    pub(crate) fn new(
        raw_path: EpisodeProviderRawPath,
        directory: OwnedFd,
    ) -> Result<Self, AppError> {
        raw_path.validate()?;
        Ok(Self {
            raw_path,
            directory,
        })
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
/// Consuming authority for one provider launch.
///
/// Command material remains private so callers must pass this token to
/// [`super::execute`] instead of spawning the provider independently.
///
/// ```compile_fail
/// use harp::context_control::provider::BoundProviderInvocation;
///
/// fn bypass_harp(invocation: BoundProviderInvocation) {
///     let _ = invocation.executable();
/// }
/// ```
pub struct BoundProviderInvocation {
    invocation: MaterializedProviderInvocation,
    authority: HeldProviderRawDirectory,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl std::fmt::Debug for BoundProviderInvocation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundProviderInvocation")
            .finish_non_exhaustive()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Debug)]
pub(super) struct ClaimedProviderInvocation {
    invocation: MaterializedProviderInvocation,
    authority: HeldProviderRawDirectory,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub struct SealedProviderEvidence {
    invocation: MaterializedProviderInvocation,
    authority: HeldProviderRawDirectory,
    entries: Vec<SealedRawMember>,
    summary: RawEvidenceSummary,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl std::fmt::Debug for SealedProviderEvidence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SealedProviderEvidence")
            .finish_non_exhaustive()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SealedRawMember {
    member: ProviderRawMember,
    version: RawFileVersion,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl BoundProviderInvocation {
    /// Revalidate immediately before spawning the provider.
    ///
    /// Harp verifies the executable, repository snapshot, raw-directory route,
    /// and absent output leaf adjacent to `spawn`. Same-UID code can still
    /// replace the executable or repository after this final check, and can
    /// rewrite evidence after Harp exits; those races are outside the V1
    /// authenticity guarantee.
    fn verify_before_spawn(&self) -> Result<(), AppError> {
        self.invocation.plan.capabilities.validate()?;
        validate_repository_root(self.invocation.plan.working_directory())?;
        validate_repository_association(
            self.invocation.plan.working_directory(),
            self.invocation.plan.repository(),
        )?;
        self.authority.verify_before_spawn()
    }

    pub(super) fn claim_launch(self) -> Result<ClaimedProviderInvocation, AppError> {
        self.verify_before_spawn()?;
        let mut claim = self
            .authority
            .create_raw_member(ProviderRawMember::LaunchClaim)?;
        claim
            .write_all(&self.invocation.launch_claim_bytes)
            .map_err(|error| {
                AppError::io(
                    "provider.launch_claim",
                    "could not write provider launch claim",
                    error,
                )
            })?;
        claim.sync_all().map_err(|error| {
            AppError::io(
                "provider.launch_claim",
                "could not synchronize provider launch claim",
                error,
            )
        })?;
        self.authority.sync_directory()?;
        match self
            .authority
            .read_raw_member(ProviderRawMember::LaunchClaim, 4096)?
        {
            Some(bytes) if bytes == self.invocation.launch_claim_bytes => {
                Ok(ClaimedProviderInvocation {
                    invocation: self.invocation,
                    authority: self.authority,
                })
            }
            _ => Err(invocation_path(
                "provider launch claim changed while it was published",
            )),
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl ClaimedProviderInvocation {
    pub(super) fn executable(&self) -> &Path {
        self.invocation.plan.executable()
    }

    pub(super) fn arguments(&self) -> &[OsString] {
        self.invocation.arguments()
    }

    pub(super) fn working_directory(&self) -> &Path {
        self.invocation.plan.working_directory()
    }

    pub(super) fn stdin_bytes(&self) -> &[u8] {
        self.invocation.plan.stdin_bytes()
    }

    pub(super) fn verify_before_spawn(&self) -> Result<(), AppError> {
        self.invocation.plan.capabilities.validate()?;
        validate_repository_root(self.working_directory())?;
        validate_repository_association(
            self.working_directory(),
            self.invocation.plan.repository(),
        )?;
        self.authority.verify_before_spawn()
    }

    pub(super) fn create_raw_member(&self, member: ProviderRawMember) -> Result<File, AppError> {
        self.authority.create_raw_member(member)
    }

    pub(super) fn read_raw_member_bounded(
        &self,
        member: ProviderRawMember,
        max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AppError> {
        self.authority.read_raw_member(member, max_bytes)
    }

    pub(super) fn sync_raw_directory(&self) -> Result<(), AppError> {
        self.authority.sync_directory()
    }

    pub(super) fn seal(self, capture_complete: bool) -> Result<SealedProviderEvidence, AppError> {
        self.seal_after(capture_complete, || {})
    }

    #[cfg(test)]
    fn seal_with_revalidation_hook(
        self,
        capture_complete: bool,
        before_final_revalidation: impl FnOnce(),
    ) -> Result<SealedProviderEvidence, AppError> {
        self.seal_after(capture_complete, before_final_revalidation)
    }

    fn seal_after(
        self,
        capture_complete: bool,
        before_final_revalidation: impl FnOnce(),
    ) -> Result<SealedProviderEvidence, AppError> {
        let (entries, summary) =
            self.authenticate_raw_evidence(capture_complete, before_final_revalidation)?;
        Ok(SealedProviderEvidence {
            invocation: self.invocation,
            authority: self.authority,
            entries,
            summary,
        })
    }

    fn authenticate_raw_evidence(
        &self,
        capture_complete: bool,
        before_final_revalidation: impl FnOnce(),
    ) -> Result<(Vec<SealedRawMember>, RawEvidenceSummary), AppError> {
        self.invocation.plan.capabilities.validate()?;
        validate_repository_root(self.working_directory())?;
        validate_repository_identity(self.working_directory(), self.invocation.plan.repository())?;
        self.authority.verify_namespace()?;
        let entries = self.authority.list_raw_members()?;
        let allowed = PROVIDER_RAW_MEMBERS
            .iter()
            .map(|member| member.name())
            .collect::<std::collections::BTreeSet<_>>();
        if entries
            .iter()
            .any(|entry| !allowed.contains(entry.as_str()))
        {
            return Err(raw_evidence(
                "provider raw directory contains an unexpected member",
            ));
        }

        let launch = self
            .read_evidence_member(ProviderRawMember::LaunchClaim, 4096)?
            .ok_or_else(|| raw_evidence("provider launch claim is missing"))?;
        if launch.bytes != self.invocation.launch_claim_bytes {
            return Err(raw_evidence(
                "provider launch claim does not match the bound invocation",
            ));
        }
        let stdout = self
            .read_evidence_member(ProviderRawMember::StdoutJsonl, MAX_STDOUT_BYTES)?
            .ok_or_else(|| raw_evidence("provider stdout capture is missing"))?;
        let stderr = self
            .read_evidence_member(ProviderRawMember::StderrBin, MAX_STDERR_BYTES)?
            .ok_or_else(|| raw_evidence("provider stderr capture is missing"))?;
        let final_message =
            self.read_evidence_member(ProviderRawMember::FinalMessage, MAX_FINAL_MESSAGE_BYTES)?;
        let archive = self
            .read_evidence_member(ProviderRawMember::RawArchive, MAX_RAW_ARCHIVE_BYTES)?
            .ok_or_else(|| raw_evidence("provider raw archive is missing"))?;
        let members_manifest = self
            .read_evidence_member(ProviderRawMember::RawMembersManifest, 4096)?
            .ok_or_else(|| raw_evidence("provider raw-members manifest is missing"))?;

        if !capture_complete {
            return Err(raw_evidence(
                "incomplete provider capture cannot publish a completion receipt",
            ));
        }
        validate_raw_members_manifest(
            &members_manifest.bytes,
            &stdout,
            &stderr,
            final_message.as_ref(),
        )?;
        validate_canonical_raw_archive(&archive.bytes, &stdout, &stderr, final_message.as_ref())?;

        let mut sealed = vec![
            sealed_member(&launch),
            sealed_member(&stdout),
            sealed_member(&stderr),
            sealed_member(&archive),
            sealed_member(&members_manifest),
        ];
        if let Some(final_message) = &final_message {
            sealed.push(sealed_member(final_message));
        }
        sealed.sort_by_key(|member| member.member.name());
        before_final_revalidation();
        self.authority.revalidate_sealed_members(&sealed)?;
        if self.authority.list_raw_members()? != entries {
            return Err(raw_evidence(
                "provider raw directory changed during evidence verification",
            ));
        }
        self.authority.verify_namespace()?;
        Ok((
            sealed,
            RawEvidenceSummary {
                capture_complete,
                stdout_sha256: stdout.sha256,
                stderr_sha256: stderr.sha256,
                final_message_sha256: final_message.map(|member| member.sha256),
                raw_archive_sha256: archive.sha256,
            },
        ))
    }

    fn read_evidence_member(
        &self,
        member: ProviderRawMember,
        max_bytes: usize,
    ) -> Result<Option<RawMemberEvidence>, AppError> {
        self.authority
            .read_raw_member_evidence(member, max_bytes)
            .map(|evidence| {
                evidence.map(|(bytes, version)| RawMemberEvidence {
                    member,
                    sha256: sha256_prefixed(&bytes),
                    bytes,
                    version,
                })
            })
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl SealedProviderEvidence {
    pub(crate) fn provider(&self) -> ProviderId {
        self.invocation.provider()
    }

    pub(crate) fn provider_version(&self) -> &str {
        self.invocation.plan.capabilities.version()
    }

    pub(crate) fn provider_capabilities_sha256(&self) -> &str {
        self.invocation.plan.capabilities.capability_sha256()
    }

    pub(crate) fn command_sha256(&self) -> &str {
        self.invocation.command_sha256()
    }

    pub(crate) fn prompt_sha256(&self) -> &str {
        self.invocation.prompt_sha256()
    }

    pub(crate) fn repository(&self) -> &RepositorySnapshot {
        self.invocation.repository()
    }

    pub(crate) fn raw_path(&self) -> &EpisodeProviderRawPath {
        self.invocation.raw_path()
    }

    pub(super) fn summary(&self) -> &RawEvidenceSummary {
        &self.summary
    }

    pub(crate) fn capture_complete(&self) -> bool {
        self.summary.capture_complete
    }

    pub(crate) fn stdout_sha256(&self) -> &str {
        &self.summary.stdout_sha256
    }

    pub(crate) fn stderr_sha256(&self) -> &str {
        &self.summary.stderr_sha256
    }

    pub(crate) fn final_message_sha256(&self) -> Option<&str> {
        self.summary.final_message_sha256.as_deref()
    }

    pub(crate) fn raw_archive_sha256(&self) -> &str {
        &self.summary.raw_archive_sha256
    }

    pub(crate) fn revalidate(&self) -> Result<(), AppError> {
        self.authority.revalidate_sealed_members(&self.entries)
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn sealed_member(member: &RawMemberEvidence) -> SealedRawMember {
    SealedRawMember {
        member: member.member,
        version: member.version,
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Debug)]
struct HeldProviderRawDirectory {
    provider: ProviderId,
    relative_path: PathBuf,
    path: PathBuf,
    fd: OwnedFd,
    route: Vec<HeldPathComponent>,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug)]
struct HeldPathComponent {
    name: std::ffi::CString,
    identity: FileIdentity,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl HeldProviderRawDirectory {
    fn from_episode_directory(authority: EpisodeRawDirectoryAuthority) -> Result<Self, AppError> {
        let EpisodeRawDirectoryAuthority {
            raw_path,
            directory,
        } = authority;
        let EpisodeProviderRawPath {
            provider,
            relative_path,
            path,
        } = raw_path;
        validate_provider_raw_relative_path(provider, &relative_path)?;
        validate_provider_raw_path(provider, &relative_path, &path)?;
        validate_private_directory(&directory)?;
        let held_identity = fd_identity(&directory)?;
        let components = absolute_path_components(&path)?;
        let mut current = open_filesystem_root()?;
        let mut route = Vec::with_capacity(components.len());
        for name in components {
            let fd = open_directory_at(current.as_raw_fd(), &name)?;
            let identity = fd_identity(&fd)?;
            route.push(HeldPathComponent { name, identity });
            current = fd;
        }
        if fd_identity(&current)? != held_identity {
            return Err(invocation_path(
                "provider raw-directory pathname does not resolve to episode authority",
            ));
        }
        let authority = Self {
            provider,
            relative_path,
            path,
            fd: directory,
            route,
        };
        authority.verify_before_spawn()?;
        Ok(authority)
    }

    fn verify_namespace(&self) -> Result<(), AppError> {
        validate_provider_raw_relative_path(self.provider, &self.relative_path)?;
        validate_provider_raw_path(self.provider, &self.relative_path, &self.path)?;
        let mut current = open_filesystem_root()?;
        for component in &self.route {
            let next = open_directory_at(current.as_raw_fd(), &component.name)?;
            if fd_identity(&next)? != component.identity {
                return Err(invocation_path(
                    "provider raw-directory pathname identity changed",
                ));
            }
            current = next;
        }
        if fd_identity(&self.fd)?
            != self
                .route
                .last()
                .expect("absolute provider raw path has components")
                .identity
        {
            return Err(invocation_path(
                "held provider raw-directory identity changed",
            ));
        }
        validate_private_directory(&self.fd)?;
        Ok(())
    }

    fn list_raw_members(&self) -> Result<Vec<String>, AppError> {
        self.verify_namespace()?;
        let current =
            std::ffi::CString::new(".").expect("fixed provider directory component has no NUL");
        let directory_fd = open_directory_at(self.fd.as_raw_fd(), &current)?;
        let directory = DirectoryStream::open(directory_fd)?;
        let result = (|| {
            let mut names = Vec::new();
            loop {
                set_errno_zero();
                let entry = unsafe { libc::readdir(directory.as_ptr()) };
                if entry.is_null() {
                    let error = std::io::Error::last_os_error();
                    if error.raw_os_error().unwrap_or(0) == 0 {
                        break;
                    }
                    return Err(AppError::io(
                        "provider.raw_member",
                        "could not list provider raw directory",
                        error,
                    ));
                }
                let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
                if name.to_bytes() == b"." || name.to_bytes() == b".." {
                    continue;
                }
                let name = name.to_str().map_err(|_| {
                    invocation_path("provider raw directory contains a non-UTF-8 member")
                })?;
                names.push(name.to_owned());
            }
            names.sort();
            Ok(names)
        })();
        directory.close()?;
        let names = result?;
        self.verify_namespace()?;
        Ok(names)
    }

    fn verify_before_spawn(&self) -> Result<(), AppError> {
        self.verify_namespace()?;
        ensure_leaf_absent(self.fd.as_raw_fd(), FINAL_MESSAGE_NAME)?;
        self.verify_namespace()
    }

    fn create_raw_member(&self, member: ProviderRawMember) -> Result<File, AppError> {
        self.verify_namespace()?;
        let name = raw_member_name(member);
        let fd = unsafe {
            libc::openat(
                self.fd.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EEXIST) {
                return Err(AppError::invalid_input(
                    "provider.output_exists",
                    format!("provider raw member {} already exists", member.name()),
                ));
            }
            return Err(AppError::io(
                "provider.raw_member",
                "could not create provider raw member descriptor-relatively",
                error,
            ));
        }
        let file = unsafe { File::from_raw_fd(fd) };
        if unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
            return Err(AppError::io(
                "provider.raw_member",
                "could not set provider raw-member mode",
                std::io::Error::last_os_error(),
            ));
        }
        validate_open_raw_member(self.fd.as_raw_fd(), &name, &file, member)?;
        self.verify_namespace()?;
        Ok(file)
    }

    fn read_raw_member(
        &self,
        member: ProviderRawMember,
        max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AppError> {
        self.read_raw_member_evidence(member, max_bytes)
            .map(|evidence| evidence.map(|(bytes, _version)| bytes))
    }

    fn read_raw_member_evidence(
        &self,
        member: ProviderRawMember,
        max_bytes: usize,
    ) -> Result<Option<(Vec<u8>, RawFileVersion)>, AppError> {
        let read_limit = max_bytes.checked_add(1).ok_or_else(|| {
            AppError::invalid_input(
                raw_member_limit_code(member),
                "provider raw-member byte limit is too large",
            )
        })?;
        self.verify_namespace()?;
        let name = raw_member_name(member);
        let Some(mut file) = open_raw_member_at(self.fd.as_raw_fd(), &name, member)? else {
            self.verify_namespace()?;
            return Ok(None);
        };
        let before = validate_open_raw_member(self.fd.as_raw_fd(), &name, &file, member)?;
        let before_version = raw_file_version(&before);
        let max_bytes_u64 = u64::try_from(max_bytes).unwrap_or(u64::MAX);
        if before.len() > max_bytes_u64 {
            return Err(AppError::invalid_input(
                raw_member_limit_code(member),
                format!(
                    "provider raw member {} exceeds {max_bytes} bytes",
                    member.name()
                ),
            ));
        }
        let mut bytes = Vec::with_capacity(
            usize::try_from(before.len().min(max_bytes_u64)).unwrap_or(max_bytes),
        );
        std::io::Read::by_ref(&mut file)
            .take(u64::try_from(read_limit).unwrap_or(u64::MAX))
            .read_to_end(&mut bytes)
            .map_err(|error| {
                AppError::io(
                    raw_member_error_code(member),
                    "could not read provider raw member",
                    error,
                )
            })?;
        if bytes.len() > max_bytes {
            return Err(AppError::invalid_input(
                raw_member_limit_code(member),
                format!(
                    "provider raw member {} exceeds {max_bytes} bytes",
                    member.name()
                ),
            ));
        }
        let after = validate_open_raw_member(self.fd.as_raw_fd(), &name, &file, member)?;
        if raw_file_version(&after) != before_version || bytes.len() as u64 != after.len() {
            return Err(invocation_path(format!(
                "provider raw member {} changed while it was read",
                member.name()
            )));
        }
        self.verify_namespace()?;
        Ok(Some((bytes, raw_file_version(&after))))
    }

    fn raw_member_version(&self, member: ProviderRawMember) -> Result<RawFileVersion, AppError> {
        self.verify_namespace()?;
        let name = raw_member_name(member);
        let file = open_raw_member_at(self.fd.as_raw_fd(), &name, member)?.ok_or_else(|| {
            raw_evidence(format!("provider raw member {} is missing", member.name()))
        })?;
        let metadata = validate_open_raw_member(self.fd.as_raw_fd(), &name, &file, member)?;
        self.verify_namespace()?;
        Ok(raw_file_version(&metadata))
    }

    fn revalidate_sealed_members(&self, entries: &[SealedRawMember]) -> Result<(), AppError> {
        self.verify_namespace()?;
        let actual = self.list_raw_members()?;
        let expected = entries
            .iter()
            .map(|entry| entry.member.name().to_owned())
            .collect::<Vec<_>>();
        if actual != expected {
            return Err(raw_evidence(
                "sealed provider raw directory member set changed",
            ));
        }
        for entry in entries {
            if self.raw_member_version(entry.member)? != entry.version {
                return Err(raw_evidence(format!(
                    "sealed provider raw member {} changed",
                    entry.member.name()
                )));
            }
        }
        self.verify_namespace()
    }

    fn sync_directory(&self) -> Result<(), AppError> {
        self.verify_namespace()?;
        if unsafe { libc::fsync(self.fd.as_raw_fd()) } != 0 {
            return Err(AppError::io(
                "provider.raw_member",
                "could not synchronize provider raw directory",
                std::io::Error::last_os_error(),
            ));
        }
        self.verify_namespace()
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn raw_member_name(member: ProviderRawMember) -> std::ffi::CString {
    std::ffi::CString::new(member.name()).expect("fixed provider raw member contains no NUL")
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn open_raw_member_at(
    parent: i32,
    name: &std::ffi::CStr,
    member: ProviderRawMember,
) -> Result<Option<File>, AppError> {
    let fd = unsafe {
        libc::openat(
            parent,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd >= 0 {
        return Ok(Some(unsafe { File::from_raw_fd(fd) }));
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENOENT) {
        Ok(None)
    } else if error.raw_os_error() == Some(libc::ELOOP) {
        Err(invocation_path(format!(
            "provider raw member {} must not be a symlink",
            member.name()
        )))
    } else {
        Err(AppError::io(
            raw_member_error_code(member),
            "could not open provider raw member descriptor-relatively",
            error,
        ))
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_open_raw_member(
    parent: i32,
    name: &std::ffi::CStr,
    file: &File,
    member: ProviderRawMember,
) -> Result<fs::Metadata, AppError> {
    let metadata = file.metadata().map_err(|error| {
        AppError::io(
            raw_member_error_code(member),
            "could not inspect provider raw member",
            error,
        )
    })?;
    let mode = metadata.permissions().mode() & 0o777;
    let mode_valid = mode == 0o600 || (member == ProviderRawMember::FinalMessage && mode == 0o644);
    if !metadata.is_file()
        || !mode_valid
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.nlink() != 1
    {
        return Err(invocation_path(format!(
            "provider raw member {} must be a regular file with the expected owner and mode",
            member.name()
        )));
    }
    if entry_identity(parent, name, member)? != metadata_identity(&metadata) {
        return Err(invocation_path(format!(
            "provider raw-member pathname identity changed: {}",
            member.name()
        )));
    }
    Ok(metadata)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn raw_file_version(metadata: &fs::Metadata) -> RawFileVersion {
    RawFileVersion {
        identity: metadata_identity(metadata),
        length: metadata.len(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_raw_members_manifest(
    bytes: &[u8],
    stdout: &RawMemberEvidence,
    stderr: &RawMemberEvidence,
    final_message: Option<&RawMemberEvidence>,
) -> Result<(), AppError> {
    let expected = canonical_raw_members_manifest(
        &stdout.bytes,
        &stderr.bytes,
        final_message.map(|member| member.bytes.as_slice()),
    );
    if bytes != expected {
        return Err(raw_evidence(
            "provider raw-members manifest does not match captured members",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_canonical_raw_archive(
    bytes: &[u8],
    stdout: &RawMemberEvidence,
    stderr: &RawMemberEvidence,
    final_message: Option<&RawMemberEvidence>,
) -> Result<(), AppError> {
    let expected = canonical_raw_archive(
        &stdout.bytes,
        &stderr.bytes,
        final_message.map(|member| member.bytes.as_slice()),
    )?;
    if bytes != expected {
        return Err(raw_evidence(
            "provider raw archive is not the canonical archive of captured members",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) fn canonical_raw_members_manifest(
    stdout: &[u8],
    stderr: &[u8],
    final_message: Option<&[u8]>,
) -> Vec<u8> {
    let mut rows = vec![(STDERR_NAME, stderr), (STDOUT_NAME, stdout)];
    if let Some(final_message) = final_message {
        rows.push((FINAL_MESSAGE_NAME, final_message));
    }
    rows.sort_by_key(|(name, _)| *name);
    let mut bytes = Vec::new();
    for (name, member) in rows {
        use std::fmt::Write as _;
        let mut row = String::new();
        writeln!(
            row,
            "{}\t{}\t{}",
            sha256_prefixed(member),
            member.len(),
            name
        )
        .expect("writing a String cannot fail");
        bytes.extend_from_slice(row.as_bytes());
    }
    bytes
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(super) fn canonical_raw_archive(
    stdout: &[u8],
    stderr: &[u8],
    final_message: Option<&[u8]>,
) -> Result<Vec<u8>, AppError> {
    let mut members = vec![(STDERR_NAME, stderr), (STDOUT_NAME, stdout)];
    if let Some(final_message) = final_message {
        members.push((FINAL_MESSAGE_NAME, final_message));
    }
    members.sort_by_key(|(name, _)| *name);

    let mut bytes = Vec::new();
    {
        let encoder = flate2::GzBuilder::new()
            .mtime(0)
            .operating_system(255)
            .write(&mut bytes, flate2::Compression::default());
        let mut archive = tar::Builder::new(encoder);
        for (name, member) in members {
            let mut header = tar::Header::new_old();
            header.set_size(u64::try_from(member.len()).map_err(|_| {
                raw_evidence("provider raw member size does not fit archive format")
            })?);
            header.set_mode(0o644);
            header.set_uid(0);
            header.set_gid(0);
            header.set_mtime(0);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_cksum();
            archive
                .append_data(&mut header, name, std::io::Cursor::new(member))
                .map_err(|error| {
                    AppError::io(
                        "provider.raw_archive",
                        "could not build canonical provider raw archive",
                        error,
                    )
                })?;
        }
        archive.finish().map_err(|error| {
            AppError::io(
                "provider.raw_archive",
                "could not finish canonical provider raw archive",
                error,
            )
        })?;
        let encoder = archive.into_inner().map_err(|error| {
            AppError::io(
                "provider.raw_archive",
                "could not recover canonical provider archive encoder",
                error,
            )
        })?;
        encoder.finish().map_err(|error| {
            AppError::io(
                "provider.raw_archive",
                "could not finish canonical provider archive compression",
                error,
            )
        })?;
    }
    Ok(bytes)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const fn raw_member_error_code(member: ProviderRawMember) -> &'static str {
    match member {
        ProviderRawMember::FinalMessage => "provider.final_message",
        ProviderRawMember::LaunchClaim => "provider.launch_claim",
        _ => "provider.raw_member",
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const fn raw_member_limit_code(member: ProviderRawMember) -> &'static str {
    match member {
        ProviderRawMember::FinalMessage => "provider.final_message_limit",
        ProviderRawMember::LaunchClaim => "provider.launch_claim_limit",
        _ => "provider.raw_member_limit",
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_provider_raw_relative_path(provider: ProviderId, path: &Path) -> Result<(), AppError> {
    validate_invocation_path(path, "provider raw-directory relative path")?;
    let components = path.components().collect::<Vec<_>>();
    let normal = components
        .iter()
        .map(|component| match component {
            Component::Normal(value) => Ok(*value),
            _ => Err(invocation_path(
                "provider raw-directory authority must use canonical relative components",
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if normal.len() != 6
        || normal[0] != OsStr::new("repositories")
        || !is_repository_id(normal[1])
        || normal[2] != OsStr::new("episodes")
        || !is_episode_id(normal[3])
        || normal[4] != OsStr::new("raw")
        || normal[5] != OsStr::new(provider_name(provider))
    {
        return Err(invocation_path(
            "provider raw-directory authority is not an episode provider namespace",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_provider_raw_path(
    provider: ProviderId,
    relative_path: &Path,
    path: &Path,
) -> Result<(), AppError> {
    validate_invocation_path(path, "provider raw directory")?;
    if !path.is_absolute() || !has_only_canonical_components(path) || !path.ends_with(relative_path)
    {
        return Err(invocation_path(
            "provider raw directory must be an absolute canonical path ending in the episode namespace",
        ));
    }
    if path.file_name() != Some(OsStr::new(provider_name(provider)))
        || path.parent().and_then(Path::file_name) != Some(OsStr::new("raw"))
    {
        return Err(invocation_path(format!(
            "provider raw-directory authority must end in raw/{provider}"
        )));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_repository_id(value: &OsStr) -> bool {
    value
        .to_str()
        .and_then(|value| value.strip_prefix("sha256:"))
        .is_some_and(is_lowercase_sha256_hex)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_episode_id(value: &OsStr) -> bool {
    let Some(value) = value.to_str() else {
        return false;
    };
    let mut parts = value.split('-');
    parts.next() == Some("ep")
        && parts.next().is_some_and(|part| is_lowercase_hex(part, 16))
        && parts.next().is_some_and(|part| is_lowercase_hex(part, 8))
        && parts.next().is_some_and(|part| is_lowercase_hex(part, 8))
        && parts.next().is_none()
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn absolute_path_components(path: &Path) -> Result<Vec<std::ffi::CString>, AppError> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(component) => {
                components.push(std::ffi::CString::new(component.as_bytes()).map_err(|_| {
                    invocation_path("provider raw-directory component contains NUL")
                })?);
            }
            Component::Prefix(_) | Component::CurDir | Component::ParentDir => {
                return Err(invocation_path(
                    "provider raw directory must use canonical components",
                ));
            }
        }
    }
    if components.is_empty() {
        return Err(invocation_path(
            "provider raw directory must not be filesystem root",
        ));
    }
    Ok(components)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_existing_directory_ancestors(path: &Path) -> Result<(), AppError> {
    let mut current = open_filesystem_root()?;
    for component in absolute_path_components(path)? {
        let fd = unsafe {
            libc::openat(
                current.as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd >= 0 {
            current = unsafe { OwnedFd::from_raw_fd(fd) };
            continue;
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(());
        }
        return Err(invocation_path(format!(
            "provider raw-directory path contains a symlink or non-directory ancestor: {error}"
        )));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn open_filesystem_root() -> Result<OwnedFd, AppError> {
    let fd = unsafe {
        libc::open(
            c"/".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        Err(AppError::io(
            "provider.invocation_path",
            "could not open filesystem root",
            std::io::Error::last_os_error(),
        ))
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn open_directory_at(parent: i32, name: &std::ffi::CStr) -> Result<OwnedFd, AppError> {
    let fd = unsafe {
        libc::openat(
            parent,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        Err(AppError::io(
            "provider.invocation_path",
            "could not open provider raw-directory component",
            std::io::Error::last_os_error(),
        ))
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn fd_identity(fd: &OwnedFd) -> Result<FileIdentity, AppError> {
    let metadata = File::from(duplicate_fd(fd.as_raw_fd())?)
        .metadata()
        .map_err(|error| {
            AppError::io(
                "provider.invocation_path",
                "could not inspect held provider raw directory",
                error,
            )
        })?;
    Ok(metadata_identity(&metadata))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn metadata_identity(metadata: &fs::Metadata) -> FileIdentity {
    FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn duplicate_fd(fd: i32) -> Result<OwnedFd, AppError> {
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        Err(AppError::io(
            "provider.invocation_path",
            "could not duplicate provider raw-directory descriptor",
            std::io::Error::last_os_error(),
        ))
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
    }
}

#[cfg(target_os = "macos")]
fn set_errno_zero() {
    unsafe {
        *libc::__error() = 0;
    }
}

#[cfg(target_os = "linux")]
fn set_errno_zero() {
    unsafe {
        *libc::__errno_location() = 0;
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_private_directory(fd: &OwnedFd) -> Result<(), AppError> {
    let metadata = File::from(duplicate_fd(fd.as_raw_fd())?)
        .metadata()
        .map_err(|error| {
            AppError::io(
                "provider.invocation_path",
                "could not inspect provider raw-directory metadata",
                error,
            )
        })?;
    if !metadata.is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(invocation_path(
            "provider raw-directory authority must be an owner-only directory owned by the current user",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn ensure_leaf_absent(parent: i32, member: &str) -> Result<(), AppError> {
    let member =
        std::ffi::CString::new(member).expect("fixed final-message member contains no NUL");
    let mut stat = std::mem::MaybeUninit::<libc::stat>::zeroed();
    let result = unsafe {
        libc::fstatat(
            parent,
            member.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result == 0 {
        return Err(AppError::invalid_input(
            "provider.output_exists",
            "provider final-message member already exists",
        ));
    }
    let error = std::io::Error::last_os_error();
    if error.raw_os_error() == Some(libc::ENOENT) {
        Ok(())
    } else {
        Err(AppError::io(
            "provider.invocation_path",
            "could not inspect provider final-message member",
            error,
        ))
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn entry_identity(
    parent: i32,
    name: &std::ffi::CStr,
    member: ProviderRawMember,
) -> Result<FileIdentity, AppError> {
    let mut stat = std::mem::MaybeUninit::<libc::stat>::zeroed();
    let result = unsafe {
        libc::fstatat(
            parent,
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        return Err(AppError::io(
            raw_member_error_code(member),
            "could not revalidate provider raw member",
            std::io::Error::last_os_error(),
        ));
    }
    let stat = unsafe { stat.assume_init() };
    Ok(FileIdentity {
        device: stat.st_dev as u64,
        inode: stat.st_ino,
    })
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn has_only_canonical_components(path: &Path) -> bool {
    if path
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return false;
    }
    has_canonical_native_spelling(path)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn has_canonical_native_spelling(path: &Path) -> bool {
    let bytes = path.as_os_str().as_bytes();
    bytes.first() == Some(&b'/')
        && bytes.last() != Some(&b'/')
        && !bytes.windows(2).any(|window| window == b"//")
        && !bytes[1..]
            .split(|byte| *byte == b'/')
            .any(|component| component.is_empty() || component == b"." || component == b"..")
}

fn validate_capabilities(
    capabilities: &ProviderCapabilitySnapshot,
    options: &ProviderRunOptions,
) -> Result<(), AppError> {
    capabilities.validate()?;
    for (requested, supported, name) in [
        (options.model.is_some(), capabilities.model, "model"),
        (options.profile.is_some(), capabilities.profile, "profile"),
        (options.sandbox.is_some(), capabilities.sandbox, "sandbox"),
        (
            options.approval.is_some(),
            capabilities.approval_config,
            "approval",
        ),
    ] {
        if requested && !supported {
            return Err(AppError::invalid_input(
                "provider.unsupported_option",
                format!(
                    "{} capability snapshot does not support requested {name} option",
                    capabilities.provider
                ),
            ));
        }
    }
    for (supported, name) in [
        (capabilities.exec_json, "--json"),
        (capabilities.output_last_message, "--output-last-message"),
        (capabilities.working_directory, "--cd"),
        (capabilities.approval_config, "--config"),
    ] {
        if !supported {
            return Err(AppError::invalid_input(
                "provider.missing_capability",
                format!(
                    "{} capability snapshot is missing required option {name}",
                    capabilities.provider
                ),
            ));
        }
    }
    Ok(())
}

fn validate_context(context: &ContextBundle) -> Result<(), AppError> {
    if context.schema_version != CONTEXT_BUNDLE_SCHEMA {
        return Err(AppError::invalid_input(
            "provider.invocation_context",
            format!(
                "unsupported context bundle schema: {}",
                context.schema_version
            ),
        ));
    }
    if !is_release_id(&context.release_id)
        || context.rendered_markdown.contains('\0')
        || !is_lowercase_sha256_hex(&context.rendered_context_sha256)
        || context.rendered_context_sha256 != sha256_hex(context.rendered_markdown.as_bytes())
    {
        return Err(AppError::invalid_input(
            "provider.invocation_context",
            "context bundle identity or rendered-context digest is invalid",
        ));
    }
    Ok(())
}

fn validate_task(task: &str) -> Result<(), AppError> {
    if task.trim().is_empty() || task.contains('\0') {
        return Err(AppError::invalid_input(
            "provider.invocation_task",
            "provider task must be nonempty and contain no NUL bytes",
        ));
    }
    Ok(())
}

fn validate_options(options: &ProviderRunOptions) -> Result<(), AppError> {
    for (value, label) in [
        (options.model.as_deref(), "model"),
        (options.profile.as_deref(), "profile"),
    ] {
        if let Some(value) = value {
            if value.is_empty()
                || value.len() > MAX_OPTION_BYTES
                || value.contains('\0')
                || value.starts_with('-')
            {
                return Err(AppError::invalid_input(
                    "provider.invocation_option",
                    format!(
                        "{label} must be nonempty, at most {MAX_OPTION_BYTES} bytes, contain no NUL, and not resemble a provider flag"
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn render_prompt(context: &ContextBundle, task: &str) -> Vec<u8> {
    let task_sha256 = sha256_hex(task.as_bytes());
    let mut prompt = Vec::with_capacity(context.rendered_markdown.len() + task.len() + 320);
    prompt.extend_from_slice(PROMPT_ENCODING_MAGIC);
    encode_prompt_field(
        &mut prompt,
        "context-release",
        context.release_id.as_bytes(),
    );
    encode_prompt_field(
        &mut prompt,
        "context-sha256",
        context.rendered_context_sha256.as_bytes(),
    );
    encode_prompt_field(&mut prompt, "context", context.rendered_markdown.as_bytes());
    encode_prompt_field(&mut prompt, "task-sha256", task_sha256.as_bytes());
    encode_prompt_field(&mut prompt, "task", task.as_bytes());
    prompt
}

fn encode_prompt_field(target: &mut Vec<u8>, name: &str, bytes: &[u8]) {
    target.extend_from_slice(name.as_bytes());
    target.push(b' ');
    target.extend_from_slice(bytes.len().to_string().as_bytes());
    target.push(b'\n');
    target.extend_from_slice(bytes);
    target.push(b'\n');
}

/// Canonically encode an executable and its materialized native argument vector.
///
/// Each native string is length-prefixed, so argument boundaries and non-UTF-8
/// Unix bytes are preserved without shell quoting or lossy conversion.
pub fn encode_provider_command(executable: &OsStr, arguments: &[OsString]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(COMMAND_ENCODING_MAGIC);
    encode_os_field(&mut bytes, b'e', executable);
    encode_u64(&mut bytes, arguments.len() as u64);
    for argument in arguments {
        encode_os_field(&mut bytes, b'a', argument);
    }
    bytes
}

fn encode_provider_command_plan(executable: &OsStr, arguments: &[ProviderArgumentPlan]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(COMMAND_PLAN_ENCODING_MAGIC);
    encode_os_field(&mut bytes, b'e', executable);
    encode_u64(&mut bytes, arguments.len() as u64);
    for argument in arguments {
        match argument {
            ProviderArgumentPlan::Literal(value) => encode_os_field(&mut bytes, b'l', value),
            ProviderArgumentPlan::OutputMember(member) => {
                encode_os_field(&mut bytes, b'o', member.as_os_str());
            }
        }
    }
    bytes
}

fn encode_utf8_field(target: &mut Vec<u8>, tag: u8, value: &str) {
    encode_bytes_field(target, tag, value.as_bytes());
}

fn encode_os_field(target: &mut Vec<u8>, tag: u8, value: &OsStr) {
    #[cfg(unix)]
    encode_bytes_field(target, tag, value.as_bytes());
    #[cfg(windows)]
    {
        let mut bytes = Vec::new();
        for unit in value.encode_wide() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        encode_bytes_field(target, tag, &bytes);
    }
    #[cfg(not(any(unix, windows)))]
    encode_bytes_field(target, tag, value.to_string_lossy().as_bytes());
}

fn encode_bytes_field(target: &mut Vec<u8>, tag: u8, value: &[u8]) {
    target.push(tag);
    encode_u64(target, value.len() as u64);
    target.extend_from_slice(value);
}

fn encode_u64(target: &mut Vec<u8>, value: u64) {
    target.extend_from_slice(&value.to_be_bytes());
}

fn encode_i64(target: &mut Vec<u8>, value: i64) {
    target.extend_from_slice(&value.to_be_bytes());
}

fn encode_executable_identity(target: &mut Vec<u8>, identity: ExecutableIdentity) {
    for value in [identity.device, identity.inode, identity.length] {
        encode_u64(target, value);
    }
    for value in [
        identity.modified_seconds,
        identity.modified_nanoseconds,
        identity.changed_seconds,
        identity.changed_nanoseconds,
    ] {
        encode_i64(target, value);
    }
}

fn validate_raw_capabilities(capabilities: &ProviderCapabilities) -> Result<(), AppError> {
    if capabilities.schema_version != PROVIDER_CAPABILITIES_SCHEMA
        || !valid_version(&capabilities.version)
    {
        return Err(invalid_capabilities(
            "provider capability snapshot schema or version is invalid",
        ));
    }
    validate_executable_provider(capabilities.provider, &capabilities.executable)?;
    let payload = LegacyCapabilityDigest {
        schema_version: &capabilities.schema_version,
        provider: capabilities.provider,
        executable: provider_executable_name(capabilities.provider)
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
    if capabilities.capability_sha256 != sha256_prefixed(&canonical) {
        return Err(invalid_capabilities(
            "provider capability digest does not match probed content",
        ));
    }
    for (supported, name) in [
        (capabilities.exec_json, "--json"),
        (capabilities.output_last_message, "--output-last-message"),
        (capabilities.working_directory, "--cd"),
        (capabilities.approval_config, "--config"),
    ] {
        if !supported {
            return Err(AppError::invalid_input(
                "provider.missing_capability",
                format!(
                    "{} capability snapshot is missing required option {name}",
                    capabilities.provider
                ),
            ));
        }
    }
    Ok(())
}

fn validate_executable_provider(provider: ProviderId, executable: &Path) -> Result<(), AppError> {
    validate_invocation_path(executable, "provider executable")?;
    if !executable.is_absolute()
        || executable.file_name() != Some(provider_executable_name(provider))
    {
        return Err(AppError::invalid_input(
            "provider.executable_mismatch",
            format!("{provider} capabilities do not identify the expected absolute executable"),
        ));
    }
    Ok(())
}

fn provider_executable_name(provider: ProviderId) -> &'static OsStr {
    OsStr::new(match provider {
        ProviderId::Trae => "traecli",
        ProviderId::Codex => "codex",
    })
}

#[cfg(unix)]
fn executable_identity(path: &Path) -> Result<ExecutableIdentity, AppError> {
    let metadata = fs::metadata(path).map_err(|error| {
        AppError::io(
            "provider.executable",
            "could not inspect provider executable",
            error,
        )
    })?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(invalid_capabilities(
            "provider executable must be a regular executable file",
        ));
    }
    Ok(ExecutableIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        length: metadata.len(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    })
}

#[cfg(not(unix))]
fn executable_identity(path: &Path) -> Result<ExecutableIdentity, AppError> {
    let metadata = fs::metadata(path).map_err(|error| {
        AppError::io(
            "provider.executable",
            "could not inspect provider executable",
            error,
        )
    })?;
    if !metadata.is_file() {
        return Err(invalid_capabilities(
            "provider executable must be a regular file",
        ));
    }
    Ok(ExecutableIdentity {
        device: 0,
        inode: 0,
        length: metadata.len(),
        modified_seconds: 0,
        modified_nanoseconds: 0,
        changed_seconds: 0,
        changed_nanoseconds: 0,
    })
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
        && prerelease.is_none_or(|value| valid_version_identifiers(value, true))
        && build.is_none_or(|value| valid_version_identifiers(value, false))
}

fn valid_numeric_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_VERSION_IDENTIFIER_BYTES
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn valid_version_identifiers(value: &str, reject_numeric_leading_zero: bool) -> bool {
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

fn is_release_id(value: &str) -> bool {
    value
        .strip_prefix("sha256-")
        .is_some_and(is_lowercase_sha256_hex)
}

fn is_lowercase_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    format!("sha256:{}", sha256_hex(bytes))
}

fn provider_tag(provider: ProviderId) -> u8 {
    match provider {
        ProviderId::Trae => 1,
        ProviderId::Codex => 2,
    }
}

fn validate_invocation_path(path: &Path, label: &str) -> Result<(), AppError> {
    if path.as_os_str().is_empty() || os_contains_nul(path.as_os_str()) {
        return Err(invocation_path(format!(
            "{label} must be nonempty and contain no NUL bytes"
        )));
    }
    Ok(())
}

fn validate_repository_root(path: &Path) -> Result<(), AppError> {
    validate_invocation_path(path, "repository root")?;
    if os_starts_with_hyphen(path.as_os_str()) || !path.is_absolute() {
        return Err(invocation_path(
            "repository root must be an absolute canonical directory and not resemble a provider flag",
        ));
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    if !has_only_canonical_components(path) {
        return Err(invocation_path(
            "repository root must use canonical absolute path components",
        ));
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        invocation_path(format!(
            "repository root must be an existing canonical directory: {error}"
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(invocation_path(
            "repository root must be an existing non-symlink directory",
        ));
    }
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    validate_directory_route(path, "repository root")?;
    Ok(())
}

fn validate_repository_association(
    repository_root: &Path,
    expected: &RepositorySnapshot,
) -> Result<(), AppError> {
    let actual = RepositorySnapshot::capture(repository_root)?;
    if &actual != expected {
        return Err(AppError::invalid_input(
            "provider.repository_changed",
            "repository snapshot does not match the invocation working directory",
        ));
    }
    Ok(())
}

fn validate_repository_identity(
    repository_root: &Path,
    expected: &RepositorySnapshot,
) -> Result<(), AppError> {
    if repository_id(repository_root)? != expected.repository_id {
        return Err(AppError::invalid_input(
            "provider.repository_changed",
            "repository identity does not match the invocation working directory",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn os_starts_with_hyphen(value: &OsStr) -> bool {
    value.as_bytes().first() == Some(&b'-')
}

#[cfg(windows)]
fn os_starts_with_hyphen(value: &OsStr) -> bool {
    value.encode_wide().next() == Some(u16::from(b'-'))
}

#[cfg(not(any(unix, windows)))]
fn os_starts_with_hyphen(value: &OsStr) -> bool {
    value.to_string_lossy().starts_with('-')
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_directory_route(path: &Path, label: &str) -> Result<(), AppError> {
    let components = absolute_path_components(path)?;
    let mut current = open_filesystem_root()?;
    for component in components {
        current = open_directory_at(current.as_raw_fd(), &component).map_err(|error| {
            invocation_path(format!(
                "{label} contains a non-directory or symlink component: {error}"
            ))
        })?;
    }
    Ok(())
}

#[cfg(unix)]
fn os_contains_nul(value: &OsStr) -> bool {
    value.as_bytes().contains(&0)
}

#[cfg(windows)]
fn os_contains_nul(value: &OsStr) -> bool {
    value.encode_wide().any(|unit| unit == 0)
}

#[cfg(not(any(unix, windows)))]
fn os_contains_nul(value: &OsStr) -> bool {
    value.to_string_lossy().contains('\0')
}

fn invocation_path(message: impl Into<String>) -> AppError {
    AppError::invalid_input("provider.invocation_path", message)
}

fn invalid_capabilities(message: impl Into<String>) -> AppError {
    AppError::invalid_input("provider.invocation_capabilities", message)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn raw_evidence(message: impl Into<String>) -> AppError {
    AppError::invalid_input("provider.raw_evidence", message)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
const fn provider_name(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::Trae => "trae",
        ProviderId::Codex => "codex",
    }
}

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::fs::{self, File};
    use std::io::Write as _;
    use std::os::fd::OwnedFd;
    use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
    use std::os::unix::fs::{symlink, PermissionsExt as _};
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use super::{
        build_invocation, build_invocation_from_snapshot, canonical_raw_archive,
        canonical_raw_members_manifest, div_ceil, encode_provider_command,
        encode_provider_command_plan, executable_identity, provider_executable_name,
        sha256_prefixed, tar_member_bound, ApprovalPolicy, EpisodeProviderRawPath,
        EpisodeRawDirectoryAuthority, LegacyCapabilityDigest, ProviderArgumentPlan,
        ProviderCapabilitySnapshot, ProviderInvocation, ProviderRawMember, ProviderRunOptions,
        SandboxMode, COMMAND_ENCODING_MAGIC, COMMAND_PLAN_ENCODING_MAGIC,
        DEFLATE_STORED_BLOCK_BYTES, DEFLATE_STORED_BLOCK_OVERHEAD, GZIP_WRAPPER_BYTES,
        MAX_FINAL_MESSAGE_BYTES, MAX_RAW_ARCHIVE_BYTES, MAX_RAW_TAR_BYTES, MAX_STDERR_BYTES,
        MAX_STDOUT_BYTES, PROMPT_ENCODING_MAGIC, TAR_END_BYTES,
    };
    use crate::context_control::canonical::sha256_hex;
    use crate::context_control::context::ContextBundle;
    use crate::context_control::provider::ProviderCapabilities;
    use crate::context_control::routing::RouteDecision;
    use crate::context_control::{
        ProviderId, RepositorySnapshot, WorkflowId, CONTEXT_BUNDLE_SCHEMA,
        PROVIDER_CAPABILITIES_SCHEMA, PROVIDER_INVOCATION_SCHEMA,
    };

    const RELEASE_ID: &str =
        "sha256-1111111111111111111111111111111111111111111111111111111111111111";
    const RENDERED_CONTEXT: &str = "Context line.\n";
    const RENDERED_CONTEXT_SHA256: &str =
        "1a9c05b1b69fe63bcf22d3426c8ad380b8ff5d3bc692c8850f5e27ad9d877ef4";
    const TASK: &str = "Fix the failing test; keep \"quotes\" literal.";
    const TASK_SHA256: &str = "a7443ab615a15fc585de118ed6bf9bac65669a58b23c5f952c8bec0d30a20d4c";
    const REPOSITORY_ID: &str =
        "sha256:2222222222222222222222222222222222222222222222222222222222222222";
    const EPISODE_ID: &str = "ep-0123456789abcdef-01234567-89abcdef";

    #[test]
    fn invocation_plan_is_complete_before_episode_publication() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());

        assert_eq!(plan.schema_version(), PROVIDER_INVOCATION_SCHEMA);
        assert_eq!(plan.repository(), &fixture.repository_snapshot);
        assert_eq!(plan.final_message_member(), Path::new("final_message.bin"));
        assert_eq!(
            plan.command_plan_sha256(),
            sha256_prefixed(plan.command_plan_bytes())
        );
        assert_eq!(plan.prompt_sha256(), sha256_prefixed(plan.stdin_bytes()));
        assert!(!fixture.raw_path(ProviderId::Trae).exists());
    }

    #[test]
    fn launch_claim_is_create_only_and_binds_the_materialized_invocation() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let first = fixture.bind(&plan, ProviderId::Trae);
        let second = fixture.bind(&plan, ProviderId::Trae);
        let expected = first.invocation.launch_claim_bytes.clone();

        let first = first.claim_launch().unwrap();

        assert_eq!(
            first
                .read_raw_member_bounded(ProviderRawMember::LaunchClaim, 4096)
                .unwrap(),
            Some(expected)
        );
        assert_eq!(
            second.claim_launch().unwrap_err().code(),
            "provider.output_exists"
        );
    }

    #[test]
    fn bound_invocation_debug_output_does_not_expose_command_material() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let bound = fixture.bind(&plan, ProviderId::Trae);

        let debug = format!("{bound:?}");

        assert_eq!(debug, "BoundProviderInvocation { .. }");
        assert!(!debug.contains(
            fixture
                .executable(ProviderId::Trae)
                .to_string_lossy()
                .as_ref()
        ));
        assert!(!debug.contains(TASK));
        assert!(!debug.contains(&fixture.repository.to_string_lossy().into_owned()));
    }

    #[test]
    fn sealed_evidence_debug_output_does_not_expose_command_material() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let claimed = fixture
            .bind(&plan, ProviderId::Trae)
            .claim_launch()
            .unwrap();
        let stdout = b"stdout\n";
        let stderr = b"stderr\n";
        let archive = canonical_raw_archive(stdout, stderr, None).unwrap();
        let manifest = canonical_raw_members_manifest(stdout, stderr, None);
        for (member, bytes) in [
            (ProviderRawMember::StdoutJsonl, stdout.as_slice()),
            (ProviderRawMember::StderrBin, stderr.as_slice()),
            (ProviderRawMember::RawArchive, archive.as_slice()),
            (ProviderRawMember::RawMembersManifest, manifest.as_slice()),
        ] {
            let mut file = claimed.create_raw_member(member).unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        claimed.sync_raw_directory().unwrap();
        let evidence = claimed.seal(true).unwrap();

        let debug = format!("{evidence:?}");

        assert_eq!(debug, "SealedProviderEvidence { .. }");
        assert!(!debug.contains(
            fixture
                .executable(ProviderId::Trae)
                .to_string_lossy()
                .as_ref()
        ));
        assert!(!debug.contains(TASK));
        assert!(!debug.contains(&fixture.repository.to_string_lossy().into_owned()));
    }

    #[test]
    fn actual_episode_repository_id_materializes_before_publication() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let raw_path = fixture.raw_path_plan(ProviderId::Trae);

        let materialized = plan.materialize(&raw_path).unwrap();

        assert_eq!(
            raw_path.relative_path(),
            Path::new("repositories")
                .join(&fixture.repository_snapshot.repository_id)
                .join("episodes")
                .join(EPISODE_ID)
                .join("raw")
                .join("trae")
        );
        assert_eq!(
            materialized.command_sha256(),
            sha256_prefixed(materialized.command_bytes())
        );
        assert_ne!(
            materialized.command_sha256(),
            plan.command_plan_sha256(),
            "the actual final-message path must participate in the executed command digest"
        );
        assert!(!fixture.raw_path(ProviderId::Trae).exists());
    }

    #[test]
    fn invocation_reuses_the_authoritative_preflight_snapshot() {
        let fixture = InvocationFixture::new();
        let authoritative = RepositorySnapshot::capture(&fixture.repository).unwrap();

        let plan = build_invocation_from_snapshot(
            &fixture.snapshot(ProviderId::Trae),
            &fixture.repository,
            &authoritative,
            &context_bundle(),
            TASK,
            &ProviderRunOptions::default(),
        )
        .unwrap();

        assert_eq!(plan.repository(), &authoritative);
    }

    #[test]
    fn invocation_rejects_repository_drift_instead_of_swapping_the_snapshot() {
        let fixture = InvocationFixture::new();
        let authoritative = RepositorySnapshot::capture(&fixture.repository).unwrap();
        fs::write(
            fixture.repository.join("tracked.txt"),
            "changed after preflight\n",
        )
        .unwrap();

        let error = build_invocation_from_snapshot(
            &fixture.snapshot(ProviderId::Trae),
            &fixture.repository,
            &authoritative,
            &context_bundle(),
            TASK,
            &ProviderRunOptions::default(),
        )
        .unwrap_err();

        assert_eq!(error.code(), "provider.repository_changed");
    }

    #[test]
    fn repository_root_must_be_an_absolute_canonical_real_directory() {
        let fixture = InvocationFixture::new();
        let snapshot = fixture.snapshot(ProviderId::Trae);
        let file = fixture.root.join("not-a-directory");
        fs::write(&file, b"file").unwrap();
        let real = fixture.root.join("real-repository");
        fs::create_dir(&real).unwrap();
        let symlinked = fixture.root.join("symlinked-repository");
        symlink(&real, &symlinked).unwrap();

        for repository in [
            PathBuf::new(),
            PathBuf::from("--ignore-rules"),
            PathBuf::from("relative/repository"),
            file,
            symlinked,
            fixture.repository.join("..").join("repository"),
        ] {
            assert_eq!(
                build_invocation(
                    &snapshot,
                    &repository,
                    &context_bundle(),
                    TASK,
                    &ProviderRunOptions::default(),
                )
                .unwrap_err()
                .code(),
                "provider.invocation_path",
                "{repository:?}"
            );
        }
    }

    #[test]
    fn both_providers_bind_exact_ordered_arguments_and_identical_prompt_bytes() {
        let fixture = InvocationFixture::new();
        let options = ProviderRunOptions {
            model: Some("gpt-5.4".to_owned()),
            profile: Some("review".to_owned()),
            sandbox: Some(SandboxMode::WorkspaceWrite),
            approval: Some(ApprovalPolicy::OnRequest),
        };
        let mut prompts = Vec::new();

        for provider in [ProviderId::Trae, ProviderId::Codex] {
            let plan = fixture.plan(provider, &options);
            let expected = vec![
                OsString::from("exec"),
                OsString::from("--cd"),
                fixture.repository.as_os_str().to_owned(),
                OsString::from("--json"),
                OsString::from("--output-last-message"),
                fixture.final_message_path(provider).into_os_string(),
                OsString::from("--model"),
                OsString::from("gpt-5.4"),
                OsString::from("--profile"),
                OsString::from("review"),
                OsString::from("--sandbox"),
                OsString::from("workspace-write"),
                OsString::from("--config"),
                OsString::from("approval_policy=\"on-request\""),
                OsString::from("-"),
            ];
            let materialized = plan.materialize(&fixture.raw_path_plan(provider)).unwrap();

            assert_eq!(plan.provider(), provider);
            assert_eq!(plan.executable(), fixture.executable(provider));
            assert_eq!(materialized.arguments(), expected);
            assert_eq!(plan.working_directory(), fixture.repository);
            assert!(!materialized.final_message_path().exists());
            assert_no_forbidden_arguments(materialized.arguments());
            prompts.push(plan.stdin_bytes().to_vec());
            materialized.bind(fixture.authority(provider)).unwrap();
        }

        assert_eq!(prompts[0], prompts[1]);
        assert_eq!(
            decode_prompt(&prompts[0]),
            vec![
                ("context-release".to_owned(), RELEASE_ID.as_bytes().to_vec()),
                (
                    "context-sha256".to_owned(),
                    RENDERED_CONTEXT_SHA256.as_bytes().to_vec()
                ),
                ("context".to_owned(), RENDERED_CONTEXT.as_bytes().to_vec()),
                ("task-sha256".to_owned(), TASK_SHA256.as_bytes().to_vec()),
                ("task".to_owned(), TASK.as_bytes().to_vec()),
            ]
        );
    }

    #[test]
    fn optional_arguments_preserve_fixed_prefix_and_stdin_marker() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let materialized = plan
            .materialize(&fixture.raw_path_plan(ProviderId::Trae))
            .unwrap();

        assert_eq!(
            materialized.arguments(),
            [
                OsString::from("exec"),
                OsString::from("--cd"),
                fixture.repository.as_os_str().to_owned(),
                OsString::from("--json"),
                OsString::from("--output-last-message"),
                fixture
                    .final_message_path(ProviderId::Trae)
                    .into_os_string(),
                OsString::from("-"),
            ]
        );
        materialized
            .bind(fixture.authority(ProviderId::Trae))
            .unwrap()
            .claim_launch()
            .unwrap();
    }

    #[test]
    fn sandbox_and_approval_modes_have_exact_wire_values() {
        assert_eq!(SandboxMode::ReadOnly.as_str(), "read-only");
        assert_eq!(SandboxMode::WorkspaceWrite.as_str(), "workspace-write");
        assert_eq!(SandboxMode::DangerFullAccess.as_str(), "danger-full-access");
        assert_eq!(ApprovalPolicy::Untrusted.as_str(), "untrusted");
        assert_eq!(ApprovalPolicy::OnRequest.as_str(), "on-request");
        assert_eq!(ApprovalPolicy::Never.as_str(), "never");
    }

    #[test]
    fn quoted_option_values_remain_single_native_arguments() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(
            ProviderId::Trae,
            &ProviderRunOptions {
                model: Some("model with spaces".to_owned()),
                profile: Some("profile\"quoted".to_owned()),
                ..ProviderRunOptions::default()
            },
        );
        let materialized = plan
            .materialize(&fixture.raw_path_plan(ProviderId::Trae))
            .unwrap();

        assert_eq!(materialized.arguments()[7], OsStr::new("model with spaces"));
        assert_eq!(materialized.arguments()[9], OsStr::new("profile\"quoted"));
        assert!(!materialized
            .arguments()
            .iter()
            .any(|argument| argument == OsStr::new("'model with spaces'")));
        materialized
            .bind(fixture.authority(ProviderId::Trae))
            .unwrap();
    }

    #[test]
    fn prompt_length_framing_recovers_old_delimiters_exactly() {
        let fixture = InvocationFixture::new();
        let markdown = "before </harp-context>\nafter </user-task>\n";
        let task = "task </user-task> \"quotes\"\n--dangerously-bypass-approvals-and-sandbox";
        let context = context_with_markdown(markdown);
        let plan = build_invocation(
            &fixture.snapshot(ProviderId::Codex),
            &fixture.repository,
            &context,
            task,
            &ProviderRunOptions::default(),
        )
        .unwrap();
        let fields = decode_prompt(plan.stdin_bytes());

        assert_eq!(field(&fields, "context"), markdown.as_bytes());
        assert_eq!(field(&fields, "task"), task.as_bytes());
        assert_eq!(
            field(&fields, "task-sha256"),
            sha256_hex(task.as_bytes()).as_bytes()
        );
        assert_no_forbidden_argument_plan(plan.argument_plan());
    }

    #[test]
    fn prompt_bytes_and_hash_are_stable() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let expected = format!(
            "harp-provider-prompt-v1\n\
             context-release 71\n{RELEASE_ID}\n\
             context-sha256 64\n{RENDERED_CONTEXT_SHA256}\n\
             context 14\n{RENDERED_CONTEXT}\n\
             task-sha256 64\n{TASK_SHA256}\n\
             task {}\n{TASK}\n",
            TASK.len()
        )
        .into_bytes();

        assert_eq!(plan.stdin_bytes(), expected);
        assert_eq!(
            plan.prompt_sha256(),
            "sha256:b06c2bfada0605b28c4eee856dbcaed5ecb0330b1894fd66c6b3015009cdb61b"
        );
    }

    #[test]
    fn command_encoding_preserves_actual_argument_boundaries() {
        let first = encode_provider_command(
            OsStr::new("/bin/provider"),
            &[OsString::from("ab"), OsString::from("c")],
        );
        let second = encode_provider_command(
            OsStr::new("/bin/provider"),
            &[OsString::from("a"), OsString::from("bc")],
        );
        assert_ne!(first, second);
        assert_eq!(
            decode_command(&first, COMMAND_ENCODING_MAGIC),
            (
                b"/bin/provider".to_vec(),
                vec![(b'a', b"ab".to_vec()), (b'a', b"c".to_vec())]
            )
        );
    }

    #[test]
    fn command_plan_encoding_distinguishes_literal_and_output_member() {
        let literal = encode_provider_command_plan(
            OsStr::new("/bin/provider"),
            &[ProviderArgumentPlan::Literal(OsString::from(
                "final_message.bin",
            ))],
        );
        let output = encode_provider_command_plan(
            OsStr::new("/bin/provider"),
            &[ProviderArgumentPlan::OutputMember(PathBuf::from(
                "final_message.bin",
            ))],
        );

        assert_ne!(literal, output);
        assert_eq!(
            decode_command(&literal, COMMAND_PLAN_ENCODING_MAGIC),
            (
                b"/bin/provider".to_vec(),
                vec![(b'l', b"final_message.bin".to_vec())]
            )
        );
    }

    #[test]
    fn command_encoding_preserves_non_utf8_native_bytes() {
        let executable = OsString::from_vec(b"/tmp/provider-\xff".to_vec());
        let argument = OsString::from_vec(b"argument-\x80".to_vec());
        let encoded = encode_provider_command(&executable, std::slice::from_ref(&argument));
        let (decoded_executable, decoded_arguments) =
            decode_command(&encoded, COMMAND_ENCODING_MAGIC);

        assert_eq!(decoded_executable, executable.as_bytes());
        assert_eq!(decoded_arguments, [(b'a', argument.as_bytes().to_vec())]);
    }

    #[test]
    fn rejects_invalid_context_identity_digest_and_nul_markdown() {
        let fixture = InvocationFixture::new();
        let snapshot = fixture.snapshot(ProviderId::Trae);
        let mut cases = Vec::new();
        let mut bad_release = context_bundle();
        bad_release.release_id = String::new();
        cases.push(bad_release);
        let mut uppercase_release = context_bundle();
        uppercase_release.release_id =
            "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned();
        cases.push(uppercase_release);
        let mut tampered = context_bundle();
        tampered.rendered_markdown.push_str("tampered");
        cases.push(tampered);
        cases.push(context_with_markdown("context\0suffix"));

        for context in cases {
            assert_eq!(
                build_invocation(
                    &snapshot,
                    &fixture.repository,
                    &context,
                    TASK,
                    &ProviderRunOptions::default(),
                )
                .unwrap_err()
                .code(),
                "provider.invocation_context"
            );
        }
    }

    #[test]
    fn rejects_empty_or_nul_tasks() {
        let fixture = InvocationFixture::new();
        let snapshot = fixture.snapshot(ProviderId::Trae);
        for task in ["", " \t\n", "task\0suffix"] {
            assert_eq!(
                build_invocation(
                    &snapshot,
                    &fixture.repository,
                    &context_bundle(),
                    task,
                    &ProviderRunOptions::default(),
                )
                .unwrap_err()
                .code(),
                "provider.invocation_task"
            );
        }
    }

    #[test]
    fn rejects_oversized_nul_and_flag_shaped_options() {
        let fixture = InvocationFixture::new();
        let snapshot = fixture.snapshot(ProviderId::Trae);
        for options in [
            ProviderRunOptions {
                model: Some("x".repeat(257)),
                ..ProviderRunOptions::default()
            },
            ProviderRunOptions {
                profile: Some("\u{00e9}".repeat(129)),
                ..ProviderRunOptions::default()
            },
            ProviderRunOptions {
                model: Some("model\0suffix".to_owned()),
                ..ProviderRunOptions::default()
            },
            ProviderRunOptions {
                profile: Some("--ignore-rules".to_owned()),
                ..ProviderRunOptions::default()
            },
            ProviderRunOptions {
                model: Some("--dangerously-bypass-approvals-and-sandbox".to_owned()),
                ..ProviderRunOptions::default()
            },
        ] {
            assert_eq!(
                build_invocation(
                    &snapshot,
                    &fixture.repository,
                    &context_bundle(),
                    TASK,
                    &options,
                )
                .unwrap_err()
                .code(),
                "provider.invocation_option"
            );
        }
    }

    #[test]
    fn rejects_each_unsupported_typed_option() {
        let fixture = InvocationFixture::new();
        let cases = [
            (
                "model",
                ProviderRunOptions {
                    model: Some("gpt-5.4".to_owned()),
                    ..ProviderRunOptions::default()
                },
            ),
            (
                "profile",
                ProviderRunOptions {
                    profile: Some("review".to_owned()),
                    ..ProviderRunOptions::default()
                },
            ),
            (
                "sandbox",
                ProviderRunOptions {
                    sandbox: Some(SandboxMode::ReadOnly),
                    ..ProviderRunOptions::default()
                },
            ),
            (
                "approval_config",
                ProviderRunOptions {
                    approval: Some(ApprovalPolicy::Never),
                    ..ProviderRunOptions::default()
                },
            ),
        ];

        for (capability, options) in cases {
            let mut snapshot = fixture.snapshot(ProviderId::Trae);
            match capability {
                "model" => snapshot.model = false,
                "profile" => snapshot.profile = false,
                "sandbox" => snapshot.sandbox = false,
                "approval_config" => snapshot.approval_config = false,
                _ => unreachable!(),
            }
            reseal(&mut snapshot);
            assert_eq!(
                build_invocation(
                    &snapshot,
                    &fixture.repository,
                    &context_bundle(),
                    TASK,
                    &options,
                )
                .unwrap_err()
                .code(),
                "provider.unsupported_option"
            );
        }
    }

    #[test]
    fn rejects_incomplete_required_capability_snapshots() {
        let fixture = InvocationFixture::new();
        for capability in [
            "exec_json",
            "output_last_message",
            "working_directory",
            "approval_config",
        ] {
            let mut snapshot = fixture.snapshot(ProviderId::Trae);
            match capability {
                "exec_json" => snapshot.exec_json = false,
                "output_last_message" => snapshot.output_last_message = false,
                "working_directory" => snapshot.working_directory = false,
                "approval_config" => snapshot.approval_config = false,
                _ => unreachable!(),
            }
            reseal(&mut snapshot);
            assert_eq!(
                build_invocation(
                    &snapshot,
                    &fixture.repository,
                    &context_bundle(),
                    TASK,
                    &ProviderRunOptions::default(),
                )
                .unwrap_err()
                .code(),
                "provider.missing_capability"
            );
        }
    }

    #[test]
    fn rejects_invalid_raw_capability_version_digest_and_executable_match() {
        let fixture = InvocationFixture::new();

        let mut bad_version = fixture.raw_capabilities(ProviderId::Trae);
        bad_version.version = "01.2.3".to_owned();
        seal_raw_capabilities(&mut bad_version);
        assert_snapshot_error(&bad_version, "provider.invocation_capabilities");

        let mut bad_digest = fixture.raw_capabilities(ProviderId::Trae);
        bad_digest.capability_sha256 =
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned();
        assert_snapshot_error(&bad_digest, "provider.invocation_capabilities");

        let mut wrong_name = fixture.raw_capabilities(ProviderId::Trae);
        wrong_name.executable = fixture.executable(ProviderId::Codex).to_owned();
        seal_raw_capabilities(&mut wrong_name);
        assert_snapshot_error(&wrong_name, "provider.executable_mismatch");
    }

    #[test]
    fn rejects_snapshot_content_mutation_even_when_private_fields_are_changed() {
        let fixture = InvocationFixture::new();
        let mut snapshot = fixture.snapshot(ProviderId::Trae);
        snapshot.model = false;

        assert_eq!(
            build_invocation(
                &snapshot,
                &fixture.repository,
                &context_bundle(),
                TASK,
                &ProviderRunOptions::default(),
            )
            .unwrap_err()
            .code(),
            "provider.invocation_capabilities"
        );
    }

    #[test]
    fn executable_identity_and_full_path_are_bound_into_snapshot() {
        let fixture = InvocationFixture::new();
        let first = fixture.snapshot(ProviderId::Trae);
        let alternate_dir = fixture.root.join("alternate");
        fs::create_dir(&alternate_dir).unwrap();
        let alternate = alternate_dir.join("traecli");
        write_executable(&alternate);
        let mut raw = fixture.raw_capabilities(ProviderId::Trae);
        raw.executable = alternate;
        seal_raw_capabilities(&mut raw);
        let alternate_identity = executable_identity(&raw.executable).unwrap();
        let second = ProviderCapabilitySnapshot::from_probed(&raw, alternate_identity).unwrap();

        assert_ne!(first.canonical_bytes(), second.canonical_bytes());
        assert_ne!(first.capability_sha256(), second.capability_sha256());
    }

    #[test]
    fn rejects_executable_replacement_after_capability_probe() {
        let fixture = InvocationFixture::new();
        let snapshot = fixture.snapshot(ProviderId::Trae);
        let executable = fixture.executable(ProviderId::Trae);
        fs::rename(executable, fixture.root.join("old-traecli")).unwrap();
        write_executable(executable);

        assert_eq!(
            build_invocation(
                &snapshot,
                &fixture.repository,
                &context_bundle(),
                TASK,
                &ProviderRunOptions::default(),
            )
            .unwrap_err()
            .code(),
            "provider.executable_changed"
        );
    }

    #[test]
    fn rejects_provider_mismatch_descriptor_mismatch_and_path_escape() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        assert_eq!(
            plan.materialize(&fixture.raw_path_plan(ProviderId::Codex))
                .unwrap_err()
                .code(),
            "provider.invocation_path"
        );

        let actual_path = fixture.ensure_raw(ProviderId::Trae);
        let codex_path = fixture.ensure_raw(ProviderId::Codex);
        let materialized = plan
            .materialize(&fixture.raw_path_plan(ProviderId::Trae))
            .unwrap();
        let mismatched = authority(
            ProviderId::Trae,
            fixture.raw_relative(ProviderId::Trae),
            actual_path,
            &codex_path,
        );
        assert_eq!(
            materialized.bind(mismatched).unwrap_err().code(),
            "provider.invocation_path"
        );

        for relative in [
            PathBuf::from("outside"),
            fixture.raw_relative(ProviderId::Codex),
            fixture
                .raw_relative(ProviderId::Trae)
                .with_file_name("trae-prefix"),
            PathBuf::from(format!(
                "repositories/{REPOSITORY_ID}/episodes/{EPISODE_ID}/raw/Trae"
            )),
            PathBuf::from(format!(
                "repositories/{REPOSITORY_ID}/episodes/{EPISODE_ID}/raw/../raw/trae"
            )),
        ] {
            assert_eq!(
                EpisodeProviderRawPath::new(
                    ProviderId::Trae,
                    relative,
                    fixture.raw_path(ProviderId::Trae),
                )
                .unwrap_err()
                .code(),
                "provider.invocation_path"
            );
        }

        let wrong_repository_syntax = PathBuf::from(format!(
            "repositories/sha256-{}/episodes/{EPISODE_ID}/raw/trae",
            "2".repeat(64)
        ));
        assert_eq!(
            EpisodeProviderRawPath::new(
                ProviderId::Trae,
                wrong_repository_syntax,
                fixture.raw_path(ProviderId::Trae),
            )
            .unwrap_err()
            .code(),
            "provider.invocation_path"
        );
    }

    #[test]
    fn rejects_symlinked_route_and_noncanonical_absolute_path() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let raw = fixture.ensure_raw(ProviderId::Trae);
        let alias = fixture.root.join("alias");
        symlink(&fixture.root, &alias).unwrap();
        let alias_path = alias.join(fixture.raw_relative(ProviderId::Trae));
        assert_eq!(
            EpisodeProviderRawPath::new(
                ProviderId::Trae,
                fixture.raw_relative(ProviderId::Trae),
                alias_path,
            )
            .unwrap_err()
            .code(),
            "provider.invocation_path"
        );

        let noncanonical = fixture
            .root
            .join(".")
            .join(fixture.raw_relative(ProviderId::Trae));
        assert_eq!(
            EpisodeProviderRawPath::new(
                ProviderId::Trae,
                fixture.raw_relative(ProviderId::Trae),
                noncanonical,
            )
            .unwrap_err()
            .code(),
            "provider.invocation_path"
        );

        let raw_path = fixture.raw_path_plan(ProviderId::Trae);
        let materialized = plan.materialize(&raw_path).unwrap();
        let token = authority(
            ProviderId::Trae,
            fixture.raw_relative(ProviderId::Trae),
            raw.clone(),
            &raw,
        );
        materialized.bind(token).unwrap();
    }

    #[test]
    fn rejects_preexisting_final_message_file_or_symlink() {
        for symlink_leaf in [false, true] {
            let fixture = InvocationFixture::new();
            let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
            let raw = fixture.ensure_raw(ProviderId::Trae);
            let final_message = raw.join("final_message.bin");
            if symlink_leaf {
                let target = fixture.root.join("target");
                fs::write(&target, b"target").unwrap();
                symlink(target, final_message).unwrap();
            } else {
                fs::write(final_message, b"existing").unwrap();
            }

            let materialized = plan
                .materialize(&fixture.raw_path_plan(ProviderId::Trae))
                .unwrap();
            assert_eq!(
                materialized
                    .bind(fixture.authority(ProviderId::Trae))
                    .unwrap_err()
                    .code(),
                "provider.output_exists"
            );
        }
    }

    #[test]
    fn detects_raw_directory_replacement_before_spawn() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let bound = fixture.bind(&plan, ProviderId::Trae);
        let raw = fixture.raw_path(ProviderId::Trae);
        fs::rename(&raw, raw.with_file_name("trae-old")).unwrap();
        fs::create_dir(&raw).unwrap();
        fs::set_permissions(&raw, fs::Permissions::from_mode(0o700)).unwrap();

        assert_eq!(
            bound.verify_before_spawn().unwrap_err().code(),
            "provider.invocation_path"
        );
    }

    #[test]
    fn reads_final_message_through_held_directory_authority() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let materialized = plan
            .materialize(&fixture.raw_path_plan(ProviderId::Trae))
            .unwrap();
        let final_message_path = materialized.final_message_path().to_owned();
        let claimed = materialized
            .bind(fixture.authority(ProviderId::Trae))
            .unwrap()
            .claim_launch()
            .unwrap();

        assert_eq!(
            claimed
                .read_raw_member_bounded(ProviderRawMember::FinalMessage, 64)
                .unwrap(),
            None
        );
        assert_eq!(
            claimed
                .read_raw_member_bounded(ProviderRawMember::FinalMessage, usize::MAX)
                .unwrap_err()
                .code(),
            "provider.final_message_limit"
        );
        fs::write(final_message_path, b"final response").unwrap();
        assert_eq!(
            claimed
                .read_raw_member_bounded(ProviderRawMember::FinalMessage, 64)
                .unwrap(),
            Some(b"final response".to_vec())
        );
        assert_eq!(
            claimed
                .read_raw_member_bounded(ProviderRawMember::FinalMessage, 4)
                .unwrap_err()
                .code(),
            "provider.final_message_limit"
        );
    }

    #[test]
    fn detects_final_message_created_after_claim_before_spawn() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let materialized = plan
            .materialize(&fixture.raw_path_plan(ProviderId::Trae))
            .unwrap();
        let final_message_path = materialized.final_message_path().to_owned();
        let claimed = materialized
            .bind(fixture.authority(ProviderId::Trae))
            .unwrap()
            .claim_launch()
            .unwrap();
        fs::write(final_message_path, b"raced final message").unwrap();

        assert_eq!(
            claimed.verify_before_spawn().unwrap_err().code(),
            "provider.output_exists"
        );
    }

    #[test]
    fn fixed_raw_members_are_descriptor_relative_and_revalidated() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let bound = fixture
            .bind(&plan, ProviderId::Trae)
            .claim_launch()
            .unwrap();
        let cases = [
            (ProviderRawMember::StdoutJsonl, b"stdout".as_slice()),
            (ProviderRawMember::StderrBin, b"stderr".as_slice()),
            (ProviderRawMember::RawArchive, b"archive".as_slice()),
            (
                ProviderRawMember::RawMembersManifest,
                b"manifest".as_slice(),
            ),
        ];

        for (member, bytes) in cases {
            let mut file = bound.create_raw_member(member).unwrap();
            file.write_all(bytes).unwrap();
            drop(file);
            let actual = bound
                .read_raw_member_bounded(member, bytes.len())
                .unwrap()
                .unwrap();
            assert_eq!(actual, bytes);
            assert_eq!(
                bound.create_raw_member(member).unwrap_err().code(),
                "provider.output_exists"
            );
        }
    }

    #[test]
    fn seal_rejects_byte_distinct_noncanonical_archive() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let claimed = fixture
            .bind(&plan, ProviderId::Trae)
            .claim_launch()
            .unwrap();
        let stdout = b"stdout\n";
        let stderr = b"stderr\n";
        for (member, bytes) in [
            (ProviderRawMember::StdoutJsonl, stdout.as_slice()),
            (ProviderRawMember::StderrBin, stderr.as_slice()),
        ] {
            let mut file = claimed.create_raw_member(member).unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        let mut archive = canonical_raw_archive(stdout, stderr, None).unwrap();
        archive.push(0);
        let mut archive_file = claimed
            .create_raw_member(ProviderRawMember::RawArchive)
            .unwrap();
        archive_file.write_all(&archive).unwrap();
        archive_file.sync_all().unwrap();
        let manifest = canonical_raw_members_manifest(stdout, stderr, None);
        let mut manifest_file = claimed
            .create_raw_member(ProviderRawMember::RawMembersManifest)
            .unwrap();
        manifest_file.write_all(&manifest).unwrap();
        manifest_file.sync_all().unwrap();
        claimed.sync_raw_directory().unwrap();

        assert_eq!(
            claimed.seal(true).unwrap_err().code(),
            "provider.raw_evidence"
        );
    }

    #[test]
    fn seal_rejects_member_replacement_during_final_revalidation() {
        let fixture = InvocationFixture::new();
        let plan = fixture.plan(ProviderId::Trae, &ProviderRunOptions::default());
        let claimed = fixture
            .bind(&plan, ProviderId::Trae)
            .claim_launch()
            .unwrap();
        let stdout = b"stdout\n";
        let stderr = b"stderr\n";
        let archive = canonical_raw_archive(stdout, stderr, None).unwrap();
        let manifest = canonical_raw_members_manifest(stdout, stderr, None);
        for (member, bytes) in [
            (ProviderRawMember::StdoutJsonl, stdout.as_slice()),
            (ProviderRawMember::StderrBin, stderr.as_slice()),
            (ProviderRawMember::RawArchive, archive.as_slice()),
            (ProviderRawMember::RawMembersManifest, manifest.as_slice()),
        ] {
            let mut file = claimed.create_raw_member(member).unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        claimed.sync_raw_directory().unwrap();
        let raw = fixture.raw_path(ProviderId::Trae);
        let stdout_path = raw.join(ProviderRawMember::StdoutJsonl.name());
        let displaced = fixture.root.join("displaced-stdout");

        let error = claimed
            .seal_with_revalidation_hook(true, || {
                fs::rename(&stdout_path, &displaced).unwrap();
                fs::write(&stdout_path, b"xxxxxx\n").unwrap();
                fs::set_permissions(&stdout_path, fs::Permissions::from_mode(0o600)).unwrap();
            })
            .unwrap_err();

        assert_eq!(error.code(), "provider.raw_evidence");
    }

    #[test]
    fn archive_limit_covers_every_accepted_member_at_its_maximum() {
        let archive_limit = std::hint::black_box(MAX_RAW_ARCHIVE_BYTES);
        let accepted_member_bytes =
            std::hint::black_box(MAX_STDOUT_BYTES + MAX_STDERR_BYTES + MAX_FINAL_MESSAGE_BYTES);

        assert_eq!(
            MAX_RAW_TAR_BYTES,
            tar_member_bound(MAX_STDOUT_BYTES)
                + tar_member_bound(MAX_STDERR_BYTES)
                + tar_member_bound(MAX_FINAL_MESSAGE_BYTES)
                + TAR_END_BYTES
        );
        assert!(archive_limit > accepted_member_bytes);
        assert_eq!(
            MAX_RAW_ARCHIVE_BYTES,
            MAX_RAW_TAR_BYTES
                + div_ceil(MAX_RAW_TAR_BYTES, DEFLATE_STORED_BLOCK_BYTES)
                    * DEFLATE_STORED_BLOCK_OVERHEAD
                + GZIP_WRAPPER_BYTES
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn preserves_non_utf8_absolute_repository_directory_without_lossy_conversion() {
        let fixture = InvocationFixture::new();
        let repository = fixture
            .root
            .join(OsString::from_vec(b"non-utf8-repository-\x80".to_vec()));
        let native_path = std::ffi::CString::new(repository.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkdir(native_path.as_ptr(), 0o700) }, 0);
        let plan = build_invocation(
            &fixture.snapshot(ProviderId::Trae),
            &repository,
            &context_bundle(),
            TASK,
            &ProviderRunOptions::default(),
        )
        .unwrap();

        assert_eq!(plan.working_directory(), repository);
        assert_eq!(
            literal_argument(plan.argument_plan(), 2).as_bytes(),
            repository.as_os_str().as_bytes()
        );
    }

    #[test]
    fn rejects_nul_in_os_paths() {
        let fixture = InvocationFixture::new();
        let nul = PathBuf::from(OsString::from_vec(b"/tmp/nul\0path".to_vec()));
        assert_eq!(
            build_invocation(
                &fixture.snapshot(ProviderId::Trae),
                &nul,
                &context_bundle(),
                TASK,
                &ProviderRunOptions::default(),
            )
            .unwrap_err()
            .code(),
            "provider.invocation_path"
        );

        let raw = fixture.ensure_raw(ProviderId::Trae);
        assert_eq!(
            EpisodeProviderRawPath::new(ProviderId::Trae, nul.clone(), nul)
                .unwrap_err()
                .code(),
            "provider.invocation_path"
        );
        drop(raw);
    }

    fn assert_no_forbidden_arguments(arguments: &[OsString]) {
        for forbidden in forbidden_arguments() {
            assert!(
                !arguments
                    .iter()
                    .any(|argument| argument == OsStr::new(forbidden)),
                "forbidden provider flag present: {forbidden}"
            );
        }
    }

    fn assert_no_forbidden_argument_plan(arguments: &[ProviderArgumentPlan]) {
        for forbidden in forbidden_arguments() {
            assert!(
                !arguments.iter().any(|argument| {
                    matches!(
                        argument,
                        ProviderArgumentPlan::Literal(value)
                            if value == OsStr::new(forbidden)
                    )
                }),
                "forbidden provider flag present: {forbidden}"
            );
        }
    }

    fn forbidden_arguments() -> [&'static str; 6] {
        [
            "--ephemeral",
            "--ignore-user-config",
            "--ignore-rules",
            "--dangerously-bypass-approvals-and-sandbox",
            "--dangerously-bypass-hook-trust",
            "--search",
        ]
    }

    #[cfg(target_os = "linux")]
    fn literal_argument(arguments: &[ProviderArgumentPlan], index: usize) -> &OsStr {
        match &arguments[index] {
            ProviderArgumentPlan::Literal(value) => value,
            ProviderArgumentPlan::OutputMember(_) => panic!("expected literal argument"),
        }
    }

    fn decode_prompt(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut remaining = bytes
            .strip_prefix(PROMPT_ENCODING_MAGIC)
            .expect("prompt magic");
        let mut fields = Vec::new();
        while !remaining.is_empty() {
            let header_end = remaining
                .iter()
                .position(|byte| *byte == b'\n')
                .expect("field header");
            let header = std::str::from_utf8(&remaining[..header_end]).unwrap();
            let (name, length) = header.rsplit_once(' ').expect("field length");
            let length = length.parse::<usize>().unwrap();
            remaining = &remaining[header_end + 1..];
            assert!(remaining.len() > length);
            let value = remaining[..length].to_vec();
            assert_eq!(remaining[length], b'\n');
            remaining = &remaining[length + 1..];
            fields.push((name.to_owned(), value));
        }
        fields
    }

    fn field<'a>(fields: &'a [(String, Vec<u8>)], name: &str) -> &'a [u8] {
        fields
            .iter()
            .find_map(|(field, value)| (field == name).then_some(value.as_slice()))
            .unwrap()
    }

    fn decode_command(bytes: &[u8], magic: &[u8]) -> (Vec<u8>, Vec<(u8, Vec<u8>)>) {
        let mut offset = magic.len();
        assert_eq!(&bytes[..offset], magic);
        let (executable_tag, executable, next) = decode_command_field(bytes, offset);
        assert_eq!(executable_tag, b'e');
        offset = next;
        let count = read_u64(bytes, &mut offset) as usize;
        let mut arguments = Vec::new();
        for _ in 0..count {
            let (tag, value, next) = decode_command_field(bytes, offset);
            arguments.push((tag, value));
            offset = next;
        }
        assert_eq!(offset, bytes.len());
        (executable, arguments)
    }

    fn decode_command_field(bytes: &[u8], mut offset: usize) -> (u8, Vec<u8>, usize) {
        let tag = bytes[offset];
        offset += 1;
        let length = read_u64(bytes, &mut offset) as usize;
        let value = bytes[offset..offset + length].to_vec();
        (tag, value, offset + length)
    }

    fn read_u64(bytes: &[u8], offset: &mut usize) -> u64 {
        let value = u64::from_be_bytes(bytes[*offset..*offset + 8].try_into().unwrap());
        *offset += 8;
        value
    }

    fn context_bundle() -> ContextBundle {
        context_with_markdown(RENDERED_CONTEXT)
    }

    fn context_with_markdown(markdown: &str) -> ContextBundle {
        ContextBundle {
            schema_version: CONTEXT_BUNDLE_SCHEMA.to_owned(),
            release_id: RELEASE_ID.to_owned(),
            workflow: WorkflowId::GeneralCoding,
            repository_invariants: Vec::new(),
            workflow_steps: Vec::new(),
            relevant_patterns: Vec::new(),
            anti_patterns: Vec::new(),
            verification_expectations: Vec::new(),
            selected_item_ids: Vec::new(),
            rejected_items: Vec::new(),
            routing_trace: RouteDecision {
                selected: WorkflowId::GeneralCoding,
                explicit: true,
                considered: Vec::new(),
            },
            estimated_tokens: 4,
            rendered_context_sha256: sha256_hex(markdown.as_bytes()),
            rendered_markdown: markdown.to_owned(),
        }
    }

    fn reseal(snapshot: &mut ProviderCapabilitySnapshot) {
        snapshot.capability_sha256 = sha256_prefixed(&snapshot.canonical_bytes());
    }

    fn seal_raw_capabilities(capabilities: &mut ProviderCapabilities) {
        let payload = LegacyCapabilityDigest {
            schema_version: &capabilities.schema_version,
            provider: capabilities.provider,
            executable: provider_executable_name(capabilities.provider)
                .to_str()
                .unwrap(),
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
        capabilities.capability_sha256 = sha256_prefixed(&serde_json::to_vec(&payload).unwrap());
    }

    fn assert_snapshot_error(capabilities: &ProviderCapabilities, code: &str) {
        let identity = executable_identity(&capabilities.executable).unwrap();
        assert_eq!(
            ProviderCapabilitySnapshot::from_probed(capabilities, identity)
                .unwrap_err()
                .code(),
            code
        );
    }

    fn authority(
        provider: ProviderId,
        relative_path: PathBuf,
        path: PathBuf,
        descriptor_path: &Path,
    ) -> EpisodeRawDirectoryAuthority {
        let directory: OwnedFd = File::open(descriptor_path).unwrap().into();
        let raw_path = EpisodeProviderRawPath::new(provider, relative_path, path).unwrap();
        EpisodeRawDirectoryAuthority::new(raw_path, directory).unwrap()
    }

    fn write_executable(path: &Path) {
        fs::write(path, b"#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    struct InvocationFixture {
        _temporary: TempDir,
        root: PathBuf,
        repository: PathBuf,
        repository_snapshot: crate::context_control::RepositorySnapshot,
        trae_executable: PathBuf,
        codex_executable: PathBuf,
    }

    impl InvocationFixture {
        fn new() -> Self {
            let temporary = TempDir::new().unwrap();
            let root = fs::canonicalize(temporary.path()).unwrap();
            let repository = root.join("repository");
            fs::create_dir(&repository).unwrap();
            git(&repository, &["init", "-q"]);
            git(
                &repository,
                &["config", "user.email", "harp@example.invalid"],
            );
            git(&repository, &["config", "user.name", "Harp Test"]);
            fs::write(repository.join("tracked.txt"), b"tracked\n").unwrap();
            git(&repository, &["add", "tracked.txt"]);
            git(&repository, &["commit", "-qm", "initial"]);
            let repository_snapshot =
                crate::context_control::RepositorySnapshot::capture(&repository).unwrap();
            let trae_executable = root.join("traecli");
            let codex_executable = root.join("codex");
            write_executable(&trae_executable);
            write_executable(&codex_executable);
            Self {
                _temporary: temporary,
                root,
                repository,
                repository_snapshot,
                trae_executable,
                codex_executable,
            }
        }

        fn executable(&self, provider: ProviderId) -> &Path {
            match provider {
                ProviderId::Trae => &self.trae_executable,
                ProviderId::Codex => &self.codex_executable,
            }
        }

        fn raw_capabilities(&self, provider: ProviderId) -> ProviderCapabilities {
            let mut capabilities = ProviderCapabilities {
                schema_version: PROVIDER_CAPABILITIES_SCHEMA.to_owned(),
                provider,
                executable: self.root.join(provider_executable_name(provider)),
                version: "1.2.3-rc.1+build.5".to_owned(),
                exec_json: true,
                output_last_message: true,
                working_directory: true,
                model: true,
                profile: true,
                sandbox: true,
                approval_config: true,
                resume_json: true,
                app_server_schema: true,
                capability_sha256: String::new(),
            };
            seal_raw_capabilities(&mut capabilities);
            capabilities
        }

        fn snapshot(&self, provider: ProviderId) -> ProviderCapabilitySnapshot {
            let capabilities = self.raw_capabilities(provider);
            let identity = executable_identity(&capabilities.executable).unwrap();
            ProviderCapabilitySnapshot::from_probed(&capabilities, identity).unwrap()
        }

        fn plan(&self, provider: ProviderId, options: &ProviderRunOptions) -> ProviderInvocation {
            build_invocation(
                &self.snapshot(provider),
                &self.repository,
                &context_bundle(),
                TASK,
                options,
            )
            .unwrap()
        }

        fn raw_relative(&self, provider: ProviderId) -> PathBuf {
            PathBuf::from("repositories")
                .join(&self.repository_snapshot.repository_id)
                .join("episodes")
                .join(EPISODE_ID)
                .join("raw")
                .join(provider.to_string())
        }

        fn raw_path(&self, provider: ProviderId) -> PathBuf {
            self.root.join(self.raw_relative(provider))
        }

        fn ensure_raw(&self, provider: ProviderId) -> PathBuf {
            let path = self.raw_path(provider);
            fs::create_dir_all(&path).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            path
        }

        fn authority(&self, provider: ProviderId) -> EpisodeRawDirectoryAuthority {
            let path = self.ensure_raw(provider);
            authority(provider, self.raw_relative(provider), path.clone(), &path)
        }

        fn bind(
            &self,
            plan: &ProviderInvocation,
            provider: ProviderId,
        ) -> super::BoundProviderInvocation {
            plan.materialize(&self.raw_path_plan(provider))
                .unwrap()
                .bind(self.authority(provider))
                .unwrap()
        }

        fn raw_path_plan(&self, provider: ProviderId) -> EpisodeProviderRawPath {
            EpisodeProviderRawPath::new(
                provider,
                self.raw_relative(provider),
                self.raw_path(provider),
            )
            .unwrap()
        }

        fn final_message_path(&self, provider: ProviderId) -> PathBuf {
            self.raw_path(provider).join("final_message.bin")
        }
    }

    fn git(repository: &Path, arguments: &[&str]) {
        assert!(std::process::Command::new("git")
            .args(arguments)
            .current_dir(repository)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .status()
            .unwrap()
            .success());
    }
}
