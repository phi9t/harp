import { useEffect, useRef, useState } from "react";

import {
  canonicalChapters,
  canonicalCorpus,
  canonicalReaderRoutes,
} from "../content/canonical";
import { workstreamsReference } from "../content/workstreams";
import type {
  CanonicalDocument,
  ConceptId,
  DocumentId,
} from "../content/types";
import "../styles/tokens.css";
import "../styles/layout.css";
import "../styles/components.css";
import "../styles/research.css";
import { ResearchHome, areaLabels } from "./ResearchHome";
import { CanonicalDocumentView, ChapterReader } from "./ChapterReader";
import { DiagnosticWorkbench } from "./DiagnosticWorkbench";
import {
  formatRoute,
  parseRoute,
  type AtlasRoute,
} from "./routes";
import { LineageLoom } from "./LineageLoom";
import { LessonRunner } from "./LessonRunner";
import { KnowledgeHome } from "./KnowledgeHome";
import { SystemArticle } from "./SystemArticle";
import { SystemLibrary } from "./SystemLibrary";
import { WengReader } from "./WengReader";
import { WorkstreamsDashboard } from "./WorkstreamsDashboard";

function documentById(documentId: string): CanonicalDocument {
  const document = canonicalCorpus.documents.find(
    (candidate) => candidate.concept_id === documentId,
  );
  if (!document) {
    throw new Error(`Validated document disappeared: ${documentId}`);
  }
  return document;
}

function technicalSectionNavigationLabel(
  document: CanonicalDocument,
): string | undefined {
  return document.concept_id === "context-engineering-deep-dive"
    ? "Context engineering technical sections"
    : undefined;
}

function AuxiliaryDocumentReader({
  documentId,
  sectionId,
}: {
  documentId: DocumentId;
  sectionId: string | null;
}) {
  const document = documentById(documentId);
  return (
    <div className="route-reader">
      <CanonicalDocumentView
        document={document}
        sectionId={sectionId}
        sectionNavigationLabel={technicalSectionNavigationLabel(document)}
      />
    </div>
  );
}

function chapterConceptId(document: CanonicalDocument): ConceptId {
  const entry = canonicalCorpus.coverage.find(
    (candidate) =>
      candidate.coverage_depth === "chapter"
      && candidate.canonical_markdown_path === document.canonical_markdown_path,
  );
  if (!entry) {
    throw new Error(`Validated chapter coverage disappeared: ${document.concept_id}`);
  }
  return entry.concept_id;
}

function navigate(route: AtlasRoute): void {
  const hash = formatRoute(route);
  if (window.location.hash === hash) {
    window.dispatchEvent(new HashChangeEvent("hashchange"));
  } else {
    window.location.hash = hash;
  }
}

function currentLabel(route: AtlasRoute): string {
  switch (route.kind) {
    case "home":
      return "Home";
    case "area":
      return areaLabels[route.area];
    case "weng":
      return "Weng";
    case "workstreams":
      return "Workstreams";
    case "systems":
    case "system":
      return "Systems";
    case "lesson":
      return "Lessons";
    case "diagnose":
      return "Diagnose";
    case "chapter":
      return "Chapters";
    case "document":
      return "Technical reading";
    case "sources":
      return "Sources";
    case "legacy":
      if (route.routeId === "knowledge") {
        return "Knowledge";
      }
      return canonicalReaderRoutes.find(
        (candidate) => candidate.route.route_id === route.routeId,
      )?.route.label ?? route.routeId;
    default: {
      const exhaustive: never = route;
      return exhaustive;
    }
  }
}

