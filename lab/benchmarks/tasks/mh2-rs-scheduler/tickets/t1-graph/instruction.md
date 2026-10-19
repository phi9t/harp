Implement deterministic dependency planning in `src/graph.rs`.

Required behavior:

- `DependencyGraph::from_jobs` must reject duplicate ids and dependencies that
  name an unknown job. Return `SchedulerError`, not strings or panics.
- `DependencyGraph::initial_ready` returns all jobs whose dependencies are
  already satisfied. Canonical order is priority descending, then id ascending.
- `DependencyGraph::complete(id)` marks a job complete and returns the newly
  ready jobs in the same canonical order.
- `DependencyGraph::pending_ids` returns unfinished ids sorted by id.
- `plan_jobs` performs a deterministic topological plan using the same ready
  job ordering. A cycle returns `SchedulerError::Cycle` with sorted members.

Also update the implemented component registry in `src/lib.rs` by adding
`graph`. Preserve the cross-cutting error display and canonical ordering rules
from the top-level instruction.
