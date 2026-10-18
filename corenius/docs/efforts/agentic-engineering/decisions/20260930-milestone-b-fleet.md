# milestone-b-fleet

Date: 20260930
Supersedes:

## Decision

Milestone B Tier 1 features F5 to F10 are built by a fleet of six traecli workers (GPT-5.5, xhigh, inside the `agent` capsule, one worktree each),
specified in `docs/efforts/agentic-engineering/milestone-b/SPEC.md` with three contracts and six tracked ticket files. The lead owns contracts, tickets, shared
files (`check.sh`, `AGENTS.md`, the feature register), verification and landing. Scope: Tier 1 only. Landing: the lead verifies, then lands locally by
fast-forward; nothing is pushed (owner decision, 2026-09-30).

## Why this shape

- The couplings are real but contract-shaped: `wt.py` loads `land` through a fixed plug-in hook; `claims.py` writes against the evidence record API. Shared files are
  lead-owned and every feature adds one `checks.d/NN-name.sh`, so workers never edit the same file (conflicts are designed out, not manufactured).
- Workers cannot run git on their own tree or on the real repo inside the capsule, so their tests use throwaway repos; real-repo acceptance is run by the lead.
- `tools/agents/check.sh` (new, green on the existing suites before dispatch) is the gate; it runs the existing suites, the tracker lint once it exists, and every
  `checks.d` script. Cargo runs only with `CHECK_CARGO=1` (the main checkout still carries uncommitted AIME work; the feature spec's Decision 2 stays open).

## Consequences

- Each worker's verdict is judged by the deterministic supervisor; a CLEAN verdict means the boundary held, not that the work is right. Verification notes and outcomes are
  appended to this record.
- After landing, F5 to F10 are used on this run (land T3-T6 through `wt.py`/`land.py`, tracker list/lint over these tickets, evidence records of the six runs, claim sampling of the
  lead's notes, handoff messages), and the feature register is updated.

## Dispatch log

- Round 1 (08:09Z) was a dispatch mistake of mine: the generated task preambles named `checks.d/NN-name.sh` with the ticket number (`04-msg.sh`), while the write allowlist and
  the ticket used the checks number (`40-msg.sh`). The `msg` worker stopped after 48 s and said so instead of violating its boundary (DEGRADED, no files); the other five
  were running with the same text. I killed all six (the first kill only reached the launcher shell; the `timeout` and capsule processes had to be killed separately),
  confirmed every worktree was clean, removed the six empty trial worktrees and branches, fixed the preamble (the three file names are now stated as authoritative), and
  relaunched at 08:11Z. Nothing from round 1 was kept. A worker that flagged a conflicting instruction rather than guessing is the behaviour we want.
- `bench/tmuxh/bench-tmux watch RUN_DIR...` (new) opens one tmux window per worker following its formatted event stream (`stream.py --follow`); used for round 2.


## Verification results (lead, 2026-09-30)

All six workers ran GPT-5.5 (xhigh) inside the `agent` capsule; every verdict was CLEAN; in every case the changed files were exactly the three allowed.

| Ticket | Worker run | Result | What the lead re-ran beyond the worker's tests |
| --- | --- | --- | --- |
| 01 wt-core (F5) | mb-wt-core, 492 s | accepted | new, duplicate refusal, status, finish refusing unmerged and dirty, finish from inside the removed worktree, gc dry run then `--apply`; `status` on the real repo: 36 rows = `git worktree list` |
| 02 land (F6) | mb-land, 662 s | accepted | 8 scenarios in throwaway repos with a bare remote: failing gate, lost race (gate re-run, both commits land), overlap with a tracked change (exit 3), untracked file does not block, merge commit (exit 4), push-first proven by a pre-receive hook, no remote; `wt.py land` plug-in works; then landed this branch on the real repo with `check.sh` as the gate |
| 03 tracker (F7) | mb-tracker, 512 s | accepted | claim/resolve flow, blocked claim, six lint errors planted one at a time, lint and list over the real tickets |
| 04 msg (F8) | mb-msg, 379 s | accepted | ordering, NEEDS-ACK, ack sidecar leaves the message byte-identical, traversal refused (exit 64), lint exit code |
| 05 evidence (F9) | mb-evidence 562 s (rejected), mb-evidence-2 280 s (accepted) | round 1 rejected | round 1 hashed `git diff`, which ignores untracked files: on a real run the digest was sha256 of nothing and tampering with an untracked file still verified. Round 2 re-verified: digest non-empty; editing, adding and deleting an untracked file each fail `verify` naming the field; restore passes |
| 06 claims (F10) | mb-claims, 354 s | accepted | determinism and seed sensitivity on real reports; chained into the real `evidence.py` (a forged sidecar line is detected) |

Dispatch mistake, round 1 (logged above): six workers killed and relaunched after the `checks.d` name conflict.

Evidence records (F9 used on this run; stored in `.corenius/evidence/`, gitignored; each `verify`s with exit 0 at record time):

| Run | Digest |
| --- | --- |
| `mb-wt-core-20260930T081120Z` | `sha256:fad1f0ca43cba52b16ed444c94c76d73150b735f47595e9836d0330cc6905b50` |
| `mb-land-20260930T081120Z` | `sha256:9400071f418509e166cdbe388a267d17803083a5962f2e417c70f3d46aaa0609` |
| `mb-tracker-20260930T081121Z` | `sha256:a1a536a085a0175cf2114ba8451f27c36a004a665a2b871d7dc2753bc4cfc32f` |
| `mb-msg-20260930T081121Z` | `sha256:ae83bfd8a7c3dd86864a0b090c47c16c2efeb78e03928b215e19da06dbf3c6c3` |
| `mb-claims-20260930T081121Z` | `sha256:a43d12259ead81300afb4d5f0b9680895de594b00df2ccb09a25c47e7a8e647c` |
| `mb-evidence-20260930T081121Z` | `sha256:e88a76e55f1efb96295b2b715489dbf1d97b5f2fd1555d32100719c786242c00` |
| `mb-evidence-2-20260930T082134Z` | `sha256:0ba8d77337a56081ec0557d4d353563c1d4542c3950cdd742f6abb8ec7741d5e` |

Claim sampling (F10 used on the lead's own notes, `milestone-b/verification-notes.md`): two of the first four sampled claims failed and were mine: one cited the lint path of `tracker.py` where the claim path was meant, one cited a file that does not exist in this tree. Both were corrected and the corrected claims re-recorded as passed; the failed outcomes stay in `.corenius/claims/`.

## Limits and things not done

- `wt.py gc` marks merged clean worktrees as removable but cannot tell that one is in use: on the real repo it listed `effort/multi-agent-bench`, which a running experiment was using. `--apply` was not run on the real repo. A follow-up should skip worktrees with a live process.
- Tests and acceptance for F5 to F10 ran on Linux with this machine's git; pushing paths of `land.py` were tested only against a local bare remote, never a real remote. Nothing was pushed.
- Landing on main happens by fast-forward through `land.py` with `check.sh`; `CHECK_CARGO` stayed off because the main checkout still carried uncommitted AIME work (spec Decision 2 open). Later the same day the owner had that work committed (`c1f458b`) and cargo joined the default gate; a test committed in between (`tests/interception.rs`) needed the new `profile` field added.
- Tier 2 (F11 to F17) is not started. F12 still waits on the provider-sandbox decision.
- Worker output is machine-written code I read selectively; the acceptance scenarios are what was verified, not every line.
