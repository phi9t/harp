## Built
Bookkeeping can live on an explicit `Record branch: effort/<effort>-records` while code still lands on `Integration branch: effort/<effort>`.

I chose a separate record branch because it lets claims, reviews, acceptance records, notes and landing records be committed while workers keep running from the same code tip, and those records remain fetchable from a fresh clone. I rejected the old batching/uncommitted-record workflow because it preserves the exact write-order dependency this ticket is meant to remove.

The lead note `/data02/home/philip.yang/.cache/corenius/readiness/lead-worker-readiness/16-bookkeeping-off-code-branch/note-1.md` is included: the `--repo` effort copy wins, legacy no-integration records in other worktrees are not ambiguous, and `effort.py status` reports per-effort problems instead of failing the whole command.

Review round 1 is included: after a move, a worker still writes `docs/efforts/<effort>/evidence/NN-report.md` in its Candidate. When `acceptance.py record` accepts that Candidate, it imports the exact `git show <candidate>:docs/efforts/<effort>/evidence/NN-report.md` bytes into the record branch as `NN-report-round-N.md`. Later findings and claim-sampling links use the imported record-branch copy. `tracker.py findings` still reports a code-branch-only `NN-report.md` as `report NN (not yet accepted)` fallback, but launch briefs use only accepted record-branch report copies.

`move-records` does not alter already-running worker worktrees that still contain the old effort directory. Workers launched after the move use a code worktree without `docs/efforts/<effort>/`; their brief carries the pinned ticket and spec text plus content digests from the record branch.

