import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import userEvent from "@testing-library/user-event";

import { canonicalCorpus } from "../content/canonical";
import { CanonicalDocumentView } from "./ChapterReader";

const baseDocument = canonicalCorpus.documents.find(
  (document) => document.concept_id === "context-engineering-deep-dive",
);

if (!baseDocument) {
  throw new Error("Validated chapter-reader fixture disappeared");
}

describe("canonical document view", () => {
  it("navigates repeated section titles by their own identities", async () => {
    const user = userEvent.setup();
    render(<CanonicalDocumentView document={{
      ...baseDocument,
      html: "<h1>Proof routes</h1><h2>Assumptions</h2><p>First route</p><h2>Assumptions</h2><p>Second route</p>",
    }} sectionNavigationLabel="Proof sections" />);
    await user.click(screen.getAllByRole("button", { name: /Assumptions/ })[1]);
    expect(screen.getAllByRole("heading", { name: "Assumptions" })[1]).toHaveFocus();
  });
  it("makes plain Markdown sections addressable without altering code whitespace", () => {
    const document = {
      ...baseDocument,
      html: '<h1>Executor</h1><h2>Harp\'s executor and scheduler</h2><pre><code>A  --&gt;  B\n         |\n         C</code></pre>',
    };
    render(<CanonicalDocumentView document={document} sectionId="harp-s-executor-and-scheduler" />);
    expect(screen.getByRole("heading", { name: "Harp's executor and scheduler" })).toHaveFocus();
    expect(screen.getByText(/A.*B/).textContent).toBe("A  -->  B\n         |\n         C");
  });
  it("exposes document metadata and accessible Obsidian fallbacks", () => {
    const documentWithObsidianSyntax = {
      ...baseDocument,
      html: [
        "<h1>Metadata</h1>",
        '<aside class="obsidian-callout" role="note"><p>Evidence-aware reading</p></aside>',
        '<a class="obsidian-embed-fallback" data-obsidian-embed="true" href="#documents/darwinx-index">DarwinX</a>',
      ].join(""),
    };

    render(<CanonicalDocumentView document={documentWithObsidianSyntax} />);

    expect(screen.getByText("Confidence")).toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent("Evidence-aware reading");
    expect(screen.getByRole("link", { name: "DarwinX" }))
      .toHaveAttribute("href", "#documents/darwinx-index");
  });

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
