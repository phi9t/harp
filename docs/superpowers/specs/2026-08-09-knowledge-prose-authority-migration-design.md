# Knowledge prose authority migration design

**Status:** Approved design, pending implementation planning
**Date:** 2026-08-09
**Decision:** Technical prose moves out of `content/`; `knowledge/` becomes
the only technical-prose authority.

## 1. Objective

Move Harp's complete technical Markdown corpus from `content/` to a managed
namespace under `knowledge/`, without weakening its corpus contracts,
offline-reader behavior, evidence boundaries, or deterministic validation.

The migration corrects the ownership model rather than merely relocating files:

- `knowledge/` owns technical prose;
- `content/` owns structured machine-readable inputs and diagnostics only;
- `evidence/` owns captured upstream bytes and source-specific metadata only;
- Atlas JSON and static HTML remain derived artifacts; and
- `knowledge/` is not a catch-all search root for local drafts or editor state.

This work does not create the requested Agentic Harness Engineering case study.
That packet follows after the authority migration lands, under the new
`knowledge/` contract.

## 2. Approved layout

### 2.1 Managed technical corpus

The existing `content/**/*.md` tree moves, path-preservingly, to
`knowledge/rsi/`:

```text
content/chapters/recursive-improvement-loop.md
  -> knowledge/rsi/chapters/recursive-improvement-loop.md
content/concepts/harness-components.md
  -> knowledge/rsi/concepts/harness-components.md
content/systems/ahe.md
  -> knowledge/rsi/systems/ahe.md
content/weng-sources/ahe.md
  -> knowledge/rsi/weng-sources/ahe.md
content/sicp/course/seminars/09-eval-apply-and-executable-semantics.md
  -> knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics.md
content/rsi_index.md
  -> knowledge/rsi/rsi_index.md
```

This includes every tracked Markdown file currently below `content/`: root
orientation and deep dives, chapters, concepts, system readings, Weng
companions and source cards, lessons, SICP material, and source-oriented
Markdown records. The existing field note moves from
`knowledge/lw_rsi_harness.md` to `knowledge/rsi/lw_rsi_harness.md`, so it joins
the managed RSI corpus rather than remaining a loose root-level document.

The move is a Git rename where possible; it does not rewrite captured evidence
or reorganize concepts into a new taxonomy. Preserving the subtrees makes
review, link repair, and corpus-registry migration mechanically auditable.

### 2.2 First-class topic packets

Existing packet roots remain in place and are canonical technical prose:

```text
knowledge/darwin_godel_machine/
knowledge/meta_harness/
knowledge/harness_benchmarks/
```

They retain their packet-specific entrypoints and structure. Their notices,
links, and tests must stop describing them as non-authoritative projections
whose canonical claims live under `content/`. A packet can still distinguish
its synthesis from a primary-source claim; the authority move does not lower
the evidence ceiling.

The later AHE work will create a sibling packet:

```text
knowledge/agentic_harness_engineering/
```

It is deliberately out of scope for this migration.

### 2.3 Structured inputs and diagnostics

`content/` remains the home of non-Markdown machine inputs:

```text
content/coverage-map.tsv
content/retained-concepts.tsv
content/lessons.json
content/weng-*.{json,tsv}
content/sources/*.tsv
content/systems/system_readings.json
content/diagnostics/**/*.json
```

The migration does not rename these data paths. Their fields that name
canonical Markdown paths change from `content/.../*.md` to
`knowledge/rsi/.../*.md`. In particular, coverage rows, lesson manifests,
system readings, Weng reading-map companion paths, claim-ladder routes, source
card routes, and generated reference links must use the new canonical paths.

The validator gains a structural ownership invariant: a tracked
`content/**/*.md` file is invalid. The check must be recursive, reject
symlinks, and make the failure identify the prohibited path. This prevents
future prose from silently restoring the split authority model.

## 3. Product boundary and path contracts

### 3.1 Explicit prose roots

