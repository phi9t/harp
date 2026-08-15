# Contributing to Harp

## Bootstrap

```sh
mise run bootstrap
```

The locked Cargo and pnpm registries may be used during bootstrap. Normal
build, validation, reading, search, and offline source verification must not
contact research sites.

## Agentic engineering

Harp follows the agentic engineering reference captured under
`knowledge/agentic_engineering/`: scale agent execution, not agent authority.
Humans retain intent, architecture, merge, release, and authority-widening
decisions. Agents work on bounded tasks in isolated worktrees, with explicit
verification and repair before landing.

The local constitution for agent behavior is: honor the request, act with
judgment, finish authorized work, protect existing work, verify reality,
communicate for humans, and learn in shared project files. Put durable process
lessons in `AGENTS.md`, `CONTEXT.md`, ADRs, maintained docs, tests, or code;
do not rely on private agent memory as a project interface.

### Local tools

This repository is bound to the local Kata project `harp` through `.kata.toml`.
Use Kata as the shared intent ledger for real Harp work; `.kata.local.toml` is
ignored for per-machine daemon overrides. Do not run `kata init --with-agents`
because Harp owns its agent guidance directly.

roborev may be used for independent local verification, but hooks, daemons,
GitHub App integration, PR comments, and CI polling require explicit approval.
AgentsView may be used for local session and token observability; its records
inform later distillation but are not themselves project guidance.

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

When authored material encounters an AlphaXiv reference, resolve the paper and
link its canonical `https://arxiv.org/abs/...` abstract page. Do not rewrite
AlphaXiv strings or any other references inside captured evidence; captured
upstream bytes remain byte-faithful.

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

## Context-control changes

Keep repository policy restrictions-only. `.harp/context-control.json` may
disable Harp, narrow workflows, pin a release, lower a context budget, or
declare required verification labels; it must not inject prompts, choose
executables, or broaden sandbox, approval, network, tool, MCP, or
authentication authority. Labels are currently validated and included in the
policy digest, but no verifier or outcome gate enforces them yet.

Provider tests use fake executables for the selected Trae CLI or Codex CLI
adapter. Preserve the `harp run` stream contract in every end-to-end test:
byte-exact provider stdout requested in JSONL mode is the only stdout, while
Harp lifecycle records and the final episode ID use stderr. Preserve malformed
provider output unchanged. `mise run verify` must never launch a real provider
or require provider credentials.

Document context manifests as partial until a provider exposes enough
telemetry to reconstruct the complete effective prompt, native skill
selection, and compaction state. Do not describe ACE/MCE suggestion generation,
evaluation, canarying, promotion, or rollback as implemented.

## Licensing

Do not add a repository-wide license or claim that captured works are
relicensed. Preserve source-specific license files and explicit unknown states.
