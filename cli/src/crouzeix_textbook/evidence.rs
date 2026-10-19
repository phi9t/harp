use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::corpus::{canonical_heading_ids, canonical_markdown_body};
use crate::fs::HeldDirectory;

use super::markdown::Document;
use super::{TextbookContracts, TextbookDiagnostic};

const SOURCE_REGISTRY_PATH: &str = "knowledge/crouzeix_textbook/source_registry.md";
const CLAIM_LEDGER_PATH: &str = "knowledge/crouzeix_textbook/claim_evidence_ledger.md";
const SOURCE_ID_CODE: &str = "crouzeix-textbook.evidence.source-id";
const REVIEW_SOURCE_ID_CODE: &str = "crouzeix-textbook.evidence.review-source-id";
const CLAIM_ENTRY_CODE: &str = "crouzeix-textbook.evidence.claim-entry";
const CLAIM_CLASS_CODE: &str = "crouzeix-textbook.evidence.claim-class";
const CLAIM_SOURCE_CODE: &str = "crouzeix-textbook.evidence.claim-source";
const CLAIM_LOCATOR_CODE: &str = "crouzeix-textbook.evidence.claim-locator";
const PRIORITY_ROUTE_CODE: &str = "crouzeix-textbook.evidence.priority-route";
const MAX_CLAIM_ENTRIES: usize = 256;
const MAX_LOCATOR_TARGET_BYTES: usize = 2 * 1024 * 1024;
const REQUIRED_CLAIM_FIELDS: [&str; 9] = [
    "Class",
    "Statement",
    "Source",
    "Locator",
    "Scope",
    "Reproduction",
    "Caveat",
    "Mode",
    "Source stability",
];

pub(super) fn validate(
    repository: &HeldDirectory,
    documents: &BTreeMap<PathBuf, Document>,
    contracts: Option<&TextbookContracts>,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let source_path = PathBuf::from(SOURCE_REGISTRY_PATH);
    let ledger_path = PathBuf::from(CLAIM_LEDGER_PATH);
    let source_ids = if let Some(registry) = documents.get(&source_path) {
        parse_source_registry(registry, diagnostics)
    } else {
        diagnostics.push(diagnostic(
            SOURCE_ID_CODE,
            None,
            "source_registry",
            "bounded textbook source registry",
            "missing",
            source_path,
        ));
        BTreeSet::new()
    };
    if let Some(contracts) = contracts {
        validate_contract_sources(contracts, &source_ids, diagnostics);
    }
    if let Some(ledger) = documents.get(&ledger_path) {
        validate_claim_ledger(repository, ledger, &source_ids, diagnostics);
    } else {
        diagnostics.push(diagnostic(
            CLAIM_ENTRY_CODE,
            None,
            "claim_ledger",
            "bounded textbook claim-evidence ledger",
            "missing",
            ledger_path,
        ));
    }
}

