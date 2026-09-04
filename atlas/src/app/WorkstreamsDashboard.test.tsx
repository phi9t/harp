import { readFileSync } from "node:fs";

import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  Activity,
  BadgeCheck,
  Check,
  Circle,
  CircleDot,
  ListTodo,
  OctagonX,
  Pencil,
} from "lucide-react";

import type { WorkstreamsSnapshot } from "../content/workstreams";
import { workstreamsReference } from "../content/workstreams";
import {
  stageStatusPresentation,
  workstreamStageDetailId,
  workstreamStatusPresentation,
} from "./workstreamsPresentation";
import { WorkstreamsDashboard } from "./WorkstreamsDashboard";

function getStageTooltip(id: string): HTMLElement {
  const tooltip = document.getElementById(id);
  expect(tooltip).not.toBeNull();
  return tooltip as HTMLElement;
}

function normalizeCssWhitespace(value: string): string {
  return value
    .replace(/\s+/gu, " ")
    .replace(/\s*([,:(){}])\s*/gu, "$1")
    .trim();
}

function cssBlock(source: string, prelude: string): string {
  const normalizedSource = normalizeCssWhitespace(source);
  const blockStart = normalizedSource.indexOf(`${normalizeCssWhitespace(prelude)}{`);
  expect(blockStart, `Missing CSS block: ${prelude}`).toBeGreaterThanOrEqual(0);

  const openingBrace = normalizedSource.indexOf("{", blockStart);
  let depth = 1;
  let closingBrace = openingBrace + 1;
  while (closingBrace < normalizedSource.length && depth > 0) {
    if (normalizedSource[closingBrace] === "{") {
      depth += 1;
    } else if (normalizedSource[closingBrace] === "}") {
      depth -= 1;
    }
    closingBrace += 1;
  }
  expect(depth, `Unclosed CSS block: ${prelude}`).toBe(0);
  return normalizedSource.slice(openingBrace + 1, closingBrace - 1);
}

function expectCssDeclarations(
  source: string,
  selectors: string | readonly string[],
  declarations: Readonly<Record<string, string>>,
): void {
  const selectorPrelude = typeof selectors === "string" ? selectors : selectors.join(",");
  const ruleBody = cssBlock(source, selectorPrelude);
  const actualDeclarations = new Map(
    ruleBody
      .split(";")
      .map((declaration) => declaration.trim())
      .filter((declaration) => declaration.length > 0)
      .map((declaration) => {
        const separator = declaration.indexOf(":");
        expect(separator, `Invalid CSS declaration: ${declaration}`).toBeGreaterThan(0);
        return [
          declaration.slice(0, separator).trim(),
          normalizeCssWhitespace(declaration.slice(separator + 1)),
        ];
      }),
  );

  for (const [property, value] of Object.entries(declarations)) {
    expect(actualDeclarations.get(property), `${selectorPrelude} ${property}`).toBe(
      normalizeCssWhitespace(value),
    );
  }
}

