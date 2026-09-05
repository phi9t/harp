# Textbook mainline landing

Prepared 2026-09-04. Kata issue `azr7`, related to PDF issue `aj4g` and whole-book completion issue `00zs`. This is an integration plan, not a claim that the textbook is finished or its proofs were freshly verified.

## Objective and authority

The owner requested both polished PDFs and a clean website UI, then approved specification and execution. Both reading formats use canonical mathematical sources and consistent theorem identifiers. The accepted editorial design is recorded in reading-editions-spec.md. Preserve mainline's existing reader. Visual review uses local rendering; no browser mockup approval session is required.

Bring a coherent committed textbook, its formal correspondence and publication machinery, and its PDF exporter onto current `master`. Preserve newer mainline behavior and every unfinished source edit. The user requested mainlining and a GitHub push, then requested a landing party. This preparation makes no mainline or remote writes. Execution should use the candidate and gates below; any wider work requires a new decision.

The PDF-only verification exception applied to `afe523a2` on its feature branch. It is not a waiver of full integration or proof verification. No cache hydration, forced push, repository-wide license, or GitHub release upload is authorized by this plan.

## Frozen observations

- Primary checkout: `repository root`, branch `master`.
- Mainline and GitHub HEAD observed at `35bfda85b07c834b0d68126c4535e119fe38c5ce`.
- Remote: `https://github.com/phi9t/harp.git`, default branch `master`.
- Source worktree: `.worktrees/crouzeix-textbook`.
- Source branch: `codex/crouzeix-textbook`, tip `afe523a2`.
- Common ancestor: `8723d1c01bb6aae1e6eab133a3614c3056cdd002`.
- Mainline/source unique commits: 130/210, or 129/209 after patch-equivalence filtering.
- Source `e7fe1f40` and mainline `0c090909` are patch-equivalent. Do not replay the fix twice.
- Mainline has the Crouzeix research packet, but no `knowledge/crouzeix_textbook/`. The exporter alone cannot run there.

Recheck these refs before execution and before push. They are observations, not locks.

## Recommended candidate

Use `0424501af92bc799e766650ee8c13d6ed09b4b48` as the committed mathematical/publication baseline. It follows Wave 4 publication `60fbb5a3` and its formatting/receipt closeout. Add the seven-file PDF change from `afe523a2`, with the integration repairs below. Use an explicit, reviewed delta from the common ancestor, not a wholesale copy of the source tree and not a replay of 210 historical commits.

`source-inventory.tsv` records the candidate baseline delta plus the PDF commit paths. It is an input inventory, not permission to overwrite every listed mainline file. LP-01 must classify each row as take, adapt, regenerate, or exclude, with a reason. In particular, independently inspect `crates/harp/src/context_control/episode.rs` and omit it unless a textbook dependency is demonstrated.

Keep these source commits deferred together:

- `62d59429`, frozen Wave 5 contract, its test module and inclusion, and its specification.
- `3bcc70cd`, the corresponding receipt refresh.

Static inspection shows the committed Wave 5 tests expect `contract-frozen` and reject premature chapter completion. No fresh test failure was established. Choosing the earlier checkpoint limits scope; it does not justify weakening tests. If the owner elects to include Wave 5, include the contract and tests together and rerun the same gates.

The baseline status records 35 chapters, 210 coverage rows, 66 exact correspondences, and 72 solved exercises. These are historical baseline counts, not acceptance substitutes. Whole-book completion remains `00zs` and is not closed by this landing.

## Preserve unfinished work

Do not restore, stash, copy, stage, or delete source-worktree edits. Preserve:

- `content/crouzeix_textbook/coverage.json` and `exercises.json`.
- `crates/harp/tests/support/crouzeix_textbook_wave5_contract.rs`.
- `docs/superpowers/specs/2026-08-29-crouzeix-textbook-analysis-operator-contract.md`.
- `formalization/lean/CrouzeixTextbook/ExportReceipt.lean`, `Part01/Chapter01.lean`, and `Part03/Chapter13.lean`.
- `knowledge/crouzeix_textbook/notation_and_glossary.md` and `part_01_linear_structure/01_objects_and_representations.md`.
- The deletion of `knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/13_metric_and_normed_spaces.md`.
- Untracked `docs/superpowers/plans/2026-09-04-crouzeix-chapter01-coordinate-workshop.md` and `docs/superpowers/specs/2026-08-30-crouzeix-textbook-mathematics-array-notation-design.md`.

The primary checkout also has unrelated `CONTEXT.md`, ADR, plan, capture, and nested knowledge changes, plus `.pnpm-store/` scratch. Record exact current status before landing. Do not include or clean these. Stop if they overlap the candidate or prevent a safe fast-forward.

## Landing party and ownership

| Role | Owns | Must not change |
| --- | --- | --- |
| Integration lead | Candidate inventory, shared Git index, commit grouping, final landing | Unfinished source files |
| Proof/publication worker | Lean imports, receipt exporter, Rust textbook validation, scoped provider dependencies | Mainline proof-evidence policy without review |
| Mainline compatibility worker | XDG task/wrapper integration, Atlas routes/contracts, related regression tests | Generated output during concurrent work |
| PDF worker | `tools/textbook_pdf/`, dependency and provenance tests | Canonical mathematical statements |
| Independent reviewers | Standards, intent, proof-claim boundaries, rendered PDF review | Implementation or merge authority |

