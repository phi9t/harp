# Textbook completion implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Use subagent-driven development only when delegation is authorized. Steps use checkbox syntax for tracking.

**Goal:** Complete the existing 36-chapter Harp textbook as a readable mathematical development with honest, compiler-backed Lean correspondence, then deliver matching website and PDF editions.

**Architecture:** Canonical prose stays in registered knowledge packets; structured coverage and exercises stay in their existing contracts. Compiler receipts establish formal declarations, separate semantic review establishes correspondence, and generated editions project the same accepted source revision.

**Tech Stack:** Markdown, Lean 4/Mathlib, Rust corpus and publication validators, Atlas, maintained Node/Python PDF tooling.

---

## Status and scope

Specification prepared 2026-09-08 against local master `2c44de8b`. This is a detailed remaining-work specification and sequencing plan, not a claim that every future theorem already has an implementation-ready Lean signature. Before each chapter is implemented, freeze its exact target packet against the then-current source and Mathlib APIs. Do not invent declarations in advance or turn this roadmap into unattended permission for new mathematical claims.

This turn writes only this plan. No new compilation, full verification, source-evidence refresh, commit, push, or deployment is claimed.

The primary checkout contains unrelated untracked paths. Leave them untouched. This document lives in the isolated `codex/textbook-remaining-spec` worktree.

## Session recap and evidence

The approved September 7 package had three deliverables: theorem assurance, a finite-horizon remainder, and a reviewed foundations core. All three have local acceptance records.

- `4a72888f` added the Harp statement audit, scalar remainder, one-witness operator application, two registered supplements, and publication/presentation integration.
- `50324fcd` completed the Chapters 1–4 core, distinct exercise solutions, exact correspondence, narrative validation repairs, compiler-backed metadata, reviewed website/PDF outputs, and shared Lean-output coordination guidance.
- Another session added `a966223d` and merged the branch into local master at `f6c539ec`. On inspection today, no merge remains in progress. Local master is 14 commits ahead of the locally recorded origin/master; this is not a fresh remote check.

Accepted session records:

- `docs/workstream/harp-mathematics/theorem-assurance-review.md`
- `docs/workstream/harp-mathematics/remainder-review.md`
- `docs/workstream/harp-mathematics/foundations-review.md`
- `docs/workstream/harp-mathematics/chapter01-review.md` through `chapter04-review.md`
- `docs/workstream/harp-mathematics/narrative-validator-review.md`

The final foundations gate recorded aggregate exit 0, 8831 Lean build jobs, 137 Atlas tests, 296 textbook tests, and independent source/integration review. Two existing conditional proof-reproduction skips were explicitly recorded. These are historical candidate-specific results, not a new verification of today's master.

The session built a 487-page full book, 19-page Chapter 1 edition, and 48-page Harp edition. All artifact checks passed and 13 pages were visually sampled. Preserve their PDF, HTML, manifest, and 145 linked source snapshots before deleting the older mathematics worktree. These artifacts are not automatically the latest master edition.

The Jin, Lorist–Schwenninger, and Harp terminal sources were unchanged by the foundations package. Selected immutable evidence was refreshed during the assurance package. Local compilation and axiom inspection support the maintained formal declarations in the pinned environment. They do not establish external human review, historical priority, or unrestricted correctness of surrounding prose.

## Baseline and completion accounting

The checked-in status surface reports:

| Metric | Accepted snapshot | Remaining |
| --- | ---: | ---: |
| Chapters | 36 active | Active does not mean complete |
| Indexed cards | 216 | Preserve every existing identity |
| Exact correspondence | 96 | 120 checkpoint or unmapped |
| Reconstructible proofs | 89 | 125 summaries |
| Proof not applicable | 2 | Definitions, not missing proofs |
| Distinct solved exercises | 96 | 120 unresolved |

These dimensions differ: an exactly represented theorem can still have only summary prose. Six cards have formal mode definition, but only two exposition rows currently use not-applicable. Audit that distinction; do not infer a denominator from labels alone.

Chapters 1–4 have 24 exact correspondences and 24 distinct solutions. Four explicit previews remain: characteristic polynomial in Chapter 1, and characteristic polynomial/determinant/trace in Chapter 4. Their full development belongs in Chapters 5–6. Existing solved exercise chapters are 1–4 and 25–36. The 120 unsolved exercises are in Chapters 5–24.

