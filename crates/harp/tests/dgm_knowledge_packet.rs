use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{BrokenLink, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

const EXPECTED_FILES: [&str; 17] = [
    "01_orientation.md",
    "02_paper_walkthrough.md",
    "03_algorithm_derivation.md",
    "04_system_architecture.md",
    "05_repository_walkthrough.md",
    "06_evaluation_analysis.md",
    "07_open_endedness.md",
    "08_safety_and_failure.md",
    "09_critical_review.md",
    "10_successor_design.md",
    "11_hyperagents_successor.md",
    "claim_evidence_crosswalk.md",
    "darwin_godel_machine_index.md",
    "glossary.md",
    "learning_path.md",
    "maintenance.md",
    "source_registry.md",
];

const REQUIRED_FRONTMATTER: [&str; 9] = [
    "id",
    "title",
    "type",
    "status",
    "created",
    "updated",
    "tags",
    "confidence",
    "canonical",
];

const ALLOWED_TYPES: [&str; 8] = [
    "index",
    "concept",
    "deep-dive",
    "claim-ledger",
    "source-registry",
    "learning-path",
    "derivation",
    "runbook",
];

const AUTHORITY_NOTICE: &str = concat!(
    "> This file is a maintained Harp technical packet. It separates Harp's\n",
    "> synthesis from primary-source claims; primary-source captures and pinned\n",
    "> implementation files live under `evidence/`."
);

const DGM_LEDGER_IDS: [&str; 40] = [
    "DGM-001", "DGM-005", "DGM-009", "DGM-013", "DGM-020", "DGM-022", "DGM-026", "DGM-029A",
    "DGM-029B", "DGM-029C", "DGM-034", "DGM-041", "DGM-042", "DGM-044", "DGM-049", "DGM-050",
    "DGM-056", "DGM-057", "DGM-058", "DGM-059", "DGM-060", "DGM-061", "DGM-062", "DGM-063",
    "DGM-064", "DGM-065", "DGM-066", "DGM-067", "DGM-068", "DGM-069", "DGM-070", "DGM-071",
    "DGM-072", "DGM-073", "DGM-074", "DGM-075", "DGM-076", "DGM-077", "DGM-078", "DGM-079",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceClass {
    Evidence,
    SourceClaim,
    Inference,
    Missing,
}

impl EvidenceClass {
    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().trim_matches('`') {
            "EVIDENCE" => Ok(Self::Evidence),
            "SOURCE CLAIM" => Ok(Self::SourceClaim),
            "INFERENCE" => Ok(Self::Inference),
            "MISSING" => Ok(Self::Missing),
            other => Err(format!("invalid evidence class `{other}`")),
        }
    }
}

#[derive(Debug)]
struct LedgerEntry {
    id: String,
    slug: String,
    fields: BTreeMap<String, Vec<String>>,
}

impl LedgerEntry {
    fn field(&self, name: &str) -> Result<&str, String> {
        let values = self
            .fields
            .get(name)
            .ok_or_else(|| format!("{} is missing field {name}", self.id))?;
        if values.len() != 1 {
            return Err(format!("{} field {name} must appear exactly once", self.id));
        }
        let value = values[0].trim();
        if value.is_empty() {
            return Err(format!("{} has empty field {name}", self.id));
        }
        Ok(value)
    }

    fn class(&self) -> Result<EvidenceClass, String> {
        EvidenceClass::parse(self.field("Class")?).map_err(|error| format!("{}: {error}", self.id))
    }

    fn fields(&self, name: &str) -> &[String] {
        self.fields.get(name).map(Vec::as_slice).unwrap_or(&[])
    }
}

#[derive(Debug)]
struct MarkdownHeading {
    level: HeadingLevel,
    text: String,
    range: Range<usize>,
}

#[derive(Debug)]
struct MainClaim {
    class: EvidenceClass,
    id: String,
    target: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
        .to_path_buf()
}

fn packet_root() -> PathBuf {
    workspace_root().join("knowledge/darwin_godel_machine")
}

fn packet_files() -> Vec<PathBuf> {
    let mut files = fs::read_dir(packet_root())
        .expect("read DGM knowledge packet")
        .map(|entry| entry.expect("read packet entry").path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH
}

fn markdown_slug(heading: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in heading.to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            if separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(character);
            separator = false;
        } else if character.is_ascii_whitespace() || character == '-' {
            separator = true;
        }
    }
    slug
}

