use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

const DARWINX_FILES: [&str; 9] = [
    "darwinx_index.md",
    "01_mechanism_and_selection.md",
    "02_evaluation_audit.md",
    "03_critical_review.md",
    "04_comparative_synthesis.md",
    "05_successor_experiment.md",
    "claim_evidence_ledger.md",
    "source_registry.md",
    "maintenance.md",
];

const REQUIRED_FRONTMATTER: [&str; 8] = [
    "id",
    "title",
    "type",
    "status",
    "created",
    "updated",
    "tags",
    "confidence",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
        .to_path_buf()
}

fn packet_root() -> PathBuf {
    workspace_root().join("knowledge/darwinx")
}

fn canonical_system_ids() -> BTreeSet<String> {
    serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(workspace_root().join("content/systems/system_readings.json"))
            .expect("read canonical RSI system registry"),
    )
    .expect("parse canonical RSI system registry")["systems"]
        .as_array()
        .expect("canonical RSI system list")
        .iter()
        .map(|system| {
            system["system_id"]
                .as_str()
                .expect("canonical RSI system ID")
                .to_owned()
        })
        .collect()
}

#[test]
fn public_wiki_resolver_rejects_a_repository_root_source_path() {
    let root = workspace_root();
    let error = harp::knowledge::resolve_wiki_links(&root, &root, "[[note]]")
        .expect_err("repository root cannot be a wiki-link source path");
    assert_eq!(error.code(), "knowledge.obsidian.source_path");
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH
        | Options::ENABLE_HEADING_ATTRIBUTES
}

fn slug(value: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in value.to_ascii_lowercase().chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(character);
            separator = false;
        } else if character.is_ascii_whitespace() || character == '-' {
            separator = true;
        }
    }
    output
}

fn headings(text: &str) -> BTreeSet<String> {
    let mut headings = BTreeSet::new();
    let mut current: Option<(HeadingLevel, Option<String>, String)> = None;
    for event in Parser::new_ext(text, markdown_options()) {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                current = Some((level, id.map(|value| value.into_string()), String::new()));
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
            Event::End(TagEnd::Heading(_)) => {
                let (_, explicit, text) = current.take().expect("heading start precedes end");
                headings.insert(explicit.unwrap_or_else(|| slug(text.trim())));
            }
            _ => {}
        }
    }
    headings
}

fn markdown_links(text: &str) -> Vec<String> {
    Parser::new_ext(text, markdown_options())
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. })
            | Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.into_string()),
            _ => None,
        })
        .collect()
}

fn frontmatter(text: &str) -> Result<BTreeMap<&str, &str>, String> {
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| "document has no YAML frontmatter".to_owned())?;
    let (frontmatter, _) = rest
        .split_once("\n---\n")
        .ok_or_else(|| "document has unclosed YAML frontmatter".to_owned())?;
    let mut fields = BTreeMap::new();
    for line in frontmatter.lines() {
        if line.is_empty() || line.starts_with([' ', '\t']) {
            return Err("frontmatter must use top-level one-line values".to_owned());
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("frontmatter line has no colon: {line}"))?;
        if key.trim().is_empty()
            || value.trim().is_empty()
            || fields.insert(key.trim(), value.trim()).is_some()
        {
            return Err(format!("invalid frontmatter line: {line}"));
        }
    }
    for field in REQUIRED_FRONTMATTER {
        if !fields.contains_key(field) {
            return Err(format!("frontmatter is missing {field}"));
        }
    }
    if !matches!(fields["confidence"], "low" | "medium" | "high")
        || !fields["tags"].starts_with('[')
        || !fields["tags"].ends_with(']')
    {
        return Err("frontmatter values violate the packet contract".to_owned());
    }
    Ok(fields)
}

