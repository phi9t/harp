# Effort: lead-worker-readiness

Status: integration candidate under independent review; implementation tickets resolved except ticket 14 live trial and ticket 15 TraeCLI parity; no full live-trial qualification claimed.
Base: main
Integration branch: effort/lead-worker-readiness
Record branch: effort/lead-worker-readiness-records
Lead: interactive session (Claude)

## Intent

Make Corenius's own lead and worker workflow good enough to run a multi-ticket effort with parallel
workers, and say exactly what "good enough" means. Readiness is one set of scenarios; three instantiations
of the idea (lead-driven, local controller, workflow controller) are measured against it through the
benchmark harness. The motivation is the case study of a sibling repo running such an effort with an
improvised coordinator (`docs/practice/imported/sureal-semantic-layout.md`). Adoption by other repositories
comes after this effort and is not part of it.

## Scope and non-goals

In scope: the readiness scenarios in `SPEC.md`, their deterministic and live proof inside Corenius, and
the changes needed to pass them.

Not in scope: running any Corenius tool against a sibling repository, importing another repository's
tickets, replacing the sibling's coordinator, and anything that changes the accepted status or repair caps
of the `sqlite-controller` effort.

## Authority

The owner asked for this focus on 2026-10-05 ("getting corenius to be good enough first", with the case
study as background). Initial authority at that interview covered the case study update and specification
only. It was superseded by decisions/20261005-cleanup-review-land-and-trial.md.
The current user explicitly requested pending-work review, bounded fixes and local
fast-forward landing on 2026-10-07. This integration does not launch live trials.

## Classification

Gated path: it touches the worker hand-off contract, landing, and a security boundary (what a worker may
reach). Recorded before implementation as `docs/agents/agentic-engineering.md` requires.

Skill fallback: `to-spec` is a human-only command in this harness. Its process and template were applied
directly to produce `SPEC.md`; it was not invoked.

## Milestones and exit criteria

- [x] M1 The owner confirms the test seam (`decisions/20261005-seam-substrate-runtime-and-network.md`) and the three instantiations (`decisions/20261005-three-instantiations.md`).
- [x] M2 Independent critique of the spec (`evidence/critique-1.md`); findings reconciled in draft 2.
- [x] M3 The owner answers the open items of draft 2 (`decisions/20261005-eight-answers.md`).
- [ ] M4 Tickets with blocking edges exist under `tickets/` and `tracker.py lint lead-worker-readiness` is clean.
- [ ] M5 Every deterministic story passes, or is declared not applicable with its reason, for the lead-driven instantiation; the command and result are in `evidence/`.
- [ ] M6 One live trial of the lead-driven instantiation inside Corenius; every intervention is recorded; the record and the running notes are in `evidence/`.
- [ ] M7 The comparison table holds a result or "not yet run" for every scenario and instantiation.
