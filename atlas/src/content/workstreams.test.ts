import { describe, expect, it } from "vitest";

import {
  parseWorkstreamsSnapshot,
  workstreamsReference,
} from "./workstreams";

describe("workstreams snapshot boundary", () => {
  it("accepts the fixed reference snapshot", () => {
    const snapshot = parseWorkstreamsSnapshot(workstreamsReference);

    expect(snapshot.schemaVersion).toBe("harp-workstreams/v1");
    expect(snapshot.source).toBe("repository-reference");
    expect(snapshot.workstreams.map((workstream) => workstream.title)).toEqual([
      "Crouzeix Conjecture",
      "Autodiff Geometry",
      "Agentic Research",
      "NNG4 / Foundations",
    ]);
    expect(snapshot.agent.velocity.samples).toHaveLength(12);
    expect(snapshot.insights).toHaveLength(3);
    expect(snapshot.watchlist).toHaveLength(4);
  });

  it("rejects duplicate workstream ids", () => {
    const [first, second, ...rest] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [first, { ...second, id: first.id }, ...rest],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/unique/i);
  });

  it("rejects a workstream with too few stages", () => {
    const [first, ...rest] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [{ ...first, stages: first.stages.slice(0, 3) }, ...rest],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/stages/i);
  });

  it("rejects a workstream with empty stages", () => {
    const [first, ...rest] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [{ ...first, stages: [] }, ...rest],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/stages/i);
  });

  it("rejects a blocked workstream without blocker evidence", () => {
    const [first, second, ...rest] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [first, { ...second, blocker: null }, ...rest],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/blocker/i);
  });

  it("rejects a blocked workstream without a blocked stage", () => {
    const [first, second, ...rest] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [
        first,
        {
          ...second,
          stages: second.stages.map((stage) =>
            stage.status === "blocked"
              ? { ...stage, status: "queued" as const }
              : stage,
          ),
        },
        ...rest,
      ],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/blocked stage/i);
  });

  it("rejects a complete-local workstream with an active stage", () => {
    const [first, second, third, fourth] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [
        first,
        second,
        third,
        {
          ...fourth,
          stages: [
            { ...fourth.stages[0], status: "active" as const },
            ...fourth.stages.slice(1),
          ],
        },
      ],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/complete-local/i);
  });

  it("rejects a complete-local workstream with a blocker", () => {
    const [first, second, third, fourth] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [
        first,
        second,
        third,
        { ...fourth, blocker: "Unexpected blocker" },
      ],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/complete-local/i);
  });

  it("rejects an empty next action", () => {
    const [first, ...rest] = workstreamsReference.workstreams;
    const input = {
      ...workstreamsReference,
      workstreams: [{ ...first, nextAction: "   " }, ...rest],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/nextAction/i);
  });

  it("rejects the wrong number of velocity samples", () => {
    const input = {
      ...workstreamsReference,
      agent: {
        ...workstreamsReference.agent,
        velocity: {
          ...workstreamsReference.agent.velocity,
          samples: workstreamsReference.agent.velocity.samples.slice(0, 11),
        },
      },
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/12/i);
  });

  it("rejects negative or non-finite velocity samples", () => {
    const negativeInput = {
      ...workstreamsReference,
      agent: {
        ...workstreamsReference.agent,
        velocity: {
          ...workstreamsReference.agent.velocity,
          samples: [
            {
              ...workstreamsReference.agent.velocity.samples[0],
              value: -1,
            },
            ...workstreamsReference.agent.velocity.samples.slice(1),
          ],
        },
      },
    };
    const nonFiniteInput = {
      ...workstreamsReference,
      agent: {
        ...workstreamsReference.agent,
        velocity: {
          ...workstreamsReference.agent.velocity,
          samples: [
            {
              ...workstreamsReference.agent.velocity.samples[0],
              value: Number.POSITIVE_INFINITY,
            },
            ...workstreamsReference.agent.velocity.samples.slice(1),
          ],
        },
      },
    };

    expect(() => parseWorkstreamsSnapshot(negativeInput)).toThrow(/sample/i);
    expect(() => parseWorkstreamsSnapshot(nonFiniteInput)).toThrow(/sample/i);
  });

  it("rejects duplicate insight ids", () => {
    const [first, second, ...rest] = workstreamsReference.insights;
    const input = {
      ...workstreamsReference,
      insights: [first, { ...second, id: first.id }, ...rest],
    };

    expect(() => parseWorkstreamsSnapshot(input)).toThrow(/unique/i);
  });

  it("rejects malformed datetimes and artifact dates", () => {
    const malformedObservedAt = {
      ...workstreamsReference,
      observedAt: "2026-09-03T16:00:00Z",
    };
    const malformedQueuedAt = {
      ...workstreamsReference,
      agent: {
        ...workstreamsReference.agent,
        runs: [
          ...workstreamsReference.agent.runs,
          {
            kind: "queued" as const,
            id: "queue-red",
            command: "node_modules/.bin/vitest run src/content/workstreams.test.ts",
            queuedAt: "2026-09-03 16:00:00.000Z",
          },
        ],
      },
    };
    const malformedArtifactDate = {
      ...workstreamsReference,
      insights: [
        {
          ...workstreamsReference.insights[0],
          artifacts: [
            {
              ...workstreamsReference.insights[0].artifacts[0],
              updatedAt: "2026-9-03",
            },
            ...workstreamsReference.insights[0].artifacts.slice(1),
          ],
        },
        ...workstreamsReference.insights.slice(1),
      ],
    };

    expect(() => parseWorkstreamsSnapshot(malformedObservedAt)).toThrow(
      /observedAt/i,
    );
    expect(() => parseWorkstreamsSnapshot(malformedQueuedAt)).toThrow(/queuedAt/i);
    expect(() => parseWorkstreamsSnapshot(malformedArtifactDate)).toThrow(
      /updatedAt/i,
    );
  });

  it("rejects missing or duplicate artifact kinds", () => {
    const missingKindInput = {
      ...workstreamsReference,
      insights: [
        {
          ...workstreamsReference.insights[0],
          artifacts: workstreamsReference.insights[0].artifacts.slice(0, 2),
        },
        ...workstreamsReference.insights.slice(1),
      ],
    };
    const duplicateKindInput = {
      ...workstreamsReference,
      insights: [
        {
          ...workstreamsReference.insights[0],
          artifacts: [
            workstreamsReference.insights[0].artifacts[0],
            {
              ...workstreamsReference.insights[0].artifacts[1],
              kind: "research" as const,
            },
            workstreamsReference.insights[0].artifacts[2],
          ],
        },
        ...workstreamsReference.insights.slice(1),
      ],
    };

    expect(() => parseWorkstreamsSnapshot(missingKindInput)).toThrow(/artifact/i);
    expect(() => parseWorkstreamsSnapshot(duplicateKindInput)).toThrow(
      /artifact/i,
    );
  });

  it("rejects invalid or external hrefs", () => {
    const invalidHrefInput = {
      ...workstreamsReference,
      workstreams: [
        {
          ...workstreamsReference.workstreams[0],
          href: "#outside-scope",
        },
        ...workstreamsReference.workstreams.slice(1),
      ],
    };
    const externalHrefInput = {
      ...workstreamsReference,
      watchlist: [
        {
          ...workstreamsReference.watchlist[0],
          href: "https://example.com",
        },
        ...workstreamsReference.watchlist.slice(1),
      ],
    };

    expect(() => parseWorkstreamsSnapshot(invalidHrefInput)).toThrow(/href/i);
    expect(() => parseWorkstreamsSnapshot(externalHrefInput)).toThrow(/href/i);
  });

  it("rejects broken watchlist references and duplicate watchlist or run ids", () => {
    const brokenReferenceInput = {
      ...workstreamsReference,
      watchlist: [
        {
          ...workstreamsReference.watchlist[0],
          workstreamId: "missing-workstream",
        },
        ...workstreamsReference.watchlist.slice(1),
      ],
    };
    const duplicateWatchlistInput = {
      ...workstreamsReference,
      watchlist: [
        workstreamsReference.watchlist[0],
        {
          ...workstreamsReference.watchlist[1],
          id: workstreamsReference.watchlist[0].id,
        },
        ...workstreamsReference.watchlist.slice(2),
      ],
    };
    const duplicateRunInput = {
      ...workstreamsReference,
      agent: {
        ...workstreamsReference.agent,
        runs: [
          workstreamsReference.agent.runs[0],
          {
            ...workstreamsReference.agent.runs[1],
            id: workstreamsReference.agent.runs[0].id,
          },
          ...workstreamsReference.agent.runs.slice(2),
        ],
      },
    };

    expect(() => parseWorkstreamsSnapshot(brokenReferenceInput)).toThrow(
      /workstream/i,
    );
    expect(() => parseWorkstreamsSnapshot(duplicateWatchlistInput)).toThrow(
      /unique/i,
    );
    expect(() => parseWorkstreamsSnapshot(duplicateRunInput)).toThrow(/unique/i);
  });
});
