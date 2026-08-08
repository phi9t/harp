import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { diagnose, forkCase } from "../diagnostics/engine";
import { exportDiagnosisJson } from "../diagnostics/exports";
import { ExportPanel } from "./ExportPanel";

function fixture() {
  const aflow = canonicalCorpus.diagnostics.cases.find(
    (diagnosticCase) => diagnosticCase.case_id === "aflow",
  );
  if (!aflow) {
    throw new Error("AFlow diagnostic fixture disappeared");
  }
  const state = forkCase(aflow);
  return { state, result: diagnose(state) };
}

describe("diagnostic export panel", () => {
  const downloads: { name: string; blob: Blob }[] = [];

  beforeEach(() => {
    downloads.length = 0;
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: vi.fn((blob: Blob) => {
        downloads.push({ name: "", blob });
        return `blob:rsi-${downloads.length}`;
      }),
    });
    Object.defineProperty(URL, "revokeObjectURL", {
      configurable: true,
      value: vi.fn(),
    });
    vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (
      this: HTMLAnchorElement,
    ) {
      const download = downloads.at(-1);
      if (download) {
        download.name = this.download;
      }
    });
  });

  it("downloads the three exact offline artifacts with stable MIME types", async () => {
    const user = userEvent.setup();
    const input = fixture();
    render(
      <ExportPanel
        {...input}
        commentary=""
        onCommentaryChange={() => undefined}
        onImport={() => undefined}
      />,
    );

    await user.click(screen.getByRole("button", { name: "Download Markdown" }));
    await user.click(screen.getByRole("button", { name: "Download JSON" }));
    await waitFor(() => expect(downloads).toHaveLength(2));
    await user.click(
      screen.getByRole("button", { name: "Download critique packet" }),
    );

    expect(downloads.map(({ name, blob }) => [name, blob.type])).toEqual([
      ["rsi-diagnosis.md", "text/markdown"],
      ["rsi-diagnosis.json", "application/json"],
      ["rsi-ai-critique.md", "text/markdown"],
    ]);
    expect(await downloads[0].blob.text()).toContain("# RSI decision brief");
    expect(JSON.parse(await downloads[1].blob.text())).toMatchObject({
      diagnosis: {
        classification: "harness-improvement",
        claimCeiling: "harness-improvement",
      },
    });
    expect(await downloads[2].blob.text()).toContain(
      "Do not rewrite the deterministic verdict",
    );
  });

  it("imports current and stale diagnosis files through the file boundary", async () => {
    const user = userEvent.setup();
    const input = fixture();
    const onImport = vi.fn();
    render(
      <ExportPanel
        {...input}
        commentary=""
        onCommentaryChange={() => undefined}
        onImport={onImport}
      />,
    );
    const fileInput = screen.getByLabelText("Import diagnosis JSON");
    const currentJson = await exportDiagnosisJson(input);

    await user.upload(
      fileInput,
      new File([currentJson], "rsi-diagnosis.json", {
        type: "application/json",
      }),
    );
    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Diagnosis imported");
    });
    expect(onImport).toHaveBeenLastCalledWith(
      expect.objectContaining({
        status: "current",
        state: expect.objectContaining({ case_id: "aflow" }),
        commentary: null,
      }),
    );

    const stale = JSON.parse(currentJson);
    stale.corpus_sha256 = "0".repeat(64);
    await user.upload(
      fileInput,
      new File([`${JSON.stringify(stale)}\n`], "stale.json", {
        type: "application/json",
      }),
    );
    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent(
        "Stale diagnosis imported; saved verdict preserved",
      );
    });
  });
});
