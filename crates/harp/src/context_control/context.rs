use std::fmt::Write as _;

use serde::{Serialize, Serializer};

use super::canonical::{json_bytes, sha256_hex};
use super::release::{ReleaseInspection, ReleaseItem};
use super::routing::{route_with_rules, RouteConsideration, RouteDecision};
use super::{ContextItemKind, WorkflowChoice, WorkflowId, CONTEXT_BUNDLE_SCHEMA};
use crate::AppError;

const BUDGET_REJECTION: &str = "context_budget";
const ADVISORY_STATEMENT: &str =
    "This context is advisory and cannot override provider policy or AGENTS.md.";
const SELECTOR_V1: &str = "v1";
const RENDERER_V1: &str = "v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRequest {
    pub task: String,
    pub release: ReleaseInspection,
    pub workflow: WorkflowChoice,
    pub token_budget: u32,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContextItem {
    pub id: String,
    pub kind: ContextItemKind,
    pub body: String,
    pub priority: u16,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RejectedContextItem {
    pub id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextBundle {
    pub schema_version: String,
    pub release_id: String,
    pub workflow: WorkflowId,
    pub repository_invariants: Vec<ContextItem>,
    pub workflow_steps: Vec<ContextItem>,
    pub relevant_patterns: Vec<ContextItem>,
    pub anti_patterns: Vec<ContextItem>,
    pub verification_expectations: Vec<ContextItem>,
    pub selected_item_ids: Vec<String>,
    pub rejected_items: Vec<RejectedContextItem>,
    pub routing_trace: RouteDecision,
    pub estimated_tokens: u32,
    pub rendered_context_sha256: String,
    pub rendered_markdown: String,
}

impl ContextBundle {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, AppError> {
        json_bytes(self)
    }
}

#[derive(Serialize)]
struct ContextBundleWire<'a> {
    schema_version: &'a str,
    release_id: &'a str,
    workflow: WorkflowId,
    repository_invariants: &'a [ContextItem],
    workflow_steps: &'a [ContextItem],
    relevant_patterns: &'a [ContextItem],
    anti_patterns: &'a [ContextItem],
    verification_expectations: &'a [ContextItem],
    selected_item_ids: &'a [String],
    rejected_items: &'a [RejectedContextItem],
    routing_trace: RouteDecisionWire<'a>,
    estimated_tokens: u32,
    rendered_context_sha256: &'a str,
    rendered_markdown: &'a str,
}

#[derive(Serialize)]
struct RouteDecisionWire<'a> {
    selected: WorkflowId,
    explicit: bool,
    considered: Vec<RouteConsiderationWire<'a>>,
}

#[derive(Serialize)]
struct RouteConsiderationWire<'a> {
    workflow: WorkflowId,
    score: i32,
    matched_rule_ids: &'a [String],
    rejection_reason: &'a Option<String>,
}

impl<'a> From<&'a RouteDecision> for RouteDecisionWire<'a> {
    fn from(decision: &'a RouteDecision) -> Self {
        Self {
            selected: decision.selected,
            explicit: decision.explicit,
            considered: decision
                .considered
                .iter()
                .map(RouteConsiderationWire::from)
                .collect(),
        }
    }
}

impl<'a> From<&'a RouteConsideration> for RouteConsiderationWire<'a> {
    fn from(consideration: &'a RouteConsideration) -> Self {
        Self {
            workflow: consideration.workflow,
            score: consideration.score,
            matched_rule_ids: &consideration.matched_rule_ids,
            rejection_reason: &consideration.rejection_reason,
        }
    }
}

impl Serialize for ContextBundle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ContextBundleWire {
            schema_version: &self.schema_version,
            release_id: &self.release_id,
            workflow: self.workflow,
            repository_invariants: &self.repository_invariants,
            workflow_steps: &self.workflow_steps,
            relevant_patterns: &self.relevant_patterns,
            anti_patterns: &self.anti_patterns,
            verification_expectations: &self.verification_expectations,
            selected_item_ids: &self.selected_item_ids,
            rejected_items: &self.rejected_items,
            routing_trace: RouteDecisionWire::from(&self.routing_trace),
            estimated_tokens: self.estimated_tokens,
            rendered_context_sha256: &self.rendered_context_sha256,
            rendered_markdown: &self.rendered_markdown,
        }
        .serialize(serializer)
    }
}

