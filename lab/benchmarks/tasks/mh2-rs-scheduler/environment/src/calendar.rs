use std::collections::BTreeMap;

use crate::error::{SchedulerError, SchedulerResult};
use crate::job::{JobSpec, Placement};

#[derive(Clone, Debug)]
pub struct Calendar {
    day_start: u32,
    day_end: u32,
}

impl Calendar {
    pub fn new(day_start: u32, day_end: u32) -> Self {
        Self { day_start, day_end }
    }

    pub fn place_all(
        &mut self,
        jobs: &[JobSpec],
        order: &[String],
    ) -> SchedulerResult<Vec<Placement>> {
        let by_id: BTreeMap<String, JobSpec> = jobs
            .iter()
            .map(|job| (job.id.clone(), job.clone()))
            .collect();
        let mut placements = Vec::new();
        for id in order {
            let job = by_id
                .get(id)
                .ok_or_else(|| SchedulerError::UnknownJob { id: id.clone() })?;
            let start = self.day_start.max(job.window_start);
            placements.push(Placement {
                id: id.clone(),
                start_minute: start,
                end_minute: start + job.duration_min,
            });
        }
        Ok(placements)
    }
}