fn parse_source_registry(
    document: &Document,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) -> BTreeSet<String> {
    let mut source_ids = BTreeSet::new();
    for section in level_two_sections(&document.body) {
        let source_id = section.title.trim();
        if !is_source_id(source_id) {
            diagnostics.push(diagnostic_at(
                document,
                section.heading_offset,
                SOURCE_ID_CODE,
                Some(source_id.to_owned()),
                "heading",
                "exact `## SOURCE-ID` heading",
                source_id,
            ));
            continue;
        }
        let ParsedFields {
            values: fields,
            duplicates,
            offsets: field_offsets,
        } = parse_fields(section.body, section.body_offset);
        let mut valid_record = true;
        for (duplicate, duplicate_offset) in duplicates {
            valid_record = false;
            diagnostics.push(diagnostic_at(
                document,
                duplicate_offset,
                SOURCE_ID_CODE,
                Some(source_id.to_owned()),
                duplicate,
                "source-record field listed once",
                "duplicate",
            ));
        }
        for field in ["Identity", "Role"] {
            if fields
                .get(field)
                .is_none_or(|value| value.trim().is_empty())
            {
                valid_record = false;
                diagnostics.push(diagnostic_at(
                    document,
                    field_offsets
                        .get(field)
                        .copied()
                        .unwrap_or(section.heading_offset),
                    SOURCE_ID_CODE,
                    Some(source_id.to_owned()),
                    field,
                    "present and non-empty source-record field",
                    fields.get(field).map_or("missing", String::as_str),
                ));
            }
        }
        if ["Boundary", "Cannot support"].iter().all(|field| {
            fields
                .get(*field)
                .is_none_or(|value| value.trim().is_empty())
        }) {
            valid_record = false;
            diagnostics.push(diagnostic_at(
                document,
                section.heading_offset,
                SOURCE_ID_CODE,
                Some(source_id.to_owned()),
                "Boundary",
                "non-empty Boundary or Cannot support source-record field",
                "missing",
            ));
        }
        if !valid_record {
            continue;
        }
        if !source_ids.insert(source_id.to_owned()) {
            diagnostics.push(diagnostic_at(
                document,
                section.heading_offset,
                SOURCE_ID_CODE,
                Some(source_id.to_owned()),
                "heading",
                "unique source ID heading",
                "duplicate",
            ));
        }
    }
    if source_ids.is_empty() {
        diagnostics.push(diagnostic(
            SOURCE_ID_CODE,
            None,
            "heading",
            "at least one exact `## SOURCE-ID` heading",
            "none",
            document.path.clone(),
        ));
    }
    source_ids
}

fn validate_contract_sources(
    contracts: &TextbookContracts,
    source_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let path = PathBuf::from(super::contract::COVERAGE_PATH);
    for row in contracts.theorems() {
        for source_id in &row.source_ids {
            if !source_ids.contains(source_id) {
                diagnostics.push(diagnostic(
                    SOURCE_ID_CODE,
                    Some(row.item_id.clone()),
                    "source_ids",
                    "source ID registered by an exact source-registry heading",
                    source_id,
                    path.clone(),
                ));
            }
        }
        let review_source = row.review_status.source_id();
        let listed = row
            .source_ids
            .iter()
            .any(|source_id| source_id == review_source);
        let registered = source_ids.contains(review_source);
        if !listed || !registered {
            diagnostics.push(diagnostic(
                REVIEW_SOURCE_ID_CODE,
                Some(row.item_id.clone()),
                "review_status.source_id",
                "registered source ID also listed in source_ids",
                review_source,
                path.clone(),
            ));
        }
    }
}

