use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SanitizationCategory {
    Pii,
    MachineIdentifiable,
    Proprietary,
    Secret,
}

impl std::fmt::Display for SanitizationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pii => write!(f, "PII"),
            Self::MachineIdentifiable => write!(f, "MachineIdentifiable"),
            Self::Proprietary => write!(f, "Proprietary"),
            Self::Secret => write!(f, "Secret"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SanitizationFinding {
    pub category: SanitizationCategory,
    pub rule_name: String,
    pub line_number: usize,
    pub start_col: usize,
    pub snippet: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileSanitizationReport {
    pub path: PathBuf,
    pub findings: Vec<SanitizationFinding>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SanitizationReport {
    pub scanned_files: usize,
    pub total_findings: usize,
    pub file_reports: Vec<FileSanitizationReport>,
    pub clean: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SanitizerConfig {
    pub custom_proprietary_terms: Vec<String>,
    pub allowlisted_patterns: Vec<String>,
    pub scrub_ips: bool,
    pub scrub_emails: bool,
    pub scrub_user_paths: bool,
    pub scrub_hostnames: bool,
    pub scrub_secrets: bool,
    pub scrub_proprietary: bool,
}

impl Default for SanitizerConfig {
    fn default() -> Self {
        Self {
            custom_proprietary_terms: vec![
                "bytedance".to_string(),
                "byted".to_string(),
                "lark".to_string(),
                "feishu".to_string(),
                "myriatron".to_string(),
                "qianchuan".to_string(),
            ],
            allowlisted_patterns: Vec::new(),
            scrub_ips: true,
            scrub_emails: true,
            scrub_user_paths: true,
            scrub_hostnames: true,
            scrub_secrets: true,
            scrub_proprietary: true,
        }
    }
}
