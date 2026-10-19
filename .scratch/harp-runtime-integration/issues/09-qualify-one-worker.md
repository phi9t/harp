# 09: Qualify the complete one-worker milestone

Status: done
Owner: Codex on `codex/harp-durable-runtime`
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Complete a real coding task with Codex and another with Trae through Harp, each under capacity one and ending in exact verification, explicit human approval and synchronized local landing.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Run each provider on Linux at devx_b200_dbg using the same admitted lifecycle. Each task produces a useful inspectable code change, passing exact verification and an explicitly approved landing.
- [x] Observe that actual active worker count never exceeds one and a queued second task is admitted only after the old execution is cleaned up.
- [x] Exercise integrated cancellation and runtime-restart recovery with the real provider adapters; completed results are preserved and ambiguous active effects require explicit reconciliation.
- [x] Retain exact source/provider/runtime/SQLite versions, execution contract, isolation/backend evidence, commands, attempt/candidate identities, check results, approval and landing outcomes.
- [x] All required deterministic fault, isolation, stale-result and Git-boundary checks pass. Missing credentials, delegated capability or prerequisites remain explicit blockers.
- [x] Normal operation and verification need neither PostgreSQL nor Temporal. A retained old scheduler, historical test record or fixture-only provider result cannot satisfy acceptance.

## Blocked by

- [01: Remove PostgreSQL/Temporal from active builds and gates](01-remove-postgres-temporal.md)
- [04: Execute a managed Codex worker](04-managed-codex-worker.md)
- [05: Execute a managed Trae worker](05-managed-trae-worker.md)
- [08: Approve and physically land a verified candidate](08-approve-land-candidate.md)

## Scope

Obtain explicit implementation-stage authorization for live provider use and any required setup. Ticket publication alone grants neither. This acceptance is the gate before two-worker qualification.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

2026-10-18: both real providers completed independently checked useful coding
tasks under capacity one in `/data03/harp-runtime-integration-philip-yang/runs/live-one-04`.
Source was frozen under `runs/provider-complete-03`; its 38 provider/native cases
passed with zero skips. The local changed-surface gate against `43b55351` passed
96 cases (35 explicit Linux-only skips, supplemented by delegated Linux evidence).
Runtime replay preserved each completed coding attempt/result. Captured bundle
hashes were rechecked before reviewing the exact diffs locally.

| Provider | Base | Candidate | Tree |
| --- | --- | --- | --- |
| codex | `f732a5678cb553403a385ac4fa4fe5c549db6731` | `330797ab90b8e1b3b5b7054914a7b46754c1b492` | `07d839f241e62194e3506cb302ae72798b7f270d` |
| trae | `e2902792cd0d696d9de860895f26d1e108e5ce1a` | `a0a5e25aa12150aecb17488c6249df4923d9b59f` | `1c581cc2afd0ebe8c1c8f0b621227807d15e288c` |

Both destinations are separate task-owned repositories under `runs/live-one-04`.
The public owner capability was registered outside projected inputs, without
printing its contents. The exact two candidates were presented for human approval;
approval and physical landing subsequently completed as recorded below. Deterministic integrated adapter
cancellation/queued-capacity/restart qualification passed in
`runs/provider-recovery-03`: three tests covering both actual adapters per case,
zero skips, 60.969 seconds, 47 frozen source files unchanged. Fixture executables
are explicitly distinguished from the separately qualified paid providers.
The cancellation case holds a live guardian with a provider and descendant in
a populated kernel cgroup, proves the queued task has no attempt, then checks
the next launch timestamp follows the prior cleanup receipt. Runtime death
requires explicit reconciliation without duplicate dispatch; a crash after the
completed receipt preserves the receipt, bundle, result and single attempt
through two restarts. The integrated all-module Linux suite at `runs/integrated-one-01` passed all
99 tests with zero skips in 411.586 seconds; all 47 source files remained
unchanged. The exact public Harp binary was
`d8fa70ef769f6bcb5cb064f19401beab64c96b3354454d553c52b48891e94163`.

The complete-branch changed-surface gate, `mise run verify -- --base 1418455d`,
passed after the production/recovery test payload and import receipt settled:
repository/LFS/import checks; 99 host runtime cases (38 explicitly Linux-only
skips, all supplied by the fresh zero-skip Linux run above); 14 selector cases;
12 import-verifier cases; 467 Rust library cases; five public durable CLI bridge
cases; 16 general CLI cases; and document validation. Independent acceptance
review confirmed that separate real provider runs plus controlled adapter fault
tests match the specification. No further live model probe is needed for this
milestone. Exact human approval and physical local landing remain outstanding;
tickets 10–14 have not been claimed as delivered.

The owner explicitly approved both exact candidates in the approval reply.
Both were approved and physically landed through the public CLI; retained
`runs/live-one-04/landings.json` and `landing-commands.jsonl` contain the outcomes.
Codex approval: `approval-6115812dcb0018e54ebe73f9edf78c249f64390e0f3a2ccce60394b9c8f31d71`.
Trae approval: `approval-d2fd0a245d7d47111438d67fff8d7c25b96d0c1340029f09ef51fa801f7fa86b`.
For each destination, branch HEAD, index tree, tracked checkout bytes and clean
status matched the approved commit/tree. No candidate was substituted or
automatically reapproved. The one-worker milestone is complete. Earlier pending
notes above describe the preapproval checkpoint.