## Common chapter acceptance contract

Every chapter packet must deliver all of the following before it is marked complete:

1. A dependency list whose prerequisites are actually taught earlier, or an explicit optional branch. No circular teaching dependency.
2. Definitions, precise theorem statements, a proof roadmap, full algebra/analysis, and an explanation of where each assumption enters.
3. A worked example and an assumption-boundary example. Examples are newly authored, not reconstructed copyrighted passages.
4. Separate labels for motivation, historical context, ML analogy, mathematical theorem, and computational experiment.
5. Square-bracket matrices using `bmatrix`, explicit column-vector convention, dimensions, scalar field, and a consistent complex-inner-product convention.
6. For every mathematical claim associated with Lean: public declaration, substantive provider, source link, exact hypotheses, specialization map, proof mode, and compiler-derived metadata. Support lemmas link to compiled sources even when unindexed.
7. Distinct exercise statements and genuine solutions. Reusing Mathlib is allowed and labeled; reusing the exercise's checkpoint as its solution is not acceptance.
8. **Provider-proof fidelity.** Every displayed proof must narrate a derivation that is actually checked, and the prose must name which declaration checks it. Read the provider's proof term before writing the paragraph. Three cases, each with a required disclosure:
   - The provider runs the argument the book wants to teach. Say so, and name the library results it composes.
   - The provider runs a *different* argument. Either compile the book's argument locally and cite that declaration, or narrate the provider's argument instead. Do not describe one proof and cite another.
   - The provider is `rfl`, a single instantiation, or one library application. Say that plainly. An instantiation dressed up as an argument is the failure this obligation exists to catch.
   A displayed step that no declaration checks must be labeled uncompiled. A locally compiled restatement that leans on the very result it re-derives is a reformulation, not an independent proof, and must say so.
9. Independent mathematical and correspondence review, recorded findings and repairs, narrow compilation, and presentation checks. See *Review authority* below: this obligation is not satisfiable by the implementer alone.

Do not seek 100% Lean coverage by pretending a motivation paragraph or historical claim is a theorem. Every formalizable mathematical obligation must be represented or explicitly remain open. Definitions need an exact formal counterpart, not a fake theorem proof.

### Review authority

Obligation 9 requires a reviewer who is not the implementer. An implementer working without delegation authority cannot discharge it, and must not claim to.

When delegation is unavailable, the implementer performs an author self-review, records it as such under `docs/workstream/harp-mathematics/chapterNN-review.md`, and leaves the chapter's Kata issue **open** with the reason stated. Do not close a chapter issue on self-review. Do not silently downgrade the criterion.

The deferred reviews are then a single explicit obligation of the package they belong to, discharged in one pass over the chapters that accumulated. A package is not complete while any of its chapters carries a deferred review, and the count of deferred reviews is part of the package's exit report.

**Obligation 8 is not self-applicable.** The Chapters 5–10 independent pass, recorded in [the Chapters 5–10 independent review](../../workstream/harp-mathematics/chapters05-10-independent-review.md), found five blocking defects and about thirty smaller ones in six chapters whose author self-reviews had passed. Every correspondence finding in that pass was an obligation 8 violation — a displayed argument cited to a provider that runs a different one — found by a reviewer and missed by the author who had applied obligation 8 to the same chapters days earlier. The failure mode is structural, not incidental: a self-review re-checks the author's claims against the author's own reading of the providers, so a misreading survives both passes. Two of the defects the reviewers themselves missed were found only by a third reading against a different question (a forward reference to a chapter that does not exist, and an opening claim contradicted by an earlier card).

Two consequences for the remaining chapters:

- Do not report a chapter as meeting obligation 8 on the strength of a self-review. Report it as *self-reviewed, obligation 8 unverified*, and say so in the chapter's exit line.
- Run the independent pass over a package while its chapters are still cheap to change. Four of the five blocking defects above had already been committed and pushed, and one was a false mathematical claim in reader-facing prose. The cost of the delay was not rework; it was the interval during which the book asserted something untrue.

### Two defect classes the reviews keep finding, and the checks that catch them

