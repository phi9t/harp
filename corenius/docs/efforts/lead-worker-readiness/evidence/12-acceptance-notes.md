# Ticket 12: notes beside the tool-written acceptance record

The acceptance record for ticket 12 is `acceptance-12.jsonl`, written with `tools/agents/acceptance.py`, and the
Candidate 536a2b1 was landed with `tools/agents/land.py --effort lead-worker-readiness --ticket 12` run from the
main checkout. This is the first ticket of the effort accepted and landed with the effort's own tools.

Review: one round (`12-review-1.md`) and one re-integration (`12-reintegrate-1.md`).
Required checks named in the record: `gate` (lander, acceptance, worker, tracker and lead-launch suites; every
bench harness suite; every scenario; tracker lint) and `real-codex-stream` (the lead ran one real
`codex exec --json`, kept as `12-codex-exec-probe.jsonl`, and fed it to the adapter: one command found once,
session id found, usage keys present).
Not checked by anyone: a real Codex or traecli resume.
