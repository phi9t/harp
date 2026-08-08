use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use sha2::{Digest as _, Sha256};
use walkdir::WalkDir;

use crate::error::AppError;

const IMPLEMENTATION_MANIFEST: &str = "evidence/implementations/manifest.tsv";

#[derive(Debug, Serialize)]
pub struct SourcesReport {
    pub evidence_artifacts: usize,
    pub snapshot_files: usize,
    pub binary_objects: usize,
    pub implementation_sources: usize,
    pub local_locators: usize,
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

pub fn verify(repo_root: &Path) -> Result<SourcesReport, AppError> {
    let mut expected_digests = BTreeMap::new();
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
    let sicp = verify_sicp_manifest(repo_root, &mut expected_digests)?;
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
        evidence_artifacts: weng + rlm + sicp,
        snapshot_files: snapshot_rows.len(),
        binary_objects: binaries.len(),
        implementation_sources: snapshot_rows
            .iter()
            .map(|row| row.source_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        local_locators,
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
            && matches!(
                entry.path().extension().and_then(OsStr::to_str),
                Some("pdf" | "png" | "jpg" | "jpeg")
            )
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
