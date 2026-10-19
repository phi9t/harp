---
id: harness-benchmark-field-guide
title: Harness benchmark field guide
type: benchmark-guide
status: active
tags: [agentic-engineering, evaluation, benchmarks]
confidence: medium
---

# Harness benchmark field guide

A harness evaluation must identify the exact agent candidate, task distribution,
allowed tools, evaluator, resource budget and retained execution evidence.
Separate correctness from quality, cost, latency and the authority needed to
perform a task. Historical research reading is condensed in
[[lab/article_survey|the article survey]].

## Evaluation contract

- Pin task and candidate revisions before execution.
- Keep held-out tasks, expected results and scoring policy outside candidate writes.
- Record provider/model identity, commands, permissions, timeouts and resource limits.
- Preserve unsuccessful attempts and interruptions alongside successful runs.
- Distinguish evaluator output from an independently reproduced result.
- Report task-level outcomes and costs; a single score does not explain failures.

## Runtime behavior

Use deterministic fixtures to test scheduling, cancellation, replay, artifact
boundaries and structured result validation before measuring model-dependent
performance. A restart test must exercise persisted state and recovery, rather
than rerunning the task from scratch.

[[lab/benchmarks/harness_benchmarks/dynamic_workflow_need_and_eval|Dynamic workflow evaluation]]
defines the retained runtime cases. The
[[lab/benchmarks/evaluator_integrity/evaluator_integrity_benchmark_suite|evaluator-integrity guide]]
separates benchmark-selection and evaluation-policy questions.

## Evidence limit

This guide states evaluation requirements. It claims no benchmark execution,
score reproduction, or qualification of the complete live agent system.
