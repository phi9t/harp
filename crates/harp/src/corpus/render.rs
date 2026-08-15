use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::contracts::{normalize_link_path, ValidatedCanonicalSource};
use super::obsidian::{rewrite_wiki_links, WikiLink, WikiSubpath};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RouteTarget {
    Reader(&'static str),
    Chapter(String),
    Document {
        document_id: String,
        heading_ids: BTreeSet<String>,
    },
}

pub(super) fn compile_document(
    source: &ValidatedCanonicalSource,
    route_targets: &BTreeMap<String, RouteTarget>,
) -> Result<CanonicalDocument, AppError> {
    let body = markdown_body(&source.markdown, &source.path)?;
    debug_assert_eq!(source.body_sha256, sha256(body.as_bytes()));
    let title = first_heading(body).ok_or_else(|| {
        invalid(
            "knowledge.rsi.heading",
            format!("{} must contain one H1 title", source.path),
        )
    })?;
    let html = render_markdown_with_targets(body, &source.path, route_targets);
    let concept_id = source_document_id(source);
    Ok(CanonicalDocument {
        concept_id,
        title,
        canonical_markdown_path: source.path.clone(),
        markdown_sha256: source.body_sha256.clone(),
        html_sha256: sha256(html.as_bytes()),
        html,
    })
}

fn source_document_id(source: &ValidatedCanonicalSource) -> String {
    source
        .entries
        .iter()
        .find(|entry| entry.section_id.is_none())
        .or_else(|| source.entries.first())
        .map(|entry| entry.concept_id.clone())
        .unwrap_or_else(|| auxiliary_document_id(&source.path))
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
            if path.starts_with("knowledge/rsi/systems/") {
                return Some(stem.to_owned());
            }
            if path.starts_with("knowledge/rsi/lessons/") {
                return stem
                    .split_once('-')
                    .filter(|(prefix, _)| prefix.bytes().all(|byte| byte.is_ascii_digit()))
                    .map(|(_, section)| format!("lesson-{section}"));
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
    legacy_heading_ids(markdown).into_iter().collect()
}

pub(super) fn heading_ids_for_source(
    markdown: &str,
    source_path: &str,
) -> Result<BTreeSet<String>, AppError> {
    if !is_crouzeix_packet(source_path) {
        return Ok(legacy_heading_ids(markdown).into_iter().collect());
    }

    let mut unique = BTreeSet::new();
    for heading_id in parsed_heading_ids(markdown, source_path) {
        if !valid_id(&heading_id) || !unique.insert(heading_id.clone()) {
            return Err(invalid(
                "knowledge.rsi.heading_id",
                format!("{source_path} has an invalid or duplicate heading ID: {heading_id}"),
            ));
        }
    }
    Ok(unique)
}

fn parsed_heading_ids(markdown: &str, source_path: &str) -> Vec<String> {
    if !is_crouzeix_packet(source_path) {
        return legacy_heading_ids(markdown);
    }

    let mut headings = Vec::new();
    let mut current = None::<(Option<String>, String)>;
    for event in Parser::new_ext(markdown, markdown_options(source_path)) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2 | HeadingLevel::H3,
                id,
                ..
            }) => {
                current = Some((id.map(|value| value.into_string()), String::new()));
            }
            Event::Text(text) | Event::Code(text) | Event::InlineMath(text) => {
                if let Some((_, heading)) = &mut current {
                    heading.push_str(&text);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((explicit_id, heading)) = current.take() {
                    headings.push(explicit_id.unwrap_or_else(|| slug(&heading)));
                }
            }
            _ => {}
        }
    }
    headings
}