Five independent passes have now run: Chapters 5–10, then 11, 12, 13, 14, with
five, five, three, six and one blocking defect respectively. Two root causes
recur often enough to be worth naming and checking mechanically, because both are
invisible to the author and cheap for anyone else to spot.

**Stale self-reference.** The most common root cause by a wide margin. The
implementer changes a declaration, a mode, a count or a registry row, and leaves
prose or metadata describing the previous state. Chapter 11 said "the six items
were re-exports" when five were; Chapter 12 said all six cards were
`reexported-proof` in three separate sentences after one had been changed to
`proved-here`; Chapter 14 shipped `skills` and `pedagogical_prerequisites` rows
byte-identical to the withdrawn sketch while the prose was written fresh; and the
count projector published a four-part sum of 213 out of 216 rows because the word
"six" was hardcoded where a derived count belonged.

*The check.* Before dispatching reviewers, diff every file the chapter touches
against the commit the chapter started from, and for each changed contract row or
declaration, re-read every prose sentence that describes it. Registry rows are
the highest-risk item because nothing in the prose forces them to move: verify
explicitly that this chapter's `pedagogical_prerequisites` and `skills` rows were
edited in the same change as the prose, or that they were deliberately left alone.

**Asserted library semantics.** The second class: stating what a Mathlib
declaration says, or how it is proved, from its name rather than its source.
`AnalyticOnNhd` was described as stronger than pointwise analyticity when it is
*defined* as pointwise analyticity; `le_radius_of_bound` was described twice
without the geometric factor its statement carries; "a supremum bounds its set
unconditionally" ignored that Mathlib returns a junk value on an unbounded set,
which made a card false rather than merely imprecise; and `HasFDerivAt.unique`
was claimed to run an argument it does not.

*The check.* Do not describe a library declaration's statement, hypotheses or
proof without opening its source. A cited name is not evidence; the file is.

**Unresolved cross-references** are a cheap third case, and have now occurred
twice: Chapter 12 forward-referenced orthogonalization to a chapter that does not
develop it, and Chapter 15 was written with a `Next:` link to a filename that does
not exist. `harp build` catches a broken wikilink target, but only after the prose
is otherwise complete, and it says nothing about a link that resolves to the wrong
chapter. Resolve every `[[...]]` target and every "Chapter N does X" claim against
the index before dispatching reviewers.

Neither check requires a reviewer. Both are mechanical, and both would have
removed roughly half the blocking findings of the last three passes.

**Run `python3 scripts/check_chapter_prose.py NN` before dispatching reviewers.**
Writing these down as prose was not enough: Chapter 15 then shipped a `Next:` link
to a filename that does not exist and a forward-dependency claim, asserted three
times, that named the wrong Part — both instances of checks recorded in this very
section and not applied. The script makes the mechanical half mechanical. It fails
on unresolved cross-references, lists registry rows left untouched while the prose
was rewritten, and prints every sentence naming a formal mode beside the registry's
actual tally.

What it deliberately does not do is judge. A first version compared "Chapter NN
does X" claims against the index title by word overlap and produced sixteen false
positives on Chapter 15 while catching none of its real defects; a second tried to
parse quantities out of mode sentences and missed the exact phrasing that was
Chapter 12's blocking defect. Both were removed. A check that cries wolf trains its
reader to skip it, which is worse than no check, and the same failure as a
regression test written adjacent to a bug rather than on it.

Reviewers must be barred from Lean builds — the shared Lake cache corrupts under concurrent builds — and from `docs/workstream/harp-mathematics/`, so they cannot anchor on the self-reviews they are meant to check.

### Known structural walls

These are correct behaviors of the verification machinery, not defects. Each has cost a build cycle when discovered late; check for them before writing a card.

