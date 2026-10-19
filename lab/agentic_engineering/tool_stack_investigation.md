---
id: agentic-engineering-tool-stack-investigation
title: Kenn tool stack investigation
type: investigation
status: draft
created: 2026-08-15
updated: 2026-08-15
tags: [agentic-engineering, kata, forge, ghosthub, agentsview, roborev, superpowers]
---

# Kenn tool stack investigation

Mode: `SOURCE-BACKED TOOL INVESTIGATION`.

This note inspects public primary sources for the tools named in Wes
McKinney's agentic engineering workflow post. It does not reproduce Kenn's
private deployment or operational metrics.

## Source set

| Tool | Source IDs | Claim ceiling |
|-|-|-|
| Kata | `AE-004`, `AE-005`, `AE-006` | Public docs and GitHub repository metadata for a local-first issue tracker |
| Forge | `AE-007`, `AE-008` | Public docs and GitHub repository metadata for a maintainer console |
| Ghosthub | `AE-009`, `AE-010` | Public site and GitHub repository metadata for a multiplexer-native terminal |
| AgentsView | `AE-011`, `AE-012` | Public docs and GitHub repository metadata for session and token intelligence |
| roborev | `AE-013`, `AE-014`, `AE-015` | Public docs and GitHub repository metadata for continuous code review |
| Superpowers | `AE-016` | GitHub repository metadata and public page for the skills methodology |

## Kata: durable intent ledger

**SOURCE CLAIM — [AE-004].** Kata describes itself as "the issue tracker built
for coding agents and the humans steering them" and as "a local-first task
ledger agents drive from the CLI and humans supervise in the terminal or
browser." It advertises stable short refs, JSON and agent output, idempotent
creates, a claim flow, semantic-aware search, predictable failure modes, a TUI,
a web UI, SQLite state under `KATA_HOME`, and a small repository binding file.

**SOURCE CLAIM — [AE-006].** The GitHub API metadata for `kenn-io/kata`
describes the repository as "Local-first issue tracking for AI-assisted
software work, with an agent-friendly CLI and human-facing TUI"; it reports Go
as the primary language, MIT license metadata, and homepage
`https://www.katatracker.com`.

**SOURCE CLAIM — [AE-005].** Kata federation docs describe hub/spoke roles,
config-driven enrollment, token boundaries, leases, write gates, push/poll
sync, quarantine, schema compatibility, and explicit leave/rejoin operations.
They say leases are hub-authoritative, optional coordination for non-comment
mutations, and not durable ownership.

**INFERENCE — [AE-004], [AE-005], [AE-006].** Kata is the clearest public
example of intent separated from prompt and Git history. It models the task
ledger as durable local state with explicit sync and federation semantics,
rather than as a Markdown todo list or chat transcript.

## Forge: human control plane

**SOURCE CLAIM — [AE-007].** Forge describes itself as a local console for
repository activity, pull requests, issues, reviews, and working sessions. It
syncs provider data into SQLite and serves the UI from one binary. It has views
for activity, pulls, issues, workspaces, repositories, and optional Kata/docs
modes. It says provider, Kata daemons, and local files remain the source of
truth.

**SOURCE CLAIM — [AE-008].** The GitHub API metadata for `kenn-io/forge`
describes the repository as a local-first maintainer console with built-in
agent workspaces for PRs, issues, and CI across GitHub, GitLab, Forgejo, and
Gitea; it reports Go as the primary language and license metadata as
`NOASSERTION`.

**INFERENCE — [AE-007], [AE-008].** Forge fits the human-control plane. Its
public docs emphasize fast navigation, review, and workspace creation rather
than handing merge authority to agents.

## Ghosthub: session multiplexing

**SOURCE CLAIM — [AE-009].** Ghosthub describes itself as a native terminal for
tmux, Herdr, and Zellij sessions across Mac, SSH hosts, and exe.dev. It
supports optional tmux-backed worktrees, keepalives, automatic reconnect, and
Git worktree management.

