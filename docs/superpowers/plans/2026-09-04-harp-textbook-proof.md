# Harp textbook proof implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Publish a reconstructible Chapter 36 with six exact Lean correspondences, six distinct exercise solutions, and matching website and PDF editions.

**Architecture:** Canonical Markdown and structured textbook contracts feed the existing Lean correspondence exporter, Rust publisher, Atlas reader, and PDF builder. Reuse Harp's certified providers and disclose their LS support dependencies. Keep the new chapter independent of other terminal providers.

**Tech stack:** Existing pinned Lean/mathlib, Rust, TypeScript, Node PDF tools, and Python verification. No new package or compiler acquisition.

**Approved specification:** `docs/superpowers/specs/2026-09-04-harp-textbook-proof-design.md`.

**Workspace:** `.worktrees/textbook-harp-proof`, branch `codex/textbook-harp-proof`, base `6f26d6e9`, Kata `fq4m`.

## Execution rules

The specification contains the required mathematical content, section by section.
Read it with each task. Do not replace its displayed calculations with endpoint
references. Exact Lean signatures must come from the source and compiler, not
from the abbreviated mathematical notation in this plan.

The only pre-existing change in this worktree is the uncommitted specification.
Do not import work from other feature worktrees. The primary checkout has
unrelated changes that must remain untouched. Use `apply_patch` for edits.
Use `/opt/homebrew/bin` in PATH for Git LFS and the installed mise executable.

Do not commit incomplete intermediate publication states. The chapter, roster,
contracts, and generated outputs form one integrated candidate. Run the required
full gate before committing it, then stage only the reviewed paths. No push or
mainline merge is authorized.

## Task 1: Record the baseline and preflight dependencies

Files read: `AGENTS.md`, `mise.toml`, `formalization/lean/lean-toolchain`,
`formalization/lean/lake-manifest.json`, `scripts/harp_xdg_env.sh`,
`scripts/check_lean_library.sh`, and the approved specification.

- [x] Record branch, clean tracked baseline, and disk capacity:

```sh
export PATH=/opt/homebrew/bin:$PATH
git status --short
git rev-parse HEAD
df -h .
ls -ld ../../formalization/lean/.lake
cat formalization/lean/lean-toolchain
```

- [x] Resolve the primary `.lake` target and verify its ancestry, required
  dependency artifacts, and the pinned executable before linking it. Create
  the feature-worktree `.lake` symlink only if its location is absent. If it
  already exists, inspect rather than replace it. No `lake update` or cache fetch.
- [x] Verify worktree-local mise trust and the configured build-cache paths.
  Run `mise run lean-crouzeix-harp` as the narrow baseline. Record the command,
  result, and whether execution was local or Seatbelt-contained. Never describe
  an ordinary local build as hermetic.
- [x] Search and update existing Kata issue `fq4m`; do not create duplicate work.

## Task 2: Build the chapter's Lean teaching interface

Create: `formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean`.
Read: all nine `formalization/lean/Crouzeix/Harp/*.lean` modules and the LS
norm-attainment, recurrence, scalar, and dilation support actually imported.

- [x] Start the new file with this import boundary:

```lean
import Crouzeix.Harp.Consequences

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture
noncomputable section
```

- [x] Add six public theorem interfaces, with names
  `harp_positive_moment_cubature`, `harp_finite_dilation_exists`,
  `harp_finite_recurrence`, `harp_normalized_norm_two`,
  `harp_polynomial_constant_two`, and `harp_two_spectral_set`.
  Copy complete provider signatures, including instance arguments and universe
  restrictions, from the six providers named in the specification. For the
  dilation existence statement, use `Nonempty (FiniteAtomicL2DilationWitness core N)`
  and construct it from the existing data-valued provider. Classify these
  direct provider aliases as reexports, not original textbook proofs. The
  `Nonempty` wrapper is `proved-here` only as propositional packaging of the
  existing data-valued provider; its provenance must state that distinction.
- [x] Compile the new module before changing the publication roster. Check
  its inferred statements and axiom dependencies with the pinned compiler.
  Correct any mismatch before writing the corresponding theorem card.
