# Chapter 1 prerequisite repair

Date: 2026-09-07. Implementer status: DONE_WITH_CONCERNS.
This record covers the bounded Chapter 1 prerequisite repair, not whole-wave
acceptance, publication acceptance, or external mathematical peer review.

## Source and ownership

Base revision: `4a72888fd9b09d86e958da35074e86c08615a257` on
`codex/harp-mathematics-spec`.

Owned implementation files:

- `knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md`
- `formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean`
- this review record.

The first frozen candidate had SHA-256
`2b82dd102d6a8628e44c0e79af890d7e838e76b4524cf0982f859da4df8807f4`
for the chapter prose and
`523be86fdaac60964d7df1e7e4e8024f2859f03aa64f2096fdaf17196c78e5ce`
for its Lean module. Publication metadata and subsequent review repairs may
change these digests; they identify the candidate inspected here.

Existing public Lean declarations and all six
`Exercises.Chapter01.exercise_YY_solution` types and bodies were preserved.
The source diff adds basis-aware declarations and norm/family helpers. No
shared JSON, generated corpus, receipt, edition, issue, or commit was changed
by this implementation subtask.

## Findings and repairs

The old CFT-01-002 mapping only selected a column in the standard coordinate
basis. CFT-01-003 unfolded a definition of matrix action rather than proving
the arbitrary-basis claim in the prose. The replacement declarations carry
explicit bases; coordinate action is proved from `Module.Basis.sum_repr` by
moving the two linear maps through the finite sum. The prose reconstructs
existence and uniqueness from basis expansions and derives coordinate
conjugacy, including both inverse identities and equality on every column.

The old CFT-01-006 declaration only compared output lengths at one vector.
That does not alone establish different operator norms. The new indexed
candidate proves both inverse identities, similarity, uniform squared bounds,
and a unit vector attaining both bounds. The prose gives the square-root and
supremum argument and explicitly distinguishes these Euclidean sums from the
default norm on Lean's function type.

The running family has an explicit basis
`((s⁻¹,0),(0,1))`, old-to-new coordinate map `diag(s,1)`, conjugacy
`A_(lambda,alpha) ↦ A_(lambda,s*alpha)`, and squared output calculation
`alpha² + lambda² ↦ s²*alpha² + lambda²`. The transported metric calculation
shows why the fixed underlying vector retains its geometric length.

The revised E03 asks for the exact existing semiring column-expansion
statement and separately explains its field/basis interpretation. E04 names
the determinant prerequisites. E05 asks for the characteristic polynomial,
trace, explicit counterexample, and real orthogonal dot-product identity that
its existing solution actually proves. The complex unitary norm corollary is
labeled as a prose consequence beyond that exercise's formal type.

Motivation, historical framing, and the four ML fields are labeled. The
historical section makes no unsupported attribution or priority claim.

## Publication recommendations

All declaration names below have prefix `CrouzeixTextbook.Part01.` and source
`formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean`.
These are recommendations for the maintained publisher, not compiler receipts.

| Card | Public declaration | Substantive support | Prose proof scope |
| --- | --- | --- | --- |
| CFT-01-001 | `LinearTransformation` | Mathlib `LinearMap` definition | not-applicable |
| CFT-01-002 | `basis_image_columns` | `LinearMap.toMatrix_apply` | reconstructible |
| CFT-01-003 | `basis_coordinate_action` | `Module.Basis.sum_repr`, `LinearMap.toMatrix_apply`, local finite-sum proof | reconstructible |
| CFT-01-004 | `basis_change_conjugacy` | `LinearMap.toMatrix_comp`; unindexed `basis_change_inverse` | reconstructible |
| CFT-01-005 | `similarity_preserves_charpoly` | `Matrix.charpoly_units_conj` | summary: forward determinant dependency |
| CFT-01-006 | `nonunitary_similarity_norm_counterexample` | local `nonnormalExample_similarity`, `nonunitary_similarity_changes_output_length_sq`, sharp-bound proof | reconstructible |

The intended correspondence for all six is exact at the stated specialization
or explicit equivalent formulation. CFT-01-004 uses opposite-direction
identity matrices in its formal equality; the separate compiled inverse
helper justifies the inverse notation. CFT-01-006 uses attained squared
Euclidean gain bounds as the explicit equivalent formulation of its norm
values. Cards 002–004 assume a field and finite bases; card 005 permits any
commutative ring and requires a matrix unit; card 006 is over the real numbers.
Card 001's semiring/semimodule alias is explicitly specialized to vector spaces.

