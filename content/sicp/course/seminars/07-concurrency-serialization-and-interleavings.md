# Seminar 07 — Concurrency, serialization, and interleavings

## Reading route

Read §3.4 by replacing the vague question “which process runs first?” with the sharper question “which event orders does the design permit?” The section starts with shared-state failures, defines a correctness criterion in terms of admissible sequential outcomes, introduces serializers to remove harmful interleavings, and then shows why operations spanning several resources need a global acquisition rule.

- Required: [§3.4.1, printed pp. 403–410 / PDF pp. 431–438](../../../../../evidence/sicp/sicp.pdf#page=431) for read/compute/write interleavings and the requirement that a concurrent result agree with some permitted sequential order.
- Required: [§3.4.2, printed pp. 410–418 / PDF pp. 438–446](../../../../../evidence/sicp/sicp.pdf#page=438) for serializers, account-local protection, and the need to protect a whole operation across multiple accounts.
- Required: [the multi-resource, mutex, and deadlock discussion in §3.4.2, printed pp. 418–427 / PDF pp. 446–455](../../../../../evidence/sicp/sicp.pdf#page=446). Read the atomic `test-and-set!` boundary and the stable account-number ordering closely.
- Optional: [the chapter introduction, printed pp. 401–403 / PDF pp. 429–431](../../../../../evidence/sicp/sicp.pdf#page=429) for the motivation for parallelism; [Exercises 3.38 and 3.40–3.47, printed pp. 409–424 / PDF pp. 437–452](../../../../../evidence/sicp/sicp.pdf#page=437) for more interleaving, account, mutex, and semaphore cases; and [Exercise 3.49, printed p. 426 / PDF p. 454](../../../../../evidence/sicp/sicp.pdf#page=454) for the limit of ordering resources known only after an operation has begun.
- Skim on a first pass: [the hardware-arbiter footnote, printed pp. 423–424 / PDF pp. 451–452](../../../../../evidence/sicp/sicp.pdf#page=451) after identifying why `test-and-set!` must be atomic, and [“Concurrency, time, and communication,” printed pp. 426–427 / PDF pp. 454–455](../../../../../evidence/sicp/sicp.pdf#page=454) after recording its warning that “shared state” is harder to define in cached and distributed systems. Do not skim the deadlock example on printed pp. 425–426 / PDF pp. 453–454.

## System-design problem

Two operations can each be locally reasonable and still corrupt shared state when their internal events interleave. Preventing every overlap sacrifices useful concurrency, while protecting only individual writes leaves compound invariants exposed. How can a system state which interleavings are legal, preserve atomicity across the real invariant boundary, acquire several resources without circular wait, and give delegated work equally explicit capacity, lineage, and cancellation rules?

## Vocabulary

- **Process-local order**: the required order among events in one process, such as read before compute before write.
- **Interleaving**: one total ordering of events from multiple processes that preserves each process's local order.
- **Serial equivalence**: a concurrent execution producing the same result as some allowed sequential ordering of the complete operations. SICP uses this as one useful correctness requirement, not as the only possible concurrency specification.
- **Lost update**: a completed write based on a stale read overwrites another process's update, so the final state reflects only one change.
- **Serializer**: a constructor for a set of procedures whose executions cannot overlap with one another.
- **Critical section**: the region that must execute without conflicting access for an invariant to remain valid. Its boundary may span several reads and writes.
- **Mutex**: a mutual-exclusion object that admits one holder and makes later acquirers wait until release.
- **Deadlock**: a cycle of waiting in which each process holds a resource required by another and none can proceed.
- **Stable resource order**: one total order on resource identities that every operation uses when acquiring more than one resource.
- **Lineage**: the recorded parent/child and path identity of delegated work, distinct from whether that work is currently allowed to execute.

## Argument map

1. **CLAIM.** Concurrent behavior is determined by the allowed ordering of observable events, not by the textual placement of two process definitions. A balance update contains separate read, compute, and write events, so another process can change the balance between the read and the corresponding write. [§3.4.1 gives the two-withdrawal timing diagram and identifies the three events inside the assignment (printed pp. 403–406 / PDF pp. 431–434)](../../../../../evidence/sicp/sicp.pdf#page=431). **EVIDENCE.** Peter and Paul can both read 100, compute 90 and 75 independently, and then leave 75 or 90 depending on which stale result is written last. **INFERENCE.** Reviewing the two operations one at a time cannot prove the shared-state result; the proof obligation is over permitted cross-process event orders.
2. **CLAIM.** One useful concurrent-correctness rule is that the outcome must equal some sequential ordering of the complete operations; this permits harmless overlap and may permit more than one correct answer. [§3.4.1 contrasts a global ban on shared-state overlap with executions equivalent to some sequential order (printed pp. 406–410 / PDF pp. 434–438)](../../../../../evidence/sicp/sicp.pdf#page=434). **EVIDENCE.** A $40 deposit followed by withdrawing half yields 70, while withdrawing half followed by the deposit yields 90; both are sequentially explainable. **INFERENCE.** Correctness therefore needs an operation-level specification. “Deterministic” is not required, and “all tasks eventually finished” is not sufficient.
3. **CLAIM.** A serializer constrains interleavings by allowing at most one execution from a distinguished procedure set at a time. Placing a shared variable's read and dependent write inside one serialized procedure prevents another procedure in the same set from changing the variable between them. [§3.4.2 defines serialization and applies one serializer to square and increment procedures and to account deposits and withdrawals (printed pp. 410–416 / PDF pp. 438–444)](../../../../../evidence/sicp/sicp.pdf#page=438). **EVIDENCE.** Serializing the entire square and increment procedures removes the three nonsequential results and leaves only 101 and 121. **INFERENCE.** The serializer does not make arbitrary code correct; the protected set and procedure boundary must coincide with the state invariant.
4. **CLAIM.** Serializing each account's deposit and withdrawal is insufficient for an exchange whose decision depends on two account balances: another exchange can modify one account after the difference is computed but before both updates finish. The complete exchange must be protected by both accounts' serializers. [§3.4.2 develops exchange and `serialized-exchange` across multiple shared resources (printed pp. 416–420 / PDF pp. 444–448)](../../../../../evidence/sicp/sicp.pdf#page=444). **INFERENCE.** Per-resource mutual exclusion protects local updates, while an invariant spanning resources needs a compound protocol. Exporting the serializer weakens the abstraction barrier by making every caller responsible for composing serializers compatibly; the mutex is the serializer's lower-level implementation mechanism.
5. **CLAIM.** SICP's serializer is implemented by acquiring a mutex, applying the protected procedure, and releasing the mutex; the mutex in turn depends on an atomic `test-and-set!`, because a non-atomic test followed by a set admits two holders. [§3.4.2 implements serializers and mutexes and identifies the atomic boundary (printed pp. 421–424 / PDF pp. 449–452)](../../../../../evidence/sicp/sicp.pdf#page=449). **INFERENCE.** A high-level exclusion guarantee rests on a lower-level indivisible operation. Moving the race into a lock implementation does not remove it.
6. **CLAIM.** Protecting an exchange with both accounts' serializers can still deadlock when two processes enter serialized procedures protecting the same accounts in opposite order. Assigning every account a unique number and always entering the procedure protected by the lower-numbered account first breaks that circular-wait pattern for this known resource set. [§3.4.2 introduces `serialized-exchange`, traces its two-account deadlock, and proposes numbered acquisition order (printed pp. 418–427 / PDF pp. 446–455)](../../../../../evidence/sicp/sicp.pdf#page=446). **INFERENCE.** Stable ordering is a protocol shared by all callers, not a property of either mutex in isolation; it does not solve cases where the complete resource set is unknowable before acquisition.

## Mechanism trace

### Trace A: enumerate an unsynchronized two-update state space

Start with `balance = 100`. Process A withdraws 10 and process B withdraws 25. Expand each assignment into three events:

```text
Aᵣ: read shared balance into local a
A꜀: compute a' = a - 10
A𝓌: write a' to shared balance

Bᵣ: read shared balance into local b
B꜀: compute b' = b - 25
B𝓌: write b' to shared balance
```

Each process must preserve its local order: `Aᵣ < A꜀ < A𝓌` and `Bᵣ < B꜀ < B𝓌`. There are `6 choose 3 = 20` interleavings that satisfy those two constraints. The table enumerates all of them; a row's final number is obtained by executing the read, local computation, and write literally.

| Event order | Reads and writes that determine the result | Final balance |
|---|---|---:|
| `Aᵣ A꜀ A𝓌 Bᵣ B꜀ B𝓌` | A writes 90; B then reads 90 and writes 65 | 65 |
| `Aᵣ A꜀ Bᵣ A𝓌 B꜀ B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Aᵣ A꜀ Bᵣ B꜀ A𝓌 B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Aᵣ A꜀ Bᵣ B꜀ B𝓌 A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Aᵣ Bᵣ A꜀ A𝓌 B꜀ B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Aᵣ Bᵣ A꜀ B꜀ A𝓌 B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Aᵣ Bᵣ A꜀ B꜀ B𝓌 A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Aᵣ Bᵣ B꜀ A꜀ A𝓌 B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Aᵣ Bᵣ B꜀ A꜀ B𝓌 A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Aᵣ Bᵣ B꜀ B𝓌 A꜀ A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Bᵣ Aᵣ A꜀ A𝓌 B꜀ B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Bᵣ Aᵣ A꜀ B꜀ A𝓌 B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Bᵣ Aᵣ A꜀ B꜀ B𝓌 A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Bᵣ Aᵣ B꜀ A꜀ A𝓌 B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Bᵣ Aᵣ B꜀ A꜀ B𝓌 A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Bᵣ Aᵣ B꜀ B𝓌 A꜀ A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Bᵣ B꜀ Aᵣ A꜀ A𝓌 B𝓌` | both read 100; A writes 90, then B writes 75 | 75 |
| `Bᵣ B꜀ Aᵣ A꜀ B𝓌 A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Bᵣ B꜀ Aᵣ B𝓌 A꜀ A𝓌` | both read 100; B writes 75, then A writes 90 | 90 |
| `Bᵣ B꜀ B𝓌 Aᵣ A꜀ A𝓌` | B writes 75; A then reads 75 and writes 65 | 65 |

Only the first and last rows are complete sequential executions, and both finish at 65. The other 18 rows have both reads occur before either relevant dependent write; the final write wins and loses the other withdrawal, producing 75 or 90. The source example uses this same read/compute/write decomposition and illustrates the final value 75. [§3.4.1, printed pp. 404–406 / PDF pp. 432–434](../../../../../evidence/sicp/sicp.pdf#page=432)

Serializing all of A before B or all of B before A reduces the allowed set from 20 event orders to the two sequential orders. It does not choose which process wins admission, but both admitted orders preserve the invariant `100 - 10 - 25 = 65`.

### Trace B: ordered two-account exchange and lock release

Let account `a1` have stable ID 1 and balance 10; let `a2` have ID 2 and balance 20. Peter requests `exchange(a2, a1)` while Paul concurrently requests `exchange(a1, a2)`. The semantic argument orders differ, but both operations normalize lock acquisition to ID order `a1` then `a2`.

```text
Peter: normalize resources [a2, a1] -> lock order [a1#1, a2#2]
Paul:  normalize resources [a1, a2] -> lock order [a1#1, a2#2]

P1  Peter acquires a1#1.
Q1  Paul attempts a1#1 and waits; Paul holds no account lock.
P2  Peter acquires a2#2.
P3  Peter preserves his semantic argument order (a2, a1):
      read a2=20 and a1=10; difference=10
      withdraw 10 from a2 -> 10
      deposit 10 into a1 -> 20
P4  Peter releases a2#2, then releases a1#1.

Q2  Paul acquires a1#1 after Peter's release.
Q3  Paul acquires a2#2.
Q4  Paul preserves his semantic argument order (a1, a2):
      read a1=20 and a2=10; difference=10
      withdraw 10 from a1 -> 10
      deposit 10 into a2 -> 20
Q5  Paul releases a2#2, then releases a1#1.

final: a1=10, a2=20; both exchanges completed
```

The decisive invariant is not “Peter goes first.” It is that no operation can hold `a2#2` while waiting for `a1#1`, because every operation requests `a1#1` first. Thus the two-process wait cycle in SICP cannot form. Releasing in reverse acquisition order mirrors nested ownership and shortens the period during which the inner resource is retained; the deadlock proof here depends on acquisition order, not on a special release order. If an operation discovers a third required resource only after taking one lock, this proof no longer applies automatically.

## Rust lens

```rust
// Explanatory sketch; not compiled by this course. Types and error plumbing are omitted.
use std::sync::Mutex;

struct Account {
    id: u64,                 // unique and stable for the lock-order domain
    balance: Mutex<i64>,
}

fn ordered_exchange(a: &Account, b: &Account) -> Result<(), ExchangeError> {
    if a.id == b.id {
        return Err(ExchangeError::DuplicateResource);
    }

    let (low, high, a_is_low) = if a.id < b.id {
        (a, b, true)
    } else {
        (b, a, false)
    };

    // Every caller acquires the same resource IDs in the same order.
    let mut low_balance = low.balance.lock().map_err(|_| ExchangeError::Poisoned)?;
    let mut high_balance = high.balance.lock().map_err(|_| ExchangeError::Poisoned)?;

    // Lock order is independent of the semantic a -> b argument order.
    let (a_balance, b_balance) = if a_is_low {
        (&mut *low_balance, &mut *high_balance)
    } else {
        (&mut *high_balance, &mut *low_balance)
    };
    let difference = a_balance.checked_sub(*b_balance)
        .ok_or(ExchangeError::Arithmetic)?;
    *a_balance = a_balance.checked_sub(difference)
        .ok_or(ExchangeError::Arithmetic)?;
    *b_balance = b_balance.checked_add(difference)
        .ok_or(ExchangeError::Arithmetic)?;

    drop(high_balance); // release ID-high before ID-low
    drop(low_balance);
    Ok(())
}
```

This sketch translates one concept: define a stable lock order separately from the operation's semantic argument order. It deliberately omits account construction, persistence, authorization, fair admission, cross-process locking, and recovery after a partially external effect.

Rust also adds concerns outside SICP's serializer presentation. `std::sync::Mutex` is **poisoned** if a thread panics while holding its guard, forcing the caller to choose a recovery policy. Cancellation while locks are held normally drops Rust guards when the owning future or scope is dropped, but it can still interrupt the larger logical operation after partial non-memory effects; holding a guard across `.await` also couples lock lifetime to cancellation and suspension. Finally, a blocking mutex acquisition inside an asynchronous runtime can block an executor worker rather than merely suspend the current task. An async-aware mutex changes that scheduling behavior but does not supply transactionality, cancellation safety, or a correct resource order by itself.

## Agent-harness bridge

**SICP CLAIM.** Concurrent correctness requires controlling the interleavings that cross an invariant boundary, while multi-resource operations need one shared acquisition discipline to avoid circular wait. [§§3.4.1–3.4.2, printed pp. 403–426 / PDF pp. 431–454](../../../../../evidence/sicp/sicp.pdf#page=431)

**HARNESS EVIDENCE — Codex.** At pinned OpenAI Codex commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, one `AgentControl` is shared across a root thread/session tree, keeping the registry and rollout budget scoped to that tree; the control initializes a maximum-thread limiter, checks execution capacity before starting a turn, supports a targeted interrupt, and can list a live agent with its spawn descendants. [`codex-rs/core/src/agent/control.rs:90–453`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control.rs#L90-L453) Agent creation reserves capacity before spawning, carries `parent_thread_id`, depth, and agent path through spawn metadata, and commits that reservation only after creation. The spawn path calls a best-effort edge helper; for a non-ephemeral child with a parent and an available graph store, that helper attempts an edge upsert and logs rather than propagates an upsert failure. [`codex-rs/core/src/agent/control.rs:701–729`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control.rs#L701-L729) Fork creation then requires a parent spawn-call identity, resolves the parent thread, flushes its rollout before loading the fork context, and distinguishes full-history from last-N-turn modes. [`codex-rs/core/src/agent/control/spawn.rs:365–720`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/agent/control/spawn.rs#L365-L720) These excerpts directly evidence tree scope, capacity reservation, targeted interruption, and lineage fields, plus a conditional best-effort attempt to record a spawn edge. They do not establish durable lineage, show that interrupting one node automatically interrupts all descendants, or show that reservation order provides fairness.

**HARNESS EVIDENCE — Pi.** At pinned Pi commit `4488ad55c18f07ae89a489096c90de8667b3adfb`, the subagent extension documents single, parallel, and chain modes; parallel mode is limited to eight tasks and four concurrent processes, streams per-task status, and returns each completed child's result or failure to the parent. Its error contract says a user abort kills the subprocess and throws an error, while a chain stops at its first failed step. [`packages/coding-agent/examples/extensions/subagent/README.md:91–175`](../../../../../evidence/implementations/pi/snapshot/packages/coding-agent/examples/extensions/subagent/README.md#L91-L175) This is evidence for bounded fan-out in this example, parent-facing completion aggregation, and one abort behavior. It does not establish nested tree lineage, arbitrary descendant cancellation, or a repository-wide Pi scheduler contract.

**HARNESS EVIDENCE — Hermes.** At pinned Hermes Agent commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`, `delegate_task` permits leaf and orchestrator roles but checks configured spawn depth before building children. It reads the configured maximum concurrent-child count and rejects an oversized batch. Background batches execute as one unit, join every child, and return one consolidated result after all children finish. [`tools/delegate_tool.py:2778–2985`](../../../../../evidence/implementations/hermes/snapshot/tools/delegate_tool.py#L2778-L2985) The excerpt also shows that pausing delegation blocks new fan-out without interrupting already-running children. This is evidence for depth and batch-capacity admission plus join semantics; it is not evidence for cancellation propagation or durable lineage.

**INFERENCE.** Delegation needs at least four separately stated concurrency invariants. No cited harness establishes all four:

1. **Tree-scoped concurrency.** Scheduling and control state belong to a named root tree, so unrelated roots do not silently share a registry or budget. Codex most directly evidences this scope; Pi's example and the cited Hermes range do not.
2. **Bounded capacity.** Spawn and turn admission must reserve from explicit limits, release or reconcile reservations on failure, and define whether waiting is fair. Codex evidences reservations and an execution limiter, Pi documents a fixed example cap, and Hermes checks configured depth and child count. These are different mechanisms, not one shared guarantee.
3. **Cancellation propagation.** A harness must specify whether cancellation targets one turn, one child process, or an entire descendant subtree; whether new fan-out is blocked; and when capacity is released. Pi documents user-abort termination for its subprocess, Codex exposes targeted interrupt and subtree discovery as separate operations, and Hermes explicitly distinguishes pausing new spawns from interrupting running children. The cited evidence therefore reveals the policy choices but does not prove recursive cancellation for every harness.
4. **Lineage.** Every child result, failure, budget charge, and cancellation should remain attributable to a parent edge and stable child identity. Codex carries parent, depth, and path metadata and conditionally attempts a best-effort graph-edge upsert, which evidences lineage intent and metadata but not a durable lineage guarantee. Pi returns per-task results to the parent, and Hermes aggregates task results, but the cited excerpts do not establish durable parent/child lineage for those systems.

**AGENT-HARNESS QUESTION — INFERENCE.** What state machine should an agent harness use so a root-scoped scheduler atomically reserves bounded capacity, records a durable parent/child edge before work becomes runnable, propagates cancellation according to an explicit node-versus-subtree policy, and releases capacity only after every affected child reaches a receipt-backed terminal state?

## Limits of the analogy

SICP serializers protect procedures sharing in-memory variables; they are not an agent scheduler, task queue, process supervisor, budget ledger, or cancellation protocol. A mutex holder blocks conflicting access to a resource, whereas a delegated agent may run independently and only later merge a result. Stable lock ordering prevents one circular-wait pattern for resources known in advance; it does not decide task priority, prevent starvation, bound model tokens, or recover an external effect. Conversely, parent/child lineage and a concurrency cap do not serialize access to files, subprocesses, services, or credentials. Cancellation is also not rollback: killing a child cannot generally undo tools it already invoked. The bridge transfers the discipline of naming admissible orderings and invariant boundaries, not the claim that delegation is literally mutual exclusion.

## Practice

### Manual trace

Start with `x = 10`. Process P performs `x := x * x` using two separate reads of `x`, one multiplication, and one write; process Q performs `x := x + 1` using one read, one addition, and one write. Enumerate every distinct final value admitted when only each process's local event order is constrained. For each value, provide one complete event ordering and mark the exact read values used by the computation. Then put the whole P and Q operations under one shared serializer and identify which event orders remain. Do not assume the multiplication reads `x` only once.

### SICP exercises

- Exercise 3.39: determine which outcomes remain under the exercise's partial serialization boundary. Purpose: test whether protecting only the multiplication computation also protects the outer assignment. Invariants: preserve each process's event order, treat both reads performed by `(* x x)` explicitly, and serialize only the region wrapped by `s`. No list of surviving values is supplied here.
- Exercise 3.48: explain why entering the serialized procedures for numbered accounts in increasing order prevents deadlock for exchange, then revise the account and exchange interfaces accordingly. Purpose: turn “always order protection consistently” into a protocol that can be inspected. Invariants: account numbers are unique and stable, both call directions enter the lower-numbered account's serialized procedure first, both serializers wrap the whole exchange, and every acquired underlying mutex or guard is released on every exit path. No implementation or proof is supplied here.

### Transfer prompt

Take one coding-agent delegation tree with a root, two children, and one grandchild. Define separate invariants for spawn capacity, runnable capacity, token or turn budget, parent/child lineage, targeted cancellation, subtree cancellation, result aggregation, and terminal receipts. Then trace a failure in which the grandchild performs an external effect just as the root is cancelled. State which records must survive, which capacity is released at each transition, and why termination cannot be reported as rollback. Do not use a single “cancelled” bit to stand for all of those facts.

## Dialogue

**PROVISIONAL.** `When an agent harness cancels one delegated task, what evidence should determine whether only that task, its descendants, its siblings, or the whole root run must stop?`

This question is queued; [the dialogue state](../dialogue_state.md) still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [Seminar 06 — Mutation, simulation, propagation, and constraints](06-mutation-simulation-and-constraints.md) · [Course guide](../sicp_course_guide.md) · Next: [Seminar 08 — Streams, delay, and infinite processes](08-streams-delay-and-infinite-processes.md)
