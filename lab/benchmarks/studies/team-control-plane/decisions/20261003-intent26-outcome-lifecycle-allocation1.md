# Adopt the accepted-work lifecycle; bounded follow-up allocation1

Commit read: `bccb4aaa3c798c6e1d58f9bb7f518bc23e256f41`
Authority: [explicit full-goal Owner authorization](20261003-owner-authorizes-full-goal-execution.md)

Adopt the exact [outcome-lifecycle proposal](20261003-intent26-outcome-lifecycle-proposal.md)
SHA256 `ed3c1db9a5b8a2b2e2cbfed1a58e9814ccdd836f2f2f321b2f2a994904826d4e`.
Independent [design critique](../evidence/intent26/repair3/outcome-design-critique/report.md)
PASS resolves the architecture escalation. No blocking architectural uncertainty
remains. This decision supplements the amendment-retirement and Plan-ceilings
contracts; it does not override either or accept the rejected code.

## Required implementation guardrails

- Derive accepted current work from exact Task/IntentRevision/outcome subject,
  keeping production proof projections and inert initial v1 mapping distinct.
  Use one small internal lifecycle interface rather than another authority table.
- Preserve authenticated exact outcome.accept and artifact Accept receipt replay.
  A blanket mutation guard before receipt lookup breaks this exception; moving
  every cached command ahead of terminal policy revives forbidden work. Cached
  proposal/binding/verification/launch is not an accepted-receipt inspection.
- Authorized later-work intent.admit must pass its separate amendment eligibility
  check over accepted work while retaining cancellation/delegation fences. Shared
  dispatch helpers must still refuse fresh consequences of completed work.
- Check operation targets, including Owner-targeted mailbox delivery/runtime
  binding; actor.task alone is insufficient. Raw immutable blob storage and
  physical-effect evidence collection confer no candidate/report authority and
  remain available under existing authentication.
- Preserve current scoped actors' existing historical inspect/Stop/log/Release/
  usage-reconciliation paths. Do not widen legacy CancelAttempt/ReconcileAttempt,
  revive old credentials, refund unknown/consumed usage or change original
  nullable Attempt provenance. Exact event-bound retirement remains mandatory.

## New bounded assignment

The original repair rounds1–3 are exhausted and preserved as rejected history.
This is one newly explicit post-architecture allocation, `outcome-lifecycle1`,
under the Owner's full-goal authority. It is not a reset or renumbering of those
rounds. Assign one fresh clean-context writer in its own new Corenius worktree;
retain the former author tree unchanged. Root owns intent/integration, not code
implementation, and runs no overlapping Cargo/PostgreSQL gates.

Initialize the new writer's baseline from the exact20 frozen repair3 source
postimages, after checking root repair2 preimages against the retained preflight.
Record that overlay as a rejected baseline, preserving0008 and0009 bytes and all
previous seals. Implement only this accepted-work lifecycle and its direct
consumers, tests and necessary documentation. No scope27, interpreter, runtime
provider or migration redesign is authorized here.

Approve the existing real HTTP/PostgreSQL and supervised fixture seams for the
seven-case architecture proof matrix and four critique guardrails. Show actual
RED before each changed behavior, then GREEN with genuinely measured evidence.
Preserve all failed/setup runs and distinguish them from behavioral proof.
No mere source assertions, fabricated production PASS or mocked lifecycle proof.

The writer owns an exclusive local Cargo/PostgreSQL slot with private target,
build jobs2 and serialized tests. At most10 top-level Cargo test launches,
including setup/fixture failures, plus existing formatting/strict lint checks.
Each command has an1800s timeout and cumulative compile/test wall time is bounded
by3600s. Existing supervised local verification fixtures and isolated PostgreSQL
are admitted; use unique run/container/workspace identities. No Temporal,
Trae/provider, remote or live-v2 qualification run, full repository gate,
protected-ref mutation, author staging/commit or destructive cleanup is admitted.
Stop and report before exceeding this allocation; no silent retries/extensions.

Freeze source-bound runs, exact base and rejected-baseline patches/preimages,
public records, measured ArtifactService bytes, immutable original operation
attribution, terminal results and independent stop/release/usage/resource
observations under evidence/intent26/outcome-lifecycle1/. Do not alter old evidence
or promote incomplete cleanup to confirmed release. Report resource disposition
explicitly before root may acquire the slot.

This allocation includes one fresh independent Standards/Spec conformance pass.
If a blocker remains, preserve it and stop implementation for explicit disposition.
No automatic repair loop follows. Passing conformance permits root to verify and
integrate exact postimages for focused/full gates and independent whole-branch
review; only all accepted gates permit gated fast-forward landing. Ticket26
remains claimed and27 blocked until those gates pass.
