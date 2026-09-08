use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Deserialize;

use crate::corpus::{
    canonical_frontmatter_identity, canonical_heading_ids, canonical_markdown_body,
};
use crate::error::AppError;
use crate::fs::{FileTreeLimits, HeldDirectory};

use super::{
    ExerciseRow, ProseProofStatus, TextbookContracts, TextbookDiagnostic, TheoremRow,
    ValidatedTextbookDocument,
};

const KNOWLEDGE_ROOT: &str = "knowledge/crouzeix_textbook";
const COMPATIBILITY_PATH: &str = "content/crouzeix_textbook/compatibility_routes.json";
const COMPATIBILITY_SCHEMA: &str = "crouzeix-textbook-compatibility-routes/v1";
const MAX_MARKDOWN_BYTES: usize = 512 * 1024;
const MAX_COMPATIBILITY_BYTES: usize = 128 * 1024;
const MAX_PACKET_DEPTH: usize = 8;
const MAX_PACKET_ENTRIES: usize = 256;
const MAX_PACKET_MARKDOWN_FILES: usize = 64;
const MAX_PACKET_MARKDOWN_BYTES: usize = 1024 * 1024;
const INPUT_CODE: &str = "crouzeix-textbook.markdown.input";
const ENCODING_CODE: &str = "crouzeix-textbook.markdown.encoding";
const FRONTMATTER_CODE: &str = "crouzeix-textbook.markdown.frontmatter";
const DUPLICATE_ID_CODE: &str = "crouzeix-textbook.markdown.duplicate-identity";
const HEADING_CODE: &str = "crouzeix-textbook.markdown.heading";
const ROUTE_CODE: &str = "crouzeix-textbook.markdown.compatibility-route";
const COLLISION_CODE: &str = "crouzeix-textbook.markdown.identity-collision";
const THEOREM_PATH_CODE: &str = "crouzeix-textbook.markdown.theorem-path";
const THEOREM_ANCHOR_CODE: &str = "crouzeix-textbook.markdown.theorem-anchor";
const THEOREM_CARD_CODE: &str = "crouzeix-textbook.markdown.theorem-card";

