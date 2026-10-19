# 07: Capture and independently verify an exact candidate

Status: done
Owner: Codex
Branch: codex/harp-durable-runtime
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Capture the proposed repository revision from an admitted worker and run operator-configured checks in an independent materialization, with durable evidence tied to that exact candidate.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] The captured candidate records the admitted task/input identity, expected base, commit/tree and relevant execution contract and remains available after worker cleanup.
- [x] The verifier runs the configured check command against independently materialized candidate bytes, without granting the worker control over the verifier or its evidence.
- [x] Outcome, command, environment identity and retained logs identify the exact verification subject. Failed/timed-out checks are inspectable and never become a passing result.
- [x] Changed candidate bytes, task/required inputs or substituted verification state cannot reuse stale evidence. A worker-reported success or unverified result file is insufficient.
- [x] The public CLI supports inspecting the candidate and evidence. Controlled Git fixtures demonstrate that verification remains bound to the captured revision after the worker workspace changes.

## Blocked by

- [03: Admit a pinned task into an isolated workspace](03-admit-pinned-task.md)

## Scope

The deterministic coding activity from ticket 03 can produce the candidate, so this ticket does not depend on either live provider adapter.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

2026-10-18: Five real public-CLI Linux cases passed in 56.774 seconds,
zero skips, using independent Git materializations and nested Insula namespaces:
capture survived original-payload removal; failed checks retained logs and never
passed; candidate-written fake proof/log files could not replace parent evidence;
tracked source mutation failed verification; a hung/forked check hit the outer
deadline with partial output retained, no passing result and an empty kernel
cgroup after cleanup. SQLite 3.51.3; 37 frozen source files were unchanged.
Evidence: `/data03/harp-runtime-integration-philip-yang/qualification/verification-01-result.json`
and `verification-01.log`. The actual Linux Harp binary SHA-256 was
`d8fa70ef769f6bcb5cb064f19401beab64c96b3354454d553c52b48891e94163`.
The verifier uses unbuffered parent-owned logs so timeout cleanup retains output.
Tests are committed in `b1db1562`.

The selected batch gate is recorded in the integration commit. The macOS
package run passed 62 cases with 21 explicit Linux/delegation skips; the Linux
qualification above supplies the required real boundary evidence.
