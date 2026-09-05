# Getting started

[Documentation](README.md) / Getting started

Open the [Harp Atlas](https://phi9t.github.io/harp/) to start reading in your
browser. No clone or build is needed. To read offline or use the CLI, follow the
setup below. Running the
full contributor gate also needs the pinned Lean toolchain and a warm
dependency cache.

## Get the repository

```sh
git clone https://github.com/phi9t/harp.git
cd harp
```

Captured PDFs, images, and some archives use Git LFS. If Git LFS was already
installed and configured when you cloned, Git normally downloads those objects.
Otherwise, follow the tool setup below and run `mise exec -- git lfs pull`.
Missing LFS objects do not prevent reading the self-contained Atlas, but they
do prevent opening the affected evidence files and passing full verification.

## Read the Atlas

Use the [hosted reader](https://phi9t.github.io/harp/), or open
`atlas/dist/harp-atlas.html` from the cloned checkout in your browser.
On macOS:

```sh
open atlas/dist/harp-atlas.html
```

The single file contains the compiled corpus, scripts, and styles. It does not
need a development server. External source links still need a network
connection; the export does not bundle every file under `evidence/`.
The hosted reader also excludes those files. For repository-relative evidence
locators, use a local checkout or browse the
[source repository](https://github.com/phi9t/harp).

The home page offers four reading paths: RSI research, durable execution,
mathematics and Lean, and evaluation and evidence. Use Library to search
compiled titles and topic tags. More contains system comparisons, the Weng
reader, lessons, diagnosis tools, and Workstreams.

Within an article, open In this reading to jump to a section. Provenance and
checksums remain available in expandable panels; source folds stay closed
until opened. Wide code blocks, schematics, tables, and equations scroll
within the reading column on small screens.

Workstreams shows the checked-in reference snapshot of research and proof
work, not current process status.

### Use Obsidian instead

Open the repository root as a vault, not just `knowledge/`. Begin at
[Harp knowledge home](../knowledge/harp_knowledge_home.md). Root-qualified
wikilinks connect notes to their captured evidence. GitHub's Markdown view does
not provide all of Obsidian's navigation.

To install the optional reviewed profile, run from the repository root:

```sh
python3 tools/obsidian/apply_profile.py --vault "$(git rev-parse --show-toplevel)"
```

The installer is create-only by default. It preserves existing settings unless
you explicitly use `--replace`. Personal `.obsidian/` state is ignored by Git.

## Build the CLI

Prerequisites: Git, Mise, Python 3.9 or newer, and the native C/C++ compiler and
linker required by Rust dependencies. Review `mise.toml` before trusting it.
Then run:

```sh
mise trust
mise install
mise run bootstrap
mise exec -- git lfs pull
mise run build
```

Mise installs the pinned Rust, Node, and Git LFS versions. Bootstrap configures
local LFS filters, fetches locked Cargo dependencies, and installs the pinned
pnpm dependencies through Corepack. These setup steps need network access.
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

Most commands use a schema-versioned JSON envelope. Live `harp run` is the
exception: provider stdout stays byte-exact, with Harp lifecycle records on
stderr. See [agent workflows](agent-workflows.md).

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

## Work on the reader

After tool setup, start the Atlas development server:

```sh
cd atlas
mise exec -- corepack pnpm run dev
```

Follow the local URL printed by Vite. The development command compiles the
corpus first. To regenerate and test the single-file export, use
`mise exec -- corepack pnpm run test:export` from `atlas/`.

## Common setup problems

| Symptom | What to check |
| --- | --- |
| A PDF or image is a short text file beginning with `version https://git-lfs.github.com/spec/v1` | Run `mise exec -- git lfs pull` from the checkout. |
| A tool is missing or has the wrong version | Run `mise install`; use `mise run` or `mise exec --` so the pinned tools are on PATH. |
| A search has no index or reflects old content | Run `harp search refresh` using the built executable above. |
| `build --check` reports stale generated content | Follow the [content-change workflow](contributing.md#content-changes) to regenerate the corpus and Atlas together. |
| `lean-env`, proof-evidence checks, or the full gate fail on a fresh machine | The Lean cache is a separate prerequisite. Follow [verification](contributing.md#verification); do not run cache hydration as a routine repair. |

## Next steps

- [Agent workflows](agent-workflows.md) for live providers and restartable tasks.
- [Contributing](contributing.md) for focused tests, content ownership, and landing.
- [Native release candidates](releasing.md) for packaging and local installation.
