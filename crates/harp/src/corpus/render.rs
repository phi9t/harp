use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};

use super::contracts::{normalize_link_path, ValidatedCanonicalSource};
use super::*;

pub(super) fn compile_document(
    source: &ValidatedCanonicalSource,
    coverage: &[CoverageEntry],
    canonical_sources: &BTreeMap<String, ValidatedCanonicalSource>,
) -> Result<CanonicalDocument, AppError> {
    let body = markdown_body(&source.markdown, &source.path)?;
    debug_assert_eq!(source.body_sha256, sha256(body.as_bytes()));
    let title = first_heading(body).ok_or_else(|| {
        invalid(
            "knowledge.rsi.heading",
            format!("{} must contain one H1 title", source.path),
        )
    })?;
    let html = render_markdown_with_sources(body, &source.path, coverage, canonical_sources);
    let concept_id = source
        .entries
        .iter()
        .find(|entry| entry.section_id.is_none())
        .or_else(|| source.entries.first())
        .map(|entry| entry.concept_id.clone())
        .unwrap_or_else(|| auxiliary_document_id(&source.path));
    Ok(CanonicalDocument {
        concept_id,
        title,
        canonical_markdown_path: source.path.clone(),
        markdown_sha256: source.body_sha256.clone(),
        html_sha256: sha256(html.as_bytes()),
        html,
    })
}

fn auxiliary_document_id(path: &str) -> String {
    if let Some((document_id, _)) = AUXILIARY_DOCUMENTS
        .iter()
        .find(|(_, document_path)| *document_path == path)
    {
        return (*document_id).to_owned();
    }
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| {
            if path.starts_with("content/systems/") {
                return Some(stem.to_owned());
            }
            stem.split_once('-')
                .filter(|(prefix, _)| prefix.bytes().all(|byte| byte.is_ascii_digit()))
                .map(|(_, section)| format!("weng-{section}"))
        })
        .unwrap_or_default()
}

pub(super) fn markdown_body<'a>(markdown: &'a str, path: &str) -> Result<&'a str, AppError> {
    let Some(rest) = markdown.strip_prefix("---\n") else {
        return Ok(markdown);
    };
    let Some(end) = rest.find("\n---\n") else {
        return Err(invalid(
            "knowledge.rsi.frontmatter",
            format!("{path} has unclosed YAML frontmatter"),
        ));
    };
    Ok(&rest[end + 5..])
}

fn first_heading(markdown: &str) -> Option<String> {
    markdown
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
}

pub(super) fn heading_ids(markdown: &str) -> BTreeSet<String> {
    markdown
        .lines()
        .filter_map(|line| {
            line.strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))
                .map(slug)
        })
        .collect()
}

fn slug(value: &str) -> String {
    let mut output = String::new();
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
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

#[cfg(test)]
pub(super) fn render_markdown(
    markdown: &str,
    source_path: &str,
    coverage: &[CoverageEntry],
) -> String {
    render_markdown_with_sources(markdown, source_path, coverage, &BTreeMap::new())
}

fn render_markdown_with_sources(
    markdown: &str,
    source_path: &str,
    coverage: &[CoverageEntry],
    canonical_sources: &BTreeMap<String, ValidatedCanonicalSource>,
) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST;
    let source_parent = Path::new(source_path)
        .parent()
        .expect("canonical RSI source has a parent");
    let events = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw)
            if matches!(
                raw.trim(),
                "<details>"
                    | "</details>"
                    | SOURCE_SUMMARY
                    | REFERENCE_SUMMARY
                    | "<summary>Check your answer</summary>"
            ) =>
        {
            Event::Html(raw)
        }
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: offline_link_destination_with_sources(
                &dest_url,
                source_parent,
                coverage,
                canonical_sources,
            )
            .into(),
            title,
            id,
        }),
        Event::Start(Tag::Table(_)) => Event::Html(
            "<div class=\"canonical-table-scroll\" role=\"region\" aria-label=\"Scrollable data table\" tabindex=\"0\"><table>".into(),
        ),
        Event::End(TagEnd::Table) => Event::Html("</table></div>".into()),
        other => other,
    });
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}

#[cfg(test)]
pub(super) fn offline_link_destination(
    value: &str,
    source_parent: &Path,
    coverage: &[CoverageEntry],
) -> String {
    offline_link_destination_with_sources(value, source_parent, coverage, &BTreeMap::new())
}

pub(super) fn offline_link_destination_with_sources(
    value: &str,
    source_parent: &Path,
    coverage: &[CoverageEntry],
    canonical_sources: &BTreeMap<String, ValidatedCanonicalSource>,
) -> String {
    if value.starts_with('#')
        || value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("mailto:")
    {
        return value.to_owned();
    }
    let (path, fragment) = value
        .split_once('#')
        .map_or((value, None), |(path, fragment)| (path, Some(fragment)));
    let resolved = if let Some(rooted) = path.strip_prefix('/') {
        normalize_link_path(Path::new(""), Path::new(rooted))
    } else {
        normalize_link_path(source_parent, Path::new(path))
    };
    let Some(resolved) = resolved else {
        return value.to_owned();
    };
    let resolved = resolved.to_string_lossy();
    if let Some(chapter) = coverage.iter().find(|entry| {
        entry.coverage_depth == CoverageDepth::Chapter && entry.canonical_markdown_path == resolved
    }) {
        return format!("#chapters/{}", chapter.concept_id);
    }
    if let Some(source) = canonical_sources.get(resolved.as_ref()) {
        if let Some(owner) = source
            .entries
            .iter()
            .find(|entry| entry.section_id.is_none())
            .or_else(|| source.entries.first())
        {
            return format!("#documents/{}", owner.concept_id);
        }
        return format!("#documents/{}", auxiliary_document_id(&source.path));
    }
    let suffix = fragment.map_or(String::new(), |fragment| format!("#{fragment}"));
    format!("../../{resolved}{suffix}")
}
