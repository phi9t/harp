# Evidence Survey: Remote Substrate & Model Workloads (2026-10-09)

[Program](README.md) / Evidence survey

Read-only survey of the remote GPU development host (`~/workspace`), the `devx` repository, and the model workstreams, taken 2026-10-09. Nothing was mutated or launched on the remote host during this survey.

---

## 1. Workload Repositories: Model Training & Evaluation

The primary workloads for long-horizon multi-agent collaboration are real model development efforts. The remote B200 host contains active repositories across the ML spectrum:

| Repository & Path | Domain & Model Tasks | Head Commit / Branch | Active Work & Status |
|---|---|---|---|
| **`torchtitan`**<br>`~/workspace/torchtitan` | Distributed Pretraining & CPT (FSDP2, TP, PP) | `843948e9f`<br>`ultron/mainline` | Branch `codex/qwen3-cpt-insula`: Qwen3 Continued Pre-Training within Insula rootfs; checkpoint management and PII scrub checks. |
| **`verl`**<br>`~/workspace/verl` | Post-Training Reinforcement Learning (PPO, GRPO, HybridFlow) | `3f395aef`<br>`main` | Branch `codex/eb6b-qwen3-grpo`: Single-file from-scratch PPO/GRPO for tiny language models; agentic engineering bootstrap tests. |
| **`sureal`**<br>`~/workspace/sureal` | Vision & Autonomous Perception (Waymo suite) | `93a373d`<br>`phi9t/mainline` | Branches `work/semantic-layout/*`: Evaluation pipeline launch helpers and evaluation facade refactoring. |

---

## 2. Infrastructure & Coordination Repositories

| Repository & Path | Domain & Responsibility | Head Commit / Branch | Current State & Evidence |
|---|---|---|---|
| **`corenius`**<br>`~/workspace/corenius` | Local Multi-Agent Controller & Coordination Engine | `b7bee20`<br>`main` | Dehydrated on 2026-10-07 following owner-directed shutdown. Holds the 47-story readiness spec, the 15-point Sureal failure analysis, and SQLite controller. |
| **`corenius-controller-bench-resume`** | Ticket 07 Repair2 Frozen Worktree | Base `b7bee20` | Complete, verified implementation of atomic run admission policy (`admission.rs`, `open_with_clock` clock seam, zero-capacity denials). 3 review rounds PASS. |
| **`insula`**<br>`~/workspace/insula` | Hermetic Rootfs & Bubblewrap Runtime | `7115176`<br>`main` | Reference slice complete (`scripts/demo all` passes with byte-identical builds). Adds persistent developer projections and declared apt/Python/CUDA closures. |
| **`devx`**<br>`~/workspace/devx` (remote) | Daemon Lifecycle & Process Supervision | `2724406`<br>`feature/doom-ai-lsp` | `test/verify-ai-lsp-launcher`: Unit and lifecycle test suites proving exact status retention, PGID termination, and temporary root cleanup. |
| **`devx`** (local) | Remote Transport Protocol | `6c52828`<br>`main` | `prototypes/remote-tools`: 5-byte framed binary wire protocol over SSH stdio; proved zero shell-quoting bugs and clean PGID kill on disconnect. |

---

## 3. The 15 Sureal Empirical Failure Modes & Mitigations

The Corenius team's investigation of the Sureal multi-agent run (`docs/practice/imported/sureal-semantic-layout.md`) cataloged 15 critical failure modes that emerge in naive multi-agent runs, and developed concrete mitigations:

| # | Failure Mode in Naive Multi-Agent Swarms | Concrete Mitigation in Corenius / Insula / DevX |
|---|---|---|
| **1** | **Done set before commit** (worker marked ticket done before files committed). | Candidate commit hashing; state transitions only on verified commit objects. |
| **2** | **Authority collision** (lead tried to commit worker's uncommitted tree). | Strict worktree isolation; zero lead write permissions in worker worktrees. |
| **3** | **Sleep-based sync** (coordinator used arbitrary `sleep 60` heuristics). | Explicit event receipts and lock files replacing timing guesses. |
| **4** | **Mid-run baseline** (state snapshot taken 7m33s into run, missing early writes). | Preflight frozen checksum manifest (`sha256sum`) taken before process admission. |
| **5** | **Shallow baseline** (checked only top-level directory mtimes). | Deep recursive tree hashes and lockfile validation. |
| **6** | **Instruction-only boundaries** (prompts asked agents not to touch paths, but gave full access). | **Insula Bubblewrap sandboxing**: Only designated worktrees and checkpoint dirs mounted read-write. |
| **7** | **No caps or stall detection** (runaway training/eval runs without timeouts). | Monotonic clock deadlines (`open_with_clock`), 300s preflight, 60s cleanup bounds. |
| **8** | **Zombie processes** (interactive TUIs lingering with `exec zsh`, appearing active). | Process-tree supervisors (`supervisor.rs`, `verify-ai-lsp`) tracking PGIDs and forcing exit. |
| **9** | **Progress lag** (tickets showed 0% progress for 30 minutes, then completed). | Incremental turn receipts and streaming evaluation telemetry. |
| **10** | **Ungated merges** (code merged without running tests or eval benchmarks). | Independent verifier worktree running full test suites before fast-forwarding. |
| **11** | **Human ticket ingestion** (agents claimed tasks reserved for humans). | Typed ticket schema state machines enforcing role permissions. |
| **12** | **Stale-base landing races** (two workers branching from same base collided on landing). | Ticket 07 atomic admission requiring genuine stale candidate proofs before re-integration. |
| **13** | **Shared worktree dirt** (multiple workers running in overlapping directories). | Ephemeral Git worktrees detached and pruned per attempt. |
| **14** | **Unstructured prose handoffs** (critical dependency details buried in chat comments). | Typed Kata issue items and validated JSON input briefs. |
| **15** | **Subjective reviews** (code accepted without independent verification). | Three-round independent review protocol (Branch, Spec, Standards). |

---

## 4. Harp's Role: The Central Tracking & Knowledge Authority

Under the contributor guidelines (`AGENTS.md`), Harp is a standalone RSI product. It does not introduce runtime dependencies on external codebases. Instead, Harp serves as:
1. **The Intent & Research Ledger:** Defining the research program, scenarios, and phased milestones.
2. **The Evaluator Authority:** Enforcing truthful, disentangled outcome metrics (Task vs. Orchestration vs. Effect) and preventing false autonomous passes.
3. **The Evidence Synthesis Archive:** Aggregating experiment logs, intervention ratios, and model receipts across `torchtitan`, `verl`, and `sureal`.