- [x] Under `Exercises.Chapter36`, write six new statements and proof bodies
  following the approved exercise requirements. E01 uses `congrArg` to
  evaluate a tuple-valued identity. E02 expands the normalized mass inner
  product. E03 proves finite telescoping. E04 proves the sign-sensitive
  displacement substitution. E05 performs positive rescaling. E06 combines
  the geometric-series limit and scalar inequalities.
- [x] For each exercise, first compile the explicit statement with its intended
  proof attempt, then repair it against the source interfaces. Do not leave
  `sorry` or `admit`. No exercise may call its parent checkpoint or a terminal
  Crouzeix theorem as its solution.
- [x] Close both namespaces and the noncomputable section. Recompile the module.
  Review hypotheses, proof dependencies, and the six distinct exercise types.

## Task 3: Write and review sections 001 and 002

Create: `knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof.md`.

- [x] Add frontmatter using the approved ID, chapter 36, part 6, and canonical
  filename. Use the established chapter template without inventing publication
  metrics. Add previous/index navigation and the prerequisite recap.
- [x] Write CFT-36-001 from the cubature proof. Explain the real observable,
  convex-hull argument, positive weights, mass, and simultaneous moments. State
  the exact dimension-plus-one cardinality bound.
- [x] Write CFT-36-002 from the atomic L2 construction. Display the density
  factorization, normalized embedding, mass identity, contraction, and each
  compressed moment calculation. Track factors of two and adjoint orientation.
- [x] Add the matching E01/E02 statements, Lean solution links, and prose
  solutions. Mark motivation and ML analogies separately.
- [x] Review these sections against the actual Lean statements and bodies.
  Record the review in Kata, including discrepancies and their corrections.

## Task 4: Write and review sections 003 and 004

Modify: the Chapter 36 Markdown and Lean file from Tasks 2–3.

- [x] Define the fixed core and the horizon-dependent witness before using
  either. Define `m_k` and `b_N` with the actual inner-product convention.
- [x] Expand the adjacent-power defect, completed square, and telescoping
  calculation. Retain `kappa^(-N) m_(N+1)` and the exact index ranges.
- [x] Prove `b_N <= C` with one horizon-independent `C`. Explain why replacing
  the negative error term by `-C/(kappa^2-kappa)` is allowed.
- [x] Derive the uniform bound on `m_k`, the disappearing terminal term,
  geometric-series limit, and the contradiction for `kappa > 2`.
  Treat `N=0` and `kappa <= 1` explicitly.
- [x] Add E03/E04 and their full solutions. Review every inequality direction,
  denominator condition, and quantifier. No common finite dilation is asserted.

## Task 5: Write and review sections 005 and 006

Modify: Chapter 36 Markdown; Chapter 35 navigation and comparison pointers.

- [x] Follow `harpFiniteHorizonMainTheorem` line by line. Write both `M=0`
  and `M>0` branches, the fixed-domain simple-spectrum estimate, rescaling,
  the matrix limit, and then the outer-domain limit.
- [x] State the rational and Hilbert-space consequences with all assumptions.
  Explain the finite Krylov compression and refer back to Chapter 35 for its
  full shared adapter proofs. Use the Harp providers in the theorem cards.
- [x] Add E05/E06, complete solutions, and a final synthesis of the proof.
  Distinguish exact existential certificates from numerical diagnostics.
- [x] Review the whole chapter as a reader. Ensure each decisive calculation
  is reconstructible without opening Lean. Keep historical assertions limited
  to established local provenance; consult primary sources before adding any
  further historical claim.

## Task 6: Extend the publication contract with regression tests

Modify:

- `crates/harp/src/crouzeix_textbook/contract.rs`
- `crates/harp/tests/crouzeix_textbook.rs`
- `content/crouzeix_textbook/coverage.json`
- `content/crouzeix_textbook/exercises.json`
- `content/crouzeix_textbook/compatibility_routes.json`
- `crates/harp/src/corpus/mod.rs` and its tests where packet membership is owned
- `formalization/lean/CrouzeixTextbook/Correspondence.lean`, through its owner

- [x] Add chapter-36 acceptance and chapter-37 rejection regressions, plus
  missing-CFT-36 coverage. Preserve existing duplicate-identity and unresolved
  declaration rejection tests and temporary fixtures. Observe the focused
  chapter-boundary regression fail before changing production code; verify
  the complete textbook suite through the full repository gate.
