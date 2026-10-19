use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use sha2::{Digest as _, Sha256};

use crate::error::AppError;
use crate::fs::HeldDirectory;
use crate::knowledge::WikiLinkResolution;

mod contracts;
mod obsidian;
mod registration;
mod render;

use contracts::ValidatedInputs;

pub(crate) fn canonical_frontmatter_identity(
    markdown: &str,
    path: &str,
) -> Result<String, AppError> {
    contracts::canonical_frontmatter_identity(markdown, path)
}

pub(crate) fn canonical_markdown_body<'a>(
    markdown: &'a str,
    path: &str,
) -> Result<&'a str, AppError> {
    render::markdown_body(markdown, path)
}

pub(crate) fn canonical_heading_ids(markdown: &str, path: &str) -> Result<Vec<String>, AppError> {
    render::validated_heading_ids(markdown, path)
}

pub(super) const CONTENT_ROOT: &str = "lab/math/textbook";
const SOURCE_SUMMARY: &str = "<summary>Original sources for this mechanism</summary>";
const REFERENCE_SUMMARY: &str = "<summary>Reference records and operational metadata</summary>";
const MAX_MARKDOWN_BYTES: usize = 512 * 1024;
pub(crate) fn resolve_wiki_links(
    repository_root: &Path,
    source_path: &Path,
    markdown: &str,
) -> Result<Vec<WikiLinkResolution>, AppError> {
    let source_path = source_path.strip_prefix(repository_root).map_err(|_| {
        AppError::invalid_input(
            "knowledge.obsidian.source_path",
            "Obsidian source path must be inside the repository",
        )
    })?;
    if source_path.as_os_str().is_empty()
        || source_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AppError::invalid_input(
            "knowledge.obsidian.source_path",
            "Obsidian source path must be a nonempty normal repository-relative path",
        ));
    }
    let repository = HeldDirectory::open(repository_root, "Harp repository")?;
    obsidian::parse_wiki_links(markdown)
        .map_err(|error| AppError::invalid_input("knowledge.obsidian.parse", error))?
        .into_iter()
        .map(|link| {
            let resolved =
                obsidian::resolve_wiki_link(&repository, &source_path.to_string_lossy(), &link)?;
            Ok(WikiLinkResolution {
                target: resolved.target,
                heading_id: resolved.heading_id,
                requested_heading: match link.subpath {
                    Some(obsidian::WikiSubpath::Heading(heading)) => Some(heading),
                    _ => None,
                },
                pdf_page: resolved.pdf_page,
                embed: resolved.embed,
                display: resolved.display,
            })
        })
        .collect()
}

