# Corenius Rust Rebuild Design

## Goal

Rebuild Corenius as a standalone Rust terminal agent while preserving the V1
external contracts from the TypeScript prototype: Codex direct Responses
transport, permissioned tools, `insula` bwrap execution, JSONL transcript
storage, and an interactive terminal UI.

## Scope

The Rust rebuild replaces the TypeScript/Node project in this repository. It
does not preserve the Node package entrypoints. The supported public entrypoint
is a Cargo-built `corenius` binary with these commands:

- `corenius run --workspace <path>`
- `corenius insula scan --store-root <path> [--workspace <path>]`
- `corenius insula select --rootfs <path> --store-root <path> [--workspace <path>]`
- `corenius insula doctor [--workspace <path>] [--dry-run]`

## Architecture

Corenius is split into focused Rust modules under one crate:

- `core`: runtime events, tool/provider traits, `Coordinator`, `TurnRunner`,
  approval flow, and transcript store.
- `provider::codex`: Codex auth, SSE parsing, streaming Responses loop, tool
  continuation, usage accounting, and max tool-step enforcement.
- `tools`: default read and shell tools with JSON schemas and capability flags.
- `insula`: rootfs marker/selection validation, bwrap command planning, command
  execution, timeout cleanup, scan/select/doctor helpers.
- `tui`: terminal state reducer and `ratatui` rendering/input loop.
- `cli`: `clap` command parsing and composition.

The key runtime rule is that events are delivered through an async channel.
Approval requests are emitted to the TUI before the runner waits for the
approval answer, avoiding the deadlock found in the TypeScript prototype.

## External Contracts

- Selection file: `.corenius/insula/selected.json`, ignored by git.
- Rootfs marker: `.corenius-insula-rootfs.json` inside selected rootfs.
- Marker requires `schema_version: 1`, `owner: "corenius-insula"`, `digest`,
  `node_path`, and optional `dependency_lock_digest`.
- Rootfs path must not be a symlink and must live under declared `store_root`.
- Selection load must verify the selected digest still matches the marker.
- Transcript log is JSONL under `.corenius/transcript.jsonl`.
- Read tools are constrained to the workspace.
- Shell commands execute through `/bin/sh -lc` inside bwrap.
- Shell/write-capable tools require explicit approval; denial returns a
  structured tool error to the model.

## Sandbox Execution

The bwrap plan uses:

- `--die-with-parent`
- `--unshare-pid`
- `--ro-bind <rootfs> /`
- `--bind <workspace> /workspace`
- `--tmpfs /tmp`
- `--tmpfs /run`
- isolated home at `/home/corenius`
- `--proc /proc`
- `--dev /dev`
- `--chdir <workspace-relative cwd>`
- `/bin/sh -lc <command>`

Environment is filtered to terminal and proxy variables only:
`TERM`, `COLORTERM`, `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`, and lowercase
variants. Corenius sets `CORENIUS_IN_INSULA=1`, `HOME=/home/corenius`,
`TMPDIR=/tmp`, `PATH`, and `PWD=/workspace`.

## Codex Direct Provider

Credentials are read from `CODEX_HOME` or `~/.codex/auth.json`. The file must be
a private regular file and not a symlink. The provider supports:

- text deltas
- `response.completed` usage accounting
- `response.failed` and malformed/truncated SSE errors
- function calls
- function-call outputs
- max-step exhaustion
- model selection from `SAND_CODEX_MODEL`, then Codex config, then `gpt-5.4`

## TUI

The terminal UI uses `ratatui` and `crossterm`. It shows:

- transcript pane
- live status line
- tool activity rows with result summaries
- active approval prompt with approve/deny keys
- input prompt

Ctrl-C cancels the active turn and shuts down cleanly.

## Testing

Tests are written with Cargo's built-in test harness. Required coverage:

- Codex SSE parser: text deltas, tool calls, malformed JSON, truncated stream,
  failed responses, and max-step exhaustion.
- Tool registry schemas and capability flags.
- Policy gate read auto-allow and shell/write approval.
- Transcript JSONL replay.
- Insula selection rejection for symlink, missing marker, wrong owner, escaping
  store root, malformed JSON, and digest mismatch.
- Insula deterministic bwrap command plan and env filtering.
- Fake-provider integration for read tool and shell approval denial.
- TUI state reducer smoke coverage for streaming text, approval, tool result,
  and finished turn.