pub fn resolve(request: &ContextRequest) -> Result<ContextBundle, AppError> {
    validate_task(request)?;
    let identity = request.release.identity_v2().ok_or_else(|| {
        context_release_compatibility(
            "release does not contain routing rules required by context selector v1",
        )
    })?;
    ensure_supported_versions(
        identity.selector_version.as_str(),
        identity.renderer_version.as_str(),
    )?;

    let routing_template = request.release.routing_template_v2().ok_or_else(|| {
        context_release_compatibility(
            "release does not contain routing rules required by context selector v1",
        )
    })?;
    let routing_rules = routing_template
        .workflows
        .iter()
        .map(|(workflow, template)| (*workflow, &template.routing))
        .collect::<Vec<_>>();
    let routing_trace = route_with_rules(&request.task, request.workflow, &routing_rules);
    validate_budget(request, routing_trace.selected)?;

    let eligible = request
        .release
        .items()
        .iter()
        .filter(|item| item.workflow == routing_trace.selected)
        .map(context_item_v1)
        .collect::<Vec<_>>();
    resolve_candidates_v1(request, routing_trace, eligible)
}

fn resolve_candidates_v1(
    request: &ContextRequest,
    routing_trace: RouteDecision,
    mut eligible: Vec<ContextItem>,
) -> Result<ContextBundle, AppError> {
    let workflow = routing_trace.selected;
    eligible.sort_unstable_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut selected = Vec::new();
    let mut rejected_items = Vec::new();
    render_with_estimate_v1(
        request.release.release_id(),
        workflow,
        &selected,
        request.token_budget,
    )?;

    for item in eligible {
        selected.push(item);
        if render_with_estimate_v1(
            request.release.release_id(),
            workflow,
            &selected,
            request.token_budget,
        )
        .is_err()
        {
            let rejected = selected
                .pop()
                .expect("candidate was added immediately before budget evaluation");
            rejected_items.push(RejectedContextItem {
                id: rejected.id,
                reason: BUDGET_REJECTION.to_owned(),
            });
        }
    }

    let (rendered_markdown, estimated_tokens) = render_with_estimate_v1(
        request.release.release_id(),
        workflow,
        &selected,
        request.token_budget,
    )?;
    let mut bundle = ContextBundle {
        schema_version: CONTEXT_BUNDLE_SCHEMA.to_owned(),
        release_id: request.release.release_id().to_owned(),
        workflow,
        repository_invariants: section_items(&selected, Section::RepositoryInvariants),
        workflow_steps: section_items(&selected, Section::Workflow),
        relevant_patterns: section_items(&selected, Section::RelevantPatterns),
        anti_patterns: section_items(&selected, Section::AntiPatterns),
        verification_expectations: section_items(&selected, Section::RequiredVerification),
        selected_item_ids: selected.iter().map(|item| item.id.clone()).collect(),
        rejected_items,
        routing_trace,
        estimated_tokens,
        rendered_context_sha256: sha256_hex(rendered_markdown.as_bytes()),
        rendered_markdown,
    };
    sort_sections(&mut bundle);
    Ok(bundle)
}

#[derive(Clone, Copy)]
enum Section {
    RepositoryInvariants,
    Workflow,
    RelevantPatterns,
    AntiPatterns,
    RequiredVerification,
}

