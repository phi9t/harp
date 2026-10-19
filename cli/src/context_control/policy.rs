use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest as _, Sha256};

use super::workflow::WorkflowPackage;
use super::{WorkflowId, REPOSITORY_POLICY_SCHEMA};
use crate::fs::HeldDirectory;
use crate::AppError;

const POLICY_PATH: &str = ".harp/context-control.json";
const MAX_POLICY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RepositoryPolicy {
    pub schema_version: String,
    pub enabled: bool,
    pub allowed_workflows: Option<BTreeSet<WorkflowId>>,
    pub pinned_release: Option<String>,
    pub max_context_tokens: Option<u32>,
    pub required_verification_labels: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryPolicyWire {
    schema_version: String,
    enabled: bool,
    #[serde(default)]
    allowed_workflows: Presence<Vec<WorkflowId>>,
    #[serde(default)]
    pinned_release: Presence<String>,
    #[serde(default)]
    max_context_tokens: Presence<u32>,
    required_verification_labels: Vec<String>,
}

#[derive(Default)]
enum Presence<T> {
    #[default]
    Missing,
    Present(Option<T>),
}

impl<'de, T> Deserialize<'de> for Presence<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(Self::Present)
    }
}

impl<'de> Deserialize<'de> for RepositoryPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RepositoryPolicyWire::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl RepositoryPolicy {
    fn from_wire(wire: RepositoryPolicyWire) -> Result<Self, AppError> {
        let allowed_workflows = require_presence(wire.allowed_workflows, "allowed_workflows")?
            .map(|workflows| {
                let mut unique = BTreeSet::new();
                for workflow in workflows {
                    if !unique.insert(workflow) {
                        return Err(AppError::invalid_input(
                            "repository.policy",
                            format!("duplicate workflow ID: {workflow}"),
                        ));
                    }
                }
                Ok(unique)
            })
            .transpose()?;
        let policy = Self {
            schema_version: wire.schema_version,
            enabled: wire.enabled,
            allowed_workflows,
            pinned_release: require_presence(wire.pinned_release, "pinned_release")?,
            max_context_tokens: require_presence(wire.max_context_tokens, "max_context_tokens")?,
            required_verification_labels: wire.required_verification_labels,
        };
        policy.validate()?;
        Ok(policy)
    }
}

fn require_presence<T>(presence: Presence<T>, field: &'static str) -> Result<Option<T>, AppError> {
    match presence {
        Presence::Present(value) => Ok(value),
        Presence::Missing => Err(AppError::invalid_input(
            "repository.policy",
            format!("missing field `{field}`"),
        )),
    }
}

impl Default for RepositoryPolicy {
    fn default() -> Self {
        Self {
            schema_version: REPOSITORY_POLICY_SCHEMA.to_owned(),
            enabled: true,
            allowed_workflows: None,
            pinned_release: None,
            max_context_tokens: None,
            required_verification_labels: Vec::new(),
        }
    }
}

impl RepositoryPolicy {
    pub fn parse(bytes: &[u8]) -> Result<Self, AppError> {
        let wire: RepositoryPolicyWire = serde_json::from_slice(bytes).map_err(|error| {
            AppError::invalid_input(
                "repository.policy",
                format!("invalid repository policy: {error}"),
            )
        })?;
        Self::from_wire(wire)
    }

    pub fn load(repository_root: &Path) -> Result<Self, AppError> {
        let repository = HeldDirectory::open(repository_root, "repository policy root")?;
        let Some(bytes) = repository.read_optional_regular_file_bounded(
            Path::new(POLICY_PATH),
            "repository policy",
            MAX_POLICY_BYTES,
        )?
        else {
            return Ok(Self::default());
        };
        Self::parse(&bytes)
    }

    pub fn digest(&self) -> Result<String, AppError> {
        let bytes = serde_json::to_vec(self).map_err(|error| {
            AppError::external(
                "repository.policy_digest",
                format!("could not serialize repository policy: {error}"),
            )
        })?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }

