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
