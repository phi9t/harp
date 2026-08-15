use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

const EXPECTED_FILES: [&str; 13] = [
    "01_problem_and_prior_barrier.md",
    "02_shared_power_family.md",
    "03_jin_proof_spine.md",
    "04_jin_positive_real_completion.md",
    "05_lorist_schwenninger_proof.md",
    "06_proof_comparison.md",
    "07_jin_lean_verification.md",
    "08_ai_assisted_discovery.md",
    "09_status_and_critical_assessment.md",
    "claim_evidence_ledger.md",
    "crouzeix_conjecture_index.md",
    "glossary.md",
    "source_registry.md",
];

const REQUIRED_CLAIMS: [&str; 34] = [
    "CC-001", "CC-002", "CC-003", "CC-004", "CC-005", "CC-010", "CC-011", "CC-012", "CC-013",
    "CC-014", "CC-015", "CC-016", "CC-017", "CC-020", "CC-021", "CC-022", "CC-023", "CC-024",
    "CC-025", "CC-030", "CC-031", "CC-032", "CC-033", "CC-034", "CC-035", "CC-036", "CC-037",
    "CC-040", "CC-041", "CC-042", "CC-043", "CC-044", "CC-045", "CC-046",
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

const REQUIRED_SOURCES: [&str; 12] = [
    "jin-v4-audited-formalization-matched-git-manuscript",
    "jin-repo-head-pinned-repository-head",
    "jin-v4-head-later-v4-manuscript",
    "jin-annmath-annals-formatted-manuscript",
    "jin-preprints-v1-preprints-org-metadata-record",
    "ls-arxiv-v1-lorist-schwenninger-arxiv-v1",
    "crouzeix-2007-earlier-universal-numerical-range-bound",
    "crouzeix-palencia-2017-one-plus-square-root-two-result",
    "delyon-delyon-1999-double-layer-calculus",
    "ransford-schwenninger-2018-prior-proof-analysis",
    "schwenninger-devries-2025-double-layer-review",
    "harp-local-verify-local-build-and-scan-observations",
];

const AUTHORITY_NOTICE: &str = concat!(
    "> This file is a maintained Harp technical packet. It separates Harp's\n",
    "> synthesis from primary-source claims; primary-source receipts and local\n",
    "> verification logs live under `evidence/`."
);

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
struct Heading {
    level: HeadingLevel,
    text: String,
    id: Option<String>,
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct Claim {
    id: String,
    fields: BTreeMap<String, Vec<String>>,
}

impl Claim {
    fn field(&self, name: &str) -> Result<&str, String> {
        let values = self
            .fields
            .get(name)
            .ok_or_else(|| format!("{} is missing field {name}", self.id))?;
        if values.len() != 1 {
            return Err(format!("{} field {name} must appear once", self.id));
        }
        let value = values[0].trim();
        if value.is_empty() {
            return Err(format!("{} field {name} is empty", self.id));
        }
        Ok(value)
    }

    fn class(&self) -> Result<EvidenceClass, String> {
        EvidenceClass::parse(self.field("Class")?)
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
        .to_path_buf()
}

fn packet_root() -> PathBuf {
    workspace_root().join("knowledge/crouzeix_conjecture")
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
        if character.is_ascii_alphanumeric() {
            if separator && !output.is_empty() {
                output.push('-');
            }
            output.push(character);
            separator = false;
        } else {
            separator = true;
        }
    }
    output
}

fn headings(text: &str) -> Vec<Heading> {
    let mut output = Vec::new();
    let mut current: Option<(HeadingLevel, Option<String>, usize, String)> = None;
    for (event, range) in Parser::new_ext(text, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                current = Some((
                    level,
                    id.map(|value| value.into_string()),
                    range.start,
                    String::new(),
                ));
            }
            Event::Text(value) | Event::Code(value) | Event::InlineMath(value)
                if current.is_some() =>
            {
                current
                    .as_mut()
                    .expect("heading is present")
                    .3
                    .push_str(&value);
            }
            Event::SoftBreak | Event::HardBreak if current.is_some() => {
                current.as_mut().expect("heading is present").3.push(' ');
            }
            Event::End(TagEnd::Heading(level)) => {
                let (start_level, id, start, text) =
                    current.take().expect("heading start precedes end");
                assert_eq!(start_level, level);
                output.push(Heading {
                    level,
                    text: text.trim().to_owned(),
                    id,
                    start,
                    end: range.end,
                });
            }
            _ => {}
        }
    }
    output
}