- **Unmaintained namespaces.** The receipt exporter treats only the `CrouzeixTextbook.`, `CrouzeixConjecture.`, `Crouzeix.Jin.`, `CrouzeixJin.`, `Crouzeix.LoristSchwenninger.`, `CrouzeixLoristSchwenninger.`, `Crouzeix.Harp.` and `CrouzeixHarp.` prefixes as maintained. A card aliasing into any other namespace — `MathematicalFoundations.` in particular — cannot name its provider as an underlying declaration, so the validator cannot check the alias target and the row can only ever be a checkpoint. Such cards must be proved locally. Hit in Chapters 7, 8 and 9.
- **Wrapper fixtures track the imports.** The shared-wrapper tests in `crates/harp/tests/lean_library.rs` write a fake Mathlib artifact tree, and the cache preflight refuses to build a target whose scanned imports are missing from it. A chapter that adds a Mathlib import must add it to `write_fake_all_mathlib_artifacts` in the same change, or three wrapper tests fail closed. Hit at Chapter 5 and again at Chapter 10.
- **Eta-alias rejection.** A `proved-here` card whose body elaborates to an exact eta alias of another constant is classified `direct-alias` by the receipt and rejected. This is the protection that stops a renamed import from claiming to be a local proof. Give the declaration a genuine derivation — a named intermediate step, or the argument the prose displays. Hit twice while reproving cards that had been aliases.
- **The payload digest is the last commit before the gate.** `repository verify` recomputes the standalone payload digest over tracked repository content, so *any* commit that adds or changes a tracked file re-stales the pin — including a commit that has nothing to do with the textbook. Refreshing the receipt and then committing anything else puts the tree straight back into the state the refresh just fixed. Refresh it once, as the final commit, and run the gate without committing again. Use `scripts/refresh_import_receipt.py`: it refuses to edit unless the verifier really reports a stale digest, which is the check whose absence let a refresh run against an unrelated failure and write an empty pin into the receipt.
- **Frozen duplicate providers.** Two indexed cards may rest on the same provider and therefore index one theorem twice. Card identities are frozen, so the roster is not adjusted; the later card's boundary paragraph must disclose the duplication and give the reason the identity is indexed again. The roster currently indexes 216 cards resting on 208 distinct proofs; `mise run textbook-counts` with `--audit` lists the eight that restate another.
- **The identity key is the provider, never the type fingerprint.** `type_sha256` hashes a pretty-printed type, and it is unsound as an identity key in both directions at once.
  - *False negatives.* The hash carries universe metavariable names, so two aliases of one theorem differ whenever those names differ. `CFT-06-001` and `CFT-19-006` alias the same provider and hash differently for exactly this reason; the receipt validator accepts them because it compares alpha-equivalence, not the hash. A fingerprint-keyed audit misses them.
  - *False positives.* Two genuinely independent proofs of the same statement necessarily share a type. `CFT-32-003` with `CFT-35-003`, and `CFT-35-005` with `CFT-36-006`, are the Jin, Lorist–Schwenninger and Harp routes reaching the same theorem — the book's central independence claim. A fingerprint-keyed audit reports the independence story as duplication, which is exactly backwards.
  Any duplicate reasoning, automated or by hand, must group by `underlying_declaration` where present and by the declaration name otherwise.

## Package 0: reconcile inventory and ownership

**Owners:** `knowledge/crouzeix_textbook/status_and_scope.md`, `content/crouzeix_textbook/coverage.json`, `content/crouzeix_textbook/exercises.json`, existing correspondence/publication generator, Kata issues.

- [ ] Recompute current counts from the actual records; compare with the status document and accepted receipt.
- [ ] List every remaining card and exercise by stable ID, chapter, missing obligation, provider, prerequisite, and disposition.
- [ ] Classify previews separately from missing proof bodies and missing correspondence.
- [ ] Search Kata and attach work to existing issues. Keep whole-book `00zs` open. Reconcile `8mq3`, `m4qy`, and `q8h9` against accepted core work; do not reopen completed Chapters 2–3 without a new defect.
- [ ] Reconcile the separately landed Cauchy–Schwarz controls and laboratory design with Chapter 7 work. They are not equivalent to a completed chapter or production verifier.

**Exit:** one exact backlog with no dropped IDs, duplicate ownership, or false completion claims. No numerical percentage of the Lax book until its source-item denominator is inventoried.

## Package 1: finish algebra, Chapters 5–6

**Prose:** `knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra.md` and `06_eigenvalues_and_polynomial_algebra.md`.

**Lean:** `formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean` and `Chapter06.lean`.

