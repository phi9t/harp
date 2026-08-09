use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

const NUMBERED_SOURCE_IDS: [&str; 39] = [
    "GOOD-1965",
    "YUDKOWSKY-2008",
    "ASP",
    "ABSOLUTE-ZERO",
    "SELF-REWARDING",
    "SPIN",
    "ACE",
    "MCE",
    "META-HARNESS",
    "AI-SCIENTIST",
    "SCIENTISTONE",
    "AUTODATA",
    "ADAS",
    "SELF-REFINE",
    "AFLOW",
    "STOP",
    "SELF-HARNESS",
    "PROMPTBREEDER",
    "GEPA",
    "ALPHAEVOLVE",
    "SHINKAEVOLVE",
    "THETAEVOLVE",
    "DGM",
    "HYPERAGENTS",
    "LEARNING-DISCOVER",
    "EPISTEMIC-DISCOVERY",
    "SIA",
    "NOT-SCIENTISTS",
    "GPT5-SCIENCE",
    "PAPERBENCH",
    "REBENCH",
    "MLEBENCH",
    "SCIENCEAGENTBENCH",
    "COREBENCH",
    "KERNELBENCH",
    "HARNESS-DISENTANGLE",
    "AHE",
    "CONTINUAL-HARNESS",
    "DEMOEVOLVE",
];

const BODY_LINK_SOURCE_IDS: [&str; 3] = ["KARPATHY-AUTORESEARCH", "WENG-REWARD", "ANTHROPIC-RSI"];

const REQUIRED_HEADINGS: [&str; 5] = [
    "## Problem",
    "## Core mechanism",
    "## Reported evidence",
    "## Key limitation",
    "## Why Weng cites it",
];

const LESSONS: [&str; 10] = [
    "lessons/0001-system-being-improved.html",
    "lessons/0002-harness-design-patterns.html",
    "lessons/0003-harness-vs-core-intelligence.html",
    "lessons/0004-context-engineering.html",
    "lessons/0005-workflow-design-and-auto-research.html",
    "lessons/0006-self-improving-harnesses.html",
    "lessons/0007-evolutionary-search.html",
    "lessons/0008-joint-harness-weight-optimization.html",
    "lessons/0009-future-challenges-and-evaluation.html",
    "lessons/0010-synthesis-and-oral-defense.html",
];

const REFERENCES: [&str; 4] = [
    "reference/weng-harness-map.html",
    "reference/weng-source-cards.html",
    "reference/rsi-claim-ladder.html",
    "reference/harness-comparison-matrix.html",
];

const MATRIX_HEADER: [&str; 11] = [
    "source_id",
    "weng_locator",
    "title",
    "section_id",
    "card_path",
    "primary_url",
    "captured_path",
    "evidence_state",
    "claim_ceiling",
    "lesson_ids",
    "retrieval_state",
];

const CARD_METADATA: [&str; 13] = [
    "source_id",
    "title",
    "weng_locator",
    "section_id",
    "primary_url",
    "captured_path",
    "publication_state",
    "evidence_state",
    "edited_object_family",
    "claim_ceiling",
    "lesson_ids",
    "card_path",
    "canonical_route",
];

const SECTION_IDS: [&str; 9] = [
    "system-being-improved",
    "harness-design-patterns",
    "harness-layer-vs-core-intelligence",
    "context-engineering",
    "workflow-design-and-search",
    "self-improving-harnesses",
    "evolutionary-search",
    "joint-harness-weight-optimization",
    "future-challenges",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
        .to_path_buf()
}

fn row_maps(relative: &str) -> Vec<BTreeMap<String, String>> {
    let text = fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    let mut lines = text.lines();
    let header_line = lines
        .next()
        .unwrap_or_else(|| panic!("missing TSV header in {relative}"));
    let header = header_line
        .split('\t')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut header_names = BTreeSet::new();
    for (column, name) in header.iter().enumerate() {
        assert!(
            !name.is_empty(),
            "empty header in {relative} at column {}",
            column + 1
        );
        assert!(
            header_names.insert(name),
            "duplicate header {name:?} in {relative}"
        );
    }
    lines
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(index, line)| {
            let row_number = index + 2;
            let values = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
            assert_eq!(
                values.len(),
                header.len(),
                "malformed row {row_number} in {relative}"
            );
            header.iter().cloned().zip(values).collect()
        })
        .collect()
}

fn course_state() -> String {
    let path = workspace_root().join("content/weng-course-status.json");
    let value: Value = serde_json::from_slice(&fs::read(path).expect("read course status"))
        .expect("valid course status JSON");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["expected_source_cards"], 42);
    assert_eq!(value["expected_lessons"], 10);
    assert_eq!(value["expected_references"], 4);
    value["state"]
        .as_str()
        .expect("course state string")
        .to_owned()
}

fn source_registry() -> BTreeMap<String, String> {
    row_maps("content/sources/source_registry.tsv")
        .into_iter()
        .map(|row| (row["source_id"].clone(), row["label"].clone()))
        .collect()
}

fn matrix_rows() -> Vec<BTreeMap<String, String>> {
    let relative = "content/weng-source-cards.tsv";
    let text = fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    let actual_header = text
        .lines()
        .next()
        .expect("Weng source-card matrix header")
        .split('\t')
        .collect::<Vec<_>>();
    assert_eq!(
        actual_header, MATRIX_HEADER,
        "unexpected Weng source-card matrix header"
    );
    row_maps(relative)
}

fn reference_locator(evidence_locator: &str) -> String {
    let suffix = evidence_locator
        .strip_prefix("Reference ")
        .unwrap_or_else(|| panic!("unknown Weng reference locator: {evidence_locator}"));
    let digit_count = suffix.bytes().take_while(u8::is_ascii_digit).count();
    assert!(
        digit_count > 0,
        "unknown Weng reference locator: {evidence_locator}"
    );
    let remainder = &suffix[digit_count..];
    assert!(
        remainder.is_empty() || remainder.starts_with(char::is_whitespace),
        "unknown Weng reference locator: {evidence_locator}"
    );
    let number = suffix[..digit_count]
        .parse::<u8>()
        .expect("Weng reference number");
    assert!(
        (1..=39).contains(&number),
        "Weng reference number is out of range: {number}"
    );
    format!("reference-{number}")
}

