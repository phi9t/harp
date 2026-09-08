# Chapter 2 bounded implementation record

Date: 2026-09-07. Implementer status: DONE_WITH_CONCERNS pending independent
specification/quality review and coordinated publication verification.
This record is not whole-wave acceptance, a compiler receipt, or external
mathematical peer review.

## Ownership and frozen candidate

Base revision: `4a72888fd9b09d86e958da35074e86c08615a257`, with the accepted
Chapter 1 prerequisite and shared-contract work already in progress in
`codex/harp-mathematics-spec`.

Owned files:

- `knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md`
- `formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean`
- this record.

Frozen source SHA-256 values before independent review:

- Prose: `6ac603ec4e721221ee0bdc05ed22ec13ae3f5464ad7ba55cbc5febd2ab92fbda`
- Lean: `1cb7a2c9ccbbbd2aa8c8f73712472b2bce56129fcac9cb2292f1e4ec67cc9b8b`

All six CFT-02 card IDs, their public declaration names, and existing theorem
types were retained. The two membership/surjectivity checkpoints received
local proof bodies so they need not be registered as bare aliases. Other
indexed proof bodies were preserved. No shared JSON, exporter, generated
artifact, other chapter, issue, commit, or push was changed by this subtask.

## Mathematical changes

The span proof now constructs finite combinations, checks zero/addition/scalar
closure, proves containment in every containing subspace, and identifies the
construction with the intersection definition. Basis coordinates separate
existence from spanning and uniqueness from independence, including finite
support for infinite index sets. Basis transport is derived in both directions;
the finite dimension interpretation is separated from general Lean `finrank`,
which is zero on infinite cardinal dimension. Direct-sum uniqueness remains
the subtraction/intersection argument; existence explicitly requires the sum
to be the whole space.

The quotient section proves the equivalence relation, representative-independent
addition and scalar multiplication, inherited vector-space laws, projection
surjectivity and kernel, and universal factorization. The factorization proof
checks representative independence, linearity, the commuting identity, and
uniqueness. These claims have separate compiled supporting declarations and
are not attributed collectively to projection surjectivity.

The opening and ML example use the linear map
`T(x,y,z) = (x+z,y+z)` with kernel span of `(-1,-1,1)`. The text proves that
the quotient records predictor identities on the full real input plane, and
explicitly excludes nonlinear network symmetries and finite-dataset equality
from that transfer. It computes the complementary representative at `z=0`.

The triangular family calculates its invariant first-coordinate line,
`N = A - lambda I` and `N² = 0`, ordinary versus exponent-two generalized
eigenspaces, lack of an invariant complement when alpha is nonzero, and the
scalar quotient action. Local terms are defined before use. The formal
obstruction considers every normalized complement candidate `(t,1)`; the
prose supplies the normalization argument relating this to complementary
lines. The diagonal quotient's explicit inverse is labeled an additional
prose consequence beyond E04's unique-factorization type.

Exactly eight required level-two headings are present. Motivation is nested,
historical context is labeled without unsupported attribution, and the four
ML fields are explicit. Display matrices use `bmatrix` and math uses `$`/`$$`.
Each card has statement, proof, hypotheses, boundary, public declaration and
support names, and a source link to the chapter's own Lean file. No `.lake`
symlink source links were added.

## Exercise prompt changes

All six declarations are new theorems under
`CrouzeixTextbook.Part01.Exercises.Chapter02`.

| Exercise | Old prompt | New prompt and matching checked conclusion |
| --- | --- | --- |
| E01 | State abstract closure laws | Check zero, addition, and scalar closure for the concrete diagonal equality on real pairs |
| E02 | Compute span, with determinant-based answer | Prove the explicit half-sum/half-difference coordinates and the resulting whole-plane span equality |
| E03 | Prove general span minimality | Prove the concrete inclusion of spans of `(2,0),(0,2)` into the span of `(1,1),(1,-1)` using sum and difference witnesses |
| E04 | Prove direct-sum uniqueness | Prove existence and uniqueness of the linear factor of `x-y` through the diagonal quotient |
| E05 | Give two subspaces with nonsubspace union | Exhibit two real pairs in the coordinate-axis union whose sum has neither coordinate zero |
| E06 | Reapply span minimality in Lean | Prove equal running-map outputs iff the parameter difference is a scalar multiple of the null direction |

The new prompts are written beside their solutions. The generic span and
direct-sum proofs remain in the formal development rather than being repeated
as exercise aliases. E01–E06 have six distinct mathematical theorem types;
the publisher must independently verify their fingerprints against all cards
and exercises, rather than relying on this source inspection.

## Publication recommendations

All public names below are prefixed by `CrouzeixTextbook.Part01.` and live in
`formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean`.

