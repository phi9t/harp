# Ticket01 repair round1/3

Original candidate: 6fb501904a2900aae91139e67c615032925af38f. Standards PASS;
Spec REJECT P1 and independently confirmed P2. P3 is minor; the socket deadline
concern is relevant service robustness. Independent original reports are retained
byte-for-byte at `../reviews/original/`. This author repair does not replace the
root-owned scoped Standards/Spec re-review or independent branch review.

P1: `create_project` now requires a full 40- or 64-character ASCII hexadecimal Git
commit OID, and admission revalidates the persisted base before creating an
attempt. Symbolic, abbreviated, empty and malformed identifiers are refused.
Public negative cases prove no project/revision/event/capacity/receipt appears;
the same idempotency key accepts a subsequent valid command. Positive admission
covers both supported lengths and retains the exact OID. Fixtures and README
use full identifiers. Actual object existence and retention remain ticket04.

P2/P3: message event task/audience derives from authenticated/persisted message
participants. Message/ack events never use the general task visibility shortcut;
replay checks retained sender or recipient using the event's immutable message
identity. This also protects historical v1 events without rewriting them or
migrating the schema. Tests show A→operator content with forged taskB stays hidden
from B in snapshots and events, while A/operator can observe it and A/B can
observe their own peer communication. Acknowledgement replay reaches both original
message participants. Authority and command/query envelopes remain unchanged.

Service concern: socket reads and response writes share one fixed one-second
transport deadline, recomputed before every I/O. A slow byte drip cannot reset
that budget. The real-daemon test observes a healthy queued request in1003ms,
after its original RED observation of2402ms. The service remains sequential;
this change does not claim a preemptive CPU/database execution deadline. A lost
response after a committed command is recovered through the same idempotency key.

Verification: original14 tests plus4 new behavior tests pass (15 public +3 real
process), formatting and clippy all-targets/all-features with warnings denied
pass. `tested-source.json` pins final source hashes; `verification.txt` records
commands/status. All commands reached terminal status. No dependency/schema
change, adapter, provider, Git effect, live trial, landing, push or root-checkout
mutation. Ticket remains claimed. Original source/evidence remains unchanged in
its original commit and original evidence files remain byte-for-byte untouched.

Limits remain: operator execution observations require ticket02 supervisor
validation; supported restore/rollback reconciliation and power-loss proof are
absent; candidate object validation/retention is ticket04. These are unchanged
scope boundaries, not claims of wider baseline qualification.
