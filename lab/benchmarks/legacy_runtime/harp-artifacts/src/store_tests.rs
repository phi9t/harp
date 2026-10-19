use std::fs;
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::str::FromStr;
use std::sync::{Arc, Barrier};
use std::thread;

use harp_artifacts::{ArtifactError, ArtifactStore, AttemptKey, AttemptStorageUsage};
use harp_contracts::{ArtifactRef, AttemptId, Checkpoint, RunId, TaskId};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const MAX_OBJECT_BYTES: usize = 64 * 1024 * 1024;
const MAX_CHECKPOINT_VERSIONS: usize = 4096;
const MAX_EVIDENCE_RECORDS: usize = 65_536;

fn store() -> (TempDir, ArtifactStore) {
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(root.path()).unwrap();
    (root, store)
}

fn key(task_id: &str) -> AttemptKey {
    AttemptKey {
        run_id: RunId::new(),
        task_id: TaskId::from_str(task_id).unwrap(),
        attempt_id: AttemptId::new(),
    }
}

fn object_path(root: &TempDir, artifact: &ArtifactRef) -> std::path::PathBuf {
    root.path()
        .join("objects")
        .join("sha256")
        .join(&artifact.sha256[..2])
        .join(&artifact.sha256)
}

fn checkpoint_bytes(checkpoint: &Checkpoint) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(checkpoint).unwrap();
    bytes.push(b'\n');
    bytes
}

fn padded_checkpoint_bytes(checkpoint: &Checkpoint, size: usize) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(checkpoint).unwrap();
    assert!(bytes.len() < size);
    bytes.resize(size - 1, b' ');
    bytes.push(b'\n');
    bytes
}

fn checkpoint_name(timestamp: i64, bytes: &[u8]) -> String {
    let encoded_timestamp = (timestamp as u64) ^ (1_u64 << 63);
    let digest = format!("{:x}", Sha256::digest(bytes));
    format!("{encoded_timestamp:016x}-{digest}.json")
}

fn checkpoints_path(store: &ArtifactStore, key: &AttemptKey) -> std::path::PathBuf {
    store.attempt_dir(key).unwrap().join("checkpoints")
}

fn write_checkpoint_version(
    checkpoint_dir: &std::path::Path,
    key: &AttemptKey,
    timestamp: i64,
) -> Checkpoint {
    let checkpoint = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        format!("version-{timestamp}"),
        timestamp,
    )
    .unwrap();
    let bytes = checkpoint_bytes(&checkpoint);
    fs::write(
        checkpoint_dir.join(checkpoint_name(timestamp, &bytes)),
        bytes,
    )
    .unwrap();
    checkpoint
}

fn all_names(root: &std::path::Path) -> Vec<String> {
    fn visit(path: &std::path::Path, names: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries {
            let entry = entry.unwrap();
            names.push(entry.file_name().to_string_lossy().into_owned());
            if entry.file_type().unwrap().is_dir() {
                visit(&entry.path(), names);
            }
        }
    }

    let mut names = Vec::new();
    visit(root, &mut names);
    names
}

#[test]
fn publish_is_content_addressed_and_idempotent() {
    let (root, store) = store();

    let first = store.publish(b"hello", "text/plain").unwrap();
    let second = store.publish(b"hello", "text/plain").unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first.uri,
        "artifact://sha256/2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
    assert_eq!(first.sha256, first.uri["artifact://sha256/".len()..]);
    assert_eq!(first.size_bytes, 5);
    assert_eq!(first.media_type, "text/plain");
    assert_eq!(fs::read(object_path(&root, &first)).unwrap(), b"hello");
    assert_eq!(store.read_verified(&first).unwrap(), b"hello");
}

#[test]
fn different_bytes_publish_to_different_paths() {
    let (root, store) = store();

    let first = store.publish(b"first", "application/octet-stream").unwrap();
    let second = store
        .publish(b"second", "application/octet-stream")
        .unwrap();

    assert_ne!(first.sha256, second.sha256);
    assert_ne!(object_path(&root, &first), object_path(&root, &second));
    assert_eq!(fs::read(object_path(&root, &first)).unwrap(), b"first");
    assert_eq!(fs::read(object_path(&root, &second)).unwrap(), b"second");
}

#[test]
fn oversized_publish_is_rejected_before_creating_object_tree() {
    let (root, store) = store();
    let bytes = vec![0_u8; MAX_OBJECT_BYTES + 1];

    let error = store
        .publish(&bytes, "application/octet-stream")
        .unwrap_err();

    assert!(matches!(error, ArtifactError::SizeMismatch { .. }));
    assert!(!root.path().join("objects").exists());
}

