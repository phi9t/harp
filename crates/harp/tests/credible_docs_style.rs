use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

const EXPECTED_FILES: [&str; 7] = [
    "CONTEXT.md",
    "STYLE_GUIDE.md",
    "claim-ledger-template.md",
    "main-document-template.md",
    "review-checklist.md",
    "source-registry-template.md",
    "worked-example.md",
];

const REQUIRED_FIELDS: [&str; 9] = [
    "Class",
    "Statement",
    "Source",
    "Locator",
    "Scope",
    "Reproduction",
    "Confidence",
    "Confidence basis",
    "Caveat",
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

    fn is_source_derived(self) -> bool {
        matches!(self, Self::Evidence | Self::SourceClaim)
    }
}

#[derive(Debug)]
struct ClaimEntry {
    id: String,
    slug: String,
    fields: BTreeMap<String, Vec<String>>,
}

impl ClaimEntry {
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

    fn fields(&self, name: &str) -> &[String] {
        self.fields.get(name).map(Vec::as_slice).unwrap_or(&[])
    }

    fn class(&self) -> Result<EvidenceClass, String> {
        EvidenceClass::parse(self.field("Class")?).map_err(|error| format!("{}: {error}", self.id))
    }
}

#[derive(Debug)]
struct MainClaim {
    class: EvidenceClass,
    class_label: String,
    id: String,
    target: LocalLink,
    body: String,
}

#[derive(Clone, Debug)]
struct LocalLink {
    target: String,
    native_wiki: bool,
}

#[derive(Debug)]
struct MarkdownHeading {
    level: HeadingLevel,
    text: String,
    range: Range<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MarkdownBlockKind {
    Paragraph,
    List,
}

#[derive(Debug)]
struct MarkdownBlock {
    kind: MarkdownBlockKind,
    range: Range<usize>,
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
        .to_path_buf()
}

fn style_root() -> PathBuf {
    workspace_root().join("docs/writing-style")
}

fn markdown_links(text: &str) -> Vec<String> {
    Parser::new_ext(text, markdown_options())
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. })
            | Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.to_string()),
            _ => None,
        })
        .collect()
}

fn markdown_local_links(text: &str) -> Vec<LocalLink> {
    markdown_links(text)
        .into_iter()
        .map(|target| LocalLink {
            target,
            native_wiki: false,
        })
        .collect()
}

fn headings(text: &str) -> BTreeSet<String> {
    markdown_headings(text)
        .into_iter()
        .filter(|heading| heading.level == HeadingLevel::H2)
        .map(|heading| markdown_slug(&heading.text))
        .collect()
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
                let (_, _, heading) = current.as_mut().expect("heading is present");
                heading.push_str(&value);
            }
            Event::SoftBreak | Event::HardBreak if current.is_some() => {
                let (_, _, heading) = current.as_mut().expect("heading is present");
                heading.push(' ');
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

fn markdown_blocks(text: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut depth = 0_usize;
    for (event, range) in Parser::new_ext(text, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    let kind = match tag {
                        Tag::Paragraph => Some(MarkdownBlockKind::Paragraph),
                        Tag::List(_) => Some(MarkdownBlockKind::List),
                        _ => None,
                    };
                    if let Some(kind) = kind {
                        blocks.push(MarkdownBlock { kind, range });
                    }
                }
                depth += 1;
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).expect("balanced Markdown events");
            }
            _ => {}
        }
    }
    blocks
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

fn repository_relative_path(source: &Path, relative: &str) -> Result<PathBuf, String> {
    let root = workspace_root();
    let mut resolved = source
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", source.display()))?
        .to_path_buf();

    if relative.is_empty() {
        return Ok(source.to_path_buf());
    }
    let target = Path::new(relative);
    if target.is_absolute() {
        return Err(format!("absolute local link is not allowed: {relative}"));
    }
    for component in target.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !resolved.pop() || !resolved.starts_with(&root) {
                    return Err(format!(
                        "{} has local link that escapes the repository: {relative}",
                        source.display()
                    ));
                }
            }
            Component::Normal(part) => resolved.push(part),
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("absolute local link is not allowed: {relative}"));
            }
        }
    }
    if !resolved.starts_with(&root) {
        return Err(format!(
            "{} has local link that escapes the repository: {relative}",
            source.display()
        ));
    }
    if resolved.exists() {
        let canonical_root =
            fs::canonicalize(&root).map_err(|error| format!("resolve repository root: {error}"))?;
        let canonical_target = fs::canonicalize(&resolved)
            .map_err(|error| format!("resolve {}: {error}", resolved.display()))?;
        if !canonical_target.starts_with(canonical_root) {
            return Err(format!(
                "{} has local link that escapes through a symlink: {relative}",
                source.display()
            ));
        }
    }
    Ok(resolved)
}

