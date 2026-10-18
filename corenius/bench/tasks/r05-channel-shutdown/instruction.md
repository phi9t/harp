`run_jobs(jobs)` executes work items on a small worker pool. A `Work(n)` item
produces `n * 2`; a `Fail(message)` item is the first error and must make
`run_jobs` return `Err(PoolError::JobFailed(message))`.

On the first error, the pool must stop accepting later jobs, close the work
queue so idle workers can exit, join every worker, and return promptly. The
success path must still return all doubled values sorted ascending. Fix
`src/lib.rs`. The verifier runs `cargo test --offline -- --test-threads=1` and
contains a timeout around the error case so a deadlock fails instead of hanging.
