import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
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

describe("WorkstreamsDashboard", () => {
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
    expect(screen.getByText("Online at snapshot")).toBeInTheDocument();

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
    const knowledgeButton = controls.getByRole("button", { name: "Knowledge" });
    const settingsControl = controls.getByRole("button", { name: "Settings" });
    expect(settingsControl).toBeDisabled();
    expect(controls.getByText("Recently Active")).toBeInTheDocument();
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

    const agentVelocity = screen.getByRole("region", { name: "Agent velocity" });
    await user.click(runsButton);
    expect(agentVelocity).toHaveFocus();

    const recentlyActive = screen.getByRole("list", { name: "Recently Active list" });
    await user.click(agentsButton);
    expect(recentlyActive).toHaveFocus();

    await user.click(knowledgeButton);
    expect(onNavigate).toHaveBeenCalledWith("#knowledge");

    await user.click(screen.getByRole("button", { name: "FOCUS NOW" }));
    expect(onNavigate).toHaveBeenCalledWith("#mathematical-foundations");

    const agentVelocityRegion = screen.getByRole("region", { name: "Agent velocity" });
    const insightsRegion = screen.getByRole("region", { name: "Insights" });
    expect(agentVelocityRegion).toHaveAttribute("id", "agent-velocity");
    expect(agentVelocityRegion).toHaveAttribute("title", "Agent velocity");
    expect(insightsRegion).toHaveAttribute("id", "insights");
    expect(insightsRegion).toHaveAttribute("title", "Insights");
    expect(agentVelocityRegion).toBeEmptyDOMElement();
    expect(insightsRegion).toBeEmptyDOMElement();
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
});
