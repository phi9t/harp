use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::error::AppError;
use crate::fs::HeldDirectory;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WikiSubpath {
    Heading(String),
    Block(String),
    PdfPage(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct WikiLink {
    pub(super) embed: bool,
    pub(super) path: Option<String>,
    pub(super) subpath: Option<WikiSubpath>,
    pub(super) alias: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ResolvedWikiLink {
    pub(super) target: PathBuf,
    pub(super) heading_id: Option<String>,
    pub(super) pdf_page: Option<u32>,
    pub(super) embed: bool,
    pub(super) display: String,
}

pub(super) fn parse_wiki_links(markdown: &str) -> Result<Vec<WikiLink>, String> {
    let mut links = Vec::new();
    let mut fence = None;
    let mut inline_code = None;

    for line in markdown.split_inclusive('\n') {
        if update_fence(line, &mut fence) {
            continue;
        }
        if fence.is_some() {
            continue;
        }

        let bytes = line.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'`' {
                let delimiter_length = delimiter_length(bytes, index, b'`');
                inline_code = match inline_code {
                    Some(length) if length == delimiter_length => None,
                    Some(length) => Some(length),
                    None => Some(delimiter_length),
                };
                index += delimiter_length;
                continue;
            }
            if inline_code.is_some() {
                index += 1;
                continue;
            }

            let (embed, open_index) = if bytes[index..].starts_with(b"![[".as_slice()) {
                (true, index + 1)
            } else if bytes[index..].starts_with(b"[[") {
                (false, index)
            } else {
                index += 1;
                continue;
            };
            let content_start = open_index + 2;
            let Some(close_offset) = line[content_start..].find("]]") else {
                return Err("unclosed Obsidian wikilink".to_owned());
            };
            let content_end = content_start + close_offset;
            links.push(parse_wiki_link(&line[content_start..content_end], embed)?);
            index = content_end + 2;
        }
    }

    if fence.is_some() {
        return Err("unclosed Markdown code fence".to_owned());
    }
    Ok(links)
}

pub(super) fn rewrite_wiki_links(
    markdown: &str,
    mut replacement: impl FnMut(&WikiLink) -> String,
) -> Result<String, String> {
    let mut output = String::with_capacity(markdown.len());
    let mut fence = None;
    let mut inline_code = None;

    for line in markdown.split_inclusive('\n') {
        if update_fence(line, &mut fence) {
            output.push_str(line);
            continue;
        }
        if fence.is_some() {
            output.push_str(line);
            continue;
        }

        let bytes = line.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'`' {
                let delimiter_length = delimiter_length(bytes, index, b'`');
                inline_code = match inline_code {
                    Some(length) if length == delimiter_length => None,
                    Some(length) => Some(length),
                    None => Some(delimiter_length),
                };
                output.push_str(&line[index..index + delimiter_length]);
                index += delimiter_length;
                continue;
            }
            if inline_code.is_some() {
                let character = line[index..]
                    .chars()
                    .next()
                    .expect("index is within UTF-8 string");
                output.push(character);
                index += character.len_utf8();
                continue;
            }

            let (embed, open_index) = if bytes[index..].starts_with(b"![[".as_slice()) {
                (true, index + 1)
            } else if bytes[index..].starts_with(b"[[") {
                (false, index)
            } else {
                let character = line[index..]
                    .chars()
                    .next()
                    .expect("index is within UTF-8 string");
                output.push(character);
                index += character.len_utf8();
                continue;
            };
            let content_start = open_index + 2;
            let Some(close_offset) = line[content_start..].find("]]") else {
                return Err("unclosed Obsidian wikilink".to_owned());
            };
            let content_end = content_start + close_offset;
            let link = parse_wiki_link(&line[content_start..content_end], embed)?;
            output.push_str(&replacement(&link));
            index = content_end + 2;
        }
    }

    if fence.is_some() {
        return Err("unclosed Markdown code fence".to_owned());
    }
    Ok(output)
}

pub(super) fn line_is_code_context(
    line: &str,
    fence: &mut Option<(u8, usize)>,
    inline_code: &mut Option<usize>,
) -> bool {
    if inline_code.is_none() && update_fence(line, fence) {
        return true;
    }
    if fence.is_some() {
        return true;
    }

    let protected = inline_code.is_some();
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'`' {
            let length = delimiter_length(bytes, index, b'`');
            *inline_code = match *inline_code {
                Some(open_length) if open_length == length => None,
                Some(open_length) => Some(open_length),
                None => Some(length),
            };
            index += length;
        } else {
            index += 1;
        }
    }
    protected
}

