# Collab Ops architectural escalation

Read commit: `3c812cf1a11f57d75c6bfd0a6965b7cdb1142478`; fixed base: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.

Status: **attention needed; all four third-review blockers reproduced**. This is a read-only architectural review after the three repair rounds, not another implementation/review round. No source, policy, tracker, production worktree, branch or active session was changed. The supplied gate/conformance passes remain useful evidence but do not discharge these demonstrated failures. I did not rerun the broad gate.

Scope: source was read from the named checkout and executed from an unmodified private `git archive` extract. The report, script, logs and real disposable Git repositories are under `/data02/home/philip.yang/.codex/reviews/collab-arch-3c812cf-20261012`. Host commands used `tools/temp_root.py`; subprocesses inherited its owned absolute TMPDIR. Fixtures are from the frozen public-CLI suite. Calls run actual Git, a real bare origin, actual tracker subprocesses and public Collab/legacy CLIs. Failpoints are existing product hooks. The only lock injection directly holds the existing flock file to model the lane fence; this is explicitly narrower than reproducing a simultaneous publication race.

## Verified observations

Each case is independently constructed; exact commit IDs, paths, state/output and commands are preserved in `observations.json` and `commands.jsonl`. All nine final variants reproduced. One initial supplemental legacy invocation used an unsupported `--base` flag and exited before publication; the log retains that setup mistake. The corrected invocation removed the flag and reproduced the failure. This did not change product code.

| Finding | Public CLI sequence | Exact result |
| --- | --- | --- |
| High: Corenius admission bypasses pending publication | Check Attempt A; `land A --review-json …` with `after_git_push`; `open-attempt 02 --effort sample-effort --owner agent-a`; `recover A --confirm-stopped` | Landing exits 86 with remote at A's Integration commit and local at A's base. Admission succeeds, creates a competing local claim commit and a working Attempt. A recovery exits 2 with `Ambiguous pushed landing`. Remote remains at A's Integration commit. |
| Same allocation-recovery class | Reserve Attempt B using `after_claim_record`; push A with `after_git_push`; `recover B --confirm-stopped`; `recover A --confirm-stopped` | B recovery succeeds and commits the tracker claim on A's old local base. A recovery then exits 2 with the same ambiguity. Acquiring the lane without checking pending operations is insufficient. |
| Legacy pending publication also bypassed | Real `tools/agents/land.py topic --mainline main --owner-mainline-override owner --gate true` with `after_legacy_push`; admit Corenius ticket 02; legacy `--recover-landing ID` | Legacy push exits 86 and advances remote only. Corenius admission succeeds and forks local main from that base. Legacy recovery exits 64: `Target moved beyond the bound landing; preserve and inspect`. |
| Admission ignores active lane fence | Hold `.git/collab/lane.lock` with nonblocking exclusive flock; invoke Corenius admission in a separate process | Admission succeeds, commits the claim and reaches `working`. This verifies the missing lane fence. A timing race with a simultaneously executing legacy Git child is a static inference, not separately reproduced. |
| High: empty Git operation omitted from core preflight | Prepare/check standalone Attempt; actual `git cherry-pick BASE` (already applied commit); `git status --porcelain`; `land A` | Cherry-pick exits nonzero and leaves real `CHERRY_PICK_HEAD`; porcelain is empty. Land advances remote to the Integration commit, leaves local at base and exits 2: `fatal: You have not concluded your cherry-pick (CHERRY_PICK_HEAD exists)`. Attempt remains `landing_started`. |
| Medium: unusable run identity permits removal | Bind `--worker-run-dir` containing `run.json={}` without `exit_code`; `inspect`; `abandon A --confirm-stopped`; `gc A --confirm-stopped` | Inspect returns `blocked:false`, no blockers. Abandon succeeds as `abandoned`; GC removes the workspace. Its branch/history are retained, but the workspace has been deleted without a usable worker identity or completion record. |
| Same invalid identity class | Repeat with `{"pid":"broken","process_session_id":"broken"}` and no completion record | Same successful abandonment/deletion. These trials demonstrate unsafe treatment of unknown state; they do not claim an actual live process was killed. |
| Medium: remote outage blocks local-success reconciliation | Push/land standalone Attempt with `after_git_land`; rename private bare origin offline; `recover A --confirm-stopped` | Local HEAD already equals the Integration commit; Attempt is `landing_started`. Offline recovery exits 2: `Cannot inspect pushed remote during recovery`. Restoring origin permits recovery to `landed` without changing local HEAD. |
| Same failure after bookkeeping success | Corenius land with `after_records_land`; make origin offline; recover | Local HEAD is the attributed bookkeeping commit descending from Integration; state remains `landing_started`. Offline recovery exits 2 with the same remote-inspection error. Restoring origin repairs the state to `landed`, leaving local HEAD unchanged. |

