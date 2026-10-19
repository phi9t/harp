# Lead review 1 of ticket 21: the landing gate fails on your exact commit

Reviewed Candidate: b9cfd0b. I recorded the acceptance and ran the landing; the gate refused it:

    FAIL bench/harness/test_lead_driven.py
    gate failed for b9cfd0b8e6670ff416ddf5f24d2319eb9bb56695

I then ran that suite on a fresh detached checkout of b9cfd0b: 35 tests, one error, and the same error three
times out of three when run alone, with and without `TMPDIR` set:

    ERROR: test_unexpected_worker_stderr_fails_participant (__main__.LeadDrivenTest...)
      File ".../bench/harness/lead_driven.py", line 1058, in launch_and_supervise
        details.extend(worker_execution_failures(Path(launched["run_dir"])))
      File ".../bench/harness/lead_driven.py", line 583, in worker_execution_failures
        events = json_lines(run_dir / "events.jsonl")
      File ".../bench/harness/lead_driven.py", line 2500, in json_lines
        return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)

    Ran 1 test in 0.200s
    FAILED (errors=1)

So in a clean checkout the worker's event file in that test contains a line that is not JSON, and the
harness crashes on it instead of reporting the participant as failed, which is exactly what the test is
about. Your report says the suite passed in your worktree. Something differs between your worktree and
your commit: an untracked or ignored file, a file mode, an environment variable, or leftover state.

Do:
1. Reproduce it the way the gate sees it: `git worktree add --detach <dir under your TMPDIR> HEAD`, run
   `python3 bench/harness/test_lead_driven.py LeadDrivenTest.test_unexpected_worker_stderr_fails_participant`
   there, and find what your worktree has that the commit lacks (`git status --ignored`, file modes with
   `git ls-files -s`, your shell environment). Remove that detached worktree afterwards with
   `git worktree remove --force`.
2. Fix both sides: the harness must never crash on a non-JSON line in a worker's event file (a real runtime
   prints such lines; record the line as an unexpected-output failure of that participant), and the test
   must pass in a clean checkout.
3. From now on run your gate in that clean detached checkout of your final commit before the marker commit,
   and say in the report that you did. Add one line under `## For later tickets`:
   `- to 14: run each ticket's gate on a clean checkout of the exact commit before handing over; ticket 21's first hand-off passed in its worktree and failed in a clean checkout.`

New commits on top of b9cfd0b; no rewriting. Update the report. Finish with a new final commit
`ticket 21: ready for review` on a clean tree. Nothing has landed since your base; no re-integration is needed.
