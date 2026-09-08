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
  for (const [number, slug] of [
    ["01", "objects-and-representations"],
    ["02", "vector-spaces-and-subspaces"],
    ["03", "linear-maps-and-exact-structure"],
    ["04", "coordinates-and-duality"],
  ]) {
    it(`renders foundations Chapter ${number} with retained anchors and formal solutions`, () => {
      const chapter = canonicalCorpus.documents.find((document) =>
        document.concept_id === `cft-chapter-${number}-${slug}`);
      if (!chapter) throw new Error(`Missing foundations Chapter ${number}`);
      const { container } = render(<CanonicalDocumentView document={chapter} />);
      const article = container.querySelector("article.textbook-reading");
      if (!article) throw new Error("Missing textbook reading article");
      expect(article.querySelector("math")).not.toBeNull();
      expect(article.querySelector(".math-error, .katex-error")).toBeNull();
      for (let index = 1; index <= 6; index++) {
        const card = `cft-${number}-${String(index).padStart(3, "0")}`;
        const exercise = `exercise-cft-${number}-e${String(index).padStart(2, "0")}`;
        expect(article.querySelectorAll(`[id="${card}"]`)).toHaveLength(1);
        expect(article.querySelectorAll(`[id="${exercise}"]`)).toHaveLength(1);
        expect(article).toHaveTextContent(`exercise_${String(index).padStart(2, "0")}_solution`);
      }
      expect(article.querySelector(
        `a[href^="https://github.com/phi9t/harp/blob/master/formalization/lean/CrouzeixTextbook/Part01/Chapter${number}.lean"]`,
      )).not.toBeNull();
    });
  }
  it("includes the Harp chapter with rendered mathematics and exact source links", async () => {
    const chapter = canonicalCorpus.documents.find((document) =>
      document.concept_id === "cft-chapter-36-harp-finite-horizon-proof");
    if (!chapter) throw new Error("Missing Harp chapter");
    const { container } = render(<CanonicalDocumentView document={chapter} sectionId="cft-36-003" />);
    expect(screen.getByText("Browse the 36 chapters")).toBeInTheDocument();
    expect(container.querySelector("#cft-36-003")).toHaveFocus();
    expect(container.querySelector(".math-display math")).not.toBeNull();
    expect(container.querySelector(".math-error")).toBeNull();
    expect(screen.getByRole("link", { name: "Previous: Chapter 35: Comparison, verification, and boundaries" }))
      .toHaveAttribute("href", "#documents/cft-chapter-35-comparison-verification-and-boundaries");
    await userEvent.setup().click(screen.getByText("Proof references"));
    expect(screen.getByRole("link", { name: "Lean chapter module" })).toHaveAttribute(
      "href", "https://github.com/phi9t/harp/blob/master/formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean");
    expect(screen.getByRole("link", { name: "harp_finite_recurrence" })).toHaveAttribute(
      "href", "https://github.com/phi9t/harp/blob/master/formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean#L47");
  });
  it("keeps exact theorem source locations usable on the hosted reader", () => {
    render(<CanonicalDocumentView document={{ ...baseDocument,
      canonical_markdown_path: "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers.md",
      html: '<h1>Boundary proof</h1><h2>Statement</h2><a href="../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L14">Exact Lean theorem</a>',
    }} />);
    expect(screen.queryByRole("link", { name: "Exact Lean theorem" })).toHaveAttribute(
      "href", "https://github.com/phi9t/harp/blob/master/formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L14");
  });
  it("offers textbook chapter and proof references without hiding the argument", async () => {
    const chapter = canonicalCorpus.documents.find((document) =>
      document.concept_id === "cft-chapter-01-objects-and-representations");
    if (!chapter) throw new Error("Missing textbook fixture");
    render(<CanonicalDocumentView document={chapter} />);
    expect(screen.queryByRole("navigation", { name: "Textbook chapters" })).toBeInTheDocument();
    await userEvent.setup().click(screen.getByText("Proof references"));
    expect(screen.queryByRole("link", { name: "Lean chapter module" })).toHaveAttribute(
      "href", "https://github.com/phi9t/harp/blob/master/formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean");
    expect(screen.queryByRole("link", { name: "Next: Chapter 2: Vector spaces and subspaces" })).toHaveAttribute(
      "href", "#documents/cft-chapter-02-vector-spaces-and-subspaces");
    // Scope this plain heading directly: jsdom cannot compute styles for the
    // MathML descendants of other, formula-bearing chapter headings.
    expect(screen.getByText("Opening problem", { selector: "h2" })).toBeVisible();
    expect(screen.queryByRole("link", { name: "Correspondence and limits" })).toHaveAttribute(
      "href", "#documents/cft-lean-coverage-ledger");
  });
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
