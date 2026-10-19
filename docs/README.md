# Harp documentation

[Repository home](../README.md)

## Use Harp

| Guide | What it covers |
| --- | --- |
| [Getting started](getting-started.md) | Clone, read Markdown, open the Obsidian vault, build the CLI, and search |
| [Durable coding work](durable-runtime.md) | Prepare, admit, run, recover, verify and land isolated Codex or Trae work |
| [Agent workflows](agent-workflows.md) | Retained context tools and retirement guidance for old execution commands |
| [Native release candidates](releasing.md) | Build a verified host-native archive and install it locally |

Start with the [knowledge home](../knowledge/harp_knowledge_home.md) or
[brief article survey](../knowledge/article_survey.md). Those notes belong to the
maintained corpus under `knowledge/`; this directory explains how to use and
contribute to the software.

## Contribute and understand the design

- [Contributor guide](contributing.md): setup, focused tests, content and
  evidence changes, and the full landing gate.
- [Product contract](product-contract.md): ownership, schemas, and implemented
  behavior.
- [Evidence and claim-route decision](adr/0001-source-backed-evidence-and-clickable-claim-routes.md):
  how claims connect to sources.
- [Durable task-graph decision](adr/0002-dynamic-workflow-compiles-to-durable-task-graph.md):
  the historical design that compiled dynamic workflows into task graphs.
- [Technical writing guide](writing-style/STYLE_GUIDE.md): source-backed
  explanations and precise evidence claims.
- [Release-readiness planner](superpowers/plans/2026-09-04-github-release-readiness.md):
  milestone status and deferred architecture improvements.

Files under `superpowers/plans/` record implementation plans and historical
decisions. They are not the starting point for user setup.
