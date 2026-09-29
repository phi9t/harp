# Agentic policy copies and cross-repo efforts inventory

Snapshot 2026-09-29, read-only scan of /data02/home/philip.yang/workspace.
Canonical = corenius-collab-spike (`CONSTITUTION.md`, `docs/agents/*`). Statements marked (inference) are judgment, not observed fact.

## 1. Canonical copies

| File | Version header |
|---|---|
| CONSTITUTION.md | Clanker Constitution v2026.08.11 (kenn-io) |
| agentic-engineering.md | v2026.09.29.1, source `corenius`, "managed by Corenius" |
| skill-orchestration.md | v2026.09.29.1, same |
| agentic-workflow-repos.json | 2026.09.29.1, source_repository `corenius`, mainline_branch `ultron/mainline`, adds a `corenius` repo entry (base main) |
| issue-tracker / domain / triage-labels | no header; local-tracker templates |

Only difference from ultron main (v2026.09.02.1): the header text, and "Ultron" -> "Corenius" in the cross-repo `.scratch/<effort>/` bullet. The skill-orchestration diff is header only. The repos JSON in ultron is 2026.09.07.1 (a version ahead of the 09.02.1 markdown). So the 09.29.1 canonical is content-equivalent to ultron 09.02.1 plus renaming.

Canonical lacks the following content that exists in some copies (see section 2):
- the "Ultron branch authority" section (08.15.2);
- Kata / active-work text (08.15.1);
- torchtitan's clean-context-subagent default;
- ds4's "Route recipes" and Matt-routing wording;
- ferric/ds4's evidence-bound-method and task-runs pointers.

## 2. Copies by repo (main checkout + worktrees)

Legend: CONST = CONSTITUTION.md (byte-identical to canonical in every ultron-managed repo and worktree). AE = agentic-engineering.md and SO = skill-orchestration.md. "Local trackers" = issue-tracker, domain, triage-labels. Every copy of AE and SO differs from canonical at least in its header (Ultron vs Corenius), so none is byte-identical. "Drift" below is measured against the ultron-era baselines (08.14.1 and 09.02.1).

| Repo (main checkout) | CONST | AE | SO | Nature of drift |
|---|---|---|---|---|
| cutlass | same | 09.02.1 | 08.14.1 | none vs ultron main (only stale vs canonical); local trackers identical to canonical templates |
| ds4 | same | 09.02.1 header, content forked | 08.14.1 header, content forked | AE adds evidence-bound-method/task-runs reading, "Matt routing phase" instead of `ask-matt`, ~70 lines of "Route recipes". SO wording edits (Matt-Superpowers, punctuation, "Matt routing phase"). Local trackers repo-specific. |
| ferric_continuum | same | 08.14.1 | 08.14.1 | AE has two extra pointers (evidence-bound-method.md, task-runs.md). SO clean. No issue-tracker.md (Kata-only); domain/triage-labels local. |
| megatronlm | same | 08.14.1 | 08.14.1 | none vs 08.14.1; lacks the 09.02.1 `ultron/mainline` paragraph |
| miles_diffusion | same | 08.14.1 | 08.14.1 | none vs 08.14.1 (its worktree publication-sanitized carries 09.02.1) |
| monarch | same | 08.14.1 | 08.14.1 | none vs 08.14.1 |
| nccl | same | 08.14.1 | 08.14.1 | none vs 08.14.1; local trackers identical to canonical templates |
| torchtitan | same | header 08.15.2, content forked | 08.14.1 header, content forked | AE adds clean-context-subagent default, `.scratch` tracker plus Kata-as-additional, "Ultron branch authority" section. SO makes subagents the default for implementation. Also `active-work.md`, `ultron-mainline.md`, and an AGENTS.md symlink to `.claude/CLAUDE.md`. |
| vaso | same | 08.14.1 | 08.14.1 | none vs 08.14.1 (worktrees publication-sanitized and readme-landing-page: 09.02.1). Adds `cross-agent-collaboration.md` + templates. |
| ultron | same | 09.02.1 | 08.14.1 | this is the pre-absorption source. Its docs/agents has NO active-work.md or ultron-mainline.md (those live only in worktrees and downstream repos). |
| lagrange-torch-mlu-insula | v2026.08.11 differing | none | none | only CONSTITUTION, drifted (not the kenn-io text hash); no docs/agents, no managed block |
| torchetta | v2026.08.11 differing | own unmanaged AE/SO/trackers, no version header | | not Ultron-managed; own workflow docs; no managed block. Several worktrees lack CONSTITUTION. |
| verl | v2026.08.11 differing | none | none | CONSTITUTION only; no ultron block |
| corenius (main checkout) | none | none | none | not yet carrying any policy files |
| corenius-collab-spike | canonical | canonical | canonical | source of truth, no managed AGENTS.md block |

