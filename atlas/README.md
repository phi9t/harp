# Harp Atlas

Harp Atlas is the Weng-first technical reader for canonical chapters, concepts,
system readings, and comparison lessons under `knowledge/rsi/`. Interaction
code supplies navigation, prediction/reveal state, deterministic diagnosis,
and local exports. It does not contain a second technical explanation.

[Return to the canonical RSI orientation packet](../knowledge/rsi/rsi_index.md).
[Open the checked-in offline atlas](dist/harp-atlas.html).

The app does not fetch source material at runtime. Markdown under
`knowledge/rsi/` and `coverage-map.tsv` under `content/` remain the content
authority. `cargo run -p harp -- build`
validates those files and produces
`src/content/generated/corpus.json` with rendered HTML and source digests.
That JSON is derived build input, not a second prose source. The production
entry point imports only this validated corpus.

## Reading and diagnosis

- Weng is the default route and follows the original article in order.
- Weng's context-engineering section opens a navigable deep dive through ACE,
  MCE, Meta-Harness, the runtime context pipeline, state continuity, causal
  ablations, and the claim ceiling.
- Systems lists sixteen source-bound identities and renders published canonical
  system articles.
- Lessons provides six prediction/reveal comparisons with session-only state.
- Diagnose starts blank or forks one of twelve source-backed cases.
- Markdown, JSON, and external critique-packet downloads run in the browser.
- Imported AI commentary remains separate and cannot alter the rule result.
- The workbench shows mechanism classification and honest claim ceiling as
  separate outputs.
- Stale JSON imports keep the saved verdict visible beside the current
  interpretation.
- JSON exports use `rsi-diagnosis/v2`; authentic v1 exports are verified,
  migrated conservatively, and shown as stale saved verdicts.
- Thesis, Loop, Methods, Harnesses, Weng, Experiment, and Sources are stable
  hash routes over canonical Markdown documents.
- The Loop route includes an interactive outcome, integrity, and authority gate
  that blocks the child candidate when any gate is closed.
- Each route renders one canonical Markdown chapter with its content digest.
- Native source and reference folds stay closed until the reader opens them.
- Hash routes are stable, for example `#weng/context-engineering`,
  `#systems/aflow`, `#lessons/adas-vs-aflow`, and `#diagnose/aflow`.

## Local commands

~~~text
pnpm install --offline --frozen-lockfile
pnpm run content:build
cd .. && cargo run -p harp -- search refresh
cd .. && cargo run -p harp -- search status
pnpm run dev
pnpm run lint
pnpm run typecheck
pnpm run test
pnpm run test:export
~~~

The export writes `dist/harp-atlas.html`. It inlines the built JavaScript and
CSS, removes external stylesheet and module references, and keeps the SPA
usable from a local file URL.

The complete repository-owned gate is:

~~~text
mise run verify
~~~

The three downloaded files are:

~~~text
rsi-diagnosis.md       text/markdown
rsi-diagnosis.json     application/json
rsi-ai-critique.md     text/markdown
~~~

## Validation contract

The Rust content gate checks retained-concept ownership, Weng and system
references, diagnostic rules and cases, lesson references, local links,
six-part original-source reading routes, canonical headings, body digests, and
that handwritten TypeScript does not own canonical technical prose. The app
strictly validates schema v5, evaluates bounded rules, rejects inconsistent
current imports, and labels stale saved verdicts without silently replacing
them.
`predev`, `pretest`, `pretypecheck`, and `prebuild` regenerate the payload so
local workflows cannot silently use stale Markdown.
