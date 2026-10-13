# Scope the local verification gate

Status: done
Owner: codex
Branch: codex/harp-surface-verification

## What to build

Make local verification select changed surfaces and relevant consumers. Preserve
complete release verification, update shared docs and workflow skills, and land
locally using focused checks.

## Acceptance criteria

- `verify` selects from the candidate diff and exposes a reviewable plan/base.
- Ordinary guidance changes execute without Lean/React prerequisites.
- Selected failures and unmapped surfaces block; no silent skipped checks.
- `verify-all` retains release coverage and release candidates always invoke it.
- Atlas verification no longer implicitly builds Lean proofs.
- Docs/skills describe relevant prerequisites and consumer checks.
- Focused behavior checks pass; unrelated work is preserved at local landing.

## Blocked by

None for this scoped candidate. Full release execution still requires its existing
frontend and proof prerequisites and is not claimed by a local landing.

## Verification

- Actual `mise run verify` passed: 10 dispatch-contract tests, 10 release-candidate
  behavior tests, diff checks, and changed documentation/skill validation.
- Fixtures cover docs-only execution, explicit Git bases, clean candidates,
  staged/untracked files, both rename surfaces, delivered HTML without frontend
  dependencies, affected Rust consumers, selected-command failure propagation,
  unknown-surface rejection, deleted docs, and broken local links.
- Five skill-creator validators passed. Mise parsed the actual 34-task inventory;
  TOML validation confirmed no task auto-install and separate local/release routes.
- The full release/proof/frontend gates were not run: this candidate changes gate
  routing and guidance, and does not claim release or proof readiness.
- Diff review preserves full release prerequisites and the primary’s unrelated
  sanitization edits. The local landing commit records this candidate.

## Comments

The owner authorized the change and local landing. Use the isolated candidate
rather than the dirty primary to select checks. Preserve the primary’s existing
sanitization configuration separately from the committed gate change.
