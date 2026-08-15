use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use tempfile::Builder;
use walkdir::WalkDir;

use crate::error::AppError;

const INDEX_PATH: &str = "target/search/harp.sqlite";
const RECEIPT_PATH: &str = "target/search/receipt.json";
const RECEIPT_SCHEMA: &str = "harp-search/v1";
const SMOKE_QUERY: &str = "recursive improvement";
const MAX_DOCUMENT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchReceipt {
    pub schema_version: String,
    pub corpus_sha256: String,
    pub document_count: usize,
    pub sqlite_version: String,
    pub index_path: String,
    pub smoke_query_result: SmokeQueryResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SmokeQueryResult {
    pub query: String,
    pub result_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub document_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    pub score: f64,
    pub snippet: String,
}

struct SearchDocument {
    path: String,
    title: String,
    document_kind: &'static str,
    source_id: Option<String>,
    body: String,
}

pub fn refresh(repo_root: &Path) -> Result<SearchReceipt, AppError> {
    let (documents, corpus_sha256) = load_documents(repo_root)?;
    let search_root = repo_root.join("target/search");
    fs::create_dir_all(&search_root)
        .map_err(|error| AppError::io("search.directory", "search directory", error))?;
    reject_symlink(&search_root, "search directory")?;

    let temporary = Builder::new()
        .prefix(".harp-index-")
        .suffix(".sqlite")
        .tempfile_in(&search_root)
        .map_err(|error| AppError::io("search.temp", "temporary search index", error))?;
    let temporary_path = temporary.path().to_path_buf();
    drop(temporary);

    let connection = open_database(&temporary_path, false)?;
    connection
        .execute_batch(
            "
            PRAGMA journal_mode = DELETE;
            PRAGMA synchronous = FULL;
            CREATE VIRTUAL TABLE documents USING fts5(
                path UNINDEXED,
                title,
                document_kind UNINDEXED,
                source_id UNINDEXED,
                body,
                tokenize = 'unicode61'
            );
            ",
        )
        .map_err(database_error)?;
    {
        let transaction = connection.unchecked_transaction().map_err(database_error)?;
        {
            let mut insert = transaction
                .prepare(
                    "INSERT INTO documents(path, title, document_kind, source_id, body)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .map_err(database_error)?;
            for document in &documents {
                insert
                    .execute(params![
                        document.path,
                        document.title,
                        document.document_kind,
                        document.source_id,
                        document.body
                    ])
                    .map_err(database_error)?;
            }
        }
        transaction.commit().map_err(database_error)?;
    }
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(database_error)?;
    if integrity != "ok" {
        return Err(AppError::external(
            "search.integrity",
            format!("search index integrity check failed: {integrity}"),
        ));
    }
    let smoke_count = query_count(&connection, SMOKE_QUERY)?;
    if smoke_count == 0 {
        return Err(AppError::invalid_input(
            "search.smoke",
            format!("search smoke query {SMOKE_QUERY:?} returned no results"),
        ));
    }
    let sqlite_version: String = connection
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .map_err(database_error)?;
    connection
        .execute_batch("PRAGMA optimize;")
        .map_err(database_error)?;
    drop(connection);

    let index_path = repo_root.join(INDEX_PATH);
    reject_existing_symlink(&index_path, "search index")?;
    fs::rename(&temporary_path, &index_path)
        .map_err(|error| AppError::io("search.publish", "publish search index", error))?;

    let receipt = SearchReceipt {
        schema_version: RECEIPT_SCHEMA.to_owned(),
        corpus_sha256,
        document_count: documents.len(),
        sqlite_version,
        index_path: INDEX_PATH.to_owned(),
        smoke_query_result: SmokeQueryResult {
            query: SMOKE_QUERY.to_owned(),
            result_count: smoke_count,
        },
    };
    write_receipt(repo_root, &receipt)?;
    Ok(receipt)
}

pub fn status(repo_root: &Path) -> Result<SearchReceipt, AppError> {
    let receipt_path = repo_root.join(RECEIPT_PATH);
    let index_path = repo_root.join(INDEX_PATH);
    if !receipt_path.is_file() || !index_path.is_file() {
        return Err(AppError::invalid_input(
            "search.missing",
            "search index is missing; run `harp search refresh`",
        ));
    }
    reject_existing_symlink(&receipt_path, "search receipt")?;
    reject_existing_symlink(&index_path, "search index")?;
    let receipt_bytes = fs::read(&receipt_path)
        .map_err(|error| AppError::io("search.receipt", "read search receipt", error))?;
    let receipt: SearchReceipt = serde_json::from_slice(&receipt_bytes).map_err(|error| {
        AppError::invalid_input("search.receipt", format!("invalid search receipt: {error}"))
    })?;
    if receipt.schema_version != RECEIPT_SCHEMA || receipt.index_path != INDEX_PATH {
        return Err(AppError::invalid_input(
            "search.receipt",
            "search receipt has an unsupported schema or index path",
        ));
    }
    let (documents, corpus_sha256) = load_documents(repo_root)?;
    if receipt.corpus_sha256 != corpus_sha256 || receipt.document_count != documents.len() {
        return Err(AppError::invalid_input(
            "search.stale",
            "search index is stale; run `harp search refresh`",
        ));
    }
    let connection = open_database(&index_path, true)?;
    let smoke_count = query_count(&connection, &receipt.smoke_query_result.query)?;
    if smoke_count == 0 || smoke_count != receipt.smoke_query_result.result_count {
        return Err(AppError::invalid_input(
            "search.stale",
            "search index is stale; smoke-query receipt does not match",
        ));
    }
    Ok(receipt)
}

pub fn query(repo_root: &Path, query: &str, limit: usize) -> Result<Vec<SearchResult>, AppError> {
    status(repo_root)?;
    if query.trim().is_empty() {
        return Err(AppError::invalid_input(
            "search.query",
            "search query must not be empty",
        ));
    }
    if !(1..=100).contains(&limit) {
        return Err(AppError::invalid_input(
            "search.limit",
            "search limit must be between 1 and 100",
        ));
    }
    let connection = open_database(&repo_root.join(INDEX_PATH), true)?;
    let mut statement = connection
        .prepare(
            "SELECT path, title, document_kind, source_id,
                    bm25(documents) AS score,
                    snippet(documents, 4, '[', ']', ' … ', 24) AS snippet
             FROM documents
             WHERE documents MATCH ?1
             ORDER BY score
             LIMIT ?2",
        )
        .map_err(database_error)?;
    let sql_limit = i64::try_from(limit).map_err(|_| {
        AppError::invalid_input("search.limit", "search limit does not fit SQLite integer")
    })?;
    let rows = statement
        .query_map(params![query, sql_limit], |row| {
            Ok(SearchResult {
                path: row.get(0)?,
                title: row.get(1)?,
                document_kind: row.get(2)?,
                source_id: row.get(3)?,
                score: row.get(4)?,
                snippet: row.get(5)?,
            })
        })
        .map_err(database_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(database_error)
}

fn load_documents(repo_root: &Path) -> Result<(Vec<SearchDocument>, String), AppError> {
    let roots = [
        ("knowledge/rsi", "canonical-markdown", "md"),
        ("knowledge/darwin_godel_machine", "canonical-markdown", "md"),
        ("knowledge/meta_harness", "canonical-markdown", "md"),
        ("knowledge/harness_benchmarks", "canonical-markdown", "md"),
        (
            "knowledge/self_improving_agents_survey",
            "canonical-markdown",
            "md",
        ),
        (
            "knowledge/verified_coevolution_agenda",
            "canonical-markdown",
            "md",
        ),
        ("knowledge/agentic_engineering", "canonical-markdown", "md"),
        ("knowledge/crouzeix_conjecture", "canonical-markdown", "md"),
        ("knowledge/darwinx", "canonical-markdown", "md"),
        (
            "knowledge/mathematical_foundations",
            "canonical-markdown",
            "md",
        ),
        (
            "knowledge/harp_knowledge_home.md",
            "canonical-markdown",
            "md",
        ),
        ("evidence/weng/text", "weng-source", "txt"),
        ("evidence/rlm/text", "rlm-source", "txt"),
    ];
    let mut paths = Vec::new();
    for (root, kind, extension) in roots {
        let absolute = repo_root.join(root);
        if !absolute.exists() {
            continue;
        }
        reject_symlink(&absolute, root)?;
        if absolute.is_file() {
            if absolute.extension().and_then(|value| value.to_str()) == Some(extension) {
                paths.push((absolute, kind));
            }
            continue;
        }
        for entry in WalkDir::new(&absolute).follow_links(false) {
            let entry = entry.map_err(|error| {
                AppError::external("search.walk", format!("could not walk {root}: {error}"))
            })?;
            if entry.file_type().is_symlink() {
                return Err(AppError::invalid_input(
                    "search.symlink",
                    format!(
                        "search corpus cannot contain symlink {}",
                        entry.path().display()
                    ),
                ));
            }
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some(extension)
            {
                paths.push((entry.path().to_path_buf(), kind));
            }
        }
    }
    paths.sort_by(|left, right| left.0.cmp(&right.0));

    let mut digest = Sha256::new();
    let mut documents = Vec::with_capacity(paths.len());
    for (absolute, kind) in paths {
        let metadata = fs::metadata(&absolute)
            .map_err(|error| AppError::io("search.metadata", "search document", error))?;
        if metadata.len() > MAX_DOCUMENT_BYTES {
            return Err(AppError::invalid_input(
                "search.document_size",
                format!("search document is too large: {}", absolute.display()),
            ));
        }
        let bytes = fs::read(&absolute)
            .map_err(|error| AppError::io("search.read", "search document", error))?;
        let relative = absolute.strip_prefix(repo_root).map_err(|_| {
            AppError::invalid_input("search.path", "search document escaped repository")
        })?;
        let path = slash_path(relative);
        digest.update((path.len() as u64).to_le_bytes());
        digest.update(path.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(&bytes);
        let body = String::from_utf8(bytes).map_err(|error| {
            AppError::invalid_input(
                "search.utf8",
                format!("search document {path} is not UTF-8: {error}"),
            )
        })?;
        documents.push(SearchDocument {
            title: document_title(&body, relative),
            source_id: (kind != "canonical-markdown").then(|| source_id(relative)),
            path,
            document_kind: kind,
            body,
        });
    }
    Ok((documents, format!("{:x}", digest.finalize())))
}

fn document_title(body: &str, path: &Path) -> String {
    if let Some(title) = body
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
    {
        return title.to_owned();
    }
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .replace(['-', '_'], " ")
}

fn source_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown")
        .replace('_', "-")
        .to_ascii_uppercase()
}

fn query_count(connection: &Connection, query: &str) -> Result<usize, AppError> {
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM documents WHERE documents MATCH ?1",
            [query],
            |row| row.get(0),
        )
        .map_err(database_error)?;
    usize::try_from(count).map_err(|_| {
        AppError::external(
            "search.sqlite",
            "SQLite returned an invalid negative or oversized result count",
        )
    })
}

fn open_database(path: &Path, read_only: bool) -> Result<Connection, AppError> {
    let flags = if read_only {
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
    } | OpenFlags::SQLITE_OPEN_NOFOLLOW;
    Connection::open_with_flags(path, flags).map_err(database_error)
}

fn write_receipt(repo_root: &Path, receipt: &SearchReceipt) -> Result<(), AppError> {
    let receipt_path = repo_root.join(RECEIPT_PATH);
    reject_existing_symlink(&receipt_path, "search receipt")?;
    let parent = receipt_path.parent().expect("receipt has parent");
    let mut temporary = Builder::new()
        .prefix(".harp-receipt-")
        .tempfile_in(parent)
        .map_err(|error| AppError::io("search.receipt_temp", "search receipt", error))?;
    serde_json::to_writer_pretty(&mut temporary, receipt).map_err(|error| {
        AppError::external(
            "search.receipt_write",
            format!("could not serialize search receipt: {error}"),
        )
    })?;
    use std::io::Write as _;
    temporary
        .write_all(b"\n")
        .and_then(|()| temporary.as_file_mut().sync_all())
        .map_err(|error| AppError::io("search.receipt_write", "search receipt", error))?;
    temporary
        .persist(&receipt_path)
        .map_err(|error| AppError::io("search.receipt_publish", "search receipt", error.error))?;
    Ok(())
}

fn reject_symlink(path: &Path, label: &str) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| AppError::io("search.metadata", label, error))?;
    if metadata.file_type().is_symlink() {
        return Err(AppError::invalid_input(
            "search.symlink",
            format!("{label} cannot be a symlink"),
        ));
    }
    Ok(())
}

fn reject_existing_symlink(path: &Path, label: &str) -> Result<(), AppError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(AppError::invalid_input(
            "search.symlink",
            format!("{label} cannot be a symlink"),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::io("search.metadata", label, error)),
    }
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn database_error(error: rusqlite::Error) -> AppError {
    AppError::external("search.sqlite", format!("SQLite search error: {error}"))
}
