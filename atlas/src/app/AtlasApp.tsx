import { useEffect, useState } from "react";

import {
  canonicalChapters,
  canonicalCorpus,
  canonicalReaderRoutes,
} from "../content/canonical";
import type {
  CanonicalDocument,
  ConceptId,
  DocumentId,
} from "../content/types";
import "../styles/tokens.css";
import "../styles/layout.css";
import "../styles/components.css";
import { CanonicalDocumentView, ChapterReader } from "./ChapterReader";
import { DiagnosticWorkbench } from "./DiagnosticWorkbench";
import {
  formatRoute,
  parseRoute,
  type AtlasRoute,
} from "./routes";
import { LineageLoom } from "./LineageLoom";
import { LessonRunner } from "./LessonRunner";
import { SystemArticle } from "./SystemArticle";
import { SystemLibrary } from "./SystemLibrary";
import { WengReader } from "./WengReader";

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

function AuxiliaryDocumentReader({ documentId }: { documentId: DocumentId }) {
  const document = documentById(documentId);
  return (
    <div className="route-reader">
      <CanonicalDocumentView
        document={document}
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
    case "weng":
      return "Weng";
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
      return route.routeId;
    default: {
      const exhaustive: never = route;
      return exhaustive;
    }
  }
}

export function AtlasApp() {
  const [route, setRoute] = useState<AtlasRoute>(() =>
    parseRoute(window.location.hash));

  useEffect(() => {
    if (!window.location.hash) {
      window.location.hash = formatRoute(parseRoute(""));
    }
    const readHash = () => setRoute(parseRoute(window.location.hash));
    window.addEventListener("hashchange", readHash);
    return () => window.removeEventListener("hashchange", readHash);
  }, []);

  const sourceRoute = canonicalReaderRoutes.find(
    (candidate) => candidate.route.route_id === "sources",
  );
  if (!sourceRoute) {
    throw new Error("Validated source route disappeared");
  }

  return (
    <div className="atlas-shell">
      <a className="skip-link" href="#main-content">Skip to content</a>
      <header className="site-header reader-header">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">
            <span />
            <span />
            <span />
          </div>
          <div>
            <strong>RSI / ATLAS</strong>
            <span>Weng reader + workbench</span>
          </div>
        </div>
        <nav className="top-nav" aria-label="Atlas routes">
          <button
            className={route.kind === "weng" ? "nav-button active" : "nav-button"}
            type="button"
            onClick={() =>
              navigate({
                kind: "weng",
                sectionId: canonicalCorpus.weng_sections[0].section_id,
              })}
          >
            Weng
          </button>
          <button
            className={
              route.kind === "systems" || route.kind === "system"
                ? "nav-button active"
                : "nav-button"
            }
            type="button"
            onClick={() => navigate({ kind: "systems" })}
          >
            Systems
          </button>
          <button
            className={route.kind === "lesson" ? "nav-button active" : "nav-button"}
            type="button"
            onClick={() =>
              navigate({
                kind: "lesson",
                lessonId: canonicalCorpus.lessons[0].lesson_id,
              })}
          >
            Lessons
          </button>
          <button
            className={route.kind === "diagnose" ? "nav-button active" : "nav-button"}
            type="button"
            onClick={() => navigate({ kind: "diagnose", caseId: null })}
          >
            Diagnose
          </button>
          <button
            className={route.kind === "chapter" ? "nav-button active" : "nav-button"}
            type="button"
            onClick={() =>
              navigate({
                kind: "chapter",
                conceptId: chapterConceptId(canonicalChapters[0]),
              })}
          >
            Chapters
          </button>
          <button
            className={route.kind === "sources" ? "nav-button active" : "nav-button"}
            type="button"
            onClick={() => navigate({ kind: "sources" })}
          >
            Sources
          </button>
        </nav>
        <div className="header-route" role="status" aria-label="Current route">
          <strong>{currentLabel(route)}</strong>
        </div>
      </header>
      <main id="main-content" tabIndex={-1}>
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
          <AuxiliaryDocumentReader documentId={route.documentId} />
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
            {route.routeId === "loop" ? <LineageLoom /> : null}
            <CanonicalDocumentView
              document={
                canonicalReaderRoutes.find(
                  (candidate) => candidate.route.route_id === route.routeId,
                )?.document ?? sourceRoute.document
              }
            />
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
