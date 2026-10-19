use std::collections::BTreeMap;

use crate::job::JobSpec;

#[derive(Clone, Debug, Default)]
pub struct ReadyQueue {
    tiers: BTreeMap<i32, Vec<JobSpec>>,
    last_priority: Option<i32>,
}

impl ReadyQueue {
    pub fn new() -> Self {
        Self {
            tiers: BTreeMap::new(),
            last_priority: None,
        }
    }

    pub fn push(&mut self, job: JobSpec) {
        let bucket = self.tiers.entry(job.priority).or_default();
        bucket.push(job);
        bucket.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn pop(&mut self) -> Option<JobSpec> {
        let priorities: Vec<i32> = self
            .tiers
            .iter()
            .rev()
            .filter(|(_, jobs)| !jobs.is_empty())
            .map(|(priority, _)| *priority)
            .collect();
        let mut chosen = *priorities.first()?;
        if priorities.len() > 1 && self.last_priority == Some(chosen) {
            chosen = priorities[1];
        }

        let (job, empty) = {
            let bucket = self.tiers.get_mut(&chosen).expect("chosen tier exists");
            let job = bucket.remove(0);
            (job, bucket.is_empty())
        };
        if empty {
            self.tiers.remove(&chosen);
        }
        self.last_priority = Some(chosen);
        Some(job)
    }

    pub fn drain_ids(&mut self) -> Vec<String> {
        let mut ids = Vec::new();
        while let Some(job) = self.pop() {
            ids.push(job.id);
        }
        ids
    }
}