Worktree drift (ultron-managed repos):

| Group | Version | Difference |
|---|---|---|
| ferric_continuum worktrees (13) | 08.14.1 | same as ferric main (extra method pointers); `active-work-tracker` 08.15.1; `ultron-mainline-policy` AE 08.15.2 |
| `*/active-work-tracker` worktrees (ferric, megatronlm, monarch, nccl, ultron) | AE and SO 08.15.1 | Kata replaces the tracker language ("work in Kata per `docs/agents/active-work.md`", legacy trackers read-only after cutover; cross-repo work via Ultron Kata parent and child issues). Adds active-work.md; drops or changes issue-tracker.md. |
| `*/ultron-mainline-policy` worktrees (ferric, megatronlm, monarch, nccl, ultron) | AE 08.15.2, SO 08.14.1 content | adds "Ultron branch authority" section (work starts from exact `ultron/mainline` tip, merge-base check, never work on it) and ultron-mainline.md |
| ultron `production-release-*`, `canonical-model-mastery-01`, `active-work-ultron-mainline`, `ultron-mainline`, `argus-mycroft-observability-design`, `ultron-mainline-contract-r2` | mixed 09.02.1 / 08.15.2 | no content divergence beyond version |
| ultron kernel-*/pinned-gpu-proof/direct-triton worktrees | 08.14.1 | none |
| torchtitan qwen3-formal-verifier(-final) | 08.15.2 | same forked content as torchtitan main |
| megatronlm rebase-*, monarch glm52-gpu-workload, nccl mycroft-gap-impl/reliability-loop/argus-* | 08.14.1 | none (nccl mycroft-gap-impl and reliability-loop have locally modified trackers) |

Findings:
- Version skew: the fleet spans 08.14.1 to 09.02.1, with 08.15.1 and 08.15.2 variants in branches. Nothing carries 09.29.1 except corenius-collab-spike. No AE or SO copy anywhere is byte-identical to canonical, so the first sync will touch every repo.
- Only ultron, cutlass, ds4, miles_diffusion/vaso (some worktrees) carry the 09.02.1 `ultron/mainline` paragraph. The other repos have the branch (see section 3) but not the policy text describing it.
- Forks that must be reconciled by hand, not overwritten: torchtitan, ds4, ferric_continuum.

## 3. Managed AGENTS.md block and branch pointers

Block markers are `<!-- ultron-agentic-workflow:start -->` ... `<!-- ultron-agentic-workflow:end -->` (with a mandatory-read instruction for CONSTITUTION.md and docs/agents/agentic-engineering.md). Present in the instruction file of every checkout and worktree of cutlass, ds4, ferric_continuum, megatronlm, miles_diffusion, monarch, nccl, torchtitan, vaso, ultron. Absent in lagrange-torch-mlu-insula, torchetta, verl, corenius, corenius-collab-spike. The marker name still says "ultron" and will need a rename or migration decision.

`git branch --list` results:

| Repo | `ultron/mainline` | Other relevant branches |
|---|---|---|
| cutlass, miles_diffusion, vaso, ferric_continuum, megatronlm, monarch, nccl, ultron | present | |
| torchtitan | present, and it is the checked-out branch (ADR 0006 exception: default branch on GitHub is `ultron/mainline`) | |
| ds4 | absent | |
| lagrange-torch-mlu-insula, torchetta, verl, corenius, corenius-collab-spike | absent | torchetta uses `diziet/mainline`, `land/diziet-mainline` |
| ferric_continuum, megatronlm, monarch, nccl, ultron | (as above) | `codex/ultron-mainline-policy` (plus `codex/active-work-ultron-mainline` in nccl and ultron; `codex/ultron-mainline-contract`, `-contract-r2`, `-design` in ultron) |

`ultron/mainline` is also checked out as a worktree named `ultron-mainline` in ultron, ferric, monarch, nccl, megatronlm.

## 4. Repo-specific docs under docs/agents/ (not managed policy)

