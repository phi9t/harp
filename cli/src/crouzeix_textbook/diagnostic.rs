use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TextbookDiagnostic {
    pub code: &'static str,
    pub identity: Option<String>,
    pub field: String,
    pub expected: String,
    pub observed: String,
    pub path: Option<PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl TextbookDiagnostic {
    pub(super) fn new(
        code: &'static str,
        identity: Option<String>,
        field: impl Into<String>,
        expected: impl Into<String>,
        observed: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        Self {
            code,
            identity,
            field: field.into(),
            expected: expected.into(),
            observed: observed.into(),
            path: Some(path),
            line: None,
            column: None,
        }
    }
}
