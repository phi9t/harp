import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { diagnose, forkCase } from "../diagnostics/engine";
import { exportDiagnosisJson } from "../diagnostics/exports";
import { AtlasApp } from "./AtlasApp";

describe("diagnostic workbench", () => {
  beforeEach(() => {
    window.location.hash = "#diagnose";
  });

  it("starts a blank diagnosis incomplete and names required fields", () => {
    render(<AtlasApp />);

    expect(
      screen.getByRole("heading", { name: "Diagnostic workbench" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Diagnosis status" }))
      .toHaveTextContent("Incomplete");
    expect(screen.getByRole("textbox", { name: "Candidate name" }))
      .toBeInTheDocument();
    expect(
      screen.getByRole("group", { name: "Task outcome is measured" }),
    ).toBeInTheDocument();
    expect(
      within(
        screen.getByRole("group", { name: "Task outcome is measured" }),
      ).getByRole("radio", { name: "Yes" }),
    ).not.toBeChecked();
    expect(
      screen.getByRole("checkbox", { name: "No editable components" }),
    ).not.toBeChecked();
    expect(screen.getByRole("button", { name: "Download Markdown" }))
      .toBeDisabled();
    expect(screen.getByRole("button", { name: "Download JSON" }))
      .toBeDisabled();
    expect(screen.getByRole("button", { name: "Download critique packet" }))
      .toBeDisabled();
  });

  it("records explicit negative answers instead of defaulting missing facts", async () => {
    const user = userEvent.setup();
    render(<AtlasApp />);
    const outcomeGroup = screen.getByRole("group", {
      name: "Task outcome is measured",
    });

    await user.click(within(outcomeGroup).getByRole("radio", { name: "No" }));
    await user.click(
      screen.getByRole("checkbox", { name: "No editable components" }),
    );

    expect(within(outcomeGroup).getByRole("radio", { name: "No" }))
      .toBeChecked();
    expect(screen.getByRole("checkbox", { name: "No editable components" }))
      .toBeChecked();
    expect(screen.getAllByText("Reader assertion")).toHaveLength(2);
  });

  it("forks AFlow and updates provenance plus findings after an edit", async () => {
    const user = userEvent.setup();
    window.location.hash = "#diagnose/aflow";
    render(<AtlasApp />);

    expect(screen.getByRole("heading", { name: "AFlow diagnosis" }))
      .toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Primary classification" }))
      .toHaveTextContent("Harness improvement");
    expect(screen.getByRole("status", { name: "Claim ceiling" }))
      .toHaveTextContent("Harness improvement");

    await user.click(
      within(
        screen.getByRole("group", {
          name: "Candidate can write evaluator state",
        }),
      ).getByRole("radio", { name: "Yes" }),
    );

    expect(screen.getByText("Reader assertion")).toBeInTheDocument();
    expect(screen.getByText(/invalidates self-certification/)).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Primary classification" }))
      .toHaveTextContent("Harness improvement");
    expect(screen.getByRole("status", { name: "Claim ceiling" }))
      .toHaveTextContent("Output refinement");
    expect(screen.getByText("Reader assertion")).toHaveClass("field-provenance");
  });

  it("opens AFlow diagnosis from its system article", async () => {
    const user = userEvent.setup();
    window.location.hash = "#systems/aflow";
    render(<AtlasApp />);

    await user.click(
      screen.getByRole("button", { name: "Diagnose this system" }),
    );

    expect(window.location.hash).toBe("#diagnose/aflow");
    expect(screen.getByRole("heading", { name: "AFlow diagnosis" }))
      .toBeInTheDocument();
  });

  it("keeps imported commentary separate from the deterministic verdict", async () => {
    const user = userEvent.setup();
    window.location.hash = "#diagnose/aflow";
    render(<AtlasApp />);

    await user.type(
      screen.getByRole("textbox", { name: "Imported AI commentary" }),
      "External model calls this recursive.",
    );

    expect(screen.getByRole("status", { name: "Primary classification" }))
      .toHaveTextContent("Harness improvement");
    expect(
      screen.getByRole("textbox", { name: "Imported AI commentary" }),
    ).toHaveValue("External model calls this recursive.");
  });

  it("shows a stale saved verdict separately from current evaluation", async () => {
    const user = userEvent.setup();
    window.location.hash = "#diagnose/aflow";
    render(<AtlasApp />);
    const aflow = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "aflow",
    );
    if (!aflow) {
      throw new Error("AFlow diagnostic fixture disappeared");
    }
    const state = forkCase(aflow);
    const payload = JSON.parse(
      await exportDiagnosisJson({ state, result: diagnose(state) }),
    );
    payload.corpus_sha256 = "0".repeat(64);

    await user.upload(
      screen.getByLabelText("Import diagnosis JSON"),
      new File([`${JSON.stringify(payload)}\n`], "stale.json", {
        type: "application/json",
      }),
    );

    expect(
      await screen.findByRole("heading", {
        name: "Saved verdict from older contracts",
      }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Saved stale diagnosis")).toHaveTextContent(
      "Harness improvement",
    );
    expect(screen.getByRole("status", { name: "Claim ceiling" }))
      .toHaveTextContent("Harness improvement");
    expect(screen.getByRole("status", { name: "Import and export status" }))
      .toHaveTextContent("Stale diagnosis imported; saved verdict preserved");
  });
});
