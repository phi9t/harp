use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use walkdir::WalkDir;

use crate::error::AppError;

mod crouzeix;

const IMPLEMENTATION_MANIFEST: &str = "evidence/implementations/manifest.tsv";
const BENCHMARK_MANIFEST: &str = "evidence/benchmarks/manifest.tsv";
const META_HARNESS_ROOT: &str = "evidence/meta_harness";
const AGENTIC_ENGINEERING_ROOT: &str = "evidence/agentic_engineering";
const META_HARNESS_ARCHIVE_MAX_BYTES: u64 = 16 * 1024 * 1024;
const META_HARNESS_DECOMPRESSED_MAX_BYTES: u64 = 64 * 1024 * 1024;
const META_HARNESS_ARCHIVE_MAX_MEMBERS: usize = 256;

#[derive(Debug, Serialize)]
pub struct SourcesReport {
    pub evidence_artifacts: usize,
    pub snapshot_files: usize,
    pub binary_objects: usize,
    pub implementation_sources: usize,
    pub local_locators: usize,
    pub crouzeix_source_receipts: usize,
    pub crouzeix_verification_receipts: usize,
}

#[derive(Debug, Serialize)]
pub struct MaterializeReport {
    pub sources: Vec<MaterializedSource>,
}

#[derive(Debug, Serialize)]
pub struct MaterializedSource {
    pub source_id: String,
    pub revision: String,
    pub path: String,
    pub snapshot_files: usize,
}

#[derive(Clone, Debug)]
struct SnapshotRow {
    source_id: String,
    remote: String,
    revision: String,
    snapshot_path: PathBuf,
    bytes: u64,
    sha256: String,
}

#[derive(Debug)]
struct SourceDefinition {
    source_id: String,
    directory: String,
    remote: String,
    revision: String,
    rows: Vec<SnapshotRow>,
}

