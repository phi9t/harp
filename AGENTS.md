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

## Workspace management and landing

- Do not push unless the project owner explicitly asks for a push.
- Start new feature work in an isolated git worktree. Keep the primary checkout
  operator-facing and clean.
- Land locally with focused, independently verifiable commits grouped by
  concern. Do not use `git add .`; stage explicit path groups.
- If the primary checkout is dirty, classify every path before acting:
  restore accidental derived-file deletions, ignore editor/build scratch, and
  commit only coherent repo-owned artifacts.
- Refresh `docs/import-receipt.md` only after the rest of the tracked payload is
  settled. Copy the digest reported by `harp repository verify`, then rerun the
  relevant verifier.
- Run `mise run verify` before a local landing unless the owner explicitly asks
  for a narrower gate. Use focused tests while iterating, but do not treat them
  as a release gate.
- Preserve unrelated local changes. If a dirty path is not part of the current
  landing, leave it alone or add an ignore rule for clearly local scratch.

## Agentic engineering operating model

- Scale agent execution, not agent authority. Humans own intent, architecture,
  merge, release, and any authority-widening decision.
- Keep intent, reasoning, execution, verification, observability, and human
  control separate. Do not blur a prompt, transcript, issue, commit, or memory
  into the system of record for intent.
- Use agents for bounded implementation in isolated worktrees. Parallel agents
  need non-overlapping work and independently verifiable issue surfaces.
- Treat independent verification as a first-class budget. Tests, adversarial
  review, and repair are part of the work, not optional cleanup after code
  generation.
- Follow the local constitution: honor the request, act with judgment, finish
  authorized work, protect existing work, verify reality, communicate for
  humans, and learn in shared project files.
- Distill durable lessons into `AGENTS.md`, `CONTEXT.md`, ADRs, maintained
  docs, tests, and code. Do not let private agent memory or raw execution notes
  become the durable project interface.

## Local agentic engineering tools

- Kata is the local intent ledger for this repository. The committed
  `.kata.toml` binds the workspace to the `harp` project; `.kata.local.toml`
  is ignored for per-machine daemon overrides.
- Use Kata for real Harp work only. Search before creating issues, prefer
  updating existing issues, and close only when work is verified with evidence.
  Do not run `kata init --with-agents`; Harp owns this guidance.
- roborev is an independent verification tool, not merge authority. Do not run
  `roborev init`, install hooks, start daemons, configure GitHub integration,
  or post PR comments unless explicitly requested.
- AgentsView is observability for local agent sessions and token/activity
  analysis. Do not treat AgentsView records as durable project guidance until
  a lesson is distilled into shared repo files.
- For no-daemon smoke checks, use `kata version`, `roborev version`, and
  `agentsview --version`. Commands such as `kata list`, `roborev status`, and
  `agentsview projects` may start local services.
- Forge, Ghosthub, Kata federation, remote daemons, GitHub sync, PostgreSQL
  sync, and persistent background services require explicit approval before
  setup.

## Licensing

Do not add a repository-wide license unless the project owner explicitly
chooses one. Preserve upstream license files and status records. Missing
upstream license evidence must remain explicit rather than inferred.