- [x] Extend the contract's chapter range from `1..=35` to `1..=36` and update
  its error message. Keep strict parsing and duplicate/graph checks unchanged.
- [x] Register the new chapter and six theorem/exercise pairs. Populate each
  record from the compiled interfaces, with honest correspondence modes and
  full hypothesis maps. Do not fabricate compiler metadata to make validation pass.
- [x] Add dependency edges from common machinery and the actual shared LS
  support. Do not make Chapter 35 or a Jin terminal a proof prerequisite.
- [x] Update live roster assertions to 36/216/216. Preserve explicitly
  historical fixture expectations. Resolve each matching occurrence separately.
- [x] Regenerate the Lean correspondence through the established contract
  owner, compile it, and run `mise run crouzeix-textbook-publication` to export,
  publish, and check the fresh receipt. Re-run the Rust textbook tests.

## Task 7: Integrate navigation, website, and PDF discovery

Modify: the textbook index, reading guide, status/scope, source registry,
notation, managed ledgers, and `tools/textbook_pdf/{inputs.cjs,inputs.test.cjs,build.cjs,README.md}`.
Regenerate: `atlas/src/content/generated/corpus.json` and `atlas/dist/harp-atlas.html`.

- [x] Extend PDF fixture generation to 36 chapters and add these behavioral
  checks using the existing `fixture` helper:

```javascript
test('discovers Chapter 36 and rejects its absence', t => {
  const {root, dir} = fixture(t);
  assert.equal(discoverChapters(root).length, 36);
  fs.unlinkSync(path.join(dir, '36_chapter.md'));
  assert.throws(() => discoverChapters(root), /36|Missing/);
});
```

- [x] Run `node --test tools/textbook_pdf/inputs.test.cjs` and observe the
  failure against the old discovery range. Update the discovery count,
  contiguous-sequence message, and module range to 36. Add rejection tests
  for a duplicate chapter and out-of-range module request.
- [x] Update the PDF cover's live chapter count and README. Run
  `node --test tools/textbook_pdf/inputs.test.cjs tools/textbook_pdf/build.test.cjs`.
- [x] Extend the builder's edition list, which currently hardcodes only the
  full book and Chapter 1. Preserve both outputs and add
  `chapter-36-harp-finite-horizon-proof` selecting `chapters[35]`. Replace
  Chapter-1-only standalone title/subtitle and PDF metadata branches with
  explicit edition metadata. Test that the new standalone cover, HTML title,
  PDF title, contents, and embedded-source list describe Chapter 36, while
  Chapter 1 retains its existing names. Do not merely send a different list
  to `documentHTML` while retaining the Chapter 1 cover.
- [x] Regenerate maintained prose metrics from the fresh contracts. Add
  Chapter 36 to reading/navigation and preserve every previous route.
- [x] Regenerate the corpus and static export through the documented repository
  tasks. Run `cargo test -p harp corpus::tests --lib -- --test-threads=1`
  and `corepack pnpm --dir atlas run test`.
- [x] Test the built reader's Chapter 35 next link, Chapter 36 route, theorem
  anchors, and actual Lean source destinations. Check narrow and desktop widths.

## Task 8: Build reading editions and freeze the candidate

Read the PDF skill and call workspace dependency discovery before PDF work.
Follow the existing `tools/textbook_pdf/README.md` invocation and validator.

- [x] Build the full textbook and standalone Chapter 36 PDFs from the same
  candidate. Use the builder's staged-source checks after explicit staging.
- [x] Run its PDF validator on both outputs. Inspect rendered pages containing
  the finite recurrence, scalar endpoint, and terminal theorem, plus contents
  and source-link pages. Fix clipped equations, isolated headings, and poor breaks.
- [x] Compare embedded source snapshot hashes with the actual candidate bytes.
  The PDF validator alone must not stand in for current-source comparison.
- [x] If current aggregate evidence requires regeneration, use the existing
  `refresh-local` workflow and retain every prior generation. Never edit
  immutable proof evidence to match changed sources.
- [x] Run `mise run verify` on the frozen candidate. Record the fresh results,
  including any skipped check. A failure is not a successful release gate.
