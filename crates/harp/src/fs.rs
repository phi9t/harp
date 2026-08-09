use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest as _, Sha256};
use tempfile::NamedTempFile;

use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct FileSnapshot {
    digest: [u8; 32],
}

#[derive(Debug)]
pub struct HeldDirectory {
    root: PathBuf,
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
        })
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

    pub fn read_optional_regular_file_snapshot_bounded(
        &self,
        relative: &Path,
        label: &str,
        max_bytes: usize,
    ) -> Result<Option<FileSnapshot>, AppError> {
        self.read_optional_regular_file_bounded(relative, label, max_bytes)
            .map(|bytes| {
                bytes.map(|bytes| FileSnapshot {
                    digest: Sha256::digest(bytes).into(),
                })
            })
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

    pub fn compare_and_replace_public_regular_file(
        &self,
        relative: &Path,
        bytes: &[u8],
        expected: Option<&FileSnapshot>,
        label: &str,
    ) -> Result<(), AppError> {
        let destination = self.resolve_write_target(relative, label)?;
        let parent = destination.parent().ok_or_else(|| {
            AppError::invalid_input("fs.output", format!("{label} has no parent"))
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| AppError::io("fs.create_parent", label, error))?;
        self.reject_symlink_ancestors(relative, label)?;

        let current = match fs::symlink_metadata(&destination) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(AppError::invalid_input(
                        "fs.output",
                        format!("{label} target must be a regular file"),
                    ));
                }
                Some(
                    fs::read(&destination)
                        .map_err(|error| AppError::io("fs.read_current", label, error))?,
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(AppError::io("fs.output_metadata", label, error)),
        };
        let matches_expected = match (current.as_ref(), expected) {
            (None, None) => true,
            (Some(current), Some(expected)) => {
                <[u8; 32]>::from(Sha256::digest(current)) == expected.digest
            }
            _ => false,
        };
        if !matches_expected {
            return Err(AppError::invalid_input(
                "fs.concurrent_change",
                format!("{label} changed while it was being generated"),
            ));
        }

        let mut temporary =
            NamedTempFile::new_in(parent).map_err(|error| AppError::io("fs.temp", label, error))?;
        temporary
            .write_all(bytes)
            .and_then(|()| temporary.as_file_mut().sync_all())
            .map_err(|error| AppError::io("fs.write", label, error))?;
        temporary
            .persist(&destination)
            .map_err(|error| AppError::io("fs.publish", label, error.error))?;
        Ok(())
    }

    fn open_regular_file(&self, relative: &Path, label: &str) -> Result<File, AppError> {
        let path = self.resolve_read_target(relative, label)?;
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        let file = options.open(&path).map_err(|error| {
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
        Ok(file)
    }

    fn resolve_read_target(&self, relative: &Path, label: &str) -> Result<PathBuf, AppError> {
        validate_relative(relative, label)?;
        self.reject_symlink_ancestors(relative, label)?;
        Ok(self.root.join(relative))
    }

    fn resolve_write_target(&self, relative: &Path, label: &str) -> Result<PathBuf, AppError> {
        validate_relative(relative, label)?;
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