fn markdown_headings(text: &str) -> Vec<MarkdownHeading> {
    let mut headings = Vec::new();
    let mut current: Option<(HeadingLevel, usize, String)> = None;

    for (event, range) in Parser::new_ext(text, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some((level, range.start, String::new()));
            }
            Event::Text(value) | Event::Code(value) if current.is_some() => {
                current
                    .as_mut()
                    .expect("heading is present")
                    .2
                    .push_str(&value);
            }
            Event::SoftBreak | Event::HardBreak if current.is_some() => {
                current.as_mut().expect("heading is present").2.push(' ');
            }
            Event::End(TagEnd::Heading(level)) => {
                let (start_level, start, heading) =
                    current.take().expect("heading start precedes end");
                assert_eq!(start_level, level, "balanced Markdown heading events");
                headings.push(MarkdownHeading {
                    level,
                    text: heading.trim().to_owned(),
                    range: start..range.end,
                });
            }
            _ => {}
        }
    }
    headings
}

fn markdown_links(text: &str) -> Vec<String> {
    Parser::new_ext(text, markdown_options())
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. }) => Some(dest_url.to_string()),
            _ => None,
        })
        .collect()
}

fn wiki_links(text: &str) -> Result<Vec<String>, String> {
    let mut links = Vec::new();
    let mut fenced = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let mut remainder = line;
        while let Some(start) = remainder.find("[[") {
            remainder = &remainder[start + 2..];
            let (raw, rest) = remainder
                .split_once("]]")
                .ok_or_else(|| "unclosed native wiki link".to_owned())?;
            let (target, alias) = raw
                .split_once('|')
                .map_or((raw, None), |(target, alias)| (target, Some(alias)));
            if target.split('|').count() != 1 || alias.is_some_and(|value| value.trim().is_empty())
            {
                return Err("malformed native wiki link".to_owned());
            }
            let target = target.trim();
            if target.is_empty() {
                return Err("native wiki link has an empty target".to_owned());
            }
            links.push(target.to_owned());
            remainder = rest;
        }
    }
    if fenced {
        return Err("unclosed Markdown code fence".to_owned());
    }
    Ok(links)
}

fn resolve_wiki_link(source: &Path, target: &str) -> Result<PathBuf, String> {
    let (path, heading) = target
        .split_once('#')
        .map_or((target, None), |(path, heading)| (path, Some(heading)));
    if path.is_empty() {
        return Err("same-note wiki links are unsupported by this packet contract".to_owned());
    }
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || !candidate
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!("native wiki link escapes repository: {target}"));
    }
    let root = workspace_root();
    let rooted = matches!(
        candidate.components().next(),
        Some(Component::Normal(component))
            if matches!(component.to_str(), Some("knowledge" | "evidence" | "content" | "labs" | "crates"))
    );
    let resolved = if rooted {
        root.join(candidate)
    } else {
        source
            .parent()
            .ok_or_else(|| format!("{} has no parent", source.display()))?
            .join(candidate)
    };
    let resolved = if resolved.extension().is_some() {
        resolved
    } else {
        resolved.with_extension("md")
    };
    if !resolved.is_file() {
        return Err(format!("unresolved native wiki link: {target}"));
    }
    let canonical_root = fs::canonicalize(&root).map_err(|error| error.to_string())?;
    let canonical_target = fs::canonicalize(&resolved).map_err(|error| error.to_string())?;
    if !canonical_target.starts_with(canonical_root) {
        return Err(format!(
            "native wiki link escapes through symlink: {target}"
        ));
    }
    if let Some(heading) = heading {
        if resolved
            .extension()
            .and_then(|extension| extension.to_str())
            != Some("md")
        {
            return Err(format!(
                "native wiki heading targets non-Markdown file: {target}"
            ));
        }
        let headings =
            markdown_headings(&fs::read_to_string(&resolved).map_err(|error| error.to_string())?)
                .into_iter()
                .map(|heading| markdown_slug(&heading.text))
                .collect::<BTreeSet<_>>();
        if !headings.contains(&markdown_slug(heading)) {
            return Err(format!("native wiki heading is missing: {target}"));
        }
    }
    Ok(resolved)
}

fn parse_ledger_fields(id: &str, body: &str) -> Result<BTreeMap<String, Vec<String>>, String> {
    validate_ledger_field_list(id, body)?;
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (event, range) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
        let Event::Start(Tag::Item) = event else {
            continue;
        };
        let raw = body[range].trim();
        let item = raw.strip_prefix("- ").unwrap_or(raw);
        let (name, value) = item
            .split_once(':')
            .ok_or_else(|| format!("{id} has malformed field item `{item}`"))?;
        let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
        fields
            .entry(name.trim().to_owned())
            .or_default()
            .push(value);
    }
    Ok(fields)
}

