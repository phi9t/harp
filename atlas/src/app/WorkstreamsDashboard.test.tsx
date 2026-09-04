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

    await user.click(runsButton);
    expect(screen.getByRole("heading", { name: "Recent Runs" })).toHaveFocus();

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
    expect(
      within(agentVelocityRegion).getByRole("heading", { level: 2, name: "Harp agent" }),
    ).toBeInTheDocument();
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
