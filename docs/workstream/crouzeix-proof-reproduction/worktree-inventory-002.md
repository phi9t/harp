# Crouzeix Proof Worktree Inventory 002

## Claim Ceiling

This inventory records current proof-related branch and worktree state. It is
not a proof claim, not a Lean validation result, not approval to run live proof
agents, and not approval to delete worktrees or branch refs.

## Baseline

- Primary checkout: `<harp-root>`
- Inventory date: `2026-08-16`
- Current `master`: `8dce1b0f54c5b8f65772eb83b4ce7cbc6decc0a3`
- Proof source branch: `feat/crouzeix-proof-reproduction`
- Proof source head: `2799028625932936799d0b9d12b590645150aa98`
- Proof source divergence from `master`: `116 60`
- Proof source merge base:
  `9ca3f3a22812346a4b26c5f976298758fac49563`
- Proof source tracker validation:
  `PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org`,
  exit `0`.

## Source Worktree Status

The proof source worktree is not clean. Its current tracked dirty files are:

```text
## feat/crouzeix-proof-reproduction
 M docs/import-receipt.md
 M docs/workstream/crouzeix-proof-reproduction/tracker.org
```

No untracked files were reported with `--untracked-files=all`.

The dirty tracker hunk records a newer CPFR-032 control-plane state than the
committed proof source head:

- stale `expert-frontier-001` is preserved as superseded control-plane
  evidence;
- fresh `expert-frontier-002` was prepared with current ticket boundaries;
- root execution stopped before provider invocation with
  `protocol.ValidationError: ticket context_sha256 does not match provider context`;
- no root provider call executed on `expert-frontier-002`;
- no root ticket was consumed to `running`;
- no admission, mathematical node, frontier event, evaluator phase, generation
  phase, finalization phase, or CPFR-033 action occurred; and
- CPFR-032 remains blocked on the fresh-run pre-provider digest-boundary
  mismatch.

These dirty facts are live evidence for classification, not a direct landing
surface. They require validation before any tracker claim is ported.

## Stale Prior Inventory Facts

| Prior statement | Current correction | Evidence |
|-|-|-|
| Baseline commit is `1d70062ba047...` | Current `master` is `8dce1b0f54c5b8f65772eb83b4ce7cbc6decc0a3`. | `git rev-parse master` |
| Proof source head is `71d46df...` | Proof source head is `2799028625932936799d0b9d12b590645150aa98`. | `git rev-parse feat/crouzeix-proof-reproduction` |
| Proof source divergence is `101 56` | Current divergence is `116 60`. | `git rev-list --left-right --count master...feat/crouzeix-proof-reproduction` |
| Proof source has raw untracked amendments `005` through `012` | Current proof source has no untracked files; raw amendments are tracked in the source diff and superseded for direct landing by the normalized capture on `master`. | `git -C <proof-source> status --short --untracked-files=all` |
| RSI lane still needs landing | RSI lane landed on `master` at `8dce1b0`. | `git log --oneline --decorate --max-count=5` |
| Crouzeix capsule work is a pending RSI lane item | Capsule work remains blocked for a future adapter; it is not part of this baseline proof-reconciliation lane. | `worktree-inventory-001.md` and current `master` |

## Active Reconciliation Source

`feat/crouzeix-proof-reproduction` is the only active proof-reproduction source
for this phase. Worker branches are evidence only unless a later
classification finds missing commits.

The current source branch top commits are:

```text
2799028 Record CPFR-032 typed blocker
6bd5b7a chore(crouzeix): complete CPFR-R014
7eccf62 docs(crouzeix): repair retrospective receipt
941bfbd docs(crouzeix): land retrospective trace inventory
71d46df chore(crouzeix): block cpfr-032 after root recovery
3991a5a chore(crouzeix): record CPFR-032 readiness blocker
6277b65 fix(crouzeix): close expert frontier readiness gaps
343e6de feat(crouzeix): prepare expert frontier run
```

The earlier plan expected `6bd5b7a` as the source head. That commit is now the
second commit behind `2799028`; current disk and Git state are authoritative.

## Worktree Disposition

| Worktree | Branch | Head | Status | Disposition |
|-|-|-|-|-|
| `<harp-root>/.worktrees/crouzeix-baseline-reconciliation` | `integrate/crouzeix-baseline-reconciliation` | `8dce1b0f54c5b8f65772eb83b4ce7cbc6decc0a3` | clean at creation | current integration lane |
| `<harp-root>/.worktrees/crouzeix-proof-reproduction` | `feat/crouzeix-proof-reproduction` | `2799028625932936799d0b9d12b590645150aa98` | dirty tracked receipt and tracker; no untracked files | active source; reconcile through this integration branch |
| `<harp-root>/.worktrees/crouzeix-formal-validation` | `codex/crouzeix-formal-validation` | `1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf` | contained in `master` | candidate for later retirement only |
| `<harp-root>/.worktrees/crouzeix-conjecture-deep-dive` | `integrate/crouzeix-conjecture` | `127c0e0474ee3cc534ee7ad4f2ccf38b681e65c5` | contained in `master` | candidate for later retirement only |
| `<harp-root>/.worktrees/crouzeix-multi-worktree-landing-design` | `docs/crouzeix-multi-worktree-landing-design` | `73eeaab4c117d63df277a54406f6d2472c540475` | contained in `master` | candidate for later retirement only |
| `<harp-root>/.worktrees/crouzeix-rsi-integration` | `integrate/crouzeix-rsi-current` | `8dce1b0f54c5b8f65772eb83b4ce7cbc6decc0a3` | contained in `master` | candidate for later retirement only |
| `<harp-root>/.worktrees/crouzeix-rsi-workflow-design` | `codex/crouzeix-rsi-workflow-design` | `530e0edeb1f22eade7888552df7f5106aec48276` | contained in `master`; quarantined worktree state previously observed | no direct merge or cleanup |

Branch-level merged checks also report these Crouzeix branches contained in
current `master`:

- `codex/crouzeix-formal-validation`
- `codex/crouzeix-rsi-workflow-design`
- `docs/crouzeix-multi-worktree-landing-design`
- `integrate/crouzeix-baseline-reconciliation`
- `integrate/crouzeix-conjecture`
- `integrate/crouzeix-rsi-current`

## Remaining Owner Decisions

1. Whether to retire contained worktrees after baseline reconciliation.
2. Whether to delete stale branch refs after all proof work is landed.
3. Whether to keep or remove unrelated untracked prompt docs in the primary
   checkout.
4. Whether the dirty CPFR-032 `expert-frontier-002` evidence should be
   validated and captured in this reconciliation lane or left as blocked source
   evidence for a later live-frontier plan.

## Verification Commands

```sh
git status --short --branch
git worktree list --porcelain
git rev-parse master
git rev-parse feat/crouzeix-proof-reproduction
git rev-list --left-right --count master...feat/crouzeix-proof-reproduction
git merge-base master feat/crouzeix-proof-reproduction
git log --oneline --decorate --max-count=8 feat/crouzeix-proof-reproduction
git -C <proof-source> status --short --branch --untracked-files=all
PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
```
