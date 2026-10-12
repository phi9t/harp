# Collab Ops safety follow-up

Status: approved follow-up for tickets 20-25; qualification remains pending.
Read commit: `3c812cf1a11f57d75c6bfd0a6965b7cdb1142478`.
Combined source with existing mainline cleanup fixes: `365876ed00be3cdd50f3baebfdb06378e1423e39`.

The implementation covers the 18 original tickets. The default gate and fresh
Standards/Spec reviews passed, but independent branch review found four blockers.
The architectural review reproduced them in nine public CLI scenarios. See
[evidence/architecture-review.md](evidence/architecture-review.md) and its retained
observations and reproduction script. These results do not establish readiness.

On 2026-10-12 the user explicitly requested: “let's commit what we have now, land
them on mainline, and draft up follow up work”. This authorizes committing and
fast-forwarding the current snapshot despite the recorded findings. It does not
resolve the findings, accept the original tickets, approve the proposed design,
or extend the exhausted three-iteration implementation/review budget. No Git
remote is configured; this landing is local.

## Combined-mainline qualification

The merged source at `365876ed` ran 170 Collab tests: four archive/sweep
failures remain. Existing mainline commit `e69d9ce` adds an extraction filter
argument unavailable on this host Python. Ticket 25 covers compatibility without
weakening archive safety. See `evidence/mainline-combined-validation.json`.
The earlier 169-test pass applies to frozen source `3c812cf`, not this merge.

## Proposed contract

Deepen the existing `landing_protocol.py` module so every managed trunk mutation
uses the same lock order, pending-operation admission and checkout preflight.
Core and legacy remain the existing operation-record adapters. A caller cannot
exempt another operation during admission or recovery. Preserve the public CLI,
strict core cleanliness and legacy disjoint-edit compatibility.

Approve the scope explicitly: include retained toolbox writers (`lead_launch`
claims/releases and effort record commits), or limit the guarantee to Collab and
record the adjacent bypass. Do not redesign the controller or migration system.

Return explicit liveness decisions with evidence. A supplied unusable run binding
is unknown and retains workspaces; an explicitly unbound Attempt keeps the
existing manual confirmation contract. Consumers use the decision consistently.

Reconcile three effects separately: remote publication, local publication and
record repair. Establish exact attributed local success before remote I/O.
Repair established local records offline and idempotently; retain uncertainty
when a pushed Integration commit still needs a local fast-forward.

## Draft tickets and order

| Ticket | Outcome | Blocked by |
| --- | --- | --- |
| [19](tickets/19-mutation-design.md) | Approve shared interface, scope and bounded repair budget | — |
| [20](tickets/20-mutation-fence.md) | Fence admission, allocation recovery and all approved trunk writers | 19 |
| [21](tickets/21-checkout-preflight.md) | Share Git-operation and checkout preflight before push | 19 |
| [22](tickets/22-liveness-evidence.md) | Retain supplied worker bindings with unusable evidence | 19 |
| [23](tickets/23-local-record-recovery.md) | Repair exact local success while remote is offline | 19 |
| [25](tickets/25-archive-python-compat.md) | Restore archives on supported Python versions | — |
| [24](tickets/24-safety-qualification.md) | Clear behavioral and independent review gates | 20, 21, 22, 23, 25 |

Ticket 19 approved the shared mutation contract, all-managed-writers scope,
liveness contract, three-effect recovery contract and bounded repair/review
budget in
[decisions/2026-10-12-mutation-contract.md](decisions/2026-10-12-mutation-contract.md).
Existing original ticket claims and acceptance records are preserved.

## Verification

Use the already approved public CLI and compatibility-shim seams with real Git
repositories. Turn the nine retained defect reproductions into red-green tests;
add the complete acceptance scenarios in the architectural report. Check all
operation markers, conservative liveness consumers, inherited child locks and
exact-operation recovery, including record repair without remote access.

After focused proof passes, run affected toolbox/shim suites, Collab tests, lint
and the full default gate. Require fresh Standards, Spec and independent branch
reviews without blockers before marking the effort accepted. Record a new
explicit repair budget; the earlier three rounds are exhausted.
