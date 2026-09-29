# Cross-Agent Collaboration: Lead and Follower

How two coding agents work the same effort in one repository at the same time,
without editing each other's working tree. Any agent can take either role,
whether Claude Code, TRAE, Codex or another. The human keeps decision and merge
authority (`docs/agents/agentic-engineering.md`, `CONSTITUTION.md`).

The machinery is `scripts/agents/relay.py`, tested in
`tests/test_agent_relay.py`. The templates are in `docs/agents/templates/`.

## When to use it

Use it when one agent owns a long-running execution environment and another
agent does planning, tooling, integration and review in parallel. The
execution environment might be the sealed insula, a GPU box, or a checkout with
hours-long builds. If both agents only edit code, give them separate tickets and
worktrees under the normal gated path instead.

## Roles

| | Lead | Follower |
|---|---|---|
| **Works in** | Its own worktrees only (`.worktrees/<name>`) | The primary checkout, and the long-running environment (for example `./run.sh`) |
| **Owns** | Specs and tickets, guards and tooling, the landing branch, review of follower commits | Captures, builds and insula proofs, its tickets |
| **Integrates by** | Keeping the landing branch rebased on the target and green | Fast-forwarding the landing branch into the target at turn boundaries |
| **Never** | Edits the follower's tree or the target ref; runs the follower's environment (for example the shared Bazel output base) | Merges non-ff or rebases the lead's branch; edits `.worktrees/*` |

The human decides direction, authorizes landing, relays the first message to the
follower, and settles anything either agent escalates.

## Invariants

1. **One writer per working tree.** No agent edits, restores, stashes or moves
   files in a tree another agent is using. That includes "helpful" cleanup and
   syncing into it while it's mid-turn.
2. **The target branch moves only by fast-forward.** The follower commits to it;
   landing is `git merge --ff-only <landing>`. If the fast-forward refuses, the
   lead rebases. The follower never force-merges.
3. **Durable intent lives in the tracker** (`.scratch/<effort>/`, see
   `docs/agents/issue-tracker.md`), not in chat or agent memory. Messages
   between agents are files, and the human relays only the first message.
4. **Guards are the contract.** Rules that matter are tests that fail on both
   agents' commits. Examples: exact ODR keys, provider version parity, the
   Python ABI literal ratchet. Neither agent negotiates them in prose.
5. **Only discard what you have proven superseded.** Before the follower drops
   dirty files, the lead records their content hashes (`relay.py manifest`);
   `relay.py land` refuses if anything else is dirty or the content has changed.
6. **The landing ref only ever points at verified commits.** The follower
   can fast-forward at any moment, so rebase and verify a detached HEAD and move
   the branch last (`relay.py rebase` does this). Never commit known-red state
   to the landing branch, even briefly.
7. **Green means the lead ran it on the exact tip.** Rerun the check after every
   rebase. A pass on a different or half-rebased tree doesn't count. Bazel's
   content-addressed cache hits are valid, because they're keyed on the exact
   inputs; hand-copied "it passed earlier" results are not.

## Artifacts

In `.scratch/<effort>/`:

- `spec.md` and `issues/NN-*.md` are the tracker. Claims go on the `Status:`
  line; evidence and `BLOCKED:` notes go under `## Comments`.
- `lead.md` is the lead state, built from the `lead-state.md` template. It is
  everything a new lead needs to take over (see "Lead handover").
- `<follower>-handoff.md` holds instructions for the follower, built from the
  `follower-handoff.md` template: a landing protocol at the top, ownership,
  freeze and guards, ordered work, and an `## Acknowledgement` section.
- `reviews.md` is the lead's one-line-per-commit review log of follower commits.

Lead-private state lives in the lead's scratch space, never in the repo: the
`sync-tracker` state file and the backups of superseded follower state.

## Protocol

### 1. Bootstrap (lead)

1. Inspect the follower's tree read-only: `git status`, recent commits, running
   processes. Identify its in-flight work and what it overlaps with.
2. Write the spec, tickets and handoff. Name the landing branch
   (`vaso/<effort>`) and create the lead worktree from the target tip.
3. If the follower has in-flight work the lead is taking over, snapshot it
   read-only (`git diff` plus copies of untracked files into lead scratch),
   harden it on the landing branch, then record the manifest:
   ```sh
   scripts/agents/relay.py manifest --follower <follower checkout> \
     --landing vaso/<effort> --tracker .scratch/<effort> \
     --superseded <paths…> --out .scratch/<effort>/landing-manifest.json
   ```