fn legacy_heading_ids(markdown: &str) -> Vec<String> {
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

pub(super) fn route_targets(
    coverage: &[CoverageEntry],
    canonical_sources: &BTreeMap<String, ValidatedCanonicalSource>,
) -> BTreeMap<String, RouteTarget> {
    let mut targets = BTreeMap::new();

    for chapter in coverage
        .iter()
        .filter(|entry| entry.coverage_depth == CoverageDepth::Chapter)
    {
        targets.insert(
            chapter.canonical_markdown_path.clone(),
            RouteTarget::Chapter(chapter.concept_id.clone()),
        );
    }
    for (document_id, path) in AUXILIARY_DOCUMENTS {
        if let Some(source) = canonical_sources.get(path) {
            targets.insert(
                path.to_owned(),
                document_target(document_id.to_owned(), source),
            );
        }
    }
    for (path, source) in canonical_sources {
        targets
            .entry(path.clone())
            .or_insert_with(|| document_target(source_document_id(source), source));
    }
    for (route_id, _, path) in READER_ROUTES {
        targets.insert(path.to_owned(), RouteTarget::Reader(route_id));
    }
    targets
}

fn document_target(document_id: String, source: &ValidatedCanonicalSource) -> RouteTarget {
    let body = markdown_body(&source.markdown, &source.path).unwrap_or(&source.markdown);
    RouteTarget::Document {
        document_id,
        heading_ids: addressable_heading_ids(body, &source.path),
    }
}

fn addressable_heading_ids(markdown: &str, source_path: &str) -> BTreeSet<String> {
    if !is_crouzeix_packet(source_path) {
        return BTreeSet::new();
    }
    Parser::new_ext(markdown, markdown_options(source_path))
        .filter_map(|event| match event {
            Event::Start(Tag::Heading { id: Some(id), .. }) => Some(id.into_string()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
pub(super) fn render_markdown(
    markdown: &str,
    source_path: &str,
    _coverage: &[CoverageEntry],
) -> String {
    render_markdown_with_targets(markdown, source_path, &BTreeMap::new())
}

#[cfg_attr(test, allow(dead_code))]
pub(super) fn render_markdown_with_targets(
    markdown: &str,
    source_path: &str,
    route_targets: &BTreeMap<String, RouteTarget>,
) -> String {
    let markdown = rewrite_wiki_links(markdown, |link| {
        render_wiki_link(link, source_path, route_targets)
    })
    .unwrap_or_else(|_| markdown.to_owned());
    let options = markdown_options(source_path);
    let source_parent = Path::new(source_path)
        .parent()
        .expect("canonical RSI source has a parent");
    let events = Parser::new_ext(&markdown, options).map(|event| match event {
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
        Event::InlineMath(tex) => Event::Html(math_span(&tex, false).into()),
        Event::DisplayMath(tex) => Event::Html(math_span(&tex, true).into()),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: offline_link_destination_with_targets(
                &dest_url,
                source_parent,
                route_targets,
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

fn render_wiki_link(
    link: &WikiLink,
    source_path: &str,
    route_targets: &BTreeMap<String, RouteTarget>,
) -> String {
    let Some(path) = &link.path else {
        return link.alias.clone().unwrap_or_default();
    };
    let target = if has_vault_root(Path::new(path)) {
        PathBuf::from(path)
    } else {
        Path::new(source_path)
            .parent()
            .expect("canonical source has a parent")
            .join(path)
    };
    let mut target = target;
    if target.extension().is_none() {
        target.set_extension("md");
    }
    let target_text = target.to_string_lossy();
    let destination = if let Some(route) = route_targets.get(target_text.as_ref()) {
        match route {
            RouteTarget::Reader(route_id) => format!("#{route_id}"),
            RouteTarget::Chapter(concept_id) => format!("#chapters/{concept_id}"),
            RouteTarget::Document {
                document_id,
                heading_ids,
            } => match &link.subpath {
                Some(WikiSubpath::Heading(heading)) => {
                    let heading_id = slug(heading);
                    if heading_ids.contains(&heading_id) {
                        format!("#documents/{document_id}?section={heading_id}")
                    } else {
                        format!("#documents/{document_id}")
                    }
                }
                _ => format!("#documents/{document_id}"),
            },
        }
    } else {
        let suffix = match link.subpath {
            Some(WikiSubpath::PdfPage(page)) => format!("#page={page}"),
            Some(WikiSubpath::Heading(ref heading)) => format!("#{}", slug(heading)),
            None => String::new(),
        };
        format!("../../{target_text}{suffix}")
    };
    let display = link
        .alias
        .as_deref()
        .unwrap_or_else(|| path.rsplit('/').next().unwrap_or(path));
    format!("[{display}]({destination})")
}

fn has_vault_root(path: &Path) -> bool {
    path.components()
        .next()
        .and_then(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .is_some_and(|root| ["knowledge", "evidence", "content", "labs", "crates"].contains(&root))
}

fn markdown_options(source_path: &str) -> Options {
    let mut options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST;
    if is_crouzeix_packet(source_path) {
        options |= Options::ENABLE_HEADING_ATTRIBUTES | Options::ENABLE_MATH;
    }
    options
}

fn is_crouzeix_packet(source_path: &str) -> bool {
    source_path.starts_with("knowledge/crouzeix_conjecture/")
}

fn math_span(tex: &str, display: bool) -> String {
    let kind = if display { "display" } else { "inline" };
    format!(
        "<span class=\"math math-{kind}\" data-tex=\"{}\">{}</span>",
        escape_html_attribute(tex),
        escape_html_text(tex),
    )
}

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_html_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
pub(super) fn offline_link_destination(
    value: &str,
    source_parent: &Path,
    coverage: &[CoverageEntry],
) -> String {
    let targets = coverage
        .iter()
        .filter(|entry| entry.coverage_depth == CoverageDepth::Chapter)
        .map(|entry| {
            (
                entry.canonical_markdown_path.clone(),
                RouteTarget::Chapter(entry.concept_id.clone()),
            )
        })
        .collect();
    offline_link_destination_with_targets(value, source_parent, &targets)
}

#[cfg(test)]
pub(super) fn offline_link_destination_with_sources(
    value: &str,
    source_parent: &Path,
    coverage: &[CoverageEntry],
    canonical_sources: &BTreeMap<String, ValidatedCanonicalSource>,
) -> String {
    let targets = route_targets(coverage, canonical_sources);
    offline_link_destination_with_targets(value, source_parent, &targets)
}

pub(super) fn offline_link_destination_with_targets(
    value: &str,
    source_parent: &Path,
    route_targets: &BTreeMap<String, RouteTarget>,
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
    if let Some(target) = route_targets.get(resolved.as_ref()) {
        return match target {
            RouteTarget::Reader(route_id) => format!("#{route_id}"),
            RouteTarget::Chapter(concept_id) => format!("#chapters/{concept_id}"),
            RouteTarget::Document {
                document_id,
                heading_ids,
            } => fragment
                .filter(|fragment| heading_ids.contains(*fragment))
                .map_or_else(
                    || format!("#documents/{document_id}"),
                    |fragment| format!("#documents/{document_id}?section={fragment}"),
                ),
        };
    }
    let suffix = fragment.map_or(String::new(), |fragment| format!("#{fragment}"));
    format!("../../{resolved}{suffix}")
}
