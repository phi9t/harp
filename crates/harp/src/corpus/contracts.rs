use super::obsidian::{parse_wiki_links, resolve_wiki_link};
use super::*;
use pulldown_cmark::{Event, Options, Parser, Tag};
use serde::Deserialize;
use std::path::Component;
use url::Url;

use super::render::{heading_ids, heading_ids_for_source, markdown_body};

pub(super) const WENG_MAP_PATH: &str = "content/weng-reading-map.json";
pub(super) const SYSTEM_READINGS_PATH: &str = "content/systems/system_readings.json";
const MAX_CONTRACT_BYTES: usize = 512 * 1024;
const REQUIRED_WENG_HEADINGS: [&str; 6] = [
    "what-weng-claims",
    "mechanism",
    "hidden-assumption",
    "demonstrated-versus-proposed",
    "what-would-weaken-this-interpretation",
    "reader-checkpoint",
];

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WengReadingMap {
    pub(super) schema_version: u8,
    pub(super) source_id: String,
    pub(super) source_sha256: String,
    pub(super) sections: Vec<WengSection>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WengSection {
    pub(super) order: u8,
    pub(super) section_id: String,
    pub(super) title: String,
    pub(super) public_url: String,
    pub(super) captured_locator: String,
    pub(super) companion_path: String,
    pub(super) companion_section: String,
    pub(super) system_ids: Vec<String>,
    pub(super) exercise_ids: Vec<String>,
    pub(super) figure_locators: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SystemRegistry {
    pub(super) schema_version: u8,
    pub(super) systems: Vec<SystemReading>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SystemReading {
    pub(super) system_id: String,
    pub(super) title: String,
    pub(super) source_ids: Vec<String>,
    pub(super) treatment: SystemTreatment,
    pub(super) publication_state: PublicationState,
    pub(super) canonical_markdown_path: String,
    pub(super) weng_section_ids: Vec<String>,
    pub(super) paper_routes: Vec<PaperRoute>,
    pub(super) diagnostic_case_path: Option<String>,
    pub(super) related_system_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PaperRoute {
    pub(super) source_id: String,
    pub(super) public_url: String,
    pub(super) captured_locator: String,
    pub(super) reading_sequence: ReadingSequence,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadingSequence {
    pub(super) method: String,
    pub(super) algorithm: String,
    pub(super) main_evaluation: String,
    pub(super) ablation: String,
    pub(super) limitations: String,
    pub(super) appendix: String,
}

pub(super) struct ValidatedRsiInputs {
    pub(super) retained_concepts: Vec<RetainedConcept>,
    pub(super) coverage: Vec<CoverageEntry>,
    pub(super) registered_sources: BTreeSet<String>,
    pub(super) canonical_sources: BTreeMap<String, ValidatedCanonicalSource>,
    pub(super) weng_map: Option<WengReadingMap>,
    pub(super) system_registry: Option<SystemRegistry>,
}

pub(super) struct ValidatedCanonicalSource {
    pub(super) path: String,
    pub(super) markdown: String,
    pub(super) body_sha256: String,
    pub(super) entries: Vec<CoverageEntry>,
}

fn validate_no_content_markdown(repository: &HeldDirectory) -> Result<(), AppError> {
    let markdown = repository.regular_files_with_extension(
        Path::new(CONTENT_ROOT),
        "md",
        "structured RSI content",
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

pub(super) fn load(repository: &HeldDirectory) -> Result<ValidatedRsiInputs, AppError> {
    validate_no_content_markdown(repository)?;
    let retained_concepts = load_retained_concepts(repository)?;
    let coverage = load_coverage(repository)?;
    validate_coverage(&coverage, &retained_concepts)?;
    let registered_sources = validate_registry_coverage(repository, &retained_concepts)?;
    validate_evidence_graph(repository, &registered_sources)?;
    let system_registry = load_system_registry(repository, &registered_sources)?;
    let weng_map = load_weng_map(repository, &registered_sources, system_registry.as_ref())?;

    let mut entries_by_path = BTreeMap::<String, Vec<CoverageEntry>>::new();
    for entry in &coverage {
        entries_by_path
            .entry(entry.canonical_markdown_path.clone())
            .or_default()
            .push(entry.clone());
    }
    for (_, _, path) in READER_ROUTES {
        entries_by_path.entry(path.to_owned()).or_default();
    }
    for (_, path) in AUXILIARY_DOCUMENTS {
        entries_by_path.entry(path.to_owned()).or_default();
    }
    let knowledge_home = Path::new("knowledge/harp_knowledge_home.md");
    if repository
        .read_optional_regular_file_bounded(
            knowledge_home,
            "Harp knowledge home",
            MAX_MARKDOWN_BYTES,
        )?
        .is_some()
    {
        entries_by_path
            .entry(knowledge_home.to_string_lossy().into_owned())
            .or_default();
    }
    if let Some(map) = &weng_map {
        for section in &map.sections {
            entries_by_path
                .entry(section.companion_path.clone())
                .or_default();
        }
    }
    if let Some(registry) = &system_registry {
        for system in &registry.systems {
            if system.publication_state == PublicationState::Published {
                entries_by_path
                    .entry(system.canonical_markdown_path.clone())
                    .or_default();
            }
        }
    }

    let mut canonical_sources = BTreeMap::new();
    for (path, entries) in entries_by_path {
        let markdown = read_canonical_markdown(repository, &path)?;
        if markdown.contains('\0') || markdown.contains('\r') {
            return Err(invalid(
                "knowledge.rsi.canonical_encoding",
                format!("{path} must use LF line endings and contain no NUL bytes"),
            ));
        }
        let coverage_entries = coverage
            .iter()
            .filter(|entry| entry.canonical_markdown_path == path)
            .collect::<Vec<_>>();
        if coverage_entries.is_empty() {
            let body = markdown_body(&markdown, &path)?;
            validate_local_links(&path, body, repository)?;
        } else {
            validate_document(&path, &markdown, &coverage_entries, repository)?;
        }
        let body = markdown_body(&markdown, &path)?.to_owned();
        heading_ids_for_source(&body, &path)?;
        document_metadata(&markdown, &path, &entries)?;
        canonical_sources.insert(
            path.clone(),
            ValidatedCanonicalSource {
                path,
                markdown,
                body_sha256: sha256(body.as_bytes()),
                entries,
            },
        );
    }

    Ok(ValidatedRsiInputs {
        retained_concepts,
        coverage,
        registered_sources,
        canonical_sources,
        weng_map,
        system_registry,
    })
}

pub(super) fn document_metadata(
    markdown: &str,
    path: &str,
    entries: &[CoverageEntry],
) -> Result<DocumentMetadata, AppError> {
    let default_id = entries
        .iter()
        .find(|entry| entry.section_id.is_none())
        .or_else(|| entries.first())
        .map(|entry| entry.concept_id.clone())
        .unwrap_or_else(|| {
            AUXILIARY_DOCUMENTS
                .iter()
                .find(|(_, document_path)| *document_path == path)
                .map(|(document_id, _)| (*document_id).to_owned())
                .unwrap_or_else(|| {
                    Path::new(path)
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("document")
                        .to_owned()
                })
        });
    let mut metadata = DocumentMetadata {
        id: default_id,
        kind: "technical-document".to_owned(),
        status: "legacy".to_owned(),
        tags: Vec::new(),
        confidence: "medium".to_owned(),
        mode: None,
        source_ids: Vec::new(),
        coverage_keys: entries
            .iter()
            .map(|entry| entry.concept_id.clone())
            .collect(),
    };
    let Some(frontmatter) = markdown.strip_prefix("---\n") else {
        return Ok(metadata);
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
            return Ok(metadata);
        }
        let (key, value) = line.split_once(':').ok_or_else(|| {
            invalid(
                "knowledge.rsi.frontmatter",
                format!("{path} frontmatter entry has no colon"),
            )
        })?;
        if key.is_empty() || key.trim() != key || value.is_empty() {
            return Ok(metadata);
        }
        if fields.insert(key, value.trim()).is_some() {
            return Ok(metadata);
        }
    }
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
            "coverage_keys" => metadata.coverage_keys = metadata_list(value, path, key)?,
            _ => {}
        }
    }
    metadata.tags.sort();
    metadata.tags.dedup();
    metadata.source_ids.sort();
    metadata.source_ids.dedup();
    metadata.coverage_keys.sort();
    metadata.coverage_keys.dedup();
    Ok(metadata)
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

fn load_system_registry(
    repository: &HeldDirectory,
    registered_sources: &BTreeSet<String>,
) -> Result<Option<SystemRegistry>, AppError> {
    let Some(bytes) = repository.read_optional_regular_file_bounded(
        Path::new(SYSTEM_READINGS_PATH),
        "RSI system registry",
        MAX_CONTRACT_BYTES,
    )?
    else {
        return Ok(None);
    };
    let value = crate::json::parse_strict_bounded(
        &bytes,
        MAX_CONTRACT_BYTES,
        "knowledge.rsi.system_registry_size",
        "knowledge.rsi.system_registry",
        "RSI system registry",
    )?;
    let registry: SystemRegistry = serde_json::from_value(value).map_err(|error| {
        invalid(
            "knowledge.rsi.system_registry",
            format!("RSI system registry does not match the strict schema: {error}"),
        )
    })?;
    if registry.schema_version != 1 || registry.systems.is_empty() {
        return Err(invalid(
            "knowledge.rsi.system_registry",
            "RSI system registry must use schema version 1 and contain systems",
        ));
    }

    let mut system_ids = BTreeSet::new();
    for system in &registry.systems {
        if !valid_id(&system.system_id)
            || !system_ids.insert(system.system_id.as_str())
            || system.title.trim().is_empty()
            || system.title.trim() != system.title
            || system.source_ids.is_empty()
            || system.paper_routes.is_empty()
        {
            return Err(invalid(
                "knowledge.rsi.system_registry",
                format!(
                    "system {} has an invalid identity or shape",
                    system.system_id
                ),
            ));
        }
        require_unique_tokens(
            &system.source_ids,
            "knowledge.rsi.system_source",
            &system.system_id,
        )?;
        require_unique_ids(
            &system.weng_section_ids,
            "knowledge.rsi.system_weng",
            &system.system_id,
        )?;
        require_unique_ids(
            &system.related_system_ids,
            "knowledge.rsi.system_related",
            &system.system_id,
        )?;
        if system
            .source_ids
            .iter()
            .any(|source_id| !registered_sources.contains(source_id))
        {
            return Err(invalid(
                "knowledge.rsi.system_source",
                format!("system {} references an unknown source", system.system_id),
            ));
        }
        let path = Path::new(&system.canonical_markdown_path);
        if !system
            .canonical_markdown_path
            .starts_with("knowledge/rsi/systems/")
            || !system.canonical_markdown_path.ends_with(".md")
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(invalid(
                "knowledge.rsi.system_path",
                format!("system {} has an invalid canonical path", system.system_id),
            ));
        }
        if system.diagnostic_case_path.as_ref().is_some_and(|path| {
            !path.starts_with("content/diagnostics/cases/") || !path.ends_with(".json")
        }) {
            return Err(invalid(
                "knowledge.rsi.system_case",
                format!(
                    "system {} has an invalid diagnostic case path",
                    system.system_id
                ),
            ));
        }
        for route in &system.paper_routes {
            if !system.source_ids.contains(&route.source_id)
                || !valid_https_url(&route.public_url)
                || !valid_captured_locator(&route.captured_locator)
                || route
                    .reading_sequence
                    .ordered()
                    .iter()
                    .any(|locator| !valid_reading_locator(locator))
            {
                return Err(invalid(
                    "knowledge.rsi.system_route",
                    format!("system {} has an invalid paper route", system.system_id),
                ));
            }
            validate_locator(repository, &route.captured_locator)?;
        }
        if system.publication_state == PublicationState::Published {
            read_canonical_markdown(repository, &system.canonical_markdown_path).map_err(|_| {
                invalid(
                    "knowledge.rsi.system_path",
                    format!(
                        "published system {} is missing canonical Markdown",
                        system.system_id
                    ),
                )
            })?;
        }
        match system.treatment {
            SystemTreatment::Full | SystemTreatment::Card => {}
        }
    }
    for system in &registry.systems {
        if system
            .related_system_ids
            .iter()
            .any(|related| related == &system.system_id || !system_ids.contains(related.as_str()))
        {
            return Err(invalid(
                "knowledge.rsi.system_related",
                format!(
                    "system {} references an invalid related system",
                    system.system_id
                ),
            ));
        }
    }
    Ok(Some(registry))
}

impl ReadingSequence {
    fn ordered(&self) -> [&str; 6] {
        [
            &self.method,
            &self.algorithm,
            &self.main_evaluation,
            &self.ablation,
            &self.limitations,
            &self.appendix,
        ]
    }
}

fn valid_reading_locator(value: &str) -> bool {
    value.trim() == value
        && !value.is_empty()
        && (value.starts_with("not-applicable: ")
            || value.contains('§')
            || value.contains("Appendix")
            || value.contains("README")
            || value.contains("Methods")
            || value.contains("Results")
            || value.contains("Experiments")
            || value.contains("Ablation")
            || value.contains("ablation")
            || value.contains("Limitations")
            || value.contains("Discussion")
            || value.contains("Figure")
            || value.contains("Table")
            || value.contains("Algorithm")
            || value.contains("program.md"))
}

fn load_weng_map(
    repository: &HeldDirectory,
    registered_sources: &BTreeSet<String>,
    system_registry: Option<&SystemRegistry>,
) -> Result<Option<WengReadingMap>, AppError> {
    let Some(bytes) = repository.read_optional_regular_file_bounded(
        Path::new(WENG_MAP_PATH),
        "Weng reading map",
        MAX_CONTRACT_BYTES,
    )?
    else {
        if system_registry.is_some() {
            return Err(invalid(
                "knowledge.rsi.weng_missing",
                "system registry requires a Weng reading map",
            ));
        }
        return Ok(None);
    };
    let value = crate::json::parse_strict_bounded(
        &bytes,
        MAX_CONTRACT_BYTES,
        "knowledge.rsi.weng_size",
        "knowledge.rsi.weng_schema",
        "Weng reading map",
    )?;
    let map: WengReadingMap = serde_json::from_value(value).map_err(|_| {
        invalid(
            "knowledge.rsi.weng_schema",
            "Weng reading map does not match the strict schema",
        )
    })?;
    if map.schema_version != 1
        || map.sections.is_empty()
        || !registered_sources.contains(&map.source_id)
        || !valid_sha256(&map.source_sha256)
    {
        return Err(invalid(
            "knowledge.rsi.weng_schema",
            "Weng reading map has an invalid version, source, digest, or section list",
        ));
    }
    let registered_digest = source_digest(repository, &map.source_id)?;
    if registered_digest.as_deref() != Some(map.source_sha256.as_str()) {
        return Err(invalid(
            "knowledge.rsi.weng_digest",
            "Weng reading map digest does not match the source registry",
        ));
    }

    let system_ids = system_registry
        .map(|registry| {
            registry
                .systems
                .iter()
                .map(|system| system.system_id.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut section_ids = BTreeSet::new();
    let mut anchors = BTreeSet::new();
    let mut exercise_ids = BTreeSet::new();
    for (index, section) in map.sections.iter().enumerate() {
        if usize::from(section.order) != index + 1 {
            return Err(invalid(
                "knowledge.rsi.weng_order",
                "Weng section order must be contiguous and match array order",
            ));
        }
        if !valid_id(&section.section_id)
            || !section_ids.insert(section.section_id.as_str())
            || section.title.trim().is_empty()
            || section.title.trim() != section.title
        {
            return Err(invalid(
                "knowledge.rsi.weng_section",
                "Weng section IDs and titles must be nonempty and unique",
            ));
        }
        let url = Url::parse(&section.public_url).map_err(|_| {
            invalid(
                "knowledge.rsi.weng_url",
                format!(
                    "Weng section {} has an invalid public URL",
                    section.section_id
                ),
            )
        })?;
        let anchor = url.fragment().unwrap_or("");
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || anchor.is_empty()
            || !anchors.insert(anchor.to_owned())
        {
            return Err(invalid(
                "knowledge.rsi.weng_url",
                format!(
                    "Weng section {} requires a unique HTTPS anchor",
                    section.section_id
                ),
            ));
        }
        if !valid_captured_locator(&section.captured_locator)
            || !section.captured_locator.starts_with("evidence/weng/")
        {
            return Err(invalid(
                "knowledge.rsi.weng_locator",
                format!(
                    "Weng section {} has an invalid captured locator",
                    section.section_id
                ),
            ));
        }
        let companion = Path::new(&section.companion_path);
        if !section.companion_path.starts_with("knowledge/rsi/weng/")
            || !section.companion_path.ends_with(".md")
            || companion
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || !valid_id(&section.companion_section)
        {
            return Err(invalid(
                "knowledge.rsi.weng_companion",
                format!(
                    "Weng section {} has an invalid companion path or heading",
                    section.section_id
                ),
            ));
        }
        let markdown = read_canonical_markdown(repository, &section.companion_path)?;
        let body = markdown_body(&markdown, &section.companion_path)?;
        let headings = heading_ids(body);
        if !headings.contains(&section.companion_section)
            || REQUIRED_WENG_HEADINGS
                .iter()
                .any(|heading| !headings.contains(*heading))
            || body.matches("<details>").count() != 1
            || body.matches("</details>").count() != 1
            || body.matches("<summary>Check your answer</summary>").count() != 1
            || body.contains("<details open")
        {
            return Err(invalid(
                "knowledge.rsi.weng_companion",
                format!(
                    "Weng section {} references an incomplete companion",
                    section.section_id
                ),
            ));
        }
        validate_locator(repository, &section.captured_locator)?;
        require_unique_ids(
            &section.system_ids,
            "knowledge.rsi.weng_system",
            &section.section_id,
        )?;
        if section
            .system_ids
            .iter()
            .any(|system_id| !system_ids.contains(system_id.as_str()))
        {
            return Err(invalid(
                "knowledge.rsi.weng_system",
                format!(
                    "Weng section {} references an unknown system",
                    section.section_id
                ),
            ));
        }
        require_unique_ids(
            &section.exercise_ids,
            "knowledge.rsi.weng_exercise",
            &section.section_id,
        )?;
        for exercise_id in &section.exercise_ids {
            if !exercise_ids.insert(exercise_id.as_str()) {
                return Err(invalid(
                    "knowledge.rsi.weng_exercise",
                    format!("Weng exercise {exercise_id} is duplicated"),
                ));
            }
        }
        if section
            .figure_locators
            .iter()
            .any(|locator| !valid_captured_locator(locator))
        {
            return Err(invalid(
                "knowledge.rsi.weng_locator",
                format!(
                    "Weng section {} has an invalid figure locator",
                    section.section_id
                ),
            ));
        }
        for locator in &section.figure_locators {
            validate_locator(repository, locator)?;
        }
    }

    if let Some(registry) = system_registry {
        for system in &registry.systems {
            if system
                .weng_section_ids
                .iter()
                .any(|section_id| !section_ids.contains(section_id.as_str()))
            {
                return Err(invalid(
                    "knowledge.rsi.system_weng",
                    format!(
                        "system {} references an unknown Weng section",
                        system.system_id
                    ),
                ));
            }
            for section_id in &system.weng_section_ids {
                let section = map
                    .sections
                    .iter()
                    .find(|section| &section.section_id == section_id)
                    .expect("Weng section was validated");
                if !section.system_ids.contains(&system.system_id) {
                    return Err(invalid(
                        "knowledge.rsi.system_weng",
                        format!(
                            "system {} and Weng section {} are not reciprocal",
                            system.system_id, section_id
                        ),
                    ));
                }
            }
        }
    }
    Ok(Some(map))
}

fn require_unique_ids(values: &[String], code: &'static str, owner: &str) -> Result<(), AppError> {
    let mut unique = BTreeSet::new();
    if values
        .iter()
        .any(|value| !valid_id(value) || !unique.insert(value.as_str()))
    {
        return Err(invalid(
            code,
            format!("{owner} contains invalid or duplicate IDs"),
        ));
    }
    Ok(())
}

fn require_unique_tokens(
    values: &[String],
    code: &'static str,
    owner: &str,
) -> Result<(), AppError> {
    let mut unique = BTreeSet::new();
    if values.iter().any(|value| {
        value.is_empty()
            || value.trim() != value
            || value.contains(['\0', '\r', '\n', '\t', ','])
            || !unique.insert(value.as_str())
    }) {
        return Err(invalid(
            code,
            format!("{owner} contains invalid or duplicate tokens"),
        ));
    }
    Ok(())
}

fn valid_https_url(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

fn valid_captured_locator(value: &str) -> bool {
    !value.is_empty()
        && !value.contains(['\0', '\r', '\n'])
        && !value.starts_with('/')
        && !value.contains('\\')
        && value
            .split(['#', ':'])
            .next()
            .is_some_and(|path| !path.is_empty() && !path.split('/').any(|part| part == ".."))
}

fn validate_locator(repository: &HeldDirectory, locator: &str) -> Result<(), AppError> {
    let (path, suffix) = locator.split_once('#').map_or_else(
        || {
            locator
                .rsplit_once(':')
                .map_or((locator, None), |(path, suffix)| {
                    if suffix
                        .split_once('-')
                        .map_or(suffix, |(start, _)| start)
                        .bytes()
                        .all(|byte| byte.is_ascii_digit())
                    {
                        (path, Some(suffix))
                    } else {
                        (locator, None)
                    }
                })
        },
        |(path, anchor)| (path, Some(anchor)),
    );
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(path),
            "RSI captured locator",
            8 * 1024 * 1024,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.weng_locator",
                format!("captured locator path is missing: {path}"),
            )
        })?;
    if let Some(suffix) = suffix {
        if suffix
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
        {
            let start = suffix
                .split_once('-')
                .map_or(suffix, |(start, _)| start)
                .parse::<usize>()
                .unwrap_or(0);
            if start == 0 || bytes.split(|byte| *byte == b'\n').count() < start {
                return Err(invalid(
                    "knowledge.rsi.weng_locator",
                    format!("captured locator line is absent: {locator}"),
                ));
            }
        } else {
            let text = std::str::from_utf8(&bytes).map_err(|_| {
                invalid(
                    "knowledge.rsi.weng_locator",
                    format!("captured anchor source is not UTF-8: {path}"),
                )
            })?;
            if !text.contains(&format!("id=\"{suffix}\"")) {
                return Err(invalid(
                    "knowledge.rsi.weng_locator",
                    format!("captured locator anchor is absent: {locator}"),
                ));
            }
        }
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn source_digest(repository: &HeldDirectory, source_id: &str) -> Result<Option<String>, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(SOURCE_REGISTRY_PATH),
            "RSI source registry",
            MAX_COVERAGE_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.source_registry",
                format!("{SOURCE_REGISTRY_PATH} is missing or unreadable"),
            )
        })?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        invalid(
            "knowledge.rsi.source_registry",
            "source registry is not UTF-8",
        )
    })?;
    let mut lines = text.split_terminator('\n');
    let columns = lines.next().unwrap_or("").split('\t').collect::<Vec<_>>();
    let source_index = columns.iter().position(|column| *column == "source_id");
    let digest_index = columns
        .iter()
        .position(|column| *column == "version_or_digest");
    let (Some(source_index), Some(digest_index)) = (source_index, digest_index) else {
        return Err(invalid(
            "knowledge.rsi.source_registry",
            "source registry must expose source_id and version_or_digest",
        ));
    };
    let row = lines
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|cells| cells.get(source_index).copied() == Some(source_id));
    let Some(cells) = row else {
        return Ok(None);
    };
    Ok(cells
        .get(digest_index)
        .into_iter()
        .flat_map(|cell| cell.split_whitespace())
        .find_map(|part| part.strip_prefix("sha256:"))
        .filter(|digest| valid_sha256(digest))
        .map(str::to_owned))
}

