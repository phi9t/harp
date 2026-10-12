# Collab Lite

Collab Lite coordinates local agents through committed Git tickets, isolated
worktrees, and SQLite attempts. It is an optional Linux engineering tool.
It does not start agents, replace the retained controller, or activate a ledger
in Corenius. Adoption and any migration require an operator decision.

Git owns `.tickets/<exact-id>.md` and accepted code. The Git common directory
owns `collab/state.sqlite3`, lock files, and check logs. Linked worktrees share
that state. SQLite records attempts and receipts, rather than a second backlog.

## Requirements

Use Python 3.10+, Git, and a local Linux filesystem with `fcntl.flock` support.
The CLI uses the standard library and the sibling `tools/temp_root.py` helper.
If you copy the CLI elsewhere, preserve that relative layout and helper.

Host commands need a safe, absolute, owned temporary root. The wrapper below
validates `TMPDIR`, or selects Corenius's `.scratch/tmp`. The CLI also validates
the root for its Git and check children. System temporary-directory output
paths are refused. Private capsule execution uses its own temporary boundary;
do not bind a host temporary root into a capsule writable.

## Operator workflow

Start with a clean, committed repository on the intended trunk. The following
examples are instructions for deliberate adoption, not automatic setup:

```sh
COLLAB_SOURCE=/absolute/path/to/corenius
PROJECT=/absolute/path/to/project
collab() {
  python3 "$COLLAB_SOURCE/tools/temp_root.py" -- \
    python3 "$COLLAB_SOURCE/tools/collab/collab.py" --repo "$PROJECT" "$@"
}

collab init --trunk main --check-json '["python3", "-m", "unittest"]' \
  --check-timeout-seconds 120 \
  --workspace-root /absolute/path/to/project-worktrees
collab new T-example 'A narrow outcome'
# Edit, review, and explicitly commit the ticket on trunk.
collab ready
collab open-attempt T-example --owner worker-a
```

Use the actual trunk and verification command. `--check-json` must be a nonempty
JSON array of nonempty strings. `--check-timeout-seconds`, when present, must be
positive. Workspace roots must be outside the primary
checkout. Initialization refuses existing or partial state. Repositories can
choose their protected path list with `--protected-paths-json`; omit it for the
Corenius defaults.

`open-attempt` returns an attempt ID, an `owner/topic` branch, and a locked
worktree. A launcher can record the worker with `--worker-pid PID` or
`--worker-run-dir RUN_DIR`; run directories are the `worker.py` layout with
`run.json` and, after exit, `exit_code`. Start the chosen agent in that
worktree. Commit a nonempty, linear contribution on its assigned branch, then
run:

```sh
collab submit a-ACTUAL-ID
collab check a-ACTUAL-ID
# Review the checked diff, receipt, log, and ticket acceptance criteria.
collab land a-ACTUAL-ID --expected-tip CANDIDATE-SHA
```

`submit` seals the exact candidate. It refuses dirty or substituted workspaces,
stale bases, changed ticket bytes, unmet dependencies, empty deltas, merge
commits, and changes to the configured protected paths. The Corenius default
protected paths are `.tickets/`, `.github/`, `.corenius/`, `docs/agents/`,
root `AGENTS.md`, root `CLAUDE.md`, `tools/agents/checks.d/`,
`tools/agents/check.sh`, and root `CONSTITUTION.md`.

Clean-workspace checks conservatively refuse `assume-unchanged` and
`skip-worktree` index entries, even if their files currently match Git.
Submission, primary-checkout operations, cleanup, and final check conditions
apply this rule. The CLI preserves the files and flags for manual classification.

`check` builds a fresh detached integration commit from the candidate delta and
the ticket closure. Its only parent is the admitted trunk base. It runs the
stored argv directly, without a shell, and appends output to a log file that is
readable while the command is still running. Success requires zero exit, no
timeout, a clean workspace, unchanged detached HEAD and repository identity, and
unchanged trunk. Ignored check outputs may remain, but cleanup refuses them. The
verified state and durable v2 receipt are published in one SQLite transaction.
`show` includes the receipt, exact subject, argv, outcome, timeout, conditions,
and log hash. Check children use the same sanitized Git environment as internal
Git commands; inherited repository or index redirection cannot change the check
subject. If completed checking leaves its workspace unavailable, the CLI
publishes a failed receipt with an inspection diagnostic. An unreadable trunk is
recorded as unknown, and cannot pass verification. A timed-out check is recorded
as the distinct `timeout` outcome.

`land` is an explicit operator approval action. It requires the matching
receipt, unchanged check log, contract and predecessor, unchanged sealed
Candidate and a clean primary checkout in the original repository.
`--expected-tip` pins the reviewed Candidate and refuses a moved branch before
publication. By default the configured trunk must have an upstream: landing
verifies the local fast-forward and operator-file overlap first, records push
intent, pushes the exact Integration commit to that upstream, then
fast-forwards the local checkout. `--no-push` skips remote publication and
lands locally only. A fast-forward merge lands exactly the checked commit and
closure together.
Ignored and untracked operator content is rechecked immediately before the
merge so content appearing during a long check is protected against overwrite.
Reserve the primary checkout for the operator; avoid concurrent manual edits
during landing.

## Tickets

Only committed trunk tickets are dispatchable. IDs are exact, without prefix
resolution. Tickets must be regular files with mode `100644` and UTF-8 bytes:

```markdown
---
id: T-example
status: open
deps: [T-prerequisite]
---

# A narrow outcome

## Outcome
Describe the requested behavior.

## Invariants
State the constraints.

## Acceptance
Describe observable proof.
```

