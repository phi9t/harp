use super::obsidian::{parse_wiki_links, resolve_wiki_link};
use super::render::{heading_ids_for_source, markdown_body};
use super::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::path::Component;
pub(super) struct ValidatedInputs {
    pub(super) canonical_sources: BTreeMap<String, ValidatedCanonicalSource>,
    pub(super) registered_document_ids: BTreeMap<String, String>,
    pub(super) registered_document_aliases: BTreeMap<String, String>,
}
pub(super) struct ValidatedCanonicalSource {
    pub(super) path: String,
    pub(super) markdown: String,
    pub(super) body_sha256: String,
}
fn validate_no_content_markdown(repository: &HeldDirectory) -> Result<(), AppError> {
    let markdown = repository.regular_files_with_extension(
        Path::new(CONTENT_ROOT),
        "md",
        "structured content",
    )?;
    if let Some(path) = markdown.first() {
        return Err(invalid(
            "knowledge.rsi.content_markdown",
            format!(
                "technical Markdown must live under knowledge/, not {}",
                path.display()
            ),
        ));
    }
    Ok(())
}

pub(super) fn load(repository: &HeldDirectory) -> Result<ValidatedInputs, AppError> {
    validate_no_content_markdown(repository)?;
    let registration = super::registration::static_registration()?;
    let textbook = crate::crouzeix_textbook::validated_registration(repository)
        .map_err(textbook_registration_error)?;
    let registered_document_aliases = textbook
        .iter()
        .map(|d| (d.legacy_alias.clone(), d.canonical_id.clone()))
        .collect();
    let mut registered_document_ids = registration
        .auxiliary_documents()
        .map(|d| (d.path.clone(), d.id.clone()))
        .collect::<BTreeMap<_, _>>();
    registered_document_ids.extend(
        textbook
            .iter()
            .map(|d| (d.canonical_path.clone(), d.canonical_id.clone())),
    );
    let mut paths = registration
        .reader_routes()
        .chain(registration.auxiliary_documents())
        .map(|d| d.path.clone())
        .collect::<BTreeSet<_>>();
    paths.extend(textbook.iter().map(|d| d.canonical_path.clone()));
    if repository
        .read_optional_regular_file_bounded(
            Path::new(super::KNOWLEDGE_HOME.2),
            "knowledge home",
            MAX_MARKDOWN_BYTES,
        )?
        .is_some()
    {
        paths.insert(super::KNOWLEDGE_HOME.2.into());
    }
    let mut canonical_sources = BTreeMap::new();
    for path in paths {
        let markdown = match textbook.iter().find(|d| d.canonical_path == path) {
            Some(d) => d.markdown.clone(),
            None => read_canonical_markdown(repository, &path)?,
        };
        if markdown.contains('\0') || markdown.contains('\r') {
            return Err(invalid(
                "knowledge.rsi.canonical_encoding",
                format!("{path} must use LF and contain no NUL"),
            ));
        }
        let body = markdown_body(&markdown, &path)?;
        validate_local_links(&path, body, repository)?;
        heading_ids_for_source(body, &path)?;
        document_metadata(&markdown, &path)?;
        canonical_sources.insert(
            path.clone(),
            ValidatedCanonicalSource {
                path,
                body_sha256: sha256(body.as_bytes()),
                markdown,
            },
        );
    }
    Ok(ValidatedInputs {
        canonical_sources,
        registered_document_ids,
        registered_document_aliases,
    })
}
fn textbook_registration_error(
    diagnostics: Vec<crate::crouzeix_textbook::TextbookDiagnostic>,
) -> AppError {
    let detail = diagnostics
        .into_iter()
        .map(|diagnostic| {
            format!(
                "{} {}: expected {}; observed {}",
                diagnostic.code, diagnostic.field, diagnostic.expected, diagnostic.observed
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    invalid("knowledge.crouzeix_textbook.registration", detail)
}

pub(super) fn document_metadata(markdown: &str, path: &str) -> Result<DocumentMetadata, AppError> {
    document_metadata_with_frontmatter_mode(markdown, path, false)
}

fn document_metadata_with_frontmatter_mode(
    markdown: &str,
    path: &str,
    require_canonical_frontmatter: bool,
) -> Result<DocumentMetadata, AppError> {
    let default_id = super::registration::static_registration()?
        .document_id_by_path(path)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document")
                .replace('_', "-")
        });
    let mut metadata = DocumentMetadata {
        id: default_id,
        kind: "technical-document".to_owned(),
        status: "legacy".to_owned(),
        tags: Vec::new(),
        confidence: "medium".to_owned(),
        mode: None,
        source_ids: Vec::new(),
    };
    let Some(fields) = frontmatter_fields(markdown, path, require_canonical_frontmatter)? else {
        if require_canonical_frontmatter {
            return Err(invalid(
                "knowledge.rsi.frontmatter",
                format!("{path} must have canonical frontmatter"),
            ));
        }
        return Ok(metadata);
    };
    if require_canonical_frontmatter && !fields.contains_key("id") {
        return Err(invalid(
            "knowledge.rsi.frontmatter",
            format!("{path} frontmatter must contain id"),
        ));
    }
    apply_frontmatter_fields(&mut metadata, fields, path)?;
    Ok(metadata)
}

pub(crate) fn canonical_frontmatter_identity(
    markdown: &str,
    path: &str,
) -> Result<String, AppError> {
    Ok(document_metadata_with_frontmatter_mode(markdown, path, true)?.id)
}

fn frontmatter_fields<'a>(
    markdown: &'a str,
    path: &str,
    strict: bool,
) -> Result<Option<BTreeMap<&'a str, &'a str>>, AppError> {
    let Some(frontmatter) = markdown.strip_prefix("---\n") else {
        return Ok(None);
    };
    let (frontmatter, _) = frontmatter.split_once("\n---\n").ok_or_else(|| {
        invalid(
            "knowledge.rsi.frontmatter",
            format!("{path} has unclosed YAML frontmatter"),
        )
    })?;
    let mut fields = BTreeMap::new();
    for line in frontmatter.lines() {
        if line.is_empty() || line.starts_with([' ', '\t']) {
            if strict {
                return Err(invalid(
                    "knowledge.rsi.frontmatter",
                    format!("{path} frontmatter entries must be top-level nonempty lines"),
                ));
            }
            return Ok(None);
        }
        let (key, value) = line.split_once(':').ok_or_else(|| {
            invalid(
                "knowledge.rsi.frontmatter",
                format!("{path} frontmatter entry has no colon"),
            )
        })?;
        if key.is_empty() || key.trim() != key || value.is_empty() {
            if strict {
                return Err(invalid(
                    "knowledge.rsi.frontmatter",
                    format!("{path} has an invalid frontmatter entry"),
                ));
            }
            return Ok(None);
        }
        if fields.insert(key, value.trim()).is_some() {
            if strict {
                return Err(invalid(
                    "knowledge.rsi.frontmatter",
                    format!("{path} has a duplicate frontmatter field: {key}"),
                ));
            }
            return Ok(None);
        }
    }
    Ok(Some(fields))
}

