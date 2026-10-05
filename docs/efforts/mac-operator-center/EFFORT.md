# MAC operator center

Status: active; owner-requested improvement of the existing operator views.
Source read: c9d78bf; live prototype corenius-team and its private team_view.py.

## Goal

Give the lead one readable TUI and a maintained tmux session for the entire
MAC effort. Preserve existing clients and keep implementation coordination
independent of the future product controller's runtime assignments.

## Scope

Improve the working happy path first. Ticket01 owns the dashboard; ticket02
owns tmux view maintenance. They run in separate worktrees with disjoint files.
Core controller source, worker corrections and acceptance remain with corenius-impl.
This is a bounded extension of the owner-approved operator-view design. Use
the repository skill crosswalk without creating another architecture RFC.

Dashboard sections: overview/goal/next action; tasks and source versions; agents
and their native views; progress/questions; evidence and pending gates; Git
workspace/landing status; infrastructure and Luna/context health. Display only
available structured source fields. Unknown, stale and missing differ from
healthy, idle and empty. Do not read transcripts, credentials or private reasoning.
Keep navigation, scrolling and source freshness visible on a small terminal.

Tmux maintenance follows authorized concrete bindings. Add missing views to
the existing corenius-team session, distinguish current from historical views,
and preserve old clients. Do not send prompts, model calls, worker input,
interruptions, settings changes, or destructive cleanup. Deduplicate by endpoint
and thread, not a bare thread ID. Only manage explicitly owned windows.
One operator reconciliation process may refresh view metadata; it is not a
task scheduler or a replacement for the existing review timer.

## Shared interface

New standalone Python tools live at tools/agents/mac_dashboard.py and
tools/agents/mac_tmux.py. Both accept --config PATH. Config version1 contains:
session, owner, state_dir, current_path, luna_path, codex_executable, endpoint,
coordinator:{role,thread,cwd,endpoint}, extra_bindings:[{role,thread,cwd,endpoint}],
refresh_seconds, and dashboard_argv (literal argument vector).
No secrets belong in this config. Endpoint defaults may be explicit local Unix
sockets; desktop/native thread stores must not be silently interchanged.

Optional agent_roster_path gives the dashboard a parent-owned operator roster:
{observed_at, agents:[{name,scope,status,model,thread?,endpoint?,worktree?,ticket?}]}.
This exposes lead-owned tooling authors and Luna without claiming product task
assignments. Missing native bindings remain unavailable; no view is guessed.
Optional adopt_manifest_path lets the tmux manager validate the existing
prototype's owner, concrete endpoint/thread and window/process identity before
adding its metadata. Preserve the old overview; the lead deploys its replacement.

The current code-author binding comes from CURRENT.thread/worktree/writer_state.
Extra bindings are explicitly operator-admitted views, not new task assignments.
Lifecycle status and product versus implementation scope remain source-labelled.
No native client is launched from a guessed UUID or unrelated thread store.

Ticket02 writes state_dir/manifest.json atomically: session, observed_at, bindings
with role/thread/endpoint/cwd/window/current, and health/errors. Dashboard reads
it without modifying it. Historical windows remain listed as historical.
Ticket02 supports --once and --watch. Dashboard --render emits a plain-text view
for diagnosis and tests. Invalid source data is visible and does not admit views.

## Completion evidence

Use focused tests for source freshness/malformed data, truthful missing fields,
endpoint/thread identity, duplicate reconciliation, replacement preservation and
no worker inputs. Verify actual terminal rendering and live native client views.
Review Standards and Spec, install the tested tools into the existing operator
session, and record exact source and launch commands. Broader product release
qualification is separate; these tools cannot declare the baseline accepted.
