use std::ffi::{CStr, CString, OsStr};
use std::fs::File;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest as _, Sha256};

use crate::AppError;

static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectIdentity {
    pub(crate) device: u64,
    pub(crate) inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FileSnapshot {
    pub(crate) identity: ObjectIdentity,
    pub(crate) length: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
    pub(crate) uid: u32,
    pub(crate) mode: u32,
    pub(crate) nlink: u64,
    pub(crate) digest: [u8; 32],
}

impl FileSnapshot {
    pub(crate) fn matches_moved_object(&self, other: &Self) -> bool {
        self.identity == other.identity
            && self.length == other.length
            && self.uid == other.uid
            && self.mode == other.mode
            && self.nlink == other.nlink
            && self.digest == other.digest
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

#[derive(Clone, Debug)]
struct AnchoredComponent {
    name: CString,
    identity: ObjectIdentity,
}

#[derive(Debug)]
pub(crate) struct AnchoredDirectory {
    fd: OwnedFd,
    route: Vec<AnchoredComponent>,
}

#[derive(Debug)]
pub(crate) struct StagedFile {
    parent: AnchoredDirectory,
    name: CString,
    snapshot: FileSnapshot,
    active: bool,
}

#[derive(Debug)]
pub(crate) struct StagedDirectory {
    parent: AnchoredDirectory,
    name: CString,
    identity: ObjectIdentity,
    directory: AnchoredDirectory,
    children: Vec<(CString, ObjectIdentity)>,
    active: bool,
}

#[derive(Debug)]
struct CreatedFileGuard {
    parent: AnchoredDirectory,
    name: CString,
    identity: Option<ObjectIdentity>,
    active: bool,
}

#[derive(Debug)]
struct CreatedDirectoryGuard {
    parent: AnchoredDirectory,
    name: CString,
    identity: Option<ObjectIdentity>,
    active: bool,
}

#[derive(Debug)]
pub(crate) struct HeldFileLock {
    parent: AnchoredDirectory,
    name: CString,
    file: File,
    identity: ObjectIdentity,
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CreationStage {
    DirectoryAfterMkdir,
    DirectoryAfterIdentityObservation,
    DirectoryAfterIdentityBound,
    DirectoryChmod,
    FileAfterOpen,
    FileAfterIdentityObservation,
    FileAfterIdentityBound,
    FileChmod,
    FileWrite,
    FileSync,
    FileOwnerDuplicate,
    DirectoryOwnerDuplicate,
}

#[cfg(test)]
thread_local! {
    static CREATION_FAULT: std::cell::RefCell<Option<CreationStage>> = const {
        std::cell::RefCell::new(None)
    };
}

fn creation_stage(stage: CreationStage) -> Result<(), AppError> {
    #[cfg(test)]
    if CREATION_FAULT.with(|fault| {
        let mut fault = fault.borrow_mut();
        if *fault == Some(stage) {
            *fault = None;
            true
        } else {
            false
        }
    }) {
        return Err(AppError::external(
            "fs.creation_setup",
            format!("injected creation setup fault at {stage:?}"),
        ));
    }
    #[cfg(not(test))]
    let _ = stage;
    Ok(())
}

#[cfg(test)]
pub(crate) fn with_creation_fault<T>(stage: CreationStage, operation: impl FnOnce() -> T) -> T {
    CREATION_FAULT.with(|fault| {
        assert!(fault.borrow().is_none(), "creation fault is already active");
        *fault.borrow_mut() = Some(stage);
    });
    let result = operation();
    CREATION_FAULT.with(|fault| {
        assert!(fault.borrow().is_none(), "creation fault was not consumed");
    });
    result
}

impl AnchoredDirectory {
    pub(crate) fn open(path: &Path, label: &str) -> Result<Self, AppError> {
        let expected = std::fs::symlink_metadata(path)
            .map_err(|error| AppError::io("fs.root", label, error))?;
        if expected.file_type().is_symlink() || !expected.is_dir() {
            return Err(AppError::invalid_input(
                "fs.root",
                format!("{label} must be a real directory"),
            ));
        }
        let absolute =
            std::fs::canonicalize(path).map_err(|error| AppError::io("fs.root", label, error))?;
        let mut current = open_filesystem_root(label)?;
        for component in absolute.components() {
            match component {
                Component::RootDir => {}
                Component::Normal(name) => {
                    current = current.open_directory_component(&os_string(name, label)?, label)?;
                }
                Component::Prefix(_) | Component::CurDir | Component::ParentDir => {
                    return Err(AppError::invalid_input(
                        "fs.path",
                        format!("{label} must use an absolute path without traversal"),
                    ));
                }
            }
        }
        current.verify_namespace(label)?;
        if current.identity(label)? != identity(&expected) {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} identity changed while it was anchored"),
            ));
        }
        Ok(current)
    }

    pub(crate) fn duplicate(&self, label: &str) -> Result<Self, AppError> {
        Ok(Self {
            fd: duplicate_fd(self.fd.as_raw_fd(), label)?,
            route: self.route.clone(),
        })
    }

    pub(crate) fn walk(&self, relative: &Path, label: &str) -> Result<Self, AppError> {
        let components = relative_components(relative, label)?;
        let mut current = self.duplicate(label)?;
        for component in components {
            current = current.open_directory_component(&component, label)?;
        }
        Ok(current)
    }

    pub(crate) fn walk_or_create(
        &self,
        relative: &Path,
        mode: u32,
        label: &str,
    ) -> Result<Self, AppError> {
        let components = relative_components(relative, label)?;
        let mut current = self.duplicate(label)?;
        for component in components {
            current = current.open_or_create_directory(&component, mode, label)?;
        }
        Ok(current)
    }

    pub(crate) fn walk_parent(
        &self,
        relative: &Path,
        label: &str,
    ) -> Result<(Self, CString), AppError> {
        let mut components = relative_components(relative, label)?;
        let name = components
            .pop()
            .ok_or_else(|| AppError::invalid_input("fs.path", format!("{label} is empty")))?;
        let mut parent = self.duplicate(label)?;
        for component in components {
            parent = parent.open_directory_component(&component, label)?;
        }
        Ok((parent, name))
    }

    pub(crate) fn open_or_create_directory(
        &self,
        name: &CStr,
        mode: u32,
        label: &str,
    ) -> Result<Self, AppError> {
        match self.open_directory_component(name, label) {
            // Repository ancestors are not Harp-owned publication directories:
            // they may be secure private or group-managed directories.  Callers
            // verify exact modes at the boundaries they create and own.
            Ok(directory) => Ok(directory),
            Err(error) if error.code() == "fs.missing" => {
                self.verify_namespace(label)?;
                if unsafe {
                    libc::mkdirat(self.fd.as_raw_fd(), name.as_ptr(), mode as libc::mode_t)
                } != 0
                {
                    let error = std::io::Error::last_os_error();
                    if error.kind() == std::io::ErrorKind::AlreadyExists {
                        let directory = self.open_directory_component(name, label)?;
                        return Ok(directory);
                    }
                    return Err(AppError::io("fs.create_parent", label, error));
                }
                let directory = self.open_directory_component(name, label)?;
                directory.chmod(mode, label)?;
                directory.verify_owner_mode(mode, label)?;
                self.sync(label)?;
                Ok(directory)
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn open_optional_directory(
        &self,
        name: &CStr,
        label: &str,
    ) -> Result<Option<Self>, AppError> {
        match self.open_directory_component(name, label) {
            Ok(directory) => Ok(Some(directory)),
            Err(error) if error.code() == "fs.missing" => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub(crate) fn verify_namespace(&self, label: &str) -> Result<(), AppError> {
        let mut current = open_filesystem_root(label)?;
        for component in &self.route {
            current = current.open_directory_component(&component.name, label)?;
            if current.identity(label)? != component.identity {
                return Err(AppError::invalid_input(
                    "fs.concurrent_change",
                    format!("{label} namespace identity changed"),
                ));
            }
        }
        let expected = self
            .route
            .last()
            .map(|component| component.identity)
            .unwrap_or(current.identity(label)?);
        if self.identity(label)? != expected {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} held directory identity changed"),
            ));
        }
        Ok(())
    }

    pub(crate) fn identity(&self, label: &str) -> Result<ObjectIdentity, AppError> {
        let metadata = File::from(duplicate_fd(self.fd.as_raw_fd(), label)?)
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if !metadata.is_dir() {
            return Err(AppError::invalid_input(
                "fs.type",
                format!("{label} must be a directory"),
            ));
        }
        Ok(identity(&metadata))
    }

    pub(crate) fn verify_owner_mode(&self, mode: u32, label: &str) -> Result<(), AppError> {
        let metadata = File::from(duplicate_fd(self.fd.as_raw_fd(), label)?)
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if !metadata.is_dir()
            || metadata.uid() != effective_uid()
            || metadata.mode() & 0o7777 != mode
        {
            return Err(AppError::invalid_input(
                "fs.permissions",
                format!("{label} must be caller-owned with mode {mode:04o}"),
            ));
        }
        Ok(())
    }

    pub(crate) fn sync(&self, label: &str) -> Result<(), AppError> {
        self.verify_namespace(label)?;
        if unsafe { libc::fsync(self.fd.as_raw_fd()) } != 0 {
            return Err(AppError::io(
                "fs.sync",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        self.verify_namespace(label)
    }

    pub(crate) fn read_optional_bounded(
        &self,
        name: &CStr,
        label: &str,
        max_bytes: usize,
        policy: FilePolicy,
    ) -> Result<Option<(Vec<u8>, FileSnapshot)>, AppError> {
        self.read_optional_bounded_with_hook(name, label, max_bytes, policy, || {})
    }

    pub(crate) fn read_optional_bounded_with_hook(
        &self,
        name: &CStr,
        label: &str,
        max_bytes: usize,
        policy: FilePolicy,
        hook: impl FnOnce(),
    ) -> Result<Option<(Vec<u8>, FileSnapshot)>, AppError> {
        self.verify_namespace(label)?;
        let fd = unsafe {
            libc::openat(
                self.fd.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err(classify_open_error(self.fd.as_raw_fd(), name, label, error));
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        let before = file
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        validate_file_metadata(&before, label, policy)?;
        if before.len() > max_bytes as u64 {
            return Err(size_error(label, max_bytes));
        }
        let before_snapshot = snapshot_without_digest(&before);
        hook();
        self.verify_entry_identity(name, before_snapshot.identity, false, label)?;
        self.verify_namespace(label)?;
        let mut bytes = Vec::with_capacity((before.len() as usize).min(max_bytes));
        Read::by_ref(&mut file)
            .take((max_bytes as u64).saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| AppError::io("fs.read", label, error))?;
        if bytes.len() > max_bytes {
            return Err(size_error(label, max_bytes));
        }
        let after = file
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        validate_file_metadata(&after, label, policy)?;
        let mut snapshot = snapshot_without_digest(&after);
        if before_snapshot != snapshot || bytes.len() as u64 != snapshot.length {
            return Err(AppError::invalid_input(
                "fs.changed",
                format!("{label} changed while it was read"),
            ));
        }
        snapshot.digest = Sha256::digest(&bytes).into();
        self.verify_entry_identity(name, snapshot.identity, false, label)?;
        self.verify_namespace(label)?;
        Ok(Some((bytes, snapshot)))
    }

    pub(crate) fn entry_names_bounded(
        &self,
        label: &str,
        max_entries: usize,
    ) -> Result<Vec<CString>, AppError> {
        self.verify_namespace(label)?;
        let raw = unsafe {
            libc::openat(
                self.fd.as_raw_fd(),
                c".".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(AppError::io(
                "fs.read_dir",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        let enumerated = unsafe { OwnedFd::from_raw_fd(raw) };
        let enumerated_identity = File::from(duplicate_fd(enumerated.as_raw_fd(), label)?)
            .metadata()
            .map(|metadata| identity(&metadata))
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if enumerated_identity != self.identity(label)? {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} identity changed before enumeration"),
            ));
        }
        let raw = enumerated.as_raw_fd();
        let stream = unsafe { libc::fdopendir(raw) };
        if stream.is_null() {
            return Err(AppError::io(
                "fs.read_dir",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        std::mem::forget(enumerated);
        let mut names = Vec::new();
        loop {
            set_errno_zero();
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                let error = std::io::Error::last_os_error();
                unsafe { libc::closedir(stream) };
                if error.raw_os_error() == Some(0) {
                    break;
                }
                return Err(AppError::io("fs.read_dir", label, error));
            }
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
            if name.to_bytes() == b"." || name.to_bytes() == b".." {
                continue;
            }
            if names.len() >= max_entries {
                unsafe { libc::closedir(stream) };
                return Err(AppError::invalid_input(
                    "fs.limit",
                    format!("{label} exceeds {max_entries} entries"),
                ));
            }
            names.push(name.to_owned());
        }
        names.sort_by(|left, right| left.to_bytes().cmp(right.to_bytes()));
        self.verify_namespace(label)?;
        Ok(names)
    }

    #[allow(dead_code)] // compatibility wrapper for descriptor callers without a trace hook
    pub(crate) fn create_staged_file(
        &self,
        prefix: &str,
        bytes: &[u8],
        mode: u32,
        label: &str,
    ) -> Result<StagedFile, AppError> {
        self.create_staged_file_with_cleanup_sync_hook(prefix, bytes, mode, label, |_| Ok(()))
    }

    pub(crate) fn create_staged_file_with_cleanup_sync_hook(
        &self,
        prefix: &str,
        bytes: &[u8],
        mode: u32,
        label: &str,
        mut cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<StagedFile, AppError> {
        for _ in 0..32 {
            let name = unique_name(prefix)?;
            creation_stage(CreationStage::FileOwnerDuplicate)?;
            // Obtain the eventual owner before creating the name. A descriptor
            // allocation failure therefore cannot strand a completed temp file.
            let staged_parent = self.duplicate(label)?;
            match self.create_named_file(&name, bytes, mode, label, &mut cleanup_sync_event) {
                Ok(snapshot) => {
                    return Ok(StagedFile {
                        parent: staged_parent,
                        name,
                        snapshot,
                        active: true,
                    });
                }
                Err(error) if error.code() == "fs.exists" => continue,
                Err(error) => return Err(error),
            }
        }
        Err(AppError::external(
            "fs.temp",
            format!("could not allocate {label}"),
        ))
    }

    pub(crate) fn create_file_noreplace(
        &self,
        name: &CStr,
        bytes: &[u8],
        mode: u32,
        label: &str,
    ) -> Result<FileSnapshot, AppError> {
        self.create_file_noreplace_with_cleanup_sync_hook(name, bytes, mode, label, |_| Ok(()))
    }

    pub(crate) fn create_file_noreplace_with_cleanup_sync_hook(
        &self,
        name: &CStr,
        bytes: &[u8],
        mode: u32,
        label: &str,
        mut cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        self.create_named_file(name, bytes, mode, label, &mut cleanup_sync_event)
    }

    #[allow(dead_code)] // compatibility wrapper for descriptor callers without a trace hook
    pub(crate) fn create_staged_directory(
        &self,
        prefix: &str,
        mode: u32,
        label: &str,
    ) -> Result<StagedDirectory, AppError> {
        self.create_staged_directory_with_cleanup_sync_hook(prefix, mode, label, |_| Ok(()))
    }

    pub(crate) fn create_staged_directory_with_cleanup_sync_hook(
        &self,
        prefix: &str,
        mode: u32,
        label: &str,
        mut cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<StagedDirectory, AppError> {
        for _ in 0..32 {
            let name = unique_name(prefix)?;
            self.verify_namespace(label)?;
            let cleanup_parent = self.duplicate(label)?;
            creation_stage(CreationStage::DirectoryOwnerDuplicate)?;
            // Both guard and staged owner are acquired before mkdirat so every
            // post-create path has identity-bound, explicitly-syncable ownership.
            let staged_parent = self.duplicate(label)?;
            if unsafe { libc::mkdirat(self.fd.as_raw_fd(), name.as_ptr(), mode as libc::mode_t) }
                != 0
            {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::EEXIST) {
                    continue;
                }
                return Err(AppError::io("fs.create_parent", label, error));
            }
            let mut cleanup = CreatedDirectoryGuard::new(cleanup_parent, name.clone());
            if let Err(error) = creation_stage(CreationStage::DirectoryAfterMkdir) {
                self.remove_unobserved_created_directory(&name, label, &mut cleanup_sync_event)?;
                return Err(error);
            }
            let created = match statat_nofollow_retry(self.fd.as_raw_fd(), &name) {
                Ok(created) => created,
                Err(error) => {
                    self.remove_unobserved_created_directory(
                        &name,
                        label,
                        &mut cleanup_sync_event,
                    )?;
                    return Err(AppError::io("fs.metadata", label, error));
                }
            };
            let identity = identity_from_stat(&created);
            cleanup.bind_identity(identity);
            if let Err(error) = creation_stage(CreationStage::DirectoryAfterIdentityObservation) {
                cleanup.cleanup_with_parent_sync(label, &mut cleanup_sync_event)?;
                return Err(error);
            }
            if created.st_mode & libc::S_IFMT != libc::S_IFDIR {
                let error = AppError::invalid_input(
                    "fs.type",
                    format!("{label} staged directory has an unsafe type"),
                );
                cleanup.cleanup_with_parent_sync(label, &mut cleanup_sync_event)?;
                return Err(error);
            }
            if let Err(error) = creation_stage(CreationStage::DirectoryAfterIdentityBound) {
                cleanup.cleanup_with_parent_sync(label, &mut cleanup_sync_event)?;
                return Err(error);
            }
            let directory = (|| -> Result<AnchoredDirectory, AppError> {
                self.verify_entry_identity(&name, identity, true, label)?;
                let directory = self.open_directory_component(&name, label)?;
                if directory.identity(label)? != identity {
                    return Err(AppError::invalid_input(
                        "fs.concurrent_change",
                        format!("{label} staged directory changed during creation"),
                    ));
                }
                creation_stage(CreationStage::DirectoryChmod)?;
                directory.chmod(mode, label)?;
                directory.verify_owner_mode(mode, label)?;
                self.verify_entry_identity(&name, identity, true, label)?;
                Ok(directory)
            })();
            let directory = match directory {
                Ok(directory) => directory,
                Err(error) => {
                    cleanup.cleanup_with_parent_sync(label, &mut cleanup_sync_event)?;
                    return Err(error);
                }
            };
            let staged = StagedDirectory {
                parent: staged_parent,
                name,
                identity,
                directory,
                children: Vec::new(),
                active: true,
            };
            cleanup.disarm();
            return Ok(staged);
        }
        Err(AppError::external(
            "fs.temp",
            format!("could not allocate {label}"),
        ))
    }

    pub(crate) fn publish_staged_directory_noreplace_with_hook(
        &self,
        staged: &mut StagedDirectory,
        destination: &CStr,
        label: &str,
        after_rename: impl FnOnce() -> Result<(), AppError>,
    ) -> Result<AnchoredDirectory, AppError> {
        self.verify_entry_identity(&staged.name, staged.identity, true, label)?;
        self.verify_namespace(label)?;
        renameat_noreplace(
            self.fd.as_raw_fd(),
            &staged.name,
            self.fd.as_raw_fd(),
            destination,
        )
        .map_err(|error| AppError::io("fs.publish", label, error))?;
        // The digest name is now the sole ownership path.  Do not allow a
        // fallible readback to make staged cleanup traverse the old name.
        staged.disarm();
        after_rename()?;
        let published = self
            .open_optional_directory(destination, label)?
            .ok_or_else(|| {
                AppError::invalid_input(
                    "fs.concurrent_change",
                    format!("{label} disappeared during publication"),
                )
            })?;
        if published.identity(label)? != staged.identity {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} identity changed during publication"),
            ));
        }
        Ok(published)
    }

    pub(crate) fn acquire_file_lock(
        &self,
        name: &CStr,
        exclusive: bool,
        create: bool,
        label: &str,
    ) -> Result<Option<HeldFileLock>, AppError> {
        self.verify_namespace(label)?;
        let mut flags = libc::O_RDWR | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC;
        if create {
            flags |= libc::O_CREAT;
        }
        let fd = unsafe { libc::openat(self.fd.as_raw_fd(), name.as_ptr(), flags, 0o600) };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if !create && error.raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err(classify_open_error(self.fd.as_raw_fd(), name, label, error));
        }
        let file = unsafe { File::from_raw_fd(fd) };
        let metadata = file
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        validate_file_metadata(
            &metadata,
            label,
            FilePolicy {
                owner: Some(effective_uid()),
                mode: None,
                nlink: Some(1),
            },
        )?;
        if create && unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
            return Err(AppError::io(
                "fs.permissions",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        if create {
            file.sync_all()
                .map_err(|error| AppError::io("fs.sync", label, error))?;
            self.sync(label)?;
        }
        let metadata = file
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        validate_file_metadata(
            &metadata,
            label,
            FilePolicy::exact(effective_uid(), 0o600, 1),
        )?;
        let operation = if exclusive {
            libc::LOCK_EX | libc::LOCK_NB
        } else {
            libc::LOCK_SH | libc::LOCK_NB
        };
        if unsafe { libc::flock(file.as_raw_fd(), operation) } != 0 {
            let error = std::io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN)
            {
                return Err(AppError::invalid_input(
                    "fs.lock_busy",
                    format!("{label} is already locked"),
                ));
            }
            return Err(AppError::io("fs.lock", label, error));
        }
        let identity = identity(&metadata);
        self.verify_entry_identity(name, identity, false, label)?;
        self.verify_namespace(label)?;
        Ok(Some(HeldFileLock {
            parent: self.duplicate(label)?,
            name: name.to_owned(),
            file,
            identity,
        }))
    }

    pub(crate) fn exchange_staged(
        &self,
        staged: &StagedFile,
        destination: &CStr,
        destination_identity: ObjectIdentity,
        label: &str,
    ) -> Result<(), AppError> {
        self.verify_entry_identity(&staged.name, staged.snapshot.identity, false, label)?;
        self.verify_entry_identity(destination, destination_identity, false, label)?;
        self.verify_namespace(label)?;
        renameat_exchange(
            self.fd.as_raw_fd(),
            &staged.name,
            self.fd.as_raw_fd(),
            destination,
        )
        .map_err(|error| AppError::io("fs.publish", label, error))?;
        self.verify_namespace(label)
    }

    pub(crate) fn exchange_entries_if_identities(
        &self,
        left: &CStr,
        left_identity: ObjectIdentity,
        right: &CStr,
        right_identity: ObjectIdentity,
        label: &str,
    ) -> Result<(), AppError> {
        self.verify_entry_identity(left, left_identity, false, label)?;
        self.verify_entry_identity(right, right_identity, false, label)?;
        self.verify_namespace(label)?;
        renameat_exchange(self.fd.as_raw_fd(), left, self.fd.as_raw_fd(), right)
            .map_err(|error| AppError::io("fs.rollback", label, error))?;
        self.verify_namespace(label)
    }

    pub(crate) fn publish_staged_noreplace(
        &self,
        staged: &mut StagedFile,
        destination: &CStr,
        label: &str,
    ) -> Result<FileSnapshot, AppError> {
        self.verify_entry_identity(&staged.name, staged.snapshot.identity, false, label)?;
        self.verify_namespace(label)?;
        renameat_noreplace(
            self.fd.as_raw_fd(),
            &staged.name,
            self.fd.as_raw_fd(),
            destination,
        )
        .map_err(|error| AppError::io("fs.publish", label, error))?;
        // The destination is now the sole ownership path.  Carry the
        // descriptor snapshot into the caller's transaction rather than
        // letting a fallible readback leave Drop pointing at the old name.
        let snapshot = staged.snapshot.clone();
        staged.active = false;
        Ok(snapshot)
    }

    pub(crate) fn rename_noreplace(
        &self,
        source: &CStr,
        destination: &CStr,
        label: &str,
    ) -> Result<(), AppError> {
        self.verify_namespace(label)?;
        renameat_noreplace(
            self.fd.as_raw_fd(),
            source,
            self.fd.as_raw_fd(),
            destination,
        )
        .map_err(|error| AppError::io("fs.rename", label, error))?;
        self.verify_namespace(label)
    }

    pub(crate) fn remove_if_snapshot_with_hook(
        &self,
        name: &CStr,
        expected: &FileSnapshot,
        label: &str,
        max_bytes: usize,
        policy: FilePolicy,
        hook: impl FnOnce(),
    ) -> Result<(), AppError> {
        self.remove_if_snapshot_with_hooks_and_sync(
            RemovalRequest {
                name,
                expected,
                label,
                max_bytes,
                policy,
            },
            hook,
            |_, _| Ok(()),
        )
    }

    pub(crate) fn remove_if_snapshot_with_hooks_and_sync(
        &self,
        request: RemovalRequest<'_>,
        hook: impl FnOnce(),
        mut sync_event: impl FnMut(RemovalSync, bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        let RemovalRequest {
            name,
            expected,
            label,
            max_bytes,
            policy,
        } = request;
        let initial = self
            .read_optional_bounded(name, label, max_bytes, policy)?
            .ok_or_else(|| {
                AppError::invalid_input(
                    "fs.concurrent_change",
                    format!("{label} disappeared before removal"),
                )
            })?
            .1;
        if &initial != expected {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} changed before removal"),
            ));
        }
        hook();
        let quarantine = unique_name(".harp-quarantine")?;
        self.rename_noreplace(name, &quarantine, label)?;
        let observed = self.read_optional_bounded(&quarantine, label, max_bytes, policy);
        match observed {
            Ok(Some((_, snapshot))) if snapshot.matches_moved_object(expected) => {
                self.remove_name(&quarantine, snapshot.identity, label)?;
                sync_event(RemovalSync::Cleanup, true)?;
                self.sync(label)?;
                sync_event(RemovalSync::Cleanup, false)
            }
            observed => {
                let restore = self.rename_noreplace(&quarantine, name, label);
                if let Err(restore) = restore {
                    return Err(AppError::external(
                        "fs.rollback",
                        format!(
                            "{label} changed during removal and its quarantined replacement could not be restored: {}",
                            restore.message
                        ),
                    ));
                }
                sync_event(RemovalSync::Rollback, true)?;
                self.sync(label)?;
                sync_event(RemovalSync::Rollback, false)?;
                match observed {
                    Err(error) if error.code() != "fs.missing" => Err(error),
                    _ => Err(AppError::invalid_input(
                        "fs.concurrent_change",
                        format!("{label} identity changed during removal"),
                    )),
                }
            }
        }
    }

    pub(crate) fn remove_name(
        &self,
        name: &CStr,
        expected: ObjectIdentity,
        label: &str,
    ) -> Result<(), AppError> {
        self.verify_entry_identity(name, expected, false, label)?;
        if unsafe { libc::unlinkat(self.fd.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            return Err(AppError::io(
                "fs.remove",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        Ok(())
    }

    fn remove_directory(
        &self,
        name: &CStr,
        expected: ObjectIdentity,
        label: &str,
    ) -> Result<(), AppError> {
        self.verify_entry_identity(name, expected, true, label)?;
        if unsafe { libc::unlinkat(self.fd.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
            return Err(AppError::io(
                "fs.remove",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        Ok(())
    }

    pub(crate) fn verify_entry_identity(
        &self,
        name: &CStr,
        expected: ObjectIdentity,
        directory: bool,
        label: &str,
    ) -> Result<(), AppError> {
        let stat = statat_nofollow(self.fd.as_raw_fd(), name)
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        let expected_kind = if directory {
            libc::S_IFDIR
        } else {
            libc::S_IFREG
        };
        if stat.st_mode & libc::S_IFMT != expected_kind || identity_from_stat(&stat) != expected {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} identity changed"),
            ));
        }
        Ok(())
    }

    pub(crate) fn cstring(value: &str, label: &str) -> Result<CString, AppError> {
        CString::new(value)
            .map_err(|_| AppError::invalid_input("fs.path", format!("{label} contains NUL")))
    }

    fn open_directory_component(&self, name: &CStr, label: &str) -> Result<Self, AppError> {
        let fd = unsafe {
            libc::openat(
                self.fd.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(classify_open_error(
                self.fd.as_raw_fd(),
                name,
                label,
                std::io::Error::last_os_error(),
            ));
        }
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        let metadata = File::from(duplicate_fd(fd.as_raw_fd(), label)?)
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if !metadata.is_dir() {
            return Err(AppError::invalid_input(
                "fs.type",
                format!("{label} component must be a directory"),
            ));
        }
        let mut route = self.route.clone();
        route.push(AnchoredComponent {
            name: name.to_owned(),
            identity: identity(&metadata),
        });
        Ok(Self { fd, route })
    }

    fn create_named_file(
        &self,
        name: &CStr,
        bytes: &[u8],
        mode: u32,
        label: &str,
        cleanup_sync_event: &mut impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        self.verify_namespace(label)?;
        let cleanup_parent = self.duplicate(label)?;
        let fd = unsafe {
            libc::openat(
                self.fd.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                mode,
            )
        };
        if fd < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EEXIST) {
                return Err(AppError::invalid_input(
                    "fs.exists",
                    format!("{label} already exists"),
                ));
            }
            return Err(AppError::io("fs.create", label, error));
        }
        let mut file = unsafe { File::from_raw_fd(fd) };
        let mut cleanup = CreatedFileGuard::new(cleanup_parent, name.to_owned());
        if let Err(error) = creation_stage(CreationStage::FileAfterOpen) {
            self.remove_unobserved_created_file(name, label, cleanup_sync_event)?;
            return Err(error);
        }
        let created = match stat_fd_retry(file.as_raw_fd()) {
            Ok(created) => created,
            Err(error) => {
                self.remove_unobserved_created_file(name, label, cleanup_sync_event)?;
                return Err(AppError::io("fs.metadata", label, error));
            }
        };
        let created_identity = identity_from_stat(&created);
        cleanup.bind_identity(created_identity);
        if let Err(error) = creation_stage(CreationStage::FileAfterIdentityObservation) {
            cleanup.cleanup_with_parent_sync(label, cleanup_sync_event)?;
            return Err(error);
        }
        if created.st_mode & libc::S_IFMT != libc::S_IFREG {
            let error = AppError::invalid_input(
                "fs.type",
                format!("{label} staged file has an unsafe type"),
            );
            cleanup.cleanup_with_parent_sync(label, cleanup_sync_event)?;
            return Err(error);
        }
        if let Err(error) = creation_stage(CreationStage::FileAfterIdentityBound) {
            cleanup.cleanup_with_parent_sync(label, cleanup_sync_event)?;
            return Err(error);
        }
        let snapshot = (|| -> Result<FileSnapshot, AppError> {
            self.verify_entry_identity(name, created_identity, false, label)?;
            creation_stage(CreationStage::FileChmod)?;
            if unsafe { libc::fchmod(file.as_raw_fd(), mode as libc::mode_t) } != 0 {
                return Err(AppError::io(
                    "fs.permissions",
                    label,
                    std::io::Error::last_os_error(),
                ));
            }
            creation_stage(CreationStage::FileWrite)?;
            file.write_all(bytes)
                .map_err(|error| AppError::io("fs.write", label, error))?;
            creation_stage(CreationStage::FileSync)?;
            file.sync_all()
                .map_err(|error| AppError::io("fs.sync", label, error))?;
            let metadata = file
                .metadata()
                .map_err(|error| AppError::io("fs.metadata", label, error))?;
            if identity(&metadata) != created_identity {
                return Err(AppError::invalid_input(
                    "fs.concurrent_change",
                    format!("{label} staged file descriptor changed during creation"),
                ));
            }
            validate_file_metadata(
                &metadata,
                label,
                FilePolicy::exact(effective_uid(), mode, 1),
            )?;
            let mut snapshot = snapshot_without_digest(&metadata);
            snapshot.digest = Sha256::digest(bytes).into();
            self.verify_entry_identity(name, snapshot.identity, false, label)?;
            Ok(snapshot)
        })();
        let snapshot = match snapshot {
            Ok(snapshot) => snapshot,
            Err(error) => {
                cleanup.cleanup_with_parent_sync(label, cleanup_sync_event)?;
                return Err(error);
            }
        };
        cleanup.disarm();
        Ok(snapshot)
    }

    fn chmod(&self, mode: u32, label: &str) -> Result<(), AppError> {
        if unsafe { libc::fchmod(self.fd.as_raw_fd(), mode as libc::mode_t) } != 0 {
            return Err(AppError::io(
                "fs.permissions",
                label,
                std::io::Error::last_os_error(),
            ));
        }
        Ok(())
    }

    // Before the first identity observation, O_EXCL/mkdirat's unique name is
    // recoverable only under the cooperating-writer lock contract. A same-UID
    // lock-ignoring actor can race any pathname syscall and is outside that
    // contract; after observation all cleanup is identity-bound instead.
    fn remove_unobserved_created_file(
        &self,
        name: &CStr,
        label: &str,
        sync_event: &mut impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        if unsafe { libc::unlinkat(self.fd.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ENOENT) {
                return Err(AppError::io("fs.remove", label, error));
            }
        }
        sync_event(true)?;
        self.sync(label)?;
        sync_event(false)
    }

    fn remove_unobserved_created_directory(
        &self,
        name: &CStr,
        label: &str,
        sync_event: &mut impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        if unsafe { libc::unlinkat(self.fd.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ENOENT) {
                return Err(AppError::io("fs.remove", label, error));
            }
        }
        sync_event(true)?;
        self.sync(label)?;
        sync_event(false)
    }
}

impl StagedFile {
    pub(crate) fn name(&self) -> &CStr {
        &self.name
    }

    pub(crate) fn snapshot(&self) -> &FileSnapshot {
        &self.snapshot
    }

    pub(crate) fn replace_snapshot(&mut self, snapshot: FileSnapshot) {
        self.snapshot = snapshot;
    }

    pub(crate) fn disarm(&mut self) {
        self.active = false;
    }

    pub(crate) fn cleanup_with_parent_sync(
        &mut self,
        label: &str,
        mut sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        if !self.active {
            return Ok(());
        }
        self.parent
            .remove_name(&self.name, self.snapshot.identity, label)?;
        self.active = false;
        sync_event(true)?;
        self.parent.sync(label)?;
        sync_event(false)
    }
}

impl StagedDirectory {
    pub(crate) fn directory(&self) -> &AnchoredDirectory {
        &self.directory
    }

    #[allow(dead_code)] // compatibility wrapper for descriptor callers without a trace hook
    pub(crate) fn create_file_noreplace(
        &mut self,
        name: &CStr,
        bytes: &[u8],
        mode: u32,
        label: &str,
    ) -> Result<FileSnapshot, AppError> {
        let snapshot = self
            .directory
            .create_file_noreplace(name, bytes, mode, label)?;
        self.children.push((name.to_owned(), snapshot.identity));
        Ok(snapshot)
    }

    pub(crate) fn create_file_noreplace_with_cleanup_sync_hook(
        &mut self,
        name: &CStr,
        bytes: &[u8],
        mode: u32,
        label: &str,
        cleanup_sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        let snapshot = self
            .directory
            .create_file_noreplace_with_cleanup_sync_hook(
                name,
                bytes,
                mode,
                label,
                cleanup_sync_event,
            )?;
        self.children.push((name.to_owned(), snapshot.identity));
        Ok(snapshot)
    }

    pub(crate) fn disarm(&mut self) {
        self.active = false;
    }

    pub(crate) fn cleanup_with_parent_sync(
        &mut self,
        label: &str,
        mut sync_event: impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        if !self.active {
            return Ok(());
        }
        for (name, identity) in self.children.iter().rev() {
            self.directory.remove_name(name, *identity, label)?;
        }
        self.children.clear();
        self.parent
            .remove_directory(&self.name, self.identity, label)?;
        self.active = false;
        sync_event(true)?;
        self.parent.sync(label)?;
        sync_event(false)
    }
}

impl HeldFileLock {
    pub(crate) fn verify(&self, label: &str) -> Result<(), AppError> {
        let metadata = self
            .file
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        validate_file_metadata(
            &metadata,
            label,
            FilePolicy {
                owner: Some(effective_uid()),
                mode: Some(0o600),
                nlink: None,
            },
        )?;
        if metadata.nlink() != 1 || identity(&metadata) != self.identity {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} descriptor identity changed"),
            ));
        }
        self.parent
            .verify_entry_identity(&self.name, self.identity, false, label)?;
        self.parent.verify_namespace(label)
    }
}

impl Drop for HeldFileLock {
    fn drop(&mut self) {
        unsafe {
            libc::flock(self.file.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        if self.active {
            let _ = self.parent.remove_name(
                &self.name,
                self.snapshot.identity,
                "staged file drop cleanup",
            );
        }
    }
}

impl Drop for StagedDirectory {
    fn drop(&mut self) {
        if self.active {
            for (name, identity) in self.children.iter().rev() {
                let _ =
                    self.directory
                        .remove_name(name, *identity, "staged directory drop cleanup");
            }
            let _ = self.parent.remove_directory(
                &self.name,
                self.identity,
                "staged directory drop cleanup",
            );
        }
    }
}

impl CreatedFileGuard {
    fn new(parent: AnchoredDirectory, name: CString) -> Self {
        Self {
            parent,
            name,
            identity: None,
            active: true,
        }
    }

    fn bind_identity(&mut self, identity: ObjectIdentity) {
        self.identity = Some(identity);
    }

    fn disarm(&mut self) {
        self.active = false;
    }

    fn cleanup_with_parent_sync(
        &mut self,
        label: &str,
        sync_event: &mut impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        let Some(identity) = self.identity.filter(|_| self.active) else {
            return Ok(());
        };
        self.parent.remove_name(&self.name, identity, label)?;
        self.active = false;
        sync_event(true)?;
        self.parent.sync(label)?;
        sync_event(false)
    }
}

impl Drop for CreatedFileGuard {
    fn drop(&mut self) {
        if let Some(identity) = self.identity.filter(|_| self.active) {
            let _ = self
                .parent
                .remove_name(&self.name, identity, "created file cleanup");
        }
    }
}

impl CreatedDirectoryGuard {
    fn new(parent: AnchoredDirectory, name: CString) -> Self {
        Self {
            parent,
            name,
            identity: None,
            active: true,
        }
    }

    fn bind_identity(&mut self, identity: ObjectIdentity) {
        self.identity = Some(identity);
    }

    fn disarm(&mut self) {
        self.active = false;
    }

    fn cleanup_with_parent_sync(
        &mut self,
        label: &str,
        sync_event: &mut impl FnMut(bool) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        let Some(identity) = self.identity.filter(|_| self.active) else {
            return Ok(());
        };
        self.parent.remove_directory(&self.name, identity, label)?;
        self.active = false;
        sync_event(true)?;
        self.parent.sync(label)?;
        sync_event(false)
    }
}

impl Drop for CreatedDirectoryGuard {
    fn drop(&mut self) {
        if let Some(identity) = self.identity.filter(|_| self.active) {
            let _ = self
                .parent
                .remove_directory(&self.name, identity, "created directory cleanup");
        }
    }
}

fn open_filesystem_root(label: &str) -> Result<AnchoredDirectory, AppError> {
    let fd = unsafe {
        libc::open(
            c"/".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(AppError::io(
            "fs.root",
            label,
            std::io::Error::last_os_error(),
        ));
    }
    Ok(AnchoredDirectory {
        fd: unsafe { OwnedFd::from_raw_fd(fd) },
        route: Vec::new(),
    })
}

fn validate_file_metadata(
    metadata: &std::fs::Metadata,
    label: &str,
    policy: FilePolicy,
) -> Result<(), AppError> {
    if !metadata.is_file() {
        return Err(AppError::invalid_input(
            "fs.type",
            format!("{label} must be a regular file"),
        ));
    }
    if policy.owner.is_some_and(|owner| metadata.uid() != owner)
        || policy
            .mode
            .is_some_and(|mode| metadata.mode() & 0o7777 != mode)
        || policy.nlink.is_some_and(|nlink| metadata.nlink() != nlink)
    {
        return Err(AppError::invalid_input(
            "fs.permissions",
            format!("{label} has unsafe ownership, mode, or link count"),
        ));
    }
    Ok(())
}

fn snapshot_without_digest(metadata: &std::fs::Metadata) -> FileSnapshot {
    FileSnapshot {
        identity: identity(metadata),
        length: metadata.len(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
        uid: metadata.uid(),
        mode: metadata.mode() & 0o7777,
        nlink: metadata.nlink(),
        digest: [0; 32],
    }
}

fn identity(metadata: &std::fs::Metadata) -> ObjectIdentity {
    ObjectIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    }
}

fn identity_from_stat(stat: &libc::stat) -> ObjectIdentity {
    ObjectIdentity {
        device: stat.st_dev as u64,
        inode: stat.st_ino,
    }
}

fn statat_nofollow(dirfd: RawFd, name: &CStr) -> std::io::Result<libc::stat> {
    let mut stat = MaybeUninit::<libc::stat>::zeroed();
    if unsafe {
        libc::fstatat(
            dirfd,
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } == 0
    {
        Ok(unsafe { stat.assume_init() })
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn statat_nofollow_retry(dirfd: RawFd, name: &CStr) -> std::io::Result<libc::stat> {
    loop {
        match statat_nofollow(dirfd, name) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

fn stat_fd(fd: RawFd) -> std::io::Result<libc::stat> {
    let mut stat = MaybeUninit::<libc::stat>::zeroed();
    if unsafe { libc::fstat(fd, stat.as_mut_ptr()) } == 0 {
        Ok(unsafe { stat.assume_init() })
    } else {
        Err(std::io::Error::last_os_error())
    }
}

fn stat_fd_retry(fd: RawFd) -> std::io::Result<libc::stat> {
    loop {
        match stat_fd(fd) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

fn duplicate_fd(fd: RawFd, label: &str) -> Result<OwnedFd, AppError> {
    let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
    if duplicate < 0 {
        return Err(AppError::io(
            "fs.open",
            label,
            std::io::Error::last_os_error(),
        ));
    }
    Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
}

fn relative_components(relative: &Path, label: &str) -> Result<Vec<CString>, AppError> {
    if relative.is_absolute() || relative.as_os_str().is_empty() {
        return Err(AppError::invalid_input(
            "fs.path",
            format!("{label} must be a non-empty relative path"),
        ));
    }
    let mut components = Vec::new();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(AppError::invalid_input(
                "fs.path",
                format!("{label} must not traverse"),
            ));
        };
        components.push(os_string(component, label)?);
    }
    Ok(components)
}

fn os_string(value: &OsStr, label: &str) -> Result<CString, AppError> {
    CString::new(value.as_bytes())
        .map_err(|_| AppError::invalid_input("fs.path", format!("{label} contains NUL")))
}

fn unique_name(prefix: &str) -> Result<CString, AppError> {
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    CString::new(format!("{prefix}-{}-{counter}", std::process::id()))
        .map_err(|_| AppError::invalid_input("fs.path", "temporary name contains NUL"))
}

fn effective_uid() -> u32 {
    unsafe { libc::geteuid() }
}

fn size_error(label: &str, max_bytes: usize) -> AppError {
    AppError::invalid_input("fs.size", format!("{label} exceeds {max_bytes} bytes"))
}

fn classify_open_error(dirfd: RawFd, name: &CStr, label: &str, error: std::io::Error) -> AppError {
    match error.raw_os_error() {
        Some(libc::ENOENT) => AppError::invalid_input("fs.missing", format!("{label} is missing")),
        Some(libc::ELOOP) => {
            AppError::invalid_input("fs.symlink", format!("{label} cannot be a symlink"))
        }
        Some(libc::ENOTDIR) => match statat_nofollow(dirfd, name) {
            Ok(stat) if stat.st_mode & libc::S_IFMT == libc::S_IFLNK => {
                AppError::invalid_input("fs.symlink", format!("{label} cannot be a symlink"))
            }
            _ => AppError::invalid_input("fs.type", format!("{label} has a non-directory parent")),
        },
        _ => AppError::io("fs.open", label, error),
    }
}

#[cfg(target_os = "macos")]
fn renameat_noreplace(
    source_dir: RawFd,
    source: &CStr,
    destination_dir: RawFd,
    destination: &CStr,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameatx_np(
            source_dir,
            source.as_ptr(),
            destination_dir,
            destination.as_ptr(),
            libc::RENAME_EXCL,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
fn renameat_noreplace(
    source_dir: RawFd,
    source: &CStr,
    destination_dir: RawFd,
    destination: &CStr,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameat2(
            source_dir,
            source.as_ptr(),
            destination_dir,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "macos")]
fn renameat_exchange(
    left_dir: RawFd,
    left: &CStr,
    right_dir: RawFd,
    right: &CStr,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameatx_np(
            left_dir,
            left.as_ptr(),
            right_dir,
            right.as_ptr(),
            libc::RENAME_SWAP,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
fn renameat_exchange(
    left_dir: RawFd,
    left: &CStr,
    right_dir: RawFd,
    right: &CStr,
) -> std::io::Result<()> {
    if unsafe {
        libc::renameat2(
            left_dir,
            left.as_ptr(),
            right_dir,
            right.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "macos")]
fn set_errno_zero() {
    unsafe { *libc::__error() = 0 };
}

#[cfg(target_os = "linux")]
fn set_errno_zero() {
    unsafe { *libc::__errno_location() = 0 };
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        identity, with_creation_fault, AnchoredDirectory, CreatedDirectoryGuard, CreatedFileGuard,
        CreationStage,
    };

    #[test]
    fn file_creation_setup_faults_clean_the_created_name_and_consume_sync_hooks() {
        let stages = [
            CreationStage::FileAfterOpen,
            CreationStage::FileAfterIdentityObservation,
            CreationStage::FileAfterIdentityBound,
            CreationStage::FileChmod,
            CreationStage::FileWrite,
            CreationStage::FileSync,
        ];
        for (index, stage) in stages.into_iter().enumerate() {
            let temp = tempfile::tempdir().expect("temporary descriptor root");
            let parent = AnchoredDirectory::open(temp.path(), "descriptor root").expect("parent");
            let name = AnchoredDirectory::cstring(&format!("ledger-{index}"), "ledger")
                .expect("ledger name");
            let syncs = std::cell::RefCell::new(Vec::new());
            let error = with_creation_fault(stage, || {
                parent.create_file_noreplace_with_cleanup_sync_hook(
                    &name,
                    b"ledger\n",
                    0o600,
                    "ledger",
                    |before| {
                        syncs.borrow_mut().push(before);
                        Ok(())
                    },
                )
            })
            .expect_err("injected setup failure");
            assert_eq!(error.code(), "fs.creation_setup");
            assert!(!temp.path().join(name.to_string_lossy().as_ref()).exists());
            assert_eq!(*syncs.borrow(), vec![true, false]);
            parent
                .create_file_noreplace(&name, b"ledger\n", 0o600, "ledger")
                .expect("the released name remains reusable");
        }
    }

    #[test]
    fn directory_creation_setup_faults_leave_no_staged_residue_and_consume_sync_hooks() {
        let stages = [
            CreationStage::DirectoryAfterMkdir,
            CreationStage::DirectoryAfterIdentityObservation,
            CreationStage::DirectoryAfterIdentityBound,
            CreationStage::DirectoryChmod,
        ];
        for stage in stages {
            let temp = tempfile::tempdir().expect("temporary descriptor root");
            let parent = AnchoredDirectory::open(temp.path(), "descriptor root").expect("parent");
            let syncs = std::cell::RefCell::new(Vec::new());
            let error = with_creation_fault(stage, || {
                parent.create_staged_directory_with_cleanup_sync_hook(
                    ".prepared-generation",
                    0o700,
                    "staged generation",
                    |before| {
                        syncs.borrow_mut().push(before);
                        Ok(())
                    },
                )
            })
            .expect_err("injected setup failure");
            assert_eq!(error.code(), "fs.creation_setup");
            assert!(
                fs::read_dir(temp.path())
                    .expect("read temporary root")
                    .next()
                    .is_none(),
                "{stage:?} must not leave a staged directory"
            );
            assert_eq!(*syncs.borrow(), vec![true, false]);
            let mut staged = parent
                .create_staged_directory(".prepared-generation", 0o700, "staged generation")
                .expect("allocator remains reusable");
            staged
                .cleanup_with_parent_sync("staged cleanup", |_| Ok(()))
                .expect("cleanup reusable staged generation");
        }
    }

    #[test]
    fn actual_ledger_write_setup_failure_cleans_the_child_before_generation_cleanup() {
        let temp = tempfile::tempdir().expect("temporary descriptor root");
        let parent = AnchoredDirectory::open(temp.path(), "descriptor root").expect("parent");
        let mut staged = parent
            .create_staged_directory(".prepared-generation", 0o700, "staged generation")
            .expect("staged generation");
        let name = AnchoredDirectory::cstring("ledger.md", "ledger").expect("ledger name");
        let syncs = std::cell::RefCell::new(Vec::new());
        let error = with_creation_fault(CreationStage::FileWrite, || {
            staged.create_file_noreplace_with_cleanup_sync_hook(
                &name,
                b"ledger\n",
                0o600,
                "ledger",
                |before| {
                    syncs.borrow_mut().push(before);
                    Ok(())
                },
            )
        })
        .expect_err("ledger write setup fault");
        assert_eq!(error.code(), "fs.creation_setup");
        assert!(!temp
            .path()
            .read_dir()
            .expect("temporary root")
            .next()
            .expect("staged generation")
            .expect("staged entry")
            .path()
            .join("ledger.md")
            .exists());
        assert_eq!(*syncs.borrow(), vec![true, false]);
        staged
            .cleanup_with_parent_sync("staged cleanup", |_| Ok(()))
            .expect("staged cleanup");
    }

    #[test]
    fn owner_descriptor_faults_precede_creation_and_leave_no_residue() {
        for (stage, directory) in [
            (CreationStage::FileOwnerDuplicate, false),
            (CreationStage::DirectoryOwnerDuplicate, true),
        ] {
            let temp = tempfile::tempdir().expect("temporary descriptor root");
            let parent = AnchoredDirectory::open(temp.path(), "descriptor root").expect("parent");
            let syncs = std::cell::RefCell::new(Vec::new());
            let result = with_creation_fault(stage, || {
                if directory {
                    parent
                        .create_staged_directory_with_cleanup_sync_hook(
                            ".prepared-generation",
                            0o700,
                            "staged generation",
                            |before| {
                                syncs.borrow_mut().push(before);
                                Ok(())
                            },
                        )
                        .map(|_| ())
                } else {
                    parent
                        .create_staged_file_with_cleanup_sync_hook(
                            ".harp-public",
                            b"pointer\n",
                            0o600,
                            "staged pointer",
                            |before| {
                                syncs.borrow_mut().push(before);
                                Ok(())
                            },
                        )
                        .map(|_| ())
                }
            });
            assert_eq!(
                result.expect_err("owner descriptor fault").code(),
                "fs.creation_setup"
            );
            assert!(fs::read_dir(temp.path()).expect("root").next().is_none());
            assert!(
                syncs.borrow().is_empty(),
                "pre-create fault must not clean up"
            );
            if directory {
                let mut staged = parent
                    .create_staged_directory_with_cleanup_sync_hook(
                        ".prepared-generation",
                        0o700,
                        "staged generation",
                        |before| {
                            syncs.borrow_mut().push(before);
                            Ok(())
                        },
                    )
                    .expect("directory allocator retry");
                staged
                    .cleanup_with_parent_sync("staged cleanup", |before| {
                        syncs.borrow_mut().push(before);
                        Ok(())
                    })
                    .expect("directory cleanup");
            } else {
                let mut staged = parent
                    .create_staged_file_with_cleanup_sync_hook(
                        ".harp-public",
                        b"pointer\n",
                        0o600,
                        "staged pointer",
                        |before| {
                            syncs.borrow_mut().push(before);
                            Ok(())
                        },
                    )
                    .expect("file allocator retry");
                staged
                    .cleanup_with_parent_sync("staged cleanup", |before| {
                        syncs.borrow_mut().push(before);
                        Ok(())
                    })
                    .expect("file cleanup");
            }
            assert_eq!(*syncs.borrow(), vec![true, false]);
        }
    }

    #[test]
    fn file_creation_setup_cleanup_never_removes_a_same_name_replacement() {
        let temp = tempfile::tempdir().expect("temporary descriptor root");
        let path = temp.path().join("created");
        fs::write(&path, b"created").expect("created file");
        let parent = AnchoredDirectory::open(temp.path(), "descriptor root").expect("parent");
        let mut guard = CreatedFileGuard::new(
            parent
                .duplicate("cleanup parent")
                .expect("duplicate parent"),
            AnchoredDirectory::cstring("created", "created file").expect("name"),
        );
        guard.bind_identity(identity(&fs::metadata(&path).expect("created metadata")));

        fs::remove_file(&path).expect("remove created file");
        fs::write(&path, b"replacement").expect("replacement file");
        drop(guard);

        assert_eq!(
            fs::read(&path).expect("replacement survives"),
            b"replacement"
        );
    }

    #[test]
    fn directory_creation_setup_cleanup_never_removes_a_same_name_replacement() {
        let temp = tempfile::tempdir().expect("temporary descriptor root");
        let path = temp.path().join("created");
        fs::create_dir(&path).expect("created directory");
        let parent = AnchoredDirectory::open(temp.path(), "descriptor root").expect("parent");
        let mut guard = CreatedDirectoryGuard::new(
            parent
                .duplicate("cleanup parent")
                .expect("duplicate parent"),
            AnchoredDirectory::cstring("created", "created directory").expect("name"),
        );
        guard.bind_identity(identity(&fs::metadata(&path).expect("created metadata")));

        fs::remove_dir(&path).expect("remove created directory");
        fs::create_dir(&path).expect("replacement directory");
        drop(guard);

        assert!(path.is_dir(), "replacement directory survives");
    }
}
