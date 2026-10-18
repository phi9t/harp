use std::collections::BTreeMap;

use crate::error::{SchedulerError, SchedulerResult};
use crate::job::{JobSpec, Placement};

#[derive(Clone, Debug)]
pub struct Calendar {
    day_start: u32,
    day_end: u32,
    occupied: Vec<Placement>,
}

impl Calendar {
    pub fn new(day_start: u32, day_end: u32) -> Self {
        Self {
            day_start,
            day_end,
            occupied: Vec::new(),
        }
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
            validate_window(job)?;

            let earliest = self.day_start.max(job.window_start);
            let latest = self.day_end.min(job.window_end);
            let start = self
                .first_open_slot(earliest, latest, job.duration_min)
                .ok_or_else(|| SchedulerError::NoWindow { job: id.clone() })?;
            let placement = Placement {
                id: id.clone(),
                start_minute: start,
                end_minute: start + job.duration_min,
            };
            self.occupied.push(placement.clone());
            self.occupied.sort_by(|a, b| {
                a.start_minute
                    .cmp(&b.start_minute)
                    .then_with(|| a.id.cmp(&b.id))
            });
            placements.push(placement);
        }

        Ok(placements)
    }

    fn first_open_slot(&self, earliest: u32, latest: u32, duration: u32) -> Option<u32> {
        let mut cursor = earliest;
        while cursor.saturating_add(duration) <= latest {
            let mut moved = false;
            for placement in &self.occupied {
                if overlaps(
                    cursor,
                    cursor + duration,
                    placement.start_minute,
                    placement.end_minute,
                ) {
                    cursor = placement.end_minute;
                    moved = true;
                    break;
                }
            }
            if !moved {
                return Some(cursor);
            }
        }
        None
    }
}

fn validate_window(job: &JobSpec) -> SchedulerResult<()> {
    if job.duration_min == 0 || job.window_start >= job.window_end {
        return Err(SchedulerError::InvalidWindow {
            job: job.id.clone(),
            start: job.window_start,
            end: job.window_end,
        });
    }
    Ok(())
}

fn overlaps(left_start: u32, left_end: u32, right_start: u32, right_end: u32) -> bool {
    left_start < right_end && right_start < left_end
}
