use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use sha2::{Digest as _, Sha256};

use crate::error::AppError;
use crate::fs::HeldDirectory;

mod contracts;
mod lessons;
mod render;
mod rules;

use contracts::ValidatedRsiInputs;
#[cfg(test)]
use contracts::{SYSTEM_READINGS_PATH, WENG_MAP_PATH};

pub(super) const COVERAGE_PATH: &str = "content/coverage-map.tsv";
pub(super) const COVERAGE_HEADER: &str =
    "concept_id\tcoverage_depth\tcanonical_markdown_path\tsection_id\tparent_concept_id";
pub(super) const RETAINED_CONCEPTS_PATH: &str = "content/retained-concepts.tsv";
pub(super) const CONTENT_ROOT: &str = "content";
const RETAINED_CONCEPTS_HEADER: &str = "concept_id\tsource_ids";
const SOURCE_SUMMARY: &str = "<summary>Original sources for this mechanism</summary>";
const REFERENCE_SUMMARY: &str = "<summary>Reference records and operational metadata</summary>";
const MAX_COVERAGE_BYTES: usize = 256 * 1024;
const MAX_MARKDOWN_BYTES: usize = 512 * 1024;
const SOURCE_REGISTRY_PATH: &str = "content/sources/source_registry.tsv";
const EVIDENCE_GRAPH_PATH: &str = "content/sources/evidence_graph.tsv";
pub(super) const EVIDENCE_GRAPH_HEADER: &str =
    "label\tsource_id\trelationship\ttarget_id\tevidence_locator\tstatus\tboundary";
