import corpusData from "./generated/corpus.json";

import { parseCorpus } from "./contracts";
import type {
  PublishedSystemReading,
  SystemId,
  SystemReading,
} from "./types";

export const canonicalCorpus = parseCorpus(corpusData);

export const canonicalChapters = canonicalCorpus.coverage
  .filter((entry) => entry.coverage_depth === "chapter")
  .map((entry) => {
    const document = canonicalCorpus.documents.find(
      (candidate) =>
        candidate.canonical_markdown_path === entry.canonical_markdown_path,
    );
    if (!document) {
      throw new Error("Validated canonical chapter document disappeared");
    }
    return document;
  });

export const canonicalReaderRoutes = canonicalCorpus.reader_routes.map((route) => {
  const document = canonicalCorpus.documents.find(
    (candidate) =>
      candidate.canonical_markdown_path === route.canonical_markdown_path,
  );
  if (!document) {
    throw new Error("Validated canonical route document disappeared");
  }
  return { route, document };
});

export function getSystem(systemId: SystemId): SystemReading | null {
  return canonicalCorpus.systems.find(
    (system) => system.system_id === systemId,
  ) ?? null;
}

export function getPublishedSystems(): PublishedSystemReading[] {
  return canonicalCorpus.systems.flatMap((system) => {
    if (system.publication_state !== "published") {
      return [];
    }
    const article = canonicalCorpus.documents.find(
      (document) =>
        document.canonical_markdown_path === system.canonical_markdown_path,
    );
    if (!article) {
      throw new Error(`Published system ${system.system_id} lost its article`);
    }
    return [{ ...system, publication_state: "published", article }];
  });
}
