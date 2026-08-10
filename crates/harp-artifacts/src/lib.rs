mod checkpoint;
mod platform;
mod store;

use std::io;
use std::path::PathBuf;

pub use checkpoint::CheckpointVersion;
pub use store::verify_runtime_workspace_authority;
pub use store::{
    ArtifactStore, ArtifactStoreIdentity, AttemptKey, AttemptStorageUsage, StorageLimitObservation,
    MAX_ATTEMPT_SCAN_BYTES,
};
#[doc(hidden)]
pub use store::{ResolvedAttemptScratch, VerifiedRuntimePath};

pub type ArtifactResult<T> = Result<T, ArtifactError>;

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("invalid artifact store root {path}: {context}")]
    InvalidRoot {
        path: PathBuf,
        context: String,
        #[source]
        source: Option<io::Error>,
    },

    #[error("invalid artifact reference")]
    InvalidArtifactRef {
        #[source]
        source: harp_contracts::ContractError,
    },

    #[error("invalid checkpoint")]
    InvalidCheckpoint {
        #[source]
        source: harp_contracts::ContractError,
    },
    #[error("invalid runtime workspace authority")]
    InvalidWorkspaceAuthority {
        #[source]
        source: harp_contracts::ContractError,
    },

    #[error("{operation} failed for {context}")]
    Io {
        operation: &'static str,
        context: String,
        #[source]
        source: io::Error,
    },

    #[error("digest mismatch for {context}: expected {expected}, got {actual}")]
    DigestMismatch {
        context: String,
        expected: String,
        actual: String,
    },

    #[error("size mismatch for {context}: expected {expected} bytes, got {actual}")]
    SizeMismatch {
        context: String,
        expected: u64,
        actual: u64,
    },

    #[error("evidence record exceeds the {max_bytes}-byte encoded line limit")]
    EvidenceTooLarge { max_bytes: usize },

    #[error("evidence recovery required for {context}: {message}")]
    EvidenceRecoveryRequired { context: String, message: String },

    #[error("evidence contains more than {max_records} records")]
    EvidenceTooManyRecords { max_records: usize },

    #[error("{context} contains more than {max_entries} entries")]
    TooManyEntries { context: String, max_entries: usize },

    #[error("attempt storage policy violation at {context}: {message}")]
    StoragePolicyViolation {
        context: String,
        message: String,
        observed_bytes: Option<u128>,
    },

    #[error("checkpoint scan exceeds the {max_bytes}-byte aggregate limit")]
    CheckpointScanTooLarge { max_bytes: usize },

    #[error("symlink or wrong entry type at {context}")]
    SymlinkOrWrongType {
        context: String,
        #[source]
        source: Option<io::Error>,
    },

    #[error("JSON serialization failed for {context}")]
    Serialization {
        context: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("identity mismatch for {context}: expected {expected}, got {actual}")]
    IdentityMismatch {
        context: String,
        expected: String,
        actual: String,
    },

    #[error("publication conflict at {context}: {message}")]
    Conflict {
        context: String,
        message: String,
        #[source]
        source: Option<io::Error>,
    },

    #[error("unsupported artifact-store operation: {context}")]
    Unsupported { context: String },
}