4. Commit the tracker on the landing branch. Until the first landing, keep an
   identical untracked copy in the follower checkout so the follower can read
   it, using guarded sync:
   ```sh
   scripts/agents/relay.py sync-tracker --lead-worktree .worktrees/<effort> \
     --follower <follower checkout> --tracker .scratch/<effort> \
     --state <lead scratch>/sync-state.json
   ```
5. Ask the human to relay one message to the follower: "Read
   `.scratch/<effort>/<follower>-handoff.md` and follow its landing protocol at
   every turn boundary." The message is delivered when the follower's current
   turn ends, not before.

### 2. Steady state

**Lead: watch.**

```sh
scripts/agents/relay.py watch --follower <follower checkout> \
  --target <target branch> --tracker .scratch/<effort> \
  --handoff .scratch/<effort>/<follower>-handoff.md \
  --follower-pid <pid> --busy-cmd '<exits 0 while mid-turn>'
```

The busy probe depends on the follower. TRAE CLI holds a `systemd-inhibit`
child during a turn: `pstree -p <pid> | grep -q systemd-inhibit`. Put the probe
you use in `lead.md`.

**Lead: runbook.**

| Event | Action |
|---|---|
| `TARGET_MOVED` to a follower commit | Review it against the tickets, freeze and guards, and add a line to `reviews.md`. Run `relay.py rebase --lead-worktree … --target … --verify '<host test suite>'`. It moves the landing branch only after verify passes. On `REBASE_CONFLICT` or `VERIFY_FAILED`, fix on the detached HEAD (keep both sides for additive files, and add follow-up fixes such as allowlists), rerun the verify, then `git checkout -B <landing>`. |
| `TARGET_MOVED` to the landing tip | The follower landed. Check the reflog shows a fast-forward, the tree holds only expected untracked files, the acknowledgement is present, and the pre-merge copy is gone. |
| `FOLLOWER busy -> idle` | A turn boundary. Make sure the landing branch is committed, rebased and green now; the follower may land within seconds. |
| `TRACKER_CHANGED` | If it matches the lead's sync state, the lead caused it; ignore it. Otherwise the follower edited the tracker: read it, and never overwrite it. |
| `ACK_CHANGED` | Read the acknowledgement and check its HEAD and test result. |
| `FOLLOWER_DELETED_TRACKED` | Check whether the target or landing branch still needs the file. If it does, ask the human to relay `git restore -- <path>` right away, and fix whatever instruction caused it. If the follower commits the deletion, restore the file by committing it again on the landing branch. |
| `FOLLOWER_EXITED` | Stop. Report to the human. |

**Follower: land at every turn boundary.**

1. Commit your own work first. Tracked files may only differ from HEAD if the
   manifest lists them as superseded. Untracked files are left alone unless the
   landing branch would create the same path.
2. Land:
   ```sh
   scripts/agents/relay.py land --manifest .scratch/<effort>/landing-manifest.json \
     --tracker .scratch/<effort> --verify '<host test suite>'
   ```
   Use `--landing vaso/<effort>` instead of `--manifest` once there is no
   superseded state. Run `--dry-run` first if in doubt.
3. On `REFUSED`, stop. Do what it says: commit, or wait for the lead's rebase.
   Never work around it.
4. Append the time, HEAD and verify result under `## Acknowledgement` on the
   first landing, then commit it with your next commit.

## Execution I/O hygiene

All agent disk I/O goes to a declared, disk-backed, per-agent root on the
estate volume; shared host scratch is never used implicitly.

- **Host verifies:** run them through `relay.py land|rebase --agent <name>
  --io-root "$VASO_ESTATE_ROOT"`, and use `"$VASO_BAZEL_OB"` as Bazel's
  `--output_base`. `TMPDIR` is pinned to `<root>/agents/<name>/tmp`.
  tmpfs/ramfs roots are refused, which matters because `/tmp` and `/var/tmp`
  are both tmpfs on the shared host.
- **Idle Bazel servers exit after 15 minutes** (`.bazelrc` `startup
  --max_idle_secs=900`).
- **Estate placement** checks the seat's real filesystem (`tools/estate.py`).
- **Insula:** the Bazel output base, caches and Spack store live under
  `/vaso/cache` on the estate disk. Per-run `TMPDIR` under `/vaso/tmp` and a
  bounded private `/tmp` are tracked in the effort's ticket 13.
- **Vigilance:** the lead's watch includes `--io-health /tmp /var/tmp`.
  Cleaning another agent's state needs the human's approval.

