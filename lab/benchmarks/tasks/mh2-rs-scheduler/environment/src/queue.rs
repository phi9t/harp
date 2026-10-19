use crate::job::JobSpec;

#[derive(Clone, Debug, Default)]
pub struct ReadyQueue {
    jobs: Vec<JobSpec>,
}

impl ReadyQueue {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn push(&mut self, job: JobSpec) {
        self.jobs.push(job);
    }

    pub fn pop(&mut self) -> Option<JobSpec> {
        self.jobs
            .sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.id.cmp(&b.id)));
        if self.jobs.is_empty() {
            None
        } else {
            Some(self.jobs.remove(0))
        }
    }

    pub fn drain_ids(&mut self) -> Vec<String> {
        let mut ids = Vec::new();
        while let Some(job) = self.pop() {
            ids.push(job.id);
        }
        ids
    }
}
