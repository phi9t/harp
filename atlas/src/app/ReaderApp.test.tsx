import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { canonicalChapters } from "../content/canonical";
import { ReaderApp } from "./ReaderApp";

describe("RSI technical reader", () => {
  beforeEach(() => {
    window.location.hash = "#chapters/recursive-improvement-loop";
  });

  it("opens on the canonical nine-chapter spine", () => {
    render(<ReaderApp />);

    expect(window.location.hash).toBe("#chapters/recursive-improvement-loop");
    expect(
      screen.getByRole("heading", {
        name: "What makes an improvement loop recursive",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("navigation", { name: "Atlas routes" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("navigation", { name: "Technical chapters" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("article", {
        name: "What makes an improvement loop recursive",
      }),
    ).toHaveAttribute(
      "data-markdown-sha256",
      expect.stringMatching(/^[a-f0-9]{64}$/),
    );
  });

  it("routes between canonical chapters without duplicating prose", async () => {
    const user = userEvent.setup();
    render(<ReaderApp />);

    await user.click(
      screen.getByRole("button", {
        name: /09Evaluation, promotion, and containment/,
      }),
    );

    expect(window.location.hash).toBe(
      "#chapters/evaluation-promotion-containment",
    );
    expect(
      screen.getByRole("heading", {
        name: "Evaluation, promotion, and containment",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "content/chapters/evaluation-promotion-containment.md",
      ),
    ).toBeInTheDocument();
  });

  it("keeps source sections native and closed by default", () => {
    render(<ReaderApp />);

    const sourceFold = screen.getByText(
      "Original sources for this mechanism",
    ).closest("details");
    expect(sourceFold).not.toBeNull();
    expect(sourceFold).not.toHaveAttribute("open");
  });

  it("loads exactly the nine chapter documents from the validated corpus", () => {
    expect(canonicalChapters).toHaveLength(9);
    expect(
      new Set(canonicalChapters.map((chapter) => chapter.canonical_markdown_path))
        .size,
    ).toBe(9);
  });

  it("routes to canonical Weng Markdown without TypeScript prose", async () => {
    window.location.hash = "#weng";
    render(<ReaderApp />);

    expect(window.location.hash).toBe("#weng");
    expect(
      screen.getByRole("heading", {
        name: "RSI harness by Lil'Log, deconstructed",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "content/rsi_harness_by_lil_log_deconstructed.md",
      ),
    ).toBeInTheDocument();
  });

  it("blocks lineage promotion when any protected gate closes", async () => {
    const user = userEvent.setup();
    window.location.hash = "#loop";
    render(<ReaderApp />);

    expect(screen.getByRole("status", { name: "Promotion status" })).toHaveTextContent(
      "Promotion path open",
    );
    await user.click(screen.getByRole("button", { name: "Close Integrity gate" }));
    expect(screen.getByRole("status", { name: "Promotion status" })).toHaveTextContent(
      "Promotion blocked",
    );
    expect(screen.getByRole("button", { name: "Open Integrity gate" })).toBeInTheDocument();
  });
});
