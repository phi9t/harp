declare const conceptIdBrand: unique symbol;
declare const caseIdBrand: unique symbol;
declare const documentIdBrand: unique symbol;
declare const fieldIdBrand: unique symbol;
declare const ruleIdBrand: unique symbol;
declare const sourceIdBrand: unique symbol;
declare const systemIdBrand: unique symbol;
declare const wengSectionIdBrand: unique symbol;

export type ConceptId = string & { readonly [conceptIdBrand]: true };
export type CaseId = string & { readonly [caseIdBrand]: true };
export type DocumentId = string & { readonly [documentIdBrand]: true };
export type FieldId = string & { readonly [fieldIdBrand]: true };
export type RuleId = string & { readonly [ruleIdBrand]: true };
export type SourceId = string & { readonly [sourceIdBrand]: true };
export type SystemId = string & { readonly [systemIdBrand]: true };
export type WengSectionId = string & { readonly [wengSectionIdBrand]: true };

export type CoverageDepth =
  | "chapter"
  | "supporting-page"
  | "system-reading"
  | "worked-example";

export type ReaderRouteId =
  | "thesis"
  | "loop"
  | "methods"
  | "harnesses"
  | "weng"
  | "experiment"
  | "sources"
  | "agentic-eval-apply"
  | "benchmarks"
  | "evaluator-integrity"
  | "survey"
  | "verified-coevolution"
  | "agentic-engineering"
  | "crouzeix-conjecture"
  | "knowledge";

export const readerRouteIds: readonly ReaderRouteId[] = [
  "thesis",
  "loop",
  "methods",
  "harnesses",
  "weng",
  "experiment",
  "sources",
  "agentic-eval-apply",
  "benchmarks",
  "evaluator-integrity",
  "survey",
  "verified-coevolution",
  "agentic-engineering",
  "crouzeix-conjecture",
  "knowledge",
];

export type RetainedConcept = {
  concept_id: ConceptId;
  source_ids: SourceId[];
};

export type CoverageEntry = {
  concept_id: ConceptId;
  coverage_depth: CoverageDepth;
  canonical_markdown_path: string;
  section_id: string | null;
  parent_concept_id: ConceptId | null;
};

export type ReaderRoute = {
  route_id: ReaderRouteId;
  label: string;
  canonical_markdown_path: string;
};

export type CanonicalDocument = {
  concept_id: DocumentId;
  title: string;
  canonical_markdown_path: string;
  markdown_sha256: string;
  html_sha256: string;
  html: string;
  metadata: DocumentMetadata;
};

export type DocumentMetadata = {
  id: string;
  kind: string;
  status: string;
  tags: string[];
  confidence: "low" | "medium" | "high";
  mode: string | null;
  source_ids: string[];
  coverage_keys: string[];
};

export type SystemTreatment = "full" | "card";
export type PublicationState = "planned" | "published";

export type PaperRoute = {
  source_id: SourceId;
  public_url: string;
  captured_locator: string;
  reading_sequence: {
    method: string;
    algorithm: string;
    main_evaluation: string;
    ablation: string;
    limitations: string;
    appendix: string;
  };
};

export type SystemReading = {
  system_id: SystemId;
  title: string;
  source_ids: SourceId[];
  treatment: SystemTreatment;
  publication_state: PublicationState;
  canonical_markdown_path: string;
  weng_section_ids: WengSectionId[];
  paper_routes: PaperRoute[];
  diagnostic_case_path: string | null;
  related_system_ids: SystemId[];
};

export type PublishedSystemReading = SystemReading & {
  publication_state: "published";
  article: CanonicalDocument;
};

export type WengSection = {
  order: number;
  section_id: WengSectionId;
  title: string;
  public_url: string;
  captured_locator: string;
  companion_document_id: DocumentId;
  companion_section: string;
  system_ids: SystemId[];
  exercise_ids: string[];
  figure_locators: string[];
};