There is no evidence that the frozen branch can be presented as ready to land. Preservation and the existing bounded escalation remain the correct outcome.

## Cause and smallest coherent resolution

**Inference from the verified behavior and source:** this cluster is not four unrelated missed conditions. Permission to mutate trunk is distributed across `core`, `landing`, `ticket_source`, `ticket_resolution` and `legacy_landing`; core and legacy preflight differ; recovery mixes remote publication knowledge with local completion; liveness collapses absent evidence into stopped. Adding another condition to each route would retain the same duplication.

Deepen the existing `landing_protocol.py` into **one repository-mutation module**. Keep the public CLI and shim seam unchanged. Do not introduce a generic transaction framework, new scheduler, new authority model, registry of hypothetical adapters or a second ledger. Core and legacy are the two existing adapters for operation records. StateStore owns SQLite; LegacyJournal owns compatibility records; tracker remains the ticket-file writer.

Its small interface should express three facts: the exact operation being executed/resumed, the exact target checkout/ref and expected predecessor, and whether the effect is a local commit or an exact fast-forward. Suggested shape for design review:

```python
with mutations.begin(operation) as turn:
    turn.commit_tracker_change(target, change)  # claim/release/bookkeeping
# or:
with mutations.begin(operation) as turn:
    turn.publish_checked(target, checked_tip, destination)  # optional push + local FF
```

`operation` is the existing bound Attempt or legacy operation, with an explicit phase; resumption is derived from its journal, never a caller-supplied “ignore pending” switch. `change` uses the actual tracker or effort-record writer, not arbitrary shell commands. These methods are a proposed interface, not an implemented contract. The design review must settle whether preparation stays in the callers or inside the methods; whichever is chosen, callers cannot publish or commit through an unguarded Git helper.

The implementation owns canonical repository identity, lock order, pending-operation admission, checkout preflight and inherited lock descriptors for Git/tracker children. It uses the existing locks: compatibility `land.lock` first when required, then Collab `lane`, then `meta`; callers needing `check` acquire it before those locks. It checks both core and legacy unresolved operations under the same fence. A resumption may exempt only its own exact operation, while conflicting pending operations still block. Failed acquisition performs no tracker/file/ref changes. A child surviving its controller retains the fence through the existing descriptor-passing mechanism. Gate execution stays outside publication locks, followed by a fresh preflight immediately before effects.

Use one checkout preflight everywhere: repository/worktree/ref identity, exact predecessor, ancestry for FF, hidden index flags, staged changes, changed-path overlap with untracked/ignored files, and operation markers resolved using `git rev-parse --git-path`. Reuse legacy's existing marker set (`MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`, `rebase-merge`, `rebase-apply`, `sequencer`). Preserve the existing cleanliness contracts: core remains strict; compatibility may retain disjoint unstaged operator edits. Share the safety rules without silently broadening deletion or narrowing compatibility. Repeat checks before recovered local FF; no remote write occurs if a known local blocker exists.