Atlas compilation and local search use explicit canonical Markdown roots:

```text
knowledge/rsi/
knowledge/darwin_godel_machine/
knowledge/meta_harness/
knowledge/harness_benchmarks/
```

The compiler's required chapters, reader routes, auxiliary documents, system
registry, Weng map, coverage contract, and lesson contract point at
`knowledge/rsi/` paths. The specialized packets remain explicitly registered
where Atlas exposes them; no automatic traversal of arbitrary `knowledge/`
files is introduced.

Search replaces its `content` plus `knowledge/meta_harness` roots with the
same explicit managed technical roots. It continues to index the existing
Weng and RLM text captures. It must not index:

- unregistered future notes under `knowledge/`;
- `knowledge/.obsidian/`;
- user-local scratch Markdown such as `knowledge/Untitled.md`;
- generated Atlas artifacts;
- evidence snapshots beyond its existing text roots; or
- symlinked files.

### 3.2 Stable generated-output boundary

The following derived paths remain unchanged:

```text
atlas/src/content/generated/corpus.json
atlas/dist/harp-atlas.html
atlas/dist/harp-atlas.receipt.json
```

Their payloads will change because canonical document paths change. The
compiler, Atlas export receipt, and repository verifier continue to reject
unsafe generated targets and require all derived artifacts to agree with their
canonical inputs.

The TypeScript source directory `atlas/src/content/` remains an application
module namespace for generated corpus parsing and is not a claim that
repository prose belongs in top-level `content/`.

### 3.3 Links and projections

The one-to-one tree move preserves most relative links inside the migrated
`content/` tree. Implementation must still validate every local Markdown link
against its new source path and rewrite all links that cross the old root:

- links from topic packets to `../../content/...` become links into
  `../rsi/...`;
- repository documentation and Atlas README links to `content/*.md` become
  `knowledge/rsi/*.md`;
- generated `reference/*.html` links to canonical Markdown change from
  `../content/...` to `../knowledge/rsi/...`;
- course-builder constants, fixtures, and emitted HTML labels stop calling
  `content/weng-sources/` the Markdown authority; and
- prose statements that say packet material is non-authoritative solely
  because it lives in `knowledge/` are removed or revised.

`reference/` and `lessons/` remain generated teaching projections. Their
structured inputs remain in `content/`; their canonical Markdown targets are
in `knowledge/`.

## 4. Implementation surface

### 4.1 Rust corpus compiler and contracts

The migration updates:

- `crates/harp/src/corpus/mod.rs` constants, reader routes, required chapters,
  and auxiliary documents;
- `crates/harp/src/corpus/contracts.rs` path guards for coverage, systems,
  Weng companion documents, and canonical Markdown roots;
- `crates/harp/src/corpus/lessons.rs` lesson Markdown path validation;
- `crates/harp/src/corpus/render.rs` canonical-path classification;
- `crates/harp/src/corpus/tests.rs` fixtures and targeted ownership tests; and
- `crates/harp/src/search.rs` explicit corpus roots and search tests.

The diagnostics paths in `crates/harp/src/corpus/rules.rs` remain under
`content/diagnostics/` because they are JSON contracts, not technical prose.
The corpus compiler must not use an unrestricted `knowledge/` walk as a
shortcut for discovering canonical documents.

### 4.2 Structured registries and course generator

The migration updates the canonical-path values in:

- `content/coverage-map.tsv`;
- `content/lessons.json`;
- `content/systems/system_readings.json`;
- `content/weng-reading-map.json`;
- `content/weng-claim-ladder.json`;
- `content/weng-source-cards.tsv`; and
- any other structured registry or claim map that stores a Markdown route.

`scripts/build_weng_course.py` keeps structured `content/` input paths but
uses `knowledge/rsi/` paths for companion documents and canonical route
validation. Its deterministic self-tests and generated reference pages must
assert the new routes.

### 4.3 Atlas and tests