#[derive(Debug)]
struct MetaHarnessReport {
    site_artifacts: usize,
    normalized_artifacts: usize,
    raw_archive_members: usize,
    raw_archive_bytes: u64,
    expected_digests: BTreeMap<PathBuf, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetaHarnessRunReceipt {
    schema_version: String,
    source_id: String,
    repository_source_id: String,
    repository_revision: String,
    model: String,
    candidate_count: usize,
    valid_candidate_count: usize,
    first_schema_valid_attempt: usize,
    first_model_reaching_attempt: usize,
    command_returncode: i32,
    proposal_schema_validated: bool,
    candidate_interfaces_assessed: bool,
    candidate_interfaces_passed: bool,
    invalid_candidates_retained: bool,
    workspace_boundary_validated: bool,
    benchmark_invoked: bool,
    held_out_test_invoked: bool,
    paid_model_evaluation_reproduced: bool,
    benchmark_score_reproduced: bool,
    secret_scan: String,
    raw_archive_sha256: String,
    raw_archive_bytes: u64,
    limitations: Vec<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct TraeCandidate {
    name: String,
    path: String,
    axis: String,
    hypothesis: String,
    #[serde(default)]
    components: Option<Vec<String>>,
    #[serde(default)]
    base_system: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraeFinalResponse {
    schema_version: String,
    candidates: Vec<TraeCandidate>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraePendingEval {
    candidates: Vec<TraeCandidate>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraeValidationReport {
    schema_version: String,
    candidate_count: usize,
    valid_candidate_count: usize,
    candidates: Vec<TraeCandidateValidation>,
    interface_checks_passed: bool,
    benchmark_invoked: bool,
    held_out_test_invoked: bool,
    workspace_boundary_passed: bool,
    workspace_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraeCandidateValidation {
    name: String,
    path: String,
    sha256: String,
    valid: bool,
    error: Option<String>,
}

pub fn verify(repo_root: &Path) -> Result<SourcesReport, AppError> {
    let mut expected_digests = BTreeMap::new();
    let crouzeix = crouzeix::verify(repo_root)?;
    let weng = verify_artifact_inventory(
        repo_root,
        Path::new("evidence/weng/artifact_inventory.tsv"),
        0,
        2,
        3,
        &mut expected_digests,
    )?;
    let rlm = verify_artifact_inventory(
        repo_root,
        Path::new("evidence/rlm/artifact_inventory.tsv"),
        0,
        1,
        2,
        &mut expected_digests,
    )?;
    let darwinx = verify_artifact_inventory(
        repo_root,
        Path::new("evidence/darwinx/artifact_inventory.tsv"),
        0,
        1,
        2,
        &mut expected_digests,
    )?;
    let self_improving_agents_survey = verify_artifact_inventory(
        repo_root,
        Path::new("evidence/self_improving_agents_survey/artifact_inventory.tsv"),
        0,
        2,
        3,
        &mut expected_digests,
    )?;
    let sicp = verify_sicp_manifest(repo_root, &mut expected_digests)?;
    let benchmarks = verify_benchmark_manifest(repo_root)?;
    let agentic_engineering = verify_relative_artifact_manifest(
        repo_root,
        Path::new(AGENTIC_ENGINEERING_ROOT),
        Path::new("evidence/agentic_engineering/manifest.tsv"),
        &mut expected_digests,
    )?
    .len()
    .saturating_sub(1);
    let meta_harness = verify_meta_harness_bundle(repo_root)?;
    let _verified_raw_archive = (
        meta_harness.raw_archive_members,
        meta_harness.raw_archive_bytes,
    );
    for (path, digest) in &meta_harness.expected_digests {
        if expected_digests
            .insert(path.clone(), digest.clone())
            .is_some()
        {
            return Err(AppError::invalid_input(
                "sources.duplicate_artifact",
                format!(
                    "artifact appears in more than one manifest: {}",
                    path.display()
                ),
            ));
        }
    }
    let snapshot_rows = load_snapshot_rows(repo_root)?;
    for row in &snapshot_rows {
        verify_file(
            repo_root,
            &row.snapshot_path,
            row.bytes,
            &row.sha256,
            "implementation snapshot",
        )?;
    }
    verify_implementation_identity(repo_root, &snapshot_rows)?;
    verify_license_status(repo_root)?;
    verify_capture_receipts(repo_root)?;

    let binaries = collect_binary_files(repo_root)?;
    for binary in &binaries {
        let relative = binary.strip_prefix(repo_root).map_err(|_| {
            AppError::invalid_input("sources.binary_path", "binary escaped repository")
        })?;
        let expected = expected_digests.get(relative).ok_or_else(|| {
            AppError::invalid_input(
                "sources.binary_manifest",
                format!(
                    "binary evidence is absent from a digest manifest: {}",
                    relative.display()
                ),
            )
        })?;
        let actual = sha256_file(binary)?;
        if &actual != expected {
            return Err(AppError::invalid_input(
                "sources.binary_digest",
                format!("binary evidence digest mismatch: {}", relative.display()),
            ));
        }
    }
    verify_lfs_attributes(repo_root, &binaries)?;
    let local_locators = verify_content_locators(repo_root)?;

    Ok(SourcesReport {
        evidence_artifacts: weng
            + rlm
            + darwinx
            + self_improving_agents_survey
            + sicp
            + benchmarks
            + agentic_engineering
            + meta_harness.site_artifacts
            + meta_harness.normalized_artifacts
            + crouzeix.source_receipts
            + crouzeix.verification_receipts,
        snapshot_files: snapshot_rows.len(),
        binary_objects: binaries.len(),
        implementation_sources: snapshot_rows
            .iter()
            .map(|row| row.source_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        local_locators,
        crouzeix_source_receipts: crouzeix.source_receipts,
        crouzeix_verification_receipts: crouzeix.verification_receipts,
    })
}

fn verify_meta_harness_bundle(repo_root: &Path) -> Result<MetaHarnessReport, AppError> {
    let root = Path::new(META_HARNESS_ROOT);
    let mut expected_digests = BTreeMap::new();
    let site_manifest = root.join("site_manifest.tsv");
    let site_rows = read_tsv(repo_root, &site_manifest)?;
    if site_rows.first().map(Vec::as_slice)
        != Some(
            [
                "source_url",
                "local_path",
                "captured_at",
                "last_modified",
                "bytes",
                "sha256",
            ]
            .map(str::to_owned)
            .as_slice(),
        )
    {
        return Err(AppError::invalid_input(
            "sources.meta_harness_site",
            "Meta-Harness site manifest has an unexpected header",
        ));
    }
    for (index, columns) in site_rows.iter().enumerate().skip(1) {
        if columns.len() != 6
            || !columns[0].starts_with("https://yoonholee.com/meta-harness/")
            || columns[2].is_empty()
            || columns[3] != "Tue, 04 Aug 2026 03:07:36 GMT"
        {
            return Err(malformed_manifest(
                &site_manifest,
                index + 1,
                "invalid Meta-Harness site capture row",
            ));
        }
        let relative = safe_relative_path(&columns[1], "Meta-Harness site path")?;
        if !relative.starts_with(root.join("site")) {
            return Err(AppError::invalid_input(
                "sources.meta_harness_site",
                format!("site artifact escaped capture root: {}", relative.display()),
            ));
        }
        let bytes = parse_u64(columns.get(4), &site_manifest, index + 1, "bytes")?;
        let digest = parse_digest(columns.get(5), &site_manifest, index + 1)?;
        verify_file(
            repo_root,
            &relative,
            bytes,
            &digest,
            "Meta-Harness site artifact",
        )?;
        if expected_digests.insert(relative.clone(), digest).is_some() {
            return Err(AppError::invalid_input(
                "sources.meta_harness_site",
                format!("duplicate site artifact {}", relative.display()),
            ));
        }
    }
    if site_rows.len() != 14 {
        return Err(AppError::invalid_input(
            "sources.meta_harness_site",
            format!(
                "Meta-Harness site capture requires 13 artifacts, found {}",
                site_rows.len().saturating_sub(1)
            ),
        ));
    }
    verify_meta_harness_capture_receipt(repo_root)?;

    let run_root = root.join("trae_run");
    let normalized_manifest = run_root.join("normalized_manifest.tsv");
    let normalized_rows = verify_relative_artifact_manifest(
        repo_root,
        &run_root,
        &normalized_manifest,
        &mut expected_digests,
    )?;
    verify_meta_harness_normalized_run(repo_root, &run_root, &normalized_rows)?;

    let receipt_path = repo_root.join(run_root.join("receipt.json"));
    let receipt: MetaHarnessRunReceipt =
        serde_json::from_slice(&fs::read(&receipt_path).map_err(|error| {
            AppError::io(
                "sources.meta_harness_run",
                "read Meta-Harness TRAE run receipt",
                error,
            )
        })?)
        .map_err(|error| {
            AppError::invalid_input(
                "sources.meta_harness_run",
                format!("invalid Meta-Harness TRAE run receipt: {error}"),
            )
        })?;
    verify_meta_harness_run_receipt(&receipt)?;
    let archive_relative = run_root.join("raw_traecli_run.tar.gz");
    verify_file(
        repo_root,
        &archive_relative,
        receipt.raw_archive_bytes,
        &receipt.raw_archive_sha256,
        "Meta-Harness raw TRAE archive",
    )?;
    if receipt.raw_archive_bytes > META_HARNESS_ARCHIVE_MAX_BYTES {
        return Err(AppError::invalid_input(
            "sources.meta_harness_archive",
            "Meta-Harness raw archive exceeds the compressed-size limit",
        ));
    }
    expected_digests.insert(archive_relative.clone(), receipt.raw_archive_sha256);
    let archive_members = verify_meta_harness_archive(
        repo_root,
        &archive_relative,
        &run_root.join("raw_members.tsv"),
    )?;

    Ok(MetaHarnessReport {
        site_artifacts: site_rows.len() - 1,
        normalized_artifacts: normalized_rows.len() - 1,
        raw_archive_members: archive_members,
        raw_archive_bytes: receipt.raw_archive_bytes,
        expected_digests,
    })
}

pub fn materialize(
    repo_root: &Path,
    source: Option<&str>,
    all: bool,
) -> Result<MaterializeReport, AppError> {
    if source.is_some() == all {
        return Err(AppError::invalid_input(
            "sources.selection",
            "choose either --source ID or --all",
        ));
    }
    let definitions = load_source_definitions(repo_root)?;
    let selected = if all {
        definitions.values().collect::<Vec<_>>()
    } else {
        let source = source.expect("selection checked");
        vec![definitions.get(source).ok_or_else(|| {
            AppError::invalid_input(
                "sources.unknown",
                format!("unknown public source ID {source}"),
            )
        })?]
    };
    let sources_root = repo_root.join(".sources");
    ensure_real_directory(&sources_root, ".sources")?;

    let mut reports = Vec::new();
    for definition in selected {
        let checkout = sources_root.join(&definition.directory);
        materialize_one(&checkout, definition)?;
        verify_materialized_snapshot(&checkout, definition)?;
        reports.push(MaterializedSource {
            source_id: definition.source_id.clone(),
            revision: definition.revision.clone(),
            path: slash_path(
                checkout
                    .strip_prefix(repo_root)
                    .expect("checkout is repository-relative"),
            ),
            snapshot_files: definition.rows.len(),
        });
    }
    Ok(MaterializeReport { sources: reports })
}

fn verify_artifact_inventory(
    repo_root: &Path,
    manifest: &Path,
    path_column: usize,
    bytes_column: usize,
    digest_column: usize,
    expected_digests: &mut BTreeMap<PathBuf, String>,
) -> Result<usize, AppError> {
    let rows = read_tsv(repo_root, manifest)?;
    for (line_number, columns) in rows.iter().enumerate().skip(1) {
        let artifact_path = columns
            .get(path_column)
            .ok_or_else(|| malformed_manifest(manifest, line_number + 1, "missing path column"))?;
        let bytes = parse_u64(
            columns.get(bytes_column),
            manifest,
            line_number + 1,
            "bytes",
        )?;
        let digest = parse_digest(columns.get(digest_column), manifest, line_number + 1)?;
        let relative = safe_relative_path(artifact_path, "artifact manifest path")?;
        verify_file(repo_root, &relative, bytes, &digest, "evidence artifact")?;
        if expected_digests.insert(relative.clone(), digest).is_some() {
            return Err(AppError::invalid_input(
                "sources.duplicate_artifact",
                format!(
                    "artifact appears in more than one manifest: {}",
                    relative.display()
                ),
            ));
        }
    }
    Ok(rows.len().saturating_sub(1))
}

fn verify_meta_harness_capture_receipt(repo_root: &Path) -> Result<(), AppError> {
    let root = repo_root.join(META_HARNESS_ROOT);
    let receipt = read_key_value_tsv(&root.join("capture_receipt.tsv"))?;
    for (key, expected) in [
        ("repository", "Harp"),
        ("source_id", "META-HARNESS-SITE"),
        ("capture_state", "captured-and-offline-verified"),
        ("asset_count", "13"),
        ("http_last_modified", "Tue, 04 Aug 2026 03:07:36 GMT"),
    ] {
        if receipt.get(key).map(String::as_str) != Some(expected) {
            return Err(AppError::invalid_input(
                "sources.meta_harness_receipt",
                format!("Meta-Harness capture receipt has invalid {key}"),
            ));
        }
    }
    for (key, relative) in [
        ("acquisition_script_sha256", "acquire.sh"),
        ("site_manifest_sha256", "site_manifest.tsv"),
    ] {
        let expected = receipt
            .get(key)
            .and_then(|value| value.strip_prefix("sha256:"))
            .ok_or_else(|| {
                AppError::invalid_input(
                    "sources.meta_harness_receipt",
                    format!("Meta-Harness capture receipt is missing {key}"),
                )
            })?;
        if sha256_file(&root.join(relative))? != expected {
            return Err(AppError::invalid_input(
                "sources.meta_harness_receipt",
                format!("Meta-Harness capture receipt digest is stale for {relative}"),
            ));
        }
    }
    if !receipt
        .get("venue_claim_boundary")
        .is_some_and(|value| value.contains("dated first-party site claim"))
    {
        return Err(AppError::invalid_input(
            "sources.meta_harness_receipt",
            "Meta-Harness venue claim boundary is missing",
        ));
    }
    Ok(())
}

fn verify_relative_artifact_manifest(
    repo_root: &Path,
    manifest_root: &Path,
    manifest: &Path,
    expected_digests: &mut BTreeMap<PathBuf, String>,
) -> Result<Vec<Vec<String>>, AppError> {
    let rows = read_tsv(repo_root, manifest)?;
    if rows.first().map(|row| row.join("\t")) != Some("path\tbytes\tsha256".to_owned()) {
        return Err(AppError::invalid_input(
            "sources.meta_harness_manifest",
            format!("{} has an unexpected header", manifest.display()),
        ));
    }
    let mut seen = BTreeSet::new();
    for (index, columns) in rows.iter().enumerate().skip(1) {
        if columns.len() != 3 {
            return Err(malformed_manifest(
                manifest,
                index + 1,
                "expected three columns",
            ));
        }
        let member = safe_relative_path(&columns[0], "Meta-Harness artifact path")?;
        if !seen.insert(member.clone()) {
            return Err(AppError::invalid_input(
                "sources.meta_harness_manifest",
                format!("duplicate Meta-Harness artifact {}", member.display()),
            ));
        }
        let relative = manifest_root.join(member);
        let bytes = parse_u64(columns.get(1), manifest, index + 1, "bytes")?;
        let digest = parse_digest(columns.get(2), manifest, index + 1)?;
        verify_file(
            repo_root,
            &relative,
            bytes,
            &digest,
            "Meta-Harness normalized artifact",
        )?;
        if expected_digests.insert(relative.clone(), digest).is_some() {
            return Err(AppError::invalid_input(
                "sources.duplicate_artifact",
                format!(
                    "artifact appears in more than one manifest: {}",
                    relative.display()
                ),
            ));
        }
    }
    Ok(rows)
}

fn verify_meta_harness_normalized_run(
    repo_root: &Path,
    run_root: &Path,
    manifest_rows: &[Vec<String>],
) -> Result<(), AppError> {
    if manifest_rows.len() != 9 {
        return Err(AppError::invalid_input(
            "sources.meta_harness_run",
            format!(
                "normalized Meta-Harness run requires 8 artifacts, found {}",
                manifest_rows.len().saturating_sub(1)
            ),
        ));
    }
    let normalized = repo_root.join(run_root).join("normalized");
    let final_response: TraeFinalResponse = read_strict_json(
        &normalized.join("final_response.json"),
        "TRAE final response",
    )?;
    let pending: TraePendingEval = read_strict_json(
        &normalized.join("pending_eval.json"),
        "TRAE pending evaluation",
    )?;
    let validation: TraeValidationReport = read_strict_json(
        &normalized.join("validation.json"),
        "TRAE validation report",
    )?;
    if final_response.schema_version != "harp-meta-harness-trae-final/v1"
        || final_response.candidates.len() != 3
        || final_response.candidates != pending.candidates
    {
        return Err(AppError::invalid_input(
            "sources.meta_harness_run",
            "normalized proposal and pending evaluation do not match the v1 contract",
        ));
    }
    let mut names = BTreeSet::new();
    for candidate in &final_response.candidates {
        if !valid_snake_case(&candidate.name)
            || !names.insert(candidate.name.as_str())
            || candidate.path != format!("agents/{}.py", candidate.name)
            || candidate.axis.trim().is_empty()
            || candidate.hypothesis.trim().is_empty()
            || candidate.components.as_ref().is_some_and(|items| {
                items.is_empty() || items.iter().any(|item| item.trim().is_empty())
            })
            || candidate
                .base_system
                .as_ref()
                .is_some_and(|value| value.trim().is_empty())
        {
            return Err(AppError::invalid_input(
                "sources.meta_harness_run",
                format!("invalid normalized candidate {}", candidate.name),
            ));
        }
    }
    if validation.schema_version != "harp-meta-harness-trae-validation/v1"
        || validation.candidate_count != 3
        || validation.valid_candidate_count != 0
        || validation.candidates.len() != 3
        || validation.interface_checks_passed
        || validation.benchmark_invoked
        || validation.held_out_test_invoked
        || !validation.workspace_boundary_passed
        || validation
            .workspace_files
            .iter()
            .any(|path| path.contains("__pycache__") || path.ends_with(".pyc"))
    {
        return Err(AppError::invalid_input(
            "sources.meta_harness_run",
            "normalized TRAE validation report violates the recorded boundary",
        ));
    }
    for (candidate, result) in final_response
        .candidates
        .iter()
        .zip(validation.candidates.iter())
    {
        if result.name != candidate.name
            || result.path != candidate.path
            || result.valid
            || result
                .error
                .as_deref()
                .is_none_or(|error| !error.contains("import or interface failure"))
            || !valid_digest(&result.sha256)
            || sha256_file(&normalized.join(&candidate.path))? != result.sha256
        {
            return Err(AppError::invalid_input(
                "sources.meta_harness_run",
                format!("candidate validation mismatch for {}", candidate.name),
            ));
        }
    }
    let reference: serde_json::Value = read_strict_json(
        &normalized.join("reference_state.json"),
        "TRAE reference state",
    )?;
    if reference["schema_version"] != "harp-meta-harness-reference-state/v1"
        || reference["reported_site_state"]["measurement_status"]
            != "first-party-reported-not-local"
        || reference["local_measurements"]
            .as_array()
            .is_none_or(|items| !items.is_empty())
        || !reference["missing_inputs"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item == "frontier_val.json"))
    {
        return Err(AppError::invalid_input(
            "sources.meta_harness_run",
            "TRAE reference state does not preserve the reported-vs-local boundary",
        ));
    }
    let prompt = fs::read_to_string(normalized.join("prompt.md"))
        .map_err(|error| AppError::io("sources.meta_harness_run", "read TRAE prompt", error))?;
    for required in [
        "Do not invent `frontier_val.json`",
        "Candidate quality is not being benchmarked",
        "preamble",
        "Markdown fence",
        "trailing prose",
    ] {
        if !prompt.contains(required) {
            return Err(AppError::invalid_input(
                "sources.meta_harness_run",
                format!("normalized TRAE prompt is missing boundary {required:?}"),
            ));
        }
    }
    Ok(())
}

fn verify_meta_harness_run_receipt(receipt: &MetaHarnessRunReceipt) -> Result<(), AppError> {
    if receipt.schema_version != "harp-meta-harness-trae-run/v1"
        || receipt.source_id != "META-HARNESS-TRAE-RUN"
        || receipt.repository_source_id != "META-HARNESS-REPO"
        || receipt.repository_revision != "44b9942127847f7421db70d8c7e48407f09a3c70"
        || receipt.model != "gpt-5.4"
        || receipt.candidate_count != 3
        || receipt.valid_candidate_count != 0
        || receipt.first_model_reaching_attempt != 2
        || receipt.first_schema_valid_attempt != 3
        || receipt.command_returncode != 0
        || !receipt.proposal_schema_validated
        || !receipt.candidate_interfaces_assessed
        || receipt.candidate_interfaces_passed
        || !receipt.invalid_candidates_retained
        || !receipt.workspace_boundary_validated
        || receipt.benchmark_invoked
        || receipt.held_out_test_invoked
        || receipt.paid_model_evaluation_reproduced
        || receipt.benchmark_score_reproduced
        || receipt.secret_scan != "passed-without-redaction"
        || !valid_digest(&receipt.raw_archive_sha256)
        || receipt.raw_archive_bytes == 0
        || receipt.limitations.len() < 4
    {
        return Err(AppError::invalid_input(
            "sources.meta_harness_run",
            "Meta-Harness TRAE run receipt violates the v1 contract",
        ));
    }
    Ok(())
}

fn verify_meta_harness_archive(
    repo_root: &Path,
    archive_relative: &Path,
    inventory_relative: &Path,
) -> Result<usize, AppError> {
    let inventory_rows = read_tsv(repo_root, inventory_relative)?;
    if inventory_rows.first().map(|row| row.join("\t")) != Some("path\tbytes\tsha256".to_owned()) {
        return Err(AppError::invalid_input(
            "sources.meta_harness_archive",
            "Meta-Harness raw member inventory has an unexpected header",
        ));
    }
    let mut expected = BTreeMap::new();
    for (index, columns) in inventory_rows.iter().enumerate().skip(1) {
        if columns.len() != 3 {
            return Err(malformed_manifest(
                inventory_relative,
                index + 1,
                "expected three columns",
            ));
        }
        let path = safe_relative_path(&columns[0], "raw archive member path")?;
        let bytes = parse_u64(columns.get(1), inventory_relative, index + 1, "bytes")?;
        let digest = parse_digest(columns.get(2), inventory_relative, index + 1)?;
        if expected.insert(path.clone(), (bytes, digest)).is_some() {
            return Err(AppError::invalid_input(
                "sources.meta_harness_archive",
                format!("duplicate raw archive member {}", path.display()),
            ));
        }
    }
    if expected.is_empty() || expected.len() > META_HARNESS_ARCHIVE_MAX_MEMBERS {
        return Err(AppError::invalid_input(
            "sources.meta_harness_archive",
            "Meta-Harness raw archive member count is outside the bounded range",
        ));
    }
    let archive_file = fs::File::open(repo_root.join(archive_relative)).map_err(|error| {
        AppError::io(
            "sources.meta_harness_archive",
            "open Meta-Harness raw archive",
            error,
        )
    })?;
    let mut archive = tar::Archive::new(GzDecoder::new(archive_file));
    let mut seen = BTreeMap::new();
    let mut total_bytes = 0u64;
    for entry in archive.entries().map_err(|error| {
        AppError::external(
            "sources.meta_harness_archive",
            format!("read Meta-Harness raw archive: {error}"),
        )
    })? {
        let entry = entry.map_err(|error| {
            AppError::external(
                "sources.meta_harness_archive",
                format!("read Meta-Harness raw archive member: {error}"),
            )
        })?;
        if !entry.header().entry_type().is_file() {
            return Err(AppError::invalid_input(
                "sources.meta_harness_archive",
                "Meta-Harness raw archive may contain regular files only",
            ));
        }
        let path = entry.path().map_err(|error| {
            AppError::external(
                "sources.meta_harness_archive",
                format!("decode Meta-Harness raw archive path: {error}"),
            )
        })?;
        let path = safe_relative_path(
            path.to_str().ok_or_else(|| {
                AppError::invalid_input(
                    "sources.meta_harness_archive",
                    "raw archive member path is not UTF-8",
                )
            })?,
            "raw archive member path",
        )?;
        let declared = entry.size();
        total_bytes = total_bytes.checked_add(declared).ok_or_else(|| {
            AppError::invalid_input(
                "sources.meta_harness_archive",
                "raw archive decompressed size overflowed",
            )
        })?;
        if total_bytes > META_HARNESS_DECOMPRESSED_MAX_BYTES {
            return Err(AppError::invalid_input(
                "sources.meta_harness_archive",
                "Meta-Harness raw archive exceeds the decompressed-size limit",
            ));
        }
        let mut bytes = Vec::new();
        entry
            .take(declared.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| {
                AppError::io(
                    "sources.meta_harness_archive",
                    "read Meta-Harness raw archive member",
                    error,
                )
            })?;
        if bytes.len() as u64 != declared {
            return Err(AppError::invalid_input(
                "sources.meta_harness_archive",
                format!("raw archive member size mismatch: {}", path.display()),
            ));
        }
        reject_sensitive_raw_member(&path, &bytes)?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if seen.insert(path.clone(), (declared, digest)).is_some() {
            return Err(AppError::invalid_input(
                "sources.meta_harness_archive",
                format!("duplicate raw archive member {}", path.display()),
            ));
        }
    }
    if seen != expected {
        return Err(AppError::invalid_input(
            "sources.meta_harness_archive",
            "Meta-Harness raw archive does not match its member digest inventory",
        ));
    }
    Ok(seen.len())
}

fn reject_sensitive_raw_member(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    let lower = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    let fixed_markers = [
        "-----begin private key-----",
        "-----begin rsa private key-----",
        "-----begin ec private key-----",
        "-----begin openssh private key-----",
        "\"authorization\":\"bearer ",
        "\"authorization\": \"bearer ",
        "cookie: session=",
        "set-cookie: session=",
    ];
    let sk_key = lower.match_indices("sk-").any(|(index, _)| {
        lower[index + 3..]
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .take(16)
            .count()
            == 16
    });
    if sk_key || fixed_markers.iter().any(|marker| lower.contains(marker)) {
        return Err(AppError::invalid_input(
            "sources.meta_harness_secret",
            format!(
                "sensitive material found in raw archive member {}",
                path.display()
            ),
        ));
    }
    Ok(())
}

fn read_strict_json<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, AppError> {
    serde_json::from_slice(
        &fs::read(path).map_err(|error| AppError::io("sources.meta_harness_run", label, error))?,
    )
    .map_err(|error| {
        AppError::invalid_input(
            "sources.meta_harness_run",
            format!("invalid {label}: {error}"),
        )
    })
}

fn valid_snake_case(value: &str) -> bool {
    let mut parts = value.split('_');
    parts.clone().count() >= 2
        && parts.all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(index, byte)| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() && index > 0
                })
        })
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn verify_sicp_manifest(
    repo_root: &Path,
    expected_digests: &mut BTreeMap<PathBuf, String>,
) -> Result<usize, AppError> {
    let manifest = Path::new("evidence/sicp/manifest.tsv");
    let rows = read_tsv(repo_root, manifest)?;
    for (line_number, columns) in rows.iter().enumerate().skip(1) {
        let relative = safe_relative_path(
            columns.get(3).ok_or_else(|| {
                malformed_manifest(manifest, line_number + 1, "missing local_path")
            })?,
            "SICP manifest path",
        )?;
        let bytes = parse_u64(columns.get(5), manifest, line_number + 1, "bytes")?;
        let digest = parse_digest(columns.get(6), manifest, line_number + 1)?;
        verify_file(repo_root, &relative, bytes, &digest, "SICP artifact")?;
        expected_digests.insert(relative, digest);
    }
    Ok(rows.len().saturating_sub(1))
}

fn verify_benchmark_manifest(repo_root: &Path) -> Result<usize, AppError> {
    let manifest = Path::new(BENCHMARK_MANIFEST);
    let rows = read_tsv(repo_root, manifest)?;
    const HEADER: &str = "benchmark\ttask_id\tartifact_role\tsource_url\timmutable_identity\tlocal_path\tbytes\tsha256\tvisibility\tlicense_status";
    if rows.first().map(|row| row.join("\t")).as_deref() != Some(HEADER) {
        return Err(benchmark_manifest_error(manifest, 1, "unexpected header"));
    }

    let mut seen = BTreeSet::new();
    for (index, columns) in rows.iter().enumerate().skip(1) {
        let line = index + 1;
        if columns.len() != 10 {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "expected ten columns",
            ));
        }
        let (
            benchmark,
            task_id,
            role,
            source_url,
            identity,
            local,
            bytes,
            digest,
            visibility,
            license,
        ) = (
            &columns[0],
            &columns[1],
            &columns[2],
            &columns[3],
            &columns[4],
            &columns[5],
            &columns[6],
            &columns[7],
            &columns[8],
            &columns[9],
        );
        if [
            benchmark, task_id, role, source_url, identity, visibility, license,
        ]
        .iter()
        .any(|value| value.trim() != **value || value.is_empty())
            || (local != "-" && local.trim() != local)
        {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "fields must be canonical trimmed strings",
            ));
        }
        if !source_url.starts_with("https://") {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "source_url must use HTTPS",
            ));
        }
        let immutable_git = identity.strip_prefix("git:").is_some_and(|value| {
            value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
        let immutable_blob = identity.strip_prefix("sha256:").is_some_and(valid_digest);
        if !immutable_git && !immutable_blob {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "immutable_identity must be git:<40 hex> or sha256:<64 lowercase hex>",
            ));
        }
        if !matches!(visibility.as_str(), "reader-facing" | "verifier-only") {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "visibility must be reader-facing or verifier-only",
            ));
        }
        if !matches!(
            license.as_str(),
            "MIT"
                | "Apache-2.0"
                | "BSD-3-Clause"
                | "CC-BY-4.0"
                | "unconfirmed"
                | "not-redistributable"
        ) {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "license_status must be explicit",
            ));
        }
        if !seen.insert((benchmark, task_id, role)) {
            return Err(benchmark_manifest_error(
                manifest,
                line,
                "duplicate benchmark/task_id/artifact_role row",
            ));
        }
        if local == "-" {
            let _ = bytes
                .parse::<u64>()
                .map_err(|_| benchmark_manifest_error(manifest, line, "invalid bytes"))?;
            let expected_digest = digest.strip_prefix("sha256:").unwrap_or(digest);
            if !valid_digest(expected_digest) {
                return Err(benchmark_manifest_error(
                    manifest,
                    line,
                    "remote receipts require a 64-character lowercase hex sha256",
                ));
            }
        } else {
            if visibility == "reader-facing" {
                return Err(benchmark_manifest_error(
                    manifest,
                    line,
                    "reader-facing receipts must remain remote-only",
                ));
            }
            let local_path = safe_relative_path(local, "benchmark local_path")?;
            if !local_path.starts_with("evidence/benchmarks/") {
                return Err(benchmark_manifest_error(
                    manifest,
                    line,
                    "local_path must remain below evidence/benchmarks",
                ));
            }
            let expected_bytes = bytes
                .parse::<u64>()
                .map_err(|_| benchmark_manifest_error(manifest, line, "invalid bytes"))?;
            let expected_digest = digest.strip_prefix("sha256:").unwrap_or(digest);
            if !valid_digest(expected_digest) {
                return Err(benchmark_manifest_error(
                    manifest,
                    line,
                    "sha256 must be a 64-character lowercase hex digest",
                ));
            }
            verify_file(
                repo_root,
                &local_path,
                expected_bytes,
                expected_digest,
                "benchmark receipt",
            )
            .map_err(|error| {
                benchmark_manifest_error(manifest, line, &format!("vendored receipt: {error}"))
            })?;
        }
    }
    if rows.len() == 1 {
        return Err(benchmark_manifest_error(
            manifest,
            1,
            "manifest has no receipts",
        ));
    }
    Ok(rows.len() - 1)
}

