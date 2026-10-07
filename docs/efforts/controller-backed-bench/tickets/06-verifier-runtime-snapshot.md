# 06 verifier-runtime-snapshot: Pin the runtime actually consumed by controller verification

Type: task
Status: resolved
Owner: trae-controller-bench-runtime
Blocked by:
Claimed:

Triage: ready-for-agent

## Acceptance

- [x] Controller verification consumes a private pinned runtime snapshot instead of ambient host libraries; worker inspection compatibility and acceptance semantics stay intact.
- [x] Runtime tree includes Python/stdlib/extensions/loader/shared dependencies; missing or altered contents and escaping symlinks rejected before dispatch and trusted proof validation.
- [x] Runtime manifest/digest bind to actual verifier environment/proof and retained benchmark metadata.
- [x] Original complete title-case and clamp tests each fail on noop and pass on independent fix through this runtime.
- [x] Behavioral proof: snapshot tampering rejects verification/acceptance, isolated ambient fixture changes cannot affect verification, workers cannot write runtime or load omitted ambient libraries.
- [x] Relevant controller and benchmark checks pass; no host library modification or existing user-state mutation.

## Approved design and parent

Parent SPEC plus VERIFIER-RUNTIME-PREREQUISITE.md. User authorized fixing this situation on 2026-10-05. Gated path: verification boundary change; independently critiqued private snapshot design approved. Public test seams: actual controller verification/proof validation and deterministic benchmark CLI, red-green behavior required.

## Scope

Optional verifier-only runtime policy, independently owned content-addressed snapshot from benchmark adapter, proof identity/enforcement. Preserve legacy configurations and worker inspection/acceptance semantics. Linux/Corenius only. No stale acceptance redesign, recovery redesign, live Codex trial, commits/pushes/mainline landing.

Input: reviewed ticket01 uncommitted execution overlay at committed d7f6748, content manifest in worker run directory. No claim this overlay is committed or runtime-qualified. Sole writer per worktree.

## Verification

Visible red-green, focused checks and bounded relevant fixtures, separate Standards/Spec review, independent verification, maximum three repair rounds. Record exact commands/status/artifacts/source identities/owned-process disposition. Do not repeat or broaden passing tests without new justification.

## Evidence

Implementation evidence is retained under `/data02/home/philip.yang/workspace/corenius-codex-controller-bench-integration/.scratch/controller-backed-bench/implementation-06/`. The retained CLI artifact set `retained-cli-artifacts-20261005T175217Z/` contains t1/t2 `result.json` files plus copied controller `source-manifest.json`, `git.json`, `runtime-manifest.json`, and noop/fixed verifier proof log/report artifacts.

Verified behavior now uses `TMPDIR=/data02/home/philip.yang/.cb-runtime` for controller run roots, builds a private verifier-only runtime snapshot, mounts that snapshot read-only at `/usr`, and binds the runtime manifest digest into the verifier environment/proof. The benchmark snapshot includes the resolved pinned Python executable path, stdlib, extension modules, dynamic loader and shared-library dependencies. Runtime tampering, missing entries, tampered regular files, direct escaping symlinks, and chained `..` symlink targets are rejected before verifier dispatch or trusted-proof acceptance. Legacy Git policies without `verifier_runtime` remain compatible.

## Comments
- 2026-10-05T17:18:00Z coordinator: User authorized scoped runtime prerequisite and proactive unblocking/escalation. Canonical integration ticket now fixed.

- 2026-10-05T18:00:00Z coordinator: Acceptance checked against fresh retained t1/t2 results, proof tamper and invalid-input tests, 20 harness tests, legacy fixture, both independent reviews and exact source manifest.
- 2026-10-05T18:01:33Z trae-controller-bench-runtime: resolved
