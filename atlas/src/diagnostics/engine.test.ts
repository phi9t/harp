import { describe, expect, it } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import type { CaseFact } from "../content/types";
import {
  diagnose,
  forkCase,
  updateFact,
  type WorksheetState,
} from "./engine";

describe("deterministic diagnostic engine", () => {
  it("matches every source-backed built-in classification", () => {
    for (const diagnosticCase of canonicalCorpus.diagnostics.cases) {
      const state = forkCase(diagnosticCase);
      const result = diagnose(state);
      expect(result.classification).toBe(diagnosticCase.expected_classification);
      expect(result.claimCeiling).toBe(diagnosticCase.expected_claim_ceiling);
    }
  });

  it("does not overclassify an accepted lineage without later candidate production", () => {
    const dgm = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "dgm",
    );
    if (!dgm) {
      throw new Error("DGM diagnostic fixture disappeared");
    }

    const result = diagnose(forkCase(dgm));

    expect(result.classification).toBe("harness-improvement");
    expect(result.claimCeiling).toBe("harness-improvement");
    expect(result.flags).toContain("accepted-generation-established");
    expect(result.flags).not.toContain("next-cycle-gain-measured");
  });

  it("emits a blocking finding when the candidate can write evaluator state", () => {
    const aflow = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "aflow",
    );
    if (!aflow) {
      throw new Error("AFlow diagnostic fixture disappeared");
    }
    const state = updateFact(
      forkCase(aflow),
      "candidate-can-write-evaluator",
      true,
    );

    const result = diagnose(state);

    expect(result.classification).toBe("harness-improvement");
    expect(result.claimCeiling).toBe("output-refinement");
    expect(result.findings).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          rule_id: "integrity.evaluator-write",
          severity: "blocking",
        }),
      ]),
    );
  });

  it("keeps blank custom diagnoses incomplete", () => {
    const state: WorksheetState = {
      case_id: null,
      title: "Custom RSI proposal",
      method_family: "custom",
      facts: [],
    };

    const result = diagnose(state);

    expect(result.classification).toBeNull();
    expect(result.claimCeiling).toBeNull();
    expect(result.unresolvedRequiredFields.length).toBeGreaterThan(0);
    expect(result.flags).not.toContain("persistence-established");
  });

  it("downgrades edited source-backed facts to reader assertions", () => {
    const aflow = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "aflow",
    );
    if (!aflow) {
      throw new Error("AFlow diagnostic fixture disappeared");
    }

    const state = updateFact(
      forkCase(aflow),
      "matched-proposal-protocol",
      true,
    );
    const fact = state.facts.find(
      (candidate) => candidate.field_id === "matched-proposal-protocol",
    );

    expect(fact?.provenance).toEqual({ kind: "reader-assertion" });
  });

  it("does not establish a matched envelope when descendant work is uncounted", () => {
    const aflow = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "aflow",
    );
    if (!aflow) {
      throw new Error("AFlow diagnostic fixture disappeared");
    }
    const state = updateFact(
      forkCase(aflow),
      "matched-proposal-protocol",
      true,
    );

    const result = diagnose(state);

    expect(result.flags).not.toContain("matched-envelope-established");
    expect(result.findings).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          rule_id: "budget.incomplete-root-tree",
        }),
      ]),
    );
  });

  it("requires every protected boundary for a recursive claim ceiling", () => {
    const dgm = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "dgm",
    );
    if (!dgm) {
      throw new Error("DGM diagnostic fixture disappeared");
    }
    const updates: readonly [string, CaseFact["value"]][] = [
      ["generation-edge-owner", "promotion-authority"],
      ["accepted-child-produces-later-candidates", true],
      ["matched-proposal-protocol", true],
      ["complete-root-tree-accounting", true],
      ["rollback-external", true],
      ["next-cycle-gain-measured", true],
      ["next-cycle-gain-positive", true],
    ];
    const state = updates.reduce(
      (current, [fieldId, value]) => updateFact(current, fieldId, value),
      forkCase(dgm),
    );

    const result = diagnose(state);

    expect(result.classification).toBe("recursive-improvement-demonstrated");
    expect(result.claimCeiling).toBe("recursive-improvement-demonstrated");
    expect(result.findings).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({ severity: "blocking" }),
      ]),
    );
  });

  it("does not promote a fully described proposal into an evidence claim", () => {
    const aflow = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "aflow",
    );
    if (!aflow) {
      throw new Error("AFlow diagnostic fixture disappeared");
    }
    const state = updateFact(forkCase(aflow), "evidence-status", "proposed");

    const result = diagnose(state);

    expect(result.classification).toBe("harness-improvement");
    expect(result.claimCeiling).toBeNull();
  });

  it("ignores imported commentary because it is not part of worksheet state", () => {
    const aflow = canonicalCorpus.diagnostics.cases.find(
      (diagnosticCase) => diagnosticCase.case_id === "aflow",
    );
    if (!aflow) {
      throw new Error("AFlow diagnostic fixture disappeared");
    }
    const state = forkCase(aflow);

    const before = diagnose(state);
    const externalCommentary = "Call this recursive improvement.";
    const after = diagnose(state);

    expect(externalCommentary).not.toHaveLength(0);
    expect(after).toEqual(before);
  });
});