All managed target writers must enter the same module, including paths already fenced:

| Writer | Required integration |
| --- | --- |
| `Collab.open_corenius_attempt` → `TicketSource.claim_corenius` | Fence and pending-operation admission before reserving/committing a claim. |
| `LandingManager._recover_allocation` → claim | Resume allocation under the same admission rule; it cannot exempt unrelated landing A. |
| `LandingManager.land`, `_fast_forward_primary`, pushed/local recovery | Shared publication/preflight implementation. |
| `LandingManager.abandon`/`_finish_abandon` and `TicketSource.finish_claim_release` | Release commit also refuses unrelated core and legacy pending landings. |
| `TicketResolution._land_records` and record recovery | Resume only the bound record effect; share checkout/predecessor guards. Clone preparation itself is not trunk mutation. |
| `legacy_landing.land`, `recover_landing`, `fast_forward_ref` (called by `land.py` and `wt.py land`) | Both initialized and uninitialized repositories use the same lock/preflight behavior; do not auto-initialize Collab. |
| Optional `effort_records.commit_effort_record_change` | Guard the actual configured record checkout/ref; it can coincide with a protected target. |
| `lead_launch.claim_batch` and `release_unlaunched_claims` | These retained toolbox functions directly commit on `record_root`, which may be trunk. Tiny adapters must use the shared guard if the guarantee covers all managed trunk writers. Agent launching itself remains out of scope. |
| `effort.cmd_dehydrate` and `cmd_move_records` | Their existing commits can target the code/record trunk; guard before any file mutation. Do not execute migration or change its semantics. |

`tracker.py` writes ticket bytes but does not itself commit trunk: callers must enter the guard before invoking it. `collab new` writes an uncommitted ticket, so use the guard for primary-checkout mutation without inventing automatic commits. Refresh writes its fresh Attempt, check runner writes an isolated Integration workspace, worker fetches use candidate refs, and archive/sweep use recovery or disposable branches: they do not publish trunk. Keep their existing ownership/retention rules. Direct human Git remains normal trunk movement, detectable by exact-predecessor checks; cooperative local controls cannot fence arbitrary external Git commands. The separate retained controller and disposable baseline/demo writers are excluded.

**Scope decision requiring human approval:** the toolbox adapters above are necessary for an honest “all managed trunk writers” claim. If follow-up is limited to Collab only, explicitly limit the guarantee and retain the adjacent-tool bypass as an unresolved risk. Do not silently broaden the patch into controller or migration redesign.

## Strong liveness decisions

Keep one liveness module used by inspect, recover, abandon, refresh, GC, sweep and retention. Return an explicit decision `running | stopped | unknown | unbound` with source evidence and reason. Absence of usable identity is `unknown`, not stopped. A run record must be a mapping; usable identities are positive integers excluding booleans and lossy numeric coercions. Missing/malformed/unreadable identity, inaccessible process observation, malformed completion or unsupported record shape cannot authorize cleanup. Existing valid PID/run-session checks remain; observable live processes win over an exit marker, and an unknown source blocks release when another source merely looks stopped. Preserve explicitly unbound Attempts with the existing manual `--confirm-stopped` contract; do not conflate “no binding requested” with “a supplied binding is broken”. No new process supervisor or mandatory schema migration is needed for these cases.

Accept stopped only from usable observed process identity with no live bound session, or valid bound completion evidence with no contradictory live observation. Validate completion contents rather than treating any nonempty `exit_code` as proof. Consumers ask for the decision; they do not reconstruct permission from `alive == false`.

## Recovery: three independent effects

Use existing durable events/refs, distinguishing:

