//! Select immutable local evidence without applying today's input checks to history.

use super::*;
use crate::fs::HeldDirectory;

const POINTER: &str = "local_formalization.current.json";
const GENERATIONS: &str = "local_formalization_generations";
const MAX_POINTER_BYTES: usize = 64 * 1024;
const MAX_GENERATIONS: usize = 128;
const MAX_BUNDLE_ENTRIES: usize = 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    schema_version: String,
    active_generation: String,
    generations: Vec<Generation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Generation {
    id: String,
    bundle_sha256: String,
}

fn error(detail: impl Into<String>) -> AppError {
    invalid(
        "sources.crouzeix.local_formalization_selection",
        detail.into(),
    )
}

fn valid_id(id: &str) -> bool {
    id == "legacy"
        || (id.len() == 32
            && id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

fn generation_bundle(id: &str) -> PathBuf {
    if id == "legacy" {
        Path::new(ROOT).join(LOCAL_FORMALIZATION_ROOT)
    } else {
        Path::new(ROOT).join(GENERATIONS).join(id)
    }
}

pub(super) fn verify(repo_root: &Path) -> Result<Option<BTreeSet<String>>, AppError> {
    let pointer = Path::new(ROOT).join(POINTER);
    let generations_root = repo_root.join(ROOT).join(GENERATIONS);
    match fs::symlink_metadata(repo_root.join(&pointer)) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return match fs::symlink_metadata(&generations_root) {
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
                _ => Err(error(
                    "generation directory exists without a selection catalog",
                )),
            };
        }
        Err(err) => return Err(error(format!("cannot inspect evidence selection: {err}"))),
        Ok(_) => {}
    }
    let repository = HeldDirectory::open(repo_root, "local evidence selection")?;
    let bytes = repository
        .read_optional_regular_single_link_file_bounded(
            &pointer,
            "local evidence selection",
            MAX_POINTER_BYTES,
        )?
        .ok_or_else(|| error("evidence selection disappeared during verification"))?;
    let selection: Selection = serde_json::from_slice(&bytes)
        .map_err(|err| error(format!("invalid local evidence selection: {err}")))?;
    if selection.schema_version != "crouzeix-local-formalization-selection/v1"
        || selection.generations.is_empty()
        || selection.generations.len() > MAX_GENERATIONS
    {
        return Err(error(
            "selection requires the supported schema and 1..128 generations",
        ));
    }
    let mut ids = BTreeSet::new();
    for generation in &selection.generations {
        if !valid_id(&generation.id)
            || !valid_digest(&generation.bundle_sha256)
            || !ids.insert(generation.id.as_str())
        {
            return Err(error(
                "generation IDs and digests must be canonical and IDs unique",
            ));
        }
    }
    if !ids.contains("legacy") || !ids.contains(selection.active_generation.as_str()) {
        return Err(error(
            "catalog must contain legacy and the selected active generation",
        ));
    }
    let expected = ids
        .iter()
        .copied()
        .filter(|id| *id != "legacy")
        .collect::<BTreeSet<_>>();
    let mut actual = BTreeSet::new();
    match fs::symlink_metadata(&generations_root) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && expected.is_empty() => {}
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            for entry in fs::read_dir(&generations_root).map_err(|err| error(err.to_string()))? {
                let entry = entry.map_err(|err| error(err.to_string()))?;
                if !entry
                    .file_type()
                    .map_err(|err| error(err.to_string()))?
                    .is_dir()
                {
                    return Err(error(
                        "generation root may contain only catalogued directories",
                    ));
                }
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| error("generation name is not UTF-8"))?;
                actual.insert(name);
                if actual.len() > MAX_GENERATIONS {
                    return Err(error("too many generation directories"));
                }
            }
        }
        _ => return Err(error("generation root must be a regular directory")),
    }
    if actual.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return Err(error(
            "generation directory roster differs from the selection catalog",
        ));
    }
    let mut files = BTreeSet::from([POINTER.to_owned()]);
    for generation in &selection.generations {
        let bundle = generation_bundle(&generation.id);
        let manifest = bundle.join("manifest.tsv");
        let absolute = repo_root.join(&bundle);
        let mut records = BTreeMap::new();
        for (index, entry) in WalkDir::new(&absolute)
            .follow_links(false)
            .min_depth(1)
            .into_iter()
            .enumerate()
        {
            if index >= MAX_BUNDLE_ENTRIES {
                return Err(error("bundle inventory exceeds its entry bound"));
            }
            let entry = entry.map_err(|err| error(err.to_string()))?;
            if entry.file_type().is_symlink()
                || (!entry.file_type().is_dir() && !entry.file_type().is_file())
            {
                return Err(error("bundle contains a symlink or special file"));
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(&absolute)
                .expect("walked bundle member");
            let relative = relative
                .to_str()
                .ok_or_else(|| error("bundle path is not UTF-8"))?;
            if relative.contains(['\t', '\n', '\r']) {
                return Err(error("bundle path contains record delimiters"));
            }
            let path = bundle.join(relative);
            let maximum = if relative == "manifest.tsv" {
                LOCAL_MANIFEST_MAX_BYTES
            } else {
                LOCAL_ARTIFACT_MAX_BYTES
            };
            let bytes = repository
                .read_optional_regular_single_link_file_bounded(
                    &path,
                    "immutable evidence member",
                    maximum as usize,
                )?
                .ok_or_else(|| error("bundle member disappeared"))?;
            records.insert(relative.to_owned(), digest_bytes(&bytes));
            files.insert(slash_path(
                path.strip_prefix(ROOT).expect("evidence-relative member"),
            ));
        }
        verify_local_formalization_bundle_roster(&absolute, &manifest)?;
        let digest = digest_bytes(
            records
                .into_iter()
                .map(|(path, digest)| format!("{path}\t{digest}\n"))
                .collect::<String>()
                .as_bytes(),
        );
        if digest != generation.bundle_sha256 {
            return Err(error(format!(
                "immutable bundle digest mismatch: {}",
                generation.id
            )));
        }
    }
    let active_manifest = generation_bundle(&selection.active_generation).join("manifest.tsv");
    verify_local_formalization_manifest(repo_root, &active_manifest)?;
    Ok(Some(files))
}
