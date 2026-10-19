## Multi-agent collaboration (git and shared resources)

Several agents may work on this repository and host at once. The Insula
candidate is a worktree's **staged index**, so sharing a worktree means
testing someone else's changes. The workflow is encoded in `milano/scm.py`;
use it rather than improvising.

- **Run `python3 -m milano.scm` from your own worktree.** `python -m` imports
  the `milano/` of the current directory, so running it elsewhere uses that
  checkout's (possibly older) tool.
- **Setup, once per clone:** run `python3 -m milano.scm setup`. It sets
  versioned hooks (`.githooks`), the `code.byted.org` push key, and
  `pull.rebase`.
- **One worktree per task:**
  `python3 -m milano.scm new --agent <you> --topic <task>` creates
  `../milano.worktrees/<you>-<task>` on branch `<you>/<task>` from
  `origin/diziet/mainline`. Work, stage and run gates only there. The main
  checkout on `diziet/mainline` is the integration point, so never commit or
  stage in it, and never touch another agent's worktree.
- **Commits:**
  - One task per commit, staging only that task's files.
  - The message says what and why, then `Verified in the Insula: …` with the
    gates run and any evidence run id.
  - Agents add their `Co-Authored-By:` trailer.
  - Tick the task's roadmap boxes in the same commit. If the plan conflicts on
    rebase, keep both sides.
- **Integration, never by hand:**
  `python3 -m milano.scm integrate <you>/<task> --gate repo [--gate "runtime-test"] ...`
  does the following, serialized by a lock:
  1. rebases the branch onto `origin/diziet/mainline`;
  2. runs each gate on the exact commit it will push
     (`run_local_insula.sh --rev`);
  3. fast-forwards `diziet/mainline` and pushes it;
  4. retries from the rebase if another agent pushed first.

  Choose gates that cover what changed. Anything touching `torch_accel/` needs
  `runtime-test`, and distributed code also needs
  `"accel-nccl --world-size 2 --collectives-only --ddp"`. Milestone exits need
  `package` then `b200-release`.
- **"Merge" means rebase plus fast-forward.** Mainline is linear: no merge
  commits, no force pushes, no deletion. The pre-push hook refuses them.
  Force-push (with `--force-with-lease`) only your own topic branch.
- **Cleanup:** once integrated, `python3 -m milano.scm finish <you>/<task>`
  removes the worktree and branch. `python3 -m milano.scm status` shows every
  worktree, its branch, ahead/behind counts and dirtiness.
- **GPUs are leased, never assumed.** Every GPU mode takes exclusive `flock`
  leases on `$MILANO_CACHE_ROOT/locks/gpu/gpu-N`, which are shared across
  sandboxes. It waits up to `MILANO_GPU_LEASE_WAIT` seconds (default 900),
  printing who holds each busy GPU. Qualification holds all 8 for its whole
  run, and its gates inherit the lease. `CUDA_VISIBLE_DEVICES` restricts
  which GPUs may be leased.
- **The cache is shared.** Content-addressed entries are safe to share.
  Never delete another run's state by hand, and run `cache-gc --apply` only
  after reviewing its dry run. Evidence is append-only.
- **Parallel runs from one worktree are fine.** The launcher and driver use
  `GIT_OPTIONAL_LOCKS=0`, so they never contend on `index.lock`.

