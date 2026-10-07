## Built

Tracker tickets now support an optional `## For later tickets` section with
structured `- to NN: text` findings. The same finding grammar is also read from
worker reports in `evidence/NN-report.md` and from the lead's
`evidence/lead-handoffs.md` file. The tracker can append a finding to a ticket
and list all findings addressed to a ticket in text or JSON form.

Lead launch now carries addressed findings from all three sources into the worker
brief and records the same structured findings in the launch record. Identical
target/text findings appear once with every source named. The lead-driven
readiness scenario has an end-to-end case for stories 42 and 43.

## Acceptance

- "An optional ticket section holds findings as `- to NN: text`; lint accepts it, and reports a malformed line, an unknown target and a self target." Proven by `python3 tools/agents/test_tracker.py` exit 0, including `test_lint_validates_for_later_ticket_findings`; also by `python3 tools/agents/tracker.py lint` exit 0 on this repository.
- "A command appends a finding and a command lists the findings addressed to a ticket, with a JSON form." Proven by `python3 tools/agents/test_tracker.py` exit 0, including `test_findings_command_appends_and_lists_text_and_json` and `test_findings_command_reads_reports_lead_handoffs_and_deduplicates_sources`.
- "Launching the addressed ticket puts those findings, with their source ticket, in the brief; the launch record shows them." Proven by `python3 tools/agents/test_lead_launch.py` exit 0, including `test_launch_includes_findings_addressed_to_the_ticket`; by `python3 bench/harness/test_lead_driven.py` exit 0; and by the readiness scenario loop, where `launch-by-edges-and-triage handoffs-into-briefs` passed.
- "Claiming and resolving a ticket that has the section leave it intact; tickets without it stay valid." Proven by `python3 tools/agents/test_tracker.py` exit 0, including `test_claim_and_resolve_preserve_for_later_sections`; also by `python3 tools/agents/tracker.py lint` exit 0.

Review 1 acceptance:

- Collection now gathers findings from ticket files, `evidence/NN-report.md` sections, and `evidence/lead-handoffs.md`. Proven by `python3 tools/agents/test_tracker.py` exit 0, including `test_findings_command_reads_reports_lead_handoffs_and_deduplicates_sources`.
- `tracker.py findings` lists all three source types; the brief shows the source of each; the launch record stores them. Proven by `python3 tools/agents/test_tracker.py` exit 0 and `python3 tools/agents/test_lead_launch.py` exit 0, including `test_launch_includes_findings_addressed_to_the_ticket`.
- Lint reports malformed and unknown-target findings in report sections and `lead-handoffs.md`; `none` as the only content of a report section is valid. Proven by `python3 tools/agents/test_tracker.py` exit 0, including `test_lint_validates_report_and_lead_handoff_findings`.
- A finding identical in text and target from two sources appears once, with both sources named. Proven by `python3 tools/agents/test_tracker.py` exit 0, including `test_findings_command_reads_reports_lead_handoffs_and_deduplicates_sources`.
- The readiness scenario case now includes one finding from a report and one from the lead file. Proven by `python3 bench/harness/test_lead_driven.py` exit 0 and the readiness scenario loop.

Gate commands run:

- `python3 tools/agents/test_tracker.py` -> exit 0, 17 tests OK.
- `python3 tools/agents/test_lead_launch.py` -> exit 0, 7 tests OK.
- `python3 -m unittest discover -s bench/harness -p 'test_*.py'` -> exit 0, 57 tests OK. It emitted repeated pre-existing `ResourceWarning: unclosed file` messages from `bench/harness/tasklib.py:90`; no test failed.
- For each directory under `bench/readiness/`, `python3 bench/harness/lead_driven.py <scenario> --out "$(mktemp -d)"` -> exit 0 for `access-profile`, `external-state-report`, `handoff`, `land-exact-candidate`, `land-stale-candidate`, and `launch-by-edges-and-triage`.
- `python3 tools/agents/tracker.py lint` -> exit 0.
- `git diff --check` -> exit 0.

RED evidence observed before implementation:

