use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::canonical::{json_bytes, json_bytes_with_newline, sha256_hex, sha256_id};
use super::state::StateRoot;
use super::workflow::{PlaybookItem, VerificationRecipe, WorkflowPackage};
use super::{
    ContextItemKind, ProviderId, WorkflowId, CONTEXT_ITEM_SCHEMA, CONTEXT_TEMPLATE_SCHEMA,
    RELEASE_IDENTITY_SCHEMA, RELEASE_SCHEMA,
};
use crate::AppError;

const RELEASE_MEMBERS: [&str; 4] = [
    "context_template.json",
    "identity.json",
    "items.jsonl",
    "manifest.json",
];
const HASHED_MEMBERS: [&str; 3] = ["context_template.json", "identity.json", "items.jsonl"];
const MAX_RELEASE_MEMBER_BYTES: usize = 1024 * 1024;
const RELEASE_AUTHORIZATION_SCHEMA: &str = "harp-context-release-authorization/v1";
const MAX_RELEASE_AUTHORIZATION_BYTES: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseIdentity {
    pub schema_version: String,
    pub packages: BTreeMap<WorkflowId, String>,
    pub selector_version: String,
    pub renderer_version: String,
    pub context_template_schema: String,
    pub context_budgets: BTreeMap<WorkflowId, u32>,
    pub compatible_providers: BTreeSet<ProviderId>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub schema_version: String,
    pub release_id: String,
    pub members: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContextTemplate {
    pub schema_version: String,
    pub workflows: BTreeMap<WorkflowId, WorkflowContextTemplate>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowContextTemplate {
    pub context_budget_tokens: u32,
    pub required_sections: Vec<String>,
    pub verification_recipes: Vec<VerificationRecipe>,
    pub verified_success: Vec<String>,
    pub weak_signals: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct ReleaseItem {
    pub schema_version: String,
    pub workflow: WorkflowId,
    pub id: String,
    pub kind: ContextItemKind,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledRelease {
    release_id: String,
    identity: ReleaseIdentity,
    manifest: ReleaseManifest,
    context_template: ContextTemplate,
    items: Vec<ReleaseItem>,
    identity_bytes: Vec<u8>,
    manifest_bytes: Vec<u8>,
    context_template_bytes: Vec<u8>,
    items_bytes: Vec<u8>,
}

impl CompiledRelease {
    pub fn release_id(&self) -> &str {
        &self.release_id
    }

    pub fn identity(&self) -> &ReleaseIdentity {
        &self.identity
    }

    pub fn manifest(&self) -> &ReleaseManifest {
        &self.manifest
    }

    pub fn context_template(&self) -> &ContextTemplate {
        &self.context_template
    }

    pub fn items(&self) -> &[ReleaseItem] {
        &self.items
    }

    fn members(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        BTreeMap::from([
            (
                PathBuf::from("context_template.json"),
                self.context_template_bytes.clone(),
            ),
            (PathBuf::from("identity.json"), self.identity_bytes.clone()),
            (PathBuf::from("items.jsonl"), self.items_bytes.clone()),
            (PathBuf::from("manifest.json"), self.manifest_bytes.clone()),
        ])
    }

    fn validate_invariants(&self) -> Result<(), AppError> {
        validate_release_id(&self.release_id).map_err(release_invariant_from)?;
        validate_identity(&self.identity).map_err(release_invariant_from)?;
        validate_context_template(&self.context_template, &self.identity)
            .map_err(release_invariant_from)?;
        validate_manifest(&self.manifest, &self.release_id).map_err(release_invariant_from)?;
        if self.identity_bytes != json_bytes(&self.identity)?
            || self.context_template_bytes != json_bytes_with_newline(&self.context_template)?
            || self.items_bytes != json_lines(&self.items)?
            || self.manifest_bytes != json_bytes_with_newline(&self.manifest)?
            || self.release_id != sha256_id(&self.identity_bytes)
        {
            return Err(release_invariant(
                "compiled release fields do not match their canonical bytes",
            ));
        }
        let expected_hashes = member_hashes(
            &self.identity_bytes,
            &self.context_template_bytes,
            &self.items_bytes,
        );
        if self.manifest.members != expected_hashes {
            return Err(release_invariant(
                "compiled release manifest does not match member bytes",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ReleaseAuthorization {
    schema_version: String,
    release_id: String,
    members: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ReleaseSummary {
    pub release_id: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ReleaseList {
    pub releases: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct ReleaseInspection {
    pub release_id: String,
    pub identity: ReleaseIdentity,
    pub manifest: ReleaseManifest,
    pub context_template: ContextTemplate,
    pub items: Vec<ReleaseItem>,
}

pub fn compile_baseline_release() -> Result<CompiledRelease, AppError> {
    let packages = WorkflowId::ALL
        .into_iter()
        .map(|workflow| WorkflowPackage::builtin(workflow).map(|package| (workflow, package)))
        .collect::<Result<Vec<_>, _>>()?;

    let selector_version = shared_version(
        &packages,
        |package| package.manifest().selector_version.as_str(),
        "selector",
    )?;
    let renderer_version = shared_version(
        &packages,
        |package| package.manifest().renderer_version.as_str(),
        "renderer",
    )?;

    let mut package_ids = BTreeMap::new();
    let mut context_budgets = BTreeMap::new();
    let mut workflows = BTreeMap::new();
    let mut items = Vec::new();
    for (workflow, package) in &packages {
        let manifest = package.manifest();
        package_ids.insert(*workflow, sha256_id(&json_bytes(manifest)?));
        context_budgets.insert(*workflow, manifest.context_budget_tokens);
        workflows.insert(
            *workflow,
            WorkflowContextTemplate {
                context_budget_tokens: manifest.context_budget_tokens,
                required_sections: package.context_schema().required_sections.clone(),
                verification_recipes: package.verification().recipes.clone(),
                verified_success: package.outcome_rules().verified_success.clone(),
                weak_signals: package.outcome_rules().weak_signals.clone(),
            },
        );

        let mut playbook = package.playbook().iter().collect::<Vec<_>>();
        playbook.sort_unstable_by(|left, right| left.id.cmp(&right.id));
        items.extend(
            playbook
                .into_iter()
                .map(|item| release_item(*workflow, item)),
        );
    }

    let identity = ReleaseIdentity {
        schema_version: RELEASE_IDENTITY_SCHEMA.to_owned(),
        packages: package_ids,
        selector_version,
        renderer_version,
        context_template_schema: CONTEXT_TEMPLATE_SCHEMA.to_owned(),
        context_budgets,
        compatible_providers: BTreeSet::from([ProviderId::Trae, ProviderId::Codex]),
    };
    let identity_bytes = json_bytes(&identity)?;
    let release_id = sha256_id(&identity_bytes);

    let context_template = ContextTemplate {
        schema_version: CONTEXT_TEMPLATE_SCHEMA.to_owned(),
        workflows,
    };
    let context_template_bytes = json_bytes_with_newline(&context_template)?;
    let items_bytes = json_lines(&items)?;
    let members = member_hashes(&identity_bytes, &context_template_bytes, &items_bytes);
    let manifest = ReleaseManifest {
        schema_version: RELEASE_SCHEMA.to_owned(),
        release_id: release_id.clone(),
        members,
    };
    let manifest_bytes = json_bytes_with_newline(&manifest)?;

    Ok(CompiledRelease {
        release_id,
        identity,
        manifest,
        context_template,
        items,
        identity_bytes,
        manifest_bytes,
        context_template_bytes,
        items_bytes,
    })
}

pub fn compile_and_publish_baseline() -> Result<ReleaseSummary, AppError> {
    let release = compile_baseline_release()?;
    let state = StateRoot::open_from_environment()?;
    publish_release(&state, &release)?;
    Ok(ReleaseSummary {
        release_id: release.release_id,
    })
}

/// Lists only releases whose authorization commit marker and stored members validate.
///
/// Incomplete or release-invalid entries are invisible. State-layer errors such as namespace
/// races, symlinks, permissions failures, and unsupported secure state remain typed and visible.
pub fn list_releases() -> Result<ReleaseList, AppError> {
    let state = StateRoot::open_from_environment()?;
    Ok(ReleaseList {
        releases: list_releases_at(&state)?,
    })
}

/// Returns a release only after validating its ID, members, manifest, and authorization marker.
///
/// Missing, incomplete, or marker-mismatched releases fail closed with a typed release error.
/// Owner-only state has no external trust root in V1; arbitrary same-UID offline replacement of
/// both content and authorization is outside the authenticity guarantee.
pub fn inspect_release(release_id: &str) -> Result<ReleaseInspection, AppError> {
    let state = StateRoot::open_from_environment()?;
    inspect_release_at(&state, release_id)
}

/// Publishes release content and then its immutable authorization commit marker.
///
/// The authorization record protects against untrusted repository input, public API misuse, and
/// symlink, race, or namespace attacks during Harp operations. V1 uses owner-only local state
/// without an external trust root, so arbitrary same-UID offline rewriting of both release content
/// and authorization is outside its authenticity guarantee.
pub fn publish_release(state: &StateRoot, release: &CompiledRelease) -> Result<(), AppError> {
    publish_release_with_authorization(state, release, |state, release_id, bytes| {
        state.publish_release_authorization(release_id, bytes)
    })
}

fn publish_release_with_authorization<F>(
    state: &StateRoot,
    release: &CompiledRelease,
    publish_authorization: F,
) -> Result<(), AppError>
where
    F: FnOnce(&StateRoot, &str, &[u8]) -> Result<(), AppError>,
{
    release.validate_invariants()?;
    let members = release.members();
    state
        .publish_immutable_tree(&Path::new("releases").join(&release.release_id), &members)
        .map_err(map_publication_error)?;
    let authorization = ReleaseAuthorization {
        schema_version: RELEASE_AUTHORIZATION_SCHEMA.to_owned(),
        release_id: release.release_id.clone(),
        members: authorization_hashes(&members),
    };
    let authorization_bytes = json_bytes_with_newline(&authorization)?;
    publish_authorization(state, &release.release_id, &authorization_bytes)
        .map_err(map_publication_error)
}

/// Validates a release against its immutable authorization commit marker before returning data.
///
/// `release.integrity` means the local content is incomplete, tampered relative to its marker, or
/// lacks a marker. As with publication, V1 does not authenticate arbitrary same-UID offline
/// replacement of both owner-only content and its authorization without an external trust root.
fn inspect_release_at(state: &StateRoot, release_id: &str) -> Result<ReleaseInspection, AppError> {
    validate_release_id(release_id)?;
    let authorization = read_authorization(state, release_id)?;
    let release_path = Path::new("releases").join(release_id);
    let names = state
        .list_private_directory(&release_path)
        .map_err(map_release_content_error)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    let expected = RELEASE_MEMBERS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if names != expected {
        return Err(release_schema(format!(
            "release members must be exactly: {}",
            RELEASE_MEMBERS.join(", ")
        )));
    }

    let manifest_bytes = state
        .read_private_file_bounded(
            &release_path.join("manifest.json"),
            MAX_RELEASE_MEMBER_BYTES,
        )
        .map_err(map_release_content_error)?;
    let identity_bytes = state
        .read_private_file_bounded(
            &release_path.join("identity.json"),
            MAX_RELEASE_MEMBER_BYTES,
        )
        .map_err(map_release_content_error)?;
    let context_template_bytes = state
        .read_private_file_bounded(
            &release_path.join("context_template.json"),
            MAX_RELEASE_MEMBER_BYTES,
        )
        .map_err(map_release_content_error)?;
    let items_bytes = state
        .read_private_file_bounded(&release_path.join("items.jsonl"), MAX_RELEASE_MEMBER_BYTES)
        .map_err(map_release_content_error)?;
    let manifest: ReleaseManifest = parse_json(&manifest_bytes, "manifest")?;
    validate_manifest(&manifest, release_id)?;
    if json_bytes_with_newline(&manifest)? != manifest_bytes {
        return Err(release_schema("release manifest JSON is not canonical"));
    }

    let actual_hashes = BTreeMap::from([
        ("context_template.json", sha256_hex(&context_template_bytes)),
        ("identity.json", sha256_hex(&identity_bytes)),
        ("items.jsonl", sha256_hex(&items_bytes)),
    ]);
    for (member, actual) in actual_hashes {
        if manifest.members.get(member) != Some(&actual) {
            return Err(release_digest(format!("{member} digest mismatch")));
        }
    }
    if sha256_id(&identity_bytes) != release_id {
        return Err(release_digest(
            "identity digest does not match the release ID",
        ));
    }

    let identity: ReleaseIdentity = parse_json(&identity_bytes, "identity")?;
    validate_identity(&identity)?;
    if json_bytes(&identity)? != identity_bytes {
        return Err(release_schema("identity JSON is not canonical"));
    }
    let context_template: ContextTemplate =
        parse_json(&context_template_bytes, "context template")?;
    validate_context_template(&context_template, &identity)?;
    if json_bytes_with_newline(&context_template)? != context_template_bytes {
        return Err(release_schema("context template JSON is not canonical"));
    }
    let items = parse_items(&items_bytes, &identity)?;

    if authorization.members
        != authorization_hashes_from_bytes(
            &identity_bytes,
            &manifest_bytes,
            &context_template_bytes,
            &items_bytes,
        )
    {
        return Err(release_integrity(
            "stored release bytes do not match immutable Harp publication provenance",
        ));
    }

    Ok(ReleaseInspection {
        release_id: release_id.to_owned(),
        identity,
        manifest,
        context_template,
        items,
    })
}

fn list_releases_at(state: &StateRoot) -> Result<Vec<String>, AppError> {
    let entries = match state.list_private_directory(Path::new("releases")) {
        Ok(ids) => ids,
        Err(error) if error.code() == "state.missing" => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut releases = Vec::new();
    for id in entries {
        if id.starts_with(".harp-tree-") || validate_release_id(&id).is_err() {
            continue;
        }
        match read_authorization(state, &id) {
            Ok(_) => {}
            Err(error) if error.code().starts_with("release.") => continue,
            Err(error) => return Err(error),
        }
        match inspect_release_at(state, &id) {
            Ok(_) => releases.push(id),
            Err(error) if error.code().starts_with("release.") => {}
            Err(error) => return Err(error),
        }
    }
    Ok(releases)
}

fn shared_version(
    packages: &[(WorkflowId, WorkflowPackage)],
    version: impl Fn(&WorkflowPackage) -> &str,
    label: &'static str,
) -> Result<String, AppError> {
    let first = packages
        .first()
        .map(|(_, package)| version(package))
        .ok_or_else(|| release_schema("baseline release has no workflow packages"))?;
    if first.is_empty()
        || packages
            .iter()
            .any(|(_, package)| version(package) != first)
    {
        return Err(AppError::invalid_input(
            if label == "selector" {
                "release.selector_version"
            } else {
                "release.renderer_version"
            },
            format!("workflow packages do not share one {label} version"),
        ));
    }
    Ok(first.to_owned())
}

fn release_item(workflow: WorkflowId, item: &PlaybookItem) -> ReleaseItem {
    ReleaseItem {
        schema_version: item.schema_version.clone(),
        workflow,
        id: item.id.clone(),
        kind: item.kind,
        body: item.body.clone(),
    }
}

fn json_lines<T: Serialize>(items: &[T]) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    for item in items {
        bytes.extend(json_bytes_with_newline(item)?);
    }
    Ok(bytes)
}

fn validate_manifest(manifest: &ReleaseManifest, release_id: &str) -> Result<(), AppError> {
    if manifest.schema_version != RELEASE_SCHEMA {
        return Err(release_schema(format!(
            "release manifest schema must be {RELEASE_SCHEMA}"
        )));
    }
    if manifest.release_id != release_id {
        return Err(release_digest(
            "release manifest ID does not match its directory",
        ));
    }
    let names = manifest
        .members
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if names != HASHED_MEMBERS {
        return Err(release_schema(format!(
            "release manifest hashes must be exactly: {}",
            HASHED_MEMBERS.join(", ")
        )));
    }
    if manifest
        .members
        .values()
        .any(|digest| !valid_hex_digest(digest))
    {
        return Err(release_schema(
            "release manifest hashes must be lowercase SHA-256 digests",
        ));
    }
    Ok(())
}

fn validate_identity(identity: &ReleaseIdentity) -> Result<(), AppError> {
    if identity.schema_version != RELEASE_IDENTITY_SCHEMA {
        return Err(release_schema(format!(
            "release identity schema must be {RELEASE_IDENTITY_SCHEMA}"
        )));
    }
    let expected = WorkflowId::ALL.into_iter().collect::<BTreeSet<_>>();
    if identity.packages.keys().copied().collect::<BTreeSet<_>>() != expected
        || identity
            .context_budgets
            .keys()
            .copied()
            .collect::<BTreeSet<_>>()
            != expected
    {
        return Err(release_schema(
            "release identity must bind every approved workflow exactly once",
        ));
    }
    if identity.packages.values().any(|id| !valid_release_id(id))
        || identity.selector_version.is_empty()
        || identity.renderer_version.is_empty()
        || identity.context_template_schema != CONTEXT_TEMPLATE_SCHEMA
        || identity.context_budgets.values().any(|budget| *budget == 0)
        || identity.compatible_providers.is_empty()
    {
        return Err(release_schema("release identity contains an invalid field"));
    }
    Ok(())
}

fn validate_context_template(
    template: &ContextTemplate,
    identity: &ReleaseIdentity,
) -> Result<(), AppError> {
    if template.schema_version != CONTEXT_TEMPLATE_SCHEMA
        || template.workflows.keys().copied().collect::<BTreeSet<_>>()
            != WorkflowId::ALL.into_iter().collect()
    {
        return Err(release_schema(
            "context template has an invalid workflow set",
        ));
    }
    for (workflow, context) in &template.workflows {
        if identity.context_budgets.get(workflow) != Some(&context.context_budget_tokens) {
            return Err(release_schema(format!(
                "{workflow} context budget does not match release identity"
            )));
        }
    }
    Ok(())
}

fn parse_items(bytes: &[u8], identity: &ReleaseIdentity) -> Result<Vec<ReleaseItem>, AppError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| release_schema(format!("items.jsonl is not UTF-8: {error}")))?;
    let mut items = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.is_empty() {
            return Err(release_schema(format!(
                "items.jsonl line {} is empty",
                index + 1
            )));
        }
        let item: ReleaseItem = serde_json::from_str(line).map_err(|error| {
            release_schema(format!("invalid items.jsonl line {}: {error}", index + 1))
        })?;
        if !identity.packages.contains_key(&item.workflow) {
            return Err(release_schema(format!(
                "item {} has an unbound workflow",
                item.id
            )));
        }
        if item.schema_version != CONTEXT_ITEM_SCHEMA || item.id.is_empty() {
            return Err(release_schema(format!(
                "items.jsonl line {} has an invalid schema or item ID",
                index + 1
            )));
        }
        if json_bytes(&item)? != line.as_bytes() {
            return Err(release_schema(format!(
                "items.jsonl line {} is not canonical",
                index + 1
            )));
        }
        items.push(item);
    }
    if items.is_empty()
        || !bytes.ends_with(b"\n")
        || !items.windows(2).all(|pair| {
            (pair[0].workflow, pair[0].id.as_str()) < (pair[1].workflow, pair[1].id.as_str())
        })
    {
        return Err(release_schema(
            "release items must be nonempty and ordered by workflow then item ID",
        ));
    }
    Ok(items)
}

fn parse_json<T>(bytes: &[u8], label: &str) -> Result<T, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(bytes)
        .map_err(|error| release_schema(format!("invalid release {label}: {error}")))
}

fn validate_release_id(release_id: &str) -> Result<(), AppError> {
    if valid_release_id(release_id) {
        Ok(())
    } else {
        Err(AppError::invalid_input(
            "release.id",
            format!("invalid release ID: {release_id}"),
        ))
    }
}

fn valid_release_id(value: &str) -> bool {
    value.strip_prefix("sha256-").is_some_and(valid_hex_digest)
}

fn valid_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn map_publication_error(error: AppError) -> AppError {
    if error.code() == "state.immutable_collision" {
        AppError::invalid_input("release.conflict", error.message)
    } else {
        error
    }
}

fn map_authorization_read_error(error: AppError) -> AppError {
    if error.code() == "state.missing" {
        release_integrity("release has no immutable Harp publication provenance")
    } else {
        error
    }
}

fn map_release_content_error(error: AppError) -> AppError {
    if error.code() == "state.missing" {
        release_integrity("authorized release content is missing")
    } else {
        error
    }
}

fn read_authorization(
    state: &StateRoot,
    release_id: &str,
) -> Result<ReleaseAuthorization, AppError> {
    let bytes = state
        .read_release_authorization(release_id, MAX_RELEASE_AUTHORIZATION_BYTES)
        .map_err(map_authorization_read_error)?;
    let authorization: ReleaseAuthorization =
        parse_json(&bytes, "authorization").map_err(release_integrity_from)?;
    let expected_members = RELEASE_MEMBERS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if json_bytes_with_newline(&authorization).map_err(release_integrity_from)? != bytes
        || authorization.schema_version != RELEASE_AUTHORIZATION_SCHEMA
        || authorization.release_id != release_id
        || authorization
            .members
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>()
            != expected_members
        || authorization
            .members
            .values()
            .any(|digest| !valid_hex_digest(digest))
    {
        return Err(release_integrity(
            "release authorization commit marker is invalid",
        ));
    }
    Ok(authorization)
}

fn member_hashes(
    identity_bytes: &[u8],
    context_template_bytes: &[u8],
    items_bytes: &[u8],
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "context_template.json".to_owned(),
            sha256_hex(context_template_bytes),
        ),
        ("identity.json".to_owned(), sha256_hex(identity_bytes)),
        ("items.jsonl".to_owned(), sha256_hex(items_bytes)),
    ])
}

fn authorization_hashes(members: &BTreeMap<PathBuf, Vec<u8>>) -> BTreeMap<String, String> {
    members
        .iter()
        .map(|(name, bytes)| (name.to_string_lossy().into_owned(), sha256_hex(bytes)))
        .collect()
}

fn authorization_hashes_from_bytes(
    identity_bytes: &[u8],
    manifest_bytes: &[u8],
    context_template_bytes: &[u8],
    items_bytes: &[u8],
) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "context_template.json".to_owned(),
            sha256_hex(context_template_bytes),
        ),
        ("identity.json".to_owned(), sha256_hex(identity_bytes)),
        ("items.jsonl".to_owned(), sha256_hex(items_bytes)),
        ("manifest.json".to_owned(), sha256_hex(manifest_bytes)),
    ])
}

fn release_invariant_from(error: AppError) -> AppError {
    release_invariant(error.message)
}

fn release_integrity_from(error: AppError) -> AppError {
    release_integrity(error.message)
}

fn release_invariant(message: impl Into<String>) -> AppError {
    AppError::invalid_input("release.invariant", message)
}

fn release_schema(message: impl Into<String>) -> AppError {
    AppError::invalid_input("release.schema", message)
}

fn release_digest(message: impl Into<String>) -> AppError {
    AppError::invalid_input("release.digest", message)
}

fn release_integrity(message: impl Into<String>) -> AppError {
    AppError::invalid_input("release.integrity", message)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use super::*;
    use crate::context_control::state::StateRoot;
    use crate::context_control::workflow::WorkflowPackage;

    #[test]
    fn baseline_release_id_is_stable_and_excludes_time_and_repository_revision() {
        let first = compile_baseline_release().unwrap();
        let second = compile_baseline_release().unwrap();

        assert_eq!(first.release_id(), second.release_id());
        assert_eq!(first.identity_bytes, second.identity_bytes);
        assert_eq!(first.release_id(), sha256_id(&first.identity_bytes));
        assert_eq!(
            first.release_id(),
            "sha256-7b450e33e6d5d62b2cc88b3edce7f5692787c908300ba00d1edfd127d2b3cc3a"
        );

        let identity = String::from_utf8(first.identity_bytes.clone()).unwrap();
        assert!(!identity.contains("created_at"));
        assert!(!identity.contains("repository"));
        assert_eq!(
            first.identity().compatible_providers,
            BTreeSet::from([ProviderId::Trae, ProviderId::Codex])
        );
        assert_eq!(
            first.context_template().schema_version,
            "harp-context-template/v1"
        );
        assert_eq!(
            first.identity().context_template_schema,
            "harp-context-template/v1"
        );
    }

    #[test]
    fn corrected_template_contract_has_a_distinct_release_id_and_path() {
        let fixture = ReleaseFixture::new();
        let corrected = compile_baseline_release().unwrap();
        let (legacy_id, legacy_members) = legacy_release(&corrected);
        assert_eq!(
            legacy_id,
            "sha256-2e94ef16b28e9d041b125e39352ccab95c51e778816f2de600de5688254ed83a"
        );
        assert_ne!(legacy_id, corrected.release_id());

        fixture
            .state
            .publish_immutable_tree(&Path::new("releases").join(&legacy_id), &legacy_members)
            .unwrap();
        publish_release(&fixture.state, &corrected).unwrap();

        assert!(fixture.release_path(&legacy_id).is_dir());
        assert!(fixture.release_path(corrected.release_id()).is_dir());
        assert_eq!(
            list_releases_at(&fixture.state).unwrap(),
            vec![corrected.release_id().to_owned()]
        );
    }

    #[test]
    fn identity_uses_canonical_workflow_order_and_binds_package_manifests() {
        let release = compile_baseline_release().unwrap();
        let identity = String::from_utf8(release.identity_bytes.clone()).unwrap();
        let positions = WorkflowId::ALL.map(|workflow| {
            identity
                .find(&format!("\"{workflow}\""))
                .unwrap_or_else(|| panic!("identity is missing {workflow}"))
        });
        assert!(positions.windows(2).all(|window| window[0] < window[1]));

        for workflow in WorkflowId::ALL {
            let package = WorkflowPackage::builtin(workflow).unwrap();
            let expected = sha256_id(&json_bytes(package.manifest()).unwrap());
            assert_eq!(release.identity().packages[&workflow], expected);
        }
    }

    #[test]
    fn release_items_are_ordered_by_workflow_then_item_id() {
        let release = compile_baseline_release().unwrap();
        let actual = release
            .items()
            .iter()
            .map(|item| (item.workflow, item.id.as_str()))
            .collect::<Vec<_>>();
        let mut expected = actual.clone();
        expected.sort_unstable();

        assert_eq!(actual, expected);
        assert_eq!(release.items().len(), 16);
    }

    #[test]
    fn publishing_identical_release_is_idempotent_and_conflicting_member_fails() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();

        publish_release(&fixture.state, &release).unwrap();
        publish_release(&fixture.state, &release).unwrap();

        let mut conflicting_members = release.members();
        conflicting_members
            .get_mut(Path::new("context_template.json"))
            .unwrap()
            .push(b' ');
        let conflict = compile_baseline_release().unwrap();
        let conflict_path = Path::new("releases").join(conflict.release_id());
        let other = ReleaseFixture::new();
        other
            .state
            .publish_immutable_tree(&conflict_path, &conflicting_members)
            .unwrap();
        assert_eq!(
            publish_release(&other.state, &conflict).unwrap_err().code(),
            "release.conflict"
        );

        let names = fs::read_dir(fixture.release_path(release.release_id()))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from([
                "context_template.json".to_owned(),
                "identity.json".to_owned(),
                "items.jsonl".to_owned(),
                "manifest.json".to_owned(),
            ])
        );
    }

    #[test]
    fn publication_rejects_mutated_compiled_release_invariants() {
        let fixture = ReleaseFixture::new();
        let mut release = compile_baseline_release().unwrap();
        release.context_template_bytes.push(b' ');

        assert_eq!(
            publish_release(&fixture.state, &release)
                .unwrap_err()
                .code(),
            "release.invariant"
        );
        assert!(!fixture.release_path(release.release_id()).exists());
    }

    #[test]
    fn publication_preserves_non_content_state_conflict_typing() {
        assert_eq!(
            map_publication_error(AppError::invalid_input("state.conflict", "namespace race"))
                .code(),
            "state.conflict"
        );
        assert_eq!(
            map_publication_error(AppError::invalid_input(
                "state.immutable_collision",
                "content collision"
            ))
            .code(),
            "release.conflict"
        );
    }

    #[test]
    fn inspect_rejects_a_tampered_member_before_returning_release_data() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();
        fs::write(
            fixture
                .release_path(release.release_id())
                .join("items.jsonl"),
            b"{}\n",
        )
        .unwrap();

        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap_err()
                .code(),
            "release.digest"
        );
        assert!(list_releases_at(&fixture.state).unwrap().is_empty());
    }

    #[test]
    fn inspect_rejects_self_consistent_members_not_derived_from_approved_packages() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();

        let mut forged_template = release.context_template().clone();
        forged_template
            .workflows
            .get_mut(&WorkflowId::CiRepair)
            .unwrap()
            .required_sections
            .push("forged_section".to_owned());
        let forged_template_bytes = json_bytes_with_newline(&forged_template).unwrap();
        let mut forged_manifest = release.manifest().clone();
        forged_manifest.members.insert(
            "context_template.json".to_owned(),
            sha256_hex(&forged_template_bytes),
        );
        fs::write(
            fixture
                .release_path(release.release_id())
                .join("context_template.json"),
            forged_template_bytes,
        )
        .unwrap();
        fs::write(
            fixture
                .release_path(release.release_id())
                .join("manifest.json"),
            json_bytes_with_newline(&forged_manifest).unwrap(),
        )
        .unwrap();

        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap_err()
                .code(),
            "release.integrity"
        );
        assert!(list_releases_at(&fixture.state).unwrap().is_empty());
    }

    #[test]
    fn exact_release_bytes_from_generic_state_publication_are_not_authorized() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        fixture
            .state
            .publish_immutable_tree(
                &Path::new("releases").join(release.release_id()),
                &release.members(),
            )
            .unwrap();

        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap_err()
                .code(),
            "release.integrity"
        );
    }

    #[test]
    fn interrupted_publication_content_is_not_listed_without_commit_marker() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        let error = publish_release_with_authorization(&fixture.state, &release, |_, _, _| {
            Err(AppError::invalid_input(
                "test.interrupted",
                "injected interruption",
            ))
        })
        .unwrap_err();

        assert_eq!(error.code(), "test.interrupted");
        assert!(fixture.release_path(release.release_id()).is_dir());
        assert!(list_releases_at(&fixture.state).unwrap().is_empty());
    }

    #[test]
    fn removing_authorization_commit_marker_hides_release_from_list() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();
        fs::remove_dir_all(fixture.authorization_path(release.release_id())).unwrap();

        assert!(list_releases_at(&fixture.state).unwrap().is_empty());
        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap_err()
                .code(),
            "release.integrity"
        );
    }

    #[test]
    fn removing_committed_release_content_hides_it_and_inspection_fails_closed() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();
        fs::remove_dir_all(fixture.release_path(release.release_id())).unwrap();

        assert!(list_releases_at(&fixture.state).unwrap().is_empty());
        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap_err()
                .code(),
            "release.integrity"
        );
    }

    #[test]
    fn authorized_historical_release_survives_simulated_baseline_evolution() {
        let fixture = ReleaseFixture::new();
        let original = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &original).unwrap();

        let evolved = evolved_release(&original);
        publish_release(&fixture.state, &evolved).unwrap();

        assert_eq!(
            inspect_release_at(&fixture.state, original.release_id())
                .unwrap()
                .release_id,
            original.release_id()
        );
        assert_eq!(
            inspect_release_at(&fixture.state, evolved.release_id())
                .unwrap()
                .release_id,
            evolved.release_id()
        );
    }

    #[test]
    fn legacy_staging_residue_does_not_break_release_listing() {
        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();
        let residue = fixture.root().join("releases/.harp-tree-interrupted");
        fs::create_dir(&residue).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&residue, fs::Permissions::from_mode(0o700)).unwrap();
        }

        assert_eq!(
            list_releases_at(&fixture.state).unwrap(),
            vec![release.release_id().to_owned()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn same_uid_offline_rewrite_of_content_and_authorization_is_outside_v1() {
        use std::os::unix::fs::PermissionsExt;

        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();

        let mut rewritten_template = release.context_template().clone();
        rewritten_template
            .workflows
            .get_mut(&WorkflowId::CiRepair)
            .unwrap()
            .required_sections
            .push("same_uid_offline_rewrite".to_owned());
        let template_bytes = json_bytes_with_newline(&rewritten_template).unwrap();
        let mut rewritten_manifest = release.manifest().clone();
        rewritten_manifest.members.insert(
            "context_template.json".to_owned(),
            sha256_hex(&template_bytes),
        );
        let manifest_bytes = json_bytes_with_newline(&rewritten_manifest).unwrap();
        fs::write(
            fixture
                .release_path(release.release_id())
                .join("context_template.json"),
            &template_bytes,
        )
        .unwrap();
        fs::write(
            fixture
                .release_path(release.release_id())
                .join("manifest.json"),
            &manifest_bytes,
        )
        .unwrap();

        let mut members = release.members();
        members.insert(PathBuf::from("context_template.json"), template_bytes);
        members.insert(PathBuf::from("manifest.json"), manifest_bytes);
        let authorization = ReleaseAuthorization {
            schema_version: RELEASE_AUTHORIZATION_SCHEMA.to_owned(),
            release_id: release.release_id().to_owned(),
            members: authorization_hashes(&members),
        };
        let authorization_path = fixture.authorization_path(release.release_id());
        fs::remove_dir_all(&authorization_path).unwrap();
        fs::create_dir(&authorization_path).unwrap();
        fs::set_permissions(&authorization_path, fs::Permissions::from_mode(0o700)).unwrap();
        let authorization_file = authorization_path.join("authorization.json");
        fs::write(
            &authorization_file,
            json_bytes_with_newline(&authorization).unwrap(),
        )
        .unwrap();
        fs::set_permissions(&authorization_file, fs::Permissions::from_mode(0o600)).unwrap();

        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap()
                .context_template,
            rewritten_template
        );
    }

    #[test]
    fn release_items_require_the_context_item_schema_and_nonempty_ids() {
        let release = compile_baseline_release().unwrap();
        let invalid_schema = br#"{"schema_version":"unknown","workflow":"ci_repair","id":"step","kind":"workflow_step","body":"Step"}
"#;
        assert_eq!(
            parse_items(invalid_schema, release.identity())
                .unwrap_err()
                .code(),
            "release.schema"
        );

        let empty_id = br#"{"schema_version":"harp-context-item/v1","workflow":"ci_repair","id":"","kind":"workflow_step","body":"Step"}
"#;
        assert_eq!(
            parse_items(empty_id, release.identity())
                .unwrap_err()
                .code(),
            "release.schema"
        );
    }

    #[cfg(unix)]
    #[test]
    fn inspect_rejects_a_symlinked_releases_directory() {
        use std::os::unix::fs::symlink;

        let fixture = ReleaseFixture::new();
        let release = compile_baseline_release().unwrap();
        publish_release(&fixture.state, &release).unwrap();
        let outside = fixture.root().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::rename(
            fixture.root().join("releases"),
            fixture.root().join("detached-releases"),
        )
        .unwrap();
        symlink(&outside, fixture.root().join("releases")).unwrap();

        assert_eq!(
            inspect_release_at(&fixture.state, release.release_id())
                .unwrap_err()
                .code(),
            "state.symlink"
        );
        assert_eq!(
            list_releases_at(&fixture.state).unwrap_err().code(),
            "state.symlink"
        );
    }

    struct ReleaseFixture {
        _temp: TempDir,
        root: PathBuf,
        state: StateRoot,
    }

    impl ReleaseFixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let root = fs::canonicalize(temp.path()).unwrap().join("state");
            let state = StateRoot::open_or_create(&root).unwrap();
            Self {
                _temp: temp,
                root,
                state,
            }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn release_path(&self, release_id: &str) -> PathBuf {
            self.root.join("releases").join(release_id)
        }

        fn authorization_path(&self, release_id: &str) -> PathBuf {
            self.root
                .join(".harp-internal/release-authorizations")
                .join(release_id)
        }
    }

    fn evolved_release(original: &CompiledRelease) -> CompiledRelease {
        let mut evolved = original.clone();
        evolved.identity.renderer_version = "v2".to_owned();
        evolved.identity_bytes = json_bytes(&evolved.identity).unwrap();
        evolved.release_id = sha256_id(&evolved.identity_bytes);
        evolved.manifest.release_id = evolved.release_id.clone();
        evolved.manifest.members.insert(
            "identity.json".to_owned(),
            sha256_hex(&evolved.identity_bytes),
        );
        evolved.manifest_bytes = json_bytes_with_newline(&evolved.manifest).unwrap();
        evolved
    }

    fn legacy_release(corrected: &CompiledRelease) -> (String, BTreeMap<PathBuf, Vec<u8>>) {
        #[derive(Serialize)]
        struct LegacyReleaseIdentity {
            schema_version: String,
            packages: BTreeMap<WorkflowId, String>,
            selector_version: String,
            renderer_version: String,
            context_budgets: BTreeMap<WorkflowId, u32>,
            compatible_providers: BTreeSet<ProviderId>,
        }

        let identity = corrected.identity();
        let legacy_identity = LegacyReleaseIdentity {
            schema_version: identity.schema_version.clone(),
            packages: identity.packages.clone(),
            selector_version: identity.selector_version.clone(),
            renderer_version: identity.renderer_version.clone(),
            context_budgets: identity.context_budgets.clone(),
            compatible_providers: identity.compatible_providers.clone(),
        };
        let identity_bytes = json_bytes(&legacy_identity).unwrap();
        let release_id = sha256_id(&identity_bytes);
        let mut legacy_template = corrected.context_template().clone();
        legacy_template.schema_version = "harp-context-schema/v1".to_owned();
        let context_template_bytes = json_bytes_with_newline(&legacy_template).unwrap();
        let items_bytes = corrected.items_bytes.clone();
        let manifest = ReleaseManifest {
            schema_version: RELEASE_SCHEMA.to_owned(),
            release_id: release_id.clone(),
            members: member_hashes(&identity_bytes, &context_template_bytes, &items_bytes),
        };
        (
            release_id,
            BTreeMap::from([
                (
                    PathBuf::from("context_template.json"),
                    context_template_bytes,
                ),
                (PathBuf::from("identity.json"), identity_bytes),
                (PathBuf::from("items.jsonl"), items_bytes),
                (
                    PathBuf::from("manifest.json"),
                    json_bytes_with_newline(&manifest).unwrap(),
                ),
            ]),
        )
    }
}
