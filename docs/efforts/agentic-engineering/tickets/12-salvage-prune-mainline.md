# 12 salvage-prune-mainline: Salvage pending work and prune Corenius branches/worktrees to a pristine mainline

Type: task
Status: claimed
Owner: codex-salvage-prune
Blocked by:
Claimed: 2026-10-07T09:14:21Z expires 2026-10-08T09:14:21Z

Triage: needs-triage

## Acceptance
- [x] Identify and land all currently landable work with independent reviews and required gates.
- [x] Preserve every remaining unique commit, pending file, useful ignored artifact and recovery mapping before retirement.
- [x] Retire recovered inactive worktrees and prune eligible branches without losing live-owner work.
- [x] Preserve mainline private/untracked artifacts outside the checkout and verify pristine main status.
- [x] Audit current refs/worktrees and recovery coverage against the complete objective.

## Scope and authority

User explicitly requests all landable work, fixes for encountered issues, a pristine
mainline and salvage/pruning of all branches/worktrees. This extends Ticket11’s
reviewed queue into repository cleanup. Gated path: broad destructive-data scope;
preservation and exact-target verification must precede retirement. User’s explicit
pruning instruction authorizes recovered inactive checkout/ref retirement. Never
force-delete unresolved work or mutate live-owner trees. Keep main and policy-required
ultron/mainline; record any other unavoidable active ownership separately.

## Progress

See ../evidence/salvage-prune-20261007-progress.json. Fresh initial inventory124
worktrees/134 branches; all history bundled and verified. Four clean, merged,
artifact-free inactive trees retired, with three non-force merged branch deletions.
120 remaining trees received stable pending-file snapshots (5,498 entries).
Ignored scratch and live-owner work remain preserved in place, requiring further
salvage and retirement. Main currently has its transcript and nested worktree
untracked; pristine completion is not yet proved.

## Comments
- 2026-10-07T09:14:21Z codex-salvage-prune: claimed until 2026-10-08T09:14:21Z

## Second cleanup increment

Main’s tracked/untracked status is clean after preserving the transcript and the
superseded nested Sureal report outside the checkout. This criterion is verified;
final completion still requires a fresh whole-goal audit. Current inventory58
worktrees/73 branches. Eleven clean merged trees and49 matching pending trees
were preserved by same-filesystem directory renames, then registrations/merged
refs retired. The nested tree was independently reviewed and retired. One cross-
filesystem archive passed12,359 file digests including43 hardlinks before removal.
Bundle-only recovery reproduced original pending status for61 directory archives.
See ../evidence/salvage-prune-20261007-archives.json for full receipts and hashes.

The live Ticket03 owner remains in place; user routing question is pending. Its
latest repair has47 affected tests passing but no completed full gate or independent
acceptance. The integrating watcher and Kata services also retain their checkouts.
No stale/parked variant was activated merely to eliminate its branch.

## Third cleanup increment

Thirty inactive same-filesystem trees and19 cross-filesystem trees are fully
preserved; every retirement was verified through bundle-only status reconstruction.
Nineteen orphan refs were also salvaged and pruned with exact expected-tip guards.
Three historical readiness trees were recovered and retired; current EFFORT/STATE
routing no longer points at their retired branches. Tickets14/15 remain open.
Two independently verified orphan Kata evaluation fixtures were stopped gracefully
through their pinned daemon CLI, their complete homes preserved, and the evaluation
checkout recovered and retired. Unix endpoints were verified unbound and retained
in raw backups, with all regular-file bytes independently copied and hashed.

Current inventory5 trees/5 refs: main, this owned integration, the persistent Kata
spike, controller integration watcher, and the live Ticket03 owner. Main is clean.
The user routing questions for the remaining live owners are pending; this goal
remains active. See ../evidence/salvage-prune-20261007-final-inactive.json.

## Operational pointer verification and handoff

Managed dehydration explicitly forgets recovered retired checkouts in all five
machine-bearing efforts. Old machine snapshots remain in Git history and archives;
worker-run history remains retained. This prevents a future rehydrate from restoring
retired branches. Readiness’s historical duplicate headings were retagged after the
real briefing selected superseded next actions; the corrected briefing selects
current instructions and preserves14/15 as open. Main advancement may produce the
effort tool’s WARN1, while missing/diverged pointers remain errors and are rejected.

After this metadata publication, the owned reconciliation checkout is scheduled
for full-directory/metadata archival, bundle-only recovery, and guarded ref cleanup.
Its final receipt will be retained in the recovery archive named in the ledger.
The only remaining user-owned/live checkouts will be the persistent Kata spike,
controller watcher and live Ticket03 owner. Pending routing questions are not
approvals. No goal-complete claim or ticket resolution is made yet.

## Controller03 bounded salvage landed

Ticket13 repaired the frozen pending controller delta, retained current readiness APIs, and corrected three Kata
defaults broken by checkout retirement. Both independent reviews and all final gates passed, including94 mixed
tests,41 task validators,6 default controller cases and the real native Kata protocol fixture. Main was clean
after fast-forward publication to89ff376. The owned candidate was completely recovered and retired; see the
controller03 retirement evidence. The original live owner, integration watcher and persistent Kata spike remain.
Their pending routing questions are not approvals, and this whole-goal ticket stays claimed.

## Owner-authorized final cleanup

The user explicitly authorized graceful shutdown and handoff of the final three live owners. All five PIDs
are missing, and the post-stop process scan found no remaining process using their checkouts. All original
checkouts except main are recovered and retired, with their complete files/indexes/history retained and
independent dirty-status reconstruction. Kata service data is retained in place and separately hash-copied
(1,128 files/102,472,140 bytes). Independent review found no additional landable changes and accepted
original Ticket03; its deterministic source is already on main. Future04/05 are not implementations to land.
The only remaining registered checkout besides main is this sole-writer metadata candidate; it will be
archived/recovered and retired after publication. Whole-goal completion is asserted only after that final
retirement and a fresh one-checkout/two-required-refs audit recorded outside the tree.
See ../evidence/salvage-prune-20261007-final-live.json and the archive final completion receipt.
