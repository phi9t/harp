# Seminar 06 — Mutation, simulation, propagation, and constraints

## Reading route

Read §3.3 as a progression from changing one pair to coordinating a network of changing objects. Mutable pairs make identity and sharing observable. Queues and tables put interface boundaries around linked mutation. The circuit simulator then adds logical time and delayed actions, while the constraint system replaces a one-way calculation with local relationships that can propagate in more than one direction.

- Required: [[evidence/sicp/sicp.pdf#page=370|§3.3.1, printed pp. 342–353 / PDF pp. 370–381]] for mutable pairs, sharing, identity, and cycles.
- Required: [[evidence/sicp/sicp.pdf#page=381|§3.3.2, printed pp. 353–361 / PDF pp. 381–389]] and [[evidence/sicp/sicp.pdf#page=389|§3.3.3, printed pp. 361–369 / PDF pp. 389–397]] for queue and table interfaces over mutable representations.
- Required: [[evidence/sicp/sicp.pdf#page=397|§3.3.4, printed pp. 369–386 / PDF pp. 397–414]] for wires, gate actions, propagation delays, and the agenda. Follow the sample half-adder through [[evidence/sicp/sicp.pdf#page=409|printed pp. 381–382 / PDF pp. 409–410]].
- Required: [[evidence/sicp/sicp.pdf#page=414|§3.3.5, printed pp. 386–400 / PDF pp. 414–428]] for connectors, informants, retraction, contradiction, and bidirectional local constraints.
- Optional: [[evidence/sicp/sicp.pdf#page=378|Exercises 3.16–3.20, printed pp. 350–353 / PDF pp. 378–381]] on counting and cyclic structures; [[evidence/sicp/sicp.pdf#page=395|Exercises 3.24–3.27, printed pp. 367–369 / PDF pp. 395–397]] on table generalization and memoization; and [[evidence/sicp/sicp.pdf#page=403|Exercises 3.28–3.32, printed pp. 375–386 / PDF pp. 403–414]] on circuit construction and agenda ordering. Use them when the representation invariants, rather than the course-wide systems bridge, are your main target.
- Skim on a first pass: [[evidence/sicp/sicp.pdf#page=380|the procedural implementation of mutable pairs, printed pp. 352–353 / PDF pp. 380–381]]; the line-by-line [[evidence/sicp/sicp.pdf#page=384|queue implementation, printed pp. 356–358 / PDF pp. 384–386]] and [[evidence/sicp/sicp.pdf#page=389|table implementations, printed pp. 361–367 / PDF pp. 389–395]] after their interface contracts are clear; and [[evidence/sicp/sicp.pdf#page=423|the full connector dispatcher, printed pp. 395–397 / PDF pp. 423–425]] after tracing one Celsius–Fahrenheit propagation. Do not skim the agenda's FIFO-within-a-time-segment rule or the connector's informant rule; both determine observable behavior.

## System-design problem

Once several objects can mutate shared structure, correctness depends on more than the final values. It depends on which object was changed, which observers share it, which action was admitted first, and whether a completed effect has durable evidence. How can a system expose a small mutation interface, order delayed work deterministically, propagate local relationships, and preserve enough history to recover without silently repeating an external effect?

## Vocabulary

- **Mutable pair**: a pair whose `car` or `cdr` pointer can be replaced after construction with `set-car!` or `set-cdr!`.
- **Identity**: the property that lets two references denote the same changing object, as distinct from two separate objects with equal current contents.
- **Sharing**: two or more paths reaching the same mutable pair, so mutation through one path changes what is observed through the others.
- **Cycle**: a reachable pointer path that eventually returns to a pair already visited; ordinary structural recursion may therefore fail to terminate.
- **Representation invariant**: a condition hidden behind an interface that every mutation must preserve. In SICP's queue, the front pointer alone determines emptiness; when the queue is nonempty, the rear pointer designates its final pair; and insertion into an empty queue resets both pointers to the new pair.
- **Wire action**: a no-argument procedure registered on a simulated wire. `accept-action-procedure!` runs a newly registered action immediately to initialize connected outputs; later, an actual signal change runs all registered actions again.
- **Agenda**: the mutable schedule of future simulator actions, ordered by logical time and FIFO within one time segment.
- **Constraint**: a local relationship among connectors that can derive a missing value from enough known neighboring values and can retract values it supplied.
- **Informant**: the object recorded as the source of a connector value; only that source may retract that value in SICP's connector protocol.

## Argument map

1. **CLAIM.** Pair mutation makes object identity and sharing observable: changing a pair through one reference changes every structure that reaches that same pair, while mutating an equal but separately allocated pair does not. Mutation can also redirect a pointer back into an existing structure and create a cycle. [[evidence/sicp/sicp.pdf#page=370|§3.3.1 develops `set-car!` and `set-cdr!`, contrasts shared with unshared structures, and constructs cyclic lists (printed pp. 342–353 / PDF pp. 370–381)]]. **INFERENCE.** A snapshot of printed contents is insufficient to reconstruct a mutable heap: alias relationships and cycle identity are part of the state.
2. **CLAIM.** A queue interface can hide a mutable representation with front and rear pointers, but its operations must preserve the abstraction's FIFO behavior and its actual empty and nonempty representation rules. [[evidence/sicp/sicp.pdf#page=381|§3.3.2 defines `front-queue`, `insert-queue!`, and `delete-queue!` over a front/rear representation (printed pp. 353–361 / PDF pp. 381–389)]]. **EVIDENCE.** The front pointer alone determines emptiness. When nonempty, the rear pointer designates the final pair; insertion mutates that pair and advances the rear pointer. Insertion into an empty queue sets both pointers to the new pair. Deletion advances only the front pointer, so deleting the final item may intentionally leave the rear pointer aimed at the deleted pair while the empty front pointer remains authoritative. **INFERENCE.** Encapsulation narrows where mutation may occur, but correctness still depends on every implementation path maintaining the representation's real invariants rather than a cleaner invariant the code does not enforce.
3. **CLAIM.** A mutable table similarly hides how keys and records are linked: lookup follows the representation, while insertion either updates an existing record or mutates the headed structure to add one. [[evidence/sicp/sicp.pdf#page=389|§3.3.3 develops one- and two-dimensional table interfaces and their mutable association-list representations (printed pp. 361–369 / PDF pp. 389–397)]]. **INFERENCE.** An operation such as “insert” names an abstract state transition; it does not expose which internal pointer changes, nor does it by itself promise durability, isolation, or crash recovery.
4. **CLAIM.** The circuit simulator separates three responsibilities: wires retain signal values and notify registered actions, gate procedures translate input changes into delayed output changes, and the agenda orders those future actions by simulation time. [[evidence/sicp/sicp.pdf#page=397|§3.3.4 introduces event-driven simulation, defines wire and gate operations, and gives `after-delay` and `propagate` (printed pp. 369–386 / PDF pp. 397–414)]]. **EVIDENCE.** A gate action reads its input signals now, computes the future output value now, and places a closure that will set the output after the gate-specific delay; `propagate` repeatedly takes and runs the agenda's first item. **INFERENCE.** “Requested output,” “scheduled output transition,” and “observed wire value” are distinct simulator states.
5. **CLAIM.** Agenda ordering is semantic, not merely an implementation convenience: time segments are sorted by increasing time, and actions appointed for the same time are kept in a FIFO queue. [[evidence/sicp/sicp.pdf#page=410|The agenda implementation and Exercise 3.32 specify increasing-time segments and insertion order within a segment (printed pp. 382–386 / PDF pp. 410–414)]]. **INFERENCE.** Replacing the queue with last-in-first-out storage can change circuit behavior even if every scheduled action eventually runs, because later actions may have been computed from different earlier signal states.
6. **CLAIM.** Constraint propagation represents relationships as local, potentially bidirectional rules: an adder can compute its sum from two addends or a missing addend from the sum and the other addend; connectors remember values and informants, propagate new information, reject contradictions, and propagate retractions. [[evidence/sicp/sicp.pdf#page=414|§3.3.5 introduces constraint networks and implements adders, multipliers, and connectors (printed pp. 386–400 / PDF pp. 414–428)]]. **EVIDENCE.** The same Celsius–Fahrenheit network computes Fahrenheit from Celsius, then after retraction computes Celsius from Fahrenheit. **INFERENCE.** Nondirectionality here comes from explicit local inverse cases and provenance-aware retraction; it is not arbitrary search and does not make every relation uniquely invertible.

## Mechanism trace

Use SICP's gate delays—an inverter delay of 2, an AND-gate delay of 3, and an OR-gate delay of 5—and the half-adder wiring below. Immediate execution when actions are registered seeds the delayed events that initialize the circuit. After the first input change and propagation, the sample is settled at logical time 8 with `A = 1`, `B = 0`, `D = 1`, `C = 0`, `E = 1`, and `S = 1`. [[evidence/sicp/sicp.pdf#page=399|`half-adder`, wire initialization, the gate actions, and the sample timings appear in §3.3.4 (printed pp. 371–382 / PDF pp. 399–410)]].

```text
A ----\
       OR ---- D --------------------------\
B ----/                                     \
                                             AND(D,E) ---- S (sum)
A ----\                                     /
       AND ---- C ---- inverter ---- E -----/
B ----/        |
               +------------------------------ C (carry)
```

Now change `B` from 0 to 1 at simulation time 8 and call `propagate`:

```text
t = 8: set-signal!(B, 1)
  The OR gate registered its action on B first. The first AND gate
  registered later, and add-action! prepends, so B's action list is
  [first-AND, OR]. call-each therefore runs first-AND before OR.

  first-AND action reads A=1 and B=1, computes future C=1,
  then after-delay(3, set C=1) inserts an action at t=11.

  OR action reads A=1 and B=1, computes future D=1,
  then after-delay(5, set D=1) inserts an action at t=13.

agenda: t=11 [set C=1]
        t=13 [set D=1]

t = 11: first-agenda-item advances logical time and runs set C=1
  C changes 0 -> 1.
  The carry probe was registered first; the inverter registered later
  and was prepended, so C's action list is [inverter, carry probe].
  The inverter action reads C=1, computes future E=0,
  then after-delay(2, set E=0) appends that action at t=13.
  The carry probe then observes C=1 at t=11.

agenda: t=13 [set D=1, set E=0]   // FIFO for this time segment

t = 13: run set D=1
  D was already 1, so the wire sees no signal change and runs no actions.

t = 13: run set E=0
  E changes 1 -> 0.
  The final-AND action reads D=1 and E=0, computes future S=0,
  then after-delay(3, set S=0) inserts an action at t=16.

agenda: t=16 [set S=0]

t = 16: run set S=0
  S changes 1 -> 0.
  The sum probe observes S=0 at t=16.
  The agenda is empty; propagate returns done.
```

The carry changes before the sum because the carry is one AND-gate delay from the changed input, while the new sum must pass through the carry's inverter and the final AND gate. The agenda executes closures sequentially under one logical clock. It does not sleep for two, three, or five wall-clock units, spawn gate threads, or let host scheduling choose an interleaving. Those delays are data used to order a deterministic discrete-event simulation, not concurrent wall-clock execution.

## Rust lens

An idiomatic Rust translation can make the agenda's ownership and ordering policy explicit without translating the whole Scheme simulator:

```rust
// Explanatory sketch; not compiled by this course.
use std::collections::{BTreeMap, VecDeque};

type SimTime = u64;
type WireId = usize;

#[derive(Clone, Copy)]
struct SetSignal {
    wire: WireId,
    value: bool,
}

struct Agenda {
    now: SimTime,
    // BTreeMap orders time; VecDeque preserves FIFO at equal time.
    scheduled: BTreeMap<SimTime, VecDeque<SetSignal>>,
}

impl Agenda {
    fn after_delay(&mut self, delay: SimTime, event: SetSignal) {
        let time = self.now.checked_add(delay).expect("simulation time overflow");
        self.scheduled.entry(time).or_default().push_back(event);
    }

    fn pop_next(&mut self) -> Option<SetSignal> {
        let mut entry = self.scheduled.first_entry()?;
        self.now = *entry.key();
        let event = entry
            .get_mut()
            .pop_front()
            .expect("agenda never stores an empty time segment");
        if entry.get().is_empty() {
            entry.remove();
        }
        Some(event)
    }
}
```

`BTreeMap` supplies increasing logical-time order, while `VecDeque` states the same-time FIFO policy. `checked_add` exposes a simulation-time overflow decision that the Scheme presentation does not discuss. The sketch deliberately omits wire action registries, gate closures, the propagation loop, and stale-event policy. It is non-runnable teaching code.

Running `pop_next` in one loop is deterministic when insertion order is deterministic. Replacing it with one async task per event would create a different system: wall-clock scheduling, cancellation, backpressure, shared-state synchronization, and races would become part of the semantics. A parallel simulator can be designed, but it must preserve or explicitly revise the ordering contract; Rust threads do not automatically implement SICP's agenda.

## Agent-harness bridge

**SICP CLAIM.** An immediate gate action reads its input wires, computes a future output value, and schedules a delayed output-setting closure; the agenda stores that closure until `propagate` later runs it, so observable behavior depends on ordered propagation through shared wires. [[evidence/sicp/sicp.pdf#page=397|§3.3.4, printed pp. 369–386 / PDF pp. 397–414]]

**HARNESS EVIDENCE — Hermes.** At pinned Hermes Agent commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`, the conversation loop calls `_flush_messages_to_session_db` before tool execution. If the flush reports `False`, it marks the turn failed and breaks without running side-effecting tools; `_execute_tool_calls` appears only after that gate. [[evidence/implementations/hermes/snapshot/agent/conversation_loop.py|`agent/conversation_loop.py:6058–6104`]] ([exact lines 6058–6104](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L6058-L6104)) The flush implementation can instead return `None` when persistence is disabled or no session database exists. [[evidence/implementations/hermes/snapshot/run_agent.py|`run_agent.py:1919–1962`]] ([exact lines 1919–1962](../../../../../evidence/implementations/hermes/snapshot/run_agent.py#L1919-L1962)) This is evidence for a persist-before-effect gate in the active session-database path, not proof that every Hermes configuration durably appends a turn. It also does not show that the assistant message is an authorization decision, that every external effect is idempotent, or that the pre-effect row proves what the tool later did.

**INFERENCE.** A recoverable tool path should model at least four distinct states, each with separate evidence:

1. **Proposal** — a structured action has been formed and validated in memory, but no effect authority or durable execution claim follows merely from its existence.
2. **Journal admission** — the exact action identity has been durably accepted before dispatch. Hermes's canonical assistant-turn append demonstrates one persist-before-effect ordering role; it is not itself the effect or an authorization receipt.
3. **Effect** — an executor has attempted the external operation. After this boundary, a crash can leave the world changed even when no success result reached the caller.
4. **Receipt** — durable evidence binds an outcome to the admitted action. An admitted intent without a trustworthy receipt is ambiguous, not automatically safe to replay and not automatically evidence of failure.

**ARCHITECTURE QUESTION — INFERENCE.** Can native and durable orchestration modes share one action identity, journal-admission, executor, reconciliation, redaction, and receipt vocabulary? Proposal records must not grant execution authority; unknown post-dispatch outcomes must enter reconciliation instead of blind retry; and terminal success must depend on verifier-backed evidence. The captured public sources do not establish this complete effect-plane contract.

## Limits of the analogy

SICP's agenda is an in-memory deterministic simulation structure; it is not a write-ahead log, transaction manager, durable queue, or audit record. Running an agenda action is ordinary Scheme procedure application, whereas an agent tool may cross a process or trust boundary, require authorization, and leave an irreversible effect. A wire signal is a simulated Boolean with controlled fan-out, not an external world's authoritative state. A constraint connector's informant supports local retraction, but it is not cryptographic provenance or a multi-writer conflict protocol. Persist-before-effect narrows one crash window; it does not by itself provide exactly-once execution, and an intent without a receipt still requires effect-specific reconciliation.

## Practice

### Manual trace

Start from the end of the mechanism trace at time 16: `A = 1`, `B = 1`, `D = 1`, `C = 1`, `E = 0`, and `S = 0`. Change `B` to 0. Without running code, write the agenda after every wire action and every executed event. For each entry record appointment time, insertion order within that time, the input values captured when the gate action ran, whether `set-signal!` actually changes its target, and which downstream actions it triggers. Predict the carry and sum probe times. Preserve FIFO ordering for equal-time events and distinguish “event executed” from “wire changed.”

### SICP exercises

- Exercise 3.23: implement a deque with front and rear insertion and deletion. Purpose: make a two-ended mutation interface preserve one coherent sequence despite several pointer-update paths. Invariants: the empty representation is unambiguous, every operation preserves forward and rear access, and the printed representation is not confused with the abstract deque contents. No implementation or pointer layout is supplied here.
- Exercise 3.33: define the requested averager constraint using SICP's primitive constraint system. Purpose: test whether a relation can propagate from enough known connectors in either supported direction while respecting connector provenance. Invariants: insufficient information produces no invented value, contradictory assignments are detected, and retraction removes only values justified by the retracting informant. The constraint construction is intentionally left open.

### Transfer prompt

Choose one side-effecting coding-agent action, such as writing a file, starting a subprocess, or publishing an artifact. Specify its proposal identity, admission record, authority check, dispatch boundary, success receipt, failure receipt, and reconciliation observation. Then analyze crashes immediately before journal admission, after admission but before dispatch, after the external effect but before receipt persistence, and after receipt persistence but before the caller sees the result. State which cases permit replay, which require observation, and which must remain blocked. Do not claim “exactly once” unless an external idempotency or reconciliation mechanism actually proves it.

## Dialogue

**PROVISIONAL.** `What evidence lets a harness distinguish an action that was merely proposed, durably admitted, actually attempted, and verified complete after a crash?`

This question is queued; [[knowledge/rsi/sicp/course/dialogue_state|the dialogue state]] still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments|Seminar 05 — State, identity, and environments]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/07-concurrency-serialization-and-interleavings|Seminar 07 — Concurrency, serialization, and interleavings]]