fn validate_claim_ledger(
    repository: &HeldDirectory,
    document: &Document,
    source_ids: &BTreeSet<String>,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let sections = level_two_sections(&document.body);
    if sections.len() > MAX_CLAIM_ENTRIES {
        diagnostics.push(diagnostic(
            CLAIM_ENTRY_CODE,
            None,
            "entries",
            format!("at most {MAX_CLAIM_ENTRIES} claim entries"),
            sections.len().to_string(),
            document.path.clone(),
        ));
    }
    if sections.is_empty() {
        diagnostics.push(diagnostic(
            CLAIM_ENTRY_CODE,
            None,
            "entries",
            "at least one CFT-CL-NNN claim entry",
            "none",
            document.path.clone(),
        ));
    }
    let mut claim_ids = BTreeSet::new();
    for section in sections.into_iter().take(MAX_CLAIM_ENTRIES) {
        let claim_id = section
            .title
            .split_once(':')
            .map_or(section.title.as_str(), |(identity, _)| identity)
            .trim();
        let identity = Some(claim_id.to_owned());
        if !is_claim_id(claim_id) {
            diagnostics.push(diagnostic_at(
                document,
                section.heading_offset,
                CLAIM_ENTRY_CODE,
                identity,
                "heading_id",
                "unique CFT-CL-NNN heading identity",
                &section.title,
            ));
            continue;
        }
        if !claim_ids.insert(claim_id.to_owned()) {
            diagnostics.push(diagnostic_at(
                document,
                section.heading_offset,
                CLAIM_ENTRY_CODE,
                identity.clone(),
                "heading_id",
                "unique CFT-CL-NNN heading identity",
                "duplicate",
            ));
        }
        let expected_anchor = claim_id.to_ascii_lowercase();
        if section.anchor.as_deref() != Some(expected_anchor.as_str()) {
            diagnostics.push(diagnostic_at(
                document,
                section.heading_offset,
                CLAIM_ENTRY_CODE,
                identity.clone(),
                "heading_id",
                format!("explicit stable anchor {expected_anchor}"),
                section.anchor.as_deref().unwrap_or("missing"),
            ));
        }

        let ParsedFields {
            values: fields,
            duplicates,
            offsets: field_offsets,
        } = parse_fields(section.body, section.body_offset);
        for (duplicate, duplicate_offset) in duplicates {
            diagnostics.push(diagnostic_at(
                document,
                duplicate_offset,
                CLAIM_ENTRY_CODE,
                identity.clone(),
                duplicate,
                "field listed once",
                "duplicate",
            ));
        }
        for field in REQUIRED_CLAIM_FIELDS {
            if fields
                .get(field)
                .is_none_or(|value| value.trim().is_empty())
            {
                diagnostics.push(diagnostic_at(
                    document,
                    field_offsets
                        .get(field)
                        .copied()
                        .unwrap_or(section.heading_offset),
                    if field == "Locator" {
                        CLAIM_LOCATOR_CODE
                    } else {
                        CLAIM_ENTRY_CODE
                    },
                    identity.clone(),
                    field,
                    "present and non-empty",
                    fields.get(field).map_or("missing", |value| value.as_str()),
                ));
            }
        }
        if let Some(class) = fields.get("Class") {
            let class = trim_rendering(class);
            if !matches!(class, "EVIDENCE" | "SOURCE CLAIM" | "INFERENCE") {
                diagnostics.push(diagnostic_at(
                    document,
                    field_offsets
                        .get("Class")
                        .copied()
                        .unwrap_or(section.heading_offset),
                    CLAIM_CLASS_CODE,
                    identity.clone(),
                    "Class",
                    "one of EVIDENCE, SOURCE CLAIM, INFERENCE",
                    class,
                ));
            }
        }
        if let Some(source) = fields.get("Source") {
            validate_claim_source(
                ClaimSourceValidation {
                    claim_id,
                    field: "Source",
                    value: source,
                    source_ids,
                    document,
                    code: CLAIM_SOURCE_CODE,
                    offset: field_offsets.get("Source").copied(),
                },
                diagnostics,
            );
        }
        if let Some(locator) = fields.get("Locator") {
            if !locator.trim().is_empty()
                && !validate_stable_locator(repository, locator).unwrap_or(false)
            {
                diagnostics.push(diagnostic_at(
                    document,
                    field_offsets
                        .get("Locator")
                        .copied()
                        .unwrap_or(section.heading_offset),
                    CLAIM_LOCATOR_CODE,
                    identity.clone(),
                    "Locator",
                    "non-empty stable code, artifact, section, table, figure, or dated semantic locator",
                    locator,
                ));
            }
        }
        if fields.get("Priority claim").is_some_and(|value| {
            matches!(
                trim_rendering(value).to_ascii_lowercase().as_str(),
                "yes" | "true"
            )
        }) || asserts_priority(&section.title)
            || fields
                .get("Statement")
                .is_some_and(|statement| asserts_priority(statement))
        {
            validate_priority_route(
                repository,
                claim_id,
                &fields,
                &field_offsets,
                source_ids,
                document,
                diagnostics,
            );
        }
    }
}

