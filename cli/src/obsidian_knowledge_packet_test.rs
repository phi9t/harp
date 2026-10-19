use std::path::{Path, PathBuf};
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}
#[test]
fn public_wiki_resolver_rejects_a_repository_root_source_path() {
    let root = workspace_root();
    let error = harp::knowledge::resolve_wiki_links(&root, &root, "[[note]]")
        .expect_err("source must be a document");
    assert_eq!(error.code(), "knowledge.obsidian.source_path");
}
