# Focus tests on useful contracts

Status: done
Owner: codex
Branch: codex/harp-contract-testing

## What to build

Encode the owner’s rule of thumb: protect public behavior, API contracts, schemas,
and meaningful invariants; keep implementation choices free to evolve.

## Acceptance criteria

- Shared testing guidance explains test selection and handling incidental coupling.
- Planning, execution, and review skills refer to that guidance.
- Focused documentation checks pass and changes land locally.

## Blocked by

None.

## Verification

All five skill frontmatter validators passed; 20 local links resolved. Reviewed
the guidance against contract-preserving refactors, real contract regressions,
and meaningful internal invariants. Diff whitespace checks passed. The owner’s
existing docs-only gate applies; no runtime build is needed for this change.
The commit containing this ticket records the local landing.

## Comments

This task changes guidance. A broad audit or deletion of existing tests is outside
this change’s scope.