fn validate_priority_route(
    repository: &HeldDirectory,
    claim_id: &str,
    fields: &BTreeMap<String, String>,
    field_offsets: &BTreeMap<String, usize>,
    source_ids: &BTreeSet<String>,
    document: &Document,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let main_sources = fields
        .get("Source")
        .map_or_else(BTreeSet::new, |source| parsed_source_identities(source));
    let priority_sources = match fields.get("Priority evidence source") {
        Some(source) if !source.trim().is_empty() => validate_claim_source(
            ClaimSourceValidation {
                claim_id,
                field: "Priority evidence source",
                value: source,
                source_ids,
                document,
                code: PRIORITY_ROUTE_CODE,
                offset: field_offsets.get("Priority evidence source").copied(),
            },
            diagnostics,
        ),
        observed => {
            diagnostics.push(diagnostic_at(
                document,
                field_offsets
                    .get("Priority evidence source")
                    .copied()
                    .unwrap_or(0),
                PRIORITY_ROUTE_CODE,
                Some(claim_id.to_owned()),
                "Priority evidence source",
                "separate registered source supporting the priority assertion",
                observed.map_or("missing", String::as_str),
            ));
            BTreeSet::new()
        }
    };
    if !priority_sources.is_empty() && !main_sources.is_disjoint(&priority_sources) {
        diagnostics.push(diagnostic_at(
            document,
            field_offsets
                .get("Priority evidence source")
                .copied()
                .unwrap_or(0),
            PRIORITY_ROUTE_CODE,
            Some(claim_id.to_owned()),
            "Priority evidence source",
            "source identity separate from the main claim source",
            fields
                .get("Priority evidence source")
                .map_or("missing", String::as_str),
        ));
    }
    match fields.get("Priority evidence locator") {
        Some(locator)
            if validate_stable_locator(repository, locator).unwrap_or(false)
                && fields
                    .get("Locator")
                    .is_none_or(|main| !locators_share_identity(main, locator)) => {}
        observed => diagnostics.push(diagnostic_at(
            document,
            field_offsets
                .get("Priority evidence locator")
                .copied()
                .unwrap_or(0),
            PRIORITY_ROUTE_CODE,
            Some(claim_id.to_owned()),
            "Priority evidence locator",
            "separate non-empty stable exact locator supporting the priority assertion",
            observed.map_or("missing", String::as_str),
        )),
    }
}

struct ClaimSourceValidation<'a> {
    claim_id: &'a str,
    field: &'a str,
    value: &'a str,
    source_ids: &'a BTreeSet<String>,
    document: &'a Document,
    code: &'static str,
    offset: Option<usize>,
}

fn validate_claim_source(
    validation: ClaimSourceValidation<'_>,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) -> BTreeSet<String> {
    let routes = parse_source_routes(validation.value);
    if routes.is_empty() {
        diagnostics.push(diagnostic_at(
            validation.document,
            validation.offset.unwrap_or(0),
            validation.code,
            Some(validation.claim_id.to_owned()),
            validation.field,
            "at least one parsed source identity registered by an exact source-registry heading",
            validation.value,
        ));
        return BTreeSet::new();
    }
    let mut identities = BTreeSet::new();
    for route in routes {
        identities.insert(route.identity.clone());
        let registered = validation.source_ids.contains(&route.identity);
        let valid_route = normalize_source_registry_target(&route.target) == SOURCE_REGISTRY_PATH
            && route.anchor.as_deref() == Some(heading_anchor(&route.identity).as_str());
        if !registered || !valid_route {
            diagnostics.push(diagnostic_at(
                validation.document,
                validation.offset.unwrap_or(0),
                validation.code,
                Some(validation.claim_id.to_owned()),
                validation.field,
                "registered source ID routed through the textbook source registry and its canonical anchor",
                route.rendered,
            ));
        }
    }
    identities
}

#[derive(Debug)]
struct SourceRoute {
    identity: String,
    target: String,
    anchor: Option<String>,
    rendered: String,
}

fn parse_source_routes(value: &str) -> Vec<SourceRoute> {
    let mut routes = Vec::new();
    let mut remaining = value;
    while let Some(start) = remaining.find("[[") {
        if !is_source_separator(&remaining[..start]) {
            return Vec::new();
        }
        let after_start = &remaining[start + 2..];
        let Some(end) = after_start.find("]]") else {
            return Vec::new();
        };
        let rendered = &after_start[..end];
        let (destination, display) = rendered
            .rsplit_once('|')
            .map_or((rendered, None), |(destination, display)| {
                (destination, Some(display.trim()))
            });
        let (target, anchor) = destination
            .split_once('#')
            .map_or((destination.trim(), None), |(target, anchor)| {
                (target.trim(), Some(anchor.trim()))
            });
        let identity = display.or(anchor).unwrap_or_default();
        if !is_source_identity(identity) {
            return Vec::new();
        }
        routes.push(SourceRoute {
            identity: identity.to_owned(),
            target: target.to_owned(),
            anchor: anchor.map(str::to_owned),
            rendered: format!("[[{rendered}]]"),
        });
        remaining = &after_start[end + 2..];
    }
    if !routes.is_empty() {
        return if is_source_separator(remaining) {
            routes
        } else {
            Vec::new()
        };
    }
    Vec::new()
}

