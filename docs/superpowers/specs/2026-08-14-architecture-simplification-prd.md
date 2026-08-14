# Harp Architecture Simplification PRD

## Objective

Make Harp easier for agents and maintainers to operate by deepening three
high-leverage modules without adding a second authority for execution, corpus
identity, or side-effect policy.

The work is not a general cleanup. A candidate lands only when it deletes
caller knowledge and makes one existing concept easier to test through a
smaller interface.

## Scope

This PRD covers three candidates:

1. `DurableRunCommand` for shared `rlm` and `workflow` command execution.
2. `AtlasCorpusView` for frontend access to the generated corpus.
3. `SideEffectPolicy` implementation follow-through from the approved design.

The `SideEffectPolicy` design already exists at
`docs/superpowers/specs/2026-08-13-dynamic-workflow-side-effect-policy-design.md`.
This PRD references that design and adds the shared simplicity gate. It does
not reopen the side-effect design unless implementation proves a contradiction.

## Non-goals

- Do not create a second durable executor.
- Do not create a second corpus authority.
- Do not create a second route policy for Atlas.
- Do not move Dynamic Workflow semantics into runtime adapters.
- Do not introduce a framework-wide architecture layer.
- Do not land all candidates in one implementation branch.
- Do not change TypeScript/Bun/frontend scope beyond the Atlas simplification
  issue.

## Simplicity Gate

Every issue generated from this PRD must answer these questions before
implementation:

```text
What caller knowledge does this delete?
Which interface shrinks?
Where does locality improve?
What authority does this avoid duplicating?
If we delete the new module, where does the complexity reappear?
```

An issue fails the gate when it only renames code, adds a pass-through module,
or spreads one concept across more call sites.

## Candidate 1: DurableRunCommand

### Problem

The `rlm` and `workflow` CLI paths both execute durable runs through the same
Engine, but their orchestration is duplicated in `crates/harp/src/main.rs`.
Both paths open state, build an `ActivityRuntime`, construct or load a graph,
execute or resume, and serialize run status. ADR 0002 says Dynamic Workflow
compiles to `TaskGraph` and the Engine remains the only durable executor, but
the command implementation still makes maintainers reason through two similar
execution paths.

### Desired shape

Introduce an internal durable command module below Clap dispatch and above the
Engine. It should own shared command behavior:

- execute a validated `TaskGraph`;
- resume one run or all incomplete runs;
- produce status values;
- produce checkpoint values;
- construct runtime adapters through one command-aware path; and
- serialize shared run summaries.

The `rlm` command remains the adapter from a task-graph JSON file to the shared
module. The `workflow` command remains the adapter from authored
`DynamicWorkflow` JSON to a compiled `TaskGraph`.

### Simplicity proof

- Deletes caller knowledge: command handlers no longer know shared resume,
  status, runtime-construction, and summary-shape details.
- Shrinks interface: one durable-run interface replaces two command-local
  orchestration blocks.
- Improves locality: status and resume changes live in one module.
- Avoids duplicate authority: the Engine remains the executor; the new module
  is command orchestration only.
- Deletion test: deleting the module puts duplicate run/resume/status logic
  back into both `rlm` and `workflow`.

### Acceptance

- Shared status fields are produced by one implementation.
- `rlm` and `workflow` preserve existing command output.
- Workflow-only fields such as `workflow_name` and `compiled_graph_sha256`
  remain workflow-owned.
- Existing `rlm` process restart tests still pass.
- New tests cover shared resume selection behavior for both command families.

## Candidate 2: AtlasCorpusView

### Problem

Atlas strictly parses `atlas/src/content/generated/corpus.json`, but after
parsing, React modules repeatedly join documents, systems, Weng sections,
diagnostic cases, lessons, and reader routes. This spreads generated-corpus
relationship knowledge across view modules.

### Desired shape

Deepen `atlas/src/content/canonical.ts` into an Atlas corpus view module. It
should build lookup maps once from the validated corpus and expose small
queries such as:

- `document(id)`;
- `routeDocument(routeId)`;
- `publishedSystem(id)`;
- `systemArticle(id)`;
- `wengSectionProjection(id)`;
- `lessonProjection(id)`; and
- `diagnosticCase(id)`.

React modules should render projections rather than reimplement corpus joins.
The generated corpus remains the input. Strict TypeScript parsing remains the
boundary parser.

### Simplicity proof

- Deletes caller knowledge: React views stop knowing how corpus collections
  join.
- Shrinks interface: views ask for projections instead of searching global
  arrays.
- Improves locality: corpus relationship changes live in the corpus view
  module.
- Avoids duplicate authority: generated `corpus.json` remains the data
  authority; this is a read projection only.
- Deletion test: deleting the module pushes route/document/system/lesson joins
  back into multiple views.

### Acceptance

- Existing Atlas behavior and routes remain unchanged.
- Tests cover the corpus view interface directly.
- View tests focus on rendering, not lookup mechanics.
- No new runtime fetches or server assumptions are introduced.
- Offline export still works from `file://`.

## Candidate 3: SideEffectPolicy

### Problem

Writable workflow execution needs replay safety. `WorkspaceMode::GitWorktree`
currently grants write authority, but without mutation admission, worktree
identity, idempotency keys, mutation receipts, and recovery disposition, Harp
cannot prove that restart does not duplicate a completed mutation.

### Desired shape

Implement the already-approved `SideEffectPolicy` design for every `TaskGraph`
node that requests `WorkspaceMode::GitWorktree`.

The first implementation should:

- require a clean Git worktree at admission;
- persist side-effect state in a dedicated table;
- keep idempotency keys Engine-only;
- record before and after worktree identity;
- publish a bounded mutation receipt; and
- force reconciliation before semantic replay after crash ambiguity.

### Simplicity proof

- Deletes caller knowledge: Engine callers stop reasoning about write replay
  with provider result text.
- Shrinks interface: mutation admission and reconciliation sit behind one
  policy module.
- Improves locality: worktree identity, idempotency, and receipt rules live in
  one place.
- Avoids duplicate authority: runtime adapters still execute only inside the
  workspace authority they receive.
- Deletion test: deleting the module forces replay safety back into scheduler,
  state transitions, and provider-result interpretation.

### Acceptance

- `GitWorktree` nodes cannot run without side-effect admission.
- Crash after provider dispatch reconciles before new provider work.
- Published mutation receipts are reused on resume.
- Dirty admission is rejected.
- Drifted worktree identity fails closed.
- Too-large mutation evidence yields a bounded terminal disposition.

## Issue Split

Each implementation issue must be independently reviewable and landable:

1. DurableRunCommand.
2. AtlasCorpusView.
3. SideEffectPolicy state and receipt contract.
4. SideEffectPolicy Engine enforcement.
5. SideEffectPolicy recovery fixtures.

Issues 3-5 split SideEffectPolicy because it touches state, Engine execution,
and recovery. They should still share one design contract.

## Verification

Every issue must run the narrowest meaningful tests while iterating and
`mise run verify` before landing. Any tracked content or code change must
refresh `docs/import-receipt.md` as the final tracked change.

## Open Risks

- DurableRunCommand can become a shallow pass-through if it only moves code
  without deleting duplicated command knowledge.
- AtlasCorpusView can become a second corpus authority if it starts validating
  facts independently of the generated corpus parser.
- SideEffectPolicy can become too large if the first slice admits dirty
  baselines, copy-on-write semantics, remote writes, or provider-facing
  idempotency.

The simplicity gate exists to reject those outcomes before implementation.
