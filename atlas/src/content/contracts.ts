import {
  readerRouteIds,
  type CaseId,
  type CanonicalCorpus,
  type ConceptId,
  type CoverageDepth,
  type DocumentId,
  type PaperRoute,
  type PublicationState,
  type ReaderRouteId,
  type SourceId,
  type SystemId,
  type SystemTreatment,
  type WengSectionId,
} from "./types";
import { parseDiagnostics } from "../diagnostics/contracts";

const digestPattern = /^[a-f0-9]{64}$/;
const idPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

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

function stringValue(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() !== value || value.length === 0) {
    throw new Error(`${label} must be a nonempty canonical string`);
  }
  return value;
}

function nullableString(value: unknown, label: string): string | null {
  if (value === null) {
    return null;
  }
  return stringValue(value, label);
}

function opaqueText(value: unknown, label: string): string {
  if (
    typeof value !== "string"
    || value.length === 0
    || value.includes("\0")
  ) {
    throw new Error(`${label} must be nonempty NUL-free text`);
  }
  return value;
}

function stringArray(value: unknown, label: string): string[] {
  if (!Array.isArray(value)) {
    throw new Error(`${label} must be an array`);
  }
  return value.map((item, index) => stringValue(item, `${label}[${index}]`));
}

function assertRouteId(value: string): asserts value is ReaderRouteId {
  if (!readerRouteIds.some((routeId) => routeId === value)) {
    throw new Error(`Unknown reader route ${value}`);
  }
}

function assertId(value: string, label: string): void {
  if (!idPattern.test(value)) {
    throw new Error(`${label} must be a canonical ID`);
  }
}

function conceptId(value: unknown, label: string): ConceptId {
  const parsed = stringValue(value, label);
  assertId(parsed, label);
  function assertBrand(id: string): asserts id is ConceptId {
    assertId(id, label);
  }
  assertBrand(parsed);
  return parsed;
}

function caseId(value: unknown, label: string): CaseId {
  const parsed = stringValue(value, label);
  assertId(parsed, label);
  function assertBrand(id: string): asserts id is CaseId {
    assertId(id, label);
  }
  assertBrand(parsed);
  return parsed;
}

function documentId(value: unknown, label: string): DocumentId {
  const parsed = stringValue(value, label);
  assertId(parsed, label);
  function assertBrand(id: string): asserts id is DocumentId {
    assertId(id, label);
  }
  assertBrand(parsed);
  return parsed;
}

function sourceId(value: unknown, label: string): SourceId {
  const parsed = stringValue(value, label);
  if (!/^[A-Z0-9]+(?:-[A-Z0-9]+)*$/.test(parsed)) {
    throw new Error(`${label} must be a canonical source ID`);
  }
  function assertBrand(id: string): asserts id is SourceId {
    if (!/^[A-Z0-9]+(?:-[A-Z0-9]+)*$/.test(id)) {
      throw new Error(`${label} must be a canonical source ID`);
    }
  }
  assertBrand(parsed);
  return parsed;
}

function systemId(value: unknown, label: string): SystemId {
  const parsed = stringValue(value, label);
  assertId(parsed, label);
  function assertBrand(id: string): asserts id is SystemId {
    assertId(id, label);
  }
  assertBrand(parsed);
  return parsed;
}

function wengSectionId(value: unknown, label: string): WengSectionId {
  const parsed = stringValue(value, label);
  assertId(parsed, label);
  function assertBrand(id: string): asserts id is WengSectionId {
    assertId(id, label);
  }
  assertBrand(parsed);
  return parsed;
}

function coverageDepth(value: unknown): CoverageDepth {
  if (
    value !== "chapter"
    && value !== "supporting-page"
    && value !== "system-reading"
    && value !== "worked-example"
  ) {
    throw new Error("Coverage entry has an invalid depth");
  }
  return value;
}

function systemTreatment(value: unknown): SystemTreatment {
  if (value !== "full" && value !== "card") {
    throw new Error("System has an invalid treatment");
  }
  return value;
}

