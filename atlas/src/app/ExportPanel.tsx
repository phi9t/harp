import { useState } from "react";

import type {
  DiagnosisResult,
  WorksheetState,
} from "../diagnostics/engine";
import {
  exportCritiquePacket,
  exportDiagnosisJson,
  exportDiagnosisMarkdown,
  importCommentary,
  importDiagnosisJson,
  type ImportedDiagnosis,
} from "../diagnostics/exports";

function download(name: string, value: string, type: string): void {
  const url = URL.createObjectURL(new Blob([value], { type }));
  const link = document.createElement("a");
  link.href = url;
  link.download = name;
  link.click();
  URL.revokeObjectURL(url);
}

export function ExportPanel({
  state,
  result,
  commentary,
  onCommentaryChange,
  onImport,
}: {
  state: WorksheetState;
  result: DiagnosisResult;
  commentary: string;
  onCommentaryChange: (value: string) => void;
  onImport: (diagnosis: ImportedDiagnosis) => void;
}) {
  const [status, setStatus] = useState("");
  const input = { state, result, commentary };
  const complete = result.unresolvedRequiredFields.length === 0;

  return (
    <section className="export-panel">
      <h3>Export and external critique</h3>
      <div className="export-actions">
        <button
          type="button"
          disabled={!complete}
          onClick={() =>
            download(
              "rsi-diagnosis.md",
              exportDiagnosisMarkdown(input),
              "text/markdown",
            )}
        >
          Download Markdown
        </button>
        <button
          type="button"
          disabled={!complete}
          onClick={() => {
            void exportDiagnosisJson(input).then((value) =>
              download(
                "rsi-diagnosis.json",
                value,
                "application/json",
              ));
          }}
        >
          Download JSON
        </button>
        <button
          type="button"
          disabled={!complete}
          onClick={() =>
            download(
              "rsi-ai-critique.md",
              exportCritiquePacket(input),
              "text/markdown",
            )}
        >
          Download critique packet
        </button>
        <label className="file-import">
          <span>Import diagnosis JSON</span>
          <input
            type="file"
            accept="application/json,.json"
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (!file) {
                return;
              }
              void file.text()
                .then(importDiagnosisJson)
                .then((imported) => {
                  onImport(imported);
                  setStatus(
                    imported.status === "current"
                      ? "Diagnosis imported"
                      : "Stale diagnosis imported; saved verdict preserved",
                  );
                })
                .catch((error: unknown) => {
                  setStatus(
                    error instanceof Error ? error.message : "Import failed",
                  );
                });
            }}
          />
        </label>
      </div>
      <label className="commentary-input">
        <span>Imported AI commentary</span>
        <textarea
          rows={5}
          value={commentary}
          onChange={(event) => {
            try {
              onCommentaryChange(importCommentary(event.target.value));
              setStatus("");
            } catch (error: unknown) {
              setStatus(
                error instanceof Error ? error.message : "Commentary rejected",
              );
            }
          }}
        />
      </label>
      <p role="status" aria-label="Import and export status">{status}</p>
    </section>
  );
}