export type WorksheetSection =
  | "candidate"
  | "persistence"
  | "generation"
  | "evaluation"
  | "resources-authority"
  | "evidence";

type WorksheetFieldBase = {
  field_id: FieldId;
  section: WorksheetSection;
  label: string;
  required: boolean;
};

export type WorksheetField =
  | (WorksheetFieldBase & { kind: "boolean"; values: [] })
  | (WorksheetFieldBase & { kind: "text"; values: [] })
  | (WorksheetFieldBase & { kind: "enum"; values: string[] })
  | (WorksheetFieldBase & { kind: "set"; values: string[] });

export type PrimaryClassification =
  | "output-refinement"
  | "persistent-adaptation"
  | "harness-improvement"
  | "automated-ai-research"
  | "joint-harness-weight-adaptation"
  | "successor-improvement"
  | "recursive-improvement-demonstrated";

export type DiagnosticFlag =
  | "persistence-established"
  | "accepted-generation-established"
  | "matched-envelope-established"
  | "evaluator-independence-established"
  | "complete-root-tree-accounting-established"
  | "next-cycle-gain-measured"
  | "independent-reproduction-present";

export type FindingSeverity = "blocking" | "warning" | "information";

export type Predicate =
  | { kind: "bool-equals"; field_id: FieldId; value: boolean }
  | { kind: "enum-equals"; field_id: FieldId; value: string }
  | { kind: "set-contains"; field_id: FieldId; value: string }
  | { kind: "all"; predicates: Predicate[] }
  | { kind: "any"; predicates: Predicate[] }
  | { kind: "not"; predicate: Predicate };

export type RuleEffect =
  | { kind: "classification"; classification: PrimaryClassification }
  | { kind: "claim-ceiling"; classification: PrimaryClassification }
  | { kind: "flag"; flag: DiagnosticFlag }
  | {
      kind: "finding";
      severity: FindingSeverity;
      explanation: string;
      canonical_concept_id: ConceptId;
      evidence_needed: string;
    };

export type DiagnosticRule = {
  rule_id: RuleId;
  version: number;
  precedence: number;
  predicate: Predicate;
  effect: RuleEffect;
};

export type FactProvenance =
  | { kind: "source-backed"; source_id: SourceId; locator: string }
  | { kind: "reader-assertion" };

export type CaseFact = {
  field_id: FieldId;
  value: boolean | string | string[];
  provenance: FactProvenance;
};

export type DiagnosticCase = {
  schema_version: 1;
  case_id: CaseId;
  system_id: SystemId | null;
  title: string;
  method_family: string;
  expected_classification: PrimaryClassification;
  expected_claim_ceiling: PrimaryClassification;
  facts: CaseFact[];
};

export type DiagnosticContracts = {
  worksheet: {
    schema_version: 1;
    fields: WorksheetField[];
  };
  rules: {
    schema_version: 1;
    classifications: PrimaryClassification[];
    rules: DiagnosticRule[];
  };
  cases: DiagnosticCase[];
  export_schema: {
    schema_version: 1;
    export_id: "rsi-diagnosis/v2";
    required_sections: string[];
    max_import_bytes: number;
    max_commentary_bytes: number;
    digest_algorithm: "sha256";
    commentary_affects_diagnosis: false;
  };
};

export type Lesson = {
  order: number;
  lesson_id: string;
  title: string;
  canonical_markdown_path: string;
  case_ids: CaseId[];
  system_ids: SystemId[];
  prompt_ids: string[];
  concept_ids: ConceptId[];
  transfer_case_ids: CaseId[];
};

export type CanonicalCorpus = {
  schema_version: "rsi-technical-atlas/v5";
  retained_concepts: RetainedConcept[];
  coverage: CoverageEntry[];
  reader_routes: ReaderRoute[];
  documents: CanonicalDocument[];
  systems: SystemReading[];
  weng_sections: WengSection[];
  diagnostics: DiagnosticContracts;
  lessons: Lesson[];
};
