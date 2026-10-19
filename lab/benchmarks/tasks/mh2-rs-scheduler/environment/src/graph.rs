use std::collections::{BTreeMap, BTreeSet};

use crate::error::SchedulerResult;
use crate::job::JobSpec;

#[derive(Clone, Debug)]
pub struct DependencyGraph {
    jobs: BTreeMap<String, JobSpec>,
    completed: BTreeSet<String>,
}

impl DependencyGraph {
    pub fn from_jobs(jobs: &[JobSpec]) -> SchedulerResult<Self> {
        let mut by_id = BTreeMap::new();
        for job in jobs {
            by_id.insert(job.id.clone(), job.clone());
        }
        Ok(Self {
            jobs: by_id,
            completed: BTreeSet::new(),
        })
    }

    pub fn initial_ready(&self) -> Vec<JobSpec> {
        self.jobs.values().cloned().collect()
    }

    pub fn complete(&mut self, id: &str) -> SchedulerResult<Vec<JobSpec>> {
        self.completed.insert(id.to_string());
        Ok(Vec::new())
    }

    pub fn pending_ids(&self) -> Vec<String> {
        self.jobs
            .keys()
            .filter(|id| !self.completed.contains(*id))
            .cloned()
            .collect()
    }
}

pub fn plan_jobs(jobs: &[JobSpec]) -> SchedulerResult<Vec<String>> {
    let mut ids: Vec<String> = jobs.iter().map(|job| job.id.clone()).collect();
    ids.sort();
    Ok(ids)
}
