use crate::calendar::Calendar;
use crate::error::SchedulerResult;
use crate::graph::plan_jobs;
use crate::report::ScheduleReport;
use crate::retry::BackoffPolicy;
use crate::JobSpec;

pub fn build_schedule(
    jobs: &[JobSpec],
    outcomes: &[(&str, Vec<bool>)],
) -> SchedulerResult<ScheduleReport> {
    let order = plan_jobs(jobs)?;
    let mut calendar = Calendar::new(0, 24 * 60);
    let placements = calendar.place_all(jobs, &order)?;
    let mut attempts = Vec::new();
    for job in jobs {
        let outcome = outcomes
            .iter()
            .find(|(id, _)| *id == job.id)
            .map(|(_, values)| values.as_slice())
            .unwrap_or(&[true]);
        attempts.extend(BackoffPolicy::for_job(job).schedule(&job.id, outcome)?);
    }
    Ok(ScheduleReport {
        order,
        placements,
        attempts,
        jobs: jobs.to_vec(),
    })
}