**Chapter 5 obligations:** develop alternating multilinearity and determinant normalization, determinant behavior under composition/change of basis, invertibility criterion, trace invariance, and the relation between determinant and top-degree exterior action. Separate geometric interpretation from proof. Explain why rectangular matrices have no determinant in this sense. Use explicit small matrices to connect invariant maps to array calculations.

**Chapter 6 obligations:** develop eigenvectors/eigenspaces, characteristic versus minimal polynomial, Cayley–Hamilton and annihilating polynomials, diagonalizability criteria, and generalized eigenspaces where required by the later reading path. State field-splitting assumptions. Include real rotation and a nontrivial Jordan block as boundary examples; do not imply every matrix is diagonalizable.

- [ ] Freeze the twelve existing card targets and twelve exercises; enumerate additional support lemmas without inflating indexed counts.
- [ ] Review precise Lean targets before proof implementation.
- [ ] Complete Chapter 5 and review it before dependent Chapter 6 results.
- [ ] Add twelve distinct checked solutions and full proof exposition.
- [ ] Resolve the four early previews by an explicit editorial rule: either add their local proofs or label them as forward-reference summaries outside the completed-proof count. Link each to its full derivation and retain the original anchor.

**Exit:** all twelve chapter rows exactly represented, all twelve exercises solved, no unresolved algebra prerequisites for subsequent chapters. If no unrelated counts change, solved exercises rise from 96 to 108.

## Package 2: geometry and calculus, Chapters 7–12

**Owners:** existing Markdown in `part_02_geometry_and_calculus`, corresponding `CrouzeixTextbook/Part02` modules, shared contracts and chapter tests.

| Chapter | Required development | Boundary or ML bridge |
| --- | --- | --- |
| 7 | Inner products, Cauchy–Schwarz with equality cases, orthogonality, projection | Complex conjugation; zero vector; Gram/feature geometry |
| 8 | Self-adjointness, positive semidefiniteness, Gram matrices, kernels | PSD versus positive definite versus entrywise positivity |
| 9 | Induced norm, singular values, SVD, spectral versus norm control | Rank deficiency; nonnormal amplification; rectangular shapes |
| 10 | Multilinearity, tensor products, contractions, basis dependence | Tensor product versus array reshape; shape-checked contraction |
| 11 | Fréchet derivative, chain rule, adjoint/pullback interpretation | Jacobian layout; reverse accumulation; nonsmooth points |
| 12 | Alternating forms, orientation, integration and required Stokes interface | Sign conventions and hypotheses; explicit optional reading branch |

- [ ] Reuse the Cauchy–Schwarz trusted controls where signatures match, retaining attribution and proof-mode labels.
- [ ] Treat proof reconstruction and library reexport as distinct teaching choices.
- [ ] Create exact packets one chapter at a time, with six distinct exercise solutions each.
- [ ] Keep Chapter 12's analytic/geometric prerequisites explicit. Do not hide a full manifold theory inside an introductory argument or replace its claim by an easier finite identity.

**Exit:** 36 more distinct solutions, complete proofs along the declared route, reviewed tensor/derivative notation. Cumulative target after Package 1 is 144 solved exercises.

## Package 3: analysis and functional calculus, Chapters 13–18

**Owners:** existing `part_03_analysis_and_complex_functions` prose and corresponding Part03 Lean modules.

- [ ] Chapter 13: metric/normed completeness, continuity, compactness assumptions, and finite-dimensional norm equivalence.
- [ ] Chapter 14: operator-series convergence, Neumann series, and justified passage of limits through bounded operations.
- [ ] Chapters 15–16: complex differentiability and the Cauchy-theory results actually consumed downstream. State domains, contours, regularity, and interchange hypotheses explicitly.
- [ ] Chapter 17: polynomial/rational/holomorphic functional calculus interfaces, resolvent assumptions, and pole restrictions. Explain where constructions agree.
- [ ] Chapter 18: positive-real analytic functions and exactly the positivity/representation machinery used later.
- [ ] Supply six exact card correspondences and six distinct solutions per chapter, with support lemmas for each formerly omitted analytic step.

**Exit:** no unexplained convergence or integral interchange in the main route; 36 more solutions, cumulative target 180. Any imported analysis theorem outside the promised self-contained scope must have a named prerequisite note, not an invisible dependency.

