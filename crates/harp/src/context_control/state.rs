#[cfg(any(target_os = "macos", target_os = "linux"))]
mod secure {
    use std::collections::{BTreeMap, BTreeSet};
    use std::env;
    use std::ffi::{CStr, CString, OsStr};
    use std::fs::{File, Metadata};
    use std::io::{Read, Write};
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Component, Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    #[cfg(test)]
    use std::cell::RefCell;

    use sha2::{Digest as _, Sha256};

    use crate::AppError;

    const DIRECTORY_MODE: u32 = 0o700;
    const FILE_MODE: u32 = 0o600;
    const INTERNAL_NAMESPACE: &str = ".harp-internal";
    const RELEASE_AUTHORIZATIONS: &str = "release-authorizations";
    const RELEASE_AUTHORIZATION_FILE: &str = "authorization.json";
    const STAGING_NAMESPACE: &str = "staging";
    const LOCK_WAIT_LIMIT: Duration = Duration::from_secs(10);
    const LOCK_RETRY_DELAY: Duration = Duration::from_millis(5);

    static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(1);
    #[cfg(test)]
    thread_local! {
        static SYNC_TRACE: RefCell<Vec<SyncEvent>> = const { RefCell::new(Vec::new()) };
        static CREATION_FAULT: RefCell<Option<(String, CreationFault)>> = const {
            RefCell::new(None)
        };
    }