fn validate_ledger_field_list(id: &str, body: &str) -> Result<(), String> {
    let mut depth = 0_usize;
    let mut top_level_lists = 0_usize;
    for event in Parser::new_ext(body, markdown_options()) {
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    if matches!(tag, Tag::List(_)) {
                        top_level_lists += 1;
                    } else {
                        return Err(format!("{id} has content outside its field list"));
                    }
                } else if matches!(tag, Tag::List(_)) {
                    return Err(format!("{id} has a nested field list"));
                }
                depth += 1;
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).expect("balanced Markdown events");
            }
            Event::Text(value) | Event::Code(value) if depth == 0 && !value.trim().is_empty() => {
                return Err(format!("{id} has content outside its field list"));
            }
            Event::Html(value) | Event::InlineHtml(value)
                if depth == 0 && !value.trim().is_empty() =>
            {
                return Err(format!("{id} has content outside its field list"));
            }
            _ => {}
        }
    }
    if top_level_lists != 1 {
        return Err(format!("{id} must contain exactly one field list"));
    }
    Ok(())
}

fn ledger_entries(text: &str) -> Result<Vec<LedgerEntry>, String> {
    let headings = markdown_headings(text);
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, heading) in headings.iter().enumerate() {
        if heading.level != HeadingLevel::H2 {
            continue;
        }
        let Some((id, title)) = heading.text.split_once(':') else {
            continue;
        };
        let id = id.trim();
        if !id.starts_with("DGM-") {
            continue;
        }
        if title.trim().is_empty() || !seen.insert(id.to_owned()) {
            return Err(format!(
                "malformed or duplicate claim heading {}",
                heading.text
            ));
        }
        let body_end = headings
            .iter()
            .skip(index + 1)
            .find(|next| next.level == HeadingLevel::H2)
            .map_or(text.len(), |next| next.range.start);
        entries.push(LedgerEntry {
            id: id.to_owned(),
            slug: markdown_slug(&heading.text),
            fields: parse_ledger_fields(id, &text[heading.range.end..body_end])?,
        });
    }
    Ok(entries)
}

fn main_claims(text: &str) -> Result<Vec<MainClaim>, String> {
    let mut claims = Vec::new();
    for (event, range) in Parser::new_ext(text, markdown_options()).into_offset_iter() {
        let Event::Start(Tag::Paragraph) = event else {
            continue;
        };
        let paragraph = text[range].trim();
        let Some(line) = paragraph.lines().next() else {
            continue;
        };
        let Some(marker) = line.trim().strip_prefix("**[") else {
            continue;
        };
        let (label, rest) = marker
            .split_once("](")
            .ok_or_else(|| format!("malformed claim marker `{line}`"))?;
        let target = rest
            .strip_suffix(").**")
            .ok_or_else(|| format!("malformed claim target `{line}`"))?;
        let (class, id) = label
            .split_once(" - ")
            .ok_or_else(|| format!("malformed claim label `{label}`"))?;
        claims.push(MainClaim {
            class: EvidenceClass::parse(class)?,
            id: id.to_owned(),
            target: target.to_owned(),
        });
    }
    Ok(claims)
}

fn validate_dgm_ledger(path: &Path, text: &str) -> Result<Vec<LedgerEntry>, String> {
    let entries = ledger_entries(text)?;
    let ids = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    for required in DGM_LEDGER_IDS {
        if !ids.contains(required) {
            return Err(format!("ledger is missing {required}"));
        }
    }
    for entry in &entries {
        for field in [
            "Class",
            "Statement",
            "Source",
            "Locator",
            "Scope",
            "Reproduction",
            "Confidence",
            "Confidence basis",
            "Caveat",
        ] {
            entry.field(field)?;
        }
        let class = entry.class()?;
        let confidence = entry.field("Confidence")?.trim().trim_matches('`');
        if !matches!(confidence, "high" | "medium" | "low") {
            return Err(format!(
                "{} has invalid confidence `{confidence}`",
                entry.id
            ));
        }
        validate_source_routes(path, entry)?;
        let locator = entry.field("Locator")?;
        let links = markdown_links(locator);
        if links.is_empty() {
            return Err(format!("{} locator must be clickable", entry.id));
        }
        for link in links {
            let relative = link.split_once('#').map_or(link.as_str(), |(path, _)| path);
            let resolved =
                fs::canonicalize(path.parent().expect("ledger parent").join(relative))
                    .map_err(|error| format!("resolve {} locator {link}: {error}", entry.id))?;
            let evidence = fs::canonicalize(workspace_root().join("evidence"))
                .map_err(|error| format!("resolve evidence root: {error}"))?;
            if relative.is_empty() || !resolved.starts_with(&evidence) {
                return Err(format!(
                    "{} locator must target underlying evidence: {link}",
                    entry.id
                ));
            }
            validate_line_anchor(&resolved, &link)
                .map_err(|error| format!("{}: {error}", entry.id))?;
        }
        match class {
            EvidenceClass::Evidence | EvidenceClass::SourceClaim => {
                let mode = entry.field("Mode")?.trim().trim_matches('`');
                if !matches!(mode, "quote" | "paraphrase") {
                    return Err(format!("{} has invalid mode `{mode}`", entry.id));
                }
                let stability = entry.field("Source stability")?.trim().trim_matches('`');
                if stability != "pinned" {
                    return Err(format!(
                        "{} has unsupported source stability `{stability}`",
                        entry.id
                    ));
                }
            }
            EvidenceClass::Inference => {
                entry.field("Weakens if")?;
                entry.field("Falsified by")?;
            }
            EvidenceClass::Missing => {
                entry.field("Resolves when")?;
            }
        }
    }
    validate_ledger_relationships(&entries)?;
    Ok(entries)
}

