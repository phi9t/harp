# Independent review 1 of `main..effort/lead-worker-readiness` at 0a28a3d (2026-10-05)

Reviewer: a clean-context agent of the lead's model family, read-only, told to form its findings before reading the lead's records. Text below is its report, unedited.

Review of `main..effort/lead-worker-readiness` at `0a28a3d` (base `d7f6748`). The branch is not safe to fast-forward the mainline to yet: 16 of the 23 findings below were reproduced in throwaway repositories, and four are blocking.

Suites run in my detached worktree all pass: `test_land` 24, `test_acceptance` 15, `test_tracker` 22, `test_effort` 15, `test_wt` 12, `test_lead_launch` 25, `test_worker` 72, `test_sync_agentic_workflow` 24, `bench/harness/test_lead_driven` 29, and all 16 readiness scenarios. So every defect below is one the tests do not see.

"Known" means the records under `docs/efforts/lead-worker-readiness/evidence/` already had it. I searched them by keyword after forming the findings and read `lead/notes.md` and `17-review-1.md` in full; the bench-harness findings (15–20) were not searched individually, so their "not known" is weaker.

## Findings

**1. Blocking, A. Any other agent's commit makes a running worker VIOLATED and not ready.** `tools/agents/worker.py:884` and `:1195-1209`.
- Launch snapshots every local branch of the shared repository; each supervise cycle reports any moved or new branch as `shared-branch-moved` VIOLATION, which adds `boundary-violation` to `not_ready`.
- Repro: launched a ticket with a fake runtime, committed on an unrelated branch from another worktree and created one new branch. Result: `verdict=VIOLATED ready=False`, findings `shared-branch-moved agent/unrelated …` and `agent/new-branch <new>`.
- The same code path fires when the lead lands another ticket on the integration branch (not run separately).
- In this repository, with over 100 worktrees and several agents, almost every run would be judged this way.
- Known: no. Ticket 17's probes used a repository with only `main` and `side`; only peer worker branches are exempt (`17-report.md`).
- Fix: check only what the worker could have moved, or report foreign movement as a NOTE without attributing it or blocking readiness.

**2. Blocking, B. `lead_launch.py rework` has no three-round cap and records receipts with no observation.** `tools/agents/lead_launch.py:708-744` and `:822-871`; `tools/agents/worker.py:1822`.
- Repro: five consecutive `rework` calls returned rc 0 with `round=1..5`, `counts_toward_cap=True`.
- The fifth used a runtime whose `exec resume` exits 7 at once; the delivery was still `acknowledged=True` (`exit_code` file held 7).
- The fresh-worker path also hardcodes `acknowledged: True` and starts a new `run.json`, so its counter restarts at zero (code path).
- The cap exists only in `acceptance.py request-rework` and `worker.py deliver-findings`, each with its own counter; none consults the others. Stories 12 and 16 are core.
- Known: no. The records cite the cap only for the acceptance path; the lead did flag ticket 04 for recording `acknowledged` at send time.
- Fix: one per-ticket counter in the effort record enforced by every path; set `acknowledged` only from an observed worker-side event.

**3. Blocking, A/B. The acceptance record that authorizes a landing is whatever sits in the current checkout, tracked or not.** `tools/agents/land.py:182-184` (the local copy wins before `Record branch` or `Integration branch` is consulted) and `:445`; `tools/agents/acceptance.py:31-37`.
- Repro (a): effort directory on `main` and on the integration branch, acceptance recorded from the integration worktree. `land.py` from the main checkout: "acceptance record missing". `acceptance.py record` from the main checkout wrote an untracked `docs/efforts/demo/evidence/` into it. `tracker.py list` printed different first rows from the two checkouts (not checked whether content or ordering differs).
- Repro (b): a third worktree held a hand-written untracked `acceptance-01.jsonl` (reviewer "worker") and no other checkout held a record. `land.py … --effort demo --ticket 01` from there returned 0 and moved `effort/demo`.
- Once this branch is on `main`, every checkout cut from it carries a copy of the effort, so the "local" copy will usually be a stale one.
- `docs/agents/issue-tracker.md` now says tracker and acceptance commands "resolve the record branch"; that holds only where no local copy exists.
- Known: partly. The lead met the multi-worktree ambiguity (notes 11:48 and 12:15); the local-first rule was the fix. The forged or stale local record is not in the records.
- Fix: resolve by `Record branch`, then `Integration branch`, and use a local copy only if it is that checkout; for landing, read the record from the committed tip of the record or integration branch.

