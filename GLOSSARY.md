# Harp agent engineering

Language for coding work and its reviewed delivery. Mathematical study terms
remain in their topic-specific educational glossaries.

## Language

**Task**:
A bounded coding outcome requested by the owner, with criteria for deciding
whether the result is useful and acceptable.

**Worker**:
An agent assigned to carry out a task in an isolated workspace.

**Candidate**:
An exact proposed repository revision produced for review and verification.

**Verification**:
Checks and their recorded results against a specific candidate and the task's
acceptance criteria.

**Landing**:
The owner-approved incorporation of a verified candidate into the destination
repository's primary branch.
_Avoid_: Treating worker completion or a successful check as landing approval.

**Attempt**:
One admitted execution of a task, whose identity survives observation,
cancellation and recovery. A fresh execution is a new attempt.

**Durable runtime**:
The single authority for admitted work and its execution, recovery, verification
and landing history.
_Avoid_: Using controller to imply a second owner of that same lifecycle.

**Approval**:
An explicit owner decision permitting a particular verified candidate to land.
_Avoid_: Inferring approval from an agent role, successful verification or task completion.

**Workflow**:
A recorded sequence of steps that carries a task toward an approved landing.

**Activity**:
A workflow step that performs an external effect, such as running a worker,
checking a candidate or landing an approved revision.

**Uncertain effect**:
An external action whose outcome is not established well enough to accept its
result or repeat it safely.

**Reconciliation**:
An explicit resolution of an uncertain effect using retained evidence and current
observations, separate from proving that the old execution has stopped.