fn validate_source_routes(path: &Path, entry: &LedgerEntry) -> Result<(), String> {
    let registry_path = path
        .parent()
        .expect("ledger parent")
        .join("source_registry.md");
    let registry = fs::read_to_string(&registry_path)
        .map_err(|error| format!("read source registry: {error}"))?;
    let registry_headings = markdown_headings(&registry)
        .into_iter()
        .filter(|heading| heading.level == HeadingLevel::H2)
        .map(|heading| markdown_slug(&heading.text))
        .collect::<BTreeSet<_>>();
    let links = markdown_links(entry.field("Source")?);
    if links.is_empty() {
        return Err(format!("{} source route must be clickable", entry.id));
    }
    for link in links {
        let (target, anchor) = link
            .split_once('#')
            .ok_or_else(|| format!("{} source route lacks an anchor: {link}", entry.id))?;
        if target != "source_registry.md" || !registry_headings.contains(anchor) {
            return Err(format!(
                "{} source route does not land on a registry heading: {link}",
                entry.id
            ));
        }
    }
    Ok(())
}

fn validate_line_anchor(path: &Path, link: &str) -> Result<(), String> {
    let Some((_, anchor)) = link.split_once('#') else {
        return Ok(());
    };
    let Some(line) = anchor.strip_prefix('L') else {
        return Err(format!("unsupported evidence anchor `{anchor}`"));
    };
    let line = line
        .parse::<usize>()
        .map_err(|_| format!("malformed line anchor `{anchor}`"))?;
    let count = fs::read_to_string(path)
        .map_err(|error| format!("read {}: {error}", path.display()))?
        .lines()
        .count();
    if line == 0 || line > count {
        return Err(format!(
            "line anchor {anchor} exceeds {count} lines in {}",
            path.display()
        ));
    }
    Ok(())
}

fn validate_ledger_relationships(entries: &[LedgerEntry]) -> Result<(), String> {
    let ids = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut reciprocal = BTreeSet::new();
    for entry in entries {
        for relationship in entry.fields("Relationship") {
            let mut parts = relationship.trim().trim_matches('`').split_whitespace();
            let kind = parts
                .next()
                .ok_or_else(|| format!("{} has an empty relationship", entry.id))?;
            let target = parts
                .next()
                .ok_or_else(|| format!("{} relationship has no target", entry.id))?;
            if parts.next().is_some()
                || !matches!(
                    kind,
                    "contradicts" | "unresolved-with" | "supersedes" | "narrows"
                )
            {
                return Err(format!(
                    "{} has invalid relationship `{relationship}`",
                    entry.id
                ));
            }
            if target == entry.id || !ids.contains(target) {
                return Err(format!(
                    "{} relationship has invalid target {target}",
                    entry.id
                ));
            }
            if matches!(kind, "contradicts" | "unresolved-with") {
                reciprocal.insert((entry.id.as_str(), kind, target));
            }
        }
    }
    for &(source, kind, target) in &reciprocal {
        if !reciprocal.contains(&(target, kind, source)) {
            return Err(format!(
                "{source} relationship `{kind} {target}` is not reciprocal"
            ));
        }
    }
    Ok(())
}

fn frontmatter(text: &str) -> &str {
    let rest = text
        .strip_prefix("---\n")
        .expect("packet document must start with YAML frontmatter");
    rest.split_once("\n---\n")
        .expect("packet document must close YAML frontmatter")
        .0
}

fn document_body(text: &str) -> &str {
    if let Some(rest) = text.strip_prefix("---\n") {
        rest.split_once("\n---\n")
            .expect("packet document must close YAML frontmatter")
            .1
    } else {
        text
    }
}

fn parse_frontmatter(text: &str) -> Result<BTreeMap<&str, &str>, String> {
    let mut values = BTreeMap::new();
    for (index, line) in frontmatter(text).lines().enumerate() {
        if line.is_empty() || line.starts_with(' ') || line.starts_with('\t') {
            return Err(format!(
                "frontmatter line {} must be one top-level key/value",
                index + 1
            ));
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("frontmatter line {} is missing ':'", index + 1))?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(format!(
                "frontmatter line {} has an empty key or value",
                index + 1
            ));
        }
        if values.insert(key, value).is_some() {
            return Err(format!("duplicate frontmatter key {key}"));
        }
    }
    Ok(values)
}

