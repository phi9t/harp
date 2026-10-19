# Ticket 20 report

## Built

The three rework entrypoints now share one effort-local ticket counter in
`docs/efforts/<effort>/evidence/rework-<ticket>.jsonl`: `acceptance.py
request-rework`, `worker.py deliver-findings`, and `lead_launch.py rework`.
The fourth review-driven round is refused and triages the ticket to
`ready-for-human`; stale-base redos do not advance the cap.

Delivery receipts now distinguish "sent" from "acknowledged". Terminal delivery
is acknowledged only after a later worker event mentions the findings path or
sha256, or after a later commit/marker handoff. Detached resume starts as
`acknowledged: false` and is acknowledged later by `status`/`supervise` only
after a worker-authored observation.

Resume now records process group/session identity, keeps the launch boundary
when building resume commands, keeps the worker private git dir writable, and
fresh-worker rework carries the launch sandbox, handoff mode, capsule, protected
paths, external paths, and access profile.

## Acceptance

- "Finding 2: there is one review-round counter per ticket, kept with the
  effort's records, and every path that starts a review-driven round consults
  and advances it: `acceptance.py request-rework`, `worker.py deliver-findings`,
  `lead_launch.py rework` (same session and fresh worker). A fourth
  review-driven round is refused on each of the three commands and sets the
  ticket's triage to `ready-for-human`. A stale-base redo does not advance it.
  A test makes four consecutive `lead_launch.py rework` calls and four mixed
  calls."
  Proven by `python3 tools/agents/test_acceptance.py` (15 tests OK),
  `python3 tools/agents/test_lead_launch.py` (31 tests OK),
  `python3 tools/agents/test_worker.py` (73 tests OK), and
  `python3 bench/harness/lead_driven.py rework-round --out "$(mktemp -d)"`
  (all three rework cases passed).

- "Finding 2 and 11: `acknowledged` is true only on an observation that could
  not have been produced by the sender: a worker event or commit that follows
  the delivery and references the findings file's digest or path in a
  worker-authored event, or a new marker commit. The lead's own typed message
  appearing in a pane, an unrelated event from a busy worker, a resume process
  that merely started, and a runtime that exits non-zero are each
  `acknowledged: false` with the reason. Tests for each of those four."
  Proven by `python3 tools/agents/test_worker.py` (73 tests OK), including the
  explicit nonzero-resume, detached-resume, pane-echo, unrelated-event, and
  concrete-path/digest acknowledgement tests.

- "Finding 4: a resumed turn runs under the same boundary as the launch: the
  same sandbox mode, the same working directory, the worker's private git
  directory writable, and inside the capsule when the launch was in one. Where
  a runtime's `resume` command has no flag for this, it is passed through
  configuration overrides; where it cannot be done at all, resume for that
  runtime and mode is refused with a message, not run unbounded. A fresh-worker
  rework carries the whole launch profile (sandbox, hand-off mode, protected
  paths, external protected paths, capsule, access profile)."
  Proven by `python3 tools/agents/test_worker.py` (73 tests OK) and
  `python3 tools/agents/test_lead_launch.py` (31 tests OK). The tests assert
  resume config carries sandbox/cwd/writable git dir, detached resume is
  supervised, and fresh-worker rework preserves the original launch profile.

- "Finding 4, real proof: for each runtime, the real launch-then-resume above
  ends with both commits on the worker's branch, verdict ready, and the
  Candidate imported; the two event streams per runtime are in `evidence/` and
  the verdict lines are in the report. If a runtime cannot do it under
  `workspace-write`, stop with `needs-info` and the exact error."
  TraeCLI passed the real launch-then-resume proof. Its event streams are
  `20-real-traecli-launch-events.jsonl` and
  `20-real-traecli-resume-events.jsonl`; the verdict line is
  `20-real-traecli-verdict.json` and shows `verdict: CLEAN`,
  `ready_for_review: true`, and a Candidate.

  Codex is not proven: blocked by the host's full `/tmp`, exact error above.
  Its event streams are `20-real-codex-launch-events.jsonl` and
  `20-real-codex-resume-events.jsonl`; the verdict line is
  `20-real-codex-verdict.json` and shows `verdict: DEGRADED`,
  `missing_allowed_outputs: ["b.txt"]`, `ready_for_review: false`,
  `not_ready: ["dirty-tree", "no-commit-after-base"]`. The exact blocker from
  Codex's workspace-write shell is
  `bwrap: Can't mkdir /tmp/.aws: No space left on device`.

- "Finding 10: a worker is reported stopped only when no process of its session
  is alive: the run records the session or process group, the wall-clock wrapper
  does not detach from it, and `account` after a scenario shows no surviving
  process (the test checks the process table, not the run record)."
  Proven by `python3 tools/agents/test_lead_launch.py` (31 tests OK), including
  the `/proc`-based live session child check, and by
  `python3 bench/harness/lead_driven.py watching-limits-accounting --out
  "$(mktemp -d)"` (both cases passed).

- "Every existing scenario still matches its expectation."
  Proven by the full scenario loop:
  `for f in bench/readiness/*/scenario.toml; do scenario=$(basename "$(dirname
  "$f")"); out=$(mktemp -d); python3 bench/harness/lead_driven.py "$scenario"
  --out "$out"; rm -rf "$out"; done`. All 16 readiness scenarios passed.

