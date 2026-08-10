//! Descriptor-relative filesystem trust boundary.
//!
//! All mutation after [`Dir::open_root`] is relative to held directory file
//! descriptors. The unsafe code in this module is limited to Unix syscalls,
//! immediately wraps returned descriptors in `OwnedFd`, retries interrupted
//! operations, and checks the opened inode against a no-follow directory entry.

#[cfg(any(target_vendor = "apple", target_os = "linux"))]
mod unix {
    use std::ffi::{CString, OsStr};
    use std::fmt;
    use std::fs::File;
    use std::io::{self, Read, Seek, SeekFrom, Write};
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    #[derive(Debug)]
    pub(crate) enum SecureError {
        NotFound,
        AlreadyExists,
        SymlinkOrWrongType(io::Error),
        IdentityMismatch {
            expected: Identity,
            actual: Identity,
        },
        TooManyEntries {
            max_entries: usize,
        },
        Io(io::Error),
        Unsupported,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum TempInitFault {
        Tmpfile,
        Fchmod,
        Fstat,
        Dup,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct Identity {
        device: u64,
        inode: u64,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum EntryKind {
        Directory,
        Regular,
        Other,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct EntryMetadata {
        pub(crate) kind: EntryKind,
        pub(crate) identity: Identity,
        pub(crate) size: u64,
        pub(crate) links: u64,
    }

    impl Identity {
        pub(crate) const fn device(self) -> u64 {
            self.device
        }

        pub(crate) const fn inode(self) -> u64 {
            self.inode
        }
    }

    impl fmt::Display for Identity {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}:{}", self.device, self.inode)
        }
    }

    #[derive(Debug)]
    pub(crate) struct Dir {
        descriptor: OwnedFd,
    }

    #[derive(Debug)]
    pub(crate) struct RegularFile {
        file: File,
        identity: Identity,
        size: u64,
        links: u64,
    }

    #[derive(Debug)]
    pub(crate) struct TempEntryGuard {
        file: File,
        _identity: Identity,
        _purpose: String,
    }

    struct TmpFileStream(*mut libc::FILE);

    impl TmpFileStream {
        fn close(mut self) -> io::Result<()> {
            let stream = std::mem::replace(&mut self.0, std::ptr::null_mut());
            if unsafe { libc::fclose(stream) } == 0 {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    impl Drop for TmpFileStream {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    libc::fclose(self.0);
                }
            }
        }
    }

    struct DirectoryStream(*mut libc::DIR);

    impl Drop for DirectoryStream {
        fn drop(&mut self) {
            unsafe {
                libc::closedir(self.0);
            }
        }
    }

    impl RegularFile {
        pub(crate) fn identity(&self) -> Identity {
            self.identity
        }

        pub(crate) fn size(&self) -> u64 {
            self.size
        }

        pub(crate) fn link_count(&self) -> u64 {
            self.links
        }

        pub(crate) fn write_append_line(&self, bytes: &[u8]) -> io::Result<()> {
            let mut written = 0;
            while written < bytes.len() {
                let result = unsafe {
                    libc::write(
                        self.file.as_raw_fd(),
                        bytes[written..].as_ptr().cast(),
                        bytes.len() - written,
                    )
                };
                if result > 0 {
                    written += result as usize;
                    continue;
                }
                if result == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "append write returned zero bytes",
                    ));
                }
                let error = io::Error::last_os_error();
                if error.kind() != io::ErrorKind::Interrupted {
                    return Err(error);
                }
            }
            Ok(())
        }

        pub(crate) fn read_bounded(&self, max_bytes: usize) -> io::Result<Vec<u8>> {
            let mut bytes = Vec::with_capacity(self.size.min(max_bytes as u64) as usize);
            let mut file = self.file.try_clone()?;
            file.seek(SeekFrom::Start(0))?;
            Read::by_ref(&mut file)
                .take(max_bytes.saturating_add(1) as u64)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        }

        pub(crate) fn sync_data(&self) -> io::Result<()> {
            self.file.sync_data()
        }

        pub(crate) fn sync_all(&self) -> io::Result<()> {
            self.file.sync_all()
        }

