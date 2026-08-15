# Seminar 03 — Trees, sequences, and functional interfaces

## Reading route

Read §2.2 as an argument about controlling structural complexity. Start with the closure property that makes nested data possible, follow the recursive shape of trees, and then watch the sequence interface separate selection and transformation from traversal. The picture language is worth reading as an architectural example: its importance is the layering, not the graphics API.

- Required: [[evidence/sicp/sicp.pdf#page=160|§2.2 opening and §§2.2.1–2.2.3, printed pp. 132–171 / PDF pp. 160–199]].
- Optional: [[evidence/sicp/sicp.pdf#page=200|§2.2.4, printed pp. 172–191 / PDF pp. 200–219]], concentrating on the language's levels and means of combination rather than reproducing every painter.
- Skim: the detailed painter definitions and exercise sequence in [[evidence/sicp/sicp.pdf#page=209|the latter half of §2.2.4, printed pp. 181–191 / PDF pp. 209–219]]; return when you want to test whether a new operation composes at every level.

## System-design problem

Hierarchical data is useful because a small constructor can build structures of unbounded depth, but that same freedom can entangle traversal, selection, transformation, and combination in every operation. How can a system follow the shape of nested data where necessary, expose common stages through a stable interface, and let higher-level combinations remain intelligible when the representation or traversal strategy changes?

## Vocabulary

- **Closure property**: the result of combining values with an operation can itself be combined again with that operation, permitting arbitrary nesting.
- **Tree recursion**: a process whose recursive calls follow the recursive branches of hierarchical data and whose base cases handle an empty structure and individual leaves.
- **Sequence interface**: a conventional set of operations—such as enumeration, filtering, mapping, and accumulation—that lets a pipeline state what happens to elements separately from how a structure is traversed.
- **Enumeration**: conversion of a structure's relevant members into an ordered sequence for subsequent stages.
- **Accumulation**: combination of a sequence into a result using an initial value and a combining operation; also called a fold in the Rust discussion below.
- **Stratified design**: organization into levels where each level has its own primitives, combination methods, and abstractions built from the level below.

## Argument map

1. **CLAIM.** Closure under pair construction makes pairs a basis for hierarchical data because the components of a pair may themselves be pairs. [[evidence/sicp/sicp.pdf#page=160|The opening of §2.2 identifies closure as the property that permits hierarchical structures (printed pp. 132–134 / PDF pp. 160–162)]]. **INFERENCE.** A representation gains expressive reach from recursive composition, but every consumer must then be prepared for structure at more than one depth.
2. **CLAIM.** Operations on list-encoded trees distinguish an empty tree or list, a non-pair leaf, and a compound pair whose branches are processed recursively. [[evidence/sicp/sicp.pdf#page=175|§2.2.2 develops tree recursion through counting leaves, scaling trees, and mapping over trees (printed pp. 147–153 / PDF pp. 175–181)]]. **EVIDENCE.** The concrete `count-leaves` analysis handles the empty list first, counts a non-pair as one leaf, and otherwise combines recursive results from the pair's two components (printed pp. 148–149 / PDF pp. 176–177). **INFERENCE.** This correspondence gives a review test—each data variant needs a matching computational case—but it does not by itself determine time, stack, or allocation costs.
3. **CLAIM.** Enumeration, filtering, mapping, and accumulation form a conventional interface that reorganizes programs around successive transformations of sequences. [[evidence/sicp/sicp.pdf#page=182|§2.2.3 extracts these stages from examples including sums over trees and presents them as signal-flow-like pipelines (printed pp. 154–171 / PDF pp. 182–199)]]. **EVIDENCE.** Once a tree operation produces a sequence, the predicate, element transformation, and result combination can vary without rewriting that traversal.
4. **CLAIM.** Accumulation captures a shared combining pattern behind several sequence operations. [[evidence/sicp/sicp.pdf#page=186|§2.2.3 derives sequence operations in terms of `accumulate` and extends the idea to nested mappings (printed pp. 158–171 / PDF pp. 186–199)]]. **INFERENCE.** A common fold boundary reduces duplicated control structure only when the combining operation, initial value, and order make the intended result explicit; naming everything `accumulate` does not make unequal semantics interchangeable.
5. **CLAIM.** The picture language demonstrates stratified design: primitive painters are combined into patterns, and patterns are combined again into larger designs using operations appropriate to each level. [[evidence/sicp/sicp.pdf#page=200|§2.2.4 presents the picture language and closes by analyzing its levels and language of combination (printed pp. 172–191 / PDF pp. 200–219)]]. **INFERENCE.** A good layer does more than hide detail; it supplies combinations whose results remain valid inputs at that layer, preserving the ability to build larger structures without dropping down a level.

## Mechanism trace

Compute the sum of the squares of the odd leaves in this concrete tree:

```text
((1 2) (3 (4 5)))
```

Treat a number as a leaf and a parenthesized group as a branch. A left-to-right tree enumeration follows the recursive representation before any numerical policy is applied:

```text
enumerate ((1 2) (3 (4 5)))
  enumerate (1 2)       -> (1 2)
  enumerate (3 (4 5))
    enumerate 3         -> (3)
    enumerate (4 5)     -> (4 5)
                           -------- append branch results
                           (3 4 5)
                         ----------- append root results
leaves                  -> (1 2 3 4 5)
```

The remaining stages operate on the resulting sequence:

```text
enumerate                 (1 2 3 4 5)
filter odd?               (1   3   5)
map square                (1   9  25)
accumulate + from 0       1 + (9 + (25 + 0)) = 35
```

The stage boundaries expose distinct obligations. Enumeration must visit every leaf exactly once in the chosen left-to-right order. Filtering may remove elements but must not invent or reorder them. Mapping preserves the filtered sequence's cardinality and order while replacing each element with its square. SICP's `accumulate` combines the first value with the recursively accumulated remainder, so this trace is a right fold with an explicitly chosen terminal value (`0`) and combining operation (`+`). The final answer `35` therefore depends on both the tree traversal and the numerical stages, but those decisions no longer occupy one recursive procedure.

This separation also makes changes local and observable. Replacing `odd?` with `even?` does not alter tree traversal. Replacing squaring with cubing does not alter membership. Replacing `+` and `0` with multiplication and `1` changes the reduction contract without changing the earlier sequence. A streaming implementation might fuse these stages and avoid allocating intermediate lists, but it would still need to preserve the same element order and stage semantics if it claims the same result.

## Rust lens

Rust iterators provide a useful translation of the sequence interface, but not the same operational representation. This explanatory sketch is deliberately not compiled by the course:

```rust
// Explanatory sketch; not compiled by this course.
enum Tree<T> {
    Leaf(T),
    Branch(Vec<Tree<T>>),
}

impl<T> Tree<T> {
    fn leaves(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self {
            Tree::Leaf(value) => Box::new(std::iter::once(value)),
            Tree::Branch(children) => {
                Box::new(children.iter().flat_map(|child| child.leaves()))
            }
        }
    }
}

fn sum_of_odd_leaf_squares(tree: &Tree<i64>) -> i64 {
    tree.leaves()
        .copied()
        .filter(|value| *value % 2 != 0)
        .map(|value| value * value)
        .fold(0, |sum, square| sum + square)
}
```

The adapter chain is lazy: `filter` and `map` describe work that `fold` requests element by element, so it need not construct the three intermediate sequences shown in the mechanism trace. Its `Iterator::fold` threads a left accumulator through the values, producing `((0 + 1) + 9) + 25`; SICP's `accumulate` expands to the right as `1 + (9 + (25 + 0))`. Addition makes those two groupings agree here, but a non-associative or order-sensitive combining operation can produce different results, types, or observable effects. SICP's sequence examples in §2.2.3 use Scheme lists and present stages as sequence-producing operations, which makes their intermediate materialization conceptually—and under the ordinary list implementation, operationally—visible. The Rust chain transfers the separation of concerns, not the allocation behavior or fold direction. Boxing the recursive iterator also introduces dynamic dispatch and allocation choices absent from the abstract sequence equations; an optimized Rust design might use a custom traversal type or an explicit stack. The numerical sketch assumes every square and partial sum is representable as `i64`; it does not define an overflow policy.

## Agent-harness bridge

**INFERENCE (architectural transfer).** Context assembly can be examined as a pipeline: enumerate candidate items from permitted sources, filter them by eligibility and budget rules, map accepted items into a model-facing representation, and fold them into an ordered context with receipts. This is a proposed decomposition for design review, not a source-backed claim that any particular agent harness is implemented this way. The SICP evidence supports the value of separating structural traversal from sequence operations; it does not establish the state model, security policy, or context behavior of Pi, Hermes, Codex, or a future system.

## Limits of the analogy

A tree of immutable numbers has stable membership and pure transformations. Real context assembly may read mutable stores, perform I/O, deduplicate correlated records, enforce authority, reserve tokens, redact secrets, or stop when a budget is exhausted. Those effects can make enumeration order observable and can invalidate algebraic rewrites that are safe for pure sequences. A fold over context items is therefore not automatically associative, commutative, replayable, or parallelizable. Treat enumerate/filter/map/fold as a diagnostic partition of responsibilities; verify ordering, effects, failures, and provenance separately before treating it as an implementation plan.

## Practice

### Manual trace

For the tree `((2 (7 8)) ((3) 10 11))`, trace a pipeline that enumerates leaves left to right, retains only odd leaves, squares each retained value, and uses SICP's `accumulate` with addition and terminal value zero. Write every intermediate sequence, then show the complete right-fold expansion and its inside-out unwind; do not replace it with running left-accumulator states. Optionally trace Rust's left-fold accumulator over the same mapped values and explain why addition yields the same total. Finally, state one invariant for enumeration and one invariant for the SICP accumulation.

### SICP exercises

- Exercise 2.27: define the behavior of deep reversal over a nested list and then implement it. Purpose: distinguish reversing one sequence level from recursively transforming every nested level. Invariants: every original leaf occurs exactly once in the result, nesting depth and branch cardinalities are preserved, and the order at each reversed branch follows the exercise's specification. These checks define what to test; they are not an answer key.
- Exercise 2.33: express the requested sequence operations using accumulation. Purpose: identify the combining step and initial value that make each familiar operation a fold. Invariants: the result preserves the source order where the target operation requires it, no source element is visited more than once by the fold, and the empty-sequence result agrees with the chosen initial value. Do not copy a finished definition before stating those contracts.

### Transfer prompt

Decompose context assembly into four pure stages: enumerate candidates, filter them, map accepted candidates into model-facing items, and fold those items into an ordered context. For each stage, specify its input, output, invariant, and failure representation. Identify where token accounting and provenance receipts enter the data, and state which real-world effects you deliberately excluded to keep the exercise pure. This is an architectural transfer exercise, not a claim about any particular harness or a proposed API.

## Dialogue

**PROVISIONAL.** `When does decomposing a traversal into enumerate, filter, map, and fold clarify the contract, and when does it hide an ordering or effect that should stay explicit?`

This question is queued; [[knowledge/rsi/sicp/course/dialogue_state|the dialogue state]] still marks Seminar 01 as `CURRENT`, and missing input does not advance that cursor.

## Navigation

Previous: [[knowledge/rsi/sicp/course/seminars/02-data-abstraction-and-immutable-representation|Seminar 02 — Data abstraction and immutable representation]] · [[knowledge/rsi/sicp/course/sicp_course_guide|Course guide]] · Next: [[knowledge/rsi/sicp/course/seminars/04-symbols-sets-compression-and-generic-dispatch|Seminar 04 — Symbols, sets, compression, and generic dispatch]]
