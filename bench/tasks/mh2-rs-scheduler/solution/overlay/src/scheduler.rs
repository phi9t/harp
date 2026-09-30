use std::collections::BTreeMap;

use crate::calendar::Calendar;
use crate::error::{SchedulerError, SchedulerResult};
use crate::graph::DependencyGraph;
use crate::queue::ReadyQueue;
use crate::report::ScheduleReport;
use crate::retry::BackoffPolicy;
use crate::JobSpec;

pub fn build_schedule(
    jobs: &[JobSpec],
    outcomes: &[(&str, Vec<bool>)],
) -> SchedulerResult<ScheduleReport> {
    let mut graph = DependencyGraph::from_jobs(jobs)?;
    let mut queue = ReadyQueue::new();
    for job in graph.initial_ready() {
        queue.push(job);
    }

    let mut order = Vec::new();
    while let Some(job) = queue.pop() {
        let id = job.id.clone();
        order.push(id.clone());
        for ready in graph.complete(&id)? {
            queue.push(ready);
        }
    }

    if order.len() != jobs.len() {
        return Err(SchedulerError::Cycle {
            members: graph.pending_ids(),
        });
    }

    let mut calendar = Calendar::new(0, 24 * 60);
    let placements = calendar.place_all(jobs, &order)?;
    let outcome_map: BTreeMap<&str, &[bool]> = outcomes
        .iter()
        .map(|(id, values)| (*id, values.as_slice()))
        .collect();
    let job_map: BTreeMap<&str, &JobSpec> = jobs.iter().map(|job| (job.id.as_str(), job)).collect();
    let mut attempts = Vec::new();

    for id in &order {
        let job = job_map.get(id.as_str()).expect("ordered job exists");
        let default_success = [true];
        let outcomes = outcome_map
            .get(id.as_str())
            .copied()
            .unwrap_or(default_success.as_slice());
        attempts.extend(BackoffPolicy::for_job(job).schedule(id, outcomes)?);
    }

    Ok(ScheduleReport {
        order,
        placements,
        attempts,
        jobs: jobs.to_vec(),
    })
}
