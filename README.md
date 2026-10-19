# Harp

A local-first agent runtime with a maintained knowledge and mathematics library.

Harp combines agent execution and engineering tools with a library of SICP,
mathematics and proof work. Run restartable agent tasks with recorded results,
or read the maintained explanations and their source evidence.

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
| Review the articles previously read | [Brief article survey](knowledge/article_survey.md) |
| Study interpreters and agent engineering | [SICP](knowledge/sicp/sicp_index.md) and [agentic engineering](knowledge/agentic_engineering/agentic_engineering_index.md) |
| Search and validate the local corpus | [CLI setup and first commands](docs/getting-started.md#build-the-cli) |
| Run Codex or Trae in isolated, restartable coding attempts | [Durable coding work](docs/durable-runtime.md) |
| Inspect context tools and old command dispositions | [Agent workflows](docs/agent-workflows.md) |

The library also covers harness benchmarks, agentic engineering,
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

- `knowledge/` holds maintained educational and engineering prose.
- `evidence/` holds captured source bytes, manifests, and source-specific
  license records.
- `content/` holds machine-readable contracts.
- `content/generated/` holds the compiled headless corpus and local publication outputs.
- `crates/` holds the CLI, reusable contracts and retained Rust libraries.
- `harp_durable/` exposes the selected coding lifecycle over the imported lightweight runtime.
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
