import { canonicalCorpus } from "../content/canonical";
import type {
  CaseFact,
  CaseId,
  FactProvenance,
  FieldId,
  PrimaryClassification,
  RuleId,
  SourceId,
} from "../content/types";
import {
  conceptId,
  ruleId,
  savedRuleId,
} from "./contracts";
import {
  diagnose,
  type DiagnosisResult,
  type Finding,
  type WorksheetState,
} from "./engine";

export type ExportInput = {
  state: WorksheetState;
  result: DiagnosisResult;
  commentary?: string | null;
};

export type ImportedDiagnosis = {
  status: "current" | "stale";
  state: WorksheetState;
  savedDiagnosis: DiagnosisResult;
  commentary: string | null;
};

export type RuleLedgerEntry = {
  rule_id: RuleId;
  version: number;
  precedence: number;
};

type LegacyFact = {
  field_id: string;
  value: CaseFact["value"];
  provenance: FactProvenance;
};

const legacyFieldLabels: Readonly<Record<string, string>> = {
  "candidate-name": "Candidate name",
  "editable-components": "Editable components",
  "output-only": "Change affects only the current output",
  "persistence-scope": "Persistence scope",
  "persistence-mechanism": "Persistence mechanism",
  "accepted-generation": "External authority accepted a parent-child generation edge",
  "rejected-branches-recorded": "Rejected branches remain recorded",
  "child-produces-later-candidates": "Accepted child produces later candidates",
  "next-cycle-gain-measured": "Next-cycle gain is measured",
  "held-out-evaluation": "Held-out outcomes are evaluated",
  "evaluator-owner": "Evaluator owner",
  "candidate-evaluator-write-access": "Candidate can write evaluator state",
  "matched-envelope": "Proposal, tasks, permissions, and budget are matched",
  "root-tree-accounting-complete":
    "Root-tree accounting includes failed branches and descendants",
  "integrity-failure": "An integrity or authority gate failed",
  "automated-research-loop": "System automates hypothesis-to-experiment research",
  "evidence-status": "Evidence status",
  "controls-and-ablations": "Controls and ablations",
  "environment-revision": "Environment and revision",
  "evidence-locator": "Evidence locator",
};

const legacyRulePrecedence: Readonly<Record<string, number>> = {
  "integrity-evaluator-write": 800,
  "integrity-authority-failure": 790,
  "class-recursive-improvement": 700,
  "class-successor-improvement": 600,
  "class-joint-adaptation": 500,
  "class-automated-research": 400,
  "class-harness-improvement": 300,
  "class-persistent-adaptation": 200,
  "class-output-refinement": 100,
  "flag-persistence-established": 50,
  "flag-accepted-generation": 49,
  "flag-matched-envelope": 48,
  "flag-evaluator-independent": 47,
  "flag-root-accounting": 46,
  "flag-next-cycle-gain": 45,
  "flag-independent-reproduction": 44,
};

const legacyFindingRules: Readonly<Record<string, {
  severity: Finding["severity"];
  explanation: string;
  canonicalConceptId: string;
  evidenceNeeded: string;
}>> = {
  "integrity-evaluator-write": {
    severity: "blocking",
    explanation:
      "Candidate write access to evaluator state invalidates self-certification.",
    canonicalConceptId: "evaluator-independence",
    evidenceNeeded:
      "A capability audit showing evaluator state is read-only to the candidate.",
  },
  "integrity-authority-failure": {
    severity: "blocking",
    explanation:
      "A failed integrity or authority gate blocks promotion and lowers the claim ceiling.",
    canonicalConceptId: "reward-hacking",
    evidenceNeeded:
      "A cleared integrity receipt from an authority outside candidate control.",
  },
};

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

function readable(value: string): string {
  const phrase = value.replaceAll("-", " ");
  return phrase.charAt(0).toUpperCase() + phrase.slice(1);
}

function stableValue(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(stableValue);
  }
  if (isRecord(value)) {
    const output: Record<string, unknown> = {};
    for (const key of Object.keys(value).sort()) {
      output[key] = stableValue(value[key]);
    }
    return output;
  }
  return value;
}

function stableJson(value: unknown): string {
  return `${JSON.stringify(stableValue(value), null, 2)}\n`;
}