## Failure modes seen in practice (and the rule each one produced)

| Seen | Rule |
|---|---|
| The lead waited for the follower to go idle so it could land into the follower's tree. The follower committed every few minutes and its turns ran for hours. | The follower lands, and the lead only keeps the branch ready (invariant 1). |
| A naive `git merge --ff-only` refused because the follower's tree held superseded dirty files and an untracked tracker copy. | Use a manifest and move the tracker aside (`relay.py land`). The failure is safe, but it stalls landing. |
| A capture registered only package-local tests, so shared guards that scan declared Bazel `data` never saw its files (a false green). | Guards need a coverage check that compares scanned files against the source tree (for example `python_abi_literal_guard --coverage-anchor`). |
| Tests run on a half-finished rebase looked meaningful. | Rerun only after `git status` is clean and the rebase is complete (invariant 7). |
| Both agents edited the same JSON field (ledger `front`) and the rebase conflicted. | Give each shared file an owner, and have the other agent append only. Resolve by keeping the owner's text and re-appending your own line. |
| The lead's own tracker sync raised `TRACKER_CHANGED`. | Compare against the sync state before acting. |
| The follower was mid-landing (tracker moved aside) while the lead tried to sync. | `sync-tracker` refuses when the follower copy is missing or changed. It never recreates a copy. |
| The follower's uncommitted acknowledgement sat in a file the lead was about to change. | While the follower has a tracker file dirty, the lead must not change that file on the landing branch, or the next fast-forward refuses. |
| The follower did frozen work before the handoff reached it. | Record it in `reviews.md` as pre-notice. Allowlist the debt with a ticket pointer instead of blocking landing. Reverting is the human's call. |
| The lead's rebase moved the landing ref to a commit it knew needed a follow-up fix (a new guard on top of the follower's flip, allowlist not yet committed). The follower landed it within 30 seconds, and the target was red until the fix landed. | Rebase detached, verify, then move the ref (invariant 6). Fixes go in before the ref moves. |
| The follower created a fresh Bazel output base under `/tmp` for every verify. Thirteen of them used all `/tmp` inodes on a shared host, with 13 idle server JVMs. | Every agent uses one reused, disk-backed output base per role and shuts it down after verifying. The lead's watch should include a host-health check; cleaning another agent's state needs the human's approval. |
| The lead's own verify harness passed its command through an unquoted heredoc. The outer shell expanded `"$VASO_BAZEL_OB"` to empty, and Bazel built its output base (109,075 files) inside the source tree. `git add -A` committed it to the landing tip, which was pulled back before the follower landed it. | A verify must never write into the checkout: `relay.py` fails a verify that dirties the tree (`VERIFY_DIRTIED_TREE`). Run verifies through `relay.py`, not ad-hoc harnesses, and quote the `--verify` string with single quotes. Stage paths explicitly, never `git add -A`. |
| `git` output was `.strip()`-ed, which corrupted porcelain status parsing. | Parse `git status --porcelain -z` unstripped (covered in `tests/test_agent_relay.py`). |
| The follower re-ran a pre-landing "delete these untracked files" instruction after those files had become tracked, deleting the ODR gate's checker. It happened at every sync: a long-running follower thread replays commands from its own context and doesn't re-read an updated handoff file. | Never put one-shot destructive commands (`rm`, `restore`) in a handoff; put them behind a tool that verifies state (`relay.py land` with a manifest). When a protocol changes, have the human send the follower a direct message, because file edits don't reach a thread's context. The watch reports tracked deletions. |
| Unquoted `$VAR` in zsh doesn't word-split, so the command silently didn't run. | Run multi-word commands through `bash -c` or arrays, and assert expected output. |

## Lead handover

Any agent can take over as lead, or be added as a second lead on a different
effort, by reading the effort's `lead.md` and doing this:

1. Read `CONSTITUTION.md`, `docs/agents/agentic-engineering.md`, this file, and
   the effort's `spec.md`, `lead.md`, handoff and `reviews.md`.
2. Check that the landing branch is an ancestor-or-descendant of the target
   (`git merge-base --is-ancestor`), that the lead worktree is clean, and that
   the verify command passes on the tip.
3. Start `relay.py watch` with the parameters in `lead.md`.
4. Append a handover line to `lead.md`: date, agent, the landing tip it
   verified, and anything unresolved it inherited.

The previous lead must first commit or park its work, stop its watch, and
record in `lead.md` any open item that exists only in its context.