#[test]
fn read_verified_detects_corruption_and_truncation() {
    let (root, store) = store();
    let corrupted = store.publish(b"abcdef", "text/plain").unwrap();
    fs::write(object_path(&root, &corrupted), b"abcdeg").unwrap();

    assert!(matches!(
        store.read_verified(&corrupted).unwrap_err(),
        ArtifactError::DigestMismatch { .. }
    ));

    let truncated = store.publish(b"123456", "text/plain").unwrap();
    fs::write(object_path(&root, &truncated), b"123").unwrap();
    assert!(matches!(
        store.read_verified(&truncated).unwrap_err(),
        ArtifactError::SizeMismatch { .. }
    ));
}

#[test]
fn corrupted_existing_object_is_never_replaced_on_republish() {
    let (root, store) = store();
    let artifact = store.publish(b"immutable", "text/plain").unwrap();
    let path = object_path(&root, &artifact);
    fs::write(&path, b"corrupted").unwrap();

    assert!(matches!(
        store.publish(b"immutable", "text/plain").unwrap_err(),
        ArtifactError::DigestMismatch { .. }
    ));
    assert_eq!(fs::read(path).unwrap(), b"corrupted");
}

#[test]
fn read_verified_rejects_invalid_artifact_reference() {
    let (_root, store) = store();
    let mut artifact = ArtifactRef::sha256("a".repeat(64), "text/plain", 1).unwrap();
    artifact.uri = format!("artifact://sha256/{}", "b".repeat(64));

    assert!(matches!(
        store.read_verified(&artifact).unwrap_err(),
        ArtifactError::InvalidArtifactRef { .. }
    ));
}

#[cfg(unix)]
#[test]
fn existing_object_symlink_and_non_regular_entry_are_rejected() {
    use std::os::unix::fs::symlink;

    let (root, store) = store();
    let expected = ArtifactRef::sha256(
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        "text/plain",
        5,
    )
    .unwrap();
    let path = object_path(&root, &expected);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let target = root.path().join("target");
    fs::write(&target, b"hello").unwrap();
    symlink(&target, &path).unwrap();

    assert!(matches!(
        store.publish(b"hello", "text/plain").unwrap_err(),
        ArtifactError::SymlinkOrWrongType { .. }
    ));

    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(matches!(
        store.publish(b"hello", "text/plain").unwrap_err(),
        ArtifactError::SymlinkOrWrongType { .. }
    ));
}

#[cfg(unix)]
#[test]
fn held_root_descriptor_prevents_ancestor_substitution() {
    use std::os::unix::fs::symlink;

    let parent = tempfile::tempdir().unwrap();
    let visible_root = parent.path().join("store");
    let held_root = parent.path().join("held-store");
    let outside = parent.path().join("outside");
    fs::create_dir(&visible_root).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("sentinel"), b"untouched").unwrap();
    let store = ArtifactStore::open(&visible_root).unwrap();

    fs::rename(&visible_root, &held_root).unwrap();
    symlink(&outside, &visible_root).unwrap();
    let artifact = store.publish(b"descriptor-bound", "text/plain").unwrap();

    let held_object = held_root
        .join("objects")
        .join("sha256")
        .join(&artifact.sha256[..2])
        .join(&artifact.sha256);
    assert_eq!(fs::read(held_object).unwrap(), b"descriptor-bound");
    assert_eq!(fs::read(outside.join("sentinel")).unwrap(), b"untouched");
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
}

#[test]
fn concurrent_same_byte_publishers_converge_without_temp_files() {
    let root = tempfile::tempdir().unwrap();
    let store = Arc::new(ArtifactStore::open(root.path()).unwrap());
    let barrier = Arc::new(Barrier::new(9));
    let mut threads = Vec::new();

    for _ in 0..8 {
        let store = Arc::clone(&store);
        let barrier = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            barrier.wait();
            store
                .publish(b"same concurrent bytes", "application/octet-stream")
                .unwrap()
        }));
    }
    barrier.wait();
    let artifacts: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();

    assert!(artifacts
        .windows(2)
        .all(|pair| pair[0].sha256 == pair[1].sha256));
    assert!(all_names(root.path())
        .iter()
        .all(|name| !name.starts_with(".tmp-")));
}

#[test]
fn independent_store_handles_converge_on_same_object() {
    let root = tempfile::tempdir().unwrap();
    let barrier = Arc::new(Barrier::new(9));
    let mut threads = Vec::new();

    for _ in 0..8 {
        let root = root.path().to_path_buf();
        let barrier = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            let store = ArtifactStore::open(root).unwrap();
            barrier.wait();
            store
                .publish(b"independent handles", "application/octet-stream")
                .unwrap()
        }));
    }
    barrier.wait();
    let artifacts: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();

    assert!(artifacts.windows(2).all(|pair| pair[0] == pair[1]));
    assert!(all_names(root.path())
        .iter()
        .all(|name| !name.starts_with(".tmp-")));
}