fn benchmark_manifest_error(manifest: &Path, line: usize, detail: &str) -> AppError {
    AppError::invalid_input(
        "sources.benchmark_manifest",
        format!("{} line {line}: {detail}", manifest.display()),
    )
}

fn load_snapshot_rows(repo_root: &Path) -> Result<Vec<SnapshotRow>, AppError> {
    let manifest = Path::new(IMPLEMENTATION_MANIFEST);
    let rows = read_tsv(repo_root, manifest)?;
    let mut snapshots = Vec::new();
    for (line_number, columns) in rows.iter().enumerate().skip(1) {
        if columns.len() != 6 {
            return Err(malformed_manifest(
                manifest,
                line_number + 1,
                "expected six columns",
            ));
        }
        let revision = columns[2].to_owned();
        if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(malformed_manifest(
                manifest,
                line_number + 1,
                "revision must be a 40-character Git object ID",
            ));
        }
        snapshots.push(SnapshotRow {
            source_id: columns[0].to_owned(),
            remote: columns[1].to_owned(),
            revision,
            snapshot_path: safe_relative_path(&columns[3], "snapshot path")?,
            bytes: parse_u64(columns.get(4), manifest, line_number + 1, "bytes")?,
            sha256: parse_digest(columns.get(5), manifest, line_number + 1)?,
        });
    }
    Ok(snapshots)
}

