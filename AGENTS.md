# Harp contributor guide

Harp is a standalone RSI product. Do not add runtime, build, test, source, or
documentation dependencies on another local checkout.

## Canonical ownership

- Managed Markdown under `knowledge/rsi/` and registered topic packets under
  `knowledge/` are the only authorities for technical prose. `content/` is
  reserved for structured machine-readable contracts and diagnostics; do not
  add Markdown beneath it.
- `atlas/src/content/generated/corpus.json` and
  `atlas/dist/harp-atlas.html` are derived and must be regenerated together
  with their canonical inputs.
- `evidence/` contains captured upstream bytes, source-specific licenses,
  manifests, and narrow public-source snapshots.
- Captured files under `evidence/*/artifacts/` must not be rewritten for
  branding or formatting.
- `docs/import-receipt.md` is the only file that may name the source repository
  used for the initial import.

## Verification

Run `mise run verify` before committing. Use focused tests while iterating:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cd atlas && corepack pnpm run test
```

Keep Rust errors typed, validate JSON at boundaries, reject symlinked generated
targets, and preserve strict TypeScript boundary parsing and exhaustive variant
handling.

## Licensing

Do not add a repository-wide license unless the project owner explicitly
chooses one. Preserve upstream license files and status records. Missing
upstream license evidence must remain explicit rather than inferred.
