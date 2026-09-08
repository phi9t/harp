# Chapter 3 bounded implementation record

Date: 2026-09-07. Implementer status: DONE_WITH_CONCERNS pending independent
specification and quality review and coordinated publication verification.
This record is not whole-wave acceptance or a compiler publication receipt.

Final disposition (2026-09-07): the independent reviews recorded below both
passed, and the chapter was accepted locally as part of the approved
foundations core; see [foundations-review.md](foundations-review.md) for the
coordinated publication, presentation and full-gate evidence. The implementer
status above records the state at freeze time and is kept unchanged. Kata
`6bsx` is closed.

## Independent acceptance

The specification review found one omission: the power-zero and higher-power
maps were calculated, but their kernels and ranges were left implicit. The
repair adds the explicit prose formulas and two matching Lean theorems, for
every real alpha including zero. The same reviewer checked the repair and
independently compiled it successfully with no warnings or errors.

A separate quality reviewer read both complete files and the record, checked
the final hashes, all exercise arguments, hypotheses and source links, and
returned PASS with no critical or important findings. The reviewer noted an
optional explanation of why adjoining a vector outside a span preserves
independence; the existing argument was judged reconstructible.

The actual PDF renderer also passed the chapter formula and six named-solution
reference check. These are local source-acceptance results. Compiler metadata,
publication and the full repository gate remain integration requirements.

## Ownership

Base revision: `4a72888fd9b09d86e958da35074e86c08615a257`, with frozen Chapters
1–2 and shared contract changes already present in `codex/harp-mathematics-spec`.
This subtask owns only Chapter 3 Lean, Chapter 3 canonical prose, and this record.
No shared JSON, exporter, generated file, other chapter, issue, commit, or push
was changed by this subtask.

Frozen source SHA-256 values after the first specification-review repair:

- Lean: `eeb2edcc68c823e9689d1e2f2eb4800d982d2d7503f88c8fbfc164be8572561e`.
- Prose: `5437a0a9e5f9ce15cd5b0d5c074873ccef4694e17eb9bb635a14aeba40148b7a`.

All six CFT-03 identities and public declaration names/types remain intact.
`invariantRestriction` remains a definition with explicit invariance evidence,
and `injective_iff_kernel_bottom` now has a local subtraction proof.

## Mathematical coverage

Kernel and range are constructed and shown closed before their membership
cards. Rank–nullity is proved by extending a kernel basis, then proving every
step of image spanning and image independence. The extension step names the
separate pinned Mathlib `Module.Basis.sumExtend` provider. Local helpers
`image_spanning_from_decomposition` and `image_independence_from_kernel` check
the two image arguments under the respective decomposition and independence
consequences of the extended basis; the prose derives these hypotheses. The
rank–nullity checkpoint alone is not cited as support for these intermediate
claims.

The first isomorphism proof constructs the quotient-to-range map, checks
representative independence and linearity, proves injectivity and surjectivity,
and describes inverse classes and their independence of preimage choice. Its
definition uses Mathlib's quotient-lift construction, with separate local
evaluation, inverse-class, bijectivity, well-definedness, and linear-operation
helpers. It does not assume a canonical complement.

The running map is consistently `T(x,y,z)=(x+z,y+z)` using Chapter 2 imports.
Kernel span of `(-1,-1,1)` is reused from Chapter 2. The range is the full
real plane with preimage `(p,q,0)`. The exercise basis certificate includes
both kernel and image independence and image spanning. The first-isomorphism
example gives inverse classes and the chosen `z=0` representative; restriction
to that plane is proved injective.

For the cumulative triangular family, the chapter defines the first-coordinate
line, computes kernel and range when alpha is nonzero, calculates square and
all higher powers, constructs its invariant restriction and proves that it is
zero, and separately checks kernel/range at alpha zero. The general exercise
on restriction and powers is a genuine induction. All terms needed for these
calculations are introduced locally.

Exactly eight required level-two headings are retained. Motivation is nested,
historical context is explicitly labeled without attributing unsupported
history, and the four ML fields are explicit. All display matrices use
`bmatrix`; math uses dollar delimiters. Each indexed card supplies its statement,
proof (construction for the definition), hypotheses, boundary, named provider,
and local Lean source link. Mathlib basis and isomorphism links point to official
GitHub sources pinned at `520045ab14e26149ee970e2e617ca04b09bde5d6`, not `.lake`.

## Exercise prompt changes

All six theorem names are under `CrouzeixTextbook.Part01.Exercises.Chapter03`.
Each written prompt and solution matches the full checked statement, rather
than restating an indexed checkpoint.

| Exercise | Previous prompt | New checked task |
| --- | --- | --- |
| E01 | Interpret a feature projection kernel | For every real triple, zero output iff the triple is `(-z,-z,z)` |
| E02 | Rank/kernel of `[[1,2],[2,4]]` | Running kernel span and full range plus explicit independent spanning certificates for both bases |
| E03 | General injectivity/kernel equivalence | Inverse of the quotient-to-range equivalence at `Tv` equals both `[v]` and its `z=0` representative class |
| E04 | General composition range inclusion | Equal running-map outputs for two triples with third coordinate zero imply equality of triples |
| E05 | Example of strict composition rank inequality | Strict subspace range inclusion for zero followed by identity on the real line |
| E06 | Reapply range inclusion in Lean | For every natural power, ambient evaluation of the invariant restriction power agrees with the ambient map power, proved by induction |

## Publication recommendations

All names in this table have prefix `CrouzeixTextbook.Part01.` and reside in
`formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean`.

