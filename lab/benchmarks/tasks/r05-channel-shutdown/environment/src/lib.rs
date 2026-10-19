use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    Work(i32),
    Fail(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolError {
    JobFailed(String),
    WorkerPanicked,
}

pub fn run_jobs(jobs: Vec<Job>) -> Result<Vec<i32>, PoolError> {
    let (work_tx, work_rx) = mpsc::channel::<Job>();
    let (result_tx, result_rx) = mpsc::channel::<i32>();
    let shared_rx = Arc::new(Mutex::new(work_rx));
    let mut handles = Vec::new();

    for _ in 0..2 {
        let rx = Arc::clone(&shared_rx);
        let tx = result_tx.clone();
        handles.push(thread::spawn(move || loop {
            let job = match rx.lock().expect("work queue poisoned").recv() {
                Ok(job) => job,
                Err(_) => break,
            };
            if let Job::Work(value) = job {
                let _ = tx.send(value * 2);
            }
        }));
    }
    drop(result_tx);

    for job in jobs {
        match job {
            Job::Work(value) => work_tx.send(Job::Work(value)).expect("workers stopped"),
            Job::Fail(message) => {
                for handle in handles {
                    if handle.join().is_err() {
                        return Err(PoolError::WorkerPanicked);
                    }
                }
                return Err(PoolError::JobFailed(message));
            }
        }
    }

    drop(work_tx);
    for handle in handles {
        if handle.join().is_err() {
            return Err(PoolError::WorkerPanicked);
        }
    }
    let mut results: Vec<i32> = result_rx.try_iter().collect();
    results.sort();
    Ok(results)
}
