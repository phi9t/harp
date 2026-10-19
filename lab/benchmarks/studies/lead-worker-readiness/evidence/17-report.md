## Built

`worker.py launch` creates a private local clone for each worker instead of a linked git worktree in the shared repository. Normal provider `workspace-write` launches use `git clone --separate-git-dir`, with the checkout's `.git` as a pointer file and the private git directory under the run directory. The launcher passes that private git directory as an additional writable root, so sandboxed Codex and TraeCLI workers can run `git add` and `git commit` without write access to the shared repository's Git metadata.

For `--sandbox insula`, the capsule keeps embedded `.git` metadata inside the private clone because only `/workspace` is mounted inside the capsule. The lead side imports only a ready Candidate commit from the private clone into the shared repository under the worker branch.

Reintegration fixes made the ticket 06 and ticket 10 surfaces private-repository aware: overlap and wait read dirty state, tip and commits-ahead from the worker checkout; accounting reports `worker_repo_bytes`, including a separate private git directory; resume refreshes the private clone's integration branch from the shared repository; and stale-base rework resumes the original session when possible before supervising/importing the successor Candidate.

The practice note and run-worker skill now describe the private clone workspace kind, the separate-git-dir layout, the insula embedded-git layout, and when the shared worker branch is published.

## Acceptance

- "The real probe (create one file with the edit tool, `git add`, `git commit`) run through `worker.py launch` and `supervise` under the default sandbox ends ready for review with a pinned Candidate, for the Codex runtime and for the traecli runtime; both event streams are in `evidence/`."
  Proved by the real probes run after review 1 authorized them. Codex: `python3 tools/agents/worker.py launch --runtime codex --model gpt-5.5 --effort low --allow result.txt ...` then `python3 tools/agents/worker.py supervise ...` ended `verdict=CLEAN`, `ready_for_review=true`, Candidate `91b550768affa76a2ec6f25456461bf59e5b110a`; event stream is `docs/efforts/lead-worker-readiness/evidence/17-probe-codex-events.jsonl`. TraeCLI: the same probe with `--runtime traecli` ended `verdict=CLEAN`, `ready_for_review=true`, Candidate `512da2a649e8e0b81ed0167703503fcbecd527a9`; event stream is `docs/efforts/lead-worker-readiness/evidence/17-probe-traecli-events.jsonl`. I did not spend additional model budget during reintegration.

- "In that run the main repository's refs other than the worker's own branch, its index and its checked-out files are unchanged; a test with a scripted participant that tries to move another branch or write the main repository's git data shows it cannot or that it is reported."
  Proved by `python3 tools/agents/test_worker.py` as part of the full `tools/agents/test_*.py` gate: PASS. The covered tests assert separate private Git metadata, no shared worker branch before publish, unchanged shared `main` and `side` after Candidate publication, foreign branch movement reported as a violation, and peer worker branch publication ignored as expected. The real probes also reported clean shared repository status with `main` and `side` unchanged and only the worker branch at the Candidate.

- "The Candidate handed over can be gated and landed by the existing lander with an acceptance record, from the main checkout, in the layout with a mainline checkout, an integration worktree and several worker workspaces."
  Proved by `python3 bench/harness/lead_driven.py bookkeeping-off-code-branch --out "$(mktemp -d /data02/home/philip.yang/tmp/l17scenario.XXXXXX)"`: PASS (`bookkeeping-while-worker-runs expected=checked observed=checked`). Also proved by the full readiness scenario sweep and `python3 bench/harness/test_lead_driven.py`: PASS.

- "The marker hand-off, Candidate pinning, `candidate-moved`, the rework successor, resume and the overlap report still work with the new workspace kind; every existing scenario matches its expectation."
  Proved by all `tools/agents/test_*.py` suites: PASS; all `bench/harness/test_*.py` suites: PASS; and every `bench/readiness/*` scenario command with `python3 bench/harness/lead_driven.py <scenario> --out "$(mktemp -d /data02/home/philip.yang/tmp/l17scenario.XXXXXX)"`: PASS. The scenario sweep included `acceptance-failing-check`, `acceptance-missing-record`, `acceptance-other-candidate`, `acceptance-ticket-tamper`, `acceptance-waived-check`, `access-profile`, `bookkeeping-off-code-branch`, `external-state-report`, `handoff`, `integration-branch`, `land-exact-candidate`, `land-stale-candidate`, `launch-by-edges-and-triage`, `parallel-workers-and-overlap`, `rework-round`, and `watching-limits-accounting`.

- "A worker launched from an integration branch tip starts from exactly that commit, and a worker asked to re-integrate onto a moved tip can do so from inside its sandbox."
  Proved by `python3 tools/agents/test_lead_launch.py`: PASS, including integration-tip launch, fresh stale-base rework from a moved tip, and stale-base resume without counting against the review cap. Proved in the full scenario sweep by `integration-branch`: PASS, `rework-round`: PASS, and `parallel-workers-and-overlap`: PASS after ticket 02 receives stale-base rework, refreshes the private integration ref, supervises a successor Candidate, and lands it.

- "Disk: the report states the size of a worker workspace before and after this change for this repository."
  Measured in a same-device disposable temp directory under `/data02/home/philip.yang/workspace`, then removed. Linked worktree: 219,864 KiB. Embedded private local clone: 325,376 KiB. Separate-git-dir private clone: checkout 219,864 KiB plus private git directory 105,516 KiB, total 325,380 KiB. A first `/tmp` measurement attempt failed because `git clone --local` could not hardlink across devices; `worker.py` now detects that error and retries with copied objects, recording `clone_object_mode`.

