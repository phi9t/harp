use std::io;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn invalid_input(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn external(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn io(code: &'static str, context: &str, error: io::Error) -> Self {
        Self {
            code,
            message: format!("{context}: {error}"),
        }
    }
}
