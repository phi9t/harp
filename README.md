# Harp

A local-first agent runtime with a maintained knowledge and mathematics library.

Harp brings research notes, captured sources, and runnable experiments into one
repository. Use it to compare self-improving systems, trace claims to evidence,
or run restartable agent tasks with recorded results.

[Get started](docs/getting-started.md) ·
[Agent workflows](docs/agent-workflows.md) ·
[Documentation](docs/README.md) ·
[Contribute](docs/contributing.md)

## Start reading

Open the repository root as an Obsidian vault and visit the
[knowledge home](knowledge/harp_knowledge_home.md), or read Markdown directly.
The library includes SICP, educational materials, mathematical foundations,
and Crouzeix proof studies. The browser Atlas and offline HTML export are retired.
See the [reading guide](docs/getting-started.md#read-the-knowledge-library) for
local reading and evidence setup.

## What you can do

| Goal | Where to start |
| --- | --- |
| Understand recursive self-improvement and compare systems | [RSI orientation](knowledge/rsi/rsi_index.md) and [system readings](knowledge/rsi/systems/system_readings_index.md) |
| Check what supports a research claim | [Claim ledger](knowledge/rsi/claim_evidence_ledger.md), [source registry](knowledge/rsi/source_registry.md), and [missing evidence](knowledge/rsi/missing_evidence.md) |
| Search and validate the local corpus | [CLI setup and first commands](docs/getting-started.md#build-the-cli) |
| Run and recover Codex task graphs | [Durable task graphs](docs/agent-workflows.md#durable-task-graphs) |
| Run Codex or Trae CLI with recorded context and output | [Provider workflows](docs/agent-workflows.md#provider-workflows) |

The research collection also covers harness benchmarks, agentic engineering,
SICP, mathematical foundations, and proof-reproduction studies. Workstream records describe checked-in research and proof work.

## Build the CLI

The Corenius terminal agent, SQLite controller and collaboration tools live in
[`corenius/`](corenius/README.md). See [engineering subsystem usage and scope](docs/corenius-integration.md)
for its pinned toolchain, imported work and current qualification limits.

With Mise and Python 3.9 or newer installed, run these commands from the
repository root after reviewing `mise.toml`:

```sh
mise trust
mise install
mise run bootstrap
mise exec -- git lfs pull
mise run build
.build/harp-target/size/harp check
```

Mise pins Rust, Node, and Git LFS; bootstrap installs locked Cargo dependencies. See [getting started](docs/getting-started.md) for cloning,
search, JSON output, and troubleshooting. Reading and ordinary CLI use do not
require the full contributor verification gate.

## How the repository fits together

- `knowledge/` holds the maintained research notes.
- `evidence/` holds captured source bytes, manifests, and source-specific
  license records.
- `content/` holds machine-readable contracts and diagnostics.
- `content/generated/` holds the compiled headless corpus and local publication outputs.
- `crates/` holds the CLI, contracts, and durable execution engine.
- `labs/` and `formalization/` hold experiments and Lean companions.

Markdown is the authority for technical prose. The compiled headless corpus is generated from it. The
[product contract](docs/product-contract.md) records the detailed boundaries;
the [contributor guide](docs/contributing.md) explains how to change them.

## Scope and status

Harp works locally. Reading, search, and normal validation do not fetch
research sources. Live agent runs are opt-in and need separately installed,
configured provider CLIs; they may use the provider's network and paid services.

The durable task-graph runtime supports Codex. Its Trae adapter is not yet
implemented, although the separate provider workflow supports both Codex and
Trae CLI. Research notes and proposer experiments do not establish a benchmark
score or an autonomous self-improvement result.

Native packaging is available through
[`mise run release-candidate`](docs/releasing.md). It builds and checks an
executable for the current host; it does not publish a GitHub Release.
Architecture and release follow-ups are tracked in the
[release-readiness planner](docs/superpowers/plans/2026-09-04-github-release-readiness.md).

## Evidence and licensing

Harp has no repository-wide license. Captured works retain their upstream
copyright and license status. See each evidence bundle's `PROVENANCE.md`,
manifests, license records, and `evidence/implementations/*/LICENSE_STATUS`.
Harp-authored metadata does not relicense captured works.
