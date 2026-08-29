import { canonicalCorpus } from "../content/canonical";
import type { SystemId } from "../content/types";

function wengSections(systemId: SystemId): string {
  const system = canonicalCorpus.systems.find(
    (candidate) => candidate.system_id === systemId,
  );
  if (!system) {
    throw new Error(`Validated system disappeared: ${systemId}`);
  }
  return system.weng_section_ids.map((sectionId) => {
    const section = canonicalCorpus.weng_sections.find(
      (candidate) => candidate.section_id === sectionId,
    );
    if (!section) {
      throw new Error(`Validated Weng section disappeared: ${sectionId}`);
    }
    return section.title;
  }).join(", ");
}

export function SystemLibrary({
  onSelectSystem,
}: {
  onSelectSystem: (systemId: SystemId) => void;
}) {
  return (
    <div className="route-reader">
      <section className="panel system-index">
        <header>
          <p className="eyebrow">Seventeen source-bound identities</p>
          <h1>Technical system readings</h1>
          <p>
            Every row opens a compiled canonical article and retains its Weng
            section and original-paper routes.
          </p>
        </header>
        <div className="system-table-scroll">
          <table aria-label="Technical system readings">
            <thead>
              <tr>
                <th scope="col">System</th>
                <th scope="col">Weng section</th>
                <th scope="col">Treatment</th>
                <th scope="col">Source</th>
                <th scope="col">State</th>
              </tr>
            </thead>
            <tbody>
              {canonicalCorpus.systems.map((system) => (
                <tr key={system.system_id}>
                  <th scope="row">
                    <button
                      aria-label={`Open ${system.title} system reading`}
                      type="button"
                      onClick={() => onSelectSystem(system.system_id)}
                    >
                      {system.title}
                    </button>
                  </th>
                  <td>{wengSections(system.system_id)}</td>
                  <td>{system.treatment}</td>
                  <td>{system.source_ids.join(", ")}</td>
                  <td>
                    <span data-state={system.publication_state}>
                      {system.publication_state}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
