import { canonicalCorpus } from "../content/canonical";
import type {
  CanonicalDocument,
  SystemId,
  WengSection,
  WengSectionId,
} from "../content/types";
import { CanonicalDocumentView } from "./ChapterReader";

function companion(section: WengSection): CanonicalDocument {
  const document = canonicalCorpus.documents.find(
    (candidate) => candidate.concept_id === section.companion_document_id,
  );
  if (!document) {
    throw new Error(`Validated Weng companion disappeared: ${section.section_id}`);
  }
  return document;
}

export function WengReader({
  sectionId,
  onSelectSection,
  onSelectSystem,
}: {
  sectionId: WengSectionId;
  onSelectSection: (sectionId: WengSectionId) => void;
  onSelectSystem: (systemId: SystemId, returnTo: WengSectionId) => void;
}) {
  const section = canonicalCorpus.weng_sections.find(
    (candidate) => candidate.section_id === sectionId,
  );
  if (!section) {
    throw new Error("Validated Weng section disappeared");
  }
  const systems = section.system_ids.map((systemId) => {
    const system = canonicalCorpus.systems.find(
      (candidate) => candidate.system_id === systemId,
    );
    if (!system) {
      throw new Error(`Validated system disappeared: ${systemId}`);
    }
    return system;
  });

  return (
    <div className="weng-reader">
      <aside className="weng-rail">
        <p className="eyebrow">Original article</p>
        <nav aria-label="Weng article sections">
          <ol>
            {canonicalCorpus.weng_sections.map((candidate) => (
              <li key={candidate.section_id}>
                <button
                  type="button"
                  aria-label={candidate.title}
                  aria-current={
                    candidate.section_id === section.section_id
                      ? "page"
                      : undefined
                  }
                  onClick={() => onSelectSection(candidate.section_id)}
                >
                  <span>{String(candidate.order).padStart(2, "0")}</span>
                  {candidate.title}
                </button>
              </li>
            ))}
          </ol>
        </nav>
      </aside>
      <div className="weng-article">
        <header className="weng-source-bar">
          <div>
            <span>Weng / {String(section.order).padStart(2, "0")}</span>
            <code>{section.captured_locator}</code>
          </div>
          <a href={section.public_url} rel="noreferrer" target="_blank">
            Read original section
          </a>
        </header>
        <CanonicalDocumentView document={companion(section)} />
      </div>
      <aside className="weng-trail">
        <p className="eyebrow">System trail</p>
        {systems.length === 0 ? (
          <p className="weng-empty">No new system is introduced in this section.</p>
        ) : (
          <ul>
            {systems.map((system) => (
              <li key={system.system_id}>
                <button
                  type="button"
                  aria-label={system.title}
                  onClick={() => onSelectSystem(system.system_id, section.section_id)}
                >
                  <strong>{system.title}</strong>
                  <span>{system.publication_state}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className="weng-checkpoint">
          <span>Reader checkpoint</span>
          <strong>{section.exercise_ids.length}</strong>
          <small>prediction before reveal</small>
        </div>
      </aside>
    </div>
  );
}
