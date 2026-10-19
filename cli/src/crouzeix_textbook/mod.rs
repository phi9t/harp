mod contract;
mod diagnostic;
mod evidence;
mod lean;
mod markdown;
mod publish;

use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_path_to_error::{Path as ErrorPath, Segment};

pub use contract::{
    ExerciseRow, FormalMode, LeanCorrespondenceStatus, LeanDeclaration, LeanSolution, LeanStarter,
    ProseProofStatus, PublicationStatus, ReviewStatus, TextbookContracts, TheoremRow,
};
pub use diagnostic::TextbookDiagnostic;
pub use lean::{check_lean_receipt, generate_correspondence};
pub use publish::{publish_crouzeix_textbook, TextbookPublishMode, TextbookPublishResult};

use crate::fs::HeldDirectory;

use contract::{RawContracts, RawCoverage, RawExercises, COVERAGE_PATH, EXERCISES_PATH};

const INPUT_CODE: &str = "crouzeix-textbook.contract.input";
const DESERIALIZE_CODE: &str = "crouzeix-textbook.contract.deserialize";
// The v2 contracts are authored metadata, not bulk evidence. This limit bounds
// allocation while leaving ample room for the complete 35-chapter book.
const MAX_CONTRACT_BYTES: usize = 1024 * 1024;

pub(crate) struct ValidatedTextbookDocument {
    pub(crate) canonical_id: String,
    pub(crate) legacy_alias: String,
    pub(crate) canonical_path: String,
    pub(crate) markdown: String,
}

pub(crate) fn validated_registration(
    repository: &HeldDirectory,
) -> Result<Vec<ValidatedTextbookDocument>, Vec<TextbookDiagnostic>> {
    markdown::validated_registration(repository)
}

pub fn check(repo_root: &Path) -> Result<TextbookContracts, Vec<TextbookDiagnostic>> {
    let repository = HeldDirectory::open(repo_root, "Crouzeix textbook repository")
        .map_err(|error| vec![input_diagnostic(None, error.message)])?;
    let (contracts, mut diagnostics) = check_contracts(&repository)?;
    if let Err(mut errors) = markdown::validate(&repository, Some(&contracts)) {
        diagnostics.append(&mut errors);
    }
    if diagnostics.is_empty() {
        Ok(contracts)
    } else {
        Err(diagnostics)
    }
}

pub fn check_packet(repo_root: &Path) -> Result<(), Vec<TextbookDiagnostic>> {
    let repository = HeldDirectory::open(repo_root, "Crouzeix textbook repository")
        .map_err(|error| vec![input_diagnostic(None, error.message)])?;
    markdown::validate(&repository, None)
}

fn check_contracts(
    repository: &HeldDirectory,
) -> Result<(TextbookContracts, Vec<TextbookDiagnostic>), Vec<TextbookDiagnostic>> {
    let coverage = read_coverage(repository);
    let exercises = read_exercises(repository);
    match (coverage, exercises) {
        (Ok(coverage), Ok(exercises)) => Ok(contract::validate(RawContracts {
            coverage,
            exercises,
        })),
        (coverage, exercises) => {
            let mut diagnostics = Vec::new();
            if let Err(mut errors) = coverage {
                diagnostics.append(&mut errors);
            }
            if let Err(mut errors) = exercises {
                diagnostics.append(&mut errors);
            }
            Err(diagnostics)
        }
    }
}

fn read_coverage(repository: &HeldDirectory) -> Result<RawCoverage, Vec<TextbookDiagnostic>> {
    let bytes = read_contract_bytes(repository, COVERAGE_PATH, "textbook theorem contract")?;
    deserialize_contract(&bytes, "items", "item_id", COVERAGE_PATH)
}

fn read_exercises(repository: &HeldDirectory) -> Result<RawExercises, Vec<TextbookDiagnostic>> {
    let bytes = read_contract_bytes(repository, EXERCISES_PATH, "textbook exercise contract")?;
    deserialize_contract(&bytes, "exercises", "exercise_id", EXERCISES_PATH)
}

fn read_contract_bytes(
    repository: &HeldDirectory,
    relative_path: &'static str,
    label: &'static str,
) -> Result<Vec<u8>, Vec<TextbookDiagnostic>> {
    repository
        .read_optional_regular_file_bounded(Path::new(relative_path), label, MAX_CONTRACT_BYTES)
        .map_err(|error| {
            vec![input_diagnostic(
                Some(PathBuf::from(relative_path)),
                error.message,
            )]
        })?
        .ok_or_else(|| {
            vec![input_diagnostic(
                Some(PathBuf::from(relative_path)),
                "missing",
            )]
        })
}

