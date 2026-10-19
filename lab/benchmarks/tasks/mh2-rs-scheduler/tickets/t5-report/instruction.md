Wire the scheduler and report renderer in `src/scheduler.rs` and `src/report.rs`.

This ticket depends on contracts from the other tickets:

- `DependencyGraph::initial_ready` and `complete` expose ready jobs in canonical
  priority/id order and report cycles through `pending_ids`.
- `ReadyQueue::pop` applies the fairness rule while preserving id ordering
  inside each priority tier.
- `Calendar::place_all` accepts the scheduler's dispatch order and returns
  minute placements.
- `BackoffPolicy::for_job(...).schedule(...)` returns retry attempt rows with
  elapsed millisecond offsets.

Required behavior:

- `build_schedule` must dispatch only ready jobs, using `ReadyQueue` for fair
  selection within the current ready set. If the queue drains while jobs remain,
  return a sorted `SchedulerError::Cycle`.
- Missing outcome rows default to a single successful attempt.
- `render_report` must produce the exact stable sections described by the
  public tests: order, windows, retries and component registry. Report rows must
  follow the schedule order, not input order.

Do not edit `src/lib.rs` from this ticket; the component registry is maintained
by the graph and queue tickets.
