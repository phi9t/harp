import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
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

function shellUsesExternalResource(value) {
  const resourceAttributes = {
    script: ["src"],
    link: ["href"],
    img: ["src", "srcset"],
    source: ["src", "srcset"],
    image: ["href", "xlink:href", "xlinkHref"],
  };
  for (const tagMatch of value.matchAll(/<(script|link|img|source|image)\b[^>]*>/gi)) {
    const tagName = tagMatch[1].toLowerCase();
    const attributes = resourceAttributes[tagName];
    const attributePattern = new RegExp(
      `(?:^|\\s)(?:${attributes.join("|")})\\s*=\\s*(?:\\{\\s*)?(?:"([^"]*)"|'([^']*)'|\`([^\`]*)\`|([^\\s>}]+))`,
      "gi",
    );
    for (const attributeMatch of tagMatch[0].matchAll(attributePattern)) {
      const attributeValue = attributeMatch.slice(1).find((part) => part !== undefined) ?? "";
      if (/(?:https?:)?\/\//i.test(attributeValue)) {
        return true;
      }
    }
  }
  return false;
}

function maskJavaScriptProse(value) {
  const masked = [...value];
  const stringTokens = [];
  const templateExpressionDepths = [];
  let mode = "code";
  let stringStart = -1;
  let templateHasExpression = false;

  const blank = (index) => {
    if (value[index] !== "\n" && value[index] !== "\r") {
      masked[index] = " ";
    }
  };

  for (let index = 0; index < value.length; index += 1) {
    const current = value[index];
    const next = value[index + 1];

    if (mode === "line-comment") {
      blank(index);
      if (current === "\n" || current === "\r") {
        mode = "code";
      }
      continue;
    }
    if (mode === "block-comment") {
      blank(index);
      if (current === "*" && next === "/") {
        blank(index + 1);
        index += 1;
        mode = "code";
      }
      continue;
    }
    if (mode === "single-quote" || mode === "double-quote") {
      blank(index);
      if (current === "\\") {
        if (next !== undefined) {
          blank(index + 1);
          index += 1;
        }
      } else if (
        (mode === "single-quote" && current === "'")
        || (mode === "double-quote" && current === '"')
      ) {
        stringTokens.push({
          start: stringStart,
          end: index + 1,
          value: value.slice(stringStart + 1, index),
        });
        mode = "code";
      }
      continue;
    }
    if (mode === "template") {
      blank(index);
      if (current === "\\") {
        if (next !== undefined) {
          blank(index + 1);
          index += 1;
        }
      } else if (current === "`") {
        stringTokens.push({
          start: stringStart,
          end: index + 1,
          value: value.slice(stringStart + 1, index),
          templateHasExpression,
        });
        mode = "code";
      } else if (current === "$" && next === "{") {
        templateHasExpression = true;
        templateExpressionDepths.push(1);
        masked[index + 1] = "{";
        index += 1;
        mode = "code";
      }
      continue;
    }

    if (current === "/" && next === "/") {
      blank(index);
      blank(index + 1);
      index += 1;
      mode = "line-comment";
    } else if (current === "/" && next === "*") {
      blank(index);
      blank(index + 1);
      index += 1;
      mode = "block-comment";
    } else if (current === "'" || current === '"') {
      stringStart = index;
      blank(index);
      mode = current === "'" ? "single-quote" : "double-quote";
    } else if (current === "`") {
      stringStart = index;
      templateHasExpression = false;
      blank(index);
      mode = "template";
    } else if (templateExpressionDepths.length > 0 && current === "{") {
      templateExpressionDepths[templateExpressionDepths.length - 1] += 1;
    } else if (templateExpressionDepths.length > 0 && current === "}") {
      const depthIndex = templateExpressionDepths.length - 1;
      templateExpressionDepths[depthIndex] -= 1;
      if (templateExpressionDepths[depthIndex] === 0) {
        templateExpressionDepths.pop();
        mode = "template";
      }
    }
  }

  return { masked: masked.join(""), stringTokens };
}

