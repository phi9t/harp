import { describe, expect, it } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { formatRoute, parseRoute } from "./routes";

const firstSectionId = canonicalCorpus.weng_sections[0].section_id;
const aflowId = canonicalCorpus.systems.find(
  (system) => system.system_id === "aflow",
)?.system_id;
const contextDocumentId = canonicalCorpus.documents.find(
  (document) => document.concept_id === "context-engineering-deep-dive",
)?.concept_id;

if (!aflowId || !contextDocumentId) {
  throw new Error("Validated route fixtures disappeared");
}

describe("Atlas routes", () => {
  it("defaults to the first Weng section", () => {
    expect(parseRoute("")).toEqual({
      kind: "weng",
      sectionId: firstSectionId,
    });
  });

  it("formats system and auxiliary document routes", () => {
    expect(formatRoute({
      kind: "system",
      systemId: aflowId,
      returnTo: null,
    }))
      .toBe("#systems/aflow");
    expect(formatRoute({
      kind: "document",
      documentId: contextDocumentId,
    })).toBe("#documents/context-engineering-deep-dive");
  });

  it("falls back when a route references an unknown identity", () => {
    expect(parseRoute("#systems/absent")).toEqual({
      kind: "weng",
      sectionId: firstSectionId,
    });
    expect(parseRoute("#diagnose/absent")).toEqual({
      kind: "weng",
      sectionId: firstSectionId,
    });
  });
});
