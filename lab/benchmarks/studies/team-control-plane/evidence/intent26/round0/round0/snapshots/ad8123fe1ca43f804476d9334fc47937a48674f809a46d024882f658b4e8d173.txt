# Corenius engineering team

Language for the persistent engineering team and its authority boundaries.
These definitions describe the domain; they do not claim implementation readiness.

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