#[test]
fn checkpoint_selects_newest_immutable_version() {
    let (_root, store) = store();
    let key = key("checkpoint-task");
    let older = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "started",
        1_700_000_000,
    )
    .unwrap();
    let mut newer = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "complete",
        1_700_000_001,
    )
    .unwrap();
    newer.completed_units.push("unit-1".to_owned());

    assert_eq!(store.read_checkpoint(&key).unwrap(), None);
    store.write_checkpoint(&older).unwrap();
    assert_eq!(store.read_checkpoint(&key).unwrap(), Some(older.clone()));
    store.write_checkpoint(&newer).unwrap();
    assert_eq!(store.read_checkpoint(&key).unwrap(), Some(newer.clone()));

    let names: Vec<_> = fs::read_dir(checkpoints_path(&store, &key))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&checkpoint_name(
        older.updated_at_unix_seconds,
        &checkpoint_bytes(&older)
    )));
    assert!(names.contains(&checkpoint_name(
        newer.updated_at_unix_seconds,
        &checkpoint_bytes(&newer)
    )));
}

#[test]
fn writing_older_checkpoint_later_does_not_replace_newest() {
    let (_root, store) = store();
    let key = key("checkpoint-order");
    let newer = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "newer",
        10,
    )
    .unwrap();
    let older = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "older",
        -10,
    )
    .unwrap();

    store.write_checkpoint(&newer).unwrap();
    store.write_checkpoint(&older).unwrap();

    assert_eq!(store.read_checkpoint(&key).unwrap(), Some(newer));
}

#[test]
fn equal_timestamp_checkpoint_selection_uses_digest_tiebreak() {
    let (_root, store) = store();
    let key = key("checkpoint-tie");
    let first = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "alpha",
        42,
    )
    .unwrap();
    let second = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "omega",
        42,
    )
    .unwrap();
    let first_digest = format!("{:x}", Sha256::digest(checkpoint_bytes(&first)));
    let second_digest = format!("{:x}", Sha256::digest(checkpoint_bytes(&second)));
    let expected = if first_digest > second_digest {
        first.clone()
    } else {
        second.clone()
    };

    store.write_checkpoint(&first).unwrap();
    store.write_checkpoint(&second).unwrap();

    assert_eq!(store.read_checkpoint(&key).unwrap(), Some(expected));
}

#[test]
fn checkpoint_unknown_fields_and_identity_mismatches_are_rejected() {
    let (_root, store) = store();
    let expected_key = key("expected-task");
    let other_key = key("other-task");
    let checkpoint = Checkpoint::new(
        other_key.run_id.clone(),
        other_key.task_id.clone(),
        other_key.attempt_id.clone(),
        "running",
        42,
    )
    .unwrap();
    let checkpoint_dir = checkpoints_path(&store, &expected_key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let bytes = checkpoint_bytes(&checkpoint);
    fs::write(
        checkpoint_dir.join(checkpoint_name(checkpoint.updated_at_unix_seconds, &bytes)),
        &bytes,
    )
    .unwrap();

    assert!(matches!(
        store.read_checkpoint(&expected_key).unwrap_err(),
        ArtifactError::IdentityMismatch { .. }
    ));

    let mut unknown = serde_json::to_value(
        Checkpoint::new(
            expected_key.run_id.clone(),
            expected_key.task_id.clone(),
            expected_key.attempt_id.clone(),
            "running",
            43,
        )
        .unwrap(),
    )
    .unwrap();
    unknown["unexpected"] = json!(true);
    let mut unknown_bytes = serde_json::to_vec(&unknown).unwrap();
    unknown_bytes.push(b'\n');
    fs::remove_dir_all(&checkpoint_dir).unwrap();
    fs::create_dir(&checkpoint_dir).unwrap();
    fs::write(
        checkpoint_dir.join(checkpoint_name(43, &unknown_bytes)),
        unknown_bytes,
    )
    .unwrap();
    assert!(matches!(
        store.read_checkpoint(&expected_key).unwrap_err(),
        ArtifactError::Serialization { .. }
    ));
}

#[test]
fn malformed_checkpoint_json_is_rejected() {
    let (_root, store) = store();
    let key = key("malformed-checkpoint");
    let checkpoint_dir = checkpoints_path(&store, &key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let bytes = b"{\"phase\":\n";
    fs::write(checkpoint_dir.join(checkpoint_name(1, bytes)), bytes).unwrap();

    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::Serialization { .. }
    ));
}

#[test]
fn unrelated_checkpoint_directory_entry_is_rejected_fail_closed() {
    let (_root, store) = store();
    let key = key("unrelated-checkpoint-entry");
    let checkpoint_dir = checkpoints_path(&store, &key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    fs::write(checkpoint_dir.join("README"), b"not a checkpoint").unwrap();

    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::Conflict { .. }
    ));
}

#[test]
fn checkpoint_wrong_entry_type_is_rejected() {
    let (_root, store) = store();
    let key = key("checkpoint-directory");
    let checkpoint_dir = checkpoints_path(&store, &key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    fs::create_dir(checkpoint_dir.join(format!("{:016x}-{}.json", 1_u64 << 63, "a".repeat(64))))
        .unwrap();

    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::SymlinkOrWrongType { .. }
    ));
}

