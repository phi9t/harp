use std::path::{Path, PathBuf};
#[cfg(test)]
use std::sync::Arc;

use harp_contracts::{ArtifactRef, AttemptId, RunId, TaskId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::platform::{Dir, EntryKind, SecureError, TempInitFault};
#[cfg(test)]
use crate::test_support::{FaultPoint, SubstitutionPurpose, TestHooks};
use crate::{ArtifactError, ArtifactResult};

pub(crate) const MAX_OBJECT_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_EVIDENCE_LINE_BYTES: usize = 1024 * 1024;
pub(crate) const CHECKPOINT_DIR: &str = "checkpoints";
pub(crate) const EVIDENCE_FILE: &str = "evidence.jsonl";
const MAX_ATTEMPT_SCAN_DEPTH: usize = 16;
const MAX_ATTEMPT_SCAN_ENTRIES: usize = 10_000;
pub const MAX_ATTEMPT_SCAN_BYTES: u64 = i64::MAX as u64;

#[derive(Clone, Copy)]
pub(crate) enum ImmutablePurpose {
    Object,
    Checkpoint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptKey {
    pub run_id: RunId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AttemptStorageUsage {
    pub checkpoint_bytes: u128,
    pub evidence_bytes: u128,
    pub notes_bytes: u128,
    pub result_scratch_bytes: u128,
    pub other_bytes: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageLimitObservation {
    Within {
        logical_bytes: u128,
        persisted_bytes: u64,
    },
    Exceeded {
        logical_bytes: u128,
        persisted_bytes: u64,
    },
}

impl AttemptStorageUsage {
    pub fn total_bytes(&self) -> ArtifactResult<u128> {
        [
            self.checkpoint_bytes,
            self.evidence_bytes,
            self.notes_bytes,
            self.result_scratch_bytes,
            self.other_bytes,
        ]
        .into_iter()
        .try_fold(0_u128, |total, bytes| {
            total.checked_add(bytes).ok_or(ArtifactError::SizeMismatch {
                context: "attempt storage accounting overflow".to_owned(),
                expected: u64::MAX,
                actual: u64::MAX,
            })
        })
    }

    pub fn limit_observation(&self, max_bytes: u64) -> StorageLimitObservation {
        let logical_bytes = self.total_bytes().unwrap_or(u128::MAX);
        let persisted_bytes = logical_bytes.min(u128::from(i64::MAX as u64)) as u64;
        if logical_bytes > u128::from(max_bytes) {
            StorageLimitObservation::Exceeded {
                logical_bytes,
                persisted_bytes,
            }
        } else {
            StorageLimitObservation::Within {
                logical_bytes,
                persisted_bytes,
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactStoreIdentity {
    pub canonical_root: PathBuf,
    pub device: u64,
    pub inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAttemptScratch {
    key: AttemptKey,
    store_identity: ArtifactStoreIdentity,
    canonical_path: PathBuf,
    device: u64,
    inode: u64,
}

#[derive(Debug)]
pub struct VerifiedRuntimePath {
    path: PathBuf,
}

impl VerifiedRuntimePath {
    pub fn as_path(&self) -> &Path {
        &self.path
    }
}

impl ResolvedAttemptScratch {
    pub fn key(&self) -> &AttemptKey {
        &self.key
    }

    pub fn store_identity(&self) -> &ArtifactStoreIdentity {
        &self.store_identity
    }

    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    pub const fn device(&self) -> u64 {
        self.device
    }

    pub const fn inode(&self) -> u64 {
        self.inode
    }

    pub fn runtime_authority(&self) -> ArtifactResult<harp_contracts::RuntimeWorkspaceAuthority> {
        let canonical_path = self
            .canonical_path
            .to_str()
            .ok_or_else(|| ArtifactError::Conflict {
                context: "runtime workspace authority".to_owned(),
                message: "canonical attempt path is not UTF-8".to_owned(),
                source: None,
            })?
            .to_owned();
        let canonical_root = self
            .store_identity
            .canonical_root
            .to_str()
            .ok_or_else(|| ArtifactError::Conflict {
                context: "runtime workspace authority".to_owned(),
                message: "canonical artifact root is not UTF-8".to_owned(),
                source: None,
            })?
            .to_owned();
        let key_bytes = serde_json::to_vec(&(
            self.key.run_id.to_string(),
            self.key.task_id.to_string(),
            self.key.attempt_id.to_string(),
        ))
        .map_err(|source| ArtifactError::Serialization {
            context: "runtime workspace attempt key".to_owned(),
            source,
        })?;
        Ok(harp_contracts::RuntimeWorkspaceAuthority {
            canonical_path,
            canonical_root,
            root_device: self.store_identity.device,
            root_inode: self.store_identity.inode,
            attempt_device: self.device,
            attempt_inode: self.inode,
            attempt_key_sha256: format!("{:x}", Sha256::digest(key_bytes)),
        })
    }
}

pub fn verify_runtime_workspace_authority(
    authority: &harp_contracts::RuntimeWorkspaceAuthority,
) -> ArtifactResult<()> {
    authority
        .validate()
        .map_err(|source| ArtifactError::InvalidWorkspaceAuthority { source })?;
    let root = Dir::open_root(Path::new(&authority.canonical_root)).map_err(|error| {
        ArtifactError::from_secure(
            "open runtime workspace authority root",
            &authority.canonical_root,
            error,
        )
    })?;
    let root_identity = root.identity().map_err(|error| {
        ArtifactError::from_secure(
            "inspect runtime workspace authority root",
            &authority.canonical_root,
            error,
        )
    })?;
    if root_identity.device() != authority.root_device
        || root_identity.inode() != authority.root_inode
    {
        return Err(ArtifactError::IdentityMismatch {
            context: "runtime workspace authority root".to_owned(),
            expected: format!("{}:{}", authority.root_device, authority.root_inode),
            actual: root_identity.to_string(),
        });
    }
    let relative = Path::new(&authority.canonical_path)
        .strip_prefix(&authority.canonical_root)
        .map_err(|_| ArtifactError::Conflict {
            context: authority.canonical_path.clone(),
            message: "runtime workspace is outside authoritative root".to_owned(),
            source: None,
        })?;
    let mut current = root;
    for component in relative.components() {
        let std::path::Component::Normal(component) = component else {
            return Err(ArtifactError::Conflict {
                context: authority.canonical_path.clone(),
                message: "runtime workspace path contains a non-normal component".to_owned(),
                source: None,
            });
        };
        let component = component.to_str().ok_or_else(|| ArtifactError::Conflict {
            context: authority.canonical_path.clone(),
            message: "runtime workspace component is not UTF-8".to_owned(),
            source: None,
        })?;
        current = current.open_dir(component).map_err(|error| {
            ArtifactError::from_secure("walk runtime workspace authority", component, error)
        })?;
    }
    let identity = current.identity().map_err(|error| {
        ArtifactError::from_secure(
            "inspect runtime workspace authority attempt",
            &authority.canonical_path,
            error,
        )
    })?;
    if identity.device() != authority.attempt_device || identity.inode() != authority.attempt_inode
    {
        return Err(ArtifactError::IdentityMismatch {
            context: "runtime workspace authority attempt".to_owned(),
            expected: format!("{}:{}", authority.attempt_device, authority.attempt_inode),
            actual: identity.to_string(),
        });
    }
    Ok(())
}

#[derive(Debug)]
pub struct ArtifactStore {
    pub(crate) root_path: PathBuf,
    pub(crate) root: Dir,
    #[cfg(test)]
    pub(crate) test_hooks: Arc<TestHooks>,
}

impl ArtifactStore {
    /// Opens an existing artifact-store root and pins all subsequent mutation
    /// to its directory descriptor.
    ///
    /// The caller must securely create the root directory before calling this
    /// method. Harp never creates a missing root through an untrusted parent.
    /// On Darwin, the store root must be on the same device as the system
    /// anonymous temporary-file filesystem so `fclonefileat` can publish
    /// immutable objects atomically.
    pub fn open(root: impl AsRef<Path>) -> ArtifactResult<Self> {
        #[cfg(test)]
        {
            Self::open_internal(root, Arc::new(TestHooks::default()))
        }
        #[cfg(not(test))]
        Self::open_internal(root)
    }

    pub fn identity(&self) -> ArtifactResult<ArtifactStoreIdentity> {
        let canonical_root =
            std::fs::canonicalize(&self.root_path).map_err(|source| ArtifactError::Io {
                operation: "canonicalize artifact root",
                context: self.root_path.display().to_string(),
                source,
            })?;
        let identity = self.root.identity().map_err(|error| {
            ArtifactError::from_secure(
                "inspect artifact root identity",
                self.root_path.display().to_string(),
                error,
            )
        })?;
        Ok(ArtifactStoreIdentity {
            canonical_root,
            device: identity.device(),
            inode: identity.inode(),
        })
    }

    fn open_internal(
        root: impl AsRef<Path>,
        #[cfg(test)] test_hooks: Arc<TestHooks>,
    ) -> ArtifactResult<Self> {
        let root_path = root.as_ref().to_path_buf();
        let descriptor =
            Dir::open_root(&root_path).map_err(|error| ArtifactError::InvalidRoot {
                path: root_path.clone(),
                context: "root must exist as a real directory and retain the opened identity"
                    .to_owned(),
                source: secure_source(error),
            })?;
        let anonymous_source =
            descriptor
                .create_temp("store-open-probe", None)
                .map_err(|error| {
                    ArtifactError::from_secure(
                        "create anonymous publication source probe",
                        root_path.display().to_string(),
                        error,
                    )
                })?;
        let root_device = descriptor.device_id().map_err(|error| {
            ArtifactError::from_secure(
                "inspect artifact store device",
                root_path.display().to_string(),
                error,
            )
        })?;
        let anonymous_device = {
            #[cfg(test)]
            {
                test_hooks
                    .anonymous_device_override()
                    .unwrap_or_else(|| anonymous_source.device_id())
            }
            #[cfg(not(test))]
            {
                anonymous_source.device_id()
            }
        };
        if root_device != anonymous_device {
            return Err(ArtifactError::Unsupported {
                context: "anonymous publication source and store root are on different filesystems; immutable atomic publication unsupported".to_owned(),
            });
        }
        drop(anonymous_source);
        Ok(Self {
            root_path,
            root: descriptor,
            #[cfg(test)]
            test_hooks,
        })
    }

    #[cfg(test)]
    pub(crate) fn open_with_test_hooks(
        root: impl AsRef<Path>,
        test_hooks: Arc<TestHooks>,
    ) -> ArtifactResult<Self> {
        Self::open_internal(root, test_hooks)
    }

    pub fn publish(&self, bytes: &[u8], media_type: &str) -> ArtifactResult<ArtifactRef> {
        if bytes.len() > MAX_OBJECT_BYTES {
            return Err(ArtifactError::SizeMismatch {
                context: "artifact publication limit".to_owned(),
                expected: MAX_OBJECT_BYTES as u64,
                actual: bytes.len() as u64,
            });
        }

        let digest = hex_digest(bytes);
        let artifact = ArtifactRef::sha256(&digest, media_type, bytes.len() as u64)
            .map_err(|source| ArtifactError::InvalidArtifactRef { source })?;
        let directory =
            self.object_directory(&digest, true)?
                .ok_or_else(|| ArtifactError::Conflict {
                    context: digest.clone(),
                    message: "object directory was not created".to_owned(),
                    source: None,
                })?;

        self.publish_immutable_bytes(
            &directory,
            &directory,
            &digest,
            bytes,
            MAX_OBJECT_BYTES,
            ImmutablePurpose::Object,
        )?;
        Ok(artifact)
    }

    pub fn read_verified(&self, artifact: &ArtifactRef) -> ArtifactResult<Vec<u8>> {
        artifact
            .validate()
            .map_err(|source| ArtifactError::InvalidArtifactRef { source })?;
        if artifact.size_bytes > MAX_OBJECT_BYTES as u64 {
            return Err(ArtifactError::SizeMismatch {
                context: artifact.sha256.clone(),
                expected: MAX_OBJECT_BYTES as u64,
                actual: artifact.size_bytes,
            });
        }

        let directory = self
            .object_directory(&artifact.sha256, false)?
            .ok_or_else(|| ArtifactError::Io {
                operation: "open object directory",
                context: artifact.sha256.clone(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let object = directory
            .open_regular_optional(&artifact.sha256)
            .map_err(|error| ArtifactError::from_secure("open object", &artifact.sha256, error))?
            .ok_or_else(|| ArtifactError::Io {
                operation: "open object",
                context: artifact.sha256.clone(),
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        verify_open_object(object, artifact)
    }

    /// Returns the display/materialization path derived from a validated
    /// attempt identity. Filesystem authority remains with the held
    /// descriptor inside this store; callers must not use this path to mutate
    /// checkpoint or evidence state.
    pub fn attempt_dir(&self, key: &AttemptKey) -> ArtifactResult<PathBuf> {
        Ok(self
            .root_path
            .join("runs")
            .join(key.run_id.to_string())
            .join("tasks")
            .join(key.task_id.to_string())
            .join("attempts")
            .join(key.attempt_id.to_string()))
    }

    pub fn resolve_attempt_scratch(
        &self,
        key: &AttemptKey,
    ) -> ArtifactResult<ResolvedAttemptScratch> {
        let directory =
            self.attempt_directory(key, true)?
                .ok_or_else(|| ArtifactError::Conflict {
                    context: "attempt scratch directory".to_owned(),
                    message: "attempt directory was not created".to_owned(),
                    source: None,
                })?;
        let identity = directory.identity().map_err(|error| {
            ArtifactError::from_secure(
                "inspect attempt scratch identity",
                key.attempt_id.to_string(),
                error,
            )
        })?;
        let canonical_path =
            std::fs::canonicalize(self.attempt_dir(key)?).map_err(|source| ArtifactError::Io {
                operation: "canonicalize attempt scratch",
                context: key.attempt_id.to_string(),
                source,
            })?;
        Ok(ResolvedAttemptScratch {
            key: key.clone(),
            store_identity: self.identity()?,
            canonical_path,
            device: identity.device(),
            inode: identity.inode(),
        })
    }

    pub fn verify_runtime_path(
        &self,
        scratch: &ResolvedAttemptScratch,
    ) -> ArtifactResult<VerifiedRuntimePath> {
        if scratch.store_identity != self.identity()? {
            return Err(ArtifactError::IdentityMismatch {
                context: "runtime scratch store".to_owned(),
                expected: format!(
                    "{}:{}",
                    scratch.store_identity.device, scratch.store_identity.inode
                ),
                actual: format!("{}:{}", self.identity()?.device, self.identity()?.inode),
            });
        }
        let visible_root =
            Dir::open_root(&scratch.store_identity.canonical_root).map_err(|error| {
                ArtifactError::from_secure(
                    "open visible runtime artifact root",
                    scratch.store_identity.canonical_root.display().to_string(),
                    error,
                )
            })?;
        let visible_root_identity = visible_root.identity().map_err(|error| {
            ArtifactError::from_secure("inspect visible runtime artifact root", "root", error)
        })?;
        if visible_root_identity.device() != scratch.store_identity.device
            || visible_root_identity.inode() != scratch.store_identity.inode
        {
            return Err(ArtifactError::IdentityMismatch {
                context: "visible runtime artifact root".to_owned(),
                expected: format!(
                    "{}:{}",
                    scratch.store_identity.device, scratch.store_identity.inode
                ),
                actual: visible_root_identity.to_string(),
            });
        }
        let visible_attempt = walk_attempt_directory(&visible_root, &scratch.key)?;
        let visible_attempt_identity = visible_attempt.identity().map_err(|error| {
            ArtifactError::from_secure(
                "inspect visible runtime attempt",
                scratch.key.attempt_id.to_string(),
                error,
            )
        })?;
        if visible_attempt_identity.device() != scratch.device
            || visible_attempt_identity.inode() != scratch.inode
        {
            return Err(ArtifactError::IdentityMismatch {
                context: "visible runtime attempt".to_owned(),
                expected: format!("{}:{}", scratch.device, scratch.inode),
                actual: visible_attempt_identity.to_string(),
            });
        }
        let canonical =
            std::fs::canonicalize(&scratch.canonical_path).map_err(|source| ArtifactError::Io {
                operation: "canonicalize verified runtime attempt",
                context: scratch.canonical_path.display().to_string(),
                source,
            })?;
        if canonical != scratch.canonical_path {
            return Err(ArtifactError::IdentityMismatch {
                context: "verified runtime path".to_owned(),
                expected: scratch.canonical_path.display().to_string(),
                actual: canonical.display().to_string(),
            });
        }
        Ok(VerifiedRuntimePath {
            path: scratch.canonical_path.clone(),
        })
    }

    pub fn attempt_storage_usage(
        &self,
        key: &AttemptKey,
        max_bytes: u64,
    ) -> ArtifactResult<AttemptStorageUsage> {
        if max_bytes == 0 || max_bytes > MAX_ATTEMPT_SCAN_BYTES {
            return Err(ArtifactError::StoragePolicyViolation {
                context: "attempt storage scan limit".to_owned(),
                message: format!(
                    "scan limit must be within 1..={MAX_ATTEMPT_SCAN_BYTES}, got {max_bytes}"
                ),
                observed_bytes: None,
            });
        }
        let Some(attempt) = self.attempt_directory(key, false)? else {
            return Ok(AttemptStorageUsage::default());
        };
        let mut usage = AttemptStorageUsage::default();
        let mut entries = 0usize;
        scan_attempt_directory(&attempt, "", 0, &mut entries, &mut usage, max_bytes)?;
        Ok(usage)
    }

    pub(crate) fn attempt_directory(
        &self,
        key: &AttemptKey,
        create: bool,
    ) -> ArtifactResult<Option<Dir>> {
        let Some(runs) = self.child_directory(&self.root, "runs", create, "runs")? else {
            return Ok(None);
        };
        let run = key.run_id.to_string();
        let Some(run_dir) = self.child_directory(&runs, &run, create, &run)? else {
            return Ok(None);
        };
        let Some(tasks) = self.child_directory(&run_dir, "tasks", create, "tasks")? else {
            return Ok(None);
        };
        let task = key.task_id.to_string();
        let Some(task_dir) = self.child_directory(&tasks, &task, create, &task)? else {
            return Ok(None);
        };
        let Some(attempts) = self.child_directory(&task_dir, "attempts", create, "attempts")?
        else {
            return Ok(None);
        };
        let attempt = key.attempt_id.to_string();
        self.child_directory(&attempts, &attempt, create, &attempt)
    }

    /// Publishes immutable bytes from an unlinked held source descriptor.
    ///
    /// The store's descriptor-bound `0700` directories are the authority
    /// boundary. The final directory-entry identity check is the success
    /// boundary; substitutions after this method returns require control of
    /// that trusted directory and are outside this operation.
    pub(crate) fn publish_immutable_bytes(
        &self,
        scratch_directory: &Dir,
        destination_directory: &Dir,
        destination_name: &str,
        bytes: &[u8],
        max_bytes: usize,
        purpose: ImmutablePurpose,
    ) -> ArtifactResult<()> {
        if bytes.len() > max_bytes {
            return Err(ArtifactError::SizeMismatch {
                context: destination_name.to_owned(),
                expected: max_bytes as u64,
                actual: bytes.len() as u64,
            });
        }
        if let Some(existing) = destination_directory
            .open_regular_optional(destination_name)
            .map_err(|error| {
                ArtifactError::from_secure("open immutable destination", destination_name, error)
            })?
        {
            let identity = existing.identity();
            verify_exact_file(&existing, bytes, max_bytes, destination_name)?;
            return self.finish_immutable_destination(
                destination_directory,
                destination_name,
                identity,
                purpose,
                &existing,
            );
        }

        let init_fault = self.take_temp_init_fault();
        let mut temporary = scratch_directory
            .create_temp(purpose.label(), init_fault)
            .map_err(|error| {
                ArtifactError::from_secure("create immutable temp", destination_name, error)
            })?;
        #[cfg(test)]
        if self.take_fault(FaultPoint::TempWrite) {
            return Err(ArtifactError::Io {
                operation: "write immutable temp",
                context: destination_name.to_owned(),
                source: injected_error(FaultPoint::TempWrite),
            });
        }
        if let Err(source) = temporary.write_all(bytes) {
            return Err(ArtifactError::Io {
                operation: "write immutable temp",
                context: destination_name.to_owned(),
                source,
            });
        }
        #[cfg(test)]
        if self.take_fault(FaultPoint::FileSync) {
            return Err(ArtifactError::Io {
                operation: "sync immutable temp",
                context: destination_name.to_owned(),
                source: injected_error(FaultPoint::FileSync),
            });
        }
        if let Err(source) = temporary.sync_all() {
            return Err(ArtifactError::Io {
                operation: "sync immutable temp",
                context: destination_name.to_owned(),
                source,
            });
        }

        #[cfg(test)]
        let publish_result = if self.take_fault(FaultPoint::ImmutablePublish) {
            Err(SecureError::Io(injected_error(
                FaultPoint::ImmutablePublish,
            )))
        } else {
            temporary.publish_immutable(destination_directory, destination_name)
        };
        #[cfg(not(test))]
        let publish_result = temporary.publish_immutable(destination_directory, destination_name);

        match publish_result {
            Ok(destination) => {
                let identity = destination.identity();
                #[cfg(test)]
                if self.take_fault(FaultPoint::DestinationVerification) {
                    return Err(ArtifactError::Conflict {
                        context: destination_name.to_owned(),
                        message: "injected destination verification failure".to_owned(),
                        source: None,
                    });
                }
                verify_exact_file(&destination, bytes, max_bytes, destination_name)?;
                self.finish_immutable_destination(
                    destination_directory,
                    destination_name,
                    identity,
                    purpose,
                    &destination,
                )?;
            }
            Err(SecureError::AlreadyExists) => {
                let existing = destination_directory
                    .open_regular_optional(destination_name)
                    .map_err(|error| {
                        ArtifactError::from_secure(
                            "open concurrent immutable destination",
                            destination_name,
                            error,
                        )
                    })?
                    .ok_or_else(|| ArtifactError::Conflict {
                        context: destination_name.to_owned(),
                        message: "concurrent immutable destination disappeared".to_owned(),
                        source: None,
                    })?;
                let identity = existing.identity();
                verify_exact_file(&existing, bytes, max_bytes, destination_name)?;
                self.finish_immutable_destination(
                    destination_directory,
                    destination_name,
                    identity,
                    purpose,
                    &existing,
                )?;
            }
            Err(error) => {
                return Err(ArtifactError::from_secure(
                    "publish immutable destination",
                    destination_name,
                    error,
                ));
            }
        }
        Ok(())
    }

    fn finish_immutable_destination(
        &self,
        directory: &Dir,
        name: &str,
        identity: crate::platform::Identity,
        _purpose: ImmutablePurpose,
        file: &crate::platform::RegularFile,
    ) -> ArtifactResult<()> {
        file.sync_all().map_err(|source| ArtifactError::Io {
            operation: "sync immutable destination",
            context: name.to_owned(),
            source,
        })?;
        #[cfg(test)]
        self.test_hooks.record_destination_file_sync();

        #[cfg(test)]
        if self.take_fault(FaultPoint::PublishDirectorySync) {
            return Err(ArtifactError::Io {
                operation: "sync immutable destination directory",
                context: name.to_owned(),
                source: injected_error(FaultPoint::PublishDirectorySync),
            });
        }
        directory.sync().map_err(|error| {
            ArtifactError::from_secure("sync immutable destination directory", name, error)
        })?;
        #[cfg(test)]
        self.test_hooks.record_destination_directory_sync();

        #[cfg(test)]
        if self.take_fault(FaultPoint::DestinationVerification) {
            return Err(ArtifactError::Conflict {
                context: name.to_owned(),
                message: "injected destination verification failure".to_owned(),
                source: None,
            });
        }
        #[cfg(test)]
        self.maybe_substitute_destination(directory, name, _purpose.substitution_purpose())?;
        directory
            .verify_entry_identity(name, identity)
            .map_err(|error| {
                ArtifactError::from_secure("verify immutable destination identity", name, error)
            })?;
        #[cfg(test)]
        self.test_hooks.record_destination_identity_check();
        Ok(())
    }

    fn object_directory(&self, digest: &str, create: bool) -> ArtifactResult<Option<Dir>> {
        let Some(objects) = self.child_directory(&self.root, "objects", create, "objects")? else {
            return Ok(None);
        };
        let Some(sha256) = self.child_directory(&objects, "sha256", create, "sha256")? else {
            return Ok(None);
        };
        self.child_directory(&sha256, &digest[..2], create, &digest[..2])
    }

    fn child_directory(
        &self,
        parent: &Dir,
        name: &str,
        create: bool,
        context: &str,
    ) -> ArtifactResult<Option<Dir>> {
        if create {
            let (directory, created) = parent
                .ensure_dir(name)
                .map_err(|error| ArtifactError::from_secure("ensure directory", context, error))?;
            #[cfg(not(test))]
            let _ = created;
            #[cfg(test)]
            if created && self.take_fault(FaultPoint::ParentDirectorySync) {
                return Err(ArtifactError::Io {
                    operation: "sync parent directory",
                    context: context.to_owned(),
                    source: injected_error(FaultPoint::ParentDirectorySync),
                });
            }
            Ok(Some(directory))
        } else {
            match parent.open_dir(name) {
                Ok(directory) => Ok(Some(directory)),
                Err(SecureError::NotFound) => Ok(None),
                Err(error) => Err(ArtifactError::from_secure("open directory", context, error)),
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn take_fault(&self, point: FaultPoint) -> bool {
        self.test_hooks.take_failure(point)
    }

    #[cfg(test)]
    pub(crate) fn maybe_substitute_destination(
        &self,
        directory: &Dir,
        name: &str,
        purpose: SubstitutionPurpose,
    ) -> ArtifactResult<()> {
        if self.take_fault(FaultPoint::DestinationIdentitySubstitution) {
            if let Some(target) = self.test_hooks.take_substitution(purpose) {
                directory
                    .replace_entry_with_symlink(name, &target)
                    .map_err(|error| {
                        ArtifactError::from_secure(
                            "substitute immutable destination test hook",
                            name,
                            error,
                        )
                    })?;
            }
        }
        Ok(())
    }

    fn take_temp_init_fault(&self) -> Option<TempInitFault> {
        #[cfg(test)]
        {
            if self.take_fault(FaultPoint::AnonymousTmpfile) {
                return Some(TempInitFault::Tmpfile);
            }
            if self.take_fault(FaultPoint::TempFchmod) {
                return Some(TempInitFault::Fchmod);
            }
            if self.take_fault(FaultPoint::TempFstat) {
                return Some(TempInitFault::Fstat);
            }
            if self.take_fault(FaultPoint::TempDup) {
                return Some(TempInitFault::Dup);
            }
        }
        None
    }
}

fn walk_attempt_directory(root: &Dir, key: &AttemptKey) -> ArtifactResult<Dir> {
    let components = [
        "runs".to_owned(),
        key.run_id.to_string(),
        "tasks".to_owned(),
        key.task_id.to_string(),
        "attempts".to_owned(),
        key.attempt_id.to_string(),
    ];
    let mut current = root.open_dir(&components[0]).map_err(|error| {
        ArtifactError::from_secure("walk visible runtime path", &components[0], error)
    })?;
    for component in &components[1..] {
        current = current.open_dir(component).map_err(|error| {
            ArtifactError::from_secure("walk visible runtime path", component, error)
        })?;
    }
    Ok(current)
}

fn scan_attempt_directory(
    directory: &Dir,
    prefix: &str,
    depth: usize,
    entries: &mut usize,
    usage: &mut AttemptStorageUsage,
    max_bytes: u64,
) -> ArtifactResult<()> {
    if depth > MAX_ATTEMPT_SCAN_DEPTH {
        return Err(ArtifactError::StoragePolicyViolation {
            context: "attempt storage scan depth".to_owned(),
            message: format!("depth exceeds {MAX_ATTEMPT_SCAN_DEPTH}"),
            observed_bytes: None,
        });
    }
    let names = directory
        .read_entry_names(MAX_ATTEMPT_SCAN_ENTRIES.saturating_add(1))
        .map_err(|error| match error {
            SecureError::TooManyEntries { .. } => ArtifactError::StoragePolicyViolation {
                context: prefix.to_owned(),
                message: format!("entry count exceeds {MAX_ATTEMPT_SCAN_ENTRIES}"),
                observed_bytes: None,
            },
            other => ArtifactError::from_secure("enumerate attempt storage", prefix, other),
        })?;
    for name in names {
        *entries = entries.saturating_add(1);
        if *entries > MAX_ATTEMPT_SCAN_ENTRIES {
            return Err(ArtifactError::StoragePolicyViolation {
                context: "attempt storage scan".to_owned(),
                message: format!("entry count exceeds {MAX_ATTEMPT_SCAN_ENTRIES}"),
                observed_bytes: None,
            });
        }
        let relative = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        let metadata = directory.entry_metadata(&name).map_err(|error| {
            ArtifactError::from_secure("inspect attempt storage entry", &relative, error)
        })?;
        match metadata.kind {
            EntryKind::Directory => {
                let child = directory.open_dir(&name).map_err(|error| {
                    ArtifactError::from_secure("open attempt storage directory", &relative, error)
                })?;
                scan_attempt_directory(&child, &relative, depth + 1, entries, usage, max_bytes)?;
            }
            EntryKind::Regular => {
                let file = directory
                    .open_regular_optional(&name)
                    .map_err(|error| {
                        ArtifactError::from_secure("open attempt storage file", &relative, error)
                    })?
                    .ok_or_else(|| ArtifactError::Conflict {
                        context: relative.clone(),
                        message: "attempt storage entry disappeared during scan".to_owned(),
                        source: None,
                    })?;
                if metadata.links != 1 || file.link_count() != 1 {
                    return Err(ArtifactError::StoragePolicyViolation {
                        context: relative,
                        message: "attempt storage regular file has multiple hard links".to_owned(),
                        observed_bytes: None,
                    });
                }
                add_attempt_file_usage(usage, &relative, file.size(), max_bytes)?;
            }
            EntryKind::Other => {
                return Err(ArtifactError::StoragePolicyViolation {
                    context: relative,
                    message: "attempt storage entry is not a regular file or directory".to_owned(),
                    observed_bytes: None,
                });
            }
        }
    }
    Ok(())
}

fn add_attempt_file_usage(
    usage: &mut AttemptStorageUsage,
    relative: &str,
    bytes: u64,
    max_bytes: u64,
) -> ArtifactResult<()> {
    let category = if relative.starts_with("checkpoints/") {
        &mut usage.checkpoint_bytes
    } else if relative == EVIDENCE_FILE {
        &mut usage.evidence_bytes
    } else if relative == "notes.md" {
        &mut usage.notes_bytes
    } else if relative == "result.json" {
        &mut usage.result_scratch_bytes
    } else {
        &mut usage.other_bytes
    };
    *category = category
        .checked_add(u128::from(bytes))
        .ok_or(ArtifactError::SizeMismatch {
            context: "attempt storage usage overflow".to_owned(),
            expected: u64::MAX,
            actual: u64::MAX,
        })?;
    let logical_bytes = usage.total_bytes()?;
    if logical_bytes > u128::from(max_bytes) {
        return Err(ArtifactError::StoragePolicyViolation {
            context: "attempt storage scan limit".to_owned(),
            message: format!("logical bytes {logical_bytes} exceed configured limit {max_bytes}"),
            observed_bytes: Some(logical_bytes),
        });
    }
    Ok(())
}

impl ImmutablePurpose {
    fn label(self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Checkpoint => "checkpoint",
        }
    }

    #[cfg(test)]
    fn substitution_purpose(self) -> SubstitutionPurpose {
        match self {
            Self::Object => SubstitutionPurpose::ObjectDestination,
            Self::Checkpoint => SubstitutionPurpose::CheckpointDestination,
        }
    }
}

fn verify_exact_file(
    file: &crate::platform::RegularFile,
    expected: &[u8],
    max_bytes: usize,
    context: &str,
) -> ArtifactResult<()> {
    if file.size() != expected.len() as u64 {
        return Err(ArtifactError::SizeMismatch {
            context: context.to_owned(),
            expected: expected.len() as u64,
            actual: file.size(),
        });
    }
    let actual = file
        .read_bounded(max_bytes)
        .map_err(|source| ArtifactError::Io {
            operation: "read immutable destination",
            context: context.to_owned(),
            source,
        })?;
    if actual != expected {
        return Err(ArtifactError::DigestMismatch {
            context: context.to_owned(),
            expected: hex_digest(expected),
            actual: hex_digest(&actual),
        });
    }
    Ok(())
}

fn verify_open_object(
    object: crate::platform::RegularFile,
    artifact: &ArtifactRef,
) -> ArtifactResult<Vec<u8>> {
    if object.size() != artifact.size_bytes {
        return Err(ArtifactError::SizeMismatch {
            context: artifact.sha256.clone(),
            expected: artifact.size_bytes,
            actual: object.size(),
        });
    }
    if object.size() > MAX_OBJECT_BYTES as u64 {
        return Err(ArtifactError::SizeMismatch {
            context: artifact.sha256.clone(),
            expected: MAX_OBJECT_BYTES as u64,
            actual: object.size(),
        });
    }
    let bytes = object
        .read_bounded(MAX_OBJECT_BYTES)
        .map_err(|source| ArtifactError::Io {
            operation: "read object",
            context: artifact.sha256.clone(),
            source,
        })?;
    if bytes.len() as u64 != artifact.size_bytes {
        return Err(ArtifactError::SizeMismatch {
            context: artifact.sha256.clone(),
            expected: artifact.size_bytes,
            actual: bytes.len() as u64,
        });
    }
    verify_object_bytes(&bytes, artifact)?;
    Ok(bytes)
}

fn verify_object_bytes(bytes: &[u8], artifact: &ArtifactRef) -> ArtifactResult<()> {
    if bytes.len() as u64 != artifact.size_bytes {
        return Err(ArtifactError::SizeMismatch {
            context: artifact.sha256.clone(),
            expected: artifact.size_bytes,
            actual: bytes.len() as u64,
        });
    }
    let actual = hex_digest(bytes);
    if actual != artifact.sha256 {
        return Err(ArtifactError::DigestMismatch {
            context: artifact.sha256.clone(),
            expected: artifact.sha256.clone(),
            actual,
        });
    }
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn secure_source(error: SecureError) -> Option<std::io::Error> {
    match error {
        SecureError::SymlinkOrWrongType(source) | SecureError::Io(source) => Some(source),
        SecureError::NotFound => Some(std::io::Error::from(std::io::ErrorKind::NotFound)),
        SecureError::AlreadyExists => Some(std::io::Error::from(std::io::ErrorKind::AlreadyExists)),
        SecureError::IdentityMismatch { expected, actual } => Some(std::io::Error::other(format!(
            "expected identity {expected}, opened {actual}"
        ))),
        SecureError::TooManyEntries { max_entries } => Some(std::io::Error::other(format!(
            "directory contains more than {max_entries} entries"
        ))),
        SecureError::Unsupported => Some(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "descriptor-relative store is unsupported on this platform",
        )),
    }
}

#[cfg(test)]
pub(crate) fn injected_error(point: FaultPoint) -> std::io::Error {
    std::io::Error::other(format!("injected {point:?} failure"))
}
