# The consolidated runtime

Harp has one supported coding lifecycle:
`harp durable`. Harp’s [integration layer](../durable/cli.py) uses the
[Python + SQLite engine](../durable/runtime/engine.py), extended with
task admission, Codex/Trae activities, candidate capture, verification and local
landing. There is no Temporal server, PostgreSQL dependency or second controller.
The [ownership ADR](adr/0001-lightweight-durable-runtime-ownership.md) supersedes
earlier controller selections; the [integration specification](../.scratch/harp-runtime-integration/spec.md)
defines the combined contract. [Durable coding work](durable-runtime.md) gives
operator commands.

## Intent and operational state

Humans own intent and acceptance in Git-managed Markdown and committed acceptance
JSON. [Admission](../durable/jobs/coding/admission.py) freezes their revisions and bytes,
source base, workflow version, execution contract and selected provider. Reusing
a run ID with identical inputs deduplicates admission; conflicting reuse fails.
SQLite stores operational history, operations, attempts and accepted outcomes;
files retain inputs, logs, artifacts and receipts. Neither the database nor a
provider conversation becomes another editable backlog.

The engine re-enters a workflow from the beginning, returning stored results for
completed calls. It checks operation order, keys, kinds and specifications rather
than restoring a Python stack. Timers keep their deadlines and keyed signals are
one-shot. External effects belong in activities. The
[coding workflow](../durable/jobs/coding/workflow.py) is deliberately small: one managed
worker activity with manual recovery. Replay preserves accepted work; it does
not automatically resume a provider session.

## Activities, containment and recovery

The engine persists attempt claim and release intent before execution.
A [guardian](../durable/runtime/guardian.py) controls launch, timeout,
cancellation and descendant cleanup, retaining the controller lock until cleanup
finishes. It publishes a durable completion receipt. The engine validates that
receipt against the current attempt before accepting a result; a worker-written
result file alone grants no authority. After controller death, an existing valid
receipt can be accepted without repeating the activity.

[Insula binding](../durable/runtime/execution.py) and the
[capsule mechanism](../insula/capsule.py) project pinned runtime
code, immutable inputs and a private writable attempt directory into a Linux
namespace. Controller state and guardian receipts remain outside it. Integrated
coding uses [delegated cgroup v2 ownership](../insula/cgroup.py):
membership is established before payload release, controls stay outside the
payload, and kernel-observed cleanup precedes capacity reuse. There is no silent
fallback to the weaker trusted-process-group demo backend.

Capacity is project-state admission accounting, configurable from one through
four. One controller holds the state lock; guardians prevent takeover while
cleanup continues. Unproved cleanup blocks further dispatch. CPU/GPU slots do
not establish CPU quotas, GPU memory isolation or fleet scheduling.

Cancellation stops owned processes; it does not undo filesystem or external
effects. A started manual-recovery activity with an ambiguous outcome remains
`UNCERTAIN`, including after cancellation or sibling failure. Cleanup
acknowledgment and effect resolution are separate evidence-bearing operator
decisions, and neither launches work. Only an explicitly retry-safe activity or
destination deduplication justifies automatic repetition. The runtime makes no
arbitrary exactly-once external-effect guarantee.

## Provider proposals and independent checks

The [worker](../durable/jobs/coding/worker.py) clones the admitted source bundle into its
own workspace. [Codex](../durable/jobs/coding/providers/codex.py) and
[Trae](../durable/jobs/coding/providers/trae.py) are fresh, explicit adapters behind that
same activity contract. They use private homes and only the selected credential,
ignore host configuration/rules and disable plugins and daemon startup. Codex
uses its packaged code-mode host with apps disabled. Their inner sandbox bypass
runs inside the admitted Insula boundary; it grants no controller or landing
authority. Provider exit and bounded final-output evidence produce a proposal,
not accepted source changes.