#[cfg(unix)]
#[test]
fn checkpoint_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let (root, store) = store();
    let key = key("checkpoint-symlink");
    let checkpoint_dir = checkpoints_path(&store, &key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let target = root.path().join("outside-checkpoint");
    fs::write(&target, b"{}").unwrap();
    symlink(
        &target,
        checkpoint_dir.join(format!("{:016x}-{}.json", 1_u64 << 63, "b".repeat(64))),
    )
    .unwrap();

    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::SymlinkOrWrongType { .. }
    ));
}

#[test]
fn corrupted_checkpoint_version_is_an_integrity_error() {
    let (_root, store) = store();
    let key = key("checkpoint-corruption");
    let checkpoint = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "running",
        44,
    )
    .unwrap();

    store.write_checkpoint(&checkpoint).unwrap();
    let bytes = checkpoint_bytes(&checkpoint);
    let path = checkpoints_path(&store, &key).join(checkpoint_name(44, &bytes));
    fs::write(path, b"corrupt\n").unwrap();

    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::DigestMismatch { .. }
    ));
}

#[test]
fn concurrent_independent_checkpoint_publishers_converge() {
    let root = tempfile::tempdir().unwrap();
    let key = key("checkpoint-convergence");
    let checkpoint = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "stable",
        55,
    )
    .unwrap();
    let barrier = Arc::new(Barrier::new(9));
    let mut threads = Vec::new();

    for _ in 0..8 {
        let root = root.path().to_path_buf();
        let key = key.clone();
        let checkpoint = checkpoint.clone();
        let barrier = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            let store = ArtifactStore::open(root).unwrap();
            barrier.wait();
            store.write_checkpoint(&checkpoint).unwrap();
            store.read_checkpoint(&key).unwrap()
        }));
    }
    barrier.wait();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), Some(checkpoint.clone()));
    }
    assert_eq!(
        fs::read_dir(checkpoints_path(
            &ArtifactStore::open(root.path()).unwrap(),
            &key
        ))
        .unwrap()
        .count(),
        1
    );
}

#[test]
fn checkpoint_version_count_accepts_exact_limit_and_rejects_one_more() {
    let (_root, store) = store();
    let key = key("checkpoint-version-limit");
    let checkpoint_dir = checkpoints_path(&store, &key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let mut expected = None;
    for timestamp in 0..MAX_CHECKPOINT_VERSIONS as i64 {
        expected = Some(write_checkpoint_version(&checkpoint_dir, &key, timestamp));
    }

    assert_eq!(store.read_checkpoint(&key).unwrap(), expected);

    write_checkpoint_version(&checkpoint_dir, &key, MAX_CHECKPOINT_VERSIONS as i64);
    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::TooManyEntries {
            max_entries: MAX_CHECKPOINT_VERSIONS,
            ..
        }
    ));
}

#[test]
fn checkpoint_total_bytes_accepts_exact_limit_and_rejects_overflow() {
    const VERSION_BYTES: usize = 1024 * 1024;
    const EXACT_VERSIONS: usize = MAX_OBJECT_BYTES / VERSION_BYTES;

    let (_root, store) = store();
    let key = key("checkpoint-total-bytes");
    let checkpoint_dir = checkpoints_path(&store, &key);
    fs::create_dir_all(&checkpoint_dir).unwrap();
    let mut expected = None;
    for timestamp in 0..EXACT_VERSIONS as i64 {
        let checkpoint = Checkpoint::new(
            key.run_id.clone(),
            key.task_id.clone(),
            key.attempt_id.clone(),
            format!("total-{timestamp}"),
            timestamp,
        )
        .unwrap();
        let bytes = padded_checkpoint_bytes(&checkpoint, VERSION_BYTES);
        fs::write(
            checkpoint_dir.join(checkpoint_name(timestamp, &bytes)),
            bytes,
        )
        .unwrap();
        expected = Some(checkpoint);
    }

    assert_eq!(store.read_checkpoint(&key).unwrap(), expected);

    let overflow = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "overflow",
        EXACT_VERSIONS as i64,
    )
    .unwrap();
    let bytes = padded_checkpoint_bytes(&overflow, VERSION_BYTES);
    fs::write(
        checkpoint_dir.join(checkpoint_name(EXACT_VERSIONS as i64, &bytes)),
        bytes,
    )
    .unwrap();
    assert!(matches!(
        store.read_checkpoint(&key).unwrap_err(),
        ArtifactError::CheckpointScanTooLarge {
            max_bytes: MAX_OBJECT_BYTES
        }
    ));
}

