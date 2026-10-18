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

[Preservation inventory](01-preservation-inventory.md). The owner chose to include
all branches/worktrees. Local offline resolution lacks tokio-postgres in the
existing combined Corenius workspace; test on Linux using its existing cache
without adopting PostgreSQL/Temporal or installing dependencies.

## Verification

The initial pre-import probe was:

    cargo +1.95.0 test --offline --locked --manifest-path <snapshot>/Cargo.toml -p corenius --no-run

It exited 101: no matching package named tokio-postgres found. This is a local
cache/resolution limitation, not evidence of a migrated-code defect. The source
snapshot is inspection-only and is not yet an operational import.