[Capture](../durable/jobs/coding/candidates.py) validates the guardian-accepted proposal,
copies and hashes its Git bundle into retained immutable inputs, and checks base,
commit, tree and ancestry independently of the mutable worker repository.
Verification is a separate durable run against that captured subject and frozen
acceptance commands. The [verifier](../durable/jobs/coding/verification.py) materializes a
fresh checkout and runs checks offline in nested namespaces. Checks can write
their candidate workspace but cannot access the parent’s proof, logs, inputs or
credentials. Tracked bytes are compared directly with Git blobs before and after
checks. The proof binds the exact subject, checks, execution identity and logs.
Passing commands establish only their configured acceptance claims.

## Owner approval and physical landing

[Landing](../durable/jobs/coding/landing.py) requires a separately registered private
owner capability outside state, source, rootfs and projected inputs. The human
reviews the concrete candidate and verification evidence, then approves that
exact verified subject and destination base. Worker roles, successful checks and
provider messages cannot manufacture approval.

Landing serializes Git mutation and synchronizes the destination branch, index
and tracked worktree. Dirty, substituted or advanced destinations are refused
and preserved. An advanced base needs a fresh admitted integration candidate,
checks and approval. Durable landing intent and exact Git identities allow an
interrupted mutation to be reconciled before another mutation; moving a ref alone
is not a successful physical landing. Canonical blob bytes are required;
unqualified checkout transformations are refused.

## Lifetime and qualification limits

The operator explicitly prepares, starts and stops a project runtime; there is
no installed always-on service. SQLite and evidence persist between invocations.
`stop` drains the identified runtime instance rather than signaling a replacement
through a reused PID. [Preparation](../durable/runtime/setup.py) pins runtime code,
native rootfs contents and provider executable bytes, requiring private local
state, supported SQLite and an explicit writable delegation.

That OS delegation must survive controller restarts. When it ends, the old
execution record cannot dispatch new work, though retained status and evidence
remain inspectable. Fresh work requires a newly prepared execution record with
fresh explicit delegation. Reusing private project state first requires no active
attempts, unresolved cleanup blocks or held controller lock. Old running or
blocked work retains its original contract and needs reconciliation under it;
a new profile never rebinds old attempts. Changed runtime/rootfs/provider profiles
also require a new execution record. Inspect and approve old results with their
old record. Host reboot recovery and automatic capability replacement are not
promised.

The [integration guide](corenius-integration.md) records real single-worker
Codex and Trae runs through independent checks and approved physical landings,
and a real overlapping mixed pair with stale-base refusal and fresh integration.
The [four-worker milestone](../.scratch/harp-runtime-integration/issues/14-qualify-four-workers.md)
qualified two real Codex and two real Trae workers on Harp changes, including
independent verification, reviewed exact approvals and physical landings. The
composed gate and 102 native lifecycle tests passed. These results qualify the
recorded Linux environment and pinned provider profiles.
Configuring capacity four, passing deterministic structural tests or skipping
unavailable cgroup tests does not qualify four real provider workers.

The [component disposition](corenius-integration.md) retains pure protocol/check
contracts, parsers, snapshot helpers and offline reference material. Old Rust
controller/supervisor/gate/terminal-agent, Collab, lead/worker, tracker and legacy
worktree/landing launchers refuse execution; historical databases are not
silently migrated. Low-level durable demos and capsule tools are subordinate
mechanisms. Earlier [Rust task-graph ADR](adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md)
semantics and historical design records explain retained ideas, not a supported
parallel executor or a requirement for feature parity with every experiment.

## Workspace estate

[Estate](../durable/estate/workspaces.py) owns repository identities and workspace
resources in the same SQLite journal. The runtime reserves an attempt-bound lease
before releasing a worker; coding and command jobs materialize independent bundle
clones through estate's payload helper. The worker cannot access the journal.

Workspace handles bind repository, base, run, attempt, epoch, host identity and
managed location. Directory identities fence replacements. Reclamation requires
runtime cleanup proof and the controller lock, plus a clean Git checkout and an
independently retained bundle. Recorded allocation and cleanup intents can resume
after interruption. Unknown ownership and dirty or unretained work are preserved.
Estate does not schedule attempts, verify candidates or authorize landing.