- [x] Review `git diff --check`, the full diff, generated changes, provider
  dependencies, and axiom receipts. Refresh `docs/import-receipt.md` from the
  verifier's reported payload digest only after the payload is settled, then
  rerun the relevant repository verifier.
Commit disposition: commit only explicit reviewed path groups after the gate passes.
  Report the branch, commit, PDFs, proof coverage, and remaining limitations.
  Leave mainline and GitHub untouched.

## Plan review and execution status

- [x] Map the six proof sections and six exercises to implementation tasks.
- [x] Include roster/contract migration, truthful Lean links, and dependency checks.
- [x] Include per-section mathematical review, web/PDF parity, and full gate.
- [x] Keep dependency acquisition, push, deployment, and mainlining out of scope.
- [x] Select subagent-driven or inline execution with the user.
- [x] Record implementation and verification results in `fq4m`; retain the final
  commit identity and branch disposition there.

This plan is not evidence that any new theorem or artifact has been verified.

### Candidate review record

Tasks 1–7 are implemented and passed source/specification and quality reviews.
Task 8's PDF, browser, and aggregate verification checks passed. See Kata
`fq4m` for the final commit identity and feature-branch disposition. The chapter adds six exact, reconstructible records and six
distinct exercise solutions without changing any terminal provider.

- Local, non-hermetic `CrouzeixHarp` build: passed, 3395 jobs.
- Local, non-hermetic `CrouzeixTextbook` build: passed, 3715 jobs. The accepted
  receipt has 400 declarations, 466620 bytes, SHA-256
  `36d86c0836ebe86d8e1c62ce14da0fbe034dc098f4dc99bc603dc16a368b67b9`.
  Both Rust publication and read-only publication checking passed.
- PDF Node tests: 18 passed. Final staged editions contain 447, 15, and 29
  pages (book, Chapter 1, Chapter 36). All 128 bundled source snapshots match
  the current candidate, with no unresolved links or outside-page glyphs.
  Independent PNG review approved the mathematical pages and added glossary.
- Reader tests: seven passed. Static-export tests: four passed. Browser review
  at 1280 and 390 pixels found 401 rendered formulas, no math errors, and no
  page-wide horizontal overflow; Chapter 35's next link reaches Chapter 36.
- Review repairs: preserved named universes while making the cubature alias
  resolvable by the unchanged source-location validator; converted Chapter 36
  math to the shared dollar-delimiter dialect; replaced a hardcoded chapter
  count by the actual roster; updated stale live packet-count assertions.
- The full gate exposed the generated reader's growth beyond the old 4 MiB
  artifact cap. Its current 4252191-byte HTML is valid but exceeds that limit.
  A 4 MiB-plus-one-byte regression failed under the old cap and passed with
  an explicit 8 MiB reader-only cap; rejection above the new bound remains
  tested. Direct validation of the actual canonical reader then passed.
  The initial CLI count failure and this reader-size failure are not successful
  aggregate gates. Neither repair changes a theorem or historical proof receipt.
- Whole-book exact correspondence is still incomplete. None of these local
  checks establishes independent peer review or hermetic execution.

### Verification closeout

- Full local, non-hermetic `mise run verify`: passed with aggregate exit 0.
  This includes 294 textbook tests, 133 Atlas tests, four static-export tests,
  824 proof-reproduction tests, the 8825-job aggregate Lean build, Git LFS
  integrity, and repository verification of 521 import rows.
- The proof-reproduction suite skipped two optional historical fixtures:
  the absent verified temporary Jin source and the obsolete absent-route
  fixture after canonical Jin artifacts were published. Neither is a skipped
  terminal proof build.
- All 28 goal-validation tests also passed separately. The second oversized
  reader fixture now uses the configured cap plus one byte; both corpus and
  HTML rejection tests enforce the same fixed 8 MiB bound.
- Independent final integration review approved the verified candidate with
  no blocking findings. The final documentation-only closeout and refreshed
  import receipt receive a separate repository check before committing.
- All three PDF hashes and their 128 bundled source snapshots remain those of
  the reviewed mathematical candidate. No terminal provider was changed.

See Kata `fq4m` for the final commit identity and disposition.
No push, deployment, or mainline merge is part of this plan.