Implementation workers need isolated worktrees and explicit owned paths. Shared `mise.toml`, wrapper, Rust CLI/tests, and generated files are integrated serially by the lead. Three read-only reviewers completed preparation: `landing_history`, `landing_proofs`, and `landing_pdf`. Their findings are captured here; implementation still needs independent review of the final candidate.

## Integration decisions

### Mathematical and publication closure

Keep `knowledge/crouzeix_textbook/`, `content/crouzeix_textbook/`, `formalization/lean/CrouzeixTextbook*`, Rust publication validation, tests, and receipt generation coherent. Resolve all imported and linked providers. Chapter 35 also requires changes outside the textbook directory in `Crouzeix/Harp/Consequences.lean`, `FiniteAtomicDilation.lean`, `FiniteAtomicL2Dilation.lean`, and `MainTheorem.lean`. Review `CrouzeixConjecture/Sharpness.lean` too.

Retain the exact/reexport/checkpoint distinctions and incomplete coverage. A source name or local PDF build is not a fresh compiler receipt. Preserve the separation between bounded receipt evidence, local Lean compilation, hermetic execution, and independent mathematical review.

### Mainline compatibility

Preserve mainline XDG/cache setup and wrapper hardening. The source adds obsolete `/private/tmp/harp-mathematical-foundations-elan` task environments; adapt textbook tasks to `scripts/harp_xdg_env.sh`. Do not copy the old wrapper or `labs/` wholesale. Preserve current route manifests, historical LS evidence checks, six-route validation, Atlas reading-path redesign, accessibility, Pages, releases, and onboarding.

A read-only three-way merge inspection found 28 conflict hunks across 12 paths: the three generated Atlas files, `atlas/src/app/routes.test.ts`, `crates/harp/tests/cli.rs`, `crates/harp/tests/lean_library.rs`, `docs/import-receipt.md`, `labs/crouzeix_proof_reproduction/ls_receipts.py`, its two relevant test files, `mise.toml`, and `scripts/check_lean_library.sh`. Clean textual merges still require semantic review. Generated files must be rebuilt, not resolved by choosing a side.

### PDF repairs before landing

1. Derive edition notes from actual source provenance. A clean checkout has Chapter 13, so it must not claim fallback. Do not claim uncommitted Chapter 1 additions or compilation evidence.
2. Make the build/test input boundary executable before committing candidate content. Current import-time `git ls-tree HEAD` rejects staged chapters. Prefer safe filesystem discovery with exactly 35 expected regular chapter files, and record Git revision plus per-input hashes. Add tests for missing chapters, wrong count, symlinked input, and source escape. Unit tests should not require loading the whole book.
3. In release mode, reject source symlinks, fallback and unresolved references. The present output guard is symlink-safe, but source reads follow symlinks. Require tracked candidate inputs and compare their hashes against the frozen tree.
4. Pin and document Node packages, Python checker dependencies, Chromium and fonts used for layout. Do not promise byte-for-byte reproducibility: dates, metadata and platform fonts vary. Require a repeatable build in the recorded environment.
5. Rebuild both PDFs from the clean candidate. Existing 431/22-page files contain dirty inputs and are not candidate evidence. Review representative pages, portable source links and status claims.

Generated PDF bundles remain ignored local artifacts for this Git push. A separate distribution step must include notices for bundled upstream sources and embedded assets, and review actual redistribution permissions. Current source snapshots do not include license companions automatically. Do not upload them as release assets as part of the code push.

## Verification and push boundary

Fresh read-only checks found the pinned Lean 4.32.1 executable under the existing XDG cache. The old temporary executable is absent. The planning worktree has no `.lake`; the source worktree has a real `.lake` directory rather than the required canonical-root symlink. Neither fact warrants cache hydration. In the eventual integration worktree, validate primary cache ancestry, package artifacts and headers, toolchain pin, and mise trust before creating the permitted whole-root symlink to the primary cache. Missing dependencies stop the proof gate and require explicit owner direction.

After integration, run focused Rust textbook/corpus/CLI/wrapper tests, Python proof-evidence tests, Atlas tests, fresh textbook compilation/publication, and PDF checks. Freeze the payload and obtain independent reviews, then run `mise run verify`. Regenerate canonical-derived outputs together. Refresh `docs/import-receipt.md` with the actual verifier digest last, then rerun the repository check and full gate on the settled candidate. Never infer success from partial output or old receipts.

Before push, recheck GitHub and local mainline. Fast-forward only if the reviewed candidate contains the latest mainline and the primary checkout can advance without touching unrelated work. Otherwise integrate the new mainline changes and reverify. Use ordinary `git push origin master:master`, never force. Read the remote ref back and confirm it equals the landed commit. Branch protection requiring a PR is a stop condition, not permission to bypass it.

Keep the dirty textbook worktree and branch after landing. Do not call the book complete, publish PDFs, or assert fresh Jin/LS/Harp validation unless the corresponding recorded gate actually passed.
