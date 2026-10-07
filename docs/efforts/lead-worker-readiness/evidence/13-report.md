## Built

Added readiness result rows beside the lead-driven scenario runner. Each scenario command now appends a
`readiness-results.jsonl` row with instantiation, runtime, proof kind, outcome, interventions, elapsed
time and model usage.

Added a readiness report command that prints one table per scenario/proof across the three instantiations,
plus a core-set summary. Added a notes command that appends timestamped `obstacle`, `intervention` or
`rule` entries in the existing running-notes format.

Red evidence: before implementation, `python3 bench/harness/test_lead_driven.py` failed four new tests
because `--intervention`, `--report`, `--note` and `--notes-file` were not recognized.

## Acceptance

- "Each scenario run appends one row with the fields above; a run recorded with an intervention has outcome `passes with intervention` and lists it." Proven by `python3 bench/harness/test_lead_driven.py`. Result: 14 tests OK. Specific test: `test_scenario_appends_one_readiness_row_with_intervention_details`.
- "The report prints every scenario defined under the readiness directory against the three instantiations, with `not yet run` for missing rows and `not applicable by design` rows showing their reason." Proven by `python3 bench/harness/test_lead_driven.py`. Result: 14 tests OK. Specific test: `test_report_prints_all_instantiations_and_core_summary`.
- "The report states, for the core set, how many scenarios pass, pass with intervention, fail and are not yet run." Proven by `python3 bench/harness/test_lead_driven.py`. Result: 14 tests OK. Specific test: `test_report_prints_all_instantiations_and_core_summary`.
- "Deterministic and live rows are never merged into one figure." Proven by `python3 bench/harness/test_lead_driven.py`. Result: 14 tests OK. Specific test: `test_report_keeps_deterministic_and_live_rows_separate`.
- "The notes command appends an entry with time, kind and text, and refuses an unknown kind." Proven by `python3 bench/harness/test_lead_driven.py` and by `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py --note obstacle 'disk nearly full' --notes-file "$tmp/notes.md"; rc1=$?; python3 bench/harness/lead_driven.py --note surprise 'bad kind' --notes-file "$tmp/notes.md" >/tmp/corenius-ticket13-note.out 2>/tmp/corenius-ticket13-note.err; rc2=$?; rm -rf "$tmp" /tmp/corenius-ticket13-note.out /tmp/corenius-ticket13-note.err; test $rc1 -eq 0 && test $rc2 -eq 2`. Result: both commands behaved as expected and the shell check exited 0.

Additional gate evidence:

- `python3 bench/harness/test_run.py`. Result: 17 tests OK. It emitted pre-existing `ResourceWarning: unclosed file` warnings from `bench/harness/tasklib.py:90`; the suite exited 0.
- `python3 -m unittest discover bench/harness -p 'test_*.py'`. Result: 51 tests OK. It emitted the same `ResourceWarning` warnings from `bench/harness/tasklib.py:90`; the suite exited 0.
- `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py handoff --out "$tmp"; rc=$?; rm -rf "$tmp"; exit $rc`. Result: exit 0; both default handoff participants printed `passes`.
- `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py handoff --participant commit-ready --intervention 'lead:waited for idle' --out "$tmp" >/tmp/corenius-ticket13-run.out; rc=$?; if [ $rc -eq 0 ]; then python3 bench/harness/lead_driven.py --report "$tmp"; rc=$?; fi; rm -rf "$tmp" /tmp/corenius-ticket13-run.out; exit $rc`. Result: exit 0; the report showed `passes with intervention`, `unknown` model usage and `not yet run` for the other two instantiations.
- `python3 tools/agents/tracker.py lint`. Result: exit 0.

## Claims

