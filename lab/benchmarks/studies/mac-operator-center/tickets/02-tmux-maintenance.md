# 02 tmux-maintenance: Maintained team session

Type: task
Status: resolved
Owner:
Blocked by:
Claimed:
Spec: ../EFFORT.md

## Requirements

Own controller/operator/mac_tmux.py and controller/operator/mac_tmux_test.py only.
Implement the shared config, atomic manifest and --once/--watch interface.
Adopt the existing operator session's ownership only when explicitly configured.
Preserve all existing windows and native clients. Add views for exact admitted
bindings and mark superseded views historical. Keep ownership and health visible.

## Acceptance


- [x] Repeated reconciliation does not duplicate native clients.
- [x] Endpoint plus thread is the view identity; unrelated sessions are untouched.
- [x] Replacement and source-unavailable cases retain prior views and report status.
- [x] No initial prompt, input, interrupt, configuration override or agent launch is introduced by creating a native view. Missing bindings are not guessed.
- [x] Focused command-boundary tests and a disposable real tmux check pass.
## Retained discussion

Existing Codex clients already rejoin the pinned shared daemon's loaded threads.
Core controller admissions/corrections and the review timer remain unchanged.

## Comments

- 2026-10-05T05:54:54Z main-lead: Normalize tracker metadata for landing; retained prior disposition: reviewed and deployed (source e6ae1ad; STATE.md records evidence). Product baseline acceptance and runtime readiness are unchanged.
