# 01 workload-contract: Agree on real Qwen3 training and RL acceptance contracts

Type: task
Status: resolved
Owner: codex
Blocked by:
Claimed:

## Acceptance
- [x] Inspect existing TorchTitan and verl paths and retain source-bounded receipts.
- [x] Resolve continued-pretraining versus SFT and GRPO versus full PPO with the user.
- [x] Write a concrete proposed design with runtime gates, evidence and bounded runs.
- [x] Obtain independent design critique and reconcile findings.
- [x] Obtain human approval of model/topology/run limits and implementation design.

## Scope

Discovery and design only. Implementation stays in owning repositories after
approval. See [effort](../EFFORT.md) and [design](../DESIGN.md).

## Comments
- 2026-10-06T23:31:11Z codex: claimed until 2026-10-07T23:31:11Z

- 2026-10-06T23:34:42Z codex: CPT and GRPO selected by user; independent design review findings resolved. Human approval of DESIGN.md remains pending. No implementation or GPU execution.

- 2026-10-06T23:51:18Z codex: User approved design and live runs. Implementation authorized within the approved GPU limits; no commit/push/merge authorization inferred.
- 2026-10-06T23:51:18Z codex: resolved
