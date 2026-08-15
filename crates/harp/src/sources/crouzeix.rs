use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use walkdir::WalkDir;

use super::{sha256_file, valid_digest, verify_file};
use crate::error::AppError;

pub(super) const ROOT: &str = "evidence/crouzeix_conjecture";

pub(super) const SOURCE_HEADER: &str = "schema_version\treceipt_id\tsource_id\tsource_class\trole\timmutable_identity\tsource_url\tupstream_path\tbytes\tsha256\tlocal_path\tobserved\tlicense_status\tredistribution_status";
pub(super) const VERIFICATION_HEADER: &str = "schema_version\treceipt_id\tsource_id\tsource_commit\tsource_tree\toperation\tcommand_sha256\tacquisition_script_sha256\tnormalization_version\ttoolchain\tmathlib_revision\tobserved_at_utc\texit_code\tresult\tlog_path\tlog_bytes\tlog_sha256";

#[derive(Debug)]
pub(super) struct Report {
    pub(super) source_receipts: usize,
    pub(super) verification_receipts: usize,
}

pub(super) fn verify(repo_root: &Path) -> Result<Report, AppError> {
    verify_exact_roster(repo_root)?;
    let source_receipts = verify_source_manifest(repo_root)?;
    let verification_receipts = verify_verification_manifest(repo_root)?;
    Ok(Report {
        source_receipts,
        verification_receipts,
    })
}

fn verify_exact_roster(repo_root: &Path) -> Result<(), AppError> {
    let expected = [
        "PROVENANCE.md",
        "acquire.sh",
        "source_manifest.tsv",
        "verification/jin-565b6a3-build.log",
        "verification/jin-565b6a3-scan.log",
        "verification/jin-9df0783-build.log",
        "verification/jin-9df0783-scan.log",
        "verification_manifest.tsv",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let root = repo_root.join(ROOT);
    let mut actual = BTreeSet::new();
    for entry in WalkDir::new(&root).follow_links(false) {
        let entry = entry.map_err(|error| {
            invalid(
                "sources.crouzeix.roster",
                format!("could not walk {}: {error}", root.display()),
            )
        })?;
        if entry.file_type().is_symlink() {
            return Err(invalid(
                "sources.crouzeix.roster",
                format!(
                    "Crouzeix evidence cannot contain symlink {}",
                    entry.path().display()
                ),
            ));
        }
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(&root)
                .expect("walked entry stays beneath root");
            actual.insert(slash_path(relative));
        }
    }
    if actual != expected {
        return Err(invalid(
            "sources.crouzeix.roster",
            format!(
                "Crouzeix evidence file roster differs: expected={expected:?}, actual={actual:?}"
            ),
        ));
    }
    Ok(())
}