Status is `open`, `in_progress`, or `closed`. Dependencies must exist, have no
cycles, and be closed on trunk. The parser supports this narrow tk-compatible
scalar and inline-list subset, including unknown scalar fields. It is not a
general YAML parser or complete `tk` integration. Keep essential requirements
in the ticket: referenced documents are not pinned transitively. An operator's
committed manual closure counts as a dependency decision in this model.
Only standalone `---` lines delimit the header; triple hyphens inside IDs,
scalar values, or body text remain ordinary content. `new` rejects duplicate
and self dependencies before writing an operator file.

Corenius effort tickets use the tracker format under
`docs/efforts/<effort>/tickets/NN-slug.md`:

```sh
collab open-attempt 03 --effort collab-ops --owner worker-a
```

For Corenius efforts, `open-attempt` preflights readiness through the tracker,
then calls `tools/agents/tracker.py claim` to mutate the ticket. The tracker is
the only writer of the ticket file. Collab commits that tracker claim on trunk,
records an Attempt in SQLite, and creates the locked worktree from the committed
claim. Blocked, already claimed, resolved, untriaged or uncommitted tracker
tickets are refused before any Attempt worktree is opened.

## Trunk movement

An attempt cannot change its admitted base. Preserve its workspace and branch.
Stop its processes, then abandon it and claim a new attempt at current trunk:

```sh
collab abandon a-OLD-ID --confirm-stopped
collab open-attempt T-example --owner worker-a
# In the new assigned worktree, cherry-pick only the old contribution commits.
# Resolve conflicts, run focused tests, commit, submit, and check again.
```

Changing the ticket also requires explicit abandonment and a fresh claim.

## Recovery, cleanup, and backup

Use `status` for committed tickets and attempt history, `show <attempt>` for
one attempt, and `inspect <attempt>` to list blockers such as live recorded
workers, hidden index flags, dirty files, ignored files, missing or substituted
worktrees, and busy collab locks. Before recovery or release, inspect and stop
the old CLI, Git, check, and agent processes. Confirmation does not stop them;
`recover` and `abandon` still refuse while a recorded worker PID or worker run
directory is live:

```sh
collab inspect a-ACTUAL-ID
collab recover a-ACTUAL-ID --confirm-stopped
collab abandon a-ACTUAL-ID --confirm-stopped
collab gc a-ACTUAL-ID --confirm-stopped
collab sweep
collab sweep --apply
collab backup /absolute/path/to/new-snapshot.sqlite3
```

Failed allocation retains its claim. Recovery can adopt only the registered
workspace on the assigned branch at its original base. A standalone missing
workspace requires abandonment before another claim. For a Corenius tracker
ticket, recovery may create the missing workspace when trunk moved only by the
tracker claim owned by the Attempt. Interrupted checking becomes `check_failed`
and must run again.

Landing records `landing_prepared` before publication, `push_intended` before a
push, `push_completed` after a successful push, and `landing_started` before
the local fast-forward. A prepared operation at unchanged base can recover to
`verified`. A pushed operation can recover by finishing the local
fast-forward when the remote already names the Integration commit, so the
remote is not left ahead without a record. Once local Git may have started,
absent attribution is ambiguous even at base. Attributed reflog evidence with
the expected predecessor and integration ancestry can reconcile a lost
acknowledgment to `landed`. A successful local fast-forward followed by failed
record writes is reported as `landed_records_incomplete` with a repair step;
after the record blocker is fixed, `recover --confirm-stopped` completes the
landing record. Missing or truncated history never establishes nonexecution.
An unresolved landing blocks all other final checks and landings; it cannot be
abandoned. Preserve the state and reconcile deliberately; there is no unsafe
retry or database-reset command.

`gc` requires a terminal attempt and validates every workspace before removal.
It refuses tracked modifications, untracked files, ignored outputs, repository
substitution, and changed check HEADs. Classify and preserve useful files first.
Git removes the worktrees without force. Branches, sealed refs, logs, database
history, and older check worktrees remain.

`sweep` is dry-run by default. It lists clean unlocked terminal Attempt worktrees
and their branches with the cleanup reason, but removes nothing unless `--apply`
is passed. Applied cleanup removes clean merged terminal Attempt worktrees and
branches. Clean unmerged terminal Attempt branches are first archived under
`refs/collab/archive/` so their commits stay reachable, then the worktree and
branch are removed. Fresh, in-progress and locked worktrees are skipped with a
reason. Foreign unmerged branches are listed with their owner prefix and left
alone. Dirty-worktree archival, stale-ref pruning, old-log pruning and orphaned
check-worktree pruning are not implemented.

`backup` creates an exclusive new SQLite snapshot and refuses overwrite,
including symlink destinations and competing writers. Preserve Git objects,
refs, live workspaces, and logs as well for a complete project backup.

## Limits and verification

This is cooperative, single-machine coordination. Same-user agents can bypass
the CLI and edit shared state. Worktrees and advisory locks are not a hostile
code sandbox. Distributed execution, network filesystems, process supervision,
automatic leases or retries, scheduling, remote publication, authenticated
review, full YAML support, and power-loss guarantees are outside scope.

SQLite uses short `BEGIN IMMEDIATE` transactions, uniqueness constraints, a
five-second busy timeout, `synchronous=FULL`, and DELETE journals. No transaction
spans Git or verification execution. Cooperating metadata and lane locks serialize
mutations; `check` holds a separate live-check lock while the verification
command runs, so the landing lane is not held across the gate. Git and check
children inherit the needed lock descriptors.

Run the fresh public CLI suite in disposable real Git repositories:

```sh
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v
```

`tools/agents/checks.d/65-collab.sh` registers the same suite in the repository
gate. Tests use `COLLAB_FAILPOINT` for process death; leave it unset in normal
operation. Prior attachment results do not establish verification of this copy.