| File | What it encodes | Disposition |
|---|---|---|
| cutlass/handoff-cutlass-gemm.md (209 lines, 2026-08-15) | Session handoff. CUTLASS fp32/bf16 GEMM, cuBLASLt backends, attention forward/backward, single-rank NCCL wired. B200 smoke and 16/16 gtests pass. Next scaffold frontier. | leave in cutlass (task history) |
| cutlass/infra-research.md (322 lines) | Research on bwrap rootfs, Bazel, hermetic CUDA, local-run scripts, drawn from monarch and nccl patterns. Input to a grill-with-docs session. | leave; could be referenced from a corenius infra pattern doc (inference) |
| ferric_continuum and ds4 evidence-bound-method.md (425 lines) and task-runs.md (471 lines) | Kata-backed Engineering Task / Task Contract / Task Run / Agent Attempt model and evidence-bound loop. The two repos' copies differ. | candidate to absorb into corenius policy (see section 7) |
| torchtitan active-work.md (08.15.1, "managed by Ultron") | Kata control plane: claim via `kata next`, `KATA_REF`, `work.attention` states. Also present in the active-work-tracker worktrees. | absorb or reconcile: this is policy with an ultron-managed header, not repo-specific |
| torchtitan and `*/ultron-mainline-policy` ultron-mainline.md | Dual mainlines, `ultron/mainline` never rebased or force-updated, bootstrap commit, TorchTitan exception. | absorb: mainline pointer convention now belongs to corenius |
| vaso/cross-agent-collaboration.md (200 lines, 2026-09-29) + templates lead-state.md, follower-handoff.md | Lead/follower protocol for two agents in one repo via `scripts/agents/relay.py`. Overlaps the corenius collab design space. | reference or absorb |
| roborev/docs/agents/index.md | Product docs for roborev supported agents. Unrelated to policy. | leave |

## 5. Cross-repo efforts and trackers

| Item | What it is | Status | Recommendation |
|---|---|---|---|
| `active-work-tracker` worktrees (ferric, megatronlm, monarch, nccl, ultron; branch `codex/active-work-tracker`) | Kata-as-system-of-record rollout (08.15.1 policy + active-work.md + `tools/active_work.py`). ultron's copy has 12 dirty files, last commit 2026-08-15 "Stabilize agentic workflow synchronization". | 6 weeks stale, unmerged (inference: superseded by the 09.29 direction), with uncommitted edits | reference; mine for Kata/active-work design (overlaps corenius `kata-coord-ledger`); do not delete without owner review |
| `ultron-mainline` worktrees | The `ultron/mainline` branch checkout. | current | leave; it is the branch anchor |
| `ultron-mainline-policy` worktrees (`codex/ultron-mainline-policy`) | AE 08.15.2 and ultron-mainline.md rollout. | unmerged, since 2026-08-15/23 | absorb the branch-authority text, then treat branches as stale |
| ultron `ultron-mainline-contract`, `-contract-r2`, `-design`, `active-work-ultron-mainline` | Design and contract branches for the mainline convention. Dirty (7, 8, 1, 0 files). | stale since 2026-08-15 to 09-07 | report only; likely abandoned |
| ferric_continuum/.scratch/evidence-bound-agentic-engineering | Spec plus 10 issues, all `ready-for-agent`: contract validator, run journal, task admission, dirty-worktree recovery, evidence records and finish gate, wrappers, supervision packets, estate ledger, closure receipts. Plus pending-workstreams.org and 1 receipt (deepseek-v41-reconciliation). | untouched implementation queue (last commit 2026-09-21 on unrelated work) | absorb into corenius: it is the strongest candidate for the evidence-bound method and Task Contract tooling |
| ultron/.scratch/workstream-worklog | Observability/tracing cohort (ndtimeline, py-spy, torchprofile, DCGM), 12+ issues; spec status `initial-phase-complete`. | active 2026-09-29 | leave in ultron (distributed-training robustness scope) |
| ultron/.scratch/model-mlsys-closed-loop | Model-MLSys evidence contract: lineage, SGLang gate, scorecard, agentic leverage. `ready-for-human`. | active 2026-09-29 | leave in ultron; reference the "agentic leverage" issue |
| ultron/.scratch/production-release-readiness (only estate-ledger.json + tools remain; 8 issues gone) | Fleet release-readiness effort incl. "06-policy-and-synchronization". Ultron commits on 2026-09-29 removed "synthetic lab, kernel loop, release readiness, ollama harness" and refocused Ultron on robust training. Monarch has its own production-release-readiness/ (approved, gated, sanitizing `ultron/mainline` candidate, B200 qualification). | ultron copy gutted; monarch copy live | reference; the policy-sync issue content survives only in git history and the worktrees `production-release-policy-sync` / `-estate` (detached, dirty) |
| ultron/.scratch other (canonical-model-mastery, resilient-titan, marin-complete-training-recipe, persistent-artifact-storage, manage-cutlass, ultron-build, ultron-incident-replay, ultron-thesis, tarocco-knowledge-sync, ollama-rootfs) | Model, training and storage efforts. Not about agentic policy. | mostly dormant since Aug to mid-Sep | leave |
| monarch/.scratch (glm52-*, glm53-flash, ginkgo-insula, monarch-unified-control-plane, production-release-readiness) | Model-serving, control-plane and release efforts. | mixed | leave |
| monarch/docs/agentic-ai (glm52-local-serving.md, glm53-flash-deep-dive.md) | Despite the name, these deploy GLM locally for Codex-style agent traffic, not workflow policy. | reference material | leave (inference) |
| kube/investigations (33 dirs, `aj-66f9dc22...` Arnold-job traces, 2026-07-03, tarballs) | Incident investigation evidence on a specific job (failover, NCCL probes). Not agentic policy. | frozen; kube last commit 2026-07-03 | leave |
| corenius-collab-spike/.scratch/kata-coord-ledger | Spike on kata as `coord` ledger backend (needs-triage, 8 issues). Moved from vaso 2026-09-29. | needs human authorization | this is the corenius-side home for the active-work/Kata thread |