fn body_link_locator(source_id: &str) -> &'static str {
    match source_id {
        "KARPATHY-AUTORESEARCH" => "body-link-workflow-automation",
        "WENG-REWARD" => "body-link-reward-hacking",
        "ANTHROPIC-RSI" => "body-link-ai-progress",
        _ => panic!("unknown Weng body-link source: {source_id}"),
    }
}

fn weng_source_locators() -> BTreeMap<String, String> {
    let rows = row_maps("content/sources/evidence_graph.tsv");
    let mut expected = BTreeMap::new();
    let mut locator_sources = BTreeMap::new();
    for row in rows {
        if row["source_id"] != "WENG-HARNESS" {
            continue;
        }
        let locator = match row["relationship"].as_str() {
            "cites" => reference_locator(&row["evidence_locator"]),
            "body-links" => body_link_locator(&row["target_id"]).to_owned(),
            relationship => panic!("unknown Weng evidence relationship: {relationship}"),
        };
        let source_id = row["target_id"].clone();
        assert!(
            expected
                .insert(source_id.clone(), locator.clone())
                .is_none(),
            "duplicate Weng source edge: {source_id}"
        );
        assert!(
            locator_sources
                .insert(locator.clone(), source_id.clone())
                .is_none(),
            "duplicate Weng locator: {locator}"
        );
    }
    assert_eq!(
        expected.len(),
        42,
        "Weng evidence graph must define 42 source locators"
    );
    expected
}

fn assert_matrix_locators(rows: &[BTreeMap<String, String>], expected: &BTreeMap<String, String>) {
    let mut present_sources = BTreeSet::new();
    let mut present_locators = BTreeSet::new();
    for row in rows {
        let source_id = &row["source_id"];
        let locator = &row["weng_locator"];
        assert!(
            present_sources.insert(source_id.clone()),
            "duplicate Weng source-card row {source_id}"
        );
        assert!(
            present_locators.insert(locator.clone()),
            "duplicate Weng source-card locator {locator}"
        );
        assert_eq!(
            expected
                .get(source_id)
                .unwrap_or_else(|| panic!("unexpected Weng source-card row {source_id}")),
            locator,
            "{source_id} has the wrong Weng locator"
        );
    }
    if rows.len() == expected.len() {
        assert_eq!(
            present_sources,
            expected.keys().cloned().collect(),
            "complete Weng matrix does not match evidence graph roster"
        );
    }
}

fn assert_all_sources_promoted(
    rows: &[BTreeMap<String, String>],
    registry: &BTreeMap<String, String>,
) {
    for row in rows {
        let source_id = &row["source_id"];
        let label = registry
            .get(source_id)
            .unwrap_or_else(|| panic!("{source_id} is absent from the source registry"));
        assert_ne!(label, "MISSING", "{source_id} remains identity-only");
    }
}

fn assert_files_exist(paths: &[&str]) {
    for relative in paths {
        strict_workspace_file(relative, "generated teaching file");
    }
}

fn card_frontmatter(text: &str) -> BTreeMap<String, String> {
    let rest = text
        .strip_prefix("---\n")
        .expect("source card must start with frontmatter");
    let frontmatter = rest
        .split_once("\n---\n")
        .expect("source card must close frontmatter")
        .0;
    let mut values = BTreeMap::new();
    let mut ordered_keys = Vec::new();
    for line in frontmatter.lines() {
        assert!(
            !line.is_empty() && !line.starts_with([' ', '\t']),
            "source-card frontmatter must use one-line top-level scalars"
        );
        let (key, value) = line
            .split_once(':')
            .expect("source-card frontmatter key/value");
        let key = key.trim();
        let value = value.trim();
        assert!(
            !key.is_empty() && !value.is_empty(),
            "source-card frontmatter key/value must be nonempty"
        );
        assert!(
            values.insert(key.to_owned(), value.to_owned()).is_none(),
            "duplicate source-card frontmatter key {key}"
        );
        ordered_keys.push(key.to_owned());
    }
    assert_eq!(
        ordered_keys, CARD_METADATA,
        "unexpected source-card frontmatter schema"
    );
    values
}

fn is_ordered_list_item(line: &str) -> bool {
    let digit_count = line.bytes().take_while(u8::is_ascii_digit).count();
    digit_count > 0 && line[digit_count..].starts_with(". ")
}

fn assert_plain_card_section(heading: &str, text: &str) {
    assert!(!text.is_empty(), "empty source-card section: {heading}");
    assert!(
        !text.contains('`') && !text.contains("](") && !text.contains("!["),
        "{heading} must contain plain paragraphs"
    );
    for line in text.lines() {
        let trimmed = line.trim_start();
        assert!(
            !["- ", "* ", "+ ", "> "]
                .iter()
                .any(|prefix| trimmed.starts_with(prefix))
                && !is_ordered_list_item(trimmed),
            "{heading} must not contain Markdown list or quote syntax"
        );
        let contains_html_tag = trimmed.match_indices('<').any(|(index, _)| {
            let suffix = &trimmed[index + 1..];
            if !suffix.contains('>') {
                return false;
            }
            if suffix.starts_with(['!', '?']) {
                return true;
            }
            suffix
                .strip_prefix('/')
                .unwrap_or(suffix)
                .chars()
                .next()
                .is_some_and(char::is_alphabetic)
        });
        assert!(!contains_html_tag, "{heading} must not contain HTML tags");
    }
}