| Card | Retained declaration | Source proof body / support | Scope |
| --- | --- | --- | --- |
| CFT-03-001 | `mem_kernel_iff` | Retained Mathlib `LinearMap.mem_ker` alias; local `kernel_range_closure` | Semiring and semimodule; no finite dimension |
| CFT-03-002 | `mem_range_iff` | Retained Mathlib `LinearMap.mem_range` alias; local `kernel_range_closure` | Semiring and semimodule; no finite dimension |
| CFT-03-003 | `rank_nullity` | Retained `LinearMap.finrank_range_add_finrank_ker` alias; distinct basis-extension and image-family support | Division ring and modules; finite domain |
| CFT-03-004 | `injective_iff_kernel_bottom` | Local proof by equal outputs, subtraction, and zero kernel | Division ring and additive groups; no finite dimension |
| CFT-03-005 | `invariantRestriction` | Retained definition constructing subtype output and linearity fields; separate evaluation theorem | Semiring and semimodule; explicit invariant submodule |
| CFT-03-006 | `composition_range_le` | Retained `LinearMap.range_comp_le_range` alias; strict example in E05 | Semiring and semimodule; no finite dimension |

The intended exact correspondences are for the six indexed statements, with
the real/complex specialization and general Lean hypotheses stated explicitly.
The publisher owns formal-mode/provider classification: preserved bare aliases
must be classified honestly rather than advertised as new local proofs.
Definition 005 must not become a theorem. No provider lists, expression hashes,
axiom sets, declaration positions, or receipt data were fabricated here.

## Verification

Preflight inspected pinned `leanprover/lean4:v4.32.1`, the worktree `.lake`
symlink to the canonical primary-checkout cache, the required existing basis
artifact, and mise trust for both primary checkout and worktree.

The TDD skill was used at the explicitly requested exercise-declaration seam.
Before implementation, six retained `#check` clients were appended. The direct
compile exited 1 with exactly six unknown identifiers, one for each missing
exercise declaration. This is the observed RED state.

During implementation the coordinator detected another worktree replacing
shared Harp build outputs. Dependent compiles were paused; no other task's
process was modified. The coordinator built frozen Chapters 1–2 into the private
overlay `/tmp/harp-foundations-olean.ajHyrU`. Chapter 3 compiles use that overlay
first, then warm package artifacts, and do not emit shared build output.

The direct command, from the worktree's `formalization/lean`, is:

```sh
chapter_lean_path=/tmp/harp-foundations-olean.ajHyrU
for chapter_pkg in .lake/packages/*/.lake/build/lib/lean; do
  chapter_lean_path="$chapter_lean_path:$chapter_pkg"
done
LEAN_PATH="$chapter_lean_path" \
  /Users/bytedance/.elan/toolchains/leanprover--lean4---v4.32.1/bin/lean \
  CrouzeixTextbook/Part01/Chapter03.lean
```

The first implementation compile exited 1 at the concrete division witness
`alpha * (p / alpha) = p`; all six exercise declarations printed their types
successfully. The next compile reduced the witness to a residual zero-arithmetic
goal and exited 1; a final simplification closed that goal. The final narrow
compile exited 0 with no warnings or errors, printed all six distinct exercise
types, and checked every helper and indexed declaration. It also checks the
explicit alpha-zero boundary and additional first-isomorphism operation helpers.

`git diff --check` exited 0 after the source freeze. A heading check confirmed
all eight required level-two headings. Source review confirmed the retained
card/exercise anchors, all six exercise provider references, and line-qualified
links to the six current public declaration positions. The source hashes above
were captured after the final Lean and prose edits.

Execution is an ordinary local, non-hermetic compile against warm dependencies.
No Lake command, dependency update, hydration, download, root textbook compile,
or shared `.olean` write was performed by this subtask.

## Acceptance boundary

The first specification review identified an omitted explicit calculation:
the kernel and range of the zeroth and higher powers of `A_{0,alpha}`.
The bounded repair adds `nilpotent_power_zero_kernel_range` (kernel bottom,
range top at power zero) and `nilpotent_higher_power_kernel_range` (kernel top,
range bottom at every power `n+2`). Both quantify over all real alpha, including
zero. The prose now states and explains both calculations explicitly, retains
the power-one alpha case split, and links to the new Lean support at line 187.

The first repair compile exited 1 because simplification did not identify the
endomorphism unit with `LinearMap.id`. Making that definitional identity explicit
resolved it. The subsequent direct Chapter 3 compile against the private overlay
exited 0 with no warnings/errors and all six unchanged exercise types printed.
`git diff --check` also exited 0. The updated hashes above freeze this repaired
candidate for specification re-review; only the three owned files changed.

Independent specification/quality review, exercise fingerprint publication,
shared coverage and exporter classification, corpus/PDF publication, `lean-all`,
and the full repository gate belong to the coordinating task. This record does
not claim those checks have completed.
No remaining mathematical or local Lean compile defect is known in this frozen
candidate. The outstanding concern is the acceptance boundary above, including
honest classification of the preserved aliases and definition.

## Final editorial integration

The integrated contract initially rejected the noncanonical bold label
`Statement (definition)` and the implicit reference to Chapter 2's matrix
family. The label now reads `Statement`, with `(Definition.)` in its body;
the family reference explicitly names `A_{\lambda,\alpha}`. No mathematical
statement or Lean source changed. All nine scoped contract tests then passed.

The same independent quality reviewer reversed exactly those two substitutions
in memory and recovered the previously reviewed prose hash. Quality acceptance
is preserved for final prose SHA-256
`ce2cd2a8bf19b64d6bd6b7763f73914b039d0960661f11a64ba2bf1ca4515a83`.
Lean remains `eeb2edcc68c823e9689d1e2f2eb4800d982d2d7503f88c8fbfc164be8572561e`.
