# 03 bounded-outcomes: Enforce bounded runs and report honest outcomes

Type: task
Status: claimed
Owner: trae-controller-bench-03
Blocked by: 02
Claimed: 2026-10-07T08:15:09Z expires 2026-10-07T12:15:09Z

Triage: ready-for-agent

## Acceptance

- [ ] Fix configuration and validation for live execution clock: 30 minutes from durable run admission immediately before first mission submission, including post-admission model startup and final verification; setup and cleanup are separately measured.
- [ ] Define finite setup-stage and cleanup timeouts before running; preserve pinned verifier timeout and report actual dispatch cutoff. Failed preflight consumes no model calls.
- [ ] On timeout or provider failure stop new dispatch through normal authority, attempt bounded owned-process reconciliation and record any remaining uncertain identities; never infer physical stop from timeout alone.
- [ ] Introduce a versioned controller result integrated with validation, loading and reporting while preserving existing solo/fleet records and interpretations.
- [ ] Record task verdict, orchestration verdict, effect disposition, failure stage, interventions, Attempts and setup/execution/cleanup timing independently. A known FAIL remains visible when effects are UNRESOLVED.
- [ ] Exercise missing prerequisites, unsupported tasks, delivery failure, failed verifier, timeout, provider failure, missing evidence and manual intervention: none may yield autonomous PASS.
- [ ] Manual rescue prevents autonomous PASS; no automatic repeat after timeout/provider failure and no capacity-release bypass or retry beyond the accepted fresh integration Attempt.
- [ ] Retain successful and failed repositories, manifests, events, proofs, logs/digests and process disposition; public summaries exclude credentials while durable result pointers/content identities survive scratch cleanup.
- [ ] Model usage is reported only as measured with completeness labels; missing totals remain unknown. Fleet textual conflicts and controller stale refusals are separate counters, with no one-run speed/cost superiority claim.
- [ ] Existing benchmark result/report and tmux tests remain meaningful; add controlled failure fixtures without claiming broad recovery qualification.

## Parent

Controller-backed benchmark dogfooding: first milestone specification, same durable effort. Source inspected at d7f674829e9844d011869b711d64fcf55313f7f1; implementation uses the reviewed integration revision containing completed blockers.

## What to build

Operators get bounded execution and reconciled retained outcomes; benchmark reports distinguish task correctness, orchestration correctness and unresolved effects while continuing to read existing fleet results.

## Exclusions

No general financial budgets, automatic trial repeat, production cleanup authority, new providers or broad crash/restore qualification.

## Verification

Deterministic deadline/failure/unknown/intervention tests through run lifecycle and report loading; mixed legacy/controller record reporting; bounded cleanup and retained-process evidence. No live-provider run required.

## Authority and execution

Linux only. Use one writer per isolated worktree and normal controller role interfaces. Preserve existing work, consumed repair authority and historical evidence. Ticket publication authorizes no product execution, model spending, commits, pushes, pull requests or shared-ref landing; advance through the subsequent explicitly requested execution phase. Review repository Standards and parent Spec; a ticket result cannot broaden qualification beyond its stated scenario.

## Comments
- 2026-10-05T23:52:30Z trae-controller-bench-03: claimed until 2026-10-06T03:52:30Z
- 2026-10-07T08:15:09Z trae-controller-bench-03: claimed until 2026-10-07T12:15:09Z
