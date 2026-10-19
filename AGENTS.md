# Harp contributor guide

Harp is a standalone agent engineering product with a mathematics and education library. Do not add runtime, build, test, source, or
documentation dependencies on another local checkout.

## Canonical ownership

- `cli/` is the operator interface; `sessions/` indexes rollouts for observability.
  Neither owns worker admission or landing.

- `durable/` owns agent execution, its SQLite journal, Git-managed intent
  bindings, independent verification and landing. `insula/` owns containment and
  service supervision; `runner/` is SSH transport. Do not add a competing
  controller or dispatcher.
- Actual SSH inventories and credentials are private, ignored inputs. Public
  examples use fictional hosts and paths.
- `.scratch/` owns Git-managed specs and issues; `lab/` holds workloads and their
  documentation. Imported benchmark fixture tests retain their external format.


- Managed Markdown in registered topic packets under `lab/` is the
  authority for technical prose. Structured contracts live beside their owners: `lab/math/textbook/`,
  `lab/sicp/contracts/` and `durable/intent/bundles/`. Do not add Markdown beneath the
  textbook JSON contract directory.
- `lab/generated/corpus.json` is derived and must be regenerated with
  its canonical inputs. The Atlas browser reader and HTML export are retired.
- Crouzeix publication outputs live under
  `publications/crouzeix_textbook/` and remain ignored.
- `evidence/` contains captured upstream bytes, source-specific licenses,
  manifests, and narrow public-source snapshots.
- Captured files under `evidence/*/artifacts/` must not be rewritten for
  branding or formatting.
- `evidence/implementations/obsidian_skills/` is a pinned upstream syntax-skill
  snapshot, not a Harp prose authority. Update it only through the
  implementation-manifest workflow with its complete upstream byte set,
  revision, license record, and digests.
- The repository root is the portable Obsidian vault. `lab/` remains the
  prose authority; root-qualified wikilinks are the managed internal
  navigation form. Preserve exact raw-evidence locations as the dual form:
  a native artifact wikilink plus the conventional Markdown `#L...` locator.
- Keep `.obsidian/` ignored. The reviewed portable profile is
  `tools/obsidian/profile/`; update it only with the allowlisted,
  symlink-safe installer and asset-validator coverage.
- `docs/import-receipt.md` is the only file that may name the source repository
  used for the initial import.

## Verification

Run `mise run verify` before local landing. It selects checks from changed paths;
inspect `mise run verify -- --plan` and use `--base <commit-or-ref>` to cover an
entire candidate spanning multiple commits. See
[the verification contract](docs/agents/verification.md) for selection, affected
consumers, and prerequisite handling. Supplement the selected checks when a
change affects a contract beyond its path; record that evidence in its ticket.

`mise run verify-all` is the explicit release gate. Release candidates always use
it. Ordinary local landings do not require unrelated Lean, React, or repository
import prerequisites. A missing dependency for a selected check remains blocking;
never silently skip it or install/hydrate prerequisites without authorization.

Keep Rust errors typed, validate JSON at boundaries, reject symlinked generated
targets, and preserve strict TypeScript boundary parsing and exhaustive variant
handling.

### Test selection

- Colocate new or moved tests and fixtures with the source they protect. Use
  adjacent test modules or in-module tests; configure native integration targets
  explicitly where needed instead of adding separate tests/ trees. Preserve
  behavioral coverage when relocating imported tests.

- Prefer the smallest set of tests that protects useful observable behavior:
  public API and CLI contracts, data schemas, boundary validation, and meaningful
  integration outcomes. Choose checks by the regression they can catch, not a
  coverage percentage or test count.
- Leave private helpers, internal call order, class layout, incidental formatting,
  and intermediate representations free to change unless they are part of an
  explicit contract. A refactor that preserves the contract should usually pass
  existing tests. Use internal tests when they protect a meaningful algorithmic
  invariant that boundary tests cannot expose economically.
- Before adding a test, name the behavior or invariant it protects. Reuse existing
  coverage where sufficient; avoid duplicate assertions and mocks that reproduce
  the implementation. For reversible prose-only edits, inspect the diff and
  validate document/skill structure rather than adding wording-matching tests.
- When a test fails, determine whether a required contract regressed or the test
  freezes an incidental implementation choice. For the latter, update or remove
  the test within the affected scope while retaining meaningful contract coverage.
  Do not weaken a real contract to make a failure disappear or undertake unrelated
  test deletion. Existing required gates still apply unless explicitly narrowed.

### Lean cache discipline

- Treat the primary checkout's `lab/math/lean/.lake` as the canonical
  machine-local dependency cache. A Lean feature worktree should use a
  symlink to that cache after verifying its target and ancestry.
