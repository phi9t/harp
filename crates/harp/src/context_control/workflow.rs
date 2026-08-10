use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use super::{
    ContextItemKind, WorkflowId, CONTEXT_ITEM_SCHEMA, CONTEXT_SCHEMA, OUTCOME_RULES_SCHEMA,
    ROUTING_SCHEMA, VERIFICATION_SCHEMA, WORKFLOW_SCHEMA,
};
use crate::AppError;

const MEMBER_NAMES: [&str; 5] = [
    "context_schema.json",
    "outcome_rules.json",
    "playbook.jsonl",
    "routing.json",
    "verification.json",
];

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowManifest {
    pub schema_version: String,
    pub id: WorkflowId,
    pub title: String,
    pub context_budget_tokens: u32,
    pub selector_version: String,
    pub renderer_version: String,
    pub members: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RoutingRules {
    pub schema_version: String,
    pub positive_phrases: Vec<String>,
    pub negative_phrases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContextSchema {
    pub schema_version: String,
    pub required_sections: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VerificationRecipe {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VerificationRules {
    pub schema_version: String,
    pub recipes: Vec<VerificationRecipe>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OutcomeRules {
    pub schema_version: String,
    pub verified_success: Vec<String>,
    pub weak_signals: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlaybookItem {
    pub schema_version: String,
    pub id: String,
    pub kind: ContextItemKind,
    pub body: String,
}

/// A digest-validated package whose members are observable only through shared references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowPackage {
    manifest: WorkflowManifest,
    routing: RoutingRules,
    context_schema: ContextSchema,
    verification: VerificationRules,
    outcome_rules: OutcomeRules,
    playbook: Vec<PlaybookItem>,
    member_digests: BTreeMap<String, String>,
}

#[derive(Clone, Copy)]
struct EmbeddedWorkflow {
    manifest: &'static [u8],
    routing: &'static [u8],
    context_schema: &'static [u8],
    verification: &'static [u8],
    outcome_rules: &'static [u8],
    playbook: &'static [u8],
}

impl WorkflowPackage {
    pub fn builtin(workflow: WorkflowId) -> Result<Self, AppError> {
        let embedded = EmbeddedWorkflow::for_id(workflow);
        let package = Self::parse(
            embedded.manifest,
            embedded.routing,
            embedded.context_schema,
            embedded.verification,
            embedded.outcome_rules,
            embedded.playbook,
        )?;
        if package.manifest.id != workflow {
            return Err(AppError::invalid_input(
                "workflow.manifest",
                format!(
                    "embedded {workflow} manifest declares {}",
                    package.manifest.id
                ),
            ));
        }
        Ok(package)
    }

    pub fn manifest(&self) -> &WorkflowManifest {
        &self.manifest
    }

    pub fn routing(&self) -> &RoutingRules {
        &self.routing
    }

    pub fn context_schema(&self) -> &ContextSchema {
        &self.context_schema
    }

    pub fn verification(&self) -> &VerificationRules {
        &self.verification
    }

    pub fn outcome_rules(&self) -> &OutcomeRules {
        &self.outcome_rules
    }

    pub fn playbook(&self) -> &[PlaybookItem] {
        &self.playbook
    }

    pub fn member_digests(&self) -> &BTreeMap<String, String> {
        &self.member_digests
    }

    fn parse(
        manifest_bytes: &[u8],
        routing_bytes: &[u8],
        context_schema_bytes: &[u8],
        verification_bytes: &[u8],
        outcome_rules_bytes: &[u8],
        playbook_bytes: &[u8],
    ) -> Result<Self, AppError> {
        let manifest = Self::parse_manifest(manifest_bytes)?;
        validate_manifest(&manifest)?;

        let members = BTreeMap::from([
            ("context_schema.json", context_schema_bytes),
            ("outcome_rules.json", outcome_rules_bytes),
            ("playbook.jsonl", playbook_bytes),
            ("routing.json", routing_bytes),
            ("verification.json", verification_bytes),
        ]);
        let member_digests = verify_member_digests(&manifest, &members)?;

        let routing = Self::parse_routing(routing_bytes)?;
        let context_schema = Self::parse_context_schema(context_schema_bytes)?;
        let verification = Self::parse_verification(verification_bytes)?;
        let outcome_rules = Self::parse_outcome_rules(outcome_rules_bytes)?;
        let playbook = Self::parse_playbook(playbook_bytes)?;

        Ok(Self {
            manifest,
            routing,
            context_schema,
            verification,
            outcome_rules,
            playbook,
            member_digests,
        })
    }

    fn parse_manifest(bytes: &[u8]) -> Result<WorkflowManifest, AppError> {
        serde_json::from_slice(bytes).map_err(|error| {
            AppError::invalid_input(
                "workflow.manifest",
                format!("invalid workflow manifest: {error}"),
            )
        })
    }

    fn parse_routing(bytes: &[u8]) -> Result<RoutingRules, AppError> {
        let rules: RoutingRules = parse_member(bytes, "routing rules")?;
        require_schema(&rules.schema_version, ROUTING_SCHEMA, "routing rules")?;
        Ok(rules)
    }

    fn parse_context_schema(bytes: &[u8]) -> Result<ContextSchema, AppError> {
        let schema: ContextSchema = parse_member(bytes, "context schema")?;
        require_schema(&schema.schema_version, CONTEXT_SCHEMA, "context schema")?;
        Ok(schema)
    }

    fn parse_verification(bytes: &[u8]) -> Result<VerificationRules, AppError> {
        let rules: VerificationRules = parse_member(bytes, "verification rules")?;
        require_schema(
            &rules.schema_version,
            VERIFICATION_SCHEMA,
            "verification rules",
        )?;
        Ok(rules)
    }

    fn parse_outcome_rules(bytes: &[u8]) -> Result<OutcomeRules, AppError> {
        let rules: OutcomeRules = parse_member(bytes, "outcome rules")?;
        require_schema(&rules.schema_version, OUTCOME_RULES_SCHEMA, "outcome rules")?;
        Ok(rules)
    }

    fn parse_playbook(bytes: &[u8]) -> Result<Vec<PlaybookItem>, AppError> {
        let text = std::str::from_utf8(bytes).map_err(|error| {
            AppError::invalid_input("workflow.schema", format!("playbook is not UTF-8: {error}"))
        })?;
        let mut items = Vec::new();
        let mut item_ids = BTreeSet::new();

        for (index, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let item: PlaybookItem = serde_json::from_str(line).map_err(|error| {
                AppError::invalid_input(
                    "workflow.schema",
                    format!("invalid playbook line {}: {error}", index + 1),
                )
            })?;
            require_schema(&item.schema_version, CONTEXT_ITEM_SCHEMA, "playbook item")?;
            if item.id.is_empty() {
                return Err(AppError::invalid_input(
                    "workflow.schema",
                    format!("playbook line {} has an empty item ID", index + 1),
                ));
            }
            if !item_ids.insert(item.id.clone()) {
                return Err(AppError::invalid_input(
                    "workflow.playbook_duplicate",
                    format!("duplicate playbook item ID: {}", item.id),
                ));
            }
            items.push(item);
        }

        if items.is_empty() {
            return Err(AppError::invalid_input(
                "workflow.schema",
                "playbook must contain at least one item",
            ));
        }
        Ok(items)
    }
}

impl EmbeddedWorkflow {
    fn for_id(workflow: WorkflowId) -> Self {
        match workflow {
            WorkflowId::CiRepair => Self {
                manifest: include_bytes!(
                    "../../../../content/context_control/workflows/ci_repair/manifest.json"
                ),
                routing: include_bytes!(
                    "../../../../content/context_control/workflows/ci_repair/routing.json"
                ),
                context_schema: include_bytes!(
                    "../../../../content/context_control/workflows/ci_repair/context_schema.json"
                ),
                verification: include_bytes!(
                    "../../../../content/context_control/workflows/ci_repair/verification.json"
                ),
                outcome_rules: include_bytes!(
                    "../../../../content/context_control/workflows/ci_repair/outcome_rules.json"
                ),
                playbook: include_bytes!(
                    "../../../../content/context_control/workflows/ci_repair/playbook.jsonl"
                ),
            },
            WorkflowId::CodeReview => Self {
                manifest: include_bytes!(
                    "../../../../content/context_control/workflows/code_review/manifest.json"
                ),
                routing: include_bytes!(
                    "../../../../content/context_control/workflows/code_review/routing.json"
                ),
                context_schema: include_bytes!(
                    "../../../../content/context_control/workflows/code_review/context_schema.json"
                ),
                verification: include_bytes!(
                    "../../../../content/context_control/workflows/code_review/verification.json"
                ),
                outcome_rules: include_bytes!(
                    "../../../../content/context_control/workflows/code_review/outcome_rules.json"
                ),
                playbook: include_bytes!(
                    "../../../../content/context_control/workflows/code_review/playbook.jsonl"
                ),
            },
            WorkflowId::DependencyUpdate => Self {
                manifest: include_bytes!(
                    "../../../../content/context_control/workflows/dependency_update/manifest.json"
                ),
                routing: include_bytes!(
                    "../../../../content/context_control/workflows/dependency_update/routing.json"
                ),
                context_schema: include_bytes!(
                    "../../../../content/context_control/workflows/dependency_update/context_schema.json"
                ),
                verification: include_bytes!(
                    "../../../../content/context_control/workflows/dependency_update/verification.json"
                ),
                outcome_rules: include_bytes!(
                    "../../../../content/context_control/workflows/dependency_update/outcome_rules.json"
                ),
                playbook: include_bytes!(
                    "../../../../content/context_control/workflows/dependency_update/playbook.jsonl"
                ),
            },
            WorkflowId::GeneralCoding => Self {
                manifest: include_bytes!(
                    "../../../../content/context_control/workflows/general_coding/manifest.json"
                ),
                routing: include_bytes!(
                    "../../../../content/context_control/workflows/general_coding/routing.json"
                ),
                context_schema: include_bytes!(
                    "../../../../content/context_control/workflows/general_coding/context_schema.json"
                ),
                verification: include_bytes!(
                    "../../../../content/context_control/workflows/general_coding/verification.json"
                ),
                outcome_rules: include_bytes!(
                    "../../../../content/context_control/workflows/general_coding/outcome_rules.json"
                ),
                playbook: include_bytes!(
                    "../../../../content/context_control/workflows/general_coding/playbook.jsonl"
                ),
            },
        }
    }
}

fn parse_member<T>(bytes: &[u8], label: &str) -> Result<T, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(bytes).map_err(|error| {
        AppError::invalid_input("workflow.schema", format!("invalid {label}: {error}"))
    })
}

fn validate_manifest(manifest: &WorkflowManifest) -> Result<(), AppError> {
    if manifest.schema_version != WORKFLOW_SCHEMA {
        return Err(AppError::invalid_input(
            "workflow.manifest",
            format!(
                "workflow manifest schema must be {WORKFLOW_SCHEMA}, got {}",
                manifest.schema_version
            ),
        ));
    }
    let member_names = manifest
        .members
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if member_names != MEMBER_NAMES {
        return Err(AppError::invalid_input(
            "workflow.manifest",
            format!(
                "workflow manifest members must be exactly: {}",
                MEMBER_NAMES.join(", ")
            ),
        ));
    }
    for (name, digest) in &manifest.members {
        if !is_lowercase_sha256(digest) {
            return Err(AppError::invalid_input(
                "workflow.manifest",
                format!("{name} must use a lowercase SHA-256 digest"),
            ));
        }
    }
    Ok(())
}

fn verify_member_digests(
    manifest: &WorkflowManifest,
    members: &BTreeMap<&str, &[u8]>,
) -> Result<BTreeMap<String, String>, AppError> {
    let mut digests = BTreeMap::new();
    for (name, bytes) in members {
        let actual = format!("{:x}", Sha256::digest(bytes));
        let expected = manifest.members.get(*name).ok_or_else(|| {
            AppError::invalid_input(
                "workflow.manifest",
                format!("workflow manifest is missing {name}"),
            )
        })?;
        if actual != *expected {
            return Err(AppError::invalid_input(
                "workflow.digest",
                format!("{name} digest mismatch"),
            ));
        }
        digests.insert((*name).to_owned(), actual);
    }
    Ok(digests)
}

fn require_schema(actual: &str, expected: &str, label: &str) -> Result<(), AppError> {
    if actual != expected {
        return Err(AppError::invalid_input(
            "workflow.schema",
            format!("{label} schema must be {expected}, got {actual}"),
        ));
    }
    Ok(())
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::context_control::{
        CONTEXT_ITEM_SCHEMA, CONTEXT_SCHEMA, OUTCOME_RULES_SCHEMA, ROUTING_SCHEMA,
        VERIFICATION_SCHEMA, WORKFLOW_SCHEMA,
    };

    #[test]
    fn all_builtin_workflows_parse_and_bind_member_digests() {
        for workflow in WorkflowId::ALL {
            let package = WorkflowPackage::builtin(workflow).unwrap();

            assert_eq!(package.manifest().id, workflow);
            assert_eq!(package.manifest().schema_version, WORKFLOW_SCHEMA);
            assert!(!package.playbook().is_empty());
            assert_eq!(package.member_digests().len(), 5);
            assert_eq!(package.routing().schema_version, ROUTING_SCHEMA);
            assert_eq!(package.context_schema().schema_version, CONTEXT_SCHEMA);
            assert_eq!(package.verification().schema_version, VERIFICATION_SCHEMA);
            assert_eq!(package.outcome_rules().schema_version, OUTCOME_RULES_SCHEMA);
            assert!(package
                .playbook()
                .iter()
                .all(|item| item.schema_version == CONTEXT_ITEM_SCHEMA));
        }
    }

    #[test]
    fn validated_package_data_is_observed_through_shared_references() {
        fn assert_immutable_accessors(
            _manifest: for<'a> fn(&'a WorkflowPackage) -> &'a WorkflowManifest,
            _routing: for<'a> fn(&'a WorkflowPackage) -> &'a RoutingRules,
            _context_schema: for<'a> fn(&'a WorkflowPackage) -> &'a ContextSchema,
            _verification: for<'a> fn(&'a WorkflowPackage) -> &'a VerificationRules,
            _outcome_rules: for<'a> fn(&'a WorkflowPackage) -> &'a OutcomeRules,
            _playbook: for<'a> fn(&'a WorkflowPackage) -> &'a [PlaybookItem],
            _member_digests: for<'a> fn(&'a WorkflowPackage) -> &'a BTreeMap<String, String>,
        ) {
        }

        assert_immutable_accessors(
            WorkflowPackage::manifest,
            WorkflowPackage::routing,
            WorkflowPackage::context_schema,
            WorkflowPackage::verification,
            WorkflowPackage::outcome_rules,
            WorkflowPackage::playbook,
            WorkflowPackage::member_digests,
        );
    }

    #[test]
    fn builtin_manifests_have_exact_titles_and_budgets() {
        let expected = [
            (WorkflowId::CiRepair, "CI repair", 2200),
            (WorkflowId::CodeReview, "Code review", 1800),
            (WorkflowId::DependencyUpdate, "Dependency update", 2000),
            (WorkflowId::GeneralCoding, "General coding", 1600),
        ];

        for (workflow, title, budget) in expected {
            let package = WorkflowPackage::builtin(workflow).unwrap();
            assert_eq!(package.manifest().title, title);
            assert_eq!(package.manifest().context_budget_tokens, budget);
        }
    }

    #[test]
    fn builtin_routing_uses_exact_vocabulary() {
        let expected = [
            (
                WorkflowId::CiRepair,
                &[
                    "ci failed",
                    "test failure",
                    "tests failing",
                    "lint failed",
                    "typecheck failed",
                    "build failed",
                    "reproduce failure",
                ][..],
                &["review this diff", "upgrade dependency"][..],
            ),
            (
                WorkflowId::CodeReview,
                &[
                    "code review",
                    "review this diff",
                    "review this change",
                    "find bugs",
                    "audit this patch",
                ][..],
                &["fix the failing test", "upgrade dependency"][..],
            ),
            (
                WorkflowId::DependencyUpdate,
                &[
                    "upgrade dependency",
                    "update dependency",
                    "bump version",
                    "refresh lockfile",
                    "toolchain update",
                ][..],
                &["review this diff", "test failure"][..],
            ),
            (WorkflowId::GeneralCoding, &[][..], &[][..]),
        ];

        for (workflow, positive_phrases, negative_phrases) in expected {
            let package = WorkflowPackage::builtin(workflow).unwrap();
            assert_eq!(package.routing().positive_phrases, positive_phrases);
            assert_eq!(package.routing().negative_phrases, negative_phrases);
        }
    }

    #[test]
    fn builtin_playbooks_contain_required_semantics() {
        let expected = [
            (
                WorkflowId::CiRepair,
                [
                    "Reproduce the narrowest failure",
                    "Classify before editing",
                    "Do not weaken tests",
                    "Rerun the original verification mode",
                ],
            ),
            (
                WorkflowId::CodeReview,
                [
                    "Inspect the diff before conclusions",
                    "Report only actionable findings",
                    "Verify each finding against source",
                    "Avoid style-only inflation",
                ],
            ),
            (
                WorkflowId::DependencyUpdate,
                [
                    "Identify direct and transitive change",
                    "Preserve lockfile consistency",
                    "Avoid unrelated upgrades",
                    "Run compatibility verification",
                ],
            ),
            (
                WorkflowId::GeneralCoding,
                [
                    "Restate the bounded requirement",
                    "Inspect local contracts before editing",
                    "Make the smallest causal change",
                    "Run repository verification",
                ],
            ),
        ];

        for (workflow, required_items) in expected {
            let package = WorkflowPackage::builtin(workflow).unwrap();
            let bodies = package
                .playbook()
                .iter()
                .map(|item| item.body.as_str())
                .collect::<Vec<_>>();
            for required_item in required_items {
                assert!(
                    bodies.contains(&required_item),
                    "{workflow} is missing {required_item:?}"
                );
            }
            assert_eq!(package.verification().recipes.len(), 1);
            assert_eq!(package.outcome_rules().weak_signals.len(), 1);
            assert!(package.outcome_rules().weak_signals[0]
                .to_ascii_lowercase()
                .contains("not verified success"));
        }
    }

    #[test]
    fn workflow_parser_rejects_unknown_manifest_fields_and_digest_mismatch() {
        let unknown = br#"{
            "schema_version":"harp-workflow/v1",
            "id":"ci_repair",
            "title":"CI repair",
            "context_budget_tokens":2200,
            "selector_version":"v1",
            "renderer_version":"v1",
            "members":{},
            "extra":true
        }"#;
        assert_eq!(
            WorkflowPackage::parse_manifest(unknown).unwrap_err().code(),
            "workflow.manifest"
        );

        let manifest = WorkflowManifest {
            schema_version: WORKFLOW_SCHEMA.to_owned(),
            id: WorkflowId::CiRepair,
            title: "CI repair".to_owned(),
            context_budget_tokens: 2200,
            selector_version: "v1".to_owned(),
            renderer_version: "v1".to_owned(),
            members: BTreeMap::from([
                ("context_schema.json".to_owned(), "0".repeat(64)),
                ("outcome_rules.json".to_owned(), "0".repeat(64)),
                ("playbook.jsonl".to_owned(), "0".repeat(64)),
                ("routing.json".to_owned(), "0".repeat(64)),
                ("verification.json".to_owned(), "0".repeat(64)),
            ]),
        };
        assert_eq!(
            WorkflowPackage::parse(
                serde_json::to_vec(&manifest).unwrap().as_slice(),
                br#"{"schema_version":"harp-routing/v1","positive_phrases":[],"negative_phrases":[]}"#,
                br#"{"schema_version":"harp-context-schema/v1","required_sections":[]}"#,
                br#"{"schema_version":"harp-verification/v1","recipes":[]}"#,
                br#"{"schema_version":"harp-outcome-rules/v1","verified_success":[],"weak_signals":[]}"#,
                br#"{"schema_version":"harp-context-item/v1","id":"step","kind":"workflow_step","body":"Step"}"#,
            )
            .unwrap_err()
            .code(),
            "workflow.digest"
        );
    }

    #[test]
    fn workflow_parser_rejects_unknown_fields_in_every_member_type() {
        assert_schema_error(
            "routing",
            WorkflowPackage::parse_routing(
                br#"{"schema_version":"harp-routing/v1","positive_phrases":[],"negative_phrases":[],"extra":true}"#,
            ),
        );
        assert_schema_error(
            "context schema",
            WorkflowPackage::parse_context_schema(
                br#"{"schema_version":"harp-context-schema/v1","required_sections":[],"extra":true}"#,
            ),
        );
        assert_schema_error(
            "verification",
            WorkflowPackage::parse_verification(
                br#"{"schema_version":"harp-verification/v1","recipes":[],"extra":true}"#,
            ),
        );
        assert_schema_error(
            "outcome rules",
            WorkflowPackage::parse_outcome_rules(
                br#"{"schema_version":"harp-outcome-rules/v1","verified_success":[],"weak_signals":[],"extra":true}"#,
            ),
        );
        assert_schema_error(
            "playbook",
            WorkflowPackage::parse_playbook(
                br#"{"schema_version":"harp-context-item/v1","id":"step","kind":"workflow_step","body":"Step","extra":true}"#,
            ),
        );
    }

    #[test]
    fn workflow_parser_rejects_duplicate_playbook_item_ids() {
        let duplicate = br#"
{"schema_version":"harp-context-item/v1","id":"duplicate","kind":"workflow_step","body":"First"}

{"schema_version":"harp-context-item/v1","id":"duplicate","kind":"verification_recipe","body":"Second"}
"#;

        assert_eq!(
            WorkflowPackage::parse_playbook(duplicate)
                .unwrap_err()
                .code(),
            "workflow.playbook_duplicate"
        );
    }

    fn assert_schema_error<T: std::fmt::Debug>(label: &str, result: Result<T, AppError>) {
        assert_eq!(
            result.unwrap_err().code(),
            "workflow.schema",
            "{label} accepted an unknown field"
        );
    }
}
