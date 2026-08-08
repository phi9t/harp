import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { rm } from "node:fs/promises";
import { resolve } from "node:path";

const isCodexSeatbeltSandbox = process.env.CODEX_SANDBOX === "seatbelt";
const fileEntryRedirect = /\s*<script id="file-entry-redirect">[\s\S]*?<\/script>/;

export default defineConfig({
  base: "./",
  build: {
    outDir: "dist/client",
    chunkSizeWarningLimit: 900,
    modulePreload: {
      polyfill: false,
    },
  },
  plugins: [
    react(),
    {
      name: "strip-file-entry-redirect",
      apply: "build",
      transformIndexHtml(html) {
        if (!fileEntryRedirect.test(html)) {
          throw new Error("root index is missing the expected file-entry redirect");
        }
        return html.replace(fileEntryRedirect, "");
      },
    },
    {
      name: "clean-rsi-dist",
      apply: "build",
      async buildStart() {
        await rm(resolve("dist"), { recursive: true, force: true });
      },
    },
  ],
  server: isCodexSeatbeltSandbox
    ? { watch: { useFsEvents: false, usePolling: true } }
    : undefined,
});
