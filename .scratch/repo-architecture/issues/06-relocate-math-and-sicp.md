# 06: Put retained executable mathematics and SICP under lab

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Move formalization and proof companion programs into lab/math/ and the evaluator/executable education into lab/sicp/. Keep shared knowledge, contracts and captured evidence canonical, linked through explicit workload inputs. Update proof routes, Cargo membership, publication paths and documentation without changing mathematical claims.

## Acceptance criteria

- [ ] Existing Crouzeix theorem and route identities remain authoritative, with fresh relevant Lean build/receipt evidence.
- [ ] SICP evaluator behavior and existing educational consumers continue to work after Cargo/path migration.
- [ ] The PDF publication mechanism builds retained source through the new proof/program paths.
- [ ] Move canonical cache ownership deliberately; verify the existing warm-cache target and serialize shared Harp outputs. Never hydrate dependencies incidentally.
- [ ] No duplicate proof/source authorities or stale managed links; regenerated corpus agrees with canonical inputs.

## Blocked by

- [01-unify-durable-control-plane](01-unify-durable-control-plane.md)

## Verification

Preflight the pinned toolchain and existing cache without Lake, then run relevant Lean route/receipt checks, evaluator tests, managed link/corpus checks and PDF consumer checks. Use the selected gate, adding consumers affected by moved paths.

No implementation or qualification result is recorded yet.
