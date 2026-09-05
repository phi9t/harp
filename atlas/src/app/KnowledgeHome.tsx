import { useState } from "react";
import { canonicalCorpus } from "../content/canonical";
import type { CanonicalCorpus } from "../content/types";
import { formatRoute } from "./routes";

const readerKinds = new Set([
  "research-index",
  "benchmark-guide",
  "reference-architecture",
  "index",
]);
const evidenceKinds = new Set([
  "claim-ledger",
  "source-registry",
  "missing-evidence",
]);

type KnowledgeGroup = {
  title: string;
  documents: CanonicalCorpus["documents"];
};

function groupDocuments(corpus: CanonicalCorpus): KnowledgeGroup[] {
  const readerRoutes = new Set(
    corpus.reader_routes.map((route) => route.canonical_markdown_path),
  );
  const reader = corpus.documents.filter(
    (document) =>
      readerRoutes.has(document.canonical_markdown_path)
      || readerKinds.has(document.metadata.kind),
  );
  const evidence = corpus.documents.filter((document) =>
    evidenceKinds.has(document.metadata.kind),
  );
  const assigned = new Set([...reader, ...evidence].map((document) => document.concept_id));
  const deepDives = corpus.documents.filter(
    (document) =>
      !assigned.has(document.concept_id)
      && (document.metadata.kind === "technical-deep-dive"
        || document.metadata.kind === "system-reading"
        || document.metadata.kind === "technical-review"
        || document.metadata.kind === "technical-synthesis"
        || document.metadata.kind === "research-design"),
  );
  return [
    { title: "Reader routes", documents: reader },
    { title: "Deep dives", documents: deepDives },
    { title: "Claim and evidence", documents: evidence },
    { title: "Concepts and supporting readings", documents: corpus.documents.filter(
      (document) => !assigned.has(document.concept_id) && !deepDives.includes(document),
    ) },
  ].filter((group) => group.documents.length > 0);
}

export function KnowledgeHome({
  corpus = canonicalCorpus,
}: {
  corpus?: CanonicalCorpus;
}) {
  const [query, setQuery] = useState("");
  const terms = query.toLowerCase().trim().split(/\s+/).filter(Boolean);
  const matches = corpus.documents.filter((document) => {
    const text = [document.title, ...document.metadata.tags].join(" ").toLowerCase();
    return terms.every((term) => text.includes(term));
  });
  const groups = terms.length > 0
    ? [{ title: "Search results", documents: matches }]
    : groupDocuments(corpus);
  return (
    <section className="knowledge-home" aria-labelledby="knowledge-home-title">
      <header>
        <p className="eyebrow">Knowledge atlas</p>
        <h1 id="knowledge-home-title">Library</h1>
        <p>
          Browse the compiled readings or search by title and topic.
        </p>
      </header>
      <label className="library-search">
        Search readings
        <input type="search" value={query} onChange={(event) => setQuery(event.target.value)}
          placeholder="Try Crouzeix, harness, or evaluation" />
      </label>
      <p className="library-search-status" role="status">
        {matches.length === 0 ? "No readings found. Try a broader title or topic." : `${matches.length} readings`}
      </p>
      {query ? <button className="filter-button" type="button" onClick={() => setQuery("")}>Clear search</button> : null}
      {groups.filter((group) => group.documents.length > 0).map((group) => (
        <section key={group.title} aria-labelledby={`knowledge-${group.title.toLowerCase().replaceAll(" ", "-")}`}>
          <h2 id={`knowledge-${group.title.toLowerCase().replaceAll(" ", "-")}`}>{group.title}</h2>
          <div className="knowledge-card-grid">
            {group.documents.map((document) => (
              <a
                className="knowledge-card"
                href={formatRoute({
                  kind: "document",
                  documentId: document.concept_id,
                  sectionId: null,
                })}
                key={document.concept_id}
              >
                <strong>{document.title}</strong>
                {document.metadata.tags.length > 0 ? (
                  <small>{document.metadata.tags.join(" · ")}</small>
                ) : null}
              </a>
            ))}
          </div>
        </section>
      ))}
    </section>
  );
}
