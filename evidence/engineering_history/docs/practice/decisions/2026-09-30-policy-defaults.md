# Decision: fleet policy defaults adopted, and four owner decisions

Date: 2026-09-30. Decided by the owner ("follow your decision") on the lead's
recommendations. Sources: `docs/practice/guideline-map.md` section 5 (the seven
decisions needed before the guideline documents are written) and the open
items in `docs/efforts/agentic-engineering/STATE.md`.

These are decisions, not yet policy text. Writing them into `docs/agents/`
is a policy change: it bumps the version header and the manifest version
together and goes through `tools/test_sync_agentic_workflow.py`.

## Guideline-map decisions (section 5)

1. **Repo profiles: adopted.** Initial set, from the kinds the map names:
   `harness` (GPU-scarce build harness: milano, torchetta), `product`
   (product monorepo: arnold), `docs` (docs and observatory repos), `vendored`
   (an upstream mirror: megatronlm). Each repo's profile is a field in the
   manifest; a repo that fits none is assigned the closest one with a recorded
   exception. Assigning every repo is follow-up work in the manifest.
2. **Merge authority is a per-repo manifest setting**, `human` or
   `agent-after-gates`, never implicit. Profile defaults: `harness` is
   `agent-after-gates` (milano's practice, gates on the exact tip); the others
   are `human`. Corenius is `human` until it has a remote (see below).
3. **The tracker floor is the markdown ticket file** in the F7 format
   (`tracker.py`): fixed status set `open`, `claimed`, `resolved`, a claim line
   with expiry, a comments log. Any richer system (Kata, GitLab issues, a
   ledger) is a projection, labelled "projection of `<path>`", and never the
   only copy.
4. **Spike, scout, prototype, production** are defined as in map section 2.6.
   A spike owes a report with the question, verdict, evidence, unverified
   assumptions and a recommendation (`docs/spikes/collab/interception.md` is
   the template). Promotion means re-implementing under the gated path; spike
   code is not merged by default.
5. **Message delivery: append-only message files with a fixed header** (F8,
   `msg.py`) plus delivery at turn start by the runtime. The event-ledger
   alternative stays documented in the design-space note and is not adopted.
6. **Vocabulary:** the observatory's four properties, hermetic, isolated,
   offline and reproducible, become fleet vocabulary with its "not" columns.
7. **One repair budget: 3 fix rounds**, then stop and escalate to the lead or
   the human for review. Three rather than five because it bounds spend and
   matches what this effort practised (worker re-dispatch was bounded to two
   rounds). A repo profile may raise it with a recorded reason.

## Owner decisions from the effort state

8. **F12 (provider sandbox) is Tier 3, not a Milestone B requirement**, but
   the spike ran the same day. Finding: the `agent` capsule already runs
   `traecli` inside bubblewrap with network, proxy, the binary and two config
   files (`~/.trae/traecli.toml`, `~/.trae/cli/auth.json`), so "what traecli
   needs" is known. With `--ro-workspace` and one `--rw` bind per allowed file,
   a write outside the allowed paths fails with "Read-only file system" at the
   kernel (dry probe, no model). New allowed files must exist before launch so
   they can be bound. A live GPT-5.5 probe in the same setup wrote its allowed
   file and reported "Failed to write file" for `/workspace/probe.txt` and
   `README.md`. F12 was then built as `worker.py launch --sandbox insula`.
9. **Corenius push authority:** corenius has no git remote, and no repository
   named `philip.yang/corenius` exists on `code.byted.org` (checked with
   `git ls-remote`, "not found"). Creating one is the owner's act. Until then
   agents land with `land.py --no-push`, and the first real push is the
   owner's, so that `land.py`'s push path is exercised once by hand.
10. **Milano `claude/scm-retry-after-lost-race`:** integrate through milano's
    own `scm integrate` with the `repo` gate, and push. Done the same day:
    the first attempt failed because the Insula rootfs was missing from the
    cache (the launcher exits 1 without a message); after `rootfs-build`, the
    gate passed (143 tests) and `origin/diziet/mainline` is `f8c0120`. The
    worktree and branch were removed with `scm finish`.
11. **The `ultron-agentic-workflow` marker is renamed `corenius-agentic-workflow`**
    and the `ultron/mainline` pointer becomes `corenius/mainline`. Migration is
    a sync-tool change: recognise both markers for one policy version, rewrite
    on `--apply`, then drop the old name. Not executed yet.

## What this leaves open

- The Tier 1 boundary (F1 to F10 as the Milestone B minimum) is settled by
  having shipped it; no further decision.
- Milano's `AGENTS.md` sentence that the candidate is extracted read-only was
  checked before the landing and is accurate: `milano/isolation.py` requires
  the candidate to be `--ro-bind` mounted. An earlier note calling it wrong
  was itself wrong.