The JSON schema does not change: canonical paths are still validated strings
and schema version remains `rsi-technical-atlas/v5`. Atlas receives the new
paths through regenerated corpus JSON. Its fixtures and route tests must
expect `knowledge/rsi/...` canonical paths; application imports from
`atlas/src/content/` remain unchanged.

The migration updates Rust integration tests, TypeScript canonical-corpus
tests, DGM packet tests, Weng curriculum tests, credible-document style tests,
and CLI/search fixtures. Tests must cover:

1. a `knowledge/rsi/...` canonical path compiling successfully;
2. a `content/.../*.md` canonical path being rejected;
3. a tracked Markdown file beneath `content/` being rejected;
4. a managed packet file being searchable; and
5. a loose/unregistered `knowledge/` Markdown file being excluded from search.

### 4.4 Contributor documentation

Update `AGENTS.md`, `README.md`, `docs/product-contract.md`, `atlas/README.md`,
and `RESOURCES.md` to describe:

- technical Markdown under the managed `knowledge/` roots as authoritative;
- `content/` as structured contracts and diagnostics only;
- generated JSON/HTML as derived;
- explicit canonical search roots; and
- no new prose under `content/`.

The product contract retains the exact corpus-count contract unless the
compiler demonstrates that the move itself changes a count. No count is
changed merely to accommodate a new directory prefix.

## 5. Non-goals

This migration does not:

- write the AHE packet or change the substantive AHE reading;
- change the Atlas corpus schema or diagnosis export schemas;
- move JSON, TSV, or diagnostics out of `content/`;
- alter evidence bytes, source licenses, manifests, or evidence locators;
- change the generated Atlas output paths;
- introduce automatic indexing of every `knowledge/` file;
- absorb untracked editor state, pasted images, or drafts; or
- add dependencies on another checkout.

## 6. Migration sequence and commits

The implementation plan will split the work into independently verifiable
commits:

1. **Contract and tests:** introduce the `knowledge/rsi/` path constants and
   failing ownership/search tests while canonical files are still in their old
   locations.
2. **Canonical move:** rename tracked Markdown to `knowledge/rsi/`, rewrite
   structured path fields and local links, then make compiler and generator
   tests pass.
3. **Product surfaces:** update documentation, reference projections, search
   roots, and Atlas fixtures; regenerate corpus and static Atlas artifacts.
4. **Release receipt:** run the full gate against only the staged surface,
   refresh `docs/import-receipt.md` if the repository verifier reports the
   expected payload digest, and commit the derived receipt together with the
   final migration surface.

Each commit stages explicit paths only. Before validation, use
`git stash push --keep-index -u` to isolate the staged migration from unrelated
untracked work. No blanket `git add .` is permitted.

## 7. Validation

Focused validation during implementation:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cargo test -p harp --test dgm_knowledge_packet
cargo test -p harp --test weng_teaching_curriculum
python3 scripts/build_weng_course.py --check
cd atlas && corepack pnpm run test
```

Final validation:

```sh
cargo run -p harp -- check
cargo run -p harp -- build
cargo run -p harp -- build --check
cargo run -p harp -- search refresh
cargo run -p harp -- search status
cd atlas && corepack pnpm run test:export
mise run verify
```

The final commit must include regenerated
`atlas/src/content/generated/corpus.json`,
`atlas/dist/harp-atlas.html`, and its Atlas receipt when their bytes change.
If `mise run verify` reports a stale import payload digest, update only
`docs/import-receipt.md` with the verifier-provided value and rerun the full
gate.

## 8. Design self-review

- The target location, managed search roots, and no-Markdown-in-`content/`
  invariant are explicit.
- Existing packets remain canonical and are not flattened into `knowledge/rsi/`.
- Structured inputs and generated outputs retain their established locations.
- Link repair, generated references, compiler contracts, tests, documentation,
  and receipt regeneration are included.
- The AHE case study is deliberately deferred until this ownership boundary is
  established.