fn input_diagnostic(path: Option<PathBuf>, observed: impl Into<String>) -> TextbookDiagnostic {
    TextbookDiagnostic {
        code: INPUT_CODE,
        identity: None,
        field: "file".to_owned(),
        expected: format!("regular JSON file no larger than {MAX_CONTRACT_BYTES} bytes"),
        observed: observed.into(),
        path,
        line: None,
        column: None,
    }
}

fn deserialize_contract<T: DeserializeOwned>(
    bytes: &[u8],
    collection_field: &'static str,
    identity_field: &'static str,
    relative_path: &'static str,
) -> Result<T, Vec<TextbookDiagnostic>> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    match serde_path_to_error::deserialize(&mut deserializer) {
        Ok(contract) => deserializer
            .end()
            .map(|()| contract)
            .map_err(|error| vec![deserialize_diagnostic(None, None, relative_path, &error)]),
        Err(error) => {
            let identity = identity_at_path(bytes, error.path(), collection_field, identity_field);
            let field_path = field_at_path(error.path(), collection_field);
            let error = error.into_inner();
            Err(vec![deserialize_diagnostic(
                field_path,
                identity,
                relative_path,
                &error,
            )])
        }
    }
}

fn deserialize_diagnostic(
    field_path: Option<String>,
    identity: Option<String>,
    relative_path: &'static str,
    error: &serde_json::Error,
) -> TextbookDiagnostic {
    let message = error.to_string();
    let (field, expected, observed) = if let Some(field) = quoted_after(&message, "unknown field `")
    {
        (
            qualified_field(field_path.as_deref(), field),
            "known field".to_owned(),
            "unknown field".to_owned(),
        )
    } else if let Some(field) = quoted_after(&message, "missing field `") {
        (
            qualified_field(field_path.as_deref(), field),
            "present".to_owned(),
            "missing field".to_owned(),
        )
    } else if let Some(value) = quoted_after(&message, "unknown variant `") {
        (
            field_path.unwrap_or_else(|| "$".to_owned()),
            expected_after(&message).unwrap_or_else(|| "supported value".to_owned()),
            value,
        )
    } else {
        (
            "$".to_owned(),
            "valid version-two contract".to_owned(),
            message.clone(),
        )
    };
    TextbookDiagnostic {
        code: DESERIALIZE_CODE,
        identity,
        field,
        expected,
        observed,
        path: Some(PathBuf::from(relative_path)),
        line: u32::try_from(error.line()).ok().filter(|line| *line > 0),
        column: u32::try_from(error.column())
            .ok()
            .filter(|column| *column > 0),
    }
}

fn qualified_field(parent: Option<&str>, field: String) -> String {
    match parent {
        Some(parent) if parent.rsplit('.').next() != Some(field.as_str()) => {
            format!("{parent}.{field}")
        }
        Some(parent) => parent.to_owned(),
        None => field,
    }
}

fn identity_at_path(
    bytes: &[u8],
    path: &ErrorPath,
    collection_field: &str,
    identity_field: &str,
) -> Option<String> {
    let index = collection_index(path, collection_field)?;
    serde_json::from_slice::<Value>(bytes)
        .ok()?
        .get(collection_field)?
        .get(index)?
        .get(identity_field)?
        .as_str()
        .map(str::to_owned)
}

fn collection_index(path: &ErrorPath, collection_field: &str) -> Option<usize> {
    let mut segments = path.iter();
    match (segments.next(), segments.next()) {
        (Some(Segment::Map { key }), Some(Segment::Seq { index })) if key == collection_field => {
            Some(*index)
        }
        _ => None,
    }
}

fn field_at_path(path: &ErrorPath, collection_field: &str) -> Option<String> {
    let mut segments = path.iter();
    if !matches!(segments.next(), Some(Segment::Map { key }) if key == collection_field)
        || !matches!(segments.next(), Some(Segment::Seq { .. }))
    {
        return None;
    }

    let mut field = String::new();
    for segment in segments {
        match segment {
            Segment::Map { key } => push_field_name(&mut field, key),
            Segment::Enum { variant } => push_field_name(&mut field, variant),
            Segment::Seq { index } => {
                field.push('[');
                field.push_str(&index.to_string());
                field.push(']');
            }
            Segment::Unknown => push_field_name(&mut field, "?"),
        }
    }
    (!field.is_empty()).then_some(field)
}

fn push_field_name(path: &mut String, name: &str) {
    if !path.is_empty() {
        path.push('.');
    }
    path.push_str(name);
}

fn quoted_after(message: &str, prefix: &str) -> Option<String> {
    let tail = message
        .strip_prefix(prefix)
        .or_else(|| message.split_once(prefix).map(|(_, tail)| tail))?;
    Some(tail.split('`').next()?.to_owned())
}

fn expected_after(message: &str) -> Option<String> {
    let (_, tail) = message.split_once(", expected ")?;
    Some(
        tail.rsplit_once(" at line ")
            .map_or(tail, |(value, _)| value)
            .to_owned(),
    )
}
