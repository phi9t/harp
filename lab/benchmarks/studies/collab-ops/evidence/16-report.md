# Ticket 16: Landing shim and retained caller contract

Spec base: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.
Implementation base: `ff66dd24df0f90ae85c3f1dab286cd3b0c156c8c`.
Status: provisional dependency implementation; root acceptance of tickets 06, 07 and 15 remains pending.

`tools/agents/land.py` delegates to collab modules and retains the old Python import surface.
Its standalone help and registered `land` subcommand help carry a deprecation notice.
The compatibility route refuses Candidates owned by active collab Attempts.
Operators must use the collab lifecycle for those Candidates.

## Caller inventory

Read commit for this inventory: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.
The table lists observed imports, rather than inferred callers.

| Caller | Retained Python surface | Fresh compatibility proof |
| --- | --- | --- |
| `worker.py` | `resolve_effort_record` | 89 existing worker tests |
| `effort.py` | `EffortRecord`, `resolve_effort_record`, `parse_effort_metadata`, `worktree_for_ref`, `local_mainline_ref`, `scan_effort_records`, `select_effort_record` | 17 existing effort tests |
| `tracker.py` | `repo_root`, `resolve_effort_record`, `parse_effort_metadata`, `git_result`, `scan_effort_records`, `select_effort_record` | 28 existing tracker tests |
| `acceptance.py` | `resolve_effort_record`, `parse_effort_metadata` | 18 existing acceptance tests |
| `wt.py` | `add_parser`, parser callback `run` | Existing landing parser test; integrated worktree shim verification belongs to ticket 17 |

All legacy landing flags remain supported: Candidate, gate, explicit no-gate, no-push,
repository, remote, mainline, owner override, effort and ticket.
Exit codes remain `0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 64` with their old mapping.
Push refusal still returns 6 for compatibility, but reports the actual push cause.
Code 12 is additive and means that code landed while post-landing records remain incomplete.

Additional flags expose the audited safety boundaries: `--expected-tip`, `--gate-timeout`,
`--landing-status` and `--recover-landing`.
An uninitialized repository admits override owners only from local Git configuration,
using repeatable `collab.overrideOwners` values.
Initialized repositories use `mainline_override_owners` from admitted collab configuration.
An owner string alone does not grant authority.

## Safety and recovery

Shared `landing_protocol.preflight` checks the exact target base and local fast-forward conditions.
It detects tracked, untracked and ignored collisions, including path-prefix collisions.
It refuses hidden index flags, staged changes and an active Git operation before push.
The compatibility route repeats preflight and acceptance checks after its gate.
The gate runs outside the landing lock and records argv, subject, outcome, timeout and log digest.
Landing rechecks that receipt and the exact Candidate before acting.

`LegacyJournal` writes atomic, fsynced records beneath the common Git directory.
It does not initialize a collab ledger for a previously uninitialized repository.
Initialized ledgers receive the same events, including standalone owner overrides.
Push intent precedes push; successful push and local movement have distinct durable phases.
Unknown push outcomes remain unresolved and block further landing.
Recovery never retries a push or rolls a remote backward.
It reconciles only the recorded base and tip and requires attributed local movement.

Root integration must call `LegacyJournal(common).require_no_pending()` under the v2 landing lane.
This prevents a new lifecycle landing from bypassing an unresolved compatibility push.

## Verification and limits

Visible red-green slices covered unauthorized owners, expected-tip refusal, ignored-file loss,
a collision during a gate, interrupted push recovery, durable gate receipts and lifecycle bypass.
Fifteen new real-Git shim tests pass, including post-local recovery and bound partial-record repair.
The historical landing suite passes all 31 tests after one owner-configuration fixture addition.
This safety correction follows the root's specification clarification at `c9d537b`.
Historical assertion text, argument use and exit-code expectations were preserved.

Caller suites pass: effort 17, tracker 28, acceptance 18 and worker 89 tests.
The worker suite emits existing subprocess and socket ResourceWarnings; its test result is green.
Regular `py_compile` checks pass, and modified Python source stays within 120 columns.
All 70 unchanged core collab tests pass with the optional record extension absent.

Tracker lint finds two existing malformed later-ticket entries in `evidence/11-report.md`, lines 40 and 41.
This ticket did not change that separately owned evidence.
The full repository gate and independent Standards/Spec review remain root completion gates.
No completion claim is made for pending dependencies or root integration.