fn card_sections(text: &str, expected_title: &str) -> BTreeMap<String, String> {
    let body = if let Some(rest) = text.strip_prefix("---\n") {
        rest.split_once("\n---\n")
            .expect("source card must close frontmatter")
            .1
    } else {
        text
    };
    assert!(
        body.starts_with('\n') && !body.starts_with("\n\n"),
        "source-card frontmatter must be followed by exactly one blank separator"
    );
    let lines = body[1..].lines().collect::<Vec<_>>();
    let title_line = lines
        .first()
        .copied()
        .unwrap_or_else(|| panic!("source card must have an H1 for {expected_title}"));
    assert!(
        title_line
            .strip_prefix("# ")
            .is_some_and(|title| !title.trim().is_empty()),
        "source card must have a nonempty H1 for {expected_title}"
    );
    let mut cursor = 1;
    assert!(
        lines.get(cursor).is_some_and(|line| line.trim().is_empty()),
        "source-card H1 must be followed by a blank line"
    );
    while lines.get(cursor).is_some_and(|line| line.trim().is_empty()) {
        cursor += 1;
    }
    assert_eq!(
        lines.get(cursor).copied(),
        Some("## Problem"),
        "source-card H1 must be followed by ## Problem"
    );

    let mut sections = BTreeMap::new();
    let mut ordered_headings = Vec::new();
    let mut current: Option<String> = None;
    let mut section_lines = Vec::new();

    let flush = |current: &mut Option<String>,
                 lines: &mut Vec<&str>,
                 sections: &mut BTreeMap<String, String>,
                 ordered_headings: &mut Vec<String>| {
        if let Some(heading) = current.take() {
            let section = lines.join("\n").trim().to_owned();
            assert_plain_card_section(&heading, &section);
            assert!(
                sections.insert(heading.clone(), section).is_none(),
                "duplicate source-card section: {heading}"
            );
            ordered_headings.push(format!("## {heading}"));
            lines.clear();
        }
    };

    for line in &lines[cursor..] {
        assert!(
            !line.trim_start().starts_with("# "),
            "source-card body must contain exactly one H1 title"
        );
        if let Some(heading) = line.strip_prefix("## ") {
            flush(
                &mut current,
                &mut section_lines,
                &mut sections,
                &mut ordered_headings,
            );
            current = Some(heading.trim().to_owned());
        } else if current.is_some() {
            section_lines.push(line);
        } else {
            assert!(
                line.trim().is_empty(),
                "source-card body must start with the first required heading"
            );
        }
    }
    flush(
        &mut current,
        &mut section_lines,
        &mut sections,
        &mut ordered_headings,
    );
    assert_eq!(
        ordered_headings, REQUIRED_HEADINGS,
        "unexpected source-card section schema"
    );
    sections
}

fn assert_resolved_file_beneath(relative: &str, base: &str, field: &str) -> PathBuf {
    let relative_path = Path::new(relative);
    assert!(
        relative_path.is_relative()
            && relative_path
                .components()
                .all(|component| matches!(component, Component::CurDir | Component::Normal(_))),
        "{field} must be repository-relative: {relative}"
    );
    let root = workspace_root();
    let base_path = Path::new(base);
    assert!(
        relative_path.starts_with(base_path),
        "{field} must stay beneath {base}: {relative}"
    );
    let path = path_without_symlinks(&root, relative_path, field);
    let resolved = canonical_workspace_file(&path, field);
    let strict_base = path_without_symlinks(&root, base_path, field);
    let resolved_base =
        fs::canonicalize(strict_base).unwrap_or_else(|error| panic!("resolve {base}: {error}"));
    assert!(
        resolved.starts_with(&resolved_base),
        "{field} must stay beneath {base}: {relative}"
    );
    resolved
}

fn lesson_id_set(raw: &str, source_id: &str) -> BTreeSet<String> {
    let parts = raw.split(',').collect::<Vec<_>>();
    assert!(!parts.is_empty(), "{source_id} has no lesson IDs");
    let mut lesson_ids = BTreeSet::new();
    for lesson_id in parts {
        assert!(
            lesson_id.len() == 4 && lesson_id.bytes().all(|byte| byte.is_ascii_digit()),
            "{source_id} has malformed lesson ID {lesson_id:?}"
        );
        let lesson_number = lesson_id
            .parse::<u8>()
            .expect("four-digit lesson ID must parse");
        assert!(
            (1..=10).contains(&lesson_number),
            "{source_id} has out-of-range lesson ID {lesson_id}"
        );
        assert!(
            lesson_ids.insert(lesson_id.to_owned()),
            "{source_id} repeats lesson ID {lesson_id}"
        );
    }
    assert!(
        lesson_ids.contains("0010"),
        "{source_id} must be assigned to lesson 0010"
    );
    lesson_ids
}

#[derive(Debug)]
struct HtmlTag {
    name: String,
    attribute_names: BTreeSet<String>,
    attributes: BTreeMap<String, String>,
    start: usize,
    end: usize,
}

impl HtmlTag {
    fn has_attribute(&self, name: &str) -> bool {
        self.attribute_names.contains(name)
    }

    fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }
}

#[derive(Debug)]
struct HtmlCloseTag {
    name: String,
    start: usize,
}

#[derive(Debug)]
struct HtmlDocument {
    opening_tags: Vec<HtmlTag>,
    closing_tags: Vec<HtmlCloseTag>,
    visible_text: String,
}

impl HtmlDocument {
    fn tags_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a HtmlTag> {
        self.opening_tags.iter().filter(move |tag| tag.name == name)
    }

    fn attribute_values<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> {
        self.opening_tags
            .iter()
            .filter_map(move |tag| tag.attribute(name))
    }
}

fn tag_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, byte) in bytes[start..].iter().copied().enumerate() {
        match quote {
            Some(delimiter) if byte == delimiter => quote = None,
            Some(_) => {}
            None if matches!(byte, b'\'' | b'"') => quote = Some(byte),
            None if byte == b'>' => return Some(start + offset),
            None => {}
        }
    }
    None
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':')
}

fn tag_name(tag: &str, closing: bool) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut cursor = if closing { 2 } else { 1 };
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    let start = cursor;
    while cursor < bytes.len() && is_name_byte(bytes[cursor]) {
        cursor += 1;
    }
    (start != cursor).then(|| tag[start..cursor].to_ascii_lowercase())
}

