# Collab Lite CLI implementation report

Status: complete; independent reviews and required checks reconciled
Date: 2026-10-11
Base read: d31c3df460132e9ab87c0a52bffd949c387543e8
Candidate: codex/collab-lite, uncommitted isolated worktree
Ticket: [01-cli.md](../tickets/01-cli.md)
Specification: [SPEC.md](../SPEC.md)

## Result

Adapted the supplied standalone CLI into `tools/collab/`, with 39 fresh public
CLI integration tests, an operator guide, root README navigation, and gate
registration. Git owns committed ticket contracts and accepted code. SQLite
owns claims, attempt transitions, and exact check receipts in the common Git
directory. The retained controller and shared policies were not changed.

The prototype and attachment README were implementation inputs. The historical
attachment transcript did not supply a test suite or establish current results.
The rebuilt suite observes CLI JSON, exits, logs, files, refs, worktrees, and
real process interruptions in disposable Git repositories. It never inspects
or mutates SQLite through test SQL.

## Design corrections

- Check completion publishes its state and durable receipt in one transaction.
  Landing validates the receipt's integration, predecessor, candidate, argv,
  workspace conditions, log path, and log hash. `show` exposes the receipt.
- Landing separates `landing_prepared` from `landing_started`. Prepared state
  at unchanged base may recover to verified. Started state without attributed
  evidence stays ambiguous, even at base. All unresolved phases block final
  checks and landings globally, and cannot be abandoned.
- Lost acknowledgments require the attributed integration reflog entry, its
  expected predecessor, and integration ancestry. Missing history proves
  neither nonexecution nor successful landing.
- Initialization requires an actual JSON list of nonempty strings without
  NULs. Exact committed ticket bytes are read without newline conversion.
  Closure preserves the original ticket line endings.
- Submit, allocation adoption, and cleanup validate registered workspace,
  common directory, exact top-level, assigned branch, and applicable HEAD.
  Cleanup validates all workspaces before unlocking any. Primary repository
  identity is validated before default-root Git operations and authoring.
- An attempt cannot silently change its admitted base. Refresh preserves the
  old branch/workspace, abandons explicitly, claims anew, and cherry-picks only
  the previous contribution. Empty file deltas and merge commits are refused.
- The CLI reuses `tools/temp_root.py` as an explicit runtime dependency.
  Its children inherit a validated safe temporary root. Forbidden output
  locations are refused, and tests allocate only under a validated root.
- Backup creates its destination exclusively under the operation locks.
  Existing files, dangling symlinks, and competing writers cannot overwrite
  a snapshot.

## Red-green evidence

Each regression was introduced through the CLI seam and run before its fix.
Characterization tests of existing behavior followed these adoption slices.

| Slice | Observed red result | Observed green result |
| --- | --- | --- |
| Adoption happy path | CLI file absent; first test failed at init | Imported prototype; 1 test passed, checked commit and closure landed together |
| JSON argv | JSON string accepted as argv | Strict list validation; 2 tests passed, invalid string/object/empty/NUL inputs refused |
| Exact CRLF bytes | Claim hash differed from raw committed CRLF ticket | Binary Git read and byte-preserving closure; 3 tests passed |
| Atomic receipt | Publication interruption scenarios completed normally because hooks were absent | Before commit: exit 86, checking, no receipt, land refused. After commit: exit 86, verified with matching receipt; 5 tests passed |
| Landing phases | Pre-Git state was undifferentiated `landing`; started interruption was unsupported | Distinct durable phases, conservative recovery, and global guards; 8 tests passed |
| Temp and branch | Unsafe direct TMPDIR accepted; claim branch lacked `codex/` | Validated child temp and output roots, codex branch prefix; 10 tests passed |
| Workspace identity | Allocation adopted moved HEAD; cleanup removed changed detached HEAD; substitution rejection occurred only after Git operations | Identity checks before mutation, complete cleanup preflight; 14 tests passed |
| Stale admission | Old claim accepted a private branch fast-forwarded to newer trunk | Submit refused changed admitted base; focused regression passed |
| Primary substitution | Landing updated a replacement clone and marked original attempt landed | Original common-dir/top-level validation; focused regression passed |
| Backup symlink | Backup followed a dangling destination link and created its target | Destination refused and target untouched; focused regression passed |
| Empty delta | An allow-empty worker commit was accepted | No file changes refused; focused nonempty/linear regression passed |

## Fresh verification

Environment observed here: Linux, Python 3.11.2, Git 2.39.5, SQLite 3.40.1.
All host commands used the repository temporary-root wrapper.

- `python3 tools/temp_root.py -- sh tools/agents/checks.d/65-collab.sh`
  passed: **39 tests, 23.341 seconds**, exit 0. Final transcript:
  `.scratch/collab-lite/focused-final.log` (regenerable scratch output).
- `python3 tools/temp_root.py -- sh tools/agents/checks.d/00-temp-paths.sh`
  passed: temporary-path scan green and **7 helper tests**, exit 0.
- `python3 tools/temp_root.py -- git diff --check` passed, exit 0.

