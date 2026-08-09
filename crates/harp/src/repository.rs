use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use walkdir::WalkDir;

use crate::error::AppError;

const IMPORT_MAP: &str = "docs/import-map.tsv";
const IMPORT_RECEIPT: &str = "docs/import-receipt.md";
const CORPUS_PATH: &str = "atlas/src/content/generated/corpus.json";
const ATLAS_HTML_PATH: &str = "atlas/dist/harp-atlas.html";
const ATLAS_RECEIPT_PATH: &str = "atlas/dist/harp-atlas.receipt.json";

#[derive(Debug, Serialize)]
pub struct RepositoryReport {
    pub import_rows: usize,
    pub original_rsi_files: usize,
    pub original_evidence_files: usize,
    pub original_evaluator_files: usize,
    pub original_compiler_files: usize,
    pub original_auxiliary_files: usize,
    pub forbidden_references: usize,
    pub generated_artifacts: usize,
    pub payload_sha256: String,
}

#[derive(Debug, Deserialize)]
struct AtlasReceipt {
    schema_version: String,
    corpus_sha256: String,
    app_inputs_sha256: String,
    html_sha256: String,
}

pub fn verify(repo_root: &Path) -> Result<RepositoryReport, AppError> {
    let import = verify_import_map(repo_root)?;
    let forbidden_references = verify_forbidden_references(repo_root)?;
    verify_atlas_artifacts(repo_root)?;
    let payload_sha256 = payload_digest(repo_root)?;
    verify_import_receipt(repo_root, &payload_sha256)?;
    Ok(RepositoryReport {
        import_rows: import.total,
        original_rsi_files: import.rsi,
        original_evidence_files: import.evidence,
        original_evaluator_files: import.evaluator,
        original_compiler_files: import.compiler,
        original_auxiliary_files: import.auxiliary,
        forbidden_references,
        generated_artifacts: 2,
        payload_sha256,
    })
}