- `python3 tools/agents/test_tracker.py` failed because `add-finding` did not exist and lint accepted malformed later-ticket findings.
- `python3 tools/agents/test_lead_launch.py` failed with missing `lead_launch.findings`.
- `python3 bench/harness/test_lead_driven.py` failed the new `handoffs-into-briefs` case with `details=handoff-findings`.
- Review 1 RED: `python3 tools/agents/test_tracker.py && python3 tools/agents/test_lead_launch.py` failed because JSON rows had no grouped `sources`, report and lead-handoff findings were not listed, and evidence malformed/unknown-target findings were not linted.

## Claims

- [R] the tracker recognizes later-ticket finding lines with the `- to NN: text` shape `tools/agents/tracker.py:52`
- [R] parsed later-ticket findings retain source kind, source id, source slug, source label, target id, text, path and line `tools/agents/tracker.py:69`
- [R] the shared finding parser accepts `none` as the only content of a report section `tools/agents/tracker.py:207`
- [R] the shared finding parser reports malformed later-ticket finding lines `tools/agents/tracker.py:213`
- [R] ticket parsing reads the optional `## For later tickets` section with ticket source labels `tools/agents/tracker.py:323`
- [R] ticket loading reports unknown and self ticket-file later-ticket targets `tools/agents/tracker.py:378`
- [R] report finding loading reads direct `evidence/NN-report.md` files `tools/agents/tracker.py:413`
- [R] lead handoff loading reads `evidence/lead-handoffs.md` `tools/agents/tracker.py:440`
- [R] cross-source finding loading includes ticket, report and lead findings in one collection `tools/agents/tracker.py:461`
- [R] report and lead handoff findings with unknown targets are reported as lint problems `tools/agents/tracker.py:476`
- [R] `findings` returns grouped rows addressed to a requested target ticket `tools/agents/tracker.py:581`
- [R] `add-finding` inserts a `## For later tickets` section before `## Comments` when the section is absent `tools/agents/tracker.py:592`
- [R] the `findings` command prints all source labels for a grouped finding `tools/agents/tracker.py:752`
- [R] the tracker exposes `add-finding` as a CLI subcommand `tools/agents/tracker.py:909`
- [R] the tracker exposes `findings` as a CLI subcommand `tools/agents/tracker.py:917`
- [R] lead launch refuses malformed later-ticket findings before collecting launch findings `tools/agents/lead_launch.py:149`
- [R] lead launch writes addressed findings into the brief with every source label `tools/agents/lead_launch.py:219`
- [R] lead launch stores the addressed findings in `run.json` under `lead_launch.findings` `tools/agents/lead_launch.py:324`
- [R] the ticket format contract documents the optional `## For later tickets` section `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md:20`
- [R] the ticket format contract documents worker report and lead handoff finding sources `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md:39`
- [R] the lead-driven launch scenario now declares stories 42 and 43 `bench/readiness/launch-by-edges-and-triage/scenario.toml:2`
- [R] the readiness fixture has a source ticket leaving a finding to ticket 10 `bench/readiness/launch-by-edges-and-triage/environment/docs/efforts/demo/tickets/02-ready.md:19`
- [R] the readiness fixture has a report finding addressed to ticket 10 `bench/readiness/launch-by-edges-and-triage/environment/docs/efforts/demo/evidence/05-report.md:7`
- [R] the readiness fixture has a lead handoff finding addressed to ticket 10 `bench/readiness/launch-by-edges-and-triage/environment/docs/efforts/demo/evidence/lead-handoffs.md:3`
- [R] the readiness harness checks that the handoff finding sources and prompt text are observed `bench/harness/test_lead_driven.py:122`

## Outside declared paths

- `bench/harness/lead_driven.py`: extended the existing lead-launch readiness verifier so story 43 is observed in the benchmark seam.
- `bench/harness/test_lead_driven.py`: added the scenario assertion for the new handoffs-into-briefs case.
- `bench/readiness/launch-by-edges-and-triage/scenario.toml`: added stories 42 and 43 to the existing lead-launch scenario and a focused case.
- `bench/readiness/launch-by-edges-and-triage/environment/docs/efforts/demo/tickets/02-ready.md`: fixture source finding used by the readiness case.
- `bench/readiness/launch-by-edges-and-triage/environment/docs/efforts/demo/evidence/05-report.md`: fixture worker-report finding used by review 1's readiness case.
- `bench/readiness/launch-by-edges-and-triage/environment/docs/efforts/demo/evidence/lead-handoffs.md`: fixture lead-handoff finding used by review 1's readiness case.
- `docs/efforts/lead-worker-readiness/evidence/11-report.md`: required hand-off report for this ticket.