fn validate_registry_coverage(
    repository: &HeldDirectory,
    retained_concepts: &[RetainedConcept],
) -> Result<BTreeSet<String>, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(SOURCE_REGISTRY_PATH),
            "RSI source registry",
            MAX_COVERAGE_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.source_registry",
                format!("{SOURCE_REGISTRY_PATH} is missing or unreadable"),
            )
        })?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        invalid(
            "knowledge.rsi.source_registry",
            format!("{SOURCE_REGISTRY_PATH} is not readable UTF-8"),
        )
    })?;
    let mut lines = text.split_terminator('\n');
    let header = lines.next().unwrap_or("");
    let columns = header.split('\t').collect::<Vec<_>>();
    let source_index = columns
        .iter()
        .position(|column| *column == "source_id")
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.source_registry",
                "source registry is missing source_id",
            )
        })?;
    let mapped = retained_concepts
        .iter()
        .flat_map(|concept| concept.source_ids.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let mut registered = BTreeSet::new();
    for (line_index, line) in lines.enumerate() {
        let cells = line.split('\t').collect::<Vec<_>>();
        if cells.len() != columns.len() {
            return Err(invalid(
                "knowledge.rsi.source_registry",
                format!("source registry row {} has the wrong width", line_index + 2),
            ));
        }
        let source_id = cells[source_index];
        if !registered.insert(source_id.to_owned()) {
            return Err(invalid(
                "knowledge.rsi.source_registry",
                format!("source registry contains duplicate source {source_id}"),
            ));
        }
        if !mapped.contains(source_id) {
            return Err(invalid(
                "knowledge.rsi.registry_coverage",
                format!("retained source {source_id} has no explanatory home"),
            ));
        }
    }
    let registered_refs = registered
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let missing = mapped
        .difference(&registered_refs)
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(invalid(
            "knowledge.rsi.registry_coverage",
            format!("retained-concept roster references absent sources {missing:?}"),
        ));
    }
    Ok(registered)
}