fn parse_opening_tag(tag: &str, start: usize, end: usize) -> Option<HtmlTag> {
    let name = tag_name(tag, false)?;
    let bytes = tag.as_bytes();
    let mut cursor = 1 + name.len();
    let mut attribute_names = BTreeSet::new();
    let mut attributes = BTreeMap::new();

    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b'/')
        {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < bytes.len() && is_name_byte(bytes[cursor]) {
            cursor += 1;
        }
        if name_start == cursor {
            cursor += 1;
            continue;
        }
        let attribute_name = tag[name_start..cursor].to_ascii_lowercase();
        assert!(
            attribute_names.insert(attribute_name.clone()),
            "duplicate HTML attribute {attribute_name:?} on <{name}>"
        );
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }

        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let value = if cursor < bytes.len() && matches!(bytes[cursor], b'\'' | b'"') {
            let delimiter = bytes[cursor];
            cursor += 1;
            let value_start = cursor;
            while cursor < bytes.len() && bytes[cursor] != delimiter {
                cursor += 1;
            }
            let value = &tag[value_start..cursor];
            if cursor < bytes.len() {
                cursor += 1;
            }
            value
        } else {
            let value_start = cursor;
            while cursor < bytes.len()
                && !bytes[cursor].is_ascii_whitespace()
                && !matches!(bytes[cursor], b'>' | b'/')
            {
                cursor += 1;
            }
            &tag[value_start..cursor]
        };
        if !value.trim().is_empty() {
            attributes.insert(attribute_name, value.to_owned());
        }
    }

    Some(HtmlTag {
        name,
        attribute_names,
        attributes,
        start,
        end,
    })
}

fn find_raw_element_close(lower: &str, from: usize, name: &str) -> Option<(usize, usize)> {
    let needle = format!("</{name}");
    let mut cursor = from;
    while let Some(offset) = lower[cursor..].find(&needle) {
        let start = cursor + offset;
        let boundary = lower.as_bytes().get(start + needle.len()).copied();
        if boundary.is_some_and(|byte| byte.is_ascii_whitespace() || byte == b'>') {
            let end = tag_end(lower.as_bytes(), start + needle.len())?;
            return Some((start, end + 1));
        }
        cursor = start + needle.len();
    }
    None
}

fn parse_html(html: &str) -> HtmlDocument {
    let lower = html.to_ascii_lowercase();
    let bytes = html.as_bytes();
    let mut opening_tags = Vec::new();
    let mut closing_tags = Vec::new();
    let mut visible_text = String::new();
    let mut cursor = 0;

    while let Some(offset) = bytes[cursor..].iter().position(|byte| *byte == b'<') {
        let start = cursor + offset;
        visible_text.push_str(&html[cursor..start]);

        if lower[start..].starts_with("<!--") {
            cursor = lower[start + 4..]
                .find("-->")
                .map_or(html.len(), |offset| start + 4 + offset + 3);
            continue;
        }

        let Some(end) = tag_end(bytes, start + 1) else {
            visible_text.push_str(&html[start..]);
            cursor = html.len();
            break;
        };
        let raw_tag = &html[start..=end];
        if raw_tag.starts_with("<!") || raw_tag.starts_with("<?") {
            cursor = end + 1;
            continue;
        }
        if raw_tag.starts_with("</") {
            if let Some(name) = tag_name(raw_tag, true) {
                closing_tags.push(HtmlCloseTag { name, start });
            }
            cursor = end + 1;
            continue;
        }

        let Some(tag) = parse_opening_tag(raw_tag, start, end + 1) else {
            visible_text.push('<');
            cursor = start + 1;
            continue;
        };
        let raw_element = matches!(tag.name.as_str(), "script" | "style");
        let raw_name = tag.name.clone();
        opening_tags.push(tag);
        if raw_element {
            if let Some((close_start, close_end)) =
                find_raw_element_close(&lower, end + 1, &raw_name)
            {
                closing_tags.push(HtmlCloseTag {
                    name: raw_name,
                    start: close_start,
                });
                cursor = close_end;
            } else {
                cursor = html.len();
            }
        } else {
            cursor = end + 1;
        }
    }
    visible_text.push_str(&html[cursor..]);

    HtmlDocument {
        opening_tags,
        closing_tags,
        visible_text,
    }
}

fn is_external_target(target: &str) -> bool {
    let lower = target.trim().to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
}

fn is_forbidden_target(target: &str) -> bool {
    let lower = target.trim().to_ascii_lowercase();
    lower.starts_with("//")
        || lower.starts_with("javascript:")
        || lower.starts_with("data:")
        || lower.starts_with("file:")
}

fn is_windows_absolute_path(target: &str) -> bool {
    let bytes = target.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/')
}

fn local_target_path(target: &str) -> &str {
    let end = ['#', '?']
        .into_iter()
        .filter_map(|delimiter| target.find(delimiter))
        .min()
        .unwrap_or(target.len());
    &target[..end]
}

fn target_fragment(target: &str) -> Option<&str> {
    target
        .split_once('#')
        .map(|(_, fragment)| fragment.split('?').next().unwrap_or(fragment))
}

fn forbidden_absolute_path(text: &str) -> Option<String> {
    if text.to_ascii_lowercase().contains("file://") {
        return Some("file://".to_owned());
    }
    let bytes = text.as_bytes();
    let windows_absolute = bytes.windows(3).enumerate().any(|(index, window)| {
        let boundary = index == 0
            || bytes[index - 1].is_ascii_whitespace()
            || matches!(
                bytes[index - 1],
                b'`' | b'\'' | b'"' | b'(' | b'[' | b'{' | b',' | b';'
            );
        let uri_scheme = window[2] == b'/' && bytes.get(index + 3) == Some(&b'/');
        boundary
            && window[0].is_ascii_alphabetic()
            && window[1] == b':'
            && matches!(window[2], b'\\' | b'/')
            && !uri_scheme
    });
    if windows_absolute {
        return Some("Windows absolute path".to_owned());
    }

    text.split(|character: char| {
        character.is_whitespace()
            || matches!(
                character,
                '`' | '\'' | '"' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
            )
    })
    .map(|token| token.trim_end_matches(|character: char| ".:!?".contains(character)))
    .find(|token| token != &"/" && token.starts_with('/') && !token.starts_with("//"))
    .map(str::to_owned)
}

