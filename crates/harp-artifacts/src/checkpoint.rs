use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

use harp_contracts::Checkpoint;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::platform::SecureError;
#[cfg(test)]
use crate::store::injected_error;
use crate::store::{
    ArtifactStore, AttemptKey, ImmutablePurpose, CHECKPOINT_DIR, EVIDENCE_FILE,
    MAX_EVIDENCE_LINE_BYTES, MAX_OBJECT_BYTES,
};
#[cfg(test)]
use crate::test_support::{FaultPoint, SubstitutionPurpose};
use crate::{ArtifactError, ArtifactResult};

const MAX_CHECKPOINT_BYTES: usize = 1024 * 1024;
const MAX_CHECKPOINT_VERSIONS: usize = 4096;
const MAX_CHECKPOINT_TOTAL_BYTES: usize = 64 * 1024 * 1024;
const MAX_EVIDENCE_RECORDS: usize = 65_536;
const CHECKPOINT_NAME_BYTES: usize = 16 + 1 + 64 + 5;
static EVIDENCE_APPEND_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

impl ArtifactStore {
    pub fn write_checkpoint(&self, checkpoint: &Checkpoint) -> ArtifactResult<()> {
        checkpoint
            .validate()
            .map_err(|source| ArtifactError::InvalidCheckpoint { source })?;
        let key = AttemptKey {
            run_id: checkpoint.run_id.clone(),
            task_id: checkpoint.task_id.clone(),
            attempt_id: checkpoint.attempt_id.clone(),
        };
        let mut bytes =
            serde_json::to_vec(checkpoint).map_err(|source| ArtifactError::Serialization {
                context: "checkpoint".to_owned(),
                source,
            })?;
        bytes.push(b'\n');
        if bytes.len() > MAX_CHECKPOINT_BYTES {
            return Err(ArtifactError::SizeMismatch {
                context: "checkpoint".to_owned(),
                expected: MAX_CHECKPOINT_BYTES as u64,
                actual: bytes.len() as u64,
            });
        }
        let digest = hex_digest(&bytes);
        let name = checkpoint_file_name(checkpoint.updated_at_unix_seconds, &digest);
        let attempt =
            self.attempt_directory(&key, true)?
                .ok_or_else(|| ArtifactError::Conflict {
                    context: "checkpoint attempt directory".to_owned(),
                    message: "attempt directory was not created".to_owned(),
                    source: None,
                })?;
        let (checkpoints, _) = attempt.ensure_dir(CHECKPOINT_DIR).map_err(|error| {
            ArtifactError::from_secure(
                "ensure checkpoint versions directory",
                CHECKPOINT_DIR,
                error,
            )
        })?;
        self.publish_immutable_bytes(
            &attempt,
            &checkpoints,
            &name,
            &bytes,
            MAX_CHECKPOINT_BYTES,
            ImmutablePurpose::Checkpoint,
        )
    }

    pub fn read_checkpoint(&self, key: &AttemptKey) -> ArtifactResult<Option<Checkpoint>> {
        Ok(self
            .read_checkpoint_version(key)?
            .map(|version| version.checkpoint))
    }

    pub fn checkpoint_digest(&self, key: &AttemptKey) -> ArtifactResult<Option<String>> {
        Ok(self
            .read_checkpoint_version(key)?
            .map(|version| version.sha256))
    }