pub(super) fn validate_evidence_graph(
    repository: &HeldDirectory,
    registered_sources: &BTreeSet<String>,
) -> Result<(), AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(EVIDENCE_GRAPH_PATH),
            "RSI evidence graph",
            MAX_COVERAGE_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.evidence_graph",
                format!("{EVIDENCE_GRAPH_PATH} is missing or unreadable"),
            )
        })?;
    if bytes.contains(&0) || bytes.contains(&b'\r') {
        return Err(invalid(
            "knowledge.rsi.evidence_graph",
            "evidence graph must be UTF-8 with LF line endings and no NUL bytes",
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        invalid(
            "knowledge.rsi.evidence_graph",
            format!("{EVIDENCE_GRAPH_PATH} is not readable UTF-8"),
        )
    })?;
    let mut lines = text.split_terminator('\n');
    if lines.next() != Some(EVIDENCE_GRAPH_HEADER) {
        return Err(invalid(
            "knowledge.rsi.evidence_graph",
            format!("evidence graph header must be exactly {EVIDENCE_GRAPH_HEADER:?}"),
        ));
    }
    let mut numbered_references = [0u8; 39];
    for (line_index, line) in lines.enumerate() {
        let cells = line.split('\t').collect::<Vec<_>>();
        if cells.len() != 7 || cells.iter().any(|cell| cell.is_empty()) {
            return Err(invalid(
                "knowledge.rsi.evidence_graph",
                format!("evidence graph row {} has the wrong shape", line_index + 2),
            ));
        }
        for (column, source_id) in [("source_id", cells[1]), ("target_id", cells[3])] {
            if !registered_sources.contains(source_id) {
                return Err(invalid(
                    "knowledge.rsi.evidence_graph_endpoint",
                    format!(
                        "evidence graph row {} {column} {source_id} is not registered",
                        line_index + 2
                    ),
                ));
            }
        }
        if cells[1] == "WENG-HARNESS" && cells[2] == "cites" {
            let Some(number) = numbered_reference(cells[4]) else {
                return Err(invalid(
                    "knowledge.rsi.weng_references",
                    format!(
                        "Weng citation row {} has an invalid numbered locator",
                        line_index + 2
                    ),
                ));
            };
            numbered_references[number - 1] = numbered_references[number - 1].saturating_add(1);
        }
    }
    if registered_sources.contains("WENG-HARNESS") {
        let invalid_numbers = numbered_references
            .iter()
            .enumerate()
            .filter_map(|(index, count)| (*count != 1).then_some((index + 1, *count)))
            .collect::<Vec<_>>();
        if !invalid_numbers.is_empty() {
            return Err(invalid(
                "knowledge.rsi.weng_references",
                format!(
                    "Weng references 1 through 39 must appear exactly once; invalid={invalid_numbers:?}"
                ),
            ));
        }
    }
    Ok(())
}

