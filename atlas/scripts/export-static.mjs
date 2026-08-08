import { createHash } from "node:crypto";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

const distDir = new URL("../dist/", import.meta.url);
const clientDir = join(distDir.pathname, "client");
const indexPath = join(clientDir, "index.html");
const atlasPath = join(distDir.pathname, "harp-atlas.html");
const receiptPath = join(distDir.pathname, "harp-atlas.receipt.json");

let html = await readFile(indexPath, "utf8");
if (!html.includes("Harp Atlas")) {
  throw new Error("dist/client/index.html does not look like Harp Atlas");
}

const scriptMatch = html.match(/<script type="module" crossorigin src="([^"]+)"><\/script>/);
if (!scriptMatch) {
  throw new Error("dist/client/index.html does not contain the expected Vite module script");
}

const styleMatch = html.match(/<link rel="stylesheet" crossorigin href="([^"]+)">/);
if (!styleMatch) {
  throw new Error("dist/client/index.html does not contain the expected Vite stylesheet");
}

const scriptPath = join(clientDir, scriptMatch[1].replace(/^\.\//, ""));
const stylePath = join(clientDir, styleMatch[1].replace(/^\.\//, ""));
const script = await readFile(scriptPath, "utf8");
const style = await readFile(stylePath, "utf8");
const moduleUrl = "data:text/javascript;base64," + Buffer.from(script, "utf8").toString("base64");

html = html
  .replace(styleMatch[0], "<style>\n" + style + "\n</style>")
  .replace(scriptMatch[0], "<script type=\"module\">\nimport " + JSON.stringify(moduleUrl) + ";\n</script>");

await writeFile(atlasPath, html);
const appInputPaths = [
  "index.html",
  "package.json",
  "scripts/export-static.mjs",
  "tsconfig.json",
  "vite.config.ts",
  "vitest.config.ts",
];
const sourceRoot = new URL("../", import.meta.url);
const sourceFiles = [];

async function collectFiles(directoryUrl) {
  const entries = await readdir(directoryUrl, { withFileTypes: true });
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const entryUrl = new URL(entry.name + (entry.isDirectory() ? "/" : ""), directoryUrl);
    if (entry.isDirectory()) {
      await collectFiles(entryUrl);
    } else if (/\.(css|ts|tsx)$/.test(entry.name)) {
      sourceFiles.push(entryUrl);
    }
  }
}

await collectFiles(new URL("src/", sourceRoot));
const inputHash = createHash("sha256");
const inputFiles = [
  ...appInputPaths.map((relative) => ({
    relative,
    fileUrl: new URL(relative, sourceRoot),
  })),
  ...sourceFiles.map((fileUrl) => ({
    relative: decodeURIComponent(fileUrl.pathname.slice(sourceRoot.pathname.length)),
    fileUrl,
  })),
].sort((left, right) =>
  left.relative < right.relative ? -1 : left.relative > right.relative ? 1 : 0
);
for (const { relative, fileUrl } of inputFiles) {
  const bytes = await readFile(fileUrl);
  inputHash.update(relative);
  inputHash.update("\0");
  inputHash.update(bytes);
}
const corpus = await readFile(new URL("../src/content/generated/corpus.json", import.meta.url));
const receipt = {
  schema_version: "harp-atlas-export/v1",
  corpus_sha256: createHash("sha256").update(corpus).digest("hex"),
  app_inputs_sha256: inputHash.digest("hex"),
  html_sha256: createHash("sha256").update(html).digest("hex"),
};
await writeFile(receiptPath, JSON.stringify(receipt, null, 2) + "\n");
console.log("wrote " + atlasPath);