fn update_fence(line: &str, fence: &mut Option<(u8, usize)>) -> bool {
    let trimmed = fence_content(line);
    let Some(delimiter) = trimmed
        .as_bytes()
        .first()
        .copied()
        .filter(|delimiter| matches!(delimiter, b'`' | b'~'))
    else {
        return false;
    };
    let length = delimiter_length(trimmed.as_bytes(), 0, delimiter);
    if length < 3 {
        return false;
    }
    match fence {
        Some((open_delimiter, open_length))
            if *open_delimiter == delimiter
                && length >= *open_length
                && trimmed[length..].trim().is_empty() =>
        {
            *fence = None;
            true
        }
        Some(_) => false,
        None => {
            *fence = Some((delimiter, length));
            true
        }
    }
}

fn fence_content(line: &str) -> &str {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix('>')
        .map(str::trim_start)
        .unwrap_or(trimmed)
}

fn delimiter_length(bytes: &[u8], start: usize, delimiter: u8) -> usize {
    bytes[start..]
        .iter()
        .take_while(|byte| **byte == delimiter)
        .count()
}

fn parse_wiki_link(value: &str, embed: bool) -> Result<WikiLink, String> {
    let mut parts = value.split('|');
    let target = parts.next().expect("split has first item").trim();
    let alias = parts.next().map(str::trim);
    if parts.next().is_some() {
        return Err("Obsidian wikilink has more than one alias separator".to_owned());
    }
    if alias.is_some_and(str::is_empty) {
        return Err("Obsidian wikilink alias cannot be empty".to_owned());
    }

    let (path, fragment) = target
        .split_once('#')
        .map_or((target, None), |(path, fragment)| {
            (path.trim(), Some(fragment.trim()))
        });
    if path.is_empty() && fragment.is_none() {
        return Err("Obsidian wikilink target cannot be empty".to_owned());
    }
    if !path.is_empty() {
        validate_path(path)?;
    }

    let subpath = match fragment {
        None => None,
        Some("") => return Err("Obsidian wikilink subpath cannot be empty".to_owned()),
        Some(page) if page.starts_with("page=") => {
            let value = page
                .strip_prefix("page=")
                .expect("checked page prefix")
                .parse::<u32>()
                .map_err(|_| "Obsidian PDF page must be a positive integer".to_owned())?;
            if value == 0 {
                return Err("Obsidian PDF page must be a positive integer".to_owned());
            }
            Some(WikiSubpath::PdfPage(value))
        }
        Some(block) if block.starts_with('^') => {
            let block = block.strip_prefix('^').expect("checked block prefix");
            if block.is_empty() {
                return Err("Obsidian block ID cannot be empty".to_owned());
            }
            Some(WikiSubpath::Block(block.to_owned()))
        }
        Some(heading) => Some(WikiSubpath::Heading(heading.to_owned())),
    };
    if path.is_empty()
        && !matches!(
            subpath,
            Some(WikiSubpath::Heading(_)) | Some(WikiSubpath::Block(_))
        )
    {
        return Err("Obsidian same-note wikilink must target a heading or block".to_owned());
    }

    Ok(WikiLink {
        embed,
        path: (!path.is_empty()).then(|| path.to_owned()),
        subpath,
        alias: alias.map(str::to_owned),
    })
}

fn validate_path(path: &str) -> Result<(), String> {
    let candidate = std::path::Path::new(path);
    if candidate.is_absolute()
        || !candidate
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err("Obsidian wikilink path must be vault-relative without traversal".to_owned());
    }
    Ok(())
}

