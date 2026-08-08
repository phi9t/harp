import { useMemo, useState } from "react";

import { canonicalCorpus } from "../content/canonical";
import type {
  CaseFact,
  FieldId,
  WorksheetField,
  WorksheetSection,
} from "../content/types";
import {
  diagnose,
  forkCase,
  updateFact,
  type WorksheetState,
} from "../diagnostics/engine";
import type { ImportedDiagnosis } from "../diagnostics/exports";
import { DecisionBrief } from "./DecisionBrief";
import { ExportPanel } from "./ExportPanel";

const sectionOrder: readonly WorksheetSection[] = [
  "candidate",
  "persistence",
  "generation",
  "evaluation",
  "resources-authority",
  "evidence",
];

function sectionLabel(section: WorksheetSection): string {
  switch (section) {
    case "candidate":
      return "Candidate";
    case "persistence":
      return "Persistence";
    case "generation":
      return "Generation";
    case "evaluation":
      return "Evaluation";
    case "resources-authority":
      return "Resources and authority";
    case "evidence":
      return "Evidence";
    default: {
      const exhaustive: never = section;
      return exhaustive;
    }
  }
}

function emptyState(): WorksheetState {
  return {
    case_id: null,
    title: "Custom RSI proposal",
    method_family: "custom",
    facts: [],
  };
}

function stateForCase(caseId: string | null): WorksheetState {
  if (caseId === null) {
    return emptyState();
  }
  const diagnosticCase = canonicalCorpus.diagnostics.cases.find(
    (candidate) => candidate.case_id === caseId,
  );
  return diagnosticCase ? forkCase(diagnosticCase) : emptyState();
}

function factValue(
  state: WorksheetState,
  fieldId: FieldId,
): CaseFact["value"] | undefined {
  return state.facts.find((fact) => fact.field_id === fieldId)?.value;
}

function provenanceLabel(state: WorksheetState, fieldId: FieldId): string | null {
  const provenance = state.facts.find(
    (fact) => fact.field_id === fieldId,
  )?.provenance;
  if (!provenance) {
    return null;
  }
  return provenance.kind === "source-backed"
    ? `${provenance.source_id} / ${provenance.locator}`
    : "Reader assertion";
}

function FieldControl({
  field,
  state,
  onChange,
}: {
  field: WorksheetField;
  state: WorksheetState;
  onChange: (fieldId: string, value: CaseFact["value"]) => void;
}) {
  const value = factValue(state, field.field_id);
  const provenance = provenanceLabel(state, field.field_id);
  return (
    <div className="worksheet-field">
      {field.kind === "boolean" ? (
        <fieldset className="binary-control">
          <legend>{field.label}</legend>
          <div>
            {[true, false].map((option) => (
              <label key={String(option)}>
                <input
                  type="radio"
                  name={field.field_id}
                  checked={value === option}
                  onChange={() => onChange(field.field_id, option)}
                />
                <span>{option ? "Yes" : "No"}</span>
              </label>
            ))}
          </div>
        </fieldset>
      ) : null}
      {field.kind === "text" ? (
        <label>
          <span>{field.label}</span>
          <input
            type="text"
            value={typeof value === "string" ? value : ""}
            onChange={(event) => onChange(field.field_id, event.target.value)}
          />
        </label>
      ) : null}
      {field.kind === "enum" ? (
        <label>
          <span>{field.label}</span>
          <select
            value={typeof value === "string" ? value : ""}
            onChange={(event) => onChange(field.field_id, event.target.value)}
          >
            <option value="">Select</option>
            {field.values.map((option) => (
              <option key={option} value={option}>
                {option.replaceAll("-", " ")}
              </option>
            ))}
          </select>
        </label>
      ) : null}
      {field.kind === "set" ? (
        <fieldset>
          <legend>{field.label}</legend>
          <div className="set-controls">
            <label>
              <input
                type="checkbox"
                checked={Array.isArray(value) && value.length === 0}
                onChange={(event) => {
                  if (event.target.checked) {
                    onChange(field.field_id, []);
                  }
                }}
              />
              <span>No editable components</span>
            </label>
            {field.values.map((option) => {
              const selected = Array.isArray(value) && value.includes(option);
              return (
                <label key={option}>
                  <input
                    type="checkbox"
                    checked={selected}
                    onChange={(event) => {
                      const current = Array.isArray(value) ? value : [];
                      onChange(
                        field.field_id,
                        event.target.checked
                          ? [...current, option]
                          : current.filter((item) => item !== option),
                      );
                    }}
                  />
                  <span>{option.replaceAll("-", " ")}</span>
                </label>
              );
            })}
          </div>
        </fieldset>
      ) : null}
      {provenance ? <small className="field-provenance">{provenance}</small> : null}
    </div>
  );
}

export function DiagnosticWorkbench({ caseId }: { caseId: string | null }) {
  const [state, setState] = useState<WorksheetState>(() => stateForCase(caseId));
  const [commentary, setCommentary] = useState("");
  const [importedDiagnosis, setImportedDiagnosis] =
    useState<ImportedDiagnosis | null>(null);
  const result = useMemo(() => diagnose(state), [state]);
  const title = state.case_id === null
    ? "Diagnostic workbench"
    : `${state.title} diagnosis`;

  return (
    <div className="diagnostic-layout">
      <section className="panel worksheet">
        <header>
          <div>
            <p className="eyebrow">Deterministic worksheet</p>
            <h1>{title}</h1>
          </div>
          <div role="status" aria-label="Diagnosis status">
            {result.unresolvedRequiredFields.length === 0
              ? "Complete"
              : "Incomplete"}
          </div>
        </header>
        {sectionOrder.map((section) => (
          <section key={section} className="worksheet-section">
            <h2>{sectionLabel(section)}</h2>
            <div className="worksheet-grid">
              {canonicalCorpus.diagnostics.worksheet.fields
                .filter((field) => field.section === section)
                .map((field) => (
                  <FieldControl
                    field={field}
                    key={field.field_id}
                    state={state}
                    onChange={(fieldId, value) =>
                      setState((current) => updateFact(current, fieldId, value))}
                  />
                ))}
            </div>
          </section>
        ))}
      </section>
      <div className="brief-column">
        <DecisionBrief
          state={state}
          result={result}
          importedDiagnosis={importedDiagnosis}
        />
        <section className="panel">
          <ExportPanel
            state={state}
            result={result}
            commentary={commentary}
            onCommentaryChange={setCommentary}
            onImport={(imported) => {
              setState(imported.state);
              setCommentary(imported.commentary ?? "");
              setImportedDiagnosis(imported);
            }}
          />
        </section>
      </div>
    </div>
  );
}