function fetchIdentifierIsNetworkUse(masked, index) {
  let previous = index - 1;
  while (previous >= 0 && /\s/.test(masked[previous])) {
    previous -= 1;
  }

  let hasGlobalReceiver = false;
  if (masked[previous] === ".") {
    let receiverEnd = previous - 1;
    if (masked[receiverEnd] === "?") {
      receiverEnd -= 1;
    }
    while (receiverEnd >= 0 && /\s/.test(masked[receiverEnd])) {
      receiverEnd -= 1;
    }
    const receiver = masked.slice(0, receiverEnd + 1).match(/([A-Za-z_$][\w$]*)$/)?.[1];
    if (!new Set(["window", "globalThis", "self"]).has(receiver)) {
      return false;
    }
    hasGlobalReceiver = true;
  } else if (masked[previous] === "]") {
    return false;
  }

  const before = masked.slice(0, index);
  const after = masked.slice(index + "fetch".length);
  return hasGlobalReceiver
    || /^\s*(?:\?\.\s*)?\(/.test(after)
    || /^\s*(?:\.\s*call|\?\.\s*call)\s*\(/.test(after)
    || /=\s*$/.test(before);
}

function maskedSourceUsesComputedNetworkMember(masked, stringTokens) {
  for (const match of masked.matchAll(/\bfetch\b/g)) {
    if (fetchIdentifierIsNetworkUse(masked, match.index)) {
      return true;
    }
  }

  for (const token of stringTokens) {
    const globalProperties = new Set([
      "fetch",
      "WebSocket",
      "EventSource",
      "XMLHttpRequest",
    ]);
    if (!globalProperties.has(token.value) && token.value !== "sendBeacon") {
      continue;
    }
    const before = masked.slice(0, token.start);
    const after = masked.slice(token.end);
    const receiver = before.match(
      /\b(window|globalThis|self|navigator)\s*(?:\?\.)?\s*\[\s*$/,
    )?.[1];
    if (!receiver || !/^\s*\]/.test(after)) {
      continue;
    }
    if (globalProperties.has(token.value) && receiver !== "navigator") {
      return true;
    }
    if (
      token.value === "sendBeacon"
      && /^\s*\]\s*(?:\?\.)?\s*(?:\(|\.call\s*\()/.test(after)
    ) {
      return true;
    }
  }
  return false;
}

function maskedSourceUsesRemoteDynamicImport(masked, stringTokens) {
  for (const token of stringTokens) {
    if (!/^(?:https?:)?\/\//i.test(token.value) && !token.templateHasExpression) {
      continue;
    }
    const before = masked.slice(0, token.start);
    const after = masked.slice(token.end);
    if (/\bimport\s*\(\s*$/.test(before) && /^\s*[),]/.test(after)) {
      return true;
    }
  }
  return false;
}

function productionSourceUsesNetwork(value, extension) {
  if (extension === ".css" && /@import\b|url\s*\(/i.test(value)) {
    return true;
  }
  if (shellUsesExternalResource(value)) {
    return true;
  }
  const { masked, stringTokens } = maskJavaScriptProse(value);
  return [
    /\bnew\s+(?:WebSocket|EventSource|XMLHttpRequest)\s*\(/,
    /\b(?:window|globalThis|self)\s*(?:\?\.\s*|\.\s*)(?:WebSocket|EventSource|XMLHttpRequest)\b/,
    /(?:\.\s*sendBeacon|\[\s*["']sendBeacon["']\s*\])\s*(?:\?\.)?\s*\(/,
  ].some((pattern) => pattern.test(masked))
    || maskedSourceUsesComputedNetworkMember(masked, stringTokens)
    || maskedSourceUsesRemoteDynamicImport(masked, stringTokens);
}

async function collectProductionSources(directoryUrl, sources = []) {
  for (const entry of await readdir(directoryUrl, { withFileTypes: true })) {
    const entryUrl = new URL(entry.name + (entry.isDirectory() ? "/" : ""), directoryUrl);
    if (entry.isDirectory()) {
      if (entry.name !== "generated") {
        await collectProductionSources(entryUrl, sources);
      }
    } else if (/\.(?:css|ts|tsx)$/.test(entry.name) && !/\.(?:test|spec)\.[^.]+$/.test(entry.name)) {
      sources.push(entryUrl);
    }
  }
  return sources;
}

test("offline guards reject every supported external resource form", () => {
  for (const fixture of [
    '<script src="https://cdn.example/app.js"></script>',
    '<script src="//cdn.example/app.js"></script>',
    '<link href="http://cdn.example/app.css" rel="stylesheet">',
    '<link href="//cdn.example/app.css" rel="stylesheet">',
    '<img src="https://cdn.example/image.png">',
    '<img srcset="https://cdn.example/image.png 2x">',
    '<source src="//cdn.example/image.webp">',
    '<source srcset="//cdn.example/image.webp 2x">',
    '<svg><image href="http://cdn.example/image.svg"></image></svg>',
    '<svg><image xlink:href="https://cdn.example/image.svg"></image></svg>',
  ]) {
    assert.equal(shellUsesExternalResource(fixture), true, `missed external resource: ${fixture}`);
  }
  assert.equal(
    shellUsesExternalResource('<a href="https://example.test/citation">citation</a>'),
    false,
    "anchor citations are content, not external resources",
  );
});

test("production-source guards reject browser network and external resource APIs", () => {
  const fixtures = [
    ['fetch("/api")', '.ts'],
    ['window.fetch("/api")', '.ts'],
    ['window?.fetch("/api")', '.ts'],
    ['globalThis.fetch?.("/api")', '.ts'],
    ['globalThis.fetch.call(globalThis, "/api")', '.ts'],
    ['self["fetch"]("/api")', '.ts'],
    ['fetch.call(window, "/api")', '.ts'],
    ['const request = fetch; request("/api");', '.ts'],
    ['const request = window . fetch;', '.ts'],
    ['const request = window["fetch"]; request("/api")', '.ts'],
    ['const request = window.fetch; request("/api")', '.ts'],
    ['new WebSocket("wss://example.test")', '.ts'],
    ['new window["WebSocket"]("wss://example.test")', '.ts'],
    ['new EventSource("/events")', '.ts'],
    ['new globalThis["EventSource"]("/events")', '.ts'],
    ['new XMLHttpRequest()', '.ts'],
    ['new self["XMLHttpRequest"]()', '.ts'],
    ['navigator.sendBeacon("/metrics")', '.ts'],
    ['navigator["sendBeacon"]("/metrics")', '.ts'],
    ['import("https://cdn.example/module.js")', '.ts'],
    ['import(`//cdn.example/module.js`)', '.ts'],
    ['import(`${scheme}://cdn.example/module.js`)', '.ts'],
    ['import("https://cdn.example/data.json", { with: { type: "json" } })', '.ts'],
    ['const style = `@import "//cdn.example/app.css"`;', '.css'],
    ['.hero { background: url("https://cdn.example/image.png"); }', '.css'],
    ['<script src="https://cdn.example/app.js" />', '.tsx'],
    ['<link href="//cdn.example/app.css" rel="stylesheet" />', '.tsx'],
    ['<img srcSet="https://cdn.example/image.png 2x" />', '.tsx'],
    ['<source src="//cdn.example/image.webp" />', '.tsx'],
    ['<image xlinkHref="https://cdn.example/image.svg" />', '.tsx'],
  ];
  for (const [fixture, extension] of fixtures) {
    assert.equal(
      productionSourceUsesNetwork(fixture, extension),
      true,
      `missed production network boundary: ${fixture}`,
    );
  }
  for (const [fixture, extension] of [
    ['<a href="https://example.test/citation">citation</a>', '.tsx'],
    ['const next = new URL(parsed);', '.ts'],
    ['parser.fetch()', '.ts'],
    ['parser . fetch()', '.ts'],
    ['parser ?. fetch()', '.ts'],
    ['const text = "WebSocket is disabled";', '.ts'],
    ['// fetch() is intentionally absent', '.ts'],
    ['/* EventSource XMLHttpRequest sendBeacon */', '.ts'],
    ['const note = "EventSource, XMLHttpRequest, and sendBeacon are disabled";', '.ts'],
    ['function WebSocket(value) { return value; }', '.ts'],
    ['const EventSource = (value) => value; EventSource("local")', '.ts'],
    ['// Never call import("https://cdn.example/module.js")', '.ts'],
    ["const note = 'Avoid import(\"https://cdn.example/module.js\")';", '.ts'],
  ]) {
    assert.equal(
      productionSourceUsesNetwork(fixture, extension),
      false,
      `rejected allowed source: ${fixture}`,
    );
  }
  assert.equal(
    productionSourceUsesNetwork('function request() { return fetch ("/api"); }', '.ts'),
    true,
    "missed whitespace-form global fetch call",
  );
});

test("handwritten production sources preserve the offline boundary", async () => {
  const sourceRoot = new URL("../src/", import.meta.url);
  const sourceUrls = await collectProductionSources(sourceRoot);
  sourceUrls.push(new URL("../index.html", import.meta.url));
  for (const sourceUrl of sourceUrls) {
    const source = await readFile(sourceUrl, "utf8");
    const extension = sourceUrl.pathname.slice(sourceUrl.pathname.lastIndexOf("."));
    assert.equal(
      productionSourceUsesNetwork(source, extension),
      false,
      `offline production source contains a network dependency: ${sourceUrl.pathname}`,
    );
  }
});

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
  assert.equal(shellUsesExternalResource(atlasShell), false);
  assert.doesNotMatch(atlasShell, /@font-face|@import\b|url\s*\(/i);
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
    "Crouzeix",
  ]) {
    assert.match(decodedModule, new RegExp(`\\b${label}\\b`));
  }
  for (const label of [
    "Console / Workstreams",
    "Harp Workstreams",
    "REFERENCE SNAPSHOT",
    "Crouzeix Conjecture",
    "Autodiff Geometry",
    "Agentic Research",
    "NNG4 / Foundations",
    "Research",
    "Spec",
    "Formalization",
    "Scroll for more columns",
  ]) {
    assert.equal(
      decodedModule.includes(label),
      true,
      `offline export is missing Workstreams label: ${label}`,
    );
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
  assert.match(decodedModule, /Crouzeix conjecture two-proof index/);
  assert.match(decodedModule, /origin sample cancels the diagonal correction/i);
  assert.match(decodedModule, /M\(2\+M\)/);
  assert.match(decodedModule, /knowledge\/crouzeix_conjecture\/04_jin_positive_real_completion\.md/);
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