pub(super) const READER_ROUTES: [(&str, &str, &str); 13] = [
    (
        "thesis",
        "Thesis",
        "knowledge/rsi/chapters/recursive-improvement-loop.md",
    ),
    (
        "loop",
        "Loop",
        "knowledge/rsi/concepts/system-state-and-notation.md",
    ),
    (
        "methods",
        "Methods",
        "knowledge/rsi/concepts/harness-search-methods.md",
    ),
    (
        "harnesses",
        "Harnesses",
        "knowledge/rsi/chapters/harness-engineering.md",
    ),
    (
        "weng",
        "Weng",
        "knowledge/rsi/rsi_harness_by_lil_log_deconstructed.md",
    ),
    (
        "experiment",
        "Experiment",
        "knowledge/rsi/evaluator_integrity_and_promotion.md",
    ),
    ("sources", "Sources", "knowledge/rsi/source_registry.md"),
    (
        "agentic-eval-apply",
        "Agentic eval/apply",
        "knowledge/rsi/sicp/agentic_eval_apply.md",
    ),
    (
        "benchmarks",
        "Benchmarks",
        "knowledge/harness_benchmarks/harness_benchmark_field_guide.md",
    ),
    (
        "evaluator-integrity",
        "Evaluator integrity",
        "knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md",
    ),
    (
        "survey",
        "Survey",
        "knowledge/self_improving_agents_survey/synthesis.md",
    ),
    (
        "agentic-engineering",
        "Agentic engineering",
        "knowledge/agentic_engineering/kenn_reference_architecture.md",
    ),
    (
        "crouzeix-conjecture",
        "Crouzeix",
        "knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md",
    ),
];
pub(super) const AUXILIARY_DOCUMENTS: [(&str, &str); 35] = [
    (
        "agentic-eval-apply",
        "knowledge/rsi/sicp/agentic_eval_apply.md",
    ),
    (
        "sicp-seminar-09-eval-apply",
        "knowledge/rsi/sicp/course/seminars/09-eval-apply-and-executable-semantics.md",
    ),
    (
        "sicp-evaluator-deep-dive",
        "knowledge/rsi/sicp/metacircular_evaluator_deep_dive.md",
    ),
    (
        "pi-harness-deep-dive",
        "knowledge/rsi/pi_harness_deep_dive.md",
    ),
    (
        "hermes-harness-deep-dive",
        "knowledge/rsi/hermes_harness_deep_dive.md",
    ),
    (
        "codex-harness-deep-dive",
        "knowledge/rsi/codex_harness_deep_dive.md",
    ),
    (
        "agent-harness-architecture-dossier",
        "knowledge/rsi/sicp/course/capstone/agent_harness_architecture_dossier.md",
    ),
    (
        "codex-state-continuity",
        "knowledge/rsi/codex_state_continuity_and_compaction.md",
    ),
    (
        "context-engineering-deep-dive",
        "knowledge/rsi/context_engineering_deep_dive.md",
    ),
    (
        "meta-harness-deep-dive",
        "knowledge/meta_harness/meta_harness_deep_dive.md",
    ),
    (
        "harness-benchmark-field-guide",
        "knowledge/harness_benchmarks/harness_benchmark_field_guide.md",
    ),
    (
        "evaluator-integrity-benchmark-suite",
        "knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md",
    ),
    (
        "self-improving-agents-survey-synthesis",
        "knowledge/self_improving_agents_survey/synthesis.md",
    ),
    (
        "self-improving-agents-survey-source-registry",
        "knowledge/self_improving_agents_survey/source_registry.md",
    ),
    (
        "self-improving-agents-survey-claim-evidence-ledger",
        "knowledge/self_improving_agents_survey/claim_evidence_ledger.md",
    ),
    (
        "self-improving-agents-survey-gap-map",
        "knowledge/self_improving_agents_survey/gap_map.md",
    ),
    (
        "agentic-engineering-index",
        "knowledge/agentic_engineering/agentic_engineering_index.md",
    ),
    (
        "agentic-engineering-reference",
        "knowledge/agentic_engineering/kenn_reference_architecture.md",
    ),
    (
        "agentic-engineering-tool-stack",
        "knowledge/agentic_engineering/tool_stack_investigation.md",
    ),
    (
        "agentic-engineering-source-registry",
        "knowledge/agentic_engineering/source_registry.md",
    ),
    (
        "agentic-engineering-claim-ledger",
        "knowledge/agentic_engineering/claim_evidence_ledger.md",
    ),
    (
        "agentic-engineering-missing-evidence",
        "knowledge/agentic_engineering/missing_evidence.md",
    ),
    (
        "crouzeix-problem-and-prior-barrier",
        "knowledge/crouzeix_conjecture/01_problem_and_prior_barrier.md",
    ),
    (
        "crouzeix-shared-power-family",
        "knowledge/crouzeix_conjecture/02_shared_power_family.md",
    ),
    (
        "crouzeix-jin-proof-spine",
        "knowledge/crouzeix_conjecture/03_jin_proof_spine.md",
    ),
    (
        "crouzeix-jin-positive-real-completion",
        "knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md",
    ),
    (
        "crouzeix-lorist-schwenninger-proof",
        "knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof.md",
    ),
    (
        "crouzeix-proof-comparison",
        "knowledge/crouzeix_conjecture/06_proof_comparison.md",
    ),
    (
        "crouzeix-jin-lean-verification",
        "knowledge/crouzeix_conjecture/07_jin_lean_verification.md",
    ),
    (
        "crouzeix-ai-assisted-discovery",
        "knowledge/crouzeix_conjecture/08_ai_assisted_discovery.md",
    ),
    (
        "crouzeix-status-and-critical-assessment",
        "knowledge/crouzeix_conjecture/09_status_and_critical_assessment.md",
    ),
    (
        "crouzeix-proof-reproduction-research",
        "knowledge/crouzeix_conjecture/reproduction_research.md",
    ),
    (
        "crouzeix-glossary",
        "knowledge/crouzeix_conjecture/glossary.md",
    ),
    (
        "crouzeix-source-registry",
        "knowledge/crouzeix_conjecture/source_registry.md",
    ),
    (
        "crouzeix-claim-evidence-ledger",
        "knowledge/crouzeix_conjecture/claim_evidence_ledger.md",
    ),
];
pub(super) const REQUIRED_CHAPTERS: [(&str, &str); 9] = [
    (
        "recursive-improvement-loop",
        "knowledge/rsi/chapters/recursive-improvement-loop.md",
    ),
    (
        "foundation-model-inside-the-loop",
        "knowledge/rsi/chapters/foundation-model-inside-the-loop.md",
    ),
    (
        "harness-engineering",
        "knowledge/rsi/chapters/harness-engineering.md",
    ),
    (
        "durable-improvement-workflows",
        "knowledge/rsi/chapters/durable-improvement-workflows.md",
    ),
    (
        "procedure-internalization",
        "knowledge/rsi/chapters/procedure-internalization.md",
    ),
    ("harness-search", "knowledge/rsi/chapters/harness-search.md"),
    (
        "automated-research",
        "knowledge/rsi/chapters/automated-research.md",
    ),
    (
        "joint-harness-weight-adaptation",
        "knowledge/rsi/chapters/joint-harness-weight-adaptation.md",
    ),
    (
        "evaluation-promotion-containment",
        "knowledge/rsi/chapters/evaluation-promotion-containment.md",
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum CoverageDepth {
    Chapter,
    SupportingPage,
    SystemReading,
    WorkedExample,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct CoverageEntry {
    pub(super) concept_id: String,
    pub(super) coverage_depth: CoverageDepth,
    pub(super) canonical_markdown_path: String,
    pub(super) section_id: Option<String>,
    pub(super) parent_concept_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct RetainedConcept {
    pub(super) concept_id: String,
    pub(super) source_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct CanonicalDocument {
    pub(super) concept_id: String,
    pub(super) title: String,
    pub(super) canonical_markdown_path: String,
    pub(super) markdown_sha256: String,
    pub(super) html_sha256: String,
    pub(super) html: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ReaderRoute {
    pub(super) route_id: String,
    pub(super) label: String,
    pub(super) canonical_markdown_path: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct WengSectionProjection {
    pub(super) order: u8,
    pub(super) section_id: String,
    pub(super) title: String,
    pub(super) public_url: String,
    pub(super) captured_locator: String,
    pub(super) companion_document_id: String,
    pub(super) companion_section: String,
    pub(super) system_ids: Vec<String>,
    pub(super) exercise_ids: Vec<String>,
    pub(super) figure_locators: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum SystemTreatment {
    Full,
    Card,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum PublicationState {
    Planned,
    Published,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct SystemProjection {
    pub(super) system_id: String,
    pub(super) title: String,
    pub(super) source_ids: Vec<String>,
    pub(super) treatment: SystemTreatment,
    pub(super) publication_state: PublicationState,
    pub(super) canonical_markdown_path: String,
    pub(super) weng_section_ids: Vec<String>,
    pub(super) paper_routes: Vec<PaperRouteProjection>,
    pub(super) diagnostic_case_path: Option<String>,
    pub(super) related_system_ids: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct PaperRouteProjection {
    pub(super) source_id: String,
    pub(super) public_url: String,
    pub(super) captured_locator: String,
    pub(super) reading_sequence: ReadingSequenceProjection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct ReadingSequenceProjection {
    pub(super) method: String,
    pub(super) algorithm: String,
    pub(super) main_evaluation: String,
    pub(super) ablation: String,
    pub(super) limitations: String,
    pub(super) appendix: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct RsiCorpus {
    pub(super) schema_version: &'static str,
    pub(super) retained_concepts: Vec<RetainedConcept>,
    pub(super) coverage: Vec<CoverageEntry>,
    pub(super) reader_routes: Vec<ReaderRoute>,
    pub(super) documents: Vec<CanonicalDocument>,
    pub(super) systems: Vec<SystemProjection>,
    pub(super) weng_sections: Vec<WengSectionProjection>,
    diagnostics: rules::DiagnosticProjection,
    lessons: Vec<lessons::Lesson>,
}

pub(super) fn compile(repo_root: &Path) -> Result<RsiCorpus, AppError> {
    let repository = HeldDirectory::open(repo_root, "RSI repository")?;
    let ValidatedRsiInputs {
        retained_concepts,
        coverage,
        registered_sources: _registered_sources,
        canonical_sources,
        weng_map,
        system_registry,
    } = contracts::load(&repository)?;
    let diagnostics = rules::load(&repository, &_registered_sources, system_registry.as_ref())?;
    let validated_lessons = lessons::load(
        &repository,
        &retained_concepts,
        system_registry.as_ref(),
        &diagnostics,
    )?;
    let lesson_manifest = validated_lessons.manifest;
    let mut all_sources = canonical_sources;
    all_sources.extend(validated_lessons.sources);
    let route_targets = render::route_targets(&coverage, &all_sources);
    let mut documents = all_sources
        .values()
        .map(|source| {
            render::compile_document(source, &route_targets)
                .map(|document| (source.path.clone(), document))
        })
        .collect::<Result<BTreeMap<_, _>, AppError>>()?;
    for lesson in &lesson_manifest.lessons {
        let document = documents
            .get_mut(&lesson.canonical_markdown_path)
            .ok_or_else(|| {
                invalid(
                    "knowledge.rsi.lesson_document",
                    format!("compiled lesson {} is missing", lesson.lesson_id),
                )
            })?;
        document.concept_id = format!("lesson-{}", lesson.lesson_id);
    }

    let reader_routes = READER_ROUTES
        .iter()
        .map(|(route_id, label, path)| {
            let document = documents.get_mut(*path).ok_or_else(|| {
                invalid(
                    "knowledge.rsi.canonical_missing",
                    format!("reader route source is missing: {path}"),
                )
            })?;
            if document.concept_id.is_empty() {
                document.concept_id = (*route_id).to_owned();
            }
            Ok(ReaderRoute {
                route_id: (*route_id).to_owned(),
                label: (*label).to_owned(),
                canonical_markdown_path: (*path).to_owned(),
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    let systems = system_registry
        .map(|registry| {
            registry
                .systems
                .into_iter()
                .map(|system| SystemProjection {
                    system_id: system.system_id,
                    title: system.title,
                    source_ids: system.source_ids,
                    treatment: system.treatment,
                    publication_state: system.publication_state,
                    canonical_markdown_path: system.canonical_markdown_path,
                    weng_section_ids: system.weng_section_ids,
                    paper_routes: system
                        .paper_routes
                        .into_iter()
                        .map(|route| PaperRouteProjection {
                            source_id: route.source_id,
                            public_url: route.public_url,
                            captured_locator: route.captured_locator,
                            reading_sequence: ReadingSequenceProjection {
                                method: route.reading_sequence.method,
                                algorithm: route.reading_sequence.algorithm,
                                main_evaluation: route.reading_sequence.main_evaluation,
                                ablation: route.reading_sequence.ablation,
                                limitations: route.reading_sequence.limitations,
                                appendix: route.reading_sequence.appendix,
                            },
                        })
                        .collect(),
                    diagnostic_case_path: system.diagnostic_case_path,
                    related_system_ids: system.related_system_ids,
                })
                .collect()
        })
        .unwrap_or_default();
    let weng_sections = weng_map
        .map(|map| {
            map.sections
                .into_iter()
                .map(|section| {
                    let companion_document_id = documents
                        .get(&section.companion_path)
                        .map(|document| document.concept_id.clone())
                        .ok_or_else(|| {
                            invalid(
                                "knowledge.rsi.weng_companion",
                                format!(
                                    "compiled Weng companion is missing: {}",
                                    section.companion_path
                                ),
                            )
                        })?;
                    Ok(WengSectionProjection {
                        order: section.order,
                        section_id: section.section_id,
                        title: section.title,
                        public_url: section.public_url,
                        captured_locator: section.captured_locator,
                        companion_document_id,
                        companion_section: section.companion_section,
                        system_ids: section.system_ids,
                        exercise_ids: section.exercise_ids,
                        figure_locators: section.figure_locators,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(RsiCorpus {
        schema_version: "rsi-technical-atlas/v5",
        retained_concepts,
        coverage,
        reader_routes,
        documents: documents.into_values().collect(),
        systems,
        weng_sections,
        diagnostics,
        lessons: lesson_manifest.lessons,
    })
}

pub(super) fn validate_build_output(output: &Path) -> Result<PathBuf, AppError> {
    if output.is_absolute()
        || !output.starts_with("atlas/src/content/generated")
        || output.extension().and_then(|value| value.to_str()) != Some("json")
        || !output
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(invalid(
            "corpus.output_path",
            "output must stay beneath atlas/src/content/generated",
        ));
    }
    Ok(output.to_path_buf())
}

fn optional_cell(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn invalid(code: &'static str, message: impl Into<String>) -> AppError {
    AppError::invalid_input(code, message)
}

#[cfg(test)]
mod tests;
