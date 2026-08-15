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
});
