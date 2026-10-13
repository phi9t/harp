//! Trusted startup-only native assets and narrowly scoped Unix transports.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::{FileTypeExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
};
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pin {
    pub path: PathBuf,
    pub sha256: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeProfile {
    pub kind: String,
    pub binary: Pin,
    pub companion: Pin,
    pub catalog: Pin,
    pub config: Pin,
    pub launcher: Pin,
    pub transport_root: PathBuf,
    pub provider_socket: Option<PathBuf>,
    pub executor_root: Option<PathBuf>,
    #[serde(default)]
    pub own_executor_only: bool,
    #[serde(skip)]
    pinned_assets: Mutex<Vec<fs::File>>,
}
fn directory(path: &Path) -> Result<(), String> {
    if !path.is_absolute() || path.starts_with("/usr") {
        return Err("private native transport directory required".into());
    }
    if !path.exists() {
        fs::create_dir(path).map_err(|e| e.to_string())?;
    }
    if path.canonicalize().map_err(|e| e.to_string())? != path {
        return Err("native directory aliases refused".into());
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())
}
impl NativeProfile {
    fn assets(&self) -> [(&Pin, &str); 5] {
        [
            (&self.binary, "/run/codex/codex"),
            (&self.companion, "/run/codex/codex-code-mode-host"),
            (&self.catalog, "/run/codex/catalog.json"),
            (&self.config, "/run/codex/config.toml"),
            (&self.launcher, "/run/codex/launcher.py"),
        ]
    }
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.kind.as_str(), "app" | "exec") {
            return Err("native profile kind refused".into());
        }
        if self.binary.sha256 != "8bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479"
            || self.companion.sha256
                != "fe64bd08c7844ca1fca91d8cb825e62012a03a05ae8040c635bbb61e5be0b4c5"
        {
            return Err("native binary version pins refused".into());
        }
        let mut pinned = self
            .pinned_assets
            .lock()
            .map_err(|_| "native assets lock poisoned")?;
        if pinned.is_empty() {
            for (pin, _) in self.assets() {
                if !pin.path.is_absolute()
                    || pin.path.canonicalize().map_err(|e| e.to_string())? != pin.path
                    || !pin.path.is_file()
                {
                    return Err("native asset aliases refused".into());
                }
                let fd = unsafe {
                    libc::memfd_create(c"corenius-pinned-native".as_ptr(), libc::MFD_ALLOW_SEALING)
                };
                if fd < 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                let mut sealed = unsafe { fs::File::from_raw_fd(fd) };
                let mut source = fs::File::open(&pin.path).map_err(|e| e.to_string())?;
                let mut hasher = Sha256::new();
                let mut buffer = [0u8; 65536];
                loop {
                    let n = source.read(&mut buffer).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                    sealed.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
                }
                if format!("{:x}", hasher.finalize()) != pin.sha256 {
                    return Err("native asset digest mismatch".into());
                }
                sealed
                    .set_permissions(fs::Permissions::from_mode(0o555))
                    .map_err(|e| e.to_string())?;
                if unsafe {
                    libc::fcntl(
                        fd,
                        libc::F_ADD_SEALS,
                        libc::F_SEAL_WRITE
                            | libc::F_SEAL_GROW
                            | libc::F_SEAL_SHRINK
                            | libc::F_SEAL_SEAL,
                    )
                } < 0
                {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                let inherited = unsafe { libc::fcntl(fd, libc::F_DUPFD, 100) };
                if inherited < 0 {
                    return Err(std::io::Error::last_os_error().to_string());
                }
                pinned.push(unsafe { fs::File::from_raw_fd(inherited) });
            }
        }
        use std::os::unix::ffi::OsStrExt;
        if self.transport_root.as_os_str().as_bytes().len() + 49 > 107 {
            return Err("native Unix transport root exceeds pathname limit".into());
        }
        directory(&self.transport_root)?;
        if let Some(socket) = &self.provider_socket
            && (!socket.is_absolute()
                || socket.starts_with("/usr")
                || socket.canonicalize().map_err(|e| e.to_string())? != *socket
                || !fs::metadata(socket)
                    .map_err(|e| e.to_string())?
                    .file_type()
                    .is_socket())
        {
            return Err("scoped provider Unix socket required".into());
        }
        if let Some(root) = &self.executor_root {
            if self.kind != "app" {
                return Err("executor cannot mount peer transport capabilities".into());
            }
            directory(root)?;
        }
        Ok(())
    }
    pub fn mount(
        &self,
        command: &mut Command,
        attempt: &str,
        token_digest: &str,
    ) -> Result<Vec<fs::File>, String> {
        let transport = self.transport_root.join(attempt);
        directory(&transport)?;
        command.args(["--uid", "0", "--gid", "0", "--cap-add", "CAP_NET_ADMIN"]);
        let pinned = self
            .pinned_assets
            .lock()
            .map_err(|_| "native assets lock poisoned")?;
        if pinned.len() != 5 {
            return Err("native assets not qualified".into());
        }
        let mut inherited = Vec::new();
        for ((_, target), file) in self.assets().iter().zip(pinned.iter()) {
            // Independent read position, no mutable pathname in the child. bwrap's
            // ro-bind-data consumes this descriptor into a private readonly mount.
            let reader = fs::File::open(format!("/proc/self/fd/{}", file.as_raw_fd()))
                .map_err(|e| e.to_string())?;
            let fd = unsafe { libc::fcntl(reader.as_raw_fd(), libc::F_DUPFD, 110) };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            inherited.push(unsafe { fs::File::from_raw_fd(fd) });
            command
                .args(["--perms", "0555", "--ro-bind-data"])
                .arg(fd.to_string())
                .arg(target);
        }
        command.arg("--bind").arg(&transport).arg("/run/bridge");
        if let Some(socket) = &self.provider_socket {
            command
                .arg("--ro-bind")
                .arg(socket)
                .arg("/run/provider.sock");
        }
        if let Some(root) = &self.executor_root {
            if self.own_executor_only {
                let own = root.join(attempt);
                directory(&own)?;
                command
                    .arg("--ro-bind")
                    .arg(&own)
                    .arg(format!("/run/executors/{attempt}"));
                command.args(["--setenv", "CORENIUS_NATIVE_OWN_EXECUTOR", attempt]);
            } else {
                command.arg("--ro-bind").arg(root).arg("/run/executors");
            }
        }
        command.args(["--setenv", "CORENIUS_NATIVE_TOKEN_SHA256", token_digest]);
        command.args([
            "--setenv",
            "CORENIUS_NATIVE_KIND",
            &self.kind,
            "--setenv",
            "CODEX_HOME",
            "/home/worker/.codex",
        ]);
        Ok(inherited)
    }
}
