# Retained imported checks

`tools/agents/check.sh --list` lists surviving bounded suites. `10-wt` now tests
no-effect retirement of Python/Rust commands and direct controller constructors.
Old worker/effort/landing/acceptance/tracker/lead/message/claims/Collab/controller/
operator suites are retired. Cargo packages build only refusal binaries.
`72-durable-execution` and `73-durable-qualification` retain the selected runtime.
Temporary-path, source-safety, evidence and review/snapshot checks remain.
`85-state-consistency` validates historical projections without querying live state.

`CHECK_BENCH=1` enables surviving benchmark/capsule tests; lead-driven authority
qualification is excluded. `CHECK_CARGO=0` skips only refusal Cargo targets.
`CHECK_SYNC`, `CHECK_TEAM_CONTROL_PLANE` and `CHECK_CONTROLLER_THREADS` cannot
restore old suites, synchronization or services. Unknown suites fail before effects.

Repeated `--suite` selects in registration order. `--timeout` bounds each suite
(default 1800 seconds); `--json PATH` retains results and partial interruption
summaries. Process-group cleanup is cooperative, not a replacement for the
coding runtime's guardian/cgroup boundary. The root Harp changed-surface gate
remains the integration gate.
