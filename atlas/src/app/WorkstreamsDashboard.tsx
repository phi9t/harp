import {
  Activity,
  ArrowRight,
  Bot,
  History,
  LayoutDashboard,
  Library,
  Search,
  Settings,
  ShieldCheck,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

import type { DashboardHref, WorkstreamsSnapshot } from "../content/workstreams";
import { AgentVelocityCard } from "./AgentVelocityCard";
import type { InsightArtifactKind, InsightSnapshot, WorkstreamSnapshot } from "../content/workstreams";
import { ResearchInsights } from "./ResearchInsights";
import { WatchlistTable } from "./WatchlistTable";
import { WorkstreamCard } from "./WorkstreamCard";

function selectionsFor(
  insights: readonly InsightSnapshot[],
  choose: (insight: InsightSnapshot) => InsightArtifactKind,
): Record<string, InsightArtifactKind> {
  return insights.reduce<Record<string, InsightArtifactKind>>((result, insight) => {
    result[insight.id] = choose(insight);
    return result;
  }, {});
}

function searchTerms(workstream: WorkstreamSnapshot): string {
  return [
    workstream.title,
    workstream.status,
    workstream.progressLabel,
    workstream.blocker ?? "",
    workstream.nextAction,
    ...workstream.stages.flatMap((stage) => [stage.name, stage.status]),
  ].join(" ").toLocaleLowerCase();
}

export function WorkstreamsDashboard({
  snapshot,
  onNavigate,
}: {
  snapshot: WorkstreamsSnapshot;
  onNavigate: (href: DashboardHref) => void;
}) {
  const headingRef = useRef<HTMLHeadingElement>(null);
  const recentRunsHeadingRef = useRef<HTMLHeadingElement>(null);
  const recentlyActiveRef = useRef<HTMLOListElement>(null);
  const insightsHeadingRef = useRef<HTMLHeadingElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const settingsButtonRef = useRef<HTMLButtonElement>(null);

  const [query, setQuery] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [selectedArtifacts, setSelectedArtifacts] = useState(() =>
    selectionsFor(snapshot.insights, () => "research"),
  );

  const normalizedQuery = query.trim().toLocaleLowerCase();
  const visibleWorkstreams = useMemo(
    () =>
      snapshot.workstreams.filter(
        (workstream) =>
          normalizedQuery === "" || searchTerms(workstream).includes(normalizedQuery),
      ),
    [snapshot.workstreams, normalizedQuery],
  );
  const visibleIds = useMemo(
    () => new Set(visibleWorkstreams.map((workstream) => workstream.id)),
    [visibleWorkstreams],
  );
  const visibleWatchlist = useMemo(
    () => snapshot.watchlist.filter((row) => visibleIds.has(row.workstreamId)),
    [snapshot.watchlist, visibleIds],
  );

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey)) {
        return;
      }
      if (event.key.toLocaleLowerCase() !== "k") {
        return;
      }
      event.preventDefault();
      searchRef.current?.focus();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const blockedFocus = [...snapshot.workstreams]
    .filter((workstream) => workstream.status === "blocked")
    .sort((left, right) => left.priority - right.priority)[0] ?? null;
  const recentlyActive = [...snapshot.workstreams]
    .sort((left, right) => left.priority - right.priority)
    .slice(0, 3);

  const clearSearch = () => {
    setQuery("");
    searchRef.current?.focus();
  };

  const selectEvaluators = () => {
    setSelectedArtifacts(
      selectionsFor(snapshot.insights, (insight) => insight.evaluatorArtifact),
    );
    insightsHeadingRef.current?.focus();
  };

  const toggleSettings = () => {
    setSettingsOpen((current) => !current);
  };

  const closeSettings = () => {
    setSettingsOpen(false);
    settingsButtonRef.current?.focus();
  };

  return (
    <div className="workstreams-shell">
      <aside
        className="workstreams-sidebar"
        aria-label="Workstreams navigation"
      >
        <nav aria-label="Workstreams navigation">
          <button
            type="button"
            className="workstreams-nav__button"
            onClick={() => headingRef.current?.focus()}
          >
            <LayoutDashboard aria-hidden="true" />
            <span>Workstreams</span>
          </button>
          <button
            type="button"
            className="workstreams-nav__button"
            onClick={() => recentRunsHeadingRef.current?.focus()}
          >
            <History aria-hidden="true" />
            <span>Runs</span>
          </button>
          <button
            type="button"
            className="workstreams-nav__button"
            onClick={() => recentlyActiveRef.current?.focus()}
          >
            <Bot aria-hidden="true" />
            <span>Agents</span>
          </button>
          <button
            type="button"
            className="workstreams-nav__button"
            onClick={selectEvaluators}
          >
            <ShieldCheck aria-hidden="true" />
            <span>Evaluators</span>
          </button>
          <button
            type="button"
            className="workstreams-nav__button"
            aria-label="Open Harp knowledge Atlas"
            onClick={() => onNavigate("#knowledge")}
          >
            <Library aria-hidden="true" />
            <span>Knowledge</span>
          </button>
          <button
            type="button"
            className="workstreams-nav__button"
            aria-controls="reference-settings"
            aria-expanded={settingsOpen}
            onClick={toggleSettings}
            ref={settingsButtonRef}
          >
            <Settings aria-hidden="true" />
            <span>Settings</span>
          </button>
          <h2>Recently Active</h2>
          <ol
            className="workstreams-nav__recent"
            aria-label="Recently Active list"
            ref={recentlyActiveRef}
            tabIndex={-1}
          >
            {recentlyActive.map((workstream) => (
              <li key={workstream.id}>{workstream.title}</li>
            ))}
          </ol>
        </nav>
      </aside>
      <main className="workstreams-main">
        <header className="workstreams-header">
          <h1 ref={headingRef} tabIndex={-1}>Harp Workstreams</h1>
          <p>REFERENCE SNAPSHOT</p>
          <p>{snapshot.observedAt}</p>
          <p className="workstreams-health">
            <ShieldCheck aria-hidden="true" />
            <span>Online at snapshot</span>
          </p>
          <div className="workstreams-search">
            <label htmlFor="workstreams-search">Search</label>
            <div>
              <Search aria-hidden="true" />
              <input
                id="workstreams-search"
                type="search"
                role="searchbox"
                aria-label="Search"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                ref={searchRef}
              />
              <span>Cmd K</span>
              {normalizedQuery !== "" ? (
                <button type="button" onClick={clearSearch} aria-label="Clear search">
                  Clear
                </button>
              ) : null}
            </div>
          </div>
        </header>
        {blockedFocus ? (
          <section
            className="workstreams-focus"
            aria-label="Blocked workstream focus"
          >
            <p>FOCUS NOW</p>
            <h2>{blockedFocus.title}</h2>
            <p>{blockedFocus.blocker}</p>
            <button
              type="button"
              className="workstreams-focus__button"
              onClick={() => onNavigate(blockedFocus.href)}
            >
              <Activity aria-hidden="true" />
              <span>FOCUS NOW</span>
              <ArrowRight aria-hidden="true" />
            </button>
          </section>
        ) : null}
        {visibleWorkstreams.length === 0 ? (
          <section className="workstreams-grid" aria-label="Workstreams">
            <p>No workstreams match the current search.</p>
          </section>
        ) : (
          <section className="workstreams-grid" aria-label="Workstreams">
            {visibleWorkstreams.map((workstream) => (
              <WorkstreamCard key={workstream.id} workstream={workstream} />
            ))}
          </section>
        )}
        <AgentVelocityCard
          agent={snapshot.agent}
          recentRunsHeadingRef={recentRunsHeadingRef}
        />
        <ResearchInsights
          insights={snapshot.insights}
          selectedArtifacts={selectedArtifacts}
          onSelect={(insightId, kind) =>
            setSelectedArtifacts((current) => ({ ...current, [insightId]: kind }))
          }
          onNavigate={onNavigate}
          headingRef={insightsHeadingRef}
        />
        <WatchlistTable
          rows={visibleWatchlist}
          workstreams={snapshot.workstreams}
          onNavigate={onNavigate}
          empty={visibleWorkstreams.length === 0}
        />
        {settingsOpen ? (
          <section
            id="reference-settings"
            aria-label="Reference mode settings"
            className="reference-settings"
          >
            <p>No live data is connected.</p>
            <p>Cmd/Ctrl+K focuses search.</p>
            <p>Shift+Enter queues a local command.</p>
            <button type="button" onClick={closeSettings} aria-label="Close settings">
              Close settings
            </button>
          </section>
        ) : null}
      </main>
    </div>
  );
}
