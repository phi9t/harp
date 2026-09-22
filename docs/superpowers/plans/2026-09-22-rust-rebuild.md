# Corenius Rust Rebuild Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the TypeScript Corenius prototype with a Rust terminal agent preserving the same V1 runtime, provider, tool, insula, transcript, CLI, and TUI contracts.

**Architecture:** Build one Rust crate with focused modules for core runtime, Codex direct transport, tools, insula, TUI state/rendering, and CLI composition. Runtime events flow through an async event channel so approval prompts are visible before the runner waits for an approval answer.

**Tech Stack:** Rust 1.95, Cargo, Tokio, Clap, Serde, Reqwest, EventSource stream parsing, Ratatui, Crossterm.

**Spec:** `docs/superpowers/specs/2026-09-22-rust-rebuild-design.md`

## Global Constraints

- Preserve CLI commands: `run`, `insula scan`, `insula select`, `insula doctor`.
- Preserve selection file `.corenius/insula/selected.json` and rootfs marker `.corenius-insula-rootfs.json`.
- Selection load verifies marker digest against selected digest.
- Read tools stay under workspace.
- Shell tools run through bwrap `/bin/sh -lc` and require approval.
- Approval events must be emitted before awaiting approval answers.
- Codex credentials come from `CODEX_HOME` or `~/.codex/auth.json` and must be private regular files.
- Tests must run with `cargo test`.

---

### Task 1: Rust Crate Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Modify: `.gitignore`
- Modify: `README.md`
- Remove: TypeScript package files and source after Rust scaffold exists.

**Interfaces:**
- Produces: `corenius` binary and library modules declared in `src/lib.rs`.

- [ ] **Step 1: Write failing scaffold test**

Create `tests/scaffold.rs`:

```rust
#[test]
fn crate_exposes_version() {
    assert!(!corenius::VERSION.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test crate_exposes_version`
Expected: fail because Cargo crate does not exist yet.

- [ ] **Step 3: Add Cargo scaffold**

Create `Cargo.toml` with dependencies:

