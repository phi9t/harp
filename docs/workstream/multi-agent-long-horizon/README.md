# Multi-Agent Collaboration for Long-Horizon Model Training & Evaluation

[Documentation](../../README.md) / Workstream / Multi-agent long-horizon

Status: active research tracking, 2026-10-09.  
Evidence survey: [survey.md](survey.md).  
Tracker: Harp-managed workstream ledger.

---

## 1. Objective

Build and track a consistent, long-horizon research program for autonomous multi-agent engineering teams. The core target workloads are **model training and evaluation** across pretraining, post-training RL, recommendation systems, and perception models.

The primary measure of autonomy across multi-day campaigns is **owner and supervisor interventions per landed milestone / verified run**, adopted from the Corenius readiness framework. Wall-clock run time, GPU utilization, model spend, and checkpoint stability are tracked continuously beside it.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Harp Research & Tracking Ledger (docs/workstream/multi-agent-long-horizon)│
│ Evidence synthesis • Intervention tracking • Research questions • Gates │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
    ┌───────────────────────────────┴───────────────────────────────┐
    ▼                                                               ▼
┌──────────────────────────────────────┐ ┌──────────────────────────────────────┐
│ L0: Insula Hermetic Environment      │ │ L1: Remote SSH & Daemon Lifecycle    │
│ • Byte-identical rootfs builds       │ │ • Binary-framed SSH stdio transport   │
│ • Pinned CUDA/PyTorch & dependencies │ │ • Zero shell-quoting bugs            │
│ • Declared GPU, dataset & ckpt mounts│ │ • Process-group supervision & kills  │
│ • "Launch everywhere" portability    │ │ • Disconnect survival & recovery     │
└──────────────────────────────────────┘ └──────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ L2: Multi-Agent Coordination & Gated Landing                           │
│ Lead-worker orchestration • Isolated worktrees • Atomic admission      │
│ Independent candidate verification • Stale-base rebase gates           │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
    ┌───────────────────────────────┴───────────────────────────────┐
    ▼                                                               ▼