    pub fn read_checkpoint_version(
        &self,
        key: &AttemptKey,
    ) -> ArtifactResult<Option<CheckpointVersion>> {
        let Some(attempt) = self.attempt_directory(key, false)? else {
            return Ok(None);
        };
        let checkpoints = match attempt.open_dir(CHECKPOINT_DIR) {
            Ok(directory) => directory,
            Err(SecureError::NotFound) => return Ok(None),
            Err(error) => {
                return Err(ArtifactError::from_secure(
                    "open checkpoint versions directory",
                    CHECKPOINT_DIR,
                    error,
                ));
            }
        };
        let names = checkpoints
            .read_entry_names(MAX_CHECKPOINT_VERSIONS)
            .map_err(|error| {
                ArtifactError::from_secure("enumerate checkpoint versions", CHECKPOINT_DIR, error)
            })?;
        let mut latest: Option<(i64, String, Checkpoint)> = None;
        let mut total_bytes = 0_u64;
        for name in names {
            let (timestamp, expected_digest) = parse_checkpoint_name(&name)?;
            let file = checkpoints
                .open_regular_optional(&name)
                .map_err(|error| {
                    ArtifactError::from_secure("open checkpoint version", &name, error)
                })?
                .ok_or_else(|| ArtifactError::Conflict {
                    context: name.clone(),
                    message: "checkpoint version disappeared during enumeration".to_owned(),
                    source: None,
                })?;
            total_bytes = total_bytes.checked_add(file.size()).ok_or(
                ArtifactError::CheckpointScanTooLarge {
                    max_bytes: MAX_CHECKPOINT_TOTAL_BYTES,
                },
            )?;
            if total_bytes > MAX_CHECKPOINT_TOTAL_BYTES as u64 {
                return Err(ArtifactError::CheckpointScanTooLarge {
                    max_bytes: MAX_CHECKPOINT_TOTAL_BYTES,
                });
            }
            if file.size() > MAX_CHECKPOINT_BYTES as u64 {
                return Err(ArtifactError::SizeMismatch {
                    context: name,
                    expected: MAX_CHECKPOINT_BYTES as u64,
                    actual: file.size(),
                });
            }
            let bytes =
                file.read_bounded(MAX_CHECKPOINT_BYTES)
                    .map_err(|source| ArtifactError::Io {
                        operation: "read checkpoint version",
                        context: name.clone(),
                        source,
                    })?;
            let actual_digest = hex_digest(&bytes);
            if actual_digest != expected_digest {
                return Err(ArtifactError::DigestMismatch {
                    context: name,
                    expected: expected_digest,
                    actual: actual_digest,
                });
            }
            let checkpoint: Checkpoint =
                serde_json::from_slice(&bytes).map_err(|source| ArtifactError::Serialization {
                    context: name.clone(),
                    source,
                })?;
            checkpoint
                .validate()
                .map_err(|source| ArtifactError::InvalidCheckpoint { source })?;
            verify_checkpoint_identity(key, &checkpoint)?;
            if checkpoint.updated_at_unix_seconds != timestamp {
                return Err(ArtifactError::IdentityMismatch {
                    context: name,
                    expected: timestamp.to_string(),
                    actual: checkpoint.updated_at_unix_seconds.to_string(),
                });
            }
            let selection = (timestamp, actual_digest.clone());
            if latest
                .as_ref()
                .is_none_or(|(current_timestamp, current_digest, _)| {
                    selection > (*current_timestamp, current_digest.clone())
                })
            {
                latest = Some((timestamp, actual_digest, checkpoint));
            }
        }
        Ok(latest.map(|(_, sha256, checkpoint)| CheckpointVersion { checkpoint, sha256 }))
    }

