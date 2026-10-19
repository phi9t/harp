# 02 relocate-terminal-agent: Relocate the terminal agent

Type: task
Status: resolved
Owner: TRAE
Blocked by:
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Developers find the terminal agent in its semantic product component and keep using existing root Cargo commands and evaluation inputs.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [x] The terminal agent implementation, integration tests and owned support assets move together while its package and binary identities remain unchanged.
- [x] Root Cargo defaults select the terminal agent and preserve documented run and test behavior; temporary workspace structure permits independent controller packages until consolidation.
- [x] Default evaluation data remains available; documented root-relative dataset interpretation and absolute dataset arguments retain their pre-migration behavior.
- [x] Component-local commands and output locations are documented accurately; active navigation and affected gates are updated within this slice.
- [x] Existing terminal_agent tests and deterministic CLI/asset checks pass without requiring live provider credentials or real-model evaluation.
- [x] Repository-local scratch outputs remain at their intended destinations; historical evaluation evidence is preserved.

## Exclusions

No controller moves, final shared dependency consolidation, provider behavior changes or new evaluation qualification.

## Verification evidence

Root cargo test --offline; deterministic cargo run -- --help and relevant subcommand help; existing CLI/provider fixtures and evaluation asset checks from documented invocation contexts.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
- 2026-10-13T00:10:32Z TRAE: moved the root package into `terminal_agent/` and preserved root default commands through the new workspace. Verified `cargo metadata --locked --offline --format-version 1 --no-deps`, `cargo run --locked --offline -- --help`, `cargo run --locked --offline -- eval aime-sanity --help` and `cargo test --locked --offline -- --test-threads=1`, all under `python3 tools/temp_root.py --`, exit 0.
