//! Build a Session index from a dual-stream TraeCLI rollout.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::detect::{detect_dialect, DialectId};
use crate::dialect::{ingest_traecli_dual_stream, IndexState};
use crate::error::SessionIndexError;
use crate::spill::{spill_oversize_line, OversizeStub};
use crate::store::session_dir;
use crate::SessionIndexResult;

const DEFAULT_OVERSIZE: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct IndexOptions {
    pub oversize_line_bytes: usize,
    pub artifacts_dir: Option<PathBuf>,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            oversize_line_bytes: DEFAULT_OVERSIZE,
            artifacts_dir: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnCounts {
    pub started: u64,
    pub completed: u64,
    pub aborted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryMutationCounts {
    pub append: u64,
    pub replace: u64,
    pub item_types: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DivergenceCounts {
    pub exec_empty_output_nonzero: u64,
    pub collab_events_missing_turn_id: u64,
    pub hm_call_ids_with_artifact: u64,
    pub hm_function_call_ids: u64,
    pub exec_command_call_ids: u64,
    pub call_id_string_intersection: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSummary {
    pub schema_version: u32,
    pub dialect: String,
    pub session_id: String,
    pub rollout_sha256: String,
    pub line_count: u64,
    pub top_type_counts: BTreeMap<String, u64>,
    pub event_msg_counts: BTreeMap<String, u64>,
    pub history_mutation: HistoryMutationCounts,
    pub turns: TurnCounts,
    pub exec_command_end: u64,
    pub collab_spawn_end: u64,
    pub oversize_lines: u64,
    pub tool_result_refs: u64,
    pub divergence: DivergenceCounts,
    pub index_digest: String,
}

pub fn index_rollout(
    rollout: &Path,
    index_root: &Path,
    options: IndexOptions,
) -> SessionIndexResult<SessionSummary> {
    let dialect = detect_dialect(rollout)?;
    if dialect != DialectId::TraeCliDualStreamV1 {
        return Err(SessionIndexError::UnsupportedDialect {
            dialect: dialect.as_str().to_string(),
            detail: "index currently supports traecli.dual_stream.v1 only".into(),
        });
    }

    let rollout_bytes = fs::read(rollout).map_err(|source| SessionIndexError::Io {
        context: format!("read {}", rollout.display()),
        source,
    })?;
    let rollout_sha256 = hex_sha256(&rollout_bytes);

    let file = File::open(rollout).map_err(|source| SessionIndexError::Io {
        context: format!("open {}", rollout.display()),
        source,
    })?;
    let reader = BufReader::new(file);

    let mut state = IndexState::default();
    let mut oversize_lines = 0u64;
    let mut oversize_stubs: Vec<OversizeStub> = Vec::new();
    let mut line_count = 0u64;

    // We need session_id before writing spills into session dir. Two-phase:
    // collect oversize into temp vec of (line_no, bytes), resolve session_id first pass...
    // Simpler: peek session_meta in a pre-scan of first 32 lines, or buffer spills until known.
    let mut pending_spills: Vec<(u64, Vec<u8>)> = Vec::new();

    for line in reader.lines() {
        line_count += 1;
        let line = line.map_err(|source| SessionIndexError::Io {
            context: format!("read line {line_count} of {}", rollout.display()),
            source,
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let line_bytes = line.as_bytes();
        if line_bytes.len() > options.oversize_line_bytes {
            oversize_lines += 1;
            pending_spills.push((line_count, line_bytes.to_vec()));
            let value = serde_json::from_slice::<Value>(line_bytes).map_err(|err| {
                SessionIndexError::InvalidJsonl {
                    path: rollout.to_path_buf(),
                    line: line_count,
                    detail: err.to_string(),
                }
            })?;
            ingest_traecli_dual_stream(&value, &mut state);
            continue;
        }

        let value: Value =
            serde_json::from_str(trimmed).map_err(|err| SessionIndexError::InvalidJsonl {
                path: rollout.to_path_buf(),
                line: line_count,
                detail: err.to_string(),
            })?;
        ingest_traecli_dual_stream(&value, &mut state);
    }

    let session_id =
        state
            .session_id
            .take()
            .ok_or_else(|| SessionIndexError::MissingSessionId {
                path: rollout.to_path_buf(),
            })?;

    let sess = session_dir(index_root, &session_id)?;
    fs::create_dir_all(&sess).map_err(|source| SessionIndexError::Io {
        context: format!("create {}", sess.display()),
        source,
    })?;
    let spill_root = sess.join("spills");
    fs::create_dir_all(&spill_root).map_err(|source| SessionIndexError::Io {
        context: format!("create {}", spill_root.display()),
        source,
    })?;

    for (line_no, bytes) in pending_spills {
        let stub = spill_oversize_line(&spill_root, line_no, &bytes)?;
        oversize_stubs.push(stub);
    }

    // Tool-result references from artifacts dir.
    // `hm_call_ids_with_artifact` is the unique HM function_call call_id set
    // that appears in at least one artifact filename (not a per-file count).
    let mut tool_result_refs = 0u64;
    let mut artifact_call_ids: BTreeSet<String> = BTreeSet::new();
    if let Some(art) = options.artifacts_dir.as_ref() {
        if art.is_dir() {
            for entry in fs::read_dir(art).map_err(|source| SessionIndexError::Io {
                context: format!("read artifacts {}", art.display()),
                source,
            })? {
                let entry = entry.map_err(|source| SessionIndexError::Io {
                    context: "artifacts entry".into(),
                    source,
                })?;
                if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if let Some(call_id) = extract_call_id(&name) {
                    tool_result_refs += 1;
                    artifact_call_ids.insert(call_id);
                }
            }
        }
    }

    state.divergence.hm_function_call_ids = state.hm_call_ids.len() as u64;
    state.divergence.exec_command_call_ids = state.exec_call_ids.len() as u64;
    state.divergence.call_id_string_intersection =
        state.hm_call_ids.intersection(&state.exec_call_ids).count() as u64;
    state.divergence.hm_call_ids_with_artifact =
        state.hm_call_ids.intersection(&artifact_call_ids).count() as u64;

    let mut summary = SessionSummary {
        schema_version: 1,
        dialect: dialect.as_str().to_string(),
        session_id: session_id.clone(),
        rollout_sha256,
        line_count,
        top_type_counts: state.top_type_counts,
        event_msg_counts: state.event_msg_counts,
        history_mutation: HistoryMutationCounts {
            append: state.hm_append,
            replace: state.hm_replace,
            item_types: state.hm_item_types,
        },
        turns: TurnCounts {
            started: state.turns_started,
            completed: state.turns_completed,
            aborted: state.turns_aborted,
        },
        exec_command_end: state.exec_command_end,
        collab_spawn_end: state.collab_spawn_end,
        oversize_lines,
        tool_result_refs,
        divergence: state.divergence,
        index_digest: String::new(),
    };
    summary.index_digest = digest_summary(&summary);

    let summary_path = sess.join("summary.json");
    let body =
        serde_json::to_vec_pretty(&summary).map_err(|source| SessionIndexError::Serialization {
            artifact: "session summary",
            source,
        })?;
    fs::write(&summary_path, body).map_err(|source| SessionIndexError::Io {
        context: format!("write {}", summary_path.display()),
        source,
    })?;

    if !oversize_stubs.is_empty() {
        let stubs_path = sess.join("oversize_stubs.json");
        let body = serde_json::to_vec_pretty(&oversize_stubs).map_err(|source| {
            SessionIndexError::Serialization {
                artifact: "oversize stubs",
                source,
            }
        })?;
        fs::write(&stubs_path, body).map_err(|source| SessionIndexError::Io {
            context: format!("write {}", stubs_path.display()),
            source,
        })?;
    }

    Ok(summary)
}

/// Extract a TraeCLI `call_*` id from an artifact filename.
///
/// TraeCLI call ids are `call_` plus ASCII alphanumerics only. Filenames often
/// continue with `_exec-<uuid>`; underscores after the id body must not be
/// consumed or the bridge to HM `function_call.call_id` fails.
pub(crate) fn extract_call_id(name: &str) -> Option<String> {
    let bytes = name.as_bytes();
    let marker = b"call_";
    if let Some(pos) = bytes.windows(marker.len()).position(|w| w == marker) {
        let rest = &name[pos..];
        let end = rest
            .char_indices()
            .skip(marker.len())
            .find(|(_, c)| !c.is_ascii_alphanumeric())
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        let id = &rest[..end];
        if id.len() > marker.len() {
            return Some(id.to_string());
        }
    }
    None
}

fn digest_summary(summary: &SessionSummary) -> String {
    // Digest excludes index_digest itself and absolute paths.
    let mut for_hash = summary.clone();
    for_hash.index_digest.clear();
    let bytes = serde_json::to_vec(&for_hash).expect("summary serializable");
    format!("sha256:{}", hex_sha256(&bytes))
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod extract_call_id_tests {
    use super::extract_call_id;

    #[test]
    fn stops_before_exec_suffix() {
        let name = "exec_command-code-mode-nested_29_call_YRXk1sFVRzgWpctL06IyaPxh_exec-2911916b-4a00-4c4a-acc7-f96d3f7b0429-5.txt";
        assert_eq!(
            extract_call_id(name).as_deref(),
            Some("call_YRXk1sFVRzgWpctL06IyaPxh")
        );
    }

    #[test]
    fn plain_call_filename() {
        assert_eq!(
            extract_call_id("exec_command-call_ARTIFACTBRIDGE1.txt").as_deref(),
            Some("call_ARTIFACTBRIDGE1")
        );
    }
}
