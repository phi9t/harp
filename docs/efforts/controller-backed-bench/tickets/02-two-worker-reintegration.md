# 02 two-worker-reintegration: Prove two-worker collaboration and reintegration

Type: task
Status: resolved
Owner: trae-controller-bench-02
Blocked by: 01
Claimed:

Triage: ready-for-agent

## Acceptance

- [x] Two initial useful worker Attempts start from the same canonical base with private complete repositories and scoped tools; measured useful-execution intervals overlap and both retain actual task candidates.
- [x] Normal lead/controller authority verifies and lands the first valid candidate; no fleet scheduler, ad hoc rebase/merge or direct state patch performs orchestration.
- [x] Preserve existing Candidate/proof acceptance semantics and require explicit stale-landing rejection; canonical freshness is enforced at conditional landing. No new stale-acceptance guard is required.
- [x] If required rejection semantics are unsupported, record an exact prerequisite blocker for separately scoped controller work; do not weaken assertions or hide product redesign in the adapter.
- [x] Admit at most one fresh integration Attempt from current canonical state, respecting at-most-two concurrent capacity including unresolved executions and normal cancellation/release authority.
- [x] Bind the fresh Candidate to fresh whole-task verification and acceptance; old proof cannot authorize changed Candidate/base content.
- [x] Final exact committed canonical snapshot passes both unchanged m01 tests in the independently restored pristine verifier view; omitting either ticket or tampering with tests cannot produce PASS.
- [x] Retain controller events, candidate/base identities, process timing, rejection evidence, proof/acceptance/landing records and final verifier evidence.
- [x] Keep deterministic and actual-model labels distinct; occupied slots or panes alone never prove useful overlap.

## Parent

Controller-backed benchmark dogfooding: first milestone specification, same durable effort. Source inspected at d7f674829e9844d011869b711d64fcf55313f7f1; implementation uses the reviewed integration revision containing completed blockers.

## What to build

A complete deterministic m01 mission produces useful concurrent worker candidates, refuses stale landing, reintegrates once through a new Attempt and passes the unchanged whole-task verifier.

## Exclusions

No provider qualification, automatic extra reintegration rounds, dependent-ticket tasks, textual-conflict task or broad recovery matrix.

## Verification

One real deterministic m01 run with stale-landing refusal and old-proof rejection for the fresh integration Candidate; exact committed-content verifier controls; useful-overlap intervals; fresh integration proof; waited owned-process evidence. No model calls.

## Authority and execution

Linux only. Use one writer per isolated worktree and normal controller role interfaces. Preserve existing work, consumed repair authority and historical evidence. Ticket publication authorizes no product execution, model spending, commits, pushes, pull requests or shared-ref landing; advance through the subsequent explicitly requested execution phase. Review repository Standards and parent Spec; a ticket result cannot broaden qualification beyond its stated scenario.

## Comments

- 2026-10-05T23:52:21Z coordinator: All acceptance criteria verified by fresh retained deterministic m01 run; independent reviews pass; required gate components pass after reviewed fixture repairs. See ../evidence/full-access-verification.md. No live-provider qualification.

## Owner amendment

The user explicitly chose existing acceptance semantics on 2026-10-05 during execution. Stale landing refusal and fresh integration proof replace the earlier stale-acceptance control. No product redesign is authorized by this amendment.
- 2026-10-05T18:02:48Z trae-controller-bench-02: claimed until 2026-10-05T22:02:48Z
- 2026-10-05T22:09:48Z trae-controller-bench-02: claimed until 2026-10-06T02:09:48Z
- 2026-10-05T23:52:30Z trae-controller-bench-02: resolved
