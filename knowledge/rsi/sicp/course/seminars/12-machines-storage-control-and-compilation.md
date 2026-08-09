# Seminar 12 — Machines, storage, control, and compilation

## Reading route

Chapter 5 is long because it lowers one coherent story through several levels. Read it in four sessions, keeping the machine state, storage representation, evaluator state, and compiler metadata distinct.

### Session 1 — Registers and stacks

- Required: [§5.1, printed pp. 668–695 / PDF pp. 696–723](../../../../../evidence/sicp/sicp.pdf#page=696). Follow the progression from data paths and controllers through labels, subroutines, `continue`, and the stack-based recursive factorial machine.
- Optional: none assigned.
- Skim: none assigned. The diagrams may be read quickly after you can reconstruct their controller text, but the complete §5.1 range is required.

### Session 2 — Simulator and assembler

- Required: [§5.2, printed pp. 696–722 / PDF pp. 724–750](../../../../../evidence/sicp/sicp.pdf#page=724). Track the simulator's `pc`, `flag`, registers, stack, label table, and instruction execution procedures as separate objects.
- Optional: none assigned.
- Skim: none assigned. Selector boilerplate may be traversed quickly only after you can say which representation boundary it protects; the complete §5.2 range is required.

### Session 3 — Storage and explicit evaluation

- Required: [§5.3, printed pp. 723–740 / PDF pp. 751–768](../../../../../evidence/sicp/sicp.pdf#page=751). Read pair representation, allocation, roots, relocation, and stop-and-copy collection as one lifetime mechanism.
- Required: [§5.4, printed pp. 741–766 / PDF pp. 769–794](../../../../../evidence/sicp/sicp.pdf#page=769). Follow `exp`, `env`, `val`, `continue`, `proc`, `argl`, and `unev` through evaluator dispatch and application.
- Optional: none assigned.
- Skim: none assigned. The complete §§5.3–5.4 range is required because the storage and control claims depend on their implementation details.

### Session 4 — Compilation

- Required: [§5.5, printed pp. 767–833 / PDF pp. 795–861](../../../../../evidence/sicp/sicp.pdf#page=795). Read targets, linkages, register-use metadata, procedure calls, lexical addressing, and evaluator interoperation as one compilation contract.
- Optional: none assigned.
- Skim: none assigned. Repetitive code-generator cases may be read quickly after you can derive their target, linkage, needs, and modifications; the complete §5.5 range is required.

## System-design problem

How can a language implementation turn implicit recursion, evaluation order, environment access, allocation, and return behavior into a finite mechanism whose state transitions and resource costs can be inspected? Chapter 5 answers by successively making control locations, saved values, storage roots, evaluator registers, and compiler obligations explicit. The resulting machine descriptions expose where a computation may continue, what must survive a subcomputation, which objects remain reachable, and which assumptions a generated instruction sequence makes about its input state.

## Vocabulary

- **Data path**: the registers and primitive operations that can hold or transform machine values.
- **Controller**: the ordered tests, branches, assignments, saves, restores, and transfers that select data-path operations.
- **Program counter (`pc`)**: a register pointing to the next simulated instruction.
- **Continuation (`continue`)**: a register holding the controller entry point to use after a subcomputation returns.
- **Stack**: last-in, first-out storage used to suspend values whose lifetimes cross nested calls.
- **Instruction execution procedure**: a zero-argument procedure attached by the assembler to an instruction datum; invoking it performs that simulated instruction and advances or redirects `pc`.
- **Root**: a machine-held pointer from which live storage can be reached.
- **Relocation**: copying a reachable object to new memory while forwarding old references to the copy.
- **Explicit-control evaluator**: a register machine that realizes evaluator dispatch, argument evaluation, procedure application, sequencing, and return behavior with named registers and controller labels.
- **Target**: the register in which compiled code must leave an expression's value.
- **Linkage**: the compiled code's required continuation: the next instruction, a named label, or return through `continue`.
- **Register-use metadata**: the registers an instruction sequence needs initialized and the registers it may modify.
- **Lexical address**: a compile-time pair of frame depth and displacement used to access a lexically known binding without a name search.

## Argument map

1. **CLAIM — Session 1.** A register-machine description must specify both data paths and the controller that sequences them. A recursive process additionally needs stack storage because suspended invocations must restore saved register values in reverse order; the factorial machine saves both `n` and `continue` around each recursive call. [§§5.1.1 and 5.1.4, printed pp. 672–693 / PDF pp. 700–721](../../../../../evidence/sicp/sicp.pdf#page=700). **INFERENCE.** A procedure's mathematical recurrence does not identify the concrete live state of its implementation; the controller and save/restore protocol do.
2. **CLAIM — Session 2.** The simulator represents a machine with registers, stack, operations, and an instruction sequence controlled by `pc`; its assembler first resolves labels, then attaches an execution procedure to each instruction. [§§5.2.1–5.2.3, printed pp. 698–717 / PDF pp. 726–745](../../../../../evidence/sicp/sicp.pdf#page=726). **INFERENCE.** Controller text is descriptive data until assembly resolves names against a particular machine and supplies executable behavior. Keeping the original instruction text aids debugging without making that text the behavior itself.
3. **CLAIM — Session 3.** SICP lowers pairs to typed pointers indexing parallel `car` and `cdr` vectors, then presents stop-and-copy collection: trace from machine roots, copy reachable pairs into the other semispace, update references, and exchange the spaces. [§§5.3.1–5.3.2, printed pp. 724–740 / PDF pp. 752–768](../../../../../evidence/sicp/sicp.pdf#page=752). **INFERENCE.** Representation and lifetime are coupled: a valid pointer discipline must still specify which references are roots and how relocation preserves identity and reachability.
4. **CLAIM — Session 3.** The explicit-control evaluator uses a stack and seven registers—`exp`, `env`, `val`, `continue`, `proc`, `argl`, and `unev`—and makes the metacircular evaluator's recursive calls into explicit `goto`, save, restore, dispatch, and return steps. [§5.4 and §5.4.1, printed pp. 741–751 / PDF pp. 769–779](../../../../../evidence/sicp/sicp.pdf#page=769). **INFERENCE.** `eval` and `apply` remain semantic organizing ideas, but their implementation is a state machine whose intermediate obligations can be inspected independently.
5. **CLAIM — Session 4.** Compiler code generators accept a target and linkage, while each generated instruction sequence carries the registers it needs, the registers it modifies, and its statements. `preserving` inserts saves and restores only when an earlier sequence modifies a register that a later sequence needs. [§5.5.1, printed pp. 772–778 / PDF pp. 800–806](../../../../../evidence/sicp/sicp.pdf#page=800). **INFERENCE.** Register metadata is a compositional contract: it lets the compiler decide preservation from local summaries instead of repeatedly re-analyzing assembled statements.
6. **CLAIM — Session 4.** Because lexical scope makes runtime environment shape parallel program nesting, a compiler can replace repeated name search with a lexical address consisting of frame depth and displacement, while leaving dynamic global lookup as a separate case. [§5.5.6, printed pp. 817–821 / PDF pp. 845–849](../../../../../evidence/sicp/sicp.pdf#page=845). **INFERENCE.** Compilation can turn a source-level invariant into a cheaper access path only within its stated scope conditions; internal definitions and mutable global bindings require additional treatment.

## Mechanism trace

### Part A — Recursive factorial as saved machine state

Trace the Figure 5.11 machine on `n = 3`. The controller and its save/restore order are shown in [§5.1.4, printed pp. 687–692 / PDF pp. 715–720](../../../../../evidence/sicp/sicp.pdf#page=715). The stack below is written bottom-to-top.

| Step | Control | `n` | `val` | `continue` | Stack | Why the state exists |
|---|---|---:|---:|---|---|---|
| 0 | entry | 3 | unspecified | `fact-done` | `[]` | The initial continuation names the machine's terminal entry point. |
| 1 | first `fact-loop` | 3 | unspecified | `fact-done` | `[fact-done, 3]` | The non-base case saves the caller's return point and the `n` needed after the subproblem. |
| 2 | descend | 2 | unspecified | `after-fact` | `[fact-done, 3]` | The same controller and registers are reused for `2!`; `after-fact` describes the suspended multiplication. |
| 3 | second `fact-loop` | 2 | unspecified | `after-fact` | `[fact-done, 3, after-fact, 2]` | This level saves its own caller state before reusing the machine for `1!`. |
| 4 | descend | 1 | unspecified | `after-fact` | `[fact-done, 3, after-fact, 2]` | The next loop reaches the base case. |
| 5 | `base-case` | 1 | 1 | `after-fact` | `[fact-done, 3, after-fact, 2]` | The base case places `1` in `val` and jumps through `continue`. |
| 6 | first `after-fact` | 2 | 2 | `after-fact` | `[fact-done, 3]` | Restoring `n` before `continue` reverses the save order; multiplying yields `2 × 1`. |
| 7 | second `after-fact` | 3 | 6 | `fact-done` | `[]` | The outer saved state is restored; multiplying yields `3 × 2`, and the restored continuation terminates the machine. |

The important invariant is not the numeral `6`: every suspended multiplication has exactly one saved caller continuation with its saved `n` immediately above it in the bottom-to-top stack notation, and every return consumes that most recently saved `n`-then-continuation pair. `val` is deliberately not saved because the subproblem's new result is precisely what the caller needs.

### Part B — A compiled application preserves its linkage

The compiler gives each expression a target and a linkage. For a compiled procedure call whose target is `val` and whose linkage is a named label such as `after-call`, the generated control has this shape:

```scheme
(assign continue (label after-call))
(assign val (op compiled-procedure-entry) (reg proc))
(goto (reg val))
after-call
;; use the callee's result from val
```

The compiled procedure ends with `(goto (reg continue))`, so assigning `after-call` before the jump preserves the caller's intended next control location. When a surrounding sequence still needs an older `continue`, the compiler's needs/modifies metadata lets `preserving` wrap the intervening sequence with the required save and restore. Conversely, a call with target `val` and linkage `return` leaves `continue` unchanged and jumps directly to the callee; this is how the compiler avoids an otherwise accumulating continuation save for tail calls. [§§5.5.1 and 5.5.3–5.5.4, printed pp. 774–798 / PDF pp. 802–826](../../../../../evidence/sicp/sicp.pdf#page=802)

This is a compilation trace, not a claim that every call uses the same three statements. Different target/linkage combinations generate different return labels and preservation work, and compiled procedure applications are conservatively declared to modify the compiler's `all-regs` set: `env`, `proc`, `val`, `argl`, and `continue`. That set does not include the explicit-control evaluator's `exp` or `unev` registers. [§5.5.3 declares `all-regs`, printed p. 796 / PDF p. 824](../../../../../evidence/sicp/sicp.pdf#page=824). The shared principle with the special-purpose factorial machine is narrower: a nested computation must retain exactly the control and data needed after it returns.

## Rust lens

```rust
// Explanatory sketch; not compiled by this course.
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Register {
    N,
    Val,
    Continue,
    Exp,
    Env,
    Proc,
    Argl,
    Unev,
}

#[derive(Clone, Debug)]
enum Value {
    Integer(i64),
    Boolean(bool),
    Label(usize),
    OpaqueDatum,
}

#[derive(Clone, Debug)]
enum Operand {
    Constant(Value),
    Register(Register),
}

#[derive(Clone, Debug)]
enum Source {
    Operand(Operand),
    Operation { name: &'static str, inputs: Vec<Operand> },
}

#[derive(Clone, Debug)]
enum Instruction {
    Assign { destination: Register, source: Source },
    Test(Source),
    Branch { target: usize },
    GotoLabel(usize),
    GotoRegister(Register),
    Save(Register),
    Restore(Register),
    Perform(Source),
    Halt,
}

#[derive(Debug)]
struct MachineState {
    pc: usize,
    flag: bool,
    registers: BTreeMap<Register, Value>,
    stack: Vec<Value>,
    halted: bool,
}

fn factorial_controller_fragment() -> Vec<Instruction> {
    vec![
        Instruction::Test(Source::Operation {
            name: "=",
            inputs: vec![
                Operand::Register(Register::N),
                Operand::Constant(Value::Integer(1)),
            ],
        }),
        Instruction::Branch { target: 7 },
        Instruction::Assign {
            destination: Register::N,
            source: Source::Operation {
                name: "-",
                inputs: vec![
                    Operand::Register(Register::N),
                    Operand::Constant(Value::Integer(1)),
                ],
            },
        },
    ]
}

fn step(_program: &[Instruction], _state: &mut MachineState) -> Result<(), &'static str> {
    todo!("resolve operations, validate operands, execute one instruction, and update pc")
}
```

The enums make the distinction between machine state and controller instructions visible, while `Vec<Value>` naturally exposes last-in, first-out stack order. `pc` and `flag` are dedicated control fields rather than duplicate entries in the named-register map; `Operand` lets an operation consume constants as well as registers, as the factorial comparison and subtraction require. The sketch does not implement SICP's assembler, lexical environments, tagged pair memory, relocation, garbage collection, primitive procedures, error system, or compiler metadata. `OpaqueDatum` intentionally hides representation details that Chapter 5 spends substantial effort exposing. Rust ownership also does not choose roots, continuation discipline, tail-call behavior, or authorization policy; those remain semantic and architectural decisions. The sketch is non-runnable and is not a proposed agent-harness API.

## Agent-harness bridge

**SICP CLAIM.** A nontrivial evaluator becomes inspectable when implicit recursive control is lowered into named registers, controller locations, stack obligations, and explicit terminal or recovery transfers; compiled instruction sequences similarly state what control and register state they require and modify. [§§5.4–5.5.1, printed pp. 741–778 / PDF pp. 769–806](../../../../../evidence/sicp/sicp.pdf#page=769)

**HARNESS EVIDENCE — Pi.** At pinned Pi commit `4488ad55c18f07ae89a489096c90de8667b3adfb`, `runLoop` has nested loops for tool-call continuations, steering, and follow-up input. It emits turn boundaries, awaits a streamed assistant result, and returns immediately on an error or aborted stop reason. Otherwise it extracts and executes complete tool calls, appends results, optionally replaces the next-turn context, and may return at `shouldStopAfterTurn`; only the ordinary path that exhausts tool and steering work checks for queued follow-up input before the outer loop exits. [`packages/agent/src/agent-loop.ts:152–371`](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) This is evidence for one concrete event-driven control loop, not a declaration of the conceptual state enum below.

**HARNESS EVIDENCE — Hermes.** At pinned Hermes Agent commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`, `run_conversation` builds a per-turn context, resets persistence and compaction flags, initializes response, failure, retry, compression, and budget counters, then enters an iteration-budget loop. The cited loop applies pending redirects, calls `_checkpoint_mgr.new_turn()` to reset per-iteration checkpoint deduplication, and handles interrupt and budget-exhaustion exits before the next API step. [`agent/conversation_loop.py:1084–1325`](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L1084-L1325) This range evidences explicit per-turn state and early control transitions; it does not, by itself, establish the later tool, persistence, checkpoint-capture, or recovery semantics of the whole function.

**HARNESS EVIDENCE — Codex.** At pinned OpenAI Codex commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, `run_turn` performs pre-sampling compaction, resolves required servers, captures a step context, records the model-visible world state, loops over pending input and sampling, triggers a mid-turn compaction transition at the context limit, follows model or queued-input continuations, and exits through distinct aborted, error, stop-hook, or ordinary completion branches. [`codex-rs/core/src/session/turn.rs:149–520`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L149-L520) These are concrete Codex transitions, not proof that every state is durably persisted or that the inferred labels below match internal type names.

**HARNESS EVIDENCE — authority outside model preference.** In the same pinned Codex checkout, sandbox kinds and platform selection are runtime types, and initial sandbox choice depends on a permission profile, a sandboxability preference, managed-network requirements, and platform availability. The transform then constructs the selected sandbox execution request. [`codex-rs/sandboxing/src/manager.rs:34–74`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/sandboxing/src/manager.rs#L34-L74) [`codex-rs/sandboxing/src/manager.rs:264–380`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/sandboxing/src/manager.rs#L264-L380) The cited code shows runtime-owned selection inputs rather than authority inferred from model prose; it does not establish who populated every permission profile or whether every effect is sandboxable.

**INFERENCE.** Read together, these mechanisms motivate a conceptual harness state model:

```text
ready -> sampling -> dispatching -> awaiting effect -> sampling
sampling|dispatching -> compacting -> sampling
any nonterminal -> recovering|cancelled|failed
sampling -> completed when no action or queued input remains
```

`Ready` means the turn has an admitted input and runtime-owned context, not that arbitrary effects are authorized. `Sampling` owns a model request. `Dispatching` interprets a structured action and resolves the runtime path. `Awaiting effect` distinguishes an admitted invocation from its receipt. `Compacting` replaces the model-visible projection without erasing canonical evidence. `Recovering` is distinct from retry because an effect may already have happened. `Completed`, `cancelled`, and `failed` are different terminal claims. The nine labels—ready, sampling, dispatching, awaiting effect, compacting, recovering, completed, cancelled, and failed—are a course synthesis across different implementations, not names used uniformly by Pi, Hermes, or Codex.

**AGENT-HARNESS QUESTION — INFERENCE.** Which registers, stack-like suspended obligations, durable transitions, authority receipts, and recovery branches would a future agent harness need so every move among those conceptual states is testable without granting authority through model preference? This state list is an input to the capstone research dossier, not a Rust API or an implementation decision for an agent harness.

## Limits of the analogy

A SICP register machine is a deterministic abstract machine with a fixed controller and primitive-operation table. An agent harness coordinates probabilistic model calls, asynchronous tools, mutable external state, interrupts, permissions, persistence, and partial failure. A model message is not an instruction sequence; a context window is not a register file; transcript history is not a stack; compaction is not garbage collection; and a tool receipt is not a procedure return value.

The compiler's needs/modifies sets summarize register behavior under a known instruction semantics. A tool specification cannot safely promise all filesystem, network, credential, or human effects in the same closed way. Likewise, a reachable heap object is live by language semantics, whereas an old session record may remain audit-critical even when omitted from model context. The useful transfer is the discipline of naming state, continuation, lifetime, and authority boundaries. It does not justify treating the LLM as the controller or allowing a model-selected action to choose its own sandbox, preflight result, recovery policy, or terminal receipt.

## Practice

### Manual trace

Trace the Figure 5.11 recursive factorial machine for `n = 4`. At every controller label, record `n`, `val`, `continue`, and the stack from bottom to top; mark every save, restore, and indirect `goto`. Then sketch the compiled call boundary for the recursive application using target `val` and a named linkage back to the pending multiplication. State which prior `continue` value must survive, where needs/modifies metadata would force preservation, and why a tail-position call with linkage `return` has a different stack obligation. Do not use the `n = 3` table above as an answer key: produce the additional level and justify every retained value.

### SICP exercises

- Exercise 5.14: measure pushes and maximum stack depth for the special-purpose recursive factorial machine as requested. Purpose: connect the qualitative stack trace to explicit time- and space-related measurements. Invariants: initialize monitoring for each input, count only actual `save` operations, distinguish total pushes from maximum simultaneous depth, and derive any claimed function from multiple nontrivial inputs. No measurements or formulas are supplied here. [Exercise 5.14, printed p. 720 / PDF p. 748](../../../../../evidence/sicp/sicp.pdf#page=748)
- Exercise 5.41: implement `find-variable` over a compile-time environment. Purpose: turn lexical scope into the frame-depth/displacement lookup used by lexical-address code generation. Invariants: search frames from innermost outward, use zero-based frame and displacement positions as the exercise specifies, return `not-found` rather than inventing a global address, and leave runtime values out of the compile-time environment. No implementation or sample results are supplied here. [Exercise 5.41, printed pp. 820–821 / PDF pp. 848–849](../../../../../evidence/sicp/sicp.pdf#page=848)

### Transfer prompt

Choose one coding-agent turn that includes a model request, one tool action, a context-limit compaction, and cancellation during or just after the effect. Draw the conceptual states ready, sampling, dispatching, awaiting effect, compacting, recovering, completed, cancelled, and failed. For every transition, name the owner, input evidence, output receipt, resumable continuation, persistent record, budget effect, and authority decision. Identify which data belongs only in the model projection and which must remain durable even when no longer reachable from that projection. Finally, explain why model preference can select a requested action but cannot mint the permission, sandbox, or receipt that authorizes and verifies it.

## Dialogue

**PROVISIONAL.** `Which harness obligations behave like explicit continuations that must survive a model or tool subcomputation, and which are durable authority or audit records that must not be treated as stack-local state?`

This question is queued; [the dialogue state](../dialogue_state.md) still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [Seminar 11 — Logic programming and declarative query](11-logic-programming-and-declarative-query.md) · [Course guide](../sicp_course_guide.md) · Next: [Agent-harness architecture dossier](../capstone/agent_harness_architecture_dossier.md)