describe("WorkstreamsDashboard", () => {
  beforeEach(() => {
    window.sessionStorage.clear();
  });

  it("exposes exhaustive StageStatus and WorkstreamStatus presentation maps with exact labels and Lucide identities", () => {
    expect(stageStatusPresentation.complete.label).toBe("Complete");
    expect(stageStatusPresentation.complete.Icon).toBe(Check);
    expect(stageStatusPresentation.active.label).toBe("Active");
    expect(stageStatusPresentation.active.Icon).toBe(CircleDot);
    expect(stageStatusPresentation.queued.label).toBe("Queued");
    expect(stageStatusPresentation.queued.Icon).toBe(Circle);
    expect(stageStatusPresentation.blocked.label).toBe("Blocked");
    expect(stageStatusPresentation.blocked.Icon).toBe(OctagonX);

    expect(workstreamStatusPresentation.active.label).toBe("Active");
    expect(workstreamStatusPresentation.active.Icon).toBe(Activity);
    expect(workstreamStatusPresentation.blocked.label).toBe("Blocked");
    expect(workstreamStatusPresentation.blocked.Icon).toBe(OctagonX);
    expect(workstreamStatusPresentation.planning.label).toBe("Planning");
    expect(workstreamStatusPresentation.planning.Icon).toBe(ListTodo);
    expect(workstreamStatusPresentation.drafting.label).toBe("Drafting");
    expect(workstreamStatusPresentation.drafting.Icon).toBe(Pencil);
    expect(workstreamStatusPresentation["complete-local"].label).toBe("Complete locally");
    expect(workstreamStatusPresentation["complete-local"].Icon).toBe(BadgeCheck);

    const allLabels = [
      stageStatusPresentation.complete.label,
      stageStatusPresentation.active.label,
      stageStatusPresentation.queued.label,
      stageStatusPresentation.blocked.label,
      workstreamStatusPresentation.active.label,
      workstreamStatusPresentation.blocked.label,
      workstreamStatusPresentation.planning.label,
      workstreamStatusPresentation.drafting.label,
      workstreamStatusPresentation["complete-local"].label,
    ].join(" ");
    expect(allLabels).not.toMatch(/[✓✔✗✕●○•◦■□◆◇⬤⭕❌]/u);
  });

  it("renders the dashboard shell, blocked focus strip, four cards, lifecycle nodes, and navigation callbacks", async () => {
    const onNavigate = vi.fn();
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={onNavigate}
      />,
    );

    const heading = screen.getByRole("heading", { level: 1, name: "Harp Workstreams" });
    expect(heading).toBeInTheDocument();
    expect(screen.getByText("REFERENCE SNAPSHOT")).toBeInTheDocument();
    expect(screen.getByText("2026-09-03T16:00:00.000Z")).toBeInTheDocument();
    const dashboardHeader = heading.closest("header");
    expect(dashboardHeader).not.toBeNull();
    expect(
      within(dashboardHeader as HTMLElement).getByText("Online at snapshot"),
    ).toBeInTheDocument();

    const sidebar = screen.getByRole("complementary", {
      name: "Workstreams navigation",
    });
    expect(sidebar).toBeInTheDocument();

    const focusStrip = screen.getByRole("region", { name: "Blocked workstream focus" });
    expect(within(focusStrip).getByRole("button", { name: "FOCUS NOW" })).toBeInTheDocument();
    expect(
      within(focusStrip).getByRole("heading", { level: 2, name: "Autodiff Geometry" }),
    ).toBeInTheDocument();
    expect(
      within(focusStrip).getByText(
        "The source-to-curriculum map is incomplete beyond finite-coordinate identities.",
      ),
    ).toBeInTheDocument();

    const workstreamsRegion = screen.getByRole("region", { name: "Workstreams" });
    const articles = within(workstreamsRegion).getAllByRole("article");
    expect(articles).toHaveLength(4);
    expect(screen.getByRole("heading", { level: 3, name: "Crouzeix Conjecture" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 3, name: "Autodiff Geometry" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 3, name: "Agentic Research" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 3, name: "NNG4 / Foundations" })).toBeInTheDocument();

    const lifecycleLists = articles.map((article) => within(article).getByRole("list"));
    expect(lifecycleLists).toHaveLength(4);
    expect(screen.getByText(/3\/3 routes complete-local/)).toBeInTheDocument();
    expect(screen.getByText("All levels proved")).toBeInTheDocument();
    expect(workstreamsRegion.querySelectorAll('[role="tooltip"]')).toHaveLength(
      workstreamsReference.workstreams.reduce(
        (count, workstream) => count + workstream.stages.length,
        0,
      ),
    );

    const blockedCard = screen.getByRole("article", { name: /Autodiff Geometry/i });
    expect(within(blockedCard).getByText("foundation snapshot recorded 2026-08-18")).toBeInTheDocument();
    expect(within(blockedCard).getByText("Evidence: The source-to-curriculum map is incomplete beyond finite-coordinate identities.")).toBeInTheDocument();

    const detailIds = new Set<string>();
    const dashboardText = workstreamsRegion.textContent ?? "";
    expect(dashboardText).not.toMatch(/[✓✔✗✕●○•◦■□◆◇⬤⭕❌]/u);
    for (const workstream of workstreamsReference.workstreams) {
      const card = screen.getByRole("article", { name: workstream.title });
      const lifecycleList = within(card).getByRole("list");
      const stageButtons = within(lifecycleList).getAllByRole("button");
      expect(stageButtons).toHaveLength(workstream.stages.length);
      expect(card.querySelectorAll("details")).toHaveLength(1);

      const fallbackSummary = within(card)
        .getByText(`All lifecycle details for ${workstream.title}`)
        .closest("summary");
      expect(fallbackSummary).not.toBeNull();
      const fallbackDetails = fallbackSummary?.closest("details");
      expect(fallbackDetails).not.toBeNull();
      expect(fallbackDetails).not.toHaveAttribute("open");

      for (const [stageIndex, stage] of workstream.stages.entries()) {
        const presentation = stageStatusPresentation[stage.status];
        const expectedName = `${stage.name}: ${presentation.label}`;
        const button = within(lifecycleList).getByRole("button", {
          name: expectedName,
        });
        const detailId = workstreamStageDetailId(workstream.id, stageIndex);
        expect(detailIds.has(detailId)).toBe(false);
        detailIds.add(detailId);

        expect(button).toHaveAttribute("aria-label", expectedName);
        expect(button).toHaveAttribute("aria-controls", detailId);
        expect(button).toHaveAttribute("aria-describedby", detailId);
        expect(button).toHaveAttribute("aria-expanded", "false");
        expect(within(button).getByText(stage.name)).toBeInTheDocument();
        expect(within(button).getByText(presentation.label)).toBeInTheDocument();
        expect(button.querySelector("svg")).not.toBeNull();

        const tooltip = getStageTooltip(detailId);
        expect(tooltip).toHaveAttribute("role", "tooltip");
        expect(tooltip).toHaveTextContent(stage.detail);
        expect(tooltip).toHaveAttribute("hidden");
        expect(tooltip).not.toBeVisible();
      }
    }

    const blockedLifecycle = within(blockedCard).getByRole("list");
    const curriculumButton = within(blockedLifecycle).getByRole("button", {
      name: "Curriculum: Blocked",
    });
    const acquireButton = within(blockedLifecycle).getByRole("button", {
      name: "Acquire: Complete",
    });
    const parseButton = within(blockedLifecycle).getByRole("button", {
      name: "Parse: Complete",
    });
    const curriculumTooltip = getStageTooltip(
      workstreamStageDetailId("autodiff-geometry", 2),
    );
    const acquireTooltip = getStageTooltip(
      workstreamStageDetailId("autodiff-geometry", 0),
    );
    const parseTooltip = getStageTooltip(
      workstreamStageDetailId("autodiff-geometry", 1),
    );

    await user.hover(curriculumButton);
    expect(curriculumButton).toHaveAttribute("aria-expanded", "true");
    expect(curriculumTooltip).not.toHaveAttribute("hidden");
    await user.unhover(curriculumButton);
    expect(curriculumButton).toHaveAttribute("aria-expanded", "false");
    expect(curriculumTooltip).toHaveAttribute("hidden");

    await user.tab();
    while (document.activeElement !== acquireButton) {
      await user.tab();
    }
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");

    await user.tab();
    expect(document.activeElement).toBe(parseButton);
    expect(acquireTooltip).toHaveAttribute("hidden");
    expect(parseTooltip).not.toHaveAttribute("hidden");
    expect(parseButton).toHaveAttribute("aria-expanded", "true");

    await user.hover(parseButton);
    expect(parseTooltip).not.toHaveAttribute("hidden");
    expect(parseButton).toHaveAttribute("aria-expanded", "true");
    await user.unhover(parseButton);
    expect(parseTooltip).not.toHaveAttribute("hidden");
    expect(parseButton).toHaveAttribute("aria-expanded", "true");

    await user.tab();
    expect(parseTooltip).toHaveAttribute("hidden");
    expect(parseButton).toHaveAttribute("aria-expanded", "false");

    await user.hover(acquireButton);
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");
    acquireButton.focus();
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");
    parseButton.focus();
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");
    await user.unhover(acquireButton);
    expect(acquireTooltip).toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "false");

    await user.click(curriculumButton);
    expect(curriculumButton).toHaveAttribute("aria-expanded", "true");
    expect(curriculumTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "false");
    expect(acquireTooltip).toHaveAttribute("hidden");

    await user.click(acquireButton);
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(curriculumButton).toHaveAttribute("aria-expanded", "false");
    expect(curriculumTooltip).toHaveAttribute("hidden");

    await user.click(acquireButton);
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");
    expect(acquireButton).toHaveFocus();
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");

    await user.tab();
    expect(acquireTooltip).not.toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "true");
    await user.unhover(acquireButton);
    expect(acquireTooltip).toHaveAttribute("hidden");
    expect(acquireButton).toHaveAttribute("aria-expanded", "false");

    acquireButton.focus();
    await user.click(curriculumButton);
    expect(curriculumButton).toHaveAttribute("aria-expanded", "true");
    expect(curriculumTooltip).not.toHaveAttribute("hidden");
    expect(curriculumButton).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(curriculumButton).toHaveAttribute("aria-expanded", "false");
    expect(curriculumTooltip).toHaveAttribute("hidden");
    expect(curriculumButton).toHaveFocus();

    const blockedFallbackSummary = within(blockedCard)
      .getByText("All lifecycle details for Autodiff Geometry")
      .closest("summary");
    expect(blockedFallbackSummary).not.toBeNull();
    const blockedFallbackDetails = blockedFallbackSummary?.closest("details");
    expect(blockedFallbackDetails).not.toBeNull();
    expect(blockedFallbackDetails).not.toHaveAttribute("open");
    await user.click(blockedFallbackSummary as HTMLElement);
    expect(blockedFallbackDetails).toHaveAttribute("open");
    expect(
      within(blockedFallbackDetails as HTMLElement).getByText(
        "Acquire: Complete. Core source registry captured",
      ),
    ).toBeInTheDocument();
    expect(
      within(blockedFallbackDetails as HTMLElement).getByText(
        "Curriculum: Blocked. Source-to-lesson dependency graph remains incomplete",
      ),
    ).toBeInTheDocument();
    expect(
      within(blockedFallbackDetails as HTMLElement).getByText(
        "Verify: Queued. Run focused Lean route only after formalization",
      ),
    ).toBeInTheDocument();

    const controls = within(sidebar);
    const workstreamsButton = controls.getByRole("button", { name: "Workstreams" });
    const runsButton = controls.getByRole("button", { name: "Runs" });
    const agentsButton = controls.getByRole("button", { name: "Agents" });
    const evaluatorsButton = controls.getByRole("button", { name: "Evaluators" });
    const knowledgeButton = controls.getByRole("button", {
      name: "Open Harp knowledge Atlas",
    });
    const settingsControl = controls.getByRole("button", { name: "Settings" });
    expect(settingsControl).not.toBeDisabled();
    expect(controls.getByText("Recently Active")).toBeInTheDocument();
    expect(knowledgeButton).toHaveTextContent("Knowledge");
    const recentItems = within(controls.getByRole("list", { name: "Recently Active list" }))
      .getAllByRole("listitem")
      .map((item) => item.textContent);
    expect(recentItems).toEqual([
      "Autodiff Geometry",
      "Crouzeix Conjecture",
      "Agentic Research",
    ]);

    heading.focus();
    await user.click(workstreamsButton);
    expect(heading).toHaveFocus();

    await user.click(runsButton);
    expect(screen.getByRole("heading", { name: "Recent Runs" })).toHaveFocus();

    const recentlyActive = screen.getByRole("list", { name: "Recently Active list" });
    await user.click(agentsButton);
    expect(recentlyActive).toHaveFocus();

    await user.click(evaluatorsButton);
    expect(screen.getByRole("heading", { name: "Latest Research Insights" })).toHaveFocus();

    await user.click(knowledgeButton);
    expect(onNavigate).toHaveBeenCalledWith("#knowledge");

    await user.click(screen.getByRole("button", { name: "FOCUS NOW" }));
    expect(onNavigate).toHaveBeenCalledWith("#mathematical-foundations");

    const agentVelocityRegion = screen.getByRole("region", { name: "Agent velocity" });
    const insightsRegion = screen.getByRole("region", { name: "Latest Research Insights" });
    expect(agentVelocityRegion).toHaveAttribute("id", "agent-velocity");
    expect(agentVelocityRegion).toHaveAttribute("title", "Agent velocity");
    expect(insightsRegion).toHaveAttribute("id", "insights");
    expect(
      within(agentVelocityRegion).getByRole("heading", { level: 2, name: "Harp agent" }),
    ).toBeInTheDocument();
    expect(
      within(insightsRegion).getByRole("heading", { name: "Latest Research Insights" }),
    ).toBeInTheDocument();
  });

  it("wires an isolated workstreams stylesheet and exposes the structural hooks it styles", () => {
    const dashboardSource = readFileSync("src/app/WorkstreamsDashboard.tsx", "utf8");
    expect(dashboardSource).toMatch(/import\s+"..\/styles\/workstreams\.css";/u);
    expect(dashboardSource).toContain('className="workstreams-shell"');
    expect(dashboardSource).toContain('className="workstreams-shortcut"');

    const stylesheet = readFileSync("src/styles/workstreams.css", "utf8");
    const requiredTokens = {
      "--ops-canvas": "#020617",
      "--ops-sidebar": "#0f172a",
      "--ops-panel": "#131c2e",
      "--ops-panel-raised": "#1e293b",
      "--ops-line": "#334155",
      "--ops-cyan": "#38bdf8",
      "--ops-green": "#3fb950",
      "--ops-amber": "#d29922",
      "--ops-red": "#f85149",
      "--ops-text": "#f8fafc",
      "--ops-muted": "#94a3b8",
    };
    for (const [token, value] of Object.entries(requiredTokens)) {
      expect(stylesheet).toContain(`${token}: ${value};`);
    }
    expect(stylesheet).not.toMatch(/--workstreams-|var\(--workstreams-/u);
    expect(stylesheet).toContain(".workstreams-shell");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-sidebar");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-main");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-shortcut");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-watchlist-cue");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-meta");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-mono");
    expect(stylesheet).toContain("min-height: 100dvh;");
    expect(stylesheet).toMatch(/font-family:\s*system-ui,\s*-apple-system,\s*BlinkMacSystemFont/u);
    const desktopStylesheet = cssBlock(stylesheet, "@media (min-width: 1280px)");
    expect(stylesheet).toContain("@media (min-width: 900px) and (max-width: 1279px)");
    const mobileStylesheet = cssBlock(stylesheet, "@media (max-width: 899px)");
    expect(stylesheet).toContain("@media (prefers-reduced-motion: reduce)");
    expect(stylesheet).toContain("@media print");
    expectCssDeclarations(desktopStylesheet, ".workstreams-shell", {
      display: "grid",
      "grid-template-columns": "224px minmax(0, 1fr)",
    });
    expectCssDeclarations(desktopStylesheet, ".workstreams-shell .workstreams-main", {
      "grid-template-columns": "minmax(0, 1fr) 18rem",
      "align-items": "start",
    });
    expectCssDeclarations(desktopStylesheet, ".workstreams-shell .workstreams-sidebar", {
      position: "sticky",
      top: "64px",
    });
    expectCssDeclarations(
      desktopStylesheet,
      [
        ".workstreams-shell .workstreams-header",
        ".workstreams-shell .workstreams-focus",
        ".workstreams-shell #insights",
        ".workstreams-shell .watchlist-scroll",
        ".workstreams-shell .reference-settings",
      ],
      { "grid-column": "1 / -1" },
    );
    expectCssDeclarations(desktopStylesheet, ".workstreams-shell .workstreams-grid", {
      "grid-column": "1",
    });
    expectCssDeclarations(desktopStylesheet, ".workstreams-shell .agent-velocity", {
      "grid-column": "2",
      "align-self": "start",
    });
    expectCssDeclarations(desktopStylesheet, ".workstreams-shell .insight-grid", {
      "grid-template-columns": "repeat(3, minmax(0, 1fr))",
    });
    expect(stylesheet).toContain("grid-template-columns: 72px minmax(0, 1fr);");
    const controlSelectors = [
      ".workstreams-shell .workstreams-nav__button",
      ".workstreams-shell .workstreams-focus__button",
      ".workstreams-shell .workstreams-search button",
      ".workstreams-shell .reference-settings button",
      ".workstreams-shell .agent-velocity__header button",
      ".workstreams-shell .agent-velocity form button",
      ".workstreams-shell .agent-velocity details summary",
      ".workstreams-shell .insight-card__panel button",
      ".workstreams-shell .watchlist-scroll button",
      '.workstreams-shell .insight-card [role="tab"]',
      ".workstreams-shell .workstream-card__details summary",
    ];
    expectCssDeclarations(stylesheet, controlSelectors, {
      "min-width": "24px",
      "min-height": "24px",
      gap: "8px",
    });
    expectCssDeclarations(
      mobileStylesheet,
      [...controlSelectors, ".workstreams-shell .workstream-card__stage-summary"],
      {
        "min-width": "44px",
        "min-height": "44px",
        gap: "8px",
      },
    );
    expect(stylesheet).toContain("font-size: 16px;");
    expect(stylesheet).toContain("overflow-x: hidden;");
    expect(stylesheet).toContain("display: none;");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-search button");
    expect(stylesheet).toContain(".workstreams-shell .workstreams-sidebar");
    expect(stylesheet).toContain(".workstreams-shell .reference-settings");
    expect(stylesheet).toContain(".workstreams-shell .agent-velocity svg line");
    expect(stylesheet).toContain("transition: opacity 160ms ease, transform 160ms ease;");
    expect(stylesheet).not.toMatch(/gradient|shadow|url\(|@font-face|animation:/u);
    const bareSelectors = [
      /(^|\n)\s*\.workstreams-sidebar\b/u,
      /(^|\n)\s*\.workstreams-main\b/u,
      /(^|\n)\s*\.workstreams-header\b/u,
      /(^|\n)\s*\.workstreams-focus\b/u,
      /(^|\n)\s*\.workstream-card\b/u,
      /(^|\n)\s*\.agent-velocity\b/u,
      /(^|\n)\s*\.research-insights\b/u,
      /(^|\n)\s*\.watchlist-scroll\b/u,
      /(^|\n)\s*\.reference-settings\b/u,
      /(^|\n)\s*\[data-state=/u,
    ];
    for (const selector of bareSelectors) {
      expect(stylesheet).not.toMatch(selector);
    }

    const radii = Array.from(stylesheet.matchAll(/border-radius:\s*([^;]+);/gu)).map(
      (match) => match[1].trim(),
    );
    expect(radii.length).toBeGreaterThan(0);
    for (const radius of radii) {
      if (radius === "999px") {
        continue;
      }
      const px = radius.match(/^([0-9]+(?:\.[0-9]+)?)px$/u);
      expect(px).not.toBeNull();
      expect(Number(px?.[1] ?? "999")).toBeLessThanOrEqual(6);
    }

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
    expect(screen.getByRole("main")).toHaveClass("workstreams-main");
    expect(
      screen.getByRole("complementary", { name: "Workstreams navigation" }),
    ).toHaveClass("workstreams-sidebar");
    expect(screen.getByRole("region", { name: "Blocked workstream focus" })).toHaveClass(
      "workstreams-focus",
    );
    expect(screen.getByRole("region", { name: "Workstreams" })).toHaveClass(
      "workstreams-grid",
    );
    expect(
      screen.getByRole("region", {
        name: "Workstream watchlist, scroll for more columns",
      }),
    ).toHaveAttribute("tabindex", "0");
    expect(screen.getByText("Cmd K")).toHaveClass("workstreams-shortcut");

    const settingsButton = screen.getByRole("button", { name: "Settings" });
    const knowledgeButton = screen.getByRole("button", {
      name: "Open Harp knowledge Atlas",
    });
    expect(settingsButton.querySelector("svg")).not.toBeNull();
    expect(knowledgeButton.querySelector("svg")).not.toBeNull();
    expect(settingsButton).toHaveTextContent("Settings");
    expect(knowledgeButton).toHaveTextContent("Knowledge");

    const blockedCard = screen.getByRole("article", { name: "Autodiff Geometry" });
    const cardStatus = blockedCard.querySelector(".workstream-card__status");
    expect(cardStatus).not.toBeNull();
    expect(cardStatus?.querySelector("svg")).not.toBeNull();
    expect(within(cardStatus as HTMLElement).getByText("Blocked")).toBeInTheDocument();

    const statusChip = document.querySelector(".status-chip[data-state]");
    expect(statusChip).not.toBeNull();
    expect(statusChip?.textContent?.trim()).not.toBe("");

    expect(
      document.querySelector('.agent-velocity__point--current[data-current="true"]'),
    ).not.toBeNull();
  });

  it("renders search, filters workstreams and watchlist rows, and supports clear plus keyboard focus shortcuts", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    const search = screen.getByRole("searchbox", { name: "Search" });
    expect(search).toBeInTheDocument();
    expect(screen.getByText("Cmd K")).toBeInTheDocument();

    expect(
      within(screen.getByRole("region", { name: "Workstreams" })).getAllByRole(
        "article",
      ),
    ).toHaveLength(4);
    expect(
      screen.getByRole("table", { name: "Workstream watchlist" }),
    ).toBeInTheDocument();

    await user.type(search, "autodiff");

    expect(
      within(screen.getByRole("region", { name: "Workstreams" })).getAllByRole(
        "article",
      ),
    ).toHaveLength(1);
    expect(screen.getByRole("article", { name: "Autodiff Geometry" }))
      .toBeInTheDocument();

    const watchlist = screen.getByRole("table", { name: "Workstream watchlist" });
    const rows = within(watchlist).getAllByRole("row");
    expect(rows).toHaveLength(2);
    expect(within(rows[1]).getByText("Autodiff Geometry")).toBeInTheDocument();

    const clear = screen.getByRole("button", { name: "Clear search" });
    await user.click(clear);
    expect(search).toHaveFocus();
    expect(search).toHaveValue("");
    expect(
      within(screen.getByRole("region", { name: "Workstreams" })).getAllByRole(
        "article",
      ),
    ).toHaveLength(4);

    await user.type(search, "nonsense");
    expect(
      within(screen.getByRole("region", { name: "Workstreams" })).queryAllByRole(
        "article",
      ),
    ).toHaveLength(0);
    expect(
      screen.getByText("No workstreams match the current search."),
    ).toBeInTheDocument();
    expect(within(watchlist).getByRole("columnheader", { name: "Workstream" }))
      .toBeInTheDocument();
    expect(within(watchlist).getAllByRole("row")).toHaveLength(2);
    expect(
      within(within(watchlist).getAllByRole("row")[1] as HTMLElement).getByText(
        "No workstreams match this search.",
      ),
    ).toBeInTheDocument();

    await user.click(clear);

    await user.keyboard("{Meta>}{KeyK}{/Meta}");
    expect(search).toHaveFocus();
    (search as HTMLInputElement).blur();
    await user.keyboard("{Control>}{KeyK}{/Control}");
    expect(search).toBeInTheDocument();
  });

  it("supports evaluator-driven artifact selection, settings toggles, and sidebar focus behavior", async () => {
    const onNavigate = vi.fn();
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={onNavigate}
      />,
    );

    const sidebar = screen.getByRole("complementary", {
      name: "Workstreams navigation",
    });
    const controls = within(sidebar);
    const heading = screen.getByRole("heading", { level: 1, name: "Harp Workstreams" });
    const recentRuns = screen.getByRole("heading", { name: "Recent Runs" });
    const recentlyActive = screen.getByRole("list", { name: "Recently Active list" });

    heading.focus();
    await user.click(controls.getByRole("button", { name: "Workstreams" }));
    expect(heading).toHaveFocus();

    await user.click(controls.getByRole("button", { name: "Runs" }));
    expect(recentRuns).toHaveFocus();

    await user.click(controls.getByRole("button", { name: "Agents" }));
    expect(recentlyActive).toHaveFocus();

    const insightsHeading = screen.getByRole("heading", { name: "Latest Research Insights" });
    expect(insightsHeading).toHaveAttribute("tabindex", "-1");

    const evaluators = controls.getByRole("button", { name: "Evaluators" });
    await user.click(evaluators);
    expect(insightsHeading).toHaveFocus();

    const insightsSection = screen.getByRole("region", { name: "Latest Research Insights" });
    const insightCards = within(insightsSection).getAllByRole("article");
    expect(insightCards).toHaveLength(3);
    for (const card of insightCards) {
      const tablist = within(card).getByRole("tablist");
      const selectedTabs = within(tablist).getAllByRole("tab", { selected: true });
      expect(selectedTabs).toHaveLength(1);
      expect(within(tablist).getByRole("tab", { selected: true })).toHaveTextContent(
        /Research|Spec|Formalization/u,
      );
    }

    const settings = controls.getByRole("button", { name: "Settings" });
    expect(settings).not.toBeDisabled();
    await user.click(settings);
    expect(settings).toHaveAttribute("aria-expanded", "true");
    const panel = screen.getByRole("region", { name: "Reference mode settings" });
    expect(panel).toBeInTheDocument();
    expect(within(panel).getByText(/No live data is connected\./)).toBeInTheDocument();
    expect(within(panel).getByText(/Cmd\/Ctrl\+K/)).toBeInTheDocument();
    expect(within(panel).getByText(/Shift\+Enter/)).toBeInTheDocument();
    await user.click(within(panel).getByRole("button", { name: "Close settings" }));
    expect(panel).not.toBeInTheDocument();
    expect(settings).toHaveFocus();

    await user.click(
      controls.getByRole("button", { name: "Open Harp knowledge Atlas" }),
    );
    expect(onNavigate).toHaveBeenCalledWith("#knowledge");
  });

  it("implements per-card roving tabs with keyboard wrapping, focus movement, and correct ARIA linkage", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard snapshot={workstreamsReference} onNavigate={vi.fn()} />,
    );

    const insights = screen.getByRole("region", { name: "Latest Research Insights" });
    const cards = within(insights).getAllByRole("article");
    expect(cards).toHaveLength(3);

    const first = cards[0] as HTMLElement;
    const second = cards[1] as HTMLElement;
    const firstTablist = within(first).getByRole("tablist");
    const secondTablist = within(second).getByRole("tablist");

    const firstResearch = within(firstTablist).getByRole("tab", { name: "Research" });
    const firstSpec = within(firstTablist).getByRole("tab", { name: "Spec" });
    const firstFormal = within(firstTablist).getByRole("tab", { name: "Formalization" });
    const secondResearch = within(secondTablist).getByRole("tab", { name: "Research" });

    expect(firstResearch).toHaveAttribute("aria-selected", "true");
    expect(firstResearch).toHaveAttribute("tabindex", "0");
    expect(firstSpec).toHaveAttribute("tabindex", "-1");
    expect(firstFormal).toHaveAttribute("tabindex", "-1");
    expect(secondResearch).toHaveAttribute("aria-selected", "true");

    firstResearch.focus();
    await user.keyboard("{ArrowRight}");
    expect(firstSpec).toHaveAttribute("aria-selected", "true");
    expect(firstSpec).toHaveFocus();
    expect(secondResearch).toHaveAttribute("aria-selected", "true");

    await user.keyboard("{ArrowLeft}");
    expect(firstResearch).toHaveAttribute("aria-selected", "true");
    expect(firstResearch).toHaveFocus();
    expect(secondResearch).toHaveAttribute("aria-selected", "true");

    await user.keyboard("{ArrowLeft}");
    expect(firstFormal).toHaveAttribute("aria-selected", "true");
    expect(firstFormal).toHaveFocus();
    expect(secondResearch).toHaveAttribute("aria-selected", "true");

    await user.keyboard("{ArrowRight}");
    expect(firstResearch).toHaveAttribute("aria-selected", "true");
    expect(firstResearch).toHaveFocus();

    await user.keyboard("{End}");
    expect(firstFormal).toHaveAttribute("aria-selected", "true");
    expect(firstFormal).toHaveFocus();

    await user.keyboard("{Home}");
    expect(firstResearch).toHaveAttribute("aria-selected", "true");
    expect(firstResearch).toHaveFocus();

    expect(firstResearch).toHaveAttribute("tabindex", "0");
    expect(firstSpec).toHaveAttribute("tabindex", "-1");
    expect(firstFormal).toHaveAttribute("tabindex", "-1");

    const researchPanelId = firstResearch.getAttribute("aria-controls");
    expect(researchPanelId).not.toBeNull();
    expect(firstResearch).toHaveAttribute("id");
    expect(firstResearch).toHaveAttribute("aria-controls", researchPanelId ?? "");

    const panel = within(first).getByRole("tabpanel");
    expect(panel).toHaveAttribute("id", researchPanelId ?? "");
    expect(panel).toHaveAttribute(
      "aria-labelledby",
      firstResearch.getAttribute("id") ?? "",
    );
  });

  it("chooses the blocked focus strip from the lowest numeric blocked priority", async () => {
    const onNavigate = vi.fn();
    const user = userEvent.setup();
    const snapshot: WorkstreamsSnapshot = {
      ...workstreamsReference,
      workstreams: workstreamsReference.workstreams.map((workstream) => {
        if (workstream.id === "crouzeix-conjecture") {
          return {
            ...workstream,
            status: "blocked",
            priority: 7,
            blocker: "Publication review is waiting on the external packet.",
            href: "#crouzeix-conjecture",
            stages: workstream.stages.map((stage, index) =>
              index === workstream.stages.length - 1
                ? {
                    ...stage,
                    status: "blocked",
                    detail: "Publication review is waiting on the external packet",
                  }
                : stage,
            ),
          };
        }
        if (workstream.id === "autodiff-geometry") {
          return {
            ...workstream,
            priority: 2,
          };
        }
        return workstream;
      }),
    };

    render(<WorkstreamsDashboard snapshot={snapshot} onNavigate={onNavigate} />);

    const focusStrip = screen.getByRole("region", { name: "Blocked workstream focus" });
    expect(
      within(focusStrip).getByRole("heading", { level: 2, name: "Autodiff Geometry" }),
    ).toBeInTheDocument();
    expect(
      within(focusStrip).getByText(
        "The source-to-curriculum map is incomplete beyond finite-coordinate identities.",
      ),
    ).toBeInTheDocument();

    await user.click(within(focusStrip).getByRole("button", { name: "FOCUS NOW" }));
    expect(onNavigate).toHaveBeenCalledWith("#mathematical-foundations");
  });

  it("renders agent velocity text, accessible chart metadata, and the exact 12-sample disclosure table", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    const velocityRegion = screen.getByRole("region", { name: "Agent velocity" });
    const velocitySummary = velocityRegion.querySelector(".agent-velocity__summary");
    expect(velocitySummary).not.toBeNull();
    expect(within(velocityRegion).getByText("runs / hour")).toBeInTheDocument();
    expect(within(velocityRegion).getByText("12h")).toBeInTheDocument();
    expect(within(velocityRegion).getByText("+18%")).toBeInTheDocument();
    expect(within(velocitySummary as HTMLElement).getByText("2")).toBeInTheDocument();
    expect(within(velocitySummary as HTMLElement).getByText("10")).toBeInTheDocument();
    expect(
      within(velocitySummary as HTMLElement).getByText(
        "Reference throughput rises from 2 to 10 runs per hour.",
      ),
    ).toBeInTheDocument();
    expect(within(velocityRegion).getByText("Online at snapshot")).toBeInTheDocument();
    expect(within(velocityRegion).getByText("Harp agent")).toBeInTheDocument();

    const chart = within(velocityRegion).getByRole("img", {
      name: "Agent velocity over 12 hours",
    });
    expect(chart.querySelector("title")).toHaveTextContent("Agent velocity over 12 hours");
    expect(chart.querySelector("desc")).toHaveTextContent(
      "Reference throughput rises from 2 to 10 runs per hour.",
    );

    const circles = within(velocityRegion).getAllByLabelText(
      /\d{2}:\d{2}: \d+ runs \/ hour/u,
    );
    expect(circles).toHaveLength(12);
    expect(circles[0]).toHaveAttribute("aria-label", "05:00: 2 runs / hour");
    expect(circles[11]).toHaveAttribute("aria-label", "16:00: 10 runs / hour");

    const samplesSummary = within(velocityRegion)
      .getByText("View 12-hour samples")
      .closest("summary");
    expect(samplesSummary).not.toBeNull();
    await user.click(samplesSummary as HTMLElement);
    const rows = within(velocityRegion).getAllByRole("row");
    expect(rows).toHaveLength(13);
    expect(within(rows[1]).getByText("05:00")).toBeInTheDocument();
    expect(within(rows[1]).getByText("2")).toBeInTheDocument();
    expect(within(rows[12]).getByText("16:00")).toBeInTheDocument();
    expect(within(rows[12]).getByText("10")).toBeInTheDocument();
  });

  it("focuses recent runs and announces the run count", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "View Recent Runs" }));

    const heading = screen.getByRole("heading", { name: "Recent Runs" });
    expect(heading).toHaveFocus();
    expect(screen.getByRole("status")).toHaveTextContent("3 recent runs");
  });

  it("shows an inline error for an empty local command", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Queue command" }));

    expect(screen.getByRole("status")).toHaveTextContent("Command must contain 1–240 characters");
    expect(screen.getByLabelText("Local command")).toHaveValue("");
  });

  it("queues a local command from the form button", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.type(screen.getByLabelText("Local command"), "verify Jin route");
    await user.click(screen.getByRole("button", { name: "Queue command" }));

    expect(screen.getByRole("status")).toHaveTextContent("Queued locally");
    expect(screen.getByLabelText("Local command")).toHaveFocus();
    expect(screen.getByLabelText("Local command")).toHaveValue("");
    expect(screen.getByText("Queued locally")).toBeInTheDocument();
    expect(screen.getByText("verify Jin route")).toBeInTheDocument();
  });

  it("queues a local command on Shift+Enter", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.type(
      screen.getByLabelText("Local command"),
      "verify LS route{shift>}{enter}{/shift}",
    );

    expect(screen.getByRole("status")).toHaveTextContent("Queued locally");
    expect(screen.getByText("verify LS route")).toBeInTheDocument();
  });

  it("renders queued local runs before the fixed repository runs", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.type(screen.getByLabelText("Local command"), "verify Harp route");
    await user.click(screen.getByRole("button", { name: "Queue command" }));

    const list = screen.getByRole("list", { name: "Recent runs list" });
    const items = within(list).getAllByRole("listitem");
    expect(within(items[0]).getByText("Queued locally")).toBeInTheDocument();
    expect(within(items[0]).getByText("verify Harp route")).toBeInTheDocument();
    expect(within(items[1]).getByText("Jin route")).toBeInTheDocument();
  });

  it("does not expose execution affordances for queued or fixed runs", async () => {
    const user = userEvent.setup();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.type(screen.getByLabelText("Local command"), "verify queue");
    await user.click(screen.getByRole("button", { name: "Queue command" }));

    const recentRuns = screen.getByRole("list", { name: "Recent runs list" });
    expect(within(recentRuns).queryByRole("button")).toBeNull();
    expect(within(recentRuns).queryByRole("link")).toBeNull();
  });

  it("persists queued commands across unmount and remount", async () => {
    const user = userEvent.setup();
    const first = render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.type(
      screen.getByLabelText("Local command"),
      "verify Jin route{shift>}{enter}{/shift}",
    );
    expect(screen.getByRole("status")).toHaveTextContent("Queued locally");
    first.unmount();

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    expect(screen.getByText("verify Jin route")).toBeInTheDocument();
  });

  it("retains the command and reports storage errors when sessionStorage.setItem fails", async () => {
    const user = userEvent.setup();
    const setItemSpy = vi
      .spyOn(Storage.prototype, "setItem")
      .mockImplementation(() => {
        throw new Error("setItem failed");
      });

    render(
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={vi.fn()}
      />,
    );

    await user.type(screen.getByLabelText("Local command"), "verify blocked route");
    await user.click(screen.getByRole("button", { name: "Queue command" }));

    expect(screen.getByRole("status")).toHaveTextContent("setItem failed");
    expect(screen.getByLabelText("Local command")).toHaveValue("verify blocked route");

    setItemSpy.mockRestore();
  });

  it("survives inaccessible sessionStorage and reports that the local queue is unavailable", async () => {
    const user = userEvent.setup();
    const storageOwner = [window, Window.prototype, Object.getPrototypeOf(window)].find(
      (candidate) =>
        Object.getOwnPropertyDescriptor(candidate, "sessionStorage") !== undefined,
    );
    expect(storageOwner).toBeDefined();
    const originalDescriptor = Object.getOwnPropertyDescriptor(
      storageOwner as object,
      "sessionStorage",
    );
    expect(originalDescriptor).toBeDefined();

    Object.defineProperty(storageOwner as object, "sessionStorage", {
      configurable: true,
      get() {
        throw new DOMException("Blocked", "SecurityError");
      },
    });

    try {
      render(
        <WorkstreamsDashboard
          snapshot={workstreamsReference}
          onNavigate={vi.fn()}
        />,
      );

      await user.type(screen.getByLabelText("Local command"), "verify guarded queue");
      await user.click(screen.getByRole("button", { name: "Queue command" }));

      expect(screen.getByRole("status")).toHaveTextContent("Local queue unavailable");
      expect(screen.getByLabelText("Local command")).toHaveValue("verify guarded queue");
      expect(screen.queryByText("Queued locally")).toBeNull();
    } finally {
      if (originalDescriptor) {
        Object.defineProperty(storageOwner as object, "sessionStorage", originalDescriptor);
      }
    }
  });
});
