Independent Corenius v0.2 adoption critique. Read-only review of commit `942f037800102f3d786ac547e72f11ab928b50ae`; accepted kernel `264756edd550225449202b6f20ac4b560808d14e` remains valid within its recorded v1 scope. No runtime qualification or new test execution occurred.

Adopt the owner's target architecture. A SPEC addendum plus blocker-linked tickets is appropriate; captured product assertions cannot substitute for executable qualification. Existing Kata/jj deferral is historical evidence, superseded for the target. Three source conflicts require explicit compatibility treatment: `domain.rs:49` requires GitInput; migration 0005 keeps UNIQUE(attempt_id), even after 0007 makes it nullable; migration 0004/`temporal.rs:30` supports mission/task aggregates. Conversely, role separation already exists: migration 0006 defines mission_leads/lead_generation, `mailbox.rs:1430` replaces only that seat, and `supervisor.rs:242` checks role/task generations separately. Describe extension rather than reinvention.

Material requirements to add:

1. **Admitted dependencies are exact contracts.** Bind readiness to prerequisite WorkRef + admitted IntentRevision + accepted subject, never Kata CLOSED or current mutable relationship edges. Define whether a later prerequisite amendment blocks future effects while preserving already admitted operations. Include tracker deployment/tenant identity, tombstones, missing revisions, and canonical snapshot digest. Keep acceptance, plan, role and scope changes independently authorized.
2. **Kata mutations need their own uncertain-effect protocol.** Journal request/key/payload before I/O; reconcile lost acknowledgments using qualified API semantics. Creation of children and dependency edges is a partially completable bundle, not an atomic cross-service transaction. Do not admit partially reconstructed decomposition. Specify cursor namespace/order, duplicate/out-of-order handling, retention gaps, snapshot/high-watermark resync, bounded backpressure and projection-origin suppression. Kata outage must preserve admitted execution; fresh admissions needing unavailable evidence stop.
3. **Scope owns jobs and resource conservation.** Separate producing-attempt provenance from scope ownership and mutable controller. Multiple jobs per attempt/scope, child allocations, outstanding effects and unknown usage must survive role/attempt replacement. Define atomic parent-child allocation transfer rather than charging the same allocation twice; cancellation receipt names previously granted effects and unresolved termination. Distinguish runtime incarnation, workspace generation, scope epoch, role epoch and deployment recovery epoch. Migration must preserve original attribution and close admission during ambiguous backfill.
4. **Plan IR needs operational semantics.** Stable step/operation identities, effect allowlists, bounded loops, join/repair behavior and immutable plan versions are necessary. Define amendment safe points and fate of in-flight operations; a plan edit cannot revive revoked authority, reset budgets or reinterpret completed steps. Carry version/cursor/wait identities across CAN and retain compatible interpreter versions for old histories. The illustrative changed=False loop loses a wake arriving during refresh/ensure. Current workflows also clear a boolean after drain, but their 500 ms timer bounds delay (`temporal.rs:724,759`). Preserve that safety before lengthening waits; prove sequence-based wake handling plus anti-entropy.
5. **Exact landing requires a physical gate.** `artifacts.rs:1120` explicitly prepares fixture candidates without ref mutation. Bind grant, candidate/tree, verification policy/environment, target and expected predecessor; enforce fast-forward ancestry. Serialize the protected mutation, journal intent before CAS, retain effect attribution, and reconcile success-before-PG/lost acknowledgment before releasing conflicts. Merely seeing candidate at target does not prove this operation performed the effect. Out-of-band writers need an explicit exclusion or reconciliation contract. jj change IDs remain lineage only.
6. **ExecutionContract is a provider seam, not a product capability claim.** Pin Armada/Insula versions and qualify stable job identity, idempotency/discovery, independent inventory, cancellation versus confirmed termination, checkpoint/log integrity, admission resource limits and orphan coverage. The accepted local PID-namespace supervisor is useful evidence; it does not establish remote/GPU semantics.

Smallest sound graph: A intent/acceptance/Plan contracts → B scope/epochs/conserved allocations; A+B → C Kata adapter; B → D independent execution adapter; A+B+D → E per-scope Temporal interpreter; A+B+D → F protected gate. Qualify jj 19 and selected backend 25 against A/B/D/F. API-only Kata and jj experiments can proceed before implementations. Require one v0.2 assembled proof after C/D/E/F/25 before live 21; demonstrations 22/23 follow 21; restore 24 covers every selected adapter and migration. Avoid a cycle where D needs E while E needs D: D exposes independently testable ensure/inspect/stop operations. Preserve accepted 16 evidence; do not silently relabel it v0.2 assembly acceptance.

F01–F32 coverage must cite retained acceptance manifests and observable assertions. The following maps available source evidence conservatively; “partial” requires further v0.2 proof, and “gap” claims no passing evidence. Paths below are under `crates/team-control-plane/tests/`.

| Fault | Exact existing evidence or gap |
|---|---|
| F01–F02 | `kernel.rs:31` admission idempotency; scoped v1 evidence. |
| F03 | Partial: ticket 13 repair1 outbox/Activity recovery; exact broker-commit/pre-dispatch crash attribution needed. |
| F04 | `temporal.rs:907` real lost-hint/CAN/replay; per-scope sequence races gap. |
| F05 | `gateway.rs:27`, ticket 12 accepted manifest; live provider gap. |
| F06 | Partial: broker-only continuation baseline; dual native-owner fault gap. |
| F07 | `supervisor.rs:600` initiator death; independent remote job/scope gap. |
| F08 | `mailbox.rs:185–208` old-lead rejection; scoped evidence. |
| F09 | `supervisor.rs:544` stale-generation fencing; workspace epoch extension needed. |
| F10–F11 | Partial: `assembled_cancellation.rs:13`, `supervisor.rs:496`; scope receipt/remote unreachable gap. |
| F12 | `assembled_vertical.rs:2326` rejects cancelled submissions; bounded late-result quarantine gap. |
| F13 | Gap: concurrent nested allocation conservation. |
| F14–F15 | Partial: `kernel.rs:603`, `assembled_vertical.rs:2294`; provider-switch/cumulative-inclusion gap. |
| F16 | `artifacts.rs:643` stale candidate/spec/base evidence rejection. |
| F17–F18 | `assembled_vertical.rs:6609,8345,523`; policy/tamper/false-success scoped evidence. |
| F19–F20 | Gap: physical protected-ref CAS and success-before-PG receipt. |
| F21 | Gap: primary-checkout untracked collision preservation. |
| F22 | Partial: ticket 15 namespace/isolation acceptance; missing-capability admission fault needs exact attribution. |
| F23 | Gap: poisoned cache provenance. |
| F24 | `artifacts.rs:5` interrupted publication only; ENOSPC/checkpoint preservation gap. |
| F25 | `artifacts.rs:877` referenced-artifact GC; unresolved-operation dependency race gap. |
| F26 | Gap: restore 24 post-backup survivor drill. |
| F27 | ticket 13 retained v1 replay; incompatible interpreter deployment gap. |
| F28–F29 | Bounded mailbox/operator sources; telemetry flood and long silent job faults gap. |
| F30 | mailbox reconstruction evidence; real hidden-transcript demonstrations 22/23 gap. |
| F31–F32 | Gap: research acceptance and evaluator-change isolation contracts/proofs. |

External Kata/jj/Temporal/Armada claims were not independently retrieved. Capture provenance is verified separately from capability qualification.