```toml
[package]
name = "corenius"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"

[dependencies]
anyhow = "1"
async-trait = "0.1"
clap = { version = "4", features = ["derive"] }
crossterm = "0.28"
dirs = "6"
futures-util = "0.3"
ratatui = "0.29"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
thiserror = "2"
tokio = { version = "1", features = ["macros", "process", "rt-multi-thread", "sync", "time"] }
tokio-util = "0.7"
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

Create `src/lib.rs`:

```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod cli;
pub mod core;
pub mod insula;
pub mod provider;
pub mod tools;
pub mod tui;
```

Create `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    corenius::cli::run().await
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test crate_exposes_version`
Expected: pass.

---

### Task 2: Core Runtime, Policy, Transcript

**Files:**
- Create: `src/core/mod.rs`
- Create: `src/core/types.rs`
- Create: `src/core/policy.rs`
- Create: `src/core/transcript.rs`
- Create: `src/core/runner.rs`
- Test: `tests/core.rs`

**Interfaces:**
- Produces: `RuntimeEvent`, `ToolDefinition`, `ToolCall`, `ApprovalRequest`, `SandboxCommand`, `SandboxResult`, `ProviderSession`, `ToolRegistryLike`, `Coordinator`, `TurnRunner`, `TranscriptStore`, `PolicyGate`.

- [ ] **Step 1: Write failing tests**

Tests:
- policy auto-allows read-only tool and requests approval for shell/write tool.
- transcript JSONL replay projects user and assistant messages.
- fake provider shell call emits `approval-requested` before waiting for approval.

- [ ] **Step 2: Run tests to verify fail**

Run: `cargo test --test core`
Expected: fail because modules are missing.

- [ ] **Step 3: Implement minimal core**

Implement serde-serializable events and transcript records. Use `tokio::sync::mpsc` for runtime event delivery and `tokio::sync::oneshot` for approval decisions. `TurnRunner` must send `tool-requested` and `approval-requested` before awaiting the approval answer.

- [ ] **Step 4: Run tests**

Run: `cargo test --test core`
Expected: pass.

---

### Task 3: Insula Sandbox

**Files:**
- Create: `src/insula.rs`
- Test: `tests/insula.rs`

**Interfaces:**
- Produces: `InsulaMarker`, `InsulaSelection`, `InsulaSandbox::scan`, `select`, `load`, `doctor`, `build_command`, `run`.

- [ ] **Step 1: Write failing tests**

Tests:
- rejects symlink rootfs.
- rejects missing/malformed/wrong-owner marker.
- rejects rootfs escaping store root.
- rejects selected digest mismatch.
- builds deterministic bwrap args and filters env.

- [ ] **Step 2: Run tests to verify fail**

Run: `cargo test --test insula`
Expected: fail because module is missing.

- [ ] **Step 3: Implement insula**

Use `std::fs::symlink_metadata`, `canonicalize`, `strip_prefix`, serde marker parsing, and `tokio::process::Command` for execution with timeout cleanup.

- [ ] **Step 4: Run tests**

Run: `cargo test --test insula`
Expected: pass.

---

### Task 4: Tools

**Files:**
- Create: `src/tools.rs`
- Test: `tests/tools.rs`

**Interfaces:**
- Produces: `ToolRegistry`, default `read_file` and `shell` tools.

- [ ] **Step 1: Write failing tests**

Tests:
- default tool definitions expose stable schemas and capabilities.
- read tool rejects path escape and reads workspace files.
- shell tool delegates to sandbox runner and preserves command/cwd/timeout.

- [ ] **Step 2: Run tests to verify fail**

Run: `cargo test --test tools`
Expected: fail because module is missing.

- [ ] **Step 3: Implement tools**

Implement trait-object registry with async executors. Keep read path resolution under workspace.

- [ ] **Step 4: Run tests**

Run: `cargo test --test tools`
Expected: pass.

---

### Task 5: Codex Direct Provider

**Files:**
- Create: `src/provider/mod.rs`
- Create: `src/provider/codex.rs`
- Test: `tests/provider_codex.rs`

**Interfaces:**
- Produces: `CodexDirectProvider`, `stream_codex_direct`, `parse_sse_events`, credential/model helpers.

- [ ] **Step 1: Write failing tests**

Tests:
- parses text deltas and completed usage.
- executes function call and sends function_call_output continuation.
- rejects malformed JSON, truncated SSE, failed responses.
- fails on max-step exhaustion.
- credential loader rejects symlink or world/group-readable auth file.

- [ ] **Step 2: Run tests to verify fail**

Run: `cargo test --test provider_codex`
Expected: fail because module is missing.

- [ ] **Step 3: Implement provider**

Use `reqwest::Client`, byte-stream SSE parsing, serde JSON, and callback-style tool execution. Use `SAND_CODEX_MODEL`, Codex config model, then `gpt-5.4`.

- [ ] **Step 4: Run tests**

Run: `cargo test --test provider_codex`
Expected: pass.

---

### Task 6: CLI and TUI

**Files:**
- Create: `src/cli.rs`
- Create: `src/tui.rs`
- Test: `tests/cli_tui.rs`

**Interfaces:**
- Produces: `corenius` CLI and TUI reducer/render model.

- [ ] **Step 1: Write failing tests**

Tests:
- `corenius insula select` writes selection and `doctor --dry-run` prints bwrap plan.
- TUI state reducer handles turn start, assistant delta, tool request, approval request, tool result summary, and finish.

- [ ] **Step 2: Run tests to verify fail**

Run: `cargo test --test cli_tui`
Expected: fail because modules are missing.

- [ ] **Step 3: Implement CLI and TUI**

Use `clap` for commands. Use `ratatui` state rendering and `crossterm` input. Ctrl-C cancels active turn. Approval keys are `y` and `n`.

- [ ] **Step 4: Run tests**

Run: `cargo test --test cli_tui`
Expected: pass.

---

### Task 7: Final Verification and Cleanup

**Files:**
- Modify: `README.md`
- Modify: `.gitignore`

**Interfaces:**
- Consumes: all previous tasks.

- [ ] **Step 1: Remove TypeScript artifacts**

Remove `package.json`, `package-lock.json`, `tsconfig.json`, `src/**/*.ts`, `src/**/*.tsx`, and `tests/core.test.ts` after Rust equivalents pass.

- [ ] **Step 2: Update README**

Document Cargo commands and the insula/Codex contracts.

- [ ] **Step 3: Run full verification**

Run:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Expected: all pass.

---

## Self-Review

- Spec coverage: all spec sections map to Tasks 1-7.
- Placeholder scan: no TODO/TBD placeholders.
- Type consistency: task names use the module paths declared in Task 1.
