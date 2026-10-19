use std::collections::BTreeMap;

use crate::implemented_components;
use crate::job::{format_minute, AttemptEvent, JobSpec, Placement};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleReport {
    pub order: Vec<String>,
    pub placements: Vec<Placement>,
    pub attempts: Vec<AttemptEvent>,
    pub jobs: Vec<JobSpec>,
}

pub fn render_report(report: &ScheduleReport) -> String {
    let jobs: BTreeMap<&str, &JobSpec> = report
        .jobs
        .iter()
        .map(|job| (job.id.as_str(), job))
        .collect();
    let placements: BTreeMap<&str, &Placement> = report
        .placements
        .iter()
        .map(|placement| (placement.id.as_str(), placement))
        .collect();
    let attempts_by_job = attempts_by_job(&report.attempts);

    let mut out = String::new();
    out.push_str("SCHEDULE v1\n");
    out.push_str("ORDER ");
    out.push_str(&report.order.join(" -> "));
    out.push('\n');

    out.push_str("WINDOWS\n");
    for id in &report.order {
        let job = jobs.get(id.as_str()).expect("report job exists");
        let placement = placements
            .get(id.as_str())
            .expect("report placement exists");
        let attempts = attempts_by_job
            .get(id.as_str())
            .map(|items| items.len())
            .unwrap_or(0);
        let ok = attempts_by_job
            .get(id.as_str())
            .and_then(|items| items.last())
            .map(|event| event.success)
            .unwrap_or(false);
        out.push_str(&format!(
            "{} {}-{} priority={} attempts={} {}\n",
            id,
            format_minute(placement.start_minute),
            format_minute(placement.end_minute),
            job.priority,
            attempts,
            if ok { "ok" } else { "failed" }
        ));
    }

    out.push_str("RETRIES\n");
    for id in &report.order {
        if let Some(events) = attempts_by_job.get(id.as_str()) {
            for event in events {
                out.push_str(&format!(
                    "{} #{} +{}ms {}\n",
                    id,
                    event.attempt,
                    event.elapsed_ms,
                    if event.success { "ok" } else { "failed" }
                ));
            }
        }
    }

    out.push_str("COMPONENTS ");
    let components = implemented_components();
    if components.is_empty() {
        out.push_str("none");
    } else {
        out.push_str(&components.join(", "));
    }
    out.push('\n');
    out
}

fn attempts_by_job(attempts: &[AttemptEvent]) -> BTreeMap<&str, Vec<&AttemptEvent>> {
    let mut grouped: BTreeMap<&str, Vec<&AttemptEvent>> = BTreeMap::new();
    for event in attempts {
        grouped.entry(event.id.as_str()).or_default().push(event);
    }
    for events in grouped.values_mut() {
        events.sort_by_key(|event| event.attempt);
    }
    grouped
}