fn heading_ids(text: &str) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    for heading in headings(text) {
        let id = heading.id.unwrap_or_else(|| slug(&heading.text));
        if id.is_empty() || !ids.insert(id.clone()) {
            return Err(format!("invalid or duplicate heading ID {id}"));
        }
    }
    Ok(ids)
}

fn explicit_heading_ids(text: &str) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    for heading in headings(text) {
        let Some(id) = heading.id else {
            continue;
        };
        if id.is_empty() || !ids.insert(id.clone()) {
            return Err(format!("invalid or duplicate explicit heading ID {id}"));
        }
    }
    Ok(ids)
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
    let mut values = BTreeMap::new();
    for line in frontmatter.lines() {
        if line.is_empty() || line.starts_with([' ', '\t']) {
            return Err("frontmatter must use top-level one-line values".to_owned());
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("frontmatter line has no colon: {line}"))?;
        if key.trim().is_empty()
            || value.trim().is_empty()
            || values.insert(key.trim(), value.trim()).is_some()
        {
            return Err(format!("invalid frontmatter line: {line}"));
        }
    }
    let keys = values.keys().copied().collect::<BTreeSet<_>>();
    let expected = REQUIRED_FRONTMATTER.into_iter().collect::<BTreeSet<_>>();
    if keys != expected {
        return Err(format!("frontmatter schema drifted: {keys:?}"));
    }
    if values["created"] != "2026-08-14"
        || values["updated"] != "2026-08-14"
        || !matches!(values["confidence"], "low" | "medium" | "high")
        || !values["tags"].starts_with('[')
        || !values["tags"].ends_with(']')
        || !values["canonical"].ends_with(".md")
        || values["canonical"].contains('/')
    {
        return Err("frontmatter values violate the packet contract".to_owned());
    }
    Ok(values)
}

fn parse_fields(id: &str, body: &str) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut fields = BTreeMap::<String, Vec<String>>::new();
    let mut depth = 0_usize;
    let mut lists = 0_usize;
    for (event, range) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
        match event {
            Event::Start(Tag::List(_)) if depth == 0 => {
                lists += 1;
                depth += 1;
            }
            Event::Start(Tag::Item) => {
                depth += 1;
                let raw_item = body[range].trim();
                let raw = raw_item.strip_prefix("- ").unwrap_or(raw_item);
                let (name, value) = raw
                    .split_once(':')
                    .ok_or_else(|| format!("{id} has malformed field `{raw}`"))?;
                fields
                    .entry(name.trim().to_owned())
                    .or_default()
                    .push(value.split_whitespace().collect::<Vec<_>>().join(" "));
            }
            Event::Start(_) => depth += 1,
            Event::End(_) => depth = depth.checked_sub(1).expect("balanced Markdown events"),
            Event::Text(value) | Event::Code(value) if depth == 0 && !value.trim().is_empty() => {
                return Err(format!("{id} has content outside its field list"));
            }
            _ => {}
        }
    }
    if lists != 1 {
        return Err(format!("{id} must have one field list"));
    }
    Ok(fields)
}

fn claims(text: &str) -> Result<Vec<Claim>, String> {
    let all_headings = headings(text);
    let mut claims = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, heading) in all_headings.iter().enumerate() {
        if heading.level != HeadingLevel::H2 {
            continue;
        }
        let Some((id, title)) = heading.text.split_once(':') else {
            continue;
        };
        let id = id.trim();
        if !id.starts_with("CC-") {
            continue;
        }
        if title.trim().is_empty() || !seen.insert(id.to_owned()) {
            return Err(format!(
                "malformed or duplicate claim heading {}",
                heading.text
            ));
        }
        let explicit = heading
            .id
            .as_deref()
            .ok_or_else(|| format!("{id} lacks an explicit heading ID"))?;
        if !explicit.starts_with(&format!("{}-", id.to_ascii_lowercase())) {
            return Err(format!(
                "{id} has mismatched explicit heading ID {explicit}"
            ));
        }
        let body_end = all_headings
            .iter()
            .skip(index + 1)
            .find(|next| next.level == HeadingLevel::H2)
            .map_or(text.len(), |next| next.start);
        claims.push(Claim {
            id: id.to_owned(),
            fields: parse_fields(id, &text[heading.end..body_end])?,
        });
    }
    Ok(claims)
}