const REQUIRED_THEOREM_SUBSECTIONS: [&str; 10] = [
    "Purpose",
    "Statement",
    "Hypothesis ledger",
    "Proof roadmap",
    "Proof",
    "Boundary case",
    "Pedagogical prerequisites",
    "Lean correspondence",
    "Historical context",
    "ML analogy",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompatibilityRoutes {
    schema_version: String,
    pub(super) routes: Vec<CompatibilityRoute>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompatibilityRoute {
    pub(super) canonical_id: String,
    pub(super) legacy_concept_id: String,
    pub(super) canonical_path: PathBuf,
}

pub(super) struct Document {
    pub(super) id: String,
    pub(super) path: PathBuf,
    pub(super) body: String,
    markdown: String,
    pub(super) heading_ids: Vec<String>,
    pub(super) body_start_line: u32,
}

struct RegistrationState {
    documents: BTreeMap<PathBuf, Document>,
    routes: CompatibilityRoutes,
    diagnostics: Vec<TextbookDiagnostic>,
}

pub(super) fn validate(
    repository: &HeldDirectory,
    contracts: Option<&TextbookContracts>,
) -> Result<(), Vec<TextbookDiagnostic>> {
    let state = registration_state(repository)?;
    let mut diagnostics = Vec::new();
    super::evidence::validate(repository, &state.documents, contracts, &mut diagnostics);
    diagnostics.extend(state.diagnostics);
    if let Some(contracts) = contracts {
        validate_namespace(
            &state.documents,
            &state.routes.routes,
            contracts,
            &mut diagnostics,
        );
        validate_theorems(&state.documents, contracts.theorems(), &mut diagnostics);
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn registration_state(
    repository: &HeldDirectory,
) -> Result<RegistrationState, Vec<TextbookDiagnostic>> {
    let documents = discover_documents(repository)?;
    let routes = read_compatibility_routes(repository)?;
    let mut diagnostics = Vec::new();
    validate_routes(&documents, &routes, &mut diagnostics);
    validate_alias_namespace(&documents, &routes.routes, &mut diagnostics);
    Ok(RegistrationState {
        documents,
        routes,
        diagnostics,
    })
}

pub(super) fn validated_registration(
    repository: &HeldDirectory,
) -> Result<Vec<ValidatedTextbookDocument>, Vec<TextbookDiagnostic>> {
    let state = registration_state(repository)?;
    if !state.diagnostics.is_empty() {
        return Err(state.diagnostics);
    }
    Ok(state
        .routes
        .routes
        .into_iter()
        .map(|route| {
            let document = state
                .documents
                .get(&route.canonical_path)
                .expect("validated route must resolve to its discovered document");
            ValidatedTextbookDocument {
                canonical_id: route.canonical_id,
                legacy_alias: route.legacy_concept_id,
                canonical_path: route.canonical_path.to_string_lossy().into_owned(),
                markdown: document.markdown.clone(),
            }
        })
        .collect())
}

fn discover_documents(
    repository: &HeldDirectory,
) -> Result<BTreeMap<PathBuf, Document>, Vec<TextbookDiagnostic>> {
    let snapshots = repository
        .bounded_regular_single_link_file_snapshots_with_extension(
            Path::new(KNOWLEDGE_ROOT),
            "md",
            "textbook prose",
            MAX_MARKDOWN_BYTES,
            FileTreeLimits {
                max_depth: MAX_PACKET_DEPTH,
                max_entries: MAX_PACKET_ENTRIES,
                max_matching_files: MAX_PACKET_MARKDOWN_FILES,
                max_aggregate_bytes: MAX_PACKET_MARKDOWN_BYTES,
            },
        )
        .map_err(|error| vec![snapshot_discovery_diagnostic(error)])?;
    let mut documents = BTreeMap::new();
    let mut identities = BTreeMap::<String, PathBuf>::new();
    let mut diagnostics = Vec::new();
    for (path, bytes) in snapshots {
        if bytes.contains(&b'\0') || bytes.contains(&b'\r') {
            diagnostics.push(diagnostic(
                ENCODING_CODE,
                None,
                "contents",
                "UTF-8 text with LF line endings and no NUL bytes",
                "contains NUL or carriage return",
                path.clone(),
            ));
            continue;
        }
        let markdown = match String::from_utf8(bytes) {
            Ok(markdown) => markdown,
            Err(error) => {
                diagnostics.push(diagnostic(
                    ENCODING_CODE,
                    None,
                    "contents",
                    "valid UTF-8",
                    error.to_string(),
                    path.clone(),
                ));
                continue;
            }
        };
        let display_path = path.to_string_lossy();
        let id = match canonical_frontmatter_identity(&markdown, &display_path) {
            Ok(id) => id,
            Err(error) => {
                diagnostics.push(diagnostic(
                    FRONTMATTER_CODE,
                    None,
                    "frontmatter",
                    "explicit canonical frontmatter id",
                    error.message,
                    path.clone(),
                ));
                continue;
            }
        };
        if let Some(previous) = identities.insert(id.clone(), path.clone()) {
            diagnostics.push(diagnostic(
                DUPLICATE_ID_CODE,
                Some(id.clone()),
                "id",
                "unique canonical document identity",
                format!("also used by {}", previous.display()),
                path.clone(),
            ));
        }
        let body = match canonical_markdown_body(&markdown, &display_path) {
            Ok(body) => body,
            Err(error) => {
                diagnostics.push(diagnostic(
                    FRONTMATTER_CODE,
                    Some(id),
                    "frontmatter",
                    "closed canonical frontmatter",
                    error.message,
                    path.clone(),
                ));
                continue;
            }
        };
        let body_offset = body.as_ptr() as usize - markdown.as_ptr() as usize;
        let body_start_line = u32::try_from(
            1 + markdown.as_bytes()[..body_offset]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count(),
        )
        .unwrap_or(u32::MAX);
        let body = body.to_owned();
        let heading_ids = match canonical_heading_ids(&body, &display_path) {
            Ok(ids) => ids,
            Err(error) => {
                diagnostics.push(diagnostic(
                    HEADING_CODE,
                    None,
                    "heading_id",
                    "unique valid canonical heading IDs",
                    error.message,
                    path.clone(),
                ));
                Vec::new()
            }
        };
        documents.insert(
            path.clone(),
            Document {
                id,
                path,
                body,
                markdown,
                heading_ids,
                body_start_line,
            },
        );
    }
    if diagnostics.is_empty() {
        Ok(documents)
    } else {
        Err(diagnostics)
    }
}

fn read_compatibility_routes(
    repository: &HeldDirectory,
) -> Result<CompatibilityRoutes, Vec<TextbookDiagnostic>> {
    let path = PathBuf::from(COMPATIBILITY_PATH);
    let bytes = repository
        .read_optional_regular_single_link_file_bounded(
            &path,
            "textbook compatibility routes",
            MAX_COMPATIBILITY_BYTES,
        )
        .map_err(|error| {
            vec![input_diagnostic(
                Some(path.clone()),
                "textbook compatibility JSON",
                MAX_COMPATIBILITY_BYTES,
                "regular single-link",
                error.message,
            )]
        })?
        .ok_or_else(|| {
            vec![input_diagnostic(
                Some(path.clone()),
                "textbook compatibility JSON",
                MAX_COMPATIBILITY_BYTES,
                "regular single-link",
                "missing",
            )]
        })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        vec![diagnostic(
            ROUTE_CODE,
            None,
            "$",
            "strict compatibility route contract",
            error.to_string(),
            path,
        )]
    })
}

pub(super) fn compatibility_routes(
    repository: &HeldDirectory,
) -> Result<CompatibilityRoutes, Vec<TextbookDiagnostic>> {
    read_compatibility_routes(repository)
}

fn validate_routes(
    documents: &BTreeMap<PathBuf, Document>,
    routes: &CompatibilityRoutes,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    if routes.schema_version != COMPATIBILITY_SCHEMA {
        diagnostics.push(diagnostic(
            ROUTE_CODE,
            None,
            "schema_version",
            COMPATIBILITY_SCHEMA,
            &routes.schema_version,
            PathBuf::from(COMPATIBILITY_PATH),
        ));
    }
    let mut mapped_paths = BTreeSet::new();
    for route in &routes.routes {
        if !is_textbook_markdown_path(&route.canonical_path) {
            diagnostics.push(diagnostic(
                ROUTE_CODE,
                Some(route.canonical_id.clone()),
                "canonical_path",
                "normal .md path beneath knowledge/crouzeix_textbook",
                route.canonical_path.display().to_string(),
                PathBuf::from(COMPATIBILITY_PATH),
            ));
            continue;
        }
        if !mapped_paths.insert(route.canonical_path.clone()) {
            diagnostics.push(diagnostic(
                ROUTE_CODE,
                Some(route.canonical_id.clone()),
                "canonical_path",
                "one route per discovered document",
                route.canonical_path.display().to_string(),
                PathBuf::from(COMPATIBILITY_PATH),
            ));
        }
        match documents.get(&route.canonical_path) {
            Some(document) if document.id == route.canonical_id => {}
            Some(document) => diagnostics.push(diagnostic(
                ROUTE_CODE,
                Some(route.canonical_id.clone()),
                "canonical_id",
                &document.id,
                &route.canonical_id,
                PathBuf::from(COMPATIBILITY_PATH),
            )),
            None => diagnostics.push(diagnostic(
                ROUTE_CODE,
                Some(route.canonical_id.clone()),
                "canonical_path",
                "discovered canonical document",
                route.canonical_path.display().to_string(),
                PathBuf::from(COMPATIBILITY_PATH),
            )),
        }
    }
    for document in documents.values() {
        if !mapped_paths.contains(&document.path) {
            diagnostics.push(diagnostic(
                ROUTE_CODE,
                Some(document.id.clone()),
                "canonical_path",
                "exactly one compatibility route",
                "missing",
                PathBuf::from(COMPATIBILITY_PATH),
            ));
        }
    }
}

fn validate_alias_namespace(
    documents: &BTreeMap<PathBuf, Document>,
    routes: &[CompatibilityRoute],
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let canonical = documents
        .values()
        .map(|document| (document.id.as_str(), &document.path))
        .collect::<BTreeMap<_, _>>();
    let mut aliases = BTreeSet::new();
    for route in routes {
        if !aliases.insert(route.legacy_concept_id.as_str()) {
            collision(route, "duplicate compatibility alias", diagnostics);
        }
        if canonical
            .get(route.legacy_concept_id.as_str())
            .is_some_and(|path| **path != route.canonical_path)
        {
            collision(route, "canonical identity of another document", diagnostics);
        }
    }
}

fn validate_namespace(
    documents: &BTreeMap<PathBuf, Document>,
    routes: &[CompatibilityRoute],
    contracts: &TextbookContracts,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let canonical = documents
        .values()
        .map(|document| document.id.as_str())
        .collect::<BTreeSet<_>>();
    let aliases = routes
        .iter()
        .map(|route| route.legacy_concept_id.as_str())
        .collect::<BTreeSet<_>>();
    let theorem_ids = contracts
        .theorems()
        .iter()
        .map(TheoremRow::item_id)
        .collect::<BTreeSet<_>>();
    let exercise_ids = contracts
        .exercises()
        .iter()
        .map(ExerciseRow::exercise_id)
        .collect::<BTreeSet<_>>();

    for route in routes {
        if theorem_ids.contains(route.legacy_concept_id.as_str())
            || exercise_ids.contains(route.legacy_concept_id.as_str())
        {
            collision(route, "theorem or exercise identity", diagnostics);
        }
    }
    for theorem in contracts.theorems() {
        if canonical.contains(theorem.item_id())
            || aliases.contains(theorem.item_id())
            || exercise_ids.contains(theorem.item_id())
        {
            diagnostics.push(namespace_diagnostic(theorem.item_id(), "item_id"));
        }
    }
    for exercise in contracts.exercises() {
        if canonical.contains(exercise.exercise_id())
            || aliases.contains(exercise.exercise_id())
            || theorem_ids.contains(exercise.exercise_id())
        {
            diagnostics.push(namespace_diagnostic(exercise.exercise_id(), "exercise_id"));
        }
    }
}

fn validate_theorems(
    documents: &BTreeMap<PathBuf, Document>,
    theorems: &[TheoremRow],
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    for theorem in theorems {
        if !is_textbook_markdown_path(&theorem.prose_path) {
            diagnostics.push(diagnostic(
                THEOREM_PATH_CODE,
                Some(theorem.item_id.clone()),
                "prose_path",
                "normal .md path beneath knowledge/crouzeix_textbook",
                theorem.prose_path.display().to_string(),
                PathBuf::from("content/crouzeix_textbook/coverage.json"),
            ));
            continue;
        }
        let Some(document) = documents.get(&theorem.prose_path) else {
            diagnostics.push(diagnostic(
                THEOREM_PATH_CODE,
                Some(theorem.item_id.clone()),
                "prose_path",
                "one discovered canonical document",
                theorem.prose_path.display().to_string(),
                PathBuf::from("content/crouzeix_textbook/coverage.json"),
            ));
            continue;
        };
        let count = document
            .heading_ids
            .iter()
            .filter(|heading| *heading == &theorem.anchor)
            .count();
        if count != 1 {
            diagnostics.push(diagnostic(
                THEOREM_ANCHOR_CODE,
                Some(theorem.item_id.clone()),
                "anchor",
                "exactly one canonical heading in prose_path",
                format!("{} occurrences of {}", count, theorem.anchor),
                theorem.prose_path.clone(),
            ));
            continue;
        }
        if theorem.prose_proof_status() != ProseProofStatus::Reconstructible {
            continue;
        }
        let anchors = theorems
            .iter()
            .filter(|row| row.prose_path == theorem.prose_path)
            .map(|row| row.anchor.as_str())
            .collect::<BTreeSet<_>>();
        let body = theorem_card_body(document, &theorem.anchor, &anchors);
        let subsection_counts = theorem_card_subsection_counts(body);
        if subsection_counts.is_empty() {
            let fields = narrative_fields(body);
            if !fields.is_empty() {
                let invalid = ["Statement", "Proof", "Boundary", "Formal correspondence"]
                    .into_iter()
                    .filter_map(|label| {
                        let matches = fields
                            .iter()
                            .filter(|field| field.labels.contains(&label))
                            .collect::<Vec<_>>();
                        let valid = matches.len() == 1
                            && matches[0].has_content
                            && (label != "Formal correspondence" || matches[0].has_provider);
                        (!valid).then(|| {
                            format!(
                                "{label}: expected one nonempty field{}; found {}",
                                if label == "Formal correspondence" {
                                    " naming a Lean provider"
                                } else {
                                    ""
                                },
                                matches.len()
                            )
                        })
                    })
                    .collect::<Vec<_>>();
                if !invalid.is_empty() {
                    diagnostics.push(diagnostic(
                        THEOREM_CARD_CODE,
                        Some(theorem.item_id.clone()),
                        "subsections",
                        "narrative Statement, Proof, Boundary, and Formal correspondence with a named Lean provider",
                        invalid.join(", "),
                        theorem.prose_path.clone(),
                    ));
                }
                continue;
            }
        }
        let invalid = REQUIRED_THEOREM_SUBSECTIONS
            .iter()
            .filter_map(|heading| {
                let count = subsection_counts.get(*heading).copied().unwrap_or(0);
                (count != 1).then(|| format!("{heading}={count}"))
            })
            .collect::<Vec<_>>();
        if !invalid.is_empty() {
            diagnostics.push(diagnostic(
                THEOREM_CARD_CODE,
                Some(theorem.item_id.clone()),
                "subsections",
                format!(
                    "exactly one each of {}",
                    REQUIRED_THEOREM_SUBSECTIONS.join(", ")
                ),
                invalid.join(", "),
                theorem.prose_path.clone(),
            ));
        }
    }
}

// Reuse the canonical heading identities, including generated slugs. A nested
// registered card ends the narrative just as a sibling heading does.
fn theorem_card_body<'a>(
    document: &'a Document,
    target: &str,
    anchors: &BTreeSet<&str>,
) -> &'a str {
    let mut ids = document.heading_ids.iter();
    let mut card = None;
    let mut pending_level = None;
    let mut quote_depth = 0;
    for (event, range) in
        Parser::new_ext(&document.body, theorem_markdown_options()).into_offset_iter()
    {
        match event {
            Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => quote_depth -= 1,
            Event::Start(Tag::Heading { level, .. }) => {
                let id = match level {
                    HeadingLevel::H2 | HeadingLevel::H3 => ids.next().map(String::as_str),
                    _ => None,
                };
                let level = heading_level_number(level);
                if quote_depth == 0 {
                    if let Some((card_level, start)) = card {
                        if level <= card_level || id.is_some_and(|id| anchors.contains(id)) {
                            return &document.body[start..range.start];
                        }
                    }
                    if id == Some(target) {
                        pending_level = Some(level);
                    }
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(level) = pending_level.take() {
                    card = Some((level, range.end));
                }
            }
            _ => {}
        }
    }
    card.map_or("", |(_, start)| &document.body[start..])
}

struct NarrativeField {
    labels: &'static [&'static str],
    has_content: bool,
    has_provider: bool,
}