impl ArtifactError {
    pub(crate) fn from_secure(
        operation: &'static str,
        context: impl Into<String>,
        error: platform::SecureError,
    ) -> Self {
        let context = context.into();
        match error {
            platform::SecureError::NotFound => Self::Io {
                operation,
                context,
                source: io::Error::from(io::ErrorKind::NotFound),
            },
            platform::SecureError::AlreadyExists => Self::Conflict {
                context,
                message: "entry already exists".to_owned(),
                source: Some(io::Error::from(io::ErrorKind::AlreadyExists)),
            },
            platform::SecureError::SymlinkOrWrongType(source) => Self::SymlinkOrWrongType {
                context,
                source: Some(source),
            },
            platform::SecureError::IdentityMismatch { expected, actual } => {
                Self::IdentityMismatch {
                    context,
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                }
            }
            platform::SecureError::Io(source) => Self::Io {
                operation,
                context,
                source,
            },
            platform::SecureError::TooManyEntries { max_entries } => Self::TooManyEntries {
                context,
                max_entries,
            },
            platform::SecureError::Unsupported => Self::Unsupported { context },
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub(crate) enum FaultPoint {
        AnonymousTmpfile,
        TempFchmod,
        TempFstat,
        TempDup,
        TempWrite,
        FileSync,
        ImmutablePublish,
        DestinationVerification,
        DestinationIdentitySubstitution,
        ParentDirectorySync,
        PublishDirectorySync,
        EvidenceDirectorySync,
        EvidencePartialWrite,
        EvidenceIdentitySubstitution,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum SubstitutionPurpose {
        ObjectDestination,
        CheckpointDestination,
        Evidence,
    }

    #[derive(Default)]
    pub(crate) struct TestHooks {
        state: Mutex<TestHookState>,
    }

    #[derive(Default)]
    struct TestHookState {
        failures: HashMap<FaultPoint, usize>,
        substitution: Option<(SubstitutionPurpose, PathBuf)>,
        anonymous_device_override: Option<u64>,
        destination_file_syncs: usize,
        destination_directory_syncs: usize,
        destination_identity_checks: usize,
    }

    impl std::fmt::Debug for TestHooks {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("TestHooks")
        }
    }

    impl TestHooks {
        pub(crate) fn fail_next(&self, point: FaultPoint) {
            let mut state = self.state.lock().expect("test hook lock");
            *state.failures.entry(point).or_default() += 1;
        }

        pub(crate) fn take_failure(&self, point: FaultPoint) -> bool {
            let mut state = self.state.lock().expect("test hook lock");
            let Some(remaining) = state.failures.get_mut(&point) else {
                return false;
            };
            *remaining -= 1;
            if *remaining == 0 {
                state.failures.remove(&point);
            }
            true
        }

        pub(crate) fn substitute_next_destination(
            &self,
            purpose: SubstitutionPurpose,
            target: PathBuf,
        ) {
            self.state.lock().expect("test hook lock").substitution = Some((purpose, target));
        }

        pub(crate) fn take_substitution(&self, purpose: SubstitutionPurpose) -> Option<PathBuf> {
            let mut state = self.state.lock().expect("test hook lock");
            if state
                .substitution
                .as_ref()
                .is_some_and(|(configured, _)| *configured == purpose)
            {
                state.substitution.take().map(|(_, target)| target)
            } else {
                None
            }
        }

        pub(crate) fn record_destination_file_sync(&self) {
            self.state
                .lock()
                .expect("test hook lock")
                .destination_file_syncs += 1;
        }

        pub(crate) fn record_destination_directory_sync(&self) {
            self.state
                .lock()
                .expect("test hook lock")
                .destination_directory_syncs += 1;
        }

        pub(crate) fn record_destination_identity_check(&self) {
            self.state
                .lock()
                .expect("test hook lock")
                .destination_identity_checks += 1;
        }

        pub(crate) fn reset_destination_gates(&self) {
            let mut state = self.state.lock().expect("test hook lock");
            state.destination_file_syncs = 0;
            state.destination_directory_syncs = 0;
            state.destination_identity_checks = 0;
        }

        pub(crate) fn destination_gates(&self) -> (usize, usize, usize) {
            let state = self.state.lock().expect("test hook lock");
            (
                state.destination_file_syncs,
                state.destination_directory_syncs,
                state.destination_identity_checks,
            )
        }

        pub(crate) fn override_anonymous_device(&self, device: u64) {
            self.state
                .lock()
                .expect("test hook lock")
                .anonymous_device_override = Some(device);
        }

        pub(crate) fn anonymous_device_override(&self) -> Option<u64> {
            self.state
                .lock()
                .expect("test hook lock")
                .anonymous_device_override
        }
    }
}

#[cfg(test)]
mod hardening_tests {
    use std::fs;
    use std::str::FromStr;
    use std::sync::Arc;

    use harp_contracts::{AttemptId, Checkpoint, RunId, TaskId};
    use sha2::Digest;

    use crate::test_support::{FaultPoint, SubstitutionPurpose, TestHooks};
    use crate::{ArtifactError, ArtifactStore, AttemptKey};

    fn key() -> AttemptKey {
        AttemptKey {
            run_id: RunId::new(),
            task_id: TaskId::from_str("fault-task").unwrap(),
            attempt_id: AttemptId::new(),
        }
    }

    fn hooked_store(root: &tempfile::TempDir, hooks: Arc<TestHooks>) -> ArtifactStore {
        ArtifactStore::open_with_test_hooks(root.path(), hooks).unwrap()
    }

    fn temp_entries(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        fn visit(path: &std::path::Path, entries: &mut Vec<std::path::PathBuf>) {
            let Ok(children) = fs::read_dir(path) else {
                return;
            };
            for child in children {
                let child = child.unwrap();
                if child.file_name().to_string_lossy().starts_with(".tmp-") {
                    entries.push(child.path());
                }
                if child.file_type().unwrap().is_dir() {
                    visit(&child.path(), entries);
                }
            }
        }

        let mut entries = Vec::new();
        visit(root, &mut entries);
        entries
    }

    #[test]
    fn artifact_store_open_accepts_same_device_anonymous_source() {
        let root = tempfile::tempdir().unwrap();

        ArtifactStore::open(root.path()).unwrap();

        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn artifact_store_open_rejects_cross_device_anonymous_source() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.override_anonymous_device(u64::MAX);

        assert!(matches!(
            ArtifactStore::open_with_test_hooks(root.path(), hooks).unwrap_err(),
            ArtifactError::Unsupported { context }
                if context == "anonymous publication source and store root are on different filesystems; immutable atomic publication unsupported"
        ));
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }

    #[test]
    fn unnamed_held_source_publishes_without_a_scratch_entry() {
        let root = tempfile::tempdir().unwrap();
        let store = ArtifactStore::open(root.path()).unwrap();

        let artifact = store.publish(b"object", "text/plain").unwrap();
        assert_eq!(store.read_verified(&artifact).unwrap(), b"object");
        assert!(temp_entries(root.path()).is_empty());
    }

    #[test]
    fn unpublished_held_source_has_no_scratch_entry() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.fail_next(FaultPoint::ImmutablePublish);
        let store = hooked_store(&root, hooks);

        assert!(store.publish(b"never-published", "text/plain").is_err());
        assert!(temp_entries(root.path()).is_empty());
    }

    #[test]
    fn injected_object_failures_never_report_success_or_leak_owned_temp() {
        for point in [
            FaultPoint::TempWrite,
            FaultPoint::FileSync,
            FaultPoint::ImmutablePublish,
            FaultPoint::DestinationVerification,
            FaultPoint::PublishDirectorySync,
        ] {
            let root = tempfile::tempdir().unwrap();
            let hooks = Arc::new(TestHooks::default());
            hooks.fail_next(point);
            let store = hooked_store(&root, hooks);

            assert!(
                store.publish(b"fault", "text/plain").is_err(),
                "{point:?} returned false success"
            );
            assert!(
                temp_entries(root.path()).is_empty(),
                "{point:?} leaked an owned temp"
            );
        }
    }

    #[test]
    fn injected_parent_directory_sync_failure_is_propagated() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.fail_next(FaultPoint::ParentDirectorySync);
        let store = hooked_store(&root, hooks);

        assert!(matches!(
            store.publish(b"fault", "text/plain").unwrap_err(),
            ArtifactError::Io { .. }
        ));
        assert!(temp_entries(root.path()).is_empty());
    }

