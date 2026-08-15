import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const indexHtml = await readFile(new URL("../dist/client/index.html", import.meta.url), "utf8");
const atlasHtml = await readFile(new URL("../dist/harp-atlas.html", import.meta.url), "utf8");
const exportReceipt = JSON.parse(
  await readFile(new URL("../dist/harp-atlas.receipt.json", import.meta.url), "utf8"),
);
const sourceIndexHtml = await readFile(new URL("../index.html", import.meta.url), "utf8");
const atlasShell = atlasHtml.replace(
  /<script type="module">[\s\S]*?<\/script>/,
  "<script type=\"module\"></script>",
);
const inlineScript = atlasHtml.match(/<script type="module">([\s\S]*?)<\/script>/)?.[1] ?? "";
const encodedModule = inlineScript.match(
  /data:text\/javascript;base64,([A-Za-z0-9+/=]+)/,
)?.[1] ?? "";
const decodedModule = Buffer.from(encodedModule, "base64").toString("utf8");

test("exports an offline single-file Harp atlas", () => {
  assert.equal(exportReceipt.schema_version, "harp-atlas-export/v1");
  assert.match(exportReceipt.corpus_sha256, /^[0-9a-f]{64}$/);
  assert.match(exportReceipt.app_inputs_sha256, /^[0-9a-f]{64}$/);
  assert.match(exportReceipt.html_sha256, /^[0-9a-f]{64}$/);
  assert.match(sourceIndexHtml, /\.\/dist\/harp-atlas\.html/);
  assert.match(sourceIndexHtml, /id="file-entry-redirect"/);
  assert.match(indexHtml, /Harp Atlas/);
  assert.match(atlasHtml, /Harp Atlas/);
  assert.match(atlasHtml, /<script type="module">/);
  assert.match(atlasHtml, /import "data:text\/javascript;base64,/);
  assert.match(atlasHtml, /<style>/);
  assert.doesNotMatch(inlineScript, /<\/script>/i);
  assert.doesNotMatch(atlasShell, /<script[^>]+src=/);
  assert.doesNotMatch(atlasShell, /<link[^>]+rel="stylesheet"/);
  assert.doesNotMatch(atlasShell, /@font-face|url\s*\(/i);
  assert.doesNotMatch(indexHtml, /file-entry-redirect|dist\/harp-atlas\.html/);
  assert.doesNotMatch(atlasHtml, /file-entry-redirect|dist\/harp-atlas\.html/);
  assert.match(decodedModule, /application\/x-tex/);
  assert.match(decodedModule, /\?section=/);
  assert.match(decodedModule, /math-error/);
  assert.match(decodedModule, /What makes an improvement loop recursive/);
  assert.match(decodedModule, /Evaluation, promotion, and containment/);
  assert.match(decodedModule, /RSI harness by Lil'Log, deconstructed/);
  assert.match(decodedModule, /Promotion path open/);
  assert.match(decodedModule, /Promotion blocked/);
  for (const label of [
    "Weng",
    "Systems",
    "Lessons",
    "Diagnose",
    "Chapters",
    "Sources",
  ]) {
    assert.match(decodedModule, new RegExp(`\\b${label}\\b`));
  }
  assert.match(decodedModule, /workflow-design-and-search/);
  assert.match(decodedModule, /Workflow design becomes a search problem/);
  assert.match(decodedModule, /systems\/aflow/);
  assert.match(decodedModule, /AFlow: MCTS over agentic workflows/);
  assert.match(decodedModule, /ADAS versus AFlow/);
  assert.match(decodedModule, /Context engineering reading map/);
  assert.match(decodedModule, /Context pipeline checkpoint/);
  assert.match(decodedModule, /Context engineering technical sections/);
  assert.match(decodedModule, /ACE original paper/);
  assert.match(decodedModule, /MCE original paper/);
  assert.match(decodedModule, /Meta-Harness original paper/);
  assert.match(decodedModule, /class\.harness-improvement/);
  assert.match(decodedModule, /ceiling\.harness-improvement/);
  assert.match(decodedModule, /integrity\.evaluator-write/);
  assert.match(decodedModule, /Download Markdown/);
  assert.match(decodedModule, /Download JSON/);
  assert.match(decodedModule, /Download critique packet/);
  assert.match(decodedModule, /rsi-diagnosis\.md/);
  assert.match(decodedModule, /rsi-diagnosis\.json/);
  assert.match(decodedModule, /rsi-ai-critique\.md/);
  assert.match(decodedModule, /rsi-diagnosis\/v2/);
  assert.match(decodedModule, /rsi-diagnosis\/v1/);
  assert.match(decodedModule, /Import diagnosis JSON/);
  assert.match(decodedModule, /Imported AI commentary/);
  assert.match(decodedModule, /Stale diagnosis imported; saved verdict preserved/);
  assert.doesNotMatch(
    decodedModule,
    /api[-_]?key|endpoint configuration|credential configuration/i,
  );
  assert.doesNotMatch(decodedModule, /(?<![.\w}])fetch\s*\(/);
  assert.doesNotMatch(decodedModule, /\b(?:window|globalThis|self)\.fetch\s*\(/);
  assert.match(decodedModule, /knowledge\/rsi\/chapters\/harness-engineering\.md/);
  assert.match(
    decodedModule,
    /knowledge\/rsi\/rsi_harness_by_lil_log_deconstructed\.md/,
  );
});
