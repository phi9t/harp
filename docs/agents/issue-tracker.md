# Issue tracker: local Markdown

Harp uses the local-Markdown convention supported by Matt Pocock's engineering skills. The tracker is Git-managed repository state, even though its directory is named `.scratch`. No tracker CLI, daemon, account, or remote synchronization is required.

## Files and operations

- Feature intent/spec: `.scratch/<feature-slug>/spec.md`.
- Implementation tickets: `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, starting at `01`, one ticket per file. Reference tickets by their full repository-relative path; numbers alone are unique only within a feature.
- Read/search existing tickets before creating one. When a skill says publish, create or update these files and include them in the relevant local commit. Fetch means read the referenced file.
- Put `Status:` near the top, using [the default triage roles](triage-labels.md). Execution uses `in-progress`, `blocked`, and `done`. Record owner and branch/worktree when claiming work. Keep a blocked ticket's reason and next resolving action explicit.
- Include `## What to build`, `## Acceptance criteria`, and `## Blocked by`. Use file links for parents and blockers. Append discussion under `## Comments` and verification commands, results, limitations, and commit references under `## Verification`.
- Mark done only when the requested scope and applicable checks pass. Preserve explicit user approval requirements. Parallel writers own separate tickets; the coordinator reconciles names and dependencies before integration.

A small task needs only a ticket. Add a spec when there are design decisions to capture. Existing workstream designs and plans remain useful references: link them instead of duplicating their content. These records own task intent and execution state, not Harp's technical-prose corpus.

## Optional wayfinding

For exploration, use `.scratch/<feature>/map.md` for notes, decisions, and unresolved questions. Child issue files may use `Type: research`, `prototype`, `grilling`, or `task`, with statuses `open`, `claimed`, and `resolved`. Select the lowest-numbered open, unblocked issue; record a claim before work. Resolve by appending an answer and linking its result from the map. A resolved research question does not imply an implementation ticket is done.

## Transition from Kata

The repository binding is removed. Leave existing external Kata data and captured evidence intact; they are historical records. When resuming earlier work, create or update its local ticket with the former identifier, known acceptance criteria, current state, and available evidence. Do not infer completion or rebuild the old service to recover context. Note missing historical information explicitly. Existing documents that direct agents to Kata are superseded by this contract.
