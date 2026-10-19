# Ticket13 initial independent review — unaccepted

Read base `963f58b31e518a8162dfe719e386ef9d9843321f` plus accepted12. Frozen
43-file review13 package matched all hashes. Separate Spec and Standards
contexts found five blockers; neither green fixtures nor retained replay clears them.

Spec:
1. `0004.sql:25`, `temporal.rs:221`: command key identity omits operation family.
   Persist/lookup principal+family+key; different families coexist, changed
   arguments within the same family reject.
2. `temporal.rs:819–836`: workflow validation precedes runtime mutex wait; a
   closed/replaced workflow can subsequently dispatch using Owner authority.
   Carry the workflow fence into the durable gateway dispatch-claim transaction
   after acquiring runtime serialization. Queued old work must cause no physical
   invocation; an already-claimed/in-flight RPC remains a separate boundary.
3. Registry lacks durable observed Run ID. Record lifecycle/incarnation with
   generation and observation fencing; actual CAN must update canonical state,
   and late old observations must not regress it.

Standards:
4. `temporal.rs:430–445,683`: generic scope allows Worker/Verifier startup wakeup
   acknowledgment without starting a workflow/timer. Require scheduler authority
   for relay mutation/delivery; unauthorized acknowledgment retains pending startup.
5. `tests/temporal.rs:734–735`: any unsuccessful child exit can count as SIGKILL.
   Reject a preexisting helper exit and assert exact Unix SIGKILL signal.

Atomic command/cursor, rejected callers, actual SDK retry/CAN/SIGKILL/replay,
secret handling and unknown-operation restraint provide useful partial evidence.
Both authors received consolidated repair1 briefs. Retain initial histories and
replay supported versions alongside new evidence; no silent history replacement.
Three repair rounds maximum; provider/vertical assembly still belongs to16.
