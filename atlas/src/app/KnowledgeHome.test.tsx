import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { KnowledgeHome } from "./KnowledgeHome";

describe("knowledge home", () => {
  it("derives DarwinX and evidence cards from corpus metadata", () => {
    render(<KnowledgeHome corpus={canonicalCorpus} />);

    expect(screen.getByText("Claim and evidence")).toBeInTheDocument();
    expect(
      screen
        .getAllByRole("link")
        .find((link) => link.getAttribute("href") === "#documents/darwinx-index"),
    ).toHaveTextContent("DarwinX");
  });
});