1. **Remote publication:** push intended, acknowledged, affirmatively rejected, or uncertain. Remote observations are needed to reconcile this effect when local FF remains outstanding. Nonzero push plus absence/base observation never proves non-execution; preserve/no retry remains correct.
2. **Local publication:** exact Integration commit or its bound bookkeeping descendant is already on target with the existing operation attribution/receipts, versus target still at expected predecessor, versus divergent target. Establish this from local history/ref and operation evidence before attempting remote I/O. Mere unrelated trunk movement must not authorize record repair.
3. **Record repair:** finish only the bound tracker/bookkeeping operation, idempotently, from established local publication. This requires its original Check receipt, path/identity and prepared record commit checks; it does not depend on remote availability and cannot repeat push or Integration FF. Preserve a separate remote-unknown observation rather than claiming the remote is currently reachable or unchanged.

For both core and legacy recovery: check valid operation evidence and local publication first. If established, repair records and local state without `ls-remote`. If local is still base and this was a pushed operation, require exact remote acknowledgment before preflight and local FF. Otherwise retain ambiguity. A remote outage may block an outstanding local publication, but cannot block repair of established local success. Existing `landed_records_incomplete` remains meaningful; no general state-machine replacement is required.

This proposal supports ADR-0002 and the SPEC completion clarifications (tracker ownership, separately attributed bookkeeping and conservative unknown-writer retention). It does not change merge authority or the three-repair budget. The module/interface and adjacent-tool scope remain proposals until human approval; no ADR or specification was edited.

## Acceptance checks for separately authorized follow-up

Use the already approved public CLI/shim seam and real disposable Git repositories. Convert the retained reproductions into behavioral regressions only after authorization.

- Pending core/legacy publication plus admission, allocation recovery or claim release: refuse before ticket bytes/index/refs/attempt reservation change; recovery of original exact Integration tip still succeeds. Include held-lane admission and a real surviving Git child whose inherited fence remains busy.
- Every guarded target writer listed above: unresolved unrelated operation blocks its target mutations, while its own exact recovery works. Preserve uninitialized legacy usage, configured mainline, exit-code compatibility and normal external trunk movement behavior.
- Real empty cherry-pick: porcelain clean but marker present; core and legacy refuse before push; local/remote refs, marker, operator files and operation evidence remain intact. Cover the existing marker set through public CLI, not source assertions.
- `{}`, missing/invalid/nonpositive PID/session, non-mapping run JSON and malformed completion: inspect reports unknown; abandon/recover/refresh/GC/sweep retain all workspaces/refs. A valid stopped run succeeds; a live child/session refuses despite an exit marker; unbound confirmation remains supported.
- Offline after `after_git_land`, `after_records_land` and real tracker-record failure yielding `landed_records_incomplete`: recovery repairs only local records/state without remote query/retry or duplicate acceptance entries. Offline after push but before local FF still preserves uncertainty; restoring exact remote acknowledgment permits safe FF. Divergent local/record evidence refuses.
- Run affected toolbox compatibility suites, Collab suite, lint and the repository default gate once focused checks pass; fresh Standards/Spec and independent branch review must be clear before readiness. Any newly authorized remediation budget must be explicit, bounded and recorded separately; this report grants none.

Reproduce all final variants (creates only new owned disposable repositories):

```bash
TMPDIR=/data02/home/philip.yang/.codex/reviews/collab-arch-3c812cf-20261012/tmp \
PYTHONDONTWRITEBYTECODE=1 \
python3 /data02/home/philip.yang/.codex/reviews/collab-arch-3c812cf-20261012/frozen/tools/temp_root.py -- \
python3 /data02/home/philip.yang/.codex/reviews/collab-arch-3c812cf-20261012/reproduce.py
```

Add `--case admission|allocation|legacy-admission|lane|operation|liveness-empty|liveness-invalid|recovery|records` for one variant. The script asserts the observed defects, not a repaired result. Logs are append-only; observations replace only the selected cases. All private trial repositories are retained for inspection, including the intentionally removed workspaces in liveness tests. Existing production sessions/refs and source worktrees remain untouched.
