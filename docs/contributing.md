# Contributing to Harp

[Documentation](README.md) / Contributing

Start with [getting started](getting-started.md#build-the-cli) to install the
pinned tools, fetch LFS evidence, and build the CLI. Read [AGENTS.md](../AGENTS.md)
for repository rules and the [product contract](product-contract.md) for
ownership and behavior. Harp must remain standalone; do not depend on another
local checkout.

The locked Cargo registry may be used during bootstrap. Normal
build, validation, reading, search, and offline source verification must not
contact research sites.

## Verification

Use focused checks while iterating:

```sh
mise exec -- cargo test -p harp corpus::tests --lib -- --test-threads=1
mise exec -- cargo test -p harp --test cli
```

Some CLI and source-verification tests require the Lean evidence environment.
Selected proof checks and the release gate require the pinned Lean toolchain
and an already populated dependency cache. Check those prerequisites without invoking Lake:

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

This selects checks for the changed surfaces; inspect `mise run verify -- --plan`.
Use `mise run verify-all` for the complete release gate. The gate uses fake provider
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
4. Run the changed-surface `mise run verify` gate before committing and landing locally.

Use Mise for Git commands that invoke LFS hooks, such as
`mise exec -- git commit`. Push only when the project owner explicitly asks.
For executable packaging, follow [native release candidates](releasing.md).

## Agentic engineering

Harp follows the agentic engineering reference captured under
`lab/agentic_engineering/`: scale agent execution, not agent authority.
Humans retain intent, architecture, merge, release, and authority-widening
decisions. Agents work on bounded tasks in isolated worktrees, with explicit
verification and repair before landing.

The local constitution for agent behavior is: honor the request, act with
judgment, finish authorized work, protect existing work, verify reality,
communicate for humans, and learn in shared project files. Put durable process
lessons in `AGENTS.md`, `CONTEXT.md`, ADRs, maintained docs, tests, or code;
do not rely on private agent memory as a project interface.

### Local tools

Git-managed Markdown under `.scratch/` is the shared intent and issue tracker.
Follow [the tracker contract](agents/issue-tracker.md). Kata is retired; retained
Corenius adapters do not change that ownership.

roborev may be used for independent local verification, but hooks, daemons,
GitHub App integration, PR comments, and CI polling require explicit approval.
AgentsView may be used for local session and token observability; its records
inform later distillation but are not themselves project guidance.

## Content changes

1. Edit canonical technical prose under `lab/`; use `lab/math/textbook/` only for
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
7. Follow [the landing checklist](#land-a-change), including the full gate.

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
[Harp knowledge home](../lab/harp_knowledge_home.md). `lab/` is still
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
  packet tests, and regenerate the headless corpus. Do not make
  a packet canonical merely to make it discoverable.

## Evidence changes

Do not edit captured upstream bytes to normalize formatting, names, or paths.
Review changed upstream bytes, source identity, licenses, manifests and claim
ceilings before accepting an explicit evidence refresh.

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
before production behavior changes. Corpus tests protect canonical identities, source mapping, safe links,
path escapes, symlink output and byte stability.

## Execution and context changes

The selected execution authority is the lightweight runtime behind `harp durable`.
Keep task intent, admission, worker effects, verification, human approval and
landing distinct. Test at the public CLI, real Git and durable-state boundaries.
Cancellation or recovery must not silently launch an old controller or reuse an
uncertain effect.

The direct `harp run` and old `rlm`/`workflow` commands are retired. Regression
tests must prove refusal precedes input reads, private-state initialization,
provider probes and process launch. Historical database bytes must remain intact.
Preserve the reusable graph/result contracts, including Crouzeix source-contract
decoder tests, when removing execution callers.

Capability probes and immutable context releases remain supported. Context
policy descriptions stay restriction-only; they cannot inject prompts or broaden
authority. Do not describe historical partial context records as complete prompt,
native skill or compaction telemetry.

Provider adapter tests use process fixtures. Explicit Linux qualification uses
real namespaces, delegated cgroups and supported SQLite. `mise run verify` must
never launch a real provider or require provider credentials. Live model runs
need their own explicit qualification evidence; an offline pass does not replace it.

## Licensing

Do not add a repository-wide license or claim that captured works are
relicensed. Preserve source-specific license files and explicit unknown states.