    fn validate(&self) -> Result<(), AppError> {
        if self.schema_version != REPOSITORY_POLICY_SCHEMA {
            return Err(AppError::invalid_input(
                "repository.policy",
                format!(
                    "repository policy schema must be {REPOSITORY_POLICY_SCHEMA}, got {}",
                    self.schema_version
                ),
            ));
        }
        if let Some(release) = self.pinned_release.as_deref() {
            if !valid_release_id(release) {
                return Err(AppError::invalid_input(
                    "repository.policy_release",
                    "pinned release must be sha256- followed by 64 lowercase hex characters",
                ));
            }
        }
        if let Some(maximum) = self.max_context_tokens {
            let applicable = self
                .allowed_workflows
                .as_ref()
                .map(|workflows| workflows.iter().copied().collect::<Vec<_>>())
                .unwrap_or_else(|| WorkflowId::ALL.to_vec());
            for workflow in applicable {
                let built_in = WorkflowPackage::builtin(workflow).map_err(|error| {
                    AppError::external(
                        "repository.policy_budget",
                        format!("could not load built-in workflow {workflow}: {error}"),
                    )
                })?;
                let budget = built_in.manifest().context_budget_tokens;
                if maximum > budget {
                    return Err(AppError::invalid_input(
                        "repository.policy_budget",
                        format!(
                            "repository budget {maximum} exceeds {workflow} built-in budget {budget}"
                        ),
                    ));
                }
            }
        }

        let mut labels = BTreeSet::new();
        for label in &self.required_verification_labels {
            if !valid_verification_label(label) {
                return Err(AppError::invalid_input(
                    "repository.policy_label",
                    format!("unsafe verification label: {label:?}"),
                ));
            }
            if !labels.insert(label) {
                return Err(AppError::invalid_input(
                    "repository.policy_label",
                    format!("duplicate verification label: {label}"),
                ));
            }
        }
        Ok(())
    }
}

