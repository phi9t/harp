# Harp Architecture Simplification Issues

> Implementation sessions should take exactly one issue from this file. Start
> each issue in a fresh worktree from clean `master`.

## Shared Simplicity Gate

Before editing code for any issue, write down the issue-specific answers:

```text
What caller knowledge does this delete?
Which interface shrinks?
Where does locality improve?
What authority does this avoid duplicating?
If we delete the new module, where does the complexity reappear?
```

Do not implement an issue if those answers are weak.

## Issue 1: DurableRunCommand

**Goal:** collapse shared `rlm` and `workflow` durable run command behavior
without changing the Engine or command output.

**Primary files:**

- `crates/harp/src/main.rs`
- new internal command module under `crates/harp/src/`
- `crates/harp/tests/cli.rs`
- `crates/harp/tests/rlm_process_restart.rs`

**Required behavior:**

- `rlm run` and `workflow run` still accept the same arguments.
- `rlm resume` and `workflow resume` share selection-mode validation.
- Status JSON for shared fields is produced through one implementation.
- Workflow-specific metadata stays owned by the workflow adapter.
- Runtime adapters remain `ActivityRuntime` implementations; no workflow
  semantics move into adapters.

**Suggested tests:**

- Shared resume-selection table test for `rlm` and `workflow`.
- Shared status-shape test for completed and incomplete runs.
- Regression test proving `workflow.run` still reports `workflow_name` and
  `compiled_graph_sha256`.
- Existing RLM restart tests unchanged.

**Verification:**

```sh
cargo test -p harp --test cli -- --test-threads=1
cargo test -p harp --test rlm_process_restart -- --test-threads=1
mise run verify
```

## Issue 2: AtlasCorpusView

**Goal:** make Atlas views consume a single generated-corpus projection module
instead of repeating cross-collection joins.

**Primary files:**

- `atlas/src/content/canonical.ts`
- `atlas/src/app/AtlasApp.tsx`
- `atlas/src/app/SystemLibrary.tsx`
- `atlas/src/app/SystemArticle.tsx`
- `atlas/src/app/WengReader.tsx`
- `atlas/src/app/LessonRunner.tsx`
- `atlas/src/content/canonical.test.ts`

**Required behavior:**

- Generated `corpus.json` remains the data authority.
- Strict parsing remains in `atlas/src/content/contracts.ts`.
- Views call projection helpers instead of rejoining arrays.
- Routes and visible behavior do not change.
- Offline export remains file-based and serverless.

**Suggested tests:**

- Direct corpus-view tests for document, route, system, Weng section, lesson,
  and diagnostic-case projections.
- Deterministic missing-reference errors in tests.
- Existing React tests remain focused on rendering behavior.

**Verification:**

```sh
cd atlas && corepack pnpm run test
cd atlas && corepack pnpm run test:export
mise run verify
```

## Issue 3: SideEffectPolicy State And Receipt Contract

**Goal:** add the durable state and bounded receipt contract needed for
`WorkspaceMode::GitWorktree` side effects.

**Primary files:**

- `crates/harp-state/src/model.rs`
- `crates/harp-state/src/store.rs`
- `crates/harp-state/src/transitions/`
- `crates/harp-state/migrations/`
- `crates/harp-contracts/src/`
- `crates/harp-state/tests/transitions.rs`
- `crates/harp-state/tests/recovery_reads.rs`

**Required behavior:**

- Side-effect state persists in a dedicated table.
- Receipt identity and replay disposition are queryable by run, task, attempt,
  and effect slot.
- Receipt fields are bounded and strict.
- Dirty worktree admission remains a future concern for Engine enforcement,
  not a state-layer behavior.

**Suggested tests:**

- Migration tests for old database versions.
- Dedicated table invariants and foreign-key checks.
- Round-trip parsing for receipt dispositions.
- Recovery reads expose side-effect state without exposing workflow semantics.

**Verification:**

```sh
cargo test -p harp-state -- --test-threads=1
cargo test -p harp-contracts --test schema -- --test-threads=1
mise run verify
```

## Issue 4: SideEffectPolicy Engine Enforcement

**Goal:** require side-effect admission and receipt publication for every
`TaskGraph` node with `WorkspaceMode::GitWorktree`.

**Primary files:**

- `crates/harp-engine/src/scheduler.rs`
- new `crates/harp-engine/src/side_effect.rs`
- `crates/harp-engine/src/projection.rs`
- `crates/harp-engine/tests/execution.rs`
- `crates/harp-engine/tests/dynamic_workflow.rs`

**Required behavior:**

- `ReadOnly` and `Scratch` do not invoke the policy.
- `GitWorktree` requires clean admission before runtime dispatch.
- Idempotency keys remain Engine-only.
- Worktree identity is recorded before runtime dispatch.
- Mutation receipts are published before task acceptance.
- Runtime adapters do not interpret mutation policy.

**Suggested tests:**

- `GitWorktree` node with clean admission and no mutation yields `unchanged`.
- Dirty admission fails before runtime work.
- Changed worktree yields bounded `changed` receipt.
- Result publication cannot bypass a missing receipt.

**Verification:**

```sh
cargo test -p harp-engine --test execution -- --test-threads=1
cargo test -p harp-engine --test dynamic_workflow -- --test-threads=1
mise run verify
```

## Issue 5: SideEffectPolicy Recovery Fixtures

**Goal:** prove restart behavior for admitted or dispatched side effects.

**Primary files:**

- `crates/harp-engine/src/recovery.rs`
- `crates/harp-engine/src/scheduler.rs`
- `crates/harp-engine/src/side_effect.rs`
- `crates/harp-engine/tests/recovery.rs`

**Required behavior:**

- Crash before runtime dispatch can rerun only if the admitted worktree identity
  still matches.
- Crash after runtime dispatch reconciles before any new provider activity.
- Published receipts are reused on resume.
- Drifted worktree identity fails closed.
- Too-large mutation evidence terminalizes with bounded evidence.

**Suggested tests:**

- Crash after receipt publication performs no new semantic work.
- Crash after dispatch without receipt reconciles first.
- External worktree drift after admission becomes `ambiguous` or `rejected`.
- Too-large changed-path set does not embed unbounded data.

**Verification:**

```sh
cargo test -p harp-engine --test recovery -- --test-threads=1
mise run verify
```

## Landing Rules

- Use an isolated worktree for every issue.
- Stage explicit paths only.
- Refresh `docs/import-receipt.md` after the final tracked change.
- Run `mise run verify` before local landing.
- Commit with:

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```

- Do not push unless explicitly requested.
