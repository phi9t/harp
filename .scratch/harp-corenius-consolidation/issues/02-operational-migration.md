# Migrate the committed operational component

Status: in-progress
Owner: Codex
Parent: [consolidation intent](../spec.md)

## What to build

Bring the terminal agent, SQLite controller and collaboration tools into Harp.
Preserve component boundaries, source provenance, permissions and observable
CLI contracts. Replace current policy/documentation ownership with Harp while
retaining historical evidence without rewriting captured bytes.

## Acceptance criteria

- Native terminal-agent and controller tests execute against migrated code.
- Collaboration tools work against disposable standalone repositories.
- No operation requires the old checkout or activates Kata, sibling policy
  synchronization, a provider run, or a persistent service implicitly.
- The existing Harp CLI and corpus contracts remain intact.
- Changed-surface selection covers the new component and all affected consumers.

## Blocked by

Final reconciliation verification and local landing remain. The complete imported
candidate passed its required gate at 838c1ca5; the owner then authorized merging
the existing primary work. No PostgreSQL/Temporal service run is required.

## Verification

Against combined source 64f55c72 in a separate Linux test checkout:

- Terminal agent: cargo +1.95.0 test --offline --locked -p corenius: 71 passed.
- SQLite controller: same cargo command for corenius-sqlite-controller --lib
  --features fault-injection: 24 passed.
- Native lab parsers: 21 passed; Docker rootfs helpers: 6 passed.
- Python gate: 22/23 selected suites passed. The sole failure was the benchmark
  host-temp source policy; its bounded path adaptation subsequently passed
  the policy plus all 54 materializer and six rootfs-helper cases.
- Durable suite: 73 cases, including four explicit skips requiring delegated
  cgroups. No production cgroup/runtime qualification is claimed.
- Benchmark gate: all six selected suites passed (168 tests), including runner,
  fleet, materializer, tracked-task, validator and capsule contracts.
- Independent source review: all 260 dirty-file hashes match retained snapshots;
  no actionable lost work found across all 29 branch tips and 12 dirty captures.

Checks used existing toolchains/caches, disposable repositories and model-free
fixtures. Paid providers, live two-worker acceptance, delegated-cgroup checks,
full native controller integration and parked database/service labs were not run.
Original source checkouts and worktrees remain unchanged.

The owner authorized and completed prerequisite restoration: pinned PDF packages
and lockfile without browser downloads, plus the pinned Lean cache. The full
candidate gate passes; primary reconciliation is now in progress.
