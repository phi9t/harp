# Harp

Harp is a standalone, local-first technical atlas for recursive
self-improvement research. It combines:

- managed RSI Markdown under `knowledge/rsi/` and registered topic packets
  under `knowledge/darwin_godel_machine/`, `knowledge/meta_harness/`, and
  `knowledge/harness_benchmarks/`;
- a first-class Mathematical Foundations packet under
  `knowledge/mathematical_foundations/`, with original exposition and worked
  problems informed by user-supplied Bishop and Lax inputs that remain local
  only and are not redistributed;
- a pinned Lean companion under `formalization/mathematical_foundations/` for
  the packet's small finite-domain theorem boundary; its six namespaces and
  the exact 48-problem status map are documented in
  `knowledge/mathematical_foundations/formalization_map.md`;
- captured Weng, RLM, SICP, and Meta-Harness evidence with byte-level manifests
  and source-specific license records;
- a verified coevolution research-agenda packet under
  `knowledge/verified_coevolution_agenda/`, with LADDER, PRIME-RL TTRL, NSRSA,
  SAHOO, model-collapse, Scrivens, and Godel-family evidence boundaries under
  `evidence/verified_coevolution_agenda/`;
- a source-backed agentic engineering reference packet under
  `knowledge/agentic_engineering/`, including a captured Kenn/Wes workflow
  article and Clanker Constitution evidence under `evidence/agentic_engineering/`;
- narrow public-source snapshots for Pi, Hermes Agent, Codex, ARC-AGI-3,
  Autoresearch, and Meta-Harness;
- a durable agentic execution harness for RLM task graphs, with SQLite state,
  content-addressed artifacts, restart recovery, and CLI-process supervision;
- a Rust validator/compiler and deterministic diagnosis contract;
- **Workstreams console:** an offline operational view of proof, research, evaluator, and agentic-engineering work with explicit evidence boundaries.
- a React 19 Atlas with a checked-in offline single-file export; and
- a local SQLite FTS5 index.

Managed Markdown under `knowledge/` is the technical-prose authority.
`content/` contains structured contracts and diagnostics. Generated JSON and
HTML are derived artifacts.

## Obsidian knowledge vault

Open the repository root as an Obsidian vault, then start at
[`knowledge/harp_knowledge_home.md`](knowledge/harp_knowledge_home.md). The
same canonical Markdown remains readable in the offline Atlas; Obsidian adds
native wikilinks, backlinks, Base views, and the small knowledge map without
creating a second prose source.

The repository does not track personal `.obsidian/` state. To install the
reviewed portable profile into a chosen vault, run:

```sh
python3 tools/obsidian/apply_profile.py --vault "$(git rev-parse --show-toplevel)"
```

The installer is create-only unless `--replace` is explicit. Validate the
committed navigation assets without an Obsidian desktop installation:

```sh
python3 scripts/validate_obsidian_assets.py
```

The pinned `obsidian-markdown`, `obsidian-bases`, `json-canvas`,
`obsidian-cli`, and `defuddle` skills are installed locally for TRAE CLI use;
restart TRAE CLI after this update to load them.

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
cargo run -p harp -- rlm checkpoint
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

## Agentic Engineering Harness

Harp has two implemented harness surfaces:

- `harp rlm ...` runs durable task graphs through an activity-oriented runtime.
  It is the execution path for restartable Codex CLI work.
- `labs/meta_harness_trae/` is an opt-in Meta-Harness proposer supplement. It
  validates proposal shape and candidate interfaces only; it does not claim a
  benchmark score, paid-model evaluation, held-out result, or recursive
  self-improvement result.

RLM benchmark graphs are JSON files validated by the Rust contract layer. The
checked-in schema receipts are:

- `benchmarks/codex-architecture/schemas/task-graph.json`
- `benchmarks/codex-architecture/schemas/result-envelope.json`

Run a task graph with the installed Codex CLI:

```sh
cargo run -p harp -- rlm run \
  --benchmark path/to/task-graph.json \
  --runtime codex
```

Use `--runtime-executable /path/to/codex` when `codex` is not on `PATH`.
Harness state defaults to `.harp/rlm` and contains:

- `state.sqlite` for runs, tasks, attempts, activities, leases, and recovery;
- `artifacts/` for content-addressed results, evidence, and checkpoints; and
- `runtime-home/` for supervisor-owned CLI process records and spools.

Operational commands:

```sh
cargo run -p harp -- rlm checkpoint
cargo run -p harp -- rlm status <run-id>
cargo run -p harp -- rlm resume <run-id>
cargo run -p harp -- rlm resume --all-incomplete
```

The runtime choices are `codex`, `fake`, and `traecli`; `traecli` is reserved
for a later adapter slice and currently fails before semantic work starts. The
Codex adapter uses `codex exec --json` as the integration boundary, writes a
per-activity output schema, parses bounded JSONL events, records process
fingerprints, and interrupts process groups with SIGINT, SIGTERM, then SIGKILL
when cancellation requires it.

The engine pins graph policy, projection policy, artifact-store identity, and
runtime provenance into each run. Resume rejects mismatched runtime or artifact
authority before observing existing processes or starting new semantic work.

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
Open `atlas/dist/harp-atlas.html#workstreams` for the offline Workstreams reference snapshot.

## Evidence and licensing

Harp has no repository-wide license. Captured works retain their upstream
copyright and license status. See each evidence bundle's `PROVENANCE.md`,
manifests, license records, and `evidence/implementations/*/LICENSE_STATUS`.
Harp-authored metadata does not relicense captured works.

The maintained product and contribution rules are in
`docs/product-contract.md` and `docs/contributing.md`.