fn resolve_local_link(source: &Path, target: &str) -> Result<PathBuf, String> {
    let (path, anchor) = target
        .split_once('#')
        .map_or((target, None), |(path, anchor)| (path, Some(anchor)));
    let mut resolved = source
        .parent()
        .ok_or_else(|| format!("{} has no parent", source.display()))?
        .to_path_buf();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !resolved.pop() || !resolved.starts_with(workspace_root()) {
                    return Err(format!("link escapes repository: {target}"));
                }
            }
            Component::Normal(part) => resolved.push(part),
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("absolute local link: {target}"));
            }
        }
    }
    if !resolved.is_file() {
        return Err(format!("unresolved local link: {target}"));
    }
    if let Some(anchor) = anchor {
        let target_text = fs::read_to_string(&resolved).map_err(|error| error.to_string())?;
        if let Some(line) = anchor.strip_prefix('L') {
            let line = line
                .parse::<usize>()
                .map_err(|_| format!("invalid line anchor {anchor}"))?;
            if line == 0 || line > target_text.lines().count() {
                return Err(format!("missing line anchor {target}"));
            }
        } else if resolved.extension().and_then(|value| value.to_str()) == Some("md")
            && !explicit_heading_ids(&target_text)?.contains(anchor)
        {
            return Err(format!("missing explicit heading anchor {target}"));
        }
    }
    Ok(resolved)
}

fn validate_wiki_links(source: &Path, text: &str) -> Result<(), String> {
    for link in harp::knowledge::resolve_wiki_links(&workspace_root(), source, text)
        .map_err(|error| error.to_string())?
    {
        if !link
            .target
            .starts_with(Path::new("knowledge/crouzeix_conjecture"))
            || link.requested_heading.is_none()
        {
            continue;
        }
        let explicit = explicit_heading_ids(
            &fs::read_to_string(workspace_root().join(&link.target))
                .map_err(|error| error.to_string())?,
        )?;
        if link.requested_heading.as_deref() != link.heading_id.as_deref()
            || !explicit.contains(link.requested_heading.as_deref().expect("checked heading"))
        {
            return Err(format!(
                "native wiki heading must use an explicit Crouzeix ID: {}",
                link.target.display()
            ));
        }
    }
    Ok(())
}

