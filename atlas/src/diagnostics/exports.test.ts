import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { canonicalCorpus } from "../content/canonical";
import { diagnose, forkCase, updateFact } from "./engine";
import {
  exportCritiquePacket,
  exportDiagnosisJson,
  exportDiagnosisMarkdown,
  importCommentary,
  importDiagnosisJson,
} from "./exports";

function aflowFixture() {
  const aflow = canonicalCorpus.diagnostics.cases.find(
    (diagnosticCase) => diagnosticCase.case_id === "aflow",
  );
  if (!aflow) {
    throw new Error("AFlow diagnostic fixture disappeared");
  }
  const state = forkCase(aflow);
  return { state, result: diagnose(state) };
}

async function digest(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value);
  const output = await crypto.subtle.digest("SHA-256", bytes);
  return [...new Uint8Array(output)]
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

describe("diagnostic exports", () => {
  it("emits byte-stable Markdown and JSON with trailing newlines", async () => {
    const fixture = aflowFixture();

    const firstMarkdown = exportDiagnosisMarkdown(fixture);
    const secondMarkdown = exportDiagnosisMarkdown(fixture);
    const firstJson = await exportDiagnosisJson(fixture);
    const secondJson = await exportDiagnosisJson(fixture);

    expect(firstMarkdown).toBe(secondMarkdown);
    expect(firstMarkdown).toMatch(/^# RSI decision brief: AFlow\n/);
    expect(firstMarkdown.endsWith("\n")).toBe(true);
    expect(firstJson).toBe(secondJson);
    expect(firstJson.endsWith("\n")).toBe(true);
    expect(JSON.parse(firstJson)).toEqual(
      expect.objectContaining({
        schema_version: "rsi-diagnosis/v2",
        diagnosis: expect.objectContaining({
          classification: "harness-improvement",
          claimCeiling: "harness-improvement",
        }),
        commentary: null,
      }),
    );
  });

  it("imports its own JSON without recomputing a different verdict", async () => {
    const fixture = aflowFixture();
    const exported = await exportDiagnosisJson(fixture);

    const imported = await importDiagnosisJson(exported);

    expect(imported.status).toBe("current");
    expect(imported.savedDiagnosis.classification).toBe("harness-improvement");
    expect(imported.savedDiagnosis.claimCeiling).toBe("harness-improvement");
    expect(diagnose(imported.state)).toEqual(fixture.result);
  });

  it("migrates an authentic v1 export as a preserved stale verdict", async () => {
    const legacy = await readFile(
      resolve("tests/fixtures/rsi-diagnosis-v1-aflow.json"),
      "utf8",
    );

    const imported = await importDiagnosisJson(legacy);
    const current = diagnose(imported.state);
    const fact = (fieldId: string) =>
      imported.state.facts.find((candidate) => candidate.field_id === fieldId);

    expect(imported.status).toBe("stale");
    expect(imported.savedDiagnosis.classification).toBe("harness-improvement");
    expect(imported.savedDiagnosis.claimCeiling).toBe("harness-improvement");
    expect(imported.state.facts).toHaveLength(26);
    expect(fact("matched-proposal-protocol")).toMatchObject({
      value: false,
      provenance: expect.objectContaining({ kind: "source-backed" }),
    });
    expect(fact("promotion-external")).toEqual({
      field_id: expect.any(String),
      value: false,
      provenance: { kind: "reader-assertion" },
    });
    expect(current.classification).toBe("harness-improvement");
    expect(current.claimCeiling).toBe("output-refinement");
  });

  it("preserves a stale saved rule ledger and finding verbatim", async () => {
    const fixture = aflowFixture();
    const state = updateFact(
      fixture.state,
      "candidate-can-write-evaluator",
      true,
    );
    const payload = JSON.parse(
      await exportDiagnosisJson({ state, result: diagnose(state) }),
    );
    payload.rules_sha256 = "0".repeat(64);
    const finding = payload.diagnosis.findings[0];
    const activeRule = payload.rules.find(
      (rule: { rule_id: string }) => rule.rule_id === finding.rule_id,
    );
    finding.rule_id = "legacy-evaluator-write";
    finding.explanation = "Legacy evaluator boundary explanation.";
    activeRule.rule_id = finding.rule_id;
    activeRule.version = 7;
    activeRule.precedence = 1234;
    finding.precedence = activeRule.precedence;
    const savedMarkdown = exportDiagnosisMarkdown(
      { state, result: payload.diagnosis },
      payload.rules,
    );
    payload.brief_sha256 = await digest(savedMarkdown);

    const imported = await importDiagnosisJson(`${JSON.stringify(payload)}\n`);

    expect(imported.status).toBe("stale");
    expect(imported.savedDiagnosis.findings[0]).toMatchObject({
      rule_id: "legacy-evaluator-write",
      precedence: 1234,
      explanation: "Legacy evaluator boundary explanation.",
    });
  });

  it("rejects duplicate facts and mismatched saved ledgers", async () => {
    const fixture = aflowFixture();
    const exported = JSON.parse(await exportDiagnosisJson(fixture));

    exported.worksheet.push(structuredClone(exported.worksheet[0]));
    await expect(importDiagnosisJson(`${JSON.stringify(exported)}\n`))
      .rejects.toThrow(/duplicate facts/);

    const second = JSON.parse(await exportDiagnosisJson(fixture));
    second.sources = [];
    await expect(importDiagnosisJson(`${JSON.stringify(second)}\n`))
      .rejects.toThrow(/source ledger/);
  });

  it("rejects a saved brief whose content no longer matches its digest", async () => {
    const fixture = aflowFixture();
    const exported = JSON.parse(await exportDiagnosisJson(fixture));
    exported.diagnosis.classification = "output-refinement";

    await expect(importDiagnosisJson(`${JSON.stringify(exported)}\n`))
      .rejects.toThrow(/brief digest/);
  });

  it("exports a lowered claim ceiling separately from mechanism classification", async () => {
    const fixture = aflowFixture();
    const state = updateFact(
      fixture.state,
      "candidate-can-write-evaluator",
      true,
    );
    const input = { state, result: diagnose(state) };
    const markdown = exportDiagnosisMarkdown(input);
    const json = JSON.parse(await exportDiagnosisJson(input));

    expect(input.result.classification).toBe("harness-improvement");
    expect(input.result.claimCeiling).toBe("output-refinement");
    expect(markdown).toContain("Primary classification: Harness improvement.");
    expect(markdown).toMatch(/## Honest claim ceiling\n\nOutput refinement/);
    expect(json.diagnosis).toMatchObject({
      classification: "harness-improvement",
      claimCeiling: "output-refinement",
    });
  });

  it("rejects silently normalized saved findings and duplicate derived IDs", async () => {
    const fixture = aflowFixture();
    const withFinding = {
      state: {
        ...fixture.state,
        facts: fixture.state.facts.map((fact) =>
          fact.field_id === "candidate-can-write-evaluator"
            ? { ...fact, value: true }
            : fact),
      },
      result: fixture.result,
    };
    withFinding.result = diagnose(withFinding.state);
    const exported = JSON.parse(await exportDiagnosisJson(withFinding));
    exported.diagnosis.findings[0].explanation = "Tampered explanation";
    await expect(importDiagnosisJson(`${JSON.stringify(exported)}\n`))
      .rejects.toThrow(/active rule/);

    const duplicate = JSON.parse(await exportDiagnosisJson(fixture));
    duplicate.diagnosis.flags.push(duplicate.diagnosis.flags[0]);
    await expect(importDiagnosisJson(`${JSON.stringify(duplicate)}\n`))
      .rejects.toThrow(/duplicate derived IDs/);
  });

  it("exports critique instructions without authorizing verdict rewrites", () => {
    const packet = exportCritiquePacket(aflowFixture());

    expect(packet).toContain("Do not rewrite the deterministic verdict");
    expect(packet).toMatch(/challenge causal attribution/i);
    expect(packet).toContain("Harness improvement");
  });

  it("bounds commentary and keeps it separate from the engine", () => {
    const fixture = aflowFixture();
    const before = diagnose(fixture.state);
    const commentary = importCommentary("External model disagrees.");
    const after = diagnose(fixture.state);

    expect(commentary).toBe("External model disagrees.");
    expect(after).toEqual(before);
    expect(() => importCommentary("x".repeat(262_145))).toThrow(/too large/);
  });
});