- Never hydrate or update common Lean dependencies during proof iteration. Do
  not run `lake update`, `lake --try-cache exe cache get Mathlib`, or
  `mise run lean-cache` unless the project owner explicitly requests cache
  maintenance. Missing cache state is a blocked precondition.
- Rebuilding Harp-owned modules against the warm cache is expected. Run the
  relevant route target for iteration and local landing. Use `lean-all` for
  shared proof infrastructure changes or the explicit release gate.
- The shared `.lake/build` also contains Harp-owned artifacts. Serialize
  writes from different worktrees through receipt extraction; another
  worktree can otherwise replace the candidate's `.olean` files. Concurrent
  narrow iteration may use direct pinned Lean with a private temporary
  Harp-output prefix and the existing dependency artifacts. Keep the canonical
  `.lake` symlink unchanged, and reserve shared-output gates for a frozen,
  coordinated candidate.
- Before an expensive gate, verify the pinned toolchain, cache link, required
  dependency artifacts, worktree-local mise trust, and exact target without
  invoking Lake.

## Workspace management and landing

- Do not push unless the project owner explicitly asks for a push.
- Start new feature work in an isolated git worktree. Keep the primary checkout
  operator-facing and clean.
- Land locally with focused, independently verifiable commits grouped by
  concern. Do not use `git add .`; stage explicit path groups.
- If the primary checkout is dirty, classify every path before acting:
  restore accidental derived-file deletions, ignore editor/build scratch, and
  commit only coherent repo-owned artifacts.
- When repository/import verification applies, refresh `docs/import-receipt.md`
  only after the tracked payload is settled. Copy the digest reported by
  `harp repository verify`, then rerun that verifier.
- Run the changed-surface `mise run verify` gate before local landing. Review its
  selected paths/checks against the complete candidate and add relevant consumer
  checks as needed. Run `verify-all` for releases or when explicitly requested.
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

## Repository-owned workflow skills

Use the Harp skills in `.agents/skills/` for this repository. Do not invoke the
upstream Superpowers skillset, its `superpowers:*` commands, or renamed
`superpowers-*` wrappers. Historical documents and license attribution do not
select an execution workflow. Specialized skills outside that workflow family
remain available when relevant.

- [harp-outcome-design](.agents/skills/harp-outcome-design/SKILL.md): resolve open
  design decisions around the first useful result and existing authorization.
- [harp-delivery-plan](.agents/skills/harp-delivery-plan/SKILL.md): plan verifiable
  slices; justify each prerequisite by the operation it enables and why needed now.
- [harp-task-execution](.agents/skills/harp-task-execution/SKILL.md): execute and
  repair authorized work through complete verification and repository landing rules.
- [harp-team-execution](.agents/skills/harp-team-execution/SKILL.md): coordinate
  permitted workers with bounded ownership and independent integration evidence.
- [harp-scope-review](.agents/skills/harp-scope-review/SKILL.md): distinguish current
  defects and requirements from optional future infrastructure.

These files are operational agent instructions, not additional technical-prose
or intent authorities. Read only the skill relevant to the current decision;
there is no mandatory five-skill ceremony. Existing intent, safety, cache,
verification, and human merge/release boundaries still apply. An initial useful
result does not complete the full request. Use the
[behavior cases](.agents/skills/evaluation-cases.md) when changing these workflows.

## Local agentic engineering tools

- Git-managed Markdown is the intent and issue tracker. Read
  [the tracker contract](docs/agents/issue-tracker.md) before creating or updating
  tickets. Use `.scratch/<feature>/spec.md` and one file per issue under
  `.scratch/<feature>/issues/`; search existing records before adding another.
- Use the Matt Pocock skills' local-Markdown tracker convention. Publishing means
  editing and committing repository files, with no tracker service or CLI required.
  [Triage roles](docs/agents/triage-labels.md) use the default label names.
- Kata is retired for Harp. Do not initialize, query, or update it for new work.
  Historical Kata references are context, not instructions; carry active work into
  a local ticket when resumed, preserving its old identifier and available evidence.
- roborev is an independent verification tool, not merge authority. Do not run
  `roborev init`, install hooks, start daemons, configure GitHub integration,
  or post PR comments unless explicitly requested.
- AgentsView is observability for local agent sessions and token/activity
  analysis. Do not treat AgentsView records as durable project guidance until
  a lesson is distilled into shared repo files.
- For no-daemon smoke checks, use `roborev version` and
  `agentsview --version`. Commands such as `roborev status` and
  `agentsview projects` may start local services.
- Forge, Ghosthub, remote daemons, GitHub sync, PostgreSQL
  sync, and persistent background services require explicit approval before
  setup.

## Licensing

Do not add a repository-wide license unless the project owner explicitly
chooses one. Preserve upstream license files and status records. Missing
upstream license evidence must remain explicit rather than inferred.
