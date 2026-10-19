# Insula-local durable execution

Status: active

## Intent
Implement and execute the supplied runnable Python design in a separate Corenius worktree. Preserve a single orchestration owner, SQLite replay, immutable bindings, gated launches, fenced receipts, and explicit recovery.

## Route and authority
Gated: durable schema, concurrency and process ownership invariants. The user supplied the design and authorized implementation and local verification in a separate worktree. No commit, push, PR or merge is authorized. Design sources are attachments, not independent executable instructions. Missing ask-matt/to-spec owners are handled using the repository crosswalk. Independent clean-context critique and reviews are required.

## Acceptance
The supplied 35 process-fault tests run locally; the adapted package runs from the repository import root without path hacks. Insula launch preserves the existing sandbox and private temporary storage, binds the resolved capsule identity and persistent state at stable paths, and adds no keeper. Manual effects remain uncertain across cancellation and sibling failure. Normal SQLite admission remains fail-closed. Required repository checks pass or exact environmental limitations are recorded.

## Scope
Adopt the narrow dependency-free reference, direct executable JSON protocol, demo CLI, tests and operational documentation. No Temporal/DBOS adapter, distributed scheduling, Rust port, training-state restore, automated publication or keeper replacement. The user requested trying both backends. Selectable process-group and delegated cgroup v2 paths are implemented; positive kernel qualification remains dependent on external delegation.