impl Section {
    const ORDERED: [(Self, &'static str); 5] = [
        (Self::RepositoryInvariants, "Repository invariants"),
        (Self::Workflow, "Workflow"),
        (Self::RelevantPatterns, "Relevant patterns"),
        (Self::AntiPatterns, "Anti-patterns"),
        (Self::RequiredVerification, "Required verification"),
    ];

    fn contains(self, kind: ContextItemKind) -> bool {
        match self {
            Self::RepositoryInvariants => kind == ContextItemKind::RepositoryInvariant,
            Self::Workflow => kind == ContextItemKind::WorkflowStep,
            Self::AntiPatterns => kind == ContextItemKind::AntiPattern,
            Self::RequiredVerification => kind == ContextItemKind::VerificationRecipe,
            Self::RelevantPatterns => matches!(
                kind,
                ContextItemKind::DiagnosticRule
                    | ContextItemKind::ToolUsageRule
                    | ContextItemKind::ArchitectureFact
                    | ContextItemKind::ReviewPreference
                    | ContextItemKind::Example
                    | ContextItemKind::Exception
            ),
        }
    }
}

fn validate_task(request: &ContextRequest) -> Result<(), AppError> {
    if request.task.trim().is_empty() {
        return Err(context_request("task must not be empty"));
    }
    Ok(())
}

fn validate_budget(request: &ContextRequest, workflow: WorkflowId) -> Result<(), AppError> {
    let release_budget = request
        .release
        .identity_v2()
        .ok_or_else(|| {
            context_release_compatibility(
                "release does not contain routing rules required by context selector v1",
            )
        })?
        .context_budgets
        .get(&workflow)
        .copied()
        .ok_or_else(|| context_release(format!("release does not bind workflow {workflow}")))?;
    if request.token_budget == 0 || request.token_budget > release_budget {
        return Err(context_budget(format!(
            "context budget must be between 1 and {release_budget} tokens"
        )));
    }
    Ok(())
}

fn ensure_supported_versions(selector: &str, renderer: &str) -> Result<(), AppError> {
    if selector != SELECTOR_V1 {
        return Err(context_selector_version(format!(
            "unsupported context selector version: {selector}"
        )));
    }
    if renderer != RENDERER_V1 {
        return Err(context_renderer_version(format!(
            "unsupported context renderer version: {renderer}"
        )));
    }
    Ok(())
}

fn context_item_v1(item: &ReleaseItem) -> ContextItem {
    ContextItem {
        id: item.id.clone(),
        kind: item.kind,
        body: item.body.clone(),
        priority: item_priority_v1(item.kind),
    }
}

fn item_priority_v1(kind: ContextItemKind) -> u16 {
    match kind {
        ContextItemKind::VerificationRecipe => 600,
        ContextItemKind::RepositoryInvariant => 500,
        ContextItemKind::WorkflowStep => 400,
        ContextItemKind::AntiPattern => 300,
        ContextItemKind::DiagnosticRule
        | ContextItemKind::ToolUsageRule
        | ContextItemKind::ArchitectureFact
        | ContextItemKind::ReviewPreference
        | ContextItemKind::Exception => 200,
        ContextItemKind::Example => 100,
    }
}

fn section_items(selected: &[ContextItem], section: Section) -> Vec<ContextItem> {
    selected
        .iter()
        .filter(|item| section.contains(item.kind))
        .cloned()
        .collect()
}

fn sort_sections(bundle: &mut ContextBundle) {
    for section in [
        &mut bundle.repository_invariants,
        &mut bundle.workflow_steps,
        &mut bundle.relevant_patterns,
        &mut bundle.anti_patterns,
        &mut bundle.verification_expectations,
    ] {
        section.sort_unstable_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.id.cmp(&right.id))
        });
    }
}

fn render_with_estimate_v1(
    release_id: &str,
    workflow: WorkflowId,
    selected: &[ContextItem],
    token_budget: u32,
) -> Result<(String, u32), AppError> {
    let selected_ids = selected
        .iter()
        .map(|item| item.id.as_str())
        .collect::<Vec<_>>();
    let mut estimate = 0;

    for _ in 0..16 {
        let rendered = render_document_v1(release_id, workflow, selected, &selected_ids, estimate);
        let next = estimate_tokens(&rendered)?;
        if next == estimate {
            if next > token_budget {
                return Err(context_budget(format!(
                    "complete context requires {next} tokens but budget is {token_budget}"
                )));
            }
            return Ok((rendered, next));
        }
        estimate = next;
    }

    Err(context_budget(
        "context token estimate did not reach a stable manifest value",
    ))
}