## Package 4: operator theory, Chapters 19–24

**Owners:** existing `part_04_operator_theory` prose and corresponding Part04 Lean modules.

- [ ] Chapter 19: normal versus nonnormal operators; transient amplification separated from asymptotic spectral behavior.
- [ ] Chapter 20: numerical range definitions, geometric results, and finite-dimensional hypotheses.
- [ ] Chapter 21: spectral-set bounds with the function class and norm explicit.
- [ ] Chapter 22: positive and completely positive maps, keeping scalar and matrix-level claims distinct.
- [ ] Chapter 23: compression/dilation identities; distinguish first-moment compression from a full power family.
- [ ] Chapter 24: Gramians and operator order; justify every order manipulation without assuming commutativity.
- [ ] Complete 36 distinct exercises and review all associated formal statements.

**Exit:** cumulative target 216 solved indexed exercises if scope remains unchanged. This milestone alone does not establish that all 216 cards have complete prose or exact correspondence.

## Package 5: audit the terminal teaching routes

**Owners:** `part_05_crouzeix_machinery`, `part_06_constant_two_routes`, `theorem_dependency_map.md`, `harp_mathematical_audit.md`, `harp_finite_horizon_remainder.md`, and existing formal providers.

- [ ] Re-review Chapters 25–36 against the now-expanded prerequisites rather than rewriting their already accepted proofs wholesale.
- [ ] Check each completion kernel, cancellation identity, perturbation inequality, normalization, and limit passage can be reconstructed from the text.
- [ ] Keep Jin and LS independent after the shared Chapter 29 machinery; represent Harp's actual shared imports without inventing a fully disjoint route.
- [ ] Distinguish mathematical prerequisite edges, Lean imports, and source-attribution edges. Test the intended independence and reject cycles.
- [ ] For Harp, audit positive weights/mass, embeddings/moments, fixed core versus horizon witnesses, recurrence signs, square completion, limits, and transported scope.
- [ ] Preserve theorem/evidence provenance and current terminal statements. Repair only demonstrated defects; a stronger theorem needs a separate approved packet.

**Exit:** every main-route proof step has a taught prerequisite or a linked compiled support lemma; no historical attribution inferred from an import or a successful build.

## Package 6: mathematical notation and ML teaching thread

**Owners:** `notation_and_glossary.md`, `reading_guide.md`, all touched chapters, actual equation-rendering tests.

- [ ] Use a running small nonnormal matrix family through basis changes, spectra, Gram geometry, singular values, numerical range, and powers. Separate exact results from plotted observations.
- [ ] At selected chapter boundaries give an array correspondence with shape, dtype/scalar field, contraction axes, transpose versus conjugate transpose, and the mathematical operation represented.
- [ ] Label PyTorch/JAX examples as computational illustrations unless separate semantics and numerical correctness have been proved. Verify current API details from authoritative documentation when implementing them.
- [ ] Distinguish a real-linear derivative from complex differentiation and a cotangent pullback from an assumed array-layout convention.
- [ ] Explain where linear-operator results fail to transfer to nonlinear, time-varying, or stochastic training systems.
- [ ] Cite historical context using checked sources and newly authored prose. Keep the mathematical reading path useful without requiring familiarity with ML jargon.

**Exit:** a reader can move between abstract maps and array notation without an unstated basis, shape, adjoint, or arithmetic assumption.

## Package 7: verifier assurance, separate from chapter writing

Reuse `2026-09-07-textbook-verifier-contracts.md` and the existing formalization-wave plan. Do not create a second coverage authority or duplicate the publisher.

- [ ] Reconcile fixed targets, dependency policies, source interpretation, compiled acceptance, and semantic review as separate states.
- [ ] Qualify strict record parsing and protected targets before running untrusted candidates.
- [ ] Test confinement, whole-worker termination, sealing, symlink/traversal rejection, and protection of challenge/receipt/cache data.
- [ ] Qualify real statement comparison, definition closure, transitive axiom/dependency checks, stale receipt rejection, and clean replay against the Q01–Q20 adversarial contracts.
- [ ] Resolve the toolchain qualification concern recorded in the existing wave plan through a separate reviewed environment packet. This specification neither independently verifies that advisory nor authorizes a toolchain/cache migration.