#[test]
fn evidence_append_and_bounded_read_preserve_complete_records() {
    let (_root, store) = store();
    let key = key("evidence-task");

    store
        .append_evidence(&key, &json!({"sequence": 1, "message": "first"}))
        .unwrap();
    store
        .append_evidence(&key, &json!({"sequence": 2, "message": "second"}))
        .unwrap();

    assert_eq!(
        store.read_evidence(&key, 1024).unwrap(),
        vec![
            json!({"sequence": 1, "message": "first"}),
            json!({"sequence": 2, "message": "second"})
        ]
    );
    assert!(matches!(
        store.read_evidence(&key, 1).unwrap_err(),
        ArtifactError::SizeMismatch { .. }
    ));
    assert!(matches!(
        store.read_evidence(&key, 0).unwrap_err(),
        ArtifactError::SizeMismatch { .. }
    ));
    assert!(matches!(
        store.read_evidence(&key, MAX_OBJECT_BYTES + 1).unwrap_err(),
        ArtifactError::SizeMismatch { .. }
    ));
}

struct PanickingRecord;

impl Serialize for PanickingRecord {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        panic!("user serializer panic");
    }
}

#[test]
fn panicking_evidence_serializer_does_not_poison_global_append_lock() {
    let first_root = tempfile::tempdir().unwrap();
    let first_store = ArtifactStore::open(first_root.path()).unwrap();
    let first_key = key("panic-evidence");

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = first_store.append_evidence(&first_key, &PanickingRecord);
    }));
    assert!(panic.is_err());

    let second_root = tempfile::tempdir().unwrap();
    let second_store = ArtifactStore::open(second_root.path()).unwrap();
    let second_key = key("post-panic-evidence");
    second_store
        .append_evidence(&second_key, &json!({"after": "panic"}))
        .unwrap();
    assert_eq!(
        second_store.read_evidence(&second_key, 1024).unwrap(),
        vec![json!({"after": "panic"})]
    );
}

#[test]
fn append_rejects_partial_evidence_tail_until_operator_recovers_file() {
    let (_root, store) = store();
    let key = key("partial-evidence-tail");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    let evidence = attempt_dir.join("evidence.jsonl");
    let partial = b"{\"partial\":";
    fs::write(&evidence, partial).unwrap();

    assert!(matches!(
        store
            .append_evidence(&key, &json!({"must_not": "append"}))
            .unwrap_err(),
        ArtifactError::EvidenceRecoveryRequired { .. }
    ));
    assert_eq!(fs::read(&evidence).unwrap(), partial);

    fs::write(&evidence, b"{\"recovered\":true}\n").unwrap();
    store
        .append_evidence(&key, &json!({"after": "recovery"}))
        .unwrap();
    assert_eq!(
        store.read_evidence(&key, 1024).unwrap(),
        vec![json!({"recovered": true}), json!({"after": "recovery"})]
    );
}

#[test]
fn independent_stores_serialize_concurrent_evidence_appends() {
    let root = tempfile::tempdir().unwrap();
    let key = key("evidence-concurrency");
    let barrier = Arc::new(Barrier::new(9));
    let mut threads = Vec::new();

    for writer in 0..8 {
        let root = root.path().to_path_buf();
        let key = key.clone();
        let barrier = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            let store = ArtifactStore::open(root).unwrap();
            barrier.wait();
            for sequence in 0..50 {
                store
                    .append_evidence(&key, &json!({"writer": writer, "sequence": sequence}))
                    .unwrap();
            }
        }));
    }
    barrier.wait();
    for thread in threads {
        thread.join().unwrap();
    }

    let store = ArtifactStore::open(root.path()).unwrap();
    let records = store.read_evidence(&key, MAX_OBJECT_BYTES).unwrap();
    assert_eq!(records.len(), 400);
}

#[test]
fn concurrent_near_limit_evidence_appends_never_exceed_file_cap() {
    const LINE_BYTES: usize = 1024 * 1024;
    let root = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(root.path()).unwrap();
    let key = key("evidence-near-limit");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    let evidence_path = attempt_dir.join("evidence.jsonl");
    let prefix = b"{\"padding\":\"";
    let suffix = b"\"}\n";
    let padding = vec![b'x'; LINE_BYTES - prefix.len() - suffix.len()];
    let mut file = fs::File::create(&evidence_path).unwrap();
    for _ in 0..63 {
        file.write_all(prefix).unwrap();
        file.write_all(&padding).unwrap();
        file.write_all(suffix).unwrap();
    }
    file.sync_all().unwrap();

    let payload = "y".repeat(LINE_BYTES / 2 - 3);
    let barrier = Arc::new(Barrier::new(5));
    let mut threads = Vec::new();
    for _ in 0..4 {
        let root = root.path().to_path_buf();
        let key = key.clone();
        let payload = payload.clone();
        let barrier = Arc::clone(&barrier);
        threads.push(thread::spawn(move || {
            let store = ArtifactStore::open(root).unwrap();
            barrier.wait();
            store.append_evidence(&key, &payload)
        }));
    }
    barrier.wait();
    let results: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 2);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(ArtifactError::SizeMismatch { .. })))
            .count(),
        2
    );
    assert_eq!(
        fs::metadata(evidence_path).unwrap().len(),
        MAX_OBJECT_BYTES as u64
    );
}