- [R] readiness rows are stored in `readiness-results.jsonl` `bench/harness/results.py:18`
- [R] the readiness table compares `lead-driven`, `local-controller` and `workflow-controller` `bench/harness/results.py:19`
- [R] readiness outcomes include `passes`, `passes with intervention`, `not applicable by design` and `fails` `bench/harness/results.py:20`
- [R] the core-set story list is encoded as stories 1, 2, 5, 6, 7, 9, 12, 14, 15, 16, 17, 18, 20, 27, 34 and 35 `bench/harness/results.py:22`
- [R] a `not applicable by design` row must carry a reason `bench/harness/results.py:125`
- [R] readiness rows validate intervention entries as `who` and `what` strings `bench/harness/results.py:133`
- [R] appending a readiness row writes one JSON line to the result file `bench/harness/results.py:174`
- [R] readiness scenarios are discovered from every `scenario.toml` under the readiness directory `bench/harness/results.py:196`
- [R] readiness scenario story numbers are read from each scenario file's `stories` key `bench/harness/results.py:199`
- [R] the report prints `Stories: not declared` when a scenario has no `stories` key `bench/harness/results.py:252`
- [R] missing scenario/instantiation/proof cells print as `not yet run` `bench/harness/results.py:260`
- [R] core-set counts are computed for one instantiation at a time `bench/harness/results.py:228`
- [R] readiness summaries are emitted separately for each proof kind and instantiation `bench/harness/results.py:270`
- [R] `--intervention` values are parsed as `who:what` pairs `bench/harness/lead_driven.py:317`
- [R] a passing scenario with interventions becomes `passes with intervention` `bench/harness/lead_driven.py:327`
- [R] each lead-driven scenario command appends a readiness row after participant records are written `bench/harness/lead_driven.py:359`
- [R] lead-driven scenario rows record `model_usage` as `unknown` when no runtime usage is exposed `bench/harness/lead_driven.py:369`
- [R] the notes command refuses a kind outside `obstacle`, `intervention` and `rule` `bench/harness/lead_driven.py:376`
- [R] the notes command appends timestamped `kind: text` lines `bench/harness/lead_driven.py:389`
- [R] the lead-driven CLI exposes a report command through `--report` `bench/harness/lead_driven.py:400`
- [R] the lead-driven CLI exposes a notes command through `--note` `bench/harness/lead_driven.py:401`
- [R] the handoff scenario declares its story numbers in `scenario.toml` `bench/readiness/handoff/scenario.toml:2`
- [R] the lead-driven tests assert per-instantiation core-set summary lines `bench/harness/test_lead_driven.py:217`
- [R] the lead-driven tests assert missing scenario story metadata is reported `bench/harness/test_lead_driven.py:260`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/13-report.md`: required worker hand-off report.
- `bench/readiness/handoff/scenario.toml`: review 1 required moving the handoff story numbers into the scenario definition.

## For later tickets

none

## Not checked

- Did not run `tools/agents/check.sh` or cargo, by ticket instruction.
- Did not launch real Codex or traecli model workers, by ticket instruction.
- Did not run any `tools/agents/test_*.py` suite. I touched no `tools/agents` files, and `rg -n "lead_driven|bench\\.harness\\.results|import results" tools/agents tools` found no tools-agent suite importing the touched modules.

## Review round 1

Implemented both lead findings from `review-1.md`:

- Core-set summaries now print one line per proof and instantiation, so `lead-driven`, `local-controller`
  and `workflow-controller` results are not merged.
- Scenario story numbers now live in `bench/readiness/handoff/scenario.toml`; the report prints
  `Stories: not declared` for any scenario definition without a `stories` key.

Red evidence: after updating tests but before implementation, `python3 bench/harness/test_lead_driven.py`
failed three tests: the two summary tests did not find per-instantiation lines, and the missing-stories
test did not find `Stories: not declared`.

Review-round verification:

- `python3 bench/harness/test_lead_driven.py`. Result: 14 tests OK.
- `python3 bench/harness/test_run.py`. Result: 17 tests OK. It emitted pre-existing `ResourceWarning:
  unclosed file` warnings from `bench/harness/tasklib.py:90`; the suite exited 0.
- `python3 -m unittest discover bench/harness -p 'test_*.py'`. Result: 51 tests OK. It emitted the same
  `ResourceWarning` warnings from `bench/harness/tasklib.py:90`; the suite exited 0.
- `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py handoff --out "$tmp"; rc=$?; if [ $rc -eq 0 ]; then python3 bench/harness/lead_driven.py --report "$tmp"; rc=$?; fi; rm -rf "$tmp"; exit $rc`. Result: exit 0; the report showed `Stories: 1, 2, 4, 44` and one core-set summary line per instantiation.
- `tmp=$(mktemp -d); python3 bench/harness/lead_driven.py --note rule 'per-instantiation summaries' --notes-file "$tmp/notes.md" >/tmp/corenius-ticket13-note.out; rc1=$?; python3 bench/harness/lead_driven.py --note unknown 'bad kind' --notes-file "$tmp/notes.md" >/tmp/corenius-ticket13-note2.out 2>/tmp/corenius-ticket13-note2.err; rc2=$?; rm -rf "$tmp" /tmp/corenius-ticket13-note.out /tmp/corenius-ticket13-note2.out /tmp/corenius-ticket13-note2.err; test $rc1 -eq 0 && test $rc2 -eq 2`. Result: exit 0.
- `python3 tools/agents/tracker.py lint`. Result: exit 0.
- `git diff --check`. Result: exit 0.
