# Harp

Harp is a standalone, local-first technical atlas for recursive
self-improvement research. It combines:

- canonical Markdown chapters, concept pages, implementation studies, and a
  nested SICP evaluator course;
- captured Weng, RLM, and SICP evidence with byte-level manifests and
  source-specific license records;
- narrow public-source snapshots for Pi, Hermes Agent, Codex, ARC-AGI-3, and
  Autoresearch;
- a Rust validator/compiler and deterministic diagnosis contract;
- a React 19 Atlas with a checked-in offline single-file export; and
- a local SQLite FTS5 index.

Markdown under `content/` is the technical-prose authority. Generated JSON and
HTML are derived artifacts.

## Setup

Prerequisites are Git LFS, Rust 1.92, Node.js 22 or newer, Corepack, pnpm, and
mise.

```sh
mise run bootstrap
mise run verify
```

`bootstrap` installs repository-local Git LFS filters, fetches locked Cargo
dependencies, and installs the pinned Atlas dependencies. Normal validation,
reading, building, and search do not fetch research sources.

## CLI

```sh
cargo run -p harp -- check
cargo run -p harp -- build --check
cargo run -p harp -- search refresh
cargo run -p harp -- search status
cargo run -p harp -- search query "recursive improvement"
cargo run -p harp -- sources verify
```

Add `--format json` before a command for the schema-versioned JSON envelope.
Public-source materialization is explicit:

```sh
cargo run -p harp -- sources materialize --source PI-MONO
cargo run -p harp -- sources materialize --all
```

Materialized repositories live under ignored `.sources/`; tracked snapshots
under `evidence/implementations/` remain the offline evidence.

## Atlas

```sh
cd atlas
corepack pnpm run dev
corepack pnpm run test
corepack pnpm run test:export
```

The checked-in offline reader is `atlas/dist/harp-atlas.html`. Open that file
directly in a browser; it contains its JavaScript, styles, and compiled corpus.

## Evidence and licensing

Harp has no repository-wide license. Captured works retain their upstream
copyright and license status. See each evidence bundle's `PROVENANCE.md`,
manifests, license records, and `evidence/implementations/*/LICENSE_STATUS`.
Harp-authored metadata does not relicense captured works.

The maintained product and contribution rules are in
`docs/product-contract.md` and `docs/contributing.md`.
