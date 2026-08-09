use super::*;
use serde::{Deserialize, Serialize};
use std::path::Component;

pub(super) const LESSONS_PATH: &str = "content/lessons.json";
const MAX_LESSON_MANIFEST_BYTES: usize = 256 * 1024;
const MAX_LESSONS: usize = 32;
const REQUIRED_HEADINGS: [&str; 6] = [
    "observe",
    "predict",
    "compare",
    "explain",
    "missing-fact",
    "transfer",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LessonManifest {
    pub(super) schema_version: u8,
    pub(super) lessons: Vec<Lesson>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Lesson {
    pub(super) order: u8,
    pub(super) lesson_id: String,
    pub(super) title: String,
    pub(super) canonical_markdown_path: String,
    pub(super) case_ids: Vec<String>,
    pub(super) system_ids: Vec<String>,
    pub(super) prompt_ids: Vec<String>,
    pub(super) concept_ids: Vec<String>,
    pub(super) transfer_case_ids: Vec<String>,
}

pub(super) struct ValidatedLessons {
    pub(super) manifest: LessonManifest,
    pub(super) sources: BTreeMap<String, contracts::ValidatedCanonicalSource>,
}

pub(super) fn load(
    repository: &HeldDirectory,
    retained_concepts: &[RetainedConcept],
    system_registry: Option<&contracts::SystemRegistry>,
    diagnostics: &rules::DiagnosticProjection,
) -> Result<ValidatedLessons, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(LESSONS_PATH),
            "RSI lesson manifest",
            MAX_LESSON_MANIFEST_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.lesson_missing",
                format!("{LESSONS_PATH} is missing"),
            )
        })?;
    let value = crate::json::parse_strict_bounded(
        &bytes,
        MAX_LESSON_MANIFEST_BYTES,
        "knowledge.rsi.lesson_size",
        "knowledge.rsi.lesson_json",
        "RSI lesson manifest",
    )?;
    let manifest: LessonManifest = serde_json::from_value(value).map_err(|_| {
        invalid(
            "knowledge.rsi.lesson_schema",
            "RSI lesson manifest does not match the strict schema",
        )
    })?;
    if manifest.schema_version != 1
        || manifest.lessons.is_empty()
        || manifest.lessons.len() > MAX_LESSONS
    {
        return Err(invalid(
            "knowledge.rsi.lesson_schema",
            "RSI lessons must use schema version 1 within the lesson limit",
        ));
    }

    let concepts = retained_concepts
        .iter()
        .map(|concept| concept.concept_id.as_str())
        .collect::<BTreeSet<_>>();
    let systems = system_registry
        .map(|registry| {
            registry
                .systems
                .iter()
                .map(|system| system.system_id.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let cases = diagnostics
        .cases
        .iter()
        .map(|case| case.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut lesson_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut sources = BTreeMap::new();

    for (index, lesson) in manifest.lessons.iter().enumerate() {
        if usize::from(lesson.order) != index + 1 {
            return Err(invalid(
                "knowledge.rsi.lesson_order",
                "lesson order must be contiguous and match array order",
            ));
        }
        if !valid_id(&lesson.lesson_id)
            || !lesson_ids.insert(lesson.lesson_id.as_str())
            || lesson.title.trim().is_empty()
            || lesson.title.trim() != lesson.title
        {
            return Err(invalid(
                "knowledge.rsi.lesson_identity",
                format!("lesson {} has an invalid identity", lesson.lesson_id),
            ));
        }
        let path = Path::new(&lesson.canonical_markdown_path);
        if !lesson
            .canonical_markdown_path
            .starts_with("knowledge/rsi/lessons/")
            || !lesson.canonical_markdown_path.ends_with(".md")
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || !paths.insert(lesson.canonical_markdown_path.as_str())
        {
            return Err(invalid(
                "knowledge.rsi.lesson_path",
                format!("lesson {} has an invalid canonical path", lesson.lesson_id),
            ));
        }
        require_ids(&lesson.case_ids, &cases, "case", &lesson.lesson_id)?;
        require_unique(&lesson.system_ids, "system", &lesson.lesson_id)?;
        if lesson
            .system_ids
            .iter()
            .any(|system_id| !systems.contains(system_id.as_str()))
        {
            return Err(invalid(
                "knowledge.rsi.lesson_reference",
                format!("lesson {} references unknown system IDs", lesson.lesson_id),
            ));
        }
        require_ids(&lesson.concept_ids, &concepts, "concept", &lesson.lesson_id)?;
        require_ids(
            &lesson.transfer_case_ids,
            &cases,
            "transfer case",
            &lesson.lesson_id,
        )?;
        require_unique(&lesson.prompt_ids, "prompt", &lesson.lesson_id)?;
        if lesson.prompt_ids.is_empty()
            || lesson
                .transfer_case_ids
                .iter()
                .any(|case_id| !lesson.case_ids.contains(case_id))
        {
            return Err(invalid(
                "knowledge.rsi.lesson_reference",
                format!(
                    "lesson {} has invalid prompts or transfer cases",
                    lesson.lesson_id
                ),
            ));
        }

        let markdown =
            contracts::read_canonical_markdown(repository, &lesson.canonical_markdown_path)?;
        let body = render::markdown_body(&markdown, &lesson.canonical_markdown_path)?;
        let headings = render::heading_ids(body);
        if REQUIRED_HEADINGS
            .iter()
            .any(|heading| !headings.contains(*heading))
        {
            return Err(invalid(
                "knowledge.rsi.lesson_markdown",
                format!("lesson {} is missing required headings", lesson.lesson_id),
            ));
        }
        contracts::validate_local_links(&lesson.canonical_markdown_path, body, repository)?;
        let body_sha256 = sha256(body.as_bytes());
        sources.insert(
            lesson.canonical_markdown_path.clone(),
            contracts::ValidatedCanonicalSource {
                path: lesson.canonical_markdown_path.clone(),
                markdown,
                body_sha256,
                entries: Vec::new(),
            },
        );
    }
    Ok(ValidatedLessons { manifest, sources })
}

fn require_unique(values: &[String], kind: &str, owner: &str) -> Result<(), AppError> {
    let mut unique = BTreeSet::new();
    if values
        .iter()
        .any(|value| !valid_id(value) || !unique.insert(value.as_str()))
    {
        return Err(invalid(
            "knowledge.rsi.lesson_reference",
            format!("lesson {owner} contains invalid or duplicate {kind} IDs"),
        ));
    }
    Ok(())
}

fn require_ids(
    values: &[String],
    known: &BTreeSet<&str>,
    kind: &str,
    owner: &str,
) -> Result<(), AppError> {
    require_unique(values, kind, owner)?;
    if values.is_empty() || values.iter().any(|value| !known.contains(value.as_str())) {
        return Err(invalid(
            "knowledge.rsi.lesson_reference",
            format!("lesson {owner} references unknown or missing {kind} IDs"),
        ));
    }
    Ok(())
}