        pub(crate) fn last_byte(&self) -> io::Result<Option<u8>> {
            if self.size == 0 {
                return Ok(None);
            }
            let mut byte = 0_u8;
            loop {
                let result = unsafe {
                    libc::pread(
                        self.file.as_raw_fd(),
                        (&mut byte as *mut u8).cast(),
                        1,
                        (self.size - 1) as libc::off_t,
                    )
                };
                if result == 1 {
                    return Ok(Some(byte));
                }
                if result == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "evidence file shrank while reading its final byte",
                    ));
                }
                let error = io::Error::last_os_error();
                if error.kind() != io::ErrorKind::Interrupted {
                    return Err(error);
                }
            }
        }
    }

    impl TempEntryGuard {
        pub(crate) fn device_id(&self) -> u64 {
            self._identity.device
        }

        pub(crate) fn write_all(&mut self, bytes: &[u8]) -> io::Result<()> {
            self.file.write_all(bytes)
        }

        pub(crate) fn sync_all(&self) -> io::Result<()> {
            self.file.sync_all()
        }

        pub(crate) fn publish_immutable(
            &self,
            destination_directory: &Dir,
            destination: &str,
        ) -> Result<RegularFile, SecureError> {
            let destination = component(destination)?;
            publish_from_fd(
                self.file.as_raw_fd(),
                destination_directory.descriptor.as_raw_fd(),
                &destination,
            )?;
            open_regular_at(destination_directory.descriptor.as_raw_fd(), &destination)
        }
    }

    impl Dir {
        pub(crate) fn identity(&self) -> Result<Identity, SecureError> {
            identity_for_fd(self.descriptor.as_raw_fd()).map_err(SecureError::Io)
        }

        pub(crate) fn device_id(&self) -> Result<u64, SecureError> {
            identity_for_fd(self.descriptor.as_raw_fd())
                .map(|identity| identity.device)
                .map_err(SecureError::Io)
        }

        pub(crate) fn open_root(path: &Path) -> Result<Self, SecureError> {
            let path_name = c_string(path.as_os_str()).map_err(SecureError::Io)?;
            let raw_fd = retry_fd(|| unsafe {
                libc::open(
                    path_name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            })
            .map_err(map_open_error)?;
            let descriptor = unsafe { OwnedFd::from_raw_fd(raw_fd) };
            let opened = identity_for_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
            let entry = stat_path_nofollow(&path_name).map_err(map_open_error)?;
            if !is_directory(entry.mode) {
                return Err(wrong_type("root is not a directory"));
            }
            let entry_identity = identity_from_stat(&entry);
            if opened != entry_identity {
                return Err(SecureError::IdentityMismatch {
                    expected: entry_identity,
                    actual: opened,
                });
            }
            Ok(Self { descriptor })
        }

        pub(crate) fn open_dir(&self, name: &str) -> Result<Self, SecureError> {
            let name = component(name)?;
            let entry =
                stat_at_nofollow(self.descriptor.as_raw_fd(), &name).map_err(map_open_error)?;
            if !is_directory(entry.mode) {
                return Err(wrong_type("entry is not a directory"));
            }
            let expected = identity_from_stat(&entry);
            let raw_fd = retry_fd(|| unsafe {
                libc::openat(
                    self.descriptor.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            })
            .map_err(map_open_error)?;
            let descriptor = unsafe { OwnedFd::from_raw_fd(raw_fd) };
            let actual = identity_for_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
            if expected != actual {
                return Err(SecureError::IdentityMismatch { expected, actual });
            }
            Ok(Self { descriptor })
        }

        pub(crate) fn entry_metadata(&self, name: &str) -> Result<EntryMetadata, SecureError> {
            let name = component(name)?;
            let stat =
                stat_at_nofollow(self.descriptor.as_raw_fd(), &name).map_err(map_open_error)?;
            Ok(EntryMetadata {
                kind: if is_directory(stat.mode) {
                    EntryKind::Directory
                } else if is_regular(stat.mode) {
                    EntryKind::Regular
                } else {
                    EntryKind::Other
                },
                identity: identity_from_stat(&stat),
                size: stat.size,
                links: stat.links,
            })
        }

        pub(crate) fn ensure_dir(&self, name: &str) -> Result<(Self, bool), SecureError> {
            let name = component(name)?;
            let created = match retry_zero(|| unsafe {
                libc::mkdirat(self.descriptor.as_raw_fd(), name.as_ptr(), 0o700)
            }) {
                Ok(()) => true,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => false,
                Err(error) => return Err(SecureError::Io(error)),
            };
            if created {
                self.sync()?;
            }
            self.open_dir_cstring(&name)
                .map(|directory| (directory, created))
        }

        fn open_dir_cstring(&self, name: &CString) -> Result<Self, SecureError> {
            let entry =
                stat_at_nofollow(self.descriptor.as_raw_fd(), name).map_err(map_open_error)?;
            if !is_directory(entry.mode) {
                return Err(wrong_type("entry is not a directory"));
            }
            let expected = identity_from_stat(&entry);
            let raw_fd = retry_fd(|| unsafe {
                libc::openat(
                    self.descriptor.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            })
            .map_err(map_open_error)?;
            let descriptor = unsafe { OwnedFd::from_raw_fd(raw_fd) };
            let actual = identity_for_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
            if expected != actual {
                return Err(SecureError::IdentityMismatch { expected, actual });
            }
            Ok(Self { descriptor })
        }

        pub(crate) fn open_regular_optional(
            &self,
            name: &str,
        ) -> Result<Option<RegularFile>, SecureError> {
            let name = component(name)?;
            let entry = match stat_at_nofollow(self.descriptor.as_raw_fd(), &name) {
                Ok(entry) => entry,
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(map_open_error(error)),
            };
            if !is_regular(entry.mode) {
                return Err(wrong_type("entry is not a regular file"));
            }
            let expected = identity_from_stat(&entry);
            let raw_fd = retry_fd(|| unsafe {
                libc::openat(
                    self.descriptor.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            })
            .map_err(map_open_error)?;
            let descriptor = unsafe { OwnedFd::from_raw_fd(raw_fd) };
            let stat = stat_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
            if !is_regular(stat.mode) {
                return Err(wrong_type("opened entry is not a regular file"));
            }
            let actual = identity_from_stat(&stat);
            if expected != actual {
                return Err(SecureError::IdentityMismatch { expected, actual });
            }
            Ok(Some(RegularFile {
                file: File::from(descriptor),
                identity: actual,
                size: stat.size,
                links: stat.links,
            }))
        }

        pub(crate) fn create_temp(
            &self,
            purpose: &str,
            injected_fault: Option<TempInitFault>,
        ) -> Result<TempEntryGuard, SecureError> {
            let _ = (self, purpose, injected_fault);
            #[cfg(target_vendor = "apple")]
            {
                if injected_fault == Some(TempInitFault::Tmpfile) {
                    return Err(SecureError::Io(io::Error::other(
                        "injected anonymous tmpfile failure",
                    )));
                }
                let stream_pointer = unsafe { libc::tmpfile() };
                if stream_pointer.is_null() {
                    return Err(SecureError::Io(io::Error::last_os_error()));
                }
                let stream = TmpFileStream(stream_pointer);
                let stream_fd = unsafe { libc::fileno(stream.0) };
                if stream_fd < 0 {
                    return Err(SecureError::Io(io::Error::last_os_error()));
                }
                if injected_fault == Some(TempInitFault::Dup) {
                    return Err(SecureError::Io(io::Error::other(
                        "injected anonymous fd duplication failure",
                    )));
                }
                let descriptor = duplicate_fd(stream_fd).map_err(SecureError::Io)?;
                stream.close().map_err(SecureError::Io)?;
                if injected_fault == Some(TempInitFault::Fchmod) {
                    return Err(SecureError::Io(io::Error::other(
                        "injected anonymous fchmod failure",
                    )));
                }
                retry_zero(|| unsafe { libc::fchmod(descriptor.as_raw_fd(), 0o600) })
                    .map_err(SecureError::Io)?;
                if injected_fault == Some(TempInitFault::Fstat) {
                    return Err(SecureError::Io(io::Error::other(
                        "injected anonymous fstat failure",
                    )));
                }
                let stat = stat_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
                if !is_regular(stat.mode) {
                    return Err(wrong_type(
                        "anonymous publication source is not a regular file",
                    ));
                }
                return Ok(TempEntryGuard {
                    file: File::from(descriptor),
                    _identity: identity_from_stat(&stat),
                    _purpose: purpose.to_owned(),
                });
            }
            #[allow(unreachable_code)]
            Err(SecureError::Unsupported)
        }

        pub(crate) fn open_append_create(
            &self,
            name: &str,
        ) -> Result<(RegularFile, bool), SecureError> {
            let name = component(name)?;
            let (raw_fd, expected, created) =
                match stat_at_nofollow(self.descriptor.as_raw_fd(), &name) {
                    Ok(entry) => {
                        if !is_regular(entry.mode) {
                            return Err(wrong_type("entry is not a regular file"));
                        }
                        let raw_fd = retry_fd(|| unsafe {
                            libc::openat(
                                self.descriptor.as_raw_fd(),
                                name.as_ptr(),
                                libc::O_RDWR
                                    | libc::O_APPEND
                                    | libc::O_NONBLOCK
                                    | libc::O_NOFOLLOW
                                    | libc::O_CLOEXEC,
                            )
                        })
                        .map_err(map_open_error)?;
                        (raw_fd, Some(identity_from_stat(&entry)), false)
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        match retry_fd(|| unsafe {
                            libc::openat(
                                self.descriptor.as_raw_fd(),
                                name.as_ptr(),
                                libc::O_RDWR
                                    | libc::O_APPEND
                                    | libc::O_CREAT
                                    | libc::O_EXCL
                                    | libc::O_NOFOLLOW
                                    | libc::O_CLOEXEC,
                                0o600,
                            )
                        }) {
                            Ok(raw_fd) => (raw_fd, None, true),
                            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                                let entry = stat_at_nofollow(self.descriptor.as_raw_fd(), &name)
                                    .map_err(map_open_error)?;
                                if !is_regular(entry.mode) {
                                    return Err(wrong_type("entry is not a regular file"));
                                }
                                let raw_fd = retry_fd(|| unsafe {
                                    libc::openat(
                                        self.descriptor.as_raw_fd(),
                                        name.as_ptr(),
                                        libc::O_RDWR
                                            | libc::O_APPEND
                                            | libc::O_NONBLOCK
                                            | libc::O_NOFOLLOW
                                            | libc::O_CLOEXEC,
                                    )
                                })
                                .map_err(map_open_error)?;
                                (raw_fd, Some(identity_from_stat(&entry)), false)
                            }
                            Err(error) => return Err(map_open_error(error)),
                        }
                    }
                    Err(error) => return Err(map_open_error(error)),
                };
            let descriptor = unsafe { OwnedFd::from_raw_fd(raw_fd) };
            retry_zero(|| unsafe { libc::fchmod(descriptor.as_raw_fd(), 0o600) })
                .map_err(SecureError::Io)?;
            let stat = stat_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
            if !is_regular(stat.mode) {
                return Err(wrong_type("opened append entry is not a regular file"));
            }
            let actual = identity_from_stat(&stat);
            if let Some(expected) = expected {
                if expected != actual {
                    return Err(SecureError::IdentityMismatch { expected, actual });
                }
            }
            Ok((
                RegularFile {
                    file: File::from(descriptor),
                    identity: actual,
                    size: stat.size,
                    links: stat.links,
                },
                created,
            ))
        }

        pub(crate) fn sync(&self) -> Result<(), SecureError> {
            retry_zero(|| unsafe { libc::fsync(self.descriptor.as_raw_fd()) })
                .map_err(SecureError::Io)
        }

        pub(crate) fn verify_entry_identity(
            &self,
            name: &str,
            expected: Identity,
        ) -> Result<(), SecureError> {
            let name = component(name)?;
            let stat =
                stat_at_nofollow(self.descriptor.as_raw_fd(), &name).map_err(map_open_error)?;
            if !is_regular(stat.mode) {
                return Err(wrong_type("entry is not a regular file"));
            }
            let actual = identity_from_stat(&stat);
            if actual != expected {
                return Err(SecureError::IdentityMismatch { expected, actual });
            }
            Ok(())
        }

        #[cfg(test)]
        pub(crate) fn replace_entry_with_symlink(
            &self,
            name: &str,
            target: &Path,
        ) -> Result<(), SecureError> {
            let name = component(name)?;
            retry_zero(|| unsafe { libc::unlinkat(self.descriptor.as_raw_fd(), name.as_ptr(), 0) })
                .map_err(map_open_error)?;
            let target = c_string(target.as_os_str()).map_err(SecureError::Io)?;
            retry_zero(|| unsafe {
                libc::symlinkat(target.as_ptr(), self.descriptor.as_raw_fd(), name.as_ptr())
            })
            .map_err(map_open_error)
        }

        pub(crate) fn read_entry_names(
            &self,
            max_entries: usize,
        ) -> Result<Vec<String>, SecureError> {
            let duplicated = duplicate_fd(self.descriptor.as_raw_fd()).map_err(SecureError::Io)?;
            let raw_fd = duplicated.as_raw_fd();
            let directory = unsafe { libc::fdopendir(raw_fd) };
            if directory.is_null() {
                return Err(SecureError::Io(io::Error::last_os_error()));
            }
            std::mem::forget(duplicated);
            let directory = DirectoryStream(directory);
            let mut names = Vec::new();
            loop {
                set_errno_zero();
                let entry = unsafe { libc::readdir(directory.0) };
                if entry.is_null() {
                    let error_code = errno_value();
                    if error_code == 0 {
                        return Ok(names);
                    }
                    return Err(SecureError::Io(io::Error::from_raw_os_error(error_code)));
                }
                let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
                let bytes = name.to_bytes();
                if matches!(bytes, b"." | b"..") {
                    continue;
                }
                let name = std::str::from_utf8(bytes).map_err(|error| {
                    SecureError::Io(io::Error::new(io::ErrorKind::InvalidData, error))
                })?;
                if names.len() == max_entries {
                    return Err(SecureError::TooManyEntries { max_entries });
                }
                names.push(name.to_owned());
            }
        }
    }

    pub(super) fn publish_from_fd(
        source_fd: RawFd,
        destination_directory_fd: RawFd,
        destination: &CString,
    ) -> Result<(), SecureError> {
        #[cfg(target_vendor = "apple")]
        {
            // The source authority is an already-open anonymous fd. The
            // destination is one validated component under a held directory
            // fd and fclonefileat requires it not to exist, so no pathname
            // traversal flags are needed at this boundary.
            return retry_zero(|| unsafe {
                libc::fclonefileat(source_fd, destination_directory_fd, destination.as_ptr(), 0)
            })
            .map_err(map_publish_error);
        }

        #[allow(unreachable_code)]
        Err(SecureError::Unsupported)
    }

    fn open_regular_at(directory_fd: RawFd, name: &CString) -> Result<RegularFile, SecureError> {
        let entry = stat_at_nofollow(directory_fd, name).map_err(map_open_error)?;
        if !is_regular(entry.mode) {
            return Err(wrong_type("entry is not a regular file"));
        }
        let expected = identity_from_stat(&entry);
        let raw_fd = retry_fd(|| unsafe {
            libc::openat(
                directory_fd,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NONBLOCK | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        })
        .map_err(map_open_error)?;
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        let stat = stat_fd(descriptor.as_raw_fd()).map_err(SecureError::Io)?;
        if !is_regular(stat.mode) {
            return Err(wrong_type("opened entry is not a regular file"));
        }
        let actual = identity_from_stat(&stat);
        if expected != actual {
            return Err(SecureError::IdentityMismatch { expected, actual });
        }
        Ok(RegularFile {
            file: File::from(descriptor),
            identity: actual,
            size: stat.size,
            links: stat.links,
        })
    }

    #[derive(Clone, Copy)]
    struct FileStat {
        device: u64,
        inode: u64,
        mode: libc::mode_t,
        size: u64,
        links: u64,
    }

    fn stat_fd(fd: RawFd) -> io::Result<FileStat> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        retry_zero(|| unsafe { libc::fstat(fd, stat.as_mut_ptr()) })?;
        Ok(file_stat(unsafe { stat.assume_init() }))
    }

    fn stat_at_nofollow(directory_fd: RawFd, name: &CString) -> io::Result<FileStat> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        retry_zero(|| unsafe {
            libc::fstatat(
                directory_fd,
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        })?;
        Ok(file_stat(unsafe { stat.assume_init() }))
    }

    fn stat_path_nofollow(path: &CString) -> io::Result<FileStat> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        retry_zero(|| unsafe { libc::lstat(path.as_ptr(), stat.as_mut_ptr()) })?;
        Ok(file_stat(unsafe { stat.assume_init() }))
    }

    #[allow(clippy::unnecessary_cast)] // libc inode/device widths vary by Unix target.
    fn file_stat(stat: libc::stat) -> FileStat {
        FileStat {
            device: stat.st_dev as u64,
            inode: stat.st_ino as u64,
            mode: stat.st_mode,
            size: stat.st_size.max(0) as u64,
            links: stat.st_nlink as u64,
        }
    }

    fn identity_for_fd(fd: RawFd) -> io::Result<Identity> {
        Ok(identity_from_stat(&stat_fd(fd)?))
    }

    fn duplicate_fd(fd: RawFd) -> io::Result<OwnedFd> {
        let duplicated = retry_fd(|| unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) })?;
        Ok(unsafe { OwnedFd::from_raw_fd(duplicated) })
    }

    fn identity_from_stat(stat: &FileStat) -> Identity {
        Identity {
            device: stat.device,
            inode: stat.inode,
        }
    }

    fn is_directory(mode: libc::mode_t) -> bool {
        mode & libc::S_IFMT == libc::S_IFDIR
    }

    fn is_regular(mode: libc::mode_t) -> bool {
        mode & libc::S_IFMT == libc::S_IFREG
    }

    fn component(name: &str) -> Result<CString, SecureError> {
        if name.is_empty() || name == "." || name == ".." || name.as_bytes().contains(&b'/') {
            return Err(SecureError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "filesystem name must be one non-special path component",
            )));
        }
        CString::new(name)
            .map_err(|error| SecureError::Io(io::Error::new(io::ErrorKind::InvalidInput, error)))
    }

    fn c_string(value: &OsStr) -> io::Result<CString> {
        CString::new(value.as_bytes())
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
    }

    fn wrong_type(message: &'static str) -> SecureError {
        SecureError::SymlinkOrWrongType(io::Error::other(message))
    }

    fn map_open_error(error: io::Error) -> SecureError {
        match error.raw_os_error() {
            Some(libc::ENOENT) => SecureError::NotFound,
            Some(libc::EEXIST) => SecureError::AlreadyExists,
            Some(libc::ELOOP) | Some(libc::ENOTDIR) | Some(libc::EISDIR) => {
                SecureError::SymlinkOrWrongType(error)
            }
            _ => SecureError::Io(error),
        }
    }

    fn map_publish_error(error: io::Error) -> SecureError {
        match error.raw_os_error() {
            Some(libc::EEXIST) => SecureError::AlreadyExists,
            Some(libc::EXDEV) | Some(libc::ENOTSUP) => SecureError::Unsupported,
            _ => map_open_error(error),
        }
    }

    #[cfg(target_vendor = "apple")]
    fn set_errno_zero() {
        unsafe {
            *libc::__error() = 0;
        }
    }

    #[cfg(target_vendor = "apple")]
    fn errno_value() -> i32 {
        unsafe { *libc::__error() }
    }

    #[cfg(target_os = "linux")]
    fn set_errno_zero() {
        unsafe {
            *libc::__errno_location() = 0;
        }
    }

    #[cfg(target_os = "linux")]
    fn errno_value() -> i32 {
        unsafe { *libc::__errno_location() }
    }

    fn retry_fd(mut operation: impl FnMut() -> libc::c_int) -> io::Result<RawFd> {
        loop {
            let result = operation();
            if result >= 0 {
                return Ok(result);
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }

    fn retry_zero(mut operation: impl FnMut() -> libc::c_int) -> io::Result<()> {
        loop {
            if operation() == 0 {
                return Ok(());
            }
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }
}

#[cfg(any(target_vendor = "apple", target_os = "linux"))]
pub(crate) use unix::*;

#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
mod unsupported {
    use std::fmt;
    use std::io;
    use std::path::Path;

    #[derive(Debug)]
    pub(crate) enum SecureError {
        NotFound,
        AlreadyExists,
        SymlinkOrWrongType(io::Error),
        IdentityMismatch {
            expected: Identity,
            actual: Identity,
        },
        TooManyEntries {
            max_entries: usize,
        },
        Io(io::Error),
        Unsupported,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct Identity;

    impl Identity {
        pub(crate) const fn device(self) -> u64 {
            0
        }

        pub(crate) const fn inode(self) -> u64 {
            0
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum TempInitFault {
        Tmpfile,
        Fchmod,
        Fstat,
        Dup,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) enum EntryKind {
        Directory,
        Regular,
        Other,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(crate) struct EntryMetadata {
        pub(crate) kind: EntryKind,
        pub(crate) identity: Identity,
        pub(crate) size: u64,
        pub(crate) links: u64,
    }

    impl fmt::Display for Identity {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("unsupported")
        }
    }

    #[derive(Debug)]
    pub(crate) struct Dir;

    #[derive(Debug)]
    pub(crate) struct RegularFile;

    #[derive(Debug)]
    pub(crate) struct TempEntryGuard;

    impl RegularFile {
        pub(crate) fn identity(&self) -> Identity {
            Identity
        }
        pub(crate) fn size(&self) -> u64 {
            0
        }
        pub(crate) fn link_count(&self) -> u64 {
            0
        }
        pub(crate) fn write_append_line(&self, _bytes: &[u8]) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
        pub(crate) fn read_bounded(&self, _max_bytes: usize) -> io::Result<Vec<u8>> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
        pub(crate) fn sync_data(&self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
        pub(crate) fn sync_all(&self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
        pub(crate) fn last_byte(&self) -> io::Result<Option<u8>> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
    }

    impl TempEntryGuard {
        pub(crate) fn device_id(&self) -> u64 {
            0
        }

        pub(crate) fn write_all(&mut self, _bytes: &[u8]) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
        pub(crate) fn sync_all(&self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
        }
        pub(crate) fn publish_immutable(
            &self,
            _destination_directory: &Dir,
            _destination: &str,
        ) -> Result<RegularFile, SecureError> {
            Err(SecureError::Unsupported)
        }
    }

    impl Dir {
        pub(crate) fn identity(&self) -> Result<Identity, SecureError> {
            Err(SecureError::Unsupported)
        }

        pub(crate) fn device_id(&self) -> Result<u64, SecureError> {
            Err(SecureError::Unsupported)
        }

        pub(crate) fn open_root(_path: &Path) -> Result<Self, SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn open_dir(&self, _name: &str) -> Result<Self, SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn entry_metadata(&self, _name: &str) -> Result<EntryMetadata, SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn ensure_dir(&self, _name: &str) -> Result<(Self, bool), SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn open_regular_optional(
            &self,
            _name: &str,
        ) -> Result<Option<RegularFile>, SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn create_temp(
            &self,
            _purpose: &str,
            _injected_fault: Option<TempInitFault>,
        ) -> Result<TempEntryGuard, SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn open_append_create(
            &self,
            _name: &str,
        ) -> Result<(RegularFile, bool), SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn sync(&self) -> Result<(), SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn verify_entry_identity(
            &self,
            _name: &str,
            _expected: Identity,
        ) -> Result<(), SecureError> {
            Err(SecureError::Unsupported)
        }
        pub(crate) fn read_entry_names(
            &self,
            _max_entries: usize,
        ) -> Result<Vec<String>, SecureError> {
            Err(SecureError::Unsupported)
        }
        #[cfg(test)]
        pub(crate) fn replace_entry_with_symlink(
            &self,
            _name: &str,
            _target: &Path,
        ) -> Result<(), SecureError> {
            Err(SecureError::Unsupported)
        }
    }
}

#[cfg(not(any(target_vendor = "apple", target_os = "linux")))]
pub(crate) use unsupported::*;

#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    use std::ffi::CString;

    use super::unix::{publish_from_fd, SecureError};

    #[test]
    fn immutable_fd_publication_is_explicitly_unsupported() {
        let destination = CString::new("destination").unwrap();
        assert!(matches!(
            publish_from_fd(-1, -1, &destination),
            Err(SecureError::Unsupported)
        ));
    }
}