pub fn payload_digest(repo_root: &Path) -> Result<String, AppError> {
    let mut paths = repository_payload_paths(repo_root)?;
    paths.sort();
    let mut digest = Sha256::new();
    for relative in paths {
        let bytes = fs::read(repo_root.join(&relative)).map_err(|error| {
            AppError::io(
                "repository.payload",
                &format!("read payload file {}", relative.display()),
                error,
            )
        })?;
        let path = slash_path(&relative);
        digest.update((path.len() as u64).to_le_bytes());
        digest.update(path.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

struct ImportCounts {
    total: usize,
    rsi: usize,
    evidence: usize,
    evaluator: usize,
    compiler: usize,
    auxiliary: usize,
}

fn verify_import_map(repo_root: &Path) -> Result<ImportCounts, AppError> {
    let path = repo_root.join(IMPORT_MAP);
    let text = fs::read_to_string(&path)
        .map_err(|error| AppError::io("repository.import_map", IMPORT_MAP, error))?;
    let mut lines = text.lines();
    if lines.next()
        != Some("source_path_percent_encoded\tdisposition_percent_encoded\tdestination\tnote")
    {
        return Err(AppError::invalid_input(
            "repository.import_map",
            "import map has an unexpected header",
        ));
    }
    let expected_omitted = [
        "omitted-",
        &String::from_utf8(vec![0x74, 0x61, 0x72, 0x6f, 0x63, 0x63, 0x6f]).expect("ASCII"),
        "-specific",
    ]
    .concat();
    let allowed = BTreeSet::from([
        "ported".to_owned(),
        "rewritten".to_owned(),
        "generated".to_owned(),
        "replaced".to_owned(),
        expected_omitted.clone(),
    ]);
    let mut sources = BTreeSet::new();
    let mut source_digest = Sha256::new();
    let mut counts = ImportCounts {
        total: 0,
        rsi: 0,
        evidence: 0,
        evaluator: 0,
        compiler: 0,
        auxiliary: 0,
    };
    for (index, line) in lines.enumerate() {
        let columns = line.split('\t').collect::<Vec<_>>();
        if columns.len() != 4 {
            return Err(AppError::invalid_input(
                "repository.import_map",
                format!("import map line {} must have four columns", index + 2),
            ));
        }
        let source = percent_decode(columns[0])?;
        let disposition = percent_decode(columns[1])?;
        if !sources.insert(source.clone()) {
            return Err(AppError::invalid_input(
                "repository.import_map",
                format!("duplicate import source path: {source}"),
            ));
        }
        source_digest.update((source.len() as u64).to_le_bytes());
        source_digest.update(source.as_bytes());
        if !allowed.contains(&disposition) {
            return Err(AppError::invalid_input(
                "repository.import_map",
                format!("invalid disposition for {source}: {disposition}"),
            ));
        }
        if disposition == expected_omitted {
            if columns[2] != "-" {
                return Err(AppError::invalid_input(
                    "repository.import_map",
                    format!("omitted source has a destination: {source}"),
                ));
            }
        } else {
            let destination = safe_relative(columns[2], "import destination")?;
            if !repo_root.join(&destination).exists() {
                return Err(AppError::invalid_input(
                    "repository.import_map",
                    format!(
                        "import destination is missing for {source}: {}",
                        destination.display()
                    ),
                ));
            }
        }
        counts.total += 1;
        if source.starts_with(&["knowledge", "rsi"].join("/")) {
            counts.rsi += 1;
        } else if source.starts_with(&["third", "party"].join("_")) {
            counts.evidence += 1;
        } else if source.starts_with(&["learning", "sicp_evaluator"].join("/")) {
            counts.evaluator += 1;
        } else if source == ["src", "commands", "knowledge", "rsi.rs"].join("/")
            || source.starts_with(&["src", "commands", "knowledge", "rsi"].join("/"))
        {
            counts.compiler += 1;
        } else {
            counts.auxiliary += 1;
        }
    }
    let expected = (521, 172, 322, 5, 6, 16);
    let actual = (
        counts.total,
        counts.rsi,
        counts.evidence,
        counts.evaluator,
        counts.compiler,
        counts.auxiliary,
    );
    if actual != expected {
        return Err(AppError::invalid_input(
            "repository.import_map",
            format!("import map parity mismatch: expected {expected:?}, got {actual:?}"),
        ));
    }
    let source_digest = format!("{:x}", source_digest.finalize());
    if source_digest != "f3f7f276a0a813e708bb641158f56519dd96c516b31d50753e8a84db3e40dab2" {
        return Err(AppError::invalid_input(
            "repository.import_map",
            format!("import source-path digest mismatch: {source_digest}"),
        ));
    }
    Ok(counts)
}

fn verify_forbidden_references(repo_root: &Path) -> Result<usize, AppError> {
    let source_name =
        String::from_utf8(vec![0x74, 0x61, 0x72, 0x6f, 0x63, 0x63, 0x6f]).expect("ASCII");
    let patterns = [
        source_name,
        ["third", "party"].join("_") + "/rsi",
        ["source", "graph"].join("_"),
        ["/Users", "bytedance", "workspace"].join("/"),
        ["qmd", "index"].join("-"),
        ["qmd", "://"].concat(),
        ["general Knowledge", "Atlas"].join(" "),
    ];
    let mut violations = Vec::new();
    for relative in repository_payload_paths(repo_root)? {
        if relative == Path::new(IMPORT_RECEIPT) {
            continue;
        }
        let bytes = fs::read(repo_root.join(&relative)).map_err(|error| {
            AppError::io(
                "repository.scan",
                &format!("read {}", relative.display()),
                error,
            )
        })?;
        let Ok(text) = std::str::from_utf8(&bytes) else {
            continue;
        };
        let lower = text.to_ascii_lowercase();
        for pattern in &patterns {
            if lower.contains(&pattern.to_ascii_lowercase()) {
                violations.push(format!("{} contains {pattern:?}", relative.display()));
            }
        }
    }
    if !violations.is_empty() {
        return Err(AppError::invalid_input(
            "repository.forbidden_reference",
            violations.join("; "),
        ));
    }
    Ok(0)
}

fn verify_atlas_artifacts(repo_root: &Path) -> Result<(), AppError> {
    crate::build_corpus(repo_root, None, crate::BuildMode::Check)?;
    let receipt: AtlasReceipt = serde_json::from_slice(
        &fs::read(repo_root.join(ATLAS_RECEIPT_PATH))
            .map_err(|error| AppError::io("repository.atlas", ATLAS_RECEIPT_PATH, error))?,
    )
    .map_err(|error| {
        AppError::invalid_input(
            "repository.atlas",
            format!("invalid Atlas export receipt: {error}"),
        )
    })?;
    if receipt.schema_version != "harp-atlas-export/v1" {
        return Err(AppError::invalid_input(
            "repository.atlas",
            "unsupported Atlas export receipt",
        ));
    }
    let corpus = sha256_file(&repo_root.join(CORPUS_PATH))?;
    let html = sha256_file(&repo_root.join(ATLAS_HTML_PATH))?;
    let app_inputs = atlas_input_digest(repo_root)?;
    if receipt.corpus_sha256 != corpus
        || receipt.html_sha256 != html
        || receipt.app_inputs_sha256 != app_inputs
    {
        return Err(AppError::invalid_input(
            "repository.atlas",
            format!(
                "Atlas export receipt is stale: corpus {}/{}, app {}/{}, html {}/{}",
                receipt.corpus_sha256,
                corpus,
                receipt.app_inputs_sha256,
                app_inputs,
                receipt.html_sha256,
                html
            ),
        ));
    }
    Ok(())
}

fn atlas_input_digest(repo_root: &Path) -> Result<String, AppError> {
    let atlas = repo_root.join("atlas");
    let mut inputs = [
        "index.html",
        "package.json",
        "scripts/export-static.mjs",
        "tsconfig.json",
        "vite.config.ts",
        "vitest.config.ts",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect::<Vec<_>>();
    for entry in WalkDir::new(atlas.join("src")).follow_links(false) {
        let entry = entry.map_err(|error| {
            AppError::external(
                "repository.atlas",
                format!("could not walk Atlas source: {error}"),
            )
        })?;
        if entry.file_type().is_file()
            && matches!(
                entry.path().extension().and_then(|value| value.to_str()),
                Some("css" | "ts" | "tsx")
            )
        {
            inputs.push(
                entry
                    .path()
                    .strip_prefix(&atlas)
                    .expect("Atlas file is relative")
                    .to_path_buf(),
            );
        }
    }
    inputs.sort();
    let mut digest = Sha256::new();
    for relative in inputs {
        let bytes = fs::read(atlas.join(&relative)).map_err(|error| {
            AppError::io(
                "repository.atlas",
                &format!("read Atlas input {}", relative.display()),
                error,
            )
        })?;
        digest.update(slash_path(&relative));
        digest.update([0]);
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn verify_import_receipt(repo_root: &Path, payload_sha256: &str) -> Result<(), AppError> {
    let receipt = fs::read_to_string(repo_root.join(IMPORT_RECEIPT))
        .map_err(|error| AppError::io("repository.import_receipt", IMPORT_RECEIPT, error))?;
    let marker = format!("`{payload_sha256}`");
    if !receipt.contains(&marker) {
        return Err(AppError::invalid_input(
            "repository.import_receipt",
            format!("import receipt payload digest is stale; expected {marker}"),
        ));
    }
    Ok(())
}

fn repository_payload_paths(repo_root: &Path) -> Result<Vec<PathBuf>, AppError> {
    let output = std::process::Command::new("git")
        .current_dir(repo_root)
        .args(["ls-files", "-z"])
        .output()
        .map_err(|error| {
            AppError::io(
                "repository.tracked_files",
                "enumerate tracked repository files",
                error,
            )
        })?;
    if !output.status.success() {
        return Err(AppError::external(
            "repository.tracked_files",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let mut paths = Vec::new();
    for raw in output.stdout.split(|byte| *byte == 0) {
        if raw.is_empty() {
            continue;
        }
        let text = std::str::from_utf8(raw).map_err(|error| {
            AppError::invalid_input(
                "repository.tracked_files",
                format!("tracked path is not UTF-8: {error}"),
            )
        })?;
        let relative = safe_relative(text, "tracked repository path")?;
        if relative == Path::new(IMPORT_RECEIPT) {
            continue;
        }
        let metadata = fs::symlink_metadata(repo_root.join(&relative)).map_err(|error| {
            AppError::io(
                "repository.tracked_files",
                &format!("inspect tracked file {}", relative.display()),
                error,
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(AppError::invalid_input(
                "repository.symlink",
                format!(
                    "repository payload cannot contain symlink {}",
                    relative.display()
                ),
            ));
        }
        if metadata.is_file() {
            paths.push(relative);
        }
    }
    Ok(paths)
}

fn percent_decode(value: &str) -> Result<String, AppError> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(AppError::invalid_input(
                    "repository.import_map",
                    "truncated percent escape",
                ));
            }
            let high = hex_value(bytes[index + 1])?;
            let low = hex_value(bytes[index + 2])?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|error| {
        AppError::invalid_input(
            "repository.import_map",
            format!("percent-decoded value is not UTF-8: {error}"),
        )
    })
}

fn hex_value(byte: u8) -> Result<u8, AppError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(AppError::invalid_input(
            "repository.import_map",
            "invalid percent escape",
        )),
    }
}

fn safe_relative(value: &str, label: &str) -> Result<PathBuf, AppError> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.as_os_str().is_empty()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(AppError::invalid_input(
            "repository.path",
            format!("{label} is not a normal repository-relative path: {value}"),
        ));
    }
    Ok(path.to_path_buf())
}

fn sha256_file(path: &Path) -> Result<String, AppError> {
    let bytes = fs::read(path)
        .map_err(|error| AppError::io("repository.digest", &path.display().to_string(), error))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
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
    use tempfile::TempDir;

    #[test]
    fn payload_digest_ignores_untracked_and_ignored_files() {
        let repo = TempDir::new().unwrap();
        run_git(repo.path(), ["init", "--quiet"]);
        fs::write(repo.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(repo.path().join("tracked.txt"), "tracked\n").unwrap();
        run_git(repo.path(), ["add", ".gitignore", "tracked.txt"]);

        let before = payload_digest(repo.path()).unwrap();
        fs::write(repo.path().join("ignored.txt"), "ignored\n").unwrap();
        fs::write(repo.path().join("untracked.txt"), "untracked\n").unwrap();
        let after = payload_digest(repo.path()).unwrap();

        assert_eq!(after, before);
    }

    #[test]
    fn forbidden_reference_check_allows_the_managed_rsi_root() {
        let repo = TempDir::new().unwrap();
        run_git(repo.path(), ["init", "--quiet"]);
        let document = repo.path().join("knowledge/rsi/intro.md");
        fs::create_dir_all(document.parent().unwrap()).unwrap();
        fs::write(document, "# Recursive improvement\n").unwrap();
        run_git(repo.path(), ["add", "knowledge/rsi/intro.md"]);

        assert_eq!(verify_forbidden_references(repo.path()).unwrap(), 0);
    }

    fn run_git<const N: usize>(root: &Path, arguments: [&str; N]) {
        let status = std::process::Command::new("git")
            .current_dir(root)
            .args(arguments)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
