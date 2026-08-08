import type {
  CaseFact,
  CaseId,
  ConceptId,
  DiagnosticCase,
  DiagnosticContracts,
  DiagnosticFlag,
  DiagnosticRule,
  FactProvenance,
  FieldId,
  FindingSeverity,
  Predicate,
  PrimaryClassification,
  RuleEffect,
  RuleId,
  SourceId,
  SystemId,
  WorksheetField,
  WorksheetSection,
} from "../content/types";

const idPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const ruleIdPattern =
  /^[a-z0-9]+(?:-[a-z0-9]+)*(?:\.[a-z0-9]+(?:-[a-z0-9]+)*)+$/;
const sourcePattern = /^[A-Z0-9]+(?:-[A-Z0-9]+)*$/;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function exactObject(
  value: unknown,
  keys: readonly string[],
  label: string,
): Record<string, unknown> {
  if (!isRecord(value)) {
    throw new Error(`${label} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...keys].sort();
  if (
    actual.length !== expected.length
    || actual.some((key, index) => key !== expected[index])
  ) {
    throw new Error(`${label} has missing or unknown fields`);
  }
  return value;
}

function text(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() !== value || value === "") {
    throw new Error(`${label} must be a nonempty canonical string`);
  }
  return value;
}

function id(value: unknown, label: string): string {
  const parsed = text(value, label);
  if (!idPattern.test(parsed)) {
    throw new Error(`${label} must be a canonical ID`);
  }
  return parsed;
}

function fieldId(value: unknown, label: string): FieldId {
  const parsed = id(value, label);
  function assertBrand(candidate: string): asserts candidate is FieldId {
    if (!idPattern.test(candidate)) {
      throw new Error(`${label} must be a field ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

export function ruleId(value: unknown, label: string): RuleId {
  const parsed = text(value, label);
  if (!ruleIdPattern.test(parsed)) {
    throw new Error(`${label} must be a dotted rule ID`);
  }
  function assertBrand(candidate: string): asserts candidate is RuleId {
    if (!ruleIdPattern.test(candidate)) {
      throw new Error(`${label} must be a rule ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

export function savedRuleId(value: unknown, label: string): RuleId {
  const parsed = text(value, label);
  if (!ruleIdPattern.test(parsed) && !idPattern.test(parsed)) {
    throw new Error(`${label} must be a saved rule ID`);
  }
  function assertBrand(candidate: string): asserts candidate is RuleId {
    if (!ruleIdPattern.test(candidate) && !idPattern.test(candidate)) {
      throw new Error(`${label} must be a saved rule ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

function caseId(value: unknown, label: string): CaseId {
  const parsed = id(value, label);
  function assertBrand(candidate: string): asserts candidate is CaseId {
    if (!idPattern.test(candidate)) {
      throw new Error(`${label} must be a case ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

export function conceptId(value: unknown, label: string): ConceptId {
  const parsed = id(value, label);
  function assertBrand(candidate: string): asserts candidate is ConceptId {
    if (!idPattern.test(candidate)) {
      throw new Error(`${label} must be a concept ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

function systemId(value: unknown, label: string): SystemId {
  const parsed = id(value, label);
  function assertBrand(candidate: string): asserts candidate is SystemId {
    if (!idPattern.test(candidate)) {
      throw new Error(`${label} must be a system ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

function sourceId(value: unknown, label: string): SourceId {
  const parsed = text(value, label);
  if (!sourcePattern.test(parsed)) {
    throw new Error(`${label} must be a source ID`);
  }
  function assertBrand(candidate: string): asserts candidate is SourceId {
    if (!sourcePattern.test(candidate)) {
      throw new Error(`${label} must be a source ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

function integer(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value) || typeof value !== "number") {
    throw new Error(`${label} must be an integer`);
  }
  return value;
}

function stringArray(value: unknown, label: string): string[] {
  if (!Array.isArray(value)) {
    throw new Error(`${label} must be an array`);
  }
  const parsed = value.map((item, index) => id(item, `${label}[${index}]`));
  if (new Set(parsed).size !== parsed.length) {
    throw new Error(`${label} contains duplicates`);
  }
  return parsed;
}

function worksheetSection(value: unknown): WorksheetSection {
  if (
    value !== "candidate"
    && value !== "persistence"
    && value !== "generation"
    && value !== "evaluation"
    && value !== "resources-authority"
    && value !== "evidence"
  ) {
    throw new Error("Worksheet section is invalid");
  }
  return value;
}

function classification(value: unknown): PrimaryClassification {
  if (
    value !== "output-refinement"
    && value !== "persistent-adaptation"
    && value !== "harness-improvement"
    && value !== "automated-ai-research"
    && value !== "joint-harness-weight-adaptation"
    && value !== "successor-improvement"
    && value !== "recursive-improvement-demonstrated"
  ) {
    throw new Error("Diagnostic classification is invalid");
  }
  return value;
}

function diagnosticFlag(value: unknown): DiagnosticFlag {
  if (
    value !== "persistence-established"
    && value !== "accepted-generation-established"
    && value !== "matched-envelope-established"
    && value !== "evaluator-independence-established"
    && value !== "complete-root-tree-accounting-established"
    && value !== "next-cycle-gain-measured"
    && value !== "independent-reproduction-present"
  ) {
    throw new Error("Diagnostic flag is invalid");
  }
  return value;
}

function findingSeverity(value: unknown): FindingSeverity {
  if (value !== "blocking" && value !== "warning" && value !== "information") {
    throw new Error("Finding severity is invalid");
  }
  return value;
}

function parseWorksheetField(value: unknown, index: number): WorksheetField {
  const field = exactObject(
    value,
    ["field_id", "section", "label", "kind", "required", "values"],
    `Worksheet field ${index}`,
  );
  if (typeof field.required !== "boolean") {
    throw new Error(`Worksheet field ${index} required must be boolean`);
  }
  const values = stringArray(field.values, `Worksheet field ${index} values`);
  const base = {
    field_id: fieldId(field.field_id, `Worksheet field ${index} ID`),
    section: worksheetSection(field.section),
    label: text(field.label, `Worksheet field ${index} label`),
    required: field.required,
  };
  if (field.kind === "boolean" || field.kind === "text") {
    if (values.length !== 0) {
      throw new Error(`Worksheet field ${index} must not define values`);
    }
    return { ...base, kind: field.kind, values: [] };
  }
  if (field.kind === "enum" || field.kind === "set") {
    if (values.length === 0) {
      throw new Error(`Worksheet field ${index} must define values`);
    }
    return { ...base, kind: field.kind, values };
  }
  throw new Error(`Worksheet field ${index} has an invalid kind`);
}

function parsePredicate(value: unknown, depth = 1): Predicate {
  if (depth > 8 || !isRecord(value)) {
    throw new Error("Diagnostic predicate is invalid or too deep");
  }
  switch (value.kind) {
    case "bool-equals": {
      const predicate = exactObject(
        value,
        ["kind", "field_id", "value"],
        "Boolean predicate",
      );
      if (typeof predicate.value !== "boolean") {
        throw new Error("Boolean predicate value must be boolean");
      }
      return {
        kind: "bool-equals",
        field_id: fieldId(predicate.field_id, "Boolean predicate field"),
        value: predicate.value,
      };
    }
    case "enum-equals":
    case "set-contains": {
      const predicate = exactObject(
        value,
        ["kind", "field_id", "value"],
        "Value predicate",
      );
      return {
        kind: value.kind,
        field_id: fieldId(predicate.field_id, "Value predicate field"),
        value: id(predicate.value, "Value predicate value"),
      };
    }
    case "all":
    case "any": {
      const predicate = exactObject(
        value,
        ["kind", "predicates"],
        "Composite predicate",
      );
      if (!Array.isArray(predicate.predicates) || predicate.predicates.length === 0) {
        throw new Error("Composite predicate must contain predicates");
      }
      return {
        kind: value.kind,
        predicates: predicate.predicates.map((child) =>
          parsePredicate(child, depth + 1)),
      };
    }
    case "not": {
      const predicate = exactObject(
        value,
        ["kind", "predicate"],
        "Not predicate",
      );
      return {
        kind: "not",
        predicate: parsePredicate(predicate.predicate, depth + 1),
      };
    }
    default:
      throw new Error("Diagnostic predicate kind is unsupported");
  }
}

function parseRuleEffect(value: unknown): RuleEffect {
  if (!isRecord(value)) {
    throw new Error("Rule effect must be an object");
  }
  switch (value.kind) {
    case "classification": {
      const effect = exactObject(
        value,
        ["kind", "classification"],
        "Classification effect",
      );
      return {
        kind: "classification",
        classification: classification(effect.classification),
      };
    }
    case "claim-ceiling": {
      const effect = exactObject(
        value,
        ["kind", "classification"],
        "Claim ceiling effect",
      );
      return {
        kind: "claim-ceiling",
        classification: classification(effect.classification),
      };
    }
    case "flag": {
      const effect = exactObject(value, ["kind", "flag"], "Flag effect");
      return { kind: "flag", flag: diagnosticFlag(effect.flag) };
    }
    case "finding": {
      const effect = exactObject(
        value,
        [
          "kind",
          "severity",
          "explanation",
          "canonical_concept_id",
          "evidence_needed",
        ],
        "Finding effect",
      );
      return {
        kind: "finding",
        severity: findingSeverity(effect.severity),
        explanation: text(effect.explanation, "Finding explanation"),
        canonical_concept_id: conceptId(
          effect.canonical_concept_id,
          "Finding concept",
        ),
        evidence_needed: text(effect.evidence_needed, "Finding evidence"),
      };
    }
    default:
      throw new Error("Rule effect kind is unsupported");
  }
}

function parseRule(value: unknown, index: number): DiagnosticRule {
  const rule = exactObject(
    value,
    ["rule_id", "version", "precedence", "predicate", "effect"],
    `Diagnostic rule ${index}`,
  );
  return {
    rule_id: ruleId(rule.rule_id, `Diagnostic rule ${index} ID`),
    version: integer(rule.version, `Diagnostic rule ${index} version`),
    precedence: integer(rule.precedence, `Diagnostic rule ${index} precedence`),
    predicate: parsePredicate(rule.predicate),
    effect: parseRuleEffect(rule.effect),
  };
}

function parseProvenance(value: unknown): FactProvenance {
  if (!isRecord(value)) {
    throw new Error("Fact provenance must be an object");
  }
  if (value.kind === "reader-assertion") {
    exactObject(value, ["kind"], "Reader assertion provenance");
    return { kind: "reader-assertion" };
  }
  const provenance = exactObject(
    value,
    ["kind", "source_id", "locator"],
    "Source provenance",
  );
  if (provenance.kind !== "source-backed") {
    throw new Error("Fact provenance kind is unsupported");
  }
  return {
    kind: "source-backed",
    source_id: sourceId(provenance.source_id, "Fact source"),
    locator: text(provenance.locator, "Fact locator"),
  };
}

function parseFactValue(value: unknown): CaseFact["value"] {
  if (typeof value === "boolean") {
    return value;
  }
  if (typeof value === "string") {
    return text(value, "Case fact value");
  }
  if (Array.isArray(value)) {
    return stringArray(value, "Case fact set");
  }
  throw new Error("Case fact value has an unsupported type");
}

function parseCase(value: unknown, index: number): DiagnosticCase {
  const item = exactObject(
    value,
    [
      "schema_version",
      "case_id",
      "system_id",
      "title",
      "method_family",
      "expected_classification",
      "expected_claim_ceiling",
      "facts",
    ],
    `Diagnostic case ${index}`,
  );
  if (item.schema_version !== 1 || !Array.isArray(item.facts)) {
    throw new Error(`Diagnostic case ${index} has an invalid schema`);
  }
  const system = item.system_id === null
    ? null
    : systemId(item.system_id, `Diagnostic case ${index} system`);
  return {
    schema_version: 1,
    case_id: caseId(item.case_id, `Diagnostic case ${index} ID`),
    system_id: system,
    title: text(item.title, `Diagnostic case ${index} title`),
    method_family: id(item.method_family, `Diagnostic case ${index} method`),
    expected_classification: classification(item.expected_classification),
    expected_claim_ceiling: classification(item.expected_claim_ceiling),
    facts: item.facts.map((value, factIndex) => {
      const fact = exactObject(
        value,
        ["field_id", "value", "provenance"],
        `Diagnostic case ${index} fact ${factIndex}`,
      );
      return {
        field_id: fieldId(fact.field_id, "Case fact field"),
        value: parseFactValue(fact.value),
        provenance: parseProvenance(fact.provenance),
      };
    }),
  };
}

export function parseDiagnostics(value: unknown): DiagnosticContracts {
  const diagnostics = exactObject(
    value,
    ["worksheet", "rules", "cases", "export_schema"],
    "Diagnostic contracts",
  );
  const worksheet = exactObject(
    diagnostics.worksheet,
    ["schema_version", "fields"],
    "Worksheet manifest",
  );
  const rules = exactObject(
    diagnostics.rules,
    ["schema_version", "classifications", "rules"],
    "Rule manifest",
  );
  const exportSchema = exactObject(
    diagnostics.export_schema,
    [
      "schema_version",
      "export_id",
      "required_sections",
      "max_import_bytes",
      "max_commentary_bytes",
      "digest_algorithm",
      "commentary_affects_diagnosis",
    ],
    "Export schema",
  );
  if (
    worksheet.schema_version !== 1
    || !Array.isArray(worksheet.fields)
    || rules.schema_version !== 1
    || !Array.isArray(rules.classifications)
    || !Array.isArray(rules.rules)
    || !Array.isArray(diagnostics.cases)
    || exportSchema.schema_version !== 1
    || exportSchema.export_id !== "rsi-diagnosis/v2"
    || exportSchema.digest_algorithm !== "sha256"
    || exportSchema.commentary_affects_diagnosis !== false
  ) {
    throw new Error("Diagnostic contracts have an unsupported schema");
  }
  const classifications = rules.classifications.map(classification);
  if (new Set(classifications).size !== 7) {
    throw new Error("Diagnostic classifications must contain seven unique values");
  }
  const requiredSections = stringArray(
    exportSchema.required_sections,
    "Export required sections",
  );
  const maxImportBytes = integer(
    exportSchema.max_import_bytes,
    "Export max import bytes",
  );
  const maxCommentaryBytes = integer(
    exportSchema.max_commentary_bytes,
    "Export max commentary bytes",
  );
  if (
    maxImportBytes <= 0
    || maxImportBytes > 1_048_576
    || maxCommentaryBytes <= 0
    || maxCommentaryBytes > maxImportBytes
  ) {
    throw new Error("Diagnostic export limits are invalid");
  }
  const fields = worksheet.fields.map(parseWorksheetField);
  const parsedRules = rules.rules.map(parseRule);
  const cases = diagnostics.cases.map(parseCase);
  if (
    new Set(fields.map((field) => field.field_id)).size !== fields.length
    || new Set(parsedRules.map((rule) => rule.rule_id)).size !== parsedRules.length
    || new Set(cases.map((item) => item.case_id)).size !== cases.length
  ) {
    throw new Error("Diagnostic contracts contain duplicate IDs");
  }
  return {
    worksheet: { schema_version: 1, fields },
    rules: {
      schema_version: 1,
      classifications,
      rules: parsedRules,
    },
    cases,
    export_schema: {
      schema_version: 1,
      export_id: "rsi-diagnosis/v2",
      required_sections: requiredSections,
      max_import_bytes: maxImportBytes,
      max_commentary_bytes: maxCommentaryBytes,
      digest_algorithm: "sha256",
      commentary_affects_diagnosis: false,
    },
  };
}