fn resolve_link(source: &Path, target: &str) -> Result<(), String> {
    if target.starts_with("https://")
        || target.starts_with("http://")
        || target.starts_with("mailto:")
    {
        return Ok(());
    }
    if let Some(anchor) = target.strip_prefix('#') {
        let source_text = fs::read_to_string(source).map_err(|error| error.to_string())?;
        if !headings(&source_text).contains(anchor) {
            return Err(format!(
                "{} links to missing local anchor {anchor}",
                source.display()
            ));
        }
        return Ok(());
    }
    let (relative, anchor) = target
        .split_once('#')
        .map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let resolved = repository_relative_path(source, relative)?;
    if !resolved.is_file() {
        return Err(format!("{} has unresolved link {target}", source.display()));
    }
    if let Some(anchor) = anchor {
        let target_text = fs::read_to_string(&resolved).map_err(|error| error.to_string())?;
        if let Some(line) = anchor.strip_prefix('L') {
            let line = line.parse::<usize>().map_err(|_| {
                format!(
                    "{} has malformed line anchor {anchor} in {}",
                    source.display(),
                    resolved.display()
                )
            })?;
            if line == 0 || line > target_text.lines().count() {
                return Err(format!(
                    "{} links to missing line {line} in {}",
                    source.display(),
                    resolved.display()
                ));
            }
        } else if resolved.extension().and_then(|value| value.to_str()) == Some("md") {
            if !headings(&target_text).contains(anchor) {
                return Err(format!(
                    "{} links to missing anchor {anchor} in {}",
                    source.display(),
                    resolved.display()
                ));
            }
        } else {
            return Err(format!(
                "{} uses unsupported anchor {anchor} in {}",
                source.display(),
                resolved.display()
            ));
        }
    }
    Ok(())
}

fn parse_fields(id: &str, body: &str) -> Result<BTreeMap<String, Vec<String>>, String> {
    validate_field_list_structure(id, body)?;
    let mut fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (event, range) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
        let Event::Start(Tag::Item) = event else {
            continue;
        };
        let raw_item = body[range].trim();
        let item = raw_item.strip_prefix("- ").unwrap_or(raw_item);
        let (name, value) = item
            .split_once(':')
            .ok_or_else(|| format!("{id} has malformed field item `{item}`"))?;
        let name = name.trim();
        if name.is_empty() {
            return Err(format!("{id} has a field with no name"));
        }
        let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
        fields.entry(name.to_owned()).or_default().push(value);
    }
    Ok(fields)
}

fn validate_field_list_structure(id: &str, body: &str) -> Result<(), String> {
    let mut depth = 0_usize;
    let mut top_level_lists = 0_usize;
    for (event, _) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(tag) => {
                if depth == 0 {
                    if matches!(tag, Tag::List(_)) {
                        top_level_lists += 1;
                    } else {
                        return Err(format!("{id} has content outside the field list"));
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
                return Err(format!("{id} has content outside the field list"));
            }
            Event::Html(value) | Event::InlineHtml(value)
                if depth == 0 && !value.trim().is_empty() =>
            {
                return Err(format!("{id} has content outside the field list"));
            }
            _ => {}
        }
    }
    if top_level_lists != 1 {
        return Err(format!("{id} must contain exactly one field list"));
    }
    Ok(())
}

fn claim_entries(text: &str) -> Result<Vec<ClaimEntry>, String> {
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    let headings = markdown_headings(text);

    for (index, heading) in headings.iter().enumerate() {
        if heading.level != HeadingLevel::H2 {
            continue;
        }
        let Some((id, title)) = heading.text.split_once(':') else {
            continue;
        };
        let id = id.trim();
        if id.is_empty() || title.trim().is_empty() {
            return Err(format!("malformed claim heading `{}`", heading.text));
        }
        if !seen.insert(id.to_owned()) {
            return Err(format!("duplicate claim ID {id}"));
        }
        let body_start = heading.range.end;
        let body_end = headings
            .iter()
            .skip(index + 1)
            .find(|next| next.level == HeadingLevel::H2)
            .map_or(text.len(), |next| next.range.start);
        entries.push(ClaimEntry {
            id: id.to_owned(),
            slug: markdown_slug(&heading.text),
            fields: parse_fields(id, &text[body_start..body_end])?,
        });
    }
    Ok(entries)
}

fn validate_claim_entries(text: &str) -> Result<Vec<ClaimEntry>, String> {
    validate_claim_entries_at(&style_root().join("claim-ledger-template.md"), text)
}

