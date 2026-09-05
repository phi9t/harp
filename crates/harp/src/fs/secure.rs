use std::fs::{self, File, OpenOptions};
use std::io::Read;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use crate::error::AppError;

pub(crate) use super::descriptor::FileSnapshot;
use super::descriptor::{AnchoredDirectory, FilePolicy, RemovalRequest, RemovalSync};

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
pub struct HeldDirectory {
    root: PathBuf,
    anchored: AnchoredDirectory,
}

impl HeldDirectory {
    pub fn open(root: &Path, label: &str) -> Result<Self, AppError> {
        let metadata =
            fs::symlink_metadata(root).map_err(|error| AppError::io("fs.root", label, error))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(AppError::invalid_input(
                "fs.root",
                format!("{label} must be a real directory"),
            ));
        }
        Ok(Self {
            root: root.to_path_buf(),
            anchored: AnchoredDirectory::open(root, label)?,
        })
    }

    pub(crate) fn anchored_directory(&self, label: &str) -> Result<AnchoredDirectory, AppError> {
        self.anchored.duplicate(label)
    }

    pub fn read_optional_regular_file_bounded(
        &self,
        relative: &Path,
        label: &str,
        max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AppError> {
        match self.open_regular_file(relative, label) {
            Ok(file) => {
                let length = file
                    .metadata()
                    .map_err(|error| AppError::io("fs.metadata", label, error))?
                    .len();
                if length > max_bytes as u64 {
                    return Err(AppError::invalid_input(
                        "fs.size",
                        format!("{label} exceeds {max_bytes} bytes"),
                    ));
                }
                let mut bytes = Vec::with_capacity(length as usize);
                file.take(max_bytes as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|error| AppError::io("fs.read", label, error))?;
                if bytes.len() > max_bytes {
                    return Err(AppError::invalid_input(
                        "fs.size",
                        format!("{label} exceeds {max_bytes} bytes"),
                    ));
                }
                Ok(Some(bytes))
            }
            Err(error) if error.code == "fs.missing" => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn read_optional_regular_single_link_file_bounded(
        &self,
        relative: &Path,
        label: &str,
        max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, AppError> {
        let (parent, name) = self.anchored.walk_parent(relative, label)?;
        parent
            .read_optional_bounded(
                &name,
                label,
                max_bytes,
                FilePolicy {
                    owner: None,
                    mode: None,
                    nlink: Some(1),
                },
            )
            .map_err(|error| single_link_policy_error(error, label))
            .map(|observed| observed.map(|(bytes, _snapshot)| bytes))
    }

    #[cfg(test)]
    fn read_optional_regular_single_link_file_snapshot_bounded(
        &self,
        relative: &Path,
        label: &str,
        max_bytes: usize,
    ) -> Result<Option<(Vec<u8>, FileSnapshot)>, AppError> {
        let (parent, name) = self.anchored.walk_parent(relative, label)?;
        parent.read_optional_bounded(
            &name,
            label,
            max_bytes,
            FilePolicy {
                owner: None,
                mode: None,
                nlink: Some(1),
            },
        )
    }

    pub fn read_optional_regular_file_snapshot_bounded(
        &self,
        relative: &Path,
        label: &str,
        max_bytes: usize,
    ) -> Result<Option<FileSnapshot>, AppError> {
        let (parent, name) = self.anchored.walk_parent(relative, label)?;
        parent
            .read_optional_bounded(&name, label, max_bytes, FilePolicy::REGULAR)
            .map(|observed| observed.map(|(_, snapshot)| snapshot))
    }

    pub fn regular_file_or_directory_exists(
        &self,
        relative: &Path,
        label: &str,
    ) -> Result<bool, AppError> {
        let path = self.resolve_read_target(relative, label)?;
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() || metadata.is_dir() => Ok(true),
            Ok(_) => Ok(false),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(AppError::io("fs.metadata", label, error)),
        }
    }

    pub fn regular_file_exists(&self, relative: &Path, label: &str) -> Result<bool, AppError> {
        let path = self.resolve_read_target(relative, label)?;
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => Ok(true),
            Ok(_) => Ok(false),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(AppError::io("fs.metadata", label, error)),
        }
    }

    pub fn regular_files_with_extension(
        &self,
        relative: &Path,
        extension: &str,
        label: &str,
    ) -> Result<Vec<PathBuf>, AppError> {
        let root = self.resolve_read_target(relative, label)?;
        let metadata = fs::symlink_metadata(&root)
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if !metadata.is_dir() {
            return Err(AppError::invalid_input(
                "fs.type",
                format!("{label} must be a real directory"),
            ));
        }

        let mut files = Vec::new();
        self.collect_regular_files_with_extension(relative, extension, label, false, &mut files)?;
        files.sort();
        Ok(files)
    }

    pub fn bounded_regular_single_link_file_snapshots_with_extension(
        &self,
        relative: &Path,
        extension: &str,
        label: &str,
        max_file_bytes: usize,
        limits: FileTreeLimits,
    ) -> Result<Vec<(PathBuf, Vec<u8>)>, AppError> {
        self.bounded_regular_single_link_file_snapshots_with_extension_inner(
            relative,
            extension,
            label,
            max_file_bytes,
            limits,
            || {},
        )
    }

    #[cfg(test)]
    fn bounded_regular_single_link_file_snapshots_with_extension_with_hook(
        &self,
        relative: &Path,
        extension: &str,
        label: &str,
        max_file_bytes: usize,
        limits: FileTreeLimits,
        before_first_read: impl FnOnce(),
    ) -> Result<Vec<(PathBuf, Vec<u8>)>, AppError> {
        self.bounded_regular_single_link_file_snapshots_with_extension_inner(
            relative,
            extension,
            label,
            max_file_bytes,
            limits,
            before_first_read,
        )
    }

    fn bounded_regular_single_link_file_snapshots_with_extension_inner(
        &self,
        relative: &Path,
        extension: &str,
        label: &str,
        max_file_bytes: usize,
        limits: FileTreeLimits,
        before_first_read: impl FnOnce(),
    ) -> Result<Vec<(PathBuf, Vec<u8>)>, AppError> {
        let root = self.anchored.walk(relative, label)?;
        let mut files = Vec::new();
        let mut visited_entries = 0usize;
        let mut aggregate_bytes = 0usize;
        let mut pending = vec![(root, relative.to_path_buf(), 0usize)];
        let mut before_first_read = Some(before_first_read);
        while let Some((directory, relative_directory, depth)) = pending.pop() {
            let remaining_entries = limits.max_entries.saturating_sub(visited_entries);
            let entries = directory.entry_names_bounded(label, remaining_entries)?;
            visited_entries = visited_entries.checked_add(entries.len()).ok_or_else(|| {
                AppError::invalid_input("fs.limit", format!("{label} entry count overflowed"))
            })?;
            for entry in entries {
                let child = relative_directory.join(std::ffi::OsStr::from_bytes(entry.to_bytes()));
                let child_depth = depth + 1;
                if child_depth > limits.max_depth {
                    return Err(AppError::invalid_input(
                        "fs.limit",
                        format!(
                            "{label} exceeds depth {}: {}",
                            limits.max_depth,
                            child.display()
                        ),
                    ));
                }
                let matches_extension =
                    child.extension().and_then(|value| value.to_str()) == Some(extension);
                if matches_extension {
                    if files.len() >= limits.max_matching_files {
                        return Err(AppError::invalid_input(
                            "fs.limit",
                            format!(
                                "{label} exceeds {} matching files",
                                limits.max_matching_files
                            ),
                        ));
                    }
                    if let Some(hook) = before_first_read.take() {
                        hook();
                    }
                    let (bytes, _snapshot) = directory
                        .read_optional_bounded(
                            &entry,
                            label,
                            max_file_bytes,
                            FilePolicy {
                                owner: None,
                                mode: None,
                                nlink: Some(1),
                            },
                        )
                        .map_err(|error| {
                            file_tree_entry_error(single_link_policy_error(error, label), &child)
                        })?
                        .ok_or_else(|| {
                            AppError::invalid_input(
                                "fs.concurrent_change",
                                format!("{label} entry disappeared: {}", child.display()),
                            )
                        })?;
                    aggregate_bytes =
                        aggregate_bytes.checked_add(bytes.len()).ok_or_else(|| {
                            AppError::invalid_input(
                                "fs.limit",
                                format!("{label} aggregate size overflowed"),
                            )
                        })?;
                    if aggregate_bytes > limits.max_aggregate_bytes {
                        return Err(AppError::invalid_input(
                            "fs.limit",
                            format!(
                                "{label} exceeds {} aggregate bytes",
                                limits.max_aggregate_bytes
                            ),
                        ));
                    }
                    files.push((child, bytes));
                } else {
                    match directory.open_optional_directory(&entry, label) {
                        Ok(Some(child_directory)) => {
                            pending.push((child_directory, child, child_depth));
                        }
                        Ok(None) => {
                            return Err(AppError::invalid_input(
                                "fs.concurrent_change",
                                format!("{label} entry disappeared: {}", child.display()),
                            ));
                        }
                        Err(error) if error.code() == "fs.type" => {}
                        Err(error) => return Err(error),
                    }
                }
            }
        }
        files.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(files)
    }

    pub fn compare_and_replace_public_regular_file(
        &self,
        relative: &Path,
        bytes: &[u8],
        expected: Option<&FileSnapshot>,
        label: &str,
        max_bytes: usize,
    ) -> Result<FileSnapshot, AppError> {
        self.compare_and_replace_public_regular_file_inner(
            PublicFileMutation {
                relative,
                bytes,
                expected,
                label,
                max_bytes,
                required_mode: None,
            },
            || {},
            |_| Ok(()),
        )
    }

    pub(crate) fn compare_and_replace_public_regular_file_with_mode_and_events(
        &self,
        mutation: PublicFileMutation<'_>,
        events: impl FnMut(PublicMutationEvent) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        self.compare_and_replace_public_regular_file_inner(mutation, || {}, events)
    }

    #[cfg(test)]
    fn compare_and_replace_public_regular_file_with_hook(
        &self,
        relative: &Path,
        bytes: &[u8],
        expected: Option<&FileSnapshot>,
        label: &str,
        max_bytes: usize,
        hook: impl FnOnce(),
    ) -> Result<FileSnapshot, AppError> {
        self.compare_and_replace_public_regular_file_inner(
            PublicFileMutation {
                relative,
                bytes,
                expected,
                label,
                max_bytes,
                required_mode: None,
            },
            hook,
            |_| Ok(()),
        )
    }

    fn compare_and_replace_public_regular_file_inner(
        &self,
        mutation: PublicFileMutation<'_>,
        hook: impl FnOnce(),
        mut events: impl FnMut(PublicMutationEvent) -> Result<(), AppError>,
    ) -> Result<FileSnapshot, AppError> {
        let PublicFileMutation {
            relative,
            bytes,
            expected,
            label,
            max_bytes,
            required_mode,
        } = mutation;
        if bytes.len() > max_bytes {
            return Err(AppError::invalid_input(
                "fs.size",
                format!("{label} exceeds {max_bytes} bytes"),
            ));
        }
        let (parent, name) = self.anchored.walk_parent(relative, label)?;
        let policy = FilePolicy {
            owner: Some(unsafe { libc::geteuid() }),
            mode: None,
            nlink: Some(1),
        };
        let initial = parent
            .read_optional_bounded(&name, label, max_bytes, policy)?
            .map(|(_, snapshot)| snapshot);
        if initial.as_ref() != expected {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} changed while it was being generated"),
            ));
        }
        hook();
        let current = parent
            .read_optional_bounded(&name, label, max_bytes, policy)?
            .map(|(_, snapshot)| snapshot);
        if current.as_ref() != expected {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} changed while it was being generated"),
            ));
        }

        let mode =
            required_mode.unwrap_or_else(|| expected.map_or(0o644, |snapshot| snapshot.mode));
        if expected.is_some_and(|snapshot| snapshot.mode != mode) {
            return Err(AppError::invalid_input(
                "fs.permissions",
                format!("{label} does not have required mode {mode:04o}"),
            ));
        }
        events(PublicMutationEvent {
            operation: PublicMutationOperation::FileSync,
            phase: PublicMutationPhase::Before,
        })?;
        let mut staged = parent.create_staged_file_with_cleanup_sync_hook(
            ".harp-public",
            bytes,
            mode,
            label,
            |before| {
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::CleanupSync,
                    phase: if before {
                        PublicMutationPhase::Before
                    } else {
                        PublicMutationPhase::After
                    },
                })
            },
        )?;
        if let Err(error) = events(PublicMutationEvent {
            operation: PublicMutationOperation::FileSync,
            phase: PublicMutationPhase::After,
        }) {
            staged.cleanup_with_parent_sync(label, |before| {
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::CleanupSync,
                    phase: if before {
                        PublicMutationPhase::Before
                    } else {
                        PublicMutationPhase::After
                    },
                })
            })?;
            return Err(error);
        }
        if let Some(expected) = expected {
            if let Err(error) = events(PublicMutationEvent {
                operation: PublicMutationOperation::Rename,
                phase: PublicMutationPhase::Before,
            }) {
                staged.cleanup_with_parent_sync(label, |before| {
                    events(PublicMutationEvent {
                        operation: PublicMutationOperation::CleanupSync,
                        phase: if before {
                            PublicMutationPhase::Before
                        } else {
                            PublicMutationPhase::After
                        },
                    })
                })?;
                return Err(error);
            }
            let installed = staged.snapshot().clone();
            if let Err(error) = parent.exchange_staged(&staged, &name, expected.identity, label) {
                staged.cleanup_with_parent_sync(label, |before| {
                    events(PublicMutationEvent {
                        operation: PublicMutationOperation::CleanupSync,
                        phase: if before {
                            PublicMutationPhase::Before
                        } else {
                            PublicMutationPhase::After
                        },
                    })
                })?;
                return Err(error);
            }
            let exchanged = (|| -> Result<FileSnapshot, AppError> {
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::Rename,
                    phase: PublicMutationPhase::After,
                })?;
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::ExchangeValidation,
                    phase: PublicMutationPhase::Before,
                })?;
                let displaced = parent
                    .read_optional_bounded(staged.name(), label, max_bytes, policy)?
                    .ok_or_else(|| {
                        AppError::invalid_input(
                            "fs.concurrent_change",
                            format!("{label} disappeared during exchange"),
                        )
                    })?
                    .1;
                let observed = parent
                    .read_optional_bounded(&name, label, max_bytes, policy)?
                    .ok_or_else(|| {
                        AppError::invalid_input(
                            "fs.concurrent_change",
                            format!("{label} disappeared after exchange"),
                        )
                    })?
                    .1;
                if !displaced.matches_moved_object(expected)
                    || !observed.matches_moved_object(&installed)
                {
                    return Err(AppError::invalid_input(
                        "fs.concurrent_change",
                        format!("{label} identity changed during exchange"),
                    ));
                }
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::ExchangeValidation,
                    phase: PublicMutationPhase::After,
                })?;
                staged.replace_snapshot(expected.clone());
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::ParentSync,
                    phase: PublicMutationPhase::Before,
                })?;
                parent.sync(label)?;
                let _ = events(PublicMutationEvent {
                    operation: PublicMutationOperation::ParentSync,
                    phase: PublicMutationPhase::After,
                });
                Ok(observed)
            })();
            let installed = match exchanged {
                Ok(installed) => installed,
                Err(error) => {
                    // Pre-commit recovery uses the identities captured before the exchange,
                    // never a fallible post-exchange pathname read.
                    if let Err(rollback) = parent.exchange_entries_if_identities(
                        staged.name(),
                        expected.identity,
                        &name,
                        installed.identity,
                        label,
                    ) {
                        staged.disarm();
                        return Err(AppError::external(
                            "fs.rollback",
                            format!(
                            "{label} pre-commit rollback is ambiguous; later pointer preserved: {}",
                            rollback.message
                        ),
                        ));
                    }
                    // The staged name now holds the original new pointer. Disarm before
                    // removal so an injected cleanup failure cannot fall through Drop.
                    staged.replace_snapshot(installed);
                    staged.disarm();
                    let cleanup = (|| -> Result<(), AppError> {
                        parent.remove_name(staged.name(), staged.snapshot().identity, label)?;
                        events(PublicMutationEvent {
                            operation: PublicMutationOperation::RollbackSync,
                            phase: PublicMutationPhase::Before,
                        })?;
                        parent.sync(label)?;
                        events(PublicMutationEvent {
                            operation: PublicMutationOperation::RollbackSync,
                            phase: PublicMutationPhase::After,
                        })
                    })();
                    if let Err(rollback) = cleanup {
                        return Err(AppError::external("fs.rollback", format!("{label} restored the prior pointer but cleanup synchronization failed: {}", rollback.message)));
                    }
                    return Err(error);
                }
            };
            // The pointer-parent synchronization above is the commit point.  The old
            // pointer may therefore be left as a quarantined orphan if its cleanup
            // cannot be durably synchronized; reporting that as a failed switch
            // would incorrectly imply the prior pointer still won.
            staged.disarm();
            let _ = (|| -> Result<(), AppError> {
                parent.remove_name(staged.name(), expected.identity, label)?;
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::CleanupSync,
                    phase: PublicMutationPhase::Before,
                })?;
                parent.sync(label)?;
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::CleanupSync,
                    phase: PublicMutationPhase::After,
                })
            })();
            Ok(installed)
        } else {
            if let Err(error) = events(PublicMutationEvent {
                operation: PublicMutationOperation::Rename,
                phase: PublicMutationPhase::Before,
            }) {
                staged.cleanup_with_parent_sync(label, |before| {
                    events(PublicMutationEvent {
                        operation: PublicMutationOperation::CleanupSync,
                        phase: if before {
                            PublicMutationPhase::Before
                        } else {
                            PublicMutationPhase::After
                        },
                    })
                })?;
                return Err(error);
            }
            let installed = match parent.publish_staged_noreplace(&mut staged, &name, label) {
                Ok(installed) => installed,
                Err(error) => {
                    staged.cleanup_with_parent_sync(label, |before| {
                        events(PublicMutationEvent {
                            operation: PublicMutationOperation::CleanupSync,
                            phase: if before {
                                PublicMutationPhase::Before
                            } else {
                                PublicMutationPhase::After
                            },
                        })
                    })?;
                    return Err(error);
                }
            };
            let durable = (|| -> Result<FileSnapshot, AppError> {
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::Rename,
                    phase: PublicMutationPhase::After,
                })?;
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::InstallValidation,
                    phase: PublicMutationPhase::Before,
                })?;
                let observed = parent
                    .read_optional_bounded(&name, label, max_bytes, policy)?
                    .ok_or_else(|| {
                        AppError::invalid_input(
                            "fs.concurrent_change",
                            format!("{label} disappeared during initial publication"),
                        )
                    })?
                    .1;
                if !observed.matches_moved_object(&installed) {
                    return Err(AppError::invalid_input(
                        "fs.concurrent_change",
                        format!("{label} changed during initial publication"),
                    ));
                }
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::InstallValidation,
                    phase: PublicMutationPhase::After,
                })?;
                events(PublicMutationEvent {
                    operation: PublicMutationOperation::ParentSync,
                    phase: PublicMutationPhase::Before,
                })?;
                parent.sync(label)?;
                let _ = events(PublicMutationEvent {
                    operation: PublicMutationOperation::ParentSync,
                    phase: PublicMutationPhase::After,
                });
                Ok(observed)
            })();
            let installed = match durable {
                Ok(installed) => installed,
                Err(error) => {
                    // A no-replace rename has no displaced pointer to exchange back.
                    // Remove only the pre-rename identity; a later replacement remains
                    // authoritative and is reported as rollback ambiguity.
                    let rollback = (|| -> Result<(), AppError> {
                        parent.remove_name(&name, installed.identity, label)?;
                        events(PublicMutationEvent {
                            operation: PublicMutationOperation::CleanupSync,
                            phase: PublicMutationPhase::Before,
                        })?;
                        parent.sync(label)?;
                        events(PublicMutationEvent {
                            operation: PublicMutationOperation::CleanupSync,
                            phase: PublicMutationPhase::After,
                        })
                    })();
                    if let Err(rollback) = rollback {
                        return Err(AppError::external(
                        "fs.rollback",
                        format!(
                            "{label} initial publication rollback is ambiguous; later pointer preserved: {}",
                            rollback.message
                        ),
                    ));
                    }
                    return Err(error);
                }
            };
            Ok(installed)
        }
    }

    #[allow(dead_code)] // retained public mutation surface; event-aware callers use the variant below
    pub fn compare_and_remove_public_regular_file(
        &self,
        relative: &Path,
        expected: &FileSnapshot,
        label: &str,
        max_bytes: usize,
    ) -> Result<(), AppError> {
        self.compare_and_remove_public_regular_file_inner(
            relative,
            expected,
            label,
            max_bytes,
            || {},
        )
    }

    pub(crate) fn compare_and_remove_public_regular_file_with_events(
        &self,
        relative: &Path,
        expected: &FileSnapshot,
        label: &str,
        max_bytes: usize,
        mut events: impl FnMut(PublicMutationEvent) -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        let (parent, name) = self.anchored.walk_parent(relative, label)?;
        parent.remove_if_snapshot_with_hooks_and_sync(
            RemovalRequest {
                name: &name,
                expected,
                label,
                max_bytes,
                policy: FilePolicy {
                    owner: Some(unsafe { libc::geteuid() }),
                    mode: None,
                    nlink: Some(1),
                },
            },
            || {},
            |sync, before| {
                events(PublicMutationEvent {
                    operation: match sync {
                        RemovalSync::Cleanup => PublicMutationOperation::CleanupSync,
                        RemovalSync::Rollback => PublicMutationOperation::RollbackSync,
                    },
                    phase: if before {
                        PublicMutationPhase::Before
                    } else {
                        PublicMutationPhase::After
                    },
                })
            },
        )
    }

    #[cfg(test)]
    fn compare_and_remove_public_regular_file_with_hook(
        &self,
        relative: &Path,
        expected: &FileSnapshot,
        label: &str,
        max_bytes: usize,
        hook: impl FnOnce(),
    ) -> Result<(), AppError> {
        self.compare_and_remove_public_regular_file_inner(
            relative, expected, label, max_bytes, hook,
        )
    }

    fn compare_and_remove_public_regular_file_inner(
        &self,
        relative: &Path,
        expected: &FileSnapshot,
        label: &str,
        max_bytes: usize,
        hook: impl FnOnce(),
    ) -> Result<(), AppError> {
        let (parent, name) = self.anchored.walk_parent(relative, label)?;
        parent.remove_if_snapshot_with_hook(
            &name,
            expected,
            label,
            max_bytes,
            FilePolicy {
                owner: Some(unsafe { libc::geteuid() }),
                mode: None,
                nlink: Some(1),
            },
            hook,
        )
    }

    fn open_regular_file(&self, relative: &Path, label: &str) -> Result<File, AppError> {
        let path = self.resolve_read_target(relative, label)?;
        let before = fs::symlink_metadata(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::invalid_input("fs.missing", format!("{label} is missing"))
            } else {
                AppError::io("fs.metadata", label, error)
            }
        })?;
        if before.file_type().is_symlink() || !before.is_file() {
            return Err(AppError::invalid_input(
                "fs.type",
                format!("{label} must be a regular file"),
            ));
        }
        self.open_prevalidated_regular_file(&path, label, &before)
    }

    fn open_prevalidated_regular_file(
        &self,
        path: &Path,
        label: &str,
        before: &fs::Metadata,
    ) -> Result<File, AppError> {
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                AppError::invalid_input("fs.missing", format!("{label} is missing"))
            } else {
                AppError::io("fs.open", label, error)
            }
        })?;
        let metadata = file
            .metadata()
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if !metadata.is_file() {
            return Err(AppError::invalid_input(
                "fs.type",
                format!("{label} must be a regular file"),
            ));
        }
        if !same_file(before, &metadata) || !same_link_state(before, &metadata) {
            return Err(AppError::invalid_input(
                "fs.changed",
                format!("{label} changed while it was being opened"),
            ));
        }
        Ok(file)
    }

    fn collect_regular_files_with_extension(
        &self,
        relative: &Path,
        extension: &str,
        label: &str,
        require_single_link: bool,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), AppError> {
        let directory = self.resolve_read_target(relative, label)?;
        let mut entries = fs::read_dir(&directory)
            .map_err(|error| AppError::io("fs.read_dir", label, error))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| AppError::io("fs.read_dir", label, error))?;
        entries.sort_by_key(|entry| entry.file_name());

        for entry in entries {
            let name = entry.file_name();
            let child = relative.join(&name);
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| AppError::io("fs.metadata", label, error))?;
            if metadata.file_type().is_symlink() {
                return Err(AppError::invalid_input(
                    "fs.symlink",
                    format!("{label} cannot traverse a symlink: {}", child.display()),
                ));
            }
            let matches_extension =
                child.extension().and_then(|value| value.to_str()) == Some(extension);
            if require_single_link
                && matches_extension
                && (!metadata.is_file() || !has_single_link(&metadata))
            {
                return Err(AppError::invalid_input(
                    "fs.type",
                    format!(
                        "{label} requires regular single-link {extension} files: {}",
                        child.display()
                    ),
                ));
            }
            if metadata.is_dir() {
                self.collect_regular_files_with_extension(
                    &child,
                    extension,
                    label,
                    require_single_link,
                    files,
                )?;
            } else if metadata.is_file() && matches_extension {
                files.push(child);
            }
        }
        Ok(())
    }

    fn resolve_read_target(&self, relative: &Path, label: &str) -> Result<PathBuf, AppError> {
        validate_relative(relative, label)?;
        self.reject_symlink_ancestors(relative, label)?;
        Ok(self.root.join(relative))
    }

    fn reject_symlink_ancestors(&self, relative: &Path, label: &str) -> Result<(), AppError> {
        let mut candidate = self.root.clone();
        for component in relative.components() {
            let Component::Normal(component) = component else {
                return Err(AppError::invalid_input(
                    "fs.path",
                    format!("{label} must be a relative normal path"),
                ));
            };
            candidate.push(component);
            match fs::symlink_metadata(&candidate) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(AppError::invalid_input(
                        "fs.symlink",
                        format!("{label} cannot traverse a symlink"),
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => return Err(AppError::io("fs.metadata", label, error)),
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
fn has_single_link(metadata: &fs::Metadata) -> bool {
    metadata.nlink() == 1
}

#[cfg(not(unix))]
fn has_single_link(_metadata: &fs::Metadata) -> bool {
    true
}

fn single_link_policy_error(error: AppError, label: &str) -> AppError {
    if error.code() == "fs.permissions" {
        AppError::invalid_input(
            "fs.type",
            format!("{label} must be a regular single-link file"),
        )
    } else {
        error
    }
}

fn file_tree_entry_error(error: AppError, relative_path: &Path) -> AppError {
    if error.code() == "fs.size" {
        AppError::invalid_input(
            error.code(),
            format!("{}: {}", relative_path.display(), error.message),
        )
    } else {
        error
    }
}

#[cfg(unix)]
fn same_file(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    before.dev() == after.dev() && before.ino() == after.ino()
}

#[cfg(unix)]
fn same_link_state(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    before.nlink() > 0 && before.nlink() == after.nlink()
}

#[cfg(not(unix))]
fn same_file(_before: &fs::Metadata, _after: &fs::Metadata) -> bool {
    true
}

#[cfg(not(unix))]
fn same_link_state(_before: &fs::Metadata, _after: &fs::Metadata) -> bool {
    true
}

fn validate_relative(relative: &Path, label: &str) -> Result<(), AppError> {
    if relative.is_absolute()
        || relative.as_os_str().is_empty()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(AppError::invalid_input(
            "fs.path",
            format!("{label} must be a relative normal path"),
        ));
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::collections::VecDeque;
    use std::ffi::CString;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;
    use std::path::Path;
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{
        AppError, FileTreeLimits, HeldDirectory, PublicFileMutation, PublicMutationOperation,
        PublicMutationPhase,
    };

    #[test]
    fn single_link_read_rejects_a_hardlink_added_after_discovery() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let packet = temp.path().join("packet");
        fs::create_dir(&packet).expect("packet directory");
        let markdown = packet.join("chapter.md");
        fs::write(&markdown, b"chapter").expect("packet Markdown");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let snapshots = held
            .bounded_regular_single_link_file_snapshots_with_extension(
                Path::new("packet"),
                "md",
                "test packet",
                64,
                FileTreeLimits {
                    max_depth: 2,
                    max_entries: 4,
                    max_matching_files: 2,
                    max_aggregate_bytes: 64,
                },
            )
            .expect("initial discovery");
        assert_eq!(snapshots[0].0, Path::new("packet/chapter.md"));
        assert_eq!(snapshots[0].1, b"chapter");

        fs::hard_link(&markdown, temp.path().join("late-hardlink.md"))
            .expect("add hardlink after discovery");
        let error = held
            .read_optional_regular_single_link_file_bounded(&snapshots[0].0, "test Markdown", 64)
            .expect_err("descriptor read must observe the added link");
        assert_eq!(error.code, "fs.type");
    }

    #[test]
    fn descriptor_snapshot_rejects_root_replacement_before_file_read() {
        let temp = tempfile::tempdir().expect("temporary filesystem root");
        let repository = temp.path().join("repository");
        let replacement = temp.path().join("replacement");
        for (root, bytes) in [
            (&repository, b"original".as_slice()),
            (&replacement, b"outside".as_slice()),
        ] {
            fs::create_dir_all(root.join("packet")).expect("packet directory");
            fs::write(root.join("packet/chapter.md"), bytes).expect("packet Markdown");
        }
        let held = HeldDirectory::open(&repository, "race repository").expect("held root");
        let displaced = temp.path().join("original-repository");

        let error = held
            .bounded_regular_single_link_file_snapshots_with_extension_with_hook(
                Path::new("packet"),
                "md",
                "race packet",
                64,
                FileTreeLimits {
                    max_depth: 2,
                    max_entries: 4,
                    max_matching_files: 2,
                    max_aggregate_bytes: 64,
                },
                || {
                    fs::rename(&repository, &displaced).expect("displace held root");
                    fs::rename(&replacement, &repository).expect("install replacement root");
                },
            )
            .expect_err("replacement root must not redirect descriptor reads");

        assert_eq!(error.code(), "fs.concurrent_change");
        assert_eq!(
            fs::read(repository.join("packet/chapter.md")).unwrap(),
            b"outside"
        );
    }

    #[test]
    fn descriptor_snapshot_rejects_ancestor_replacement_before_file_read() {
        let temp = tempfile::tempdir().expect("temporary filesystem root");
        let ancestor = temp.path().join("ancestor");
        let repository = ancestor.join("repository");
        let replacement_ancestor = temp.path().join("replacement-ancestor");
        let replacement = replacement_ancestor.join("repository");
        for (root, bytes) in [
            (&repository, b"original".as_slice()),
            (&replacement, b"outside".as_slice()),
        ] {
            fs::create_dir_all(root.join("packet")).expect("packet directory");
            fs::write(root.join("packet/chapter.md"), bytes).expect("packet Markdown");
        }
        let held = HeldDirectory::open(&repository, "race repository").expect("held root");
        let displaced = temp.path().join("original-ancestor");

        let error = held
            .bounded_regular_single_link_file_snapshots_with_extension_with_hook(
                Path::new("packet"),
                "md",
                "race packet",
                64,
                FileTreeLimits {
                    max_depth: 2,
                    max_entries: 4,
                    max_matching_files: 2,
                    max_aggregate_bytes: 64,
                },
                || {
                    fs::rename(&ancestor, &displaced).expect("displace held ancestor");
                    fs::rename(&replacement_ancestor, &ancestor)
                        .expect("install replacement ancestor");
                },
            )
            .expect_err("replacement ancestor must not redirect descriptor reads");

        assert_eq!(error.code(), "fs.concurrent_change");
        assert_eq!(
            fs::read(repository.join("packet/chapter.md")).unwrap(),
            b"outside"
        );
    }

    #[test]
    fn regular_file_open_rejects_a_fifo_without_blocking() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let fifo = temp.path().join("input.json");
        let fifo_path = CString::new(fifo.as_os_str().as_bytes()).expect("FIFO path");
        assert_eq!(unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) }, 0);
        let root = temp.path().to_path_buf();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let held = HeldDirectory::open(&root, "test repository").expect("held root");
            sender
                .send(held.read_optional_regular_file_bounded(
                    Path::new("input.json"),
                    "test FIFO",
                    64,
                ))
                .expect("send FIFO result");
        });
        let result = receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("FIFO validation must complete without a writer");
        assert_eq!(result.expect_err("FIFO must fail").code, "fs.type");
    }

    #[test]
    fn regular_file_open_rejects_nonregular_and_symlink_nodes() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        fs::create_dir(temp.path().join("directory.json")).expect("directory node");
        fs::write(temp.path().join("target.json"), b"target").expect("symlink target");
        std::os::unix::fs::symlink("target.json", temp.path().join("symlink.json"))
            .expect("symlink node");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        for path in ["directory.json", "symlink.json"] {
            let error = held
                .read_optional_regular_file_bounded(Path::new(path), "nonregular input", 64)
                .expect_err("nonregular input must fail");
            assert!(matches!(error.code, "fs.type" | "fs.symlink"));
        }
    }

    #[test]
    fn regular_file_open_rejects_a_fifo_substituted_after_lstat() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let input = temp.path().join("input.json");
        fs::write(&input, b"regular").expect("initial regular file");
        let before = fs::symlink_metadata(&input).expect("pre-open metadata");
        fs::remove_file(&input).expect("remove regular file");
        let fifo_path = CString::new(input.as_os_str().as_bytes()).expect("FIFO path");
        assert_eq!(unsafe { libc::mkfifo(fifo_path.as_ptr(), 0o600) }, 0);
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            sender
                .send(held.open_prevalidated_regular_file(&input, "raced FIFO", &before))
                .expect("send raced FIFO result");
        });
        let error = receiver
            .recv_timeout(Duration::from_secs(2))
            .expect("raced FIFO open must not block")
            .expect_err("raced FIFO must fail");
        assert_eq!(error.code, "fs.type");
    }

    #[test]
    fn same_uid_noncooperating_writer_replacement_is_rejected_by_inode_identity() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"same bytes\n").expect("initial pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("pointer snapshot")
            .expect("existing pointer");

        let error = held
            .compare_and_replace_public_regular_file_with_hook(
                Path::new("current.json"),
                b"next bytes\n",
                Some(&expected),
                "test pointer",
                64,
                || {
                    let replacement = temp.path().join("replacement.json");
                    fs::write(&replacement, b"same bytes\n").expect("replacement pointer");
                    // Models a same-UID actor outside the cooperating-writer lock contract.
                    fs::rename(replacement, &current).expect("swap pointer inode");
                },
            )
            .expect_err("same bytes on a different inode must fail");
        assert_eq!(error.code, "fs.concurrent_change");
        assert_eq!(
            fs::read(current).expect("preserved replacement"),
            b"same bytes\n"
        );
    }

    #[test]
    fn same_uid_noncooperating_writer_swap_at_exchange_boundary_is_restored() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"same bytes\n").expect("initial pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("pointer snapshot")
            .expect("existing pointer");
        let replacement = temp.path().join("replacement.json");
        fs::write(&replacement, b"same bytes\n").expect("replacement pointer");
        let replacement_inode = fs::metadata(&replacement)
            .expect("replacement metadata")
            .ino();

        let error = held
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new("current.json"),
                    bytes: b"next bytes\n",
                    expected: Some(&expected),
                    label: "test pointer",
                    max_bytes: 64,
                    required_mode: None,
                },
                |event| {
                    if event.operation == PublicMutationOperation::Rename
                        && event.phase == PublicMutationPhase::Before
                    {
                        fs::rename(&replacement, &current).expect("swap at exchange boundary");
                    }
                    Ok(())
                },
            )
            .expect_err("boundary swap must fail");
        assert!(matches!(
            error.code(),
            "fs.concurrent_change" | "fs.rollback"
        ));
        assert_eq!(
            fs::metadata(&current).expect("restored replacement").ino(),
            replacement_inode
        );
        assert_eq!(
            fs::read(current).expect("preserved replacement"),
            b"same bytes\n"
        );
    }

    #[test]
    fn post_exchange_validation_fault_restores_the_prior_pointer_without_drop_cleanup() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"prior\n").expect("prior pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("pointer snapshot")
            .expect("existing pointer");

        let error = held
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new("current.json"),
                    bytes: b"next\n",
                    expected: Some(&expected),
                    label: "test pointer",
                    max_bytes: 64,
                    required_mode: None,
                },
                |event| {
                    if event.operation == PublicMutationOperation::ExchangeValidation
                        && event.phase == PublicMutationPhase::Before
                    {
                        return Err(AppError::external("test.fault", "post-exchange read fault"));
                    }
                    Ok(())
                },
            )
            .expect_err("post-exchange validation fault must roll back");
        assert_eq!(error.code(), "test.fault");
        assert_eq!(fs::read(&current).expect("restored pointer"), b"prior\n");
        let restored = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("restored snapshot")
            .expect("restored pointer");
        assert!(restored.matches_moved_object(&expected));
    }

    #[test]
    fn post_initial_rename_validation_and_cleanup_faults_remove_the_new_pointer() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let mut expected_events = VecDeque::from([
            (
                PublicMutationOperation::InstallValidation,
                PublicMutationPhase::Before,
            ),
            (
                PublicMutationOperation::CleanupSync,
                PublicMutationPhase::Before,
            ),
        ]);

        let error = held
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new("current.json"),
                    bytes: b"next\n",
                    expected: None,
                    label: "test pointer",
                    max_bytes: 64,
                    required_mode: None,
                },
                |event| {
                    if expected_events.front() == Some(&(event.operation, event.phase)) {
                        expected_events.pop_front();
                        Err(AppError::external(
                            "test.fault",
                            "injected transaction fault",
                        ))
                    } else {
                        Ok(())
                    }
                },
            )
            .expect_err("post-rename validation fault must roll back");
        assert_eq!(error.code(), "fs.rollback");
        assert!(expected_events.is_empty(), "both faults were consumed");
        assert!(!temp.path().join("current.json").exists());
        assert!(
            fs::read_dir(temp.path())
                .expect("temporary root")
                .all(|entry| !entry
                    .expect("temporary entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".harp-public-")),
            "failed initial publication must not leave staging residue"
        );
        held.compare_and_replace_public_regular_file(
            Path::new("current.json"),
            b"retry\n",
            None,
            "test pointer",
            64,
        )
        .expect("allocator remains reusable after rollback");
    }

    #[test]
    fn later_pointer_replacement_during_initial_install_rollback_is_preserved() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        let replacement = temp.path().join("replacement.json");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");

        let error = held
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new("current.json"),
                    bytes: b"next\n",
                    expected: None,
                    label: "test pointer",
                    max_bytes: 64,
                    required_mode: None,
                },
                |event| {
                    if event.operation == PublicMutationOperation::InstallValidation
                        && event.phase == PublicMutationPhase::Before
                    {
                        fs::write(&replacement, b"later\n").expect("replacement pointer");
                        fs::rename(&replacement, &current).expect("replace installed pointer");
                        return Err(AppError::external("test.fault", "force rollback"));
                    }
                    Ok(())
                },
            )
            .expect_err("later replacement makes initial rollback ambiguous");
        assert_eq!(error.code(), "fs.rollback");
        assert_eq!(
            fs::read(current).expect("preserved replacement"),
            b"later\n"
        );
    }

    #[test]
    fn same_uid_noncooperating_writer_swap_after_parent_sync_cannot_retroactively_fail_commit() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"initial\n").expect("initial pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("pointer snapshot")
            .expect("existing pointer");
        let replacement = temp.path().join("replacement.json");
        fs::write(&replacement, b"next bytes\n").expect("replacement pointer");
        let replacement_inode = fs::metadata(&replacement)
            .expect("replacement metadata")
            .ino();

        held.compare_and_replace_public_regular_file_with_mode_and_events(
            PublicFileMutation {
                relative: Path::new("current.json"),
                bytes: b"next bytes\n",
                expected: Some(&expected),
                label: "test pointer",
                max_bytes: 64,
                required_mode: None,
            },
            |event| {
                if event.operation == PublicMutationOperation::ParentSync
                    && event.phase == PublicMutationPhase::After
                {
                    fs::rename(&replacement, &current).expect("swap after parent sync");
                    return Err(AppError::external("test.fault", "after-commit callback"));
                }
                Ok(())
            },
        )
        .expect("post-sync pointer swap cannot retroactively fail commit");
        assert_eq!(
            fs::metadata(&current).expect("preserved replacement").ino(),
            replacement_inode
        );
    }

    #[test]
    fn initial_parent_sync_after_callback_failure_is_nonfatal() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let installed = held
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new("current.json"),
                    bytes: b"initial\n",
                    expected: None,
                    label: "test pointer",
                    max_bytes: 64,
                    required_mode: None,
                },
                |event| {
                    if event.operation == PublicMutationOperation::ParentSync
                        && event.phase == PublicMutationPhase::After
                    {
                        return Err(AppError::external("test.fault", "after commit"));
                    }
                    Ok(())
                },
            )
            .expect("post-commit callback cannot fail initial installation");
        let current = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("read current")
            .expect("current pointer");
        assert!(current.matches_moved_object(&installed));
    }

    #[test]
    fn same_uid_noncooperating_writer_is_not_overwritten_by_rollback() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"initial\n").expect("initial pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("pointer snapshot")
            .expect("existing pointer");
        let replacement = temp.path().join("replacement.json");
        fs::write(&replacement, b"later\n").expect("later pointer");
        let replacement_inode = fs::metadata(&replacement)
            .expect("replacement metadata")
            .ino();

        let error = held
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new("current.json"),
                    bytes: b"next bytes\n",
                    expected: Some(&expected),
                    label: "test pointer",
                    max_bytes: 64,
                    required_mode: None,
                },
                |event| {
                    if event.operation == PublicMutationOperation::ParentSync
                        && event.phase == PublicMutationPhase::Before
                    {
                        fs::rename(&replacement, &current).expect("install later pointer");
                        return Err(AppError::external("test.fault", "injected sync fault"));
                    }
                    Ok(())
                },
            )
            .expect_err("sync fault with later pointer must fail");
        assert_eq!(error.code(), "fs.rollback");
        assert_eq!(
            fs::metadata(&current)
                .expect("preserved later pointer")
                .ino(),
            replacement_inode
        );
        assert_eq!(fs::read(current).expect("later bytes"), b"later\n");
    }

    #[test]
    fn compare_and_replace_bounds_a_raced_pointer_read() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"small\n").expect("initial pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                8,
            )
            .expect("pointer snapshot")
            .expect("existing pointer");

        let error = held
            .compare_and_replace_public_regular_file_with_hook(
                Path::new("current.json"),
                b"next\n",
                Some(&expected),
                "test pointer",
                8,
                || {
                    let replacement = temp.path().join("replacement.json");
                    fs::write(&replacement, [b'x'; 9]).expect("oversized replacement");
                    fs::rename(replacement, &current).expect("race pointer size");
                },
            )
            .expect_err("oversized raced pointer must fail boundedly");
        assert_eq!(error.code, "fs.size");
        assert_eq!(fs::metadata(current).expect("preserved pointer").len(), 9);
    }

    #[test]
    fn same_uid_noncooperating_writer_replacement_is_preserved_during_remove() {
        let temp = tempfile::tempdir().expect("temporary HeldDirectory root");
        let current = temp.path().join("current.json");
        fs::write(&current, b"installed\n").expect("installed pointer");
        let held = HeldDirectory::open(temp.path(), "test repository").expect("held root");
        let expected = held
            .read_optional_regular_single_link_file_snapshot_bounded(
                Path::new("current.json"),
                "test pointer",
                64,
            )
            .expect("pointer snapshot")
            .expect("existing pointer")
            .1;

        let error = held
            .compare_and_remove_public_regular_file_with_hook(
                Path::new("current.json"),
                &expected,
                "test pointer",
                64,
                || {
                    let replacement = temp.path().join("replacement.json");
                    fs::write(&replacement, b"installed\n").expect("replacement pointer");
                    fs::rename(replacement, &current).expect("replace installed pointer");
                },
            )
            .expect_err("rollback must reject a different inode");
        assert_eq!(error.code, "fs.concurrent_change");
        assert_eq!(
            fs::read(current).expect("restored replacement"),
            b"installed\n"
        );
    }
}