fn verify_source_manifest(repo_root: &Path) -> Result<usize, AppError> {
    let relative = Path::new(ROOT).join("source_manifest.tsv");
    let rows = read_manifest(
        repo_root,
        &relative,
        SOURCE_HEADER,
        "sources.crouzeix.source_manifest",
    )?;
    let mut receipt_ids = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for (index, columns) in rows.iter().enumerate().skip(1) {
        let line = index + 1;
        if columns.len() != 14 {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "expected fourteen columns",
            ));
        }
        let [schema, receipt_id, source_id, source_class, role, identity, source_url, upstream_path, bytes, digest, local_path, observed, license, redistribution] =
            columns.as_slice()
        else {
            unreachable!("column count checked");
        };
        require_canonical_fields("sources.crouzeix.source_manifest", &relative, line, columns)?;
        if schema != "crouzeix-source-receipt/v1" {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "unsupported schema version",
            ));
        }
        if !valid_receipt_id(receipt_id) || !receipt_ids.insert(receipt_id) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "receipt_id must be unique canonical uppercase text",
            ));
        }
        if !valid_receipt_id(source_id) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "source_id must be canonical uppercase text",
            ));
        }
        if !matches!(
            source_class.as_str(),
            "git-artifact" | "arxiv-artifact" | "journal-record" | "dated-page"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid source_class",
            ));
        }
        if !matches!(
            role.as_str(),
            "manuscript"
                | "source"
                | "pdf"
                | "metadata"
                | "formalization"
                | "prompt"
                | "history"
                | "prerequisite"
                | "license"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid role",
            ));
        }
        validate_identity_url(identity, source_url).map_err(|detail| {
            manifest_error("sources.crouzeix.source_manifest", &relative, line, &detail)
        })?;
        if upstream_path != "-" {
            safe_relative(upstream_path).map_err(|detail| {
                manifest_error("sources.crouzeix.source_manifest", &relative, line, &detail)
            })?;
        }
        if local_path != "-" {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "upstream artifacts must remain remote-only",
            ));
        }
        if bytes == "-" || digest == "-" {
            if bytes != "-"
                || digest != "-"
                || source_class != "dated-page"
                || redistribution != "metadata-only"
            {
                return Err(manifest_error(
                    "sources.crouzeix.source_manifest",
                    &relative,
                    line,
                    "missing bytes and digest are allowed only for metadata-only dated pages",
                ));
            }
        } else if bytes.parse::<u64>().is_err() || !valid_digest(digest) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid byte count or sha256",
            ));
        }
        if !valid_date(observed) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "observed must be an ISO date",
            ));
        }
        if !matches!(
            license.as_str(),
            "not-present-at-revision"
                | "arxiv-nonexclusive"
                | "publisher-record"
                | "unknown"
                | "not-redistributable"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid license_status",
            ));
        }
        if !matches!(
            redistribution.as_str(),
            "remote-only" | "quotation-only" | "metadata-only"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid redistribution_status",
            ));
        }
        if !artifacts.insert((source_id, identity, upstream_path)) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "duplicate source identity and upstream path",
            ));
        }
    }
    if rows.len() == 1 {
        return Err(manifest_error(
            "sources.crouzeix.source_manifest",
            &relative,
            1,
            "manifest has no receipts",
        ));
    }
    Ok(rows.len() - 1)
}

fn verify_verification_manifest(repo_root: &Path) -> Result<usize, AppError> {
    let relative = Path::new(ROOT).join("verification_manifest.tsv");
    let rows = read_manifest(
        repo_root,
        &relative,
        VERIFICATION_HEADER,
        "sources.crouzeix.verification_manifest",
    )?;
    let script_digest = sha256_file(&repo_root.join(ROOT).join("acquire.sh"))?;
    let mut receipt_ids = BTreeSet::new();
    for (index, columns) in rows.iter().enumerate().skip(1) {
        let line = index + 1;
        if columns.len() != 17 {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "expected seventeen columns",
            ));
        }
        let [schema, receipt_id, source_id, source_commit, source_tree, operation, command_digest, acquisition_digest, normalization, toolchain, mathlib_revision, observed_at, exit_code, result, log_path, log_bytes, log_digest] =
            columns.as_slice()
        else {
            unreachable!("column count checked");
        };
        require_canonical_fields(
            "sources.crouzeix.verification_manifest",
            &relative,
            line,
            columns,
        )?;
        if schema != "crouzeix-verification-receipt/v1" {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "unsupported schema version",
            ));
        }
        if !valid_receipt_id(receipt_id) || !receipt_ids.insert(receipt_id) {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "receipt_id must be unique canonical uppercase text",
            ));
        }
        if !valid_receipt_id(source_id)
            || !valid_hex(source_commit, 40)
            || !valid_hex(source_tree, 40)
        {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid source identity",
            ));
        }
        if !matches!(operation.as_str(), "lean-build" | "source-scan")
            || !valid_digest(command_digest)
            || acquisition_digest != &script_digest
            || normalization != "crouzeix-log-normalization/v1"
            || toolchain.is_empty()
            || !valid_hex(mathlib_revision, 40)
            || !valid_timestamp(observed_at)
        {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid operation, digest, toolchain, revision, or timestamp",
            ));
        }
        if !matches!(result.as_str(), "passed" | "failed" | "blocked")
            || (result == "blocked" && exit_code != "-")
            || (result != "blocked" && exit_code.parse::<i32>().is_err())
        {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid result and exit-code combination",
            ));
        }
        let log_path = safe_relative(log_path).map_err(|detail| {
            manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                &detail,
            )
        })?;
        let expected_root = Path::new(ROOT).join("verification");
        if !log_path.starts_with("verification/") {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "log path must stay beneath verification/",
            ));
        }
        let expected_bytes = log_bytes.parse::<u64>().map_err(|_| {
            manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid log byte count",
            )
        })?;
        if !valid_digest(log_digest) {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid log sha256",
            ));
        }
        verify_file(
            repo_root,
            &expected_root.join(
                log_path
                    .strip_prefix("verification")
                    .expect("prefix checked"),
            ),
            expected_bytes,
            log_digest,
            "Crouzeix verification log",
        )?;
    }
    if rows.len() == 1 {
        return Err(manifest_error(
            "sources.crouzeix.verification_manifest",
            &relative,
            1,
            "manifest has no receipts",
        ));
    }
    Ok(rows.len() - 1)
}