fn narrative_label(text: &str) -> Option<&'static [&'static str]> {
    match text.trim().trim_end_matches(['.', ':']) {
        "Statement" => Some(&["Statement"]),
        "Proof" => Some(&["Proof"]),
        "Boundary" | "Boundary case" => Some(&["Boundary"]),
        "Formal correspondence" | "Lean correspondence" => Some(&["Formal correspondence"]),
        "Boundary and Lean provider" | "Boundary and Lean providers" => {
            Some(&["Boundary", "Formal correspondence"])
        }
        _ => None,
    }
}

fn named_lean_identifier(text: &str) -> bool {
    text.contains(['.', '_'])
        && text.chars().any(char::is_alphabetic)
        && text.split('.').all(|component| {
            let mut characters = component.chars();
            characters
                .next()
                .is_some_and(|first| first.is_alphabetic() || first == '_')
                && characters.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '\''))
        })
}

fn narrative_fields(body: &str) -> Vec<NarrativeField> {
    let mut fields = Vec::<NarrativeField>::new();
    let mut active = None::<usize>;
    let mut depth = 0;
    let mut paragraph = false;
    let mut prefix = false;
    let mut label = None::<String>;
    for event in Parser::new_ext(body, theorem_markdown_options()) {
        if let Some(text) = &mut label {
            if !matches!(&event, Event::Text(_) | Event::End(TagEnd::Strong)) {
                // Labels are plain bold text. Do not synthesize a label by
                // dropping breaks, HTML, or nested inline formatting.
                text.push('\0');
            }
        }
        match event {
            Event::Start(tag) => {
                match &tag {
                    Tag::Paragraph if depth == 0 => {
                        paragraph = true;
                        prefix = true;
                    }
                    Tag::Strong if paragraph && depth == 1 && prefix => {
                        // Any leading bold label ends the previous field; unrelated
                        // labels cannot fill an empty Statement or Proof.
                        active = None;
                        label = Some(String::new());
                    }
                    Tag::Link { dest_url, .. } if paragraph && label.is_none() => {
                        if let Some(index) = active {
                            fields[index].has_provider |= dest_url
                                .split(['#', '?'])
                                .next()
                                .is_some_and(|path| path.ends_with(".lean"));
                        }
                    }
                    _ => {}
                }
                depth += 1;
            }
            Event::End(tag) => {
                depth -= 1;
                if tag == TagEnd::Strong && depth == 1 {
                    if let Some(text) = label.take() {
                        if let Some(labels) = narrative_label(&text) {
                            active = Some(fields.len());
                            fields.push(NarrativeField {
                                labels,
                                has_content: false,
                                has_provider: false,
                            });
                        }
                    }
                }
                if tag == TagEnd::Paragraph && depth == 0 {
                    paragraph = false;
                }
            }
            Event::Text(text) if label.is_some() => {
                label.as_mut().expect("label is present").push_str(&text);
            }
            Event::Code(text) if paragraph && label.is_none() => {
                prefix = false;
                if let Some(index) = active {
                    fields[index].has_content |= text.chars().any(char::is_alphanumeric);
                    fields[index].has_provider |= named_lean_identifier(&text);
                }
            }
            Event::Text(text) | Event::InlineMath(text) if paragraph && label.is_none() => {
                if !text.trim().is_empty() {
                    prefix = false;
                }
                if let Some(index) = active {
                    fields[index].has_content |= text.chars().any(char::is_alphanumeric);
                }
            }
            Event::DisplayMath(text) if depth == 0 => {
                if let Some(index) = active {
                    fields[index].has_content |= text.chars().any(char::is_alphanumeric);
                }
            }
            _ => {}
        }
    }
    fields
}

