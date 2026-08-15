# Seminar 04 — Symbols, sets, compression, and generic dispatch

## Reading route

Read this seminar as one escalating design problem: once programs can inspect symbolic data, the chosen representation determines both what can be expressed and how work is dispatched. Read the quotation, set, Huffman, tagging, data-directed, and coercion sections closely. Use the other sections to see those mechanisms assembled, but do not give every algebraic package equal attention on a first pass.

- Required: [[evidence/sicp/sicp.pdf#page=220|§2.3.1, printed pp. 192–197 / PDF pp. 220–225]], [[evidence/sicp/sicp.pdf#page=233|§2.3.3, printed pp. 205–218 / PDF pp. 233–246]], and [[evidence/sicp/sicp.pdf#page=246|§2.3.4, printed pp. 218–228 / PDF pp. 246–256]].
- Required: [[evidence/sicp/sicp.pdf#page=265|§2.4.2, printed pp. 237–242 / PDF pp. 265–270]], [[evidence/sicp/sicp.pdf#page=270|§2.4.3, printed pp. 242–253 / PDF pp. 270–281]], and [[evidence/sicp/sicp.pdf#page=290|§2.5.2, printed pp. 262–274 / PDF pp. 290–302]].
- Optional: [[evidence/sicp/sicp.pdf#page=225|§2.3.2, printed pp. 197–205 / PDF pp. 225–233]] applies constructors, selectors, and simplification policy to symbolic differentiation. [[evidence/sicp/sicp.pdf#page=257|§2.4.1, printed pp. 229–237 / PDF pp. 257–265]] develops rectangular and polar complex representations before tags let them coexist. [[evidence/sicp/sicp.pdf#page=282|§2.5.1, printed pp. 254–261 / PDF pp. 282–289]] assembles scheme-number, rational, and complex packages and introduces the two-level tag trace used below.
- Skim: [[evidence/sicp/sicp.pdf#page=302|§2.5.3, printed pp. 274–293 / PDF pp. 302–321]]. Track the separation between polynomial operations and dense or sparse term-list representations; defer the detailed polynomial implementation unless symbolic-algebra representation is your immediate concern.

## System-design problem

A system that admits new representations and operations must decide how data denotes its kind, how a request selects an implementation, and what happens when argument kinds do not match. How can those choices remain additive without turning names or tags into unchecked authority, and how do we keep the representation's cost and ambiguity visible?

## Vocabulary

- **Quotation**: the syntactic mechanism that prevents the quoted object from being evaluated and instead makes its symbolic structure available as data.
- **Set representation**: a concrete organization—such as an unordered list, ordered list, or search tree—that implements the same abstract membership and combination operations with different invariants and costs.
- **Prefix code**: a variable-length code in which no complete symbol code is the prefix of another, so a decoder can recognize a symbol without a separator.
- **Type tag**: explicit data attached to a representation so generic code can decide which implementation understands its contents.
- **Operation-and-type table**: a mapping from an operation name and argument type tags to the procedure that implements that combination.
- **Additivity**: the ability to install a new package by contributing table entries rather than editing every existing generic operation or package.
- **Coercion**: a meaning-preserving conversion that lets an object participate in an operation defined for another type.

## Argument map

1. **CLAIM.** Quotation makes a symbol or compound symbolic expression available as data rather than asking the evaluator for the values of its names. [[evidence/sicp/sicp.pdf#page=220|§2.3.1 distinguishes `(list a b)` from `(list 'a 'b)` and explains `'x` as an abbreviation for `(quote x)` (printed pp. 192–197 / PDF pp. 220–225)]]. **EVIDENCE.** With `a` and `b` bound to numbers, the unquoted form constructs their numeric values while the quoted form constructs the symbols themselves. **INFERENCE.** A printed symbolic expression may therefore be either a request for evaluation or data to inspect; the quotation boundary determines which.
2. **CLAIM.** The representation chosen for a set changes the cost of its operations even when the abstract membership, adjoining, union, and intersection contracts remain the same. [[evidence/sicp/sicp.pdf#page=233|§2.3.3 compares unordered lists, ordered lists, and binary trees (printed pp. 205–218 / PDF pp. 233–246)]]. **EVIDENCE.** Unordered-list membership is Θ(n) and intersection of two size-n sets is Θ(n²); ordering supports a merge-like Θ(n) intersection, while a balanced search tree supports Θ(log n) membership. **INFERENCE.** The balanced-tree bound depends on preserving balance, so an abstract operation name alone is not a cost contract.
3. **CLAIM.** A Huffman tree places frequent symbols nearer the root and uses each input bit to choose a branch, coupling the representation to both encoded length and decoding work. [[evidence/sicp/sicp.pdf#page=246|§2.3.4 defines the weighted prefix tree and its decoder (printed pp. 218–228 / PDF pp. 246–256)]]. **EVIDENCE.** Decoding consumes one bit per branch transition and restarts at the root whenever it reaches a leaf; the sample `10001010` follows paths of lengths three, one, and four to produce three symbols. **INFERENCE.** For a fixed tree, decoding work is linear in the encoded bit count, while the frequency-shaped path lengths determine how many bits a particular message needs. The section states Huffman's optimality result but does not prove it.
4. **CLAIM.** Type tags let rectangular and polar complex-number representations coexist because generic selectors can inspect the tag, remove it, and pass only the contents to the selected representation procedure. [[evidence/sicp/sicp.pdf#page=265|§2.4.2 develops tagged complex data and explicit dispatch (printed pp. 237–242 / PDF pp. 265–270)]]. **EVIDENCE.** The same pair `(3, 4)` denotes different quantities under `rectangular` and `polar`; the tag supplies the information required to choose whether `magnitude` computes a square root or selects the stored radius.
5. **CLAIM.** Data-directed programming moves operation-and-type choices into a table, allowing a representation package to be added by installing entries without modifying existing generic selectors. [[evidence/sicp/sicp.pdf#page=270|§2.4.3 replaces per-operation conditionals with `put`, `get`, and `apply-generic` (printed pp. 242–253 / PDF pp. 270–281)]]. **EVIDENCE.** Rectangular and polar packages keep same-named internal procedures private and publish only table entries; the generic `magnitude` remains unchanged when another package is installed. **INFERENCE.** Additivity moves integration work into registration conventions and lookup failure handling; it does not eliminate the need for a coherent operation, type, and contents contract.
6. **CLAIM.** Coercion can reduce the number of explicit mixed-type operations by raising values through related types, but a simple tower does not solve ambiguous multiple-supertypes or all useful mixed-operation cases. [[evidence/sicp/sicp.pdf#page=290|§2.5.2 develops pairwise coercion, type towers, and the inadequacies of general hierarchies (printed pp. 262–274 / PDF pp. 290–302)]]. **EVIDENCE.** A tower gives one successive `raise` path from integer toward complex, whereas an isosceles right triangle has more than one immediate supertype and therefore no unique direction to raise. **INFERENCE.** Automatic conversion needs an explicit search and preference policy; “a conversion exists” is insufficient to determine which method should run or whether information loss is acceptable.

## Mechanism trace

Trace `(magnitude z)` for the two-level representation of `3 + 4i` introduced in §2.5.1. This is explicitly a worked example of Exercise 2.77 because the seminar contract requires a detailed mechanism trace; Exercise 2.77 is not one of the two selected practice exercises below.

```text
z = tagged complex value
      outer tag: complex
      outer contents = tagged rectangular value
                         inner tag: rectangular
                         inner contents: (3 . 4)

call magnitude(z)
  -> apply-generic('magnitude, z)
     type-tags = (complex)
     get('magnitude, (complex))
       -> forwarding selector installed by the complex package
     strip outer tag with contents(z)
       -> tagged rectangular value
     invoke forwarding selector on that value

  -> forwarding magnitude calls apply-generic again
     op = magnitude
     type-tags = (rectangular)
     get('magnitude, (rectangular))
       -> rectangular package's magnitude procedure
     strip rectangular tag
       -> raw contents (3 . 4)
     invoke rectangular magnitude on (3 . 4)
       -> sqrt(3² + 4²)
       -> 5
```

The first table lookup selects the complex package's forwarding method; the second selects the rectangular representation package's method. `apply-generic` is invoked twice, and each level removes exactly the tag that selected its implementation. This is why installing `magnitude` under `(complex)` matters: without that outer entry, dispatch stops before the inner `rectangular` tag becomes visible. The two-level representation and forwarding entries are shown in [[evidence/sicp/sicp.pdf#page=286|§2.5.1, printed pp. 258–261 / PDF pp. 286–289]]; the operation-and-type lookup rule comes from [[evidence/sicp/sicp.pdf#page=272|§2.4.3, printed pp. 244–248 / PDF pp. 272–276]].

## Rust lens

This deliberately mixed design combines a closed Rust value universe with a registry that is dynamically populated over closed operation and type-tag enums:

```rust
// Explanatory sketch; not compiled by this course.
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
enum Value {
    SchemeNumber(f64),
    Rational { numer: i64, denom: i64 },
    Rectangular { real: f64, imag: f64 },
    Polar { magnitude: f64, angle: f64 },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum TypeTag { SchemeNumber, Rational, Rectangular, Polar }

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum UnaryOperation { Magnitude }

type UnaryMethod = fn(&Value) -> Result<Value, DispatchError>;

struct Registry {
    unary: BTreeMap<(UnaryOperation, TypeTag), UnaryMethod>,
}

impl Registry {
    fn apply_unary(&self, op: UnaryOperation, value: &Value) -> Result<Value, DispatchError> {
        let tag = match value {
            Value::SchemeNumber(_) => TypeTag::SchemeNumber,
            Value::Rational { .. } => TypeTag::Rational,
            Value::Rectangular { .. } => TypeTag::Rectangular,
            Value::Polar { .. } => TypeTag::Polar,
        };
        let method = self.unary.get(&(op, tag)).ok_or(DispatchError::NoMethod)?;
        method(value)
    }
}

enum DispatchError { NoMethod, WrongPayload }
```

The exhaustive `match` makes the compiler identify places affected by adding a `Value` variant. `UnaryOperation` and `TypeTag` likewise close the registry's key universe: code may populate another method for an already-declared `(UnaryOperation, TypeTag)` pair without changing the lookup algorithm, but adding a new operation or representation still requires editing an enum and recompiling affected matches. This is dynamic population, not SICP's fully additive-package sense. Missing methods and mismatched payloads remain runtime errors. A truly open plugin boundary would also need type erasure or a trait-object protocol, versioning, and explicit registration policy. Rust exhaustiveness and dynamic registration solve different change problems; neither makes the other free.

## Agent-harness bridge

**SICP CLAIM.** A data-directed system can discover an implementation by an operation-and-type key, while the tag and table entry remain distinct pieces of the dispatch mechanism. [[evidence/sicp/sicp.pdf#page=270|§2.4.3, printed pp. 242–248 / PDF pp. 270–276]]

**HARNESS EVIDENCE.** In OpenAI Codex at pinned commit `a850875a8eb603d18cb14cb2c5e80c930de9bd48`, `ToolRouter` stores model-visible specifications separately from its `ToolRegistry`, exposes deferred tool namespaces, constructs a `ToolName` from response-item namespace and name fields, and sends a constructed invocation to the registry for dispatch. [[evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs|`codex-rs/core/src/tools/router.rs:31–289`]] ([exact lines 31–289](../../../../../evidence/implementations/codex-sicp/snapshot/codex-rs/core/src/tools/router.rs#L31-L289)) In Hermes Agent at pinned commit `e444d165807f489b5c1ab8e4a612c8d09c2e67a2`, the conversation loop first attempts to repair an unrecognized tool name, checks the resulting name against `valid_tool_names`, checks that argument payloads are valid JSON, records the tool-call turn, and only then hands the surviving calls to `_execute_tool_calls`. [[evidence/implementations/hermes/snapshot/agent/conversation_loop.py|`agent/conversation_loop.py:5700–6165`]] ([exact lines 5700–6165](../../../../../evidence/implementations/hermes/snapshot/agent/conversation_loop.py#L5700-L6165))

**INFERENCE.** These mechanisms suggest three separate contracts. **Additive discovery** determines which specifications or namespaces can become visible without rewriting a central dispatcher. **Name normalization** converts an external spelling or namespace/name pair into a canonical lookup identity and can still fail. **Execution authorization** must decide whether the resolved runtime may perform the requested effect in the present context. Codex's registry dispatch and Hermes's valid-name gate are evidence about routing and validation, not proof of complete authorization: finding or repairing a name cannot itself grant sandbox, credential, or operating-system authority.

**ARCHITECTURE QUESTION — INFERENCE.** A testable agent-harness contract should preserve separate receipts for advertised or discovered specifications, normalized capability identity, registry resolution, policy decision, and effect result. An additive registration mechanism should make a new tool requestable without editing every caller, while the effect plane still rejects a request whose name resolves but whose authority or dependency attestation is absent.

## Limits of the analogy

A SICP type tag describes how a datum should be interpreted; it is not a security principal, credential, or grant. An operation-and-type table assumes trusted package installation and synchronous procedure application, whereas an agent tool may cross a process boundary, mutate external state, fail asynchronously, or require revocable authority. Huffman decoding is deterministic traversal of a fixed prefix tree; model-generated tool selection is neither compression nor prefix decoding. The useful analogy is limited to separating representation, lookup key, selected implementation, and invocation—not equating their semantics or risk.

## Practice

### Manual trace

Trace `(magnitude z)` where `z` has outer tag `complex` and inner representation `polar` with contents `(5 . θ)`. Record each call to `apply-generic`, the operation name, type-tag list, table key, selected procedure, value after `contents`, and returned value. Then repeat only the lookup decisions for an inner `rectangular` value `(3 . 4)` and identify which package changes and which generic interface remains fixed.

### SICP exercises

- Exercise 2.70: build and use the specified Huffman tree to compare its message length with a fixed-length encoding. Purpose: connect symbol frequencies, tree shape, path lengths, and total encoded size. Invariant: each symbol has exactly one root-to-leaf code and no complete code is a prefix of another. Do not treat the exercise's result as supplied here.
- Exercise 2.82: generalize generic application to multiple arguments and find a case that defeats the proposed “coerce everything to one argument's type” strategy. Purpose: expose the difference between finding some conversions and selecting every applicable mixed-type method. Invariant: coercion must preserve the represented value, terminate, and invoke only a method registered for the final ordered type tuple. No solution or preferred search algorithm is given here.

### Transfer prompt

Design an additive tool-dispatch boundary on paper. Separate discovery of a model-visible `ToolSpec`, normalization to a stable capability identity, registry lookup, contextual authorization, and effect execution. For each stage, name the input, output, owner, and failure receipt. Explain how a new tool can install its specification and runtime without editing the dispatcher, and why possession of its advertised name or normalized identity still grants no execution authority.

## Dialogue

**PROVISIONAL.** `Which facts belong in a dispatch key, and which checks must remain outside lookup so that an additive registry does not confuse recognition with authority?`

This question is queued; [[knowledge/rsi/sicp/course/dialogue_state|the dialogue state]] still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [[knowledge/rsi/sicp/course/seminars/03-trees-sequences-and-functional-interfaces|Seminar 03 — Trees, sequences, and functional interfaces]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/05-state-identity-and-environments|Seminar 05 — State, identity, and environments]]