fn read_manifest(
    repo_root: &Path,
    relative: &Path,
    expected_header: &str,
    code: &'static str,
) -> Result<Vec<Vec<String>>, AppError> {
    let bytes = fs::read(repo_root.join(relative))
        .map_err(|error| AppError::io(code, &format!("read {}", relative.display()), error))?;
    if bytes.is_empty()
        || !bytes.ends_with(b"\n")
        || bytes.contains(&b'\0')
        || bytes.contains(&b'\r')
    {
        return Err(manifest_error(
            code,
            relative,
            1,
            "manifest must be nonempty LF-terminated UTF-8 without NUL or CR",
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| manifest_error(code, relative, 1, "manifest must be UTF-8"))?;
    let rows = text
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if rows.first().map(|row| row.join("\t")).as_deref() != Some(expected_header) {
        return Err(manifest_error(code, relative, 1, "unexpected header"));
    }
    Ok(rows)
}

fn require_canonical_fields(
    code: &'static str,
    manifest: &Path,
    line: usize,
    fields: &[String],
) -> Result<(), AppError> {
    if fields
        .iter()
        .any(|field| field.is_empty() || field.trim() != field)
    {
        return Err(manifest_error(
            code,
            manifest,
            line,
            "fields must be nonempty canonical trimmed strings",
        ));
    }
    Ok(())
}

fn validate_identity_url(identity: &str, source_url: &str) -> Result<(), String> {
    if !source_url.starts_with("https://") {
        return Err("source_url must use HTTPS".to_owned());
    }
    if let Some(revision) = identity.strip_prefix("git:") {
        if !valid_hex(revision, 40) || !source_url.contains(revision) {
            return Err("Git URL must contain its declared 40-hex revision".to_owned());
        }
    } else if let Some(version) = identity.strip_prefix("arxiv:") {
        let Some((id, suffix)) = version.rsplit_once('v') else {
            return Err("arXiv identity must end with vN".to_owned());
        };
        if id.is_empty()
            || suffix.is_empty()
            || !suffix.bytes().all(|byte| byte.is_ascii_digit())
            || !source_url.contains(version)
        {
            return Err("arXiv URL must contain its exact version".to_owned());
        }
    } else if let Some(doi) = identity.strip_prefix("doi:") {
        if doi.is_empty() || !source_url.contains(doi) {
            return Err("DOI URL must contain its declared identity".to_owned());
        }
    } else if !identity.strip_prefix("sha256:").is_some_and(valid_digest) {
        return Err("unsupported immutable identity".to_owned());
    }
    Ok(())
}

fn safe_relative(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.as_os_str().is_empty()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "path must be repository-relative normal text: {value}"
        ));
    }
    Ok(path.to_path_buf())
}

