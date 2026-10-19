# Final repair: controller interruption and inherited fences

Implementation base: 795b571.
Status: implemented; final independent verification and acceptance belong to the lead.
This is repair round 3, the final authorized repair iteration.

## Public red evidence

A real check CLI ran a separate gate process that ignored TERM. SIGINT was sent
only to the controller PID. The controller returned while its gate was still
alive, leaving checking state available for unsafe recovery and subsequent cleanup.

Two real legacy landing cases paused Git's actual receive-pack inside a bare
remote's pre-receive hook. SIGINT was again sent only to the controller PID.
In the initialized case, backup incorrectly acquired the landing lane while the
remote child still owned its operation. In the uninitialized case, recovery
incorrectly escaped the inherited landing fence before receive-pack completed.
All three regressions failed against the frozen base before the repair.

## Repair

CheckRunner now drains every spawned gate process group in a finally block.
This includes KeyboardInterrupt and other exceptional exits. Cleanup defers
controller SIGINT until the entire group is quiescent, then restores the previous
signal mask. The execution lock and log stay open during this cleanup.
An interrupted check retains checking state with no new passed receipt;
explicit recovery can mark it failed after its gate has actually stopped.

StateStore.lock now closes the controller's descriptor instead of calling
LOCK_UN. Inherited descriptors refer to the same open file description. Explicit
unlock would remove the child's fence too; closing only the parent reference
keeps that fence until the last inherited reference closes.

The legacy landing lock applies the same close-only rule. Its optional initialized
lane already delegates to StateStore.lock. LegacyJournal has no independent flock
finalizer and requires no change. Journal protocol and recovery decisions are unchanged.

The legacy public tests verify that recovery remains blocked throughout the
paused receive-pack. The initialized test also proves backup refuses and creates
no database backup. After the hook is released, the remote advances to the exact
Candidate and recovery fast-forwards local mainline to that acknowledged tip.

## Verification

```sh
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_interrupt_repair.py -v
# 3 new public SIGINT regressions passed
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_execution_repair.py -q
# 10 existing execution and claim-release regressions passed
PYTHONPATH=tools/collab/tests python3 tools/temp_root.py -- python3 -m unittest test_legacy_push_repair test_legacy_landing -q
# 22 existing public legacy landing/push tests passed
python3 tools/temp_root.py -- python3 tools/agents/test_land.py
# 31 existing landing shim tests passed
python3 tools/temp_root.py -- python3 -m compileall -q tools/collab
python3 tools/temp_root.py -- git diff --check
```

Every mutation in these tests affects a disposable Git repository under an owned
temporary root. No real workspace cleanup, external publication, or shared checkout
mutation occurred. StateStore reserve/latest_event work and root bookkeeping identity
changes remain with their assigned owners; this repair changes only the lock-context
hunk in StateStore, CheckRunner cleanup, and the legacy lock-context finalizer.
The lead runs the final combined-source gate and independent review.