function publicationState(value: unknown): PublicationState {
  if (value !== "planned" && value !== "published") {
    throw new Error("System has an invalid publication state");
  }
  return value;
}

function httpsUrl(value: unknown, label: string): string {
  const parsed = stringValue(value, label);
  const url = new URL(parsed);
  if (url.protocol !== "https:" || url.username !== "" || url.password !== "") {
    throw new Error(`${label} must be a credential-free HTTPS URL`);
  }
  return parsed;
}

function unique<T>(values: T[], label: string): T[] {
  if (new Set(values).size !== values.length) {
    throw new Error(`${label} contains duplicates`);
  }
  return values;
}

function parsePaperRoute(value: unknown, index: number): PaperRoute {
  const route = exactObject(
    value,
    ["source_id", "public_url", "captured_locator", "reading_sequence"],
    `Paper route ${index}`,
  );
  const sequence = exactObject(
    route.reading_sequence,
    [
      "method",
      "algorithm",
      "main_evaluation",
      "ablation",
      "limitations",
      "appendix",
    ],
    `Paper route ${index} reading sequence`,
  );
  return {
    source_id: sourceId(route.source_id, `Paper route ${index} source`),
    public_url: httpsUrl(route.public_url, `Paper route ${index} URL`),
    captured_locator: stringValue(
      route.captured_locator,
      `Paper route ${index} capture`,
    ),
    reading_sequence: {
      method: stringValue(sequence.method, `Paper route ${index} method`),
      algorithm: stringValue(
        sequence.algorithm,
        `Paper route ${index} algorithm`,
      ),
      main_evaluation: stringValue(
        sequence.main_evaluation,
        `Paper route ${index} main evaluation`,
      ),
      ablation: stringValue(
        sequence.ablation,
        `Paper route ${index} ablation`,
      ),
      limitations: stringValue(
        sequence.limitations,
        `Paper route ${index} limitations`,
      ),
      appendix: stringValue(
        sequence.appendix,
        `Paper route ${index} appendix`,
      ),
    },
  };
}

