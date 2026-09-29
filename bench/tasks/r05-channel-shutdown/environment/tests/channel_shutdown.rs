use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use r05_channel_shutdown::{run_jobs, Job, PoolError};

fn run_with_timeout(jobs: Vec<Job>, timeout: Duration) -> Result<Result<Vec<i32>, PoolError>, String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(run_jobs(jobs));
    });
    rx.recv_timeout(timeout)
        .map_err(|_| "worker pool did not shut down after the first error".to_string())
}

#[test]
fn runs_all_jobs_on_success() {
    let jobs = vec![Job::Work(3), Job::Work(1), Job::Work(2)];

    assert_eq!(run_jobs(jobs), Ok(vec![2, 4, 6]));
}

#[test]
fn returns_first_error_without_deadlocking() {
    let jobs = vec![
        Job::Work(1),
        Job::Fail("boom".to_string()),
        Job::Work(99),
    ];

    let result = run_with_timeout(jobs, Duration::from_millis(750)).expect("pool should return promptly");
    assert_eq!(result, Err(PoolError::JobFailed("boom".to_string())));
}