#[test]
fn evidence_serialization_accepts_exact_limit_and_rejects_one_more() {
    let (_root, store) = store();
    let key = key("evidence-line-boundary");
    let exact = "x".repeat(MAX_OBJECT_BYTES / 64 - 3);
    let too_large = "x".repeat(MAX_OBJECT_BYTES / 64 - 2);

    store.append_evidence(&key, &exact).unwrap();
    assert!(matches!(
        store.append_evidence(&key, &too_large).unwrap_err(),
        ArtifactError::EvidenceTooLarge { .. }
    ));
    assert_eq!(
        store.read_evidence(&key, 2 * 1024 * 1024).unwrap(),
        vec![json!(exact)]
    );
}

struct StreamingOversizedRecord;

impl Serialize for StreamingOversizedRecord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut sequence = serializer.serialize_seq(None)?;
        for _ in 0..600_000 {
            sequence.serialize_element(&0_u8)?;
        }
        sequence.end()
    }
}

#[test]
fn evidence_streaming_serializer_stops_at_cap() {
    let (root, store) = store();
    let key = key("streaming-evidence-limit");

    assert!(matches!(
        store
            .append_evidence(&key, &StreamingOversizedRecord)
            .unwrap_err(),
        ArtifactError::EvidenceTooLarge { .. }
    ));
    assert!(!store
        .attempt_dir(&key)
        .unwrap()
        .join("evidence.jsonl")
        .exists());
    assert!(all_names(root.path())
        .iter()
        .all(|name| !name.starts_with(".tmp-")));
}

#[test]
fn empty_evidence_file_is_valid() {
    let (_root, store) = store();
    let key = key("empty-evidence");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    fs::write(attempt_dir.join("evidence.jsonl"), b"").unwrap();

    assert_eq!(
        store.read_evidence(&key, 1024).unwrap(),
        Vec::<serde_json::Value>::new()
    );
}

#[test]
fn blank_evidence_line_is_rejected() {
    let (_root, store) = store();
    let key = key("blank-evidence");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    fs::write(attempt_dir.join("evidence.jsonl"), b"{\"ok\":true}\n\n").unwrap();

    assert!(matches!(
        store.read_evidence(&key, 1024).unwrap_err(),
        ArtifactError::Serialization { .. }
    ));
}

#[test]
fn evidence_without_terminal_newline_is_rejected() {
    let (_root, store) = store();
    let key = key("unterminated-evidence");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    fs::write(attempt_dir.join("evidence.jsonl"), b"{\"ok\":true}").unwrap();

    assert!(matches!(
        store.read_evidence(&key, 1024).unwrap_err(),
        ArtifactError::Serialization { .. }
    ));
}

#[test]
fn evidence_file_accepts_exact_total_limit_and_rejects_one_more() {
    const LINE_BYTES: usize = 1024 * 1024;
    const LINE_COUNT: usize = MAX_OBJECT_BYTES / LINE_BYTES;

    let (_root, store) = store();
    let key = key("evidence-file-boundary");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    let evidence_path = attempt_dir.join("evidence.jsonl");
    let prefix = b"{\"padding\":\"";
    let suffix = b"\"}\n";
    let padding = vec![b'x'; LINE_BYTES - prefix.len() - suffix.len()];
    let mut file = fs::File::create(&evidence_path).unwrap();
    for _ in 0..LINE_COUNT {
        file.write_all(prefix).unwrap();
        file.write_all(&padding).unwrap();
        file.write_all(suffix).unwrap();
    }
    file.sync_all().unwrap();

    let records = store.read_evidence(&key, MAX_OBJECT_BYTES).unwrap();
    assert_eq!(records.len(), LINE_COUNT);

    drop(records);
    let file = fs::OpenOptions::new()
        .write(true)
        .open(&evidence_path)
        .unwrap();
    file.set_len((MAX_OBJECT_BYTES + 1) as u64).unwrap();
    assert!(matches!(
        store.read_evidence(&key, MAX_OBJECT_BYTES).unwrap_err(),
        ArtifactError::SizeMismatch { .. }
    ));
}

#[test]
fn evidence_record_count_accepts_exact_limit_and_rejects_one_more() {
    let (_root, store) = store();
    let key = key("evidence-record-limit");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    let evidence = attempt_dir.join("evidence.jsonl");
    fs::write(&evidence, b"0\n".repeat(MAX_EVIDENCE_RECORDS)).unwrap();

    assert_eq!(
        store
            .read_evidence(&key, MAX_EVIDENCE_RECORDS * 2)
            .unwrap()
            .len(),
        MAX_EVIDENCE_RECORDS
    );

    let mut file = fs::OpenOptions::new().append(true).open(&evidence).unwrap();
    file.write_all(b"0\n").unwrap();
    assert!(matches!(
        store
            .read_evidence(&key, MAX_EVIDENCE_RECORDS * 2 + 2)
            .unwrap_err(),
        ArtifactError::EvidenceTooManyRecords {
            max_records: MAX_EVIDENCE_RECORDS
        }
    ));
}