┌──────────────────────────────────────┐ ┌──────────────────────────────────────┐
│ Anchor 1: Pretraining & CPT          │ │ Anchor 2: Post-Training RL           │
│ TorchTitan (FSDP, TP, PP, Qwen3-CPT) │ │ VeRL (PPO, GRPO, HybridFlow, Rollouts)│
└──────────────────────────────────────┘ └──────────────────────────────────────┘
┌──────────────────────────────────────┐
│ Anchor 3: Perception & Multimodal    │
│ Sureal (Waymo perception, Layouts)   │
└──────────────────────────────────────┘
```

---

## 2. Core Workloads: Model Training and Evaluation

Long-horizon multi-agent collaboration is evaluated against real model development rather than toy code edits. Three specialized repositories on remote B200 cluster host anchor the research:

### 1. TorchTitan (`torchtitan`): Pretraining & Continued Pre-Training (CPT)
* **Domain:** 3D distributed parallelism (FSDP2, Tensor Parallelism, Pipeline Parallelism) for LLM pretraining and domain continuation.
* **Active Branches:** `ultron/mainline`, `codex/qwen3-cpt-insula` (Qwen3 CPT inside Insula).
* **Long-Horizon Tasks:** Multi-epoch training runs, learning rate schedule execution, loss convergence monitoring, gradient norm tracking, loss spike diagnostics, and distributed checkpoint resharding.

### 2. VeRL (`verl`): Post-Training Reinforcement Learning
* **Domain:** High-throughput RL for LLMs (PPO, GRPO, REINFORCE++, RLVR) using HybridFlow engine abstractions.
* **Active Branches:** `main`, `codex/eb6b-qwen3-grpo` (single-file PPO & Qwen3 GRPO recipes).
* **Long-Horizon Tasks:** Generation rollouts, actor-critic weight synchronization, reward modeling, rule-based reasoning evaluation (e.g., AIME/math benchmarks), reward collapse/hacking detection, and reference policy clipping.

### 3. Sureal (`sureal`): Perception & Multimodal Models
* **Domain:** Vision, autonomous driving perception (Waymo dataset suites), and spatial-temporal representations.
* **Active Branches:** `phi9t/mainline`, `work/semantic-layout/*` (evaluation pipeline launch plans and semantic layout refactors).
* **Long-Horizon Tasks:** 3D bounding box / trajectory evaluation, multi-sensor calibration, dataset distillation, feature extraction sweeps, and regression test suites.

---

## 3. Foundational Infrastructure Layers

Reliable multi-agent training and evaluation over days requires robust environmental and process guarantees:

### Layer 0: Insula as the Universal Reproducible Environment
* **Problem:** ML training fails unpredictably when host packages, CUDA drivers, Python virtualenvs, or filesystem paths differ across machines.
* **Insula Solution:**
  * Pinned, byte-identical rootfs builds (Ubuntu 24.04 + declared apt/Python/CUDA closures).
  * Bubblewrap (`bwrap`) runtime isolation with explicit projections: read-only rootfs, read-only dataset volumes, read-write checkpoint directories, and private tmpfs (`/run/corenius-tmp`).
  * **"Launch Everywhere":** The same Insula contract runs identically on a local dev box, the remote B200 cluster host, or remote cluster nodes without host state leakage.

### Layer 1: Remote Execution & Daemon Supervision
* **Problem:** Long-running training/eval runs triggered over SSH frequently fail due to shell escaping bugs, network disconnects killing parent shells, hung GPU processes, or zombie daemons consuming VRAM.
* **DevX / Corenius Solution:**
  * **Framed SSH Transport (`devx/prototypes/remote-tools`):** Length-prefixed binary stdio frames passing arbitrary `argv` and environment arrays cleanly without shell quoting or bash interpolation.
  * **Detached Daemon Continuity:** Training supervisors run as persistent background daemons, logging structured telemetry and surviving SSH disconnects.
  * **Strict Process-Group Teardown (`verify-ai-lsp` / `supervisor.rs`):** Monitored process groups (`PGID`). On cancel, timeout, or failure, signals escalate (`SIGTERM` $\to$ grace period $\to$ `SIGKILL`), ensuring no stranded PyTorch/CUDA processes retain GPU memory.
  * **Monotonic Deadlines:** Clock seams (`open_with_clock`) enforcing strict preflight (300s), cleanup (60s), and total run deadlines.

### Layer 2: Multi-Agent Coordination & Gated Landing
* **Problem:** When multiple agents write code, run experiments, or update model architectures concurrently, naive approaches suffer from the 15 Sureal failure modes (stale base collisions, dirty working trees, ungated merges).
* **Controller Solution:**
  * **Isolated Worktrees:** Each agent attempt receives a private Git worktree and private Insula sandbox.
  * **Candidate Commits & Independent Verifiers:** Workers produce candidate commits; independent verifier worktrees execute evaluation suites before any merge.
  * **Atomic Run Admission Policy (Ticket 07):** Transactional SQLite admission, future task bindings, and zero-consumption denial recording.

---

## 4. Extended Yardstick for Long-Horizon Model Work

In addition to the 47 Corenius readiness stories (covering hand-off, boundaries, and landing), this program tracks scenarios specific to multi-day model training and evaluation:

### Long-Horizon Training & Eval Scenarios (M)
* **M1. Unattended Checkpoint Resumption:** A training job that crashes or times out resumes cleanly from its last verified checkpoint without data duplication or manual script editing.
* **M2. Evaluation Regression Gate:** A worker modifying model code or hyperparameters cannot merge unless automated eval scores (loss, accuracy, AUC, benchmark pass-rate) meet or exceed baseline criteria.
* **M3. Telemetry & Loss Divergence Detection:** The coordinator detects NaN loss, gradient explosion, or stall without human intervention, terminates the run, and records an honest failure disposition.
* **M4. Multi-Agent Hyperparameter Ablation:** Two or more workers independently evaluate candidate configurations on separate GPU slices and report structured comparison tables.
* **M5. Long-Horizon State Preservation:** An experiment running across $\ge 24$ hours preserves full logs, training curves, git commits, and Insula contract hashes across lead context resets or SSH drops.

### Remote & Environmental Scenarios (E & R)
* **E1. Insula Training Contract:** Every model training or eval script runs inside an Insula capsule with verified rootfs and dependency digests.
* **E2. Host Non-Interference:** Concurrent runs on the same machine do not collide on ports, GPU memory, or `/tmp` paths.
* **R1. Framing Integrity:** Training commands with complex arguments (JSON configs, regexes, bash pipes) execute over framed SSH without string corruption.
* **R2. Zero GPU Leakage:** When an eval or training job terminates, `nvidia-smi` confirms 0 residual memory allocation for the assigned process group.

---

## 5. Research Questions & Hypotheses

| ID | Research Question | Hypothesis | Falsification Condition |
| :--- | :--- | :--- | :--- |
| **RQ1** | Can autonomous multi-agent teams reliably conduct model train/eval over 24h+ without intervention? | Teams using isolated worktrees, Insula capsules, and atomic admission achieve $\le 1$ intervention per landed model iteration. | $> 3$ human interventions required per milestone due to environment corruption or merge conflicts. |
| **RQ2** | Does Insula containerization improve model evaluation reproducibility? | Pinned Insula capsules eliminate cross-host eval discrepancy ($0\%$ variance in deterministic seeds). | Same model checkpoint yields different eval metrics across hosts despite identical Insula contract. |
| **RQ3** | What causes the most failures in long-horizon model campaigns? | Training divergence and environment drift cause more stalls than pure code syntax errors. | Failures remain dominated by basic git or syntax issues rather than training dynamics. |
| **RQ4** | Does structured remote daemon supervision eliminate GPU memory leaks? | PGID tracking with escalating kill signals guarantees $100\%$ cleanup of PyTorch/CUDA jobs. | Any orphan `python` or `torchrun` process remains after timeout/cancellation. |

---

## 6. Phased Execution Roadmap

* **Phase 1: Substrate Qualification (Deterministic)**
  * Qualify Insula rootfs for PyTorch/CUDA execution on remote B200 cluster host.
  * Validate framed SSH remote runner and process group cleanup against live GPU tasks.
  * Exit: E1–E2 and R1–R2 pass with scripted test fixtures.
* **Phase 2: Baseline Model Eval Campaign (Single Task)**
  * Execute autonomous evaluation runs across `torchtitan` (Qwen3 CPT validation) and `verl` (GRPO math benchmarks) inside Insula.
  * Record all interventions, telemetry, and checkpoint digests in Harp.
  * Exit: At least one end-to-end eval run completes unattended and publishes verified receipts.
* **Phase 3: Multi-Worker Concurrent Model Experimentation**
  * Deploy Lead-Worker coordination (Corenius I2 controller on the qualified substrate).
  * Run concurrent model experimentation (e.g. comparing two learning rate schedules or loss formulations in `sureal` or `torchtitan`).
  * Exit: Parallel candidates verified and landed through gated admission without manual git intervention.
* **Phase 4: 24h+ Autonomous Research Campaign**
  * Multi-day continuous run executing iterative model training, checkpoint evaluation, and hyperparameter refinement.
  * Exit: $\ge 24$ hours continuous operation, $\ge 10$ verified milestones, full intervention and loss telemetry logged in Harp.