fn apply_frontmatter_fields(
    metadata: &mut DocumentMetadata,
    fields: BTreeMap<&str, &str>,
    path: &str,
) -> Result<(), AppError> {
    for (key, value) in fields {
        match key {
            "id" => metadata.id = metadata_id(value, path, key)?,
            "type" => metadata.kind = metadata_text(value, path, key)?,
            "status" => metadata.status = metadata_text(value, path, key)?,
            "tags" => metadata.tags = metadata_list(value, path, key)?,
            "confidence" => {
                if !matches!(value, "low" | "medium" | "high") {
                    return Err(invalid(
                        "knowledge.rsi.frontmatter",
                        format!("{path} confidence must be low, medium, or high"),
                    ));
                }
                metadata.confidence = value.to_owned();
            }
            "mode" => metadata.mode = Some(metadata_text(value, path, key)?),
            "source_ids" => metadata.source_ids = metadata_list(value, path, key)?,
            _ => {}
        }
    }
    metadata.tags.sort();
    metadata.tags.dedup();
    metadata.source_ids.sort();
    metadata.source_ids.dedup();
    Ok(())
}

fn metadata_id(value: &str, path: &str, key: &str) -> Result<String, AppError> {
    let value = metadata_text(value, path, key)?;
    if !valid_id(&value) {
        return Err(invalid(
            "knowledge.rsi.frontmatter",
            format!("{path} {key} must be a canonical ID"),
        ));
    }
    Ok(value)
}

