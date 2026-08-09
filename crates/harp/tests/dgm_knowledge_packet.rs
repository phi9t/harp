use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use pulldown_cmark::{BrokenLink, Event, Options, Parser, Tag};

const EXPECTED_FILES: [&str; 16] = [
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
    "> This file is a learning projection. Canonical claims live under `content/`;\n",
    "> primary-source captures and pinned implementation files live under\n",
    "> `evidence/`."
);

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

    let crosswalk =
        fs::read_to_string(root.join("claim_evidence_crosswalk.md")).expect("read claim crosswalk");
    assert!(crosswalk.contains(
        "| Claim ID | Class | Claim | Canonical home | Primary evidence | Locator | \
         Reproduction status | Confidence | Caveat |"
    ));
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
        "[local](../../content/systems/dgm.md#problem-and-rsi-relevance)\n",
        "[reference][dgm]\n\n",
        "[dgm]: ../../content/systems/dgm.md \"DGM\"\n",
    );
    validate_local_links(&document, valid).expect("supported Markdown links");

    for invalid in [
        "[broken](../../content/does-not-exist.md)",
        "![broken](../../evidence/does-not-exist.png)",
        "[unclosed](../../content/systems/dgm.md",
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