**Exit:** production-verifier claims require the actual qualification suite, not the existing trusted Cauchy–Schwarz control alone. Ordinary reviewed textbook authoring may proceed with honestly labeled local compilation; arbitrary candidate execution cannot borrow that trust.

## Package 8: integrated editions and release

**Owners:** `atlas/src/content/generated/corpus.json`, `atlas/dist/harp-atlas.html` and receipt, existing reader tests, `tools/textbook_pdf/`, status/claim ledgers, import receipt.

- [ ] Freeze a candidate and generate compiler metadata through the maintained exporter/publisher. Never hand-edit type hashes or line positions.
- [ ] Regenerate corpus and offline reader together; validate the exact bound inputs before deciding whether terminal evidence needs an immutable refresh.
- [ ] Build full-book and existing standalone editions with matching source manifests; preserve linked Lean snapshots.
- [ ] Test browser navigation, anchors, source links, mathematics, keyboard access, and mobile overflow on actual exported pages.
- [ ] Inspect PDF equation-heavy pages, page breaks, contents, fonts, grayscale contrast, and source links. Automated geometry checks supplement, not replace, visual review.
- [ ] Run the full repository gate on the frozen candidate and refresh the import receipt only after tracked payloads settle.
- [ ] Obtain explicit publication authority before pushing/deploying; then check the hosted revision and download artifacts rather than equating local build success with publication.

**Exit:** one accepted revision binds formal records, prose, website, and PDFs; remaining scope is visible to readers.

## Per-packet execution and checks

1. Inspect current branch, ownership, canonical contracts, and exact provider signatures. Freeze statements and dependencies before proofs.
2. Establish the red signal before implementing. For a chapter packet this is *not* an added failing test: the contract tests pin counts, so writing the new count first is bookkeeping, not a behavioral signal. The genuine fail-closed check is `harp crouzeix-textbook check` against a fresh receipt, which rejects every unregistered card, every unproved exercise, every stale locator and every mode/status combination the contract forbids. Run it, read what it refuses, and let that list drive the work. Add a genuinely new behavioral, correspondence or rendering test only where the obligation is not already covered by that validator. Do not add tautological source-string tests as mathematical assurance.
3. Implement the chapter's prose, Lean targets, and distinct exercise proofs together.
4. Compile the narrow target after checking the pinned toolchain and warm-cache state without Lake. Use private Harp output overlays for concurrent iteration; serialize shared-output builds through receipt extraction.
5. Review mathematical correctness and prose-to-Lean correspondence separately; repair and recheck findings before dependent work.
6. Regenerate official metadata and reader projections. Run focused Rust tests, Atlas tests, and PDF tests applicable to the slice. Run the Rust suites so that an early failure does not mask a later one — `cargo test` stops at the first failing binary by default, so pass `--no-fail-fast` or invoke each suite separately. A Chapter 10 run aborted at `foundations_contract` and never reached `lean_library`, whose failure then surfaced twenty-five minutes later in the full gate.
   The contract counts are **derived data and must be generated, not hand-edited.** They currently appear by hand in `status_and_scope.md`, `claim_evidence_ledger.md`, chapters 33 and 35, the backlog inventory, and as pinned literals in `crouzeix_textbook.rs` and `foundations_contract.rs` — roughly twenty-five edit sites per chapter, every one of them computable from `coverage.json`, `exercises.json` and the compiled receipt. Until a generator exists this is the largest mechanical cost per chapter and the likeliest source of silent drift. Building it is a prerequisite of the next package, not an optional cleanup. Tests that *compute* the expected counts are preferable to tests that pin them; a pinned literal detects drift but also has to be edited every time, which is the cost being removed.
7. Freeze, run `mise run verify`, inspect the diff, update records, and stage explicit path groups for a coherent commit. Preserve unrelated work.

Existing focused commands include:

```sh
cargo test -p harp --test foundations_contract
cargo test -p harp --test foundations_narrative
cargo test -p harp --test crouzeix_textbook
cd atlas && corepack pnpm run test
node --test tools/textbook_pdf/*.test.cjs
mise run verify
```

