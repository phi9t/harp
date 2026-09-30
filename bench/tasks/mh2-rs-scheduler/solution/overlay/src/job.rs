#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobSpec {
    pub id: String,
    pub label: String,
    pub priority: i32,
    pub duration_min: u32,
    pub deps: Vec<String>,
    pub window_start: u32,
    pub window_end: u32,
    pub max_attempts: u32,
    pub retry_base_ms: u64,
}

impl JobSpec {
    pub fn new(id: &str, priority: i32, duration_min: u32) -> Self {
        Self {
            id: id.to_string(),
            label: id.to_string(),
            priority,
            duration_min,
            deps: Vec::new(),
            window_start: 0,
            window_end: 24 * 60,
            max_attempts: 1,
            retry_base_ms: 100,
        }
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn depends_on(mut self, deps: &[&str]) -> Self {
        self.deps = deps.iter().map(|dep| dep.to_string()).collect();
        self
    }

    pub fn window(mut self, start: u32, end: u32) -> Self {
        self.window_start = start;
        self.window_end = end;
        self
    }

    pub fn retry(mut self, max_attempts: u32, retry_base_ms: u64) -> Self {
        self.max_attempts = max_attempts;
        self.retry_base_ms = retry_base_ms;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: String,
    pub start_minute: u32,
    pub end_minute: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttemptEvent {
    pub id: String,
    pub attempt: u32,
    pub elapsed_ms: u64,
    pub success: bool,
}

pub fn format_minute(minute: u32) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}