**4. Blocking, B. A resumed or reworked turn does not run under the launch's boundary.** `tools/agents/worker.py:637-644`; `tools/agents/lead_launch.py:791-792`.
- `resume_argv` passes no `-s`, `-C` or `--add-dir`. `codex exec resume --help` (codex-cli 0.160.0) lists none of those flags; `traecli exec resume --help` shows no sandbox flag either.
- So the resumed turn runs under the runtime's default sandbox, and the private git directory under the run directory is not passed as a writable root. Ticket 17's item "re-integrate … from inside its sandbox" is proven only with fakes.
- A run launched with `--capsule` is resumed on the host, outside the capsule.
- `launch_fresh_rework` hardcodes `--sandbox workspace-write`, exit mode, and drops `--external-protect-path`, `--protect`, the capsule and the tmux window.
- Evidence: code path and CLI help; no real run, so the runtime's actual behaviour is inference.
- Known: no. `12-review-1.md` checked resume arguments against the help text but did not raise this.
- Fix: pass the sandbox and writable root through `-c` overrides, wrap resume in the capsule when the launch was, carry the full access profile into a fresh rework, and prove it with one real resume per runtime.

**5. Should-fix, A. Landing with a record branch commits other people's uncommitted work.** `tools/agents/land.py:262-265`.
- `git add docs/efforts/<effort>` then `git commit -- <dir>` in the record worktree.
- Repro: an untracked `draft-notes-in-progress.md` and an uncommitted edit to ticket 02 were both in the "Record landing for ticket 01" commit.
- Known: no. Fix: add and commit only `evidence/landing-NN.jsonl`.

**6. Should-fix, A. `land.py` pushes first and then cannot fast-forward locally.** `tools/agents/land.py:452-461`.
- The overlap check ignores untracked files (`:300`).
- Repro: an untracked `new.txt` in the integration checkout that the Candidate also adds. Exit 6 with "stale … a new Candidate on the new base is required"; the remote branch equals the Candidate, the local branch does not.
- Known: no. Fix: run a dry fast-forward check against the target checkout before the push, and report a diverged local state as its own code.

**7. Should-fix, B. The mainline guard is opt-in.** `tools/agents/land.py:218-219` and `:227-228`.
- Repro: `land.py <sha> --no-push` with no ticket and no gate moved `main` (default target `main`, zero gates pass).
- Repro: with `--effort/--ticket --mainline main` and an `EFFORT.md` without a `Base:` line, it landed on `main` with no owner override.
- Three efforts in the repository have no `Base:` line: `mac-operator-center`, `sqlite-controller`, `worker-monitoring`.
- Known: no. Fix: require a ticket or an explicit owner flag for any landing on `main`; treat a missing `Base:` as `main`; require at least one gate.

**8. Should-fix, A. A failed batch launch leaves tickets claimed with no worker.** `tools/agents/lead_launch.py:625-640`.
- Claims are committed before any workspace is cut.
- Repro: `launch demo 01 02` with the second workspace name taken. rc 64, commit "Claim tickets 01, 02 for workers" stays, worker 01 is running, and `launch demo 02` is refused as "status is claimed".
- Any failure after the claim does the same; the brief's `tempfile.TemporaryDirectory` (`:498`) is one such step, with `/tmp` full as it is now.
- Known: no. Fix: validate and create workspaces before claiming, or release the claims of tickets that did not launch.

**9. Should-fix, B. `monitor-tickets` reports the wrong numbers for the private-clone workspace.** `tools/agents/lead_launch.py:973-979`.
- It resolves the integration branch name inside the worker's clone, where it does not exist.
- Repro: a worker with a marker commit (one commit on its base by construction) showed `ahead: 0, branch_contains_integration: False`; the clone had only `main` and its worker branch.
- The same row said `worker_state: ready` while `supervise` had said VIOLATED and not ready.
- Known: partly. `17-review-1.md` left a finding addressed to ticket 18, which is open; the landed command is wrong meanwhile.
- Fix: compare against the integration commit recorded in `run.json`.

