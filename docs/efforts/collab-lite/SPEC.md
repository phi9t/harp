# Collab Lite implementation specification

Base commit: d31c3df460132e9ab87c0a52bffd949c387543e8
Status: ready-for-agent; user approved standalone scope and CLI test seam

## Outcome
Provide a local CLI for cooperating agents on one Linux machine. Git owns
Markdown tickets and accepted code. SQLite owns attempts and operation receipts.
Normal agent CLIs run independently inside assigned Git worktrees.

## Source and boundary
The user supplied the 609-line collab.py prototype, its README, proposal and
recorded results. Preserve the described command interface. Treat sample commands
and prior results as source material, not live authority or current verification.
The proposed tools/collab/ location follows the engineering-tool boundary in
ADR 0001-semantic-corenius-component-layout. Do not replace the retained controller
or activate .tickets in Corenius. No synchronized policy changes or live migration.

## Required behavior
1. init configures committed clean trunk, a nonempty JSON string array for check
   argv, external workspace root and shared state in the Git common directory.
   Reject repeated/partial initialization without overwriting state.
2. new authors narrow tk-compatible ticket front matter. ready/status read only
   committed trunk tickets. Validate exact IDs, regular-file mode, dependencies,
   missing dependencies and cycles. SQLite never duplicates the authoring ledger.
3. claim reserves one nonterminal attempt per ticket transactionally, hashes the
   exact committed ticket bytes, records base, owner, unique codex-prefixed branch
   and workspace, then creates a locked worktree. Failed allocation retains claim.
4. submit requires assigned clean workspace/branch in this same repository,
   unchanged admitted contract and dependency requirements, nonempty linear
   contribution based on current trunk. Protect .tickets/, .github/, .corenius/,
   docs/agents/, AGENTS.md and CLAUDE.md. Seal the exact candidate via Git refs.
5. check prepares a fresh detached integration commit containing the candidate
   delta and ticket closure with the exact expected parent. Run stored argv without
   shell interpolation. Store exact subject, argv, log hash, return code and
   workspace/HEAD conditions. Publish verified state and its check receipt in one
   transaction. A zero exit alone is insufficient: workspace and HEAD must remain
   unchanged and trunk must equal the recorded base. Interrupted checking fails.
6. land is an explicit human/operator action. Require verified exact integration,
   durable matching check receipt, unchanged predecessor/contract and clean trunk
   checkout. Record landing intent before Git; use fast-forward-only merge with
   ignored-file overwrite protection. Never push or silently close after checking.
7. recover/abandon/gc require explicit process-stop confirmation. No automatic
   leases, relaunch, timeout reassignment, force removal or reset of operator files.
   Recover allocation only by validating existing registered workspace identity.
   Interrupted check becomes check_failed. Reconcile lost landing acknowledgment
   through target ancestry and attributed reflog evidence. Ambiguity fails closed;
   record a durable pre-Git prepared phase separately from a Git-started phase.
   A prepared operation with unchanged base may return to verified. Once Git may
   have started, absent attribution is ambiguous even if trunk equals base;
   automatic retry is refused. Attributed integration ancestry allows landed
   reconciliation. Lost/truncated history never establishes nonexecution. Every unresolved
   landing phase globally blocks other final checks and landings until reconciled.
8. gc only removes validated attempt/check worktrees in terminal attempts, refusing
   tracked, untracked and ignored contents. Retain branches, sealed refs, logs,
   database history and older check worktrees. Never remove another repository or
   substituted workspace even if clean.
9. backup creates a new SQLite snapshot, refusing overwrite. Explain that Git refs,
   workspaces and logs are also needed for a complete backup.
10. Use short BEGIN IMMEDIATE transactions, uniqueness constraints, bounded busy
    timeout, synchronous FULL and DELETE journal. No database transaction spans
    Git/test execution. Metadata and lane advisory locks serialize cooperating
    mutations; children retain needed locks across parent death.
11. Host subprocesses and tests use a safe absolute owned temporary root through
    tools/temp_root.py. The CLI must also preserve/validate a safe root for its
    children. No host temporary allocation under the system temporary directory.

## Stale trunk and authority
Reject stale submission/check/landing. Preserve old workspace and branch; operator
stops/abandons the attempt, claims a new attempt at current trunk, cherry-picks
only the prior contribution, resolves conflicts, tests, submits and checks again.
Never prescribe or execute rebase. Do not merge into a shared Corenius branch.
Disposable test repos may create commits and exercise local landing for proof.

## Testing and integration
Approved public seam: CLI subprocesses using real disposable Git repositories,
observing JSON, command exit, Git history/files/worktrees and check logs. No tests
of private methods or direct mutable database inspection. The user confirmed this seam on 2026-10-11.
Rebuild tests because the attachment includes only a prior transcript, no suite.
Use visible red-green adoption/fix slices; existing prototype behavior can be
characterized after importing the user-supplied implementation. Cover race claims,
dependencies/cycles, shared worktree state, contract changes including CRLF,
protected paths, dirty workspaces, failed/mutating checks, exact closure/landing,
stale trunk and refresh, retained claims, interrupted allocation/check/landing,
unattributed movement, lost acknowledgment, backup and conservative cleanup.
Register a focused checks.d script; run focused checks and the repository gate.
Report unrelated unavailable gate fixtures separately from implementation failures.
Perform independent Standards and Spec review plus a separate branch defect audit.

## Exclusions
Controller replacement, existing ledger migration, process supervision, automatic
scheduling/retries, model gateways, hostile-code sandboxing, distributed execution,
power-loss guarantees, transitive document pinning, full YAML/tk integration,
authenticated review, budget accounting and remote publication.
