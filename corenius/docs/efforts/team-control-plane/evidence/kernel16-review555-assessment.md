# Review555 validity assessment — changes required

Inspected committed candidate `66e221a97465bfc4e17917a76b2db18fc657e97f`, frozen review8 source. Independent Spec reviewer and root source inspection agree all four findings are valid. This is read-only assessment, not a regression run or future-source PASS.

1. Cancellation: confirmed episode effects can lack independent binding to the physical workspace whose Stop is observed. A request/reply association alone cannot authorize completed cancellation. Require exact independently observed effect/execution correspondence; otherwise quarantine and hold reservations. Public negative must include an actual external survivor, with genuine registered stopped-cancellation positive control.
2. Pause: episode driver propagates DependencyUnmet into a nonretryable Activity failure. Preserve authenticated Prepared journal visibility and durable waiting while paused, then resume with exactly one effect. Stale/closed WorkflowRef, wrong scope and epoch errors must remain refusals.
3. Wait identity: Progress combines legacy admission and caller-supplied episode UUIDs, then the driver guesses by lookup. Use typed protocol references through progress/driver/rollover. Same-task and foreign-task collisions must preserve exact routing and authorization. Historical deterministic replay compatibility needs explicit treatment; no silent ambiguous-reference reinterpretation.
4. PostgreSQL-only fixture: artifact repair-cycle test unconditionally unwraps a Temporal-only evidence variable. Keep all behavioral assertions while supporting the documented PG-only runner with that variable absent and optional export when present.

Author repairs proceed in its own tree. Root candidate remains committed, unaccepted and unlanded. New evidence and frozen source reviews are required; prior full gates are historical evidence.

## Typed wait history compatibility assessment

Independent read-only assessment of accepted ticket13 fixed-base behavior confirms raw UUID waits represented admissions only. WorkflowInput and Progress serialize waits into recorded drain Activities and Continue-As-New inputs, so a decode/re-encode representation change can break deterministic replay even if UUID values match. Missing wire version must retain v1 admission-only raw shape and omitted version on historical replay; new v2 references are tagged by protocol. Transition through a recorded v2 Activity result, not reinterpretation of recorded v1 results. Ambiguous unaccepted assembled raw histories require a precise incompatibility disposition, never lookup-based guessing. Retained v1 replay/rollover, raw-shape roundtrip and colliding tagged controls remain required. This describes a repair constraint, not approval of new code.