fn metadata_text(value: &str, path: &str, key: &str) -> Result<String, AppError> {
    if value.is_empty()
        || value.trim() != value
        || value.starts_with('[')
        || value.starts_with('{')
        || value.contains('\0')
    {
        return Err(invalid(
            "knowledge.rsi.frontmatter",
            format!("{path} {key} must be a one-line scalar"),
        ));
    }
    Ok(value.to_owned())
}

fn metadata_list(value: &str, path: &str, key: &str) -> Result<Vec<String>, AppError> {
    let Some(items) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(invalid(
            "knowledge.rsi.frontmatter",
            format!("{path} {key} must be a one-line list"),
        ));
    };
    if items.is_empty() {
        return Ok(Vec::new());
    }
    items
        .split(',')
        .map(|item| metadata_text(item.trim(), path, key))
        .collect()
}

pub(super) fn read_canonical_markdown(
    repository: &HeldDirectory,
    relative: &str,
) -> Result<String, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(relative),
            "canonical RSI Markdown",
            MAX_MARKDOWN_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.canonical_missing",
                format!("canonical Markdown is missing: {relative}"),
            )
        })?;
    String::from_utf8(bytes).map_err(|_| {
        invalid(
            "knowledge.rsi.canonical_encoding",
            format!("canonical Markdown is not readable UTF-8: {relative}"),
        )
    })
}

pub(super) fn validate_local_links(
    path: &str,
    markdown: &str,
    repository: &HeldDirectory,
) -> Result<(), AppError> {
    for link in parse_wiki_links(markdown).map_err(|error| {
        invalid(
            "knowledge.obsidian.syntax",
            format!("{path} has invalid Obsidian wikilink: {error}"),
        )
    })? {
        resolve_wiki_link(repository, path, &link)?;
    }
    let parent = Path::new(path)
        .parent()
        .expect("canonical files have parents");
    let options = Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES;
    for event in Parser::new_ext(markdown, options) {
        let Event::Start(Tag::Link { dest_url, .. }) = event else {
            continue;
        };
        let destination = dest_url.as_ref();
        if destination.starts_with('#')
            || destination.starts_with("http://")
            || destination.starts_with("https://")
            || destination.starts_with("mailto:")
        {
            continue;
        }
        let without_fragment = destination.split('#').next().unwrap_or("");
        if without_fragment.is_empty() {
            continue;
        }
        let resolved = if let Some(rooted) = without_fragment.strip_prefix('/') {
            normalize_link_path(Path::new(""), Path::new(rooted))
        } else {
            normalize_link_path(parent, Path::new(without_fragment))
        };
        let exists = match resolved.as_ref() {
            Some(resolved) => {
                repository.regular_file_or_directory_exists(resolved, "canonical RSI local link")?
            }
            None => false,
        };
        if !exists {
            return Err(invalid(
                "knowledge.rsi.local_link",
                format!("{path} references missing local path {destination}"),
            ));
        }
    }
    Ok(())
}

pub(super) fn normalize_link_path(base: &Path, destination: &Path) -> Option<PathBuf> {
    let mut resolved = PathBuf::new();
    for component in base.components().chain(destination.components()) {
        match component {
            Component::Normal(value) => resolved.push(value),
            Component::CurDir => {}
            Component::ParentDir if resolved.pop() => {}
            _ => return None,
        }
    }
    (resolved.starts_with("content")
        || resolved.starts_with("knowledge")
        || resolved.starts_with("evidence")
        || resolved.starts_with("labs")
        || resolved.starts_with("crates")
        || resolved.starts_with("formalization"))
    .then_some(resolved)
}
