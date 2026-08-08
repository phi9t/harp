import {
  canonicalCorpus,
  getPublishedSystems,
  getSystem,
} from "../content/canonical";
import type { SystemId, WengSectionId } from "../content/types";
import { CanonicalDocumentView } from "./ChapterReader";

function sectionLabel(sectionId: WengSectionId): string {
  const section = canonicalCorpus.weng_sections.find(
    (candidate) => candidate.section_id === sectionId,
  );
  if (!section) {
    throw new Error(`Validated Weng section disappeared: ${sectionId}`);
  }
  return section.title;
}

export function SystemArticle({
  systemId,
  returnTo,
  onReturn,
  onDiagnose,
}: {
  systemId: SystemId;
  returnTo: WengSectionId | null;
  onReturn: (sectionId: WengSectionId) => void;
  onDiagnose: (caseId: string) => void;
}) {
  const system = getSystem(systemId);
  if (!system) {
    throw new Error(`Validated system disappeared: ${systemId}`);
  }
  const published = getPublishedSystems().find(
    (candidate) => candidate.system_id === systemId,
  );

  return (
    <div className="route-reader system-reader">
      <section className="panel system-identity">
        <header>
          <div>
            <p className="eyebrow">System reading</p>
            <h1>{system.title}</h1>
            <p>
              {published ? "Canonical technical reading" : "Full reading planned"}
            </p>
          </div>
          {returnTo !== null ? (
            <button
              type="button"
              className="return-button"
              onClick={() => onReturn(returnTo)}
            >
              Return to Weng
            </button>
          ) : null}
          {system.diagnostic_case_path !== null ? (
            <button
              type="button"
              className="return-button"
              onClick={() => onDiagnose(system.system_id)}
            >
              Diagnose this system
            </button>
          ) : null}
        </header>
        <dl className="system-ledger">
          <div>
            <dt>Weng section</dt>
            <dd>{system.weng_section_ids.map(sectionLabel).join(", ")}</dd>
          </div>
          <div>
            <dt>Treatment</dt>
            <dd>{system.treatment}</dd>
          </div>
          <div>
            <dt>Publication</dt>
            <dd>{system.publication_state}</dd>
          </div>
          <div>
            <dt>Sources</dt>
            <dd>{system.source_ids.join(", ")}</dd>
          </div>
        </dl>
        <div className="paper-routes">
          <p className="eyebrow">Original paper routes</p>
          {system.paper_routes.map((route) => (
            <section
              key={`${system.system_id}-${route.source_id}`}
              className="paper-route"
            >
              <a
                aria-label={`${route.source_id} original paper`}
                href={route.public_url}
                rel="noreferrer"
                target="_blank"
              >
                <strong>{route.source_id}</strong>
                <span>{route.captured_locator}</span>
              </a>
              <ol aria-label={`${route.source_id} reading sequence`}>
                {Object.entries(route.reading_sequence).map(
                  ([category, locator]) => (
                    <li key={category}>
                      <strong>{category.replaceAll("_", " ")}</strong>
                      <span>{locator}</span>
                    </li>
                  ),
                )}
              </ol>
            </section>
          ))}
        </div>
      </section>
      {published ? <CanonicalDocumentView document={published.article} /> : null}
    </div>
  );
}
