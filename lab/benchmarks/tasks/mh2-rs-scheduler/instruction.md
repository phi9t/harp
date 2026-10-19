Build the missing pieces of the `mh2-rs-scheduler` Rust crate. The crate models
a small job scheduler: dependency planning, fair ready-queue dispatch,
retry/backoff planning, time-window placement and a text report.

Cross-cutting invariants that every ticket must preserve:

- Canonical order is always deterministic. Job ids are the final tie-breaker,
  and any returned collection of jobs, cycle members, retry rows or report rows
  must not depend on hash-map iteration order.
- Public fallible APIs return `SchedulerResult<T>` and use `SchedulerError`.
  Do not return ad-hoc `String` errors or panic for scheduler validation
  failures. The display form is `scheduler:<kebab-kind>: <detail>`.
- Times in `JobSpec` windows and placements are minutes since midnight.
  Retry delays are milliseconds since the first attempt. Do not mix the units.
- Window ends are exclusive for searching but an item that ends exactly at the
  boundary is valid.
- The report is an integration boundary. It must render the graph order,
  calendar placements, retry attempts and implemented component registry in the
  same canonical order used by the scheduler.

The reference multi-run intentionally has one textual conflict: tickets
`t1-graph` and `t2-queue` both update the implemented component registry in
`src/lib.rs`. No other ticket should edit that file.
