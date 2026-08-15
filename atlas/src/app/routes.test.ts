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
const verifiedCoevolutionRoute = canonicalCorpus.reader_routes.find(
  (route) => route.route_id === "verified-coevolution",
);

if (!aflowId || !contextDocumentId || !verifiedCoevolutionRoute) {
  throw new Error("Validated route fixtures disappeared");
}

describe("Atlas routes", () => {
  it("defaults to the first Weng section", () => {
    expect(parseRoute("")).toEqual({
      kind: "weng",
      sectionId: firstSectionId,
    });
  });

  it("parses and formats the first-class Crouzeix route", () => {
    expect(parseRoute("#crouzeix-conjecture")).toEqual({
      kind: "legacy",
      routeId: "crouzeix-conjecture",
    });
    expect(formatRoute({
      kind: "legacy",
      routeId: "crouzeix-conjecture",
    })).toBe("#crouzeix-conjecture");
  });

  it("parses and formats the verified coevolution agenda route", () => {
    expect(parseRoute("#verified-coevolution")).toEqual({
      kind: "legacy",
      routeId: "verified-coevolution",
    });
    expect(formatRoute({
      kind: "legacy",
      routeId: "verified-coevolution",
    })).toBe("#verified-coevolution");
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
      sectionId: null,
    })).toBe("#documents/context-engineering-deep-dive");
  });

  it("round trips canonical document sections", () => {
    expect(
      parseRoute(
        "#documents/context-engineering-deep-dive?section=context-pipeline",
      ),
    ).toEqual({
      kind: "document",
      documentId: contextDocumentId,
      sectionId: "context-pipeline",
    });
    expect(formatRoute({
      kind: "document",
      documentId: contextDocumentId,
      sectionId: "context-pipeline",
    })).toBe(
      "#documents/context-engineering-deep-dive?section=context-pipeline",
    );
  });

  it("keeps malformed document sections at the document root", () => {
    for (const section of ["Context%20Pipeline", "context%252Dpipeline"]) {
      expect(
        parseRoute(
          `#documents/context-engineering-deep-dive?section=${section}`,
        ),
      ).toEqual({
        kind: "document",
        documentId: contextDocumentId,
        sectionId: null,
      });
    }
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