#[test]
fn malformed_evidence_line_is_rejected() {
    let (_root, store) = store();
    let key = key("malformed-evidence");
    store
        .append_evidence(&key, &json!({"valid": true}))
        .unwrap();
    let path = store.attempt_dir(&key).unwrap().join("evidence.jsonl");
    fs::write(path, b"{\"valid\":true}\n{\"broken\":\n").unwrap();

    assert!(matches!(
        store.read_evidence(&key, 1024).unwrap_err(),
        ArtifactError::Serialization { .. }
    ));
}

#[cfg(unix)]
#[test]
fn evidence_symlink_is_rejected_for_append_and_read() {
    use std::os::unix::fs::symlink;

    let (root, store) = store();
    let key = key("evidence-symlink");
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::create_dir_all(&attempt_dir).unwrap();
    let target = root.path().join("outside-evidence");
    fs::write(&target, b"").unwrap();
    symlink(&target, attempt_dir.join("evidence.jsonl")).unwrap();

    assert!(matches!(
        store
            .append_evidence(&key, &json!({"never": "written"}))
            .unwrap_err(),
        ArtifactError::SymlinkOrWrongType { .. }
    ));
    assert!(matches!(
        store.read_evidence(&key, 1024).unwrap_err(),
        ArtifactError::SymlinkOrWrongType { .. }
    ));
    assert_eq!(fs::read(target).unwrap(), b"");
}

#[derive(Debug)]
struct FailingRecord;

impl Serialize for FailingRecord {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("injected serialization failure"))
    }
}

#[test]
fn evidence_serialization_failure_leaves_no_temp_or_partial_file() {
    let (root, store) = store();
    let key = key("serialize-failure");
    let attempt_dir = store.attempt_dir(&key).unwrap();

    assert!(matches!(
        store.append_evidence(&key, &FailingRecord).unwrap_err(),
        ArtifactError::Serialization { .. }
    ));
    assert!(!attempt_dir.join("evidence.jsonl").exists());
    assert!(all_names(root.path())
        .iter()
        .all(|name| !name.starts_with(".tmp-")));
}

#[test]
fn attempt_dir_contains_only_derived_identity_components() {
    let (root, store) = store();
    let key = key("safe.task-id_1");

    assert_eq!(
        store.attempt_dir(&key).unwrap(),
        root.path()
            .join("runs")
            .join(key.run_id.to_string())
            .join("tasks")
            .join(key.task_id.to_string())
            .join("attempts")
            .join(key.attempt_id.to_string())
    );
}

