use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use corenius::core::{SandboxResult, ToolCall, ToolRegistryLike};
use corenius::tools::{ShellRunner, ToolRegistry};
use serde_json::json;
use tempfile::tempdir;
use tokio::fs;

#[derive(Default)]
struct RecordingShell {
    calls: Mutex<Vec<(String, PathBuf, u64)>>,
}

#[async_trait::async_trait]
impl ShellRunner for RecordingShell {
    async fn run_shell(
        &self,
        command: String,
        cwd: PathBuf,
        timeout_ms: u64,
    ) -> anyhow::Result<SandboxResult> {
        self.calls.lock().unwrap().push((command, cwd, timeout_ms));
        Ok(SandboxResult::success("ok"))
    }
}

#[tokio::test]
async fn default_tool_definitions_have_stable_capabilities() {
    let dir = tempdir().unwrap();
    let registry =
        ToolRegistry::default_for_workspace(dir.path(), Arc::new(RecordingShell::default()));
    let defs = registry.definitions();
    assert_eq!(
        defs.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        ["read_file", "shell"]
    );
    assert!(defs[0].capabilities.read_only);
    assert!(defs[1].capabilities.shell);
}

#[tokio::test]
async fn read_file_stays_under_workspace() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("note.txt"), "hello")
        .await
        .unwrap();
    let registry =
        ToolRegistry::default_for_workspace(dir.path(), Arc::new(RecordingShell::default()));
    let output = registry
        .execute(ToolCall::new(
            "1",
            "test",
            "read_file",
            json!({"path":"note.txt"}),
        ))
        .await
        .unwrap();
    assert_eq!(output, json!({"content":"hello"}));
    assert!(
        registry
            .execute(ToolCall::new(
                "2",
                "test",
                "read_file",
                json!({"path":"../outside"}),
            ))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn shell_tool_delegates_to_runner() {
    let dir = tempdir().unwrap();
    let shell = Arc::new(RecordingShell::default());
    let registry = ToolRegistry::default_for_workspace(dir.path(), shell.clone());
    let output = registry
        .execute(ToolCall::new(
            "1",
            "test",
            "shell",
            json!({"command":"pwd","timeoutMs":42}),
        ))
        .await
        .unwrap();
    assert_eq!(output["stdout"], "ok");
    let calls = shell.calls.lock().unwrap();
    assert_eq!(calls[0].0, "pwd");
    assert_eq!(calls[0].1, dir.path());
    assert_eq!(calls[0].2, 42);
}
