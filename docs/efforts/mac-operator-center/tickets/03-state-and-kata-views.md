# 03 state-and-kata-views: Kata and controller state panes

Type: task
Status: resolved
Owner:
Blocked by:
Claimed:
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

## Scoped disposition — 2026-10-05

Source15ba811: focused suite43/43 PASS. Independent Standards, Spec and branch
reviews resolved the scoped findings. Dedicated operator views were installed.
Codex0.160.0 exact native implementation/lead clients are verified. Product
controller/API and UI gateways became unavailable; their reattachment remains
pending core-coordinator reconciliation. No full product qualification credit.
See ../STATE.md and ../evidence/client-update-20261005.json.

## Public export follow-up

The coordinator now publishes explicit source paths through CURRENT.public_exports.
The viewers read those files directly. Support its controller envelope with
status/source/deployment/project/control_center and distinct last-success history.
Support Kata items with stable workref identities and synchronized
corenius.execution summaries. Preserve upstream timestamps and digests. An
unavailable controller remains unavailable; Kata execution metadata is a
recorded summary, not an independent runtime liveness observation. Strict,
bounded field projection and legacy flat shapes remain supported.

Only state-view code/tests may change. The operator owner updates private host
config and its own view panes. Preserve all trial-* service windows. Test the
actual envelopes and hostile unknown nested fields. No database/API/auth reads,
model turns, new actors, acceptance changes or core repair-cap credit.

## Scoped snapshot follow-up

Replacement84eb5a publishes numeric epoch seconds and available/current/data
controller snapshots, plus source_identity/items.issue Kata snapshots. Support
these explicit shapes without changing their source time or raw-file digest.
Reject boolean/nonfinite/out-of-range timestamps. Preserve legacy ISO sources.
Display source availability and one-time snapshot claims separately from runtime
liveness. Read kata_source_path for provenance/identity only; never render requests.
No core/source-file changes or trial-window mutations are authorized by this UI slice.

## Acceptance

- [x] Implement the bounded requirements and pass focused tests and independent review.

## Comments

- 2026-10-05T05:54:54Z main-lead: Normalize tracker metadata for landing; retained prior disposition: implemented and independently reviewed; live product recovery pending. Product baseline acceptance and runtime readiness are unchanged.