## For later tickets

none

## Not checked

- `tools/agents/check.sh` was not run because the worker brief explicitly forbids it in this worktree.
- Cargo was not run because the worker brief explicitly forbids it.
- No real `traecli` or Codex worker was launched; tests and scenarios used fake/scripted binaries as required.

## Review round 1

Implemented the blocking finding in `/data02/home/philip.yang/.cache/corenius/readiness/lead-worker-readiness/11-handoffs-into-briefs/review-1.md`.

The correction extends collection beyond ticket files to `evidence/NN-report.md` and `evidence/lead-handoffs.md`, groups identical target/text findings under all source labels, lints malformed or unknown-target findings in those evidence files, treats `none` as an empty worker-report section, and extends the lead-driven scenario fixture to include report and lead sources.

Per the review protocol, the review-fix commit stopped without creating a new `ticket 11: ready for review` marker until the lead told this worker to re-integrate after ticket 12.

## Reintegration 1

Rebased `work/lead-worker-readiness/11-handoffs-into-briefs` onto `effort/lead-worker-readiness` at
`d41faae6cc2b4df4d1939bb87074ba9cafd83c7c`. The only content conflict was in
`tools/agents/test_lead_launch.py`; the resolution keeps ticket 12's runtime-option test and ticket 11's
findings-in-brief test.

Reintegration gate commands run:

- `python3 tools/agents/test_tracker.py && python3 tools/agents/test_lead_launch.py && python3 tools/agents/test_worker.py && python3 tools/agents/test_land.py && python3 tools/agents/test_acceptance.py` -> exit 0. Counts: tracker 17 tests OK; lead-launch 8 tests OK; worker 54 tests OK; land 20 tests OK; acceptance 7 tests OK. The worker suite emitted repeated pre-existing `ResourceWarning: subprocess ... is still running` messages and expected refusal text from negative-path cases; no test failed.
- `python3 -m unittest discover -s bench/harness -p 'test_*.py'` -> exit 0, 61 tests OK. It emitted repeated pre-existing `ResourceWarning: unclosed file` messages from `bench/harness/tasklib.py:90`; no test failed.
- For every directory under `bench/readiness/`, `python3 bench/harness/lead_driven.py <scenario> --out "$(mktemp -d)"` -> exit 0 for `acceptance-failing-check`, `acceptance-missing-record`, `acceptance-other-candidate`, `acceptance-ticket-tamper`, `acceptance-waived-check`, `access-profile`, `external-state-report`, `handoff`, `land-exact-candidate`, `land-stale-candidate`, and `launch-by-edges-and-triage`.
- `python3 tools/agents/tracker.py lint` -> exit 0, including the newly landed ticket 16 and amended ticket 04.
- `python3 tools/agents/tracker.py findings lead-worker-readiness 04` -> exit 0 with this output:

```text
report 05: ticket 05 launches with marker handoff and records pinned launch sources; already-reviewed Candidate reuse refusal remains for review/rework acceptance logic.
report 12: `worker.py resume` now depends on a session id observed in `events.jsonl`; fake-runtime tests cover both adapters, but real runtime resume should still be validated before spending model budget.
report 12: resume runs in the foreground; a rework round needs it started detached and supervised like launch (cap, quiet time, hand-off detection).
lead: after ticket 01 a pinned Candidate never changes: `handoff_state` returns the pinned one, and any new commit is reported as the violation `candidate-moved`. Rework must record the new Candidate as a successor (round N) instead, and only an unrequested move stays a violation.
lead: on the first live rework round the same worker session fixed four review items in 18 minutes but went from 60% to 19% context left. Plan for a second round not fitting in the same session.
```

- `git diff --check` -> exit 0.
