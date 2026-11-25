# Shared proof infrastructure and independent ML research

Status: in-progress
Owner: Argonaut consolidation coordinator
Worktree: `consolidation/shared-proof-contract`, based on `c50a2478e64d25fe8924dba33116f0f7f344bcde`

## What to build

Implement the owner's approved repository consolidation: retain Harp as the
standalone agent workbench and teaching library; establish independent ML
research in `phi9t/lean-ml-foundations`; consume explicit installed interfaces
from the existing Argonaut infrastructure owners. Coordinate the complete
source snapshot in [programme issue 14](https://github.com/phi9t/argonaut-research/issues/14).

The seed retains thirteen proof files byte for byte and records their origin.
Historical Harp proofs and educational acceptance policies remain here. Shared
Lean preparation, dependency profiles, compilation and exact type/axiom audit
belong to `insula-lean`. Job lifecycle belongs to `argonaut-control-plane` and
Linux isolation belongs to `insula`. Existing Harp runtime implementations stay
supported until a named replacement passes native parity checks.

## Acceptance criteria

- Explicit ownership and migration boundaries; no competing scheduler.
- Declarative source/type contract accepted by the shared projector, without
  rewriting proof bytes or Harp's standalone dependency manifest.
- Source provenance and contract checks run in the changed-surface gate.
- The project's Bash task scripts run on Linux through an explicit project shell.
- Preserve Lean 4.32.1 and every dependency revision.
- All Argonaut estate, checkouts, tools and caches remain under `/mnt/data_infra`.
- Record portable checks and native qualification separately; no source change
  is described as live dispatch, controller retirement or proof acceptance.

## Blocked by

Native durable dispatch requires the existing supported user service context.
The protected connector cannot reach its user manager. This does not block
source contracts or bounded native validation inside the approved Insula.

## Comments

The earlier completed durable-estate reorganization remains historical and is
not repeated by this task. Its journal and active research owners are preserved.

## Verification

The complete changed-surface gate passed on Argonaut. Native input preparation
and public publication remain separate gates; see the implementation ticket.
