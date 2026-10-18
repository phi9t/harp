# 03: Admit a pinned task into an isolated workspace

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Submit Git-managed coding intent through Harp and execute a deterministic coding activity in an isolated workspace with a frozen task revision, base, provider selection and execution contract.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Admission records the exact task content/revision, acceptance inputs, Git base, configured provider, workflow version and execution contract. Later edits do not change that admitted attempt.
- [ ] Repeated submission with identical identity and inputs returns the original admission; conflicting reuse is refused without duplicate work.
- [ ] The admitted activity changes only its own workspace. It cannot change the primary checkout, another workspace or authoritative runtime state.
- [ ] The public CLI exposes one shared worker-activity contract for immutable requests, attempt identity, output/log collection and result acceptance, usable by both provider adapters.
- [ ] Admission and launch use the existing Insula/guardian boundary and establish backend ownership before releasing the activity. Missing required isolation capability refuses admission.
- [ ] With capacity one, a second task queues. Capacity is not reused until the owned execution is cleaned up; missing or forged activity results do not count as completion.

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
