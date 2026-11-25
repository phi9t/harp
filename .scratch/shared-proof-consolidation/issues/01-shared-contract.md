# 01: Connect the workbench to shared Lean infrastructure

Status: blocked
Owner: Argonaut consolidation coordinator
Worktree: `consolidation/shared-proof-contract` (isolated Git worktree)
Parent: [Shared proof consolidation](../spec.md)

## What to build

Publish the Harp-owned theorem selection and byte provenance as a declarative
consumer contract. Use the installed shared projector/build interface; retain
the existing standalone proof targets. Link the independent ML research owner
from the repository and shared-infrastructure guides. Repair the observed Linux task-shell failure.

## Acceptance criteria

- The selected theorem source and toolchain match the recorded hashes.
- A changed source or contract fails the focused provenance check.
- The gate selects that check for descriptors and their source consumers.
- Existing proof and runtime implementation bytes remain unchanged.
- Run the whole candidate's changed-surface gate and record its actual result.
- Qualify native managed execution separately against its exact inputs.

## Blocked by

Source implementation, local verification and publication approved by the owner
are complete. Managed Lean profile preparation and native build/type/axiom audit
and replay qualification remain pending. Existing runtime paths remain supported
until their native retirement gates pass.

## Verification

On Argonaut, `mise run verify -- --base c50a2478e64d25fe8924dba33116f0f7f344bcde`
passed: diff whitespace, exact source/contract provenance, three provenance
regressions, twenty-two changed-surface selector tests, and local document links.
The native Mise command used the explicit Bash project shell successfully.

Independent review found and repaired an overly broad metadata exemption;
unexpected Lean files under the contract directory retain the existing proof
gate. Shared-project source-only preparation passed using the original Linear
bytes. No managed proof, native runtime parity, live dispatch or full release
qualification is claimed. Teaching packet contents remain unchanged; wayfinding
lives in the repository and shared-infrastructure guides.

The owner explicitly approved publication, and public `main` was read back at
`edc6d8a0efe657b15bb10a2dce60a008ab0babcb`. Source publication is complete;
the native qualification limitations above remain.