fn parsed_source_identities(value: &str) -> BTreeSet<String> {
    parse_source_routes(value)
        .into_iter()
        .map(|route| route.identity)
        .collect()
}

fn normalize_source_registry_target(target: &str) -> String {
    target.strip_suffix(".md").unwrap_or(target).to_owned() + ".md"
}

fn is_source_separator(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_whitespace() || matches!(character, ',' | ';' | '.'))
}

fn is_source_identity(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '.'))
}

fn heading_anchor(source_id: &str) -> String {
    let mut anchor = String::new();
    for character in source_id.chars() {
        if character.is_ascii_alphanumeric() {
            anchor.push(character.to_ascii_lowercase());
        } else if !anchor.ends_with('-') {
            anchor.push('-');
        }
    }
    anchor.trim_matches('-').to_owned()
}

fn is_source_id(value: &str) -> bool {
    value
        .chars()
        .any(|character| character.is_ascii_uppercase() || character.is_ascii_digit())
        && value.chars().all(|character| {
            character.is_ascii_uppercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '.')
        })
}

fn is_claim_id(value: &str) -> bool {
    value
        .strip_prefix("CFT-CL-")
        .is_some_and(|suffix| suffix.len() == 3 && suffix.chars().all(|c| c.is_ascii_digit()))
}

fn trim_rendering(value: &str) -> &str {
    value.trim().trim_matches('`').trim()
}

