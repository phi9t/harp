# Reference-quality workflow: design for owner review

Status: owner approved for specification; no implementation authorization.
Base inspected: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Route: `ask-matt` → `grill-with-docs` → reviewed tracker spec → blocker-linked tickets.

## Outcome and accepted constraints

A Codex or Claude lead can follow Corenius guidance to plan a durable effort,
dispatch bounded traecli workers, independently verify their output, prepare
a gated landing and resume from committed context in a fresh session.
Workspace-first adoption includes explicit host/configuration assumptions.

Accepted decisions are authoritative in:

- `decisions/20261001-reference-quality.md`
- `decisions/20261001-reference-authority-and-proof.md`
- `decisions/20261001-reference-evidence-and-guidance.md`

The owner approved this design and its remaining proposals. See
`decisions/20261001-reference-design-approved.md`. The specification defines
interfaces, fixtures, tests and ticket boundaries. "Proposed" below preserves
the wording reviewed by the owner; it does not imply those choices remain open.

## Alternatives and selected direction

Guidance alone would leave the gap between instructions and behavior untested.
A Rust-runtime-first expansion would add another orchestration system before
the existing practice is proven. The owner selected guidance plus a maintained
executable workflow based on the existing Python toolbox. Rust runtime work
remains available when a demonstrated requirement calls for it.

## Shared contract and lead adapters

The shared workflow owns intent, authority, ticket dependencies, writable
paths, budgets, execution state, acceptance checks, independent review and
handoff requirements. Both lead adapters point to that contract.

Each adapter describes harness entry, how a lead launches traecli workers,
how deterministic verdicts reach the lead, how independent review is invoked,
and how a fresh session loads the effort. Adapter differences reflect actual
tool capabilities, not different safety or completion standards. Facts about
installed harness behavior are verified locally during specification.

Deterministic supervision is required. An LLM relay is optional and receives
no decision authority. Human decisions are recorded once and reused within
their stated scope.

## Documentation and truthful state

The README becomes the entry point to the practice and executable example,
while retaining a clear route to the Rust prototype. The existing canonical
policy remains authoritative. Focused guides cover the workflow, execution
boundary, verification, recovery and the two adapters.

Each operational practice includes its trigger, action, completion criterion,
evidence and limits. Historical imports and experiments carry their source
commit and status. Durable decisions supersede older decisions explicitly;
current summaries point to the effective decision rather than accumulating
contradictory updates.

Reconcile the known benchmark arithmetic and feature-status conflicts.
Trace the disputed Milano boundary at pinned commits and relevant execution
paths before changing its reference claim. Do not treat the final transcript
narration as proof by itself.

## Representative improvement and demonstration

Proposed representative effort: harden worker lifecycle from dispatch to safe
cleanup. Address consistent structured launch contracts, rejection before
launch of conflicting path declarations, complete cancellation of owned
worker processes, and GC refusal for worktrees with active runs. Retain
independent verification of artifacts and exact-tip gate behavior.

Codex and Claude leads each perform this effort independently from the same
pinned base in separate worktrees, using equivalent tickets and acceptance
criteria. Their outputs are compared; only one reviewed implementation is
selected for integration. This prevents the second lead from merely observing
or reusing the first lead's solution.

Controlled failures exercise conflicting dispatch, defective output rejection,
worker cancellation and fresh-session recovery. Failure fixtures operate on
disposable owned resources. A fresh session receives the durable effort and
repository entry instructions, without the prior conversation. It must identify
effective decisions, completed checks, rejected outputs and the next action.

One end-to-end demonstration per lead is the initial usability milestone,
not statistical evidence of reliability or speed superiority. Deterministic
fixtures cover failure branches; repeated live performance evaluation remains
separate benchmark work. Report incomplete or blocked demonstrations as such.

## Evidence and budgets

Tracked evidence identifies source and resulting commits, contracts, capsule
identity, gate commands/results, rejection and recovery outcomes, reviewer
findings, artifact digests and verification limits. Separate measured facts,
declared configuration and inference. Digest integrity does not by itself
establish correctness or trustworthy provenance.

Proposed raw-evidence rule: retain raw events locally for at least 30 days
after the reviewed effort is accepted. Record the location and retention
deadline. Commit the minimal redacted evidence needed to substantiate accepted
claims, including relevant failure diagnostics; exclude credentials and private
prompt contents. Mark unavailable raw evidence and do not claim full replay
when it cannot be reproduced from retained material. Deletion remains an
explicitly authorized operation, not automatic cleanup.

Accepted launch limits are two concurrent workers, 30 minutes per worker and
three repair rounds. Proposed whole-demonstration limit: two initial workers
and at most six repair launches per lead, eight launches total. Cancelled and
failed starts that invoke the model count. Record all attempts and available
token usage; stop at the limit and escalate. Further launches or increases
require explicit authorization. Review-model calls receive a separate declared
budget in the implementation spec.

## Authority and review

Leads inspect, plan, launch within an authorized budget, cancel owned workers,
verify and prepare reviewed changes. Session authorization governs commit and
push. Current canonical policy requires human mainline integration. Resolve
any contradictory profile defaults through a versioned policy decision before
rollout; tool capability does not grant authority.

The lead reproduces acceptance checks per ticket. Separate-context review
checks the assembled branch. Security, concurrency and landing behavior also
receive independent adversarial verification. Supervisors establish monitored
execution compliance; acceptance establishes artifact behavior; the final gate
establishes the integrated tip passed its declared checks.

## Rollout and completion

Sequence: reconcile durable truth and contract → harden lifecycle behavior →
write guides/adapters against verified commands → run both demonstrations →
review and reconcile findings → present the reviewed result for integration.
Dependencies become blocker-linked tickets; disjoint work may run concurrently.

Policy edits bump file and manifest versions together and pass sync tests.
Sibling rollout, creating a remote, unrelated Rust runtime expansion, GPU
scheduling and performance claims are outside the first milestone.

Completion requires both demonstrations with retained evidence, verified failure
and recovery paths, independent review without unresolved blockers, current
guidance agreeing with behavior, and fresh required repository checks.
