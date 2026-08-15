# Seminar 02 — Data abstraction and immutable representation

## Reading route

Read §2.1 as a study of contracts between clients and representations, not merely as a collection of pair-manipulation techniques. Follow the rational-number example through its successive representation repairs; then use procedural pairs and interval arithmetic to test what an abstraction barrier does—and does not—guarantee.

- Required: [[evidence/sicp/sicp.pdf#page=141|§2.1.1–§2.1.4, printed pp. 113–131 / PDF pp. 141–159]].
- Optional: revisit the rational-number sign-normalization discussion and Exercises 2.1–2.3 in [[evidence/sicp/sicp.pdf#page=143|§2.1.1, printed pp. 115–118 / PDF pp. 143–146]] if representation invariants still feel like client responsibilities.
- Skim: the chapter transition that frames compound data in [[evidence/sicp/sicp.pdf#page=140|§2.1 opening, printed p. 112 / PDF p. 140]]; return to it after the mechanism trace and ask which complexity the data interface has hidden.

## System-design problem

Clients need to calculate with a concept such as a rational number without depending on how it is stored. The representation still has to establish invariants, support its advertised operations, and preserve enough information for the client's purpose. How can a system isolate those responsibilities so that changing an encoding does not force every client to change, while still making losses and ambiguities at the boundary visible?

## Vocabulary

- **Constructor**: an operation that creates an abstract-data value from its constituent parts while establishing the representation invariant.
- **Selector**: an operation that observes a constituent through the abstraction's public contract rather than through its storage layout.
- **Representation invariant**: a property that every valid representation must satisfy, such as a nonzero positive denominator and lowest terms.
- **Abstraction barrier**: a boundary that assigns knowledge of a representation to one layer and lets other layers depend on named operations instead.
- **Behavioral characterization**: a specification of data by the equations its operations must satisfy, rather than by one required physical layout.
- **Dependency problem (course term)**: the inferred loss of correlation when interval operations allow two occurrences of the same uncertain quantity to range independently.

## Argument map

1. **CLAIM.** Constructors and selectors let client arithmetic depend on the rational-number contract rather than on the pair used to represent it. [[evidence/sicp/sicp.pdf#page=141|§2.1.1 introduces `make-rat`, `numer`, and `denom` and defines arithmetic in terms of them (printed pp. 113–118 / PDF pp. 141–146)]]. **EVIDENCE.** `add-rat` obtains components through selectors and returns its result through the constructor; it does not call `cons`, `car`, or `cdr` directly. **INFERENCE.** That call structure gives the representation owner one place to normalize signs and common factors.
2. **CLAIM.** An abstraction barrier organizes a system into levels, with each level using the operations supplied below it and supplying operations to the level above it. [[evidence/sicp/sicp.pdf#page=146|§2.1.2 names the barriers in the rational-number package and explains how they localize alternative implementations (printed pp. 118–121 / PDF pp. 146–149)]]. **INFERENCE.** Locality is conditional: a representation change stays behind the barrier only when clients have not bypassed the interface or relied on an unstated property such as a particular pair layout.
3. **CLAIM.** A pair can be characterized by the behavior of `cons`, `car`, and `cdr`, not only by a built-in storage representation. [[evidence/sicp/sicp.pdf#page=150|§2.1.3 presents a procedural pair and requires `car(cons x y) = x` and `cdr(cons x y) = y` (printed pp. 122–125 / PDF pp. 150–153)]]. **INFERENCE.** Two implementations are interchangeable for clients restricted to those operations; the claim does not make them indistinguishable to every possible Scheme operation, especially representation inspection or mutation outside that interface.
4. **CLAIM.** Interval arithmetic represents a quantity by lower and upper bounds and defines arithmetic in terms of those selectors, allowing alternative endpoint and center-width views to share an interface. [[evidence/sicp/sicp.pdf#page=154|§2.1.4 develops interval constructors, selectors, and center-width and percentage representations (printed pp. 126–131 / PDF pp. 154–159)]]. **EVIDENCE.** Client formulas can be written without choosing whether an interval was originally supplied as endpoints, a center plus width, or a center plus percentage tolerance.
5. **CLAIM.** Algebraically equivalent interval formulas can produce different result intervals. [[evidence/sicp/sicp.pdf#page=156|§2.1.4 demonstrates the discrepancy with two parallel-resistance formulas (printed pp. 128–131 / PDF pp. 156–159)]]. **EVIDENCE.** For a positive interval `A = [2, 4]`, ordinary interval division computes `A/A = [2, 4] × [1/4, 1/2] = [1/2, 2]`, even though choosing one concrete nonzero value `a` and evaluating `a/a` always gives `1`. **INFERENCE.** The interval operations allow repeated occurrences of one uncertain quantity to vary independently and therefore lose their correlation. A clean representation boundary cannot recover correlations that the representation never retained; abstraction controls coupling, not information loss.

## Mechanism trace

Trace the addition `1/6 + 1/4` through a rational-number interface whose invariant is:

```text
denominator > 0
gcd(|numerator|, denominator) = 1
the represented value is numerator / denominator
```

The client calls `add-rat(x, y)`. It knows only the selectors and constructor, and the operation returns a newly constructed rational rather than mutating either input:

```text
x = make-rat(1, 6)  -> representation satisfying 1/6
y = make-rat(1, 4)  -> representation satisfying 1/4

add-rat asks:
  numer(x) = 1       denom(x) = 6
  numer(y) = 1       denom(y) = 4

raw numerator   = 1 × 4 + 1 × 6 = 10
raw denominator = 6 × 4             = 24

add-rat calls make-rat(10, 24)
constructor computes gcd(10, 24) = 2
constructor stores an internal representation of 5/12

client observes:
  numer(result) = 5
  denom(result) = 12
```

Normalization belongs to `make-rat`, so every route into the abstraction establishes the invariant once. Selectors may therefore return already-normalized components. If the representation owner later stores a sign bit plus two nonnegative magnitudes, the client algorithm above need not change: only `make-rat`, `numer`, and `denom` must continue to satisfy their equations.

The barrier does not make the representation irrelevant. It makes responsibility explicit. The constructor must reject a zero denominator, handle the denominator's sign, and preserve the mathematical value; selectors must faithfully project that value; clients must not inspect the hidden layout. If any layer violates its part, the trace no longer justifies `5/12`.

## Rust lens

Rust privacy can enforce one boundary that the Scheme package enforces by convention: code outside the defining module cannot construct or read private fields directly. The sketch uses a wider intermediate type so negating `i64::MIN` does not overflow before normalization.

```rust
// Explanatory sketch; not compiled by this course.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    numer: i64,
    denom: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RationalError {
    ZeroDenominator,
    OutOfRange,
}

impl Rational {
    pub fn new(numer: i64, denom: i64) -> Result<Self, RationalError> {
        if denom == 0 {
            return Err(RationalError::ZeroDenominator);
        }

        let mut n = i128::from(numer);
        let mut d = i128::from(denom);
        if d < 0 {
            n = -n;
            d = -d;
        }
        let divisor = gcd(n.unsigned_abs(), d as u128) as i128;
        n /= divisor;
        d /= divisor;

        Ok(Self {
            numer: i64::try_from(n).map_err(|_| RationalError::OutOfRange)?,
            denom: i64::try_from(d).map_err(|_| RationalError::OutOfRange)?,
        })
    }

    pub fn numer(self) -> i64 { self.numer }
    pub fn denom(self) -> i64 { self.denom }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
```

Private fields prevent ordinary external Rust code from manufacturing `Rational { numer: 2, denom: 4 }`, but privacy is not the whole abstraction. `new` still has to establish the invariant, arithmetic methods still need an overflow policy, and serialization or unsafe code can create additional representation boundaries. Rust's nominal type and error result sharpen the ownership of those choices; they are not a translation of Scheme pairs into identical semantics.

## Agent-harness bridge

**SICP CLAIM.** A client should depend on the constructors, selectors, and behavior promised by an abstraction rather than on its concrete representation. [[evidence/sicp/sicp.pdf#page=141|§2.1.1–§2.1.2, printed pp. 113–121 / PDF pp. 141–149]]

**HARNESS EVIDENCE.** In OpenAI Codex at pinned commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, `ToolRouter` holds a `ToolRegistry` separately from `model_visible_specs`; one method clones the visible specifications, another resolves a runtime through the registry, and dispatch ultimately asks the registry to invoke the constructed `ToolInvocation`. [[evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs|`codex-rs/core/src/tools/router.rs:31–289`]] ([exact lines 31–289](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289)) In Hermes Agent at pinned commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`, memory-provider schemas are normalized before their names enter a provider-routing table; names that shadow reserved core tools and duplicate names are rejected rather than allowed to take over dispatch. [[evidence/implementations/hermes/snapshot/agent/memory_manager.py|`agent/memory_manager.py:364–455`]] ([exact lines 364–455](../../../../../evidence/implementations/hermes/snapshot/agent/memory_manager.py#L364-L455))

**INFERENCE.** These implementations support a capability-boundary rule: an advertised tool name or model-visible specification describes how a request may be formed, but it does not itself grant execution authority. Codex still requires a registry-held runtime for the resolved `ToolName`; Hermes registration policy prevents an accepted-looking schema from silently acquiring an existing name's dispatch position. The abstraction lesson transfers as separation of responsibilities, not as identity between a data selector and a tool invocation.

**AGENT-HARNESS RESEARCH QUESTION — INFERENCE.** What evidence should a future agent harness retain separately for (1) the `ToolSpec` shown to a model, (2) the stable `CapabilityId` selected from a request, and (3) the runtime-registry entry and policy decision that actually authorize invocation, so that visibility, identity, and authority cannot be confused?

## Limits of the analogy

SICP's rational and pair interfaces specify mathematical or behavioral data operations; they do not model hostile providers, revocation, sandbox policy, asynchronous failure, or external side effects. A harness registry is therefore not literally a constructor-selector package, and a tool is not merely immutable data. The useful transfer is narrower: clients should not gain hidden representation knowledge, and possession of a public description or name must remain distinct from possession of an executable capability. The cited router shows runtime resolution and dispatch, not the whole authorization stack; policy and operating-system authority must be verified at their own boundaries.

## Practice

### Manual trace

Trace `add-rat(make-rat(-2, -6), make-rat(3, -4))` under the invariant that denominators are positive and fractions are in lowest terms. Record each constructor's sign normalization and greatest-common-divisor reduction, every value observed through `numer` and `denom`, the raw numerator and denominator passed by `add-rat`, and the final normalized result. Then name exactly which steps could change if the internal representation changed and which observations the public contract must preserve.

### SICP exercises

- Exercise 2.4: test the behavioral definition of a pair using a procedural representation. Purpose: explain why the two selector equations are sufficient for clients confined to the pair interface. Invariant: for arbitrary `x` and `y`, selecting the first or second component of the constructed value yields the corresponding input; do not rely on a built-in pair layout in the explanation.
- Exercise 2.14: compare the two parallel-resistance formulations to diagnose dependency in interval arithmetic. Purpose: distinguish an implementation error from information lost when repeated uncertain quantities are treated independently. Invariant: any claimed bound must contain every result produced by admissible concrete inputs under the formula being evaluated; do not assume algebraic equivalence guarantees equally tight intervals.

### Transfer prompt

Describe separate responsibilities for a model-visible `ToolSpec`, a stable `CapabilityId`, and a runtime registry. For each, name who may create it, what invariant it must establish, what consumers may observe, and what it explicitly cannot authorize. Include the failed path for a well-formed specification whose capability is absent or forbidden, but do not define an agent-harness API.

## Dialogue

**PROVISIONAL.** `Which observable equations must remain true for clients when a representation changes, and which additional evidence is required before a visible tool description can be treated as an executable capability?`

This question is queued; [[knowledge/rsi/sicp/course/dialogue_state|the dialogue state]] still marks Seminar 01 as `CURRENT`, and silence does not advance that cursor.

## Navigation

Previous: [[knowledge/rsi/sicp/course/seminars/01-process-shape-and-higher-order-abstraction|Seminar 01 — Process shape and higher-order abstraction]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/03-trees-sequences-and-functional-interfaces|Seminar 03 — Trees, sequences, and functional interfaces]]
