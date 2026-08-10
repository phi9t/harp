use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Deserializer, Serialize};

use super::canonical::{json_bytes, sha256_hex};
use super::context::ContextBundle;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use super::provider::{
    BoundProviderInvocation, EpisodeProviderRawPath, EpisodeRawDirectoryAuthority,
    MaterializedProviderInvocation,
};
use super::state::{HeldPrivateDirectory, PrivateFileExpectation, ReplacePolicy, StateRoot};
use super::{
    ProviderId, RepositorySnapshot, WorkflowId, COMPLETION_SCHEMA, CONTEXT_BUNDLE_SCHEMA,
    EPISODE_SCHEMA, PREFLIGHT_FAILURE_SCHEMA, PROVIDER_INVOCATION_SCHEMA,
};
use crate::AppError;

const MANIFEST_FILE: &str = "manifest.json";
const CONTEXT_FILE: &str = "context.json";
const COMPLETION_FILE: &str = "completion.json";
const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
const MAX_CONTEXT_BYTES: usize = 16 * 1024 * 1024;
const MAX_COMPLETION_BYTES: usize = 1024 * 1024;
static ID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawEvidenceSummary {
    pub(crate) capture_complete: bool,
    pub(crate) stdout_sha256: String,
    pub(crate) stderr_sha256: String,
    pub(crate) final_message_sha256: Option<String>,
    pub(crate) raw_archive_sha256: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextManifestCompleteness {
    Partial,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationMode {
    NativeDefaults,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ProviderInvocationManifest {
    pub schema_version: String,
    pub provider: ProviderId,
    pub command_sha256: String,
    pub prompt_sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderInvocationManifestWire {
    schema_version: String,
    provider: ProviderId,
    command_sha256: String,
    prompt_sha256: String,
}

impl<'de> Deserialize<'de> for ProviderInvocationManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ProviderInvocationManifestWire::deserialize(deserializer)?;
        let invocation = Self {
            schema_version: wire.schema_version,
            provider: wire.provider,
            command_sha256: wire.command_sha256,
            prompt_sha256: wire.prompt_sha256,
        };
        invocation.validate().map_err(serde::de::Error::custom)?;
        Ok(invocation)
    }
}

impl ProviderInvocationManifest {
    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != PROVIDER_INVOCATION_SCHEMA {
            return Err(invalid_manifest("invalid provider invocation schema"));
        }
        for (field, digest) in [
            ("command_sha256", self.command_sha256.as_str()),
            ("prompt_sha256", self.prompt_sha256.as_str()),
        ] {
            require_sha256_id(digest, field)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct EpisodeManifest {
    pub schema_version: String,
    pub episode_id: String,
    pub repository: RepositorySnapshot,
    pub provider: ProviderId,
    pub provider_version: String,
    pub provider_capabilities_sha256: String,
    pub workflow: WorkflowId,
    pub release_id: String,
    pub context_bundle_sha256: String,
    pub context_manifest_completeness: ContextManifestCompleteness,
    pub configuration_mode: ConfigurationMode,
    pub task_sha256: String,
    pub policy_sha256: Option<String>,
    pub invocation: ProviderInvocationManifest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EpisodeManifestWire {
    schema_version: String,
    episode_id: String,
    repository: RepositorySnapshot,
    provider: ProviderId,
    provider_version: String,
    provider_capabilities_sha256: String,
    workflow: WorkflowId,
    release_id: String,
    context_bundle_sha256: String,
    context_manifest_completeness: ContextManifestCompleteness,
    configuration_mode: ConfigurationMode,
    task_sha256: String,
    policy_sha256: Option<String>,
    invocation: ProviderInvocationManifest,
}

impl<'de> Deserialize<'de> for EpisodeManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = EpisodeManifestWire::deserialize(deserializer)?;
        let manifest = Self {
            schema_version: wire.schema_version,
            episode_id: wire.episode_id,
            repository: wire.repository,
            provider: wire.provider,
            provider_version: wire.provider_version,
            provider_capabilities_sha256: wire.provider_capabilities_sha256,
            workflow: wire.workflow,
            release_id: wire.release_id,
            context_bundle_sha256: wire.context_bundle_sha256,
            context_manifest_completeness: wire.context_manifest_completeness,
            configuration_mode: wire.configuration_mode,
            task_sha256: wire.task_sha256,
            policy_sha256: wire.policy_sha256,
            invocation: wire.invocation,
        };
        manifest.validate().map_err(serde::de::Error::custom)?;
        Ok(manifest)
    }
}

impl EpisodeManifest {
    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != EPISODE_SCHEMA {
            return Err(invalid_manifest("invalid episode manifest schema"));
        }
        require_generated_id(&self.episode_id, "ep", "episode ID")?;
        validate_repository_snapshot(&self.repository)?;
        require_bounded_text(&self.provider_version, 256, "provider_version")?;
        require_sha256_id(
            &self.provider_capabilities_sha256,
            "provider_capabilities_sha256",
        )?;
        require_release_id(&self.release_id)?;
        require_sha256_id(&self.context_bundle_sha256, "context_bundle_sha256")?;
        require_sha256_id(&self.task_sha256, "task_sha256")?;
        if let Some(digest) = &self.policy_sha256 {
            require_sha256_id(digest, "policy_sha256")?;
        }
        self.invocation.validate()?;
        if self.invocation.provider != self.provider {
            return Err(invalid_manifest(
                "provider invocation does not match episode provider",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct CompletionReceipt {
    pub schema_version: String,
    pub episode_id: String,
    pub provider_exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
    pub capture_complete: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub final_message_sha256: Option<String>,
    pub raw_archive_sha256: String,
    pub post_repository: RepositorySnapshot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionReceiptWire {
    schema_version: String,
    episode_id: String,
    provider_exit_code: Option<i32>,
    terminated_by_signal: Option<i32>,
    capture_complete: bool,
    stdout_sha256: String,
    stderr_sha256: String,
    final_message_sha256: Option<String>,
    raw_archive_sha256: String,
    post_repository: RepositorySnapshot,
}

impl<'de> Deserialize<'de> for CompletionReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CompletionReceiptWire::deserialize(deserializer)?;
        let receipt = Self {
            schema_version: wire.schema_version,
            episode_id: wire.episode_id,
            provider_exit_code: wire.provider_exit_code,
            terminated_by_signal: wire.terminated_by_signal,
            capture_complete: wire.capture_complete,
            stdout_sha256: wire.stdout_sha256,
            stderr_sha256: wire.stderr_sha256,
            final_message_sha256: wire.final_message_sha256,
            raw_archive_sha256: wire.raw_archive_sha256,
            post_repository: wire.post_repository,
        };
        receipt.validate().map_err(serde::de::Error::custom)?;
        Ok(receipt)
    }
}

impl CompletionReceipt {
    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != COMPLETION_SCHEMA {
            return Err(invalid_completion("invalid completion receipt schema"));
        }
        require_generated_id(&self.episode_id, "ep", "episode ID")?;
        match (self.provider_exit_code, self.terminated_by_signal) {
            (Some(code), None) if (0..=255).contains(&code) => {}
            (None, Some(signal)) if (1..=127).contains(&signal) => {}
            _ => {
                return Err(invalid_completion(
                    "completion must contain exactly one valid exit code or signal",
                ));
            }
        }
        for (field, digest) in [
            ("stdout_sha256", Some(self.stdout_sha256.as_str())),
            ("stderr_sha256", Some(self.stderr_sha256.as_str())),
            ("final_message_sha256", self.final_message_sha256.as_deref()),
            ("raw_archive_sha256", Some(self.raw_archive_sha256.as_str())),
        ] {
            if let Some(digest) = digest {
                require_sha256_id(digest, field)?;
            }
        }
        validate_repository_snapshot(&self.post_repository)?;
        Ok(())
    }

    pub fn validate_for(&self, manifest: &EpisodeManifest) -> Result<(), AppError> {
        manifest.validate()?;
        self.validate()?;
        if self.episode_id != manifest.episode_id
            || self.post_repository.repository_id != manifest.repository.repository_id
        {
            return Err(AppError::invalid_input(
                "episode.mismatch",
                "completion receipt does not belong to the episode manifest",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct PreflightFailure {
    pub schema_version: String,
    pub preflight_id: String,
    pub repository_id: String,
    pub provider: ProviderId,
    pub error_code: String,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreflightFailureWire {
    schema_version: String,
    preflight_id: String,
    repository_id: String,
    provider: ProviderId,
    error_code: String,
    message: String,
}

impl<'de> Deserialize<'de> for PreflightFailure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PreflightFailureWire::deserialize(deserializer)?;
        let failure = Self {
            schema_version: wire.schema_version,
            preflight_id: wire.preflight_id,
            repository_id: wire.repository_id,
            provider: wire.provider,
            error_code: wire.error_code,
            message: wire.message,
        };
        failure.validate().map_err(serde::de::Error::custom)?;
        Ok(failure)
    }
}

impl PreflightFailure {
    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != PREFLIGHT_FAILURE_SCHEMA {
            return Err(invalid_preflight("invalid preflight failure schema"));
        }
        require_generated_id(&self.preflight_id, "pf", "preflight ID")?;
        require_sha256_id(&self.repository_id, "repository_id")?;
        if self.error_code.is_empty()
            || self.error_code.len() > 128
            || !self.error_code.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._".contains(&byte)
            })
        {
            return Err(invalid_preflight("invalid preflight error code"));
        }
        require_bounded_text(&self.message, 4096, "message")
            .map_err(|_| invalid_preflight("invalid preflight failure message"))
    }
}

#[derive(Debug)]
pub(crate) struct EpisodeWriter<'a> {
    state: &'a StateRoot,
    manifest: EpisodeManifest,
    manifest_bytes: Vec<u8>,
    context_bytes: Vec<u8>,
    relative_directory: PathBuf,
    raw_directory: HeldPrivateDirectory,
}

impl<'a> EpisodeWriter<'a> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub(crate) fn plan_provider_raw_path(
        state: &StateRoot,
        repository_id: &str,
        episode_id: &str,
        provider: ProviderId,
    ) -> Result<EpisodeProviderRawPath, AppError> {
        let relative_path = episode_directory(repository_id, episode_id)?
            .join("raw")
            .join(provider.to_string());
        let path = state.plan_private_path(&relative_path)?;
        EpisodeProviderRawPath::new(provider, relative_path, path)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub(crate) fn begin(
        state: &'a StateRoot,
        manifest: EpisodeManifest,
        context: &ContextBundle,
        materialized: &MaterializedProviderInvocation,
    ) -> Result<Self, AppError> {
        manifest.validate()?;
        validate_materialized_invocation(state, &manifest, materialized)?;
        let context_bytes = validate_context(&manifest, context)?;
        if context_bytes.len() > MAX_CONTEXT_BYTES {
            return Err(AppError::invalid_input(
                "episode.context_size",
                format!("canonical context bundle exceeds {MAX_CONTEXT_BYTES} bytes"),
            ));
        }
        let manifest_bytes = json_bytes(&manifest)?;
        let relative_directory =
            episode_directory(&manifest.repository.repository_id, &manifest.episode_id)?;
        let members = BTreeMap::from([
            (PathBuf::from(CONTEXT_FILE), context_bytes.clone()),
            (PathBuf::from(MANIFEST_FILE), manifest_bytes.clone()),
        ]);
        let raw_relative = Path::new("raw").join(manifest.provider.to_string());
        let directories = BTreeSet::from([raw_relative.clone()]);

        // Context and raw storage are staged first; the manifest becomes visible only with the
        // atomic episode-tree publication.
        let raw_directory = state.publish_private_tree_create_only_holding_directory(
            &relative_directory,
            &members,
            &directories,
            &raw_relative,
        )?;

        Ok(Self {
            state,
            manifest,
            manifest_bytes,
            context_bytes,
            relative_directory,
            raw_directory,
        })
    }

    pub(crate) fn relative_directory(&self) -> &Path {
        &self.relative_directory
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub(crate) fn provider_raw_path(&self) -> Result<EpisodeProviderRawPath, AppError> {
        let (raw_path, _) = self.provider_raw_parts()?;
        Ok(raw_path)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    pub(crate) fn raw_directory_authority(&self) -> Result<EpisodeRawDirectoryAuthority, AppError> {
        let (raw_path, directory) = self.provider_raw_parts()?;
        EpisodeRawDirectoryAuthority::new(raw_path, directory)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn provider_raw_parts(
        &self,
    ) -> Result<(EpisodeProviderRawPath, std::os::fd::OwnedFd), AppError> {
        let expected_relative = episode_directory(
            &self.manifest.repository.repository_id,
            &self.manifest.episode_id,
        )?
        .join("raw")
        .join(self.manifest.provider.to_string());
        let (held_relative, path, directory) = self.raw_directory.duplicate_parts()?;
        if held_relative != expected_relative {
            return Err(AppError::invalid_input(
                "episode.mismatch",
                "held provider raw directory does not match the episode manifest",
            ));
        }
        let raw_path =
            EpisodeProviderRawPath::new(self.manifest.provider, expected_relative, path)?;
        Ok((raw_path, directory))
    }

    pub(crate) fn complete(
        &self,
        completion: &CompletionReceipt,
        invocation: &BoundProviderInvocation,
    ) -> Result<(), AppError> {
        self.complete_transaction(
            completion,
            invocation,
            |relative, bytes, expectations, verify| {
                self.state.write_private_create_only_after_validating_with(
                    relative,
                    bytes,
                    expectations,
                    verify,
                )
            },
        )
    }

    #[cfg(test)]
    fn complete_with_revalidation_hook<F>(
        &self,
        completion: &CompletionReceipt,
        invocation: &BoundProviderInvocation,
        revalidation_hook: F,
    ) -> Result<(), AppError>
    where
        F: FnOnce(),
    {
        self.complete_transaction(
            completion,
            invocation,
            |relative, bytes, expectations, verify| {
                self.state.write_private_create_only_after_validating_with(
                    relative,
                    bytes,
                    expectations,
                    || {
                        verify()?;
                        revalidation_hook();
                        Ok(())
                    },
                )
            },
        )
    }

    fn complete_transaction<T>(
        &self,
        completion: &CompletionReceipt,
        invocation: &BoundProviderInvocation,
        transaction: T,
    ) -> Result<(), AppError>
    where
        T: for<'b> FnOnce(
            &Path,
            &[u8],
            &[PrivateFileExpectation<'_>],
            Box<dyn FnOnce() -> Result<(), AppError> + 'b>,
        ) -> Result<(), AppError>,
    {
        completion.validate_for(&self.manifest)?;
        validate_bound_invocation(self.state, &self.manifest, invocation)?;
        let manifest_relative = self.relative_directory.join(MANIFEST_FILE);
        let context_relative = self.relative_directory.join(CONTEXT_FILE);
        let completion_relative = self.relative_directory.join(COMPLETION_FILE);
        let completion_bytes = json_bytes(completion)?;
        let expectations = [
            PrivateFileExpectation {
                relative: &manifest_relative,
                expected_bytes: &self.manifest_bytes,
                max_bytes: MAX_MANIFEST_BYTES,
            },
            PrivateFileExpectation {
                relative: &context_relative,
                expected_bytes: &self.context_bytes,
                max_bytes: MAX_CONTEXT_BYTES,
            },
        ];
        let expected = completion.clone();
        let verify = Box::new(move || {
            let evidence = invocation.verify_raw_evidence(expected.capture_complete)?;
            validate_completion_evidence(&expected, &evidence)
        });
        let result = transaction(
            &completion_relative,
            &completion_bytes,
            &expectations,
            verify,
        );
        match result {
            Err(error) if error.code() == "state.validation" => Err(AppError::invalid_input(
                "episode.mismatch",
                "episode manifest or context changed after publication",
            )),
            result => result,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpisodeInspection {
    manifest: EpisodeManifest,
    context_bytes: Vec<u8>,
    completion: Option<CompletionReceipt>,
}

impl EpisodeInspection {
    pub fn manifest(&self) -> &EpisodeManifest {
        &self.manifest
    }

    pub fn context_bytes(&self) -> &[u8] {
        &self.context_bytes
    }

    pub fn completion(&self) -> Option<&CompletionReceipt> {
        self.completion.as_ref()
    }
}

pub(crate) fn open_episode<'a>(
    state: &'a StateRoot,
    repository_id: &str,
    episode_id: &str,
) -> Result<EpisodeWriter<'a>, AppError> {
    let inspection = inspect_episode(state, repository_id, episode_id)?;
    let relative_directory = episode_directory(repository_id, episode_id)?;
    let manifest_bytes = json_bytes(inspection.manifest())?;
    let context_bytes = inspection.context_bytes.clone();
    let raw_directory = state.hold_private_directory(
        &relative_directory
            .join("raw")
            .join(inspection.manifest.provider.to_string()),
    )?;
    Ok(EpisodeWriter {
        state,
        manifest: inspection.manifest,
        manifest_bytes,
        context_bytes,
        relative_directory,
        raw_directory,
    })
}

pub fn inspect_episode(
    state: &StateRoot,
    repository_id: &str,
    episode_id: &str,
) -> Result<EpisodeInspection, AppError> {
    require_sha256_id(repository_id, "repository_id")?;
    require_generated_id(episode_id, "ep", "episode ID")?;
    let relative_directory = episode_directory(repository_id, episode_id)?;
    let manifest_bytes = state
        .read_private_file_bounded(&relative_directory.join(MANIFEST_FILE), MAX_MANIFEST_BYTES)?;
    let manifest: EpisodeManifest = parse_canonical_json(&manifest_bytes, "episode manifest")?;
    if manifest.repository.repository_id != repository_id || manifest.episode_id != episode_id {
        return Err(AppError::invalid_input(
            "episode.mismatch",
            "stored episode manifest does not match its repository namespace",
        ));
    }

    let context_bytes = state
        .read_private_file_bounded(&relative_directory.join(CONTEXT_FILE), MAX_CONTEXT_BYTES)?;
    require_matching_digest(
        &manifest.context_bundle_sha256,
        &context_bytes,
        "stored context bundle",
    )?;

    let entries = state.list_private_directory(&relative_directory)?;
    let completion = if entries.iter().any(|entry| entry == COMPLETION_FILE) {
        let completion_bytes = state.read_private_file_bounded(
            &relative_directory.join(COMPLETION_FILE),
            MAX_COMPLETION_BYTES,
        )?;
        let completion: CompletionReceipt =
            parse_canonical_json(&completion_bytes, "completion receipt")?;
        completion.validate_for(&manifest)?;
        Some(completion)
    } else {
        None
    };

    Ok(EpisodeInspection {
        manifest,
        context_bytes,
        completion,
    })
}

pub fn publish_preflight_failure(
    state: &StateRoot,
    failure: &PreflightFailure,
) -> Result<(), AppError> {
    failure.validate()?;
    let relative = Path::new("repositories")
        .join(&failure.repository_id)
        .join("preflight_failures")
        .join(format!("{}.json", failure.preflight_id));
    state.write_private_atomic(&relative, &json_bytes(failure)?, ReplacePolicy::CreateOnly)
}

pub fn generate_episode_id() -> Result<String, AppError> {
    generate_id("ep")
}

pub fn generate_preflight_id() -> Result<String, AppError> {
    generate_id("pf")
}

fn generate_id(prefix: &str) -> Result<String, AppError> {
    let nanoseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AppError::external("episode.clock", "system clock is before Unix epoch"))?
        .as_nanos()
        .try_into()
        .map_err(|_| AppError::external("episode.clock", "system time exceeds episode ID range"))?;
    let counter = ID_COUNTER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |counter| {
            counter.checked_add(1)
        })
        .map_err(|_| AppError::external("episode.id_exhausted", "episode ID counter exhausted"))?;
    Ok(generated_id_from_parts(
        prefix,
        nanoseconds,
        std::process::id(),
        counter,
    ))
}

#[cfg(test)]
fn episode_id_from_parts(nanoseconds: u64, process_id: u32, counter: u32) -> String {
    generated_id_from_parts("ep", nanoseconds, process_id, counter)
}

fn generated_id_from_parts(
    prefix: &str,
    nanoseconds: u64,
    process_id: u32,
    counter: u32,
) -> String {
    format!("{prefix}-{nanoseconds:016x}-{process_id:08x}-{counter:08x}")
}

fn episode_directory(repository_id: &str, episode_id: &str) -> Result<PathBuf, AppError> {
    require_sha256_id(repository_id, "repository_id")?;
    require_generated_id(episode_id, "ep", "episode ID")?;
    Ok(Path::new("repositories")
        .join(repository_id)
        .join("episodes")
        .join(episode_id))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_materialized_invocation(
    state: &StateRoot,
    manifest: &EpisodeManifest,
    materialized: &MaterializedProviderInvocation,
) -> Result<(), AppError> {
    let expected_path = EpisodeWriter::plan_provider_raw_path(
        state,
        &manifest.repository.repository_id,
        &manifest.episode_id,
        manifest.provider,
    )?;
    let matches = materialized.provider() == manifest.provider
        && materialized.provider_version() == manifest.provider_version
        && materialized.provider_capabilities_sha256() == manifest.provider_capabilities_sha256
        && materialized.command_sha256() == manifest.invocation.command_sha256
        && materialized.prompt_sha256() == manifest.invocation.prompt_sha256
        && materialized.repository() == &manifest.repository
        && materialized.raw_path() == &expected_path;
    if !matches {
        return Err(AppError::invalid_input(
            "episode.mismatch",
            "materialized provider invocation does not match the episode manifest",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_bound_invocation(
    state: &StateRoot,
    manifest: &EpisodeManifest,
    invocation: &BoundProviderInvocation,
) -> Result<(), AppError> {
    let expected_path = EpisodeWriter::plan_provider_raw_path(
        state,
        &manifest.repository.repository_id,
        &manifest.episode_id,
        manifest.provider,
    )?;
    if invocation.provider() != manifest.provider
        || invocation.provider_version() != manifest.provider_version
        || invocation.provider_capabilities_sha256() != manifest.provider_capabilities_sha256
        || invocation.command_sha256() != manifest.invocation.command_sha256
        || invocation.prompt_sha256() != manifest.invocation.prompt_sha256
        || invocation.repository() != &manifest.repository
        || invocation.raw_path() != &expected_path
    {
        return Err(AppError::invalid_input(
            "episode.mismatch",
            "bound provider invocation does not match the episode manifest",
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn validate_completion_evidence(
    completion: &CompletionReceipt,
    evidence: &RawEvidenceSummary,
) -> Result<(), AppError> {
    if completion.capture_complete != evidence.capture_complete
        || completion.stdout_sha256 != evidence.stdout_sha256
        || completion.stderr_sha256 != evidence.stderr_sha256
        || completion.final_message_sha256 != evidence.final_message_sha256
        || completion.raw_archive_sha256 != evidence.raw_archive_sha256
    {
        return Err(invalid_completion(
            "completion receipt does not match authenticated raw evidence",
        ));
    }
    Ok(())
}

fn validate_context(
    manifest: &EpisodeManifest,
    context: &ContextBundle,
) -> Result<Vec<u8>, AppError> {
    if context.schema_version != CONTEXT_BUNDLE_SCHEMA
        || context.release_id != manifest.release_id
        || context.workflow != manifest.workflow
        || !valid_lowercase_hex(&context.rendered_context_sha256, 64)
        || context.rendered_context_sha256 != sha256_hex(context.rendered_markdown.as_bytes())
    {
        return Err(AppError::invalid_input(
            "episode.mismatch",
            "context bundle does not match the episode manifest",
        ));
    }
    let bytes = context.canonical_bytes()?;
    require_matching_digest(&manifest.context_bundle_sha256, &bytes, "context bundle")?;
    Ok(bytes)
}

fn require_matching_digest(expected: &str, bytes: &[u8], label: &str) -> Result<(), AppError> {
    let actual = format!("sha256:{}", sha256_hex(bytes));
    if expected != actual {
        return Err(AppError::invalid_input(
            "episode.mismatch",
            format!("{label} digest does not match the episode manifest"),
        ));
    }
    Ok(())
}

fn parse_canonical_json<T>(bytes: &[u8], label: &str) -> Result<T, AppError>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let value = serde_json::from_slice(bytes).map_err(|error| {
        AppError::invalid_input("episode.serialization", format!("invalid {label}: {error}"))
    })?;
    let canonical = json_bytes(&value)?;
    if canonical != bytes {
        return Err(AppError::invalid_input(
            "episode.noncanonical",
            format!("{label} is not canonical JSON"),
        ));
    }
    Ok(value)
}

fn validate_repository_snapshot(snapshot: &RepositorySnapshot) -> Result<(), AppError> {
    let bytes = json_bytes(snapshot)?;
    serde_json::from_slice::<RepositorySnapshot>(&bytes).map_err(|error| {
        AppError::invalid_input(
            "episode.repository",
            format!("invalid repository snapshot: {error}"),
        )
    })?;
    Ok(())
}

fn require_generated_id(value: &str, prefix: &str, label: &str) -> Result<(), AppError> {
    let mut parts = value.split('-');
    let valid = parts.next() == Some(prefix)
        && parts
            .next()
            .is_some_and(|part| valid_lowercase_hex(part, 16))
        && parts
            .next()
            .is_some_and(|part| valid_lowercase_hex(part, 8))
        && parts
            .next()
            .is_some_and(|part| valid_lowercase_hex(part, 8))
        && parts.next().is_none();
    if !valid {
        return Err(AppError::invalid_input(
            "episode.id",
            format!("{label} has an invalid wire form"),
        ));
    }
    Ok(())
}

fn require_sha256_id(value: &str, field: &str) -> Result<(), AppError> {
    if !value
        .strip_prefix("sha256:")
        .is_some_and(|digest| valid_lowercase_hex(digest, 64))
    {
        return Err(AppError::invalid_input(
            "episode.digest",
            format!("{field} must be a lowercase SHA-256 ID"),
        ));
    }
    Ok(())
}

fn require_release_id(value: &str) -> Result<(), AppError> {
    if !value
        .strip_prefix("sha256-")
        .is_some_and(|digest| valid_lowercase_hex(digest, 64))
    {
        return Err(invalid_manifest(
            "release_id must be a lowercase content-addressed release ID",
        ));
    }
    Ok(())
}

fn valid_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_bounded_text(value: &str, max_bytes: usize, field: &str) -> Result<(), AppError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value
            .chars()
            .any(|character| character == '\0' || character.is_control())
    {
        return Err(AppError::invalid_input(
            "episode.text",
            format!("{field} must be nonempty bounded text without control characters"),
        ));
    }
    Ok(())
}

fn invalid_manifest(message: impl Into<String>) -> AppError {
    AppError::invalid_input("episode.manifest", message)
}

fn invalid_completion(message: impl Into<String>) -> AppError {
    AppError::invalid_input("episode.completion", message)
}

fn invalid_preflight(message: impl Into<String>) -> AppError {
    AppError::invalid_input("episode.preflight", message)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;
    use std::io::Write as _;
    use std::path::Path;
    use std::sync::{mpsc, Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    use serde_json::{json, Value};
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    use super::*;
    use crate::context_control::canonical::{json_bytes, sha256_hex};
    use crate::context_control::context::ContextBundle;
    use crate::context_control::provider::ProviderRawMember;
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    use crate::context_control::provider::{
        build_invocation, probe_snapshot, BoundProviderInvocation, MaterializedProviderInvocation,
        ProviderRunOptions,
    };
    use crate::context_control::routing::{RouteConsideration, RouteDecision};
    use crate::context_control::state::{with_lock_release_faults, StateRoot};
    use crate::context_control::{
        ProviderId, WorkflowId, COMPLETION_SCHEMA, CONTEXT_BUNDLE_SCHEMA, EPISODE_SCHEMA,
        PREFLIGHT_FAILURE_SCHEMA, PROVIDER_INVOCATION_SCHEMA, REPOSITORY_SNAPSHOT_SCHEMA,
    };

    #[test]
    fn injected_episode_identity_has_stable_lowercase_wire_form() {
        assert_eq!(
            episode_id_from_parts(0x0123_4567_89ab_cdef, 0x1234, 0x42),
            "ep-0123456789abcdef-00001234-00000042"
        );
    }

    #[test]
    fn manifest_enums_have_only_the_approved_wire_values() {
        assert_eq!(
            serde_json::to_value(ContextManifestCompleteness::Partial).unwrap(),
            json!("partial")
        );
        assert_eq!(
            serde_json::to_value(ConfigurationMode::NativeDefaults).unwrap(),
            json!("native-defaults")
        );
        assert!(serde_json::from_value::<ContextManifestCompleteness>(json!("complete")).is_err());
        assert!(serde_json::from_value::<ConfigurationMode>(json!("ignore-user-config")).is_err());
    }

    #[test]
    fn strict_contracts_reject_unknown_fields_schemas_ids_and_digests() {
        let manifest = manifest_value();
        assert!(serde_json::from_value::<EpisodeManifest>(manifest.clone()).is_ok());

        for (field, invalid) in [
            ("schema_version", json!("harp-episode/v2")),
            ("episode_id", json!("ep-not-valid")),
            ("provider_version", json!("")),
            ("provider_capabilities_sha256", json!("sha256:ABC")),
            ("release_id", json!("sha256:wrong-separator")),
            ("context_bundle_sha256", json!("sha256:short")),
            ("task_sha256", json!("sha256:short")),
            ("policy_sha256", json!("sha256:short")),
        ] {
            let mut value = manifest.clone();
            value[field] = invalid;
            assert!(
                serde_json::from_value::<EpisodeManifest>(value).is_err(),
                "invalid {field} must be rejected"
            );
        }

        let mut unknown = manifest;
        unknown["status"] = json!("running");
        assert!(serde_json::from_value::<EpisodeManifest>(unknown).is_err());

        let completion = completion_value();
        assert!(serde_json::from_value::<CompletionReceipt>(completion.clone()).is_ok());
        for field in [
            "stdout_sha256",
            "stderr_sha256",
            "final_message_sha256",
            "raw_archive_sha256",
        ] {
            let mut value = completion.clone();
            value[field] = json!("not-a-digest");
            assert!(
                serde_json::from_value::<CompletionReceipt>(value).is_err(),
                "invalid {field} must be rejected"
            );
        }

        let failure = preflight_value();
        assert!(serde_json::from_value::<PreflightFailure>(failure.clone()).is_ok());
        for (field, invalid) in [
            ("schema_version", json!("harp-preflight-failure/v2")),
            ("preflight_id", json!("pf-not-valid")),
            ("repository_id", json!("sha256:short")),
            ("error_code", json!("Provider Missing")),
            ("message", json!("")),
        ] {
            let mut value = failure.clone();
            value[field] = invalid;
            assert!(
                serde_json::from_value::<PreflightFailure>(value).is_err(),
                "invalid preflight {field} must be rejected"
            );
        }
        let mut unknown = failure;
        unknown["episode_id"] = json!("ep-0123456789abcdef-00001234-00000042");
        assert!(serde_json::from_value::<PreflightFailure>(unknown).is_err());
    }

    #[test]
    fn completion_rejects_manifest_mismatch_and_invalid_exit_signal_states() {
        let manifest: EpisodeManifest = serde_json::from_value(manifest_value()).unwrap();
        let valid: CompletionReceipt = serde_json::from_value(completion_value()).unwrap();
        valid.validate_for(&manifest).unwrap();

        let mut mismatched = valid.clone();
        mismatched.episode_id = "ep-0123456789abcdef-00001234-00000043".to_owned();
        assert_eq!(
            mismatched.validate_for(&manifest).unwrap_err().code(),
            "episode.mismatch"
        );

        for (exit, signal) in [(None, None), (Some(0), Some(15)), (Some(-1), None)] {
            let mut invalid = valid.clone();
            invalid.provider_exit_code = exit;
            invalid.terminated_by_signal = signal;
            assert_eq!(
                invalid.validate_for(&manifest).unwrap_err().code(),
                "episode.completion"
            );
        }
    }

    #[test]
    fn begin_publishes_canonical_context_raw_directory_and_manifest_before_launch() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (manifest, materialized) = prepared_episode(&fixture, &context);

        let episode =
            EpisodeWriter::begin(&fixture.state, manifest.clone(), &context, &materialized)
                .unwrap();
        let relative = episode.relative_directory();
        let manifest_relative = relative.join("manifest.json");
        let context_relative = relative.join("context.json");
        let raw_relative = relative.join("raw/trae");

        assert_eq!(
            fixture
                .state
                .read_private_file_bounded(&manifest_relative, 1024 * 1024)
                .unwrap(),
            json_bytes(&manifest).unwrap()
        );
        assert_eq!(
            fixture
                .state
                .read_private_file_bounded(&context_relative, 1024 * 1024)
                .unwrap(),
            context.canonical_bytes().unwrap()
        );
        assert!(fixture
            .state
            .list_private_directory(&raw_relative)
            .unwrap()
            .is_empty());

        #[cfg(unix)]
        {
            assert_eq!(mode(&fixture.root.join(relative)), 0o700);
            assert_eq!(mode(&fixture.root.join(raw_relative)), 0o700);
            assert_eq!(mode(&fixture.root.join(manifest_relative)), 0o600);
            assert_eq!(mode(&fixture.root.join(context_relative)), 0o600);
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    #[test]
    fn episode_writer_authority_binds_the_provider_plan_and_rejects_a_raw_swap() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (manifest, materialized) = prepared_episode(&fixture, &context);
        let episode =
            EpisodeWriter::begin(&fixture.state, manifest, &context, &materialized).unwrap();
        let plan = invocation_plan(&fixture, &context);

        let raw_path = episode.provider_raw_path().unwrap();
        assert_eq!(
            raw_path.relative_path(),
            episode
                .relative_directory()
                .join("raw")
                .join(ProviderId::Trae.to_string())
        );
        let materialized = plan.materialize(&raw_path).unwrap();
        assert_eq!(
            materialized.command_plan_sha256(),
            plan.command_plan_sha256()
        );
        assert_ne!(
            materialized.command_sha256(),
            materialized.command_plan_sha256()
        );
        let command_sha256 = materialized.command_sha256().to_owned();
        let bound = materialized
            .bind(episode.raw_directory_authority().unwrap())
            .unwrap();
        assert_eq!(bound.command_sha256(), command_sha256);
        assert_eq!(bound.command_plan_sha256(), plan.command_plan_sha256());
        assert_eq!(
            bound.final_message_path(),
            fixture
                .root
                .join(episode.relative_directory())
                .join("raw/trae/final_message.bin")
        );

        let materialized_before_swap = plan
            .materialize(&episode.provider_raw_path().unwrap())
            .unwrap();
        let authority_before_swap = episode.raw_directory_authority().unwrap();
        let raw = fixture
            .root
            .join(episode.relative_directory())
            .join("raw/trae");
        fs::rename(&raw, raw.with_file_name("trae-replaced")).unwrap();
        fs::create_dir(&raw).unwrap();
        fs::set_permissions(&raw, fs::Permissions::from_mode(0o700)).unwrap();

        assert_eq!(
            materialized_before_swap
                .bind(authority_before_swap)
                .unwrap_err()
                .code(),
            "provider.invocation_path"
        );
        assert_eq!(
            episode.raw_directory_authority().unwrap_err().code(),
            "state.conflict"
        );
    }

    #[test]
    fn completion_is_separate_create_only_and_manifest_is_immutable() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (_manifest, episode, bound, completion) = begin_bound_episode(&fixture, &context);
        let manifest_relative = episode.relative_directory().join("manifest.json");
        let manifest_before = fixture
            .state
            .read_private_file_bounded(&manifest_relative, 1024 * 1024)
            .unwrap();
        episode.complete(&completion, &bound).unwrap();

        let manifest_after = fixture
            .state
            .read_private_file_bounded(&manifest_relative, 1024 * 1024)
            .unwrap();
        assert_eq!(manifest_after, manifest_before);
        assert_eq!(
            fixture
                .state
                .read_private_file_bounded(
                    &episode.relative_directory().join("completion.json"),
                    1024 * 1024,
                )
                .unwrap(),
            json_bytes(&completion).unwrap()
        );
        assert_eq!(
            episode.complete(&completion, &bound).unwrap_err().code(),
            "state.exists"
        );
    }

    #[test]
    fn completion_commit_ignores_cleanup_errors_after_attempting_every_lock() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (_manifest, episode, bound, completion) = begin_bound_episode(&fixture, &context);

        let (result, attempted_releases): (Result<(), AppError>, usize) =
            with_lock_release_faults(|| episode.complete(&completion, &bound));

        result.unwrap();
        assert_eq!(attempted_releases, 4);
        assert_eq!(
            fixture
                .state
                .read_private_file_bounded(
                    &episode.relative_directory().join(COMPLETION_FILE),
                    MAX_COMPLETION_BYTES,
                )
                .unwrap(),
            json_bytes(&completion).unwrap()
        );
    }

    #[test]
    fn completion_revalidates_persisted_context_before_publication() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (_manifest, episode, bound, completion) = begin_bound_episode(&fixture, &context);
        let context_relative = episode.relative_directory().join(CONTEXT_FILE);
        let snapshot = fixture
            .state
            .snapshot_private_file(&context_relative)
            .unwrap();
        fixture
            .state
            .write_private_atomic(
                &context_relative,
                b"tampered context",
                ReplacePolicy::CompareAndReplace(snapshot),
            )
            .unwrap();

        assert_eq!(
            episode.complete(&completion, &bound).unwrap_err().code(),
            "episode.mismatch"
        );
        assert!(!fixture
            .root
            .join(episode.relative_directory())
            .join(COMPLETION_FILE)
            .exists());
    }

    #[test]
    fn completion_serializes_a_context_replacement_after_publication() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (_manifest, episode, bound, completion) = begin_bound_episode(&fixture, &context);
        let context_relative = episode.relative_directory().join(CONTEXT_FILE);
        let context_snapshot = fixture
            .state
            .snapshot_private_file(&context_relative)
            .unwrap();
        let root = fixture.root.clone();
        let raced_relative = context_relative.clone();
        let writer = RefCell::new(None);
        let (started_tx, started_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();

        episode
            .complete_with_revalidation_hook(&completion, &bound, || {
                writer.replace(Some(thread::spawn(move || {
                    let state = StateRoot::open_or_create(&root).unwrap();
                    started_tx.send(()).unwrap();
                    let result = state.write_private_atomic(
                        &raced_relative,
                        b"raced context",
                        ReplacePolicy::CompareAndReplace(context_snapshot),
                    );
                    finished_tx.send(()).unwrap();
                    result
                })));
                started_rx
                    .recv_timeout(Duration::from_secs(1))
                    .expect("context writer started");
                assert!(matches!(
                    finished_rx.recv_timeout(Duration::from_millis(100)),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ));
            })
            .unwrap();

        assert!(fixture
            .root
            .join(episode.relative_directory())
            .join(COMPLETION_FILE)
            .is_file());
        finished_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("context writer resumed");
        writer
            .borrow_mut()
            .take()
            .expect("context writer")
            .join()
            .expect("context writer thread")
            .expect("serialized context replacement");
    }

    #[test]
    fn incomplete_episode_remains_inspectable_without_completion() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (manifest, materialized) = prepared_episode(&fixture, &context);
        let episode_id = manifest.episode_id.clone();
        EpisodeWriter::begin(&fixture.state, manifest.clone(), &context, &materialized).unwrap();

        let inspected = inspect_episode(
            &fixture.state,
            &manifest.repository.repository_id,
            &episode_id,
        )
        .unwrap();
        assert_eq!(inspected.manifest(), &manifest);
        assert_eq!(
            inspected.context_bytes(),
            context.canonical_bytes().unwrap().as_slice()
        );
        assert_eq!(inspected.completion(), None);
    }

    #[test]
    fn concurrent_completion_publishers_preserve_one_receipt() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (manifest, materialized) = prepared_episode(&fixture, &context);
        let episode =
            EpisodeWriter::begin(&fixture.state, manifest.clone(), &context, &materialized)
                .unwrap();
        let competing_materialized = invocation_plan(&fixture, &context)
            .materialize(&episode.provider_raw_path().unwrap())
            .unwrap();
        let first_bound = materialized
            .bind(episode.raw_directory_authority().unwrap())
            .unwrap();
        let second_bound = competing_materialized
            .bind(episode.raw_directory_authority().unwrap())
            .unwrap();
        populate_raw_evidence(&first_bound, true);
        let evidence = first_bound.verify_raw_evidence(true).unwrap();
        let mut completion = completion_fixture(&manifest);
        completion.stdout_sha256 = evidence.stdout_sha256;
        completion.stderr_sha256 = evidence.stderr_sha256;
        completion.final_message_sha256 = evidence.final_message_sha256;
        completion.raw_archive_sha256 = evidence.raw_archive_sha256;
        let root = fixture.root.clone();
        let barrier = Arc::new(Barrier::new(3));

        let completions = [completion.clone(), {
            let mut other = completion;
            other.provider_exit_code = Some(7);
            other
        }];
        let results = completions
            .into_iter()
            .zip([first_bound, second_bound])
            .map(|(completion, bound)| {
                let root = root.clone();
                let barrier = Arc::clone(&barrier);
                let repository_id = manifest.repository.repository_id.clone();
                let episode_id = manifest.episode_id.clone();
                thread::spawn(move || {
                    let state = StateRoot::open_or_create(&root).unwrap();
                    let episode = open_episode(&state, &repository_id, &episode_id).unwrap();
                    barrier.wait();
                    episode.complete(&completion, &bound)
                })
            })
            .collect::<Vec<_>>();

        barrier.wait();
        let results = results
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter_map(|result| result.as_ref().err())
                .map(AppError::code)
                .collect::<Vec<_>>(),
            ["state.exists"]
        );

        let inspected = inspect_episode(
            &fixture.state,
            &manifest.repository.repository_id,
            &manifest.episode_id,
        )
        .unwrap();
        assert!(matches!(
            inspected
                .completion()
                .expect("one completion must be published")
                .provider_exit_code,
            Some(0 | 7)
        ));
    }

    #[test]
    fn mismatched_materialized_invocations_fail_before_state_mutation() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        for mutation in [
            "command",
            "prompt",
            "provider",
            "version",
            "capabilities",
            "repository",
        ] {
            let (mut manifest, materialized) = prepared_episode(&fixture, &context);
            match mutation {
                "command" => manifest.invocation.command_sha256 = digest("1"),
                "prompt" => manifest.invocation.prompt_sha256 = digest("2"),
                "provider" => {
                    manifest.provider = ProviderId::Codex;
                    manifest.invocation.provider = ProviderId::Codex;
                }
                "version" => manifest.provider_version = "9.9.9".to_owned(),
                "capabilities" => manifest.provider_capabilities_sha256 = digest("3"),
                "repository" => manifest.repository = repository_fixture("4"),
                _ => unreachable!(),
            }
            assert_eq!(
                EpisodeWriter::begin(&fixture.state, manifest, &context, &materialized)
                    .unwrap_err()
                    .code(),
                "episode.mismatch",
                "{mutation}"
            );
            assert!(!fixture.root.join("repositories").exists(), "{mutation}");
        }

        let (manifest, _) = prepared_episode(&fixture, &context);
        let other_episode = "ep-0123456789abcdef-00001234-00000043";
        let wrong_raw_path = EpisodeWriter::plan_provider_raw_path(
            &fixture.state,
            &manifest.repository.repository_id,
            other_episode,
            manifest.provider,
        )
        .unwrap();
        let wrong_materialized = invocation_plan(&fixture, &context)
            .materialize(&wrong_raw_path)
            .unwrap();
        assert_eq!(
            EpisodeWriter::begin(&fixture.state, manifest, &context, &wrong_materialized)
                .unwrap_err()
                .code(),
            "episode.mismatch"
        );
        assert!(!fixture.root.join("repositories").exists());
    }

    #[cfg(unix)]
    #[test]
    fn failed_atomic_begin_leaves_no_visible_manifest_and_can_be_retried() {
        let fixture = EpisodeFixture::new();
        fixture
            .state
            .publish_immutable_tree(
                Path::new("registry/bootstrap"),
                &std::collections::BTreeMap::from([(
                    std::path::PathBuf::from("ready"),
                    Vec::new(),
                )]),
            )
            .unwrap();
        let staging = fixture.root.join(".harp-internal/staging");
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o500)).unwrap();
        let context = context_fixture();
        let (manifest, materialized) = prepared_episode(&fixture, &context);
        let episode_path = fixture.root.join(
            episode_directory(&manifest.repository.repository_id, &manifest.episode_id).unwrap(),
        );

        assert_eq!(
            EpisodeWriter::begin(&fixture.state, manifest.clone(), &context, &materialized)
                .unwrap_err()
                .code(),
            "state.permissions"
        );
        assert!(!episode_path.join(MANIFEST_FILE).exists());
        assert!(!episode_path.exists());

        fs::set_permissions(&staging, fs::Permissions::from_mode(0o700)).unwrap();
        let (_, retry_materialized) = prepared_episode(&fixture, &context);
        EpisodeWriter::begin(&fixture.state, manifest, &context, &retry_materialized).unwrap();
        assert!(episode_path.join(MANIFEST_FILE).is_file());
    }

    #[test]
    fn oversized_canonical_context_is_rejected_before_publication() {
        let fixture = EpisodeFixture::new();
        let mut context = context_fixture();
        context.rendered_markdown = "x".repeat(MAX_CONTEXT_BYTES);
        context.rendered_context_sha256 = sha256_hex(context.rendered_markdown.as_bytes());
        let (manifest, materialized) = prepared_episode(&fixture, &context);

        assert_eq!(
            EpisodeWriter::begin(&fixture.state, manifest, &context, &materialized)
                .unwrap_err()
                .code(),
            "episode.context_size"
        );
        assert!(!fixture.root.join("repositories").exists());
    }

    #[cfg(unix)]
    #[test]
    fn publication_rejects_symlinked_episode_namespace_without_writing_outside() {
        use std::os::unix::fs::symlink;

        let fixture = EpisodeFixture::new();
        let outside = tempdir().unwrap();
        let context = context_fixture();
        let (manifest, materialized) = prepared_episode(&fixture, &context);
        symlink(outside.path(), fixture.root.join("repositories")).unwrap();

        assert_eq!(
            EpisodeWriter::begin(&fixture.state, manifest, &context, &materialized)
                .unwrap_err()
                .code(),
            "provider.invocation_path"
        );
        assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn completion_rejects_episode_directory_with_loosened_permissions() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (_manifest, episode, bound, completion) = begin_bound_episode(&fixture, &context);
        let episode_path = fixture.root.join(episode.relative_directory());
        fs::set_permissions(&episode_path, fs::Permissions::from_mode(0o770)).unwrap();

        assert_eq!(
            episode.complete(&completion, &bound).unwrap_err().code(),
            "state.permissions"
        );
        assert!(!episode_path.join(COMPLETION_FILE).exists());
    }

    #[test]
    fn begin_rejects_context_or_episode_identity_mismatch_before_publication() {
        let fixture = EpisodeFixture::new();
        let context = context_fixture();
        let (mut manifest, materialized) = prepared_episode(&fixture, &context);
        manifest.context_bundle_sha256 = digest("0");
        assert_eq!(
            EpisodeWriter::begin(&fixture.state, manifest, &context, &materialized)
                .unwrap_err()
                .code(),
            "episode.mismatch"
        );
        assert!(!fixture.root.join("repositories").exists());

        let (mut manifest, materialized) = prepared_episode(&fixture, &context);
        manifest.invocation.provider = ProviderId::Codex;
        assert_eq!(
            EpisodeWriter::begin(&fixture.state, manifest, &context, &materialized)
                .unwrap_err()
                .code(),
            "episode.manifest"
        );
        assert!(!fixture.root.join("repositories").exists());
    }

    #[test]
    fn preflight_failure_is_canonical_create_only_and_outside_episodes() {
        let fixture = EpisodeFixture::new();
        let failure: PreflightFailure = serde_json::from_value(preflight_value()).unwrap();

        publish_preflight_failure(&fixture.state, &failure).unwrap();

        let repository = Path::new("repositories").join(&failure.repository_id);
        let failures = repository.join("preflight_failures");
        assert_eq!(
            fixture.state.list_private_directory(&repository).unwrap(),
            ["preflight_failures"]
        );
        assert_eq!(
            fixture.state.list_private_directory(&failures).unwrap(),
            [format!("{}.json", failure.preflight_id)]
        );
        assert_eq!(
            fixture
                .state
                .read_private_file_bounded(
                    &failures.join(format!("{}.json", failure.preflight_id)),
                    1024 * 1024,
                )
                .unwrap(),
            json_bytes(&failure).unwrap()
        );
        assert_eq!(
            publish_preflight_failure(&fixture.state, &failure)
                .unwrap_err()
                .code(),
            "state.exists"
        );
        assert!(!fixture.root.join(repository).join("episodes").exists());
    }

    fn manifest_value() -> Value {
        json!({
            "schema_version": EPISODE_SCHEMA,
            "episode_id": "ep-0123456789abcdef-00001234-00000042",
            "repository": repository_value("a"),
            "provider": "trae",
            "provider_version": "0.200.19",
            "provider_capabilities_sha256": digest("b"),
            "workflow": "ci_repair",
            "release_id": release_id("c"),
            "context_bundle_sha256": digest("d"),
            "context_manifest_completeness": "partial",
            "configuration_mode": "native-defaults",
            "task_sha256": digest("e"),
            "policy_sha256": digest("f"),
            "invocation": {
                "schema_version": PROVIDER_INVOCATION_SCHEMA,
                "provider": "trae",
                "command_sha256": digest("1"),
                "prompt_sha256": digest("2")
            }
        })
    }

    fn completion_value() -> Value {
        json!({
            "schema_version": COMPLETION_SCHEMA,
            "episode_id": "ep-0123456789abcdef-00001234-00000042",
            "provider_exit_code": 0,
            "terminated_by_signal": null,
            "capture_complete": true,
            "stdout_sha256": digest("3"),
            "stderr_sha256": digest("4"),
            "final_message_sha256": digest("5"),
            "raw_archive_sha256": digest("6"),
            "post_repository": repository_value("a")
        })
    }

    fn preflight_value() -> Value {
        json!({
            "schema_version": PREFLIGHT_FAILURE_SCHEMA,
            "preflight_id": "pf-0123456789abcdef-00001234-00000042",
            "repository_id": digest("a"),
            "provider": "codex",
            "error_code": "provider.missing_capability",
            "message": "required provider capability is unavailable"
        })
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn write_probe_executable(path: &Path) {
        fs::write(
            path,
            "#!/bin/sh\ncase \"$*\" in\n\
             \"--version\") printf '%s\\n' 'traecli 1.2.3' ;;\n\
             \"exec --help\") printf '%s\\n' --json --output-last-message --cd --config ;;\n\
             *) exit 0 ;;\n\
             esac\n",
        )
        .unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn repository_value(digit: &str) -> Value {
        json!({
            "schema_version": REPOSITORY_SNAPSHOT_SCHEMA,
            "repository_id": digest(digit),
            "head": "8".repeat(40),
            "status_sha256": digest("9"),
            "tracked_changes_sha256": digest("a"),
            "tracked_count": 0,
            "untracked_count": 0,
            "dirty": false
        })
    }

    fn digest(digit: &str) -> String {
        format!("sha256:{}", digit.repeat(64))
    }

    fn release_id(digit: &str) -> String {
        format!("sha256-{}", digit.repeat(64))
    }

    fn context_fixture() -> ContextBundle {
        ContextBundle {
            schema_version: CONTEXT_BUNDLE_SCHEMA.to_owned(),
            release_id: release_id("c"),
            workflow: WorkflowId::CiRepair,
            repository_invariants: Vec::new(),
            workflow_steps: Vec::new(),
            relevant_patterns: Vec::new(),
            anti_patterns: Vec::new(),
            verification_expectations: Vec::new(),
            selected_item_ids: Vec::new(),
            rejected_items: Vec::new(),
            routing_trace: RouteDecision {
                selected: WorkflowId::CiRepair,
                explicit: true,
                considered: vec![RouteConsideration {
                    workflow: WorkflowId::CiRepair,
                    score: 0,
                    matched_rule_ids: Vec::new(),
                    rejection_reason: None,
                }],
            },
            estimated_tokens: 1,
            rendered_context_sha256: sha256_hex(b"context"),
            rendered_markdown: "context".to_owned(),
        }
    }

    fn manifest_fixture(context: &ContextBundle) -> EpisodeManifest {
        let mut value = manifest_value();
        value["context_bundle_sha256"] = json!(format!(
            "sha256:{}",
            sha256_hex(&context.canonical_bytes().unwrap())
        ));
        serde_json::from_value(value).unwrap()
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn prepared_episode(
        fixture: &EpisodeFixture,
        context: &ContextBundle,
    ) -> (EpisodeManifest, MaterializedProviderInvocation) {
        let plan = invocation_plan(fixture, context);
        let mut manifest = manifest_fixture(context);
        manifest.repository = plan.repository().clone();
        manifest.provider = plan.provider();
        manifest.provider_version = plan.provider_version().to_owned();
        manifest.provider_capabilities_sha256 = plan.provider_capabilities_sha256().to_owned();
        manifest.invocation.provider = plan.provider();
        manifest.invocation.prompt_sha256 = plan.prompt_sha256().to_owned();
        let raw_path = EpisodeWriter::plan_provider_raw_path(
            &fixture.state,
            &manifest.repository.repository_id,
            &manifest.episode_id,
            manifest.provider,
        )
        .unwrap();
        let materialized = plan.materialize(&raw_path).unwrap();
        manifest.invocation.command_sha256 = materialized.command_sha256().to_owned();
        (manifest, materialized)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn invocation_plan(
        fixture: &EpisodeFixture,
        context: &ContextBundle,
    ) -> crate::context_control::provider::ProviderInvocation {
        let executable = fixture.repository.join("traecli");
        if !executable.exists() {
            write_probe_executable(&executable);
        }
        let capabilities = probe_snapshot(ProviderId::Trae, &executable).unwrap();
        build_invocation(
            &capabilities,
            &fixture.repository,
            context,
            "complete the episode",
            &ProviderRunOptions::default(),
        )
        .unwrap()
    }

    fn repository_fixture(digit: &str) -> RepositorySnapshot {
        serde_json::from_value(repository_value(digit)).unwrap()
    }

    fn completion_fixture(manifest: &EpisodeManifest) -> CompletionReceipt {
        let mut value = completion_value();
        value["episode_id"] = json!(manifest.episode_id);
        value["post_repository"]["repository_id"] = json!(manifest.repository.repository_id);
        serde_json::from_value(value).unwrap()
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn begin_bound_episode<'a>(
        fixture: &'a EpisodeFixture,
        context: &ContextBundle,
    ) -> (
        EpisodeManifest,
        EpisodeWriter<'a>,
        BoundProviderInvocation,
        CompletionReceipt,
    ) {
        let (manifest, materialized) = prepared_episode(fixture, context);
        let episode =
            EpisodeWriter::begin(&fixture.state, manifest.clone(), context, &materialized).unwrap();
        let bound = materialized
            .bind(episode.raw_directory_authority().unwrap())
            .unwrap();
        populate_raw_evidence(&bound, true);
        let evidence = bound.verify_raw_evidence(true).unwrap();
        let mut completion = completion_fixture(&manifest);
        completion.stdout_sha256 = evidence.stdout_sha256;
        completion.stderr_sha256 = evidence.stderr_sha256;
        completion.final_message_sha256 = evidence.final_message_sha256;
        completion.raw_archive_sha256 = evidence.raw_archive_sha256;
        (manifest, episode, bound, completion)
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    fn populate_raw_evidence(bound: &BoundProviderInvocation, final_message: bool) {
        bound.claim_launch().unwrap();
        let members = [
            (ProviderRawMember::StdoutJsonl, b"stdout\n".as_slice()),
            (ProviderRawMember::StderrBin, b"stderr\n".as_slice()),
        ];
        for (member, bytes) in members {
            let mut file = bound.create_raw_member(member).unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        if final_message {
            let mut file = bound
                .create_raw_member(ProviderRawMember::FinalMessage)
                .unwrap();
            file.write_all(b"final response\n").unwrap();
            file.sync_all().unwrap();
        }

        let mut archive_bytes = Vec::new();
        {
            let encoder =
                flate2::write::GzEncoder::new(&mut archive_bytes, flate2::Compression::default());
            let mut archive = tar::Builder::new(encoder);
            for (name, bytes) in [
                ("final_message.bin", b"final response\n".as_slice()),
                ("stderr.bin", b"stderr\n".as_slice()),
                ("stdout.jsonl", b"stdout\n".as_slice()),
            ] {
                if name == "final_message.bin" && !final_message {
                    continue;
                }
                let mut header = tar::Header::new_gnu();
                header.set_size(bytes.len() as u64);
                header.set_mode(0o644);
                header.set_uid(0);
                header.set_gid(0);
                header.set_mtime(0);
                header.set_cksum();
                archive.append_data(&mut header, name, bytes).unwrap();
            }
            archive.into_inner().unwrap().finish().unwrap();
        }
        let mut archive = bound
            .create_raw_member(ProviderRawMember::RawArchive)
            .unwrap();
        archive.write_all(&archive_bytes).unwrap();
        archive.sync_all().unwrap();

        let mut rows = vec![
            ("stderr.bin", b"stderr\n".as_slice()),
            ("stdout.jsonl", b"stdout\n".as_slice()),
        ];
        if final_message {
            rows.push(("final_message.bin", b"final response\n".as_slice()));
        }
        rows.sort_by_key(|(name, _)| *name);
        let rows = rows
            .into_iter()
            .map(|(name, bytes)| raw_member_row(name, bytes))
            .collect::<String>();
        let mut manifest = bound
            .create_raw_member(ProviderRawMember::RawMembersManifest)
            .unwrap();
        manifest.write_all(rows.as_bytes()).unwrap();
        manifest.sync_all().unwrap();
        bound.sync_raw_directory().unwrap();
    }

    fn raw_member_row(name: &str, bytes: &[u8]) -> String {
        format!("sha256:{}\t{}\t{name}\n", sha256_hex(bytes), bytes.len())
    }

    struct EpisodeFixture {
        _temporary: tempfile::TempDir,
        root: std::path::PathBuf,
        repository: std::path::PathBuf,
        state: StateRoot,
    }

    impl EpisodeFixture {
        fn new() -> Self {
            let temporary = tempdir().unwrap();
            let root = fs::canonicalize(temporary.path()).unwrap().join("state");
            let repository = fs::canonicalize(temporary.path())
                .unwrap()
                .join("repository");
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
            let state = StateRoot::open_or_create(&root).unwrap();
            Self {
                _temporary: temporary,
                root,
                repository,
                state,
            }
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

    #[cfg(unix)]
    fn mode(path: &Path) -> u32 {
        fs::symlink_metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[allow(dead_code)]
    fn assert_public_contract_types(
        _manifest: EpisodeManifest,
        _completion: CompletionReceipt,
        _failure: PreflightFailure,
        _invocation: ProviderInvocationManifest,
        _provider: ProviderId,
        _workflow: WorkflowId,
    ) {
    }
}
