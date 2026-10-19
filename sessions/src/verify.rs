//! Independent verification of a stored Session summary.
//!
//! Re-indexes the Rollout into a scratch directory and diffs against the
//! claimed summary. Reparse digest equality alone cannot catch stable bugs;
//! this path is the mechanical verifier bar for operator trust.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use crate::error::SessionIndexError;
use crate::index::{index_rollout, IndexOptions, SessionSummary};
use crate::store::session_dir;
use crate::SessionIndexResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldCheck {
    pub name: String,
    pub status: CheckStatus,
    pub claimed: String,
    pub observed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackSeverity {
    Pass,
    Info,
    Warn,
    Improve,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeedbackItem {
    pub id: String,
    pub severity: FeedbackSeverity,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifyVerdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifyReport {
    pub schema_version: u32,
    pub verifier: String,
    pub session_id: String,
    pub verdict: VerifyVerdict,
    pub checks_passed: u64,
    pub checks_total: u64,
    pub checks: Vec<FieldCheck>,
    pub feedback: Vec<FeedbackItem>,
    pub claimed_index_digest: String,
    pub observed_index_digest: String,
}

pub struct VerifyOptions {
    pub index_root: PathBuf,
    pub artifacts_dir: Option<PathBuf>,
    pub oversize_line_bytes: usize,
    /// When set, write `verify-report.json` beside the claimed summary.
    pub write_report: bool,
}

/// Re-index `rollout` and compare to the stored summary for `session_id`.
pub fn verify_session(
    rollout: &Path,
    session_id: &str,
    options: VerifyOptions,
) -> SessionIndexResult<VerifyReport> {
    let claimed_path = session_dir(&options.index_root, session_id)?.join("summary.json");
    let claimed_raw =
        fs::read_to_string(&claimed_path).map_err(|source| SessionIndexError::Io {
            context: format!("read claimed {}", claimed_path.display()),
            source,
        })?;
    let claimed: SessionSummary = serde_json::from_str(&claimed_raw).map_err(|source| {
        SessionIndexError::InvalidStoredSummary {
            path: claimed_path.clone(),
            source,
        }
    })?;

    let scratch = TempDir::new().map_err(|source| SessionIndexError::Io {
        context: "create verify scratch dir".into(),
        source,
    })?;
    let observed = index_rollout(
        rollout,
        scratch.path(),
        IndexOptions {
            oversize_line_bytes: options.oversize_line_bytes,
            artifacts_dir: options.artifacts_dir.clone(),
        },
    )?;

    if observed.session_id != session_id {
        return Err(SessionIndexError::SessionIdMismatch {
            expected: session_id.to_owned(),
            observed: observed.session_id,
        });
    }

    let mut checks = Vec::new();
    check_eq(
        &mut checks,
        "index_digest",
        &claimed.index_digest,
        &observed.index_digest,
    );
    check_eq(
        &mut checks,
        "rollout_sha256",
        &claimed.rollout_sha256,
        &observed.rollout_sha256,
    );
    check_eq(&mut checks, "dialect", &claimed.dialect, &observed.dialect);
    check_eq(
        &mut checks,
        "line_count",
        claimed.line_count,
        observed.line_count,
    );
    check_eq(
        &mut checks,
        "hm.append",
        claimed.history_mutation.append,
        observed.history_mutation.append,
    );
    check_eq(
        &mut checks,
        "hm.replace",
        claimed.history_mutation.replace,
        observed.history_mutation.replace,
    );
    check_eq(
        &mut checks,
        "turns.started",
        claimed.turns.started,
        observed.turns.started,
    );
    check_eq(
        &mut checks,
        "turns.completed",
        claimed.turns.completed,
        observed.turns.completed,
    );
    check_eq(
        &mut checks,
        "turns.aborted",
        claimed.turns.aborted,
        observed.turns.aborted,
    );
    check_eq(
        &mut checks,
        "exec_command_end",
        claimed.exec_command_end,
        observed.exec_command_end,
    );
    check_eq(
        &mut checks,
        "collab_spawn_end",
        claimed.collab_spawn_end,
        observed.collab_spawn_end,
    );
    check_eq(
        &mut checks,
        "tool_result_refs",
        claimed.tool_result_refs,
        observed.tool_result_refs,
    );
    check_eq(
        &mut checks,
        "oversize_lines",
        claimed.oversize_lines,
        observed.oversize_lines,
    );
    check_eq(
        &mut checks,
        "divergence.exec_empty_output_nonzero",
        claimed.divergence.exec_empty_output_nonzero,
        observed.divergence.exec_empty_output_nonzero,
    );
    check_eq(
        &mut checks,
        "divergence.collab_events_missing_turn_id",
        claimed.divergence.collab_events_missing_turn_id,
        observed.divergence.collab_events_missing_turn_id,
    );
    check_eq(
        &mut checks,
        "divergence.hm_call_ids_with_artifact",
        claimed.divergence.hm_call_ids_with_artifact,
        observed.divergence.hm_call_ids_with_artifact,
    );
    check_eq(
        &mut checks,
        "divergence.hm_function_call_ids",
        claimed.divergence.hm_function_call_ids,
        observed.divergence.hm_function_call_ids,
    );
    check_eq(
        &mut checks,
        "divergence.exec_command_call_ids",
        claimed.divergence.exec_command_call_ids,
        observed.divergence.exec_command_call_ids,
    );
    check_eq(
        &mut checks,
        "divergence.call_id_string_intersection",
        claimed.divergence.call_id_string_intersection,
        observed.divergence.call_id_string_intersection,
    );

    for (k, v) in &claimed.history_mutation.item_types {
        let obs = observed
            .history_mutation
            .item_types
            .get(k)
            .copied()
            .unwrap_or(0);
        check_eq(&mut checks, &format!("hm.item.{k}"), *v, obs);
    }
    for (k, v) in &observed.history_mutation.item_types {
        if !claimed.history_mutation.item_types.contains_key(k) {
            check_eq(&mut checks, &format!("hm.item.{k}"), 0u64, *v);
        }
    }

    let checks_passed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count() as u64;
    let checks_total = checks.len() as u64;
    let failed = checks_total - checks_passed;
    let verdict = if failed == 0 {
        VerifyVerdict::Pass
    } else {
        VerifyVerdict::Fail
    };

    let mut feedback = Vec::new();
    feedback.push(FeedbackItem {
        id: "VF-01".into(),
        severity: FeedbackSeverity::Info,
        text: format!(
            "Re-index verifier matched {checks_passed}/{checks_total} checked fields against claimed summary."
        ),
    });
    if failed == 0 {
        feedback.push(FeedbackItem {
            id: "VF-02".into(),
            severity: FeedbackSeverity::Pass,
            text: "No mechanical field mismatches between claimed summary and fresh re-index."
                .into(),
        });
    } else {
        feedback.push(FeedbackItem {
            id: "VF-FAIL".into(),
            severity: FeedbackSeverity::Critical,
            text: format!("{failed} field mismatch(es); treat the claimed summary as untrusted until re-indexed."),
        });
    }

    if observed.divergence.call_id_string_intersection == 0 {
        feedback.push(FeedbackItem {
            id: "VF-03".into(),
            severity: FeedbackSeverity::Pass,
            text: "call_* vs exec-* namespace split confirmed (intersection 0).".into(),
        });
    } else {
        feedback.push(FeedbackItem {
            id: "VF-03".into(),
            severity: FeedbackSeverity::Warn,
            text: format!(
                "Unexpected call_id string intersection ({}); dialect join assumptions may have changed.",
                observed.divergence.call_id_string_intersection
            ),
        });
    }

    feedback.push(FeedbackItem {
        id: "VF-04".into(),
        severity: FeedbackSeverity::Warn,
        text: "AgentsView archives can be false-empty for dual-stream mega-sessions; do not treat AV health as corroboration of this index.".into(),
    });

    if options.artifacts_dir.is_none() {
        feedback.push(FeedbackItem {
            id: "VF-05".into(),
            severity: FeedbackSeverity::Improve,
            text: "Verify ran without --artifacts-dir; tool-result bridge metrics were not re-checked against disk blobs.".into(),
        });
    } else if observed.tool_result_refs > observed.divergence.hm_call_ids_with_artifact {
        feedback.push(FeedbackItem {
            id: "VF-06".into(),
            severity: FeedbackSeverity::Info,
            text: format!(
                "tool_result_refs={} unique HM bridges={}; duplicate or non-HM call_* filenames are expected on nested exec artifacts.",
                observed.tool_result_refs, observed.divergence.hm_call_ids_with_artifact
            ),
        });
    }

    let unfinished = observed
        .turns
        .started
        .saturating_sub(observed.turns.completed);
    if unfinished > 0 {
        feedback.push(FeedbackItem {
            id: "VF-07".into(),
            severity: FeedbackSeverity::Info,
            text: format!(
                "Turn arithmetic: started={} completed={} aborted={} => unfinished≈{} (aborts may overlap unfinished).",
                observed.turns.started,
                observed.turns.completed,
                observed.turns.aborted,
                unfinished
            ),
        });
    }

    feedback.push(FeedbackItem {
        id: "VF-08".into(),
        severity: FeedbackSeverity::Improve,
        text: "Summary still omits day timeline, model/effort mix, and tool-name histogram; promote those into a secondary operator report.".into(),
    });

    let report = VerifyReport {
        schema_version: 1,
        verifier: "harp-session-index.verify.reindex.v1".into(),
        session_id: session_id.to_string(),
        verdict,
        checks_passed,
        checks_total,
        checks,
        feedback,
        claimed_index_digest: claimed.index_digest,
        observed_index_digest: observed.index_digest,
    };

    if options.write_report {
        let out = session_dir(&options.index_root, session_id)?.join("verify-report.json");
        let body = serde_json::to_vec_pretty(&report).map_err(|source| {
            SessionIndexError::Serialization {
                artifact: "verify report",
                source,
            }
        })?;
        fs::write(&out, body).map_err(|source| SessionIndexError::Io {
            context: format!("write {}", out.display()),
            source,
        })?;
    }

    Ok(report)
}

fn check_eq<T: ToString + PartialEq>(
    checks: &mut Vec<FieldCheck>,
    name: &str,
    claimed: T,
    observed: T,
) {
    let status = if claimed == observed {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    };
    checks.push(FieldCheck {
        name: name.to_string(),
        status,
        claimed: claimed.to_string(),
        observed: observed.to_string(),
    });
}
