import { describe, expect, it } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { renderCanonicalMath } from "./math";

describe("canonical math", () => {
  it("renders escaped TeX as MathML without external resources", () => {
    const root = document.createElement("div");
    root.innerHTML =
      '<span class="math math-inline" data-tex="\\|T\\|\\le2">\\|T\\|\\le2</span>';

    renderCanonicalMath(root);

    expect(root.querySelector("math")).not.toBeNull();
    expect(root.querySelector("[src], link, style")).toBeNull();
  });

  it("renders display math as a MathML block", () => {
    const root = document.createElement("div");
    root.innerHTML =
      '<span class="math math-display" data-tex="T^n">T^n</span>';

    renderCanonicalMath(root);

    expect(root.querySelector("math")?.getAttribute("display")).toBe("block");
  });

  it("keeps invalid TeX visible and marks the element", () => {
    const root = document.createElement("div");
    root.innerHTML =
      '<span class="math math-inline" data-tex="\\\\notacommand{">original</span>';

    renderCanonicalMath(root);

    const math = root.querySelector(".math");
    expect(math).toHaveClass("math-error");
    expect(math).toHaveTextContent("\\notacommand{");
    expect(root.querySelector("math")).toBeNull();
  });

  it("renders every registered Crouzeix formula without fallback", () => {
    let inlineCount = 0;
    let displayCount = 0;

    for (const document of canonicalCorpus.documents.filter((candidate) =>
      candidate.canonical_markdown_path.startsWith(
        "knowledge/crouzeix_conjecture/",
      )
    )) {
      const root = window.document.createElement("div");
      root.innerHTML = document.html;
      inlineCount += root.querySelectorAll(".math-inline[data-tex]").length;
      displayCount += root.querySelectorAll(".math-display[data-tex]").length;

      renderCanonicalMath(root);

      expect(
        root.querySelectorAll(".math-error"),
        document.canonical_markdown_path,
      ).toHaveLength(0);
      expect(
        root.querySelectorAll(".math[data-tex]").length,
        document.canonical_markdown_path,
      ).toBe(root.querySelectorAll("math").length);
    }

    expect(inlineCount).toBeGreaterThan(0);
    expect(displayCount).toBeGreaterThan(0);
  });

  it("renders every registered mathematical foundations formula without fallback", () => {
    const documents = canonicalCorpus.documents.filter((candidate) =>
      candidate.canonical_markdown_path.startsWith(
        "knowledge/mathematical_foundations/",
      )
    );
    let inlineCount = 0;
    let displayCount = 0;

    expect(documents).toHaveLength(12);
    for (const document of documents) {
      const root = window.document.createElement("div");
      root.innerHTML = document.html;
      inlineCount += root.querySelectorAll(".math-inline[data-tex]").length;
      displayCount += root.querySelectorAll(".math-display[data-tex]").length;

      renderCanonicalMath(root);

      expect(
        root.querySelectorAll(".math-error"),
        document.canonical_markdown_path,
      ).toHaveLength(0);
      expect(
        root.querySelectorAll(".math[data-tex]").length,
        document.canonical_markdown_path,
      ).toBe(root.querySelectorAll("math").length);
    }

    expect(inlineCount).toBeGreaterThan(0);
    expect(displayCount).toBeGreaterThan(0);
  });

  it("renders every registered Crouzeix textbook formula without fallback", () => {
    const documents = canonicalCorpus.documents.filter((candidate) =>
      candidate.canonical_markdown_path.startsWith(
        "knowledge/crouzeix_textbook/",
      )
    );
    let inlineCount = 0;
    let displayCount = 0;

    expect(documents).toHaveLength(44);
    for (const document of documents) {
      const root = window.document.createElement("div");
      root.innerHTML = document.html;
      inlineCount += root.querySelectorAll(".math-inline[data-tex]").length;
      displayCount += root.querySelectorAll(".math-display[data-tex]").length;

      renderCanonicalMath(root);

      expect(
        root.querySelectorAll(".math-error"),
        document.canonical_markdown_path,
      ).toHaveLength(0);
      expect(
        root.querySelectorAll(".math[data-tex]").length,
        document.canonical_markdown_path,
      ).toBe(root.querySelectorAll("math").length);
    }

    expect(inlineCount).toBeGreaterThan(0);
    expect(displayCount).toBeGreaterThan(0);
  });

  it("keeps theorem anchors addressable beside the textbook Lean route", () => {
    const chapter = canonicalCorpus.documents.find(
      (document) =>
        document.canonical_markdown_path
        === "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md",
    );
    const root = document.createElement("div");
    root.innerHTML = chapter?.html ?? "";

    expect(chapter?.metadata.id).toBe(chapter?.concept_id);
    expect(root.querySelector("#cft-01-006")).not.toBeNull();
    expect(
      root.querySelector(
        'a[href="#documents/cft-lean-coverage-ledger"]',
      ),
    ).not.toBeNull();
  });
});
