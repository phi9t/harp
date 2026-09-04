import { useRef, type KeyboardEvent, type RefObject } from "react";

import type {
  DashboardHref,
  InsightArtifact,
  InsightArtifactKind,
  InsightSnapshot,
} from "../content/workstreams";

const artifactKinds = ["research", "spec", "formalization"] as const;

const artifactLabels: Record<
  (typeof artifactKinds)[number],
  "Research" | "Spec" | "Formalization"
> = {
  research: "Research",
  spec: "Spec",
  formalization: "Formalization",
};

function nextTabIndex(current: number, key: string): number | null {
  if (key === "ArrowRight") {
    return (current + 1) % artifactKinds.length;
  }
  if (key === "ArrowLeft") {
    return (current - 1 + artifactKinds.length) % artifactKinds.length;
  }
  if (key === "Home") {
    return 0;
  }
  if (key === "End") {
    return artifactKinds.length - 1;
  }
  return null;
}

function artifactByKind(
  insight: InsightSnapshot,
  kind: InsightArtifactKind,
): InsightArtifact {
  const artifact = insight.artifacts.find((candidate) => candidate.kind === kind);
  if (!artifact) {
    throw new Error(`Insight ${insight.id} missing artifact kind ${kind}`);
  }
  return artifact;
}

export type ResearchInsightsProps = {
  insights: readonly InsightSnapshot[];
  selectedArtifacts: Record<string, InsightArtifactKind>;
  onSelect: (insightId: string, kind: InsightArtifactKind) => void;
  onNavigate: (href: DashboardHref) => void;
  headingRef: RefObject<HTMLHeadingElement | null>;
};

type InsightCardProps = {
  insight: InsightSnapshot;
  selectedKind: InsightArtifactKind;
  onSelect: (kind: InsightArtifactKind) => void;
  onNavigate: (href: DashboardHref) => void;
};

function InsightCard({ insight, selectedKind, onSelect, onNavigate }: InsightCardProps) {
  const buttonRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const panelId = `${insight.id}-${selectedKind}-panel`;
  const tabId = `${insight.id}-${selectedKind}-tab`;
  const artifact = artifactByKind(insight, selectedKind);

  const onTabKeyDown = (event: KeyboardEvent<HTMLButtonElement>, currentIndex: number) => {
    const nextIndex = nextTabIndex(currentIndex, event.key);
    if (nextIndex === null) {
      return;
    }
    event.preventDefault();
    const kind = artifactKinds[nextIndex];
    onSelect(kind);
    buttonRefs.current[nextIndex]?.focus();
  };

  return (
    <article className="insight-card" aria-labelledby={`${insight.id}-title`}>
      <header className="insight-card__header">
        <h3 id={`${insight.id}-title`}>{insight.title}</h3>
      </header>
      <div role="tablist" aria-label={`${insight.title} artifacts`}>
        {artifactKinds.map((kind, index) => {
          const label = artifactLabels[kind];
          const isSelected = kind === selectedKind;
          return (
            <button
              key={kind}
              type="button"
              role="tab"
              id={`${insight.id}-${kind}-tab`}
              aria-controls={`${insight.id}-${kind}-panel`}
              aria-selected={isSelected}
              tabIndex={isSelected ? 0 : -1}
              ref={(element) => {
                buttonRefs.current[index] = element;
              }}
              onClick={() => onSelect(kind)}
              onKeyDown={(event) => onTabKeyDown(event, index)}
            >
              {label}
            </button>
          );
        })}
      </div>
      <div
        role="tabpanel"
        id={panelId}
        aria-labelledby={tabId}
        className="insight-card__panel"
      >
        <p className="insight-card__meta">
          <span>{artifactLabels[artifact.kind]}</span>
          <span>{artifact.updatedAt}</span>
          {artifact.evidence === "missing" ? (
            <span className="insight-card__missing">MISSING</span>
          ) : null}
        </p>
        <p>{artifact.summary}</p>
        <button type="button" onClick={() => onNavigate(artifact.href)}>
          Open artifact
        </button>
      </div>
    </article>
  );
}

export function ResearchInsights({
  insights,
  selectedArtifacts,
  onSelect,
  onNavigate,
  headingRef,
}: ResearchInsightsProps) {
  return (
    <section id="insights" aria-labelledby="insights-title">
      <h2 id="insights-title" ref={headingRef} tabIndex={-1}>
        Latest Research Insights
      </h2>
      <div className="insight-grid">
        {insights.map((insight) => (
          <InsightCard
            key={insight.id}
            insight={insight}
            selectedKind={selectedArtifacts[insight.id] ?? "research"}
            onSelect={(kind) => onSelect(insight.id, kind)}
            onNavigate={onNavigate}
          />
        ))}
      </div>
    </section>
  );
}
