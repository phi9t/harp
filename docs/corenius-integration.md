# Corenius engineering subsystem

Corenius source now lives under `corenius/` in Harp. Its terminal agent, SQLite
controller, collaboration toolbox, benchmark fixtures, durable execution work,
engineering labs and design records are retained together. Normal use of this
source does not require the previous checkout.

The import includes all 29 original branch tips and source captures of all 13
registered worktrees. Twelve worktrees contained uncommitted source changes;
their complete captured revisions remain reachable in Git alongside the original
branches. Overlapping implementations were reconciled with the later fixes
retained. Git metadata, ignored build caches and native session stores were not
copied into the product. The original checkouts remain intact.

[The import manifest](../corenius/HARP_IMPORT.json) records branch identities,
worktree file hashes and the combined source commit. The preservation check
compares the imported files to retained Git objects and verifies their ancestry:

```sh
python3 scripts/verify_corenius_import.py
```

Captured artifacts retain their bytes and existing license records. Corenius
supplies no repository-wide license. Two dated sibling inventories remain in Git
history with portable source locators in their former locations; they are not
current Harp guidance.

## Use and ownership

The inherited [component guide](../corenius/README.md) maps the terminal agent,
controller and optional collaboration tools. Its Cargo workspace and lockfile
remain separate because it requires Rust 1.95 and a different SQLite binding
version from Harp's existing runtime. Use the already-installed pinned toolchain:

```sh
cd corenius
python3 tools/temp_root.py -- cargo +1.95.0 run --offline --locked -p corenius -- --help
```

The controller, collaboration CLI and durable process backend retain their Linux
requirements. Harp's root contributor guide and Harp-owned workflow skills govern
current repository work. Existing Kata adapters and sibling-policy tools are
retained source, not a change to Harp's Git-managed Markdown tracker. Do not run
sibling synchronization or initialize a second ledger as part of the import.

PostgreSQL/Temporal experiments and historical labs remain parked. They are not
the selected controller, and their service fixtures are not part of this import's
local runtime checks. The complete live two-worker baseline and native active-turn
recovery remain unqualified; preserving source does not establish those results.

Math, proof work, Crouzeix, SICP and other educational material remain in Harp.
The separate RSI retirement ticket preserves a brief article survey and removes
RSI-specific material without retiring those retained consumers.

## Consolidation checks

The combined source passed 71 terminal-agent tests, 24 controller unit tests,
21 native-evidence parser tests and all six selected benchmark suites on Linux
using existing offline caches. The benchmark suite covered 168 tests. A narrow
path repair then passed all 54 materializer tests, six rootfs-helper tests and
the source temp-path policy. The remaining selected Python suites passed;
four delegated-cgroup cases were explicitly skipped because no delegation was
provided. These are model-free import checks, not a live-system qualification.

The Harp-side Atlas is retired. Educational Markdown and the headless corpus
remain, alongside a brief survey of prior article reading. The discarded RSI
article archive is recoverable from Git history rather than maintained in the
current product.

## Referenced architecture discussions

The useful mechanisms in the referenced planner and architecture discussions
are already compatible with the inherited engineering direction: pin the task's
base and input identities; keep workers isolated; bind evidence to the exact
candidate revision; condition landing on the expected base; and retain uncertain
external effects for reconciliation instead of blindly retrying them. A serial
lead/worker loop is a useful starting operation without adding a service stack.

The inspected Trondheim planner demonstrates pure proposal generation from
pinned inputs and an observed frontier (11 model-free tests passed). It supplies
no execution or approval authority. That is a possible later adaptation, not a
dependency on a sibling checkout or a new prerequisite for Harp work. The
[retained engineering-profile work](../corenius/docs/efforts/engineering-profile-adoption/README.md)
and [labs](../corenius/docs/efforts/engineering-profile-labs/README.md) preserve
related upstream designs and bounded results. Older proposals for compulsory
PostgreSQL/Temporal, a second intent ledger, or amending every task into one
commit do not override current Harp instructions.
