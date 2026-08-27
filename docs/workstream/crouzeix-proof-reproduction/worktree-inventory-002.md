# Crouzeix proof worktree inventory 002

## Claim boundary

This is a local Git-state inventory, not a mathematical proof or permission to
discard unrepresented work. It records the state observed on 2026-08-27 at
`master` commit `a7d7654dc9db7ed4df42809427a7437d707a4e8f`.

The audit found 29 attached Crouzeix/CPFR worktrees and 32 matching local
branches: 29 attached branches and three branch-only refs. Eleven branch heads
are contained in `master`; 21 are not merged. No matching worktree had an
active process during the `2026-08-27 07:25:13 PDT` snapshot. A reviewer saw
an active `crouzeix-textbook` test afterward. At the final
`2026-08-27 07:38:30 PDT` snapshot the worktree had 16 tracked changes and one
active test process. The worktree is therefore preserved as live dirty work;
its counts are a dated observation rather than a stability claim. The primary
checkout's unrelated untracked paths were left untouched.

`Divergence` is `master-only/branch-only` commit count. `Patch delta` is
`git cherry master <branch>` reported as unique/equivalent branch commits. A
nonzero unique count is preserved even when another historical branch is said
to contain related work.

## Certification worktrees

| Worktree | Branch | Head | Merge base | Contained | Divergence | Dirty tracked/untracked | Patch delta | Cache role | Disposition |
|---|---|---|---|---|---|---|---|---|---|
| `cpfr-085-jin-certification` | `feat/cpfr-085-jin-certification` | `7c1963e0862d15937462f6611f98ef8878c21c9a` | `7c1963e0862d15937462f6611f98ef8878c21c9a` | yes | `70/0` | `0/0` | `0/0` | self-owned Atlas cache; consumed by preserved CPFR-090 | `preserve-cache-provider` |
| `cpfr-086-ls-certification` | `feat/cpfr-086-ls-certification` | `629b8aa9d369fc98434384c08fccecba7b60ff38` | `629b8aa9d369fc98434384c08fccecba7b60ff38` | yes | `30/0` | `0/0` | `0/0` | self-owned Atlas cache; consumed by CPFR-087/088/089 | `ordered-safe-remove-last` |
| `cpfr-086-ls-certification-design` | `docs/cpfr-086-ls-certification-design` | `56742c15cf62ea017526705e38b657d4122adcbc` | `56742c15cf62ea017526705e38b657d4122adcbc` | yes | `67/0` | `0/0` | `0/0` | self-owned Atlas cache; no consumer found | `safe-remove` |
| `cpfr-087-harp-certification` | `feat/cpfr-087-harp-certification` | `5209c19fca4212c27695f258faeb63ab50861b53` | `5209c19fca4212c27695f258faeb63ab50861b53` | yes | `11/0` | `0/0` | `0/0` | consumes CPFR-086 Atlas cache | `ordered-safe-remove-first` |
| `cpfr-088-six-route-bundle` | `feat/cpfr-088-six-route-bundle` | `0a5b855fa5a88fc18d45008fdea8c8143c834f72` | `0a5b855fa5a88fc18d45008fdea8c8143c834f72` | yes | `5/0` | `0/0` | `0/0` | consumes CPFR-086 Atlas cache | `ordered-safe-remove-first` |
| `cpfr-089-reader-reconciliation` | `docs/cpfr-089-crouzeix-proof-status` | `e08152dbf507dcc07261aa073973c8ea4a274ef2` | `e08152dbf507dcc07261aa073973c8ea4a274ef2` | yes | `2/0` | `0/0` | `0/0` | consumes CPFR-086 Atlas cache | `ordered-safe-remove-first` |
| `cpfr-090-three-route-finalization` | `review/cpfr-090-three-route-finalization` | `a7d7654dc9db7ed4df42809427a7437d707a4e8f` | `a7d7654dc9db7ed4df42809427a7437d707a4e8f` | yes | `0/0` | `0/0` | `0/0` | consumes CPFR-085 Atlas cache | `preserve-finalization-requested` |
| `cpfr-091-crouzeix-worktree-audit` | `chore/cpfr-091-crouzeix-worktree-audit` | `a7d7654dc9db7ed4df42809427a7437d707a4e8f` | `a7d7654dc9db7ed4df42809427a7437d707a4e8f` | yes | `0/0` | `2/0` | `0/0` | none | `preserve-active-audit` |