fn valid_receipt_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn valid_timestamp(value: &str) -> bool {
    value.len() >= 20
        && value.ends_with('Z')
        && value.as_bytes().get(10) == Some(&b'T')
        && valid_date(&value[..10])
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn invalid(code: &'static str, detail: String) -> AppError {
    AppError::invalid_input(code, detail)
}

fn manifest_error(code: &'static str, manifest: &Path, line: usize, detail: &str) -> AppError {
    invalid(
        code,
        format!("{} line {line}: {detail}", manifest.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest as _, Sha256};
    use tempfile::TempDir;

    fn sha256_file(path: &Path) -> String {
        format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
    }

    fn fixture() -> TempDir {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path().join(ROOT);
        fs::create_dir_all(root.join("verification")).unwrap();
        fs::write(root.join("PROVENANCE.md"), "# Provenance\n").unwrap();
        fs::write(root.join("acquire.sh"), "#!/bin/sh\nset -eu\n").unwrap();
        fs::write(
            root.join("source_manifest.tsv"),
            format!(
                "{SOURCE_HEADER}\n\
                 crouzeix-source-receipt/v1\tJIN-HEAD-README\tJIN-REPO-HEAD\tgit-artifact\tmetadata\t\
                 git:9df07838327b988e3924453daa29c8cd726d34b0\t\
                 https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/README.md\t\
                 README.md\t357\t{}\t-\t2026-08-14\tnot-present-at-revision\tmetadata-only\n",
                "0".repeat(64),
            ),
        )
        .unwrap();
        let script_digest = sha256_file(&root.join("acquire.sh"));
        let empty_digest = format!("{:x}", Sha256::digest([]));
        let mut verification = format!("{VERIFICATION_HEADER}\n");
        for (receipt, commit, operation, log) in [
            (
                "JIN-565-BUILD",
                "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                "lean-build",
                "verification/jin-565b6a3-build.log",
            ),
            (
                "JIN-565-SCAN",
                "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                "source-scan",
                "verification/jin-565b6a3-scan.log",
            ),
            (
                "JIN-HEAD-BUILD",
                "9df07838327b988e3924453daa29c8cd726d34b0",
                "lean-build",
                "verification/jin-9df0783-build.log",
            ),
            (
                "JIN-HEAD-SCAN",
                "9df07838327b988e3924453daa29c8cd726d34b0",
                "source-scan",
                "verification/jin-9df0783-scan.log",
            ),
        ] {
            fs::write(repo.path().join(ROOT).join(log), "").unwrap();
            verification.push_str(&format!(
                "crouzeix-verification-receipt/v1\t{receipt}\tJIN-REPO-HEAD\t{commit}\t{}\t{operation}\t{}\t{}\t\
                 crouzeix-log-normalization/v1\tleanprover/lean4:v4.28.0\t\
                 8f9d9cff6bd728b17a24e163c9402775d9e6a365\t2026-08-14T00:00:00Z\t0\tpassed\t{log}\t0\t{empty_digest}\n",
                "1".repeat(40),
                "2".repeat(64),
                script_digest,
            ));
        }
        fs::write(root.join("verification_manifest.tsv"), verification).unwrap();
        repo
    }

    fn replace_source_url(repo_root: &Path, replacement: &str) {
        let path = repo_root.join(ROOT).join("source_manifest.tsv");
        let text = fs::read_to_string(&path).unwrap();
        let mut rows = text.lines();
        let header = rows.next().unwrap();
        let mut fields = rows.next().unwrap().split('\t').collect::<Vec<_>>();
        fields[6] = replacement;
        fs::write(path, format!("{header}\n{}\n", fields.join("\t"))).unwrap();
    }

    #[test]
    fn rejects_unknown_files_and_symlinks_in_the_evidence_root() {
        let repo = fixture();
        fs::write(
            repo.path()
                .join("evidence/crouzeix_conjecture/copied-source.lean"),
            "theorem copied : True := by trivial\n",
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.roster");
    }

    #[test]
    fn source_manifest_requires_identity_bound_urls_and_unique_receipts() {
        let repo = fixture();
        replace_source_url(
            repo.path(),
            "https://github.com/jinshanmu/CrouzeixConjecture/blob/main/README.md",
        );

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.source_manifest");
    }

    #[test]
    fn verification_receipts_bind_commands_logs_and_acquisition_script() {
        let repo = fixture();
        fs::write(
            repo.path().join("evidence/crouzeix_conjecture/acquire.sh"),
            "#!/bin/sh\nexit 0\n# changed\n",
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.verification_manifest");
    }
}
