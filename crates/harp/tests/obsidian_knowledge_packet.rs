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
    let resolved = source
        .parent()
        .expect("packet document has parent")
        .join(path);
    if !resolved.is_file() {
        return Err(format!("unresolved Markdown link {target}"));
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

fn resolve_wiki_link(source: &Path, target: &str) -> Result<(), String> {
    let (path, heading) = target
        .split_once('#')
        .map_or((target, None), |(path, heading)| (path, Some(heading)));
    let candidate = Path::new(path);
    if path.is_empty()
        || candidate.is_absolute()
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
            .expect("packet document has parent")
            .join(candidate)
    };
    let resolved = if resolved.extension().is_some() {
        resolved
    } else {
        resolved.with_extension("md")
    };
    if !resolved.is_file() {
        return Err(format!("unresolved native wiki link {target}"));
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
            || !headings(&fs::read_to_string(&resolved).map_err(|error| error.to_string())?)
                .contains(&slug(heading))
        {
            return Err(format!("missing native wiki heading {target}"));
        }
    }
    Ok(())
}

fn claim_ids(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("## DX-"))
        .filter_map(|line| line.split_once(':').map(|(id, _)| format!("DX-{id}")))
        .collect()
}

fn reader_claim_ids(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for marker in ["**[", "**[["] {
        for fragment in text.split(marker).skip(1) {
            let label = fragment
                .split_once("](")
                .map(|(label, _)| label)
                .or_else(|| fragment.split_once('|').map(|(_, label)| label));
            if let Some((_, id)) = label.and_then(|label| label.split_once(" - ")) {
                if id.starts_with("DX-") {
                    ids.insert(id.trim_end_matches("]].**").to_owned());
                }
            }
        }
    }
    ids
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

    let mut reader_claims = BTreeSet::new();
    for name in DARWINX_FILES {
        let path = root.join(name);
        let text = fs::read_to_string(&path).expect("read DarwinX packet document");
        frontmatter(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        for target in markdown_links(&text) {
            resolve_markdown_link(&path, &target).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        for target in wiki_links(&text).unwrap_or_else(|error| panic!("{name}: {error}")) {
            resolve_wiki_link(&path, &target).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        if name != "claim_evidence_ledger.md" {
            reader_claims.extend(reader_claim_ids(&text));
        }
    }

    let index = fs::read_to_string(root.join("darwinx_index.md")).expect("read DarwinX index");
    for name in DARWINX_FILES {
        if name != "darwinx_index.md" {
            assert!(index.contains(name), "DarwinX index does not link {name}");
        }
    }

    let ledger = fs::read_to_string(root.join("claim_evidence_ledger.md"))
        .expect("read DarwinX claim ledger");
    assert_eq!(
        reader_claims,
        claim_ids(&ledger),
        "every DarwinX claim needs a reader-facing route"
    );

    let corpus = fs::read_to_string(workspace_root().join("crates/harp/src/corpus/mod.rs"))
        .expect("read corpus registry");
    let search = fs::read_to_string(workspace_root().join("crates/harp/src/search.rs"))
        .expect("read search roots");
    for name in DARWINX_FILES {
        assert!(
            !corpus.contains(&format!("knowledge/darwinx/{name}")),
            "Task 7 must register DarwinX document {name}"
        );
    }
    assert!(
        !search.contains("\"knowledge/darwinx\""),
        "Task 7 must register DarwinX in search"
    );
    assert!(
        !corpus.contains("\"darwinx\""),
        "DarwinX must remain an auxiliary packet, not a canonical RSI system"
    );
}
