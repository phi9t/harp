# Contributing to Harp

## Bootstrap

```sh
mise run bootstrap
```

The locked Cargo and pnpm registries may be used during bootstrap. Normal
build, validation, reading, search, and offline source verification must not
contact research sites.

## Content changes

1. Edit canonical Markdown or registries under `content/`.
2. Run `cargo run -p harp -- check`.
3. Run `cargo run -p harp -- build`.
4. Refresh search with `cargo run -p harp -- search refresh`.
5. Rebuild the offline Atlas with `cd atlas && corepack pnpm run test:export`.
6. Run `mise run verify`.

Do not duplicate technical explanations in TypeScript. New canonical Markdown
must have one stable role, one source/claim ceiling, valid local links, and
coverage ownership where applicable.

For source-backed research summaries, technical articles, and deep dives, use
the [credible technical documentation style guide](writing-style/STYLE_GUIDE.md).
Material claims follow a clickable route from main prose to a heading-based
claim-ledger entry and then to the exact local source locator.

## Evidence changes

Do not edit captured upstream bytes to normalize formatting, names, or paths.
Evidence refreshes are explicit:

```sh
evidence/weng/acquire.sh --refresh
evidence/rlm/acquire.sh --refresh
```

Review changed upstream bytes, source identity, licenses, manifests, claim
ceilings, and generated receipts before accepting a refresh.

For public implementation studies:

1. Pin a 40-character commit in `evidence/implementations/manifest.tsv`.
2. Track only files referenced by maintained locators, preserving upstream
   relative paths beneath `snapshot/`.
3. Preserve the upstream license or record
   `not-present-at-pinned-revision	MISSING`.
4. Run `harp sources verify`.
5. Run `harp sources materialize --source ID` to compare against a clean
   public checkout.

## Rust changes

Use typed errors and bounded boundary parsing. Keep filesystem writes
repository-relative, symlink-safe, atomic, and race-aware. Add a failing test
before production behavior changes. The inherited corpus tests cover malformed
registries, unknown systems, incomplete routes, diagnostic contracts, duplicate
IDs, source mapping, escapes, symlink output, and byte stability.

## Atlas changes

Preserve strict TypeScript, boundary parsing from `unknown`, branded IDs,
discriminated unions, and exhaustive matching. Run:

```sh
cd atlas
corepack pnpm run lint
corepack pnpm run typecheck
corepack pnpm run test
corepack pnpm run test:export
```

The decoded export test must inspect the inlined JavaScript rather than only
checking that an HTML file exists.

## Licensing

Do not add a repository-wide license or claim that captured works are
relicensed. Preserve source-specific license files and explicit unknown states.
