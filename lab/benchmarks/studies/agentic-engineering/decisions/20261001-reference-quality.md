# Decision: Corenius as an executable engineering reference

Date: 2026-10-01.
Authority: the owner accepted the three recommendations in the `ask-matt`
design discussion with "yes".

## Accepted direction

1. Serve the owner's workspace first. Document required tools, configuration,
   credentials and host assumptions explicitly. Portability to an unrelated
   fresh clone is a separate acceptance milestone.
2. Own guidance plus a maintained executable reference workflow. Codex and
   Claude leads follow a shared contract, with small harness-specific adapters
   and traecli workers. Expand the Rust orchestration runtime when demonstrated
   requirements justify it.
3. Demonstrate the first reference-quality milestone through both Codex and
   Claude leads independently completing the same representative effort:
   planning, parallel worker dispatch, rejection of defective output,
   verification, preparation of a gated landing, dehydration, and resumption
   in a fresh session. Guidance explains the workflow and reconciles stale
   claims found in the session export.

## Why this direction

Guidance alone can drift from actual behavior. A runtime-first program would
expand implementation before establishing which practices work. Guidance with
an executable example builds on the existing toolbox and makes its claims
checkable. Demonstration by both lead harnesses tests the shared contract.

## Boundaries and pending decisions

This approves the direction, not an implementation spec or execution plan.
Launch/landing authority, independent verification, scenario selection and
evidence-retention criteria remain design questions. Existing policy and Git
authority continue to apply. The owner has not authorized commits, pushes or
mainline integration in this discussion.

The design continues in `../tickets/07-reference-quality-design.md` within
the existing effort. Historical roadmap/spec artifacts remain historical
inputs; the new design will use tracker specs and blocker-linked tickets.

## Evidence and limits

Repository inspected: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Primary inputs: the owner's current request; the Claude session export
`2026-10-01-013755-this-session-is-being-continued-from-a-previous-c.txt`
(untracked in the primary checkout); `bench/RESULTS.md`; archived aggregate
results; this effort's existing state and decisions; and the canonical
workflow and skill crosswalk. The export includes collapsed tool calls, so
its narrated verification is not treated as independently reproduced proof.

Observed record conflicts to reconcile during specification: the archived
small-task rows total 39 passes in 40 runs despite a closing claim of 40/40;
the Milano reference retains a writability judgment later retracted in the
policy-defaults decision; and effort state mixes historical feature status
with later updates. This record does not adjudicate the Milano execution
boundary; that requires tracing the relevant paths at their pinned commits.
