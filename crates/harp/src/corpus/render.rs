use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use super::contracts::{normalize_link_path, ValidatedCanonicalSource};
use super::obsidian::{
    canonical_heading_map, line_is_code_context, rewrite_wiki_links, WikiLink, WikiSubpath,
};
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RouteTarget {
    Reader {
        route_id: &'static str,
    },
    Chapter {
        concept_id: String,
        document_id: String,
        heading_ids: BTreeMap<String, String>,
    },
    Document {
        document_id: String,
        heading_ids: BTreeMap<String, String>,
    },
}

pub(super) fn compile_document(
    source: &ValidatedCanonicalSource,
    route_targets: &BTreeMap<String, RouteTarget>,
) -> Result<CanonicalDocument, AppError> {
    let body = markdown_body(&source.markdown, &source.path)?;
    debug_assert_eq!(source.body_sha256, sha256(body.as_bytes()));
    let metadata =
        super::contracts::document_metadata(&source.markdown, &source.path, &source.entries)?;
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
        metadata,
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
    parsed_heading_ids(markdown, "").into_iter().collect()
}

pub(super) fn heading_ids_for_source(
    markdown: &str,
    source_path: &str,
) -> Result<BTreeSet<String>, AppError> {
    if !is_crouzeix_packet(source_path) {
        return Ok(parsed_heading_ids(markdown, source_path)
            .into_iter()
            .collect());
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
        let heading_ids = canonical_sources
            .get(&chapter.canonical_markdown_path)
            .map(|source| {
                let body =
                    markdown_body(&source.markdown, &source.path).unwrap_or(&source.markdown);
                addressable_heading_ids(body, &source.path)
            })
            .unwrap_or_default();
        targets.insert(
            chapter.canonical_markdown_path.clone(),
            RouteTarget::Chapter {
                concept_id: chapter.concept_id.clone(),
                document_id: chapter.concept_id.clone(),
                heading_ids,
            },
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
        targets.insert(path.to_owned(), RouteTarget::Reader { route_id });
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

fn addressable_heading_ids(markdown: &str, source_path: &str) -> BTreeMap<String, String> {
    canonical_heading_map(markdown, source_path)
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
    let (markdown, callouts) = preprocess_obsidian_callouts(markdown);
    let mut embeds = Vec::new();
    let markdown = rewrite_wiki_links(&markdown, |link| {
        render_wiki_link(link, source_path, route_targets, &mut embeds)
    })
    .unwrap_or(markdown);
    let options = markdown_options(source_path);
    let source_parent = Path::new(source_path)
        .parent()
        .expect("canonical RSI source has a parent");
    let events = Parser::new_ext(&markdown, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw)
            if is_allowed_html(&raw) =>
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
    for (index, callout) in callouts.iter().enumerate() {
        output = output.replace(
            &format!("<p>HARP_OBSIDIAN_CALLOUT_OPEN_{index}</p>\n"),
            &format!(
                "<aside class=\"obsidian-callout\" data-callout-type=\"{}\" role=\"note\">\n",
                callout.callout_type
            ),
        );
        if let Some(title) = &callout.title {
            output = output.replace(
                &format!("<p>HARP_OBSIDIAN_CALLOUT_TITLE_{index}</p>\n"),
                &format!(
                    "<p class=\"obsidian-callout-title\">{}</p>\n",
                    escape_html_text(title)
                ),
            );
        }
        output = output.replace(
            &format!("<p>HARP_OBSIDIAN_CALLOUT_CLOSE_{index}</p>\n"),
            "</aside>\n",
        );
    }
    for (index, embed) in embeds.iter().enumerate() {
        output = output.replace(
            &format!("HARP_OBSIDIAN_EMBED_{index}"),
            &format!(
                "<a class=\"obsidian-embed-fallback\" data-obsidian-embed=\"true\" href=\"{}\">{}</a>",
                escape_html_attribute(&embed.destination),
                escape_html_text(&embed.display),
            ),
        );
    }
    output
}

fn render_wiki_link(
    link: &WikiLink,
    source_path: &str,
    route_targets: &BTreeMap<String, RouteTarget>,
    embeds: &mut Vec<EmbedFallback>,
) -> String {
    let path = link.path.as_deref();
    let target = if let Some(path) = path {
        if has_vault_root(Path::new(path)) {
            PathBuf::from(path)
        } else {
            Path::new(source_path)
                .parent()
                .expect("canonical source has a parent")
                .join(path)
        }
    } else {
        PathBuf::from(source_path)
    };
    let mut target = target;
    if target.extension().is_none() {
        target.set_extension("md");
    }
    let target_text = target.to_string_lossy();
    let display = link
        .alias
        .as_deref()
        .or_else(|| path.and_then(|path| path.rsplit('/').next()))
        .or(match &link.subpath {
            Some(WikiSubpath::Heading(heading)) => Some(heading.as_str()),
            _ => None,
        })
        .unwrap_or(source_path);
    if link.embed && target.extension().and_then(|extension| extension.to_str()) == Some("pdf") {
        let suffix = match link.subpath {
            Some(WikiSubpath::PdfPage(page)) => format!("#page={page}"),
            _ => String::new(),
        };
        let index = embeds.len();
        embeds.push(EmbedFallback {
            destination: format!("../../{target_text}{suffix}"),
            display: display.to_owned(),
        });
        return format!("HARP_OBSIDIAN_EMBED_{index}");
    }
    let destination = if let Some(route) = route_targets.get(target_text.as_ref()) {
        match route {
            RouteTarget::Reader { route_id } => format!("#{route_id}"),
            RouteTarget::Chapter {
                concept_id,
                document_id,
                heading_ids,
            } => route_destination(
                format!("#chapters/{concept_id}"),
                document_id,
                heading_ids,
                link.subpath.as_ref(),
            ),
            RouteTarget::Document {
                document_id,
                heading_ids,
            } => route_destination(
                format!("#documents/{document_id}"),
                document_id,
                heading_ids,
                link.subpath.as_ref(),
            ),
        }
    } else {
        let suffix = match link.subpath {
            Some(WikiSubpath::PdfPage(page)) => format!("#page={page}"),
            Some(WikiSubpath::Heading(ref heading)) => format!("#{}", slug(heading)),
            Some(WikiSubpath::Block(_)) => String::new(),
            None => String::new(),
        };
        format!("../../{target_text}{suffix}")
    };
    format!("[{display}]({destination})")
}

fn route_destination(
    default_destination: String,
    document_id: &str,
    heading_ids: &BTreeMap<String, String>,
    subpath: Option<&WikiSubpath>,
) -> String {
    let Some(WikiSubpath::Heading(heading)) = subpath else {
        return default_destination;
    };
    heading_ids
        .get(heading)
        .or_else(|| heading_ids.get(&slug(heading)))
        .map(|heading_id| format!("#documents/{document_id}?section={heading_id}"))
        .unwrap_or(default_destination)
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

struct ObsidianCallout {
    callout_type: String,
    title: Option<String>,
}

struct EmbedFallback {
    destination: String,
    display: String,
}

fn preprocess_obsidian_callouts(markdown: &str) -> (String, Vec<ObsidianCallout>) {
    let mut output = String::with_capacity(markdown.len());
    let mut callouts = Vec::new();
    let mut fence = None;
    let mut inline_code = None;
    let mut lines = markdown.lines().peekable();
    while let Some(line) = lines.next() {
        if line_is_code_context(line, &mut fence, &mut inline_code) {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        let Some((callout_type, title)) = parse_callout_header(line) else {
            output.push_str(line);
            output.push('\n');
            continue;
        };
        let index = callouts.len();
        callouts.push(ObsidianCallout {
            callout_type: callout_type.to_owned(),
            title: (!title.is_empty()).then(|| title.to_owned()),
        });
        output.push_str(&format!("HARP_OBSIDIAN_CALLOUT_OPEN_{index}\n\n"));
        if !title.is_empty() {
            output.push_str(&format!("HARP_OBSIDIAN_CALLOUT_TITLE_{index}\n\n"));
        }
        while let Some(body_line) = lines.next_if(|body_line| body_line.starts_with('>')) {
            let _ = line_is_code_context(body_line, &mut fence, &mut inline_code);
            let body_line = body_line
                .strip_prefix("> ")
                .or_else(|| body_line.strip_prefix('>'))
                .expect("checked blockquote prefix");
            output.push_str(body_line);
            output.push('\n');
        }
        output.push_str(&format!("\nHARP_OBSIDIAN_CALLOUT_CLOSE_{index}\n"));
    }
    (output, callouts)
}

fn parse_callout_header(line: &str) -> Option<(&str, &str)> {
    let line = line.strip_prefix("> ")?;
    let rest = line.strip_prefix("[!")?;
    let (callout_type, title) = rest.split_once(']')?;
    (!callout_type.is_empty()
        && callout_type
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
    .then_some((callout_type, title.trim()))
}

fn is_allowed_html(raw: &str) -> bool {
    matches!(
        raw.trim(),
        "<details>"
            | "</details>"
            | SOURCE_SUMMARY
            | REFERENCE_SUMMARY
            | "<summary>Check your answer</summary>"
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
                RouteTarget::Chapter {
                    concept_id: entry.concept_id.clone(),
                    document_id: entry.concept_id.clone(),
                    heading_ids: BTreeMap::new(),
                },
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
            RouteTarget::Reader { route_id } => format!("#{route_id}"),
            RouteTarget::Chapter {
                concept_id,
                document_id,
                heading_ids,
            } => markdown_route_destination(
                format!("#chapters/{concept_id}"),
                document_id,
                heading_ids,
                fragment,
            ),
            RouteTarget::Document {
                document_id,
                heading_ids,
            } => markdown_route_destination(
                format!("#documents/{document_id}"),
                document_id,
                heading_ids,
                fragment,
            ),
        };
    }
    let suffix = fragment.map_or(String::new(), |fragment| format!("#{fragment}"));
    format!("../../{resolved}{suffix}")
}

fn markdown_route_destination(
    default_destination: String,
    document_id: &str,
    heading_ids: &BTreeMap<String, String>,
    fragment: Option<&str>,
) -> String {
    fragment
        .and_then(|fragment| {
            heading_ids
                .get(fragment)
                .or_else(|| heading_ids.get(&slug(fragment)))
        })
        .map(|heading_id| format!("#documents/{document_id}?section={heading_id}"))
        .unwrap_or(default_destination)
}
