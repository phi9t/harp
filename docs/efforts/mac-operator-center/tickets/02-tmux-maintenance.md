# 02 — Maintained team session

Status: claimed
Blocked by: none
Spec: ../EFFORT.md

## Requirements

Own tools/agents/mac_tmux.py and tools/agents/test_mac_tmux.py only.
Implement the shared config, atomic manifest and --once/--watch interface.
Adopt the existing operator session's ownership only when explicitly configured.
Preserve all existing windows and native clients. Add views for exact admitted
bindings and mark superseded views historical. Keep ownership and health visible.

## Acceptance

- Repeated reconciliation does not duplicate native clients.
- Endpoint plus thread is the view identity; unrelated sessions are untouched.
- Replacement and source-unavailable cases retain prior views and report status.
- No initial prompt, input, interrupt, configuration override or agent launch
  is introduced by creating a native view. Missing bindings are not guessed.
- Focused command-boundary tests and a disposable real tmux check pass.

## Comments

Existing Codex clients already rejoin the pinned shared daemon's loaded threads.
Core controller admissions/corrections and the review timer remain unchanged.