- "The practice note and the run-worker skill describe the workspace kind and why."
  Proved by inspection and by the full gate: `docs/practice/lead-supervisor-worker.md` and `.claude/skills/run-worker/SKILL.md` describe the private clone, separate private git directory, insula embedded git metadata, and publication of the shared worker branch only after a ready Candidate is imported.

## Claims

- [R] normal clone commands use `--local` for same-device object reuse `tools/agents/worker.py:483`
- [R] cross-device clone fallback uses `--no-hardlinks` instead of local hardlinks `tools/agents/worker.py:486`
- [R] normal private clones can use `--separate-git-dir` for Git metadata outside the checkout tree `tools/agents/worker.py:491`
- [R] private clone creation retries with copied objects when the local hardlink clone hits a cross-device error `tools/agents/worker.py:504`
- [R] private clone creation removes the `origin` remote before returning clone metadata `tools/agents/worker.py:514`
- [R] the ready Candidate is fetched from the private clone and published to the shared repository with `git update-ref` under the worker branch `tools/agents/worker.py:542`
- [R] resume refreshes the private clone's integration branch from the shared repository before invoking the runtime `tools/agents/worker.py:552`
- [R] disk accounting reports checkout bytes, private git-dir bytes, and combined worker repository bytes `tools/agents/worker.py:834`
- [R] workspace-write runtime invocations pass the separate private git directory as an additional writable root `tools/agents/worker.py:961`
- [R] insula launch binds the private clone's `.git` as writable at `/workspace/.git` `tools/agents/worker.py:904`
- [R] worker launch records `workspace_kind: private-clone`, the shared repository path, `git_dir`, `git_dir_layout`, and `clone_object_mode` in `run.json` `tools/agents/worker.py:981`
- [R] supervisor boundary checks skip peer worker branches before checking private clone branch movement `tools/agents/worker.py:1182`
- [R] supervisor boundary checks skip peer worker branches before checking shared repository branch movement `tools/agents/worker.py:1198`
- [R] rework successor handoff imports the successor Candidate into the shared repository before saving it as current `tools/agents/worker.py:1575`
- [R] resume receipts include the refreshed integration branch result `tools/agents/worker.py:1821`
- [R] overlap changed-path reporting reads dirty state and commits from the worker checkout `tools/agents/lead_launch.py:273`
- [R] monitor rows compute commits ahead, integration containment, dirty count, last commit age and changed files from the worker checkout `tools/agents/lead_launch.py:970`
- [R] fresh rework imports the previous Candidate into the shared repository before launching the next worker `tools/agents/lead_launch.py:762`
- [R] stale-base fresh rework replaces the rework base with the current integration branch tip `tools/agents/lead_launch.py:766`
- [R] stale-base rework can resume an available original session instead of forcing a fresh worker `tools/agents/lead_launch.py:830`
- [R] accounting rows call worker disk accounting for each run and the text output reports `worker_repo` bytes `tools/agents/lead_launch.py:1005`
- [R] batch launch records peer worker branches so overlapping worker publication is not treated as foreign branch movement `tools/agents/lead_launch.py:575`
- [R] the private-metadata test asserts the shared worker branch is absent before publish and shared `main` and `side` are unchanged after publish `tools/agents/test_worker.py:446`
- [R] the resume-refresh test advances the shared integration branch and asserts the worker clone refreshes it before resume `tools/agents/test_worker.py:645`
- [R] the peer-branch test asserts a peer worker branch imported in the shared repository is not a `shared-branch-moved` violation `tools/agents/test_worker.py:1241`
- [R] the disk test asserts `git_dir_bytes` and combined `worker_repo_bytes` are reported `tools/agents/test_worker.py:1516`
- [R] the integration-tip launch test asserts a worker starts exactly at the integration tip `tools/agents/test_lead_launch.py:204`
- [R] the stale-base fresh-worker test asserts a fresh rework worker starts at the moved integration tip without counting toward the review cap `tools/agents/test_lead_launch.py:705`
- [R] the stale-base resume test asserts stale-base rework resumes an available session, does not count toward the review cap, and refreshes integration baselines `tools/agents/test_lead_launch.py:751`
- [R] the parallel-overlap scenario sends stale-base rework through `lead_launch rework`, supervises the resumed worker, and lands the successor Candidate `bench/harness/lead_driven.py:1496`
- [R] the watching-limits-accounting harness checks `worker_repo_bytes` as well as worktree bytes `bench/harness/lead_driven.py:1306`
- [R] the watching-limits-accounting unit test asserts `git_dir_bytes` and combined `worker_repo_bytes` are present `bench/harness/test_lead_driven.py:465`
- [R] the practice note says the worker workspace is a private local clone and explains the separate-git-dir and insula embedded-git layouts `docs/practice/lead-supervisor-worker.md:53`
- [R] the run-worker skill says launches print a private local clone path with writable private Git metadata `.claude/skills/run-worker/SKILL.md:19`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/17-report.md`: required hand-off report for this ticket.
- `docs/efforts/lead-worker-readiness/evidence/17-probe-codex-events.jsonl`: required real Codex probe event stream.
- `docs/efforts/lead-worker-readiness/evidence/17-probe-traecli-events.jsonl`: required real TraeCLI probe event stream.

## For later tickets

- to 18: a worker's workspace is a private clone, not a worktree of the shared repository; its branch exists in the shared repository only after a ready Candidate is imported. Monitors must read the worker's own repository for commits ahead, dirty state and the tip.

## Not checked

- `tools/agents/check.sh` was not run because the direct worker brief forbids it.
- Cargo was not run because the direct worker brief forbids it.
- Real Codex and TraeCLI model probes were not rerun during reintegration; the retained probe event streams from review 1 are the evidence for that acceptance item, and the lead's reintegration note says those real probes were independently re-run against the accepted content before this rebase.