Coverage includes concurrent claims, linked-worktree shared state, committed
authoring, exact IDs and regular modes, dependency existence/cycles/closure,
protected paths, dirty worker/operator files, exact CRLF contracts, immutable
stale bases and fresh-claim refresh, literal argv, failed/file-mutating/HEAD-moving
checks, trunk movement during checking, receipt interruption and log mismatch,
exact integration closure/landing, allocation interruption/adoption/failure,
prepared/started/acknowledgment recovery, unattributed/unrelated movement and
lost history, inherited live-child lane locks after parent SIGKILL, ignored-file
overwrite protection, safe conservative cleanup, backup destination races,
and direct CLI temporary-root validation.

## Review and limits

Product code, tests, README changes, and gate registration are frozen for the
parent's independent Standards/Spec reviews and separate defect audit. Broader
repository verification remains pending. This report does not claim review
acceptance or full-gate success; unavailable unrelated fixtures must be reported
separately by the verifier.

The CLI remains a cooperative Linux tool, without hostile-code isolation,
distributed execution, process supervision, automatic leases/retries, full
YAML/tk integration, transitive contract pinning, authenticated approval, remote
publication, or power-loss guarantees. A started landing with no trustworthy
attribution intentionally blocks automatic retry, including a Git-refused
ignored-file collision. Manual reconciliation is outside this CLI's commands.
Old check worktrees and history are retained; bulk pruning is outside scope.

No Corenius commit, staging, push, PR, merge, live init, or ledger migration
was performed. Commits and local landings occurred only inside disposable test
repositories. The automatic effort dehydrate command was not run because it
commits; the handoff state is recorded manually in `STATE.md`.

## Repair round 1

Date: 2026-10-11. Independent Standards, Spec, and defect reviews identified
four blockers. The parent returned sole-writer ownership for this bounded
repair. Each fix has a fresh public CLI regression, observed red before the fix
and green afterward. The original 39-test result above remains historical
evidence for the initial implementation.

| Finding | Observed red | Observed green |
| --- | --- | --- |
| Hidden index flags | `gc` removed a worktree with unsaved bytes hidden by `assume-unchanged`; both `assume-unchanged` and `skip-worktree` check writes verified; hidden skip-worktree content submitted; primary hidden bytes reached started landing before Git refusal | Five regressions passed. Clean operations and check conditions refuse hidden index tags; files and flags remain unchanged; primary refusal occurs before Git starts |
| Redirected check child | A configured `git rev-parse HEAD` check reported primary base while the CLI verified integration | Regression passed with inherited `GIT_DIR`, `GIT_WORK_TREE`, `GIT_COMMON_DIR`, and `GIT_INDEX_FILE`; check reports exact integration through the shared sanitized child environment |
| Delimiter parsing | An authored valid `T---one` ticket was absent from ready | Full new/commit/ready/claim/submit/check/land regression passed; scalar triple hyphens, body horizontal rules, and CRLF bytes remain intact except exact status closure |
| Final inspection failure | A zero-exit check removed its own disposable workspace; state remained checking with no receipt | Regression passed: check_failed with exit 0, false clean/HEAD/identity conditions, failed receipt, and inspection diagnostic; land refused |
| Authoring validation | Self dependency created an invalid ticket and dirtied the operator checkout | Duplicate/self dependency regression passed; new refuses before writing files |

The landing comment now names the human operator, consistent with `CONTEXT.md`.
The README documents hidden-index refusal, Git environment sanitization,
standalone delimiters, and failed final-inspection receipts. Expected final
inspection errors are captured before the single receipt/state transaction;
unreadable trunk inspection records an unknown target rather than success.

Fresh repair verification:

- `python3 tools/temp_root.py -- sh tools/agents/checks.d/65-collab.sh`
  passed: **48 tests in 27.768 seconds**, exit 0. Transcript:
  `.scratch/collab-lite/repair-1-final.log`.
- `python3 tools/temp_root.py -- python3 tools/agents/tracker.py lint`
  passed, exit 0. The implementation report lives in `evidence/` because a
  numbered report under `tickets/` collided with the tracker ticket ID.
- `python3 tools/temp_root.py -- sh tools/agents/checks.d/00-temp-paths.sh`
  passed: temporary-path scan green and **7 helper tests**, exit 0.
- `python3 tools/temp_root.py -- git diff --check` passed, exit 0.

Product files are frozen for independent re-review. Broader gate results and
review acceptance remain the parent's responsibility; this repair report does
not claim either. No Corenius staging, commit, push, PR, merge, live setup,
or migration occurred. The original cooperative scope and limits still apply.

## Final review and verification

Independent Standards, Spec and defect re-reviews are clear. The full default
repository gate tested all48 final CLI cases and passed all other suites. Its
initial tracker-lint failure was corrected by moving this report out of ticket
grammar; fresh global lint passed. The aggregate historical exit remains1.
See [verification](verification.md) and [review](review.md) for precise outcomes.
Source hashes remain unchanged since review; no live adoption or Git publication
was performed.

Tracker closure remains pending: resolve refused the absent accepted, landed
Candidate. Implementation is complete; acceptance/landing records await human
Git actions. The claimed lifecycle is preserved.