fn validate_stable_locator(
    repository: &HeldDirectory,
    value: &str,
) -> Result<bool, LocatorParseError> {
    let locator = value.trim();
    if !locator
        .chars()
        .any(|character| character.is_ascii_alphanumeric())
    {
        return Ok(false);
    }
    let lower = locator.to_ascii_lowercase();
    let phrase = lower
        .trim_matches(|character: char| !character.is_ascii_alphanumeric())
        .trim();
    if phrase.starts_with("current document")
        || phrase.starts_with("current page")
        || matches!(
            phrase,
            "above" | "below" | "here" | "tbd" | "todo" | "unknown" | "missing" | "none"
        )
    {
        return Ok(false);
    }

    let routes = locator_routes(locator)?;
    if !routes.is_empty() {
        return Ok(routes
            .iter()
            .all(|route| resolve_locator_route(repository, route)));
    }
    if locator.contains("[[") || locator.contains("](") || locator.contains('`') {
        return Ok(false);
    }
    Ok(has_numbered_semantic_marker(locator))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct LocatorRoute {
    path: PathBuf,
    anchor: Option<String>,
    line: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LocatorParseError;

fn locator_routes(locator: &str) -> Result<Vec<LocatorRoute>, LocatorParseError> {
    let mut candidates = Vec::new();
    let mut covered = vec![false; locator.len()];
    let mut remaining = locator;
    let mut base = 0usize;
    while let Some(start) = remaining.find("[[") {
        let after = &remaining[start + 2..];
        let Some(end) = after.find("]]") else {
            return Err(LocatorParseError);
        };
        let target = after[..end]
            .split_once('|')
            .map_or(&after[..end], |(target, _)| target);
        candidates.push((target.trim().to_owned(), true));
        let consumed = start + 2 + end + 2;
        covered[base + start..base + consumed].fill(true);
        base += consumed;
        remaining = &remaining[consumed..];
    }

    remaining = locator;
    base = 0;
    while let Some(start) = remaining.find("](") {
        let after = &remaining[start + 2..];
        let Some(end) = after.find(')') else {
            return Err(LocatorParseError);
        };
        candidates.push((after[..end].trim().to_owned(), false));
        let consumed = start + 2 + end + 1;
        covered[base + start..base + consumed].fill(true);
        base += consumed;
        remaining = &remaining[consumed..];
    }

    remaining = locator;
    base = 0;
    while let Some(start) = remaining.find('`') {
        let after = &remaining[start + 1..];
        let Some(end) = after.find('`') else {
            return Err(LocatorParseError);
        };
        let candidate = after[..end].trim();
        if candidate.is_empty() {
            return Err(LocatorParseError);
        }
        candidates.push((candidate.to_owned(), false));
        let consumed = start + 1 + end + 1;
        covered[base + start..base + consumed].fill(true);
        base += consumed;
        remaining = &remaining[consumed..];
    }

    let uncovered = locator
        .bytes()
        .enumerate()
        .map(|(index, byte)| if covered[index] { b' ' } else { byte })
        .collect::<Vec<_>>();
    let uncovered = String::from_utf8(uncovered).map_err(|_| LocatorParseError)?;
    if uncovered.contains("[[")
        || uncovered.contains("]]")
        || uncovered.contains("](")
        || uncovered.contains('`')
    {
        return Err(LocatorParseError);
    }
    for token in uncovered.split_whitespace() {
        let candidate = token.trim_matches(|character: char| {
            matches!(
                character,
                ',' | ';' | '(' | ')' | '[' | ']' | '`' | '“' | '”'
            )
        });
        if candidate.contains('/') {
            candidates.push((candidate.trim_end_matches('.').to_owned(), false));
        }
    }

    candidates
        .into_iter()
        .map(|(candidate, wiki)| parse_locator_route(&candidate, wiki).ok_or(LocatorParseError))
        .collect::<Result<BTreeSet<_>, _>>()
        .map(|routes| routes.into_iter().collect())
}

fn parse_locator_route(candidate: &str, wiki: bool) -> Option<LocatorRoute> {
    let candidate = candidate.trim().trim_end_matches('.');
    if is_bare_declaration_component(candidate) {
        // Bare declaration names have no exact repository file and position,
        // so the bounded locator contract recognizes but does not accept them.
        return None;
    }
    let (without_fragment, fragment) = candidate
        .split_once('#')
        .map_or((candidate, None), |(path, fragment)| (path, Some(fragment)));
    let (path, colon_line) =
        without_fragment
            .rsplit_once(':')
            .map_or((without_fragment, None), |(path, suffix)| {
                suffix
                    .parse::<usize>()
                    .ok()
                    .map_or((without_fragment, None), |line| (path, Some(line)))
            });
    let (anchor, hash_line) = fragment.map_or((None, None), |fragment| {
        let line = fragment
            .strip_prefix('L')
            .and_then(|suffix| suffix.split('-').next())
            .and_then(|suffix| suffix.parse::<usize>().ok());
        if line.is_some() {
            (None, line)
        } else {
            (Some(fragment.to_owned()), None)
        }
    });
    let mut path = path.trim().to_owned();
    if wiki && Path::new(&path).extension().is_none() {
        path.push_str(".md");
    }
    (!path.is_empty() && has_repository_extension(&path)).then_some(LocatorRoute {
        path: PathBuf::from(path),
        anchor,
        line: colon_line.or(hash_line),
    })
}

fn is_bare_declaration_component(value: &str) -> bool {
    let mut segments = value.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    let mut count = 1usize;
    if !is_lean_identifier(first) {
        return false;
    }
    for segment in segments {
        count += 1;
        if !is_lean_identifier(segment) {
            return false;
        }
    }
    count > 1
}

fn is_lean_identifier(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '\''))
}

fn has_repository_extension(value: &str) -> bool {
    Path::new(value)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "lean" | "rs" | "md" | "json" | "toml" | "ts" | "tsx" | "pdf"
            )
        })
}

