#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::fs::descriptor::{AnchoredDirectory, FileSnapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PublicMutationOperation {
    FileSync,
    Rename,
    ParentSync,
    CleanupSync,
    RollbackSync,
    ExchangeValidation,
    InstallValidation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PublicMutationPhase {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PublicMutationEvent {
    pub(crate) operation: PublicMutationOperation,
    pub(crate) phase: PublicMutationPhase,
}

pub(crate) struct PublicFileMutation<'a> {
    pub(crate) relative: &'a Path,
    pub(crate) bytes: &'a [u8],
    pub(crate) expected: Option<&'a FileSnapshot>,
    pub(crate) label: &'a str,
    pub(crate) max_bytes: usize,
    pub(crate) required_mode: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct FileTreeLimits {
    pub max_depth: usize,
    pub max_entries: usize,
    pub max_matching_files: usize,
    pub max_aggregate_bytes: usize,
}

#[derive(Debug)]
pub struct HeldDirectory;

fn unsupported<T>() -> Result<T, AppError> {
    Err(AppError::external(
        "fs.unsupported",
        "secure filesystem operations require macOS or Linux",
    ))
}

impl HeldDirectory {
    pub fn open(_root: &Path, _label: &str) -> Result<Self, AppError> {
        unsupported()
    }
    pub(crate) fn anchored_directory(&self, _label: &str) -> Result<AnchoredDirectory, AppError> {
        unsupported()
    }
    pub fn read_optional_regular_file_bounded(
        &self,
        _relative: &Path,
        _label: &str,
        _max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AppError> {
        unsupported()
    }
    pub fn read_optional_regular_single_link_file_bounded(
        &self,
        _relative: &Path,
        _label: &str,
        _max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AppError> {
        unsupported()
    }
    pub fn read_optional_regular_file_snapshot_bounded(
        &self,
        _relative: &Path,
        _label: &str,
        _max_bytes: usize,
    ) -> Result<Option<FileSnapshot>, AppError> {
        unsupported()
    }
    pub fn regular_file_or_directory_exists(
        &self,
        _relative: &Path,
        _label: &str,
    ) -> Result<bool, AppError> {
        unsupported()
    }
    pub fn regular_file_exists(&self, _relative: &Path, _label: &str) -> Result<bool, AppError> {
        unsupported()
    }
    pub fn regular_files_with_extension(
        &self,
        _relative: &Path,
        _extension: &str,
        _label: &str,
    ) -> Result<Vec<PathBuf>, AppError> {
        unsupported()
    }
    pub fn bounded_regular_single_link_file_snapshots_with_extension(
        &self,
        _relative: &Path,
        _extension: &str,
        _label: &str,
        _max_file_bytes: usize,
        _limits: FileTreeLimits,
    ) -> Result<Vec<(PathBuf, Vec<u8>)>, AppError> {
        unsupported()
    }
    pub fn compare_and_replace_public_regular_file(
        &self,
        _relative: &Path,
        _bytes: &[u8],
        _expected: Option<&FileSnapshot>,
        _label: &str,
        _max_bytes: usize,
    ) -> Result<FileSnapshot, AppError> {
        unsupported()
    }
    pub(crate) fn compare_and_replace_public_regular_file_with_mode_and_events(
        &self,
        _mutation: PublicFileMutation<'_>,
        _events: impl FnMut(PublicMutationEvent) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        unsupported()
    }
    pub fn compare_and_remove_public_regular_file(
        &self,
        _relative: &Path,
        _expected: &FileSnapshot,
        _label: &str,
        _max_bytes: usize,
    ) -> Result<(), AppError> {
        unsupported()
    }
    pub(crate) fn compare_and_remove_public_regular_file_with_events(
        &self,
        _relative: &Path,
        _expected: &FileSnapshot,
        _label: &str,
        _max_bytes: usize,
        _events: impl FnMut(PublicMutationEvent) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        unsupported()
    }
}
