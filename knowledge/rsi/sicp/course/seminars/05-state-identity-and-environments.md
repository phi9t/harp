# Seminar 05 — Local state, identity, and environments

## Reading route

Read Chapter 3's opening as a deliberate change of computational model. A procedure with local state can represent an object's history, but assignment makes time and identity semantically relevant. The environment model then supplies the machinery that substitution can no longer explain: bindings live in linked frames, and a closure retains the environment in which it was created.

- Required: [§3.1.1, printed pp. 297–305 / PDF pp. 325–333](../../../../../evidence/sicp/sicp.pdf#page=325) for local state; [§§3.1.2–3.1.3, printed pp. 305–319 / PDF pp. 333–347](../../../../../evidence/sicp/sicp.pdf#page=333) for the benefits and costs of assignment.
- Required: [the opening of §3.2, printed pp. 320–322 / PDF pp. 348–350](../../../../../evidence/sicp/sicp.pdf#page=348), especially [the frame and lookup definitions on printed p. 321 / PDF p. 349](../../../../../evidence/sicp/sicp.pdf#page=349), followed by [§3.2.1, printed pp. 322–327 / PDF pp. 350–355](../../../../../evidence/sicp/sicp.pdf#page=350) for procedure creation, application, and assignment. Read [§3.2.3, printed pp. 330–337 / PDF pp. 358–365](../../../../../evidence/sicp/sicp.pdf#page=358) for closure-created local state.
- Optional: [§3.2.2, printed pp. 327–330 / PDF pp. 355–358](../../../../../evidence/sicp/sicp.pdf#page=355). Follow one simple procedure application: argument values enter a new frame, the procedure body runs in that extended environment, and the frame becomes unreachable after return unless a closure retains it. This short section is the bridge between the environment rules and the stateful trace below.
- Skim: [§3.2.4, printed pp. 337–340 / PDF pp. 365–368](../../../../../evidence/sicp/sicp.pdf#page=365). Retain the distinction between textual nesting and environment links; revisit the internal-definition details when analyzing mutually dependent local procedures.

## System-design problem

A system often needs an object to remember what happened, yet once a shared binding can change, the meaning of an operation depends on which object is referenced and when it runs. How can local state hide representation and preserve history without confusing a transient call, a retained session, recalled memory, and the external world—and what evidence is needed to reason about each lifetime?

## Vocabulary

- **Local state**: information retained by one computational object and changed across its operations, such as an account balance captured by a withdrawal procedure.
- **Assignment**: an operation such as `set!` that changes the value associated with an existing binding rather than creating a new value-only expression.
- **Identity**: the distinction between objects that can have different histories even when their currently observable contents are equal.
- **Temporal coupling**: dependence of a result on the order in which state-changing operations occur.
- **Frame**: a collection of bindings from names to values, together with an enclosing-environment link except at the global boundary.
- **Environment**: a frame plus its chain of enclosing frames; lookup searches that chain for the first binding of a name.
- **Closure**: a compound procedure paired with the environment in which its lambda expression was evaluated.
- **Aliasing**: two or more references designating the same mutable object, so a mutation observed through one reference affects observations through the others.

## Argument map

1. **CLAIM.** A procedure with a local state variable can model an object's history because later results depend on state retained and updated by earlier calls. [§3.1.1 develops a withdrawal procedure whose balance changes across calls (printed pp. 297–305 / PDF pp. 325–333)](../../../../../evidence/sicp/sicp.pdf#page=325). **EVIDENCE.** Two withdrawals through the same procedure consume one evolving balance, while procedures created by separate calls to `make-withdraw` retain independent balances. **INFERENCE.** The interface may look like an ordinary function call, but its result is no longer determined by the current argument alone.
2. **CLAIM.** Assignment can improve modularity by letting a computational object own the state needed to respond to messages instead of requiring callers to thread that state through the whole program. [§3.1.2 uses local state to decompose a Monte Carlo estimate into independently modeled random-number and experiment objects (printed pp. 305–311 / PDF pp. 333–339)](../../../../../evidence/sicp/sicp.pdf#page=333). **INFERENCE.** State ownership can narrow an interface, but only by moving a changing dependency behind it; the dependency has not disappeared.
3. **CLAIM.** Introducing assignment also introduces time and identity: two objects can be indistinguishable now yet behave differently after different mutation histories, and exchanging the order of operations can change a result. [§3.1.3 develops the costs of assignment through sameness, sharing, and imperative order dependence (printed pp. 311–319 / PDF pp. 339–347)](../../../../../evidence/sicp/sicp.pdf#page=339). **EVIDENCE.** Calls through two names for one bank-account object affect the same balance, whereas calls through independently created accounts do not; sequential imperative updates can also read values written by an earlier statement. **INFERENCE.** Equality of current observations is not a sufficient identity test for mutable objects, and a replay contract must preserve relevant ordering and alias relationships.
4. **CLAIM.** Assignment invalidates the simple substitution model because replacing a variable with the value it once denoted can erase later changes to that binding. [§3.1.3 explains why a variable in an imperative language cannot be understood as merely a name for an immutable value (printed pp. 311–319 / PDF pp. 339–347)](../../../../../evidence/sicp/sicp.pdf#page=339). **INFERENCE.** Reasoning must track storage locations or bindings through time, not just reduce expressions by textual replacement.
5. **CLAIM.** In the environment model, an environment is a sequence of frames; each frame contains bindings, and lookup follows enclosing links until it finds the first binding for a name. [The opening of §3.2 defines environments, frames, enclosing links, and first-binding lookup (printed pp. 320–322 / PDF pp. 348–350, with the definitions on PDF p. 349)](../../../../../evidence/sicp/sicp.pdf#page=349). **INFERENCE.** A name has no context-free runtime value: the current environment determines which binding is visible.
6. **CLAIM.** Applying a compound procedure extends its saved environment with a fresh parameter frame; creating a procedure pairs its code with its defining environment; and assignment changes the binding in the first enclosing frame that contains the variable. [§3.2.1 states the environment rules for procedure application, procedure creation, and assignment (printed pp. 322–327 / PDF pp. 350–355)](../../../../../evidence/sicp/sicp.pdf#page=350). **INFERENCE.** The saved-environment link selects lexical scope, while the current environment selects where lookup and assignment begin.
7. **CLAIM.** A closure can retain private state because the procedure object points to the frame in which it was created, and later calls create temporary call frames whose enclosing link leads back to that retained frame. [§3.2.3 traces `make-withdraw` and `make-account` with the environment model (printed pp. 330–337 / PDF pp. 358–365)](../../../../../evidence/sicp/sicp.pdf#page=358). **EVIDENCE.** After `make-withdraw` returns, its parameter frame remains reachable through the returned procedure's environment pointer; `set!` reached from a later call mutates the retained `balance` binding. **INFERENCE.** Privacy here is a reachability property created by lexical scope, not a guarantee against every form of leakage, concurrent access, or reflective inspection.

## Mechanism trace

Use the stateful constructor from §§3.1.1 and 3.2.3:

```scheme
(define (make-withdraw balance)
  (lambda (amount)
    (if (>= balance amount)
        (begin (set! balance (- balance amount))
               balance)
        "Insufficient funds")))

(define W1 (make-withdraw 100))
```

Evaluating the definition of `make-withdraw` creates procedure object `Pmake`, whose environment pointer is `G`. Calling it with `100` creates frame `E1`; evaluating the inner lambda in `E1` creates procedure object `Pwithdraw`, which is then bound to `W1` in `G`.

```text
G — global frame
  make-withdraw -> Pmake(parameters: balance,
                         body: inner lambda,
                         saved environment: G)
  W1            -> Pwithdraw(parameters: amount,
                             body: if/begin/set!,
                             saved environment: E1)

E1 — retained constructor/procedure-environment frame
  balance -> 100
  enclosing -> G
```

`E1` was the call frame for `make-withdraw`, but it does not disappear after that call returns: `Pwithdraw` still points to it. It is therefore the retained private environment for `W1`. The procedure object is not itself a frame.

First call, `(W1 25)`:

```text
create E2 — first withdrawal call frame
  amount    -> 25
  enclosing -> E1

evaluate (>= balance amount)
  lookup balance: E2 has none -> E1 has 100
  lookup amount:  E2 has 25
  result: true

evaluate (set! balance (- balance amount))
  (- 100 25) -> 75
  search from E2 for existing binding balance
  E2 has none -> mutate E1.balance: 100 -> 75
return balance -> 75
E2 can become unreachable; E1 remains reachable through Pwithdraw
```

Second call, `(W1 40)`:

```text
create E3 — second withdrawal call frame
  amount    -> 40
  enclosing -> E1

lookup balance through E3 -> E1 -> 75
lookup amount in E3 -> 40
(- 75 40) -> 35
set! searches from E3 and mutates E1.balance: 75 -> 35
return 35
E3 can become unreachable; E1 remains with balance = 35
```

The two call frames are different, but their enclosing link reaches the same retained frame. That shared reachability is why the second result depends on the first call. A second constructor call, `(define W2 (make-withdraw 100))`, would create a different retained frame, so mutations through `W1` would not change `W2`.

## Rust lens

The first sketch keeps state explicit and immutable. It returns the next account value rather than changing an existing one:

```rust
// Explanatory sketch; not compiled by this course.
#[derive(Clone, Copy)]
struct Account {
    balance: i64,
}

fn withdraw(account: Account, amount: i64) -> Result<Account, &'static str> {
    if amount < 0 || account.balance < amount {
        return Err("invalid withdrawal");
    }
    Ok(Account {
        balance: account.balance - amount,
    })
}

let a0 = Account { balance: 100 };
let a1 = withdraw(a0, 25)?;
let a2 = withdraw(a1, 40)?;
```

Here the history is represented by distinct values `a0`, `a1`, and `a2`; the caller chooses which version to retain and pass next. Copying an `Account` does not create an alias to mutable storage. The sketch omits overflow policy and uses `?` without supplying a surrounding function, which is why it is labeled non-runnable.

The second sketch gives multiple handles access to one mutable state cell:

```rust
// Explanatory sketch; not compiled by this course.
use std::{cell::RefCell, rc::Rc};

struct State {
    balance: i64,
}

#[derive(Clone)]
struct Withdrawal {
    state: Rc<RefCell<State>>,
}

impl Withdrawal {
    fn withdraw(&self, amount: i64) -> Result<i64, &'static str> {
        let mut state = self.state.borrow_mut();
        if amount < 0 || state.balance < amount {
            return Err("invalid withdrawal");
        }
        state.balance -= amount;
        Ok(state.balance)
    }
}
```

Cloning `Withdrawal` clones the `Rc`, so the handles alias the same `RefCell<State>` and observe one mutation history. `Rc` is single-threaded and supplies shared ownership, not synchronization. `RefCell` moves the exclusivity check from compile time to runtime: an overlapping `borrow_mut` with any live borrow panics. Replacing these types with `Arc<Mutex<State>>` would add a different concurrency and failure contract; it would not merely make this example “thread-safe.” Rust's types make ownership and selected borrowing constraints explicit, while SICP's environment diagrams make lexical reachability and binding mutation explicit. Neither representation by itself supplies persistence, transactionality, replay, or authorization.

## Agent-harness bridge

**SICP CLAIM.** A stateful object's behavior depends on a retained binding and its mutation history, while environment frames distinguish retained lexical state from temporary call bindings. [§§3.1.1 and 3.2.3, printed pp. 297–305 and 330–337 / PDF pp. 325–333 and 358–365](../../../../../evidence/sicp/sicp.pdf#page=325)

**HARNESS EVIDENCE — Pi.** In Pi at pinned commit `4488ad55c18f07ae89a489096c90de8667b3adfb`, `createTurnState` builds a turn state from the session's context messages and metadata plus resources, model settings, tool context, all tools, active tools, and a system prompt. [`packages/agent/src/harness/agent-harness.ts:395–429`](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/agent-harness.ts#L395-L429) This is evidence for construction of a request-facing **turn snapshot**; these lines do not establish that every field is immutable or durably persisted.

**HARNESS EVIDENCE — Codex.** In OpenAI Codex at pinned commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, `run_turn` receives a `TurnContext`, captures a first step context, records injected items through `record_conversation_items`, and can capture another step context later in the turn. The source comment at line 210 defines the value named `world_state` as the exact **model-visible state** used by the turn and its inline compactions; later code refreshes that value with `record_step_world_state_if_changed`. Comments in the same range state that pending input is drained into history before the next model request, subject to two ordering exceptions. [`codex-rs/core/src/session/turn.rs:149–328`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L328) This is evidence that turn/step context, recorded conversation items, history ordering, and exact model-visible state have distinct roles in that loop. The excerpt does not establish physical durability for history.

**HARNESS EVIDENCE — Hermes.** In Hermes Agent at pinned commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`, `MemoryManager` registers memory providers, gathers prefetched context, synchronizes completed turns to providers with a `session_id`, and serializes provider writes through one background worker. [`agent/memory_manager.py:364–735`](../../../../../evidence/implementations/hermes/snapshot/agent/memory_manager.py#L364-L735) This is evidence for a separate provider-mediated **memory** lifecycle. The manager delegates storage and recall semantics to providers, so the excerpt does not prove that every provider persists across sessions or that recalled content is complete.

**INFERENCE.** The sources support treating four state categories as different contracts rather than one “agent memory” bucket:

1. A **turn snapshot** is the model-facing configuration, messages, resources, and active tools assembled for one turn or sampling step; Pi most directly exhibits this construction.
2. **Durable session history** is the ordered user, assistant, and tool-event record intended to support continuation or replay beyond one request; the Codex excerpt records injected conversation items and describes when pending input is drained into history, but it leaves physical durability and replay guarantees to other components.
3. **Cross-session memory** is selectively stored and recalled material intended to outlive the immediate conversational rollout; Hermes exhibits provider-prefetch and turn-sync machinery, while each provider determines actual retention and scope.
4. **External world state** is the filesystem, processes, services, credentials, and other effectful state that a harness observes or changes. This is a course-inferred category and a required design boundary, not a behavior established by the three cited excerpts. Codex's `world_state` variable denotes exact model-visible state; it is not evidence of a filesystem, process, service, or credential snapshot.

These categories may feed one another, but they have different owners, lifetimes, failure modes, and replay claims. None of the three cited harness excerpts implements or proves all four categories.

**AGENT-HARNESS QUESTION — INFERENCE.** What separate types and receipts should an agent harness use for a turn snapshot, an append-only durable session event, a cross-session memory record, and an external-world observation or effect, so retaining one cannot be mistaken for retaining—or being authorized to mutate—the others?

## Limits of the analogy

A SICP frame is a semantic structure for lexical bindings, not a database row, prompt message, operating-system process, credential, or complete snapshot of a machine. A closure's retained frame is not automatically durable across process restart, shareable across workers, or protected from races. Conversely, an agent history or memory record can be serialized, compacted, selectively recalled, or redacted in ways that a small Scheme environment model does not describe. External world state may change independently of the harness and cannot generally be reconstructed by replaying internal bindings. The analogy is useful for asking which state is retained, which references alias it, and which lifetime owns it; it does not establish persistence, consistency, security, or recoverability.

## Practice

### Manual trace

Trace `(define W1 (make-withdraw 100))`, `(W1 25)`, and `(W1 40)` without copying the worked trace. Draw the global frame, the retained constructor frame, both temporary call frames, and every procedure object's saved-environment pointer. For each lookup and `set!`, record the frame where the search starts and the frame where it ends. Then add `(define W2 (make-withdraw 100))` and state which frame must be distinct for `W1` and `W2` to have independent histories. Do not collapse “the closure,” “its retained environment,” and “a call frame” into one box.

### SICP exercises

- Exercise 3.8: construct a procedure whose use exposes the evaluator's operand-order choice. Purpose: make hidden temporal coupling observable once procedure calls can mutate retained state. Invariants: the same procedure object is used within each tested expression, the initial state is reset between order trials, and the differing observation follows only from operand order rather than from a changed starting state. No construction or resulting values are supplied here.
- Exercise 3.11: draw the environment structures produced by the specified `make-account` interactions. Purpose: distinguish the global binding, each account's retained frame, and the fresh parameter frame created by each dispatch or operation call. Invariants: each procedure object points to its defining environment, assignment changes the nearest existing binding, two independently created accounts do not share their balance binding, and temporary call frames are not treated as persistent account state. The completed diagram is intentionally left open.

### Transfer prompt

For one coding-agent turn, classify every state item you believe exists as a turn snapshot, durable session history, cross-session memory, or external world state. For each item, name its owner, creation point, mutation rule, lifetime, persistence boundary, replay meaning, and authority boundary. Then choose one failure—process crash after a tool effect but before a receipt, compaction that drops context, stale memory recall, or an external file changed by another process—and explain which categories can and cannot reconstruct the truth. Do not design an agent-harness API yet.

## Dialogue

**PROVISIONAL.** `Which facts in an agent run must be retained as durable history, which should be selectively remembered across sessions, and which can only be re-observed from the external world?`

This question is queued; [the dialogue state](../dialogue_state.md) still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [Seminar 04 — Symbols, sets, compression, and generic dispatch](04-symbols-sets-compression-and-generic-dispatch.md) · [Course guide](../sicp_course_guide.md) · Next: [Seminar 06 — Mutation, simulation, propagation, and constraints](06-mutation-simulation-and-constraints.md)