fn validate_claims(path: &Path, entries: &[Claim]) -> Result<(), String> {
    let ids = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let evidence_root = workspace_root().join("evidence/crouzeix_conjecture");
    let mut reciprocal = BTreeSet::new();
    for entry in entries {
        for field in REQUIRED_FIELDS {
            entry.field(field)?;
        }
        let class = entry.class()?;
        if !matches!(
            entry.field("Confidence")?.trim_matches('`'),
            "low" | "medium" | "high"
        ) {
            return Err(format!("{} has invalid confidence", entry.id));
        }
        for field in ["Source", "Locator"] {
            let markdown = markdown_links(entry.field(field)?);
            let wiki =
                harp::knowledge::resolve_wiki_links(&workspace_root(), path, entry.field(field)?)
                    .map_err(|error| error.to_string())?;
            if markdown.is_empty() && wiki.is_empty() {
                return Err(format!("{} field {field} must contain a link", entry.id));
            }
            validate_wiki_links(path, entry.field(field)?)?;
        }
        let locators = markdown_links(entry.field("Locator")?);
        if !locators.iter().any(|locator| {
            locator
                .split_once('#')
                .is_some_and(|(_, anchor)| anchor.starts_with('L'))
        }) {
            return Err(format!(
                "{} locator requires a conventional Markdown line locator",
                entry.id
            ));
        }
        for locator in locators {
            if locator.starts_with("http://") || locator.starts_with("https://") {
                return Err(format!("{} locator is not local evidence", entry.id));
            }
            let resolved = resolve_local_link(path, &locator)?;
            if !resolved.starts_with(&evidence_root) {
                return Err(format!("{} locator is outside Crouzeix evidence", entry.id));
            }
        }
        if class.is_source_derived() {
            if !matches!(
                entry.field("Mode")?.trim_matches('`'),
                "quote" | "paraphrase"
            ) || !matches!(
                entry.field("Source stability")?.trim_matches('`'),
                "pinned" | "dated observation"
            ) {
                return Err(format!(
                    "{} has invalid source rendering metadata",
                    entry.id
                ));
            }
            if entry.field("Source stability")?.trim_matches('`') == "dated observation"
                && entry.field("Observed")?.trim_matches('`') != "2026-08-14"
            {
                return Err(format!("{} has invalid observation date", entry.id));
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
        for relationship in entry.fields.get("Relationship").into_iter().flatten() {
            let value = relationship.trim().trim_matches('`');
            let mut parts = value.split_whitespace();
            let (Some(kind), Some(target), None) = (parts.next(), parts.next(), parts.next())
            else {
                return Err(format!("{} has malformed relationship {value}", entry.id));
            };
            if !matches!(
                kind,
                "contradicts" | "unresolved-with" | "supersedes" | "narrows"
            ) || target == entry.id
                || !ids.contains(target)
            {
                return Err(format!("{} has invalid relationship {value}", entry.id));
            }
            if matches!(kind, "contradicts" | "unresolved-with") {
                reciprocal.insert((entry.id.as_str(), kind, target));
            }
        }
    }
    for &(source, kind, target) in &reciprocal {
        if !reciprocal.contains(&(target, kind, source)) {
            return Err(format!(
                "{source} relationship {kind} {target} is not reciprocal"
            ));
        }
    }
    Ok(())
}

fn assert_packet_registration() {
    let corpus = fs::read_to_string(workspace_root().join("crates/harp/src/corpus/mod.rs"))
        .expect("read corpus registry");
    let search = fs::read_to_string(workspace_root().join("crates/harp/src/search.rs"))
        .expect("read search roots");
    let route_types = fs::read_to_string(workspace_root().join("atlas/src/content/types.ts"))
        .expect("read Atlas route types");
    assert!(corpus.contains("\"crouzeix-conjecture\""));
    for name in EXPECTED_FILES {
        assert!(
            corpus.contains(&format!("knowledge/crouzeix_conjecture/{name}")),
            "corpus registry is missing {name}"
        );
    }
    assert!(search.contains("\"knowledge/crouzeix_conjecture\""));
    assert!(route_types.contains("\"crouzeix-conjecture\""));
}

#[test]
fn crouzeix_conjecture_packet_has_complete_observable_contract() {
    let root = packet_root();
    assert!(root.is_dir(), "missing {}", root.display());
    assert!(!root.join("README.md").exists());

    let actual = fs::read_dir(&root)
        .expect("read packet")
        .map(|entry| {
            entry
                .expect("read packet entry")
                .file_name()
                .into_string()
                .expect("UTF-8 filename")
        })
        .collect::<BTreeSet<_>>();
    let expected = EXPECTED_FILES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "packet file roster drifted");

    let mut referenced_claims = BTreeSet::new();
    for name in EXPECTED_FILES {
        let path = root.join(name);
        let text = fs::read_to_string(&path).expect("read packet file");
        let metadata =
            frontmatter(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(metadata["canonical"], name);
        assert!(
            text.contains(AUTHORITY_NOTICE),
            "{name} lacks authority notice"
        );
        heading_ids(&text).unwrap_or_else(|error| panic!("{name}: {error}"));
        for placeholder in ["TODO", "TBD", "FIXME"] {
            assert!(!text.contains(placeholder), "{name} contains {placeholder}");
        }
        for link in markdown_links(&text) {
            if link.starts_with("https://")
                || link.starts_with("http://")
                || link.starts_with("mailto:")
            {
                continue;
            }
            resolve_local_link(&path, &link).unwrap_or_else(|error| panic!("{name}: {error}"));
        }
        validate_wiki_links(&path, &text).unwrap_or_else(|error| panic!("{name}: {error}"));
        if name != "crouzeix_conjecture_index.md" {
            assert!(
                text.trim_end().ends_with(
                    "Back to the [Crouzeix conjecture index](crouzeix_conjecture_index.md)."
                ),
                "{name} lacks the exact index backlink"
            );
        }
        if name != "claim_evidence_ledger.md" {
            for id in REQUIRED_CLAIMS {
                if text.contains(&format!(" - {id}](")) {
                    referenced_claims.insert(id);
                }
            }
        }
    }

    let index =
        fs::read_to_string(root.join("crouzeix_conjecture_index.md")).expect("read packet index");
    for name in EXPECTED_FILES {
        assert!(index.contains(name), "index does not link {name}");
    }

    let registry = fs::read_to_string(root.join("source_registry.md")).expect("read registry");
    let source_ids = heading_ids(&registry).expect("source heading IDs");
    for source in REQUIRED_SOURCES {
        assert!(
            source_ids.contains(source),
            "source registry lacks {source}"
        );
    }

    let ledger_path = root.join("claim_evidence_ledger.md");
    let ledger = fs::read_to_string(&ledger_path).expect("read claim ledger");
    let entries = claims(&ledger).expect("parse claim ledger");
    validate_claims(&ledger_path, &entries).expect("validate claim ledger");
    let actual_claims = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_claims = REQUIRED_CLAIMS.into_iter().collect::<BTreeSet<_>>();
    assert_eq!(actual_claims, expected_claims, "claim roster drifted");
    assert_eq!(
        referenced_claims, expected_claims,
        "every canonical claim must have a reader-facing route"
    );

    let completion = fs::read_to_string(root.join("04_jin_positive_real_completion.md"))
        .expect("read Jin completion");
    assert!(completion.contains("H\\text{ analytic on }\\mathbb D"));
    assert!(completion.contains("|\\lambda_i|\\le1"));
    assert!(completion.contains("4\\widehat Y-\\widehat Y\\widehat P"));

    let lorist = fs::read_to_string(root.join("05_lorist_schwenninger_proof.md"))
        .expect("read Lorist-Schwenninger proof");
    assert!(lorist.contains("\\|T^n\\|"));
    assert!(lorist.contains("M(2+M)"));
    assert!(!lorist.to_ascii_lowercase().contains("missing hypothesis"));

    let lean =
        fs::read_to_string(root.join("07_jin_lean_verification.md")).expect("read Lean audit");
    assert!(lean.contains("PositiveRealCompletionStatement"));
    assert!(lean.contains("positiveRealCompletionStatement"));

    let status = fs::read_to_string(root.join("09_status_and_critical_assessment.md"))
        .expect("read critical assessment");
    assert!(status.contains("Preprints.org"));
    assert!(status.contains("metadata only"));

    assert_packet_registration();
}

#[test]
fn native_wiki_links_validate_aliases_headings_and_escape_safely() {
    let document = packet_root().join("link-test.md");
    let valid = [
        "[[crouzeix_conjecture_index|Crouzeix index]]",
        "[[source_registry#jin-v4-audited-formalization-matched-git-manuscript|Jin source]]",
        "[[evidence/crouzeix_conjecture/verification/jin-565b6a3-build.log|build log]]",
        "```md\n[[../../outside]]\n```",
    ]
    .join("\n");
    validate_wiki_links(&document, &valid)
        .expect("production wiki resolver accepts aliases, headings, and fenced text");
    for invalid in ["[[../../outside]]", "[[/etc/passwd]]"] {
        assert!(
            validate_wiki_links(&document, invalid).is_err(),
            "escaping native wiki link unexpectedly passed: {invalid}"
        );
    }
}

#[test]
fn native_wiki_heading_links_require_explicit_crouzeix_ids() {
    let document = packet_root().join("link-test.md");
    let source = "claim_evidence_ledger";
    let generated_slug = "crouzeix-constant-two-conjecture";
    assert!(
        validate_wiki_links(&document, &format!("[[{source}#{generated_slug}]]")).is_err(),
        "a generated heading slug must not bypass Crouzeix's explicit-ID contract"
    );
    validate_wiki_links(
        &document,
        &format!("[[{source}#cc-001-crouzeix-constant-two-conjecture]]"),
    )
    .expect("the documented explicit Crouzeix heading ID must validate");
}