async function sha256(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

function normalizedFacts(state: WorksheetState): CaseFact[] {
  return canonicalCorpus.diagnostics.worksheet.fields.flatMap((field) => {
    const fact = state.facts.find(
      (candidate) => candidate.field_id === field.field_id,
    );
    if (!fact) {
      return [];
    }
    return [{
      field_id: fact.field_id,
      value: Array.isArray(fact.value)
        ? [...fact.value].sort()
        : fact.value,
      provenance: { ...fact.provenance },
    }];
  });
}

function normalizedResult(result: DiagnosisResult): DiagnosisResult {
  return {
    classification: result.classification,
    claimCeiling: result.claimCeiling,
    flags: [...result.flags].sort(),
    findings: [...result.findings].sort((left, right) =>
      left.severity.localeCompare(right.severity)
      || right.precedence - left.precedence
      || left.rule_id.localeCompare(right.rule_id)),
    unresolvedRequiredFields: [...result.unresolvedRequiredFields],
  };
}

function sourceLedger(facts: CaseFact[]): {
  source_id: SourceId;
  locator: string;
}[] {
  const sources = new Map<string, { source_id: SourceId; locator: string }>();
  for (const fact of facts) {
    if (fact.provenance.kind === "source-backed") {
      const key = `${fact.provenance.source_id}\0${fact.provenance.locator}`;
      sources.set(key, {
        source_id: fact.provenance.source_id,
        locator: fact.provenance.locator,
      });
    }
  }
  return [...sources.values()].sort((left, right) =>
    left.source_id.localeCompare(right.source_id)
    || left.locator.localeCompare(right.locator));
}

function legacySourceLedger(facts: LegacyFact[]): {
  source_id: SourceId;
  locator: string;
}[] {
  const sources = new Map<string, { source_id: SourceId; locator: string }>();
  for (const fact of facts) {
    if (fact.provenance.kind === "source-backed") {
      const key = `${fact.provenance.source_id}\0${fact.provenance.locator}`;
      sources.set(key, {
        source_id: fact.provenance.source_id,
        locator: fact.provenance.locator,
      });
    }
  }
  return [...sources.values()].sort((left, right) =>
    left.source_id.localeCompare(right.source_id)
    || left.locator.localeCompare(right.locator));
}

function factDisplay(value: CaseFact["value"]): string {
  if (Array.isArray(value)) {
    return value.length === 0 ? "none" : value.join(", ");
  }
  return String(value);
}

function activeRuleLedger(): RuleLedgerEntry[] {
  return canonicalCorpus.diagnostics.rules.rules.map((rule) => ({
    rule_id: rule.rule_id,
    version: rule.version,
    precedence: rule.precedence,
  }));
}

export function exportDiagnosisMarkdown(
  input: ExportInput,
  ruleLedger: readonly RuleLedgerEntry[] = activeRuleLedger(),
): string {
  const facts = normalizedFacts(input.state);
  const result = normalizedResult(input.result);
  const lines = [
    `# RSI decision brief: ${input.state.title}`,
    "",
    "## Executive verdict",
    "",
    result.classification === null
      ? "Incomplete. Required worksheet facts are missing."
      : `Primary classification: ${readable(result.classification)}.`,
    "",
    "## Candidate and protected boundaries",
    "",
    `Method family: ${readable(input.state.method_family)}.`,
    "",
    "## Worksheet",
    "",
    ...facts.flatMap((fact) => {
      const field = canonicalCorpus.diagnostics.worksheet.fields.find(
        (candidate) => candidate.field_id === fact.field_id,
      );
      const provenance = fact.provenance.kind === "source-backed"
        ? `${fact.provenance.source_id}, ${fact.provenance.locator}`
        : "Reader assertion";
      return [
        `- ${field?.label ?? fact.field_id}: ${factDisplay(fact.value)}`,
        `  Provenance: ${provenance}`,
      ];
    }),
    "",
    "## Honest claim ceiling",
    "",
    result.claimCeiling === null
      ? "No classification until required facts are complete."
      : readable(result.claimCeiling),
    "",
    "## Findings",
    "",
    ...(result.findings.length === 0
      ? ["No blocking or warning findings."]
      : result.findings.flatMap((finding) => [
          `- ${readable(finding.severity)} / ${finding.rule_id}: ${finding.explanation}`,
          `  Input facts: ${
            finding.input_facts.length === 0
              ? "none"
              : finding.input_facts
                .map((fact) => `${fact.field_id}=${factDisplay(fact.value)}`)
                .join("; ")
          }`,
          `  Result: ${String(finding.result)}`,
          `  Canonical concept: ${finding.canonical_concept_id}`,
          `  Evidence needed: ${finding.evidence_needed}`,
        ])),
    "",
    "## Missing evidence",
    "",
    ...(result.unresolvedRequiredFields.length === 0
      ? ["No required worksheet fields are unresolved."]
      : result.unresolvedRequiredFields.map((fieldId) =>
          `- ${readable(fieldId)}`)),
    "",
    "## Smallest useful next experiment",
    "",
    result.classification === "recursive-improvement-demonstrated"
      ? "Repeat the matched next-cycle comparison on fresh tasks and seeds."
      : "Measure whether an accepted child produces better later accepted changes under a matched protected envelope.",
    "",
    "## Rule ledger",
    "",
    ...ruleLedger.map((rule) =>
      `- ${rule.rule_id} v${rule.version}, precedence ${rule.precedence}`),
    "",
    "## Source ledger",
    "",
    ...sourceLedger(facts).map((source) =>
      `- ${source.source_id}: ${source.locator}`),
    "",
    "## Imported AI commentary",
    "",
    input.commentary?.trim() || "None.",
    "",
  ];
  return lines.join("\n");
}

function exportLegacyMarkdown({
  title,
  methodFamily,
  facts,
  result,
  rules,
  unresolvedLegacy,
  commentary,
}: {
  title: string;
  methodFamily: string;
  facts: LegacyFact[];
  result: DiagnosisResult;
  rules: readonly RuleLedgerEntry[];
  unresolvedLegacy: readonly string[];
  commentary: string | null;
}): string {
  const lines = [
    `# RSI decision brief: ${title}`,
    "",
    "## Executive verdict",
    "",
    result.classification === null
      ? "Incomplete. Required worksheet facts are missing."
      : `Primary classification: ${readable(result.classification)}.`,
    "",
    "## Candidate and protected boundaries",
    "",
    `Method family: ${readable(methodFamily)}.`,
    "",
    "## Worksheet",
    "",
    ...facts.flatMap((fact) => {
      const provenance = fact.provenance.kind === "source-backed"
        ? `${fact.provenance.source_id}, ${fact.provenance.locator}`
        : "Reader assertion";
      return [
        `- ${legacyFieldLabels[fact.field_id] ?? fact.field_id}: ${
          factDisplay(fact.value)
        }`,
        `  Provenance: ${provenance}`,
      ];
    }),
    "",
    "## Honest claim ceiling",
    "",
    result.classification === null
      ? "No classification until required facts are complete."
      : readable(result.classification),
    "",
    "## Findings",
    "",
    ...(result.findings.length === 0
      ? ["No blocking or warning findings."]
      : result.findings.map((finding) =>
          `- ${readable(finding.severity)} / ${finding.rule_id}: ${
            finding.explanation
          }`)),
    "",
    "## Missing evidence",
    "",
    ...(unresolvedLegacy.length === 0
      ? ["No required worksheet fields are unresolved."]
      : unresolvedLegacy.map((fieldId) =>
          `- ${readable(fieldId)}`)),
    "",
    "## Smallest useful next experiment",
    "",
    result.classification === "recursive-improvement-demonstrated"
      ? "Repeat the matched next-cycle comparison on fresh tasks and seeds."
      : "Measure whether an accepted child produces better later accepted changes under a matched protected envelope.",
    "",
    "## Rule ledger",
    "",
    ...rules.map((rule) =>
      `- ${rule.rule_id} v${rule.version}, precedence ${rule.precedence}`),
    "",
    "## Source ledger",
    "",
    ...legacySourceLedger(facts).map((source) =>
      `- ${source.source_id}: ${source.locator}`),
    "",
    "## Imported AI commentary",
    "",
    commentary?.trim() || "None.",
    "",
  ];
  return lines.join("\n");
}

function corpusIdentity(): unknown {
  return {
    schema_version: canonicalCorpus.schema_version,
    documents: canonicalCorpus.documents.map((document) => ({
      concept_id: document.concept_id,
      markdown_sha256: document.markdown_sha256,
    })),
    systems: canonicalCorpus.systems.map((system) => ({
      system_id: system.system_id,
      diagnostic_case_path: system.diagnostic_case_path,
    })),
    cases: canonicalCorpus.diagnostics.cases.map((item) => item.case_id),
    lessons: canonicalCorpus.lessons.map((lesson) => ({
      lesson_id: lesson.lesson_id,
      case_ids: lesson.case_ids,
      concept_ids: lesson.concept_ids,
    })),
  };
}

async function currentDigests(): Promise<{
  corpus: string;
  rules: string;
}> {
  return {
    corpus: await sha256(stableJson(corpusIdentity())),
    rules: await sha256(stableJson(canonicalCorpus.diagnostics.rules)),
  };
}

export async function exportDiagnosisJson(input: ExportInput): Promise<string> {
  const facts = normalizedFacts(input.state);
  const result = normalizedResult(input.result);
  const rules = activeRuleLedger();
  const markdown = exportDiagnosisMarkdown(input, rules);
  const digests = await currentDigests();
  const payload = {
    schema_version: "rsi-diagnosis/v2",
    corpus_sha256: digests.corpus,
    rules_sha256: digests.rules,
    identity: {
      case_id: input.state.case_id,
      title: input.state.title,
      method_family: input.state.method_family,
    },
    worksheet: facts,
    provenance: facts.map((fact) => ({
      field_id: fact.field_id,
      provenance: fact.provenance,
    })),
    diagnosis: result,
    rules,
    sources: sourceLedger(facts),
    brief_sha256: await sha256(markdown),
    commentary: input.commentary?.trim() || null,
  };
  return stableJson(payload);
}

export function exportCritiquePacket(input: ExportInput): string {
  return [
    "# External critique packet",
    "",
    "Review the deterministic RSI decision brief below.",
    "",
    "- Do not rewrite the deterministic verdict.",
    "- Challenge causal attribution, evaluator independence, resource parity, and claim strength.",
    "- Identify missing controls, hidden authority, uncounted descendant work, and unsupported source transfer.",
    "- Return commentary only. Commentary cannot change worksheet facts or rule results.",
    "",
    exportDiagnosisMarkdown(input),
  ].join("\n");
}

export function importCommentary(value: string): string {
  const bytes = new TextEncoder().encode(value);
  if (bytes.length > canonicalCorpus.diagnostics.export_schema.max_commentary_bytes) {
    throw new Error("Imported commentary is too large");
  }
  if (value.includes("\0")) {
    throw new Error("Imported commentary contains NUL");
  }
  return value;
}

function knownField(value: unknown): FieldId {
  const parsed = text(value, "Imported field ID");
  const field = canonicalCorpus.diagnostics.worksheet.fields.find(
    (candidate) => candidate.field_id === parsed,
  );
  if (!field) {
    throw new Error(`Imported diagnosis references unknown field ${parsed}`);
  }
  return field.field_id;
}

function parseProvenance(value: unknown): FactProvenance {
  if (!isRecord(value)) {
    throw new Error("Imported provenance must be an object");
  }
  if (value.kind === "reader-assertion") {
    exactObject(value, ["kind"], "Imported reader provenance");
    return { kind: "reader-assertion" };
  }
  const source = exactObject(
    value,
    ["kind", "source_id", "locator"],
    "Imported source provenance",
  );
  if (source.kind !== "source-backed") {
    throw new Error("Imported provenance kind is unsupported");
  }
  const sourceText = text(source.source_id, "Imported source ID");
  const knownSource = canonicalCorpus.retained_concepts
    .flatMap((concept) => concept.source_ids)
    .find((candidate) => candidate === sourceText);
  if (!knownSource) {
    throw new Error(`Imported diagnosis references unknown source ${sourceText}`);
  }
  return {
    kind: "source-backed",
    source_id: knownSource,
    locator: text(source.locator, "Imported source locator"),
  };
}

function parseLegacyFact(value: unknown): LegacyFact {
  const fact = exactObject(
    value,
    ["field_id", "value", "provenance"],
    "Legacy fact",
  );
  const fieldId = text(fact.field_id, "Legacy field ID");
  if (!(fieldId in legacyFieldLabels)) {
    throw new Error(`Legacy diagnosis references unknown field ${fieldId}`);
  }
  const enumValues: Readonly<Record<string, readonly string[]>> = {
    "persistence-scope": [
      "none",
      "current-task",
      "cross-episode",
      "accepted-generation",
    ],
    "persistence-mechanism": [
      "none",
      "automatic",
      "selected",
      "externally-promoted",
    ],
    "evaluator-owner": ["candidate", "external", "shared", "unknown"],
    "evidence-status": [
      "executed",
      "source-reported",
      "independently-reproduced",
      "proposed",
      "missing",
    ],
  };
  const booleanFields = new Set([
    "output-only",
    "accepted-generation",
    "rejected-branches-recorded",
    "child-produces-later-candidates",
    "next-cycle-gain-measured",
    "held-out-evaluation",
    "candidate-evaluator-write-access",
    "matched-envelope",
    "root-tree-accounting-complete",
    "integrity-failure",
    "automated-research-loop",
  ]);
  let parsed: CaseFact["value"];
  if (booleanFields.has(fieldId)) {
    if (typeof fact.value !== "boolean") {
      throw new Error(`Legacy ${fieldId} must be boolean`);
    }
    parsed = fact.value;
  } else if (fieldId === "editable-components") {
    const allowed = new Set(["weights", "harness", "artifacts", "retrospection"]);
    if (
      !Array.isArray(fact.value)
      || fact.value.some(
        (item) => typeof item !== "string" || !allowed.has(item),
      )
      || new Set(fact.value).size !== fact.value.length
    ) {
      throw new Error("Legacy editable-components is invalid");
    }
    parsed = fact.value.map(String).sort();
  } else if (fieldId in enumValues) {
    const candidate = text(fact.value, `Legacy ${fieldId}`);
    if (!enumValues[fieldId].includes(candidate)) {
      throw new Error(`Legacy ${fieldId} has an unknown value`);
    }
    parsed = candidate;
  } else {
    parsed = text(fact.value, `Legacy ${fieldId}`);
  }
  return {
    field_id: fieldId,
    value: parsed,
    provenance: parseProvenance(fact.provenance),
  };
}

function parseFact(value: unknown): CaseFact {
  const fact = exactObject(
    value,
    ["field_id", "value", "provenance"],
    "Imported fact",
  );
  const field = knownField(fact.field_id);
  const definition = canonicalCorpus.diagnostics.worksheet.fields.find(
    (candidate) => candidate.field_id === field,
  );
  if (!definition) {
    throw new Error(`Imported field disappeared ${field}`);
  }
  let parsed: CaseFact["value"];
  switch (definition.kind) {
    case "boolean":
      if (typeof fact.value !== "boolean") {
        throw new Error(`Imported ${field} must be boolean`);
      }
      parsed = fact.value;
      break;
    case "text":
      parsed = text(fact.value, `Imported ${field}`);
      break;
    case "enum": {
      const option = text(fact.value, `Imported ${field}`);
      if (!definition.values.includes(option)) {
        throw new Error(`Imported ${field} has an unknown value`);
      }
      parsed = option;
      break;
    }
    case "set":
      if (
        !Array.isArray(fact.value)
        || fact.value.some(
          (option) =>
            typeof option !== "string" || !definition.values.includes(option),
        )
        || new Set(fact.value).size !== fact.value.length
      ) {
        throw new Error(`Imported ${field} has an invalid set`);
      }
      parsed = fact.value.map((option) => String(option)).sort();
      break;
    default: {
      const exhaustive: never = definition;
      throw new Error(`Unhandled imported field ${String(exhaustive)}`);
    }
  }
  return {
    field_id: field,
    value: parsed,
    provenance: parseProvenance(fact.provenance),
  };
}

function savedClassification(value: unknown): PrimaryClassification | null {
  if (value === null) {
    return null;
  }
  const known = canonicalCorpus.diagnostics.rules.classifications.find(
    (candidate) => candidate === value,
  );
  if (!known) {
    throw new Error("Imported diagnosis has an unknown classification");
  }
  return known;
}

function parseSavedDiagnosis(
  value: unknown,
  requireActiveRules: boolean,
): DiagnosisResult {
  const diagnosis = exactObject(
    value,
    [
      "classification",
      "claimCeiling",
      "flags",
      "findings",
      "unresolvedRequiredFields",
    ],
    "Imported diagnosis",
  );
  if (
    !Array.isArray(diagnosis.flags)
    || !Array.isArray(diagnosis.findings)
    || !Array.isArray(diagnosis.unresolvedRequiredFields)
  ) {
    throw new Error("Imported diagnosis collections are invalid");
  }
  const flags = diagnosis.flags.map((flag) => {
      const known = canonicalCorpus.diagnostics.rules.rules
        .flatMap((rule) => rule.effect.kind === "flag" ? [rule.effect.flag] : [])
        .find((candidate) => candidate === flag);
      if (!known) {
        throw new Error("Imported diagnosis has an unknown flag");
      }
      return known;
    });
  const findings = diagnosis.findings.map((finding) => {
      const parsedFinding = exactObject(
        finding,
        [
          "rule_id",
          "severity",
          "precedence",
          "input_facts",
          "result",
          "explanation",
          "canonical_concept_id",
          "evidence_needed",
        ],
        "Imported finding",
      );
      const ruleText = text(parsedFinding.rule_id, "Imported finding rule");
      const knownRule = canonicalCorpus.diagnostics.rules.rules.find(
        (rule) => rule.rule_id === ruleText && rule.effect.kind === "finding",
      );
      if (requireActiveRules && (!knownRule || knownRule.effect.kind !== "finding")) {
        throw new Error("Imported finding has an unknown rule");
      }
      if (
        !Array.isArray(parsedFinding.input_facts)
        || parsedFinding.result !== true
      ) {
        throw new Error("Imported finding has invalid input facts or result");
      }
      const inputFacts = parsedFinding.input_facts.map((inputFact) => {
        const parsedInput = exactObject(
          inputFact,
          ["field_id", "value"],
          "Imported finding input fact",
        );
        const field = knownField(parsedInput.field_id);
        const definition = canonicalCorpus.diagnostics.worksheet.fields.find(
          (candidate) => candidate.field_id === field,
        );
        if (!definition) {
          throw new Error(`Imported finding field disappeared ${field}`);
        }
        return {
          field_id: field,
          value: parseFact({
            field_id: field,
            value: parsedInput.value,
            provenance: { kind: "reader-assertion" },
          }).value,
        };
      });
      if (
        new Set(inputFacts.map((fact) => fact.field_id)).size
          !== inputFacts.length
      ) {
        throw new Error("Imported finding contains duplicate input facts");
      }
      const severity = parsedFinding.severity;
      if (
        severity !== "blocking"
        && severity !== "warning"
        && severity !== "information"
      ) {
        throw new Error("Imported finding has an invalid severity");
      }
      if (
        typeof parsedFinding.precedence !== "number"
        || !Number.isSafeInteger(parsedFinding.precedence)
        || parsedFinding.precedence < 0
      ) {
        throw new Error("Imported finding has an invalid precedence");
      }
      const expected: Finding = {
        rule_id: requireActiveRules && knownRule
          ? knownRule.rule_id
          : savedRuleId(ruleText, "Imported finding rule"),
        severity: requireActiveRules && knownRule?.effect.kind === "finding"
          ? knownRule.effect.severity
          : severity,
        precedence: requireActiveRules && knownRule
          ? knownRule.precedence
          : parsedFinding.precedence,
        input_facts: inputFacts,
        result: true,
        explanation: requireActiveRules && knownRule?.effect.kind === "finding"
          ? knownRule.effect.explanation
          : text(parsedFinding.explanation, "Imported finding explanation"),
        canonical_concept_id:
          requireActiveRules && knownRule?.effect.kind === "finding"
          ? knownRule.effect.canonical_concept_id
          : conceptId(
              parsedFinding.canonical_concept_id,
              "Imported finding concept",
            ),
        evidence_needed: requireActiveRules && knownRule?.effect.kind === "finding"
          ? knownRule.effect.evidence_needed
          : text(parsedFinding.evidence_needed, "Imported finding evidence"),
      };
      if (
        requireActiveRules
        && stableJson(parsedFinding) !== stableJson(expected)
      ) {
        throw new Error("Imported finding does not match the active rule");
      }
      return expected;
    });
  const unresolved = diagnosis.unresolvedRequiredFields.map(knownField);
  if (
    new Set(flags).size !== flags.length
    || new Set(findings.map((finding) => finding.rule_id)).size !== findings.length
    || new Set(unresolved).size !== unresolved.length
  ) {
    throw new Error("Imported diagnosis contains duplicate derived IDs");
  }
  return {
    classification: savedClassification(diagnosis.classification),
    claimCeiling: savedClassification(diagnosis.claimCeiling),
    flags,
    findings,
    unresolvedRequiredFields: unresolved,
  };
}

function parseRuleLedger(
  value: unknown[],
  requireActiveRules: boolean,
): RuleLedgerEntry[] {
  const parsed = value.map((entry, index) => {
    const rule = exactObject(
      entry,
      ["rule_id", "version", "precedence"],
      `Imported rule ledger entry ${index}`,
    );
    if (
      typeof rule.version !== "number"
      || !Number.isSafeInteger(rule.version)
      || rule.version <= 0
      || typeof rule.precedence !== "number"
      || !Number.isSafeInteger(rule.precedence)
      || rule.precedence < 0
    ) {
      throw new Error("Imported rule ledger has invalid metadata");
    }
    return {
      rule_id: requireActiveRules
        ? ruleId(rule.rule_id, "Imported active rule ID")
        : savedRuleId(rule.rule_id, "Imported stale rule ID"),
      version: rule.version,
      precedence: rule.precedence,
    };
  });
  if (new Set(parsed.map((rule) => rule.rule_id)).size !== parsed.length) {
    throw new Error("Imported rule ledger contains duplicate IDs");
  }
  return parsed;
}

function parseLegacyRules(value: unknown[]): RuleLedgerEntry[] {
  const parsed = value.map((entry, index) => {
    const rule = exactObject(
      entry,
      ["rule_id", "version"],
      `Legacy rule ledger entry ${index}`,
    );
    const ruleText = text(rule.rule_id, "Legacy rule ID");
    const precedence = legacyRulePrecedence[ruleText];
    if (
      precedence === undefined
      || rule.version !== 1
    ) {
      throw new Error("Legacy rule ledger is not the supported v1 rule set");
    }
    return {
      rule_id: savedRuleId(ruleText, "Legacy rule ID"),
      version: 1,
      precedence,
    };
  });
  if (
    parsed.length !== Object.keys(legacyRulePrecedence).length
    || new Set(parsed.map((rule) => rule.rule_id)).size !== parsed.length
  ) {
    throw new Error("Legacy rule ledger is incomplete or duplicated");
  }
  return parsed;
}

function legacyFlag(value: unknown): DiagnosisResult["flags"][number] {
  const known = canonicalCorpus.diagnostics.rules.rules
    .flatMap((rule) => rule.effect.kind === "flag" ? [rule.effect.flag] : [])
    .find((candidate) => candidate === value);
  if (!known) {
    throw new Error("Legacy diagnosis has an unknown flag");
  }
  return known;
}

function parseLegacyDiagnosis(value: unknown): {
  result: DiagnosisResult;
  unresolvedLegacy: string[];
} {
  const diagnosis = exactObject(
    value,
    ["classification", "flags", "findings", "unresolved_required_fields"],
    "Legacy diagnosis",
  );
  if (
    !Array.isArray(diagnosis.flags)
    || !Array.isArray(diagnosis.findings)
    || !Array.isArray(diagnosis.unresolved_required_fields)
  ) {
    throw new Error("Legacy diagnosis collections are invalid");
  }
  const flags = diagnosis.flags.map(legacyFlag);
  const findings = diagnosis.findings.map((finding) => {
    const parsed = exactObject(
      finding,
      [
        "rule_id",
        "severity",
        "precedence",
        "explanation",
        "canonical_concept_id",
        "evidence_needed",
      ],
      "Legacy finding",
    );
    const ruleText = text(parsed.rule_id, "Legacy finding rule");
    const contract = legacyFindingRules[ruleText];
    if (
      !contract
      || parsed.severity !== contract.severity
      || parsed.precedence !== legacyRulePrecedence[ruleText]
      || parsed.explanation !== contract.explanation
      || parsed.canonical_concept_id !== contract.canonicalConceptId
      || parsed.evidence_needed !== contract.evidenceNeeded
    ) {
      throw new Error("Legacy finding does not match the v1 rule contract");
    }
    return {
      rule_id: savedRuleId(ruleText, "Legacy finding rule"),
      severity: contract.severity,
      precedence: legacyRulePrecedence[ruleText],
      input_facts: [],
      result: true as const,
      explanation: contract.explanation,
      canonical_concept_id: conceptId(
        contract.canonicalConceptId,
        "Legacy finding concept",
      ),
      evidence_needed: contract.evidenceNeeded,
    };
  });
  const unresolvedLegacy = diagnosis.unresolved_required_fields.map(
    (field) => {
      const fieldId = text(field, "Legacy unresolved field");
      if (!(fieldId in legacyFieldLabels)) {
        throw new Error(`Legacy diagnosis has unknown unresolved field ${fieldId}`);
      }
      return fieldId;
    },
  );
  if (
    new Set(flags).size !== flags.length
    || new Set(findings.map((finding) => finding.rule_id)).size !== findings.length
    || new Set(unresolvedLegacy).size !== unresolvedLegacy.length
  ) {
    throw new Error("Legacy diagnosis contains duplicate derived IDs");
  }
  const classification = savedClassification(diagnosis.classification);
  return {
    result: {
      classification,
      claimCeiling: classification,
      flags,
      findings,
      unresolvedRequiredFields: [],
    },
    unresolvedLegacy,
  };
}

function legacyFactMap(facts: LegacyFact[]): Map<string, LegacyFact> {
  const output = new Map<string, LegacyFact>();
  for (const fact of facts) {
    if (output.has(fact.field_id)) {
      throw new Error("Legacy diagnosis contains duplicate facts");
    }
    output.set(fact.field_id, fact);
  }
  return output;
}

function asserted(fieldId: string, value: CaseFact["value"]): CaseFact {
  return {
    field_id: knownField(fieldId),
    value: Array.isArray(value) ? [...value] : value,
    provenance: { kind: "reader-assertion" },
  };
}

function migratedLegacyState(
  identity: Record<string, unknown>,
  legacyFacts: LegacyFact[],
): WorksheetState {
  const facts = legacyFactMap(legacyFacts);
  const read = (fieldId: string): LegacyFact => {
    const fact = facts.get(fieldId);
    if (!fact) {
      throw new Error(`Legacy diagnosis is missing field ${fieldId}`);
    }
    return fact;
  };
  const direct = (oldId: string, newId = oldId): CaseFact => {
    const fact = read(oldId);
    return {
      field_id: knownField(newId),
      value: Array.isArray(fact.value) ? [...fact.value] : fact.value,
      provenance: { ...fact.provenance },
    };
  };
  const components = read("editable-components");
  if (!Array.isArray(components.value)) {
    throw new Error("Legacy editable components disappeared");
  }
  const mappedComponents = components.value.map((value) =>
    value === "artifacts"
      ? "persistent-artifacts"
      : value === "retrospection"
        ? "retrospection-policy"
        : value);
  const persistenceScope = String(read("persistence-scope").value);
  const evaluatorOwner = String(read("evaluator-owner").value);
  const evidenceStatus = String(read("evidence-status").value);
  const automated = read("automated-research-loop").value === true;
  const hasWeights = mappedComponents.includes("weights");
  const hasHarness = mappedComponents.includes("harness");
  const systemKind = automated
    ? "automated-research"
    : hasWeights && hasHarness
      ? "joint-adaptation"
      : hasHarness
        ? "harness-optimizer"
        : mappedComponents.includes("persistent-artifacts")
          ? "context-manager"
          : "task-agent";
  return {
    case_id: importedCaseId(identity.case_id),
    title: text(identity.title, "Legacy imported title"),
    method_family: text(identity.method_family, "Legacy imported method"),
    facts: [
      direct("candidate-name"),
      asserted("system-kind", systemKind),
      {
        field_id: knownField("editable-components"),
        value: mappedComponents,
        provenance: { ...components.provenance },
      },
      asserted("proposal-owner", "unknown"),
      asserted("task-outcome-measured", read("held-out-evaluation").value),
      asserted(
        "task-outcome-improved",
        evidenceStatus !== "proposed" && evidenceStatus !== "missing",
      ),
      direct("persistence-scope"),
      direct("persistence-mechanism"),
      asserted(
        "later-consumer-present",
        persistenceScope !== "none" && persistenceScope !== "current-task",
      ),
      direct("accepted-generation", "accepted-generation-edge"),
      asserted("generation-edge-owner", "unknown"),
      direct("rejected-branches-recorded", "rejected-lineage-retained"),
      direct(
        "child-produces-later-candidates",
        "accepted-child-produces-later-candidates",
      ),
      direct("held-out-evaluation", "held-out-outcomes"),
      asserted(
        "evaluator-owner",
        evaluatorOwner === "external"
          ? "external-controller"
          : evaluatorOwner === "shared"
            ? "mixed"
            : evaluatorOwner,
      ),
      direct(
        "candidate-evaluator-write-access",
        "candidate-can-write-evaluator",
      ),
      asserted("development-heldout-split", false),
      direct("matched-envelope", "matched-proposal-protocol"),
      direct(
        "root-tree-accounting-complete",
        "complete-root-tree-accounting",
      ),
      asserted("permissions-external", false),
      asserted("archive-external", false),
      asserted("promotion-external", false),
      asserted("rollback-external", false),
      direct("next-cycle-gain-measured"),
      asserted("next-cycle-gain-positive", false),
      direct("evidence-status"),
    ],
  };
}

function importedCaseId(value: unknown): CaseId | null {
  if (value === null) {
    return null;
  }
  const parsed = text(value, "Imported case ID");
  const item = canonicalCorpus.diagnostics.cases.find(
    (candidate) => candidate.case_id === parsed,
  );
  if (!item) {
    throw new Error(`Imported diagnosis has unknown case ${parsed}`);
  }
  return item.case_id;
}

export async function importDiagnosisJson(value: string): Promise<ImportedDiagnosis> {
  const bytes = new TextEncoder().encode(value);
  if (bytes.length > canonicalCorpus.diagnostics.export_schema.max_import_bytes) {
    throw new Error("Imported diagnosis is too large");
  }
  const parsed: unknown = JSON.parse(value);
  const root = exactObject(
    parsed,
    [
      "schema_version",
      "corpus_sha256",
      "rules_sha256",
      "identity",
      "worksheet",
      "provenance",
      "diagnosis",
      "rules",
      "sources",
      "brief_sha256",
      "commentary",
    ],
    "Imported diagnosis",
  );
  if (
    !Array.isArray(root.worksheet)
    || !Array.isArray(root.provenance)
    || !Array.isArray(root.rules)
    || !Array.isArray(root.sources)
    || typeof root.corpus_sha256 !== "string"
    || typeof root.rules_sha256 !== "string"
    || !/^[a-f0-9]{64}$/.test(root.corpus_sha256)
    || !/^[a-f0-9]{64}$/.test(root.rules_sha256)
    || typeof root.brief_sha256 !== "string"
    || !/^[a-f0-9]{64}$/.test(root.brief_sha256)
  ) {
    throw new Error("Imported diagnosis has an invalid schema or digest");
  }
  if (root.schema_version === "rsi-diagnosis/v1") {
    const identity = exactObject(
      root.identity,
      ["case_id", "title", "method_family"],
      "Legacy imported identity",
    );
    const legacyFacts = root.worksheet.map(parseLegacyFact);
    const factIds = legacyFacts.map((fact) => fact.field_id);
    if (
      new Set(factIds).size !== factIds.length
      || Object.keys(legacyFieldLabels).some(
        (fieldId) =>
          !factIds.includes(fieldId)
          && !["controls-and-ablations", "environment-revision"].includes(fieldId),
      )
    ) {
      throw new Error("Legacy diagnosis has missing or duplicate facts");
    }
    const expectedProvenance = legacyFacts.map((fact) => ({
      field_id: fact.field_id,
      provenance: fact.provenance,
    }));
    if (stableJson(root.provenance) !== stableJson(expectedProvenance)) {
      throw new Error("Legacy provenance ledger does not match worksheet facts");
    }
    if (stableJson(root.sources) !== stableJson(legacySourceLedger(legacyFacts))) {
      throw new Error("Legacy source ledger does not match worksheet provenance");
    }
    const savedRules = parseLegacyRules(root.rules);
    const { result: savedDiagnosis, unresolvedLegacy } =
      parseLegacyDiagnosis(root.diagnosis);
    const commentary = root.commentary === null
      ? null
      : importCommentary(text(root.commentary, "Legacy imported commentary"));
    const savedMarkdown = exportLegacyMarkdown({
      title: text(identity.title, "Legacy imported title"),
      methodFamily: text(identity.method_family, "Legacy imported method"),
      facts: legacyFacts,
      result: savedDiagnosis,
      rules: savedRules,
      unresolvedLegacy,
      commentary,
    });
    if (await sha256(savedMarkdown) !== root.brief_sha256) {
      throw new Error("Legacy brief digest does not match saved content");
    }
    return {
      status: "stale",
      state: migratedLegacyState(identity, legacyFacts),
      savedDiagnosis,
      commentary,
    };
  }
  if (root.schema_version !== "rsi-diagnosis/v2") {
    throw new Error("Imported diagnosis has an unsupported schema version");
  }
  const identity = exactObject(
    root.identity,
    ["case_id", "title", "method_family"],
    "Imported identity",
  );
  const state: WorksheetState = {
    case_id: importedCaseId(identity.case_id),
    title: text(identity.title, "Imported title"),
    method_family: text(identity.method_family, "Imported method"),
    facts: root.worksheet.map(parseFact),
  };
  if (
    new Set(state.facts.map((fact) => fact.field_id)).size !== state.facts.length
  ) {
    throw new Error("Imported diagnosis contains duplicate facts");
  }
  const current = await currentDigests();
  const status =
    current.corpus === root.corpus_sha256
    && current.rules === root.rules_sha256
      ? "current"
      : "stale";
  const savedDiagnosis = parseSavedDiagnosis(
    root.diagnosis,
    status === "current",
  );
  const savedRules = parseRuleLedger(root.rules, status === "current");
  const commentary = root.commentary === null
    ? null
    : importCommentary(text(root.commentary, "Imported commentary"));
  const expectedProvenance = state.facts.map((fact) => ({
    field_id: fact.field_id,
    provenance: fact.provenance,
  }));
  if (stableJson(root.provenance) !== stableJson(expectedProvenance)) {
    throw new Error("Imported provenance ledger does not match worksheet facts");
  }
  const expectedRules = activeRuleLedger();
  if (
    status === "current"
    && stableJson(savedRules) !== stableJson(expectedRules)
  ) {
    throw new Error("Imported rule ledger does not match the active rule set");
  }
  if (stableJson(root.sources) !== stableJson(sourceLedger(state.facts))) {
    throw new Error("Imported source ledger does not match worksheet provenance");
  }
  const savedMarkdown = exportDiagnosisMarkdown({
    state,
    result: savedDiagnosis,
    commentary,
  }, savedRules);
  if (await sha256(savedMarkdown) !== root.brief_sha256) {
    throw new Error("Imported brief digest does not match saved content");
  }
  if (
    status === "current"
    && stableJson(normalizedResult(diagnose(state)))
      !== stableJson(normalizedResult(savedDiagnosis))
  ) {
    throw new Error(
      "Imported current-corpus verdict does not match deterministic evaluation",
    );
  }
  return {
    status,
    state,
    savedDiagnosis,
    commentary,
  };
}
