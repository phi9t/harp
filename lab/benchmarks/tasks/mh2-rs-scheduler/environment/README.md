# mh2-rs-scheduler

This crate is a compact job-scheduler exercise for multi-agent benchmarking.
It has no dependencies outside the Rust standard library.

The intended domain model:

- `JobSpec` describes a job id, priority, duration, dependencies, a placement
  window and retry settings.
- `graph::plan_jobs` returns a deterministic dependency order. Ready jobs are
  ordered by priority descending, then id ascending. Cycles and missing
  dependencies are scheduler errors, not panics.
- `queue::ReadyQueue` is a priority queue with fairness across priority tiers.
  It should not starve lower-priority work when another ready tier exists.
- `retry::BackoffPolicy` returns attempt rows with elapsed millisecond offsets.
  Backoff is capped and stops at the first successful outcome.
- `calendar::Calendar` places jobs into minute-based windows without overlap.
  A job may end exactly at its window end.
- `scheduler::build_schedule` wires the modules together, and `report` renders
  the resulting schedule.

All public error display strings use `scheduler:<kebab-kind>: <detail>`. Keep
sets and lists in canonical order; the tests deliberately include cases that
would be flaky if a `HashMap` iteration order leaked into the API.