fn theorem_markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}

fn theorem_card_subsection_counts(body: &str) -> BTreeMap<String, usize> {
    let mut current = None::<String>;
    let mut subsection_counts = BTreeMap::new();
    let mut quote_depth = 0;
    for event in Parser::new_ext(body, theorem_markdown_options()) {
        match event {
            Event::Start(Tag::BlockQuote(_)) => quote_depth += 1,
            Event::End(TagEnd::BlockQuote(_)) => quote_depth -= 1,
            Event::Start(Tag::Heading { .. }) if quote_depth == 0 => {
                current = Some(String::new());
            }
            Event::Text(text) | Event::Code(text) | Event::InlineMath(text) => {
                if let Some(heading) = &mut current {
                    heading.push_str(&text);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(heading) = current.take() {
                    if REQUIRED_THEOREM_SUBSECTIONS.contains(&heading.as_str()) {
                        *subsection_counts.entry(heading).or_insert(0) += 1;
                    }
                }
            }
            _ => {}
        }
    }
    subsection_counts
}

fn heading_level_number(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn is_textbook_markdown_path(path: &Path) -> bool {
    !path.is_absolute()
        && path.extension().and_then(|extension| extension.to_str()) == Some("md")
        && path.starts_with(KNOWLEDGE_ROOT)
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn collision(
    route: &CompatibilityRoute,
    observed: &str,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    diagnostics.push(diagnostic(
        COLLISION_CODE,
        Some(route.legacy_concept_id.clone()),
        "legacy_concept_id",
        "identity unique outside its own canonical document",
        observed,
        PathBuf::from(COMPATIBILITY_PATH),
    ));
}

fn namespace_diagnostic(identity: &str, field: &str) -> TextbookDiagnostic {
    diagnostic(
        COLLISION_CODE,
        Some(identity.to_owned()),
        field,
        "globally unique textbook identity",
        "collides with another textbook identity",
        PathBuf::from(COMPATIBILITY_PATH),
    )
}

fn discovery_diagnostic(observed: impl Into<String>) -> TextbookDiagnostic {
    TextbookDiagnostic {
        code: INPUT_CODE,
        identity: None,
        field: "file".to_owned(),
        expected: format!(
            "textbook Markdown tree of regular single-link files within depth {MAX_PACKET_DEPTH}, {MAX_PACKET_ENTRIES} entries, {MAX_PACKET_MARKDOWN_FILES} Markdown files, and {MAX_PACKET_MARKDOWN_BYTES} aggregate bytes"
        ),
        observed: observed.into(),
        path: Some(PathBuf::from(KNOWLEDGE_ROOT)),
        line: None,
        column: None,
    }
}

fn snapshot_discovery_diagnostic(error: AppError) -> TextbookDiagnostic {
    if error.code() == "fs.size" {
        let suffix = format!(": textbook prose exceeds {MAX_MARKDOWN_BYTES} bytes");
        if let Some(path) = error.message.strip_suffix(&suffix) {
            return input_diagnostic(
                Some(PathBuf::from(path)),
                "textbook Markdown",
                MAX_MARKDOWN_BYTES,
                "regular single-link",
                format!("textbook prose exceeds {MAX_MARKDOWN_BYTES} bytes"),
            );
        }
    }
    discovery_diagnostic(error.message)
}

fn input_diagnostic(
    path: Option<PathBuf>,
    input_kind: &str,
    max_bytes: usize,
    link_policy: &str,
    observed: impl Into<String>,
) -> TextbookDiagnostic {
    TextbookDiagnostic {
        code: INPUT_CODE,
        identity: None,
        field: "file".to_owned(),
        expected: format!("{link_policy} {input_kind} no larger than {max_bytes} bytes"),
        observed: observed.into(),
        path,
        line: None,
        column: None,
    }
}

fn diagnostic(
    code: &'static str,
    identity: Option<String>,
    field: impl Into<String>,
    expected: impl Into<String>,
    observed: impl Into<String>,
    path: PathBuf,
) -> TextbookDiagnostic {
    TextbookDiagnostic::new(code, identity, field, expected, observed, path)
}
