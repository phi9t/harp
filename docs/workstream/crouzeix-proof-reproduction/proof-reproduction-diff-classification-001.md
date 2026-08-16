# Crouzeix Proof-Reproduction Diff Classification 001

## Claim Ceiling

This document classifies source-branch diff groups for reconciliation. It is
not a proof claim, not a runtime receipt, and not approval for live provider
execution.

## Compared Revisions

- Base: `master` at `8dce1b0f54c5b8f65772eb83b4ce7cbc6decc0a3`
- Source: `feat/crouzeix-proof-reproduction` at
  `2799028625932936799d0b9d12b590645150aa98`
- Source worktree dirty state:
  `docs/import-receipt.md` and
  `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Divergence: `116 60`

## Classification Summary

| Group | Disposition | Reason | Minimum verification |
|-|-|-|-|
| `CONTEXT.md` | `superseded` | The proof branch glossary additions are already represented on current `master`; no reconciliation delta remains for this path. | `git diff --exit-code master feat/crouzeix-proof-reproduction -- CONTEXT.md` |
| `docs/import-receipt.md` | `adapt` | The proof-branch receipt and dirty receipt are payload-specific and stale relative to current `master`; refresh only after tracked reconciliation payloads settle. | `cargo run -q -p harp -- repository verify` |
| `docs/workstream/crouzeix-proof-reproduction/design.md` | `superseded` | The branch design file has no current-master delta and should not be reopened without a specific classified follow-up. | `git diff --exit-code master feat/crouzeix-proof-reproduction -- docs/workstream/crouzeix-proof-reproduction/design.md` |
| `docs/workstream/crouzeix-proof-reproduction/tracker.org` | `blocked` | The committed proof branch is behind current `master`, and the dirty tracker hunk adds newer `expert-frontier-002` blocked evidence that refers to ignored run state not yet validated on the integration branch. | `PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org` |
| `proof-agent-amendment-001..004.md` | `adapt` | These raw files contain unique baseline facts, but the durable surface must be the normalized trace index/capture, not raw amendment files. | amendment reviewer evidence plus `rg` checks on capture docs |
| `proof-agent-amendment-005..012.md` | `superseded` | Current `master` owns durable capture for the active `005..012` handoff; raw files must not re-land. | compare to `agent-trace-index.md` and `proof-agent-amendment-capture-001.md` |
| retrospective and old worktree-inventory docs | `superseded` | `retrospective-001.md` and the old `worktree-inventory.md` describe stale branch and dirty-state facts; current `master` owns normalized inventory, retirement, and capture docs. | `git diff --name-status feat/crouzeix-proof-reproduction..master -- docs/workstream/crouzeix-proof-reproduction` |
| Crouzeix lab runtime modules | `adapt` | Many runtime modules are byte-identical or already newer on `master`; CPFR-R014-specific readiness/pre-provider validation is useful but cannot be blindly ported. | `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v` |
| Crouzeix lab schemas | `adapt` | Most schemas are compatible, but `formal_attempt_receipt.schema.json` and formal receipt semantics are superseded by current v2 FormalReceipt contracts. | formal receipt and schema-focused unit tests |
| Crouzeix lab prompts | `adapt` | Most proof-branch prompts exist on `master`, while current `master` adds Lean/reconstruction/routing prompts; preserve current prompt set and adapt only referenced missing text. | prompt-boundary runner tests |
| Crouzeix lab tests | `adapt` | Branch tests cover initial runtime machinery, while current `master` adds FormalTarget, Jin, LS, kernel, reconstruction, and blind-frontier tests; keep useful assertions only after adapting to current interfaces. | full Crouzeix `unittest` discovery |
| branch formal-target/Jin planning docs | `superseded` | The old branch planning/spec docs are superseded by current `master`'s landed CPFR-070 through CPFR-081 implementation and active design docs. | diff current formal-target/Jin files and tests |
| baseline and runtime-ticket evidence paths | `blocked` | The branch diff adds no tracked evidence path, and dirty tracker facts refer to ignored `.runs/expert-frontier-002` material that must be validated before any tracker claim lands. | `PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-runtime labs/crouzeix_proof_reproduction/.runs/expert-frontier-002` |
| generated or receipt artifacts | `adapt` | Receipts and generated outputs must be produced from the current reconciled tree, not copied from the source branch. | `cargo run -q -p harp -- repository verify` |

## Subagent Evidence

- Inventory reviewer: current `master` is `8dce1b0`; proof source is
  `2799028625932936799d0b9d12b590645150aa98`; divergence is `116 60`;
  source tracker validation passes; the proof worktree is dirty only in
  `docs/import-receipt.md` and `tracker.org`; prior inventory facts about
  `71d46df`, `101 56`, untracked amendments, and unlanded RSI work are stale.
- Diff classifier: no required group is safe to land as-is. Lab modules,
  schemas, prompts, tests, receipts, and generated artifacts require
  adaptation; raw amendments and old inventory/retrospective docs are
  superseded; tracker and runtime-ticket evidence are blocked until the dirty
  `expert-frontier-002` evidence is validated.
- Amendment reviewer: amendments `001..004` contain unique active facts about
  ignored run evidence, flat-baseline route-family collapse pressure, recovered
  failed control-plane roots, pristine roots, and pre-provider local validation
  failures. Early resume/recovery instructions are superseded by CPFR-R014.
  Amendments `005..012` are represented by current `master` capture, though the
  capture should explicitly include the `expert_runner.py` repair seam and the
  Amendment 012 stop-review rule.
- Runtime compatibility reviewer: many Crouzeix lab modules are byte-identical
  between proof source and current `master`, but `formal_receipt.py`,
  `prepare_frontier.py`, `protocol.py`, `tickets.py`, `expert_runner.py`, and
  `run_frontier.py` need careful adaptation. Current `master`'s CPFR-070..081
  FormalTarget/FormalReceipt/Jin/LS/kernel/reconstruction/blind-frontier
  contracts supersede older branch formal-lane surfaces. CPFR-R014's
  pre-provider scope validation and read-only report idea is useful, but the
  whole root-recovery CLI must not be ported unchanged.

## Port Order

1. Update amendment index/capture for amendments `001..004` without re-landing
   raw amendment files.
2. Treat branch runtime modules as evidence. Port only missing, compatible
   assertions or code after comparing against current `master`.
3. Reconcile CPFR-R014 as a provider-free report/pre-provider validation
   contract against current `master`; do not revive stale root recovery.
4. Validate or quarantine the dirty `expert-frontier-002` tracker evidence.
5. Produce a read-only CPFR-032 gate report only after CPFR-R014 is reconciled.

## Blocked Or Deferred Work

| Item | Reason | Revisit condition |
|-|-|-|
| Live CPFR-032 provider execution | Requires reconciled CPFR-R014 and read-only gate report. | CPFR-032 gate report explicitly allows it and owner approves a separate execution plan. |
| Dirty CPFR-032 `expert-frontier-002` tracker claims | They refer to ignored run state and a pre-provider digest mismatch that has not been validated on the integration branch. | Runtime validation and read-only report evidence pass on the integration branch. |
| Worktree cleanup | Separate owner approval required. | Baseline reconciliation has landed and verification evidence is complete. |
| Full Lean release gate | `lake` unavailable on this machine. | Lean toolchain available. |

## Verification Commands

```sh
git diff --name-status master...feat/crouzeix-proof-reproduction
git -C <proof-source> status --short --branch --untracked-files=all
PYTHONDONTWRITEBYTECODE=1 python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v
cargo run -q -p harp -- repository verify
```
