import { describe, expect, it } from "vitest";

import corpusData from "./generated/corpus.json";
import { parseCorpusForTest } from "./contracts";

const digest = "a".repeat(64);

const chapterIds: readonly string[] = [
  "recursive-improvement-loop",
  "foundation-model-inside-the-loop",
  "harness-engineering",
  "durable-improvement-workflows",
  "procedure-internalization",
  "harness-search",
  "automated-research",
  "joint-harness-weight-adaptation",
  "evaluation-promotion-containment",
];

const wengSectionIds: readonly string[] = [
  "system-being-improved",
  "harness-design-patterns",
  "harness-layer-vs-core-intelligence",
  "context-engineering",
  "workflow-design-and-search",
  "self-improving-harnesses",
  "evolutionary-search",
  "joint-harness-weight-optimization",
  "future-challenges",
];

const chapterPath = (id: string): string =>
  `knowledge/rsi/chapters/${id}.md`;

const companionId = (id: string): string => `weng-${id}`;

const systemIds: readonly string[] = [
  "aflow",
  "system-02",
  "system-03",
  "system-04",
  "system-05",
  "system-06",
  "system-07",
  "system-08",
  "system-09",
  "system-10",
  "system-11",
  "system-12",
  "system-13",
  "system-14",
  "system-15",
  "system-16",
];

const systemSection = (index: number): string =>
  wengSectionIds[index % wengSectionIds.length];

const validV5Fixture: unknown = {
  schema_version: "rsi-technical-atlas/v5",
  retained_concepts: chapterIds.map((id) => ({
    concept_id: id,
    source_ids: [],
  })),
  coverage: chapterIds.map((id) => ({
    concept_id: id,
    coverage_depth: "chapter",
    canonical_markdown_path: chapterPath(id),
    section_id: null,
    parent_concept_id: null,
  })),
  reader_routes: [
    {
      route_id: "thesis",
      label: "Thesis",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "loop",
      label: "Loop",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "methods",
      label: "Methods",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "harnesses",
      label: "Harnesses",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "weng",
      label: "Weng",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "experiment",
      label: "Experiment",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "sources",
      label: "Sources",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "agentic-eval-apply",
      label: "Agentic eval/apply",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "benchmarks",
      label: "Benchmarks",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "evaluator-integrity",
      label: "Evaluator integrity",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "survey",
      label: "Survey",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "verified-coevolution",
      label: "Verified coevolution",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "agentic-engineering",
      label: "Agentic engineering",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
    {
      route_id: "crouzeix-conjecture",
      label: "Crouzeix",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
  ],
  documents: [
    ...chapterIds.map((id) => ({
      concept_id: id,
      title: id,
      canonical_markdown_path: chapterPath(id),
      markdown_sha256: digest,
      html_sha256: digest,
      html: `<h1>${id}</h1>`,
    })),
    ...wengSectionIds.map((id, index) => ({
      concept_id: companionId(id),
      title: id,
      canonical_markdown_path:
        `knowledge/rsi/weng/${String(index + 1).padStart(2, "0")}-${id}.md`,
      markdown_sha256: digest,
      html_sha256: digest,
      html: `<h1>${id}</h1>`,
    })),
  ],
  systems: systemIds.map((id, index) => {
    const source = index === 0 ? "AFLOW" : `SYSTEM-${String(index + 1).padStart(2, "0")}`;
    return {
      system_id: id,
      title: id,
      source_ids: [source],
      treatment: "full",
      publication_state: "planned",
      canonical_markdown_path: `knowledge/rsi/systems/${id}.md`,
      weng_section_ids: [systemSection(index)],
      paper_routes: [
        {
          source_id: source,
          public_url: `https://example.com/${id}`,
          captured_locator: `evidence/example/${id}.txt:1-40`,
          reading_sequence: {
            method: "§2",
            algorithm: "§3, Algorithm 1",
            main_evaluation: "§4, Results",
            ablation: "§4, Table 1",
            limitations: "§5, Limitations",
            appendix: "Appendix A",
          },
        },
      ],
      diagnostic_case_path: null,
      related_system_ids: [],
    };
  }),
  weng_sections: wengSectionIds.map((id, index) => ({
      order: index + 1,
      section_id: id,
      title: id,
      public_url: `https://example.com/article#${id}`,
      captured_locator: `evidence/example.html#${id}`,
      companion_document_id: companionId(id),
      companion_section: "mechanism",
      system_ids: systemIds.filter(
        (_system, systemIndex) => systemSection(systemIndex) === id,
      ),
      exercise_ids: [`${id}-checkpoint`],
      figure_locators: [],
  })),
  diagnostics: corpusData.diagnostics,
  lessons: corpusData.lessons,
};

describe("canonical corpus boundary", () => {
  it("rejects a Weng section with an unknown system", () => {
    const source = structuredClone(validV5Fixture);
    if (
      typeof source !== "object"
      || source === null
      || !("weng_sections" in source)
      || !Array.isArray(source.weng_sections)
      || typeof source.weng_sections[0] !== "object"
      || source.weng_sections[0] === null
    ) {
      throw new Error("Test fixture has an invalid Weng section");
    }
    source.weng_sections[0].system_ids = ["absent-system"];

    expect(() => parseCorpusForTest(source)).toThrow(/unknown system/);
  });
});
