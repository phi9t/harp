# Corenius engineering team

Language for the persistent engineering team and its authority boundaries.
These definitions describe the domain; they do not claim implementation readiness.

## Fleet, execution and developer systems

**Armada**:
The fleet-management plane for SSH-accessible compute, including machine identity,
routes, reachability, capabilities, health and ownership context.
_Avoid_: Workload executor, Insula lifecycle owner, cluster scheduler.

**Insula**:
A live bwrap-based hermetic execution instance with declared inputs, projections
and runtime policy.
_Avoid_: Capsule or environment as a separate runtime domain object, rootfs, fleet manager.

**Insula contract**:
The immutable resolved definition of an Insula's base filesystem, dependency
closure, workspace and host/device projections, and runtime policy.
_Avoid_: Live Insula, capsule contract, mutable host configuration.

**Insula ID**:
The identity of a specific live Insula instance, separate from its contract's
content identity.
_Avoid_: Contract digest, rootfs digest, RuntimeIncarnation.

**rootfs**:
A frozen base filesystem artifact from which an Insula starts.
_Avoid_: Running environment, live Insula, workspace.

**dependency bundle**:
An immutable additional closure of tools, packages or dependencies projected
into an Insula.
_Avoid_: Mutable host installation, workspace projection.

**workspace projection**:
The declared code or project state exposed inside an Insula.
_Avoid_: rootfs, dependency bundle, unrestricted host filesystem.

**DevX**:
The developer-experience system that creates and maintains durable developer
Insulas and the tools used within them.
_Avoid_: Fleet inventory, general cluster execution system.

**Metallica**:
The cluster execution system that realizes Insula-based workloads across compute
resources and distributed roles or Pods.
_Avoid_: SSH wrapper, Armada execution, one cluster-wide Insula.

**TorchTitan**:
The workload system that defines training semantics within the supplied execution
architecture.
_Avoid_: Fleet management, Insula substrate, distributed placement authority.

**SGLang**:
The workload system that defines serving semantics within the supplied execution
architecture.
_Avoid_: Fleet management, Insula substrate, distributed placement authority.

**hermetic**:
Conformance to an explicitly resolved toolchain, dependency closure, filesystem
inputs and execution assumptions, rather than ambient host inheritance.
_Avoid_: Network-isolated, hostile-code security guarantee, reproducibility guarantee.

**network-isolated**:
An Insula's explicitly constrained network connectivity, independent of its
hermetic input closure.
_Avoid_: Hermetic as a synonym, default property of every Insula.

## Work and outcomes

**WorkRef**:
A work item's identity within a particular ledger deployment and project.
_Avoid_: Bare ticket number as global identity.

**IntentRevision**:
An immutable admitted statement of a work item's objective, constraints,
dependencies and required outcome, tied to an exact source revision.
_Avoid_: Latest ticket text, plan.

**AcceptanceContract**:
The admitted criteria and evidence needed to recognize a particular outcome as
successful, including a valid negative research result when specified.
_Avoid_: Agent says done, ticket closed.

**PlanRevision**:
An admitted, versioned strategy for pursuing an IntentRevision without changing
its required outcome or granting additional authority.
_Avoid_: AcceptanceContract, conversation summary.

**Candidate**:
An immutable proposed result whose identity includes the content and context
needed to verify and accept it.
_Avoid_: Mutable change ID as proof subject.

## Ownership and execution

**ExecutionScope**:
A durable unit of admitted work and authority that retains its jobs, outstanding
effects and allocations across replacement of its reasoning processes.
_Avoid_: Thread, attempt, agent persona.

**Attempt**:
A bounded try by a reasoning process against admitted intent, with its own
provenance and workspace.
_Avoid_: ExecutionScope, worker identity.

**Episode**:
A bounded model and tool execution interval for a team role or an Attempt.
_Avoid_: Mission, durable job.

**Execution**:
An independently addressed supervised job owned by an ExecutionScope; its
creating role, Episode or Attempt remains provenance after controller replacement.
_Avoid_: Launcher PID, conversation.

**RoleEpoch**:
The authority version of a logical team seat, such as the lead, independent of
the authority version of the work it coordinates.
_Avoid_: ScopeEpoch, global agent generation.

**ScopeEpoch**:
The authority version of an ExecutionScope, used to distinguish current grants
from revoked grants without changing historical attribution.
_Avoid_: RoleEpoch, workspace generation.

**WorkspaceGeneration**:
The identity version of a writable workspace and its controller, independent
of job ownership and team roles.
_Avoid_: Runtime incarnation, ScopeEpoch.

