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

Use [durable coding work](durable-runtime.md) for current Codex/Trae execution.
The Corenius-adapted Python + SQLite journal owns admission, attempts, capacity,
cancellation, recovery, candidate verification and explicitly approved landing.
Insula and delegated cgroups provide subordinate containment and cleanup.
This is a custom replay runtime, with no Temporal server or PostgreSQL service.

The imported [component guide](../corenius/README.md) records current dispositions.
Old Rust controller/supervisor/gate/terminal-agent, Collab, agent worktree/landing,
lead/worker, tracker and coding benchmark launch paths are retired. Their commands
refuse before launching providers, opening legacy state or installing services.
Historical databases are preserved without automatic migration. The retained
Cargo workspace builds dependency-free retirement messages, not a second runtime.

Pure evidence parsers, protocol/contracts, snapshot helpers and offline benchmark
reference material remain available. Current intent is Git-managed Markdown;
Harp's root contributor instructions and repository-owned skills govern work.
The manifest records adapted files and removed active gates against original Git
objects. Captured evidence, licenses, branches and worktree revisions remain
preserved. PostgreSQL/Temporal source is recoverable from that history, with its
active build/service paths removed.

The [integration tickets](../.scratch/harp-runtime-integration/tickets.md) distinguish
completed milestones from pending qualification. Real Codex and Trae one-worker
runs passed independent checks and explicitly approved physical landings. A real
mixed pair also overlapped; its same-base proposals and fresh integration candidate
exercise serialized landing and stale-base refusal. Four-worker qualification and
the final composed repository gate remain the final acceptance work.

Math, proof work, Crouzeix, SICP, educational material and PDF publication remain
in Harp. Atlas and RSI-specific article reproduction are retired; a brief article
survey remains maintained.

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

## Retained local sanitizer

The existing Harp sanitizer is retained as an explicit CLI operation and optional
hook script. The hook checks staged Git blobs using an existing Harp binary; it
is not installed automatically. The repository gate does not scrub or require
rewriting the imported component or captured evidence. Review heuristic findings
in the intended authored surface before making edits.
