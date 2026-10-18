# 03: Admit a pinned task into an isolated workspace

Status: done
Owner: Codex
Branch: codex/harp-durable-runtime
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Submit Git-managed coding intent through Harp and execute a deterministic coding activity in an isolated workspace with a frozen task revision, base, provider selection and execution contract.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] Admission records the exact task content/revision, acceptance inputs, Git base, configured provider, workflow version and execution contract. Later edits do not change that admitted attempt.
- [x] Repeated submission with identical identity and inputs returns the original admission; conflicting reuse is refused without duplicate work.
- [x] The admitted activity changes only its own workspace. It cannot change the primary checkout, another workspace or authoritative runtime state.
- [x] The public CLI exposes one shared worker-activity contract for immutable requests, attempt identity, output/log collection and result acceptance, usable by both provider adapters.
- [x] Admission and launch use the existing Insula/guardian boundary and establish backend ownership before releasing the activity. Missing required isolation capability refuses admission.
- [x] With capacity one, a second task queues. Capacity is not reused until the owned execution is cleaned up; missing or forged activity results do not count as completion.

## Blocked by

- [02: Operate the durable runtime through Harp’s CLI](02-operate-project-runtime.md)

## Scope

The deterministic activity provides an end-to-end fixture for later provider, recovery and Git tickets. CPU/GPU accounting remains logical admission rather than a new hardware-isolation claim.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

### Implemented slice and verification

- Admission freezes committed Markdown task bytes, acceptance JSON, source base,
  task revision, provider/model, workflow version and execution contract. Input
  manifests hash exact file bytes; bundles are published under their digest.
- A private export repository advertises only the selected base for the Git
  bundle. Source refs, index and working edits remain unchanged. Repeated identical
  admission returns the retained identity; conflicting reuse refuses.
- Runtime code is snapshotted and executed from the admitted import root. Native
  files, modes and symlink targets are fully inventoried; required executable paths
  are checked before publishing an execution record.
- Coding dispatch requires Insula and a delegated cgroup, validates input bytes
  before claiming an attempt, projects only the selected input bundle read-only,
  and clones into the attempt's private writable Git repository. Guardian-owned
  receipts require kernel cleanup evidence before capacity/result acceptance.
- Real Linux qualification through the compiled Harp CLI under an ephemeral
  delegated systemd user service passed all nine admission tests in 35.564s:
  immutable Git bytes/export, repeated/conflicting admission, private candidate
  production, tampered input refusal before claim and capacity-one queueing.
  The successful coding receipt recorded `cleanup_proof.backend=cgroup-v2`.
- Independent read-only review found no required issue. A guardian-accepted worker
  proposal is the result of this slice. Independent candidate verification,
  acceptance-check execution and human-approved landing remain tickets 07/08.
- Supplementary imported runtime/Insula/cgroup suite: 76 passed, four explicit
  delegation skips in the nondelegated harness. New real namespace/read-only
  bundle tests ran there; the public CLI harness above supplies actual delegated
  cgroup execution. Local package checks: 17 passed, 11 explicit Linux/delegation
  skips. Full source provenance verifier passed.
