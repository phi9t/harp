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
        "knowledge/rsi/chapters/evaluation-promotion-containment.md",
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
        "knowledge/rsi/rsi_harness_by_lil_log_deconstructed.md",
      ),
    ).toBeInTheDocument();
  });

  it("opens the Crouzeix packet through its first-class route", async () => {
    const user = userEvent.setup();
    render(<ReaderApp />);

    await user.click(screen.getByRole("button", { name: "Crouzeix" }));

    expect(window.location.hash).toBe("#crouzeix-conjecture");
    expect(
      screen.getByRole("heading", {
        name: "Crouzeix conjecture two-proof index",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Current route" }))
      .toHaveTextContent("Crouzeix");
    const article = screen.getByRole("article", {
      name: "Crouzeix conjecture two-proof index",
    });
    expect(article.querySelector("math")).not.toBeNull();
  });

  it("opens the mathematical foundations packet through its first-class route", async () => {
    const user = userEvent.setup();
    render(<ReaderApp />);

    await user.click(screen.getByRole("button", { name: "Math foundations" }));

    expect(window.location.hash).toBe("#mathematical-foundations");
    expect(
      screen.getByRole("heading", {
        name: "Mathematical foundations for machine learning",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Current route" }))
      .toHaveTextContent("Math foundations");
  });

  it("opens the verified coevolution agenda through its first-class route", async () => {
    const user = userEvent.setup();
    render(<ReaderApp />);

    await user.click(screen.getByRole("button", { name: "Verified coevolution" }));

    expect(window.location.hash).toBe("#verified-coevolution");
    expect(
      screen.getByRole("heading", {
        name: "Beyond self-training: recursive closure, model-harness coevolution, and assurance of self-improving AI.",
      }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "knowledge/verified_coevolution_agenda/verified_coevolution_agenda.md",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Current route" }))
      .toHaveTextContent("Verified coevolution");
  });

  it("opens the workstreams console from the reader top nav and drops reader footer chrome", async () => {
    const user = userEvent.setup();
    window.location.hash = "#knowledge";
    render(<ReaderApp />);

    const consoleButton = screen.getByRole("button", {
      name: "Open Console / Workstreams",
    });
    expect(consoleButton).toHaveTextContent("Console / Workstreams");

    await user.click(consoleButton);

    expect(window.location.hash).toBe("#workstreams");
    expect(
      screen.getByRole("heading", {
        level: 1,
        name: "Harp Workstreams",
      }),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Harp Atlas / source-bound technical reader"),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("navigation", { name: "Atlas routes" }),
    ).not.toBeInTheDocument();
  });

  it("returns from the workstreams console to the Harp knowledge Atlas route", async () => {
    const user = userEvent.setup();
    window.location.hash = "#workstreams";
    render(<ReaderApp />);

    await user.click(
      screen.getByRole("button", { name: "Open Harp knowledge Atlas" }),
    );

    expect(window.location.hash).toBe("#knowledge");
    expect(
      screen.getByRole("navigation", { name: "Atlas routes" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Harp Atlas / source-bound technical reader"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", {
        level: 1,
        name: "Harp Workstreams",
      }),
    ).not.toBeInTheDocument();
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