    pub fn append_evidence<T: Serialize>(
        &self,
        key: &AttemptKey,
        record: &T,
    ) -> ArtifactResult<()> {
        let mut writer = CappedJsonWriter::new(MAX_EVIDENCE_LINE_BYTES - 1);
        let result = serde_json::to_writer(&mut writer, record);
        if writer.exceeded {
            return Err(ArtifactError::EvidenceTooLarge {
                max_bytes: MAX_EVIDENCE_LINE_BYTES,
            });
        }
        result.map_err(|source| ArtifactError::Serialization {
            context: "evidence record".to_owned(),
            source,
        })?;
        let mut line = writer.bytes;
        line.push(b'\n');
        let _guard = EVIDENCE_APPEND_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|_| ArtifactError::Conflict {
                context: "evidence writer".to_owned(),
                message: "process-wide evidence lock was poisoned".to_owned(),
                source: None,
            })?;
        let attempt =
            self.attempt_directory(key, true)?
                .ok_or_else(|| ArtifactError::Conflict {
                    context: "evidence attempt directory".to_owned(),
                    message: "attempt directory was not created".to_owned(),
                    source: None,
                })?;
        let (file, created) = attempt
            .open_append_create(EVIDENCE_FILE)
            .map_err(|error| ArtifactError::from_secure("open evidence", EVIDENCE_FILE, error))?;
        let identity = file.identity();
        attempt
            .verify_entry_identity(EVIDENCE_FILE, identity)
            .map_err(|error| {
                ArtifactError::from_secure(
                    "verify evidence identity before append",
                    EVIDENCE_FILE,
                    error,
                )
            })?;
        let tail = file.last_byte().map_err(|source| ArtifactError::Io {
            operation: "read evidence tail",
            context: EVIDENCE_FILE.to_owned(),
            source,
        })?;
        if file.size() > 0 && tail != Some(b'\n') {
            return Err(ArtifactError::EvidenceRecoveryRequired {
                context: EVIDENCE_FILE.to_owned(),
                message:
                    "existing evidence log does not end with a newline; operator recovery required"
                        .to_owned(),
            });
        }
        let resulting_size = file.size().checked_add(line.len() as u64).ok_or_else(|| {
            ArtifactError::SizeMismatch {
                context: EVIDENCE_FILE.to_owned(),
                expected: MAX_OBJECT_BYTES as u64,
                actual: u64::MAX,
            }
        })?;
        if resulting_size > MAX_OBJECT_BYTES as u64 {
            return Err(ArtifactError::SizeMismatch {
                context: EVIDENCE_FILE.to_owned(),
                expected: MAX_OBJECT_BYTES as u64,
                actual: resulting_size,
            });
        }
        #[cfg(test)]
        if self.take_fault(FaultPoint::EvidencePartialWrite) {
            let partial = &line[..line.len().saturating_sub(1)];
            file.write_append_line(partial)
                .map_err(|source| ArtifactError::Io {
                    operation: "append partial evidence",
                    context: EVIDENCE_FILE.to_owned(),
                    source,
                })?;
            file.sync_data().map_err(|source| ArtifactError::Io {
                operation: "sync partial evidence",
                context: EVIDENCE_FILE.to_owned(),
                source,
            })?;
            return Err(ArtifactError::Io {
                operation: "append evidence",
                context: EVIDENCE_FILE.to_owned(),
                source: injected_error(FaultPoint::EvidencePartialWrite),
            });
        }
        file.write_append_line(&line)
            .map_err(|source| ArtifactError::Io {
                operation: "append evidence",
                context: EVIDENCE_FILE.to_owned(),
                source,
            })?;
        file.sync_data().map_err(|source| ArtifactError::Io {
            operation: "sync evidence",
            context: EVIDENCE_FILE.to_owned(),
            source,
        })?;
        if created {
            #[cfg(test)]
            if self.take_fault(FaultPoint::EvidenceDirectorySync) {
                return Err(ArtifactError::Io {
                    operation: "sync evidence directory",
                    context: EVIDENCE_FILE.to_owned(),
                    source: injected_error(FaultPoint::EvidenceDirectorySync),
                });
            }
            attempt.sync().map_err(|error| {
                ArtifactError::from_secure("sync evidence directory", EVIDENCE_FILE, error)
            })?;
        }
        #[cfg(test)]
        if self.take_fault(FaultPoint::EvidenceIdentitySubstitution) {
            if let Some(target) = self
                .test_hooks
                .take_substitution(SubstitutionPurpose::Evidence)
            {
                attempt
                    .replace_entry_with_symlink(EVIDENCE_FILE, &target)
                    .map_err(|error| {
                        ArtifactError::from_secure(
                            "substitute evidence test hook",
                            EVIDENCE_FILE,
                            error,
                        )
                    })?;
            }
        }
        attempt
            .verify_entry_identity(EVIDENCE_FILE, identity)
            .map_err(|error| {
                ArtifactError::from_secure("verify evidence identity", EVIDENCE_FILE, error)
            })?;
        Ok(())
    }

    pub fn read_evidence(&self, key: &AttemptKey, max_bytes: usize) -> ArtifactResult<Vec<Value>> {
        if !(1..=MAX_OBJECT_BYTES).contains(&max_bytes) {
            return Err(ArtifactError::SizeMismatch {
                context: "evidence read bound".to_owned(),
                expected: MAX_OBJECT_BYTES as u64,
                actual: max_bytes as u64,
            });
        }
        let Some(attempt) = self.attempt_directory(key, false)? else {
            return Ok(Vec::new());
        };
        let Some(file) = attempt
            .open_regular_optional(EVIDENCE_FILE)
            .map_err(|error| ArtifactError::from_secure("open evidence", EVIDENCE_FILE, error))?
        else {
            return Ok(Vec::new());
        };
        if file.size() > MAX_OBJECT_BYTES as u64 || file.size() > max_bytes as u64 {
            return Err(ArtifactError::SizeMismatch {
                context: EVIDENCE_FILE.to_owned(),
                expected: max_bytes.min(MAX_OBJECT_BYTES) as u64,
                actual: file.size(),
            });
        }
        let bytes = file
            .read_bounded(max_bytes)
            .map_err(|source| ArtifactError::Io {
                operation: "read evidence",
                context: EVIDENCE_FILE.to_owned(),
                source,
            })?;
        if bytes.len() > max_bytes {
            return Err(ArtifactError::SizeMismatch {
                context: EVIDENCE_FILE.to_owned(),
                expected: max_bytes as u64,
                actual: bytes.len() as u64,
            });
        }
        if !bytes.is_empty() && !bytes.ends_with(b"\n") {
            return Err(ArtifactError::Serialization {
                context: "evidence file ends with an incomplete line".to_owned(),
                source: serde_json::from_slice::<Value>(b"").unwrap_err(),
            });
        }
        if bytes.is_empty() {
            return Ok(Vec::new());
        }
        let mut records = Vec::new();
        for (index, line) in bytes[..bytes.len() - 1]
            .split(|byte| *byte == b'\n')
            .enumerate()
        {
            if line.is_empty() {
                return Err(ArtifactError::Serialization {
                    context: format!("evidence line {} is empty", index + 1),
                    source: serde_json::from_slice::<Value>(b"").unwrap_err(),
                });
            }
            if records.len() == MAX_EVIDENCE_RECORDS {
                return Err(ArtifactError::EvidenceTooManyRecords {
                    max_records: MAX_EVIDENCE_RECORDS,
                });
            }
            if line.len() + 1 > MAX_EVIDENCE_LINE_BYTES {
                return Err(ArtifactError::EvidenceTooLarge {
                    max_bytes: MAX_EVIDENCE_LINE_BYTES,
                });
            }
            records.push(serde_json::from_slice(line).map_err(|source| {
                ArtifactError::Serialization {
                    context: format!("evidence line {}", index + 1),
                    source,
                }
            })?);
        }
        Ok(records)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointVersion {
    pub checkpoint: Checkpoint,
    pub sha256: String,
}

struct CappedJsonWriter {
    bytes: Vec<u8>,
    cap: usize,
    exceeded: bool,
}

impl CappedJsonWriter {
    fn new(cap: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(cap.min(8 * 1024)),
            cap,
            exceeded: false,
        }
    }
}

