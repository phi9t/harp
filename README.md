# Harp

A local-first research atlas and agent runtime for recursive self-improvement.

Harp brings research notes, captured sources, and runnable experiments into one
repository. Use it to compare self-improving systems, trace claims to evidence,
or run restartable agent tasks with recorded results.

[Get started](docs/getting-started.md) ·
[Agent workflows](docs/agent-workflows.md) ·
[Documentation](docs/README.md) ·
[Contribute](docs/contributing.md)

## Start reading

The offline Atlas is already built. After cloning the repository, open
[`atlas/dist/harp-atlas.html`](atlas/dist/harp-atlas.html) from your local
filesystem in a browser. It includes the compiled research corpus, JavaScript,
and styles. No server, build tools, or model account is needed to read it.
GitHub displays the HTML source; it does not run the reader.

Prefer Markdown? Start with the
[RSI orientation index](knowledge/rsi/rsi_index.md), or open the repository root
as an Obsidian vault and visit the
[knowledge home](knowledge/harp_knowledge_home.md). The
[reading guide](docs/getting-started.md#read-the-atlas) covers both options and
Git LFS setup for captured evidence.

## What you can do

| Goal | Where to start |
| --- | --- |
| Understand recursive self-improvement and compare systems | [RSI orientation](knowledge/rsi/rsi_index.md) and [system readings](knowledge/rsi/systems/system_readings_index.md) |
| Check what supports a research claim | [Claim ledger](knowledge/rsi/claim_evidence_ledger.md), [source registry](knowledge/rsi/source_registry.md), and [missing evidence](knowledge/rsi/missing_evidence.md) |
| Search and validate the local corpus | [CLI setup and first commands](docs/getting-started.md#build-the-cli) |
| Run and recover Codex task graphs | [Durable task graphs](docs/agent-workflows.md#durable-task-graphs) |
| Run Codex or Trae CLI with recorded context and output | [Provider workflows](docs/agent-workflows.md#provider-workflows) |

The research collection also covers harness benchmarks, agentic engineering,
SICP, mathematical foundations, and proof-reproduction studies. The Atlas
Workstreams view is an offline reference snapshot, not a live job monitor.

## Build the CLI

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

Mise pins Rust, Node, and Git LFS; bootstrap installs locked Cargo and Atlas
dependencies. See [getting started](docs/getting-started.md) for cloning,
search, JSON output, and troubleshooting. Reading and ordinary CLI use do not
require the full contributor verification gate.

## How the repository fits together

- `knowledge/` holds the maintained research notes.
- `evidence/` holds captured source bytes, manifests, and source-specific
  license records.
- `content/` holds machine-readable contracts and diagnostics.
- `atlas/` holds the browser reader and its generated offline export.
- `crates/` holds the CLI, contracts, and durable execution engine.
- `labs/` and `formalization/` hold experiments and Lean companions.

Markdown is the authority for technical prose. The compiled corpus and offline
Atlas are generated from it. The
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
