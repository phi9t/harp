import { describe, expect, it } from "vitest";

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
});