fn numbered_reference(locator: &str) -> Option<usize> {
    let suffix = locator.strip_prefix("Reference ")?;
    let digits = suffix
        .bytes()
        .take_while(u8::is_ascii_digit)
        .collect::<Vec<_>>();
    if digits.is_empty() {
        return None;
    }
    let boundary = suffix.as_bytes().get(digits.len()).copied();
    if boundary.is_some_and(|byte| !byte.is_ascii_whitespace()) {
        return None;
    }
    let number = std::str::from_utf8(&digits).ok()?.parse::<usize>().ok()?;
    (1..=39).contains(&number).then_some(number)
}

fn load_retained_concepts(repository: &HeldDirectory) -> Result<Vec<RetainedConcept>, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(RETAINED_CONCEPTS_PATH),
            "RSI retained-concept roster",
            MAX_COVERAGE_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.retained_missing",
                format!("{RETAINED_CONCEPTS_PATH} is missing or unreadable"),
            )
        })?;
    if bytes.contains(&0) || bytes.contains(&b'\r') {
        return Err(invalid(
            "knowledge.rsi.retained_encoding",
            "retained-concept roster must be UTF-8 with LF line endings and no NUL bytes",
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        invalid(
            "knowledge.rsi.retained_encoding",
            "retained-concept roster is not UTF-8",
        )
    })?;
    let mut lines = text.split_terminator('\n');
    if lines.next() != Some(RETAINED_CONCEPTS_HEADER) {
        return Err(invalid(
            "knowledge.rsi.retained_header",
            format!("retained-concept roster header must be exactly {RETAINED_CONCEPTS_HEADER:?}"),
        ));
    }
    let concepts = lines
        .enumerate()
        .map(|(index, line)| {
            let cells = line.split('\t').collect::<Vec<_>>();
            if cells.len() != 2 || !valid_id(cells[0]) {
                return Err(invalid(
                    "knowledge.rsi.retained_row",
                    format!(
                        "retained-concept row {} must contain a valid concept ID and source list",
                        index + 2
                    ),
                ));
            }
            let source_ids = if cells[1].is_empty() {
                Vec::new()
            } else {
                cells[1].split(',').map(str::to_owned).collect()
            };
            if source_ids
                .iter()
                .any(|source_id| source_id.is_empty() || source_id.trim() != source_id)
            {
                return Err(invalid(
                    "knowledge.rsi.retained_row",
                    format!("retained-concept row {} has invalid source IDs", index + 2),
                ));
            }
            Ok(RetainedConcept {
                concept_id: cells[0].to_owned(),
                source_ids,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if concepts.is_empty() {
        return Err(invalid(
            "knowledge.rsi.retained_empty",
            "retained-concept roster must contain concepts",
        ));
    }
    let mut concept_ids = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    for concept in &concepts {
        if !concept_ids.insert(concept.concept_id.as_str()) {
            return Err(invalid(
                "knowledge.rsi.retained_owner",
                format!(
                    "retained concept {} must appear exactly once",
                    concept.concept_id
                ),
            ));
        }
        for source_id in &concept.source_ids {
            if !source_ids.insert(source_id.as_str()) {
                return Err(invalid(
                    "knowledge.rsi.retained_source_owner",
                    format!("source {source_id} must map to exactly one retained concept"),
                ));
            }
        }
    }
    Ok(concepts)
}

fn load_coverage(repository: &HeldDirectory) -> Result<Vec<CoverageEntry>, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(COVERAGE_PATH),
            "RSI coverage map",
            MAX_COVERAGE_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.coverage_missing",
                format!("{COVERAGE_PATH} is missing or unreadable"),
            )
        })?;
    if bytes.contains(&0) || bytes.contains(&b'\r') {
        return Err(invalid(
            "knowledge.rsi.coverage_encoding",
            "coverage map must be UTF-8 with LF line endings and no NUL bytes",
        ));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        invalid(
            "knowledge.rsi.coverage_encoding",
            "coverage map is not UTF-8",
        )
    })?;
    let mut lines = text.split_terminator('\n');
    if lines.next() != Some(COVERAGE_HEADER) {
        return Err(invalid(
            "knowledge.rsi.coverage_header",
            format!("coverage map header must be exactly {COVERAGE_HEADER:?}"),
        ));
    }
    lines
        .enumerate()
        .map(|(index, line)| parse_coverage_row(index + 2, line))
        .collect()
}