fn render_document_v1(
    release_id: &str,
    workflow: WorkflowId,
    selected: &[ContextItem],
    selected_ids: &[&str],
    estimated_tokens: u32,
) -> String {
    let mut document = String::new();
    document.push_str("# Harp context release\n\n");
    for (section, heading) in Section::ORDERED {
        writeln!(document, "## {heading}").expect("writing to a String cannot fail");
        for item in selected.iter().filter(|item| section.contains(item.kind)) {
            writeln!(document, "- {} (`{}`)", item.body, item.id)
                .expect("writing to a String cannot fail");
        }
        document.push('\n');
    }

    document.push_str("## Context manifest\n");
    writeln!(document, "- Release ID: `{release_id}`").expect("writing to a String cannot fail");
    writeln!(document, "- Workflow: `{workflow}`").expect("writing to a String cannot fail");
    if selected_ids.is_empty() {
        document.push_str("- Selected item IDs: (none)\n");
    } else {
        let ids = selected_ids
            .iter()
            .map(|id| format!("`{id}`"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(document, "- Selected item IDs: {ids}").expect("writing to a String cannot fail");
    }
    writeln!(document, "- Estimated tokens: {estimated_tokens}\n")
        .expect("writing to a String cannot fail");
    writeln!(document, "{ADVISORY_STATEMENT}").expect("writing to a String cannot fail");
    document
}

fn estimate_tokens(rendered: &str) -> Result<u32, AppError> {
    let characters = rendered
        .chars()
        .count()
        .checked_add(3)
        .ok_or_else(|| context_budget("context character count overflowed"))?;
    u32::try_from(characters / 4)
        .map_err(|_| context_budget("context token estimate exceeds the supported range"))
}

fn context_request(message: impl Into<String>) -> AppError {
    AppError::invalid_input("context.request", message)
}

fn context_release(message: impl Into<String>) -> AppError {
    AppError::invalid_input("context.release", message)
}

fn context_release_compatibility(message: impl Into<String>) -> AppError {
    AppError::invalid_input("context.release_compatibility", message)
}

fn context_budget(message: impl Into<String>) -> AppError {
    AppError::invalid_input("context.budget", message)
}

fn context_selector_version(message: impl Into<String>) -> AppError {
    AppError::invalid_input("context.selector_version", message)
}

fn context_renderer_version(message: impl Into<String>) -> AppError {
    AppError::invalid_input("context.renderer_version", message)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::context_control::canonical::{json_bytes_with_newline, sha256_hex, sha256_id};
    use crate::context_control::release::{
        compile_baseline_release, inspect_release_in_state, publish_release, ContextTemplateV1,
        ReleaseInspection, ReleaseManifest, WorkflowContextTemplateV1,
    };
    use crate::context_control::routing::route_with_rules;
    use crate::context_control::state::StateRoot;
    use crate::context_control::{
        ContextItemKind, ProviderId, WorkflowChoice, WorkflowId, CONTEXT_TEMPLATE_SCHEMA,
        RELEASE_IDENTITY_SCHEMA, RELEASE_SCHEMA,
    };

    #[test]
    fn identical_inputs_produce_identical_context_bytes() {
        let input = fixture(WorkflowId::CiRepair, "tests failing in CI");

        let first = resolve(&input).unwrap();
        let second = resolve(&input).unwrap();

        assert_eq!(first.rendered_markdown, second.rendered_markdown);
        assert_eq!(
            first.rendered_context_sha256,
            second.rendered_context_sha256
        );
        assert_eq!(first.selected_item_ids, second.selected_item_ids);
        assert_eq!(
            first.canonical_bytes().unwrap(),
            second.canonical_bytes().unwrap()
        );
    }

    #[test]
    fn resolver_obeys_a_lower_budget_and_records_rejections() {
        let mut input = fixture(WorkflowId::CiRepair, "tests failing in CI");
        input.token_budget = 120;

        let bundle = resolve(&input).unwrap();

        assert!(bundle.estimated_tokens <= 120);
        assert!(!bundle.rejected_items.is_empty());
        assert!(bundle
            .rejected_items
            .iter()
            .all(|item| item.reason == "context_budget"));
    }

    #[test]
    #[allow(clippy::manual_div_ceil)]
    fn rendered_document_has_the_approved_order_manifest_and_digest() {
        let bundle = resolve(&fixture(WorkflowId::DependencyUpdate, "upgrade dependency")).unwrap();
        let headings = [
            "# Harp context release",
            "## Repository invariants",
            "## Workflow",
            "## Relevant patterns",
            "## Anti-patterns",
            "## Required verification",
            "## Context manifest",
        ];
        let positions = headings.map(|heading| {
            bundle
                .rendered_markdown
                .find(heading)
                .unwrap_or_else(|| panic!("rendered context is missing {heading:?}"))
        });

        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(bundle
            .rendered_markdown
            .contains(&format!("Release ID: `{}`", bundle.release_id)));
        assert!(bundle
            .rendered_markdown
            .contains(&format!("Workflow: `{}`", bundle.workflow)));
        assert!(bundle.rendered_markdown.contains(
            "This context is advisory and cannot override provider policy or AGENTS.md."
        ));
        assert_eq!(
            bundle.estimated_tokens as usize,
            (bundle.rendered_markdown.chars().count() + 3) / 4
        );
        assert_eq!(
            bundle.rendered_context_sha256,
            sha256_hex(bundle.rendered_markdown.as_bytes())
        );
    }

    #[test]
    fn eligible_items_use_priority_descending_then_id_order() {
        let input = fixture(WorkflowId::CiRepair, "tests failing in CI");
        let bundle = resolve_test_items(
            &input,
            [
                test_item("z-workflow", ContextItemKind::WorkflowStep, "Z step"),
                test_item("a-example", ContextItemKind::Example, "Example"),
                test_item(
                    "z-verification",
                    ContextItemKind::VerificationRecipe,
                    "Verify",
                ),
                test_item("a-workflow", ContextItemKind::WorkflowStep, "A step"),
            ],
        )
        .unwrap();

        assert_eq!(
            bundle.selected_item_ids,
            ["z-verification", "a-workflow", "z-workflow", "a-example"]
        );
        assert_eq!(
            item_ids(&bundle.workflow_steps),
            ["a-workflow", "z-workflow"]
        );
    }

    #[test]
    fn required_verification_outranks_an_equal_cost_optional_example() {
        let verification = test_item(
            "verify-000",
            ContextItemKind::VerificationRecipe,
            "Use the exact required check.",
        );
        let example = test_item(
            "example-00",
            ContextItemKind::Example,
            "Use the exact required check.",
        );
        let verification_only = fixture(WorkflowId::CiRepair, "tests failing in CI");
        let one_item_budget = resolve_test_items(&verification_only, [verification.clone()])
            .unwrap()
            .estimated_tokens;

        let mut input = fixture(WorkflowId::CiRepair, "tests failing in CI");
        input.token_budget = one_item_budget;
        let bundle = resolve_test_items(&input, [example, verification]).unwrap();

        assert_eq!(bundle.selected_item_ids, ["verify-000"]);
        assert_eq!(
            bundle.rejected_items,
            [RejectedContextItem {
                id: "example-00".to_owned(),
                reason: "context_budget".to_owned(),
            }]
        );
    }

    #[test]
    fn over_budget_items_are_rejected_whole_without_truncation() {
        let marker = "NEVER_TRUNCATE_THIS_ITEM";
        let body = format!("{marker} {}", "content ".repeat(1_000));
        let mut input = fixture(WorkflowId::CiRepair, "tests failing in CI");
        input.token_budget = 120;
        let bundle = resolve_test_items(
            &input,
            [test_item(
                "oversized-example",
                ContextItemKind::Example,
                &body,
            )],
        )
        .unwrap();

        assert_eq!(bundle.selected_item_ids, Vec::<String>::new());
        assert_eq!(
            bundle.rejected_items,
            [RejectedContextItem {
                id: "oversized-example".to_owned(),
                reason: "context_budget".to_owned(),
            }]
        );
        assert!(!bundle.rendered_markdown.contains(marker));
        assert!(bundle.estimated_tokens <= input.token_budget);
    }

    #[test]
    fn bundle_bytes_are_provider_neutral() {
        let bundle = resolve(&fixture(
            WorkflowId::GeneralCoding,
            "implement bounded context",
        ))
        .unwrap();
        let bytes = bundle.canonical_bytes().unwrap();
        let json = std::str::from_utf8(&bytes).unwrap();

        assert!(!json.contains("\"provider\""));
        assert!(!bundle.rendered_markdown.contains("Trae"));
        assert!(!bundle.rendered_markdown.contains("Codex"));
    }

    #[test]
    fn budget_must_fit_the_complete_empty_document_and_not_exceed_release_limit() {
        let mut too_small = fixture(WorkflowId::CiRepair, "tests failing in CI");
        too_small.token_budget = 1;
        assert_eq!(resolve(&too_small).unwrap_err().code(), "context.budget");

        let mut too_large = fixture(WorkflowId::CiRepair, "tests failing in CI");
        too_large.token_budget += 1;
        assert_eq!(resolve(&too_large).unwrap_err().code(), "context.budget");
    }

    #[test]
    fn changing_the_task_changes_the_automatic_route() {
        let mut input = auto_fixture("tests failing in CI");
        let repair = resolve(&input).unwrap();

        input.task = "review this diff and find bugs".to_owned();
        let review = resolve(&input).unwrap();

        assert_eq!(repair.workflow, WorkflowId::CiRepair);
        assert_eq!(review.workflow, WorkflowId::CodeReview);
        assert_ne!(repair.routing_trace, review.routing_trace);
    }

    #[test]
    fn explicit_workflow_choice_wins_over_task_matches() {
        let bundle = resolve(&fixture(
            WorkflowId::DependencyUpdate,
            "review this diff and find bugs",
        ))
        .unwrap();

        assert_eq!(bundle.workflow, WorkflowId::DependencyUpdate);
        assert!(bundle.routing_trace.explicit);
    }

    #[test]
    fn selected_workflow_budget_applies() {
        let mut review = auto_fixture("review this diff and find bugs");
        review.token_budget = 1_801;
        assert_eq!(resolve(&review).unwrap_err().code(), "context.budget");

        let mut repair = auto_fixture("tests failing in CI");
        repair.token_budget = 1_801;
        assert_eq!(resolve(&repair).unwrap().workflow, WorkflowId::CiRepair);
    }

    #[test]
    fn unsupported_selector_and_renderer_versions_are_typed() {
        assert_eq!(
            ensure_supported_versions("v2", RENDERER_V1)
                .unwrap_err()
                .code(),
            "context.selector_version"
        );
        assert_eq!(
            ensure_supported_versions(SELECTOR_V1, "v2")
                .unwrap_err()
                .code(),
            "context.renderer_version"
        );
    }

    #[test]
    fn legacy_release_without_pinned_routing_fails_closed() {
        let release = inspected_legacy_release();
        assert!(release.routing_template_v2().is_none());
        let token_budget = release
            .identity_v1()
            .expect("legacy fixture decodes as V1")
            .context_budgets[&WorkflowId::CiRepair];
        let input = ContextRequest {
            task: "tests failing in CI".to_owned(),
            release,
            workflow: WorkflowChoice::Auto,
            token_budget,
        };

        assert_eq!(
            resolve(&input).unwrap_err().code(),
            "context.release_compatibility"
        );
    }

    #[test]
    fn pinned_release_route_survives_simulated_current_routing_evolution() {
        let original = auto_fixture("tests failing in CI");
        let expected = resolve(&original).unwrap();

        let mut evolved_rules = original
            .release
            .routing_template_v2()
            .expect("V2 release has pinned routing")
            .workflows
            .iter()
            .map(|(workflow, template)| (*workflow, template.routing.clone()))
            .collect::<Vec<_>>();
        evolved_rules
            .iter_mut()
            .find(|(workflow, _)| *workflow == WorkflowId::CiRepair)
            .unwrap()
            .1
            .positive_phrases
            .clear();
        evolved_rules
            .iter_mut()
            .find(|(workflow, _)| *workflow == WorkflowId::CodeReview)
            .unwrap()
            .1
            .positive_phrases
            .push("tests failing".to_owned());
        let evolved_refs = evolved_rules
            .iter()
            .map(|(workflow, rules)| (*workflow, rules))
            .collect::<Vec<_>>();
        let evolved = route_with_rules(&original.task, WorkflowChoice::Auto, &evolved_refs);
        assert_eq!(evolved.selected, WorkflowId::CodeReview);

        let actual = resolve(&original).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.workflow, WorkflowId::CiRepair);
    }

    #[test]
    fn request_has_no_caller_supplied_routing_authority() {
        let source = include_str!("context.rs");
        let request = source
            .split("pub struct ContextRequest {")
            .nth(1)
            .and_then(|tail| tail.split("}\n").next())
            .expect("ContextRequest declaration");

        assert!(request.contains("pub workflow: WorkflowChoice"));
        assert!(!request.contains("routing_trace"));
        assert!(!request.contains("RouteDecision"));
    }

    #[test]
    fn request_release_is_an_opaque_validated_value() {
        let release_source = include_str!("release.rs");
        let inspection = release_source
            .split("pub struct ReleaseInspection {")
            .nth(1)
            .and_then(|tail| tail.split("}\n").next())
            .expect("ReleaseInspection declaration");

        assert!(!inspection.lines().any(|line| line.contains("pub ")));
        let request = fixture(WorkflowId::CiRepair, "tests failing in CI");
        assert_eq!(
            request.release.release_id(),
            compile_baseline_release().unwrap().release_id()
        );
    }

    #[test]
    fn renderer_v1_bytes_and_hash_are_golden_pinned() {
        let bundle = resolve(&fixture(WorkflowId::CiRepair, "tests failing in CI")).unwrap();

        assert_eq!(
            bundle.rendered_markdown,
            "\
# Harp context release

## Repository invariants

## Workflow
- Classify before editing (`classify-before-editing`)
- Reproduce the narrowest failure (`reproduce-narrowest-failure`)

## Relevant patterns

## Anti-patterns
- Do not weaken tests (`do-not-weaken-tests`)

## Required verification
- Rerun the original verification mode (`rerun-original-verification-mode`)

## Context manifest
- Release ID: `sha256-8115ca08c56ecdffc9939e8d0c563127633f18fbb638a24892f345c9206833f1`
- Workflow: `ci_repair`
- Selected item IDs: `rerun-original-verification-mode`, `classify-before-editing`, `reproduce-narrowest-failure`, `do-not-weaken-tests`
- Estimated tokens: 185

This context is advisory and cannot override provider policy or AGENTS.md.
"
        );
        assert_eq!(
            bundle.rendered_context_sha256,
            "ee511bd74d0f1010332bb574760c81607177873686750b5af8e117c14ac6854f"
        );
    }

    #[test]
    fn resolver_source_has_no_direct_side_effect_apis() {
        let source = include_str!("context.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .expect("context module must keep tests behind cfg(test)")
            .0;
        for forbidden in [
            "std::fs",
            "std::net",
            "std::process",
            "Command::",
            "TcpStream",
            "UdpSocket",
        ] {
            assert!(
                !production.contains(forbidden),
                "context resolver contains forbidden side-effect API {forbidden}"
            );
        }
    }

    fn fixture(workflow: WorkflowId, task: &str) -> ContextRequest {
        let release = inspected_baseline();
        let token_budget = release
            .identity_v2()
            .expect("baseline fixture decodes as V2")
            .context_budgets[&workflow];
        ContextRequest {
            task: task.to_owned(),
            release,
            workflow: WorkflowChoice::Workflow(workflow),
            token_budget,
        }
    }

    fn auto_fixture(task: &str) -> ContextRequest {
        ContextRequest {
            task: task.to_owned(),
            release: inspected_baseline(),
            workflow: WorkflowChoice::Auto,
            token_budget: 1_600,
        }
    }

    fn inspected_baseline() -> ReleaseInspection {
        let temp = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(temp.path()).unwrap().join("state");
        let state = StateRoot::open_or_create(&root).unwrap();
        let release = compile_baseline_release().unwrap();
        publish_release(&state, &release).unwrap();
        inspect_release_in_state(&state, release.release_id()).unwrap()
    }

    fn inspected_legacy_release() -> ReleaseInspection {
        #[derive(Serialize)]
        struct LegacyReleaseIdentity {
            schema_version: String,
            packages: BTreeMap<WorkflowId, String>,
            selector_version: String,
            renderer_version: String,
            context_template_schema: String,
            context_budgets: BTreeMap<WorkflowId, u32>,
            compatible_providers: BTreeSet<ProviderId>,
        }

        #[derive(Serialize)]
        struct ReleaseAuthorization {
            schema_version: String,
            release_id: String,
            members: BTreeMap<String, String>,
        }

        let temp = tempfile::tempdir().unwrap();
        let root = std::fs::canonicalize(temp.path()).unwrap().join("state");
        let state = StateRoot::open_or_create(&root).unwrap();
        let release = compile_baseline_release().unwrap();
        let identity = release.identity_v2();
        let identity_bytes = json_bytes(&LegacyReleaseIdentity {
            schema_version: RELEASE_IDENTITY_SCHEMA.to_owned(),
            packages: identity.packages.clone(),
            selector_version: identity.selector_version.clone(),
            renderer_version: identity.renderer_version.clone(),
            context_template_schema: CONTEXT_TEMPLATE_SCHEMA.to_owned(),
            context_budgets: identity.context_budgets.clone(),
            compatible_providers: identity.compatible_providers.clone(),
        })
        .unwrap();
        let release_id = sha256_id(&identity_bytes);
        let context_template = ContextTemplateV1 {
            schema_version: CONTEXT_TEMPLATE_SCHEMA.to_owned(),
            workflows: release
                .context_template_v2()
                .workflows
                .iter()
                .map(|(workflow, template)| {
                    (
                        *workflow,
                        WorkflowContextTemplateV1 {
                            context_budget_tokens: template.context_budget_tokens,
                            required_sections: template.required_sections.clone(),
                            verification_recipes: template.verification_recipes.clone(),
                            verified_success: template.verified_success.clone(),
                            weak_signals: template.weak_signals.clone(),
                        },
                    )
                })
                .collect(),
        };
        let context_template_bytes = json_bytes_with_newline(&context_template).unwrap();
        let items_bytes = release
            .items()
            .iter()
            .flat_map(|item| json_bytes_with_newline(item).unwrap())
            .collect::<Vec<_>>();
        let manifest = ReleaseManifest {
            schema_version: RELEASE_SCHEMA.to_owned(),
            release_id: release_id.clone(),
            members: BTreeMap::from([
                (
                    "context_template.json".to_owned(),
                    sha256_hex(&context_template_bytes),
                ),
                ("identity.json".to_owned(), sha256_hex(&identity_bytes)),
                ("items.jsonl".to_owned(), sha256_hex(&items_bytes)),
            ]),
        };
        let members = BTreeMap::from([
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
        ]);
        state
            .publish_immutable_tree(&Path::new("releases").join(&release_id), &members)
            .unwrap();
        let authorization = ReleaseAuthorization {
            schema_version: "harp-context-release-authorization/v1".to_owned(),
            release_id: release_id.clone(),
            members: members
                .iter()
                .map(|(name, bytes)| {
                    (
                        name.to_string_lossy().into_owned(),
                        sha256_hex(bytes.as_slice()),
                    )
                })
                .collect(),
        };
        state
            .publish_release_authorization(
                &release_id,
                &json_bytes_with_newline(&authorization).unwrap(),
            )
            .unwrap();
        inspect_release_in_state(&state, &release_id).unwrap()
    }

    fn resolve_test_items(
        input: &ContextRequest,
        items: impl IntoIterator<Item = ContextItem>,
    ) -> Result<ContextBundle, AppError> {
        let routing_trace = route_from_release(input);
        resolve_candidates_v1(input, routing_trace, items.into_iter().collect())
    }

    fn route_from_release(input: &ContextRequest) -> RouteDecision {
        let rules = input
            .release
            .routing_template_v2()
            .expect("test candidates use a V2 release")
            .workflows
            .iter()
            .map(|(workflow, template)| (*workflow, &template.routing))
            .collect::<Vec<_>>();
        route_with_rules(&input.task, input.workflow, &rules)
    }

    fn test_item(id: &str, kind: ContextItemKind, body: &str) -> ContextItem {
        ContextItem {
            id: id.to_owned(),
            kind,
            body: body.to_owned(),
            priority: item_priority_v1(kind),
        }
    }

    fn item_ids(items: &[ContextItem]) -> Vec<&str> {
        items.iter().map(|item| item.id.as_str()).collect()
    }
}
