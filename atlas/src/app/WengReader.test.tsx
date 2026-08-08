import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { AtlasApp } from "./AtlasApp";

describe("Weng-first reader", () => {
  beforeEach(() => {
    window.location.hash = "";
  });

  it("opens on the first source-bound Weng companion", () => {
    render(<AtlasApp />);

    expect(window.location.hash).toBe("#weng/system-being-improved");
    expect(
      screen.getByRole("heading", { name: "What system is being improved?" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("navigation", { name: "Weng article sections" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "Read original section" }),
    ).toHaveAttribute(
      "href",
      "https://lilianweng.github.io/posts/2026-07-04-harness/#case-study-coding-agent-harness",
    );
  });

  it("opens a published system reading and returns to its Weng section", async () => {
    const user = userEvent.setup();
    render(<AtlasApp />);

    await user.click(screen.getByRole("button", { name: "Karpathy Autoresearch" }));

    expect(window.location.hash).toBe(
      "#systems/autoresearch?return=weng/system-being-improved",
    );
    expect(
      screen.getByRole("heading", {
        name: "Autoresearch: a bounded overnight research loop",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Editable object" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Loop" }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Return to Weng" }));
    expect(window.location.hash).toBe("#weng/system-being-improved");
  });

  it("routes to compiled auxiliary technical readings", async () => {
    const user = userEvent.setup();
    render(<AtlasApp />);

    await user.click(
      screen.getByRole("button", { name: "Context engineering" }),
    );
    await user.click(
      screen.getByRole("link", { name: "Open the full context-engineering deep dive" }),
    );

    expect(window.location.hash).toBe(
      "#documents/context-engineering-deep-dive",
    );
    expect(
      screen.getByRole("heading", {
        name: "Context engineering: from learned artifacts to learned learning procedures",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("navigation", {
        name: "Context engineering technical sections",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Context engineering reading map" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Context pipeline checkpoint" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "ACE original paper" }),
    ).toHaveAttribute("href", "https://arxiv.org/abs/2510.04618");
    expect(
      screen.getByRole("link", { name: "MCE original paper" }),
    ).toHaveAttribute("href", "https://arxiv.org/abs/2601.21557");
    expect(
      screen.getByRole("link", { name: "Meta-Harness original paper" }),
    ).toHaveAttribute("href", "https://arxiv.org/abs/2603.28052");

    await user.click(
      screen.getByRole("button", { name: /Minimal mental model/ }),
    );
    expect(
      screen.getByRole("heading", { name: "Minimal mental model" }),
    ).toHaveFocus();
  });

  it("reads published AFlow from Weng and returns to the same section", async () => {
    const user = userEvent.setup();
    window.location.hash = "#weng/workflow-design-and-search";
    render(<AtlasApp />);

    await user.click(screen.getByRole("button", { name: "AFlow" }));

    expect(window.location.hash).toBe(
      "#systems/aflow?return=weng/workflow-design-and-search",
    );
    expect(
      screen.getByRole("heading", { name: "AFlow: MCTS over agentic workflows" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Algorithm" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Evaluation" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Claim ceiling" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "AFLOW original paper" }),
    ).toHaveAttribute("href", "https://openreview.net/forum?id=z5uVAKwmjf");
    const sequence = screen.getByRole("list", {
      name: "AFLOW reading sequence",
    });
    expect(sequence).toHaveTextContent("§§3.1–3.2");
    expect(sequence).toHaveTextContent("§4, Algorithm 1");
    expect(sequence).toHaveTextContent("§§5.1–5.2, Table 1");
    expect(sequence).toHaveTextContent("§5.2, Figure 5");
    expect(sequence).toHaveTextContent("§6 and §3.2 Limitations");
    expect(sequence).toHaveTextContent("Appendix A.3–A.6");

    await user.click(screen.getByRole("button", { name: "Return to Weng" }));
    expect(window.location.hash).toBe("#weng/workflow-design-and-search");
  });
});