fn resolve_locator_route(repository: &HeldDirectory, route: &LocatorRoute) -> bool {
    let bytes = match repository.read_optional_regular_single_link_file_bounded(
        &route.path,
        "claim locator target",
        MAX_LOCATOR_TARGET_BYTES,
    ) {
        Ok(Some(bytes)) => bytes,
        _ => return false,
    };
    if let Some(line) = route.line {
        if line == 0 || line > bytes.split(|byte| *byte == b'\n').count() {
            return false;
        }
    }
    if let Some(anchor) = &route.anchor {
        if route.path.extension().and_then(|value| value.to_str()) != Some("md") {
            return false;
        }
        let Ok(markdown) = std::str::from_utf8(&bytes) else {
            return false;
        };
        if canonical_markdown_body(markdown, &route.path.to_string_lossy()).is_err() {
            return false;
        }
        let Ok(headings) = canonical_heading_ids(markdown, &route.path.to_string_lossy()) else {
            return false;
        };
        if !headings.iter().any(|heading| heading == anchor) {
            return false;
        }
    }
    true
}

fn normalize_locator_text(locator: &str) -> String {
    locator
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_end_matches('.')
        .to_owned()
}

fn locators_share_identity(main: &str, priority: &str) -> bool {
    match (locator_routes(main), locator_routes(priority)) {
        (Ok(main_routes), Ok(priority_routes))
            if !main_routes.is_empty() && !priority_routes.is_empty() =>
        {
            let main_routes = main_routes.into_iter().collect::<BTreeSet<_>>();
            let priority_routes = priority_routes.into_iter().collect::<BTreeSet<_>>();
            !main_routes.is_disjoint(&priority_routes)
        }
        (Ok(main_routes), Ok(priority_routes))
            if main_routes.is_empty() && priority_routes.is_empty() =>
        {
            normalize_locator_text(main) == normalize_locator_text(priority)
        }
        _ => false,
    }
}

fn has_numbered_semantic_marker(locator: &str) -> bool {
    let lower = locator.to_ascii_lowercase();
    [
        "pages",
        "page",
        "chapter",
        "section",
        "equation",
        "theorem",
        "lemma",
        "table",
        "figure",
        "appendix",
        "algorithm",
        "proposition",
        "corollary",
    ]
    .iter()
    .any(|marker| {
        lower.match_indices(marker).any(|(index, _)| {
            let suffix = lower[index + marker.len()..].trim_start();
            let token = suffix
                .chars()
                .take_while(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '.' | '(' | ')')
                })
                .collect::<String>();
            !token.is_empty()
                && (token.chars().any(|character| character.is_ascii_digit())
                    || token.chars().all(|character| {
                        matches!(character, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm')
                    }))
        })
    })
}

fn asserts_priority(statement: &str) -> bool {
    let lower = statement.to_ascii_lowercase();
    lower.contains("first published")
        || lower.contains("earliest published")
        || lower.contains("publication priority")
        || lower.contains("historical priority")
}

struct ParsedFields {
    values: BTreeMap<String, String>,
    duplicates: Vec<(String, usize)>,
    offsets: BTreeMap<String, usize>,
}

