# Seminar 08 — Streams, delay, and infinite processes

## Reading route

Read §3.5 as a study of when a sequence element comes into existence and what keeps it alive. The sequence interface stays recognizable, but a stream delays its tail; infinite definitions then become usable because a consumer asks for only a finite prefix. Later subsections use that demand boundary to reorganize numerical iteration, feedback, and time-varying state.

- Optional orientation: [the chapter transition into streams, printed pp. 428–430 / PDF pp. 456–458](../../../../../evidence/sicp/sicp.pdf#page=456). Read it for the contrast between assignment-based objects and sequence-based descriptions of time.
- Required: [§3.5.1, printed pp. 430–441 / PDF pp. 458–469](../../../../../evidence/sicp/sicp.pdf#page=458) for stream interfaces, delayed tails, forcing, and memoization.
- Required: [§3.5.2, printed pp. 441–453 / PDF pp. 469–481](../../../../../evidence/sicp/sicp.pdf#page=469) for recursively defined infinite streams and finite demand.
- Required: [§3.5.3, printed pp. 453–470 / PDF pp. 481–498](../../../../../evidence/sicp/sicp.pdf#page=481) for stream formulations of numerical iteration, acceleration, and enumeration.
- Required: [§3.5.4, printed pp. 470–479 / PDF pp. 498–507](../../../../../evidence/sicp/sicp.pdf#page=498) for explicit delayed arguments in feedback loops and stream-based differential-equation solvers.
- Required: [§3.5.5, printed pp. 479–486 / PDF pp. 507–514](../../../../../evidence/sicp/sicp.pdf#page=507) for streams as a functional model of signals, random experiments, and time-varying state.
- Optional: the exercises distributed through [§§3.5.1–3.5.5, printed pp. 437–486 / PDF pp. 465–514](../../../../../evidence/sicp/sicp.pdf#page=465), especially those that compare two definitions under different forcing patterns. Use them to test time and space behavior rather than merely obtaining the next value.
- Skim on a first pass: the line-by-line convergence-acceleration implementation in [§3.5.3, printed pp. 455–458 / PDF pp. 483–486](../../../../../evidence/sicp/sicp.pdf#page=483), the weighted-pair enumeration and subsequent signal-processing material in [§3.5.3, printed pp. 463–470 / PDF pp. 491–498](../../../../../evidence/sicp/sicp.pdf#page=491), and the extended random-number and Monte Carlo examples in [§3.5.5, printed pp. 482–486 / PDF pp. 510–514](../../../../../evidence/sicp/sicp.pdf#page=510), after you can state what each pipeline consumes and produces. Do not skim the `delay`/`force` boundary, the memoization rule, or the delayed-integrand feedback in §3.5.4.

## System-design problem

An eager sequence couples construction cost and storage to the size of the whole result. That is unsuitable when the result is very large, conceptually infinite, or produced incrementally. Yet delaying work introduces new questions: which demand forces which computation, whether repeated demand repeats effects, what values remain reachable, and whether an old prefix can be replaced without changing meaning. How can a system separate production, consumption, retention, and replacement instead of hiding them behind one vague word such as “streaming”?

## Vocabulary

- **Stream**: SICP's sequence abstraction whose first element is available now and whose rest is represented by delayed computation.
- **Head**: the current element returned by `stream-car` without forcing the tail.
- **Delayed tail**: the expression stored by `cons-stream` for later computation rather than evaluated while the current cell is constructed.
- **Promise**: the delayed object returned by `delay` and consumed by `force`.
- **Force**: demand that runs a promise's deferred computation, unless a memoized result is already cached.
- **Memoization**: retaining the first result of a delayed computation so later forces return that result without recomputing it.
- **Prefix**: a finite initial segment demanded from a potentially unbounded stream.
- **Feedback stream**: a stream whose later elements depend on earlier elements of the same stream or a mutually defined stream; delayed construction breaks the immediate-definition cycle.
- **Retention policy**: the rule determining which produced elements, source records, cached results, and summaries remain reachable.
- **Replacement policy**: an explicit rule that substitutes a new representation, such as a summary plus a retained tail, for some prior material in a future consumer's view.

## Argument map

1. **CLAIM.** Streams retain the ordinary sequence interface for constructing a sequence, selecting its head and tail, and mapping or filtering its elements, while changing when the tail is computed. `cons-stream` evaluates its first argument immediately and delays its second; `stream-cdr` forces that delayed expression. [§3.5.1 introduces stream operations and the `cons-stream`/`stream-cdr` representation (printed pp. 430–441 / PDF pp. 458–469)](../../../../../evidence/sicp/sicp.pdf#page=458). **INFERENCE.** Clients can often preserve sequence-shaped composition while avoiding eager construction of elements they never demand, but consumers that traverse or aggregate a stream must still bound demand when termination matters. The timing of work becomes part of the abstraction's observable cost behavior.
2. **CLAIM.** A recursively defined infinite stream can be a finite program with no terminal case because construction produces one element and delays the recursive construction of the rest. Consumers such as `stream-ref` or a bounded display procedure force only the prefix they traverse. [§3.5.2 constructs infinite integer, Fibonacci, prime, and self-referential streams (printed pp. 441–453 / PDF pp. 469–481)](../../../../../evidence/sicp/sicp.pdf#page=469). **INFERENCE.** “Infinite” describes the continuation rule, not an infinite object allocated at definition time; every actual observation still has a finite demand history.
3. **CLAIM.** Stream pipelines can recast iterative numerical and enumeration processes as transformations over successive approximations or candidate pairs. A consumer controls how far the pipeline runs, and accelerators can themselves be expressed as transformations from one approximation stream to another. [§3.5.3 develops square-root approximations, π acceleration, pair enumeration, and weighted streams (printed pp. 453–470 / PDF pp. 481–498)](../../../../../evidence/sicp/sicp.pdf#page=481). **INFERENCE.** This organization exposes intermediate states as values and makes stages composable, but it does not eliminate computation: demanding farther output still triggers all dependencies needed for that output.
4. **CLAIM.** Delayed evaluation changes the time at which a dependency is required, and memoized delay makes repeated forcing reuse the first computed result. Explicitly delaying the integrand lets `integral` participate in feedback before the integrand stream is available; SICP's stream implementation uses a memoizing delayed procedure so a forced tail is not recomputed. [§3.5.4 uses a delayed integrand to define feedback-based solvers (printed pp. 470–479 / PDF pp. 498–507)](../../../../../evidence/sicp/sicp.pdf#page=498) [§3.5.1 defines memoized delayed procedures and explains why repeated forcing reuses a result (printed pp. 437–440 / PDF pp. 465–468)](../../../../../evidence/sicp/sicp.pdf#page=465). **INFERENCE.** Memoization is both a time policy and a retention policy: it prevents repeated computation while the promise is reachable, and the cached result may keep an already produced suffix reachable. Garbage collection may reclaim an unreachable prefix; streams do not promise permanent history.
5. **CLAIM.** Streams offer a functional alternative to representing a system with assignment to time-varying state: a changing quantity can instead be represented as a sequence of its successive values, and consumers combine such sequences with ordinary functional operations. [§3.5.5 contrasts assignment-based and stream-based accounts and develops signal, random-number, and Monte Carlo streams (printed pp. 479–486 / PDF pp. 507–514)](../../../../../evidence/sicp/sicp.pdf#page=507). **INFERENCE.** Making successive states explicit can improve modular composition and replay of pure computations. It does not erase physical time, input effects, memory costs, or the need to decide which historical values a real system retains.

## Mechanism trace

Trace the first five elements of `integers-starting-from(1)`, using SICP's rule that the tail expression is delayed and memoized. `Cₙ` is a realized stream cell whose head is `n`; `Pₙ` is the promise for the tail beginning at `n + 1`.

```text
construct S = integers-starting-from(1)
  allocate C1: head = 1
  create P1: delayed integers-starting-from(2)
  P1 state = pending

demand element 1
  read C1.head -> 1
  no promise is forced
  retained from root S: C1 -> P1(pending)

demand element 2
  force P1
    run delayed integers-starting-from(2)
    allocate C2: head = 2
    create P2: delayed integers-starting-from(3)
    cache C2 in P1; P1 state = forced(C2)
  read C2.head -> 2
  retained from S: C1 -> P1(cache C2) -> P2(pending)

demand element 3
  traverse P1: cache hit -> C2
  force P2
    allocate C3 with head = 3
    create P3 for integers-starting-from(4)
    cache C3 in P2
  read C3.head -> 3

demand element 4
  traverse P1 and P2: cache hits
  force P3
    allocate C4 with head = 4
    create P4 for integers-starting-from(5)
    cache C4 in P3
  read C4.head -> 4

demand element 5
  traverse P1, P2, and P3: cache hits
  force P4
    allocate C5 with head = 5
    create P5 for integers-starting-from(6)
    cache C5 in P4
  read C5.head -> 5

after the five-element demand
  produced values: [1, 2, 3, 4, 5]
  forced promises: P1, P2, P3, P4
  unforced promise: P5
  retained from S: C1, cached C2, cached C3, cached C4, cached C5, and P5

repeat the same five-element traversal from S
  P1 through P4 are cache hits; no cell or tail promise is recreated
```

Promise creation and forcing are different events: producing `C5` creates `P5`, but no request for the sixth element has forced it. Retention is likewise conditional. Keeping `S` keeps the memoized chain above reachable. If a consumer drops `S` and retains only a later suffix, an implementation with garbage collection may reclaim the unreachable prefix; memoization does not mean an eternal archive.

## Rust lens

Rust's ordinary `Iterator` trait also separates producing a next value from consuming the entire conceptual sequence, but it normally advances mutable iterator state rather than preserving memoized stream cells:

```rust
// Explanatory sketch; not compiled by this course.
struct IntegersFrom {
    next: Option<u64>,
}

impl Iterator for IntegersFrom {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.next?;
        self.next = value.checked_add(1);
        Some(value)
    }
}

fn first_five() -> Vec<u64> {
    IntegersFrom { next: Some(1) }.take(5).collect()
}
```

The iterator can describe a demand-driven progression and `take(5)` bounds demand, but an ordinary Rust iterator does **not** automatically memoize produced values. Calling `next` consumes the current cursor position. Here the `Vec`—not the iterator—retains `[1, 2, 3, 4, 5]` for replay. A separate cache or persistent lazy-cell type would be needed to share SICP-like memoized tails. Rust ownership helps state whether the cursor, collected prefix, or cache remains reachable; it does not choose a retention policy on the application's behalf. The checked increment makes this machine-integer iterator yield `u64::MAX` and then end, unlike SICP's mathematical integer example.

## Agent-harness bridge

**SICP CLAIM.** A stream separates the rule that can produce more elements from the consumer that forces a finite prefix, while memoization separately determines whether a forced tail is reused. [§§3.5.1–3.5.2, printed pp. 430–453 / PDF pp. 458–481](../../../../../evidence/sicp/sicp.pdf#page=458)

**HARNESS EVIDENCE — production.** At pinned Pi commit `4488ad55c18f07ae89a489096c90de8667b3adfb`, `streamAssistantResponse` requests a model response and, as the asynchronous response yields start, delta, completion, or error events, updates a partial message and emits corresponding message lifecycle events. [`packages/agent/src/agent-loop.ts:152–371`](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) This is an incremental event-production path; it does not establish that the response is a SICP promise or that emitted deltas are memoized.

**HARNESS EVIDENCE — consumption.** In the same pinned loop, `runLoop` awaits the completed assistant message, extracts tool calls, consumes their results, appends those results to current context, and uses stop, steering, and follow-up conditions to decide whether to request another turn. [`packages/agent/src/agent-loop.ts:152–371`](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/agent-loop.ts#L152-L371) Production and consumption therefore have different control points even though this implementation coordinates them in one loop.

**HARNESS EVIDENCE — retention.** Given one selected path, Pi's session projection applies the latest compaction entry and turns the resulting entries into model-visible messages. A compaction entry projects to a summary message plus any explicitly retained tail. [`packages/agent/src/harness/session/session.ts:61–149`](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L61-L149) Session appends are serialized through `appendTail`, receive parent identifiers, reach the store before the in-memory leaf advances, and include an explicit `appendCompaction` operation. [`packages/agent/src/harness/session/session.ts:333–515`](../../../../../evidence/implementations/pi/snapshot/packages/agent/src/harness/session/session.ts#L333-L515) This is evidence for stored entry lineage and context-projection choices, not evidence that every produced delta is retained forever or that storage alone makes a projection lossless.

**HARNESS EVIDENCE — replacement.** At pinned Codex commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, the turn loop evaluates post-sampling token state and pending continuation, records budget state, and invokes `run_auto_compact` as an explicit mid-turn rollover before continuing. [`codex-rs/core/src/session/turn.rs:418–457`](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/session/turn.rs#L418-L457) This shows that compaction is a controlled state transition under a context limit; the cited range does not establish that a generated summary preserves every fact in the material it replaces.

**INFERENCE.** An agent harness needs four independent policies:

1. **Production policy:** which model, tool, lifecycle, and control events may be emitted, and what backpressure or cancellation applies while they are produced.
2. **Consumption policy:** which component observes each event, when partial output becomes actionable, and what demand causes another model or tool step.
3. **Retention policy:** which raw events, canonical messages, tool receipts, branch edges, and cached projections remain durably addressable.
4. **Replacement policy:** when a future model-visible projection may substitute a summary and retained tail for older detail, how that substitution is identified, and which original evidence remains outside the compacted view.

Compaction is not ordinary stream-tail forcing. Forcing reveals the value specified by a delayed tail, and memoization reuses that value. Compaction computes a new, bounded representation of already produced material. A summary or replacement is not presumptively lossless, so it must not silently become the only receipt for an external effect, authorization decision, or failure.

**AGENT-HARNESS QUESTION — INFERENCE.** What separate schemas and state transitions should an agent harness use for event production, live consumption, durable retention, and model-context replacement so a compacted view stays attributable to its source range while authoritative tool receipts and lineage remain independently recoverable?

## Limits of the analogy

SICP streams are language-level sequence objects. Model token streams, UI event streams, append-only session records, and model-visible context projections are different objects with different consumers and failure modes. An asynchronous event iterator may deliver values incrementally without memoizing or retaining them. A stored session entry may be durable without being included in the next model context. A compacted summary is newly computed, potentially lossy content; it is not the same value as an unforced tail and is not made correct merely by caching it. Finally, demand-driven computation does not grant tool authority, undo external effects, provide backpressure, or prove replay safety.

## Practice

### Manual trace

Define a stream constructor `multiples-from(k, step)` whose head is `k` and whose delayed tail calls `multiples-from(k + step, step)`. Starting with `multiples-from(10, 3)`, trace a consumer that requests the first three elements, requests the third element again from the original root, then requests the fourth and fifth elements. Record every realized cell, tail-promise creation, first force, cache hit, and pending promise after each request. Draw the reachable chain while the original root is retained, then state which prefix could become unreachable if only the fifth cell were retained. Do not assume that requesting an already memoized element reruns arithmetic.

### SICP exercises

- Exercise 3.53: analyze the self-referential stream in the exercise. Purpose: test whether you can reason from a stream equation and delayed demand rather than expanding an infinite object eagerly. Invariants: distinguish the already available head from each forced tail, preserve elementwise `add-streams` behavior, and mark where memoized results are reused. The resulting sequence is intentionally not supplied here.
- Exercise 3.77: repair the integral/solver organization requested by the exercise. Purpose: test whether the integrand can participate in a feedback definition before its stream value is available. Invariants: delay exactly the dependency that would otherwise be demanded too early, force it only when the integral's tail is demanded, and preserve the initial value and `dt` recurrence. No implementation is supplied here.

### Transfer prompt

For one coding-agent turn, draw four ledgers: produced events, consumers and acknowledgements, durably retained canonical records, and model-context replacements. Include partial assistant output, a tool request, authority decision, tool receipt, follow-up message, and a compaction summary. For every transition, state whether it is demand, delivery, persistence, or replacement. Then specify what may be discarded, what may leave the next model context but remain auditable, and what must never be represented only by a lossy summary.

## Dialogue

**PROVISIONAL.** `When a harness shortens model context, which information may be summarized for future reasoning, and which raw events, authority decisions, lineage edges, or effect receipts must remain separately retrievable?`

This question is queued; [the dialogue state](../dialogue_state.md) still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [Seminar 07 — Concurrency, serialization, and interleavings](07-concurrency-serialization-and-interleavings.md) · [Course guide](../sicp_course_guide.md) · Next: [Seminar 09 — Eval/apply and executable semantics](09-eval-apply-and-executable-semantics.md)
