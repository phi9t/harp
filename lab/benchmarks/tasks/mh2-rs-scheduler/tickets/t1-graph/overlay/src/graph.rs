use std::collections::{BTreeMap, BTreeSet};

use crate::error::{SchedulerError, SchedulerResult};
use crate::job::JobSpec;

#[derive(Clone, Debug)]
pub struct DependencyGraph {
    jobs: BTreeMap<String, JobSpec>,
    dependents: BTreeMap<String, Vec<String>>,
    remaining: BTreeMap<String, usize>,
    completed: BTreeSet<String>,
}

impl DependencyGraph {
    pub fn from_jobs(jobs: &[JobSpec]) -> SchedulerResult<Self> {
        let mut by_id = BTreeMap::new();
        for job in jobs {
            if by_id.insert(job.id.clone(), job.clone()).is_some() {
                return Err(SchedulerError::DuplicateJob { id: job.id.clone() });
            }
        }

        for job in by_id.values() {
            let mut deps = job.deps.clone();
            deps.sort();
            for dependency in deps {
                if !by_id.contains_key(&dependency) {
                    return Err(SchedulerError::UnknownDependency {
                        job: job.id.clone(),
                        dependency,
                    });
                }
            }
        }

        let mut dependents: BTreeMap<String, Vec<String>> =
            by_id.keys().map(|id| (id.clone(), Vec::new())).collect();
        let mut remaining = BTreeMap::new();
        for job in by_id.values() {
            remaining.insert(job.id.clone(), job.deps.len());
            for dep in &job.deps {
                dependents
                    .get_mut(dep)
                    .expect("dependency validated")
                    .push(job.id.clone());
            }
        }
        for ids in dependents.values_mut() {
            ids.sort_by(|a, b| {
                let left = by_id.get(a).expect("dependent exists");
                let right = by_id.get(b).expect("dependent exists");
                right
                    .priority
                    .cmp(&left.priority)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }

        Ok(Self {
            jobs: by_id,
            dependents,
            remaining,
            completed: BTreeSet::new(),
        })
    }

    pub fn initial_ready(&self) -> Vec<JobSpec> {
        let mut ready: Vec<JobSpec> = self
            .jobs
            .values()
            .filter(|job| self.remaining.get(&job.id).copied().unwrap_or(0) == 0)
            .filter(|job| !self.completed.contains(&job.id))
            .cloned()
            .collect();
        sort_jobs(&mut ready);
        ready
    }

    pub fn complete(&mut self, id: &str) -> SchedulerResult<Vec<JobSpec>> {
        if !self.jobs.contains_key(id) {
            return Err(SchedulerError::UnknownJob { id: id.to_string() });
        }
        if !self.completed.insert(id.to_string()) {
            return Ok(Vec::new());
        }

        let mut ready = Vec::new();
        for child in self.dependents.get(id).cloned().unwrap_or_default() {
            let entry = self
                .remaining
                .get_mut(&child)
                .expect("remaining child exists");
            if *entry > 0 {
                *entry -= 1;
            }
            if *entry == 0 && !self.completed.contains(&child) {
                ready.push(self.jobs.get(&child).expect("child job exists").clone());
            }
        }
        sort_jobs(&mut ready);
        Ok(ready)
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
    let mut graph = DependencyGraph::from_jobs(jobs)?;
    let mut ready = graph.initial_ready();
    let mut order = Vec::new();

    while !ready.is_empty() {
        sort_jobs(&mut ready);
        let job = ready.remove(0);
        let id = job.id.clone();
        order.push(id.clone());
        ready.extend(graph.complete(&id)?);
    }

    if order.len() != jobs.len() {
        return Err(SchedulerError::Cycle {
            members: graph.pending_ids(),
        });
    }

    Ok(order)
}

fn sort_jobs(jobs: &mut [JobSpec]) {
    jobs.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
}