## 6. Stale or orphaned worktrees and branches (report only)

- Prunable registrations whose paths are under /tmp or ~/.cache: nccl `/tmp/nccl-codex-research-stage1`; vaso six `/tmp/vaso-*-verify/capture` entries; torchtitan and miles_diffusion torchetta `kairo/external-worktrees/*-clean` and `/tmp/torchetta-kairo-clean-sources-*` entries.
- Ultron worktrees at the same commit 424ba7c (2026-08-22, one dirty file each): `kernel-cpu-evidence-loop`, `kernel-evaluator-contract`, `kernel-evaluator-isolation`, `pinned-gpu-proof`, `direct-triton-softmax-lane`. The kernel effort was removed from ultron on 2026-09-29, so these look abandoned (inference).
- Ultron `production-release-estate` and `production-release-policy-sync`: detached HEAD, dirty, effort removed from main.
- Ultron `active-work-tracker` (12 dirty files), `ultron-mainline-contract(-r2)`, `-design`, `ultron-mainline-policy`: 2026-08-15 to 08-23, dirty.
- ferric_continuum: 4 `checkpoint/*-20260921-1835` worktrees (fix-pr-67/72/75, incorporate-tnsr-13/14, land-dependabot-stack) are automatic checkpoints; likely landed or abandoned PR-fix work. `incorporate-tnsr-11/12`, `ci-fix-dependabot-scrub` likewise.
- nccl `mycroft-gap-impl`, `reliability-loop`, `argus-mycroft-nccl-design`; monarch `glm52-gpu-workload`; megatronlm `rebase-core-v0.18.2` (same commit as `ultron/mainline`) and `rebase-upstream-main`.
- torchetta: `kairo*` and `scout/*` worktrees (8), all carrying an unmanaged policy set with no CONSTITUTION in half.
- lagrange-torch-mlu-insula: main checkout is on `preserve/main-wip-before-landing-20260923`; three build worktrees share commit 1dd01ee with it.
- Duplicate ultron-era branch names `codex/*ultron-mainline*` exist in 5 repos; none is `ultron/mainline`.

## 7. What Corenius should absorb or reconcile (prioritized)

1. Choose the sync tool and ownership: `ultron/tools/sync_agentic_workflow.py` (+ test) and `agentic-workflow-repos.json` are what actually distribute the policy. Corenius has copies (`tools/sync_agentic_workflow.py`, repos JSON at 2026.09.29.1). Decide the managed-block marker name (`ultron-agentic-workflow:*`), and whether the sync tool still creates `ultron/mainline`.
2. Roll out 09.29.1 to all repos with a hand-merge for the three forks (torchtitan, ds4, ferric) to avoid losing local content. Every existing copy is stale, so nothing is a clean overwrite.
3. Decide what to fold in from branches: the "Ultron branch authority" section and `ultron-mainline.md` (08.15.2), and the Kata/`active-work.md` changes (08.15.1). Canonical has neither, but 5 repos have branches and torchtitan main already carries them.
4. Absorb the evidence-bound method: `docs/agents/evidence-bound-method.md`, `task-runs.md` (ferric vs ds4 copies differ; pick a base), and the ferric_continuum evidence-bound-agentic-engineering spec (10 ready-for-agent issues).
5. Reconcile vaso `cross-agent-collaboration.md` + `scripts/agents/relay.py` with `kata-coord-ledger` and the collab design space (both are agent-to-agent coordination).
6. Reconcile the ds4 "Route recipes" and the torchtitan "clean-context subagent by default" wording, both improvements the canonical lacks.
7. Fix the non-managed repos: give lagrange-torch-mlu-insula, torchetta, verl, corenius the managed block and current policy, or record an exemption in the repos JSON. Their CONSTITUTION copies (v2026.08.11) do not match canonical bytes.
8. Housekeeping (owner decision, not for corenius to act on): the stale worktree list in section 6 and `ultron/mainline` missing from ds4.