fn assert_no_absolute_local_paths(path: &Path, document: &HtmlDocument, targets: &[&str]) {
    if let Some(forbidden) = forbidden_absolute_path(&document.visible_text) {
        panic!(
            "{} contains absolute local path {forbidden}",
            path.display()
        );
    }
    for target in targets {
        let target = target.trim();
        assert!(
            !is_forbidden_target(target),
            "{} contains forbidden target {target}",
            path.display()
        );
        if target.is_empty() || is_external_target(target) {
            continue;
        }
        let local = local_target_path(target);
        assert!(
            !Path::new(local).is_absolute() && !is_windows_absolute_path(local),
            "{} contains absolute local target {target}",
            path.display()
        );
    }
}

fn canonical_workspace_file(path: &Path, field: &str) -> PathBuf {
    let metadata =
        fs::metadata(path).unwrap_or_else(|error| panic!("{field} does not resolve: {error}"));
    assert!(metadata.is_file(), "{field} must resolve to a regular file");
    let resolved =
        fs::canonicalize(path).unwrap_or_else(|error| panic!("resolve {field}: {error}"));
    let root = fs::canonicalize(workspace_root()).expect("resolve Harp workspace root");
    assert!(
        resolved.starts_with(&root),
        "{field} must stay beneath the repository root"
    );
    resolved
}

fn path_without_symlinks(start: &Path, relative: &Path, field: &str) -> PathBuf {
    let root = fs::canonicalize(workspace_root()).expect("resolve Harp workspace root");
    let mut current =
        fs::canonicalize(start).unwrap_or_else(|error| panic!("resolve {field} base: {error}"));
    assert!(
        current.starts_with(&root),
        "{field} base must stay beneath the repository root"
    );
    for component in relative.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                assert!(
                    current.pop() && current.starts_with(&root),
                    "{field} escapes the repository root"
                );
            }
            Component::Normal(name) => {
                current.push(name);
                let metadata = fs::symlink_metadata(&current)
                    .unwrap_or_else(|error| panic!("{field} does not resolve: {error}"));
                assert!(
                    !metadata.file_type().is_symlink(),
                    "{field} must not traverse a symlink: {}",
                    current.display()
                );
            }
            Component::RootDir | Component::Prefix(_) => {
                panic!("{field} must be repository-relative")
            }
        }
    }
    current
}

fn strict_workspace_file(relative: &str, field: &str) -> PathBuf {
    let relative_path = Path::new(relative);
    assert!(
        relative_path.is_relative(),
        "{field} must be repository-relative: {relative}"
    );
    let path = path_without_symlinks(&workspace_root(), relative_path, field);
    canonical_workspace_file(&path, field)
}

fn resolve_local_target(
    document_path: &Path,
    target: &str,
    field: &str,
    reject_symlinks: bool,
) -> Option<PathBuf> {
    let target = target.trim();
    assert!(
        !is_forbidden_target(target),
        "{field} uses a forbidden target: {target}"
    );
    if target.is_empty() || target.starts_with('#') || is_external_target(target) {
        return None;
    }
    let local = local_target_path(target);
    if local.is_empty() {
        return None;
    }
    let relative = Path::new(local);
    assert!(
        relative.is_relative() && !is_windows_absolute_path(local),
        "{field} must be repository-relative: {target}"
    );
    let parent = document_path.parent().expect("HTML parent");
    let path = if reject_symlinks {
        path_without_symlinks(parent, relative, field)
    } else {
        parent.join(relative)
    };
    Some(canonical_workspace_file(&path, field))
}

fn assert_local_targets_resolve(path: &Path, targets: &[&str]) {
    for target in targets {
        resolve_local_target(path, target, "local HTML target", true);
    }
}

fn assert_stylesheet(path: &Path, document: &HtmlDocument) {
    let stylesheet = document.tags_named("link").find(|tag| {
        tag.attribute("rel").is_some_and(|rel| {
            rel.split_ascii_whitespace()
                .any(|token| token.eq_ignore_ascii_case("stylesheet"))
        }) && tag.attribute("href") == Some("../assets/course.css")
    });
    assert!(
        stylesheet.is_some(),
        "{} must use <link rel=\"stylesheet\" href=\"../assets/course.css\">",
        path.display()
    );
    let resolved = resolve_local_target(
        path,
        "../assets/course.css",
        "shared course stylesheet",
        true,
    )
    .expect("course stylesheet is local");
    assert_eq!(
        resolved,
        strict_workspace_file("assets/course.css", "shared course stylesheet"),
        "{} stylesheet must resolve to assets/course.css",
        path.display()
    );
}

fn assert_scripts_have_sources(path: &Path, document: &HtmlDocument) {
    for script in document.tags_named("script") {
        let source = script.attribute("src").unwrap_or_else(|| {
            panic!(
                "{} contains a script without a nonempty src",
                path.display()
            )
        });
        let source = source.trim();
        assert!(
            !is_external_target(source) && !is_forbidden_target(source),
            "{} contains a non-local script source {source}",
            path.display()
        );
        resolve_local_target(path, source, "shared script", true);
    }
}

fn assert_no_inline_reusable_assets(path: &Path, document: &HtmlDocument) {
    assert!(
        document.tags_named("style").next().is_none(),
        "{} contains inline style",
        path.display()
    );
    assert_scripts_have_sources(path, document);
}

fn quiz_form_is_labeled(document: &HtmlDocument) -> bool {
    let forms = document.tags_named("form").collect::<Vec<_>>();
    let marked_forms = forms
        .iter()
        .copied()
        .filter(|form| form.has_attribute("data-quiz"))
        .collect::<Vec<_>>();
    let has_quiz_marker = document
        .opening_tags
        .iter()
        .any(|tag| tag.has_attribute("data-quiz"));
    let candidates = if has_quiz_marker { marked_forms } else { forms };

    candidates.into_iter().any(|form| {
        if form.attribute("aria-label").is_some() {
            return true;
        }
        let Some(close) = document
            .closing_tags
            .iter()
            .find(|close| close.name == "form" && close.start >= form.end)
        else {
            return false;
        };
        document
            .tags_named("legend")
            .any(|legend| legend.start >= form.end && legend.start < close.start)
    })
}

fn target_resolves_to(
    document_path: &Path,
    target: &str,
    expected: &Path,
    fragment: Option<&str>,
) -> bool {
    if target_fragment(target) != fragment {
        return false;
    }
    resolve_local_target(document_path, target, "required lesson target", true)
        .is_some_and(|resolved| resolved == expected)
}