Do not fabricate `underlying_declaration`, declaration positions, type hashes,
assumptions, axiom sets, alias classifications, or verification receipts from
this table. Compiler output must determine `formal_mode` and the exact
registered provider. In particular, a theorem proved using a Mathlib result
is not necessarily classified as a publisher reexport. External support names
above are mathematical attribution, not a second provider registry. New
unindexed helpers do not increase the six-card count.

## Executed verification

This was ordinary local non-hermetic execution with the existing warm cache.
No Lake invocation, dependency download, toolchain update, or cache-maintenance
command was run.

Preflight inspected `lean-toolchain` (`leanprover/lean4:v4.32.1`), the worktree
`.lake` symlink and its canonical target and parent directories, and existing
`ToLin.olean`, `Charpoly/Basic.olean`, and `Mathlib/Tactic.olean` artifacts.
`mise trust --show` reported both the primary checkout and the worktree trusted.
The unqualified `lean --version` command was unavailable on PATH; the exact
installed executable below reported Lean 4.32.1 with commit
`f054605aea4b840552cca2e725580bffd1e1b704`.

From the worktree's `formalization/lean` directory, the executed narrow command
was:

```sh
chapter_lean_path=""
for chapter_pkg in .lake/packages/*/.lake/build/lib/lean; do
  chapter_lean_path="$chapter_lean_path:$chapter_pkg"
done
LEAN_PATH="${chapter_lean_path#:}" \
  /Users/bytedance/.elan/toolchains/leanprover--lean4---v4.32.1/bin/lean \
  CrouzeixTextbook/Part01/Chapter01.lean
```

The first run exited 1: real-matrix simplification left explicit entries in
two bound goals, and `λ` was a reserved binder token. The second run exited 1:
the running-family conjugacy needed cancellation of the nonzero scaling
factor. Both defects were repaired. The third run exited 0 with empty output,
checking the complete chapter file including all six unchanged exercise
solutions. No `.olean` output target was supplied, so this check did not write
Harp modules into the shared dependency cache.

`git diff --check` exited 0 after the freeze. The Lean and prose diffs were
read directly, including the exercise contracts and matrix products. The
declared source links resolve relative to the chapter to the existing Lean
file. This subtask did not run a browser, PDF builder, compiler receipt
publisher, `lean-all`, or `mise run verify`; the coordinating task owns those
publication and final acceptance checks.

## Remaining acceptance boundary

CFT-01-005 has exact formal statement correspondence but a forward-reference
prose proof. Its determinant multiplicativity and polynomial-matrix setup are
not derived from Chapter 1's basis prerequisites. Treating this as a fully
self-contained determinant proof would overstate the result. This is
compatible with narrow prerequisite acceptance for Chapters 2–4 and leaves
the older whole-wave requirement for reconstructible proofs open. Independent
specification and quality reviews, and current compiler/publication checks,
remain the coordinating task's acceptance responsibilities.

## Independent prerequisite reviews

The specification reviewer read the complete revised prose and Lean module,
checked preservation of the old public declarations and six exercise
signatures, and independently ran `lake env lean
CrouzeixTextbook/Part01/Chapter01.lean`. It exited 0 with no diagnostics.
The reviewer accepted the arbitrary-basis mappings, inverse convention,
sharp squared bounds and attainment, and the explicitly deferred determinant
proof. The subsequent quality reviewer independently read the full source
and base diff, checked that the exercise namespace was byte-preserved,
resolved all seven Lean source links, and checked the recorded candidate
hashes. No blocking findings remained. These are agent reviews, not human
peer review.

The integration owner also built the narrow
`CrouzeixTextbook.Part01.Chapter01` target: exit 0, 2997 jobs. The official
exporter's metadata functions compiled a 12-declaration probe covering the
six replacement card providers and six exercises. The five theorem providers
and six exercises are theorems, not direct aliases; the transformation alias
is a definition. Probe metadata is only an integration input. The complete
publisher receipt and final acceptance are recorded after all chapters freeze.

This accepts Chapter 1's coordinate foundations as prerequisites for the
current package. It does not close the older whole-chapter issue while
CFT-01-005 remains a preview. The frontmatter exact count was synchronized
after review; that mechanical metadata change is not covered by the earlier
prose snapshot digest.
