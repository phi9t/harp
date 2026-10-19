Implement retry/backoff planning in `src/retry.rs`.

Required behavior:

- `BackoffPolicy::schedule(job, outcomes)` returns one `AttemptEvent` per
  attempted run, starting at attempt 1 with elapsed time `0`.
- After each failed attempt, add the current backoff delay to the elapsed time
  for the next attempt. The next delay is multiplied by `factor` and capped at
  `max_delay_ms`. A `factor` below 1 behaves as 1.
- Stop and return the attempt rows at the first successful outcome.
- Missing outcomes count as failures, so exhausting `max_attempts` returns
  `SchedulerError::RetryBudgetExceeded`.
- `BackoffPolicy::for_job` uses the retry settings from `JobSpec` with factor 2
  and a 60 second cap.

Preserve the top-level error convention and keep retry delays in milliseconds,
not scheduler window minutes.