fn validate_references() {
    for relative in REFERENCES {
        let path = strict_workspace_file(relative, "teaching reference");
        let html = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let document = parse_html(&html);
        let targets = document
            .attribute_values("href")
            .chain(document.attribute_values("src"))
            .collect::<Vec<_>>();
        assert_stylesheet(&path, &document);
        assert_scripts_have_sources(&path, &document);
        assert_no_absolute_local_paths(&path, &document, &targets);
        assert_local_targets_resolve(&path, &targets);
    }
}

fn validate_lessons(rows: &[BTreeMap<String, String>]) {
    let source_cards =
        strict_workspace_file("reference/weng-source-cards.html", "source-card reference");
    let coverage_script = strict_workspace_file("assets/coverage.js", "coverage script");

    for (index, relative) in LESSONS.iter().enumerate() {
        let lesson_number = index + 1;
        let lesson_id = format!("{lesson_number:04}");
        let path = strict_workspace_file(relative, "teaching lesson");
        let html = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        let document = parse_html(&html);
        let hrefs = document
            .tags_named("a")
            .filter_map(|tag| tag.attribute("href"))
            .collect::<Vec<_>>();
        let scripts = document
            .tags_named("script")
            .filter_map(|tag| tag.attribute("src"))
            .collect::<Vec<_>>();
        let targets = document
            .attribute_values("href")
            .chain(document.attribute_values("src"))
            .collect::<Vec<_>>();

        assert_stylesheet(&path, &document);
        assert!(
            document.tags_named("form").next().is_some()
                || document
                    .opening_tags
                    .iter()
                    .any(|tag| tag.has_attribute("data-quiz")),
            "{relative} must contain a retrieval check"
        );
        assert!(
            quiz_form_is_labeled(&document),
            "{relative} must label its quiz form"
        );
        assert!(
            hrefs
                .iter()
                .any(|target| target.starts_with("http://") || target.starts_with("https://")),
            "{relative} must link a primary source"
        );
        assert!(
            document.visible_text.contains("Ask your teacher"),
            "{relative} must invite teacher follow-up"
        );
        assert_no_inline_reusable_assets(&path, &document);
        assert_no_absolute_local_paths(&path, &document, &targets);
        assert_local_targets_resolve(&path, &targets);

        if lesson_number < 10 {
            for row in rows {
                let assigned = row["lesson_ids"].split(',').any(|id| id == lesson_id);
                if assigned {
                    let fragment = format!("source-{}", row["source_id"]);
                    assert!(
                        hrefs.iter().any(|target| target_resolves_to(
                            &path,
                            target,
                            &source_cards,
                            Some(&fragment)
                        )),
                        "{relative} must link {}",
                        row["source_id"]
                    );
                }
            }
        } else {
            assert!(
                hrefs
                    .iter()
                    .any(|target| target_resolves_to(&path, target, &source_cards, None)),
                "{relative} must link weng-source-cards.html"
            );
            assert!(
                scripts
                    .iter()
                    .any(|target| { target_resolves_to(&path, target, &coverage_script, None) }),
                "{relative} must link ../assets/coverage.js"
            );
        }
    }
}

#[test]
fn html_parser_preserves_tag_context_and_ignores_comments_and_raw_text() {
    let html = r#"
        <!--
          <link rel="stylesheet" href="../assets/comment.css">
          <style>.comment { display: none; }</style>
          <script src="../assets/comment.js"></script>
        -->
        <link rel="stylesheet" href="../assets/course.css">
        <p>&lt;style&gt; and &lt;script src="escaped.js"&gt;</p>
        <script src="../assets/quiz.js">
          const fake = '<a href="inside-script.html">not markup</a>';
        </script>
    "#;
    let document = parse_html(html);

    let links = document.tags_named("link").collect::<Vec<_>>();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].attribute("rel"), Some("stylesheet"));
    assert_eq!(links[0].attribute("href"), Some("../assets/course.css"));
    let scripts = document.tags_named("script").collect::<Vec<_>>();
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0].attribute("src"), Some("../assets/quiz.js"));
    assert!(document.tags_named("style").next().is_none());
    assert!(document.tags_named("a").next().is_none());
    assert!(document.visible_text.contains("&lt;style&gt;"));
}

