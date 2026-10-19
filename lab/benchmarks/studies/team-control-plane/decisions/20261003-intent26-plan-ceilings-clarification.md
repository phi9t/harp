# Ticket26: Plan ceilings apply to every caller

Commit read: `d54de5a2b843553dd9b3a6a888cb28b3422f001c`.

The independent [narrow architecture assessment](../evidence/intent26/repair3/independent-plan-architecture/report.md)
identifies a material conflict in the provisional repair3 Owner exception.
Ticket26 and v0.2 reconciliation separate strategy from authority and forbid
Plan edits changing cumulative limits. Owner authentication alone does not
make a larger Plan ceiling a permitted strategy edit. Unchanged mission
counters prove unchanged consumption, not unchanged admitted limits.

Preserve the conservative written contract: the latest immutable prior Plan's
resource, repair and effect ceilings constrain every caller, including Owner,
across source-only amendments. A cleared current Plan pointer does not remove
those constraints. Owner and delegated-lead unchanged/narrower plans remain
valid under their other admission checks. Expand hard authority/envelopes only
through a separately admitted authority transition; do not invent one in
plan.admit or implement scope27 in this repair.

This supersedes root's provisional interpretation that direct Owner plan.admit
could expand these fields inside an unchanged larger Intent. The earlier
branch632 decision's requirement for explicit Owner authority was a necessary
condition, not an override of the canonical Plan invariants. Preserve the
provisional source, test-expectation changes, runs and assessment as history.
Restore prior ceiling-refusal expectations, record actual RED against the Owner
bypass, then remove it within the ongoing final repair3 allocation.

The report is static and pins a stable final read; it does not verify the
correction or constitute frozen ticket conformance. Fresh source/evidence freeze,
both review axes, root checks and branch review remain required before acceptance.
