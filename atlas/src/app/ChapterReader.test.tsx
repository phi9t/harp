import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { CanonicalDocumentView } from "./ChapterReader";

const baseDocument = canonicalCorpus.documents.find(
  (document) => document.concept_id === "context-engineering-deep-dive",
);

if (!baseDocument) {
  throw new Error("Validated chapter-reader fixture disappeared");
}

describe("canonical document view", () => {
  it("renders MathML and focuses the requested section inside its article", () => {
    const scrollIntoView = vi.fn();
    HTMLElement.prototype.scrollIntoView = scrollIntoView;
    const documentWithMath = {
      ...baseDocument,
      html: [
        "<h1>Addressable math</h1>",
        '<h2 id="bound">Bound</h2>',
        '<span class="math math-inline" data-tex="T^n">T^n</span>',
      ].join(""),
    };

    render(
      <>
        <h2 id="bound">Outside bound</h2>
        <CanonicalDocumentView
          document={documentWithMath}
          sectionId="bound"
        />
      </>,
    );

    const article = screen.getByRole("article", { name: "Context engineering: from learned artifacts to learned learning procedures" });
    const heading = Array.from(
      article.querySelectorAll<HTMLElement>("[id]"),
    ).find((candidate) => candidate.id === "bound");
    expect(heading).toHaveFocus();
    expect(scrollIntoView).toHaveBeenCalledWith({ block: "start" });
    expect(article.querySelector("math")).not.toBeNull();
    expect(screen.getByRole("heading", { name: "Outside bound" })).not.toHaveFocus();
  });
});