fn valid_release_id(value: &str) -> bool {
    value
        .strip_prefix("sha256-")
        .is_some_and(is_lowercase_sha256)
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_verification_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::context_control::{WorkflowId, REPOSITORY_POLICY_SCHEMA};

    const RELEASE_ID: &str =
        "sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn repository_policy_can_only_restrict_runtime_behavior() {
        let policy = RepositoryPolicy::parse(
            format!(
                r#"{{
                    "schema_version":"{REPOSITORY_POLICY_SCHEMA}",
                    "enabled":true,
                    "allowed_workflows":["ci_repair"],
                    "pinned_release":"{RELEASE_ID}",
                    "max_context_tokens":900,
                    "required_verification_labels":["ci"]
                }}"#
            )
            .as_bytes(),
        )
        .unwrap();

        assert_eq!(policy.schema_version, REPOSITORY_POLICY_SCHEMA);
        assert!(policy.enabled);
        assert_eq!(
            policy.allowed_workflows,
            Some([WorkflowId::CiRepair].into_iter().collect())
        );
        assert_eq!(policy.pinned_release.as_deref(), Some(RELEASE_ID));
        assert_eq!(policy.max_context_tokens, Some(900));
        assert_eq!(policy.required_verification_labels, ["ci"]);
    }

    #[test]
    fn missing_policy_defaults_to_enabled_without_restrictions() {
        let repository = tempdir().unwrap();

        let policy = RepositoryPolicy::load(repository.path()).unwrap();

        assert_eq!(policy, RepositoryPolicy::default());
        assert!(policy.enabled);
        assert!(policy.allowed_workflows.is_none());
        assert!(policy.pinned_release.is_none());
        assert!(policy.max_context_tokens.is_none());
        assert!(policy.required_verification_labels.is_empty());
    }

    #[test]
    fn policy_requires_all_six_fields_to_be_explicitly_present() {
        let policy = serde_json::json!({
            "schema_version": REPOSITORY_POLICY_SCHEMA,
            "enabled": true,
            "allowed_workflows": null,
            "pinned_release": null,
            "max_context_tokens": null,
            "required_verification_labels": [],
        });

        for field in [
            "schema_version",
            "enabled",
            "allowed_workflows",
            "pinned_release",
            "max_context_tokens",
            "required_verification_labels",
        ] {
            let mut omitted = policy.clone();
            omitted.as_object_mut().unwrap().remove(field);

            assert_eq!(
                RepositoryPolicy::parse(&serde_json::to_vec(&omitted).unwrap())
                    .unwrap_err()
                    .code(),
                "repository.policy",
                "omitting {field} must fail"
            );
        }
    }

    #[test]
    fn policy_accepts_explicit_null_for_nullable_fields() {
        let policy = RepositoryPolicy::parse(
            format!(
                r#"{{
                    "schema_version":"{REPOSITORY_POLICY_SCHEMA}",
                    "enabled":true,
                    "allowed_workflows":null,
                    "pinned_release":null,
                    "max_context_tokens":null,
                    "required_verification_labels":[]
                }}"#
            )
            .as_bytes(),
        )
        .unwrap();

        assert!(policy.allowed_workflows.is_none());
        assert!(policy.pinned_release.is_none());
        assert!(policy.max_context_tokens.is_none());
    }

    #[test]
    fn public_deserialization_rejects_semantically_invalid_policy() {
        let invalid = serde_json::json!({
            "schema_version": REPOSITORY_POLICY_SCHEMA,
            "enabled": true,
            "allowed_workflows": ["ci_repair"],
            "pinned_release": "not-a-release",
            "max_context_tokens": 2201,
            "required_verification_labels": ["../unsafe"],
        });

        assert!(serde_json::from_value::<RepositoryPolicy>(invalid).is_err());
    }

    #[test]
    fn policy_rejects_unknown_and_authority_broadening_fields() {
        for field in [
            "unknown",
            "prompt",
            "system_prompt",
            "selector",
            "executable",
        ] {
            let bytes = format!(
                r#"{{
                    "schema_version":"{REPOSITORY_POLICY_SCHEMA}",
                    "enabled":true,
                    "allowed_workflows":null,
                    "pinned_release":null,
                    "max_context_tokens":null,
                    "required_verification_labels":[],
                    "{field}":"unsafe"
                }}"#
            );

            assert_eq!(
                RepositoryPolicy::parse(bytes.as_bytes())
                    .unwrap_err()
                    .code(),
                "repository.policy"
            );
        }
    }

    #[test]
    fn policy_rejects_wrong_schema_unknown_and_duplicate_workflows() {
        let cases = [
            r#"{
                "schema_version":"harp-repository-policy/v2",
                "enabled":true,
                "allowed_workflows":null,
                "pinned_release":null,
                "max_context_tokens":null,
                "required_verification_labels":[]
            }"#,
            r#"{
                "schema_version":"harp-repository-policy/v1",
                "enabled":true,
                "allowed_workflows":["unknown"],
                "pinned_release":null,
                "max_context_tokens":null,
                "required_verification_labels":[]
            }"#,
            r#"{
                "schema_version":"harp-repository-policy/v1",
                "enabled":true,
                "allowed_workflows":["ci_repair","ci_repair"],
                "pinned_release":null,
                "max_context_tokens":null,
                "required_verification_labels":[]
            }"#,
        ];

        for bytes in cases {
            assert_eq!(
                RepositoryPolicy::parse(bytes.as_bytes())
                    .unwrap_err()
                    .code(),
                "repository.policy"
            );
        }
    }

    #[test]
    fn policy_rejects_invalid_release_budget_and_labels() {
        let base = serde_json::json!({
            "schema_version": REPOSITORY_POLICY_SCHEMA,
            "enabled": true,
            "allowed_workflows": ["ci_repair"],
            "pinned_release": RELEASE_ID,
            "max_context_tokens": 2200,
            "required_verification_labels": ["ci"],
        });

        let mut invalid_release = base.clone();
        invalid_release["pinned_release"] = serde_json::json!("sha256-not-a-digest");
        assert_eq!(
            RepositoryPolicy::parse(&serde_json::to_vec(&invalid_release).unwrap())
                .unwrap_err()
                .code(),
            "repository.policy_release"
        );

        let mut excessive_budget = base.clone();
        excessive_budget["max_context_tokens"] = serde_json::json!(2201);
        assert_eq!(
            RepositoryPolicy::parse(&serde_json::to_vec(&excessive_budget).unwrap())
                .unwrap_err()
                .code(),
            "repository.policy_budget"
        );

        for label in ["", "../ci", "CI", "two words", &"a".repeat(65)] {
            let mut unsafe_label = base.clone();
            unsafe_label["required_verification_labels"] = serde_json::json!([label]);
            assert_eq!(
                RepositoryPolicy::parse(&serde_json::to_vec(&unsafe_label).unwrap())
                    .unwrap_err()
                    .code(),
                "repository.policy_label"
            );
        }

        let mut duplicate_label = base;
        duplicate_label["required_verification_labels"] = serde_json::json!(["ci", "ci"]);
        assert_eq!(
            RepositoryPolicy::parse(&serde_json::to_vec(&duplicate_label).unwrap())
                .unwrap_err()
                .code(),
            "repository.policy_label"
        );
    }

    #[test]
    fn policy_budget_must_fit_every_applicable_builtin_workflow() {
        let bytes = format!(
            r#"{{
                "schema_version":"{REPOSITORY_POLICY_SCHEMA}",
                "enabled":true,
                "allowed_workflows":["ci_repair","general_coding"],
                "pinned_release":null,
                "max_context_tokens":1601,
                "required_verification_labels":[]
            }}"#
        );

        assert_eq!(
            RepositoryPolicy::parse(bytes.as_bytes())
                .unwrap_err()
                .code(),
            "repository.policy_budget"
        );
    }

    #[test]
    fn policy_digest_is_deterministic_for_semantically_equal_input() {
        let first = RepositoryPolicy::parse(
            format!(
                r#"{{
                    "schema_version":"{REPOSITORY_POLICY_SCHEMA}",
                    "enabled":true,
                    "allowed_workflows":["general_coding","ci_repair"],
                    "pinned_release":null,
                    "max_context_tokens":900,
                    "required_verification_labels":["unit","ci"]
                }}"#
            )
            .as_bytes(),
        )
        .unwrap();
        let second = RepositoryPolicy::parse(
            format!(
                r#"{{
                    "required_verification_labels":["unit","ci"],
                    "max_context_tokens":900,
                    "pinned_release":null,
                    "allowed_workflows":["ci_repair","general_coding"],
                    "enabled":true,
                    "schema_version":"{REPOSITORY_POLICY_SCHEMA}"
                }}"#
            )
            .as_bytes(),
        )
        .unwrap();

        assert_eq!(first.digest().unwrap(), second.digest().unwrap());
        assert!(first.digest().unwrap().starts_with("sha256:"));
    }

    #[cfg(unix)]
    #[test]
    fn policy_load_rejects_symlinked_components_and_target() {
        use std::os::unix::fs::symlink;

        let repository = tempdir().unwrap();
        let outside = tempdir().unwrap();
        fs::write(
            outside.path().join("context-control.json"),
            serde_json::to_vec(&RepositoryPolicy::default()).unwrap(),
        )
        .unwrap();
        symlink(outside.path(), repository.path().join(".harp")).unwrap();

        assert_eq!(
            RepositoryPolicy::load(repository.path())
                .unwrap_err()
                .code(),
            "fs.symlink"
        );

        fs::remove_file(repository.path().join(".harp")).unwrap();
        fs::create_dir(repository.path().join(".harp")).unwrap();
        symlink(
            outside.path().join("context-control.json"),
            repository.path().join(".harp/context-control.json"),
        )
        .unwrap();
        assert_eq!(
            RepositoryPolicy::load(repository.path())
                .unwrap_err()
                .code(),
            "fs.symlink"
        );
    }

    #[test]
    fn policy_load_rejects_files_over_64_kib() {
        let repository = tempdir().unwrap();
        fs::create_dir(repository.path().join(".harp")).unwrap();
        fs::write(
            repository.path().join(".harp/context-control.json"),
            vec![b' '; MAX_POLICY_BYTES + 1],
        )
        .unwrap();

        assert_eq!(
            RepositoryPolicy::load(repository.path())
                .unwrap_err()
                .code(),
            "fs.size"
        );
    }
}