## Acceptance
> - [ ] Scenario: a worker is launched; while it runs the lead claims a second ticket, resolves a third and records an acceptance for a fourth, all committed; the first worker's Candidate then lands without any rebase.

Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/lead_driven.py bookkeeping-off-code-branch --out "$out"`: passed with `passes bookkeeping-off-code-branch bookkeeping-while-worker-runs expected=checked observed=checked`. The full readiness loop over every `bench/readiness/*` scenario also passed after the review fix.

> - [ ] Scenario: an acceptance record is committed before its Candidate lands, and the landing succeeds; the write-uncommitted-then-land order is no longer required, and the help texts that describe it are corrected.

Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py` and `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_acceptance.py`: both passed. The land test commits an acceptance record on the record branch before landing and asserts the target fast-forwards and a landing record is written. The acceptance help test asserts the new help says records may be committed before landing and no longer says to commit after landing.

> - [ ] Launching a batch of three tickets leaves all three workers on the same base, equal to the code tip at launch.

Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: passed. The batch test launches tickets 01, 02 and 03 after records move to a record branch and asserts every worker branch and run base is the pre-claim code tip.

> - [ ] The effort's history still shows, for each ticket, who claimed it, each review round, the accepted Candidate and when it landed, readable from a fresh clone.

Proved by `PYTHONDONTWRITEBYTECODE=1 python3 bench/harness/lead_driven.py bookkeeping-off-code-branch --out "$out"` and the full readiness loop: passed. The scenario clones the repository fresh, checks out the record branch, and verifies the claim, resolve, acceptance, imported worker report and landing evidence files.

> - [ ] A ticket's brief contains the exact ticket and spec text it was launched with, by content digest, whether or not those files exist in the worker's tree.

Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: passed. Pinned-source coverage checks ticket/spec/instruction digests, the moved-record batch test checks ticket and spec source commits come from the record branch even though the integration checkout no longer has the effort files, and the new report-finding test checks later launch briefs read accepted report findings from the record branch.

> - [ ] An effort that has not been moved to the new arrangement works exactly as before; a command moves an existing effort and reports what it did.

Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_effort.py`, `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_tracker.py`, `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_acceptance.py`, `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_land.py`, and `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/test_lead_launch.py`: all passed. Legacy behavior remains covered, and `effort.py move-records` creates the record branch/worktree, annotates `EFFORT.md`, removes records from the code branch, and prints JSON describing the move.

> - [ ] Every existing scenario still matches its expectation; tracker lint is clean on every effort in the repository.

Proved by the full tools loop `for test in $(rg --files tools/agents | rg 'test_.*\.py$' | sort); do PYTHONDONTWRITEBYTECODE=1 python3 "$test"; done`: passed. Proved by the full bench loop `for test in $(rg --files bench/harness | rg 'test_.*\.py$' | sort); do PYTHONDONTWRITEBYTECODE=1 python3 "$test"; done`: passed. Proved by the full readiness loop under `.scratch` over every `bench/readiness/*` scenario: passed. Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/agents/tracker.py lint`: passed. Proved by `PYTHONDONTWRITEBYTECODE=1 python3 tools/test_sync_agentic_workflow.py`: passed. A first scenario loop using default temp placement failed partway with `No space left on device`; I removed the generated output directories and reran under a guarded `.scratch` temp root, which passed.

## Claims
- [R] effort metadata parses `Record branch:` alongside base and integration branch `tools/agents/land.py:88`
- [R] effort-record lookup returns the copy under the requested repository before considering other worktrees `tools/agents/land.py:182`
- [R] duplicate legacy records with no integration or record branch are reported as not in this checkout rather than ambiguous `tools/agents/land.py:193`
- [R] `effort.py` registers a `move-records` command `tools/agents/effort.py:595`
- [R] `move-records` creates the record branch worktree from the current code branch head `tools/agents/effort.py:342`
- [R] `move-records` removes the effort directory from the code branch checkout `tools/agents/effort.py:350`
- [R] `effort.py status` accumulates per-effort problem rows instead of aborting when one effort record cannot be selected `tools/agents/effort.py:530`
- [R] tracker commands resolve and write through the effort record root, with a local-effort fallback for legacy layouts `tools/agents/tracker.py:124`
- [R] no-argument tracker lint skips unrelated sibling worktree efforts unless they are explicit record-branch efforts `tools/agents/tracker.py:958`
- [R] acceptance imports accepted worker reports from the exact Candidate tree with `git show` `tools/agents/acceptance.py:90`
- [R] accepted worker reports are copied to `NN-report-round-N.md` in the record branch evidence directory `tools/agents/acceptance.py:97`
- [R] acceptance records store the imported worker report path, source path, source commit, round and digest `tools/agents/acceptance.py:103`
- [R] claim sampling switches to the imported record-branch report when one was imported `tools/agents/acceptance.py:317`
- [R] acceptance `review` and `request-rework` commands resolve the shared effort record root before writing records `tools/agents/acceptance.py:396`
- [R] tracker labels code-branch fallback reports as not yet accepted `tools/agents/tracker.py:424`
- [R] tracker reads pending code-branch report findings by Git ref, not by uncommitted worktree bytes `tools/agents/tracker.py:504`
- [R] `tracker.py findings` includes unaccepted report fallback findings while the shared loader default remains accepted-only `tools/agents/tracker.py:690`
- [R] `lead_launch.py` uses the accepted-only findings loader for launch briefs `tools/agents/lead_launch.py:155`
- [R] `lead_launch.py` reads ticket/spec text from the record branch and worker instructions from the code branch when building launch briefs `tools/agents/lead_launch.py:210`
- [R] when records are moved, batch launch captures the code tip before committing claims, then uses the post-claim record tip only for pinned ticket/spec sources `tools/agents/lead_launch.py:367`
- [R] `lead_launch.py rework` resolves rejected review records through the moved effort record root `tools/agents/lead_launch.py:430`
- [R] `land.py` appends and commits `landing-NN.jsonl` on moved record branches after a successful ticket landing `tools/agents/land.py:462`
- [R] the bookkeeping readiness scenario writes a worker report into the Candidate before acceptance records ticket 01 `bench/harness/lead_driven.py:1470`
- [R] the bookkeeping readiness scenario checks fresh-clone report-copy history and later findings from that imported report `bench/harness/lead_driven.py:1511`
- [R] the new readiness scenario is registered as `bookkeeping-off-code-branch` with case `bookkeeping-while-worker-runs` `bench/readiness/bookkeeping-off-code-branch/scenario.toml:1`
- [R] the issue-tracker doc states the moved-effort worker report import convention `docs/agents/issue-tracker.md:14`
- [R] the issue-tracker version bump is paired with the manifest version bump `docs/agents/agentic-workflow-repos.json:2`
- [R] the lead note required the `--repo` copy-wins lookup rule `/data02/home/philip.yang/.cache/corenius/readiness/lead-worker-readiness/16-bookkeeping-off-code-branch/note-1.md:22`
- [R] the lead review required accepted report copies to be named by ticket and round, and pending code-branch reports to be marked not yet accepted `/data02/home/philip.yang/.cache/corenius/readiness/lead-worker-readiness/16-bookkeeping-off-code-branch/review-1.md:22`

## Outside declared paths
- `docs/agents/agentic-workflow-repos.json`: required manifest version bump paired with the `docs/agents/issue-tracker.md` layout update.
- `docs/efforts/lead-worker-readiness/evidence/16-report.md`: required handoff report for this ticket.

## For later tickets
- to 14: before the live trial, this effort itself is to be moved with move-records at a moment when no worker is running; the lead does it.
- to 14: after recording acceptance on a moved effort, commit the record worktree so fresh clones contain `acceptance-NN.jsonl`, the imported `NN-report-round-N.md` file and later landing records.

## Not checked
I did not run `tools/agents/check.sh` or cargo, per ticket instructions. I did not launch real model workers; tests and readiness scenarios use fake/scripted participants. I did not fix the repeated `ResourceWarning` from `bench/harness/tasklib.py:90` or `test_worker.py` fake subprocess cleanup because the gate passed with those warnings and they are outside this ticket's scope.
