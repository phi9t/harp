# Retire Atlas and preserve headless knowledge publication

Status: blocked
Owner: Atlas retirement worker
Candidate base: `7f47dffc1a7f79d8812fc2658401807456d9d88d`

## What to build

Remove the browser Atlas, offline HTML export and Pages publishing. Preserve
mathematics, Crouzeix proofs, SICP and education, including corpus compilation,
search inputs, and transactional Crouzeix publication under `content/generated/`.

## Acceptance criteria

- Atlas frontend, export, Pages workflow and frontend gates are removed.
- The headless corpus and Crouzeix publication retain path validation and behavior checks.
- Original import-source rows and source-path digest remain unchanged; retired
  destinations are explicitly recorded with disposition `retired` and destination `-`.
- Applicable checks pass, with no unauthorized dependency or cache hydration.

## Blocked by

- Primary Lean cache packages symlink targets the missing
  `~/.cache/harp/lean/lean-4.32.1/packages`. Existing source/goal validation and
  compiler-derived Crouzeix receipt tests cannot complete. No hydration attempted.
- PDF dependencies are absent. `tools/textbook_pdf/package.json` pins direct
  versions but has no lockfile. Plain Node cannot resolve `marked`; the bundled
  runtime supplies `marked` but cannot resolve `katex`. No install attempted.
- Coordinator must refresh import receipt after integrating the settled payload.

## Comments

Canonical knowledge and captured evidence remain retained. Current Crouzeix goal
validation uses bounded corpus JSON; historical CPFR-089 landing evidence remains
bound to its original Git object even after browser files are retired. The
Crouzeix retired-reference scan excludes imported Corenius bytes and retains its
original Harp bounds. No PostgreSQL or Temporal work is included.

## Verification

- Changed-surface selector regressions: 10 passed.
- XDG environment shell behavior: passed.
- Headless corpus regeneration: passed, wrote `content/generated/corpus.json`.
- Documentation link/structure checks and diff whitespace: passed.
- Full textbook suite: 260 passed, 35 failed before final staging; 34 require
  unavailable Lean/cache receipts and one index scan included the then-unstaged
  Atlas deletions/imported source. All 9 focused deletion checks pass after staging.
- CLI suite: 12 passed, source validation blocked by missing Lean cache; repository
  verification requires staged payload and refreshed receipt.
- Full library suite and final integration verification are coordinated separately.
