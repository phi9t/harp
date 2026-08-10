# Harp

Harp is a standalone, local-first technical atlas for recursive
self-improvement research. It combines:

- managed RSI Markdown under `knowledge/rsi/` and registered topic packets
  under `knowledge/darwin_godel_machine/`, `knowledge/meta_harness/`, and
  `knowledge/harness_benchmarks/`;
- captured Weng, RLM, SICP, and Meta-Harness evidence with byte-level manifests
  and source-specific license records;
- narrow public-source snapshots for Pi, Hermes Agent, Codex, ARC-AGI-3,
  Autoresearch, and Meta-Harness;
- a Rust validator/compiler and deterministic diagnosis contract;
- a React 19 Atlas with a checked-in offline single-file export; and
- a local SQLite FTS5 index.

Managed Markdown under `knowledge/` is the technical-prose authority.
`content/` contains structured contracts and diagnostics. Generated JSON and
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

The opt-in proposer supplement is under `labs/meta_harness_trae/`. Its
deterministic tests and recorded run are part of offline verification; invoking
a new live proposer is intentionally outside `mise run verify`.

### Context-Control Surface

Harp can inspect installed Trae CLI and Codex CLI capabilities, compile an
immutable local context release, and run either provider through the same
observation surface:

```sh
cargo run -p harp -- providers doctor
cargo run -p harp -- providers doctor --provider codex
cargo run -p harp -- releases compile
cargo run -p harp -- releases list
cargo run -p harp -- releases inspect <release-id>
cargo run -p harp -- run --provider trae --workflow auto -- \
  fix the failing CI test
cargo run -p harp -- run --provider codex --workflow code_review -- \
  review this change
```

The task begins after `--`; it is not a native-provider argument channel.
Optional `--model`, `--profile`, `--sandbox`, and `--approval` values are
accepted only when the selected provider reports the required capability.

Private context-control state uses the first nonempty location:

1. `HARP_HOME` as the complete state-root path;
2. `${XDG_STATE_HOME}/harp`; or
3. `${HOME}/.local/state/harp`.

A repository may add `.harp/context-control.json` to disable Harp, restrict
allowed workflows, pin a release, lower the context budget, or declare required
verification labels. Labels are validated and bound into the policy digest;
verifier execution and outcome enforcement are deferred. The policy cannot add
prompt text, select an executable, or broaden authority.

During `harp run`, provider stdout bytes requested in JSONL mode remain
byte-exact and are the only stdout; malformed output is preserved rather than
normalized. Harp lifecycle records and the final episode ID go to stderr,
including when global `--format json` is selected. Native provider
configuration, repository rules, and enforcement remain active. Harp does not
synthesize or silently widen privileges, but explicit user-supplied
`--sandbox`, `--approval`, and `--profile` overrides are forwarded to the
provider and may change its native behavior.

The pre-launch episode manifest is explicitly partial: it records the selected
release, Harp context supplement, invocation identity, and pre-run repository
state, not the provider's complete effective prompt, native skill selection, or
compaction state. Raw provider bytes and the completion receipt are separate
post-run evidence. ACE/MCE suggestion generation, evaluation, canarying, and
promotion are deferred. `mise run verify` exercises fake provider fixtures and
never launches a real Trae CLI or Codex CLI run.

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
