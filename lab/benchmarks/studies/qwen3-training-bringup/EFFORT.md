# Real Qwen3 training workloads under Insula

Type: effort
Status: bounded live workload milestone complete; reviewed changes uncommitted
Owner: user with Codex coordination
Priority: establish real workloads before extending robust-training coverage

## Intent

Make the owning TorchTitan and verl repositories capable of running two real
Qwen3 workloads under Insula:

1. TorchTitan core pretraining-style or SFT-style training.
2. verl RL using the TorchTitan learner backend and SGLang rollout serving,
   following the TorchTitan-backend PPO-trainer example where applicable.

The purpose is to learn the workload mechanics from actual runs and use their
evidence to ground later Arnold robust-training work. A launcher that starts,
a synthetic-only loop, or an import test does not complete the effort.

## Route and authority

Requested route: ask-matt. Current phase: approved implementation following stateful design discovery using the
repository skill crosswalk, with grilling/domain-modeling and brainstorming as
support. Cross-repository distributed training takes the gated path. The accepted design in DESIGN.md is the implementation acceptance authority.

Corenius owns cross-repository coordination. TorchTitan and verl own their runnable
workloads and local tickets. Ultron retains the research synthesis. Trondheim is
prior art for Insula/sensors, not an implicit replacement for the requested repos.
Respect verl's Kata tracker rather than adding a competing Markdown issue system.

Read-only audits are authorized. Repository bring-up is the requested outcome;
workload choices and design were approved by the user, including live runs. No commit,
push, PR, merge, long training campaign, or GPU reservation is inferred.

## Accepted contract

- Pin code, model/tokenizer, real dataset, resolved configuration, rootfs and
  dependency bundles; record explicit GPU and network policy.
- TorchTitan: real next-token or supervised targets; finite loss and verified
  optimizer updates; checkpoint save plus a fresh-process bounded continuation.
- verl: real prompts and rewards; SGLang generation; TorchTitan policy update;
  updated weights consumed by the next rollout; checkpoint and supported resume
  behavior explicitly demonstrated or accurately bounded.
- Preserve run/attempt identity, commands, dataset/sample identity, losses,
  optimizer decisions, resource use and artifact locations.
- Provide reproducible Insula commands from each owning repo without ambient
  host-package fallback.
- Separate smoke/bring-up proof from convergence, model quality, performance,
  exact recovery, and reliability claims.

## Workload decisions

- Selected by user: continued pretraining from released Qwen3 weights on real text.
- Selected by user: GRPO through verl's PPO trainer, matching the linked example.
- Proposed model sizes, topology, run limits and resume boundary are concrete in
  [DESIGN.md](DESIGN.md); approved by user.

## Milestones

1. Agree on workload and evidence contracts after local source inspection.
2. Pin and qualify Insula dependency closures for both workloads.
3. Prove core TorchTitan training and continuation on real data.
4. Prove verl + TorchTitan + SGLang rollout/update/weight-sync cycle.
5. Record observed failures and controller-relevant evidence in Ultron.

All five milestones completed. See tickets/02-torchtitan-cpt.md and
tickets/03-verl-grpo.md for native evidence and claim limits. CPT completed
20 updates across a restart; GRPO completed2 initial +2 resumed updates with
verified policy transfer. Total GPU consumption was1526.53090452 GPU-seconds
(0.42403636 GPU-hours), including failed attempts. Both final reviews accepted
the bounded milestones. No commit, push, PR or merge performed.

## Source pointers

- https://verl.readthedocs.io/en/latest/workers/torchtitan_workers.html#ppo-example
- Ultron research: docs/research/arnold-robust-training-opportunities-2026-10-06.md
  in /data02/home/philip.yang/.codex/worktrees/3604/ultron.
- Readiness receipts are stored under evidence/ in this effort.
