# Core landing repair: independent review round 1

Implementation base: d4273dce28cd9754342dba0c1df5a46996a08d84.
Source reviewed: tools/collab/landing.py at that exact commit.
Scope: two P1 findings from independent review round 0, assigned by the lead.
The user authorized implementation and commits. This branch has one writer.
No tracker operations or shared checkout writes occurred. No real workspace
cleanup or shared remote push occurred. No reset, rebase, or force removal was
introduced. All Git mutations in tests affect disposable repositories.
Host commands used tools/temp_root.py. The approved test seam is the public CLI.

## Terminal cleanup

Red: a real live worker bound with --worker-pid passed submit, check, and local
landing. The existing GC returned zero and removed both worktrees despite the
live worker. The second red case changed a recorded run.json into invalid JSON;
GC also removed both worktrees despite unknown liveness.

Green: GC checks require_worker_stopped before acquiring locks, after acquiring
locks, and immediately before each removal. Every selected worktree is validated
for identity, cleanliness including ignored files, and registration lock reason
before any unlock or removal. A locked Attempt tree accepts only collab:<aid>;
a locked check tree accepts only collab-check:<aid>. Empty or foreign lock reasons
are refusals. Unlocked trees need no unlock command. Failed removal restores the
same owned lock reason, rather than changing its identity.

The test checks external locks on each of the two trees after the real worker
exits. The complete worktree registration report stays unchanged on refusal,
which proves the other tree was not partially unlocked or removed. Restoring the
exact owned locks allows the original safe terminal cleanup to remove both trees.

## Ambiguous push outcome

Red: a real executable configured as remote.origin.receivepack runs Git's actual
receive-pack, then exits 1. The remote reaches the Integration commit, local
trunk stays at base, and the old code changes the Attempt to verified.
Recovery then does nothing. Three additional red cases prove the same incorrect
rollback with an unreadable remote, remote base/absence, and a late ignored-file collision.

Green: push intent records an inspected remote predecessor or an explicit
inspection error. A nonzero push remains push_intended unless both facts hold:

- Complete porcelain output explicitly rejects the single intended remote ref.
- Successful inspection before and after the push proves that its nonempty
  predecessor remained unchanged.

Authentication, transport failure, ambiguous remote failure, missing remote
history, and unreadable outcomes remain unresolved. Failure events retain the
real cause, predecessor, later remote observation, inspection error, and whether
an unchanged rejection was proven. Diagnostics name inspect and recover commands.

Recovery inspects the recorded destination. If it acknowledges the exact
Integration commit and local trunk remains at base, recovery repeats the complete
receipt, trunk, Candidate, ticket, and collision preflight before local fast-forward.
It performs no push retry. Remote base or absence alone never proves nonexecution.
Uncertain cases remain unresolved for operator inspection.

The regression's receive-pack invocation log contains exactly one invocation
before and after recovery. A second test moves the bare remote out of reach;
recovery refuses and keeps push_intended until the same remote becomes inspectable.
The late ignored file and a tampered check log both block recovery without local
trunk movement. Restoring their original state permits completion.

## Narrow historical test correction

The lead authorized correcting the two authentication/network subcases in
CollabTests.test_push_failure_reports_authentication_and_network_distinctly.
Their shared expectation changes from verified to push_intended. Cause messages
and the assertion that local trunk stays at base remain unchanged.
No other existing test expectation changed.

## Verification

```sh
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_core_repair.py -v
# 6 regressions passed
python3 tools/temp_root.py -- python3 -m compileall -q tools/collab
python3 tools/temp_root.py -- git diff --check
```

Affected original CLI cases for post-push crashes, started landing recovery,
explicit non-fast-forward rejection, distinct authentication/network causes,
dirty-content-preserving GC, and worker liveness also passed.
The complete original test_collab.py suite ran 80 tests: 79 passed. The only
failure is the frozen base's separate structure failure at ticket_resolution.py:103,
which is outside this repair's ownership. The lead was notified. Independent
review and the full repository gate remain with the lead.

The 15 public compatibility landing tests passed. The compatibility journal
bridge is preserved: landing still refuses a pending
legacy landing before a new push intent. This repair adds no compatibility-journal
state or write path.
