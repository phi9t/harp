# Agentic Engineering Roadmap

Corenius owns agentic-engineering practice for the workspace: the policy, the
mechanisms agents collaborate through, and the evidence that work is done.
This plan sequences that work into phases. Each phase ends at a milestone with
exit criteria that a command or a document review can check.

Status as of 2026-09-29. Inputs: `docs/practice/README.md`,
`docs/spikes/collab/design-space.md`, `experiments/collab/RESULTS.md`,
`docs/reference/`, `docs/reference/agentic-inventory/`.

## Re-cut (2026-09-29, owner direction)

Two milestones now lead the plan. **Milestone A** is a concrete set of
agentic-engineering features: `docs/superpowers/specs/2026-09-29-agentic-engineering-features.md`
(F1-F23, each with an interface and acceptance checks). **Milestone B**
implements and verifies them in this repository, in the order given there,
behind one gate command. Phases 3 to 6 below are the longer horizon these
features serve: F5-F6 and F11-F15 carry phase 3 and 4's `coord` core,
F7-F10 carry the tracker and evidence work of phases 4 and 6.

## Baseline (done)

| Item | State | Evidence |
| --- | --- | --- |
| Canonical policy lives in corenius (2026.09.29.2) | done | `docs/agents/`, `CONSTITUTION.md`, `tools/sync_agentic_workflow.py` (21 tests) |
| Fleet synced to canonical | done except 4 repos | landed in ultron, nccl, miles_diffusion, cutlass, megatronlm (checked out) and in the `main`/`master` branch of monarch, ferric_continuum, vaso |
| Forks reconciled | done | torchtitan's subagent rules folded in; its repo-local rules kept as addenda on branch `corenius/policy-2026.09.29.2` |
| Collaboration design space and simulator | done | 89 tests, 980-run sweep reproduces |
| Reference and inventory of sibling repos | done, mostly unverified | `docs/reference/` |

Remaining drift, by cause: torchtitan (not landed on purpose: its rules forbid
agents advancing `ultron/mainline`, and its checkout has 9 dirty files);
monarch, ferric_continuum and vaso (landed on `main`/`master`, but their
checked-out feature branches predate it).

## Principles

- Each milestone is verifiable by command or by reading a named artifact.
- Nothing lands in a sibling repo except by fast-forward from a worktree, with
  the drift check run first. A repo's own rules win over the fleet's.
- Claims cite a commit and mark inference; unverified reference material is
  labelled as such until a milestone verifies it.
- Do the experiment before building on the assumption (phase 2 gates phase 3).

## Phase 1: Finish consolidation

Goal: one policy, no stale copies, corenius in order.

Work:
1. Land or park the four remaining repos: get torchtitan's owner to take
   `corenius/policy-2026.09.29.2`; rebase the monarch, ferric_continuum and
   vaso feature branches when their owners next touch them.
2. Commit the corenius AIME/isolation work that sits uncommitted in the main
   checkout, and make manifests record the commit that actually ran.
3. Remove Ultron's own copies once it consumes the sync; decide the
   `ultron-agentic-workflow` marker and `ultron/mainline` rename and, if yes,
   add a migration to the sync tool.
4. Give the sync tool a `local_tracker_files` mode that installs once and does
   not enforce content, so repo-specific tracker text (torchtitan's Kata
   paragraph) survives.

**Milestone M1: fleet clean.**
`python3 tools/sync_agentic_workflow.py --check --workspace-root <workspace>`
prints `CHECK OK`; `git status` in the corenius main checkout is clean;
manifests of the isolated-check runs name the commit they ran.

## Phase 2: Settle the rules

Goal: the rules agents follow agree, and the disagreements are decided by a
human, not left to whichever repo an agent happens to be in.

Work:
1. Take the 20 conflicts (C1-C20) in
   `docs/reference/agentic-inventory/guidance-and-workflows.md` to decisions:
   who merges, force-push, worktrees, commit staging, trailers, generated
   files, fix-round cap (3 vs 5).
2. Make merge authority a per-repo setting (human-merge, agent-after-gates)
   in the manifest rather than prose.
3. Write the decisions into the canonical policy as version 2026.10.x with a
   changelog, then sync.
4. Add enforcement where policy is only prose today: hooks for the rules that
   can be checked mechanically (only one hook exists in the fleet).

**Milestone M2: one rule set.**
A conflicts ledger with every C-item marked decided (with the decision and
owner); policy v2 synced; at least the git rules enforced by a shared hook that
fails a test when violated.

