# Lead review 1 of ticket 17: the fix works for traecli and not for Codex

Reviewed Candidate: 25d8d0d. The private clone per worker, the import of only a pinned ready Candidate into
the shared repository, the capsule bind for the private `.git`, and the rework changes are right and stay.

You did not run the real probes because your standing brief forbids model runs and you judged that it
overrode the ticket's exception. You were right to say so plainly. I ran them myself with your launcher, in
a throwaway repository with a `main` and a `side` branch, same task as in the ticket, `--model gpt-5.5
--effort low`:

- **traecli runtime: passes.** `git add result.txt && git commit -m "add result"` exit 0; verdict CLEAN,
  ready for review, Candidate 481fe39 pinned; afterwards the shared repository has `worker/probe-traecli` at
  that commit, `main` and `side` unchanged, clean status.
- **Codex runtime (codex-cli 0.160.0): fails.** The worker created the file and then reported:
  `fatal: Unable to create .../r-worker-probe-codex/.git/index.lock: Read-only file system`. Verdict
  DEGRADED, not ready: dirty tree, no commit after base. Codex's `workspace-write` sandbox keeps a `.git`
  directory inside a writable root read-only. Moving the git metadata inside the workspace therefore fixed
  one runtime and not the one the owner wants first.

## 1. Make the Codex runtime able to commit under its default sandbox (blocking)

One arrangement to try first: create the private clone with its git directory outside the workspace
(`git clone --separate-git-dir <dir> ...`, so `<workspace>/.git` is a file pointing at it), put that
directory beside the workspace or under the run directory, and for the Codex runtime pass it as an
additional writable root (`-c 'sandbox_workspace_write.writable_roots=["<dir>"]'`). The worker can then
write its own git data and still nothing of the shared repository. Check what traecli needs for the same
layout; if the two runtimes need different layouts, say so in the code and the help. If no arrangement
works under `workspace-write` for Codex, stop with `needs-info` and the exact errors; do not fall back to
`danger-full-access`.

## 2. Run the real probes yourself this time

This review is the lead's instruction and, for this ticket only, it overrides the line in your brief about
not launching model workers: run the probe from the ticket through `worker.py launch` and `supervise` with
`--runtime codex --model gpt-5.5 --effort low` and with `--runtime traecli --model gpt-5.5 --effort low`, at
most three times each, each in a throwaway repository created under a fresh `mktemp -d -p` on the same
filesystem as this worktree (your report notes that a clone across devices cannot hardlink). Keep one
event stream per runtime under the effort's `evidence/` as `17-probe-<runtime>-events.jsonl`, and put each
run's verdict line in the report. Acceptance item 1 is met only by those runs.

## 3. Small things

- `git clone --local` fails when the workspace is on another filesystem than the repository. Detect that
  and fall back to a clone that copies or borrows objects, and say which was used in the run record.
- The report gives 219,848 KiB for a linked worktree and 324,428 KiB for a private clone. State what the
  separate-git-dir layout costs.
- Add under `## For later tickets`: `- to 18: a worker's workspace is a private clone, not a worktree of the
  shared repository; its branch exists in the shared repository only after a ready Candidate is imported.
  Monitors must read the worker's own repository for commits ahead, dirty state and the tip.`

## Protocol

New commits on top of 25d8d0d; no rewriting. Update the report. When items 1 to 3 are done and your gate
passes, commit and then STOP without the marker commit: ticket 10 lands before you and changes
`tools/agents/lead_launch.py`, `tools/agents/worker.py` and the harness module. Wait for my message telling
you to re-integrate; the `ticket 17: ready for review` commit comes after that rebase.