Run commands from their documented working directories with the required existing runtimes. The foundations-only suites do not prove coverage of later chapters; extend the maintained chapter contracts in the same accepted slice. Discover PDF dependencies through the supported runtime tool before building. No dependency hydration, daemon setup, or shared-cache race is an incidental implementation step.

## Execution notes

Recorded from implementing Package 0 and Chapters 5 through 9 against this plan. These are observations about the plan, not about the mathematics.

**The obligation that found the most defects was missing.** Acceptance obligation 8 above was added after the fact. Statement-level correspondence review — does the prose state the Lean type — passed on work whose *proofs* described arguments the providers do not run. Reading provider proof terms found a card describing a discriminant argument where the proof uses a resultant, a card whose displayed proof had nothing checking it at all, and a card crediting the wrong provider. All three had already passed statement review. Assume the same is true of chapters not yet audited this way, including ones already marked complete.

**Mechanical cost dominates mathematical cost.** A chapter's mathematics took a fraction of the wall-clock; refreshing derived counts, receipt identities, pinned digests and generated projections took the rest, spread over about twenty-five hand edits in seven files. The full gate then costs roughly twenty-five minutes per attempt, so each mistake in that bookkeeping is expensive. This is why the generator in step 6 is a prerequisite rather than a nicety.

**Deferred reviews accumulate.** Five chapters are implemented and gate-verified with their Kata issues open, each carrying a recorded author self-review and an explicit statement that independent review was not performed. That is the honest state under the current authority, and it is now an explicit obligation of the package rather than an oversight. The count will keep growing until a review pass is authorized.

**Two defects found in already-accepted work.** Package 0's inventory surfaced five prerequisite edges in Chapters 28 and 34 that point forward or sideways, violating acceptance obligation 1. They belong to Package 5 and remain unaddressed. Chapter 9 surfaced the frozen duplicate-provider case described above. Neither was introduced by this program; both were found only because the inventory and the fidelity audit looked.

**The roster indexes more cards than it has proofs.** 216 indexed cards rest on 208 distinct proofs. That figure is now computed and published on the status page, because it was previously stated nowhere and the duplication was being discovered one chapter at a time — Chapter 10 alone restates three Chapter 5 cards, which the chapter now discloses to the reader. One of the eight, `CFT-32-002` and `CFT-32-006`, is inside already-accepted Chapter 32 and carries no disclosure; that belongs to Package 5.

**A derived surface outside the projection went stale within one chapter.** The status page's completed-prefix sentence still read "Chapters 1–9 … 54 correspondences" after Chapter 10 landed, because the projection did not cover it. It does now. The lesson is not that the sentence was missed but that any hand-maintained number adjacent to generated ones will drift on the first chapter that forgets it, so the projection's coverage is worth extending whenever a number is found outside it.

**Environment trap.** Run `mise run verify` from a shell that has not sourced `scripts/harp_xdg_env.sh`. Every Lean task sources it internally, but exporting `HARP_ELAN_HOME` into the parent shell defeats the task-scoped-ELAN tests in `autodiff_geometry_lean.rs` and produces two failures unrelated to the change under test.

## Program boundaries and recommended order

Recommended order: inventory reconciliation; Chapters 5–6; Chapters 7–9; Chapters 10–12 with explicit prerequisite routing; Chapters 13–18; Chapters 19–24; terminal-route re-review; whole-book reconciliation and release. Apply notation, review, and rendering requirements continuously rather than leaving them to a final polish pass.

Keep three completion claims separate:

1. The approved September 7 package is locally accepted and subsequently landed on local master.
2. The existing Harp textbook is complete only after all declared proof, exercise, correspondence, dependency, and edition obligations are accepted.
3. Full coverage of Lax, Spivak, and the Bishop books requires source-edition and item-level inventories from the separate four-book program. Harp's 216 cards are not that denominator.

Possible effective cubature algorithms, computable operator certificates, perturbation/rounding guarantees, matrix-valued extensions, and new universal bounds are research proposals outside this completion package. The accepted remainder is a bound for supplied witness data, not a certified witness-construction algorithm.

No elapsed-time promise is justified by the current inventory. Measure the Chapters 5–6 packet's proof, review, and rendering cost before estimating later waves.
