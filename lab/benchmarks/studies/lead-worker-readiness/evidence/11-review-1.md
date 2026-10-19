# Lead review 1 of ticket 11: one change, then wait for my word to re-integrate

Reviewed Candidate: the tip of your branch at 11:16Z. The ticket section, its lint rules, the `add-finding`
and `findings` commands, and the findings in the brief and the launch record are right and stay.

## 1. Findings are collected only from ticket files, but in this effort they are written somewhere else (blocking; a correction to the ticket from what the run has taught)

The ticket said a worker leaves a finding "in its own ticket". In the run itself workers are forbidden to
edit their ticket file (the lead owns it), so every finding so far was written in two other places, and
your launch command reads neither:
- each worker's report, `docs/efforts/<effort>/evidence/NN-report.md`, in a section headed
  `## For later tickets`, as lines `- to NN: text`;
- one file kept by the lead, `docs/efforts/<effort>/evidence/lead-handoffs.md`, whose lines of the same
  form may appear anywhere in the file.
Look at this effort's own evidence directory on your branch for real examples of both.

Required:
- Collection for a target ticket gathers `- to NN: text` lines from three sources: the
  `## For later tickets` section of every ticket file (what you built); the same section of every
  `evidence/*-report.md`; and every such line in `evidence/lead-handoffs.md`. Each finding carries its
  source: a ticket id, a report's ticket id, or `lead`.
- `tracker.py findings` lists all three; the brief shows the source of each; the launch record stores them.
- Lint: a malformed line inside a report's `## For later tickets` section or in `lead-handoffs.md`, and an
  unknown target id there, are reported like the ticket-file ones. `none` as the only content of a
  report's section is valid and yields nothing.
- A finding identical in text and target from two sources appears once, with both sources named.
- Tests for each source, the de-duplication and the lint cases, and extend your scenario case so one
  finding comes from a report and one from the lead file.
- Update the ticket-format contract to describe the three places.

## Protocol

New commits on top of your current tip; no rewriting. Update the report. When this is done and your gate
passes, commit and then STOP. Do not make the `ticket 11: ready for review` commit yet: ticket 12 lands
before you and changes `tools/agents/lead_launch.py` and the harness module, so your branch will be stale.
Wait for my message telling you to re-integrate; the marker commit comes after that rebase.