#[test]
fn builder_publication_self_tests_pass() {
    let output = Command::new("python3")
        .args(["scripts/build_weng_course.py", "--self-test"])
        .current_dir(workspace_root())
        .output()
        .unwrap_or_else(|error| panic!("failed to launch python3 for builder self-tests: {error}"));
    assert!(
        output.status.success(),
        "builder self-tests failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn card_frontmatter_rejects_missing_unknown_and_reordered_metadata() {
    let valid = [
        "source_id: TEST",
        "title: Test",
        "weng_locator: reference-1",
        "section_id: system-being-improved",
        "primary_url: https://example.com",
        "captured_path: evidence/test.txt",
        "publication_state: preprint",
        "evidence_state: card-complete",
        "edited_object_family: harness",
        "claim_ceiling: Test claim",
        "lesson_ids: 0001,0010",
        "card_path: content/weng-sources/test.md",
        "canonical_route: content/test.md",
    ];
    let cases = [
        valid[..12].join("\n"),
        valid
            .iter()
            .chain(["unknown: value"].iter())
            .copied()
            .collect::<Vec<_>>()
            .join("\n"),
        {
            let mut reordered = valid;
            reordered.swap(0, 1);
            reordered.join("\n")
        },
    ];

    for frontmatter in cases {
        let card = format!("---\n{frontmatter}\n---\n");
        assert!(
            std::panic::catch_unwind(|| card_frontmatter(&card)).is_err(),
            "schema drift unexpectedly passed:\n{card}"
        );
    }
}

#[test]
fn card_sections_reject_empty_and_markdown_or_html_prose() {
    let headings = format!("\n# Test Card\n\n{}", REQUIRED_HEADINGS.join("\n"));
    assert!(
        std::panic::catch_unwind(|| card_sections(&headings, "Test Card")).is_err(),
        "empty source-card sections unexpectedly passed"
    );

    for hostile in [
        "Use `inline code`.",
        "```text\ncode\n```",
        "Read [the paper](https://example.test).",
        "![diagram](image.png)",
        "- list item",
        "* list item",
        "+ list item",
        "> quote",
        "1. ordered item",
        "<em>raw HTML</em>",
        "<!-- raw comment -->",
    ] {
        let body = format!(
            "\n# Test Card\n\n{}",
            REQUIRED_HEADINGS
                .iter()
                .map(|heading| format!("{heading}\n{hostile}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            std::panic::catch_unwind(|| card_sections(&body, "Test Card")).is_err(),
            "non-plain card prose unexpectedly passed: {hostile}"
        );
    }

    let punctuation = format!(
        "\n# Test Card\n\n{}",
        REQUIRED_HEADINGS
            .iter()
            .map(|heading| format!("{heading}\nx < y > z. See https://example.test."))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert_eq!(
        card_sections(&punctuation, "Test Card").len(),
        REQUIRED_HEADINGS.len()
    );
}

#[test]
fn card_sections_require_one_nonempty_h1_before_problem() {
    let sections = REQUIRED_HEADINGS
        .iter()
        .map(|heading| format!("{heading}\nPlain paragraph."))
        .collect::<Vec<_>>()
        .join("\n");
    let ace_title =
        "Agentic Context Engineering: Evolving Contexts for Self-Improving Language Models";
    let valid = format!("\n# Agentic Context Engineering\n\n{sections}");
    assert_eq!(
        card_sections(&valid, ace_title).len(),
        REQUIRED_HEADINGS.len()
    );

    for invalid in [
        format!("\n{sections}"),
        format!("\n# \n\n{sections}"),
        format!("\n# Agentic Context Engineering\n\n# Duplicate\n\n{sections}"),
        format!("\n# Agentic Context Engineering\n\nExtra prose.\n\n{sections}"),
        format!("\n# Agentic Context Engineering\n\n### Extra heading\n\n{sections}"),
        format!("\n# Agentic Context Engineering\n\n{sections}\n\n# Later title"),
    ] {
        assert!(
            std::panic::catch_unwind(|| card_sections(&invalid, ace_title)).is_err(),
            "invalid H1 card contract unexpectedly passed:\n{invalid}"
        );
    }

    for invalid_separator in [
        format!("# Agentic Context Engineering\n\n{sections}"),
        format!("\n\n# Agentic Context Engineering\n\n{sections}"),
    ] {
        assert!(
            std::panic::catch_unwind(|| card_sections(&invalid_separator, ace_title)).is_err(),
            "invalid frontmatter separator unexpectedly passed:\n{invalid_separator}"
        );
    }
}

#[test]
fn evidence_locators_and_matrix_rows_preserve_source_identity() {
    assert_eq!(
        reference_locator("Reference 38 and Joint Optimization with Model Weights"),
        "reference-38"
    );
    for invalid in ["Reference", "Reference X", "Reference 1suffix", "Section 1"] {
        assert!(
            std::panic::catch_unwind(|| reference_locator(invalid)).is_err(),
            "unknown evidence locator unexpectedly passed: {invalid}"
        );
    }

    let expected = BTreeMap::from([
        ("SOURCE-A".to_owned(), "reference-1".to_owned()),
        ("SOURCE-B".to_owned(), "reference-2".to_owned()),
    ]);
    let row = |source_id: &str, locator: &str| {
        BTreeMap::from([
            ("source_id".to_owned(), source_id.to_owned()),
            ("weng_locator".to_owned(), locator.to_owned()),
        ])
    };
    let swapped = [
        row("SOURCE-A", "reference-2"),
        row("SOURCE-B", "reference-1"),
    ];
    let duplicate = [
        row("SOURCE-A", "reference-1"),
        row("SOURCE-B", "reference-1"),
    ];
    assert!(
        std::panic::catch_unwind(|| assert_matrix_locators(&swapped, &expected)).is_err(),
        "swapped source locators unexpectedly passed"
    );
    assert!(
        std::panic::catch_unwind(|| assert_matrix_locators(&duplicate, &expected)).is_err(),
        "duplicate source locators unexpectedly passed"
    );
}

#[test]
fn html_targets_reject_unsafe_schemes_and_remote_scripts() {
    let document_path = workspace_root().join("reference/test.html");
    for target in [
        " javascript:alert(1)",
        "DATA:text/plain,hello",
        "file:///tmp/private",
        "//cdn.example.test/course.js",
    ] {
        assert!(
            std::panic::catch_unwind(|| {
                resolve_local_target(&document_path, target, "hostile target", true);
            })
            .is_err(),
            "unsafe target unexpectedly passed: {target}"
        );
    }

    for source in [
        "https://example.test/course.js",
        "HTTP://example.test/course.js",
        "data:text/javascript,alert(1)",
    ] {
        let document = parse_html(&format!(r#"<script src="{source}"></script>"#));
        assert!(
            std::panic::catch_unwind(|| assert_scripts_have_sources(&document_path, &document))
                .is_err(),
            "remote or unsafe script unexpectedly passed: {source}"
        );
    }
}

#[test]
fn quiz_label_must_belong_to_the_quiz_form() {
    let unrelated_label = parse_html(
        r#"
        <nav aria-label="Course navigation"></nav>
        <form data-quiz><fieldset></fieldset></form>
        <legend>Outside the form</legend>
        "#,
    );
    assert!(!quiz_form_is_labeled(&unrelated_label));

    let marker_outside_form = parse_html(
        r#"
        <section data-quiz></section>
        <form aria-label="Unrelated form"></form>
        "#,
    );
    assert!(!quiz_form_is_labeled(&marker_outside_form));

    let form_label = parse_html(r#"<form data-quiz aria-label="Retrieval check"></form>"#);
    assert!(quiz_form_is_labeled(&form_label));

    let contained_legend =
        parse_html(r#"<form data-quiz><fieldset><legend>Question</legend></fieldset></form>"#);
    assert!(quiz_form_is_labeled(&contained_legend));
}

#[test]
fn visible_urls_are_not_windows_paths() {
    assert_eq!(forbidden_absolute_path("https://example.com/path"), None);
    assert_eq!(forbidden_absolute_path("http://example.com"), None);
    assert_eq!(
        forbidden_absolute_path("Open C:/Users/example"),
        Some("Windows absolute path".to_owned())
    );
    assert_eq!(
        forbidden_absolute_path(r"Open D:\data\file"),
        Some("Windows absolute path".to_owned())
    );
}

#[cfg(unix)]
#[test]
fn local_target_validation_rejects_symlink_traversal() {
    use std::os::unix::fs::symlink;

    let sandbox = TempDir::new_in(workspace_root()).expect("temporary teaching surface");
    let reference = sandbox.path().join("reference");
    let content = sandbox.path().join("content");
    fs::create_dir_all(&reference).expect("temporary reference directory");
    fs::create_dir_all(&content).expect("temporary content directory");
    fs::write(reference.join("page.html"), "<!doctype html>").expect("temporary HTML");
    fs::write(content.join("target.md"), "# Target\n").expect("temporary target");
    symlink("../content/target.md", reference.join("alias.md")).expect("temporary symlink");

    let result = std::panic::catch_unwind(|| {
        assert_local_targets_resolve(&reference.join("page.html"), &["alias.md"]);
    });
    assert!(
        result.is_err(),
        "symlinked local target unexpectedly passed"
    );
}

#[test]
fn weng_teaching_curriculum_has_complete_observable_contract() {
    let expected_locators = weng_source_locators();
    let numbered = expected_locators
        .iter()
        .filter(|(_, locator)| locator.starts_with("reference-"))
        .map(|(source_id, _)| source_id.clone())
        .collect::<BTreeSet<_>>();
    let body_linked = expected_locators
        .iter()
        .filter(|(_, locator)| locator.starts_with("body-link-"))
        .map(|(source_id, _)| source_id.clone())
        .collect::<BTreeSet<_>>();
    let expected_numbered = NUMBERED_SOURCE_IDS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let expected_body_linked = BODY_LINK_SOURCE_IDS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        numbered, expected_numbered,
        "numbered Weng source roster drifted"
    );
    assert_eq!(
        body_linked, expected_body_linked,
        "body-linked Weng source roster drifted"
    );

    let mut expected_roster = expected_numbered.clone();
    expected_roster.extend(expected_body_linked);
    assert_eq!(expected_roster.len(), 42, "Weng roster must contain 42 IDs");

    let rows = matrix_rows();
    assert_matrix_locators(&rows, &expected_locators);
    let state = course_state();
    let registry = source_registry();
    let mut present = BTreeSet::new();

    for row in &rows {
        let source_id = &row["source_id"];
        assert_ne!(source_id, "RLM-PAPER", "RLM-PAPER is not in Weng's roster");
        assert!(
            present.insert(source_id.clone()),
            "duplicate Weng source-card row {source_id}"
        );
        assert!(
            expected_roster.contains(source_id),
            "unexpected Weng source-card row {source_id}"
        );
        assert!(
            SECTION_IDS.contains(&row["section_id"].as_str()),
            "{source_id} has unknown section {}",
            row["section_id"]
        );
        assert_eq!(
            row["evidence_state"], "card-complete",
            "{source_id} must be card-complete"
        );
        assert_eq!(
            row["retrieval_state"], "unseen",
            "{source_id} retrieval state must start unseen"
        );
        lesson_id_set(&row["lesson_ids"], source_id);

        if expected_numbered.contains(source_id) {
            let label = registry
                .get(source_id)
                .unwrap_or_else(|| panic!("{source_id} is absent from the source registry"));
            assert_ne!(label, "MISSING", "{source_id} remains identity-only");
        }

        let card_path =
            assert_resolved_file_beneath(&row["card_path"], "content/weng-sources", "card_path");
        let card_text = fs::read_to_string(&card_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", card_path.display()));
        let metadata = card_frontmatter(&card_text);
        card_sections(&card_text, &metadata["title"]);

        for field in [
            "source_id",
            "weng_locator",
            "title",
            "section_id",
            "card_path",
            "primary_url",
            "captured_path",
            "evidence_state",
            "claim_ceiling",
            "lesson_ids",
        ] {
            assert_eq!(
                metadata
                    .get(field)
                    .unwrap_or_else(|| panic!("{source_id} card is missing {field}")),
                &row[field],
                "{source_id} card and matrix disagree on {field}"
            );
        }
        assert!(
            !metadata["publication_state"].is_empty(),
            "{source_id} publication_state must be nonempty"
        );
        assert!(
            !metadata["edited_object_family"].is_empty(),
            "{source_id} edited_object_family must be nonempty"
        );

        let primary_url = &metadata["primary_url"];
        assert!(
            primary_url.starts_with("http://") || primary_url.starts_with("https://"),
            "{source_id} primary_url must use HTTP(S)"
        );
        let captured =
            assert_resolved_file_beneath(&metadata["captured_path"], "evidence", "captured_path");
        let row_captured =
            assert_resolved_file_beneath(&row["captured_path"], "evidence", "captured_path");
        assert_eq!(
            captured, row_captured,
            "{source_id} card and matrix captured paths must resolve identically"
        );

        let canonical_route = metadata
            .get("canonical_route")
            .unwrap_or_else(|| panic!("{source_id} card is missing canonical_route"));
        assert_ne!(
            canonical_route, &row["card_path"],
            "{source_id} canonical_route must differ from card_path"
        );
        let canonical = assert_resolved_file_beneath(canonical_route, "content", "canonical_route");
        assert_ne!(
            canonical, card_path,
            "{source_id} canonical_route must not resolve to card_path"
        );
    }

    match state.as_str() {
        "building" => assert!(rows.len() <= 42),
        "cards-complete" => {
            assert_eq!(rows.len(), 42);
            assert_all_sources_promoted(&rows, &registry);
            assert_files_exist(&REFERENCES);
        }
        "complete" => {
            assert_eq!(rows.len(), 42);
            assert_all_sources_promoted(&rows, &registry);
            assert_files_exist(&REFERENCES);
            assert_files_exist(&LESSONS);
        }
        _ => panic!("unknown Weng course state: {state}"),
    }

    if matches!(state.as_str(), "cards-complete" | "complete") {
        validate_references();
    }
    if state == "complete" {
        validate_lessons(&rows);
    }
}