fn load_source_definitions(
    repo_root: &Path,
) -> Result<BTreeMap<String, SourceDefinition>, AppError> {
    let rows = load_snapshot_rows(repo_root)?;
    let mut definitions = BTreeMap::<String, SourceDefinition>::new();
    for row in rows {
        let directory = row
            .snapshot_path
            .components()
            .nth(2)
            .and_then(|component| match component {
                Component::Normal(value) => value.to_str(),
                _ => None,
            })
            .ok_or_else(|| {
                AppError::invalid_input(
                    "sources.snapshot_path",
                    format!("unexpected snapshot path {}", row.snapshot_path.display()),
                )
            })?
            .to_owned();
        let definition =
            definitions
                .entry(row.source_id.clone())
                .or_insert_with(|| SourceDefinition {
                    source_id: row.source_id.clone(),
                    directory,
                    remote: row.remote.clone(),
                    revision: row.revision.clone(),
                    rows: Vec::new(),
                });
        if definition.remote != row.remote || definition.revision != row.revision {
            return Err(AppError::invalid_input(
                "sources.snapshot_identity",
                format!("source {} has inconsistent identity rows", row.source_id),
            ));
        }
        definition.rows.push(row);
    }
    Ok(definitions)
}

fn verify_implementation_identity(repo_root: &Path, rows: &[SnapshotRow]) -> Result<(), AppError> {
    let mut seen = BTreeSet::new();
    for row in rows {
        if !seen.insert(row.source_id.as_str()) {
            continue;
        }
        let directory = row
            .snapshot_path
            .components()
            .nth(2)
            .and_then(|component| match component {
                Component::Normal(value) => value.to_str(),
                _ => None,
            })
            .ok_or_else(|| {
                AppError::invalid_input("sources.snapshot_path", "invalid snapshot directory")
            })?;
        let root = repo_root.join("evidence/implementations").join(directory);
        let remote = read_trimmed(&root.join("REMOTE"), "implementation remote")?;
        let revision = read_trimmed(&root.join("REVISION"), "implementation revision")?;
        if remote != row.remote || revision != row.revision {
            return Err(AppError::invalid_input(
                "sources.snapshot_identity",
                format!("tracked identity mismatch for {}", row.source_id),
            ));
        }
    }
    Ok(())
}