pub(super) fn resolve_wiki_link(
    repository: &HeldDirectory,
    source_path: &str,
    link: &WikiLink,
) -> Result<ResolvedWikiLink, AppError> {
    let target = match &link.path {
        Some(path) => resolve_target(repository, source_path, path)?,
        None => PathBuf::from(source_path),
    };
    let target_display = target
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(source_path);
    let display = link
        .alias
        .clone()
        .unwrap_or_else(|| target_display.to_owned());

    let (heading_id, pdf_page) = match &link.subpath {
        None => (None, None),
        Some(WikiSubpath::Block(_)) => {
            return Err(AppError::invalid_input(
                "knowledge.obsidian.block",
                "Obsidian block links are unsupported until an immutable block contract exists",
            ));
        }
        Some(WikiSubpath::Heading(heading)) => {
            if target.extension().and_then(|extension| extension.to_str()) != Some("md") {
                return Err(AppError::invalid_input(
                    "knowledge.obsidian.heading_target",
                    "Obsidian heading link must target Markdown",
                ));
            }
            let heading_id = resolve_heading(repository, &target, heading)?;
            (Some(heading_id), None)
        }
        Some(WikiSubpath::PdfPage(page)) => {
            if target.extension().and_then(|extension| extension.to_str()) != Some("pdf") {
                return Err(AppError::invalid_input(
                    "knowledge.obsidian.pdf_page_target",
                    "Obsidian PDF page link must target a PDF",
                ));
            }
            (None, Some(*page))
        }
    };

    if link.embed && target.extension().and_then(|extension| extension.to_str()) != Some("pdf") {
        return Err(AppError::invalid_input(
            "knowledge.obsidian.embed",
            "Obsidian embeds must target a PDF for the offline reader",
        ));
    }

    Ok(ResolvedWikiLink {
        target,
        heading_id,
        pdf_page,
        embed: link.embed,
        display,
    })
}

fn resolve_target(
    repository: &HeldDirectory,
    source_path: &str,
    path: &str,
) -> Result<PathBuf, AppError> {
    let candidate = Path::new(path);
    let candidate = if has_vault_root(candidate) {
        candidate.to_path_buf()
    } else {
        Path::new(source_path)
            .parent()
            .ok_or_else(|| {
                AppError::invalid_input(
                    "knowledge.obsidian.source_path",
                    "Obsidian source path must include a parent directory",
                )
            })?
            .join(candidate)
    };
    let target = if repository.regular_file_exists(&candidate, "Obsidian wikilink target")? {
        candidate
    } else if candidate.extension().is_some() {
        candidate
    } else {
        candidate.with_extension("md")
    };
    if !repository.regular_file_or_directory_exists(&target, "Obsidian wikilink target")? {
        return Err(AppError::invalid_input(
            "knowledge.obsidian.target",
            format!("Obsidian wikilink target is missing: {}", target.display()),
        ));
    }
    if !repository.regular_file_exists(&target, "Obsidian wikilink target")? {
        return Err(AppError::invalid_input(
            "knowledge.obsidian.target_type",
            format!(
                "Obsidian wikilink target must be a regular file: {}",
                target.display()
            ),
        ));
    }
    Ok(target)
}

fn has_vault_root(path: &Path) -> bool {
    path.components()
        .next()
        .and_then(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .is_some_and(|root| {
            ["knowledge", "evidence", "content", "labs", "crates", "docs"].contains(&root)
        })
}

fn resolve_heading(
    repository: &HeldDirectory,
    target: &Path,
    heading: &str,
) -> Result<String, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(target, "Obsidian heading target", 512 * 1024)?
        .ok_or_else(|| {
            AppError::invalid_input(
                "knowledge.obsidian.target",
                format!("Obsidian heading target is missing: {}", target.display()),
            )
        })?;
    let markdown = std::str::from_utf8(&bytes).map_err(|error| {
        AppError::invalid_input(
            "knowledge.obsidian.heading_encoding",
            format!("Obsidian heading target is not UTF-8: {error}"),
        )
    })?;
    match resolve_heading_id_from_markdown(markdown, &target.to_string_lossy(), heading) {
        Ok(Some(heading_id)) => Ok(heading_id),
        Ok(None) => Err(AppError::invalid_input(
            "knowledge.obsidian.heading",
            format!(
                "Obsidian wikilink heading is missing in {}: {heading}",
                target.display()
            ),
        )),
        Err(()) => Err(AppError::invalid_input(
            "knowledge.obsidian.heading",
            format!(
                "Obsidian wikilink heading is ambiguous in {}: {heading}",
                target.display()
            ),
        )),
    }
}

