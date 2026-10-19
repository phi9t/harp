use std::collections::BTreeSet;
use std::fmt;

use harp_contracts::TaskId;

pub(crate) const MAX_DIAGNOSTICS: usize = 128;
const MAX_RETAINED_DIAGNOSTICS: usize = MAX_DIAGNOSTICS - 1;
const MAX_MESSAGE_BYTES: usize = 512;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidationDiagnostic {
    pub code: &'static str,
    pub task_id: Option<TaskId>,
    pub field: Option<&'static str>,
    pub message: String,
}

impl ValidationDiagnostic {
    pub(crate) fn new(
        code: &'static str,
        task_id: Option<&TaskId>,
        field: Option<&'static str>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            task_id: task_id.cloned(),
            field,
            message: truncate_message(message.into()),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Diagnostics {
    retained: BTreeSet<ValidationDiagnostic>,
    truncated: bool,
}

impl Diagnostics {
    pub(crate) fn push(&mut self, diagnostic: ValidationDiagnostic) {
        self.retained.insert(diagnostic);
        if self.retained.len() > MAX_RETAINED_DIAGNOSTICS {
            self.retained.pop_last();
            self.truncated = true;
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.retained.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError {
    diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationError {
    pub(crate) fn from_diagnostics(diagnostics: Diagnostics) -> Self {
        let mut retained = diagnostics.retained.into_iter().collect::<Vec<_>>();
        if diagnostics.truncated {
            retained.push(ValidationDiagnostic::new(
                "validation.too_many_diagnostics",
                None,
                None,
                "additional diagnostics omitted",
            ));
        }

        Self {
            diagnostics: retained,
        }
    }

    pub fn diagnostics(&self) -> &[ValidationDiagnostic] {
        &self.diagnostics
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "TaskGraph validation failed with {} diagnostic(s)",
            self.diagnostics.len()
        )?;
        if let Some(first) = self.diagnostics.first() {
            write!(formatter, ": {}: {}", first.code, first.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

fn truncate_message(mut message: String) -> String {
    if message.len() <= MAX_MESSAGE_BYTES {
        return message;
    }

    let mut boundary = MAX_MESSAGE_BYTES - 3;
    while !message.is_char_boundary(boundary) {
        boundary -= 1;
    }
    message.truncate(boundary);
    message.push_str("...");
    message
}
