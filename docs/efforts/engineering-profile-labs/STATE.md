# Engineering profile labs state

Updated: 2026-10-11; local landing preparation
Base: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`
Branch: `codex/engineering-postgres-temporal-labs`
Checkout: `/data02/home/philip.yang/.codex/worktrees/833c/corenius-labs`

## Current outcome

All five detailed learning labs are integrated and independently reviewed.
Database final run: `/data02/home/philip.yang/.codex/worktrees/833c/corenius-labs/.scratch/engineering-profile-labs/20261011T192530Z-3138291`.
Kernel: 16 passed, 0 failed. Temporal: 7 passed, 0 failed, 1 parent-owned child
entrypoint ignored. Actual recovery and explicit replay cases passed.
Both exact owned containers were removed; the Temporal listener closed.
The integrated native reducer passes 21 model-free tests. Codex CLI's recorded
live completion remains accepted under the repaired guard. Both TraeCLI attempts
remain rejected; their cause is not established. App-server live arithmetic
completion and empty-thread persistence were observed, but its historical
projection lacks item categories required by the current tool-absence guard.
No active-turn recovery, Corenius readiness or Trondheim service adoption is claimed.

## Durable records

Intent: EFFORT.md. Specification: SPEC.md. Procedures: POSTGRESQL.md and TEMPORAL.md.
Runner: run.py. Exact commands/digests: evidence/2026-10-11-run.json.
Conclusions and initial failed launch: evidence/summary.md.
Native guides, parser fixtures and immutable projected observations: native/README.md.
Combined entrypoint: README.md. Independent review and final checks:
evidence/review.md. Native FILES.json verifies 16 content hashes; ten historical
evidence files were unchanged by repairs.

## Landing authority and remaining qualification

Independent Standards and Spec review passes with no remaining actionable
findings. Ticket 01 remains claimed pending an accepted, landed Candidate;
ticket 02 stays open behind its tracker blocker; ticket 03 records the native
learning slice. Local checks and review do not supply a promotion receipt, so
the tracker was not forced closed. The learning experiments were explicitly
authorized. After this review, the user requested "land these labs first".
The reviewed lab slice and its closure records may be committed and landed on
local main. No remote is configured. No deployment or further provider retry
is part of this landing.

Adoption gates remain: operational database recovery, active native-turn
interruption/recovery, lost-ack reconciliation and enforced provider budgets.
The database runner's deadline termination path is also not fault-qualified.

## Resume

Read the specification and evidence summary first. Inspect `git status --short`
and use `python3 tools/temp_root.py --` for commands. Use the exact-candidate
landing gate before changing main. This manual STATE retains the reviewed
learning outcomes; subsequent landing and tracker closure will identify the
accepted source commit.
Raw logs/builds are regenerable scratch.
A cold run has a 1200-second per-suite deadline; use run.py for fresh qualification.
