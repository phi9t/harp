use std::path::{Path, PathBuf};

use crate::{corpus, AppError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WikiLinkResolution {
    pub target: PathBuf,
    pub heading_id: Option<String>,
    pub requested_heading: Option<String>,
    pub pdf_page: Option<u32>,
    pub embed: bool,
    pub display: String,
}

pub fn resolve_wiki_links(
    repository_root: &Path,
    source_path: &Path,
    markdown: &str,
) -> Result<Vec<WikiLinkResolution>, AppError> {
    corpus::resolve_wiki_links(repository_root, source_path, markdown)
}
