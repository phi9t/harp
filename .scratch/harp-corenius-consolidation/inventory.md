# Corenius preservation inventory

Observed 2026-10-18 over SSH. Paths identify preserved remote work; they are
not runtime or documentation prerequisites for Harp. No worktree was modified.

Main revision: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`.

## Registered worktrees

### corenius

- Source path: `/data02/home/philip.yang/workspace/corenius`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/main`
- Disposition: preserve; content review required before migration or retirement.

```text
?? .trae/
```

### corenius-durable

- Source path: `/data02/home/philip.yang/workspace/corenius-durable`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/insula-durable`
- Disposition: preserve; content review required before migration or retirement.

```text
 M README.md
?? controller/durable/
?? docs/adr/0003-insula-durable-execution.md
?? docs/efforts/insula-durable/
?? tools/agents/checks.d/72-durable-execution.sh
?? tools/insula/durable.py
```

### corenius-durable-cgroup

- Source path: `/data02/home/philip.yang/workspace/corenius-durable-cgroup`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/insula-durable-cgroup`
- Disposition: preserve; content review required before migration or retirement.

```text
?? controller/durable/
```

### corenius-durable-runtime

- Source path: `/data02/home/philip.yang/workspace/corenius-durable-runtime`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/insula-durable-runtime`
- Disposition: preserve; content review required before migration or retirement.

```text
?? controller/durable/
```

### corenius-retro-fixes

- Source path: `/data02/home/philip.yang/workspace/corenius-retro-fixes`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/retro-fixes`
- Disposition: preserve; content review required before migration or retirement.

```text
 M AGENTS.md
 M README.md
 M bench/harness/lead_driven.py
 M bench/harness/test_lead_driven.py
 M docs/agents/agentic-engineering.md
 M docs/agents/agentic-workflow-repos.json
 M docs/agents/issue-tracker.md
 M tools/agents/check.sh
 M tools/agents/checks.d/05-gate-dispatch.sh
 M tools/agents/checks.d/70-sqlite-controller.sh
 M tools/agents/checks.d/README.md
 M tools/agents/effort.py
 M tools/agents/test_check.py
 M tools/agents/test_effort.py
 M tools/agents/test_tracker.py
 M tools/agents/tracker.py
 M tools/temp_root.py
 M tools/test_temp_root.py
?? .github/
?? controller/durable/
?? docs/adr/0003-insula-durable-execution.md
?? docs/efforts/insula-durable/
?? docs/efforts/retro-fixes/
?? tools/agents/VALIDATION.md
?? tools/agents/check.py
?? tools/agents/check_test.py
?? tools/agents/checks.d/27-review-snapshot.sh
?? tools/agents/checks.d/72-durable-execution.sh
?? tools/agents/checks.d/73-durable-qualification.sh
?? tools/agents/review_snapshot.md
?? tools/agents/review_snapshot.py
?? tools/agents/review_snapshot_test.py
?? tools/insula/durable.py
```

### corenius-retro-gate

- Source path: `/data02/home/philip.yang/workspace/corenius-retro-gate`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/retro-gate`
- Disposition: preserve; content review required before migration or retirement.

```text
 M bench/harness/lead_driven.py
 M bench/harness/test_lead_driven.py
 M tools/agents/check.sh
 M tools/agents/checks.d/05-gate-dispatch.sh
 M tools/agents/checks.d/README.md
 M tools/agents/test_check.py
 M tools/temp_root.py
 M tools/test_temp_root.py
?? tools/agents/check.py
?? tools/agents/check_test.py
```

### corenius-retro-handoff

- Source path: `/data02/home/philip.yang/workspace/corenius-retro-handoff`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/retro-handoff`
- Disposition: preserve; content review required before migration or retirement.

```text
 M tools/agents/effort.py
 M tools/agents/test_effort.py
 M tools/agents/test_tracker.py
 M tools/agents/tracker.py
```

### corenius-retro-review

