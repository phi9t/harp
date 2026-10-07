# 13 mixed-provider-faults: Complete mixed-provider and broad-fault qualification

Type: feature
Status: open
Owner:
Blocked by: 12
Claimed:

## Parent

`docs/efforts/controller-portfolio-stabilization/SPEC.md`

## What to build

Run one TraeCLI and one Codex worker concurrently under the TraeCLI Lead, then close the broader controlled-trial fault matrix without weakening per-provider bindings or controller-owned lifecycle.

## Acceptance
- [ ] Mixed workers share controller capacity and work-ledger semantics while retaining distinct provider bindings and isolated worktrees.
- [ ] Cross-provider messages, review, Candidate verification, stale-base handling and replacement cannot address the wrong provider thread or workspace.
- [ ] The remaining controlled faults cover view disconnect, controller restart, Lead replacement, lost start acknowledgement, cancel near submission, post-ref-update crash and work-ledger disconnect/replay.
- [ ] Boundary attacks fail without damage when an Attempt writes canonical refs, accesses another task repository or host secrets, signals the controller directly, or submits with revoked identity.
- [ ] All unsupported guarantees remain explicit and block corresponding readiness claims.
- [ ] Independent review finds no blocking defect and the final evidence binds exact Corenius, DevX, Insula, provider, host and Trial Repository identities.

## Comments