fn validate_frontmatter(text: &str) -> Result<BTreeMap<&str, &str>, String> {
    let values = parse_frontmatter(text)?;
    for key in REQUIRED_FRONTMATTER {
        if !values.contains_key(key) {
            return Err(format!("missing frontmatter key {key}"));
        }
    }
    if values.len() != REQUIRED_FRONTMATTER.len() {
        return Err("frontmatter contains unsupported keys".to_owned());
    }
    if !ALLOWED_TYPES.contains(&values["type"]) {
        return Err(format!("unsupported packet type {}", values["type"]));
    }
    for key in ["id", "title", "status", "confidence"] {
        let value = values[key];
        if value.starts_with('[')
            || value.starts_with('{')
            || value.contains('#')
            || value.contains('\t')
            || ((value.starts_with('"') || value.starts_with('\''))
                && value.as_bytes().last() != value.as_bytes().first())
        {
            return Err(format!("{key} must be a nonempty scalar value"));
        }
    }
    for key in ["created", "updated"] {
        let raw = values[key];
        let value = raw.as_bytes();
        let parse_part = |range: std::ops::Range<usize>| {
            raw.get(range)
                .and_then(|part| part.parse::<u32>().ok())
                .unwrap_or(0)
        };
        let year = parse_part(0..4);
        let month = parse_part(5..7);
        let day = parse_part(8..10);
        let leap =
            year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
        let max_day = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => 0,
        };
        if value.len() != 10
            || value[4] != b'-'
            || value[7] != b'-'
            || value
                .iter()
                .enumerate()
                .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
            || year == 0
            || day == 0
            || day > max_day
        {
            return Err(format!("{key} must use YYYY-MM-DD"));
        }
    }
    let tags = values["tags"];
    if !tags.starts_with('[')
        || !tags.ends_with(']')
        || !tags[1..tags.len() - 1]
            .split(',')
            .map(str::trim)
            .all(|tag| {
                !tag.is_empty()
                    && tag
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            })
    {
        return Err("tags must be a nonempty inline list".to_owned());
    }
    let canonical = values["canonical"];
    if canonical.starts_with('/')
        || canonical.contains("://")
        || !(canonical.ends_with(".md") || canonical.ends_with(".tsv"))
        || canonical.split('/').any(|part| part.is_empty())
    {
        return Err("canonical must be a repository-relative Markdown or TSV path".to_owned());
    }
    Ok(values)
}

fn markdown_table_first_column(text: &str, heading: &str) -> Result<BTreeSet<String>, String> {
    let section = text
        .split_once(heading)
        .ok_or_else(|| format!("missing section {heading}"))?
        .1;
    let section = section.split("\n## ").next().unwrap_or(section);
    let mut rows = section
        .lines()
        .filter(|line| line.trim_start().starts_with('|'));
    let header = rows
        .next()
        .ok_or_else(|| "missing table header".to_owned())?;
    let separator = rows
        .next()
        .ok_or_else(|| "missing table separator".to_owned())?;
    if !separator.contains("---") {
        return Err("malformed table separator".to_owned());
    }
    let first_header = header
        .trim_matches('|')
        .split('|')
        .next()
        .expect("table header column")
        .trim();
    if first_header != "ID" {
        return Err(format!("unexpected first table column {first_header}"));
    }

    let mut values = BTreeSet::new();
    for row in rows {
        let value = row
            .trim_matches('|')
            .split('|')
            .next()
            .expect("table row column")
            .trim()
            .trim_matches('`');
        if value.is_empty() || !values.insert(value.to_owned()) {
            return Err(format!("empty or duplicate table ID {value}"));
        }
    }
    Ok(values)
}

fn prose_outside_fences(text: &str) -> String {
    let mut in_fence = false;
    let mut prose = String::new();

    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            prose.push_str(line);
            prose.push('\n');
        }
    }

    prose
}

fn display_math_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut open = None;
    let mut offset = 0;

    for line in text.split_inclusive('\n') {
        if line.trim() == "$$" {
            if let Some(start) = open.take() {
                ranges.push(start..offset + line.len());
            } else {
                open = Some(offset);
            }
        }
        offset += line.len();
    }
    if let Some(start) = open {
        ranges.push(start..text.len());
    }
    ranges
}

fn markdown_targets(text: &str) -> Result<Vec<String>, String> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH;
    let math_ranges = display_math_ranges(text);
    let mut broken_references = Vec::new();
    let mut callback = |link: BrokenLink<'_>| {
        if !math_ranges
            .iter()
            .any(|range| range.start <= link.span.start && link.span.end <= range.end)
        {
            broken_references.push(link.reference.to_string());
        }
        None
    };
    let targets = Parser::new_with_broken_link_callback(text, options, Some(&mut callback))
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. })
            | Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if broken_references.is_empty() {
        Ok(targets)
    } else {
        Err(format!(
            "undefined reference links: {}",
            broken_references.join(", ")
        ))
    }
}

