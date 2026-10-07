# 12 codex-worker-runtime: A Codex worker can be launched, supervised and resumed

Type: feature
Status: resolved
Owner: worker-12
Blocked by: 01
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

The owner's runtime order is Codex first, then traecli. The launcher and the lead-driven mode gain a runtime choice. A Codex worker is started non-interactively with its event stream captured, is judged by the same hand-off rule and findings as a traecli worker, and can be resumed with a follow-up message (the capability ticket 04 needs). Differences between the two runtimes' event streams are handled in one adapter, and anything the supervisor cannot observe for a runtime is stated, not guessed. No model call is made in tests: a fake binary stands in.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/test_worker.py`
- `bench/harness/` lead-driven module (a runtime option)

## Acceptance
- [x] A launch option selects the runtime; the run record names it; the default stays as today.
- [x] With a fake Codex binary emitting a recorded sample of the real event format, command events are extracted and the forbidden-command and quiet-time findings work as for traecli.
- [x] The hand-off rule (exit 0, clean tree, at least one commit) is applied identically for both runtimes; one test runs the same scripted behavior under both and compares verdicts.
- [x] Resume sends a follow-up message to the same session for both runtimes with the fake binaries; where a runtime cannot resume, the tool reports that.
- [x] The help text lists, per runtime, what is and is not observed.
- [x] The sample of the Codex event format is taken from a real local Codex session record with secrets and private reasoning removed, and its origin is stated in the test.

## Comments
- 2026-10-05T10:25:30Z worker-12: claimed until 2026-10-07T10:25:30Z
- 2026-10-05T11:35:46Z worker-12: resolved
