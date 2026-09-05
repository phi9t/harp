import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import userEvent from "@testing-library/user-event";

import { canonicalCorpus } from "../content/canonical";
import { KnowledgeHome } from "./KnowledgeHome";

describe("knowledge home", () => {
  it("searches all compiled readings and explains an empty result", async () => {
    const user = userEvent.setup();
    render(<KnowledgeHome corpus={canonicalCorpus} />);
    await user.type(screen.getByRole("searchbox", { name: "Search readings" }), "Crouzeix");
    expect(screen.getByRole("link", { name: /Crouzeix proof status and critical assessment/ }))
      .toHaveAttribute("href", "#documents/crouzeix-status-and-critical-assessment");
    await user.clear(screen.getByRole("searchbox", { name: "Search readings" }));
    await user.type(screen.getByRole("searchbox", { name: "Search readings" }), "not-a-research-topic");
    expect(screen.getByRole("status")).toHaveTextContent("No readings found");
    await user.click(screen.getByRole("button", { name: "Clear search" }));
    expect(screen.getByRole("searchbox", { name: "Search readings" })).toHaveValue("");
  });
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