fn malformed_inline_links(text: &str) -> Vec<String> {
    prose_outside_fences(text)
        .lines()
        .filter(|line| {
            let mut remainder = *line;
            while let Some(start) = remainder.find("](") {
                let destination = &remainder[start + 2..];
                if !destination.contains(')') {
                    return true;
                }
                remainder = &destination[destination.find(')').expect("checked") + 1..];
            }
            false
        })
        .map(str::to_owned)
        .collect()
}

fn is_external_target(target: &str) -> bool {
    target.starts_with('#')
        || target.starts_with("https://")
        || target.starts_with("http://")
        || target.starts_with("mailto:")
}

fn normalize_link_target(target: &str) -> &str {
    target.split('#').next().expect("link path")
}

fn forbidden_absolute_path(text: &str) -> Option<String> {
    fn allowed_sandbox_path(token: &str) -> bool {
        let relative = if matches!(token, "/dgm" | "/dgm/" | "/testbed" | "/testbed/") {
            return true;
        } else if let Some(relative) = token.strip_prefix("/dgm/") {
            relative
        } else if let Some(relative) = token.strip_prefix("/testbed/") {
            relative
        } else {
            return false;
        };

        relative
            .trim_end_matches('/')
            .split('/')
            .all(|component| !matches!(component, "" | "." | ".."))
    }

    let lower = text.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let windows_absolute = bytes.windows(3).enumerate().any(|(index, window)| {
        let boundary = index == 0
            || bytes[index - 1].is_ascii_whitespace()
            || matches!(
                bytes[index - 1],
                b'`' | b'\'' | b'"' | b'(' | b'[' | b'{' | b',' | b';'
            );
        boundary
            && window[0].is_ascii_alphabetic()
            && window[1] == b':'
            && matches!(window[2], b'\\' | b'/')
    });
    if lower.contains("file://") {
        return Some("file://".to_owned());
    }
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
    .find(|token| token != &"/" && token.starts_with('/') && !allowed_sandbox_path(token))
    .map(str::to_owned)
}

fn has_forbidden_absolute_path(text: &str) -> bool {
    forbidden_absolute_path(text).is_some()
}

fn validate_local_links(path: &Path, text: &str) -> Result<(), String> {
    let text = document_body(text);
    if let Some(line) = malformed_inline_links(text).first() {
        return Err(format!("malformed inline link in {line:?}"));
    }
    for target in markdown_targets(text)? {
        if target.is_empty() || is_external_target(&target) {
            continue;
        }

        let relative = normalize_link_target(&target);
        if relative.is_empty() {
            continue;
        }
        let resolved = path.parent().expect("document parent").join(relative);
        if !resolved.exists() {
            return Err(format!("unresolved local link {target}"));
        }
    }
    for target in wiki_links(text)? {
        resolve_wiki_link(path, &target)?;
    }
    Ok(())
}