**SOURCE CLAIM — [AE-010].** The GitHub API metadata for `kenn-io/ghosthub`
describes it as a multiplexer-native terminal for local and remote sessions
with built-in Git worktree management. It reports Swift as the primary language
and AGPL-3.0 license metadata.

**INFERENCE — [AE-009], [AE-010].** Ghosthub addresses agent session
multiplexing: when one human supervises many local and remote workers, the UI
object is a session/worktree/attention state, not a single editor buffer.

## AgentsView: observability plane

**SOURCE CLAIM — [AE-011].** AgentsView describes itself as a local-first
desktop and web app for browsing, searching, and analyzing past AI coding
sessions. It reads session files from many coding agents, stores structured
data in SQLite, exposes full-text and semantic search, reports token usage and
cost, and includes activity/concurrency analytics.

**SOURCE CLAIM — [AE-012].** The GitHub API metadata for `kenn-io/agentsview`
describes it as local-first session search, analytics, insights, and token-use
statistics for coding agents. It reports Go as the primary language and MIT
license metadata.

**INFERENCE — [AE-011], [AE-012].** AgentsView is the observability plane for
agentic engineering. It turns execution traces into queryable operational
data, which is the substrate a future harness-learning loop would need.

## roborev: verification plane

**SOURCE CLAIM — [AE-013].** roborev describes itself as continuous code review
for coding agents. It reviews commits in the background, feeds findings back to
agents, supports post-commit reviews, agent hooks, branch refinement, a review
ledger, local orchestration, and PostgreSQL sync.

**SOURCE CLAIM — [AE-015].** roborev GitHub integration docs describe GitHub
App setup, personal auth setup, commit status checks, CI review workflows,
multi-review types, review matrices, synthesis, PR throttling, and safe CI
retries.

**SOURCE CLAIM — [AE-014].** The GitHub API metadata for `kenn-io/roborev`
describes it as a continuous background code review database for agents. It
reports Go as the primary language and MIT license metadata.

**INFERENCE — [AE-013], [AE-014], [AE-015].** roborev is an independent
verification budget, not merely a test runner. The public docs model review as
an accumulating ledger that remains open until findings are explicitly
addressed.

## Superpowers: reasoning and execution scaffolding

**SOURCE CLAIM — [AE-016].** The GitHub API metadata for `obra/superpowers`
describes the repository as "An agentic skills framework & software
development methodology that works." It reports Shell as the primary language,
MIT license metadata, and `main` as the default branch.

**INFERENCE — [AE-001], [AE-016].** In the Kenn workflow, Superpowers occupies
the reasoning/scaffolding plane: it helps turn design into precise specs and
plans, but the durable product knowledge is later distilled into living
architecture documents.

## Cross-tool architecture

**INFERENCE — [AE-001], [AE-004], [AE-007], [AE-009], [AE-011], [AE-013].** The
public tool stack reinforces the original packet's plane separation:

| Plane | Public tool evidence | Harp lesson |
|-|-|-|
| Intent | Kata task ledger, short refs, claims, federation | intent should be durable state, not private chat |
| Execution | Ghosthub sessions, worktrees, provider agents | execution scales through isolated sessions |
| Verification | roborev review ledger and hooks | verification has its own durable queue |
| Observability | AgentsView sessions, costs, activity | traces become operational data |
| Human control | Forge maintainer console and workspaces | humans need a control plane for parallel work |
| Reasoning scaffold | Superpowers methodology | specs/plans guide execution but are not final authority |

## Harp design implications

1. Harp should keep local-first state and explicit authority boundaries as a
   default design direction.
2. Future intent tracking should not be modeled as prompt text or private agent
   memory.
3. Side-effect receipts, review ledgers, and episode manifests are the Harp
   equivalents of "make work observable before trusting it."
4. Harness-learning work should start from trace, review, and correction data,
   then require human approval before changing durable instructions or tools.

## Claim ceiling

This investigation proves only what the public pages and GitHub API metadata
say at capture time. It does not inspect source code behavior, run the tools,
verify hosted services, reproduce sync/federation behavior, or validate Kenn's
private production workflow.
