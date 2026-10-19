# Execution and claim-release repair: review round 2

Implementation base: 09ced6d.
Status: implemented; independent verification and acceptance remain with the lead.
Scope: running-check cleanup race, process-group quiescence, and stranded Corenius claims.
The user authorized isolated implementation and commits. Host commands used tools/temp_root.py.
No tracker or effort metadata in real workspaces was changed. CLI tests use disposable real Git repositories.

## Execution fence

Red: a real public check CLI was held inside its gate command. Abandon returned
success and changed checking to abandoned while the gate still owned its workspace.

Green: abandon and GC acquire check, lane, and meta locks in that order.
The check lock is held for the entire gate execution and receipt publication.
Both operations now refuse as busy while the gate runs; its state, registration
locks, and workspaces stay unchanged. Abandon succeeds after gate completion.
Lane and metadata locks remain released during execution, so unrelated landing
preflight is not serialized behind the entire gate.

## Process-group quiescence

Red: a timed-out leader exited on TERM while its real descendant ignored TERM.
The CLI returned with the descendant still alive. A second red case had a leader
exit zero while leaving the same kind of child alive; the old code issued a passed receipt.

Green: timeout termination inspects live members of the entire process group,
allows a one-second TERM grace, sends KILL when needed, and keeps the execution
fence and log open until the group has no live members. Zombie processes do not
own workspaces or descriptors and do not prevent quiescence. The leader is reaped.
The same inspection runs after every ordinary leader exit. Surviving descendants
produce an error receipt, including when the leader exited zero, and are drained
before publication. Timeout retains its distinct timeout outcome.

These checks target the command's declared process group on Linux. Deliberate
escape into another session remains outside this local cooperative tool's execution boundary.

## Tracker-owned claim release

Red: normal Corenius abandonment changed SQLite only, leaving the committed ticket
claimed. Another red case changed the ticket owner; the old code still abandoned
without validating the authoritative claim.

Green: tracker.py owns a new release transition. It verifies the exact owner,
Attempt identity, and expected ticket SHA-256. Dry run computes the exact result
hash and timestamp without editing the ticket. The actual transition clears
Status/Owner/Claimed to open/empty/empty and records the abandoning Attempt.

Collab persists a claim_release_intent and sets state abandoning before requesting
the transition. The intent binds the old owner and original ticket hash, expected
result hash, path, and timestamp. Only the expected ticket edit can be staged and
committed. The Attempt becomes abandoned after this commit. Pending legacy or
core landings block the Git mutation.

Recover completes abandoning from any of these proven interruption points:

- after_claim_release_intent, before tracker mutation;
- after_tracker_release, with the exact expected edit still uncommitted;
- after_claim_release_commit, with the exact released bytes committed.

The public tests prove that fresh admission, GC, refresh, and sweep cannot bypass
an abandoning Attempt. Recovery commits at most one release comment, then permits
fresh admission. The original Attempt retains its owner and ticket hash; the fresh
Attempt records its new owner and new hash. Unexpected ticket/index/operator edits
remain untouched and cause a refusal. A newer independently claimed ticket also
causes recovery to refuse, preserving the original release intent.

The existing active-ticket unique index and status/sweep terminal filters already
retain abandoning as active; no SQLite schema change is needed. Recover now
handles abandoning explicitly, and refresh refuses pending release.

Refresh continues to transfer an unchanged claim under its existing operation
locks. Its old Attempt is already terminal, so normal abandon refuses to release
that transferred claim. The public refresh-transfer regression verifies this.
A changed Corenius ticket hash remains a conservative refusal until reconciled;
release cannot infer that a different claim or changed contract is the original one.

## Verification

```sh
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_execution_repair.py -v
# 10 public real-Git/process regressions passed
python3 tools/temp_root.py -- python3 tools/agents/test_tracker.py
# 28 existing tracker tests passed
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_refresh_source.py -q
# 10 existing refresh/source tests passed
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_legacy_landing.py -q
# 15 existing compatibility landing tests passed
python3 tools/temp_root.py -- python3 -m compileall -q tools/collab tools/agents/tracker.py
python3 tools/temp_root.py -- git diff --check
```

Existing timeout receipt, durable receipt, worker liveness, interrupted allocation,
GC preservation, and core push-repair tests were also checked at the CLI seam.
The lead owns the combined-source full gate and the next independent review.
Root's records_identity method and its preflight call are an integration dependency;
this branch does not duplicate or edit that independent identity repair.