fn verify_license_status(repo_root: &Path) -> Result<(), AppError> {
    let implementations = repo_root.join("evidence/implementations");
    for entry in fs::read_dir(&implementations)
        .map_err(|error| AppError::io("sources.license", "implementation licenses", error))?
    {
        let entry =
            entry.map_err(|error| AppError::io("sources.license", "license entry", error))?;
        if !entry
            .file_type()
            .map_err(|error| AppError::io("sources.license", "license entry type", error))?
            .is_dir()
        {
            continue;
        }
        let status_path = entry.path().join("LICENSE_STATUS");
        let status = read_trimmed(&status_path, "implementation license status")?;
        if let Some(file) = status.strip_prefix("captured\t") {
            let license = entry.path().join(file);
            if !license.is_file() {
                return Err(AppError::invalid_input(
                    "sources.license",
                    format!("captured license is missing: {}", license.display()),
                ));
            }
        } else if status != "not-present-at-pinned-revision\tMISSING" {
            return Err(AppError::invalid_input(
                "sources.license",
                format!("invalid license status: {}", status_path.display()),
            ));
        }
    }
    for required in [
        "evidence/sicp/LICENSE.txt",
        "evidence/rlm/artifacts/git/LICENSE",
        "evidence/weng/license_assignments.tsv",
    ] {
        if !repo_root.join(required).is_file() {
            return Err(AppError::invalid_input(
                "sources.license",
                format!("required license manifest is missing: {required}"),
            ));
        }
    }
    Ok(())
}

