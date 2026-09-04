import {
  Activity,
  ArrowRight,
  Bot,
  History,
  LayoutDashboard,
  Library,
  Settings,
  ShieldCheck,
} from "lucide-react";
import { useRef } from "react";

import type { DashboardHref, WorkstreamsSnapshot } from "../content/workstreams";
import { AgentVelocityCard } from "./AgentVelocityCard";
import { WorkstreamCard } from "./WorkstreamCard";

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

  const blockedFocus = [...snapshot.workstreams]
    .filter((workstream) => workstream.status === "blocked")
    .sort((left, right) => left.priority - right.priority)[0] ?? null;
  const recentlyActive = [...snapshot.workstreams]
    .sort((left, right) => left.priority - right.priority)
    .slice(0, 3);

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
            onClick={() => onNavigate("#knowledge")}
          >
            <Library aria-hidden="true" />
            <span>Knowledge</span>
          </button>
          <button
            type="button"
            className="workstreams-nav__button"
            disabled
            aria-disabled="true"
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
        <section
          className="workstreams-grid"
          aria-label="Workstreams"
        >
          {snapshot.workstreams.map((workstream) => (
            <WorkstreamCard key={workstream.id} workstream={workstream} />
          ))}
        </section>
        <AgentVelocityCard
          agent={snapshot.agent}
          recentRunsHeadingRef={recentRunsHeadingRef}
        />
        <section id="insights" aria-label="Insights" title="Insights" />
      </main>
    </div>
  );
}