- Source path: `/data02/home/philip.yang/workspace/corenius-retro-review`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/codex/retro-review`
- Disposition: preserve; content review required before migration or retirement.

```text
?? tools/agents/checks.d/27-review-snapshot.sh
?? tools/agents/review_snapshot.md
?? tools/agents/review_snapshot.py
?? tools/agents/review_snapshot_test.py
```

### corenius-session-failure-audit-20261013

- Source path: `/data02/home/philip.yang/workspace/corenius-session-failure-audit-20261013`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `detached`
- Disposition: preserve; content review required before migration or retirement.

```text
?? docs/practice/session-failure-audit-2026-10-12.md
```

### benchmark-exemplars

- Source path: `/data02/home/philip.yang/workspace/corenius/.trae/worktrees/benchmark-exemplars`
- HEAD: `de806765f3e141a2fa4622b8a4f4b91f5983afc8`
- Branch: `refs/heads/worktree-benchmark-exemplars`
- Disposition: preserve; content review required before migration or retirement.

```text
 M AGENTS.md
 M bench/README.md
 M bench/harness/fleet.py
 M bench/harness/run.py
 M bench/harness/tasklib.py
 M bench/harness/test_run.py
 M bench/harness/test_validate.py
 M bench/harness/validate.py
 M tools/insula/capsule.py
 M tools/insula/test_capsule.py
?? bench/harness/materialize_public_tasks.py
?? bench/harness/test_materialize_public_tasks.py
?? bench/insula/requests/deep_swe.json
?? bench/insula/requests/swe_bench.json
?? bench/insula/requests/terminal_bench.json
?? bench/inventories/
?? bench/tasks/real_terminal_bench_analyze_access_logs/
?? tools/insula/docker_image_rootfs.py
?? tools/insula/test_docker_image_rootfs.py
```

### benchmark_exemplars_main

- Source path: `/data02/home/philip.yang/workspace/corenius/.trae/worktrees/benchmark_exemplars_main`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/worktree-benchmark_exemplars_main`
- Disposition: preserve; content review required before migration or retirement.

```text
M  AGENTS.md
M  bench/README.md
M  bench/harness/fleet.py
M  bench/harness/run.py
M  bench/harness/tasklib.py
M  bench/harness/test_run.py
M  bench/harness/test_validate.py
M  bench/harness/validate.py
M  tools/insula/capsule.py
M  tools/insula/test_capsule.py
?? bench/harness/materialize_public_tasks.py
?? bench/harness/test_materialize_public_tasks.py
?? bench/insula/requests/deep_swe.json
?? bench/insula/requests/swe_bench.json
?? bench/insula/requests/terminal_bench.json
?? bench/inventories/
?? bench/tasks/real_terminal_bench_analyze_access_logs/
?? tools/insula/docker_image_rootfs.py
?? tools/insula/test_docker_image_rootfs.py
```

### underlying-session-transcript-audit

- Source path: `/data02/home/philip.yang/workspace/corenius/.trae/worktrees/underlying-session-transcript-audit`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/worktree-underlying-session-transcript-audit`
- Disposition: preserve; content review required before migration or retirement.

```text
?? docs/practice/session-tool-failure-scientific-report-2026-10-14.md
?? docs/practice/session-transcript-corpus-review-2026-10-13.md
?? docs/practice/tool-exchange-failure-corpus-research-2026-10-14.md
```

### underlying-trace-research

- Source path: `/data02/home/philip.yang/workspace/corenius/.trae/worktrees/underlying-trace-research`
- HEAD: `e045d4a6ae2a5efaa53fab93a129b3a57098f7ad`
- Branch: `refs/heads/worktree-underlying-trace-research`
- Disposition: preserve; content review required before migration or retirement.

```text
?? docs/practice/tool-exchange-failure-corpus-research-2026-10-14.md
?? docs/practice/underlying-session-corpus-audit-correction-2026-10-13.md
```

## Branches not merged into main

Preserve every branch. Divergence is not proof that its contents remain useful.

```text
  codex/collab-lite
  codex/collab-ops-22
  codex/collab-ops-25
  codex/collab-ops-atomic-refresh
  codex/collab-ops-cleanup
  codex/collab-ops-complete
  codex/collab-ops-core-repair
  codex/collab-ops-execution-repair
  codex/collab-ops-interrupt-repair
  codex/collab-ops-legacy
  codex/collab-ops-legacy-repair
  codex/collab-ops-mutation-recovery
  codex/collab-ops-operation-repair
  codex/collab-ops-push-repair
  codex/collab-ops-refresh
  codex/collab-ops-sweep-repair
```

## Durable efforts

```text
agentic-engineering
collab-lite
collab-ops
controller-backed-bench
controller-portfolio-stabilization
engineering-profile-adoption
engineering-profile-labs
lead-worker-readiness
mac-operator-center
multi-agent-bench
pi-env-durable
qwen3-training-bringup
semantic-polyglot-layout
sqlite-controller
team-control-plane
worker-monitoring
```