fn collect_binary_files(repo_root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut binaries = Vec::new();
    for entry in WalkDir::new(repo_root.join("evidence")).follow_links(false) {
        let entry = entry.map_err(|error| {
            AppError::external(
                "sources.walk",
                format!("could not walk evidence tree: {error}"),
            )
        })?;
        if entry.file_type().is_symlink() {
            return Err(AppError::invalid_input(
                "sources.symlink",
                format!(
                    "evidence tree cannot contain symlink {}",
                    entry.path().display()
                ),
            ));
        }
        if entry.file_type().is_file()
            && (matches!(
                entry.path().extension().and_then(OsStr::to_str),
                Some("pdf" | "png" | "jpg" | "jpeg" | "webp" | "woff2")
            ) || entry
                .path()
                .file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.ends_with(".tar.gz")))
        {
            binaries.push(entry.path().to_path_buf());
        }
    }
    binaries.sort();
    Ok(binaries)
}

fn verify_lfs_attributes(repo_root: &Path, binaries: &[PathBuf]) -> Result<(), AppError> {
    let attributes = read_trimmed(&repo_root.join(".gitattributes"), "Git attributes")?;
    for pattern in [
        "evidence/**/*.pdf",
        "evidence/**/*.png",
        "evidence/**/*.jpg",
        "evidence/**/*.webp",
        "evidence/**/*.woff2",
        "evidence/**/*.tar.gz",
    ] {
        if !attributes
            .lines()
            .any(|line| line.starts_with(pattern) && line.contains("filter=lfs"))
        {
            return Err(AppError::invalid_input(
                "sources.lfs_attributes",
                format!("Git LFS attribute is missing for {pattern}"),
            ));
        }
    }
    if binaries.is_empty() {
        return Err(AppError::invalid_input(
            "sources.binary_manifest",
            "no binary evidence objects were found",
        ));
    }
    let mut command = Command::new("git");
    command
        .current_dir(repo_root)
        .args(["check-attr", "filter", "diff", "merge", "--"]);
    for binary in binaries {
        command.arg(binary.strip_prefix(repo_root).map_err(|_| {
            AppError::invalid_input("sources.binary_path", "binary escaped repository")
        })?);
    }
    let output = command.output().map_err(|error| {
        AppError::io(
            "sources.lfs_attributes",
            "run Git attribute verification",
            error,
        )
    })?;
    if !output.status.success() {
        return Err(AppError::external(
            "sources.lfs_attributes",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let resolved = String::from_utf8(output.stdout).map_err(|error| {
        AppError::external(
            "sources.lfs_attributes",
            format!("Git attribute output was not UTF-8: {error}"),
        )
    })?;
    for binary in binaries {
        let relative = slash_path(binary.strip_prefix(repo_root).expect("binary is relative"));
        for attribute in ["filter: lfs", "diff: lfs", "merge: lfs"] {
            if !resolved.contains(&format!("{relative}: {attribute}")) {
                return Err(AppError::invalid_input(
                    "sources.lfs_attributes",
                    format!("{relative} does not resolve {attribute}"),
                ));
            }
        }
    }
    Ok(())
}

fn verify_capture_receipts(repo_root: &Path) -> Result<(), AppError> {
    for bundle in ["evidence/weng", "evidence/rlm"] {
        let receipt = read_key_value_tsv(&repo_root.join(bundle).join("run-receipt.tsv"))?;
        if receipt.get("repository").map(String::as_str) != Some("Harp")
            || receipt.get("capture_state").map(String::as_str)
                != Some("imported-and-offline-verified")
        {
            return Err(AppError::invalid_input(
                "sources.capture_receipt",
                format!("{bundle} has an invalid Harp capture receipt"),
            ));
        }
        for (key, relative) in [
            ("acquisition_script_sha256", "acquire.sh"),
            ("source_table_sha256", "sources.tsv"),
        ] {
            let expected = receipt
                .get(key)
                .and_then(|value| value.strip_prefix("sha256:"))
                .ok_or_else(|| {
                    AppError::invalid_input(
                        "sources.capture_receipt",
                        format!("{bundle} receipt is missing {key}"),
                    )
                })?;
            let actual = sha256_file(&repo_root.join(bundle).join(relative))?;
            if expected != actual {
                return Err(AppError::invalid_input(
                    "sources.capture_receipt",
                    format!("{bundle} receipt digest is stale for {relative}"),
                ));
            }
        }
    }
    Ok(())
}

fn read_key_value_tsv(path: &Path) -> Result<BTreeMap<String, String>, AppError> {
    let text = fs::read_to_string(path)
        .map_err(|error| AppError::io("sources.capture_receipt", "read capture receipt", error))?;
    let mut values = BTreeMap::new();
    for (line_number, line) in text.lines().enumerate() {
        let (key, value) = line.split_once('\t').ok_or_else(|| {
            AppError::invalid_input(
                "sources.capture_receipt",
                format!(
                    "{} line {} is not key/value TSV",
                    path.display(),
                    line_number + 1
                ),
            )
        })?;
        if values.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(AppError::invalid_input(
                "sources.capture_receipt",
                format!("{} has duplicate key {key}", path.display()),
            ));
        }
    }
    Ok(values)
}

fn verify_content_locators(repo_root: &Path) -> Result<usize, AppError> {
    let mut locators = BTreeSet::new();
    for entry in WalkDir::new(repo_root.join("content")).follow_links(false) {
        let entry = entry.map_err(|error| {
            AppError::external(
                "sources.locator_walk",
                format!("could not walk content: {error}"),
            )
        })?;
        if !entry.file_type().is_file()
            || !matches!(
                entry.path().extension().and_then(OsStr::to_str),
                Some("md" | "tsv" | "json")
            )
        {
            continue;
        }
        let text = fs::read_to_string(entry.path())
            .map_err(|error| AppError::io("sources.locator_read", "content locator", error))?;
        for token in text.split(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '`' | '"' | '\'' | '(' | ')' | '[' | ']' | '<' | '>' | ',' | ';'
                )
        }) {
            let Some(offset) = token.find("evidence/") else {
                continue;
            };
            let raw = &token[offset..];
            let raw = raw.trim_end_matches(['.', ':', '|', '}']);
            let raw = raw.split('#').next().unwrap_or(raw);
            let raw = strip_locator_suffix(raw);
            if raw.is_empty() || raw.ends_with('/') {
                continue;
            }
            let path = safe_relative_path(raw, "content evidence locator")?;
            let absolute = repo_root.join(&path);
            let metadata = fs::symlink_metadata(&absolute).map_err(|error| {
                AppError::io(
                    "sources.locator_missing",
                    &format!("evidence locator {}", path.display()),
                    error,
                )
            })?;
            if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
                return Err(AppError::invalid_input(
                    "sources.locator_missing",
                    format!(
                        "{} references invalid evidence locator {}",
                        entry.path().display(),
                        path.display()
                    ),
                ));
            }
            locators.insert(path);
        }
    }
    Ok(locators.len())
}

