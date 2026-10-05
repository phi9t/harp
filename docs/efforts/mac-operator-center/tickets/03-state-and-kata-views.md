# 03 — Kata and controller state panes

Status: claimed
Blocked by: none
Spec: ../EFFORT.md

## Owner request

Show Kata, controller state for the lead and workers, tasks and Git workspaces
in the dedicated tmux session. Open Codex clients for each concrete lead and
worker binding. Preserve implementation and product trial identities.

## Bounded implementation

Own tools/agents/mac_state_view.py and tools/agents/test_mac_state_view.py only.
Use the existing version1 config and standard-library terminal rendering.
Offer --view controller|kata|workspaces and --render, plus a refresh loop.
Read CURRENT and the maintained native-view manifest. Optional
controller_snapshot_path and kata_snapshot_path select explicit public JSON
projections exported by the deployment owner. Never read SQLite, auth files,
transcripts or arbitrary logs. Do not make state changes or run Kata mutations.

Controller view lists project/revision/event cursor, lead authority/runtime,
both worker slots, task and attempt states, unknown operations, questions and
landing disposition when provided. Missing product state remains unavailable.
Implementation writer status retains its CURRENT source label.
Workspaces view lists explicit attempt workspace/source/base/candidate/landing
and distinguishes source claims from observed inventory. Kata view identifies
deployment/project and displays source work items, dependencies and sync status
from an explicit public snapshot. No implicit default daemon/project selection.
Show source timestamp, digest, malformed/missing/stale status and bounded output.

Native Kata TUI may be used when its exact deployment/project is supplied and
supported CLI read tests pass. Native Codex views require exact endpoint/thread
and existing-store evidence; missing slots stay pending rather than guessed.
The lead owns host config and tmux installation. Controller coordinator owns
product runtime publication; these tools do not bootstrap a second controller.

## Verification

Focused tests for populated state, two slots, explicit missing/unknown values,
Kata identity/sync, malformed/stale source, secret/control exclusion and real
terminal startup. Two-axis review, install and inspect owned panes only.