    #[test]
    fn injected_checkpoint_publish_failure_is_propagated_without_temp_leak() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.fail_next(FaultPoint::ImmutablePublish);
        let store = hooked_store(&root, hooks);
        let key = key();
        let checkpoint =
            Checkpoint::new(key.run_id, key.task_id, key.attempt_id, "running", 1).unwrap();

        assert!(store.write_checkpoint(&checkpoint).is_err());
        assert!(temp_entries(root.path()).is_empty());
    }

    #[test]
    fn injected_temp_initialization_failures_leave_no_named_temp() {
        for point in [
            FaultPoint::AnonymousTmpfile,
            FaultPoint::TempFchmod,
            FaultPoint::TempFstat,
            FaultPoint::TempDup,
        ] {
            let root = tempfile::tempdir().unwrap();
            let hooks = Arc::new(TestHooks::default());
            hooks.fail_next(point);
            let store = hooked_store(&root, hooks);

            assert!(store.publish(b"init-fault", "text/plain").is_err());
            assert!(
                temp_entries(root.path()).is_empty(),
                "{point:?} leaked a named temp"
            );
        }
    }

    #[test]
    fn retry_existing_object_runs_all_durability_gates() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        let store = hooked_store(&root, Arc::clone(&hooks));
        hooks.fail_next(FaultPoint::DestinationVerification);

        assert!(store.publish(b"retry-object", "text/plain").is_err());
        assert_eq!(hooks.destination_gates(), (0, 0, 0));

        hooks.reset_destination_gates();
        store.publish(b"retry-object", "text/plain").unwrap();
        assert_eq!(hooks.destination_gates(), (1, 1, 1));
    }

    #[test]
    fn retry_existing_checkpoint_runs_all_durability_gates() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        let store = hooked_store(&root, Arc::clone(&hooks));
        let key = key();
        let checkpoint =
            Checkpoint::new(key.run_id, key.task_id, key.attempt_id, "retry", 3).unwrap();
        hooks.fail_next(FaultPoint::PublishDirectorySync);

        assert!(store.write_checkpoint(&checkpoint).is_err());
        assert_eq!(hooks.destination_gates(), (1, 0, 0));

        hooks.reset_destination_gates();
        store.write_checkpoint(&checkpoint).unwrap();
        assert_eq!(hooks.destination_gates(), (1, 1, 1));
    }

    #[cfg(unix)]
    #[test]
    fn newly_cloned_object_destination_substitution_fails_without_deleting_replacement() {
        let root = tempfile::tempdir().unwrap();
        let sentinel = root.path().join("destination-sentinel");
        fs::write(&sentinel, b"replacement").unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.substitute_next_destination(SubstitutionPurpose::ObjectDestination, sentinel.clone());
        hooks.fail_next(FaultPoint::DestinationIdentitySubstitution);
        let store = hooked_store(&root, hooks);

        assert!(matches!(
            store.publish(b"new-object", "text/plain").unwrap_err(),
            ArtifactError::SymlinkOrWrongType { .. } | ArtifactError::IdentityMismatch { .. }
        ));
        assert_eq!(fs::read(&sentinel).unwrap(), b"replacement");
        let digest = format!("{:x}", sha2::Sha256::digest(b"new-object"));
        let replacement = root
            .path()
            .join("objects/sha256")
            .join(&digest[..2])
            .join(digest);
        assert!(fs::symlink_metadata(replacement)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn preexisting_object_destination_substitution_fails_without_deleting_replacement() {
        let root = tempfile::tempdir().unwrap();
        let initial = ArtifactStore::open(root.path()).unwrap();
        initial.publish(b"existing-object", "text/plain").unwrap();
        let sentinel = root.path().join("existing-destination-sentinel");
        fs::write(&sentinel, b"replacement").unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.substitute_next_destination(SubstitutionPurpose::ObjectDestination, sentinel.clone());
        hooks.fail_next(FaultPoint::DestinationIdentitySubstitution);
        let store = hooked_store(&root, hooks);

        assert!(matches!(
            store.publish(b"existing-object", "text/plain").unwrap_err(),
            ArtifactError::SymlinkOrWrongType { .. } | ArtifactError::IdentityMismatch { .. }
        ));
        assert_eq!(fs::read(&sentinel).unwrap(), b"replacement");
    }

    #[cfg(unix)]
    #[test]
    fn newly_cloned_checkpoint_destination_substitution_preserves_replacement() {
        let root = tempfile::tempdir().unwrap();
        let sentinel = root.path().join("checkpoint-destination-sentinel");
        fs::write(&sentinel, b"replacement").unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.substitute_next_destination(
            SubstitutionPurpose::CheckpointDestination,
            sentinel.clone(),
        );
        hooks.fail_next(FaultPoint::DestinationIdentitySubstitution);
        let store = hooked_store(&root, hooks);
        let key = key();
        let checkpoint =
            Checkpoint::new(key.run_id, key.task_id, key.attempt_id, "running", 1).unwrap();

        assert!(matches!(
            store.write_checkpoint(&checkpoint).unwrap_err(),
            ArtifactError::SymlinkOrWrongType { .. } | ArtifactError::IdentityMismatch { .. }
        ));
        assert_eq!(fs::read(&sentinel).unwrap(), b"replacement");
    }

    #[cfg(unix)]
    #[test]
    fn preexisting_checkpoint_destination_substitution_preserves_replacement() {
        let root = tempfile::tempdir().unwrap();
        let key = key();
        let checkpoint = Checkpoint::new(
            key.run_id.clone(),
            key.task_id.clone(),
            key.attempt_id.clone(),
            "running",
            2,
        )
        .unwrap();
        ArtifactStore::open(root.path())
            .unwrap()
            .write_checkpoint(&checkpoint)
            .unwrap();
        let sentinel = root.path().join("existing-checkpoint-sentinel");
        fs::write(&sentinel, b"replacement").unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.substitute_next_destination(
            SubstitutionPurpose::CheckpointDestination,
            sentinel.clone(),
        );
        hooks.fail_next(FaultPoint::DestinationIdentitySubstitution);
        let store = hooked_store(&root, hooks);

        assert!(matches!(
            store.write_checkpoint(&checkpoint).unwrap_err(),
            ArtifactError::SymlinkOrWrongType { .. } | ArtifactError::IdentityMismatch { .. }
        ));
        assert_eq!(fs::read(&sentinel).unwrap(), b"replacement");
    }

    #[test]
    fn injected_evidence_directory_sync_failure_is_propagated() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.fail_next(FaultPoint::EvidenceDirectorySync);
        let store = hooked_store(&root, hooks);

        assert!(store
            .append_evidence(&key(), &serde_json::json!({"record": 1}))
            .is_err());
    }

    #[test]
    fn injected_evidence_partial_write_is_reported_and_never_false_success() {
        let root = tempfile::tempdir().unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.fail_next(FaultPoint::EvidencePartialWrite);
        let store = hooked_store(&root, hooks);
        let key = key();

        assert!(store
            .append_evidence(&key, &serde_json::json!({"record": "partial"}))
            .is_err());
        assert!(matches!(
            store.read_evidence(&key, 1024).unwrap_err(),
            ArtifactError::Serialization { .. }
        ));
        let evidence = store.attempt_dir(&key).unwrap().join("evidence.jsonl");
        let partial = fs::read(&evidence).unwrap();
        let error = store
            .append_evidence(&key, &serde_json::json!({"future": "blocked"}))
            .unwrap_err();
        assert!(matches!(
            error,
            ArtifactError::EvidenceRecoveryRequired { .. }
        ));
        assert_eq!(fs::read(evidence).unwrap(), partial);
    }

    #[cfg(unix)]
    #[test]
    fn evidence_identity_substitution_after_open_is_rejected() {
        let root = tempfile::tempdir().unwrap();
        let sentinel = root.path().join("evidence-sentinel");
        fs::write(&sentinel, b"untouched").unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.substitute_next_destination(SubstitutionPurpose::Evidence, sentinel.clone());
        hooks.fail_next(FaultPoint::EvidenceIdentitySubstitution);
        let store = hooked_store(&root, hooks);

        assert!(matches!(
            store
                .append_evidence(&key(), &serde_json::json!({"record": 1}))
                .unwrap_err(),
            ArtifactError::IdentityMismatch { .. } | ArtifactError::SymlinkOrWrongType { .. }
        ));
        assert_eq!(fs::read(sentinel).unwrap(), b"untouched");
    }

    #[cfg(unix)]
    #[test]
    fn created_evidence_substitution_happens_after_directory_sync() {
        let root = tempfile::tempdir().unwrap();
        let sentinel = root.path().join("created-evidence-sentinel");
        fs::write(&sentinel, b"replacement").unwrap();
        let hooks = Arc::new(TestHooks::default());
        hooks.substitute_next_destination(SubstitutionPurpose::Evidence, sentinel.clone());
        hooks.fail_next(FaultPoint::EvidenceDirectorySync);
        hooks.fail_next(FaultPoint::EvidenceIdentitySubstitution);
        let store = hooked_store(&root, Arc::clone(&hooks));
        let key = key();

        assert!(store
            .append_evidence(&key, &serde_json::json!({"first": true}))
            .is_err());
        let evidence = store.attempt_dir(&key).unwrap().join("evidence.jsonl");
        assert!(fs::symlink_metadata(&evidence)
            .unwrap()
            .file_type()
            .is_file());

        let error = store
            .append_evidence(&key, &serde_json::json!({"second": true}))
            .unwrap_err();
        assert!(matches!(
            error,
            ArtifactError::IdentityMismatch { .. } | ArtifactError::SymlinkOrWrongType { .. }
        ));
        assert!(fs::symlink_metadata(evidence)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read(sentinel).unwrap(), b"replacement");
    }
}