pub(super) fn parse_coverage_row(
    line_number: usize,
    line: &str,
) -> Result<CoverageEntry, AppError> {
    let cells = line.split('\t').collect::<Vec<_>>();
    if cells.len() != 5 || cells.iter().take(3).any(|cell| cell.is_empty()) {
        return Err(invalid(
            "knowledge.rsi.coverage_row",
            format!("coverage row {line_number} must contain five valid cells"),
        ));
    }
    let coverage_depth = match cells[1] {
        "chapter" => CoverageDepth::Chapter,
        "supporting-page" => CoverageDepth::SupportingPage,
        "system-reading" => CoverageDepth::SystemReading,
        "worked-example" => CoverageDepth::WorkedExample,
        _ => {
            return Err(invalid(
                "knowledge.rsi.coverage_depth",
                format!("coverage row {line_number} has an invalid depth"),
            ));
        }
    };
    let section_id = optional_cell(cells[3]);
    let is_concept_path = cells[2].starts_with("knowledge/rsi/concepts/");
    let is_system_path = cells[2].starts_with("knowledge/rsi/systems/");
    let valid_section = match coverage_depth {
        CoverageDepth::Chapter => section_id.is_none(),
        CoverageDepth::SupportingPage => section_id.is_some() && is_concept_path,
        CoverageDepth::SystemReading => section_id.is_none() && is_system_path,
        CoverageDepth::WorkedExample => section_id.is_some(),
    };
    if !valid_section {
        return Err(invalid(
            "knowledge.rsi.coverage_section",
            format!(
                "coverage row {line_number} must use the path and section required by its depth"
            ),
        ));
    }
    let canonical_markdown_path = normalize_canonical_path(cells[2], line_number)?;
    Ok(CoverageEntry {
        concept_id: cells[0].to_owned(),
        coverage_depth,
        canonical_markdown_path,
        section_id,
        parent_concept_id: optional_cell(cells[4]),
    })
}

