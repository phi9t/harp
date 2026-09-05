#![allow(dead_code)]

use std::ffi::{CStr, CString};
use std::path::Path;

use crate::AppError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectIdentity {
    pub(crate) device: u64,
    pub(crate) inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileSnapshot {
    pub(crate) identity: ObjectIdentity,
    pub(crate) length: u64,
    pub(crate) uid: u32,
    pub(crate) mode: u32,
    pub(crate) nlink: u64,
    pub(crate) digest: [u8; 32],
}

impl FileSnapshot {
    pub(crate) fn matches_moved_object(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FilePolicy {
    pub(crate) owner: Option<u32>,
    pub(crate) mode: Option<u32>,
    pub(crate) nlink: Option<u64>,
}

impl FilePolicy {
    pub(crate) const REGULAR: Self = Self {
        owner: None,
        mode: None,
        nlink: None,
    };

    pub(crate) fn exact(owner: u32, mode: u32, nlink: u64) -> Self {
        Self {
            owner: Some(owner),
            mode: Some(mode),
            nlink: Some(nlink),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AnchoredDirectory;
#[derive(Debug)]
pub(crate) struct StagedFile;
#[derive(Debug)]
pub(crate) struct StagedDirectory;
#[derive(Debug)]
pub(crate) struct HeldFileLock;

#[derive(Clone, Copy)]
pub(crate) enum RemovalSync {
    Cleanup,
    Rollback,
}

pub(crate) struct RemovalRequest<'a> {
    pub(crate) name: &'a CStr,
    pub(crate) expected: &'a FileSnapshot,
    pub(crate) label: &'a str,
    pub(crate) max_bytes: usize,
    pub(crate) policy: FilePolicy,
}

fn unsupported<T>() -> Result<T, AppError> {
    Err(AppError::external(
        "fs.unsupported",
        "descriptor-relative filesystem operations require macOS or Linux",
    ))
}

impl AnchoredDirectory {
    pub(crate) fn open(_path: &Path, _label: &str) -> Result<Self, AppError> {
        unsupported()
    }
    pub(crate) fn duplicate(&self, _label: &str) -> Result<Self, AppError> {
        unsupported()
    }
    pub(crate) fn walk(&self, _relative: &Path, _label: &str) -> Result<Self, AppError> {
        unsupported()
    }
    pub(crate) fn walk_or_create(
        &self,
        _relative: &Path,
        _mode: u32,
        _label: &str,
    ) -> Result<Self, AppError> {
        unsupported()
    }
    pub(crate) fn walk_parent(
        &self,
        _relative: &Path,
        _label: &str,
    ) -> Result<(Self, CString), AppError> {
        unsupported()
    }
    pub(crate) fn open_or_create_directory(
        &self,
        _name: &CStr,
        _mode: u32,
        _label: &str,
    ) -> Result<Self, AppError> {
        unsupported()
    }
    pub(crate) fn open_optional_directory(
        &self,
        _name: &CStr,
        _label: &str,
    ) -> Result<Option<Self>, AppError> {
        unsupported()
    }
    pub(crate) fn verify_namespace(&self, _label: &str) -> Result<(), AppError> {
        unsupported()
    }
    pub(crate) fn identity(&self, _label: &str) -> Result<ObjectIdentity, AppError> {
        unsupported()
    }
    pub(crate) fn verify_owner_mode(&self, _mode: u32, _label: &str) -> Result<(), AppError> {
        unsupported()
    }
    pub(crate) fn sync(&self, _label: &str) -> Result<(), AppError> {
        unsupported()
    }
    pub(crate) fn read_optional_bounded(
        &self,
        _name: &CStr,
        _label: &str,
        _max_bytes: usize,
        _policy: FilePolicy,
    ) -> Result<Option<(Vec<u8>, FileSnapshot)>, AppError> {
        unsupported()
    }
    pub(crate) fn read_optional_bounded_with_hook(
        &self,
        _name: &CStr,
        _label: &str,
        _max_bytes: usize,
        _policy: FilePolicy,
        _hook: impl FnOnce(),
    ) -> Result<Option<(Vec<u8>, FileSnapshot)>, AppError> {
        unsupported()
    }
    pub(crate) fn entry_names_bounded(
        &self,
        _label: &str,
        _max_entries: usize,
    ) -> Result<Vec<CString>, AppError> {
        unsupported()
    }
    pub(crate) fn create_staged_file(
        &self,
        _prefix: &str,
        _bytes: &[u8],
        _mode: u32,
        _label: &str,
    ) -> Result<StagedFile, AppError> {
        unsupported()
    }
    pub(crate) fn create_staged_file_with_cleanup_sync_hook(
        &self,
        _prefix: &str,
        _bytes: &[u8],
        _mode: u32,
        _label: &str,
        _cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<StagedFile, AppError> {
        unsupported()
    }
    pub(crate) fn create_file_noreplace(
        &self,
        _name: &CStr,
        _bytes: &[u8],
        _mode: u32,
        _label: &str,
    ) -> Result<FileSnapshot, AppError> {
        unsupported()
    }
    pub(crate) fn create_file_noreplace_with_cleanup_sync_hook(
        &self,
        _name: &CStr,
        _bytes: &[u8],
        _mode: u32,
        _label: &str,
        _cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        unsupported()
    }
    pub(crate) fn create_staged_directory(
        &self,
        _prefix: &str,
        _mode: u32,
        _label: &str,
    ) -> Result<StagedDirectory, AppError> {
        unsupported()
    }
    pub(crate) fn create_staged_directory_with_cleanup_sync_hook(
        &self,
        _prefix: &str,
        _mode: u32,
        _label: &str,
        _cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<StagedDirectory, AppError> {
        unsupported()
    }
    pub(crate) fn publish_staged_directory_noreplace_with_hook(
        &self,
        _staged: &mut StagedDirectory,
        _destination: &CStr,
        _label: &str,
        _after_rename: impl FnOnce() -> Result<(), AppError>,
    ) -> Result<AnchoredDirectory, AppError> {
        unsupported()
    }
    pub(crate) fn acquire_file_lock(
        &self,
        _name: &CStr,
        _exclusive: bool,
        _create: bool,
        _label: &str,
    ) -> Result<Option<HeldFileLock>, AppError> {
        unsupported()
    }
    pub(crate) fn verify_entry_identity(
        &self,
        _name: &CStr,
        _expected: ObjectIdentity,
        _directory: bool,
        _label: &str,
    ) -> Result<(), AppError> {
        unsupported()
    }
    pub(crate) fn cstring(_value: &str, _label: &str) -> Result<CString, AppError> {
        unsupported()
    }
}

impl StagedFile {
    pub(crate) fn name(&self) -> &CStr {
        unreachable!("unsupported staged file")
    }
    pub(crate) fn snapshot(&self) -> &FileSnapshot {
        unreachable!("unsupported staged file")
    }
    pub(crate) fn replace_snapshot(&mut self, _snapshot: FileSnapshot) {}
    pub(crate) fn disarm(&mut self) {}
    pub(crate) fn cleanup_with_parent_sync(
        &mut self,
        _label: &str,
        _sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        unsupported()
    }
}

impl StagedDirectory {
    pub(crate) fn directory(&self) -> &AnchoredDirectory {
        unreachable!("unsupported staged directory")
    }
    pub(crate) fn create_file_noreplace(
        &mut self,
        _name: &CStr,
        _bytes: &[u8],
        _mode: u32,
        _label: &str,
    ) -> Result<FileSnapshot, AppError> {
        unsupported()
    }
    pub(crate) fn create_file_noreplace_with_cleanup_sync_hook(
        &mut self,
        _name: &CStr,
        _bytes: &[u8],
        _mode: u32,
        _label: &str,
        _cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        unsupported()
    }
    pub(crate) fn disarm(&mut self) {}
    pub(crate) fn cleanup_with_parent_sync(
        &mut self,
        _label: &str,
        _sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        unsupported()
    }
}

impl HeldFileLock {
    pub(crate) fn verify(&self, _label: &str) -> Result<(), AppError> {
        unsupported()
    }
}