**10. Should-fix, B. A worker can be reported stopped while its runtime is alive.** `tools/agents/worker.py:963-976` and `:1633-1637`.
- The run records the pid of `sh`; `timeout` moves itself to its own process group.
- Reproduced through the bench harness: after two scenarios exited 0, six `timeout 30 …scripted_participant.py` trees were still alive with PPID 1, and `account` reported `process_state: stopped`.
- Story 40's proof therefore passes while the process runs.
- Known: no. Fix: use `timeout --foreground` or record and check the whole session.

**11. Should-fix, B. The terminal delivery acknowledgement is satisfied by the lead's own keystrokes or by unrelated activity.** `tools/agents/worker.py:1381-1389` and `:1342-1364`.
- `pane_acknowledgement` accepts a pane that changed and contains the findings path; the message just typed contains that path.
- `event_acknowledgement` accepts any event after the send offset, so a busy worker acknowledges instantly.
- The test (`test_worker.py:796-841`) writes the path into a fake pane itself.
- Evidence: code path only; I did not send keys to any pane.
- Known: partly. The lead flagged the send-time receipt in ticket 04; the replacement was accepted.
- Fix: require an event that references the findings digest, or a new commit.

**12. Should-fix, B. The access profile is declared, not enforced, outside a capsule.** `tools/agents/worker.py:618-634` and `:1096`.
- `--network on|off` adds no runtime flag; "off" is a regex for `curl|wget` in command text, and "on" is recorded without checking that the sandbox allows it.
- `--command-timeout` reaches traecli only; for Codex it is prompt text.
- Stories 31, 32, 33 and 36 are proven by a participant that emits the strings and never touches the network, the add path or a long command.
- Evidence: code path; the claim that the runtimes' `workspace-write` sandbox blocks the network by default is inference.
- Known: partly. The runtime help says Codex per-shell timeout is not observed; the network gap is not recorded.

**13. Should-fix, B. The wall-clock notice is proven only by editing the tool's own record.** `tools/agents/worker.py:1640-1649`; `tools/agents/test_worker.py:1285-1303`.
- The notice needs `alive` and `elapsed > cap`, but the process runs under `timeout <cap>` and is killed at that instant.
- The test and the scenario rewrite `cap_seconds` and `started_epoch` in `run.json` after launch.
- Known: no.

**14. Should-fix, A. `worker.py status` writes to the shared repository and its help does not say so.** `tools/agents/worker.py:1749`, `:534-546`, help at `:1960-1963`.
- With `persist=True` it fetches objects and creates or moves `refs/heads/worker/<name>` there. Ticket 10 requires the help to state what `status` writes.
- `status`, `supervise` and `deliver-findings` each rewrite `run.json` from an in-memory copy with no lock; a `status` during the `deliver-findings` acknowledgement wait can lose either write.
- Evidence: code path. Known: no.

**15. Should-fix, B. Two acceptance scenarios pass without the acceptance tool.** `bench/harness/lead_driven.py:696-717`.
- With the record command pointed at a missing file, `acceptance-failing-check` and `acceptance-other-candidate` still reported `passes`: the verdict is only exit 7 and an unchanged ref, which "record missing" also produces.
- Known: no. Fix: check the record command's exit code and match the refusal reason.

**16. Should-fix, B. `bookkeeping-off-code-branch` and the launch case of `integration-branch` never run a worker.** `bench/harness/lead_driven.py:1840-1953`.
- Their `scripted_participant.py` files are not executable (exit 126, "Permission denied" in `stderr.log`); both scenarios pass.
- The harness writes the result, commits in the worker checkout and calls `import_candidate_to_shared_repo` itself.
- Stories 1, 2, 22 and 28 claimed by the first are not observed.
- Known: no.

**17. Should-fix, B. Results rows can overstate.** `bench/harness/lead_driven.py:2049-2105` and `:2564-2566`; `bench/harness/results.py:221-225`.
- `handoff --participant commit-ready --runtime codex --proof live` wrote a `live`, `codex`, `passes` row from the scripted binary in 0.4 s.
- A run that errors writes no row, so the earlier `passes` stays.
- An empty record list yields `passes`.
- The default `handoff` run covers 2 of 9 participants and skips story 4 while the row lists it.
- Known: no.

**18. Should-fix, A. The harness follows an inherited `GIT_DIR`.** `bench/harness/lead_driven.py:101-105`.
- With `GIT_DIR` pointing at a decoy repository, `land-exact-candidate` moved the decoy's checked-out branch and created two branches there.
- Under a git hook it would do this to the real repository (inference; I found no hook wiring).
- Fix: strip `GIT_*` from every child environment.