CPFR-086 may be removed only after CPFR-087, CPFR-088, and CPFR-089 are
removed and a fresh repository-wide symlink scan finds no remaining consumer.
CPFR-085 remains because the user requested preservation of CPFR-090 and that
worktree currently consumes CPFR-085's intact Atlas cache.

## Historical worker worktrees

| Worktree | Branch | Head | Merge base | Contained | Divergence | Dirty tracked/untracked | Patch delta | Disposition |
|---|---|---|---|---|---|---|---|---|
| `cpfr-011-worker` | `work/cpfr-011` | `385cd64b54ccc8a7274a906e1ec8404f2f529d5a` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/6` | `0/0` | `1/5` | `preserve-divergent` |
| `cpfr-012-worker` | `work/cpfr-012` | `64b4ba9f758ed510adbc54220fb0b21d4cca3375` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/4` | `0/0` | `1/3` | `preserve-divergent` |
| `cpfr-013-worker` | `work/cpfr-013` | `2d41cc02858e28d5a4f6ccd256e3f3db78d11b37` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/13` | `0/0` | `1/10` | `preserve-divergent` |
| `cpfr-014-worker` | `work/cpfr-014` | `9aedcf8c46d8c162d48ed6d11437e4b33f7d5950` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/13` | `0/0` | `1/10` | `preserve-divergent` |
| `cpfr-015-worker` | `work/cpfr-015` | `41710dc1b9003c75736bac378d9229673fb00a95` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/21` | `0/0` | `1/16` | `preserve-divergent` |
| `cpfr-016-worker` | `work/cpfr-016` | `da516c0843ecd54509337a40b3d75c092544b067` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/33` | `0/0` | `1/25` | `preserve-divergent` |
| `cpfr-019-worker` | `work/cpfr-019` | `2f7cdd2c37c36383a372cf3f1acd6c4fd864e217` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/23` | `0/0` | `1/18` | `preserve-divergent` |
| `cpfr-020-worker` | `work/cpfr-020` | `e1da4d66ac37dc7c7a258d737ad5b8b7c1f4f97b` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/36` | `0/0` | `1/27` | `preserve-divergent` |
| `cpfr-021-worker` | `work/cpfr-021` | `7a8bf39522165264b45834652952dc87f0c1b0ea` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/21` | `0/0` | `1/16` | `preserve-divergent` |
| `cpfr-r-evaluator-worker` | `work/cpfr-r-evaluator` | `77cf48e94b3be6e9d112ba0e203faf17bf5f2d66` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/42` | `0/0` | `1/32` | `preserve-divergent` |
| `cpfr-r-review-worker` | `work/cpfr-r-review` | `15735f16b507384bb75763266bf4f2145b8918a7` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/41` | `0/0` | `1/31` | `preserve-divergent` |

Each branch has at least one patch-unique commit. Statements in older inventory
notes that these branches were represented in another branch are not enough
to authorize removal; preserve them until each unique patch receives a
separate content-level disposition.

## Design and research worktrees

| Worktree | Branch | Head | Merge base | Contained | Divergence | Dirty tracked/untracked | Patch delta | Cache role | Disposition |
|---|---|---|---|---|---|---|---|---|---|
| `crouzeix-baseline-reconciliation-design` | `docs/crouzeix-baseline-reconciliation-design` | `38d1165cc467ab9cc351594e2b196cd83e7d10ee` | `430a9286d415c3bd343f05989474302d908c8255` | no | `135/2` | `0/0` | `2/0` | none | `preserve-divergent` |
| `crouzeix-end-to-end-goal-design` | `docs/crouzeix-end-to-end-goal-design` | `f89cb21b6484956740c187dda48250be8f3e67c7` | `f89cb21b6484956740c187dda48250be8f3e67c7` | yes | `91/0` | `0/0` | `0/0` | self-owned Atlas cache; no consumer found | `safe-remove` |
| `crouzeix-formal-validation` | `codex/crouzeix-formal-validation` | `1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf` | `1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf` | yes | `181/0` | `0/7` | `0/0` | none | `preserve-dirty` |
| `crouzeix-proof-reproduction` | `feat/crouzeix-proof-reproduction` | `3939e89a13bd4b2e058cc65498edfe78dc559ef4` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/83` | `0/0` | `34/38` | self-owned Atlas cache | `preserve-divergent` |
| `crouzeix-proof-runway-spec` | `docs/crouzeix-proof-runway-spec` | `aa1ebde3a4e5569f407c4e0ee441680e0aa5a751` | `430a9286d415c3bd343f05989474302d908c8255` | no | `135/2` | `0/1` | `2/0` | self-owned Atlas cache and primary Lean cache link | `preserve-dirty` |
| `crouzeix-retrospective-landing-spec` | `docs/crouzeix-retrospective-landing-spec` | `fb0ad7248bedd3b6105fb72b2af1d3bb3aca3598` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/58` | `0/0` | `9/38` | none | `preserve-divergent` |
| `crouzeix-rsi-implementation` | `codex/crouzeix-rsi-implementation` | `5b1aef4d18212d8de57a79295313d2dde1e01c58` | `430a9286d415c3bd343f05989474302d908c8255` | no | `135/13` | `0/0` | `13/0` | self-owned Atlas cache | `preserve-divergent` |
| `crouzeix-rsi-workflow-design` | `codex/crouzeix-rsi-workflow-design` | `530e0edeb1f22eade7888552df7f5106aec48276` | `530e0edeb1f22eade7888552df7f5106aec48276` | yes | `313/0` | `1156/356` | `0/0` | none | `preserve-dirty` |
| `crouzeix-rsi-workflow-spec` | `codex/crouzeix-rsi-workflow-spec` | `55aa1b76254b7cf5756865827af1ed3bbc888049` | `430a9286d415c3bd343f05989474302d908c8255` | no | `135/2` | `0/0` | `2/0` | none | `preserve-divergent` |
| `crouzeix-textbook` | `codex/crouzeix-textbook` | `32bc0f9e10cbbd1013b297e5c9fc1b3ea1cd3474` | `8723d1c01bb6aae1e6eab133a3614c3056cdd002` | no | `93/133` | `16/0` | `132/1` | self-owned Atlas cache and primary Lean cache link | `preserve-active` |

The 16 dirty tracked files observed in `crouzeix-textbook` include three
derived Atlas outputs, two structured content files, one test, one design
specification, three Lean files, five canonical knowledge files, and its import
receipt. They are live user work and are not enumerated as a cleanup target.

The `crouzeix-rsi-workflow-design` index is quarantined: it records 1,156
tracked changes plus 356 untracked paths. Do not clean, remove, reset, or merge
that worktree without a separate recovery decision.

## Branch-only refs

| Branch | Head | Merge base | Contained | Divergence | Patch delta | Disposition |
|---|---|---|---|---|---|---|
| `backup/crouzeix-conjecture-pre-rebase` | `0aa3d46821ae5500fd4da7fddc06832b4128e895` | `c092c564d69ad37d07f80a908ba056d9157bdd0e` | no | `333/5` | `5/0` | `preserve-divergent` |
| `backup/feat-crouzeix-proof-reproduction-context-parity` | `3939e89a13bd4b2e058cc65498edfe78dc559ef4` | `9ca3f3a22812346a4b26c5f976298758fac49563` | no | `282/83` | `34/38` | `preserve-divergent` |
| `docs/crouzeix-conjecture-deep-dive` | `0aa3d46821ae5500fd4da7fddc06832b4128e895` | `c092c564d69ad37d07f80a908ba056d9157bdd0e` | no | `333/5` | `5/0` | `preserve-divergent` |

The two refs at `0aa3d46821ae...` alias the same divergent history. The backup
ref at `3939e89a13bd...` aliases the preserved proof-reproduction worktree. No
branch-only ref is safe to delete in this phase.

## Removal plan

The following worktrees are clean, contained in `master`, patch-equivalent,
inactive, and not protected by the user:

### Ignored-state classification

Removing the six candidates will also remove about 9.2 GiB of ignored local
state. That state was inspected before authorizing deletion and is limited to
the repository's declared generated/build classes: `.build/`, `target/`,
`atlas/dist/client/`, `atlas/tsconfig.tsbuildinfo`, Python `__pycache__/`
directories, `formalization/lean/.lake` symlinks to the canonical primary
cache, and either Atlas-cache symlinks or worktree-local `atlas/node_modules/`
installations. No candidate has an untracked non-ignored path.

The ignored `.build/` trees include transient test logs and receipts. They are
not canonical proof evidence: the durable route receipts, reviews, six-row
bundle, program-verification receipt, tracker, and ledger are tracked and
already contained in `master`. The remaining ignored classes are rebuildable
outputs or dependency caches. Their deletion is intentional, local, and not
recoverable from Git; the authoritative tracked records remain recoverable
from `master`.

### Just-in-time deletion gate

Immediately before removing each worktree, rerun all of the following and stop
on any mismatch:

1. Confirm `git worktree list --porcelain` still binds the exact path, branch,
   and head recorded in this inventory.
2. Require both tracked and ordinary untracked status to be empty with
   `git -C <path> status --porcelain=v1 --untracked-files=all`.
3. Re-enumerate ignored paths and require every top-level ignored path to be
   one of the generated/build/cache classes listed above.
4. Require no process command line to reference the worktree path and no open
   file under the worktree from a build/test process.
5. Recheck that the branch is an ancestor of `master`, has divergence
   `<master-only>/0`, and `git cherry master <branch>` has no `+` entry.
6. Re-scan every worktree's `atlas/node_modules` symlink before deleting a
   cache provider. A provider may be removed only after its consumers have
   disappeared and no remaining symlink resolves beneath that provider.
7. Remove the worktree from the primary checkout with non-force
   `git worktree remove <exact-path>`. Delete its branch with `git branch -d`
   only after removal succeeds. Never use `--force` or `-D`.

1. Remove CPFR-087, CPFR-088, and CPFR-089 first.
2. Rescan all symlink targets. Remove CPFR-086 only if no remaining worktree
   refers to its Atlas cache.
3. Remove CPFR-086's design worktree.
4. Remove the end-to-end goal-design worktree.
5. Delete each corresponding branch only after its worktree removal succeeds.

Do not remove CPFR-085 because preserved CPFR-090 currently consumes its Atlas
cache. Do not remove CPFR-090 because the owner requested that finalization
worktree be preserved. Do not remove the active CPFR-091 audit worktree.

## Verification commands

```sh
git worktree list --porcelain
git for-each-ref --format='%(refname:short)%09%(objectname)' refs/heads
git status --short --branch --untracked-files=all
git merge-base master <branch>
git merge-base --is-ancestor <branch> master
git rev-list --left-right --count master...<branch>
git cherry master <branch>
ps -axo pid,ppid,etime,state,command
```

Every matching worktree and branch has an explicit disposition. Dirty or
patch-divergent work remains preserved. No worktree or branch was removed
during this inventory pass.

## Post-cleanup reconciliation

The reviewed removal plan was executed in dependency order with a fresh
fail-closed check before every target. No force option was used. Each branch
was deleted with `git branch -d` only after its worktree removal succeeded.

Removed worktree and branch pairs:

1. `cpfr-087-harp-certification` / `feat/cpfr-087-harp-certification`
2. `cpfr-088-six-route-bundle` / `feat/cpfr-088-six-route-bundle`
3. `cpfr-089-reader-reconciliation` / `docs/cpfr-089-crouzeix-proof-status`
4. `cpfr-086-ls-certification` / `feat/cpfr-086-ls-certification`
5. `cpfr-086-ls-certification-design` / `docs/cpfr-086-ls-certification-design`
6. `crouzeix-end-to-end-goal-design` / `docs/crouzeix-end-to-end-goal-design`

After the first three removals, a repository-wide symlink scan found no
remaining consumer of CPFR-086's Atlas cache. The second-wave JIT gate then
confirmed CPFR-086 and the two independent design worktrees were still clean,
inactive, contained in `master`, and had no patch-unique commit before their
non-force removal.

The post-cleanup inventory contains 23 attached Crouzeix/CPFR worktrees and 26
matching local branches: five contained and 21 unmerged. The five dirty
worktrees remain intact. CPFR-085 remains as the Atlas-cache provider for the
user-preserved CPFR-090 worktree. CPFR-090 and the active CPFR-091 audit
worktree remain present. All branch-only and patch-divergent refs remain
present. The primary checkout retains its unrelated untracked paths.
