//! Oversize JSONL line spill helpers.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::SessionIndexError;
use crate::SessionIndexResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OversizeStub {
    pub line_number: u64,
    pub byte_len: u64,
    pub sha256: String,
    pub spill_path: String,
}

pub fn spill_oversize_line(
    spill_root: &Path,
    line_number: u64,
    bytes: &[u8],
) -> SessionIndexResult<OversizeStub> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        hex.push_str(&format!("{b:02x}"));
    }
    let file_name = format!("line-{line_number}-{hex}.jsonl.part");
    let path = spill_root.join(&file_name);
    fs::write(&path, bytes).map_err(|source| SessionIndexError::Io {
        context: format!("spill {}", path.display()),
        source,
    })?;
    Ok(OversizeStub {
        line_number,
        byte_len: bytes.len() as u64,
        sha256: format!("sha256:{hex}"),
        spill_path: file_name,
    })
}

pub fn spill_path(spill_root: &Path, stub: &OversizeStub) -> PathBuf {
    spill_root.join(&stub.spill_path)
}