fn validate_claim_entries_at(source: &Path, text: &str) -> Result<Vec<ClaimEntry>, String> {
    let entries = claim_entries(text)?;
    if entries.is_empty() {
        return Err("claim ledger has no canonical entries".to_owned());
    }

    for entry in &entries {
        for field in REQUIRED_FIELDS {
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
        for field in ["Source", "Locator"] {
            if markdown_local_links(entry.field(field)?).is_empty()
                && harp::knowledge::resolve_wiki_links(
                    &workspace_root(),
                    source,
                    entry.field(field)?,
                )
                .map_err(|error| error.to_string())?
                .is_empty()
            {
                return Err(format!("{} field {field} must contain a link", entry.id));
            }
        }
        validate_evidence_locators(source, entry)?;

        if class.is_source_derived() {
            let mode = entry.field("Mode")?.trim().trim_matches('`');
            if !matches!(mode, "quote" | "paraphrase") {
                return Err(format!("{} has invalid rendering mode `{mode}`", entry.id));
            }
            if mode == "quote" && !entry.field("Statement")?.contains('"') {
                return Err(format!(
                    "{} uses quote mode without a visible quotation boundary",
                    entry.id
                ));
            }
            let stability = entry.field("Source stability")?.trim().trim_matches('`');
            if !matches!(stability, "pinned" | "dated observation") {
                return Err(format!(
                    "{} has invalid source stability `{stability}`",
                    entry.id
                ));
            }
            if stability == "dated observation" {
                let observed = entry.field("Observed")?.trim().trim_matches('`');
                if !is_valid_date(observed) {
                    return Err(format!(
                        "{} has invalid observation date `{observed}`",
                        entry.id
                    ));
                }
            } else if entry.fields.contains_key("Observed") {
                return Err(format!(
                    "{} is pinned and must not declare an observation date",
                    entry.id
                ));
            }
        } else {
            for field in ["Mode", "Source stability", "Observed"] {
                if entry.fields.contains_key(field) {
                    return Err(format!(
                        "{} is not source-derived and must not declare {field}",
                        entry.id
                    ));
                }
            }
        }

        match class {
            EvidenceClass::Inference => {
                entry.field("Weakens if")?;
                entry.field("Falsified by")?;
            }
            EvidenceClass::Missing => {
                entry.field("Resolves when")?;
            }
            EvidenceClass::Evidence | EvidenceClass::SourceClaim => {}
        }
    }

    validate_relationships(&entries)?;
    Ok(entries)
}

fn validate_evidence_locators(source: &Path, entry: &ClaimEntry) -> Result<(), String> {
    let evidence_root = workspace_root().join("evidence");
    let canonical_evidence = fs::canonicalize(&evidence_root)
        .map_err(|error| format!("resolve evidence root: {error}"))?;
    let locator = entry.field("Locator")?;
    let markdown_locators = markdown_local_links(locator);
    let markdown_line_targets = markdown_locators
        .iter()
        .filter_map(|link| {
            let (target, anchor) = link.target.split_once('#')?;
            anchor
                .starts_with('L')
                .then(|| repository_relative_path(source, target))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for native in harp::knowledge::resolve_wiki_links(&workspace_root(), source, locator)
        .map_err(|error| error.to_string())?
    {
        let native_target = workspace_root().join(&native.target);
        if !native_target.starts_with(&evidence_root) {
            return Err(format!(
                "{} native locator must target a file under evidence/",
                entry.id
            ));
        }
        if native
            .target
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("txt")
            && !markdown_line_targets
                .iter()
                .any(|target| *target == native_target)
        {
            return Err(format!(
                "{} native raw-text locator requires a matching conventional Markdown line locator",
                entry.id
            ));
        }
    }
    for link in markdown_locators {
        let target = &link.target;
        if target.starts_with("https://")
            || target.starts_with("http://")
            || target.starts_with("mailto:")
        {
            return Err(format!(
                "{} locator must target a local file under evidence/",
                entry.id
            ));
        }
        let relative = target
            .split_once('#')
            .map_or(target.as_str(), |(path, _)| path);
        if relative.is_empty() {
            return Err(format!(
                "{} locator must target a file under evidence/, not another claim",
                entry.id
            ));
        }
        let resolved = repository_relative_path(source, relative)?;
        if !resolved.starts_with(&evidence_root) {
            return Err(format!(
                "{} locator must target a file under evidence/: {target}",
                entry.id
            ));
        }
        let canonical_target = fs::canonicalize(&resolved)
            .map_err(|error| format!("resolve evidence locator {}: {error}", resolved.display()))?;
        if !canonical_target.starts_with(&canonical_evidence) {
            return Err(format!(
                "{} locator must resolve beneath evidence/: {target}",
                entry.id
            ));
        }
        resolve_link(source, target)?;
    }
    Ok(())
}

fn is_valid_date(value: &str) -> bool {
    let Some((year, rest)) = value.split_once('-') else {
        return false;
    };
    let Some((month, day)) = rest.split_once('-') else {
        return false;
    };
    if year.len() != 4 || month.len() != 2 || day.len() != 2 {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

fn validate_relationships(entries: &[ClaimEntry]) -> Result<(), String> {
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
            if target == entry.id {
                return Err(format!("{} relationship must not target itself", entry.id));
            }
            if !ids.contains(target) {
                return Err(format!(
                    "{} relationship targets missing claim {target}",
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

fn parse_main_claims(text: &str) -> Result<Vec<MainClaim>, String> {
    let mut claims = Vec::new();
    let blocks = markdown_blocks(text);
    let mut index = 0;
    while index < blocks.len() {
        let block = &blocks[index];
        if block.kind != MarkdownBlockKind::Paragraph {
            index += 1;
            continue;
        }
        let paragraph = text[block.range.clone()].trim();
        let mut lines = paragraph.lines();
        let Some(line) = lines.next().map(str::trim) else {
            index += 1;
            continue;
        };
        let marker = if let Some(marker) = line.strip_prefix("**[[") {
            let (target, label) = marker
                .strip_suffix("]].**")
                .ok_or_else(|| format!("malformed native main claim marker `{line}`"))?
                .split_once('|')
                .ok_or_else(|| format!("malformed native main claim label `{line}`"))?;
            (
                label,
                LocalLink {
                    target: target.to_owned(),
                    native_wiki: true,
                },
            )
        } else if let Some(marker) = line.strip_prefix("**[") {
            let (label, rest) = marker
                .split_once("](")
                .ok_or_else(|| format!("malformed main claim marker `{line}`"))?;
            let target = rest
                .strip_suffix(").**")
                .ok_or_else(|| format!("malformed main claim target `{line}`"))?;
            (
                label,
                LocalLink {
                    target: target.to_owned(),
                    native_wiki: false,
                },
            )
        } else {
            index += 1;
            continue;
        };
        let (label, target) = marker;
        let (class_label, id) = label
            .split_once(" - ")
            .ok_or_else(|| format!("malformed main claim label `{label}`"))?;
        let class = EvidenceClass::parse(class_label)?;
        let mut body = Vec::new();
        for body_line in lines.map(str::trim).filter(|line| !line.is_empty()) {
            if body_line.starts_with("**[") {
                return Err(format!(
                    "claim {id} is not separated from the next claim block"
                ));
            }
            body.push(body_line.to_owned());
        }
        if body.is_empty()
            && blocks
                .get(index + 1)
                .is_some_and(|next| next.kind == MarkdownBlockKind::List)
        {
            let list = &blocks[index + 1];
            if !text[block.range.end..list.range.start].trim().is_empty() {
                return Err(format!(
                    "claim {id} bullet list must immediately follow its label"
                ));
            }
            body.push(visible_markdown_text(&text[list.range.clone()]));
            index += 1;
        }
        if body.is_empty() {
            return Err(format!("main claim {id} has no body"));
        }
        claims.push(MainClaim {
            class,
            class_label: class_label.to_owned(),
            id: id.to_owned(),
            target,
            body: body.join(" "),
        });
        index += 1;
    }
    Ok(claims)
}

fn visible_markdown_text(text: &str) -> String {
    let mut output = String::new();
    for event in Parser::new_ext(text, markdown_options()) {
        match event {
            Event::Text(value) | Event::Code(value) => {
                output.push_str(&value);
                output.push(' ');
            }
            Event::SoftBreak | Event::HardBreak | Event::End(TagEnd::Item) => output.push(' '),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn validate_main_document(source: &Path, text: &str) -> Result<Vec<MainClaim>, String> {
    let claims = parse_main_claims(text)?;
    if claims.is_empty() {
        return Err("main document has no claim blocks".to_owned());
    }

    for claim in &claims {
        let (relative, anchor) =
            claim.target.target.split_once('#').ok_or_else(|| {
                format!("main claim {} does not target a ledger heading", claim.id)
            })?;
        let ledger_path = if claim.target.native_wiki {
            let resolution = harp::knowledge::resolve_wiki_links(
                &workspace_root(),
                source,
                &format!("[[{}]]", claim.target.target),
            )
            .map_err(|error| error.to_string())?
            .into_iter()
            .next()
            .expect("one native claim marker");
            if resolution.requested_heading.as_deref() != Some(anchor) {
                return Err(format!(
                    "main claim {} does not preserve its native heading target",
                    claim.id
                ));
            }
            workspace_root().join(resolution.target)
        } else {
            resolve_link(source, &claim.target.target)?;
            repository_relative_path(source, relative)?
        };
        let ledger_text = fs::read_to_string(&ledger_path)
            .map_err(|error| format!("read {}: {error}", ledger_path.display()))?;
        let entries = validate_claim_entries_at(&ledger_path, &ledger_text)?;
        let entry = entries
            .iter()
            .find(|entry| {
                entry.slug == markdown_slug(anchor)
                    && (!claim.target.native_wiki
                        || entry.id.eq_ignore_ascii_case(
                            anchor.split_once(':').map_or(anchor, |(id, _)| id),
                        ))
            })
            .ok_or_else(|| format!("main claim {} targets missing ledger entry", claim.id))?;
        if claim.id != entry.id {
            return Err(format!(
                "main claim ID {} targets ledger claim {}",
                claim.id, entry.id
            ));
        }
        if claim.class != entry.class()? {
            return Err(format!(
                "main claim {} class {} does not match its ledger entry",
                claim.id, claim.class_label
            ));
        }
        let lower = claim.body.to_ascii_lowercase();
        let needs_inline_reproduction = claim.body.bytes().any(|byte| byte.is_ascii_digit())
            || lower.contains("benchmark")
            || lower.contains("safety-critical");
        if needs_inline_reproduction && !lower.contains("reproduc") {
            return Err(format!(
                "main claim {} requires inline reproduction status",
                claim.id
            ));
        }
    }
    Ok(claims)
}

fn validate_source_registry(text: &str) -> Result<(), String> {
    let entries = claim_entries(text)?;
    if entries.is_empty() {
        return Err("source registry has no entries".to_owned());
    }
    for entry in entries {
        for field in [
            "Class",
            "Artifact",
            "Stability",
            "Semantic locators",
            "Can prove",
            "Cannot prove",
        ] {
            entry.field(field)?;
        }
        if markdown_links(entry.field("Artifact")?).is_empty() {
            return Err(format!("{} artifact must be clickable", entry.id));
        }
        match entry.field("Stability")?.trim().trim_matches('`') {
            "pinned" => {
                entry.field("Immutable identity")?;
                if entry.fields.contains_key("Observed") {
                    return Err(format!("{} is pinned but declares Observed", entry.id));
                }
            }
            "dated observation" => {
                let observed = entry.field("Observed")?.trim().trim_matches('`');
                if !is_valid_date(observed) {
                    return Err(format!(
                        "{} has invalid observation date `{observed}`",
                        entry.id
                    ));
                }
                if entry.fields.contains_key("Immutable identity") {
                    return Err(format!(
                        "{} is a dated observation but declares Immutable identity",
                        entry.id
                    ));
                }
            }
            other => return Err(format!("{} has invalid stability `{other}`", entry.id)),
        }
    }
    Ok(())
}

#[test]
fn credible_documentation_style_has_complete_clickable_contract() {
    let root = style_root();
    let mut files = fs::read_dir(&root)
        .expect("read writing-style directory")
        .map(|entry| entry.expect("read style entry").path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    files.sort();
    let names = files
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("UTF-8 style filename")
                .to_owned()
        })
        .collect::<BTreeSet<_>>();
    let expected = EXPECTED_FILES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(names, expected, "writing-style file inventory drifted");

    for path in files {
        let text = fs::read_to_string(&path).expect("read style document");
        for target in markdown_links(&text) {
            resolve_link(&path, &target).unwrap_or_else(|error| panic!("{error}"));
        }
        assert!(
            !text.ends_with("\n\n"),
            "{} has an extra blank line at EOF",
            path.display()
        );
    }

    let guide = fs::read_to_string(root.join("STYLE_GUIDE.md")).expect("read style guide");
    for required in [
        "EVIDENCE",
        "SOURCE CLAIM",
        "INFERENCE",
        "MISSING",
        "main document",
        "claim ledger",
        "source registry",
        "reproduction status",
        "claim block",
    ] {
        assert!(
            guide.contains(required),
            "style guide is missing {required}"
        );
    }
    for deprecated in ["**[CLAIM -", "- Class: `CLAIM`"] {
        assert!(
            !guide.contains(deprecated),
            "style guide uses the deprecated bare CLAIM class as {deprecated}"
        );
    }

    let ledger_path = root.join("claim-ledger-template.md");
    let ledger = fs::read_to_string(&ledger_path).expect("read ledger template");
    let entries = validate_claim_entries(&ledger).unwrap_or_else(|error| panic!("{error}"));
    for id in ["EX-001", "EX-002", "EX-003", "EX-004", "EX-005", "EX-006"] {
        assert!(
            entries.iter().any(|entry| entry.id == id),
            "ledger template is missing {id}"
        );
    }
    assert!(
        entries.iter().any(|entry| {
            entry
                .fields
                .get("Mode")
                .and_then(|values| values.first())
                .map(String::as_str)
                == Some("`quote`")
        }),
        "ledger template needs an explicit quote example"
    );
    assert!(
        entries.iter().any(|entry| {
            entry.class() == Ok(EvidenceClass::Evidence)
                && markdown_links(entry.field("Source").expect("validated source")).len() >= 2
        }),
        "ledger template needs a valid multi-source EVIDENCE example"
    );

    let registry = fs::read_to_string(root.join("source-registry-template.md"))
        .expect("read source registry template");
    validate_source_registry(&registry).unwrap_or_else(|error| panic!("{error}"));

    let main_path = root.join("main-document-template.md");
    let main = fs::read_to_string(&main_path).expect("read main template");
    validate_main_document(&main_path, &main).unwrap_or_else(|error| panic!("{error}"));

    let worked_path = root.join("worked-example.md");
    let worked = fs::read_to_string(&worked_path).expect("read worked example");
    let claims =
        validate_main_document(&worked_path, &worked).unwrap_or_else(|error| panic!("{error}"));
    for id in ["EX-101", "EX-102", "EX-103", "EX-104"] {
        assert!(
            claims.iter().any(|claim| claim.id == id),
            "worked example is missing main claim {id}"
        );
    }
    assert!(
        worked.contains("shown together only to make the teaching route"),
        "worked example must explain why prose and ledger are colocated"
    );

    let contributing = fs::read_to_string(workspace_root().join("docs/contributing.md"))
        .expect("read contributing guide");
    assert!(contributing.contains("writing-style/STYLE_GUIDE.md"));
}

#[test]
fn duplicate_claim_ids_are_not_silently_overwritten() {
    let ledger = "\
## EX-001: First claim

- Class: `EVIDENCE`

## EX-001: Duplicate claim

- Class: `SOURCE CLAIM`
";

    let error = claim_entries(ledger).expect_err("duplicate IDs must fail");
    assert!(error.contains("duplicate claim ID EX-001"), "{error}");
}

#[test]
fn local_links_cannot_escape_the_repository() {
    let source = style_root().join("STYLE_GUIDE.md");
    assert!(
        resolve_link(&source, "../../../../../../etc/hosts").is_err(),
        "a local link that resolves outside the repository must be rejected"
    );
}

fn valid_claim_entry(class: &str, extra: &str) -> String {
    format!(
        "\
## EX-900: Fixture claim

- Class: `{class}`
- Statement: Fixture statement.
- Source: [Registry](source-registry-template.md#example-paper)
- Locator: [Paper](../../evidence/weng/text/dgm.txt)
- Scope: Fixture scope.
- Reproduction: Not independently reproduced.
- Confidence: `high`
- Confidence basis: Fixture basis.
- Caveat: Fixture caveat.
{extra}"
    )
}

#[test]
fn malformed_claim_entries_are_rejected_structurally() {
    let bad_class = valid_claim_entry(
        "CLAIM",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    );
    assert!(validate_claim_entries(&bad_class)
        .expect_err("bad class must fail")
        .contains("invalid evidence class"));

    let bad_mode = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `dated observation`\n- Source stability: `pinned`\n",
    );
    assert!(validate_claim_entries(&bad_mode)
        .expect_err("bad mode must fail")
        .contains("invalid rendering mode"));

    let incomplete_inference = valid_claim_entry("INFERENCE", "- Weakens if: Fixture weakness.\n");
    assert!(validate_claim_entries(&incomplete_inference)
        .expect_err("inference without falsifier must fail")
        .contains("Falsified by"));

    let unlinked_locator = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "- Locator: [Paper](../../evidence/weng/text/dgm.txt)",
        "- Locator: Figure 4",
    );
    assert!(validate_claim_entries(&unlinked_locator)
        .expect_err("plain-text locator must fail")
        .contains("Locator must contain a link"));

    let one_sided_contradiction = format!(
        "{}\n\n{}",
        valid_claim_entry(
            "EVIDENCE",
            "- Mode: `paraphrase`\n- Source stability: `pinned`\n- Relationship: `unresolved-with EX-901`\n",
        ),
        valid_claim_entry(
            "EVIDENCE",
            "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
        )
        .replacen("EX-900", "EX-901", 1)
    );
    assert!(validate_claim_entries(&one_sided_contradiction)
        .expect_err("one-sided contradiction must fail")
        .contains("is not reciprocal"));
}

#[test]
fn main_claims_must_match_ledger_class_and_id() {
    let root = style_root();
    let path = root.join("main-document-template.md");
    let valid = fs::read_to_string(&path).expect("read main template");

    let native = "\
**[[claim-ledger-template#EX-002: Authors report the benchmark result|SOURCE CLAIM - EX-002]].**
The authors report 50.0%; this result has not been independently reproduced.
";
    validate_main_document(&path, native).expect("native wiki claim marker must validate");

    let wrong_native_heading = "\
**[[claim-ledger-template#EX-002: Wrong title|SOURCE CLAIM - EX-002]].**
Body.
";
    let error = validate_main_document(&path, wrong_native_heading)
        .expect_err("native claim must resolve its exact ledger heading");
    assert!(
        error.contains("Obsidian wikilink heading is missing"),
        "{error}"
    );

    let missing_native_ledger = "\
**[[knowledge/example/missing_ledger#EX-002: Authors report the benchmark result|SOURCE CLAIM - EX-002]].**
Body.
";
    let error = validate_main_document(&path, missing_native_ledger)
        .expect_err("native claim must resolve its ledger target");
    assert!(
        error.contains("Obsidian wikilink target is missing"),
        "{error}"
    );

    let bad_class = valid.replacen("[EVIDENCE - EX-001]", "[SOURCE CLAIM - EX-001]", 1);
    assert!(validate_main_document(&path, &bad_class)
        .expect_err("class mismatch must fail")
        .contains("does not match its ledger entry"));

    let bad_id = valid.replacen("[EVIDENCE - EX-001]", "[EVIDENCE - EX-999]", 1);
    assert!(validate_main_document(&path, &bad_id)
        .expect_err("ID mismatch must fail")
        .contains("main claim ID EX-999 targets ledger claim EX-001"));

    let missing_reproduction =
        "**[SOURCE CLAIM - EX-002](claim-ledger-template.md#ex-002-authors-report-the-benchmark-result).**\nThe authors report 50.0%.\n";
    assert!(validate_main_document(&path, missing_reproduction)
        .expect_err("quantitative claim without reproduction status must fail")
        .contains("requires inline reproduction status"));

    let task_count_without_reproduction =
        "**[SOURCE CLAIM - EX-002](claim-ledger-template.md#ex-002-authors-report-the-benchmark-result).**\nThe authors report results on 200 tasks.\n";
    assert!(
        validate_main_document(&path, task_count_without_reproduction)
            .expect_err("numeric claim without reproduction status must fail")
            .contains("requires inline reproduction status")
    );
}

#[test]
fn fenced_examples_do_not_define_headings_claims_or_claim_blocks() {
    let ledger = format!(
        "\
# Fixture

```markdown
{}
```

{}
",
        valid_claim_entry(
            "EVIDENCE",
            "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
        ),
        valid_claim_entry(
            "EVIDENCE",
            "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
        )
        .replacen("EX-900", "EX-901", 1)
    );
    let entries = claim_entries(&ledger).expect("parse rendered claim entries");
    assert_eq!(entries.len(), 1, "fenced claim headings must be ignored");
    assert_eq!(entries[0].id, "EX-901");
    assert!(
        !headings(&ledger).contains("ex-900-fixture-claim"),
        "fenced headings must not become link targets"
    );

    let main = "\
```markdown
**[EVIDENCE - EX-900](claim-ledger-template.md#ex-900-fixture-claim).**
Fenced example body.
```

**[EVIDENCE - EX-901](claim-ledger-template.md#ex-901-fixture-claim).**
Rendered claim body.
";
    let claims = parse_main_claims(main).expect("parse rendered main claims");
    assert_eq!(claims.len(), 1, "fenced claim markers must be ignored");
    assert_eq!(claims[0].id, "EX-901");
}

#[test]
fn locators_must_target_underlying_evidence() {
    let prose_locator = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "- Locator: [Paper](../../evidence/weng/text/dgm.txt)",
        "- Locator: [Prose summary](../../rsi/systems/dgm.md#claim-ceiling)",
    );
    assert!(validate_claim_entries(&prose_locator)
        .expect_err("canonical prose cannot substitute for evidence")
        .contains("must target a file under evidence/"));
}

#[test]
fn immutable_evidence_locators_require_a_markdown_line_anchor() {
    let wiki_only = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "[Paper](../../evidence/weng/text/dgm.txt)",
        "[[evidence/weng/text/dgm.txt|MCE extracted text]]",
    );
    assert!(validate_claim_entries(&wiki_only)
        .expect_err("wiki navigation alone must not satisfy an immutable locator")
        .contains("conventional Markdown line locator"));

    let dual_link = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "[Paper](../../evidence/weng/text/dgm.txt)",
        "[[evidence/weng/text/dgm.txt|MCE extracted text]] ([exact line 1](../../evidence/weng/text/dgm.txt#L1))",
    );
    validate_claim_entries(&dual_link)
        .expect("native navigation plus a Markdown line locator must validate");
}

#[test]
fn native_evidence_locators_require_an_evidence_target_and_matching_line_target() {
    let native_knowledge_target = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "[Paper](../../evidence/weng/text/dgm.txt)",
        "[[knowledge/rsi/systems/dgm|DGM]] ([exact line 1](../../evidence/weng/text/dgm.txt#L1))",
    );
    assert!(validate_claim_entries(&native_knowledge_target)
        .expect_err("native locator cannot target maintained knowledge")
        .contains("native locator must target a file under evidence/"));

    let mismatched_raw_text = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "[Paper](../../evidence/weng/text/dgm.txt)",
        "[[evidence/weng/text/dgm.txt|MCE extracted text]] ([exact line 1](../../evidence/weng/references/dgm.txt#L1))",
    );
    assert!(validate_claim_entries(&mismatched_raw_text)
        .expect_err("raw native locator must retain a line anchor for the same target")
        .contains("matching conventional Markdown line locator"));
}

#[test]
fn relationships_support_multiple_targets_and_reject_invalid_edges() {
    let entry = |id: &str, relationships: &str| {
        valid_claim_entry(
            "EVIDENCE",
            &format!("- Mode: `paraphrase`\n- Source stability: `pinned`\n{relationships}"),
        )
        .replacen("EX-900", id, 1)
    };
    let valid = [
        entry(
            "EX-900",
            "- Relationship: `unresolved-with EX-901`\n- Relationship: `unresolved-with EX-902`\n",
        ),
        entry("EX-901", "- Relationship: `unresolved-with EX-900`\n"),
        entry("EX-902", "- Relationship: `unresolved-with EX-900`\n"),
    ]
    .join("\n\n");
    validate_claim_entries(&valid).expect("multiple reciprocal relationships are valid");

    let self_link = entry("EX-900", "- Relationship: `unresolved-with EX-900`\n");
    assert!(validate_claim_entries(&self_link)
        .expect_err("self relationship must fail")
        .contains("must not target itself"));

    let missing_target = entry("EX-900", "- Relationship: `narrows EX-999`\n");
    assert!(validate_claim_entries(&missing_target)
        .expect_err("missing relationship target must fail")
        .contains("targets missing claim EX-999"));
}

#[test]
fn required_values_and_observation_dates_are_validated() {
    let empty_scope = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace("- Scope: Fixture scope.", "- Scope:");
    assert!(validate_claim_entries(&empty_scope)
        .expect_err("empty required value must fail")
        .contains("has empty field Scope"));

    let bad_date = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `dated observation`\n- Observed: `2026-02-30`\n",
    );
    assert!(validate_claim_entries(&bad_date)
        .expect_err("invalid calendar date must fail")
        .contains("invalid observation date"));

    let registry = "\
## SOURCE-1: Mutable source

- Class: `mutable page`
- Artifact: [Capture](../../evidence/weng/metadata/dgm-arxiv.html)
- Stability: `dated observation`
- Observed: `2026-13-01`
- Semantic locators: Description.
- Can prove: Observed content.
- Cannot prove: Later content.
";
    assert!(validate_source_registry(registry)
        .expect_err("invalid registry date must fail")
        .contains("invalid observation date"));
}

#[test]
fn ledger_entries_reject_unowned_visible_content() {
    let ledger = format!(
        "{}\n\nThis paragraph is not owned by a ledger field.\n",
        valid_claim_entry(
            "EVIDENCE",
            "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
        )
    );
    let error = validate_claim_entries(&ledger).expect_err("unowned ledger prose must fail");
    assert!(error.contains("content outside the field list"), "{error}");
}

#[test]
fn main_claims_support_bullet_groups_without_bypassing_validation() {
    let path = style_root().join("main-document-template.md");
    let valid = "\
**[EVIDENCE - EX-001](claim-ledger-template.md#ex-001-released-selector-reads-the-shallow-score).**
- The selector reads `overall_performance`.
- The inspected source was not executed.
";
    let claims = validate_main_document(&path, valid).expect("bullet-group claim is valid");
    assert_eq!(claims.len(), 1);
    assert!(claims[0].body.contains("The selector reads"));

    let missing_reproduction = "\
**[SOURCE CLAIM - EX-002](claim-ledger-template.md#ex-002-authors-report-the-benchmark-result).**
- The authors report results on 200 tasks.
- The benchmark result is 50.0%.
";
    assert!(validate_main_document(&path, missing_reproduction)
        .expect_err("numeric bullet-group claim requires reproduction status")
        .contains("requires inline reproduction status"));

    let separated_list = "\
**[EVIDENCE - EX-001](claim-ledger-template.md#ex-001-released-selector-reads-the-shallow-score).**

> An intervening block breaks the claim block.

- The selector reads `overall_performance`.
";
    assert!(validate_main_document(&path, separated_list)
        .expect_err("bullet list must be adjacent to its claim label")
        .contains("must immediately follow its label"));
}

#[cfg(unix)]
#[test]
fn local_links_cannot_escape_through_symlinks() {
    use std::os::unix::fs::symlink;

    let outside = tempfile::tempdir().expect("create outside temp directory");
    let outside_file = outside.path().join("source.txt");
    fs::write(&outside_file, "outside").expect("write outside source");
    let inside = tempfile::tempdir_in(style_root()).expect("create repository temp directory");
    let source = inside.path().join("source.md");
    fs::write(&source, "# Source\n").expect("write repository source");
    symlink(&outside_file, inside.path().join("escape")).expect("create escape symlink");

    assert!(resolve_link(&source, "escape").is_err());
}

#[cfg(unix)]
#[test]
fn evidence_locators_cannot_escape_evidence_through_symlinks() {
    use std::os::unix::fs::symlink;

    let evidence = workspace_root().join("evidence");
    let inside = tempfile::tempdir_in(&evidence).expect("create evidence temp directory");
    let link = inside.path().join("escape.md");
    symlink(workspace_root().join("knowledge/rsi/systems/dgm.md"), &link)
        .expect("create evidence escape symlink");
    let relative = link
        .strip_prefix(&evidence)
        .expect("temporary link is under evidence")
        .to_string_lossy();
    let ledger = valid_claim_entry(
        "EVIDENCE",
        "- Mode: `paraphrase`\n- Source stability: `pinned`\n",
    )
    .replace(
        "../../evidence/weng/text/dgm.txt",
        &format!("../../evidence/{relative}"),
    );

    assert!(validate_claim_entries(&ledger)
        .expect_err("evidence symlink escape must fail")
        .contains("must resolve beneath evidence/"));
}