export function parseCorpus(value: unknown): CanonicalCorpus {
  const corpus = exactObject(
    value,
    [
      "schema_version",
      "retained_concepts",
      "coverage",
      "reader_routes",
      "documents",
      "systems",
      "weng_sections",
      "diagnostics",
      "lessons",
    ],
    "Canonical RSI corpus",
  );
  if (corpus.schema_version !== "rsi-technical-atlas/v5") {
    throw new Error("Canonical RSI corpus has an unsupported schema");
  }
  if (
    !Array.isArray(corpus.retained_concepts)
    || !Array.isArray(corpus.coverage)
    || !Array.isArray(corpus.reader_routes)
    || !Array.isArray(corpus.documents)
    || !Array.isArray(corpus.systems)
    || !Array.isArray(corpus.weng_sections)
    || !Array.isArray(corpus.lessons)
  ) {
    throw new Error("Canonical RSI corpus collections must be arrays");
  }
  const diagnostics = parseDiagnostics(corpus.diagnostics);

  const retainedConcepts = corpus.retained_concepts.map((value, index) => {
    const concept = exactObject(
      value,
      ["concept_id", "source_ids"],
      `Retained concept ${index}`,
    );
    return {
      concept_id: conceptId(concept.concept_id, `Retained concept ${index} ID`),
      source_ids: unique(
        stringArray(concept.source_ids, `Retained concept ${index} sources`)
          .map((source, sourceIndex) =>
            sourceId(source, `Retained concept ${index} source ${sourceIndex}`)),
        `Retained concept ${index} sources`,
      ),
    };
  });

  const coverage = corpus.coverage.map((value, index) => {
    const entry = exactObject(
      value,
      [
        "concept_id",
        "coverage_depth",
        "canonical_markdown_path",
        "section_id",
        "parent_concept_id",
      ],
      `Coverage entry ${index}`,
    );
    const parent = nullableString(
      entry.parent_concept_id,
      `Coverage entry ${index} parent`,
    );
    return {
      concept_id: conceptId(entry.concept_id, `Coverage entry ${index} ID`),
      coverage_depth: coverageDepth(entry.coverage_depth),
      canonical_markdown_path: stringValue(
        entry.canonical_markdown_path,
        `Coverage entry ${index} path`,
      ),
      section_id: nullableString(
        entry.section_id,
        `Coverage entry ${index} section`,
      ),
      parent_concept_id:
        parent === null
          ? null
          : conceptId(parent, `Coverage entry ${index} parent`),
    };
  });

  const readerRoutes = corpus.reader_routes.map((value, index) => {
    const route = exactObject(
      value,
      ["route_id", "label", "canonical_markdown_path"],
      `Reader route ${index}`,
    );
    const routeId = stringValue(route.route_id, `Reader route ${index} ID`);
    assertRouteId(routeId);
    return {
      route_id: routeId,
      label: stringValue(route.label, `Reader route ${index} label`),
      canonical_markdown_path: stringValue(
        route.canonical_markdown_path,
        `Reader route ${index} path`,
      ),
    };
  });

  const documents = corpus.documents.map((value, index) => {
    const document = exactObject(
      value,
      [
        "concept_id",
        "title",
        "canonical_markdown_path",
        "markdown_sha256",
        "html_sha256",
        "html",
      ],
      `Document ${index}`,
    );
    const markdownDigest = stringValue(
      document.markdown_sha256,
      `Document ${index} Markdown digest`,
    );
    const htmlDigest = stringValue(
      document.html_sha256,
      `Document ${index} HTML digest`,
    );
    if (!digestPattern.test(markdownDigest) || !digestPattern.test(htmlDigest)) {
      throw new Error(`Document ${index} has an invalid digest`);
    }
    return {
      concept_id: documentId(document.concept_id, `Document ${index} ID`),
      title: stringValue(document.title, `Document ${index} title`),
      canonical_markdown_path: stringValue(
        document.canonical_markdown_path,
        `Document ${index} path`,
      ),
      markdown_sha256: markdownDigest,
      html_sha256: htmlDigest,
      html: opaqueText(document.html, `Document ${index} HTML`),
    };
  });

  const systems = corpus.systems.map((value, index) => {
    const system = exactObject(
      value,
      [
        "system_id",
        "title",
        "source_ids",
        "treatment",
        "publication_state",
        "canonical_markdown_path",
        "weng_section_ids",
        "paper_routes",
        "diagnostic_case_path",
        "related_system_ids",
      ],
      `System ${index}`,
    );
    if (!Array.isArray(system.paper_routes)) {
      throw new Error(`System ${index} paper routes must be an array`);
    }
    return {
      system_id: systemId(system.system_id, `System ${index} ID`),
      title: stringValue(system.title, `System ${index} title`),
      source_ids: unique(
        stringArray(system.source_ids, `System ${index} sources`).map(
          (source, sourceIndex) =>
            sourceId(source, `System ${index} source ${sourceIndex}`),
        ),
        `System ${index} sources`,
      ),
      treatment: systemTreatment(system.treatment),
      publication_state: publicationState(system.publication_state),
      canonical_markdown_path: stringValue(
        system.canonical_markdown_path,
        `System ${index} path`,
      ),
      weng_section_ids: unique(
        stringArray(
          system.weng_section_ids,
          `System ${index} Weng sections`,
        ).map((section, sectionIndex) =>
          wengSectionId(section, `System ${index} Weng section ${sectionIndex}`)),
        `System ${index} Weng sections`,
      ),
      paper_routes: system.paper_routes.map(parsePaperRoute),
      diagnostic_case_path: nullableString(
        system.diagnostic_case_path,
        `System ${index} case`,
      ),
      related_system_ids: unique(
        stringArray(
          system.related_system_ids,
          `System ${index} related systems`,
        ).map((related, relatedIndex) =>
          systemId(related, `System ${index} related system ${relatedIndex}`)),
        `System ${index} related systems`,
      ),
    };
  });

  const wengSections = corpus.weng_sections.map((value, index) => {
    const section = exactObject(
      value,
      [
        "order",
        "section_id",
        "title",
        "public_url",
        "captured_locator",
        "companion_document_id",
        "companion_section",
        "system_ids",
        "exercise_ids",
        "figure_locators",
      ],
      `Weng section ${index}`,
    );
    if (!Number.isInteger(section.order) || section.order !== index + 1) {
      throw new Error("Weng sections must have contiguous order");
    }
    return {
      order: section.order,
      section_id: wengSectionId(
        section.section_id,
        `Weng section ${index} ID`,
      ),
      title: stringValue(section.title, `Weng section ${index} title`),
      public_url: httpsUrl(section.public_url, `Weng section ${index} URL`),
      captured_locator: stringValue(
        section.captured_locator,
        `Weng section ${index} capture`,
      ),
      companion_document_id: documentId(
        section.companion_document_id,
        `Weng section ${index} companion`,
      ),
      companion_section: stringValue(
        section.companion_section,
        `Weng section ${index} companion section`,
      ),
      system_ids: unique(
        stringArray(section.system_ids, `Weng section ${index} systems`).map(
          (system, systemIndex) =>
            systemId(system, `Weng section ${index} system ${systemIndex}`),
        ),
        `Weng section ${index} systems`,
      ),
      exercise_ids: unique(
        stringArray(
          section.exercise_ids,
          `Weng section ${index} exercises`,
        ),
        `Weng section ${index} exercises`,
      ),
      figure_locators: unique(
        stringArray(
          section.figure_locators,
          `Weng section ${index} figures`,
        ),
        `Weng section ${index} figures`,
      ),
    };
  });
  const lessons = corpus.lessons.map((value, index) => {
    const lesson = exactObject(
      value,
      [
        "order",
        "lesson_id",
        "title",
        "canonical_markdown_path",
        "case_ids",
        "system_ids",
        "prompt_ids",
        "concept_ids",
        "transfer_case_ids",
      ],
      `Lesson ${index}`,
    );
    if (!Number.isInteger(lesson.order) || lesson.order !== index + 1) {
      throw new Error("Lessons must have contiguous order");
    }
    return {
      order: lesson.order,
      lesson_id: stringValue(lesson.lesson_id, `Lesson ${index} ID`),
      title: stringValue(lesson.title, `Lesson ${index} title`),
      canonical_markdown_path: stringValue(
        lesson.canonical_markdown_path,
        `Lesson ${index} path`,
      ),
      case_ids: unique(
        stringArray(lesson.case_ids, `Lesson ${index} cases`).map(
          (item, caseIndex) =>
            caseId(item, `Lesson ${index} case ${caseIndex}`),
        ),
        `Lesson ${index} cases`,
      ),
      system_ids: unique(
        stringArray(lesson.system_ids, `Lesson ${index} systems`).map(
          (item, systemIndex) =>
            systemId(item, `Lesson ${index} system ${systemIndex}`),
        ),
        `Lesson ${index} systems`,
      ),
      prompt_ids: unique(
        stringArray(lesson.prompt_ids, `Lesson ${index} prompts`),
        `Lesson ${index} prompts`,
      ),
      concept_ids: unique(
        stringArray(lesson.concept_ids, `Lesson ${index} concepts`).map(
          (item, conceptIndex) =>
            conceptId(item, `Lesson ${index} concept ${conceptIndex}`),
        ),
        `Lesson ${index} concepts`,
      ),
      transfer_case_ids: unique(
        stringArray(
          lesson.transfer_case_ids,
          `Lesson ${index} transfers`,
        ).map((item, caseIndex) =>
          caseId(item, `Lesson ${index} transfer ${caseIndex}`)),
        `Lesson ${index} transfers`,
      ),
    };
  });

  const conceptIds = unique(
    retainedConcepts.map((concept) => concept.concept_id),
    "Retained concept IDs",
  );
  const coverageIds = unique(
    coverage.map((entry) => entry.concept_id),
    "Coverage IDs",
  );
  if (
    conceptIds.length !== coverageIds.length
    || conceptIds.some((id) => !coverageIds.includes(id))
  ) {
    throw new Error("Retained concepts and coverage differ");
  }
  const chapterPaths = coverage
    .filter((entry) => entry.coverage_depth === "chapter")
    .map((entry) => entry.canonical_markdown_path);
  if (chapterPaths.length !== 9 || new Set(chapterPaths).size !== 9) {
    throw new Error("Canonical RSI corpus must contain nine unique chapters");
  }

  unique(documents.map((document) => document.concept_id), "Document IDs");
  unique(
    documents.map((document) => document.canonical_markdown_path),
    "Document paths",
  );
  for (const path of chapterPaths) {
    if (!documents.some((document) => document.canonical_markdown_path === path)) {
      throw new Error("Canonical RSI corpus references a missing chapter");
    }
  }
  if (
    readerRoutes.length !== readerRouteIds.length
    || new Set(readerRoutes.map((route) => route.route_id)).size
      !== readerRouteIds.length
  ) {
    throw new Error("Canonical RSI corpus must contain seven reader routes");
  }
  for (const routeId of readerRouteIds) {
    const route = readerRoutes.find((candidate) => candidate.route_id === routeId);
    if (
      !route
      || !documents.some(
        (document) =>
          document.canonical_markdown_path === route.canonical_markdown_path,
      )
    ) {
      throw new Error(`Canonical RSI corpus is missing route ${routeId}`);
    }
  }

  const knownSystems = new Set(
    unique(systems.map((system) => system.system_id), "System IDs"),
  );
  const knownCases = new Set(
    diagnostics.cases.map((diagnosticCase) => diagnosticCase.case_id),
  );
  if (systems.length !== 16) {
    throw new Error("Canonical RSI corpus must contain sixteen system identities");
  }
  if (wengSections.length !== 9) {
    throw new Error("Canonical RSI corpus must contain nine Weng sections");
  }
  const knownSections = new Set(
    unique(
      wengSections.map((section) => section.section_id),
      "Weng section IDs",
    ),
  );
  for (const section of wengSections) {
    if (
      !documents.some(
        (document) => document.concept_id === section.companion_document_id,
      )
    ) {
      throw new Error(`Weng section ${section.section_id} has no companion`);
    }
    for (const system of section.system_ids) {
      if (!knownSystems.has(system)) {
        throw new Error(`Weng section ${section.section_id} has unknown system`);
      }
    }
  }
  for (const system of systems) {
    for (const section of system.weng_section_ids) {
      const owner = wengSections.find((candidate) => candidate.section_id === section);
      if (!knownSections.has(section) || !owner?.system_ids.includes(system.system_id)) {
        throw new Error(`System ${system.system_id} has unknown Weng section`);
      }
    }
    for (const related of system.related_system_ids) {
      if (!knownSystems.has(related) || related === system.system_id) {
        throw new Error(`System ${system.system_id} has unknown related system`);
      }
    }
    const articleExists = documents.some(
      (document) =>
        document.canonical_markdown_path === system.canonical_markdown_path,
    );
    if (system.publication_state === "planned" && articleExists) {
      throw new Error(`Planned system ${system.system_id} has a published article`);
    }
    if (system.publication_state === "published" && !articleExists) {
      throw new Error(`Published system ${system.system_id} has no article`);
    }
  }
  for (const lesson of lessons) {
    if (
      !documents.some(
        (document) =>
          document.canonical_markdown_path === lesson.canonical_markdown_path,
      )
      || lesson.case_ids.some((item) => !knownCases.has(item))
      || lesson.transfer_case_ids.some((item) => !knownCases.has(item))
      || lesson.system_ids.some((item) => !knownSystems.has(item))
      || lesson.concept_ids.some((item) => !conceptIds.includes(item))
    ) {
      throw new Error(`Lesson ${lesson.lesson_id} has invalid references`);
    }
  }

  return {
    schema_version: corpus.schema_version,
    retained_concepts: retainedConcepts,
    coverage,
    reader_routes: readerRoutes,
    documents,
    systems,
    weng_sections: wengSections,
    diagnostics,
    lessons,
  };
}

export function parseCorpusForTest(value: unknown): CanonicalCorpus {
  return parseCorpus(value);
}
