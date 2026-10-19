# Getting started

[Documentation](README.md) / Getting started

Read the maintained Markdown directly or open the repository root as an
Obsidian vault. Build the CLI for validation and local search. The complete
release gate also needs the pinned Lean toolchain and a warm dependency cache.

## Get the repository

```sh
git clone https://github.com/phi9t/harp.git
cd harp
```

Captured PDFs, images, and some archives use Git LFS. If Git LFS was already
installed and configured when you cloned, Git normally downloads those objects.
Otherwise, follow the tool setup below and run `mise exec -- git lfs pull`.
Missing LFS objects prevent opening the affected evidence files and passing
full verification. Markdown reading remains available.

## Read the knowledge library

Read Markdown under `lab/`, including the preserved mathematics, Crouzeix,
SICP and educational material. The Atlas reader and HTML export are retired.

### Use Obsidian

Open the repository root as a vault, not just `lab/`. Begin at
[Harp knowledge home](../lab/harp_knowledge_home.md). Root-qualified
wikilinks connect notes to their captured evidence. GitHub's Markdown view does
not provide all of Obsidian's navigation.

To install the optional reviewed profile, run from the repository root:

```sh
python3 tools/obsidian/apply_profile.py --vault "$(git rev-parse --show-toplevel)"
```

The installer is create-only by default. It preserves existing settings unless
you explicitly use `--replace`. Personal `.obsidian/` state is ignored by Git.

## Build the CLI

Prerequisites for the ordinary CLI build: Git, Mise, Python 3.9 or newer, and the
native C/C++ compiler and linker required by Rust dependencies.
[Durable coding work](durable-runtime.md#prepare-a-project) requires Python 3.11
or newer on both the Linux host and in its native rootfs. Review `mise.toml`
before trusting it, then run:

```sh
mise trust
mise install
mise run bootstrap
mise exec -- git lfs pull
mise run build
```

Mise installs the pinned Rust, Node, and Git LFS versions. Bootstrap configures
local LFS filters and fetches locked Cargo dependencies. These setup steps need network access.
They do not materialize research-source repositories.

The executable is `.build/harp-target/size/harp`. The examples below run it
directly from the repository root, so no PATH change or system installation is
needed.

### Validate and search

```sh
.build/harp-target/size/harp --version
.build/harp-target/size/harp check
.build/harp-target/size/harp build --check
.build/harp-target/size/harp search refresh
.build/harp-target/size/harp search status
.build/harp-target/size/harp search query "recursive improvement"
```

`check` validates the corpus. `build --check` compares the generated corpus
with its canonical inputs without rewriting it. `search refresh` builds the
local SQLite full-text index; rerun it after content changes.

For machine-readable output, put `--format json` before the command:

```sh
.build/harp-target/size/harp --format json search query "recursive improvement"
.build/harp-target/size/harp --help
```

Commands use schema-versioned JSON envelopes where documented. Durable coding
work retains provider stdout/stderr as attempt logs. The old `harp run` live
stream wrapper is retired; see [agent workflows](agent-workflows.md) and the
[durable operator guide](durable-runtime.md).

### Verify or refresh evidence

`harp sources verify` checks the local evidence records. It is a deeper check
than `harp check` and includes proof-evidence prerequisites. Prepare the Lean
environment described in [contributing](contributing.md#verification) first.

Fetching public implementation repositories is a separate, explicit operation:

```sh
.build/harp-target/size/harp sources materialize --source PI-MONO
```

Use `--all` instead of `--source PI-MONO` to materialize all registered
implementations. These commands use the network and write to ignored
`.sources/`. Tracked snapshots under `evidence/implementations/` remain the
offline evidence.

## Common setup problems

| Symptom | What to check |
| --- | --- |
| A PDF or image is a short text file beginning with `version https://git-lfs.github.com/spec/v1` | Run `mise exec -- git lfs pull` from the checkout. |
| A tool is missing or has the wrong version | Run `mise install`; use `mise run` or `mise exec --` so the pinned tools are on PATH. |
| A search has no index or reflects old content | Run `harp search refresh` using the built executable above. |
| `build --check` reports stale generated content | Follow the [content-change workflow](contributing.md#content-changes) to regenerate the headless corpus. |
| `lean-env`, proof-evidence checks, or the full gate fail on a fresh machine | The Lean cache is a separate prerequisite. Follow [verification](contributing.md#verification); do not run cache hydration as a routine repair. |

## Next steps

- [Agent workflows](agent-workflows.md) for live providers and restartable tasks.
- [Contributing](contributing.md) for focused tests, content ownership, and landing.
- [Native release candidates](releasing.md) for packaging and local installation.