fn strip_locator_suffix(value: &str) -> &str {
    let Some((before, after)) = value.rsplit_once(':') else {
        return value;
    };
    if after
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'-' || byte == b',' || byte == b'L')
    {
        before
    } else {
        value
    }
}

fn materialize_one(checkout: &Path, definition: &SourceDefinition) -> Result<(), AppError> {
    match fs::symlink_metadata(checkout) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(AppError::invalid_input(
                    "sources.checkout",
                    format!(
                        "materialized root must be a real directory: {}",
                        checkout.display()
                    ),
                ));
            }
            validate_existing_checkout(checkout, definition)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(checkout).map_err(|error| {
                AppError::io("sources.checkout", "create materialized checkout", error)
            })?;
            run_git(checkout, ["init", "--quiet"])?;
            run_git(checkout, ["remote", "add", "origin", &definition.remote])?;
            run_git(
                checkout,
                [
                    "fetch",
                    "--quiet",
                    "--depth",
                    "1",
                    "origin",
                    &definition.revision,
                ],
            )?;
            run_git(checkout, ["checkout", "--quiet", "--detach", "FETCH_HEAD"])?;
            validate_existing_checkout(checkout, definition)?;
        }
        Err(error) => return Err(AppError::io("sources.checkout", "inspect checkout", error)),
    }
    Ok(())
}

fn validate_existing_checkout(
    checkout: &Path,
    definition: &SourceDefinition,
) -> Result<(), AppError> {
    reject_git_unsafe_state(checkout)?;
    let remote = git_output(checkout, ["remote", "get-url", "origin"])?;
    if remote != definition.remote {
        return Err(AppError::invalid_input(
            "sources.remote",
            format!("wrong origin remote for {}", definition.source_id),
        ));
    }
    let revision = git_output(checkout, ["rev-parse", "HEAD"])?;
    if revision != definition.revision {
        return Err(AppError::invalid_input(
            "sources.revision",
            format!("wrong revision for {}", definition.source_id),
        ));
    }
    let status = git_output(
        checkout,
        ["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    if !status.is_empty() {
        return Err(AppError::invalid_input(
            "sources.dirty",
            format!("materialized checkout is dirty: {}", checkout.display()),
        ));
    }
    Ok(())
}

fn reject_git_unsafe_state(checkout: &Path) -> Result<(), AppError> {
    let git_dir_text = git_output(checkout, ["rev-parse", "--absolute-git-dir"])?;
    let git_dir = PathBuf::from(git_dir_text);
    let config = fs::read_to_string(git_dir.join("config"))
        .map_err(|error| AppError::io("sources.git_config", "read Git config", error))?;
    let normalized = config.to_ascii_lowercase();
    if normalized.contains("[include]") || normalized.contains("[includeif ") {
        return Err(AppError::invalid_input(
            "sources.git_include",
            "materialized checkout cannot use Git config includes",
        ));
    }
    let alternates = git_dir.join("objects/info/alternates");
    if alternates.exists() {
        return Err(AppError::invalid_input(
            "sources.git_alternates",
            "materialized checkout cannot use Git alternates",
        ));
    }
    let replace_dir = git_dir.join("refs/replace");
    if replace_dir.exists()
        && fs::read_dir(&replace_dir)
            .map_err(|error| AppError::io("sources.git_replace", "read replacement refs", error))?
            .next()
            .is_some()
    {
        return Err(AppError::invalid_input(
            "sources.git_replace",
            "materialized checkout cannot use replacement refs",
        ));
    }
    Ok(())
}

fn verify_materialized_snapshot(
    checkout: &Path,
    definition: &SourceDefinition,
) -> Result<(), AppError> {
    for row in &definition.rows {
        let prefix = Path::new("evidence/implementations")
            .join(&definition.directory)
            .join("snapshot");
        let relative = row.snapshot_path.strip_prefix(&prefix).map_err(|_| {
            AppError::invalid_input(
                "sources.snapshot_path",
                format!(
                    "snapshot path does not match source {}",
                    definition.source_id
                ),
            )
        })?;
        let actual = checkout.join(relative);
        let metadata = fs::metadata(&actual).map_err(|error| {
            AppError::io(
                "sources.materialized_file",
                &format!("materialized source file {}", actual.display()),
                error,
            )
        })?;
        if metadata.len() != row.bytes || sha256_file(&actual)? != row.sha256 {
            return Err(AppError::invalid_input(
                "sources.materialized_digest",
                format!("materialized snapshot mismatch: {}", actual.display()),
            ));
        }
    }
    Ok(())
}

fn ensure_real_directory(path: &Path, label: &str) -> Result<(), AppError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            Err(AppError::invalid_input(
                "sources.directory",
                format!("{label} must be a real directory"),
            ))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path).map_err(|error| AppError::io("sources.directory", label, error))
        }
        Err(error) => Err(AppError::io("sources.directory", label, error)),
    }
}