Additional gate results:

- `python3 -m py_compile tools/agents/worker.py tools/agents/lead_launch.py
  tools/agents/acceptance.py tools/agents/test_worker.py
  tools/agents/test_lead_launch.py tools/agents/test_acceptance.py
  tools/agents/test_land.py bench/harness/test_lead_driven.py
  bench/readiness/rework-round/scripted_participant.py
  bench/readiness/parallel-workers-and-overlap/scripted_participant.py`: passed.
- `python3 bench/harness/test_lead_driven.py && python3
  bench/harness/test_validate.py && python3 bench/harness/test_run.py &&
  python3 bench/harness/test_tasks_tracked.py && python3
  bench/harness/test_fleet.py`: passed.
- `python3 tools/agents/tracker.py lint`: passed with no output.

## Claims

- [R] the shared effort rework counter counts only records whose
  `counts_toward_cap` field is true `tools/agents/acceptance.py:426`
- [R] the shared counter refuses the fourth review-driven round and writes
  `ready-for-human` triage before raising `review-driven rework cap exhausted`
  `tools/agents/acceptance.py:441`
- [R] `acceptance.py request-rework` records through the shared rework helper
  `tools/agents/acceptance.py:517`
- [R] `worker.py deliver-findings` writes its delivery into the shared effort
  rework ledger before appending the local delivery record
  `tools/agents/worker.py:2107`
- [R] `lead_launch.py rework` checks the shared cap before starting either
  same-session resume or fresh-worker rework `tools/agents/lead_launch.py:902`
- [R] fresh-worker rework passes the original sandbox, network, command-timeout,
  handoff, allowed paths, external paths, and protected paths to the new launch
  `tools/agents/lead_launch.py:841`
- [R] worker event acknowledgement requires the worker event text to contain the
  findings file path or findings sha256 `tools/agents/worker.py:1527`
- [R] detached resume receipts start as `acknowledged: false` with the reason
  "resume process started; no worker-side acknowledgement observed yet"
  `tools/agents/worker.py:2017`
- [R] a successor Candidate is recorded from an acknowledged pending delivery
  and linked with `source_delivery_id` `tools/agents/worker.py:1765`
- [R] process state checks live process-session members before trusting
  `exit_code` `tools/agents/worker.py:1823`
- [R] the rework readiness scripted worker now emits a findings-bearing
  acknowledgement event `bench/readiness/rework-round/scripted_participant.py:53`
- [R] the parallel-overlap scripted worker now emits a findings-bearing
  acknowledgement event before rebasing
  `bench/readiness/parallel-workers-and-overlap/scripted_participant.py:51`
- [R] the TraeCLI real proof verdict is clean, ready for review, and has a
  Candidate `docs/efforts/lead-worker-readiness/evidence/20-real-traecli-verdict.json:1`
- [R] the Codex real proof hit `bwrap: Can't mkdir /tmp/.aws: No space left on
  device` during the launch stream
  `docs/efforts/lead-worker-readiness/evidence/20-real-codex-launch-events.jsonl:20`
- [R] the Codex real proof verdict is degraded, not ready, and missing `b.txt`
  `docs/efforts/lead-worker-readiness/evidence/20-real-codex-verdict.json:1`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/20-report.md`: required worker
  handoff report.
- `docs/efforts/lead-worker-readiness/evidence/20-real-codex-launch-events.jsonl`:
  required real-proof event stream and blocker evidence.
- `docs/efforts/lead-worker-readiness/evidence/20-real-codex-resume-events.jsonl`:
  required real-proof event stream and blocker evidence.
- `docs/efforts/lead-worker-readiness/evidence/20-real-codex-verdict.json`:
  required real-proof verdict line and blocker evidence.
- `docs/efforts/lead-worker-readiness/evidence/20-real-traecli-launch-events.jsonl`:
  required real-proof event stream.
- `docs/efforts/lead-worker-readiness/evidence/20-real-traecli-resume-events.jsonl`:
  required real-proof event stream.
- `docs/efforts/lead-worker-readiness/evidence/20-real-traecli-verdict.json`:
  required real-proof verdict line.

## For later tickets

- to 14: the real Codex launch-then-resume proof of ticket 20 is owed; it could not run while /tmp had no free inodes (bwrap: Can't mkdir /tmp/.aws). Run it before the live trial: launch (create a.txt containing one, commit), then resume (create b.txt containing two, commit), expect both commits, verdict ready, Candidate imported.

## Not checked

- I did not produce a passing Codex real proof under `workspace-write`; the
  exact blocker is recorded above and the lead directed that this proof is owed
  later rather than retried now.
- I did not run `tools/agents/test_sqlite_baseline.py`; the worker brief says it
  hardcodes `/tmp` and must be left out of this gate.
- I did not run `tools/agents/check.sh` or cargo, per the worker brief.
- I did not run unrelated `tools/agents/test_*.py` suites that neither import
  nor exercise the touched modules.
- One diagnostic command accidentally redirected to `/tmp/nonexistent` and
  failed immediately with `No space left on device`; it produced no gate result.
