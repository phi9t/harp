import type { DiagnosisResult, WorksheetState } from "../diagnostics/engine";
import type { ImportedDiagnosis } from "../diagnostics/exports";

function readable(value: string): string {
  const phrase = value.replaceAll("-", " ");
  return phrase.charAt(0).toUpperCase() + phrase.slice(1);
}

export function DecisionBrief({
  state,
  result,
  importedDiagnosis,
}: {
  state: WorksheetState;
  result: DiagnosisResult;
  importedDiagnosis: ImportedDiagnosis | null;
}) {
  const complete = result.unresolvedRequiredFields.length === 0;
  return (
    <aside className="panel decision-brief" aria-label="Decision brief">
      <header>
        <p className="eyebrow">Decision brief</p>
        <h2>{state.title}</h2>
      </header>
      {importedDiagnosis?.status === "stale" ? (
        <section className="stale-diagnosis" aria-label="Saved stale diagnosis">
          <h3>Saved verdict from older contracts</h3>
          <dl>
            <div>
              <dt>Mechanism</dt>
              <dd>
                {importedDiagnosis.savedDiagnosis.classification === null
                  ? "Incomplete"
                  : readable(importedDiagnosis.savedDiagnosis.classification)}
              </dd>
            </div>
            <div>
              <dt>Claim ceiling</dt>
              <dd>
                {importedDiagnosis.savedDiagnosis.claimCeiling === null
                  ? "Incomplete"
                  : readable(importedDiagnosis.savedDiagnosis.claimCeiling)}
              </dd>
            </div>
          </dl>
          <p>
            Current contracts are shown below. The Atlas has not overwritten
            the saved verdict.
          </p>
        </section>
      ) : null}
      <div
        className="diagnosis-verdict"
        role="status"
        aria-label="Primary classification"
      >
        <span>Mechanism classification</span>
        <strong>
          {result.classification === null
            ? "Incomplete"
            : readable(result.classification)}
        </strong>
      </div>
      <div
        className="diagnosis-verdict"
        role="status"
        aria-label="Claim ceiling"
      >
        <span>Honest claim ceiling</span>
        <strong>
          {result.claimCeiling === null
            ? "Incomplete"
            : readable(result.claimCeiling)}
        </strong>
      </div>
      {!complete ? (
        <section>
          <h3>Missing evidence</h3>
          <ul>
            {result.unresolvedRequiredFields.map((fieldId) => (
              <li key={fieldId}>{readable(fieldId)}</li>
            ))}
          </ul>
        </section>
      ) : null}
      {result.flags.length > 0 ? (
        <section>
          <h3>Established</h3>
          <ul>
            {result.flags.map((flag) => (
              <li key={flag}>{readable(flag)}</li>
            ))}
          </ul>
        </section>
      ) : null}
      {result.findings.length > 0 ? (
        <section>
          <h3>Findings</h3>
          <ul className="finding-list">
            {result.findings.map((finding) => (
              <li key={finding.rule_id} data-severity={finding.severity}>
                <strong>{readable(finding.severity)}</strong>
                <span>{finding.explanation}</span>
                <small>{finding.evidence_needed}</small>
              </li>
            ))}
          </ul>
        </section>
      ) : null}
    </aside>
  );
}