fn resolve_markdown_link(source: &Path, target: &str) -> Result<(), String> {
    if target.starts_with('#')
        || target.starts_with("https://")
        || target.starts_with("http://")
        || target.starts_with("mailto:")
    {
        return Ok(());
    }
    let (path, anchor) = target
        .split_once('#')
        .map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let root = workspace_root();
    let mut resolved = source
        .parent()
        .expect("packet document has parent")
        .to_path_buf();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !resolved.pop() || !resolved.starts_with(&root) {
                    return Err(format!("Markdown link escapes repository: {target}"));
                }
            }
            Component::Normal(part) => resolved.push(part),
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("absolute Markdown link: {target}"));
            }
        }
    }
    if !resolved.is_file() {
        return Err(format!("unresolved Markdown link {target}"));
    }
    let canonical_root = fs::canonicalize(&root).map_err(|error| error.to_string())?;
    let canonical_target = fs::canonicalize(&resolved).map_err(|error| error.to_string())?;
    if !canonical_target.starts_with(canonical_root) {
        return Err(format!("Markdown link escapes through symlink: {target}"));
    }
    if let Some(anchor) = anchor {
        if let Some(line) = anchor.strip_prefix('L') {
            let line = line
                .parse::<usize>()
                .map_err(|_| format!("invalid line anchor {anchor}"))?;
            if line == 0
                || line
                    > fs::read_to_string(&resolved)
                        .map_err(|error| error.to_string())?
                        .lines()
                        .count()
            {
                return Err(format!("missing line anchor {target}"));
            }
        } else if resolved
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("md")
            && !headings(&fs::read_to_string(&resolved).map_err(|error| error.to_string())?)
                .contains(anchor)
        {
            return Err(format!("missing Markdown heading {target}"));
        }
    }
    Ok(())
}

