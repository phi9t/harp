use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

use crate::core::SandboxResult;

const MARKER: &str = ".corenius-insula-rootfs.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsulaMarker {
    pub schema_version: u8,
    pub owner: String,
    pub digest: String,
    pub node_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_lock_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsulaSelection {
    pub rootfs: PathBuf,
    pub store_root: PathBuf,
    pub workspace: PathBuf,
    pub marker: InsulaMarker,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BwrapPlan {
    pub executable: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct InsulaSandbox {
    pub selection: InsulaSelection,
}

#[derive(Serialize, Deserialize)]
struct SelectionFile {
    rootfs: PathBuf,
    #[serde(rename = "storeRoot")]
    store_root: PathBuf,
    digest: String,
}

impl InsulaSandbox {
    pub fn new(selection: InsulaSelection) -> Self {
        Self { selection }
    }

    pub fn selection_path(workspace: &Path) -> PathBuf {
        workspace.join(".corenius/insula/selected.json")
    }

    pub fn validate_selection(
        rootfs: &Path,
        store_root: &Path,
        workspace: &Path,
        expected_digest: Option<&str>,
    ) -> anyhow::Result<InsulaSelection> {
        let rootfs = rootfs.to_path_buf();
        let store_root = store_root.to_path_buf();
        let workspace = workspace.to_path_buf();
        let meta = fs::symlink_metadata(&rootfs)
            .with_context(|| format!("stat rootfs {}", rootfs.display()))?;
        if meta.file_type().is_symlink() {
            bail!("Rootfs path must not be a symlink");
        }
        if !meta.is_dir() {
            bail!("Rootfs path must be a directory");
        }
        let canonical_rootfs = rootfs.canonicalize()?;
        let canonical_store = store_root.canonicalize()?;
        if !canonical_rootfs.starts_with(&canonical_store) {
            bail!("Rootfs path must live under store_root");
        }
        let marker = read_marker(&rootfs)?;
        if let Some(expected) = expected_digest
            && expected != marker.digest
        {
            bail!("Rootfs marker digest does not match selected digest");
        }
        Ok(InsulaSelection {
            rootfs,
            store_root,
            workspace,
            marker,
        })
    }

    pub fn write_selection(
        workspace: &Path,
        rootfs: &Path,
        store_root: &Path,
    ) -> anyhow::Result<InsulaSelection> {
        let selection = Self::validate_selection(rootfs, store_root, workspace, None)?;
        let path = Self::selection_path(workspace);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = SelectionFile {
            rootfs: selection.rootfs.clone(),
            store_root: selection.store_root.clone(),
            digest: selection.marker.digest.clone(),
        };
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&file)?))?;
        Ok(selection)
    }

    pub fn load_selection(workspace: &Path) -> anyhow::Result<Self> {
        let text = fs::read_to_string(Self::selection_path(workspace))?;
        let file: SelectionFile = serde_json::from_str(&text)?;
        let selection = Self::validate_selection(
            &file.rootfs,
            &file.store_root,
            workspace,
            Some(&file.digest),
        )?;
        Ok(Self::new(selection))
    }

    pub fn scan(store_root: &Path, workspace: &Path) -> anyhow::Result<Vec<InsulaSelection>> {
        let mut selections = Vec::new();
        if !store_root.exists() {
            return Ok(selections);
        }
        for entry in fs::read_dir(store_root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Ok(selection) =
                Self::validate_selection(&entry.path(), store_root, workspace, None)
            {
                selections.push(selection);
            }
        }
        Ok(selections)
    }

    pub fn build_command(
        &self,
        command: &str,
        cwd: &Path,
        env: &[(&str, &str)],
        timeout_ms: u64,
    ) -> anyhow::Result<BwrapPlan> {
        let workspace = self.selection.workspace.canonicalize()?;
        let cwd = cwd.canonicalize()?;
        if !cwd.starts_with(&workspace) {
            bail!("Sandbox cwd must stay under the workspace");
        }
        let rel = cwd.strip_prefix(&workspace)?;
        let workspace_cwd = if rel.as_os_str().is_empty() {
            "/workspace".to_string()
        } else {
            format!(
                "/workspace/{}",
                rel.iter()
                    .map(|part| part.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/")
            )
        };
        let args = vec![
            "--die-with-parent".to_string(),
            "--unshare-pid".to_string(),
            "--ro-bind".to_string(),
            self.selection.rootfs.to_string_lossy().into_owned(),
            "/".to_string(),
            "--bind".to_string(),
            workspace.to_string_lossy().into_owned(),
            "/workspace".to_string(),
            "--tmpfs".to_string(),
            "/run".to_string(),
            "--tmpfs".to_string(),
            "/run/corenius-tmp".to_string(),
            "--dir".to_string(),
            "/home".to_string(),
            "--dir".to_string(),
            "/home/corenius".to_string(),
            "--proc".to_string(),
            "/proc".to_string(),
            "--dev".to_string(),
            "/dev".to_string(),
            "--chdir".to_string(),
            workspace_cwd,
            "/bin/sh".to_string(),
            "-lc".to_string(),
            command.to_string(),
        ];
        let mut filtered = BTreeMap::new();
        for (key, value) in std::env::vars() {
            if preserve_env(&key) {
                filtered.insert(key, value);
            }
        }
        for (key, value) in env {
            if preserve_env(key) {
                filtered.insert((*key).to_string(), (*value).to_string());
            }
        }
        filtered.insert("CORENIUS_IN_INSULA".to_string(), "1".to_string());
        filtered.insert("HOME".to_string(), "/home/corenius".to_string());
        filtered.insert("TMPDIR".to_string(), "/run/corenius-tmp".to_string());
        filtered.insert(
            "PATH".to_string(),
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
        );
        filtered.insert("PWD".to_string(), "/workspace".to_string());
        let _ = timeout_ms;
        Ok(BwrapPlan {
            executable: "bwrap".to_string(),
            args,
            env: filtered,
        })
    }

    pub async fn run(
        &self,
        command: &str,
        cwd: &Path,
        env: &[(&str, &str)],
        timeout_ms: u64,
    ) -> anyhow::Result<SandboxResult> {
        let plan = self.build_command(command, cwd, env, timeout_ms)?;
        let started = Instant::now();
        let mut child = Command::new(&plan.executable)
            .args(&plan.args)
            .env_clear()
            .envs(&plan.env)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            stdout.read_to_end(&mut buf).await.map(|_| buf)
        });
        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            stderr.read_to_end(&mut buf).await.map(|_| buf)
        });
        let mut timed_out = false;
        let status = match timeout(Duration::from_millis(timeout_ms), child.wait()).await {
            Ok(status) => status?,
            Err(_) => {
                timed_out = true;
                let _ = child.kill().await;
                child.wait().await?
            }
        };
        let stdout = stdout_task.await??;
        let stderr = stderr_task.await??;
        Ok(SandboxResult {
            exit_code: status.code(),
            signal: None,
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
            elapsed_ms: started.elapsed().as_millis(),
            timed_out,
        })
    }

    pub fn doctor(&self, dry_run: bool) -> anyhow::Result<Vec<String>> {
        let mut lines = Vec::new();
        lines.push(format!("rootfs ok: {}", self.selection.rootfs.display()));
        let node = self
            .selection
            .rootfs
            .join(self.selection.marker.node_path.trim_start_matches('/'));
        if !node.exists() {
            bail!("node_path does not exist in rootfs: {}", node.display());
        }
        lines.push(format!("node ok: {}", self.selection.marker.node_path));
        lines.push(format!(
            "workspace ok: {}",
            self.selection.workspace.display()
        ));
        let plan = self.build_command("true", &self.selection.workspace, &[], 5_000)?;
        lines.push(format!(
            "bwrap plan: {} {}",
            plan.executable,
            plan.args.join(" ")
        ));
        if !dry_run {
            if !Path::new("/usr/bin/bwrap").exists() && !Path::new("/bin/bwrap").exists() {
                bail!("bwrap not found");
            }
            lines.push("bwrap ok".to_string());
        }
        Ok(lines)
    }
}

fn read_marker(rootfs: &Path) -> anyhow::Result<InsulaMarker> {
    let text = fs::read_to_string(rootfs.join(MARKER)).context("read rootfs marker")?;
    let marker: InsulaMarker = serde_json::from_str(&text).context("parse rootfs marker")?;
    if marker.schema_version != 1 {
        bail!("Rootfs marker schema_version must be 1");
    }
    if marker.owner != "corenius-insula" {
        bail!("Rootfs marker owner must be corenius-insula");
    }
    if marker.digest.is_empty() {
        bail!("Rootfs marker digest must be non-empty");
    }
    if !marker.node_path.starts_with('/') {
        bail!("Rootfs marker node_path must be absolute");
    }
    Ok(marker)
}

fn preserve_env(key: &str) -> bool {
    matches!(
        key,
        "TERM"
            | "COLORTERM"
            | "HTTP_PROXY"
            | "HTTPS_PROXY"
            | "NO_PROXY"
            | "http_proxy"
            | "https_proxy"
            | "no_proxy"
    )
}
