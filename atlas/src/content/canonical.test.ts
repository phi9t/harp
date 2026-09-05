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
const mathematicalFoundationsIndexPath =
  "knowledge/mathematical_foundations/mathematical_foundations_index.md";
const autodiffGeometryIndexPath =
  "knowledge/autodiff_geometry/autodiff_geometry_index.md";
const crouzeixTextbookIndexPath =
  "knowledge/crouzeix_textbook/crouzeix_textbook_index.md";

const companionId = (id: string): string => `weng-${id}`;
const documentMetadata = (id: string) => ({
  id,
  kind: "technical-deep-dive",
  status: "active",
  tags: ["test"],
  confidence: "high",
  mode: null,
  source_ids: [],
  coverage_keys: [],
});

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

const validV6Fixture: unknown = {
  schema_version: "rsi-technical-atlas/v6",
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
    {
      route_id: "crouzeix-textbook",
      label: "Crouzeix textbook",
      canonical_markdown_path: crouzeixTextbookIndexPath,
    },
    {
      route_id: "mathematical-foundations",
      label: "Math foundations",
      canonical_markdown_path: mathematicalFoundationsIndexPath,
    },
    {
      route_id: "autodiff-geometry",
      label: "Autodiff geometry",
      canonical_markdown_path: autodiffGeometryIndexPath,
    },
    {
      route_id: "knowledge",
      label: "Knowledge",
      canonical_markdown_path: chapterPath(chapterIds[0]),
    },
  ],
  document_aliases: [],
  documents: [
    ...chapterIds.map((id) => ({
      concept_id: id,
      title: id,
      canonical_markdown_path: chapterPath(id),
      markdown_sha256: digest,
      html_sha256: digest,
      html: `<h1>${id}</h1>`,
      metadata: documentMetadata(id),
    })),
    ...wengSectionIds.map((id, index) => ({
      concept_id: companionId(id),
      title: id,
      canonical_markdown_path:
        `knowledge/rsi/weng/${String(index + 1).padStart(2, "0")}-${id}.md`,
      markdown_sha256: digest,
      html_sha256: digest,
      html: `<h1>${id}</h1>`,
      metadata: documentMetadata(companionId(id)),
    })),
    {
      concept_id: "math-foundations-index",
      title: "Math foundations",
      canonical_markdown_path: mathematicalFoundationsIndexPath,
      markdown_sha256: digest,
      html_sha256: digest,
      html: "<h1>Math foundations</h1>",
      metadata: documentMetadata("math-foundations-index"),
    },
    {
      concept_id: "autodiff-geometry-index",
      title: "Autodiff geometry",
      canonical_markdown_path: autodiffGeometryIndexPath,
      markdown_sha256: digest,
      html_sha256: digest,
      html: "<h1>Autodiff geometry</h1>",
      metadata: documentMetadata("autodiff-geometry-index"),
    },
    {
      concept_id: "crouzeix-textbook-index",
      title: "Crouzeix textbook",
      canonical_markdown_path: crouzeixTextbookIndexPath,
      markdown_sha256: digest,
      html_sha256: digest,
      html: "<h1>Crouzeix textbook</h1>",
      metadata: documentMetadata("crouzeix-textbook-index"),
    },
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
  it("exports the complete forty-four-document Crouzeix textbook packet", () => {
    const corpus = parseCorpusForTest(corpusData);
    expect(corpus.reader_routes).toContainEqual({
      route_id: "crouzeix-textbook",
      label: "Crouzeix textbook",
      canonical_markdown_path: crouzeixTextbookIndexPath,
    });
    expect(
      corpus.documents.filter((document) =>
        document.canonical_markdown_path.startsWith(
          "knowledge/crouzeix_textbook/",
        ),
      ),
    ).toHaveLength(44);
  });

  it("exports canonical textbook identities, theorem anchors, and Lean routes", () => {
    const corpus = parseCorpusForTest(corpusData);
    const chapter = corpus.documents.find(
      (document) =>
        document.canonical_markdown_path
        === "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md",
    );

    expect(chapter?.concept_id).toBe(
      "cft-chapter-01-objects-and-representations",
    );
    expect(chapter?.metadata.id).toBe(chapter?.concept_id);
    expect(chapter?.html).toContain('<h3 id="cft-01-001">');
    expect(chapter?.html).toContain(
      'href="#documents/cft-lean-coverage-ledger"',
    );
  });

  it("exports one canonical document target for every textbook legacy alias", () => {
    const corpus = parseCorpusForTest(corpusData);
    const aliases = corpus.document_aliases.filter((alias) =>
      alias.alias_id.startsWith("crouzeix-textbook-")
    );

    expect(aliases).toHaveLength(44);
    expect(new Set(aliases.map((alias) => alias.alias_id)).size).toBe(44);
    for (const alias of aliases) {
      expect(
        corpus.documents.filter(
          (document) => document.concept_id === alias.canonical_document_id,
        ),
      ).toHaveLength(1);
    }
  });

  it("rejects document aliases that collide with another canonical document", () => {
    const source = structuredClone(validV6Fixture);
    if (
      typeof source !== "object"
      || source === null
      || !("document_aliases" in source)
      || !Array.isArray(source.document_aliases)
    ) {
      throw new Error("Test fixture has invalid document aliases");
    }
    source.document_aliases = [{
      alias_id: chapterIds[1],
      canonical_document_id: chapterIds[0],
    }];

    expect(() => parseCorpusForTest(source)).toThrow(/alias.*collides/i);
  });

  it("rejects document aliases with unknown canonical targets", () => {
    const source = structuredClone(validV6Fixture);
    if (
      typeof source !== "object"
      || source === null
      || !("document_aliases" in source)
      || !Array.isArray(source.document_aliases)
    ) {
      throw new Error("Test fixture has invalid document aliases");
    }
    source.document_aliases = [{
      alias_id: "retired-document",
      canonical_document_id: "missing-document",
    }];

    expect(() => parseCorpusForTest(source)).toThrow(/unknown canonical document/i);
  });

  it("rejects stable v5 instead of reinterpreting required v6 aliases", () => {
    const source = structuredClone(validV6Fixture);
    if (
      typeof source !== "object"
      || source === null
      || !("schema_version" in source)
    ) {
      throw new Error("Test fixture is not an object");
    }
    source.schema_version = "rsi-technical-atlas/v5";

    expect(() => parseCorpusForTest(source)).toThrow(/unsupported schema/i);
  });

  it("rejects invalid document metadata confidence", () => {
    const source = structuredClone(validV6Fixture);
    if (
      typeof source !== "object"
      || source === null
      || !("documents" in source)
      || !Array.isArray(source.documents)
      || typeof source.documents[0] !== "object"
      || source.documents[0] === null
      || !("metadata" in source.documents[0])
      || typeof source.documents[0].metadata !== "object"
      || source.documents[0].metadata === null
    ) {
      throw new Error("Test fixture has invalid document metadata");
    }
    source.documents[0].metadata.confidence = "certain";

    expect(() => parseCorpusForTest(source)).toThrow(
      "Document 0 metadata confidence",
    );
  });

  it("rejects a Weng section with an unknown system", () => {
    const source = structuredClone(validV6Fixture);
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
