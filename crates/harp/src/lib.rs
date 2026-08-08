mod corpus;
mod error;
mod fs;
mod json;
pub mod repository;
pub mod search;
pub mod sources;

use std::fs as std_fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

pub use error::AppError;

const DEFAULT_CORPUS_OUTPUT: &str = "atlas/src/content/generated/corpus.json";
const MAX_GENERATED_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct CorpusSummary {
    pub retained_concepts: usize,
    pub coverage_entries: usize,
    pub canonical_documents: usize,
    pub systems: usize,
    pub weng_sections: usize,
    pub diagnostic_fields: usize,
    pub diagnostic_rules: usize,
    pub diagnostic_cases: usize,
    pub lessons: usize,
    pub source_registry_rows: usize,
    pub evidence_edges: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildMode {
    Write,
    Check,
}

#[derive(Debug, Serialize)]
pub struct BuildResult {
    pub output: PathBuf,
    pub bytes: usize,
    pub matched: bool,
}

pub fn check_corpus(repo_root: &Path) -> Result<CorpusSummary, AppError> {
    let value = compile_corpus_value(repo_root)?;
    let diagnostics = &value["diagnostics"];
    Ok(CorpusSummary {
        retained_concepts: array_len(&value, "retained_concepts")?,
        coverage_entries: array_len(&value, "coverage")?,
        canonical_documents: array_len(&value, "documents")?,
        systems: array_len(&value, "systems")?,
        weng_sections: array_len(&value, "weng_sections")?,
        diagnostic_fields: diagnostics["worksheet"]["fields"]
            .as_array()
            .map(Vec::len)
            .ok_or_else(|| invalid_generated("diagnostics worksheet fields"))?,
        diagnostic_rules: diagnostics["rules"]["rules"]
            .as_array()
            .map(Vec::len)
            .ok_or_else(|| invalid_generated("diagnostic rules"))?,
        diagnostic_cases: diagnostics["cases"]
            .as_array()
            .map(Vec::len)
            .ok_or_else(|| invalid_generated("diagnostic cases"))?,
        lessons: array_len(&value, "lessons")?,
        source_registry_rows: tsv_data_rows(
            &repo_root.join("content/sources/source_registry.tsv"),
        )?,
        evidence_edges: tsv_data_rows(&repo_root.join("content/sources/evidence_graph.tsv"))?,
    })
}

pub fn build_corpus(
    repo_root: &Path,
    output: Option<&Path>,
    mode: BuildMode,
) -> Result<BuildResult, AppError> {
    let output = output.unwrap_or_else(|| Path::new(DEFAULT_CORPUS_OUTPUT));
    let output = corpus::validate_build_output(output)?;
    let value = compile_corpus_value(repo_root)?;
    let mut bytes = serde_json::to_vec_pretty(&value).map_err(|error| {
        AppError::external(
            "corpus.serialization",
            format!("could not serialize corpus: {error}"),
        )
    })?;
    bytes.push(b'\n');
    if bytes.len() > MAX_GENERATED_BYTES {
        return Err(AppError::invalid_input(
            "corpus.output_size",
            format!("corpus payload exceeds {MAX_GENERATED_BYTES} bytes"),
        ));
    }

    let repository = fs::HeldDirectory::open(repo_root, "Harp repository")?;
    let existing = repository.read_optional_regular_file_bounded(
        &output,
        "Harp generated corpus",
        MAX_GENERATED_BYTES,
    )?;
    let matched = existing.as_deref() == Some(bytes.as_slice());
    if mode == BuildMode::Check {
        if !matched {
            return Err(AppError::invalid_input(
                "corpus.stale",
                format!("{} does not match canonical inputs", output.display()),
            ));
        }
    } else if !matched {
        let expected = repository.read_optional_regular_file_snapshot_bounded(
            &output,
            "Harp generated corpus",
            MAX_GENERATED_BYTES,
        )?;
        repository.compare_and_replace_public_regular_file(
            &output,
            &bytes,
            expected.as_ref(),
            "Harp generated corpus",
        )?;
    }
    Ok(BuildResult {
        output,
        bytes: bytes.len(),
        matched,
    })
}

fn compile_corpus_value(repo_root: &Path) -> Result<Value, AppError> {
    let corpus = corpus::compile(repo_root)?;
    serde_json::to_value(corpus).map_err(|error| {
        AppError::external(
            "corpus.serialization",
            format!("could not serialize corpus: {error}"),
        )
    })
}

fn array_len(value: &Value, key: &str) -> Result<usize, AppError> {
    value[key]
        .as_array()
        .map(Vec::len)
        .ok_or_else(|| invalid_generated(key))
}

fn invalid_generated(field: &str) -> AppError {
    AppError::external(
        "corpus.generated_shape",
        format!("compiled corpus is missing {field}"),
    )
}

fn tsv_data_rows(path: &Path) -> Result<usize, AppError> {
    let text = std_fs::read_to_string(path)
        .map_err(|error| AppError::io("corpus.tsv", &path.display().to_string(), error))?;
    Ok(text.lines().skip(1).filter(|line| !line.is_empty()).count())
}