pub(super) const READER_ROUTES: &[(&str, &str, &str)] = &[
    (
        "agentic-eval-apply",
        "Agentic eval/apply",
        "lab/sicp/agentic_eval_apply.md",
    ),
    (
        "benchmarks",
        "Benchmarks",
        "lab/benchmarks/harness_benchmarks/harness_benchmark_field_guide.md",
    ),
    (
        "evaluator-integrity",
        "Evaluator integrity",
        "lab/benchmarks/evaluator_integrity/evaluator_integrity_benchmark_suite.md",
    ),
    (
        "agentic-engineering",
        "Agentic engineering",
        "lab/agentic_engineering/kenn_reference_architecture.md",
    ),
    (
        "crouzeix-conjecture",
        "Crouzeix",
        "lab/math/crouzeix_conjecture/crouzeix_conjecture_index.md",
    ),
    (
        "crouzeix-textbook",
        "Crouzeix textbook",
        "lab/math/crouzeix_textbook/crouzeix_textbook_index.md",
    ),
    (
        "mathematical-foundations",
        "Math foundations",
        "lab/math/mathematical_foundations/mathematical_foundations_index.md",
    ),
    (
        "autodiff-geometry",
        "Autodiff geometry",
        "lab/math/autodiff_geometry/autodiff_geometry_index.md",
    ),
    (
        "article-survey",
        "Article survey",
        "lab/article_survey.md",
    ),
    ("sicp", "SICP", "lab/sicp/sicp_index.md"),
];
pub(super) const AUXILIARY_DOCUMENTS: &[(&str, &str)] = &[
    ("agentic-eval-apply", "lab/sicp/agentic_eval_apply.md"),
    (
        "sicp-seminar-09-eval-apply",
        "lab/sicp/course/seminars/09-eval-apply-and-executable-semantics.md",
    ),
    (
        "sicp-evaluator-deep-dive",
        "lab/sicp/metacircular_evaluator_deep_dive.md",
    ),
    (
        "pi-harness-deep-dive",
        "lab/agentic_engineering/pi_harness_deep_dive.md",
    ),
    (
        "hermes-harness-deep-dive",
        "lab/agentic_engineering/hermes_harness_deep_dive.md",
    ),
    (
        "codex-harness-deep-dive",
        "lab/agentic_engineering/codex_harness_deep_dive.md",
    ),
    (
        "deepseek-harness-deep-dive",
        "lab/agentic_engineering/deepseek_harness_deep_dive.md",
    ),
    (
        "deepseek-harness-study-index",
        "lab/agentic_engineering/deepseek_harness/deepseek_harness_index.md",
    ),
    (
        "deepseek-harness-cordis-spatiotemporal-composition",
        "lab/agentic_engineering/deepseek_harness/01_cordis_spatiotemporal_composition.md",
    ),
    (
        "deepseek-harness-profiles-bundles-and-patch-layers",
        "lab/agentic_engineering/deepseek_harness/02_profiles_bundles_and_patch_layers.md",
    ),
    (
        "deepseek-harness-events-effects-and-reversible-lifecycle",
        "lab/agentic_engineering/deepseek_harness/03_events_effects_and_reversible_lifecycle.md",
    ),
    (
        "deepseek-harness-turn-session-and-model-visible-log",
        "lab/agentic_engineering/deepseek_harness/04_turn_session_and_model_visible_log.md",
    ),
    (
        "deepseek-harness-capability-seams",
        "lab/agentic_engineering/deepseek_harness/05_capability_seams.md",
    ),
    (
        "deepseek-harness-tool-registry-and-policy-pipeline",
        "lab/agentic_engineering/deepseek_harness/06_tool_registry_and_policy_pipeline.md",
    ),
    (
        "deepseek-harness-sandbox-permission-and-filesystem-boundaries",
        "lab/agentic_engineering/deepseek_harness/07_sandbox_permission_and_filesystem_boundaries.md",
    ),
    (
        "deepseek-harness-subagents-and-continuable-children",
        "lab/agentic_engineering/deepseek_harness/08_subagents_and_continuable_children.md",
    ),
    (
        "deepseek-harness-persistence-sdk-and-runtime-packaging",
        "lab/agentic_engineering/deepseek_harness/09_persistence_sdk_and_runtime_packaging.md",
    ),
    (
        "deepseek-harness-self-modification-and-dynamic-cordis",
        "lab/agentic_engineering/deepseek_harness/10_self_modification_and_dynamic_cordis.md",
    ),
    (
        "deepseek-harness-source-registry",
        "lab/agentic_engineering/deepseek_harness/source_registry.md",
    ),
    (
        "deepseek-harness-claim-evidence-ledger",
        "lab/agentic_engineering/deepseek_harness/claim_evidence_ledger.md",
    ),
    (
        "agent-harness-architecture-dossier",
        "lab/sicp/course/capstone/agent_harness_architecture_dossier.md",
    ),
    (
        "codex-state-continuity",
        "lab/agentic_engineering/codex_state_continuity_and_compaction.md",
    ),
    (
        "harness-benchmark-field-guide",
        "lab/benchmarks/harness_benchmarks/harness_benchmark_field_guide.md",
    ),
    (
        "evaluator-integrity-benchmark-suite",
        "lab/benchmarks/evaluator_integrity/evaluator_integrity_benchmark_suite.md",
    ),
    (
        "agentic-engineering-index",
        "lab/agentic_engineering/agentic_engineering_index.md",
    ),
    (
        "agentic-engineering-reference",
        "lab/agentic_engineering/kenn_reference_architecture.md",
    ),
    (
        "agentic-engineering-tool-stack",
        "lab/agentic_engineering/tool_stack_investigation.md",
    ),
    (
        "agentic-engineering-source-registry",
        "lab/agentic_engineering/source_registry.md",
    ),
    (
        "agentic-engineering-claim-ledger",
        "lab/agentic_engineering/claim_evidence_ledger.md",
    ),
    (
        "agentic-engineering-missing-evidence",
        "lab/agentic_engineering/missing_evidence.md",
    ),
    (
        "crouzeix-problem-and-prior-barrier",
        "lab/math/crouzeix_conjecture/01_problem_and_prior_barrier.md",
    ),
    (
        "crouzeix-shared-power-family",
        "lab/math/crouzeix_conjecture/02_shared_power_family.md",
    ),
    (
        "crouzeix-jin-proof-spine",
        "lab/math/crouzeix_conjecture/03_jin_proof_spine.md",
    ),
    (
        "crouzeix-jin-positive-real-completion",
        "lab/math/crouzeix_conjecture/04_jin_positive_real_completion.md",
    ),
    (
        "crouzeix-lorist-schwenninger-proof",
        "lab/math/crouzeix_conjecture/05_lorist_schwenninger_proof.md",
    ),
    (
        "crouzeix-proof-comparison",
        "lab/math/crouzeix_conjecture/06_proof_comparison.md",
    ),
    (
        "crouzeix-jin-lean-verification",
        "lab/math/crouzeix_conjecture/07_jin_lean_verification.md",
    ),
    (
        "crouzeix-ai-assisted-discovery",
        "lab/math/crouzeix_conjecture/08_ai_assisted_discovery.md",
    ),
    (
        "crouzeix-status-and-critical-assessment",
        "lab/math/crouzeix_conjecture/09_status_and_critical_assessment.md",
    ),
    (
        "crouzeix-jin-proof-editorial",
        "lab/math/crouzeix_conjecture/10_jin_proof_editorial.md",
    ),
    (
        "crouzeix-proof-reproduction-research",
        "lab/math/crouzeix_conjecture/reproduction_research.md",
    ),
    (
        "crouzeix-glossary",
        "lab/math/crouzeix_conjecture/glossary.md",
    ),
    (
        "crouzeix-source-registry",
        "lab/math/crouzeix_conjecture/source_registry.md",
    ),
    (
        "crouzeix-claim-evidence-ledger",
        "lab/math/crouzeix_conjecture/claim_evidence_ledger.md",
    ),
    (
        "math-foundations-index",
        "lab/math/mathematical_foundations/mathematical_foundations_index.md",
    ),
    (
        "math-foundations-linear-spaces-and-maps",
        "lab/math/mathematical_foundations/01_linear_spaces_and_maps.md",
    ),
    (
        "math-foundations-orthogonality-spectra-and-decompositions",
        "lab/math/mathematical_foundations/02_orthogonality_spectra_and_decompositions.md",
    ),
    (
        "math-foundations-probability-and-gaussian-models",
        "lab/math/mathematical_foundations/03_probability_and_gaussian_models.md",
    ),
    (
        "math-foundations-bayesian-inference-and-information",
        "lab/math/mathematical_foundations/04_bayesian_inference_and_information.md",
    ),
    (
        "math-foundations-linear-models-and-regularization",
        "lab/math/mathematical_foundations/05_linear_models_and_regularization.md",
    ),
    (
        "math-foundations-optimization-and-iterative-methods",
        "lab/math/mathematical_foundations/06_optimization_and_iterative_methods.md",
    ),
    (
        "math-foundations-curriculum-map",
        "lab/math/mathematical_foundations/curriculum_map.md",
    ),
    (
        "math-foundations-glossary",
        "lab/math/mathematical_foundations/glossary.md",
    ),
    (
        "math-foundations-source-registry",
        "lab/math/mathematical_foundations/source_registry.md",
    ),
    (
        "math-foundations-claim-evidence-ledger",
        "lab/math/mathematical_foundations/claim_evidence_ledger.md",
    ),
    (
        "math-foundations-formalization-map",
        "lab/math/mathematical_foundations/formalization_map.md",
    ),
    (
        "autodiff-geometry-index",
        "lab/math/autodiff_geometry/autodiff_geometry_index.md",
    ),
    (
        "autodiff-geometry-source-registry",
        "lab/math/autodiff_geometry/source_registry.md",
    ),
    (
        "autodiff-geometry-claim-evidence-ledger",
        "lab/math/autodiff_geometry/claim_evidence_ledger.md",
    ),
    (
        "autodiff-geometry-formalization-roadmap",
        "lab/math/autodiff_geometry/formalization_roadmap.md",
    ),
    (
        "autodiff-geometry-source-acquisition-and-parsing",
        "lab/math/autodiff_geometry/source_acquisition_and_parsing.md",
    ),
    (
        "autodiff-geometry-layered-curriculum-tracker",
        "lab/math/autodiff_geometry/layered_curriculum_tracker.md",
    ),
    (
        "sicp-dialogue-state",
        "lab/sicp/course/dialogue_state.md",
    ),
    ("sicp-exercise-map", "lab/sicp/course/exercise_map.md"),
    (
        "sicp-scheme-reading-primer",
        "lab/sicp/course/scheme_reading_primer.md",
    ),
    (
        "sicp-01-process-shape-and-higher-order-abstraction",
        "lab/sicp/course/seminars/01-process-shape-and-higher-order-abstraction.md",
    ),
    (
        "sicp-02-data-abstraction-and-immutable-representation",
        "lab/sicp/course/seminars/02-data-abstraction-and-immutable-representation.md",
    ),
    (
        "sicp-03-trees-sequences-and-functional-interfaces",
        "lab/sicp/course/seminars/03-trees-sequences-and-functional-interfaces.md",
    ),
    (
        "sicp-04-symbols-sets-compression-and-generic-dispatch",
        "lab/sicp/course/seminars/04-symbols-sets-compression-and-generic-dispatch.md",
    ),
    (
        "sicp-05-state-identity-and-environments",
        "lab/sicp/course/seminars/05-state-identity-and-environments.md",
    ),
    (
        "sicp-06-mutation-simulation-and-constraints",
        "lab/sicp/course/seminars/06-mutation-simulation-and-constraints.md",
    ),
    (
        "sicp-07-concurrency-serialization-and-interleavings",
        "lab/sicp/course/seminars/07-concurrency-serialization-and-interleavings.md",
    ),
    (
        "sicp-08-streams-delay-and-infinite-processes",
        "lab/sicp/course/seminars/08-streams-delay-and-infinite-processes.md",
    ),
    (
        "sicp-10-lazy-evaluation-and-nondeterministic-search",
        "lab/sicp/course/seminars/10-lazy-evaluation-and-nondeterministic-search.md",
    ),
    (
        "sicp-11-logic-programming-and-declarative-query",
        "lab/sicp/course/seminars/11-logic-programming-and-declarative-query.md",
    ),
    (
        "sicp-12-machines-storage-control-and-compilation",
        "lab/sicp/course/seminars/12-machines-storage-control-and-compilation.md",
    ),
    (
        "sicp-sicp-course-guide",
        "lab/sicp/course/sicp_course_guide.md",
    ),
    (
        "sicp-domain-orientation",
        "lab/sicp/domain_orientation.md",
    ),
    (
        "sicp-eval-apply-reader-companion",
        "lab/sicp/eval-apply-reader-companion.md",
    ),
    (
        "sicp-missing-evidence",
        "lab/sicp/missing_evidence.md",
    ),
    ("sicp-index", "lab/sicp/sicp_index.md"),
];
const KNOWLEDGE_HOME: (&str, &str, &str) =
    ("knowledge", "Knowledge", "lab/harp_knowledge_home.md");
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct CanonicalDocument {
    pub(super) concept_id: String,
    pub(super) title: String,
    pub(super) canonical_markdown_path: String,
    pub(super) markdown_sha256: String,
    pub(super) html_sha256: String,
    pub(super) html: String,
    pub(super) metadata: DocumentMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct DocumentMetadata {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) status: String,
    pub(super) tags: Vec<String>,
    pub(super) confidence: String,
    pub(super) mode: Option<String>,
    pub(super) source_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ReaderRoute {
    pub(super) route_id: String,
    pub(super) label: String,
    pub(super) canonical_markdown_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct DocumentAlias {
    pub(super) alias_id: String,
    pub(super) canonical_document_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct KnowledgeCorpus {
    pub(super) schema_version: &'static str,
    pub(super) reader_routes: Vec<ReaderRoute>,
    pub(super) document_aliases: Vec<DocumentAlias>,
    pub(super) documents: Vec<CanonicalDocument>,
}

pub(super) fn compile(repo_root: &Path) -> Result<KnowledgeCorpus, AppError> {
    let repository = HeldDirectory::open(repo_root, "Harp repository")?;
    let registration = registration::static_registration()?;
    let ValidatedInputs {
        canonical_sources,
        registered_document_ids,
        registered_document_aliases,
    } = contracts::load(&repository)?;
    let route_targets = render::route_targets(&canonical_sources, &registered_document_ids);
    let documents = canonical_sources
        .values()
        .map(|source| {
            render::compile_document(source, &route_targets, &registered_document_ids)
                .map(|document| (source.path.clone(), document))
        })
        .collect::<Result<BTreeMap<_, _>, AppError>>()?;
    let mut reader_routes = registration
        .reader_routes()
        .map(|document| {
            let reader = document.reader.as_ref().expect("reader registration");
            ReaderRoute {
                route_id: reader.route_id.clone(),
                label: reader.label.clone(),
                canonical_markdown_path: document.path.clone(),
            }
        })
        .collect::<Vec<_>>();
    if canonical_sources.contains_key(KNOWLEDGE_HOME.2) {
        reader_routes.push(ReaderRoute {
            route_id: KNOWLEDGE_HOME.0.into(),
            label: KNOWLEDGE_HOME.1.into(),
            canonical_markdown_path: KNOWLEDGE_HOME.2.into(),
        });
    }
    validate_document_namespace(
        &documents,
        &registered_document_ids,
        &registered_document_aliases,
    )?;
    Ok(KnowledgeCorpus {
        schema_version: "harp-knowledge/v1",
        reader_routes,
        document_aliases: registered_document_aliases
            .into_iter()
            .map(|(alias_id, canonical_document_id)| DocumentAlias {
                alias_id,
                canonical_document_id,
            })
            .collect(),
        documents: documents.into_values().collect(),
    })
}

pub(super) fn validate_build_output(output: &Path) -> Result<PathBuf, AppError> {
    if output.is_absolute()
        || !output.starts_with("lab/generated")
        || output.extension().and_then(|value| value.to_str()) != Some("json")
        || !output
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(invalid(
            "corpus.output_path",
            "output must stay beneath lab/generated",
        ));
    }
    Ok(output.to_path_buf())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|group| {
            !group.is_empty()
                && group
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn validate_document_namespace(
    documents: &BTreeMap<String, CanonicalDocument>,
    registered_document_ids: &BTreeMap<String, String>,
    registered_document_aliases: &BTreeMap<String, String>,
) -> Result<(), AppError> {
    let mut canonical_paths = BTreeMap::<&str, &str>::new();
    for (path, document) in documents {
        if !valid_id(&document.concept_id) {
            return Err(invalid(
                "knowledge.rsi.document_identity",
                format!(
                    "{path} has invalid canonical document ID {}",
                    document.concept_id
                ),
            ));
        }
        if let Some(previous_path) = canonical_paths.insert(&document.concept_id, path) {
            return Err(invalid(
                "knowledge.rsi.document_identity",
                format!(
                    "canonical document ID {} is shared by {previous_path} and {path}",
                    document.concept_id
                ),
            ));
        }
    }

    for (path, registered_id) in registered_document_ids {
        let document = documents.get(path).ok_or_else(|| {
            invalid(
                "knowledge.rsi.document_identity",
                format!("registered document {registered_id} has no compiled source at {path}"),
            )
        })?;
        if document.concept_id != *registered_id {
            return Err(invalid(
                "knowledge.rsi.document_identity",
                format!(
                    "registered document {registered_id} resolved as {} at {path}",
                    document.concept_id
                ),
            ));
        }
    }

    for (alias, canonical_id) in registered_document_aliases {
        if !valid_id(alias) || !valid_id(canonical_id) {
            return Err(invalid(
                "knowledge.rsi.document_identity",
                format!("document alias {alias} -> {canonical_id} is not a canonical ID pair"),
            ));
        }
        if !canonical_paths.contains_key(canonical_id.as_str()) {
            return Err(invalid(
                "knowledge.rsi.document_identity",
                format!("document alias {alias} has unknown canonical target {canonical_id}"),
            ));
        }
        if alias != canonical_id && canonical_paths.contains_key(alias.as_str()) {
            return Err(invalid(
                "knowledge.rsi.document_identity",
                format!("document alias {alias} collides with a canonical document"),
            ));
        }
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn invalid(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::invalid_input(code, message)
}

#[cfg(test)]
mod tests;