| Card | Retained public declaration | Substantive support | Prose proof scope |
| --- | --- | --- | --- |
| CFT-02-001 | `span_minimality` | `Submodule.span_le`; local `span_finite_combination`, `span_as_intersection`, `finite_combination_closure` | reconstructible |
| CFT-02-002 | `mem_subspace_intersection_iff` | local two-implication proof and `intersection_closure` | reconstructible |
| CFT-02-003 | `basis_coordinates_unique` | injectivity of `b.repr`; local `basis_expansion_exists_unique` | reconstructible |
| CFT-02-004 | `dimension_invariant_under_linear_equiv` | `LinearEquiv.finrank_eq`; local `basis_transport` | reconstructible at the explicit finite/general convention |
| CFT-02-005 | `direct_sum_coordinates_unique` | local subtraction proof; separate `direct_sum_existence` using `Submodule.mem_sup` | reconstructible |
| CFT-02-006 | `quotient_projection_surjective` | local representative proof using `Submodule.mkQ_surjective`; separate quotient support below | reconstructible |

The intended correspondence is exact for all six stated indexed claims. Cards
001–003 allow semirings/semimodules and are explicitly specialized to fields.
Cards 004–005 allow division rings; card 004 retains general `finrank` without
assuming finite dimension. Card 006 allows rings/modules. Concrete exercises
and running examples use real products. Supporting field helpers cover:

- `quotient_relation_laws`: reflexivity, symmetry, transitivity of difference membership.
- `quotient_operations_well_defined`: addition and scalar compatibility.
- `quotient_class_eq_iff`: exact class equality criterion, using `Submodule.Quotient.eq`.
- `quotient_projection_kernel`: kernel equality, using `Submodule.ker_mkQ`.
- `quotient_factor_well_defined`: equality of output values from difference membership in the annihilated subspace.
- `quotient_factor_exists_unique`: unique linear factor, constructed with `Submodule.liftQ`/`Submodule.liftQ_mkQ` and a local uniqueness proof.

Recommend compiler-determined theorem/local-proof modes for the two expanded
checkpoints and six exercise solutions. Do not manually fabricate modes,
providers, declaration positions, hashes, assumptions, axiom sets, or receipt
fields from these recommendations. A theorem that uses a Mathlib result is
not necessarily an exporter reexport. The maintained compiler publisher owns
the actual metadata and formal-mode classification.

## Verification executed

The preflight inspected the pinned toolchain (`leanprover/lean4:v4.32.1`),
the worktree `.lake` symlink to the canonical primary-checkout cache, its
directory ancestry, the existing quotient dependency artifact, and worktree
mise trust. Both primary checkout and worktree were trusted. The exact binary
reported Lean 4.32.1, commit `f054605aea4b840552cca2e725580bffd1e1b704`.

The TDD skill was applied to the new exercise declarations. Before adding
them, six `#check` compile clients were appended to the chapter. The first
narrow compile exited 1 with exactly six unknown-identifier errors, one for
each missing `exercise_01_solution` through `exercise_06_solution`. This
established the missing-declaration RED state.

The repeated narrow command, run from the worktree's `formalization/lean`, was:

```sh
chapter_lean_path=""
for chapter_pkg in .lake/packages/*/.lake/build/lib/lean; do
  chapter_lean_path="$chapter_lean_path:$chapter_pkg"
done
LEAN_PATH="${chapter_lean_path#:}" \
  /Users/bytedance/.elan/toolchains/leanprover--lean4---v4.32.1/bin/lean \
  CrouzeixTextbook/Part01/Chapter02.lean
```

Intermediate implementation compiles exited 1 for tuple annotation syntax,
over-eager quotient extensionality, a pair eta rewrite, and an already
simplified eigenline goal. These were corrected without changing any intended
statement. The final compile exited 0 with no warnings or errors and printed
the six distinct exercise types through their retained compile clients.
All local helpers and indexed statements were checked in that same run.

`git diff --check` exited 0 after the freeze. Direct diff inspection and the
heading check confirmed the bounded file scope and all eight required
level-two headings. SHA-256 values above were obtained after the final source
edits. This was ordinary local non-hermetic execution against the warm cache;
no Lake command, cache hydration, dependency download, toolchain update, or
shared `.olean` output was used.

## Remaining acceptance boundary

Independent specification and quality reviews both passed on the frozen
candidate. The specification reviewer checked the full source, mathematical
claims and exercise coverage, and independently compiled the module with
Lean 4.32.1 (exit 0). The quality reviewer checked the complete source,
prose-to-statement correspondence, local links and frozen hashes, and found
no blocking issue. The coordinator's narrow Lake build also passed, as did
the actual PDF renderer's Chapter 2 formula and solution-reference test.

The coordinator still owns exercise fingerprint checks, current exporter
mode/provider classification, shared coverage metadata, PDF and corpus
publication, `lean-all`, and the full repository gate. This subtask did not
run those global checks, and the local successful compile must not be reported
as a completed release gate. No remaining mathematical or Lean compile defect
is known in the frozen candidate.
