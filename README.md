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
[knowledge home](lab/harp_knowledge_home.md), or read Markdown directly.
The library includes SICP, educational materials, mathematical foundations,
and Crouzeix proof studies. The browser Atlas and offline HTML export are retired.
See the [reading guide](docs/getting-started.md#read-the-knowledge-library) for
local reading and evidence setup.

## What you can do

| Goal | Where to start |
| --- | --- |
| Review the articles previously read | [Brief article survey](lab/article_survey.md) |
| Study interpreters and agent engineering | [SICP](lab/sicp/sicp_index.md) and [agentic engineering](lab/agentic_engineering/agentic_engineering_index.md) |
| Search and validate the local corpus | [CLI setup and first commands](docs/getting-started.md#build-the-cli) |
| Run Codex in isolated, restartable coding attempts | [Durable coding work](docs/durable-runtime.md) |
| Inspect context tools and old command dispositions | [Agent workflows](docs/agent-workflows.md) |

The library also covers harness benchmarks, agentic engineering,
SICP, mathematical foundations, and proof-reproduction studies. Workstream records describe checked-in research and proof work.

## Build the CLI

The Python + SQLite control plane lives in [`durable/`](durable/).
It owns admission, isolated attempts, recovery, verification and approved Git
landing. [`insula/`](insula/) isolates workload and service processes; [`runner/`](runner/)
transports commands over SSH. See the [operator guide](docs/durable-runtime.md).

The ordinary CLI build supports Python 3.9 or newer. Durable coding work
requires Python 3.11 or newer on both the Linux host and in its native rootfs;
see [the runtime prerequisites](docs/durable-runtime.md#prepare-a-project).
With Mise installed, run these commands from the repository root after reviewing
`mise.toml`:

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

- `cli/` owns the Harp command and its supporting library; `sessions/` owns
  rollout indexing and session analysis.
- `durable/` owns the sole agent execution framework, SQLite journal,
  Git-managed intent, verification and landing. Embedded workflow context
  contracts live under `durable/intent/bundles/`.
- `insula/` owns containment and service supervision; `runner/` transports
  admitted work to a privately configured SSH host.
- `lab/math/`, `lab/sicp/`, `lab/soma/` and `lab/benchmarks/` hold workloads,
  proofs, educational exercises and preserved benchmarks. Textbook JSON
  contracts and SICP contracts live with those workloads.
- Maintained prose lives beside its workload in `lab/`; shared implementation
  studies live in `lab/agentic_engineering/`. `lab/generated/corpus.json` is the
  derived headless index. PDF outputs live under ignored `publications/`.
- `evidence/` holds pinned source bytes, licenses and verification provenance.
  See [the evidence guide](evidence/README.md).

The retired Rust controller packages are a standalone historical workload under
`lab/benchmarks/legacy_runtime/`; they are not product dependencies or controller
entrypoints. [The ownership design](docs/workstream/repo-architecture/design.md)
records the dependency boundaries and deferred workloads.

## Scope and status

Harp works locally. Reading, search, and normal validation do not fetch
research sources. Live agent runs are opt-in and need separately installed,
configured provider CLIs; they may use the provider's network and paid services.

Codex is the first qualified provider for the consolidated framework. The Trae
adapter and its prior evidence are retained; further integration is deferred.
Historical research and benchmark captures retain their original claims and do
not establish current runtime qualification.

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