fn validate_wiki_links(source: &Path, text: &str) -> Result<(), String> {
    harp::knowledge::resolve_wiki_links(&workspace_root(), source, text)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn ledger_claims(text: &str) -> Result<BTreeMap<String, (String, ClaimClass)>, String> {
    let mut claims = BTreeMap::new();
    for heading in text.match_indices("\n## DX-") {
        let heading_start = heading.0 + 1;
        let section = &text[heading_start..];
        let heading_line = section
            .lines()
            .next()
            .expect("matched DarwinX ledger heading");
        let Some(heading) = heading_line.strip_prefix("## DX-") else {
            continue;
        };
        let (suffix, title) = heading
            .split_once(':')
            .ok_or_else(|| format!("malformed DarwinX ledger heading {heading_line}"))?;
        let id = format!("DX-{suffix}");
        let section = section.split("\n## ").next().expect("ledger section");
        let class = section
            .lines()
            .find_map(|line| line.trim().strip_prefix("- Class: `"))
            .and_then(|class| class.strip_suffix('`'))
            .ok_or_else(|| format!("{id} ledger entry has no class"))?;
        claims.insert(
            slug(&format!("{id}: {title}")),
            (id, parse_claim_class(class)?),
        );
    }
    Ok(claims)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClaimClass {
    Evidence,
    SourceClaim,
    Inference,
    Missing,
}

fn parse_claim_class(value: &str) -> Result<ClaimClass, String> {
    match value {
        "EVIDENCE" => Ok(ClaimClass::Evidence),
        "SOURCE CLAIM" => Ok(ClaimClass::SourceClaim),
        "INFERENCE" => Ok(ClaimClass::Inference),
        "MISSING" => Ok(ClaimClass::Missing),
        _ => Err(format!("invalid reader claim class {value}")),
    }
}

#[derive(Debug)]
struct ReaderClaim {
    class: ClaimClass,
    id: String,
    target: String,
    native_wiki: bool,
}

fn reader_claims(text: &str) -> Result<Vec<ReaderClaim>, String> {
    let mut claims = Vec::new();
    for line in text.lines().map(str::trim) {
        let (label, target, native_wiki) = if let Some(marker) = line.strip_prefix("**[[") {
            let (target, label) = marker
                .strip_suffix("]].**")
                .ok_or_else(|| format!("malformed native reader claim marker {line}"))?
                .split_once('|')
                .ok_or_else(|| format!("native reader claim lacks a label {line}"))?;
            (label, target, true)
        } else if let Some(marker) = line.strip_prefix("**[") {
            let (label, target) = marker
                .strip_suffix(").**")
                .ok_or_else(|| format!("malformed Markdown reader claim marker {line}"))?
                .split_once("](")
                .ok_or_else(|| format!("Markdown reader claim lacks a target {line}"))?;
            (label, target, false)
        } else {
            continue;
        };
        let (class, id) = label
            .split_once(" - ")
            .ok_or_else(|| format!("reader claim label is malformed {label}"))?;
        claims.push(ReaderClaim {
            class: parse_claim_class(class)?,
            id: id.to_owned(),
            target: target.to_owned(),
            native_wiki,
        });
    }
    Ok(claims)
}

fn validate_reader_claims(
    source: &Path,
    text: &str,
    ledger: &BTreeMap<String, (String, ClaimClass)>,
) -> Result<(), String> {
    for claim in reader_claims(text)? {
        let (target, heading) = claim
            .target
            .split_once('#')
            .ok_or_else(|| format!("{} reader claim lacks a ledger heading", claim.id))?;
        let (ledger_id, ledger_class) = ledger
            .get(&slug(heading))
            .ok_or_else(|| format!("{} reader claim targets a missing ledger entry", claim.id))?;
        if claim.id != *ledger_id {
            return Err(format!("{} reader claim targets {ledger_id}", claim.id));
        }
        if claim.class != *ledger_class {
            return Err(format!(
                "{} reader claim class differs from its ledger",
                claim.id
            ));
        }
        if claim.native_wiki {
            let resolution = harp::knowledge::resolve_wiki_links(
                &workspace_root(),
                source,
                &format!("[[{}]]", claim.target),
            )
            .map_err(|error| error.to_string())?
            .into_iter()
            .next()
            .expect("one native reader claim");
            if resolution.target != Path::new("knowledge/darwinx/claim_evidence_ledger.md")
                || resolution.requested_heading.as_deref() != Some(heading)
            {
                return Err(format!(
                    "{} native reader claim misses its ledger heading",
                    claim.id
                ));
            }
        } else {
            resolve_markdown_link(source, &claim.target)?;
            if target != "claim_evidence_ledger.md" {
                return Err(format!("{} reader claim bypasses the ledger", claim.id));
            }
        }
    }
    Ok(())
}

#[test]
fn darwinx_packet_is_complete_searchable_and_atlas_routable() {
    let root = packet_root();
    let expected = DARWINX_FILES
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let actual = fs::read_dir(&root)
        .expect("read DarwinX packet")
        .map(|entry| {
            entry
                .expect("read DarwinX packet entry")
                .file_name()
                .into_string()
                .expect("UTF-8 packet filename")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "DarwinX packet file roster drifted");

    let ledger_path = root.join("claim_evidence_ledger.md");
    let ledger = fs::read_to_string(&ledger_path).expect("read DarwinX claim ledger");
    let claims = ledger_claims(&ledger).expect("parse DarwinX ledger claims");
    let reader_files = [
        "darwinx_index.md",
        "01_mechanism_and_selection.md",
        "02_evaluation_audit.md",
        "03_critical_review.md",
        "04_comparative_synthesis.md",
        "05_successor_experiment.md",
    ];
    for name in DARWINX_FILES {
        let path = root.join(name);
        let text = fs::read_to_string(&path).expect("read DarwinX packet document");
        frontmatter(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        for target in markdown_links(&text) {
            resolve_markdown_link(&path, &target).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        validate_wiki_links(&path, &text).unwrap_or_else(|error| panic!("{name}: {error}"));
        if reader_files.contains(&name) {
            validate_reader_claims(&path, &text, &claims)
                .unwrap_or_else(|error| panic!("{name}: {error}"));
        }
    }

    let index = fs::read_to_string(root.join("darwinx_index.md")).expect("read DarwinX index");
    for name in DARWINX_FILES {
        if name != "darwinx_index.md" {
            let stem = name.strip_suffix(".md").expect("DarwinX file is Markdown");
            assert!(
                index.contains(&format!("[[knowledge/darwinx/{stem}|")),
                "DarwinX index does not link {name}"
            );
        }
    }

    assert_eq!(
        reader_files
            .into_iter()
            .flat_map(|name| {
                reader_claims(
                    &fs::read_to_string(root.join(name)).expect("read DarwinX reader document"),
                )
                .expect("parse DarwinX reader claims")
            })
            .map(|claim| claim.id)
            .collect::<BTreeSet<_>>(),
        claims
            .values()
            .map(|(id, _)| id.clone())
            .collect::<BTreeSet<_>>(),
        "every DarwinX claim needs a reader-facing route"
    );

    let systems = canonical_system_ids();
    assert_eq!(systems.len(), 17, "canonical RSI system roster drifted");
    assert!(
        systems.contains("dgm")
            && systems.contains("meta-harness")
            && systems.contains("envharness"),
        "canonical RSI system roster lost known system identities"
    );
}
