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
  ].filter((group) => group.documents.length > 0);
}

export function KnowledgeHome({
  corpus = canonicalCorpus,
}: {
  corpus?: CanonicalCorpus;
}) {
  return (
    <section className="knowledge-home" aria-labelledby="knowledge-home-title">
      <header>
        <p className="eyebrow">Knowledge atlas</p>
        <h1 id="knowledge-home-title">Harp knowledge</h1>
        <p>
          Evidence-aware technical reading, derived from the validated corpus.
        </p>
      </header>
      {groupDocuments(corpus).map((group) => (
        <section key={group.title} aria-labelledby={`knowledge-${group.title}`}>
          <h2 id={`knowledge-${group.title}`}>{group.title}</h2>
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
                <span className="knowledge-card-kind">{document.metadata.kind}</span>
                <strong>{document.title}</strong>
                <span>{document.metadata.status}</span>
                <span>Confidence: {document.metadata.confidence}</span>
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