fn normalize_canonical_path(value: &str, line_number: usize) -> Result<String, AppError> {
    let path = Path::new(value);
    let allowed_root = value.starts_with("knowledge/rsi/chapters/")
        || value.starts_with("knowledge/rsi/concepts/")
        || value.starts_with("knowledge/rsi/systems/");
    let canonical = !value.contains('\\')
        && value.ends_with(".md")
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !allowed_root || !canonical {
        return Err(invalid(
            "knowledge.rsi.coverage_path",
            format!("coverage row {line_number} has an invalid canonical Markdown path"),
        ));
    }
    Ok(value.to_owned())
}

fn validate_coverage(
    entries: &[CoverageEntry],
    retained_concepts: &[RetainedConcept],
) -> Result<(), AppError> {
    if entries.is_empty() {
        return Err(invalid(
            "knowledge.rsi.coverage_empty",
            "coverage map must contain retained concepts",
        ));
    }
    let mut ids = BTreeSet::new();
    for entry in entries {
        if !valid_id(&entry.concept_id) || !ids.insert(entry.concept_id.as_str()) {
            return Err(invalid(
                "knowledge.rsi.coverage_owner",
                format!(
                    "concept {} must have exactly one primary explanatory home",
                    entry.concept_id
                ),
            ));
        }
        if let Some(parent) = &entry.parent_concept_id {
            if parent == &entry.concept_id {
                return Err(invalid(
                    "knowledge.rsi.coverage_parent",
                    format!("concept {} cannot parent itself", entry.concept_id),
                ));
            }
        }
    }
    let retained_ids = retained_concepts
        .iter()
        .map(|concept| concept.concept_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids != retained_ids {
        let uncovered = retained_ids.difference(&ids).copied().collect::<Vec<_>>();
        let unretained = ids.difference(&retained_ids).copied().collect::<Vec<_>>();
        return Err(invalid(
            "knowledge.rsi.retained_coverage",
            format!(
                "retained concepts and coverage map differ; uncovered={uncovered:?}, unretained={unretained:?}"
            ),
        ));
    }
    for entry in entries {
        if entry
            .parent_concept_id
            .as_ref()
            .is_some_and(|parent| !ids.contains(parent.as_str()))
        {
            return Err(invalid(
                "knowledge.rsi.coverage_parent",
                format!("concept {} references an unknown parent", entry.concept_id),
            ));
        }
    }
    for (id, path) in REQUIRED_CHAPTERS {
        let matches = entries
            .iter()
            .filter(|entry| {
                entry.concept_id == id
                    && entry.coverage_depth == CoverageDepth::Chapter
                    && entry.canonical_markdown_path == path
                    && entry.section_id.is_none()
            })
            .count();
        if matches != 1 {
            return Err(invalid(
                "knowledge.rsi.chapter_spine",
                format!("required chapter {id} must have the canonical path {path}"),
            ));
        }
    }
    let chapter_count = entries
        .iter()
        .filter(|entry| entry.coverage_depth == CoverageDepth::Chapter)
        .count();
    if chapter_count != REQUIRED_CHAPTERS.len() {
        return Err(invalid(
            "knowledge.rsi.chapter_spine",
            "the technical spine must contain exactly nine full chapters",
        ));
    }
    Ok(())
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

fn validate_document(
    path: &str,
    markdown: &str,
    entries: &[&CoverageEntry],
    repository: &HeldDirectory,
) -> Result<(), AppError> {
    if markdown.contains('\0') || markdown.contains('\r') {
        return Err(invalid(
            "knowledge.rsi.canonical_encoding",
            format!("{path} must use LF line endings and contain no NUL bytes"),
        ));
    }
    let body = markdown_body(markdown, path)?;
    validate_source_folds(path, body)?;
    validate_information_order(path, body)?;
    let headings = heading_ids(body);
    for entry in entries {
        if let Some(section) = &entry.section_id {
            if !headings.contains(section) {
                return Err(invalid(
                    "knowledge.rsi.coverage_section",
                    format!(
                        "worked example {} references absent section #{section} in {path}",
                        entry.concept_id
                    ),
                ));
            }
        }
    }
    validate_local_links(path, body, repository)
}

fn validate_source_folds(path: &str, markdown: &str) -> Result<(), AppError> {
    let opens = markdown.matches("<details>").count();
    let closes = markdown.matches("</details>").count();
    let source_summaries = markdown.matches(SOURCE_SUMMARY).count();
    let reference_summaries = markdown.matches(REFERENCE_SUMMARY).count();
    if opens == 0
        || opens != closes
        || source_summaries + reference_summaries != opens
        || markdown.contains("<details open")
    {
        return Err(invalid(
            "knowledge.rsi.source_fold",
            format!("{path} contains malformed or expanded source details"),
        ));
    }
    let first_open = markdown.find("<details>").unwrap_or(0);
    let first_heading = markdown.find("\n## ").unwrap_or(usize::MAX);
    if first_open < first_heading {
        return Err(invalid(
            "knowledge.rsi.source_fold",
            format!("{path} must explain a mechanism before its source details"),
        ));
    }
    Ok(())
}

fn validate_information_order(path: &str, markdown: &str) -> Result<(), AppError> {
    let first_h2 = markdown
        .lines()
        .find(|line| line.starts_with("## "))
        .unwrap_or("");
    let operational = [
        "source",
        "reference",
        "provenance",
        "validation",
        "registry",
    ];
    let normalized = first_h2.to_ascii_lowercase();
    if operational.iter().any(|word| normalized.contains(word)) {
        return Err(invalid(
            "knowledge.rsi.content_order",
            format!("{path} must put technical explanation before source metadata"),
        ));
    }
    Ok(())
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
        || resolved.starts_with("crates"))
    .then_some(resolved)
}