**RuntimeIncarnation**:
One lifetime of an agent runtime service, separate from its persistent thread
store and from the authority of its callers.
_Avoid_: Thread identity, recovery epoch.

**Quarantine**:
A recorded unresolved state that retains evidence and conflicting capacity
until effects and authority are safely reconciled.
_Avoid_: Failed means stopped, refunded on timeout.
## Controller portfolio stewardship

**Corenius**:
The program-of-record repository for the persistent engineering-team control
plane, its domain language, admitted controller contracts and accepted evidence.
_Avoid_: DevX controller, Insula controller, repository-specific orchestration copy.

**Repository Stabilization Envelope**:
A read-only preservation inventory that covers every registered worktree and
durable effort before focused stabilization can change shared repository state.
_Avoid_: Cleanup campaign, Controller Portfolio, permission to retire work.

**Controller Portfolio**:
The Corenius efforts, branches, worktrees, runtime bindings and evidence that
develop or exercise persistent multi-agent coordination and control.
_Avoid_: All Corenius work, one controller implementation line.

**Controller Portfolio Stabilization**:
A two-pass program that first records the Controller Portfolio's complete
operational closure, then applies separately reviewed human-approved actions.
_Avoid_: Bulk landing, age-based cleanup, automatic branch retirement.

**Director**:
The external coordination role that sets portfolio priorities, reviews evidence
and grants shared-state actions without becoming the execution Lead.
_Avoid_: Lead, process supervisor, tmux operator as authority.

**Lead**:
A replaceable logical team role that autonomously coordinates admitted work,
workers, verification and evidence within the Director's authority boundary.
_Avoid_: Provider session, tmux pane, fixed machine process, human merge authority.

**Scout Lead**:
A Lead whose admitted objective is evidence-only inventory and proposal; it may
write dedicated stabilization records but cannot mutate the observed portfolio.
_Avoid_: Read-only process, pass-two implementation Lead, cleanup authority.

**Bootstrap Lead**:
A short-lived Lead authorized only to build and qualify the Scout Kit before any
portfolio observation begins.
_Avoid_: Scout Lead, portfolio implementer, authority to inspect or alter observed work.

**Scout Kit**:
The reviewed schemas, source manifest, read-only collectors, redaction pipeline
and bootstrap journal tooling required before the Scout Lead can inventory work.
_Avoid_: Controller product, pass-one evidence, ad-hoc shell command set.

**Bootstrap Mandate**:
An immutable, expiring assignment that binds a pre-I2 RoleEpoch to one worktree,
provider session, operation allowlist and monotonic directive sequence.
_Avoid_: I2 command, tmux text as authority, reusable session permission.

**Observer**:
An independent read-only role that samples progress and health and alerts the
Director without steering, killing, restarting or accepting work.
_Avoid_: Supervisor, verifier, Lead, watchdog with mutation authority.

**Stabilization Evidence Record**:
An immutable strict-YAML statement of schema-validated repository or operational
facts, including provenance, freshness and explicit unknowns.
_Avoid_: Recommendation, mutable dashboard row, prose summary.

**Evidence Redaction**:
The allowlist-first removal of secret-bearing or overbroad source data before any
evidence bytes reach persistent storage.
_Avoid_: Post-hoc log scrubbing, raw environment capture, secret-shaped masking only.

**Action Packet**:
A reviewed pass-two proposal bound to exact Stabilization Evidence Records, with
preconditions, conflicts, consequences and required human authority.
_Avoid_: Inventory observation, automatic operation, blanket cleanup approval.

**Observation Round**:
One bounded pass over declared inventory sources whose facts retain source
generations and observation times; two rounds are compared before promotion.
_Avoid_: Atomic global snapshot, repository freeze, freshness assumption.

**Mechanism Promotion**:
The evidence-gated admission of a proven I1 coordination mechanism into the I2
product contract without importing I1 branch history as a unit.
_Avoid_: Branch merge, code copying without provenance, I1 supersedes I2.

**Qualification Rung**:
One cumulative proof level in the controller acceptance ladder; passing an early
rung establishes only its declared scope and never replaces later proof.
_Avoid_: Milestone completion, readiness claim without scope, superseding test.

**Trial Repository**:
A disposable controller-owned Git and work-ledger scenario materialized from a
versioned deterministic template for qualification and fault testing.
_Avoid_: Mock repository, ad-hoc temporary checkout, human source repository.

**Trial Template**:
The immutable declarative definition of a Trial Repository's seed tree, history,
work items, acceptance contract, fault hooks and expected identities.
_Avoid_: Materialized Trial Repository, shell setup transcript, golden outcome.