#[test]
fn dgm_knowledge_packet_has_complete_observable_contract() {
    let root = packet_root();
    assert!(root.is_dir(), "missing {}", root.display());
    assert!(
        !root.join("README.md").exists(),
        "packet must not use README.md"
    );

    let files = packet_files();
    let names = files
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("UTF-8 packet filename")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    let expected = EXPECTED_FILES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(names, expected, "packet file inventory drifted");

    let revision =
        fs::read_to_string(workspace_root().join("evidence/implementations/dgm/REVISION"))
            .expect("read pinned DGM revision");
    let revision = revision.trim();
    assert_eq!(
        revision.len(),
        40,
        "DGM revision must be a full Git object ID"
    );

    for path in files {
        let text = fs::read_to_string(&path).expect("read packet document");
        validate_frontmatter(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));

        assert!(
            text.contains(AUTHORITY_NOTICE),
            "{} is missing the exact authority notice",
            path.display()
        );
        assert!(
            !text.contains("\\n+"),
            "{} contains a serialized patch artifact",
            path.display()
        );
        let prose = prose_outside_fences(&text);
        for placeholder in ["TODO", "TBD", "FIXME"] {
            assert!(
                !prose.contains(placeholder),
                "{} contains placeholder {placeholder}",
                path.display()
            );
        }
        if let Some(forbidden) = forbidden_absolute_path(&text) {
            panic!(
                "{} contains an absolute local path: {forbidden}",
                path.display()
            );
        }

        if path.file_name().and_then(|name| name.to_str()) != Some("darwin_godel_machine_index.md")
        {
            assert!(
                text.contains("darwin_godel_machine_index.md"),
                "{} does not link back to the packet index",
                path.display()
            );
            assert!(
                text.trim_end()
                    .ends_with("Back to the [DGM index](darwin_godel_machine_index.md)."),
                "{} does not end with the approved index navigation line",
                path.display()
            );
        }

        validate_local_links(&path, &text)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    }

    let index = fs::read_to_string(root.join("darwin_godel_machine_index.md"))
        .expect("read DGM packet index");
    for name in EXPECTED_FILES {
        if name != "darwin_godel_machine_index.md" {
            assert!(index.contains(name), "packet index does not link {name}");
        }
    }

    let repository_walkthrough = fs::read_to_string(root.join("05_repository_walkthrough.md"))
        .expect("read repository walkthrough");
    let source_registry =
        fs::read_to_string(root.join("source_registry.md")).expect("read source registry");
    assert!(
        repository_walkthrough.contains(revision),
        "repository walkthrough revision does not match evidence receipt"
    );
    assert!(
        source_registry.contains(revision),
        "source registry revision does not match evidence receipt"
    );
    let source_ids = markdown_table_first_column(&source_registry, "## Source classes")
        .expect("parse source registry table");
    assert!(source_ids.contains("DGM"));
    assert!(source_ids.contains("DGM-REPO"));
    let source_headings = markdown_headings(&source_registry)
        .into_iter()
        .filter(|heading| heading.level == HeadingLevel::H2)
        .map(|heading| markdown_slug(&heading.text))
        .collect::<BTreeSet<_>>();
    for source in [
        "dgm-iclr-2026-paper",
        "dgm-repo-pinned-implementation",
        "harp-dgm-canonical-synthesis",
        "harp-rsi-canonical-evaluation-framework",
    ] {
        assert!(
            source_headings.contains(source),
            "source registry is missing heading {source}"
        );
    }

    let crosswalk =
        fs::read_to_string(root.join("claim_evidence_crosswalk.md")).expect("read claim crosswalk");
    let ledger_path = root.join("claim_evidence_crosswalk.md");
    let entries =
        validate_dgm_ledger(&ledger_path, &crosswalk).expect("validate heading-based DGM ledger");
    assert!(
        !crosswalk.contains("- Class: `CLAIM`"),
        "DGM ledger uses deprecated bare CLAIM"
    );

    let entry_by_slug = entries
        .iter()
        .map(|entry| (entry.slug.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut routed_claims = 0;
    for path in packet_files() {
        let text = fs::read_to_string(&path).expect("read packet document");
        assert!(
            !text.contains("**CLAIM") && !text.contains("| CLAIM |"),
            "{} uses deprecated bare CLAIM",
            path.display()
        );
        if path.file_name().and_then(|name| name.to_str()) == Some("claim_evidence_crosswalk.md") {
            continue;
        }
        for target in markdown_links(document_body(&text)) {
            if let Some(anchor) = target.strip_prefix("claim_evidence_crosswalk.md#") {
                assert!(
                    entry_by_slug.contains_key(anchor),
                    "{} links to missing ledger anchor {anchor}",
                    path.display()
                );
            }
        }
        for claim in main_claims(&text).expect("parse DGM claim blocks") {
            routed_claims += 1;
            let (ledger_file, anchor) = claim
                .target
                .split_once('#')
                .expect("DGM claim route must include a ledger anchor");
            assert_eq!(
                ledger_file,
                "claim_evidence_crosswalk.md",
                "{} claim {} does not route through the canonical ledger",
                path.display(),
                claim.id
            );
            let entry = entry_by_slug
                .get(anchor)
                .unwrap_or_else(|| panic!("{} targets missing ledger anchor {anchor}", claim.id));
            assert_eq!(claim.id, entry.id);
            assert_eq!(claim.class, entry.class().expect("validated class"));
        }
    }
    assert!(
        routed_claims >= 20,
        "packet needs at least 20 reader-facing claim routes, found {routed_claims}"
    );
}

#[test]
fn frontmatter_validation_rejects_malformed_shapes() {
    let valid = concat!(
        "---\n",
        "id: example\n",
        "title: Example\n",
        "type: concept\n",
        "status: active\n",
        "created: 2026-08-08\n",
        "updated: 2026-08-08\n",
        "tags: [example]\n",
        "confidence: high\n",
        "canonical: ../../content/example.md\n",
        "---\n",
        "# Example\n"
    );
    assert!(validate_frontmatter(valid).is_ok());

    for invalid in [
        valid.replace("title: Example\n", ""),
        valid.replace("type: concept", "type: unsupported"),
        valid.replace("tags: [example]", "tags: []"),
        valid.replace("tags: [example]", "tags: [ ]"),
        valid.replace("tags: [example]", "tags: [example, ]"),
        valid.replace("title: Example", "title: [Example]"),
        valid.replace("title: Example", "title: \"unterminated"),
        valid.replace("created: 2026-08-08", "created: today"),
        valid.replace("created: 2026-08-08", "created: 2026-99-99"),
        valid.replace("created: 2026-08-08", "created: 2026-02-30"),
        valid.replace(
            "canonical: ../../content/example.md",
            "canonical: /tmp/example.md",
        ),
        valid.replace("status: active\n", "status: active\nstatus: duplicate\n"),
        valid.replace("title: Example\n", "  title: nested\n"),
    ] {
        assert!(
            validate_frontmatter(&invalid).is_err(),
            "malformed frontmatter unexpectedly passed:\n{invalid}"
        );
    }
}

#[test]
fn source_registry_validation_requires_structural_source_ids() {
    let valid = concat!(
        "## Source classes\n\n",
        "| ID | Class |\n",
        "|---|---|\n",
        "| DGM | paper |\n",
        "| DGM-REPO | repository |\n",
    );
    let ids =
        markdown_table_first_column(valid, "## Source classes").expect("valid source registry");
    assert_eq!(
        ids,
        BTreeSet::from(["DGM".to_owned(), "DGM-REPO".to_owned()])
    );

    let missing = valid.replace("| DGM-REPO | repository |\n", "");
    let missing_ids =
        markdown_table_first_column(&missing, "## Source classes").expect("parse missing row");
    assert!(!missing_ids.contains("DGM-REPO"));

    for invalid in [
        valid.replace("| DGM-REPO | repository |\n", "| DGM | duplicate |\n"),
        valid.replace("| ID | Class |", "| Source | Class |"),
    ] {
        assert!(
            markdown_table_first_column(&invalid, "## Source classes").is_err(),
            "malformed source table unexpectedly passed: {invalid}"
        );
    }
}

#[test]
fn link_validation_covers_supported_and_broken_forms() {
    let root = workspace_root();
    let document = root.join("knowledge/darwin_godel_machine/link-test.md");
    let valid = concat!(
        "[anchor](#section)\n",
        "[external](https://example.com/path_(x))\n",
        "![image](../../evidence/implementations/dgm/LICENSE \"license\")\n",
        "[local](../rsi/systems/dgm.md#problem-and-rsi-relevance)\n",
        "[reference][dgm]\n\n",
        "[dgm]: ../rsi/systems/dgm.md \"DGM\"\n",
    );
    validate_local_links(&document, valid).expect("supported Markdown links");

    for invalid in [
        "[broken](../rsi/does-not-exist.md)",
        "![broken](../../evidence/does-not-exist.png)",
        "[unclosed](../rsi/systems/dgm.md",
        "[undefined][missing]",
        "![undefined][missing-image]",
    ] {
        assert!(
            validate_local_links(&document, invalid).is_err(),
            "broken Markdown link unexpectedly passed: {invalid}"
        );
    }
}

#[test]
fn native_wiki_links_validate_aliases_headings_and_escape_safely() {
    let document = workspace_root().join("knowledge/darwin_godel_machine/link-test.md");
    for valid in [
        "[[darwin_godel_machine_index|DGM index]]",
        "[[source_registry#DGM: ICLR 2026 paper|paper source]]",
        "[[evidence/implementations/dgm/snapshot/DGM_outer.py|controller source]]",
    ] {
        let target = wiki_links(valid)
            .expect("parse native wiki fixture")
            .into_iter()
            .next()
            .expect("one native wiki fixture");
        resolve_wiki_link(&document, &target).unwrap_or_else(|error| panic!("{valid}: {error}"));
    }
    for invalid in ["[[../../outside]]", "[[/etc/passwd]]"] {
        let target = wiki_links(invalid)
            .expect("parse native wiki fixture")
            .into_iter()
            .next()
            .expect("one native wiki fixture");
        assert!(
            resolve_wiki_link(&document, &target).is_err(),
            "escaping native wiki link unexpectedly passed: {invalid}"
        );
    }
}

#[test]
fn prose_guards_ignore_fenced_examples_but_reject_real_placeholders_and_paths() {
    let fenced = "```sh\nrg 'TODO|TBD|FIXME|/tmp/'\n```\n";
    let prose = prose_outside_fences(fenced);
    assert!(!["TODO", "TBD", "FIXME"]
        .iter()
        .any(|placeholder| prose.contains(placeholder)));
    assert!(!has_forbidden_absolute_path(
        "work in /dgm and /testbed/task"
    ));

    for invalid in [
        "TODO: replace this",
        "TBD",
        "FIXME",
        "read /home/user/file",
        "read /opt/tool/file",
        "read /var/tmp/file",
        "read /etc/passwd",
        "read /private/tmp/file",
        "read /workspace/file",
        "read /dgm/../../etc/passwd",
        "read /testbed/../private/key",
        "read C:\\work\\file",
        "read C:/work/file",
        "open file://local/path",
    ] {
        assert!(
            ["TODO", "TBD", "FIXME"]
                .iter()
                .any(|placeholder| invalid.contains(placeholder))
                || has_forbidden_absolute_path(invalid)
        );
    }
}