pub(super) fn resolve_heading_id_from_markdown(
    markdown: &str,
    source_path: &str,
    reference: &str,
) -> Result<Option<String>, ()> {
    let aliases = canonical_heading_aliases(markdown, source_path);
    let wanted = BTreeSet::from([reference.to_owned(), slug(reference)]);
    let mut matches = BTreeSet::new();
    for alias in wanted {
        let Some(ids) = aliases.get(&alias) else {
            continue;
        };
        if ids.len() != 1 {
            return Err(());
        }
        matches.insert(ids[0].as_str());
    }
    match matches.len() {
        0 => Ok(None),
        1 => Ok(matches.into_iter().next().map(str::to_owned)),
        _ => Err(()),
    }
}

pub(super) fn canonical_heading_map(markdown: &str, source_path: &str) -> BTreeMap<String, String> {
    canonical_heading_aliases(markdown, source_path)
        .into_iter()
        .filter_map(|(alias, ids)| {
            (ids.len() == 1).then(|| {
                (
                    alias,
                    ids.into_iter().next().expect("single canonical heading ID"),
                )
            })
        })
        .collect()
}

fn canonical_heading_aliases(markdown: &str, source_path: &str) -> BTreeMap<String, Vec<String>> {
    let mut options = Options::empty();
    if source_path.starts_with("knowledge/crouzeix_conjecture/") {
        options |= Options::ENABLE_HEADING_ATTRIBUTES;
    }
    let mut aliases = BTreeMap::<String, Vec<String>>::new();
    let mut current = None::<(Option<String>, String)>;
    for event in Parser::new_ext(markdown, options) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2 | HeadingLevel::H3,
                id,
                ..
            }) => current = Some((id.map(|value| value.into_string()), String::new())),
            Event::Text(text) | Event::Code(text) | Event::InlineMath(text) => {
                if let Some((_, value)) = &mut current {
                    value.push_str(&text);
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((explicit_id, value)) = current.take() {
                    let canonical_id = explicit_id.unwrap_or_else(|| slug(&value));
                    for alias in BTreeSet::from([canonical_id.clone(), value, slug(&canonical_id)])
                    {
                        aliases.entry(alias).or_default().push(canonical_id.clone());
                    }
                }
            }
            _ => {}
        }
    }
    aliases
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
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_alias_and_heading_links() {
        assert_eq!(
            parse_wiki_links("[[knowledge/darwinx/darwinx_index|DarwinX]]").unwrap(),
            vec![WikiLink {
                embed: false,
                path: Some("knowledge/darwinx/darwinx_index".into()),
                subpath: None,
                alias: Some("DarwinX".into()),
            }]
        );
        assert_eq!(
            parse_wiki_links(
                "[[knowledge/darwinx/claim_evidence_ledger#DX-024: Durable capability|DX-024]]"
            )
            .unwrap(),
            vec![WikiLink {
                embed: false,
                path: Some("knowledge/darwinx/claim_evidence_ledger".into()),
                subpath: Some(WikiSubpath::Heading("DX-024: Durable capability".into())),
                alias: Some("DX-024".into()),
            }]
        );
    }

    #[test]
    fn ignores_links_inside_code() {
        assert_eq!(
            parse_wiki_links(
                "``[[knowledge/rsi/rsi_index]]``\n~~~md\n[[knowledge/rsi/rsi_index]]\n~~~\n````md\n[[knowledge/rsi/rsi_index]]\n````\n[[knowledge/rsi/rsi_index]]"
            )
            .unwrap(),
            vec![WikiLink {
                embed: false,
                path: Some("knowledge/rsi/rsi_index".into()),
                subpath: None,
                alias: None,
            }]
        );
    }

    #[test]
    fn preserves_multiline_inline_code_state_while_parsing_and_rewriting() {
        let markdown =
            "`literal\n[[knowledge/rsi/rsi_index]]\nliteral`\n[[knowledge/rsi/rsi_index|RSI]]";
        assert_eq!(
            parse_wiki_links(markdown).unwrap(),
            vec![WikiLink {
                embed: false,
                path: Some("knowledge/rsi/rsi_index".into()),
                subpath: None,
                alias: Some("RSI".into()),
            }]
        );
        assert_eq!(
            rewrite_wiki_links(markdown, |link| {
                format!("[{}](target)", link.alias.as_deref().unwrap_or("missing"))
            })
            .unwrap(),
            "`literal\n[[knowledge/rsi/rsi_index]]\nliteral`\n[RSI](target)"
        );
    }

    #[test]
    fn ignores_links_inside_blockquote_fences() {
        assert_eq!(
            parse_wiki_links(
                "> ```md\n> [[knowledge/rsi/rsi_index]]\n> ```\n[[knowledge/rsi/rsi_index]]"
            )
            .unwrap(),
            vec![WikiLink {
                embed: false,
                path: Some("knowledge/rsi/rsi_index".into()),
                subpath: None,
                alias: None,
            }]
        );
    }

    #[test]
    fn accepts_longer_whitespace_only_fence_closers() {
        assert_eq!(
            parse_wiki_links(
                "~~~md\n[[knowledge/rsi/rsi_index]]\n~~~~  \n[[knowledge/rsi/rsi_index]]"
            )
            .unwrap(),
            vec![WikiLink {
                embed: false,
                path: Some("knowledge/rsi/rsi_index".into()),
                subpath: None,
                alias: None,
            }]
        );
    }

    #[test]
    fn rejects_fence_closers_with_non_whitespace_suffixes() {
        let error =
            parse_wiki_links("```md\n[[knowledge/rsi/rsi_index]]\n``` still code").unwrap_err();

        assert_eq!(error, "unclosed Markdown code fence");
    }

    #[test]
    fn parses_block_links_and_rejects_them_until_a_block_contract_exists() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("knowledge/example")).unwrap();
        fs::write(
            repo.path().join("knowledge/example/note.md"),
            "# Note\n\nparagraph ^block-id\n",
        )
        .unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();
        let link = parse_wiki_links("[[knowledge/example/note#^block-id|Block]]")
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(
            link.subpath,
            Some(WikiSubpath::Block("block-id".to_owned()))
        );
        assert_eq!(
            resolve_wiki_link(&directory, "knowledge/example/source.md", &link)
                .unwrap_err()
                .code(),
            "knowledge.obsidian.block"
        );
    }

    #[test]
    fn rejects_malformed_links() {
        for value in [
            "[[../../etc/passwd]]",
            "[[knowledge/darwinx/darwinx_index|]]",
            "[[knowledge/darwinx/darwinx_index|first|second]]",
            "[[knowledge/darwinx/darwinx_index",
            "[[#]]",
            "[[evidence/source.pdf#page=0]]",
        ] {
            assert!(parse_wiki_links(value).is_err(), "{value}");
        }
    }

    #[test]
    fn rewrites_links_without_touching_code() {
        let rewritten = rewrite_wiki_links(
            "[[knowledge/rsi/rsi_index|RSI]] ``[[knowledge/rsi/rsi_index]]``\n~~~md\n[[knowledge/rsi/rsi_index]]\n~~~\n````md\n[[knowledge/rsi/rsi_index]]\n````\n",
            |link| format!("[{}](target)", link.alias.as_deref().unwrap_or("missing")),
        )
        .unwrap();
        assert_eq!(
            rewritten,
            "[RSI](target) ``[[knowledge/rsi/rsi_index]]``\n~~~md\n[[knowledge/rsi/rsi_index]]\n~~~\n````md\n[[knowledge/rsi/rsi_index]]\n````\n"
        );
    }

    #[test]
    fn resolves_note_heading_and_pdf_page() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("knowledge/example")).unwrap();
        fs::create_dir_all(repo.path().join("evidence/example")).unwrap();
        fs::write(
            repo.path().join("knowledge/example/note.md"),
            "# Note\n\n## Result boundary\n\nText.\n",
        )
        .unwrap();
        fs::write(repo.path().join("evidence/example/paper.pdf"), b"%PDF").unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();

        let note = parse_wiki_links("[[knowledge/example/note#Result boundary|Result]]")
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(
            resolve_wiki_link(&directory, "knowledge/example/source.md", &note).unwrap(),
            ResolvedWikiLink {
                target: PathBuf::from("knowledge/example/note.md"),
                heading_id: Some("result-boundary".into()),
                pdf_page: None,
                embed: false,
                display: "Result".into(),
            }
        );

        let pdf = parse_wiki_links("![[evidence/example/paper.pdf#page=2|Paper]]")
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(
            resolve_wiki_link(&directory, "knowledge/example/source.md", &pdf).unwrap(),
            ResolvedWikiLink {
                target: PathBuf::from("evidence/example/paper.pdf"),
                heading_id: None,
                pdf_page: Some(2),
                embed: true,
                display: "Paper".into(),
            }
        );
    }

    #[test]
    fn resolves_extensionless_evidence_files_without_appending_markdown_suffix() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("evidence/example")).unwrap();
        fs::write(repo.path().join("evidence/example/REVISION"), "abc123\n").unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();
        let link = parse_wiki_links("[[evidence/example/REVISION|Revision]]")
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(
            resolve_wiki_link(&directory, "knowledge/example/source.md", &link)
                .unwrap()
                .target,
            PathBuf::from("evidence/example/REVISION")
        );
    }

    #[test]
    fn resolves_documentation_links_from_the_repository_root() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("docs/writing-style")).unwrap();
        fs::write(
            repo.path().join("docs/writing-style/STYLE_GUIDE.md"),
            "# Style guide\n",
        )
        .unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();
        let link = parse_wiki_links("[[docs/writing-style/STYLE_GUIDE|Style guide]]")
            .unwrap()
            .pop()
            .unwrap();

        assert_eq!(
            resolve_wiki_link(&directory, "knowledge/example/source.md", &link)
                .unwrap()
                .target,
            PathBuf::from("docs/writing-style/STYLE_GUIDE.md")
        );
    }

    #[test]
    fn rejects_non_pdf_embeds() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("knowledge/example")).unwrap();
        fs::create_dir_all(repo.path().join("evidence/example")).unwrap();
        fs::write(
            repo.path().join("knowledge/example/note.md"),
            "# Note\n\n## Result\n",
        )
        .unwrap();
        fs::write(repo.path().join("evidence/example/blob.bin"), b"blob").unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();

        for source in [
            "![[knowledge/example/note]]",
            "![[knowledge/example/note#Result]]",
            "![[evidence/example/blob.bin]]",
        ] {
            let link = parse_wiki_links(source).unwrap().pop().unwrap();
            assert_eq!(
                resolve_wiki_link(&directory, "knowledge/example/source.md", &link)
                    .unwrap_err()
                    .code(),
                "knowledge.obsidian.embed"
            );
        }
    }

    #[test]
    fn rejects_missing_or_ambiguous_heading() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("knowledge/example")).unwrap();
        fs::write(
            repo.path().join("knowledge/example/note.md"),
            "# Note\n\n## Duplicate\n\nA.\n\n## Duplicate\n\nB.\n",
        )
        .unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();
        let link = parse_wiki_links("[[knowledge/example/note#Duplicate]]")
            .unwrap()
            .pop()
            .unwrap();
        let error =
            resolve_wiki_link(&directory, "knowledge/example/source.md", &link).unwrap_err();
        assert_eq!(error.code(), "knowledge.obsidian.heading");
    }

    #[test]
    fn resolves_crouzeix_explicit_heading_ids_and_regular_slugs() {
        let repo = fixture();
        fs::create_dir_all(repo.path().join("knowledge/crouzeix_conjecture")).unwrap();
        fs::write(
            repo.path().join("knowledge/crouzeix_conjecture/note.md"),
            "# Note\n\n## Explicit heading {#explicit-id}\n\n## Regular heading\n",
        )
        .unwrap();
        let directory = HeldDirectory::open(repo.path(), "test repository").unwrap();

        for (fragment, expected) in [
            ("explicit-id", "explicit-id"),
            ("Regular heading", "regular-heading"),
        ] {
            let link = parse_wiki_links(&format!(
                "[[knowledge/crouzeix_conjecture/note#{fragment}]]"
            ))
            .unwrap()
            .pop()
            .unwrap();
            assert_eq!(
                resolve_wiki_link(&directory, "knowledge/crouzeix_conjecture/source.md", &link)
                    .unwrap()
                    .heading_id
                    .as_deref(),
                Some(expected)
            );
        }
    }

    fn fixture() -> TempDir {
        tempfile::tempdir().unwrap()
    }
}