#[cfg(unix)]
#[test]
fn attempt_storage_usage_counts_all_host_owned_categories_and_rejects_symlinks() {
    use std::os::unix::fs::symlink;

    let (root, store) = store();
    let key = key("usage");
    let checkpoint = Checkpoint::new(
        key.run_id.clone(),
        key.task_id.clone(),
        key.attempt_id.clone(),
        "usage",
        1,
    )
    .unwrap();
    let checkpoint_size = checkpoint_bytes(&checkpoint).len() as u128;
    store.write_checkpoint(&checkpoint).unwrap();
    store
        .append_evidence(&key, &json!({"claim": "evidence"}))
        .unwrap();
    let attempt_dir = store.attempt_dir(&key).unwrap();
    fs::write(attempt_dir.join("notes.md"), b"notes").unwrap();
    fs::write(attempt_dir.join("result.json"), b"result").unwrap();
    fs::write(attempt_dir.join("tmp.bin"), b"unknown").unwrap();
    fs::create_dir(attempt_dir.join("nested")).unwrap();
    fs::write(attempt_dir.join("nested/data.bin"), b"nested").unwrap();

    let usage = store
        .attempt_storage_usage(&key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
        .unwrap();

    assert_eq!(usage.checkpoint_bytes, checkpoint_size);
    assert_eq!(
        usage.evidence_bytes,
        u128::from(
            fs::metadata(attempt_dir.join("evidence.jsonl"))
                .unwrap()
                .len(),
        )
    );
    assert_eq!(usage.notes_bytes, 5);
    assert_eq!(usage.result_scratch_bytes, 6);
    assert_eq!(usage.other_bytes, 13);
    assert_eq!(
        usage.total_bytes().unwrap(),
        usage.checkpoint_bytes + usage.evidence_bytes + 24
    );

    let outside = root.path().join("outside-notes");
    fs::write(&outside, b"outside").unwrap();
    fs::remove_file(attempt_dir.join("notes.md")).unwrap();
    symlink(&outside, attempt_dir.join("notes.md")).unwrap();
    assert!(matches!(
        store
            .attempt_storage_usage(&key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
            .unwrap_err(),
        ArtifactError::StoragePolicyViolation { .. }
    ));
}

#[cfg(unix)]
#[test]
fn attempt_storage_usage_rejects_hard_links_and_special_files() {
    use std::os::unix::fs::symlink;
    use std::process::Command;

    let (root, store) = store();
    let key = key("usage-types");
    let attempt = store.resolve_attempt_scratch(&key).unwrap();
    let attempt_dir = attempt.canonical_path();
    fs::write(attempt_dir.join("source.bin"), b"source").unwrap();
    fs::hard_link(
        attempt_dir.join("source.bin"),
        attempt_dir.join("linked.bin"),
    )
    .unwrap();
    assert!(matches!(
        store
            .attempt_storage_usage(&key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
            .unwrap_err(),
        ArtifactError::StoragePolicyViolation { .. }
    ));

    fs::remove_file(attempt_dir.join("linked.bin")).unwrap();
    fs::remove_file(attempt_dir.join("source.bin")).unwrap();
    let outside = root.path().join("outside");
    fs::write(&outside, b"outside").unwrap();
    symlink(&outside, attempt_dir.join("link.bin")).unwrap();
    assert!(matches!(
        store
            .attempt_storage_usage(&key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
            .unwrap_err(),
        ArtifactError::StoragePolicyViolation { .. }
    ));

    fs::remove_file(attempt_dir.join("link.bin")).unwrap();
    assert!(Command::new("mkfifo")
        .arg(attempt_dir.join("pipe"))
        .status()
        .unwrap()
        .success());
    assert!(matches!(
        store
            .attempt_storage_usage(&key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
            .unwrap_err(),
        ArtifactError::StoragePolicyViolation { .. }
    ));
}

#[test]
fn attempt_storage_usage_rejects_depth_and_entry_limit() {
    let (_root, store) = store();
    let depth_key = key("usage-depth");
    let mut path = store
        .resolve_attempt_scratch(&depth_key)
        .unwrap()
        .canonical_path()
        .to_path_buf();
    for index in 0..=16 {
        path = path.join(format!("d{index}"));
        fs::create_dir(&path).unwrap();
    }
    assert!(matches!(
        store
            .attempt_storage_usage(&depth_key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
            .unwrap_err(),
        ArtifactError::StoragePolicyViolation { .. }
    ));

    let entries_key = key("usage-entries");
    let path = store
        .resolve_attempt_scratch(&entries_key)
        .unwrap()
        .canonical_path()
        .to_path_buf();
    for index in 0..=10_000 {
        fs::write(path.join(format!("f{index:05}")), []).unwrap();
    }
    assert!(matches!(
        store
            .attempt_storage_usage(&entries_key, harp_artifacts::MAX_ATTEMPT_SCAN_BYTES)
            .unwrap_err(),
        ArtifactError::StoragePolicyViolation { .. }
    ));
}

#[test]
fn attempt_storage_usage_supports_configured_limit_above_sixty_four_mib() {
    let (_root, store) = store();
    let key = key("usage-large-legal");
    let attempt = store.resolve_attempt_scratch(&key).unwrap();
    let path = attempt.canonical_path().join("large-sparse.bin");
    let file = fs::File::create(path).unwrap();
    file.set_len(65 * 1024 * 1024).unwrap();

    let usage = store
        .attempt_storage_usage(&key, 66 * 1024 * 1024)
        .expect("configured legal large logical file");

    assert_eq!(usage.other_bytes, 65 * 1024 * 1024);
}

#[test]
fn storage_limit_arithmetic_detects_one_byte_above_sqlite_max() {
    let limit = i64::MAX as u64;
    let exact = AttemptStorageUsage {
        other_bytes: u128::from(limit),
        ..AttemptStorageUsage::default()
    };
    let over = AttemptStorageUsage {
        other_bytes: u128::from(limit) + 1,
        ..AttemptStorageUsage::default()
    };

    assert_eq!(
        exact.limit_observation(limit),
        harp_artifacts::StorageLimitObservation::Within {
            logical_bytes: u128::from(limit),
            persisted_bytes: limit,
        }
    );
    assert_eq!(
        over.limit_observation(limit),
        harp_artifacts::StorageLimitObservation::Exceeded {
            logical_bytes: u128::from(limit) + 1,
            persisted_bytes: limit,
        }
    );
}

#[cfg(unix)]
#[test]
fn root_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let parent = tempfile::tempdir().unwrap();
    let real_root = parent.path().join("real");
    let linked_root = parent.path().join("linked");
    fs::create_dir(&real_root).unwrap();
    symlink(&real_root, &linked_root).unwrap();

    assert!(matches!(
        ArtifactStore::open(&linked_root).unwrap_err(),
        ArtifactError::InvalidRoot { .. }
    ));
}
