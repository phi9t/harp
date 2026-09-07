use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json::Value;

use crate::error::SessionIndexError;
use crate::SessionIndexResult;

/// Supported Trace dialect identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialectId {
    TraeCliDualStreamV1,
}

impl DialectId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TraeCliDualStreamV1 => "traecli.dual_stream.v1",
        }
    }
}

const HM_MATERIAL_THRESHOLD: u64 = 10;

/// Detect the Trace dialect of a rollout JSONL file.
pub fn detect_dialect(path: &Path) -> SessionIndexResult<DialectId> {
    let file = File::open(path).map_err(|source| SessionIndexError::Io {
        context: format!("open {}", path.display()),
        source,
    })?;
    let reader = BufReader::new(file);

    let mut event_msg = 0u64;
    let mut history_mutation = 0u64;
    let mut response_item = 0u64;
    let mut nonblank = 0u64;
    let mut line_no = 0u64;

    for line in reader.lines() {
        line_no += 1;
        let line = line.map_err(|source| SessionIndexError::Io {
            context: format!("read {}", path.display()),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        nonblank = nonblank
            .checked_add(1)
            .ok_or_else(|| SessionIndexError::DetectFailed {
                path: path.to_path_buf(),
                detail: "nonblank record count overflowed".to_owned(),
            })?;
        let value: Value =
            serde_json::from_str(&line).map_err(|err| SessionIndexError::InvalidJsonl {
                path: path.to_path_buf(),
                line: line_no,
                detail: err.to_string(),
            })?;
        match value.get("type").and_then(|v| v.as_str()) {
            Some("event_msg") => event_msg = checked_increment(path, event_msg, "event_msg")?,
            Some("history_mutation") => {
                history_mutation = checked_increment(path, history_mutation, "history_mutation")?
            }
            Some("response_item") => {
                response_item = checked_increment(path, response_item, "response_item")?
            }
            _ => {}
        }
    }

    if event_msg >= 1 && is_material(history_mutation, nonblank) {
        return Ok(DialectId::TraeCliDualStreamV1);
    }
    if response_item > history_mutation {
        return Err(SessionIndexError::UnsupportedDialect {
            dialect: "traecli.response_item".to_string(),
            detail: format!(
                "response_item={response_item} dominates history_mutation={history_mutation}; codec not implemented in v1"
            ),
        });
    }

    Err(SessionIndexError::DetectFailed {
        path: path.to_path_buf(),
        detail: format!(
            "nonblank={nonblank} event_msg={event_msg} history_mutation={history_mutation} response_item={response_item}"
        ),
    })
}

fn is_material(history_mutation: u64, nonblank: u64) -> bool {
    history_mutation > HM_MATERIAL_THRESHOLD
        || history_mutation
            .checked_mul(100)
            .is_some_and(|scaled| scaled >= nonblank)
}

fn checked_increment(path: &Path, count: u64, field: &'static str) -> SessionIndexResult<u64> {
    count
        .checked_add(1)
        .ok_or_else(|| SessionIndexError::DetectFailed {
            path: path.to_path_buf(),
            detail: format!("{field} count overflowed"),
        })
}