fn run_git<const N: usize>(checkout: &Path, arguments: [&str; N]) -> Result<(), AppError> {
    let output = git_command(checkout, arguments).output().map_err(|error| {
        AppError::io("sources.git", "execute Git materialization command", error)
    })?;
    if !output.status.success() {
        return Err(AppError::external(
            "sources.git",
            format!(
                "Git materialization command failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(())
}

fn git_output<const N: usize>(checkout: &Path, arguments: [&str; N]) -> Result<String, AppError> {
    let output = git_command(checkout, arguments)
        .output()
        .map_err(|error| AppError::io("sources.git", "execute Git inspection command", error))?;
    if !output.status.success() {
        return Err(AppError::external(
            "sources.git",
            format!(
                "Git inspection command failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| {
            AppError::external("sources.git", format!("Git output was not UTF-8: {error}"))
        })
}

fn git_command<const N: usize>(checkout: &Path, arguments: [&str; N]) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(checkout)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .args(arguments);
    command
}

fn verify_file(
    repo_root: &Path,
    relative: &Path,
    expected_bytes: u64,
    expected_digest: &str,
    label: &str,
) -> Result<(), AppError> {
    let absolute = repo_root.join(relative);
    let metadata = fs::symlink_metadata(&absolute)
        .map_err(|error| AppError::io("sources.file", label, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AppError::invalid_input(
            "sources.file",
            format!("{label} must be a regular file: {}", relative.display()),
        ));
    }
    if metadata.len() != expected_bytes {
        return Err(AppError::invalid_input(
            "sources.bytes",
            format!("{label} byte count mismatch: {}", relative.display()),
        ));
    }
    let actual = sha256_file(&absolute)?;
    if actual != expected_digest {
        return Err(AppError::invalid_input(
            "sources.digest",
            format!("{label} digest mismatch: {}", relative.display()),
        ));
    }
    Ok(())
}

fn read_tsv(repo_root: &Path, relative: &Path) -> Result<Vec<Vec<String>>, AppError> {
    let absolute = repo_root.join(relative);
    let text = fs::read_to_string(&absolute).map_err(|error| {
        AppError::io(
            "sources.manifest",
            &format!("read {}", relative.display()),
            error,
        )
    })?;
    let rows = text
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Err(AppError::invalid_input(
            "sources.manifest",
            format!("manifest is empty: {}", relative.display()),
        ));
    }
    Ok(rows)
}

fn parse_u64(
    value: Option<&String>,
    manifest: &Path,
    line_number: usize,
    field: &str,
) -> Result<u64, AppError> {
    value
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| malformed_manifest(manifest, line_number, &format!("invalid {field}")))
}

fn parse_digest(
    value: Option<&String>,
    manifest: &Path,
    line_number: usize,
) -> Result<String, AppError> {
    let value = value
        .ok_or_else(|| malformed_manifest(manifest, line_number, "missing sha256"))?
        .strip_prefix("sha256:")
        .unwrap_or_else(|| value.expect("checked"));
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(malformed_manifest(manifest, line_number, "invalid sha256"));
    }
    Ok(value.to_ascii_lowercase())
}

fn safe_relative_path(value: &str, label: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.as_os_str().is_empty()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(AppError::invalid_input(
            "sources.path",
            format!("{label} must be a repository-relative normal path: {value}"),
        ));
    }
    Ok(path.to_path_buf())
}

fn malformed_manifest(manifest: &Path, line: usize, detail: &str) -> AppError {
    AppError::invalid_input(
        "sources.manifest",
        format!("{} line {line}: {detail}", manifest.display()),
    )
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path)
        .map_err(|error| AppError::io("sources.digest", "read digest input", error))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn read_trimmed(path: &Path, label: &str) -> Result<String, AppError> {
    fs::read_to_string(path)
        .map(|value| value.trim().to_owned())
        .map_err(|error| AppError::io("sources.read", label, error))
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
    }

    #[test]
    fn verifies_the_offline_meta_harness_evidence_bundle() {
        let report = verify_meta_harness_bundle(workspace_root()).unwrap();

        assert_eq!(report.site_artifacts, 13);
        assert!(report.normalized_artifacts >= 8);
        assert!(report.raw_archive_members >= 7);
        assert!(report.raw_archive_bytes > 0);
    }

    #[test]
    fn benchmark_manifest_requires_immutable_unique_and_rights_aware_receipts() {
        let repo = tempdir().unwrap();
        let evidence = repo.path().join("evidence/benchmarks/swe-bench");
        fs::create_dir_all(&evidence).unwrap();
        fs::write(evidence.join("evaluator.py"), "print('verified')\n").unwrap();
        fs::write(
            repo.path().join("evidence/benchmarks/manifest.tsv"),
            concat!(
                "benchmark\ttask_id\tartifact_role\tsource_url\timmutable_identity\tlocal_path\tbytes\tsha256\tvisibility\tlicense_status\n",
                "SWE-bench Verified\tastropy__astropy-12907\tevaluator\t",
                "https://github.com/princeton-nlp/SWE-bench/blob/0123456789abcdef0123456789abcdef01234567/swebench/harness/test_spec.py\t",
                "git:0123456789abcdef0123456789abcdef01234567\t",
                "evidence/benchmarks/swe-bench/evaluator.py\t18\t",
                "8cc8c1fbd30e5c5c5f5ddf3aec94740b51e3f6c2e2f3f461bd0a9a5df97c07b7\t",
                "verifier-only\tMIT\n",
                "Terminal-Bench 2\textract-elf\ttask\t",
                "https://github.com/laude-institute/terminal-bench/blob/89abcdef0123456789abcdef0123456789abcdef/tasks/extract-elf/task.yaml\t",
                "git:89abcdef0123456789abcdef0123456789abcdef\t-\t0\t",
                "f7f9ee0c130c8fd37a4b9b5134aeedfaf52ec35f4de2b2817650f4760edb684b\t",
                "reader-facing\tunconfirmed\n"
            ),
        )
        .unwrap();

        let error = verify_benchmark_manifest(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.benchmark_manifest");

        let text =
            fs::read_to_string(repo.path().join("evidence/benchmarks/manifest.tsv")).unwrap();
        let corrected = text.replace(
            "8cc8c1fbd30e5c5c5f5ddf3aec94740b51e3f6c2e2f3f461bd0a9a5df97c07b7",
            &sha256_file(&evidence.join("evaluator.py")).unwrap(),
        );
        fs::write(
            repo.path().join("evidence/benchmarks/manifest.tsv"),
            corrected,
        )
        .unwrap();
        assert_eq!(verify_benchmark_manifest(repo.path()).unwrap(), 2);

        let duplicate = {
            let manifest =
                fs::read_to_string(repo.path().join("evidence/benchmarks/manifest.tsv")).unwrap();
            format!("{manifest}{}", manifest.lines().nth(1).unwrap())
        };
        fs::write(
            repo.path().join("evidence/benchmarks/manifest.tsv"),
            duplicate,
        )
        .unwrap();
        let error = verify_benchmark_manifest(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.benchmark_manifest");
    }
}