impl Write for CappedJsonWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.cap.saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(io::Error::other("encoded evidence exceeds line cap"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn checkpoint_file_name(timestamp: i64, digest: &str) -> String {
    let encoded_timestamp = (timestamp as u64) ^ (1_u64 << 63);
    format!("{encoded_timestamp:016x}-{digest}.json")
}

fn parse_checkpoint_name(name: &str) -> ArtifactResult<(i64, String)> {
    if name.len() != CHECKPOINT_NAME_BYTES
        || name.as_bytes()[16] != b'-'
        || !name.ends_with(".json")
    {
        return Err(invalid_checkpoint_entry(name));
    }
    let timestamp_hex = &name[..16];
    let digest = &name[17..81];
    if !timestamp_hex
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid_checkpoint_entry(name));
    }
    let encoded_timestamp =
        u64::from_str_radix(timestamp_hex, 16).map_err(|_| invalid_checkpoint_entry(name))?;
    Ok((
        (encoded_timestamp ^ (1_u64 << 63)) as i64,
        digest.to_owned(),
    ))
}

fn invalid_checkpoint_entry(name: &str) -> ArtifactError {
    ArtifactError::Conflict {
        context: name.to_owned(),
        message: "checkpoint directory contains an invalid entry name".to_owned(),
        source: None,
    }
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

fn verify_checkpoint_identity(key: &AttemptKey, checkpoint: &Checkpoint) -> ArtifactResult<()> {
    verify_identity(
        "checkpoint run ID",
        &key.run_id.to_string(),
        &checkpoint.run_id.to_string(),
    )?;
    verify_identity(
        "checkpoint task ID",
        &key.task_id.to_string(),
        &checkpoint.task_id.to_string(),
    )?;
    verify_identity(
        "checkpoint attempt ID",
        &key.attempt_id.to_string(),
        &checkpoint.attempt_id.to_string(),
    )
}

fn verify_identity(context: &str, expected: &str, actual: &str) -> ArtifactResult<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(ArtifactError::IdentityMismatch {
            context: context.to_owned(),
            expected: expected.to_owned(),
            actual: actual.to_owned(),
        })
    }
}
