# State: retro-fixes

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "e045d4a6ae2a5efaa53fab93a129b3a57098f7ad",
  "dehydrated": "2026-10-13T05:38:24Z",
  "effort": "retro-fixes",
  "evidence": [
    {
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes/docs/efforts/retro-fixes/VALIDATION.md",
      "sha256": "5150ab51bd57fdd752ebc232d0be8ad1ce553348a58f2766da1f91b5ff48a851"
    },
    {
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes/docs/efforts/retro-fixes/evidence/controller-retry.json",
      "sha256": "9fb7eac3fcaf3a2307b06132523e84427ade926f02386ae2ae313637f8cded90"
    },
    {
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes/docs/efforts/retro-fixes/evidence/final-focused.json",
      "sha256": "1518d4e1982f048f624a0e6a1e8276c3d8f1a7dd2fed1a4d10305461436a5bf5"
    },
    {
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes/docs/efforts/retro-fixes/evidence/gate.json",
      "sha256": "5b8003b594687e0c4e991842d97ba9b1023e0f9e02644eec3b4053e138d303ba"
    },
    {
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes/docs/efforts/retro-fixes/evidence/qualification.json",
      "sha256": "9ab9d2962307bb5a41cf283ff826e5513a89bc41fc047bbc78d776b87b7760f7"
    },
    {
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes/docs/efforts/retro-fixes/evidence/reviews.md",
      "sha256": "064663cc2d976c1082778a12bb65ebe00149a249f294aec1bde9cde06cd617fa"
    }
  ],
  "mode": "snapshot",
  "runs": [],
  "worktrees": [
    {
      "branch": "codex/insula-durable",
      "dirty": [
        "README.md",
        "controller/durable/README.md",
        "controller/durable/__init__.py",
        "controller/durable/__main__.py",
        "controller/durable/cgroup.py",
        "controller/durable/cgroup_test.py",
        "controller/durable/cli.py",
        "controller/durable/common.py",
        "controller/durable/demo_activity.py",
        "controller/durable/engine.py",
        "controller/durable/guard.py",
        "controller/durable/insula.py",
        "controller/durable/insula_test.py",
        "controller/durable/runtime_test.py",
        "controller/durable/workflows.py",
        "docs/adr/0003-insula-durable-execution.md",
        "docs/efforts/insula-durable/EFFORT.md",
        "docs/efforts/insula-durable/SPEC.md",
        "docs/efforts/insula-durable/STATE.md",
        "docs/efforts/insula-durable/VALIDATION.md",
        "docs/efforts/insula-durable/evidence/baseline.md",
        "docs/efforts/insula-durable/evidence/cgroup-implementation.md",
        "docs/efforts/insula-durable/evidence/design-critique.md",
        "docs/efforts/insula-durable/evidence/log-digests.json",
        "docs/efforts/insula-durable/evidence/native-retry.json",
        "docs/efforts/insula-durable/evidence/runtime-implementation.md",
        "docs/efforts/insula-durable/evidence/smoke.json",
        "docs/efforts/insula-durable/tickets/01-runtime.md",
        "docs/efforts/insula-durable/tickets/02-integration.md",
        "tools/agents/checks.d/72-durable-execution.sh",
        "tools/insula/durable.py"
      ],
      "dirty_contents": "local-only; not backed up",
      "head": "e045d4a6ae2a5efaa53fab93a129b3a57098f7ad",
      "path": "/data02/home/philip.yang/workspace/corenius-durable",
      "pushed": false
    },
    {
      "branch": "codex/retro-fixes",
      "dirty": [
        ".github/actionlint.yaml",
        ".github/workflows/check.yml",
        ".github/workflows/durable-qualification.yml",
        "AGENTS.md",
        "README.md",
        "bench/harness/lead_driven.py",
        "bench/harness/test_lead_driven.py",
        "controller/durable/README.md",
        "controller/durable/__init__.py",
        "controller/durable/__main__.py",
        "controller/durable/cgroup.py",
        "controller/durable/cgroup_test.py",
        "controller/durable/cli.py",
        "controller/durable/common.py",
        "controller/durable/demo_activity.py",
        "controller/durable/engine.py",
        "controller/durable/guard.py",
        "controller/durable/insula.py",
        "controller/durable/insula_test.py",
        "controller/durable/qualify.py",
        "controller/durable/qualify_test.py",
        "controller/durable/runtime_test.py",
        "controller/durable/workflows.py",
        "docs/adr/0003-insula-durable-execution.md",
        "docs/agents/agentic-engineering.md",
        "docs/agents/agentic-workflow-repos.json",
        "docs/agents/issue-tracker.md",
        "docs/efforts/insula-durable/EFFORT.md",
        "docs/efforts/insula-durable/SPEC.md",
        "docs/efforts/insula-durable/STATE.md",
        "docs/efforts/insula-durable/VALIDATION.md",
        "docs/efforts/insula-durable/evidence/baseline.md",
        "docs/efforts/insula-durable/evidence/cgroup-implementation.md",
        "docs/efforts/insula-durable/evidence/design-critique.md",
        "docs/efforts/insula-durable/evidence/log-digests.json",
        "docs/efforts/insula-durable/evidence/native-retry.json",
        "docs/efforts/insula-durable/evidence/runtime-implementation.md",
        "docs/efforts/insula-durable/evidence/smoke.json",
        "docs/efforts/insula-durable/tickets/01-runtime.md",
        "docs/efforts/insula-durable/tickets/02-integration.md",
        "docs/efforts/retro-fixes/EFFORT.md",
        "docs/efforts/retro-fixes/SPEC.md",
        "docs/efforts/retro-fixes/STATE.md",
        "docs/efforts/retro-fixes/VALIDATION.md",
        "docs/efforts/retro-fixes/evidence/carried-forward.json",
        "docs/efforts/retro-fixes/evidence/controller-retry.json",
        "docs/efforts/retro-fixes/evidence/final-focused.json",
        "docs/efforts/retro-fixes/evidence/gate.json",
        "docs/efforts/retro-fixes/evidence/logs.json",
        "docs/efforts/retro-fixes/evidence/qualification-provenance.md",
        "docs/efforts/retro-fixes/evidence/qualification.json",
        "docs/efforts/retro-fixes/evidence/reviews.md",
        "docs/efforts/retro-fixes/evidence/skill-change.md",
        "docs/efforts/retro-fixes/tickets/01-gate.md",
        "docs/efforts/retro-fixes/tickets/02-wip-review.md",
        "docs/efforts/retro-fixes/tickets/03-handoff.md",
        "docs/efforts/retro-fixes/tickets/04-qualification.md",
        "tools/agents/VALIDATION.md",
        "tools/agents/check.py",
        "tools/agents/check.sh",
        "tools/agents/check_test.py",
        "tools/agents/checks.d/05-gate-dispatch.sh",
        "tools/agents/checks.d/27-review-snapshot.sh",
        "tools/agents/checks.d/70-sqlite-controller.sh",
        "tools/agents/checks.d/72-durable-execution.sh",
        "tools/agents/checks.d/73-durable-qualification.sh",
        "tools/agents/checks.d/README.md",
        "tools/agents/effort.py",
        "tools/agents/review_snapshot.md",
        "tools/agents/review_snapshot.py",
        "tools/agents/review_snapshot_test.py",
        "tools/agents/test_check.py",
        "tools/agents/test_effort.py",
        "tools/agents/test_tracker.py",
        "tools/agents/tracker.py",
        "tools/insula/durable.py",
        "tools/temp_root.py",
        "tools/test_temp_root.py"
      ],
      "dirty_contents": "local-only; not backed up",
      "head": "e045d4a6ae2a5efaa53fab93a129b3a57098f7ad",
      "path": "/data02/home/philip.yang/workspace/corenius-retro-fixes",
      "pushed": false
    },
    {
      "branch": "codex/retro-gate",
      "dirty": [
        "bench/harness/lead_driven.py",
        "bench/harness/test_lead_driven.py",
        "tools/agents/check.sh",
        "tools/agents/checks.d/05-gate-dispatch.sh",
        "tools/agents/checks.d/README.md",
        "tools/agents/test_check.py",
        "tools/temp_root.py",
        "tools/test_temp_root.py",
        "tools/agents/check.py",
        "tools/agents/check_test.py"
      ],
      "dirty_contents": "local-only; not backed up",
      "head": "e045d4a6ae2a5efaa53fab93a129b3a57098f7ad",
      "path": "/data02/home/philip.yang/workspace/corenius-retro-gate",
      "pushed": false
    },
    {
      "branch": "codex/retro-handoff",
      "dirty": [
        "tools/agents/effort.py",
        "tools/agents/test_effort.py",
        "tools/agents/test_tracker.py",
        "tools/agents/tracker.py"
      ],
      "dirty_contents": "local-only; not backed up",
      "head": "e045d4a6ae2a5efaa53fab93a129b3a57098f7ad",
      "path": "/data02/home/philip.yang/workspace/corenius-retro-handoff",
      "pushed": false
    },
    {
      "branch": "codex/retro-review",
      "dirty": [
        "tools/agents/checks.d/27-review-snapshot.sh",
        "tools/agents/review_snapshot.md",
        "tools/agents/review_snapshot.py",
        "tools/agents/review_snapshot_test.py"
      ],
      "dirty_contents": "local-only; not backed up",
      "head": "e045d4a6ae2a5efaa53fab93a129b3a57098f7ad",
      "path": "/data02/home/philip.yang/workspace/corenius-retro-review",
      "pushed": false
    }
  ]
}
```
<!-- effort:machine:end -->

## Where we are

Implementation and review are complete in `codex/retro-fixes` at
`/data02/home/philip.yang/workspace/corenius-retro-fixes`, based on e045d4a.
The five retrospective repairs and carried-forward durable runtime remain
uncommitted. Main and the original durable worktree are unchanged.

Native durable qualification passed 73 tests with zero skips. The initial full
gate passed 25 entries and timed out the controller entry at 900 seconds; the
isolated controller retry passed all 138 tests. Focused regressions and
independent Standards, Spec and defect reviews passed. See VALIDATION.md and
evidence/reviews.md for exact scope and limits.

## Next actions

Await human acceptance and separate Git authorization. Tickets 01–04 retain
claimed status with implementation handed off awaiting acceptance. No dependent
work is unblocked and no accepted Candidate or landing permission is implied.
The snapshot preserves local dirty-state references; it is not a backup.

Hosted CI activation and trusted qualification runner protections remain external
setup work: this repository has no configured origin. Follow
`tools/agents/VALIDATION.md` when hosting becomes available.
