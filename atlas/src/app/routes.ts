import { canonicalCorpus } from "../content/canonical";
import type {
  ConceptId,
  DocumentId,
  ReaderRouteId,
  SystemId,
  WengSectionId,
} from "../content/types";

export type AtlasRoute =
  | { kind: "weng"; sectionId: WengSectionId }
  | { kind: "systems" }
  | { kind: "system"; systemId: SystemId; returnTo: WengSectionId | null }
  | {
      kind: "document";
      documentId: DocumentId;
      sectionId: string | null;
    }
  | { kind: "chapter"; conceptId: ConceptId }
  | { kind: "legacy"; routeId: ReaderRouteId }
  | { kind: "lesson"; lessonId: string }
  | { kind: "diagnose"; caseId: string | null }
  | { kind: "sources" };

const firstWengSection = canonicalCorpus.weng_sections[0];
const sectionIdPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

function defaultRoute(): AtlasRoute {
  return { kind: "weng", sectionId: firstWengSection.section_id };
}

function decoded(value: string): string | null {
  try {
    return decodeURIComponent(value);
  } catch {
    return null;
  }
}

function returnSection(query: string): WengSectionId | null {
  const value = new URLSearchParams(query).get("return");
  if (!value?.startsWith("weng/")) {
    return null;
  }
  const sectionId = decoded(value.slice("weng/".length));
  return canonicalCorpus.weng_sections.find(
    (section) => section.section_id === sectionId,
  )?.section_id ?? null;
}

function documentSection(query: string): string | null {
  const sectionId = new URLSearchParams(query).get("section");
  return sectionId !== null && sectionIdPattern.test(sectionId) ? sectionId : null;
}

export function parseRoute(hash: string): AtlasRoute {
  const raw = hash.startsWith("#") ? hash.slice(1) : hash;
  if (raw === "") {
    return defaultRoute();
  }
  const [path, query = ""] = raw.split("?", 2);
  const [family, encodedId = ""] = path.split("/", 2);
  const id = decoded(encodedId);

  if (family === "weng" && encodedId !== "" && id !== null) {
    const section = canonicalCorpus.weng_sections.find(
      (candidate) => candidate.section_id === id,
    );
    return section
      ? { kind: "weng", sectionId: section.section_id }
      : defaultRoute();
  }
  if (family === "systems" && encodedId === "") {
    return { kind: "systems" };
  }
  if (family === "systems" && id !== null) {
    const system = canonicalCorpus.systems.find(
      (candidate) => candidate.system_id === id,
    );
    return system
      ? {
          kind: "system",
          systemId: system.system_id,
          returnTo: returnSection(query),
        }
      : defaultRoute();
  }
  if (family === "documents" && id !== null) {
    const document = canonicalCorpus.documents.find(
      (candidate) => candidate.concept_id === id,
    );
    return document
      ? {
          kind: "document",
          documentId: document.concept_id,
          sectionId: documentSection(query),
        }
      : defaultRoute();
  }
  if (family === "chapters" && id !== null) {
    const chapter = canonicalCorpus.coverage.find(
      (entry) => entry.coverage_depth === "chapter" && entry.concept_id === id,
    );
    return chapter
      ? { kind: "chapter", conceptId: chapter.concept_id }
      : defaultRoute();
  }
  if (family === "lessons" && id !== null && id !== "") {
    const lesson = canonicalCorpus.lessons.find(
      (candidate) => candidate.lesson_id === id,
    );
    return lesson
      ? { kind: "lesson", lessonId: lesson.lesson_id }
      : defaultRoute();
  }
  if (family === "diagnose") {
    if (id === "") {
      return { kind: "diagnose", caseId: null };
    }
    const diagnosticCase = canonicalCorpus.diagnostics.cases.find(
      (candidate) => candidate.case_id === id,
    );
    return diagnosticCase
      ? { kind: "diagnose", caseId: diagnosticCase.case_id }
      : defaultRoute();
  }
  if (family === "sources") {
    return { kind: "sources" };
  }
  const legacy = canonicalCorpus.reader_routes.find(
    (candidate) => candidate.route_id === family,
  );
  return legacy
    ? { kind: "legacy", routeId: legacy.route_id }
    : defaultRoute();
}

export function formatRoute(route: AtlasRoute): string {
  switch (route.kind) {
    case "weng":
      return `#weng/${encodeURIComponent(route.sectionId)}`;
    case "systems":
      return "#systems";
    case "system": {
      const base = `#systems/${encodeURIComponent(route.systemId)}`;
      return route.returnTo === null
        ? base
        : `${base}?return=weng/${encodeURIComponent(route.returnTo)}`;
    }
    case "document": {
      const base = `#documents/${encodeURIComponent(route.documentId)}`;
      return route.sectionId === null
        ? base
        : `${base}?section=${encodeURIComponent(route.sectionId)}`;
    }
    case "chapter":
      return `#chapters/${encodeURIComponent(route.conceptId)}`;
    case "legacy":
      return `#${route.routeId}`;
    case "lesson":
      return `#lessons/${encodeURIComponent(route.lessonId)}`;
    case "diagnose":
      return route.caseId === null
        ? "#diagnose"
        : `#diagnose/${encodeURIComponent(route.caseId)}`;
    case "sources":
      return "#sources";
    default: {
      const exhaustive: never = route;
      return exhaustive;
    }
  }
}
