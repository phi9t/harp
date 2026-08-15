use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::fs::HeldDirectory;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum WikiSubpath {
    Heading(String),
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
    let mut in_fence = false;

    for line in markdown.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let bytes = line.as_bytes();
        let mut index = 0;
        let mut in_code = false;
        while index < bytes.len() {
            if bytes[index] == b'`' {
                in_code = !in_code;
                index += 1;
                continue;
            }
            if in_code {
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

    if in_fence {
        return Err("unclosed Markdown code fence".to_owned());
    }
    Ok(links)
}

pub(super) fn rewrite_wiki_links(
    markdown: &str,
    mut replacement: impl FnMut(&WikiLink) -> String,
) -> Result<String, String> {
    let mut output = String::with_capacity(markdown.len());
    let mut in_fence = false;

    for line in markdown.split_inclusive('\n') {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            output.push_str(line);
            continue;
        }
        if in_fence {
            output.push_str(line);
            continue;
        }

        let bytes = line.as_bytes();
        let mut index = 0;
        let mut in_code = false;
        while index < bytes.len() {
            if bytes[index] == b'`' {
                in_code = !in_code;
                output.push('`');
                index += 1;
                continue;
            }
            if in_code {
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

    if in_fence {
        return Err("unclosed Markdown code fence".to_owned());
    }
    Ok(output)
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
        Some(heading) => Some(WikiSubpath::Heading(heading.to_owned())),
    };
    if path.is_empty() && !matches!(subpath, Some(WikiSubpath::Heading(_))) {
        return Err("Obsidian same-note wikilink must target a heading".to_owned());
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

    if link.embed && heading_id.is_some() {
        return Err(AppError::invalid_input(
            "knowledge.obsidian.embed_heading",
            "Obsidian heading embeds are not supported by the offline reader",
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
            .expect("canonical source has a parent")
            .join(candidate)
    };
    let target = if candidate.extension().is_some() {
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
        .is_some_and(|root| ["knowledge", "evidence", "content", "labs", "crates"].contains(&root))
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
    let wanted = slug(heading);
    let matches = markdown
        .lines()
        .filter_map(|line| {
            line.strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))
                .map(slug)
        })
        .filter(|id| id == &wanted)
        .count();
    match matches {
        1 => Ok(wanted),
        0 => Err(AppError::invalid_input(
            "knowledge.obsidian.heading",
            format!(
                "Obsidian wikilink heading is missing in {}: {heading}",
                target.display()
            ),
        )),
        _ => Err(AppError::invalid_input(
            "knowledge.obsidian.heading",
            format!(
                "Obsidian wikilink heading is ambiguous in {}: {heading}",
                target.display()
            ),
        )),
    }
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
                "`[[knowledge/rsi/rsi_index]]`\n```md\n[[knowledge/rsi/rsi_index]]\n```\n[[knowledge/rsi/rsi_index]]"
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
            "[[knowledge/rsi/rsi_index|RSI]] `[[knowledge/rsi/rsi_index]]`\n```md\n[[knowledge/rsi/rsi_index]]\n```\n",
            |link| format!("[{}](target)", link.alias.as_deref().unwrap_or("missing")),
        )
        .unwrap();
        assert_eq!(
            rewritten,
            "[RSI](target) `[[knowledge/rsi/rsi_index]]`\n```md\n[[knowledge/rsi/rsi_index]]\n```\n"
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

    fn fixture() -> TempDir {
        tempfile::tempdir().unwrap()
    }
}
