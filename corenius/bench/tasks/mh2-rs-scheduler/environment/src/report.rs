use crate::job::{AttemptEvent, JobSpec, Placement};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleReport {
    pub order: Vec<String>,
    pub placements: Vec<Placement>,
    pub attempts: Vec<AttemptEvent>,
    pub jobs: Vec<JobSpec>,
}

pub fn render_report(_report: &ScheduleReport) -> String {
    "SCHEDULE v0\n".to_string()
}