    #[cfg(test)]
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum SyncEvent {
        Directory,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum CreationFault {
        FileChmod,
        FileWrite,
        FileSync,
        FileMetadata,
        DirectoryPostMkdir,
    }

    #[cfg(test)]
    pub(super) fn with_creation_fault<T>(
        prefix: &str,
        fault: CreationFault,
        operation: impl FnOnce() -> T,
    ) -> T {
        CREATION_FAULT.with(|configured| {
            let previous = configured.replace(Some((prefix.to_owned(), fault)));
            struct RestoreFault<'a> {
                configured: &'a RefCell<Option<(String, CreationFault)>>,
                previous: Option<(String, CreationFault)>,
            }
            impl Drop for RestoreFault<'_> {
                fn drop(&mut self) {
                    self.configured.replace(self.previous.take());
                }
            }
            let _restore = RestoreFault {
                configured,
                previous,
            };
            operation()
        })
    }

    fn inject_creation_fault(name: &CStr, fault: CreationFault) -> Result<(), AppError> {
        #[cfg(test)]
        {
            let injected = CREATION_FAULT.with(|configured| {
                let matches =
                    configured
                        .borrow()
                        .as_ref()
                        .is_some_and(|(prefix, configured_fault)| {
                            name.to_bytes().starts_with(prefix.as_bytes())
                                && *configured_fault == fault
                        });
                if matches {
                    configured.borrow_mut().take();
                }
                matches
            });
            if injected {
                return Err(state_io(
                    "injected creation failure",
                    std::io::Error::other(format!("{fault:?}")),
                ));
            }
        }
        #[cfg(not(test))]
        let _ = (name, fault);
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn take_sync_trace() -> Vec<SyncEvent> {
        SYNC_TRACE.with(|trace| std::mem::take(&mut *trace.borrow_mut()))
    }

    fn record_directory_sync() {
        #[cfg(test)]
        SYNC_TRACE.with(|trace| trace.borrow_mut().push(SyncEvent::Directory));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct FileIdentity {
        device: u64,
        inode: u64,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct FileVersion {
        identity: FileIdentity,
        length: u64,
        modified_seconds: i64,
        modified_nanoseconds: i64,
        changed_seconds: i64,
        changed_nanoseconds: i64,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct FileSnapshot {
        version: FileVersion,
        digest: [u8; 32],
    }

    impl FileSnapshot {
        fn matches_exchanged_file(&self, other: &Self) -> bool {
            self.version.identity == other.version.identity
                && self.version.length == other.version.length
                && self.digest == other.digest
        }
    }

    #[derive(Clone, Debug)]
    pub enum ReplacePolicy {
        CreateOnly,
        /// Serializes with all Harp writers and replaces only after the snapshot still matches.
        ///
        /// `StateRoot` rechecks the target identity before one atomic rename. Same-UID code that
        /// ignores Harp's publication lock can still mutate the pathname in the final syscall
        /// interval; such non-Harp mutations are outside this guarantee.
        CompareAndReplace(FileSnapshot),
    }

    #[derive(Debug)]
    pub struct StateRoot {
        root: PathBuf,
        directory: Directory,
    }

    impl StateRoot {
        pub fn open_from_environment() -> Result<Self, AppError> {
            let explicit = nonempty_environment_path("HARP_HOME");
            let xdg = nonempty_environment_path("XDG_STATE_HOME");
            let root = if let Some(explicit) = explicit {
                explicit
            } else if let Some(xdg) = xdg {
                xdg.join("harp")
            } else {
                let home = nonempty_environment_path("HOME").ok_or_else(|| {
                    state_error(
                        "state.path",
                        "HOME is required when HARP_HOME and XDG_STATE_HOME are unset",
                    )
                })?;
                home.join(".local/state/harp")
            };
            Self::open_or_create(&root)
        }

        pub fn open_or_create(path: &Path) -> Result<Self, AppError> {
            let root = absolute_root_path(path)?;
            let directory = open_or_create_absolute_directory(&root)?;
            Ok(Self { root, directory })
        }

        pub fn create_private_directory(&self, relative: &Path) -> Result<PathBuf, AppError> {
            let components = public_relative_components(relative, "private directory")?;
            let directory = self.directory.walk_or_create(&components)?;
            directory.verify_namespace()?;
            Ok(self.root.join(relative))
        }

        pub fn snapshot_private_file(&self, relative: &Path) -> Result<FileSnapshot, AppError> {
            let components = public_relative_components(relative, "private file")?;
            let (parent, name) = self.resolve_parent_components(&components, false)?;
            parent.verify_namespace()?;
            let snapshot = parent.read_snapshot(&name).map(|(_, snapshot)| snapshot)?;
            parent.verify_namespace()?;
            Ok(snapshot)
        }

        pub fn list_private_directory(&self, relative: &Path) -> Result<Vec<String>, AppError> {
            let components = public_relative_components(relative, "private directory")?;
            let directory = self.directory.walk(&components)?;
            directory.verify_namespace()?;
            let names = directory
                .entry_names()?
                .into_iter()
                .map(|name| {
                    name.into_string().map_err(|_| {
                        state_error("state.path", "private directory entry is not UTF-8")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            directory.verify_namespace()?;
            Ok(names)
        }

        pub fn read_private_file_bounded(
            &self,
            relative: &Path,
            max_bytes: usize,
        ) -> Result<Vec<u8>, AppError> {
            self.read_private_file_bounded_with_hooks(relative, max_bytes, || {})
        }

        #[cfg(test)]
        pub(super) fn read_private_file_bounded_with_hook<F>(
            &self,
            relative: &Path,
            max_bytes: usize,
            hook: F,
        ) -> Result<Vec<u8>, AppError>
        where
            F: FnOnce(),
        {
            self.read_private_file_bounded_with_hooks(relative, max_bytes, hook)
        }

        fn read_private_file_bounded_with_hooks<F>(
            &self,
            relative: &Path,
            max_bytes: usize,
            hook: F,
        ) -> Result<Vec<u8>, AppError>
        where
            F: FnOnce(),
        {
            let components = public_relative_components(relative, "private file")?;
            let (parent, name) = self.resolve_parent_components(&components, false)?;
            parent.verify_namespace()?;
            let bytes = parent.read_file_bounded_with_hook(&name, max_bytes, hook)?;
            parent.verify_namespace()?;
            Ok(bytes)
        }

        pub fn write_private_atomic(
            &self,
            relative: &Path,
            bytes: &[u8],
            replace: ReplacePolicy,
        ) -> Result<(), AppError> {
            self.write_private_atomic_with_hooks(relative, bytes, replace, || {}, || {})
        }

        pub fn publish_immutable_tree(
            &self,
            relative: &Path,
            members: &BTreeMap<PathBuf, Vec<u8>>,
        ) -> Result<(), AppError> {
            let target_components = public_relative_components(relative, "immutable tree")?;
            let normalized_members = validate_tree_members(members)?;
            let mut publication_lock =
                self.acquire_publication_lock_components(&target_components, LockSetupFault::None)?;
            let result = self.publish_immutable_tree_locked(
                &target_components,
                members,
                &normalized_members,
            );
            finish_publication(result, &mut publication_lock)
        }

        pub(crate) fn publish_release_authorization(
            &self,
            release_id: &str,
            bytes: &[u8],
        ) -> Result<(), AppError> {
            let target_components = release_authorization_components(release_id)?;
            let members =
                BTreeMap::from([(PathBuf::from(RELEASE_AUTHORIZATION_FILE), bytes.to_vec())]);
            let normalized_members = validate_tree_members(&members)?;
            let mut publication_lock =
                self.acquire_publication_lock_components(&target_components, LockSetupFault::None)?;
            let result = self.publish_immutable_tree_locked(
                &target_components,
                &members,
                &normalized_members,
            );
            finish_publication(result, &mut publication_lock)
        }

        pub(crate) fn read_release_authorization(
            &self,
            release_id: &str,
            max_bytes: usize,
        ) -> Result<Vec<u8>, AppError> {
            let mut components = release_authorization_components(release_id)?;
            components.push(cstring(RELEASE_AUTHORIZATION_FILE)?);
            let (parent, name) = self.resolve_parent_components(&components, false)?;
            parent.verify_namespace()?;
            let bytes = parent.read_file_bounded_with_hook(&name, max_bytes, || {})?;
            parent.verify_namespace()?;
            Ok(bytes)
        }

        #[cfg(test)]
        pub(super) fn write_private_atomic_with_hook<F>(
            &self,
            relative: &Path,
            bytes: &[u8],
            replace: ReplacePolicy,
            hook: F,
        ) -> Result<(), AppError>
        where
            F: FnOnce(),
        {
            self.write_private_atomic_with_hooks(relative, bytes, replace, hook, || {})
        }

        #[cfg(test)]
        pub(super) fn write_private_atomic_with_mutation_hook<F>(
            &self,
            relative: &Path,
            bytes: &[u8],
            replace: ReplacePolicy,
            mutation_hook: F,
        ) -> Result<(), AppError>
        where
            F: FnOnce(),
        {
            self.write_private_atomic_with_hooks(relative, bytes, replace, || {}, mutation_hook)
        }

        fn write_private_atomic_with_hooks<F, G>(
            &self,
            relative: &Path,
            bytes: &[u8],
            replace: ReplacePolicy,
            prewrite_hook: F,
            mutation_hook: G,
        ) -> Result<(), AppError>
        where
            F: FnOnce(),
            G: FnOnce(),
        {
            let components = public_relative_components(relative, "private file")?;
            let mut publication_lock =
                self.acquire_publication_lock_components(&components, LockSetupFault::None)?;
            let result = self.write_private_atomic_locked(
                &components,
                bytes,
                &replace,
                prewrite_hook,
                mutation_hook,
            );
            finish_publication(result, &mut publication_lock)
        }

        fn write_private_atomic_locked<F, G>(
            &self,
            components: &[CString],
            bytes: &[u8],
            replace: &ReplacePolicy,
            prewrite_hook: F,
            mutation_hook: G,
        ) -> Result<(), AppError>
        where
            F: FnOnce(),
            G: FnOnce(),
        {
            let (parent, name) = self.resolve_parent_components(components, true)?;
            let initial = parent.read_optional_snapshot(&name)?;
            validate_replace_policy(replace, initial.as_ref(), &name)?;
            prewrite_hook();
            parent.verify_namespace()?;

            let mut temporary = parent.create_temporary_file(".harp-file", bytes)?;
            let current = parent.read_optional_snapshot(&name)?;
            validate_unchanged(replace, initial.as_ref(), current.as_ref(), &name)?;
            parent.verify_entry_matches(&temporary.name, temporary.identity, EntryKind::File)?;
            parent.verify_namespace()?;

            match replace {
                ReplacePolicy::CreateOnly => {
                    parent.publish_new_file(&mut temporary, &name)?;
                    mutation_hook();
                }
                ReplacePolicy::CompareAndReplace(_) => {
                    let expected = initial.ok_or_else(|| {
                        state_error(
                            "state.conflict",
                            "private file disappeared before replacement",
                        )
                    })?;
                    parent.publish_replacement(&mut temporary, &name, &expected)?;
                    mutation_hook();
                    parent.verify_published_file(&name, &temporary)?;
                }
            }
            parent.sync()?;
            parent.verify_namespace()?;
            Ok(())
        }

        fn publish_immutable_tree_locked(
            &self,
            components: &[CString],
            members: &BTreeMap<PathBuf, Vec<u8>>,
            expected_directories: &BTreeSet<PathBuf>,
        ) -> Result<(), AppError> {
            let (parent, name) = self.resolve_parent_components(components, true)?;
            parent.verify_namespace()?;
            if let Some(existing) = parent.open_optional_directory(&name)? {
                if immutable_tree_matches(&existing, members, expected_directories)? {
                    parent.verify_namespace()?;
                    return Ok(());
                }
                return Err(state_error(
                    "state.immutable_collision",
                    format!(
                        "immutable tree conflicts with existing bytes: {}",
                        display_name(&name)
                    ),
                ));
            }

            let staging_parent = self
                .directory
                .walk_or_create(&[cstring(INTERNAL_NAMESPACE)?, cstring(STAGING_NAMESPACE)?])?;
            let mut staging = staging_parent.create_staged_directory(".harp-tree")?;
            for (member, bytes) in members {
                let member_components =
                    canonical_relative_components(member, "immutable-tree member")?;
                let (member_parent, member_name) = staging
                    .directory
                    .resolve_parent_components(&member_components, true)?;
                member_parent.create_file(&member_name, bytes)?;
            }
            sync_tree_directories(&staging.directory)?;

            staging_parent.verify_namespace()?;
            staging_parent.verify_entry_matches(
                &staging.name,
                staging.identity,
                EntryKind::Directory,
            )?;
            parent.verify_namespace()?;
            match renameat_noreplace(
                staging_parent.fd.as_raw_fd(),
                &staging.name,
                parent.fd.as_raw_fd(),
                &name,
            ) {
                Ok(()) => {
                    staging.disarm();
                    parent.verify_entry_matches(&name, staging.identity, EntryKind::Directory)?;
                    parent.sync()?;
                    staging_parent.sync()?;
                    parent.verify_namespace()?;
                    Ok(())
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    match parent.open_optional_directory(&name)? {
                        Some(existing)
                            if immutable_tree_matches(
                                &existing,
                                members,
                                expected_directories,
                            )? =>
                        {
                            parent.verify_namespace()?;
                            Ok(())
                        }
                        Some(_) => Err(state_error(
                            "state.immutable_collision",
                            format!(
                                "immutable tree conflicts with concurrent publication: {}",
                                display_name(&name)
                            ),
                        )),
                        None => Err(state_error(
                            "state.conflict",
                            "immutable-tree target changed during publication",
                        )),
                    }
                }
                Err(error) => Err(state_io("publish immutable tree", error)),
            }
        }

        fn resolve_parent_components(
            &self,
            components: &[CString],
            create: bool,
        ) -> Result<(Directory, CString), AppError> {
            let (name, parents) = components
                .split_last()
                .ok_or_else(|| state_path("target"))?;
            let parent = if create {
                self.directory.walk_or_create(parents)?
            } else {
                self.directory.walk(parents)?
            };
            Ok((parent, name.clone()))
        }

        #[cfg(test)]
        pub(super) fn acquire_publication_lock(
            &self,
            relative: &Path,
        ) -> Result<PublicationLock, AppError> {
            let components = canonical_relative_components(relative, "publication target")?;
            self.acquire_publication_lock_components(&components, LockSetupFault::None)
        }

        #[cfg(test)]
        pub(super) fn acquire_publication_lock_with_fault(
            &self,
            relative: &Path,
            fault: LockSetupFault,
        ) -> Result<PublicationLock, AppError> {
            let components = canonical_relative_components(relative, "publication target")?;
            self.acquire_publication_lock_components(&components, fault)
        }

        #[cfg(test)]
        pub(super) fn recover_dead_publication_lock_with_hook<F>(
            &self,
            relative: &Path,
            hook: F,
        ) -> Result<(), AppError>
        where
            F: FnOnce(),
        {
            let components = canonical_relative_components(relative, "publication target")?;
            let locks = self.directory.walk_or_create(&[cstring(".locks")?])?;
            let lock_name = lock_name(&components);
            match inspect_lock(&locks, &lock_name)? {
                LockState::Dead {
                    directory,
                    identity,
                    holder,
                    holder_snapshot,
                } => {
                    hook();
                    recover_dead_lock(
                        &locks,
                        &lock_name,
                        &directory,
                        identity,
                        holder,
                        &holder_snapshot,
                    )
                }
                LockState::Gone | LockState::Live => Err(state_error(
                    "state.lock",
                    "expected a dead publication lock",
                )),
            }
        }

        fn acquire_publication_lock_components(
            &self,
            components: &[CString],
            fault: LockSetupFault,
        ) -> Result<PublicationLock, AppError> {
            let locks = self.directory.walk_or_create(&[cstring(".locks")?])?;
            let lock_name = lock_name(components);
            let started = Instant::now();
            loop {
                match self.stage_and_publish_lock(&locks, &lock_name, fault) {
                    Ok(lock) => return Ok(lock),
                    Err(error) if error.code() == "state.lock_exists" => {
                        match inspect_lock(&locks, &lock_name)? {
                            LockState::Gone => {}
                            LockState::Live => {
                                if started.elapsed() >= LOCK_WAIT_LIMIT {
                                    return Err(state_error(
                                        "state.lock",
                                        format!(
                                            "publication lock remained held: {}",
                                            display_name(&lock_name)
                                        ),
                                    ));
                                }
                                thread::sleep(LOCK_RETRY_DELAY);
                            }
                            LockState::Dead {
                                directory,
                                identity,
                                holder,
                                holder_snapshot,
                            } => {
                                recover_dead_lock(
                                    &locks,
                                    &lock_name,
                                    &directory,
                                    identity,
                                    holder,
                                    &holder_snapshot,
                                )?;
                            }
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
        }

        fn stage_and_publish_lock(
            &self,
            locks: &Directory,
            lock_name: &CStr,
            fault: LockSetupFault,
        ) -> Result<PublicationLock, AppError> {
            let mut staged = locks.create_staged_directory(".harp-lock")?;
            if fault == LockSetupFault::AfterStage {
                return Err(state_error("state.lock", "injected lock setup failure"));
            }
            let holder = HolderIdentity::current()?;
            let holder_snapshot = staged
                .directory
                .create_file(&cstring("holder")?, &holder.encode())?;
            staged
                .directory
                .verify_snapshot(&cstring("holder")?, &holder_snapshot)?;
            staged.directory.sync()?;
            locks.verify_namespace()?;
            locks.verify_entry_matches(&staged.name, staged.identity, EntryKind::Directory)?;
            match locks.publish_staged_directory_inner(&staged.name, lock_name) {
                Ok(()) => {
                    staged.name = lock_name.to_owned();
                    staged.cleanup_holder = Some((holder, holder_snapshot.clone()));
                    locks.sync()?;
                    if fault == LockSetupFault::AfterPublish {
                        return Err(state_error("state.lock", "injected lock setup failure"));
                    }
                    let directory = locks.open_directory(lock_name)?;
                    let identity = directory.identity()?;
                    if identity != staged.identity {
                        return Err(state_error(
                            "state.lock",
                            "published lock identity changed during setup",
                        ));
                    }
                    let (holder_bytes, published_holder_snapshot) =
                        directory.read_snapshot(&cstring("holder")?)?;
                    if published_holder_snapshot != holder_snapshot
                        || HolderIdentity::decode(&holder_bytes)? != holder
                    {
                        return Err(state_error(
                            "state.lock",
                            "published lock holder changed during setup",
                        ));
                    }
                    let parent = locks.duplicate()?;
                    staged.disarm();
                    Ok(PublicationLock {
                        parent,
                        name: lock_name.to_owned(),
                        directory,
                        identity,
                        holder,
                        holder_snapshot: published_holder_snapshot,
                        released: false,
                    })
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    Err(state_error("state.lock_exists", "publication lock exists"))
                }
                Err(error) => Err(state_io("publish staged lock", error)),
            }
        }

        #[cfg(test)]
        pub(super) fn lock_path(&self, relative: &Path) -> Result<PathBuf, AppError> {
            let components = canonical_relative_components(relative, "publication target")?;
            Ok(self
                .root
                .join(".locks")
                .join(OsStr::from_bytes(lock_name(&components).to_bytes())))
        }

        #[cfg(test)]
        pub(super) fn root_directory(&self) -> Result<Directory, AppError> {
            self.directory.duplicate()
        }

        #[cfg(test)]
        pub(super) fn inspect_publication_lock_for_test(
            &self,
            relative: &Path,
        ) -> Result<(), AppError> {
            let components = canonical_relative_components(relative, "publication target")?;
            let locks = self.directory.walk_or_create(&[cstring(".locks")?])?;
            let lock_name = lock_name(&components);
            match inspect_lock(&locks, &lock_name)? {
                LockState::Gone | LockState::Live | LockState::Dead { .. } => Ok(()),
            }
        }
    }

    pub fn resolve_state_root(
        explicit: Option<&Path>,
        xdg_state_home: Option<&Path>,
        home: &Path,
    ) -> PathBuf {
        explicit
            .map(Path::to_path_buf)
            .or_else(|| xdg_state_home.map(|path| path.join("harp")))
            .unwrap_or_else(|| home.join(".local/state/harp"))
    }

    #[derive(Debug)]
    pub(super) struct Directory {
        fd: OwnedFd,
        anchor: Option<std::sync::Arc<NamespaceAnchor>>,
        route: Vec<AnchoredComponent>,
    }

    #[derive(Clone, Debug)]
    struct AnchoredComponent {
        name: CString,
        identity: FileIdentity,
    }

    #[derive(Debug)]
    struct NamespaceAnchor {
        root_fd: OwnedFd,
        root_identity: FileIdentity,
        absolute_route: Vec<AnchoredComponent>,
    }

    impl NamespaceAnchor {
        fn verify(&self) -> Result<(), AppError> {
            let filesystem_root = open_filesystem_root()?;
            let mut current = filesystem_root;
            for component in &self.absolute_route {
                current = open_namespace_component(&current, &component.name)?;
                if current.raw_identity()? != component.identity {
                    return Err(state_error(
                        "state.conflict",
                        "state root path identity changed after it was opened",
                    ));
                }
            }
            if current.raw_identity()? != self.root_identity {
                return Err(state_error(
                    "state.conflict",
                    "state root identity changed after it was opened",
                ));
            }
            Ok(())
        }

        fn duplicate_root(&self) -> Result<Directory, AppError> {
            Ok(Directory {
                fd: duplicate_fd(self.root_fd.as_raw_fd())?,
                anchor: None,
                route: Vec::new(),
            })
        }
    }

    impl Directory {
        fn duplicate(&self) -> Result<Self, AppError> {
            Ok(Self {
                fd: duplicate_fd(self.fd.as_raw_fd())?,
                anchor: self.anchor.clone(),
                route: self.route.clone(),
            })
        }

        fn raw_identity(&self) -> Result<FileIdentity, AppError> {
            File::from(duplicate_fd(self.fd.as_raw_fd())?)
                .metadata()
                .map(|metadata| file_identity(&metadata))
                .map_err(|error| state_io("inspect directory descriptor identity", error))
        }

        fn identity(&self) -> Result<FileIdentity, AppError> {
            let metadata = File::from(self.duplicate()?.fd)
                .metadata()
                .map_err(|error| state_io("inspect directory descriptor", error))?;
            validate_metadata(&metadata, EntryKind::Directory, "private directory")?;
            Ok(file_identity(&metadata))
        }

        fn verify_namespace(&self) -> Result<(), AppError> {
            let Some(anchor) = &self.anchor else {
                return Ok(());
            };
            anchor.verify()?;
            let mut current = anchor.duplicate_root()?;
            for component in &self.route {
                current = open_namespace_component(&current, &component.name)?;
                if current.raw_identity()? != component.identity {
                    return Err(state_error(
                        "state.conflict",
                        format!(
                            "private path component identity changed: {}",
                            display_name(&component.name)
                        ),
                    ));
                }
            }
            let expected = self
                .route
                .last()
                .map(|component| component.identity)
                .unwrap_or(anchor.root_identity);
            if self.raw_identity()? != expected {
                return Err(state_error(
                    "state.conflict",
                    "retained private directory identity changed",
                ));
            }
            anchor.verify()
        }

        fn sync(&self) -> Result<(), AppError> {
            self.verify_namespace()?;
            if unsafe { libc::fsync(self.fd.as_raw_fd()) } != 0 {
                return Err(state_io(
                    "synchronize private directory",
                    std::io::Error::last_os_error(),
                ));
            }
            record_directory_sync();
            self.verify_namespace()
        }

        fn walk(&self, components: &[CString]) -> Result<Self, AppError> {
            let mut current = self.duplicate()?;
            for component in components {
                current = current.open_directory(component)?;
            }
            Ok(current)
        }

        fn walk_or_create(&self, components: &[CString]) -> Result<Self, AppError> {
            let mut current = self.duplicate()?;
            for component in components {
                current = current.open_or_create_directory(component)?;
            }
            Ok(current)
        }

        fn resolve_parent_components(
            &self,
            components: &[CString],
            create: bool,
        ) -> Result<(Self, CString), AppError> {
            let (name, parents) = components
                .split_last()
                .ok_or_else(|| state_path("target"))?;
            let parent = if create {
                self.walk_or_create(parents)?
            } else {
                self.walk(parents)?
            };
            Ok((parent, name.clone()))
        }

        fn open_or_create_directory(&self, name: &CStr) -> Result<Self, AppError> {
            match self.open_directory(name) {
                Ok(directory) => Ok(directory),
                Err(error) if error.code() == "state.missing" => {
                    self.verify_namespace()?;
                    let mut staged = self.create_staged_directory(".harp-directory")?;
                    self.verify_namespace()?;
                    match self.publish_staged_directory_inner(&staged.name, name) {
                        Ok(()) => {
                            self.verify_entry_matches(name, staged.identity, EntryKind::Directory)?;
                            let directory = self.open_directory(name)?;
                            if directory.raw_identity()? != staged.identity {
                                return Err(state_error(
                                    "state.conflict",
                                    "created directory identity changed during publication",
                                ));
                            }
                            staged.disarm();
                            self.sync()?;
                            directory.verify_namespace()?;
                            Ok(directory)
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                            self.open_directory(name)
                        }
                        Err(error) => {
                            Err(classify_component_error(self.fd.as_raw_fd(), name, error))
                        }
                    }
                }
                Err(error) => Err(error),
            }
        }

        fn open_directory(&self, name: &CStr) -> Result<Self, AppError> {
            let directory = self.open_directory_unchecked(name)?;
            directory.verify_mode(DIRECTORY_MODE)?;
            directory.verify_owner()?;
            Ok(directory)
        }

        fn open_directory_unchecked(&self, name: &CStr) -> Result<Self, AppError> {
            let fd = unsafe {
                libc::openat(
                    self.fd.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                let error = std::io::Error::last_os_error();
                return Err(classify_open_error(self.fd.as_raw_fd(), name, error));
            }
            let mut route = self.route.clone();
            let directory = Self {
                fd: unsafe { OwnedFd::from_raw_fd(fd) },
                anchor: self.anchor.clone(),
                route: Vec::new(),
            };
            route.push(AnchoredComponent {
                name: name.to_owned(),
                identity: directory.raw_identity()?,
            });
            Ok(Self { route, ..directory })
        }

        fn open_optional_directory(&self, name: &CStr) -> Result<Option<Self>, AppError> {
            match self.open_directory(name) {
                Ok(directory) => Ok(Some(directory)),
                Err(error) if error.code() == "state.missing" => Ok(None),
                Err(error) => Err(error),
            }
        }

        pub(super) fn create_file(
            &self,
            name: &CStr,
            bytes: &[u8],
        ) -> Result<FileSnapshot, AppError> {
            let mut created = self.create_file_entry(name, bytes)?;
            let snapshot = created.snapshot.clone().ok_or_else(|| {
                state_error("state.conflict", "created private file has no snapshot")
            })?;
            created.disarm();
            Ok(snapshot)
        }

        fn create_file_entry(&self, name: &CStr, bytes: &[u8]) -> Result<TemporaryEntry, AppError> {
            self.verify_namespace()?;
            let cleanup_parent = self.duplicate()?;
            let cleanup_name = name.to_owned();
            let fd = unsafe {
                libc::openat(
                    self.fd.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    FILE_MODE as libc::c_uint,
                )
            };
            if fd < 0 {
                return Err(classify_create_error(
                    self.fd.as_raw_fd(),
                    name,
                    std::io::Error::last_os_error(),
                ));
            }
            let file = unsafe { File::from_raw_fd(fd) };
            let mut cleanup = CreatedFileGuard {
                parent: cleanup_parent,
                name: cleanup_name,
                file: Some(file),
                identity: None,
                active: true,
            };
            self.verify_namespace()?;
            inject_creation_fault(name, CreationFault::FileChmod)?;
            if unsafe { libc::fchmod(cleanup.file().as_raw_fd(), FILE_MODE as libc::mode_t) } != 0 {
                return Err(state_io(
                    "set private file mode",
                    std::io::Error::last_os_error(),
                ));
            }
            self.verify_namespace()?;
            let initial_metadata = cleanup
                .file()
                .metadata()
                .map_err(|error| state_io("inspect private file before writing", error))?;
            let initial_identity = file_identity(&initial_metadata);
            cleanup.bind_identity(initial_identity);
            self.verify_entry_matches(name, initial_identity, EntryKind::File)?;
            inject_creation_fault(name, CreationFault::FileWrite)?;
            cleanup
                .file_mut()
                .write_all(bytes)
                .map_err(|error| state_io("write private file", error))?;
            inject_creation_fault(name, CreationFault::FileSync)?;
            cleanup
                .file()
                .sync_all()
                .map_err(|error| state_io("synchronize private file", error))?;
            self.verify_namespace()?;
            inject_creation_fault(name, CreationFault::FileMetadata)?;
            let metadata = cleanup
                .file()
                .metadata()
                .map_err(|error| state_io("inspect private file", error))?;
            validate_metadata(&metadata, EntryKind::File, "private file")?;
            if metadata.permissions().mode() & 0o777 != FILE_MODE {
                return Err(state_error(
                    "state.permissions",
                    "private file does not have mode 0600",
                ));
            }
            let snapshot = FileSnapshot {
                version: file_version(&metadata),
                digest: Sha256::digest(bytes).into(),
            };
            self.verify_entry_matches(name, snapshot.version.identity, EntryKind::File)?;
            self.sync()?;
            self.verify_namespace()?;
            cleanup.into_temporary(snapshot)
        }

        fn read_snapshot(&self, name: &CStr) -> Result<(Vec<u8>, FileSnapshot), AppError> {
            let fd = unsafe {
                libc::openat(
                    self.fd.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err(classify_open_error(
                    self.fd.as_raw_fd(),
                    name,
                    std::io::Error::last_os_error(),
                ));
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            let before = file
                .metadata()
                .map_err(|error| state_io("inspect opened private file", error))?;
            validate_metadata(&before, EntryKind::File, "private file")?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .map_err(|error| state_io("read private file", error))?;
            let after = file
                .metadata()
                .map_err(|error| state_io("reinspect private file", error))?;
            let before_version = file_version(&before);
            if before_version != file_version(&after) || bytes.len() as u64 != after.len() {
                return Err(state_error(
                    "state.conflict",
                    "private file changed while it was read",
                ));
            }
            Ok((
                bytes.clone(),
                FileSnapshot {
                    version: before_version,
                    digest: Sha256::digest(bytes).into(),
                },
            ))
        }

        fn read_file_bounded_with_hook<F>(
            &self,
            name: &CStr,
            max_bytes: usize,
            hook: F,
        ) -> Result<Vec<u8>, AppError>
        where
            F: FnOnce(),
        {
            self.verify_namespace()?;
            let fd = unsafe {
                libc::openat(
                    self.fd.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err(classify_open_error(
                    self.fd.as_raw_fd(),
                    name,
                    std::io::Error::last_os_error(),
                ));
            }
            let mut file = unsafe { File::from_raw_fd(fd) };
            let before = file
                .metadata()
                .map_err(|error| state_io("inspect opened private file", error))?;
            validate_metadata(&before, EntryKind::File, "private file")?;
            if before.len() > max_bytes as u64 {
                return Err(state_error(
                    "state.size",
                    format!("private file exceeds {max_bytes} bytes"),
                ));
            }
            let before_version = file_version(&before);
            hook();
            self.verify_entry_matches(name, before_version.identity, EntryKind::File)?;
            self.verify_namespace()?;

            let limit = u64::try_from(max_bytes).unwrap_or(u64::MAX);
            let mut bytes = Vec::with_capacity(max_bytes.min(before.len() as usize));
            Read::by_ref(&mut file)
                .take(limit.saturating_add(1))
                .read_to_end(&mut bytes)
                .map_err(|error| state_io("read private file", error))?;
            if bytes.len() > max_bytes {
                return Err(state_error(
                    "state.size",
                    format!("private file exceeds {max_bytes} bytes"),
                ));
            }
            let after = file
                .metadata()
                .map_err(|error| state_io("reinspect private file", error))?;
            if before_version != file_version(&after) || bytes.len() as u64 != after.len() {
                return Err(state_error(
                    "state.conflict",
                    "private file changed while it was read",
                ));
            }
            self.verify_entry_matches(name, before_version.identity, EntryKind::File)?;
            self.verify_namespace()?;
            Ok(bytes)
        }

        fn read_optional_snapshot(&self, name: &CStr) -> Result<Option<FileSnapshot>, AppError> {
            match self.read_snapshot(name) {
                Ok((_, snapshot)) => Ok(Some(snapshot)),
                Err(error) if error.code() == "state.missing" => Ok(None),
                Err(error) => Err(error),
            }
        }

        fn verify_snapshot(&self, name: &CStr, expected: &FileSnapshot) -> Result<(), AppError> {
            let (_, current) = self.read_snapshot(name)?;
            if &current != expected {
                return Err(state_error(
                    "state.conflict",
                    "private file changed before mutation",
                ));
            }
            Ok(())
        }

        fn verify_entry_matches(
            &self,
            name: &CStr,
            identity: FileIdentity,
            kind: EntryKind,
        ) -> Result<(), AppError> {
            let stat = statat_nofollow(self.fd.as_raw_fd(), name)
                .map_err(|error| state_io("inspect descriptor-relative entry", error))?;
            if kind_from_stat(&stat) == Some(kind) && identity_from_stat(&stat) == identity {
                Ok(())
            } else {
                Err(state_error(
                    "state.conflict",
                    "descriptor-relative entry identity changed before mutation",
                ))
            }
        }

        fn create_temporary_file(
            &self,
            prefix: &str,
            bytes: &[u8],
        ) -> Result<TemporaryEntry, AppError> {
            for _ in 0..32 {
                let name = unique_name(prefix)?;
                match self.create_file_entry(&name, bytes) {
                    Ok(created) => return Ok(created),
                    Err(error) if error.code() == "state.exists" => {}
                    Err(error) => return Err(error),
                }
            }
            Err(state_error(
                "state.io",
                "could not allocate temporary private file",
            ))
        }

        pub(super) fn create_staged_directory(
            &self,
            prefix: &str,
        ) -> Result<StagedDirectory, AppError> {
            for _ in 0..32 {
                let name = unique_name(prefix)?;
                self.verify_namespace()?;
                let cleanup_parent = self.duplicate()?;
                let cleanup_name = name.clone();
                match mkdirat_private(self.fd.as_raw_fd(), &name) {
                    Ok(()) => {
                        let mut cleanup = CreatedDirectoryGuard::new(cleanup_parent, cleanup_name);
                        inject_creation_fault(&name, CreationFault::DirectoryPostMkdir)?;
                        let created = statat_nofollow(self.fd.as_raw_fd(), &name)
                            .map_err(|error| state_io("inspect staged directory", error))?;
                        if kind_from_stat(&created) != Some(EntryKind::Directory) {
                            return Err(if kind_from_stat(&created) == Some(EntryKind::Symlink) {
                                state_symlink_name(&name)
                            } else {
                                state_error("state.path", "staged directory has an unsafe type")
                            });
                        }
                        let created_identity = identity_from_stat(&created);
                        cleanup.bind_identity(created_identity);
                        self.verify_namespace()?;
                        chmodat_directory_bootstrap(
                            self.fd.as_raw_fd(),
                            &name,
                            DIRECTORY_MODE as libc::mode_t,
                        )
                        .map_err(|error| state_io("bootstrap private directory mode", error))?;
                        let after_chmod = statat_nofollow(self.fd.as_raw_fd(), &name)
                            .map_err(|error| state_io("reinspect staged directory", error))?;
                        if kind_from_stat(&after_chmod) == Some(EntryKind::Symlink) {
                            return Err(state_symlink_name(&name));
                        }
                        if kind_from_stat(&after_chmod) != Some(EntryKind::Directory)
                            || identity_from_stat(&after_chmod) != created_identity
                        {
                            return Err(state_error(
                                "state.conflict",
                                "staged directory identity changed before descriptor open",
                            ));
                        }
                        let directory = self.open_directory_unchecked(&name)?;
                        if directory.identity()? != created_identity {
                            return Err(state_error(
                                "state.conflict",
                                "staged directory identity changed during creation",
                            ));
                        }
                        directory.chmod(DIRECTORY_MODE as libc::mode_t)?;
                        directory.verify_mode(DIRECTORY_MODE)?;
                        directory.verify_owner()?;
                        directory.sync()?;
                        self.sync()?;
                        self.verify_namespace()?;
                        let identity = directory.identity()?;
                        let staged = StagedDirectory {
                            parent: self.duplicate()?,
                            name,
                            identity,
                            directory,
                            cleanup_holder: None,
                            active: true,
                        };
                        cleanup.disarm();
                        return Ok(staged);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        match statat_nofollow(self.fd.as_raw_fd(), &name) {
                            Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Symlink) => {
                                return Err(state_symlink_name(&name));
                            }
                            Ok(_) => {}
                            Err(race) if race.raw_os_error() == Some(libc::ENOENT) => {}
                            Err(race) => {
                                return Err(state_io("inspect staged-directory collision", race));
                            }
                        }
                    }
                    Err(error) => {
                        return Err(classify_component_error(self.fd.as_raw_fd(), &name, error));
                    }
                }
            }
            Err(state_error(
                "state.io",
                "could not allocate staged private directory",
            ))
        }

        #[cfg(test)]
        pub(super) fn publish_staged_directory(
            &self,
            mut staged: StagedDirectory,
            destination: &str,
        ) -> Result<(), AppError> {
            let destination = cstring(destination)?;
            self.verify_entry_matches(&staged.name, staged.identity, EntryKind::Directory)?;
            match self.publish_staged_directory_inner(&staged.name, &destination) {
                Ok(()) => {
                    staged.disarm();
                    self.sync()
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(self
                    .classify_existing_target(
                        &destination,
                        "state.exists",
                        "destination already exists",
                    )),
                Err(error) => Err(state_io("publish staged directory", error)),
            }
        }

        fn publish_staged_directory_inner(
            &self,
            source: &CStr,
            destination: &CStr,
        ) -> std::io::Result<()> {
            renameat_noreplace(
                self.fd.as_raw_fd(),
                source,
                self.fd.as_raw_fd(),
                destination,
            )
        }

        fn publish_new_file(
            &self,
            temporary: &mut TemporaryEntry,
            destination: &CStr,
        ) -> Result<(), AppError> {
            let expected = temporary.snapshot.as_ref().ok_or_else(|| {
                state_error("state.conflict", "temporary private file has no snapshot")
            })?;
            self.verify_snapshot(&temporary.name, expected)?;
            self.verify_namespace()?;
            match renameat_noreplace(
                self.fd.as_raw_fd(),
                &temporary.name,
                self.fd.as_raw_fd(),
                destination,
            ) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(self.classify_existing_target(
                        destination,
                        "state.exists",
                        "private file already exists",
                    ));
                }
                Err(error) => return Err(state_io("publish private file", error)),
            }
            match self.read_snapshot(destination) {
                Ok((_, current)) if current.matches_exchanged_file(expected) => {
                    temporary.disarm();
                    self.verify_namespace()?;
                    Ok(())
                }
                _ => {
                    temporary.disarm();
                    Err(state_error(
                        "state.conflict",
                        "temporary private file identity changed during publication",
                    ))
                }
            }
        }

        fn publish_replacement(
            &self,
            temporary: &mut TemporaryEntry,
            destination: &CStr,
            expected: &FileSnapshot,
        ) -> Result<(), AppError> {
            self.verify_entry_matches(&temporary.name, temporary.identity, EntryKind::File)?;
            self.verify_snapshot(destination, expected)?;
            self.verify_namespace()?;
            if unsafe {
                libc::renameat(
                    self.fd.as_raw_fd(),
                    temporary.name.as_ptr(),
                    self.fd.as_raw_fd(),
                    destination.as_ptr(),
                )
            } == 0
            {
                temporary.disarm();
                self.verify_namespace()
            } else {
                Err(state_io(
                    "replace private file",
                    std::io::Error::last_os_error(),
                ))
            }
        }

        fn verify_published_file(
            &self,
            destination: &CStr,
            temporary: &TemporaryEntry,
        ) -> Result<(), AppError> {
            let intended = temporary.snapshot.as_ref().ok_or_else(|| {
                state_error("state.conflict", "temporary private file has no snapshot")
            })?;
            match self.read_snapshot(destination) {
                Ok((_, published)) if published.matches_exchanged_file(intended) => Ok(()),
                _ => Err(state_error(
                    "state.conflict",
                    "private file changed immediately after replacement",
                )),
            }
        }

        fn classify_existing_target(
            &self,
            name: &CStr,
            default_code: &'static str,
            default_message: &str,
        ) -> AppError {
            match statat_nofollow(self.fd.as_raw_fd(), name) {
                Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Symlink) => state_error(
                    "state.symlink",
                    format!(
                        "private state cannot traverse a symlink: {}",
                        display_name(name)
                    ),
                ),
                Ok(_) => state_error(
                    default_code,
                    format!("{default_message}: {}", display_name(name)),
                ),
                Err(error) => state_io("inspect raced publication target", error),
            }
        }

        fn remove_file(&self, name: &CStr, identity: FileIdentity) -> Result<(), AppError> {
            self.verify_namespace()?;
            self.verify_entry_matches(name, identity, EntryKind::File)?;
            let quarantine = unique_name(".harp-remove-file")?;
            self.verify_namespace()?;
            renameat_noreplace(self.fd.as_raw_fd(), name, self.fd.as_raw_fd(), &quarantine)
                .map_err(|error| state_io("quarantine private file", error))?;
            self.verify_entry_matches(&quarantine, identity, EntryKind::File)?;
            self.verify_namespace()?;
            unlinkat(self.fd.as_raw_fd(), &quarantine, 0)
                .map_err(|error| state_io("remove private file", error))?;
            self.sync()
        }

        fn remove_empty_directory(
            &self,
            name: &CStr,
            identity: FileIdentity,
        ) -> Result<(), AppError> {
            self.verify_namespace()?;
            self.verify_entry_matches(name, identity, EntryKind::Directory)?;
            let quarantine = unique_name(".harp-remove-directory")?;
            self.verify_namespace()?;
            renameat_noreplace(self.fd.as_raw_fd(), name, self.fd.as_raw_fd(), &quarantine)
                .map_err(|error| state_io("quarantine private directory", error))?;
            self.verify_entry_matches(&quarantine, identity, EntryKind::Directory)?;
            self.verify_namespace()?;
            unlinkat(self.fd.as_raw_fd(), &quarantine, libc::AT_REMOVEDIR)
                .map_err(|error| state_io("remove private directory", error))?;
            self.sync()
        }

        fn chmod(&self, mode: libc::mode_t) -> Result<(), AppError> {
            self.verify_namespace()?;
            if unsafe { libc::fchmod(self.fd.as_raw_fd(), mode) } == 0 {
                self.verify_namespace()
            } else {
                Err(state_io(
                    "set private directory mode",
                    std::io::Error::last_os_error(),
                ))
            }
        }

        fn verify_mode(&self, expected: u32) -> Result<(), AppError> {
            let metadata = File::from(self.duplicate()?.fd)
                .metadata()
                .map_err(|error| state_io("inspect private directory mode", error))?;
            if metadata.permissions().mode() & 0o777 != expected {
                return Err(state_error(
                    "state.permissions",
                    format!("private directory does not have mode {expected:o}"),
                ));
            }
            Ok(())
        }

        fn verify_owner(&self) -> Result<(), AppError> {
            let metadata = File::from(self.duplicate()?.fd)
                .metadata()
                .map_err(|error| state_io("inspect private directory owner", error))?;
            validate_owner(
                metadata.uid(),
                unsafe { libc::geteuid() },
                "private directory",
            )
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum EntryKind {
        File,
        Directory,
        Symlink,
    }

    #[derive(Debug)]
    struct CreatedFileGuard {
        parent: Directory,
        name: CString,
        file: Option<File>,
        identity: Option<FileIdentity>,
        active: bool,
    }

    impl CreatedFileGuard {
        fn file(&self) -> &File {
            self.file.as_ref().expect("created file guard owns a file")
        }

        fn file_mut(&mut self) -> &mut File {
            self.file.as_mut().expect("created file guard owns a file")
        }

        fn bind_identity(&mut self, identity: FileIdentity) {
            self.identity = Some(identity);
        }

        fn into_temporary(mut self, snapshot: FileSnapshot) -> Result<TemporaryEntry, AppError> {
            let temporary = TemporaryEntry {
                parent: self.parent.duplicate()?,
                name: self.name.clone(),
                identity: snapshot.version.identity,
                snapshot: Some(snapshot),
                kind: EntryKind::File,
                active: true,
            };
            self.file.take();
            self.active = false;
            Ok(temporary)
        }
    }

    impl Drop for CreatedFileGuard {
        fn drop(&mut self) {
            if !self.active {
                return;
            }
            let identity = self.identity.or_else(|| {
                self.file
                    .as_ref()
                    .and_then(|file| file.metadata().ok())
                    .map(|metadata| file_identity(&metadata))
            });
            self.file.take();
            if let Some(identity) = identity {
                let _cleanup_result = self.parent.remove_file(&self.name, identity);
            }
        }
    }

    #[derive(Debug)]
    struct CreatedDirectoryGuard {
        parent: Directory,
        name: CString,
        identity: Option<FileIdentity>,
        active: bool,
    }

    impl CreatedDirectoryGuard {
        fn new(parent: Directory, name: CString) -> Self {
            Self {
                parent,
                name,
                identity: None,
                active: true,
            }
        }

        fn bind_identity(&mut self, identity: FileIdentity) {
            self.identity = Some(identity);
        }

        fn disarm(&mut self) {
            self.active = false;
        }
    }

    impl Drop for CreatedDirectoryGuard {
        fn drop(&mut self) {
            if !self.active {
                return;
            }
            let identity = self.identity.or_else(|| {
                statat_nofollow(self.parent.fd.as_raw_fd(), &self.name)
                    .ok()
                    .filter(|stat| kind_from_stat(stat) == Some(EntryKind::Directory))
                    .map(|stat| identity_from_stat(&stat))
            });
            if let Some(identity) = identity {
                let _cleanup_result = self.parent.remove_empty_directory(&self.name, identity);
            }
        }
    }

    #[derive(Debug)]
    struct TemporaryEntry {
        parent: Directory,
        name: CString,
        identity: FileIdentity,
        snapshot: Option<FileSnapshot>,
        kind: EntryKind,
        active: bool,
    }

    impl TemporaryEntry {
        fn remove(&mut self) -> Result<(), AppError> {
            if !self.active {
                return Ok(());
            }
            match self.kind {
                EntryKind::File => self.parent.remove_file(&self.name, self.identity)?,
                EntryKind::Directory => {
                    self.parent
                        .remove_empty_directory(&self.name, self.identity)?;
                }
                EntryKind::Symlink => {
                    return Err(state_error(
                        "state.conflict",
                        "temporary entry cannot be a symlink",
                    ));
                }
            }
            self.active = false;
            Ok(())
        }

        fn disarm(&mut self) {
            self.active = false;
        }
    }

    impl Drop for TemporaryEntry {
        fn drop(&mut self) {
            if self.active {
                let _cleanup_result = self.remove();
            }
        }
    }

    #[derive(Debug)]
    pub(super) struct StagedDirectory {
        parent: Directory,
        name: CString,
        identity: FileIdentity,
        directory: Directory,
        cleanup_holder: Option<(HolderIdentity, FileSnapshot)>,
        active: bool,
    }

    impl StagedDirectory {
        fn disarm(&mut self) {
            self.active = false;
        }
    }

    impl Drop for StagedDirectory {
        fn drop(&mut self) {
            if self.active {
                let _cleanup_result = match &self.cleanup_holder {
                    Some((holder, holder_snapshot)) => remove_lock_entry(
                        &self.parent,
                        &self.name,
                        &self.directory,
                        self.identity,
                        *holder,
                        holder_snapshot,
                    ),
                    None => remove_tree_entry(&self.parent, &self.name, self.identity),
                };
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) struct ProcessIdentity {
        pub(super) pid: u32,
        pub(super) start: u128,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) struct HolderIdentity {
        pub(super) process: ProcessIdentity,
        pub(super) nonce: u128,
    }

    impl HolderIdentity {
        fn current() -> Result<Self, AppError> {
            let process = current_process_identity()?;
            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|error| {
                    state_error("state.lock", format!("system clock error: {error}"))
                })?;
            let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed) as u128;
            Ok(Self {
                process,
                nonce: elapsed.as_nanos() ^ (counter << 64),
            })
        }

        pub(super) fn encode(self) -> Vec<u8> {
            format!(
                "{}\n{}\n{}\n",
                self.process.pid, self.process.start, self.nonce
            )
            .into_bytes()
        }

        fn decode(bytes: &[u8]) -> Result<Self, AppError> {
            let text = std::str::from_utf8(bytes)
                .map_err(|_| state_error("state.lock", "lock holder identity is not UTF-8"))?;
            let mut lines = text.lines();
            let pid = lines
                .next()
                .and_then(|value| value.parse::<u32>().ok())
                .ok_or_else(|| state_error("state.lock", "lock holder pid is invalid"))?;
            let start = lines
                .next()
                .and_then(|value| value.parse::<u128>().ok())
                .ok_or_else(|| {
                    state_error("state.lock", "lock holder start identity is invalid")
                })?;
            let nonce = lines
                .next()
                .and_then(|value| value.parse::<u128>().ok())
                .ok_or_else(|| state_error("state.lock", "lock holder nonce is invalid"))?;
            if lines.next().is_some() {
                return Err(state_error(
                    "state.lock",
                    "lock holder identity has extra fields",
                ));
            }
            Ok(Self {
                process: ProcessIdentity { pid, start },
                nonce,
            })
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum LockSetupFault {
        None,
        AfterStage,
        AfterPublish,
    }

    #[derive(Debug)]
    pub(super) struct PublicationLock {
        parent: Directory,
        name: CString,
        directory: Directory,
        identity: FileIdentity,
        holder: HolderIdentity,
        holder_snapshot: FileSnapshot,
        released: bool,
    }

    impl PublicationLock {
        pub(super) fn release(&mut self) -> Result<(), AppError> {
            if self.released {
                return Ok(());
            }
            remove_lock_entry(
                &self.parent,
                &self.name,
                &self.directory,
                self.identity,
                self.holder,
                &self.holder_snapshot,
            )?;
            self.released = true;
            Ok(())
        }

        #[cfg(test)]
        pub(super) fn holder(&self) -> HolderIdentity {
            self.holder
        }

        #[cfg(test)]
        pub(super) fn replace_holder(
            &mut self,
            replacement: HolderIdentity,
        ) -> Result<(), AppError> {
            let holder_name = cstring("holder")?;
            let mut temporary = self
                .directory
                .create_temporary_file(".holder-replacement", &replacement.encode())?;
            self.directory
                .verify_snapshot(&holder_name, &self.holder_snapshot)?;
            self.directory.publish_replacement(
                &mut temporary,
                &holder_name,
                &self.holder_snapshot,
            )?;
            self.directory.sync()
        }

        #[cfg(test)]
        pub(super) fn exists(&self) -> Result<bool, AppError> {
            self.parent
                .open_optional_directory(&self.name)
                .map(|directory| directory.is_some())
        }
    }

    impl Drop for PublicationLock {
        fn drop(&mut self) {
            if !self.released {
                let _cleanup_result = self.release();
            }
        }
    }

    #[derive(Debug)]
    enum LockState {
        Gone,
        Live,
        Dead {
            directory: Directory,
            identity: FileIdentity,
            holder: HolderIdentity,
            holder_snapshot: FileSnapshot,
        },
    }

    fn inspect_lock(locks: &Directory, name: &CStr) -> Result<LockState, AppError> {
        let lock = match locks.open_directory(name) {
            Ok(lock) => lock,
            Err(error) if error.code() == "state.missing" => return Ok(LockState::Gone),
            Err(error) => return Err(error),
        };
        let identity = lock.identity()?;
        let (holder_bytes, holder_snapshot) = match lock.read_snapshot(&cstring("holder")?) {
            Ok(holder) => holder,
            Err(error) if error.code() == "state.missing" => {
                return classify_missing_lock_holder(locks, name, identity);
            }
            Err(error) => return Err(error),
        };
        let holder = HolderIdentity::decode(&holder_bytes)?;
        match process_identity(holder.process.pid)? {
            Some(current) if current == holder.process => Ok(LockState::Live),
            _ => Ok(LockState::Dead {
                directory: lock,
                identity,
                holder,
                holder_snapshot,
            }),
        }
    }

    fn recover_dead_lock(
        locks: &Directory,
        name: &CStr,
        lock: &Directory,
        identity: FileIdentity,
        expected_holder: HolderIdentity,
        expected_holder_snapshot: &FileSnapshot,
    ) -> Result<(), AppError> {
        remove_lock_entry(
            locks,
            name,
            lock,
            identity,
            expected_holder,
            expected_holder_snapshot,
        )
    }

    fn classify_missing_lock_holder(
        locks: &Directory,
        name: &CStr,
        identity: FileIdentity,
    ) -> Result<LockState, AppError> {
        match statat_nofollow(locks.fd.as_raw_fd(), name) {
            Err(error) if error.raw_os_error() == Some(libc::ENOENT) => Ok(LockState::Gone),
            Err(error) => Err(state_io("reinspect holderless publication lock", error)),
            Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Symlink) => {
                Err(state_symlink_name(name))
            }
            Ok(stat)
                if kind_from_stat(&stat) == Some(EntryKind::Directory)
                    && identity_from_stat(&stat) == identity =>
            {
                Err(state_error(
                    "state.lock",
                    "published lock has no holder identity",
                ))
            }
            Ok(_) => Ok(LockState::Gone),
        }
    }

    fn remove_lock_entry(
        locks: &Directory,
        name: &CStr,
        lock: &Directory,
        identity: FileIdentity,
        expected_holder: HolderIdentity,
        expected_holder_snapshot: &FileSnapshot,
    ) -> Result<(), AppError> {
        locks.verify_namespace()?;
        if lock.identity()? != identity {
            return Err(state_error(
                "state.lock",
                "publication lock descriptor identity changed before quarantine",
            ));
        }
        lock.verify_snapshot(&cstring("holder")?, expected_holder_snapshot)
            .map_err(|_| {
                state_error(
                    "state.lock",
                    "publication lock holder changed before quarantine",
                )
            })?;
        locks.verify_entry_matches(name, identity, EntryKind::Directory)?;
        let quarantine_name = unique_name(".harp-stale-lock")?;
        locks.verify_namespace()?;
        renameat_noreplace(
            locks.fd.as_raw_fd(),
            name,
            locks.fd.as_raw_fd(),
            &quarantine_name,
        )
        .map_err(|error| state_io("quarantine stale publication lock", error))?;
        locks.sync()?;
        let quarantine = locks.open_directory(&quarantine_name)?;
        let quarantine_identity = quarantine.identity()?;
        if quarantine_identity != identity {
            restore_quarantined_lock(locks, &quarantine_name, name)?;
            return Err(state_error(
                "state.lock",
                "stale lock identity changed during quarantine",
            ));
        }
        let (holder_bytes, holder_snapshot) = quarantine.read_snapshot(&cstring("holder")?)?;
        if &holder_snapshot != expected_holder_snapshot
            || HolderIdentity::decode(&holder_bytes)? != expected_holder
        {
            restore_quarantined_lock(locks, &quarantine_name, name)?;
            return Err(state_error(
                "state.lock",
                "stale lock holder changed during quarantine",
            ));
        }
        quarantine.remove_file(&cstring("holder")?, holder_snapshot.version.identity)?;
        quarantine.sync()?;
        locks.remove_empty_directory(&quarantine_name, quarantine_identity)
    }

    fn restore_quarantined_lock(
        locks: &Directory,
        quarantine_name: &CStr,
        original_name: &CStr,
    ) -> Result<(), AppError> {
        locks.verify_namespace()?;
        renameat_noreplace(
            locks.fd.as_raw_fd(),
            quarantine_name,
            locks.fd.as_raw_fd(),
            original_name,
        )
        .map_err(|error| state_io("restore quarantined publication lock", error))?;
        locks.sync()
    }

    fn finish_publication(
        result: Result<(), AppError>,
        lock: &mut PublicationLock,
    ) -> Result<(), AppError> {
        match result {
            Err(error) => {
                let _cleanup_result = lock.release();
                Err(error)
            }
            Ok(()) => lock.release(),
        }
    }

    fn duplicate_fd(fd: RawFd) -> Result<OwnedFd, AppError> {
        let duplicate = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
        if duplicate < 0 {
            return Err(state_io(
                "duplicate directory descriptor",
                std::io::Error::last_os_error(),
            ));
        }
        Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
    }

    fn open_filesystem_root() -> Result<Directory, AppError> {
        let root_fd = unsafe {
            libc::open(
                c"/".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if root_fd < 0 {
            return Err(state_io(
                "open filesystem root",
                std::io::Error::last_os_error(),
            ));
        }
        Ok(Directory {
            fd: unsafe { OwnedFd::from_raw_fd(root_fd) },
            anchor: None,
            route: Vec::new(),
        })
    }

    fn open_namespace_component(parent: &Directory, name: &CStr) -> Result<Directory, AppError> {
        match parent.open_directory_unchecked(name) {
            Ok(directory) => Ok(directory),
            Err(error) if error.code() == "state.symlink" => Err(error),
            Err(error) if error.code() == "state.missing" || error.code() == "state.path" => {
                Err(state_error(
                    "state.conflict",
                    format!(
                        "anchored namespace component changed: {}",
                        display_name(name)
                    ),
                ))
            }
            Err(error) => Err(error),
        }
    }

    fn open_or_create_absolute_directory(path: &Path) -> Result<Directory, AppError> {
        let components = absolute_components(path)?;
        let mut current = open_filesystem_root()?;
        for (index, component) in components.iter().enumerate() {
            let final_component = index + 1 == components.len();
            current = match current.open_directory_unchecked(component) {
                Ok(directory) => {
                    if final_component {
                        directory.verify_mode(DIRECTORY_MODE)?;
                        directory.verify_owner()?;
                    }
                    directory
                }
                Err(error) if error.code() == "state.missing" => {
                    current.open_or_create_directory(component)?
                }
                Err(error) => return Err(error),
            };
        }
        let root_identity = current.raw_identity()?;
        let anchor = std::sync::Arc::new(NamespaceAnchor {
            root_fd: duplicate_fd(current.fd.as_raw_fd())?,
            root_identity,
            absolute_route: current.route.clone(),
        });
        current.anchor = Some(anchor);
        current.route.clear();
        current.verify_namespace()?;
        Ok(current)
    }

    fn absolute_components(path: &Path) -> Result<Vec<CString>, AppError> {
        if !path.is_absolute() {
            return Err(state_path("state root"));
        }
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::RootDir => {}
                Component::Normal(component) => components.push(os_string(component)?),
                Component::Prefix(_) | Component::CurDir | Component::ParentDir => {
                    return Err(state_path("state root"));
                }
            }
        }
        if components.is_empty() {
            return Err(state_path("state root"));
        }
        Ok(components)
    }

    fn canonical_relative_components(
        relative: &Path,
        label: &str,
    ) -> Result<Vec<CString>, AppError> {
        if relative.is_absolute() || relative.as_os_str().is_empty() {
            return Err(state_path(label));
        }
        let raw = relative.as_os_str().as_bytes();
        if raw.first() == Some(&b'/')
            || raw.last() == Some(&b'/')
            || raw.windows(2).any(|window| window == b"//")
            || raw
                .split(|byte| *byte == b'/')
                .any(|component| component.is_empty() || component == b"." || component == b"..")
        {
            return Err(state_path(label));
        }
        let mut components = Vec::new();
        for component in relative.components() {
            let Component::Normal(component) = component else {
                return Err(state_path(label));
            };
            components.push(os_string(component)?);
        }
        if components.is_empty() {
            return Err(state_path(label));
        }
        Ok(components)
    }

    fn public_relative_components(relative: &Path, label: &str) -> Result<Vec<CString>, AppError> {
        let components = canonical_relative_components(relative, label)?;
        if components
            .first()
            .is_some_and(|component| component.as_bytes() == INTERNAL_NAMESPACE.as_bytes())
        {
            return Err(state_error(
                "state.path",
                "private path uses a reserved internal namespace",
            ));
        }
        Ok(components)
    }

    fn release_authorization_components(release_id: &str) -> Result<Vec<CString>, AppError> {
        if release_id.is_empty()
            || release_id.contains('/')
            || release_id.as_bytes().contains(&0)
            || matches!(release_id, "." | "..")
        {
            return Err(state_path("release authorization ID"));
        }
        Ok(vec![
            cstring(INTERNAL_NAMESPACE)?,
            cstring(RELEASE_AUTHORIZATIONS)?,
            cstring(release_id)?,
        ])
    }

    fn validate_tree_members(
        members: &BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<BTreeSet<PathBuf>, AppError> {
        let mut directories = BTreeSet::new();
        for member in members.keys() {
            canonical_relative_components(member, "immutable-tree member")?;
            for parent in member.ancestors().skip(1) {
                if parent.as_os_str().is_empty() {
                    break;
                }
                if members.contains_key(parent) {
                    return Err(state_error(
                        "state.path",
                        format!(
                            "immutable-tree member is also a parent: {}",
                            parent.display()
                        ),
                    ));
                }
                directories.insert(parent.to_path_buf());
            }
        }
        Ok(directories)
    }

    fn immutable_tree_matches(
        directory: &Directory,
        expected_members: &BTreeMap<PathBuf, Vec<u8>>,
        expected_directories: &BTreeSet<PathBuf>,
    ) -> Result<bool, AppError> {
        let mut actual_members = BTreeMap::new();
        let mut actual_directories = BTreeSet::new();
        collect_tree(
            directory,
            Path::new(""),
            &mut actual_members,
            &mut actual_directories,
        )?;
        Ok(&actual_members == expected_members && &actual_directories == expected_directories)
    }

    fn collect_tree(
        directory: &Directory,
        relative: &Path,
        members: &mut BTreeMap<PathBuf, Vec<u8>>,
        directories: &mut BTreeSet<PathBuf>,
    ) -> Result<(), AppError> {
        for name in directory.entry_names()? {
            let child_relative = relative.join(OsStr::from_bytes(name.to_bytes()));
            let stat = statat_nofollow(directory.fd.as_raw_fd(), &name)
                .map_err(|error| state_io("inspect immutable-tree member", error))?;
            match kind_from_stat(&stat) {
                Some(EntryKind::Directory) => {
                    directories.insert(child_relative.clone());
                    let child = directory.open_directory(&name)?;
                    collect_tree(&child, &child_relative, members, directories)?;
                }
                Some(EntryKind::File) => {
                    let (bytes, _) = directory.read_snapshot(&name)?;
                    members.insert(child_relative, bytes);
                }
                Some(EntryKind::Symlink) => {
                    return Err(state_error(
                        "state.symlink",
                        format!(
                            "immutable tree cannot contain a symlink: {}",
                            child_relative.display()
                        ),
                    ));
                }
                None => {
                    return Err(state_error(
                        "state.path",
                        format!(
                            "immutable tree contains an unsafe entry: {}",
                            child_relative.display()
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    impl Directory {
        fn entry_names(&self) -> Result<Vec<CString>, AppError> {
            let duplicate = unsafe { libc::fcntl(self.fd.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) };
            if duplicate < 0 {
                return Err(state_io(
                    "duplicate directory for enumeration",
                    std::io::Error::last_os_error(),
                ));
            }
            let stream = unsafe { libc::fdopendir(duplicate) };
            if stream.is_null() {
                unsafe {
                    libc::close(duplicate);
                }
                return Err(state_io(
                    "open directory stream",
                    std::io::Error::last_os_error(),
                ));
            }
            let mut names = Vec::new();
            loop {
                set_errno_zero();
                let entry = unsafe { libc::readdir(stream) };
                if entry.is_null() {
                    let error = std::io::Error::last_os_error();
                    unsafe {
                        libc::closedir(stream);
                    }
                    if error.raw_os_error() == Some(0) {
                        break;
                    }
                    return Err(state_io("read directory stream", error));
                }
                let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
                if name.to_bytes() != b"." && name.to_bytes() != b".." {
                    names.push(name.to_owned());
                }
            }
            names.sort_by(|left, right| left.to_bytes().cmp(right.to_bytes()));
            Ok(names)
        }
    }

    fn sync_tree_directories(directory: &Directory) -> Result<(), AppError> {
        for name in directory.entry_names()? {
            let stat = statat_nofollow(directory.fd.as_raw_fd(), &name)
                .map_err(|error| state_io("inspect staged tree", error))?;
            if kind_from_stat(&stat) == Some(EntryKind::Directory) {
                let child = directory.open_directory(&name)?;
                sync_tree_directories(&child)?;
                child.sync()?;
            }
        }
        directory.sync()
    }

    fn remove_tree_entry(
        parent: &Directory,
        name: &CStr,
        identity: FileIdentity,
    ) -> Result<(), AppError> {
        parent.verify_namespace()?;
        parent.verify_entry_matches(name, identity, EntryKind::Directory)?;
        let directory = parent.open_directory(name)?;
        for child_name in directory.entry_names()? {
            let stat = statat_nofollow(directory.fd.as_raw_fd(), &child_name)
                .map_err(|error| state_io("inspect temporary tree", error))?;
            match kind_from_stat(&stat) {
                Some(EntryKind::File) => {
                    directory.remove_file(&child_name, identity_from_stat(&stat))?;
                }
                Some(EntryKind::Directory) => {
                    remove_tree_entry(&directory, &child_name, identity_from_stat(&stat))?;
                }
                Some(EntryKind::Symlink) | None => {
                    return Err(state_error(
                        "state.conflict",
                        "temporary tree contains an unexpected entry",
                    ));
                }
            }
        }
        parent.remove_empty_directory(name, identity)
    }

    fn validate_replace_policy(
        replace: &ReplacePolicy,
        current: Option<&FileSnapshot>,
        name: &CStr,
    ) -> Result<(), AppError> {
        match (replace, current) {
            (ReplacePolicy::CreateOnly, None) => Ok(()),
            (ReplacePolicy::CreateOnly, Some(_)) => Err(state_error(
                "state.exists",
                format!("private file already exists: {}", display_name(name)),
            )),
            (ReplacePolicy::CompareAndReplace(expected), Some(current)) if expected == current => {
                Ok(())
            }
            (ReplacePolicy::CompareAndReplace(_), _) => Err(state_error(
                "state.conflict",
                format!(
                    "private file changed before replacement: {}",
                    display_name(name)
                ),
            )),
        }
    }

    fn validate_unchanged(
        replace: &ReplacePolicy,
        initial: Option<&FileSnapshot>,
        current: Option<&FileSnapshot>,
        name: &CStr,
    ) -> Result<(), AppError> {
        let unchanged = match (replace, initial, current) {
            (ReplacePolicy::CreateOnly, None, None) => true,
            (ReplacePolicy::CompareAndReplace(expected), Some(initial), Some(current)) => {
                expected == initial && initial == current
            }
            _ => false,
        };
        if unchanged {
            Ok(())
        } else {
            Err(state_error(
                "state.conflict",
                format!(
                    "private file changed during replacement: {}",
                    display_name(name)
                ),
            ))
        }
    }

    fn validate_metadata(
        metadata: &Metadata,
        expected: EntryKind,
        label: &str,
    ) -> Result<(), AppError> {
        let actual = if metadata.is_file() {
            EntryKind::File
        } else if metadata.is_dir() {
            EntryKind::Directory
        } else if metadata.file_type().is_symlink() {
            EntryKind::Symlink
        } else {
            return Err(state_error(
                "state.path",
                format!("{label} has an unsupported type"),
            ));
        };
        if actual != expected {
            return Err(state_error(
                "state.path",
                format!("{label} has the wrong type"),
            ));
        }
        validate_owner(metadata.uid(), unsafe { libc::geteuid() }, label)?;
        let expected_mode = match expected {
            EntryKind::File => FILE_MODE,
            EntryKind::Directory => DIRECTORY_MODE,
            EntryKind::Symlink => {
                return Err(state_error(
                    "state.symlink",
                    format!("{label} cannot be a symlink"),
                ));
            }
        };
        if metadata.permissions().mode() & 0o777 != expected_mode {
            return Err(state_error(
                "state.permissions",
                format!("{label} does not have mode {expected_mode:04o}"),
            ));
        }
        Ok(())
    }

    pub(super) fn validate_owner(actual: u32, expected: u32, label: &str) -> Result<(), AppError> {
        if actual == expected {
            Ok(())
        } else {
            Err(state_error(
                "state.ownership",
                format!("{label} is owned by uid {actual}, expected uid {expected}"),
            ))
        }
    }

    fn file_identity(metadata: &Metadata) -> FileIdentity {
        FileIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }

    fn file_version(metadata: &Metadata) -> FileVersion {
        FileVersion {
            identity: file_identity(metadata),
            length: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }

    fn identity_from_stat(stat: &libc::stat) -> FileIdentity {
        FileIdentity {
            device: stat.st_dev as u64,
            inode: stat.st_ino,
        }
    }

    fn kind_from_stat(stat: &libc::stat) -> Option<EntryKind> {
        match stat.st_mode & libc::S_IFMT {
            libc::S_IFREG => Some(EntryKind::File),
            libc::S_IFDIR => Some(EntryKind::Directory),
            libc::S_IFLNK => Some(EntryKind::Symlink),
            _ => None,
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

    fn mkdirat(dirfd: RawFd, name: &CStr, mode: libc::mode_t) -> std::io::Result<()> {
        if unsafe { libc::mkdirat(dirfd, name.as_ptr(), mode) } == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    fn mkdirat_private(dirfd: RawFd, name: &CStr) -> std::io::Result<()> {
        mkdirat(dirfd, name, DIRECTORY_MODE as libc::mode_t)
    }

    fn chmodat_directory_bootstrap(
        dirfd: RawFd,
        name: &CStr,
        mode: libc::mode_t,
    ) -> std::io::Result<()> {
        if unsafe { libc::fchmodat(dirfd, name.as_ptr(), mode, libc::AT_SYMLINK_NOFOLLOW) } == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    fn unlinkat(dirfd: RawFd, name: &CStr, flags: libc::c_int) -> std::io::Result<()> {
        if unsafe { libc::unlinkat(dirfd, name.as_ptr(), flags) } == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
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

    fn classify_open_error(dirfd: RawFd, name: &CStr, error: std::io::Error) -> AppError {
        match error.raw_os_error() {
            Some(libc::ENOENT) => state_error(
                "state.missing",
                format!("private entry is missing: {}", display_name(name)),
            ),
            Some(libc::ELOOP) => state_error(
                "state.symlink",
                format!(
                    "private state cannot traverse a symlink: {}",
                    display_name(name)
                ),
            ),
            Some(libc::ENOTDIR) => match statat_nofollow(dirfd, name) {
                Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Symlink) => state_error(
                    "state.symlink",
                    format!(
                        "private state cannot traverse a symlink: {}",
                        display_name(name)
                    ),
                ),
                _ => state_error(
                    "state.path",
                    format!(
                        "private path component is not a directory: {}",
                        display_name(name)
                    ),
                ),
            },
            _ => state_io("open descriptor-relative entry", error),
        }
    }

    fn classify_create_error(dirfd: RawFd, name: &CStr, error: std::io::Error) -> AppError {
        match error.raw_os_error() {
            Some(libc::EEXIST) | Some(libc::ELOOP) => match statat_nofollow(dirfd, name) {
                Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Symlink) => {
                    state_symlink_name(name)
                }
                Ok(_) => state_error(
                    "state.exists",
                    format!("private entry already exists: {}", display_name(name)),
                ),
                Err(race) if race.raw_os_error() == Some(libc::ENOENT) => state_error(
                    "state.conflict",
                    format!(
                        "private entry changed during creation: {}",
                        display_name(name)
                    ),
                ),
                Err(race) => state_io("inspect failed private-file creation", race),
            },
            Some(libc::ENOTDIR) => state_error(
                "state.path",
                format!(
                    "private file parent is not a directory: {}",
                    display_name(name)
                ),
            ),
            _ => state_io("create private file", error),
        }
    }

    fn classify_component_error(dirfd: RawFd, name: &CStr, error: std::io::Error) -> AppError {
        match error.raw_os_error() {
            Some(libc::ELOOP) => state_error(
                "state.symlink",
                format!(
                    "private state cannot traverse a symlink: {}",
                    display_name(name)
                ),
            ),
            Some(libc::EEXIST) => match statat_nofollow(dirfd, name) {
                Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Symlink) => {
                    state_symlink_name(name)
                }
                Ok(stat) if kind_from_stat(&stat) == Some(EntryKind::Directory) => state_error(
                    "state.conflict",
                    format!(
                        "private directory appeared during creation: {}",
                        display_name(name)
                    ),
                ),
                Ok(_) => state_error(
                    "state.path",
                    format!(
                        "private path component is not a directory: {}",
                        display_name(name)
                    ),
                ),
                Err(race) => state_io("inspect directory-creation race", race),
            },
            Some(libc::ENOTDIR) => state_error(
                "state.path",
                format!(
                    "private path component is not a directory: {}",
                    display_name(name)
                ),
            ),
            _ => state_io("create private directory component", error),
        }
    }

    fn state_symlink_name(name: &CStr) -> AppError {
        state_error(
            "state.symlink",
            format!(
                "private state cannot traverse a symlink: {}",
                display_name(name)
            ),
        )
    }

    fn cstring(value: &str) -> Result<CString, AppError> {
        CString::new(value)
            .map_err(|_| state_error("state.path", "private path contains a NUL byte"))
    }

    fn os_string(value: &OsStr) -> Result<CString, AppError> {
        CString::new(value.as_bytes())
            .map_err(|_| state_error("state.path", "private path contains a NUL byte"))
    }

    fn unique_name(prefix: &str) -> Result<CString, AppError> {
        let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
        cstring(&format!("{prefix}-{}-{counter}", std::process::id()))
    }

    fn lock_name(components: &[CString]) -> CString {
        let mut digest = Sha256::new();
        for component in components {
            digest.update((component.as_bytes().len() as u64).to_be_bytes());
            digest.update(component.as_bytes());
        }
        CString::new(format!("{}.lock", lowercase_hex(&digest.finalize())))
            .expect("hex lock name contains no NUL")
    }

    fn lowercase_hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
        encoded
    }

    fn display_name(name: &CStr) -> String {
        OsStr::from_bytes(name.to_bytes())
            .to_string_lossy()
            .into_owned()
    }

    fn nonempty_environment_path(name: &str) -> Option<PathBuf> {
        env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
    }

    fn absolute_root_path(path: &Path) -> Result<PathBuf, AppError> {
        if path.as_os_str().is_empty()
            || path
                .components()
                .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        {
            return Err(state_path("state root"));
        }
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            env::current_dir()
                .map(|current| current.join(path))
                .map_err(|error| state_io("resolve relative state root", error))
        }
    }

    #[cfg(target_os = "macos")]
    fn process_identity(pid: u32) -> Result<Option<ProcessIdentity>, AppError> {
        let mut info = unsafe { std::mem::zeroed::<libc::proc_bsdinfo>() };
        let expected = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        set_errno_zero();
        let received = unsafe {
            libc::proc_pidinfo(
                pid as libc::c_int,
                libc::PROC_PIDTBSDINFO,
                0,
                std::ptr::from_mut(&mut info).cast(),
                expected,
            )
        };
        let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        classify_macos_process_query(pid, received, errno, &info)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn classify_macos_process_query(
        pid: u32,
        received: libc::c_int,
        errno: libc::c_int,
        info: &libc::proc_bsdinfo,
    ) -> Result<Option<ProcessIdentity>, AppError> {
        let expected = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        if received == 0 && errno == libc::ESRCH {
            return Ok(None);
        }
        if received != expected {
            return Err(state_error(
                "state.lock",
                format!(
                    "could not read complete process identity for pid {pid}: received {received} bytes, errno {errno}"
                ),
            ));
        }
        Ok(Some(ProcessIdentity {
            pid,
            start: ((info.pbi_start_tvsec as u128) << 64) | info.pbi_start_tvusec as u128,
        }))
    }

    #[cfg(target_os = "linux")]
    fn process_identity(pid: u32) -> Result<Option<ProcessIdentity>, AppError> {
        let stat = match std::fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(state_io("read process identity", error)),
        };
        let process_fields = stat
            .rsplit_once(')')
            .map(|(_, fields)| fields)
            .ok_or_else(|| state_error("state.lock", "process identity has an invalid shape"))?;
        let start = process_fields
            .split_whitespace()
            .nth(19)
            .and_then(|value| value.parse::<u128>().ok())
            .ok_or_else(|| state_error("state.lock", "process start identity is invalid"))?;
        Ok(Some(ProcessIdentity { pid, start }))
    }

    fn current_process_identity() -> Result<ProcessIdentity, AppError> {
        let pid = std::process::id();
        process_identity(pid)?.ok_or_else(|| {
            state_error(
                "state.lock",
                format!("could not read current process identity for pid {pid}"),
            )
        })
    }

    #[cfg(target_os = "macos")]
    fn set_errno_zero() {
        unsafe {
            *libc::__error() = 0;
        }
    }

    #[cfg(target_os = "linux")]
    fn set_errno_zero() {
        unsafe {
            *libc::__errno_location() = 0;
        }
    }

    fn state_path(label: &str) -> AppError {
        state_error(
            "state.path",
            format!("{label} must use one canonical relative spelling without traversal"),
        )
    }

    fn state_error(code: &'static str, message: impl Into<String>) -> AppError {
        AppError::invalid_input(code, message)
    }

    fn state_io(context: &str, error: std::io::Error) -> AppError {
        AppError::io("state.io", context, error)
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use secure::{resolve_state_root, FileSnapshot, ReplacePolicy, StateRoot};

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod unsupported {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use crate::AppError;

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct FileSnapshot {
        _private: (),
    }

    #[derive(Clone, Debug)]
    pub enum ReplacePolicy {
        CreateOnly,
        CompareAndReplace(FileSnapshot),
    }

    #[derive(Debug)]
    pub struct StateRoot;

    impl StateRoot {
        pub fn open_from_environment() -> Result<Self, AppError> {
            Err(unsupported())
        }

        pub fn open_or_create(_path: &Path) -> Result<Self, AppError> {
            Err(unsupported())
        }

        pub fn create_private_directory(&self, _relative: &Path) -> Result<PathBuf, AppError> {
            Err(unsupported())
        }

        pub fn snapshot_private_file(&self, _relative: &Path) -> Result<FileSnapshot, AppError> {
            Err(unsupported())
        }

        pub fn list_private_directory(&self, _relative: &Path) -> Result<Vec<String>, AppError> {
            Err(unsupported())
        }

        pub fn read_private_file_bounded(
            &self,
            _relative: &Path,
            _max_bytes: usize,
        ) -> Result<Vec<u8>, AppError> {
            Err(unsupported())
        }

        pub fn write_private_atomic(
            &self,
            _relative: &Path,
            _bytes: &[u8],
            _replace: ReplacePolicy,
        ) -> Result<(), AppError> {
            Err(unsupported())
        }

        pub fn publish_immutable_tree(
            &self,
            _relative: &Path,
            _members: &BTreeMap<PathBuf, Vec<u8>>,
        ) -> Result<(), AppError> {
            Err(unsupported())
        }

        pub(crate) fn publish_release_authorization(
            &self,
            _release_id: &str,
            _bytes: &[u8],
        ) -> Result<(), AppError> {
            Err(unsupported())
        }

        pub(crate) fn read_release_authorization(
            &self,
            _release_id: &str,
            _max_bytes: usize,
        ) -> Result<Vec<u8>, AppError> {
            Err(unsupported())
        }
    }

    pub fn resolve_state_root(
        explicit: Option<&Path>,
        xdg_state_home: Option<&Path>,
        home: &Path,
    ) -> PathBuf {
        explicit
            .map(Path::to_path_buf)
            .or_else(|| xdg_state_home.map(|path| path.join("harp")))
            .unwrap_or_else(|| home.join(".local/state/harp"))
    }

    fn unsupported() -> AppError {
        AppError::invalid_input(
            "state.unsupported",
            "secure context-control state is supported only on macOS and Linux",
        )
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub use unsupported::{resolve_state_root, FileSnapshot, ReplacePolicy, StateRoot};

#[cfg(all(test, target_os = "macos"))]
use secure::classify_macos_process_query;
#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
use secure::{
    take_sync_trace, validate_owner, with_creation_fault, CreationFault, HolderIdentity,
    LockSetupFault, ProcessIdentity, SyncEvent,
};

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests {
    use std::collections::BTreeMap;
    use std::fs::{self, OpenOptions};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[cfg(unix)]
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    use tempfile::TempDir;

    use crate::AppError;

    use super::*;

    struct StateFixture {
        _temporary: TempDir,
        base: PathBuf,
    }

    impl StateFixture {
        fn new() -> Self {
            let temporary = tempfile::tempdir().expect("temporary state parent");
            let base =
                fs::canonicalize(temporary.path()).expect("canonical temporary state parent");
            Self {
                _temporary: temporary,
                base,
            }
        }

        fn root(&self) -> PathBuf {
            self.base.join("harp")
        }

        fn open(&self) -> StateRoot {
            StateRoot::open_or_create(&self.root()).expect("secure state root")
        }
    }

    #[test]
    fn state_root_precedence_is_explicit_then_xdg_then_home() {
        assert_eq!(
            resolve_state_root(
                Some(Path::new("/explicit")),
                Some(Path::new("/xdg")),
                Path::new("/home"),
            ),
            PathBuf::from("/explicit")
        );
        assert_eq!(
            resolve_state_root(None, Some(Path::new("/xdg")), Path::new("/home")),
            PathBuf::from("/xdg/harp")
        );
        assert_eq!(
            resolve_state_root(None, None, Path::new("/home")),
            PathBuf::from("/home/.local/state/harp")
        );
    }

    #[cfg(unix)]
    #[test]
    fn state_root_rejects_symlink_and_group_writable_root() {
        let fixture = StateFixture::new();
        let real = fixture.base.join("real");
        fs::create_dir(&real).expect("real state directory");
        fs::set_permissions(&real, fs::Permissions::from_mode(0o700))
            .expect("private real directory");
        let linked = fixture.base.join("linked");
        std::os::unix::fs::symlink(&real, &linked).expect("state-root symlink");
        assert_eq!(
            StateRoot::open_or_create(&linked).unwrap_err().code(),
            "state.symlink"
        );

        let writable = fixture.base.join("writable");
        fs::create_dir(&writable).expect("writable state directory");
        fs::set_permissions(&writable, fs::Permissions::from_mode(0o770))
            .expect("group-writable state directory");
        assert_eq!(
            StateRoot::open_or_create(&writable).unwrap_err().code(),
            "state.permissions"
        );

        let wrong_mode = fixture.base.join("wrong-mode");
        fs::create_dir(&wrong_mode).expect("wrong-mode state directory");
        fs::set_permissions(&wrong_mode, fs::Permissions::from_mode(0o500))
            .expect("owner-only wrong-mode directory");
        assert_eq!(
            StateRoot::open_or_create(&wrong_mode).unwrap_err().code(),
            "state.permissions"
        );
    }

    #[cfg(unix)]
    #[test]
    fn ownership_validation_isolated_from_process_privileges() {
        assert_eq!(
            validate_owner(1001, 1000, "test path").unwrap_err().code(),
            "state.ownership"
        );
        validate_owner(1000, 1000, "test path").expect("matching owner");
    }

    #[test]
    fn root_and_relative_paths_reject_traversal_and_absolute_relative_paths() {
        let fixture = StateFixture::new();
        assert_eq!(
            StateRoot::open_or_create(&fixture.base.join("missing/../harp"))
                .unwrap_err()
                .code(),
            "state.path"
        );

        let state = fixture.open();
        for relative in [
            Path::new("../escape"),
            Path::new("nested/../../escape"),
            Path::new("/absolute"),
            Path::new(""),
        ] {
            assert_eq!(
                state.create_private_directory(relative).unwrap_err().code(),
                "state.path",
                "{}",
                relative.display()
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn private_directories_and_files_have_owner_only_modes() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        for relative in [
            "repositories",
            "repositories/repository",
            "releases/release",
            "repositories/repository/episodes/episode",
            "repositories/repository/episodes/episode/raw/provider",
        ] {
            state
                .create_private_directory(Path::new(relative))
                .expect("private directory");
        }
        state
            .write_private_atomic(
                Path::new("repositories/repository/episodes/episode/manifest.json"),
                b"private",
                ReplacePolicy::CreateOnly,
            )
            .expect("private file");

        for relative in [
            "",
            "repositories",
            "repositories/repository",
            "releases",
            "releases/release",
            "repositories/repository/episodes",
            "repositories/repository/episodes/episode",
            "repositories/repository/episodes/episode/raw",
            "repositories/repository/episodes/episode/raw/provider",
        ] {
            assert_eq!(mode(&fixture.root().join(relative)), 0o700, "{relative}");
        }
        assert_eq!(
            mode(
                &fixture
                    .root()
                    .join("repositories/repository/episodes/episode/manifest.json")
            ),
            0o600
        );
    }

    #[test]
    fn private_path_components_reject_symlinks() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        state
            .create_private_directory(Path::new("repositories"))
            .expect("repository parent");
        let outside = fixture.base.join("outside");
        fs::create_dir(&outside).expect("outside directory");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, fixture.root().join("repositories/redirect"))
            .expect("private path symlink");

        #[cfg(unix)]
        assert_eq!(
            state
                .create_private_directory(Path::new("repositories/redirect/child"))
                .unwrap_err()
                .code(),
            "state.symlink"
        );
    }

    #[cfg(unix)]
    #[test]
    fn existing_private_directories_and_files_require_exact_modes() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let directory = fixture.root().join("existing");
        fs::create_dir(&directory).expect("existing private directory");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o500))
            .expect("owner-only wrong-mode directory");
        assert_eq!(
            state
                .create_private_directory(Path::new("existing/child"))
                .unwrap_err()
                .code(),
            "state.permissions"
        );

        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .expect("correct private directory mode");
        let file = directory.join("value");
        fs::write(&file, b"private").expect("existing private file");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o400))
            .expect("owner-only wrong-mode file");
        assert_eq!(
            state
                .snapshot_private_file(Path::new("existing/value"))
                .unwrap_err()
                .code(),
            "state.permissions"
        );
    }

    #[test]
    fn create_only_does_not_replace_an_existing_private_file() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("registry/current.json");
        state
            .write_private_atomic(relative, b"first", ReplacePolicy::CreateOnly)
            .expect("initial private file");
        assert_eq!(
            state
                .write_private_atomic(relative, b"second", ReplacePolicy::CreateOnly)
                .unwrap_err()
                .code(),
            "state.exists"
        );
        assert_eq!(fs::read(fixture.root().join(relative)).unwrap(), b"first");
    }

    #[test]
    fn compare_and_replace_succeeds_for_snapshot_and_rejects_content_conflict() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("registry/current.json");
        state
            .write_private_atomic(relative, b"first", ReplacePolicy::CreateOnly)
            .expect("initial private file");
        let first = state
            .snapshot_private_file(relative)
            .expect("first snapshot");
        state
            .write_private_atomic(relative, b"second", ReplacePolicy::CompareAndReplace(first))
            .expect("compare and replace");

        let stale = state
            .snapshot_private_file(relative)
            .expect("second snapshot");
        fs::write(fixture.root().join(relative), b"external").expect("external file change");
        assert_eq!(
            state
                .write_private_atomic(relative, b"third", ReplacePolicy::CompareAndReplace(stale),)
                .unwrap_err()
                .code(),
            "state.conflict"
        );
        assert_eq!(
            fs::read(fixture.root().join(relative)).unwrap(),
            b"external"
        );
    }

    #[cfg(unix)]
    #[test]
    fn compare_and_replace_rejects_same_content_with_different_identity() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("registry/current.json");
        state
            .write_private_atomic(relative, b"same", ReplacePolicy::CreateOnly)
            .expect("initial private file");
        let stale = state
            .snapshot_private_file(relative)
            .expect("private snapshot");
        let destination = fixture.root().join(relative);
        fs::remove_file(&destination).expect("remove snapshotted file");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        std::io::Write::write_all(
            &mut options.open(&destination).expect("replacement file"),
            b"same",
        )
        .expect("replacement bytes");

        assert_eq!(
            state
                .write_private_atomic(relative, b"next", ReplacePolicy::CompareAndReplace(stale),)
                .unwrap_err()
                .code(),
            "state.conflict"
        );
    }

    #[cfg(unix)]
    #[test]
    fn compare_and_replace_never_overwrites_a_post_replace_racer() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("registry/current.json");
        state
            .write_private_atomic(relative, b"initial", ReplacePolicy::CreateOnly)
            .expect("initial private file");
        let snapshot = state
            .snapshot_private_file(relative)
            .expect("private snapshot");

        assert_eq!(
            state
                .write_private_atomic_with_mutation_hook(
                    relative,
                    b"replacement",
                    ReplacePolicy::CompareAndReplace(snapshot),
                    || {
                        let destination = fixture.root().join(relative);
                        let raced = fixture.root().join("registry/raced.json");
                        fs::write(&raced, b"raced").expect("raced bytes");
                        fs::set_permissions(&raced, fs::Permissions::from_mode(0o600))
                            .expect("private raced file");
                        fs::rename(&raced, &destination).expect("replace destination after check");
                    },
                )
                .unwrap_err()
                .code(),
            "state.conflict"
        );
        assert_eq!(fs::read(fixture.root().join(relative)).unwrap(), b"raced");
        assert_eq!(
            fs::read_dir(fixture.root().join("registry"))
                .expect("registry directory")
                .filter_map(Result::ok)
                .filter(|entry| entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".harp-file"))
                .count(),
            0
        );
    }

    #[test]
    fn immutable_tree_is_idempotent_for_identical_bytes_and_rejects_conflicts() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("releases/sha256-example");
        let members = BTreeMap::from([
            (PathBuf::from("identity.json"), b"identity".to_vec()),
            (PathBuf::from("nested/items.jsonl"), b"item\n".to_vec()),
        ]);
        state
            .publish_immutable_tree(relative, &members)
            .expect("initial immutable tree");
        state
            .publish_immutable_tree(relative, &members)
            .expect("idempotent immutable tree");

        let mut conflicting = members.clone();
        conflicting.insert(PathBuf::from("identity.json"), b"different".to_vec());
        assert_eq!(
            state
                .publish_immutable_tree(relative, &conflicting)
                .unwrap_err()
                .code(),
            "state.immutable_collision"
        );
        assert_eq!(
            fs::read(fixture.root().join(relative).join("identity.json")).unwrap(),
            b"identity"
        );
        let release_entries = fs::read_dir(fixture.root().join("releases"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(release_entries, vec!["sha256-example"]);
        assert!(fixture.root().join(".harp-internal/staging").is_dir());
    }

    #[test]
    fn public_state_apis_reject_the_internal_namespace() {
        let fixture = StateFixture::new();
        let state = fixture.open();

        assert_eq!(
            state
                .create_private_directory(Path::new(".harp-internal/caller"))
                .unwrap_err()
                .code(),
            "state.path"
        );
        assert_eq!(
            state
                .publish_immutable_tree(
                    Path::new(".harp-internal/caller"),
                    &BTreeMap::from([(PathBuf::from("value"), b"value".to_vec())]),
                )
                .unwrap_err()
                .code(),
            "state.path"
        );
        assert_eq!(
            state
                .read_private_file_bounded(
                    Path::new(".harp-internal/release-authorizations/value"),
                    16,
                )
                .unwrap_err()
                .code(),
            "state.path"
        );
    }

    #[test]
    fn release_authorization_is_create_only_and_descriptor_relative() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let release_id = "sha256-authorization";

        state
            .publish_release_authorization(release_id, b"authorized")
            .unwrap();
        state
            .publish_release_authorization(release_id, b"authorized")
            .unwrap();
        assert_eq!(
            state
                .publish_release_authorization(release_id, b"forged")
                .unwrap_err()
                .code(),
            "state.immutable_collision"
        );
        assert_eq!(
            state.read_release_authorization(release_id, 32).unwrap(),
            b"authorized"
        );
    }

    #[test]
    fn invalid_immutable_member_never_exposes_partial_final_tree() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("releases/sha256-invalid");
        let members = BTreeMap::from([
            (PathBuf::from("member"), b"file".to_vec()),
            (PathBuf::from("member/child"), b"child".to_vec()),
        ]);
        assert_eq!(
            state
                .publish_immutable_tree(relative, &members)
                .unwrap_err()
                .code(),
            "state.path"
        );
        assert!(!fixture.root().join(relative).exists());
    }

    #[test]
    fn concurrent_identical_publications_are_idempotent() {
        let fixture = StateFixture::new();
        let root = fixture.root();
        StateRoot::open_or_create(&root).expect("state root");
        let barrier = Arc::new(Barrier::new(3));
        let members = Arc::new(BTreeMap::from([(
            PathBuf::from("identity.json"),
            b"identity".to_vec(),
        )]));
        let handles = (0..2)
            .map(|_| {
                let root = root.clone();
                let barrier = Arc::clone(&barrier);
                let members = Arc::clone(&members);
                thread::spawn(move || {
                    let state = StateRoot::open_or_create(&root).expect("thread state root");
                    barrier.wait();
                    state.publish_immutable_tree(Path::new("releases/sha256-concurrent"), &members)
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        for handle in handles {
            handle
                .join()
                .expect("publication thread")
                .expect("idempotent publication");
        }
    }

    #[test]
    fn concurrent_conflicting_publications_preserve_one_complete_tree() {
        let fixture = StateFixture::new();
        let root = fixture.root();
        StateRoot::open_or_create(&root).expect("state root");
        let barrier = Arc::new(Barrier::new(3));
        let handles = [b"first".to_vec(), b"second".to_vec()]
            .into_iter()
            .map(|bytes| {
                let root = root.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    let state = StateRoot::open_or_create(&root).expect("thread state root");
                    let members = BTreeMap::from([(PathBuf::from("identity.json"), bytes.clone())]);
                    barrier.wait();
                    (
                        bytes,
                        state.publish_immutable_tree(
                            Path::new("releases/sha256-conflict"),
                            &members,
                        ),
                    )
                })
            })
            .collect::<Vec<_>>();
        barrier.wait();
        let results = handles
            .into_iter()
            .map(|handle| handle.join().expect("publication thread"))
            .collect::<Vec<_>>();
        assert_eq!(
            results.iter().filter(|(_, result)| result.is_ok()).count(),
            1
        );
        assert_eq!(
            results
                .iter()
                .filter_map(|(_, result)| result.as_ref().err())
                .map(|error| error.code())
                .collect::<Vec<_>>(),
            vec!["state.immutable_collision"]
        );
        let published = fs::read(
            fixture
                .root()
                .join("releases/sha256-conflict/identity.json"),
        )
        .expect("published identity");
        assert!(published == b"first" || published == b"second");
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_publication_lock_is_rejected() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        state
            .create_private_directory(Path::new(".locks"))
            .expect("lock root");
        let target = Path::new("releases/sha256-locked");
        let lock = state.lock_path(target).expect("publication lock path");
        let outside = fixture.base.join("outside-lock");
        fs::create_dir(&outside).expect("outside lock");
        std::os::unix::fs::symlink(&outside, &lock).expect("symlinked lock");
        let members = BTreeMap::from([(PathBuf::from("identity.json"), b"identity".to_vec())]);
        assert_eq!(
            state
                .publish_immutable_tree(target, &members)
                .unwrap_err()
                .code(),
            "state.symlink"
        );
    }

    #[test]
    fn lock_cleanup_preserves_a_different_recorded_holder() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let target = Path::new("registry/current.json");
        let mut guard = state
            .acquire_publication_lock(target)
            .expect("publication lock");
        let holder = guard.holder();
        let replacement = HolderIdentity {
            process: holder.process,
            nonce: holder.nonce.wrapping_add(1),
        };
        guard
            .replace_holder(replacement)
            .expect("replace holder identity");
        assert_eq!(guard.release().unwrap_err().code(), "state.lock");
        assert!(guard.exists().unwrap());
        drop(guard);
        assert!(state.lock_path(target).unwrap().exists());
    }

    #[test]
    fn noncanonical_relative_spellings_are_rejected_before_access() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        for relative in [
            Path::new("registry//current.json"),
            Path::new("registry/./current.json"),
            Path::new("registry/current.json/"),
        ] {
            assert_eq!(
                state
                    .write_private_atomic(relative, b"value", ReplacePolicy::CreateOnly)
                    .unwrap_err()
                    .code(),
                "state.path",
                "{}",
                relative.display()
            );
        }
    }

    #[test]
    fn descriptor_relative_reads_are_sorted_bounded_and_private() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let members = BTreeMap::from([
            (PathBuf::from("z.json"), b"z".to_vec()),
            (PathBuf::from("a.json"), b"alpha".to_vec()),
        ]);
        state
            .publish_immutable_tree(Path::new("releases/sha256-example"), &members)
            .expect("immutable release");

        assert_eq!(
            state
                .list_private_directory(Path::new("releases/sha256-example"))
                .unwrap(),
            vec!["a.json".to_owned(), "z.json".to_owned()]
        );
        assert_eq!(
            state
                .read_private_file_bounded(
                    Path::new("releases/sha256-example/a.json"),
                    b"alpha".len(),
                )
                .unwrap(),
            b"alpha"
        );
        assert_eq!(
            state
                .read_private_file_bounded(
                    Path::new("releases/sha256-example/a.json"),
                    b"alpha".len() - 1,
                )
                .unwrap_err()
                .code(),
            "state.size"
        );
    }

    #[cfg(unix)]
    #[test]
    fn descriptor_relative_read_rejects_release_directory_namespace_swap() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let release = Path::new("releases/sha256-example");
        state
            .publish_immutable_tree(
                release,
                &BTreeMap::from([(PathBuf::from("identity.json"), b"approved".to_vec())]),
            )
            .expect("immutable release");

        let outside = fixture.base.join("outside-release");
        fs::create_dir(&outside).expect("outside release");
        fs::set_permissions(&outside, fs::Permissions::from_mode(0o700))
            .expect("private outside release");
        let outside_identity = outside.join("identity.json");
        fs::write(&outside_identity, b"external").expect("outside identity");
        fs::set_permissions(&outside_identity, fs::Permissions::from_mode(0o600))
            .expect("private outside identity");

        let detached = fixture.root().join("releases/sha256-detached");
        let error = state
            .read_private_file_bounded_with_hook(
                Path::new("releases/sha256-example/identity.json"),
                64,
                || {
                    fs::rename(fixture.root().join(release), &detached)
                        .expect("detach approved release");
                    std::os::unix::fs::symlink(
                        &outside,
                        fixture.root().join("releases/sha256-example"),
                    )
                    .expect("replace release with symlink");
                },
            )
            .unwrap_err();

        assert!(
            matches!(error.code(), "state.conflict" | "state.symlink"),
            "unexpected error: {error:?}"
        );
        assert_eq!(fs::read(outside_identity).unwrap(), b"external");
    }

    #[test]
    fn concurrent_cas_rejects_equivalent_noncanonical_spelling() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let relative = Path::new("registry/current.json");
        state
            .write_private_atomic(relative, b"initial", ReplacePolicy::CreateOnly)
            .expect("initial private file");
        let snapshot = state
            .snapshot_private_file(relative)
            .expect("private snapshot");
        let root = fixture.root();
        let barrier = Arc::new(Barrier::new(3));
        let handles = [
            PathBuf::from("registry/current.json"),
            PathBuf::from("registry//current.json"),
        ]
        .into_iter()
        .map(|relative| {
            let root = root.clone();
            let snapshot = snapshot.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let state = StateRoot::open_or_create(&root).expect("thread state root");
                barrier.wait();
                state.write_private_atomic(
                    &relative,
                    b"replacement",
                    ReplacePolicy::CompareAndReplace(snapshot),
                )
            })
        })
        .collect::<Vec<_>>();
        barrier.wait();
        let results = handles
            .into_iter()
            .map(|handle| handle.join().expect("CAS thread"))
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter_map(|result| result.as_ref().err())
                .map(AppError::code)
                .collect::<Vec<_>>(),
            vec!["state.path"]
        );
    }

    #[cfg(unix)]
    #[test]
    fn replaced_parent_namespace_rejects_a_detached_write() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        state
            .create_private_directory(Path::new("repositories/live"))
            .expect("private parent");
        let outside = fixture.base.join("outside-race");
        fs::create_dir(&outside).expect("outside directory");
        fs::set_permissions(&outside, fs::Permissions::from_mode(0o700))
            .expect("private outside directory");

        assert_eq!(
            state
                .write_private_atomic_with_hook(
                    Path::new("repositories/live/manifest.json"),
                    b"private",
                    ReplacePolicy::CreateOnly,
                    || {
                        fs::rename(
                            fixture.root().join("repositories/live"),
                            fixture.root().join("repositories/pinned"),
                        )
                        .expect("move pinned directory");
                        std::os::unix::fs::symlink(
                            &outside,
                            fixture.root().join("repositories/live"),
                        )
                        .expect("replace component with symlink");
                    },
                )
                .unwrap_err()
                .code(),
            "state.symlink"
        );
        assert!(!fixture
            .root()
            .join("repositories/pinned/manifest.json")
            .exists());
        assert!(!outside.join("manifest.json").exists());
    }

    #[cfg(unix)]
    #[test]
    fn replaced_root_namespace_rejects_a_detached_write() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        state
            .create_private_directory(Path::new("registry"))
            .expect("private parent");
        let detached = fixture.base.join("detached-harp");

        assert_eq!(
            state
                .write_private_atomic_with_hook(
                    Path::new("registry/current.json"),
                    b"private",
                    ReplacePolicy::CreateOnly,
                    || {
                        fs::rename(fixture.root(), &detached).expect("detach state root");
                        fs::create_dir(fixture.root()).expect("replacement state root");
                        fs::set_permissions(fixture.root(), fs::Permissions::from_mode(0o700))
                            .expect("private replacement state root");
                    },
                )
                .unwrap_err()
                .code(),
            "state.conflict"
        );
        assert!(!detached.join("registry/current.json").exists());
        assert!(!fixture.root().join("registry/current.json").exists());
    }

    #[cfg(unix)]
    #[test]
    fn replaced_parent_inode_rejects_a_detached_write_as_conflict() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        state
            .create_private_directory(Path::new("repositories/live"))
            .expect("private parent");
        let detached = fixture.root().join("repositories/detached");

        assert_eq!(
            state
                .write_private_atomic_with_hook(
                    Path::new("repositories/live/manifest.json"),
                    b"private",
                    ReplacePolicy::CreateOnly,
                    || {
                        fs::rename(fixture.root().join("repositories/live"), &detached)
                            .expect("detach private parent");
                        fs::create_dir(fixture.root().join("repositories/live"))
                            .expect("replacement private parent");
                        fs::set_permissions(
                            fixture.root().join("repositories/live"),
                            fs::Permissions::from_mode(0o700),
                        )
                        .expect("private replacement parent");
                    },
                )
                .unwrap_err()
                .code(),
            "state.conflict"
        );
        assert!(!detached.join("manifest.json").exists());
        assert!(!fixture
            .root()
            .join("repositories/live/manifest.json")
            .exists());
    }

    #[test]
    fn lock_setup_failure_removes_staged_lock() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        for fault in [LockSetupFault::AfterStage, LockSetupFault::AfterPublish] {
            assert_eq!(
                state
                    .acquire_publication_lock_with_fault(Path::new("registry/current.json"), fault,)
                    .unwrap_err()
                    .code(),
                "state.lock"
            );
            let locks = fixture.root().join(".locks");
            assert!(
                fs::read_dir(locks)
                    .expect("lock directory")
                    .next()
                    .is_none(),
                "{fault:?}"
            );
        }
    }

    #[test]
    fn failed_private_file_setup_removes_harp_temporary_files() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let target = Path::new("registry/current.json");
        for fault in [
            CreationFault::FileChmod,
            CreationFault::FileWrite,
            CreationFault::FileSync,
            CreationFault::FileMetadata,
        ] {
            assert_eq!(
                with_creation_fault(".harp-file", fault, || {
                    state.write_private_atomic(target, b"private", ReplacePolicy::CreateOnly)
                })
                .unwrap_err()
                .code(),
                "state.io",
                "{fault:?}"
            );
            assert!(!fixture.root().join(target).exists(), "{fault:?}");
            assert_no_entry_prefix(&fixture.root().join("registry"), ".harp-file");
        }
    }

    #[test]
    fn failed_holder_replacement_setup_removes_temporary_files() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let target = Path::new("registry/current.json");
        for fault in [
            CreationFault::FileChmod,
            CreationFault::FileWrite,
            CreationFault::FileSync,
            CreationFault::FileMetadata,
        ] {
            let mut lock = state
                .acquire_publication_lock(target)
                .expect("publication lock");
            let holder = lock.holder();
            let replacement = HolderIdentity {
                process: holder.process,
                nonce: holder.nonce.wrapping_add(1),
            };
            assert_eq!(
                with_creation_fault(".holder-replacement", fault, || {
                    lock.replace_holder(replacement)
                })
                .unwrap_err()
                .code(),
                "state.io",
                "{fault:?}"
            );
            assert_no_entry_prefix(
                &state.lock_path(target).expect("lock path"),
                ".holder-replacement",
            );
            lock.release().expect("original holder remains releasable");
        }
    }

    #[test]
    fn post_mkdir_failure_removes_staging_directory() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let root = state.root_directory().expect("root directory");
        assert_eq!(
            with_creation_fault(
                ".harp-stage-failure",
                CreationFault::DirectoryPostMkdir,
                || root.create_staged_directory(".harp-stage-failure"),
            )
            .unwrap_err()
            .code(),
            "state.io"
        );
        assert_no_entry_prefix(&fixture.root(), ".harp-stage-failure");
    }

    #[test]
    fn published_holderless_lock_fails_without_spinning() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let target = Path::new("registry/current.json");
        state
            .create_private_directory(Path::new(".locks"))
            .expect("lock directory");
        let lock = state.lock_path(target).expect("lock path");
        fs::create_dir(&lock).expect("holderless publication lock");
        #[cfg(unix)]
        fs::set_permissions(&lock, fs::Permissions::from_mode(0o700))
            .expect("private holderless publication lock");

        assert_eq!(
            state
                .inspect_publication_lock_for_test(target)
                .unwrap_err()
                .code(),
            "state.lock"
        );
    }

    #[test]
    fn dead_lock_holder_is_quarantined_and_recovered() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let target = Path::new("registry/current.json");
        state
            .create_private_directory(Path::new(".locks"))
            .expect("lock directory");
        let lock = state.lock_path(target).expect("lock path");
        fs::create_dir(&lock).expect("stale lock directory");
        #[cfg(unix)]
        fs::set_permissions(&lock, fs::Permissions::from_mode(0o700)).expect("private stale lock");
        let stale = HolderIdentity {
            process: ProcessIdentity {
                pid: u32::MAX,
                start: 1,
            },
            nonce: 7,
        };
        fs::write(lock.join("holder"), stale.encode()).expect("stale holder");
        #[cfg(unix)]
        fs::set_permissions(lock.join("holder"), fs::Permissions::from_mode(0o600))
            .expect("private stale holder");

        state
            .write_private_atomic(target, b"recovered", ReplacePolicy::CreateOnly)
            .expect("publication after stale-lock recovery");
        assert!(!lock.exists());
        assert_eq!(fs::read(fixture.root().join(target)).unwrap(), b"recovered");
    }

    #[cfg(unix)]
    #[test]
    fn dead_lock_recovery_preserves_a_replaced_holder_inode() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let target = Path::new("registry/current.json");
        state
            .create_private_directory(Path::new(".locks"))
            .expect("lock directory");
        let lock = state.lock_path(target).expect("lock path");
        fs::create_dir(&lock).expect("stale lock directory");
        fs::set_permissions(&lock, fs::Permissions::from_mode(0o700)).expect("private stale lock");
        let stale = HolderIdentity {
            process: ProcessIdentity {
                pid: u32::MAX,
                start: 1,
            },
            nonce: 7,
        };
        let holder = lock.join("holder");
        fs::write(&holder, stale.encode()).expect("stale holder");
        fs::set_permissions(&holder, fs::Permissions::from_mode(0o600))
            .expect("private stale holder");

        assert_eq!(
            state
                .recover_dead_publication_lock_with_hook(target, || {
                    fs::remove_file(&holder).expect("remove inspected holder");
                    fs::write(&holder, stale.encode()).expect("replacement holder");
                    fs::set_permissions(&holder, fs::Permissions::from_mode(0o600))
                        .expect("private replacement holder");
                })
                .unwrap_err()
                .code(),
            "state.lock"
        );
        let remaining_holders = fs::read_dir(fixture.root().join(".locks"))
            .expect("lock directory")
            .map(|entry| entry.expect("lock entry").path().join("holder"))
            .filter(|path| path.exists())
            .collect::<Vec<_>>();
        assert_eq!(remaining_holders.len(), 1);
        assert_eq!(
            fs::read(&remaining_holders[0]).expect("preserved replacement holder"),
            stale.encode()
        );
    }

    #[cfg(unix)]
    #[test]
    fn restrictive_umask_stays_unchanged_during_concurrent_private_creation() {
        const CHILD_MARKER: &str = "HARP_RESTRICTIVE_UMASK_CHILD";
        if std::env::var_os(CHILD_MARKER).is_none() {
            let status = std::process::Command::new(
                std::env::current_exe().expect("current state test executable"),
            )
            .args([
                "--exact",
                "context_control::state::tests::restrictive_umask_stays_unchanged_during_concurrent_private_creation",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_MARKER, "1")
            .status()
            .expect("isolated restrictive-umask test process");
            assert!(status.success(), "isolated restrictive-umask test failed");
            return;
        }

        let fixture = StateFixture::new();
        let previous = unsafe { libc::umask(0o777) };
        struct RestoreUmask(libc::mode_t);
        impl Drop for RestoreUmask {
            fn drop(&mut self) {
                unsafe {
                    libc::umask(self.0);
                }
            }
        }
        let _restore = RestoreUmask(previous);

        let unrelated = fixture.base.join("unrelated");
        fs::create_dir(&unrelated).expect("unrelated directory");
        fs::set_permissions(&unrelated, fs::Permissions::from_mode(0o700))
            .expect("searchable unrelated directory");
        let running = Arc::new(AtomicBool::new(true));
        let started = Arc::new(Barrier::new(5));
        let creators = (0..4)
            .map(|worker| {
                let unrelated = unrelated.clone();
                let running = Arc::clone(&running);
                let started = Arc::clone(&started);
                thread::spawn(move || {
                    started.wait();
                    let mut index = 0;
                    while running.load(AtomicOrdering::Acquire) {
                        let path = unrelated.join(format!("{worker}-{index}"));
                        let mut options = OpenOptions::new();
                        options.write(true).create_new(true).mode(0o666);
                        options.open(path).expect("unrelated file");
                        index += 1;
                    }
                })
            })
            .collect::<Vec<_>>();
        started.wait();

        let state = fixture.open();
        for index in 0..40 {
            state
                .create_private_directory(Path::new(&format!("repositories/repository-{index}")))
                .expect("private directory under restrictive umask");
        }
        state
            .write_private_atomic(
                Path::new("repositories/repository-0/manifest.json"),
                b"private",
                ReplacePolicy::CreateOnly,
            )
            .expect("private file under restrictive umask");
        running.store(false, AtomicOrdering::Release);
        for creator in creators {
            creator.join().expect("unrelated creator");
        }

        assert_eq!(mode(&fixture.root()), 0o700);
        assert_eq!(
            mode(&fixture.root().join("repositories/repository-0")),
            0o700
        );
        assert_eq!(
            mode(
                &fixture
                    .root()
                    .join("repositories/repository-0/manifest.json")
            ),
            0o600
        );
        let unrelated_modes = fs::read_dir(unrelated)
            .expect("unrelated files")
            .map(|entry| mode(&entry.expect("unrelated entry").path()))
            .collect::<Vec<_>>();
        assert!(!unrelated_modes.is_empty());
        assert!(unrelated_modes.iter().all(|mode| *mode == 0o000));
    }

    #[test]
    fn new_state_root_syncs_itself_and_its_parent() {
        let fixture = StateFixture::new();
        take_sync_trace();
        fixture.open();
        assert_eq!(
            take_sync_trace(),
            vec![
                SyncEvent::Directory,
                SyncEvent::Directory,
                SyncEvent::Directory
            ]
        );
    }

    #[test]
    fn new_directory_syncs_itself_and_its_parent() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        take_sync_trace();
        state
            .create_private_directory(Path::new("repositories"))
            .expect("private directory");
        assert_eq!(
            take_sync_trace(),
            vec![
                SyncEvent::Directory,
                SyncEvent::Directory,
                SyncEvent::Directory
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_winning_directory_publication_is_typed() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let root = state.root_directory().expect("root directory");
        let outside = fixture.base.join("outside-mkdir-race");
        fs::create_dir(&outside).expect("outside directory");
        let staged = root
            .create_staged_directory("race-stage")
            .expect("staged directory");
        std::os::unix::fs::symlink(&outside, fixture.root().join("winner"))
            .expect("winning symlink");
        assert_eq!(
            root.publish_staged_directory(staged, "winner")
                .unwrap_err()
                .code(),
            "state.symlink"
        );
    }

    #[cfg(unix)]
    #[test]
    fn exclusive_file_creation_types_collision_and_symlink_winners() {
        let fixture = StateFixture::new();
        let state = fixture.open();
        let root = state.root_directory().expect("root directory");
        let collision = std::ffi::CString::new("collision").unwrap();
        root.create_file(&collision, b"first")
            .expect("initial collision file");
        assert_eq!(
            root.create_file(&collision, b"second").unwrap_err().code(),
            "state.exists"
        );

        let outside = fixture.base.join("outside-create-race");
        fs::write(&outside, b"outside").expect("outside file");
        let symlink = std::ffi::CString::new("symlink-winner").unwrap();
        std::os::unix::fs::symlink(&outside, fixture.root().join("symlink-winner"))
            .expect("symlink winner");
        assert_eq!(
            root.create_file(&symlink, b"private").unwrap_err().code(),
            "state.symlink"
        );
        assert_eq!(fs::read(outside).unwrap(), b"outside");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_process_query_results_fail_closed_except_authoritative_absence() {
        let expected = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        let mut info = unsafe { std::mem::zeroed::<libc::proc_bsdinfo>() };
        info.pbi_start_tvsec = 123;
        info.pbi_start_tvusec = 456;

        assert_eq!(
            classify_macos_process_query(7, 0, libc::ESRCH, &info).expect("absent process"),
            None
        );
        for errno in [libc::EPERM, libc::EACCES, libc::EINTR, libc::EIO] {
            assert_eq!(
                classify_macos_process_query(7, 0, errno, &info)
                    .unwrap_err()
                    .code(),
                "state.lock",
                "errno {errno}"
            );
        }
        assert_eq!(
            classify_macos_process_query(7, expected - 1, 0, &info)
                .unwrap_err()
                .code(),
            "state.lock"
        );
        assert_eq!(
            classify_macos_process_query(7, expected, 0, &info).expect("complete process identity"),
            Some(ProcessIdentity {
                pid: 7,
                start: (123_u128 << 64) | 456,
            })
        );
    }

    #[cfg(unix)]
    fn mode(path: &Path) -> u32 {
        fs::symlink_metadata(path)
            .expect("path metadata")
            .permissions()
            .mode()
            & 0o777
    }

    fn assert_no_entry_prefix(directory: &Path, prefix: &str) {
        assert!(
            fs::read_dir(directory)
                .expect("temporary-entry parent")
                .filter_map(Result::ok)
                .all(|entry| !entry.file_name().to_string_lossy().starts_with(prefix)),
            "found temporary entry with prefix {prefix} in {}",
            directory.display()
        );
    }
}
