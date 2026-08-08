import { canonicalCorpus } from "../content/canonical";
import type {
  CaseFact,
  CaseId,
  DiagnosticCase,
  DiagnosticFlag,
  FactProvenance,
  FieldId,
  FindingSeverity,
  Predicate,
  PrimaryClassification,
  RuleId,
} from "../content/types";

export type WorksheetState = {
  case_id: CaseId | null;
  title: string;
  method_family: string;
  facts: CaseFact[];
};

export type Finding = {
  rule_id: RuleId;
  severity: FindingSeverity;
  precedence: number;
  input_facts: {
    field_id: FieldId;
    value: CaseFact["value"];
  }[];
  result: true;
  explanation: string;
  canonical_concept_id: string;
  evidence_needed: string;
};

export type DiagnosisResult = {
  classification: PrimaryClassification | null;
  claimCeiling: PrimaryClassification | null;
  flags: DiagnosticFlag[];
  findings: Finding[];
  unresolvedRequiredFields: FieldId[];
};

type FactMap = ReadonlyMap<FieldId, CaseFact["value"]>;

function cloneValue(value: CaseFact["value"]): CaseFact["value"] {
  return Array.isArray(value) ? [...value] : value;
}

function sameValue(
  left: CaseFact["value"],
  right: CaseFact["value"],
): boolean {
  if (Array.isArray(left) && Array.isArray(right)) {
    return left.length === right.length
      && left.every((value, index) => value === right[index]);
  }
  return left === right;
}

export function forkCase(diagnosticCase: DiagnosticCase): WorksheetState {
  return {
    case_id: diagnosticCase.case_id,
    title: diagnosticCase.title,
    method_family: diagnosticCase.method_family,
    facts: diagnosticCase.facts.map((fact) => ({
      field_id: fact.field_id,
      value: cloneValue(fact.value),
      provenance: { ...fact.provenance },
    })),
  };
}

export function updateFact(
  state: WorksheetState,
  fieldId: string,
  value: CaseFact["value"],
): WorksheetState {
  const field = canonicalCorpus.diagnostics.worksheet.fields.find(
    (candidate) => candidate.field_id === fieldId,
  );
  if (!field) {
    throw new Error(`Unknown worksheet field ${fieldId}`);
  }
  const existing = state.facts.find((fact) => fact.field_id === field.field_id);
  const provenance: FactProvenance = existing && sameValue(existing.value, value)
    ? existing.provenance
    : { kind: "reader-assertion" };
  const next: CaseFact = {
    field_id: field.field_id,
    value: cloneValue(value),
    provenance,
  };
  return {
    ...state,
    facts: existing
      ? state.facts.map((fact) => fact.field_id === field.field_id ? next : fact)
      : [...state.facts, next],
  };
}

function evaluate(predicate: Predicate, facts: FactMap): boolean | null {
  switch (predicate.kind) {
    case "bool-equals": {
      const actual = facts.get(predicate.field_id);
      return actual === undefined ? null : actual === predicate.value;
    }
    case "enum-equals": {
      const actual = facts.get(predicate.field_id);
      return actual === undefined ? null : actual === predicate.value;
    }
    case "set-contains": {
      const value = facts.get(predicate.field_id);
      return value === undefined
        ? null
        : Array.isArray(value) && value.includes(predicate.value);
    }
    case "all": {
      const values = predicate.predicates.map((child) => evaluate(child, facts));
      return values.some((value) => value === false)
        ? false
        : values.every((value) => value === true)
          ? true
          : null;
    }
    case "any": {
      const values = predicate.predicates.map((child) => evaluate(child, facts));
      return values.some((value) => value === true)
        ? true
        : values.every((value) => value === false)
          ? false
          : null;
    }
    case "not": {
      const value = evaluate(predicate.predicate, facts);
      return value === null ? null : !value;
    }
    default: {
      const exhaustive: never = predicate;
      return exhaustive;
    }
  }
}

function severityOrder(severity: FindingSeverity): number {
  switch (severity) {
    case "blocking":
      return 0;
    case "warning":
      return 1;
    case "information":
      return 2;
    default: {
      const exhaustive: never = severity;
      return exhaustive;
    }
  }
}

function predicateFields(predicate: Predicate): FieldId[] {
  switch (predicate.kind) {
    case "bool-equals":
    case "enum-equals":
    case "set-contains":
      return [predicate.field_id];
    case "all":
    case "any":
      return [...new Set(predicate.predicates.flatMap(predicateFields))];
    case "not":
      return predicateFields(predicate.predicate);
    default: {
      const exhaustive: never = predicate;
      return exhaustive;
    }
  }
}

export function diagnose(state: WorksheetState): DiagnosisResult {
  const facts = new Map(
    state.facts.map((fact) => [fact.field_id, fact.value]),
  );
  const unresolved = canonicalCorpus.diagnostics.worksheet.fields
    .filter((field) => field.required && !facts.has(field.field_id))
    .map((field) => field.field_id);
  const satisfied = canonicalCorpus.diagnostics.rules.rules
    .filter((rule) => evaluate(rule.predicate, facts) === true)
    .sort((left, right) =>
      right.precedence - left.precedence
      || left.rule_id.localeCompare(right.rule_id));
  let classification: PrimaryClassification | null = null;
  let claimCeiling: PrimaryClassification | null = null;
  const flags: DiagnosticFlag[] = [];
  const findings: Finding[] = [];
  for (const rule of satisfied) {
    switch (rule.effect.kind) {
      case "classification":
        classification ??= rule.effect.classification;
        break;
      case "claim-ceiling":
        claimCeiling ??= rule.effect.classification;
        break;
      case "flag":
        flags.push(rule.effect.flag);
        break;
      case "finding":
        findings.push({
          rule_id: rule.rule_id,
          severity: rule.effect.severity,
          precedence: rule.precedence,
          input_facts: predicateFields(rule.predicate).flatMap((fieldId) => {
            const value = facts.get(fieldId);
            return value === undefined
              ? []
              : [{ field_id: fieldId, value: cloneValue(value) }];
          }),
          result: true,
          explanation: rule.effect.explanation,
          canonical_concept_id: rule.effect.canonical_concept_id,
          evidence_needed: rule.effect.evidence_needed,
        });
        break;
      default: {
        const exhaustive: never = rule.effect;
        throw new Error(`Unhandled rule effect ${String(exhaustive)}`);
      }
    }
  }
  findings.sort((left, right) =>
    severityOrder(left.severity) - severityOrder(right.severity)
    || right.precedence - left.precedence
    || left.rule_id.localeCompare(right.rule_id));
  return {
    classification: unresolved.length === 0 ? classification : null,
    claimCeiling: unresolved.length === 0 ? claimCeiling : null,
    flags: [...new Set(flags)],
    findings,
    unresolvedRequiredFields: unresolved,
  };
}
