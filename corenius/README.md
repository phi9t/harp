# Retained runtime mechanisms

Harp uses the Python + SQLite runtime in
[`controller/durable/`](controller/durable/README.md). The supported coding
lifecycle is `harp durable`; see the [Harp runtime guide](../docs/durable-runtime.md).
No PostgreSQL or Temporal service is required.

Old Rust controller, attempt supervisor/gate and terminal-agent commands return
an actionable retirement error for every invocation. Their Cargo packages expose
only dependency-free refusal binaries; old libraries, autodiscovered executables,
examples and execution tests are not registered. Old Python controller/operator,
Collab, worktree/landing compatibility and lead/worker/tracker commands refuse
before opening state or starting processes. Legacy databases are not migrated.

Provider protocols, Git/check contracts, parsers, evidence utilities and historical
design/qualification records remain reference material. `controller/durable/`,
`tools/insula/capsule.py` and `tools/temp_root.py` remain active. Low-level durable
demos and capsule preparation are subordinate tooling; Harp owns the coding
workflow and human approval boundary. Benchmark reference/noop modes and pure
validation remain; direct provider/cmd and old-controller benchmark execution
are retired. Historical harness results do not qualify Harp workers.

Current intent lives in Git-managed `.scratch/<feature>/spec.md` and `issues/*.md`.
Old effort/Kata records and sibling-policy synchronization are historical, not
an additional backlog or dependency. Source-specific licenses and captured
evidence remain unchanged. `HARP_IMPORT.json` records adaptations, retired gates
and their recoverable source identities.

The retained gate (`tools/agents/check.sh --list`) checks refusal behavior, the
selected runtime, temporary paths and pure evidence/snapshot utilities.
Old `CHECK_SYNC`, `CHECK_TEAM_CONTROL_PLANE` and `CHECK_CONTROLLER_THREADS` flags
cannot restore retired authority. The root Harp changed-surface gate remains
the integration gate.
