use super::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(super) const WORKSHEET_FIELDS_PATH: &str = "content/diagnostics/worksheet-fields.json";
pub(super) const DIAGNOSTIC_RULES_PATH: &str = "content/diagnostics/rules.json";
pub(super) const EXPORT_SCHEMA_PATH: &str = "content/diagnostics/export-schema.json";
pub(super) const SELF_REFINE_CASE_PATH: &str = "content/diagnostics/cases/self-refine.json";

const MAX_DIAGNOSTIC_BYTES: usize = 1024 * 1024;
const MAX_FIELDS: usize = 128;
const MAX_RULES: usize = 256;
const MAX_CASES: usize = 64;
const MAX_PREDICATES_PER_RULE: usize = 64;
const MAX_PREDICATE_DEPTH: usize = 8;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorksheetManifest {
    pub(super) schema_version: u8,
    pub(super) fields: Vec<WorksheetField>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorksheetField {
    pub(super) field_id: String,
    pub(super) section: WorksheetSection,
    pub(super) label: String,
    pub(super) kind: FieldKind,
    pub(super) required: bool,
    #[serde(default)]
    pub(super) values: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum WorksheetSection {
    Candidate,
    Persistence,
    Generation,
    Evaluation,
    ResourcesAuthority,
    Evidence,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum FieldKind {
    Boolean,
    Enum,
    Set,
    Text,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuleManifest {
    pub(super) schema_version: u8,
    pub(super) classifications: Vec<PrimaryClassification>,
    pub(super) rules: Vec<DiagnosticRule>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiagnosticRule {
    pub(super) rule_id: String,
    pub(super) version: u16,
    pub(super) precedence: u16,
    pub(super) predicate: Predicate,
    pub(super) effect: RuleEffect,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Predicate {
    BoolEquals { field_id: String, value: bool },
    EnumEquals { field_id: String, value: String },
    SetContains { field_id: String, value: String },
    All { predicates: Vec<Predicate> },
    Any { predicates: Vec<Predicate> },
    Not { predicate: Box<Predicate> },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum RuleEffect {
    Classification {
        classification: PrimaryClassification,
    },
    ClaimCeiling {
        classification: PrimaryClassification,
    },
    Flag {
        flag: DiagnosticFlag,
    },
    Finding {
        severity: FindingSeverity,
        explanation: String,
        canonical_concept_id: String,
        evidence_needed: String,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum PrimaryClassification {
    OutputRefinement,
    PersistentAdaptation,
    HarnessImprovement,
    AutomatedAiResearch,
    JointHarnessWeightAdaptation,
    SuccessorImprovement,
    RecursiveImprovementDemonstrated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum DiagnosticFlag {
    PersistenceEstablished,
    AcceptedGenerationEstablished,
    MatchedEnvelopeEstablished,
    EvaluatorIndependenceEstablished,
    CompleteRootTreeAccountingEstablished,
    NextCycleGainMeasured,
    IndependentReproductionPresent,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum FindingSeverity {
    Blocking,
    Warning,
    Information,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiagnosticCase {
    pub(super) schema_version: u8,
    pub(super) case_id: String,
    pub(super) system_id: Option<String>,
    pub(super) title: String,
    pub(super) method_family: String,
    pub(super) expected_classification: PrimaryClassification,
    pub(super) expected_claim_ceiling: PrimaryClassification,
    pub(super) facts: Vec<CaseFact>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseFact {
    pub(super) field_id: String,
    pub(super) value: Value,
    pub(super) provenance: FactProvenance,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum FactProvenance {
    SourceBacked { source_id: String, locator: String },
    ReaderAssertion,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExportSchema {
    pub(super) schema_version: u8,
    pub(super) export_id: String,
    pub(super) required_sections: Vec<String>,
    pub(super) max_import_bytes: usize,
    pub(super) max_commentary_bytes: usize,
    pub(super) digest_algorithm: String,
    pub(super) commentary_affects_diagnosis: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct DiagnosticProjection {
    pub(super) worksheet: WorksheetManifest,
    pub(super) rules: RuleManifest,
    pub(super) cases: Vec<DiagnosticCase>,
    pub(super) export_schema: ExportSchema,
}

pub(super) fn load(
    repository: &HeldDirectory,
    registered_sources: &BTreeSet<String>,
    systems: Option<&contracts::SystemRegistry>,
) -> Result<DiagnosticProjection, AppError> {
    let worksheet: WorksheetManifest =
        read_json(repository, WORKSHEET_FIELDS_PATH, "worksheet fields")?;
    validate_worksheet(&worksheet)?;
    let rules: RuleManifest = read_json(repository, DIAGNOSTIC_RULES_PATH, "diagnostic rules")?;
    validate_rules(&rules, &worksheet)?;
    let export_schema: ExportSchema =
        read_json(repository, EXPORT_SCHEMA_PATH, "diagnostic export schema")?;
    validate_export_schema(&export_schema)?;

    let mut paths = vec![(SELF_REFINE_CASE_PATH.to_owned(), None)];
    if let Some(registry) = systems {
        for system in &registry.systems {
            if let Some(path) = &system.diagnostic_case_path {
                paths.push((path.clone(), Some(system.system_id.as_str())));
            }
        }
    }
    if paths.len() > MAX_CASES {
        return Err(invalid(
            "knowledge.rsi.diagnostic_case_limit",
            format!("diagnostic case count exceeds {MAX_CASES}"),
        ));
    }
    let mut path_ids = BTreeSet::<String>::new();
    let mut case_ids = BTreeSet::<String>::new();
    let mut cases = Vec::new();
    for (path, expected_system_id) in paths {
        if !path_ids.insert(path.clone()) {
            return Err(invalid(
                "knowledge.rsi.diagnostic_case_path",
                format!("diagnostic case path is referenced more than once: {path}"),
            ));
        }
        let case: DiagnosticCase = read_json(repository, &path, "diagnostic case")?;
        validate_case(
            &case,
            expected_system_id,
            &worksheet,
            &rules,
            registered_sources,
        )?;
        if !case_ids.insert(case.case_id.clone()) {
            return Err(invalid(
                "knowledge.rsi.diagnostic_case_id",
                format!("duplicate diagnostic case ID {}", case.case_id),
            ));
        }
        cases.push(case);
    }
    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    Ok(DiagnosticProjection {
        worksheet,
        rules,
        cases,
        export_schema,
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(
    repository: &HeldDirectory,
    path: &str,
    label: &str,
) -> Result<T, AppError> {
    let bytes = repository
        .read_optional_regular_file_bounded(
            Path::new(path),
            "RSI diagnostic contract",
            MAX_DIAGNOSTIC_BYTES,
        )?
        .ok_or_else(|| {
            invalid(
                "knowledge.rsi.diagnostic_missing",
                format!("{label} is missing: {path}"),
            )
        })?;
    let value = crate::json::parse_strict_bounded(
        &bytes,
        MAX_DIAGNOSTIC_BYTES,
        "knowledge.rsi.diagnostic_size",
        "knowledge.rsi.diagnostic_json",
        label,
    )?;
    serde_json::from_value(value).map_err(|_| {
        invalid(
            "knowledge.rsi.diagnostic_schema",
            format!("{label} does not match the strict schema: {path}"),
        )
    })
}

fn validate_worksheet(manifest: &WorksheetManifest) -> Result<(), AppError> {
    if manifest.schema_version != 1
        || manifest.fields.is_empty()
        || manifest.fields.len() > MAX_FIELDS
    {
        return Err(invalid(
            "knowledge.rsi.worksheet",
            "worksheet must use schema version 1 within the field limit",
        ));
    }
    let mut ids = BTreeSet::new();
    for field in &manifest.fields {
        if !valid_id(&field.field_id)
            || !ids.insert(field.field_id.as_str())
            || field.label.trim().is_empty()
            || field.label.trim() != field.label
        {
            return Err(invalid(
                "knowledge.rsi.worksheet_field",
                format!("invalid or duplicate worksheet field {}", field.field_id),
            ));
        }
        let needs_values = matches!(field.kind, FieldKind::Enum | FieldKind::Set);
        if needs_values == field.values.is_empty() {
            return Err(invalid(
                "knowledge.rsi.worksheet_field",
                format!("worksheet field {} has invalid values", field.field_id),
            ));
        }
        require_unique_values(
            &field.values,
            "knowledge.rsi.worksheet_value",
            &field.field_id,
        )?;
    }
    Ok(())
}

fn validate_rules(manifest: &RuleManifest, worksheet: &WorksheetManifest) -> Result<(), AppError> {
    if manifest.schema_version != 1
        || manifest.rules.is_empty()
        || manifest.rules.len() > MAX_RULES
        || manifest.classifications.len() != 7
    {
        return Err(invalid(
            "knowledge.rsi.diagnostic_rules",
            "diagnostic rules have an invalid version or collection size",
        ));
    }
    let legal_classifications = manifest
        .classifications
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if legal_classifications.len() != 7 {
        return Err(invalid(
            "knowledge.rsi.diagnostic_rules",
            "diagnostic classifications must be unique",
        ));
    }
    let fields = worksheet
        .fields
        .iter()
        .map(|field| (field.field_id.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut classification_precedence = BTreeMap::new();
    let mut ceiling_precedence = BTreeMap::new();
    for rule in &manifest.rules {
        if !valid_rule_id(&rule.rule_id) || !ids.insert(rule.rule_id.as_str()) || rule.version == 0
        {
            return Err(invalid(
                "knowledge.rsi.diagnostic_rule",
                format!("invalid or duplicate diagnostic rule {}", rule.rule_id),
            ));
        }
        let mut predicate_count = 0;
        validate_predicate(&rule.predicate, &fields, 1, &mut predicate_count)?;
        if predicate_count > MAX_PREDICATES_PER_RULE {
            return Err(invalid(
                "knowledge.rsi.diagnostic_predicate_limit",
                format!("rule {} exceeds the predicate limit", rule.rule_id),
            ));
        }
        match &rule.effect {
            RuleEffect::Classification { classification } => {
                if !legal_classifications.contains(classification) {
                    return Err(invalid(
                        "knowledge.rsi.diagnostic_classification",
                        format!("rule {} has an unknown classification", rule.rule_id),
                    ));
                }
                if classification_precedence
                    .insert(rule.precedence, *classification)
                    .is_some()
                {
                    return Err(invalid(
                        "knowledge.rsi.diagnostic_precedence",
                        format!(
                            "classification precedence {} is assigned more than once",
                            rule.precedence
                        ),
                    ));
                }
            }
            RuleEffect::ClaimCeiling { classification } => {
                if !legal_classifications.contains(classification) {
                    return Err(invalid(
                        "knowledge.rsi.diagnostic_classification",
                        format!("rule {} has an unknown claim ceiling", rule.rule_id),
                    ));
                }
                if ceiling_precedence
                    .insert(rule.precedence, *classification)
                    .is_some()
                {
                    return Err(invalid(
                        "knowledge.rsi.diagnostic_precedence",
                        format!(
                            "claim-ceiling precedence {} is assigned more than once",
                            rule.precedence
                        ),
                    ));
                }
            }
            RuleEffect::Finding {
                explanation,
                canonical_concept_id,
                evidence_needed,
                ..
            } => {
                if explanation.trim().is_empty()
                    || !valid_id(canonical_concept_id)
                    || evidence_needed.trim().is_empty()
                {
                    return Err(invalid(
                        "knowledge.rsi.diagnostic_finding",
                        format!("rule {} has an invalid finding", rule.rule_id),
                    ));
                }
            }
            RuleEffect::Flag { .. } => {}
        }
    }
    Ok(())
}

fn valid_rule_id(value: &str) -> bool {
    let mut segments = value.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    valid_id(first) && segments.all(valid_id)
}

fn validate_predicate(
    predicate: &Predicate,
    fields: &BTreeMap<&str, &WorksheetField>,
    depth: usize,
    count: &mut usize,
) -> Result<(), AppError> {
    *count += 1;
    if depth > MAX_PREDICATE_DEPTH {
        return Err(invalid(
            "knowledge.rsi.diagnostic_predicate_depth",
            format!("predicate depth exceeds {MAX_PREDICATE_DEPTH}"),
        ));
    }
    match predicate {
        Predicate::BoolEquals { field_id, .. } => {
            require_field_kind(fields, field_id, FieldKind::Boolean).map(|_| ())
        }
        Predicate::EnumEquals { field_id, value } => {
            let field = require_field_kind(fields, field_id, FieldKind::Enum)?;
            require_field_value(field, value)
        }
        Predicate::SetContains { field_id, value } => {
            let field = require_field_kind(fields, field_id, FieldKind::Set)?;
            require_field_value(field, value)
        }
        Predicate::All { predicates } | Predicate::Any { predicates } => {
            if predicates.is_empty() {
                return Err(invalid(
                    "knowledge.rsi.diagnostic_predicate",
                    "all/any predicates must not be empty",
                ));
            }
            for child in predicates {
                validate_predicate(child, fields, depth + 1, count)?;
            }
            Ok(())
        }
        Predicate::Not { predicate } => validate_predicate(predicate, fields, depth + 1, count),
    }
}

fn require_field_kind<'a>(
    fields: &'a BTreeMap<&str, &WorksheetField>,
    field_id: &str,
    kind: FieldKind,
) -> Result<&'a WorksheetField, AppError> {
    let field = fields.get(field_id).copied().ok_or_else(|| {
        invalid(
            "knowledge.rsi.diagnostic_rule_field",
            format!("diagnostic rule references unknown field {field_id}"),
        )
    })?;
    if field.kind != kind {
        return Err(invalid(
            "knowledge.rsi.diagnostic_rule_field",
            format!("diagnostic rule uses field {field_id} with the wrong predicate"),
        ));
    }
    Ok(field)
}

fn require_field_value(field: &WorksheetField, value: &str) -> Result<(), AppError> {
    if !field.values.iter().any(|candidate| candidate == value) {
        return Err(invalid(
            "knowledge.rsi.diagnostic_rule_value",
            format!(
                "diagnostic rule uses unknown value {value} for {}",
                field.field_id
            ),
        ));
    }
    Ok(())
}

fn validate_export_schema(schema: &ExportSchema) -> Result<(), AppError> {
    if schema.schema_version != 1
        || schema.export_id != "rsi-diagnosis/v2"
        || schema.digest_algorithm != "sha256"
        || schema.commentary_affects_diagnosis
        || schema.max_import_bytes == 0
        || schema.max_import_bytes > MAX_DIAGNOSTIC_BYTES
        || schema.max_commentary_bytes == 0
        || schema.max_commentary_bytes > schema.max_import_bytes
    {
        return Err(invalid(
            "knowledge.rsi.diagnostic_export",
            "diagnostic export schema violates the v1 safety contract",
        ));
    }
    require_unique_values(
        &schema.required_sections,
        "knowledge.rsi.diagnostic_export",
        "required export sections",
    )
}

fn require_unique_values(
    values: &[String],
    code: &'static str,
    owner: &str,
) -> Result<(), AppError> {
    let mut unique = BTreeSet::new();
    if values
        .iter()
        .any(|value| !valid_id(value) || !unique.insert(value.as_str()))
    {
        return Err(invalid(
            code,
            format!("{owner} contains invalid or duplicate IDs"),
        ));
    }
    Ok(())
}

fn validate_case(
    case: &DiagnosticCase,
    expected_system_id: Option<&str>,
    worksheet: &WorksheetManifest,
    rules: &RuleManifest,
    registered_sources: &BTreeSet<String>,
) -> Result<(), AppError> {
    if case.schema_version != 1
        || !valid_id(&case.case_id)
        || case.title.trim().is_empty()
        || case.method_family.trim().is_empty()
        || case.system_id.as_deref() != expected_system_id
    {
        return Err(invalid(
            "knowledge.rsi.diagnostic_case",
            format!("diagnostic case {} has an invalid identity", case.case_id),
        ));
    }
    let fields = worksheet
        .fields
        .iter()
        .map(|field| (field.field_id.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    let mut facts = BTreeMap::new();
    for fact in &case.facts {
        let field = fields.get(fact.field_id.as_str()).copied().ok_or_else(|| {
            invalid(
                "knowledge.rsi.diagnostic_case_field",
                format!("case {} has unknown field {}", case.case_id, fact.field_id),
            )
        })?;
        if facts.insert(fact.field_id.as_str(), &fact.value).is_some() {
            return Err(invalid(
                "knowledge.rsi.diagnostic_case_field",
                format!("case {} duplicates field {}", case.case_id, fact.field_id),
            ));
        }
        validate_fact_value(field, &fact.value)?;
        match &fact.provenance {
            FactProvenance::SourceBacked { source_id, locator } => {
                if !registered_sources.contains(source_id) || locator.trim().is_empty() {
                    return Err(invalid(
                        "knowledge.rsi.diagnostic_case_source",
                        format!("case {} has invalid source provenance", case.case_id),
                    ));
                }
            }
            FactProvenance::ReaderAssertion => {
                return Err(invalid(
                    "knowledge.rsi.diagnostic_case_source",
                    format!("built-in case {} must be source-backed", case.case_id),
                ));
            }
        }
    }
    let missing = worksheet
        .fields
        .iter()
        .filter(|field| field.required && !facts.contains_key(field.field_id.as_str()))
        .map(|field| field.field_id.as_str())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(invalid(
            "knowledge.rsi.diagnostic_case_required",
            format!(
                "case {} is missing required facts {missing:?}",
                case.case_id
            ),
        ));
    }
    let actual = classify(rules, &facts).ok_or_else(|| {
        invalid(
            "knowledge.rsi.diagnostic_case_classification",
            format!("case {} satisfies no classification rule", case.case_id),
        )
    })?;
    if actual != case.expected_classification {
        return Err(invalid(
            "knowledge.rsi.diagnostic_case_classification",
            format!(
                "case {} expected {:?} but rules produce {:?}",
                case.case_id, case.expected_classification, actual
            ),
        ));
    }
    let actual_ceiling = claim_ceiling(rules, &facts).ok_or_else(|| {
        invalid(
            "knowledge.rsi.diagnostic_case_claim_ceiling",
            format!("case {} satisfies no claim-ceiling rule", case.case_id),
        )
    })?;
    if actual_ceiling != case.expected_claim_ceiling {
        return Err(invalid(
            "knowledge.rsi.diagnostic_case_claim_ceiling",
            format!(
                "case {} expected ceiling {:?} but rules produce {:?}",
                case.case_id, case.expected_claim_ceiling, actual_ceiling
            ),
        ));
    }
    Ok(())
}

fn validate_fact_value(field: &WorksheetField, value: &Value) -> Result<(), AppError> {
    let valid = match field.kind {
        FieldKind::Boolean => value.is_boolean(),
        FieldKind::Text => value.as_str().is_some_and(|text| !text.trim().is_empty()),
        FieldKind::Enum => value
            .as_str()
            .is_some_and(|text| field.values.iter().any(|candidate| candidate == text)),
        FieldKind::Set => value.as_array().is_some_and(|values| {
            let mut unique = BTreeSet::new();
            values.iter().all(|value| {
                value.as_str().is_some_and(|text| {
                    field.values.iter().any(|candidate| candidate == text) && unique.insert(text)
                })
            })
        }),
    };
    if !valid {
        return Err(invalid(
            "knowledge.rsi.diagnostic_case_value",
            format!("case fact {} has an invalid value", field.field_id),
        ));
    }
    Ok(())
}

fn classify(
    manifest: &RuleManifest,
    facts: &BTreeMap<&str, &Value>,
) -> Option<PrimaryClassification> {
    let mut rules = manifest
        .rules
        .iter()
        .filter_map(|rule| match rule.effect {
            RuleEffect::Classification { classification } => {
                Some((rule.precedence, classification, &rule.predicate))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    rules.sort_by(|left, right| right.0.cmp(&left.0));
    rules
        .into_iter()
        .find_map(|(_, classification, predicate)| {
            evaluate(predicate, facts).then_some(classification)
        })
}

fn claim_ceiling(
    manifest: &RuleManifest,
    facts: &BTreeMap<&str, &Value>,
) -> Option<PrimaryClassification> {
    let mut rules = manifest
        .rules
        .iter()
        .filter_map(|rule| match rule.effect {
            RuleEffect::ClaimCeiling { classification } => {
                Some((rule.precedence, classification, &rule.predicate))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    rules.sort_by(|left, right| right.0.cmp(&left.0));
    rules
        .into_iter()
        .find_map(|(_, classification, predicate)| {
            evaluate(predicate, facts).then_some(classification)
        })
}

fn evaluate(predicate: &Predicate, facts: &BTreeMap<&str, &Value>) -> bool {
    match predicate {
        Predicate::BoolEquals { field_id, value } => {
            facts
                .get(field_id.as_str())
                .and_then(|actual| actual.as_bool())
                == Some(*value)
        }
        Predicate::EnumEquals { field_id, value } => {
            facts
                .get(field_id.as_str())
                .and_then(|actual| actual.as_str())
                == Some(value.as_str())
        }
        Predicate::SetContains { field_id, value } => facts
            .get(field_id.as_str())
            .and_then(|actual| actual.as_array())
            .is_some_and(|values| values.iter().any(|actual| actual.as_str() == Some(value))),
        Predicate::All { predicates } => predicates
            .iter()
            .all(|predicate| evaluate(predicate, facts)),
        Predicate::Any { predicates } => predicates
            .iter()
            .any(|predicate| evaluate(predicate, facts)),
        Predicate::Not { predicate } => !evaluate(predicate, facts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn repository_json(path: &str) -> Value {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("Harp workspace root");
        serde_json::from_slice(&fs::read(workspace.join(path)).unwrap()).unwrap()
    }

    fn worksheet() -> WorksheetManifest {
        serde_json::from_value(repository_json(WORKSHEET_FIELDS_PATH)).unwrap()
    }

    #[test]
    fn strict_schema_rejects_unknown_fields_and_predicates() {
        let mut fields = repository_json(WORKSHEET_FIELDS_PATH);
        fields["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<WorksheetManifest>(fields).is_err());

        let mut rules = repository_json(DIAGNOSTIC_RULES_PATH);
        rules["rules"][0]["predicate"]["kind"] = serde_json::json!("unsupported");
        assert!(serde_json::from_value::<RuleManifest>(rules).is_err());
    }

    #[test]
    fn validation_rejects_unknown_rule_fields() {
        let mut value = repository_json(DIAGNOSTIC_RULES_PATH);
        value["rules"][0]["predicate"]["field_id"] = serde_json::json!("absent-field");
        let rules: RuleManifest = serde_json::from_value(value).unwrap();

        assert_eq!(
            validate_rules(&rules, &worksheet()).unwrap_err().code(),
            "knowledge.rsi.diagnostic_rule_field"
        );
    }

    #[test]
    fn validation_rejects_equal_classification_precedence() {
        let mut value = repository_json(DIAGNOSTIC_RULES_PATH);
        let classification_indices = value["rules"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .filter_map(|(index, rule)| {
                (rule["effect"]["kind"] == "classification").then_some(index)
            })
            .take(2)
            .collect::<Vec<_>>();
        let precedence = value["rules"][classification_indices[0]]["precedence"].clone();
        value["rules"][classification_indices[1]]["precedence"] = precedence;
        let rules: RuleManifest = serde_json::from_value(value).unwrap();

        assert_eq!(
            validate_rules(&rules, &worksheet()).unwrap_err().code(),
            "knowledge.rsi.diagnostic_precedence"
        );
    }
}
