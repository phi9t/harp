# Contributing to Harp

[Documentation](README.md) / Contributing

Start with [getting started](getting-started.md#build-the-cli) to install the
pinned tools, fetch LFS evidence, and build the CLI. Read [AGENTS.md](../AGENTS.md)
for repository rules and the [product contract](product-contract.md) for
ownership and behavior. Harp must remain standalone; do not depend on another
local checkout.

The locked Cargo and pnpm registries may be used during bootstrap. Normal
build, validation, reading, search, and offline source verification must not
contact research sites.

## Verification

Use focused checks while iterating:

```sh
mise exec -- cargo test -p harp corpus::tests --lib -- --test-threads=1
mise exec -- cargo test -p harp --test cli
mise exec -- corepack pnpm --dir atlas run test
```

Some CLI and source-verification tests require the Lean evidence environment.
The full landing gate requires the pinned Lean toolchain and an already
populated dependency cache. Check those prerequisites without invoking Lake:

```sh
mise run lean-env
```

In a feature worktree, use a symlink from `formalization/lean/.lake` to the
primary checkout's cache, after verifying the target and its ancestry. Do not
run `lake update`, `lake --try-cache exe cache get Mathlib`, or
`mise run lean-cache` during ordinary work. Missing cache state is a blocked
prerequisite; ask the project owner for cache maintenance.

Once the changes are settled, run:

```sh
mise run verify
```

This runs the Atlas, Rust, shell, Python, Lean, LFS, and repository checks. It
can take substantially longer than a focused test. It uses fake provider
fixtures, not live model sessions. A local Lean build is not a claim of
hermetic proof execution.

## Land a change

Start new work in an isolated Git worktree. Keep commits focused and stage
explicit path groups. Preserve unrelated changes in the primary checkout.

The import receipt covers the tracked payload, including documentation:

1. Settle the intended changes and stage their explicit paths so new files are
   included in the payload.
2. Run `.build/harp-target/size/harp repository verify` from the worktree.
3. If it reports a stale payload digest, copy the expected digest into
   `docs/import-receipt.md`, stage that file, and rerun the verifier.
4. Run the full `mise run verify` gate before committing and landing locally.

Use Mise for Git commands that invoke LFS hooks, such as
`mise exec -- git commit`. Push only when the project owner explicitly asks.
For executable packaging, follow [native release candidates](releasing.md).

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

1. Edit canonical technical prose under `knowledge/`; use `content/` only for
   structured machine-readable contracts and diagnostics.
2. Use vault-root-qualified wiki links for managed note, claim-ledger, and
   artifact navigation. Preserve exact raw-evidence line locators as the
   documented dual form: native artifact wiki link plus the original Markdown
   `#L...` locator.
3. Run `python3 scripts/migrate_obsidian_links.py --check` before committing a
   prose change. Resolve every reported local-link error; do not use `--write`
   without reviewing its diff.
4. Run `.build/harp-target/size/harp check`.
5. Run `.build/harp-target/size/harp build`.
6. Refresh search with `.build/harp-target/size/harp search refresh`.
7. Rebuild and test the offline Atlas with
   `mise exec -- corepack pnpm --dir atlas run test:export`.
8. Follow [the landing checklist](#land-a-change), including the full gate.

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

### Obsidian-native knowledge presentation

Open the repository root as the Obsidian vault and begin at
[Harp knowledge home](../knowledge/harp_knowledge_home.md). `knowledge/` is still
the sole technical-prose authority; Obsidian is a reader and navigation layer,
not a second source of truth.

- Author managed internal navigation with vault-root-qualified wikilinks.
  Keep external URLs as Markdown links. Never rewrite
  `evidence/*/artifacts/` captures for Obsidian syntax.
- Exact raw-evidence locations remain dual-linked: a native artifact wikilink
  for vault navigation and the conventional Markdown `#L...` locator for the
  immutable line reference.
- Keep personal `.obsidian/` state ignored. The portable reviewed profile lives
  in `tools/obsidian/profile/`; apply it explicitly with:

  ```sh
  python3 tools/obsidian/apply_profile.py --vault "$(git rev-parse --show-toplevel)"
  ```

  The installer is create-only by default. Validate the committed profile,
  Base, Canvas, and knowledge-home assets with
  `python3 scripts/validate_obsidian_assets.py`.
- A registered packet must define valid metadata, add its first-class route or
  auxiliary-document registration as appropriate, extend search roots, add
  packet tests, and regenerate the corpus and Atlas projection. Do not make
  a packet canonical merely to make it discoverable.

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
mise exec -- corepack pnpm run lint
mise exec -- corepack pnpm run typecheck
mise exec -- corepack pnpm run test
mise exec -- corepack pnpm run test:export
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