export function AtlasApp() {
  const menuRef = useRef<HTMLDetailsElement>(null);
  const [route, setRoute] = useState<AtlasRoute>(() =>
    parseRoute(window.location.hash));

  useEffect(() => {
    if (!window.location.hash) {
      window.location.hash = formatRoute(parseRoute(""));
    }
    const readHash = () => {
      // The skip link is a native focus target, not an application route.
      if (window.location.hash === "#main-content") return;
      if (menuRef.current) menuRef.current.open = false;
      setRoute(parseRoute(window.location.hash));
      window.scrollTo?.({ top: 0 });
    };
    window.addEventListener("hashchange", readHash);
    return () => window.removeEventListener("hashchange", readHash);
  }, []);

  if (route.kind === "workstreams") {
    return (
      <WorkstreamsDashboard
        snapshot={workstreamsReference}
        onNavigate={(href) => {
          window.location.hash = href;
        }}
      />
    );
  }

  const sourceRoute = canonicalReaderRoutes.find(
    (candidate) => candidate.route.route_id === "sources",
  );
  if (!sourceRoute) {
    throw new Error("Validated source route disappeared");
  }
  return (
    <div className="atlas-shell">
      <a className="skip-link" href="#main-content" onClick={(event) => {
        event.preventDefault();
        document.getElementById("main-content")?.focus();
      }}>Skip to content</a>
      <header className="site-header reader-header">
        <a className="harp-wordmark" href="#home" aria-label="Harp home">harp<span>research atlas</span></a>
        <nav className="top-nav" aria-label="Atlas routes">
          {Object.entries(areaLabels).map(([area, label]) => (
            <a className="nav-button" key={area} href={`#explore/${area}`}
              aria-current={route.kind === "area" && route.area === area ? "page" : undefined}>
              {label}
            </a>
          ))}
          <a className="nav-button" href="#knowledge"
            aria-current={route.kind === "legacy" && route.routeId === "knowledge" ? "page" : undefined}>
            Library
          </a>
        </nav>
        <details className="reader-menu" ref={menuRef} onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.currentTarget.open = false;
            event.currentTarget.querySelector("summary")?.focus();
          }
        }}>
          <summary>More</summary>
          <nav aria-label="Reader tools">
            <button type="button" onClick={() => navigate({ kind: "weng", sectionId: canonicalCorpus.weng_sections[0].section_id })}>Weng</button>
            <button type="button" onClick={() => navigate({ kind: "systems" })}>Systems</button>
            <button type="button" onClick={() => navigate({ kind: "chapter", conceptId: chapterConceptId(canonicalChapters[0]) })}>Chapters</button>
            <button type="button" onClick={() => navigate({ kind: "lesson", lessonId: canonicalCorpus.lessons[0].lesson_id })}>Lessons</button>
            <button type="button" onClick={() => navigate({ kind: "diagnose", caseId: null })}>Diagnose</button>
            <button type="button" onClick={() => navigate({ kind: "sources" })}>Sources</button>
            {canonicalReaderRoutes.filter((candidate) => ["crouzeix-conjecture", "mathematical-foundations", "verified-coevolution"].includes(candidate.route.route_id)).map(({ route: entry }) => (
              <button key={entry.route_id} type="button" onClick={() => navigate({ kind: "legacy", routeId: entry.route_id })}>{entry.label}</button>
            ))}
            <button type="button" aria-label="Open Console / Workstreams" onClick={() => navigate({ kind: "workstreams" })}>Console / Workstreams</button>
          </nav>
        </details>
      </header>
      <div className="reader-location" role="status" aria-label="Current route">{currentLabel(route)}</div>
      <main id="main-content" tabIndex={-1}>
        {route.kind === "home" ? <ResearchHome /> : null}
        {route.kind === "area" ? <ResearchHome area={route.area} /> : null}
        {route.kind === "weng" ? (
          <WengReader
            sectionId={route.sectionId}
            onSelectSection={(sectionId) =>
              navigate({ kind: "weng", sectionId })}
            onSelectSystem={(systemId, returnTo) =>
              navigate({ kind: "system", systemId, returnTo })}
          />
        ) : null}
        {route.kind === "systems" ? (
          <SystemLibrary
            onSelectSystem={(systemId) =>
              navigate({ kind: "system", systemId, returnTo: null })}
          />
        ) : null}
        {route.kind === "system" ? (
          <SystemArticle
            systemId={route.systemId}
            returnTo={route.returnTo}
            onReturn={(sectionId) => navigate({ kind: "weng", sectionId })}
            onDiagnose={(caseId) => navigate({ kind: "diagnose", caseId })}
          />
        ) : null}
        {route.kind === "document" ? (
          <AuxiliaryDocumentReader
            documentId={route.documentId}
            sectionId={route.sectionId}
          />
        ) : null}
        {route.kind === "chapter" ? (
          <ChapterReader
            document={documentById(route.conceptId)}
            onSelect={(document) =>
              navigate({
                kind: "chapter",
                conceptId: chapterConceptId(document),
              })}
          />
        ) : null}
        {route.kind === "sources" ? (
          <div className="route-reader">
            <CanonicalDocumentView document={sourceRoute.document} />
          </div>
        ) : null}
        {route.kind === "legacy" ? (
          <div className="route-reader">
            {route.routeId === "knowledge" ? (
              <KnowledgeHome />
            ) : (
              <>
                {route.routeId === "loop" ? <LineageLoom /> : null}
                <CanonicalDocumentView
                  document={
                    canonicalReaderRoutes.find(
                      (candidate) => candidate.route.route_id === route.routeId,
                    )?.document ?? sourceRoute.document
                  }
                />
              </>
            )}
          </div>
        ) : null}
        {route.kind === "lesson" ? (
          <LessonRunner
            key={route.lessonId}
            lessonId={route.lessonId}
            onSelectLesson={(lessonId) =>
              navigate({ kind: "lesson", lessonId })}
            onDiagnose={(caseId) =>
              navigate({ kind: "diagnose", caseId })}
          />
        ) : null}
        {route.kind === "diagnose" ? (
          <DiagnosticWorkbench
            key={route.caseId ?? "custom"}
            caseId={route.caseId}
          />
        ) : null}
      </main>
      <footer className="site-footer">
        <span>Harp Atlas / source-bound technical reader</span>
        <span>offline export / no live fetch</span>
      </footer>
    </div>
  );
}
