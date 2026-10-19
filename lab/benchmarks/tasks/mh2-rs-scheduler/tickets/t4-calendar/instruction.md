Implement time-window placement in `src/calendar.rs`.

This ticket depends on the graph ticket's contract: callers pass the canonical
job id order produced by `graph::plan_jobs` or by scheduler dispatch. Preserve
that order when assigning jobs, but never allow overlapping placements.

Required behavior:

- `Calendar::new(day_start, day_end)` stores the allowed day window in minutes.
- `place_all(jobs, order)` looks up each id, validates that the job has positive
  duration and a start strictly before end, then chooses the earliest
  non-overlapping slot inside both the day window and the job window.
- A placement ending exactly at `window_end` or `day_end` is valid.
- Return `SchedulerError::UnknownJob`, `InvalidWindow` or `NoWindow` as
  appropriate. Do not panic or return string errors.

The hidden edge case is the exact boundary: do not reject a job just because
`start + duration == window_end`.