## Phase 3: Ground truth for collaboration

Goal: replace assumptions with measurements before building the runtime.

Work:
1. **Interception experiment.** Can corenius observe and gate an ACP
   provider's native file writes? Test with the real TraeCLI and Codex
   providers: which writes are visible, which bypass corenius's tools.
2. **Calibrate the simulator** with real gate durations and conflict rates
   from milano's evidence and git history; rerun the sweep and record how the
   ranking moves.
3. **Verify the reference documents.** Check every claim marked read against
   its cited line for the five sibling repos plus the observatory; fix or
   strike what fails.
4. Fix the defects found on the way: milano `integrate` retry after a lost
   push race (with a test), the fair-GPU-allocation starvation and stale
   `.holder` files in `gpu_lease.sh`.

**Milestone M3: go/no-go.**
A written result for the interception experiment with a decision: claims and
leases are *enforced* or *advisory*; a calibrated `RESULTS.md`; the reference
docs carry a verification date and a list of struck claims; the milano retry
bug has a failing-then-passing test.

## Phase 4: Coordination core

Goal: the `coord` runtime from the design space, dogfooded on corenius.

Work:
1. Event-sourced ledger (append-only, content-addressed) with a replay test
   that rebuilds all state.
2. Claims and leases on paths, with conflict prediction; enforced or advisory
   per the M3 decision.
3. Landing queue that batches into `staging` with the human promoting to
   mainline; gates run on the exact tip.
4. GPU broker with fair scheduling in front of the lease locks.
5. Safety invariants from the simulator become property tests: no ungated
   landing, no lost work, mainline never red.

**Milestone M4: two agents land through `coord` in corenius.**
Two agents make concurrent changes; both land with the ledger showing claim,
gate and landing events; replaying the ledger reproduces the repository head;
invariant tests pass; an injected semantic conflict is caught by the gate.

## Phase 5: Adopt across repos

Goal: repos use the shared mechanism instead of their own.

Work:
1. A thin adapter so milano's `integrate` and GPU leases run on `coord`;
   milano keeps its gates.
2. Interoperate with vaso's lead/follower relay for the long-running
   environment case, or replace it if the results support that.
3. Shared skills directory: frontmatter schema and validator, then merge the
   duplicate review, merge and CI-repair flows (see
   `agentic-inventory/skills-and-commands.md`).
4. Observatory and other repos adopt through the adapter, one at a time.

**Milestone M5: milano on `coord`.**
Milano integrates and leases GPUs through `coord` for a full qualification
(`package` then `b200-release`); the shared skills validator runs in CI for at
least three repos; measured throughput and latency are compared with the
simulator's prediction and the gap is written down.

## Phase 6: Evidence and trust

Goal: close the gaps no repo covers.

Work:
1. Tamper-evident ledger and signed receipts, so evidence names what ran
   (the milano NCCL receipt error is the motivating case).
2. Cost and token gates for agent runs.
3. Eval variance statistics for the AIME harness.
4. A periodic freshness check that flags stale references and drifted policy.

**Milestone M6: evidence is checkable.**
A receipt verifies offline against its signature and names the exact commit,
rootfs and bundle digests; a failed gate leaves a receipt; the freshness check
runs on a schedule and reports drift.

## Sequencing

```
M1 fleet clean ──► M2 one rule set ─────────────┐
      │                                          ▼
      └──────────► M3 go/no-go ──► M4 coord ──► M5 adopt ──► M6 trust
```

M1 unblocks everything by removing noise. M2 and M3 run in parallel. M4 needs
M3's decision. M5 needs M2's merge-authority setting and M4. M6 can start its
ledger work after M4.

## Decisions needed from the owner

1. Marker and `ultron/mainline` rename (M1).
2. Merge authority per repo, and whether an agent may integrate to mainline
   after gates (M2).
3. Enforced versus advisory claims, if M3's experiment cannot decide it.
4. Which repo is the first adopter of `coord` after milano (M5).

## Risks

- **Interception may be impossible.** Then claims are advisory and the design
  leans on gates and the landing queue; M4 shrinks but does not fail.
- **The simulator is a model.** Its parameters were invented; M3 recalibrates.
- **Reference documents are agent-written.** Treat them as leads until M3 step
  3 verifies them.
- **Touching other repos.** Each landing moves someone's branch; every repo's
  own rules stay authoritative, and dirty or actively edited checkouts are left
  alone.
