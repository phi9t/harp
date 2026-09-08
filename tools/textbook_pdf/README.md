# Textbook reading PDFs

Build three working-draft reading artifacts from canonical Crouzeix Markdown:

- `output/pdf/crouzeix-foundations.pdf`: all 36 chapters, both Harp supplements, glossary and sources.
- `output/pdf/chapter-01-objects-and-representations.pdf`: Chapter 1 and glossary.
- `output/pdf/chapter-36-harp-finite-horizon-proof.pdf`: Chapter 36, both Harp supplements and glossary.

The book uses a 7.5 × 10 inch page, Georgia text, KaTeX mathematics, quiet chapter openings, clickable contents and PDF bookmarks. Calculations are unboxed; Lean panels and correspondence notes remain distinct from mathematical statements. Running headers and folios are decorative PDF artifacts. Canonical Markdown is never rewritten.

The mathematical audit and finite-horizon remainder are unnumbered supplements,
with their own contents entries and bookmarks. They do not add chapters or
indexed proof coverage. Their canonical status labels remain visible in print.

Each chapter header links to its accompanying Lean chapter module. Discovery requires exactly one `ChapterNN.lean` beneath `formalization/lean/CrouzeixTextbook/PartXX/`, independently of prose part numbering. Missing, ambiguous or symlinked modules fail. These source links provide access to the module; they do not assert compilation or certification.

## Dependencies

Use Node 22 or newer and the exact direct package versions in `package.json`. Install packages into this directory, or expose a machine-local package cache through standard `NODE_PATH`; no other checkout is a dependency. Atlas's installed packages are an optional fallback. Use locally installed Chromium compatible with Playwright. The builder never installs packages or browsers and blocks HTTP resources while printing.

Python checks require `pypdf` and `pdfplumber`. Visual review uses Poppler's `pdfinfo` and `pdftoppm`. No package lockfile or Python environment lock is supplied; direct JavaScript versions are enforced at build time. Record Python tool versions with review evidence. Package installation is a separate environment preparation step.

## Build and review

```sh
node --test tools/textbook_pdf/*.test.cjs
CHROMIUM_EXECUTABLE=/absolute/path/to/chromium node tools/textbook_pdf/build.cjs
python3 tools/textbook_pdf/check_pdf.py
```

Without `CHROMIUM_EXECUTABLE`, Playwright's installed Chromium is used. Source discovery reads the actual filesystem, including staged or uncommitted candidate files. It requires chapters 01 through 36 exactly once, glossary and source registry. Missing sources, symlinked inputs or output ancestors, repository escapes and unresolved links fail the build. There is no Git-history fallback. Unit rendering tests use fixtures; a Chapter 36 integration test also renders its canonical source to catch raw, unparsed mathematics.

For a staged landing candidate, use `node tools/textbook_pdf/build.cjs --staged`. This additionally compares every bundled source byte with the Git index. Stage the complete intended source payload first. Ordinary builds need Git only to record HEAD/branch; HEAD is explicitly not asserted to identify all source bytes.

The manifest records per-file hashes, an aggregate candidate hash, runtime/browser versions, sampled actual Chromium font names, selected CSS font stacks, and PDF hashes/page counts. Font sampling covers the first 300 matching text/math elements of each edition; it is not a full font inventory. System fonts and timestamps affect output. Do not claim byte-identical output across machines. Source changes during rendering fail verification.

Keep `sources/` and `build-manifest.json` beside all PDFs and HTML files. Relative links should be reviewed after moving the complete bundle; local-link support varies by viewer. Snapshot collection includes directly linked files, not their transitive references.

The checker verifies all PDF pages' basic content and glyph boundaries, contents destinations, relative links, candidate/source hashes and output hashes. Also render and inspect cover, contents, chapter openings, matrices, long formulas, tables, Lean references and late proof-workshop pages. Automated checks cannot establish overlap-free layout or proof correctness. Build into a fresh output directory for final acceptance so stale artifacts cannot be confused with current results.

## Publication boundary

This export performs no Lean compilation, receipt/ledger updates, Atlas regeneration or independent proof certification. Working-draft labels remain visible. Run the repository's full gate before committing; PDF tests/build/checks are additional checks, not included in that gate.

Generated outputs are ignored. Public distribution requires a separate audit of the actual snapshot and embedded font/asset licenses and any required notices. The builder does not automatically bundle license files or grant a repository-wide license.