fn parse_fields(body: &str, body_offset: usize) -> ParsedFields {
    let mut fields = BTreeMap::<String, String>::new();
    let mut duplicates = Vec::new();
    let mut offsets = BTreeMap::new();
    let mut item = None::<(usize, String)>;
    let mut item_depth = 0usize;
    let mut code_depth = 0usize;
    let mut link_destinations = Vec::new();
    for (event, range) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Item) => {
                if item_depth == 0 {
                    item = Some((body_offset + range.start, String::new()));
                }
                item_depth += 1;
            }
            Event::End(TagEnd::Item) => {
                item_depth = item_depth.saturating_sub(1);
                if item_depth == 0 {
                    if let Some((offset, text)) = item.take() {
                        if let Some((field, value)) = text.split_once(':') {
                            let field = field.trim().to_owned();
                            if fields
                                .insert(field.clone(), value.trim().to_owned())
                                .is_some()
                            {
                                duplicates.push((field.clone(), offset));
                            }
                            offsets.entry(field).or_insert(offset);
                        }
                    }
                }
            }
            Event::Start(Tag::CodeBlock(_)) => code_depth += 1,
            Event::End(TagEnd::CodeBlock) => code_depth = code_depth.saturating_sub(1),
            Event::Start(Tag::Link { dest_url, .. }) if item_depth > 0 && code_depth == 0 => {
                if let Some((_, value)) = &mut item {
                    value.push('[');
                }
                link_destinations.push(dest_url.into_string());
            }
            Event::End(TagEnd::Link) if item_depth > 0 && code_depth == 0 => {
                if let (Some((_, value)), Some(destination)) = (&mut item, link_destinations.pop())
                {
                    value.push_str("](");
                    value.push_str(&destination);
                    value.push(')');
                }
            }
            Event::Text(text) | Event::InlineMath(text) if item_depth > 0 && code_depth == 0 => {
                if let Some((_, value)) = &mut item {
                    value.push_str(&text);
                }
            }
            Event::Code(text) if item_depth > 0 && code_depth == 0 => {
                if let Some((_, value)) = &mut item {
                    value.push('`');
                    value.push_str(&text);
                    value.push('`');
                }
            }
            Event::SoftBreak | Event::HardBreak if item_depth > 0 && code_depth == 0 => {
                if let Some((_, value)) = &mut item {
                    value.push(' ');
                }
            }
            _ => {}
        }
    }
    ParsedFields {
        values: fields,
        duplicates,
        offsets,
    }
}

struct Section<'a> {
    title: String,
    anchor: Option<String>,
    body: &'a str,
    heading_offset: usize,
    body_offset: usize,
}

fn level_two_sections(body: &str) -> Vec<Section<'_>> {
    let parsed_headings = Parser::new_ext(body, markdown_options())
        .into_offset_iter()
        .filter_map(|(event, range)| {
            matches!(
                event,
                Event::Start(Tag::Heading {
                    level: HeadingLevel::H2,
                    ..
                })
            )
            .then_some(range.start)
        })
        .collect::<BTreeSet<_>>();
    let mut headings = Vec::<(String, Option<String>, usize, usize)>::new();
    let mut offset = 0usize;
    for line in body.split_inclusive('\n') {
        if parsed_headings.contains(&offset) {
            let Some(heading) = line.strip_prefix("## ") else {
                offset += line.len();
                continue;
            };
            let heading = heading.trim_end();
            let (title, anchor) = parse_heading(heading);
            headings.push((title, anchor, offset, offset + line.len()));
        }
        offset += line.len();
    }
    headings
        .iter()
        .enumerate()
        .map(|(index, (title, anchor, _, content_start))| {
            let content_end = headings
                .get(index + 1)
                .map_or(body.len(), |(_, _, heading_start, _)| *heading_start);
            Section {
                title: title.clone(),
                anchor: anchor.clone(),
                body: &body[*content_start..content_end],
                heading_offset: headings[index].2,
                body_offset: *content_start,
            }
        })
        .collect()
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH
}

fn parse_heading(heading: &str) -> (String, Option<String>) {
    if let Some(prefix) = heading.strip_suffix('}') {
        if let Some((title, anchor)) = prefix.rsplit_once(" {#") {
            return (title.trim().to_owned(), Some(anchor.trim().to_owned()));
        }
    }
    (heading.trim().to_owned(), None)
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

fn diagnostic_at(
    document: &Document,
    body_offset: usize,
    code: &'static str,
    identity: Option<String>,
    field: impl Into<String>,
    expected: impl Into<String>,
    observed: impl Into<String>,
) -> TextbookDiagnostic {
    let bounded_offset = body_offset.min(document.body.len());
    let prefix = &document.body[..bounded_offset];
    let line =
        document.body_start_line + prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.len(), |(_, suffix)| suffix.len()) as u32
        + 1;
    let mut diagnostic = TextbookDiagnostic::new(
        code,
        identity,
        field,
        expected,
        observed,
        document.path.clone(),
    );
    diagnostic.line = Some(line);
    diagnostic.column = Some(column);
    diagnostic
}