**19. Should-fix, B. Story 18 and story 11.**
- "Blockers have landed" is judged from the ticket's `Status: resolved` text (`tools/agents/lead_launch.py:238`); `tracker.py resolve` does not check a landing.
- Story 11: the lead's notes (13:00) record 0 tickets reviewed by a non-lead context. Known: yes, by the lead.

**20. Minor, A. The gate does not run the new tests.**
- `tools/agents/checks.d/` has no script for `test_worker.py` (72 tests) or `test_effort.py`.
- `test_lead_driven.py` runs only under `CHECK_BENCH=1`.
- `30-tracker.sh` lints one effort only.

**21. Minor, A. Help and documentation say things the code does not do.**
- `land.py --attempts` is validated and never used.
- The `--tmux-session` help mentions interactive launches, which do not exist on this branch.
- `AGENTS.md` still describes `land.py` as "rebase, gate the exact tip" and omits `lead_launch.py` and `acceptance.py`.
- `lead_launch.py wait` exits 0 on timeout as well as on a trigger.

**22. Minor, A. `worker.py launch` validates after side effects and mis-resolves `--base`.** `tools/agents/worker.py:864-885` and `:923`.
- `--handoff-mode marker` without a marker left the clone and run directory behind.
- `--base effort/demo` fails ("not a commit").
- `--repo <integration worktree> --base HEAD` silently used the main checkout's HEAD.
- `lead_launch.py` passes a SHA, so it is unaffected.

**23. Minor, B. Acceptance and move-records details.**
- `acceptance.py record` drops a failed check that was not named required: `--check tests=failed --required-check lint` gave `accepted: true` and stored only `lint` (reproduced). The caller chooses the required set, and no check is run.
- `effort.py move-records` makes two commits with no rollback between them; a rerun after an interruption is refused.

## Checked and found sound
- `land.py` never rebases, uses a compare-and-swap `update-ref` or `merge --ff-only`, serializes on a lock in the common git directory, runs gates in its own temporary worktree, and refuses stale, merge-commit and failing-gate cases without moving a ref.
- The external-state baseline is taken before the worker starts and reports modify, remove and add.
- Candidate pinning and `candidate-moved`.
- The private clone leaves the shared repository's refs alone until a ready Candidate is imported, and that import is compare-and-swap on the worker's own branch only.
- Git probes run with `GIT_OPTIONAL_LOCKS=0`.
- Launch refusals for triage, blockers and the parallel cap.
- The policy version header and manifest are bumped together.
- Read-only runs of `tracker.py lint` and `effort.py status` against the real repository complete in about a second and write nothing. `effort.py status` shows the same "STATE.md has no machine block markers" problem on two efforts that `main` already has.
- Stories 5, 7, 14, 15, 20, 27, 34 and 35 are tool-dependent in the scenarios.

## Not looked at
- Real codex or traecli runs, capsule and insula mode, real tmux, cargo, `check.sh`.
- `wt.py` beyond confirming it is unchanged.
- `claims.py`, `evidence.py`, `msg.py`, the `mac_*` and `sqlite_*` modules.
- Ticket-by-ticket acceptance items beyond tickets 02, 03, 05, 07, 10, 16 and 17.
- Ticket 18's in-progress worktree.
- `lead_driven.py`, `results.py` and the scenarios were reviewed by a sub-agent; I relayed its reproductions without re-running them.

## Verdict
Do not fast-forward the mainline yet. The lander's core mechanics are sound, but four things are wrong on real use here:
- workers are judged VIOLATED whenever any other agent commits (1);
- the review-rework cap and its receipts are not enforced on the command a lead would use (2);
- acceptance and tracker state are read from whichever checkout the command runs in, including a forged or stale copy, and landing this branch makes that the common case (3);
- resumed and reworked turns run outside the recorded boundary (4).

Fix those, plus the record-commit sweep (5), the push-then-fail landing (6), the `main` guard (7) and the stranded claims (8). Correct the scenarios that cannot fail (15–17) so the core-set claim rests on observations. The independent review that story 11 requires is still owed on the lead's own record. One real resume per runtime is needed to settle 4.

Cleanup: the detached worktree is removed and no processes are left. `repro.py` and the test logs remain in `/data02/home/philip.yang/devx/tmp/review-ib/`; the reproduction repositories are deleted, so re-running `repro.py` needs a checkout of the branch at the path it hardcodes.