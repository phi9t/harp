# MAC operator center

Use `tools/agents/mac_dashboard.py --config PATH` for the TUI. Use
`tools/agents/mac_tmux.py --config PATH --watch` to maintain agent views.
Both tools use Python's standard library; the session manager also requires tmux.

The seven tabs show Overview, Tasks, Agents, Work, Evidence, VCS and Infra.
Press 1–7 or Tab to change tabs. Use arrows or j/k to scroll. Press r to refresh.
Press ? for help. In Agents, use n/p to select a native view, then Enter to open it.
Press q to close only the dashboard. Use tmux Ctrl-b d to detach the session.

The dashboard reads the coordinator's CURRENT projection, Luna's observations,
the manager's manifest and an optional lead-owned roster. Each source retains
its timestamp and digest. Missing information appears as unavailable. An unknown
worker binding stays unknown. Historical clients remain available for inspection.
The display does not establish task acceptance or prove that a controller runs.

The manager adds clients for explicit endpoint/thread bindings. It sends no
agent input. Repeated reconciliation preserves clients and the selected window.
Codex clients remain interactive when a human selects them. Source failures and
exited clients appear in the manifest; the manager does not restart those clients.

Config version 1 follows the shared interface in EFFORT.md. Set `dashboard_argv`
to the literal dashboard command and use one `state_dir` per maintained session.
Run `--once` before `--watch` to inspect adoption and health. Keep the config
outside tracked source when it contains host-specific paths.

For plain text, run `mac_dashboard.py --config PATH --render`. For focused checks,
run `python3 tools/agents/test_mac_dashboard.py` and
`python3 tools/agents/test_mac_tmux.py`. Their terminal tests use disposable
clients and tmux sessions.

This operator tooling remains separate from the product controller. Controller
assignments, writer admission and corrections remain with corenius-impl.

For controller, Kata and workspace detail, use
`tools/agents/mac_state_view.py --config PATH --view controller`.
The other views are `kata` and `workspaces`; keys 1–3 switch views.
`--render` prints the bounded public projection. The tool reads explicit
`controller_snapshot_path` and `kata_snapshot_path` files. It does not query a
database, load auth or execute controller commands. Inspect the source time
before treating a displayed state as current.

To expose additional native clients, set `runtime_bindings_path` to a public
JSON file with `observed_at` and `bindings`. Each entry contains `role`,
`thread`, `cwd`, `endpoint` and boolean `current`. Never put grants or auth
values in that file. Missing or stale input prevents new client admission.
A view manager refresh is not evidence that an execution service is healthy.

The installed exact-project native Kata window and fixed-deployment public
query feed are operator-owned glue. The feed retains the last successful
snapshot on API failure and records current availability separately in
`public-feed-health.json` and the `state-feed` pane. It never changes tasks.
Product UI transport and model process recovery belong to the core coordinator.

Use the exact paths in `CURRENT.public_exports` as the three snapshot/roster
config paths. The coordinator owns those exports; viewers never overwrite them.
Controller envelopes preserve unavailable status and separate last-success
history. Kata items preserve native status and synchronized execution summaries.
The roster permits optional public `status` and bounded `reason` fields.
`unavailable` or `pending` admits no clients; `observed`, `available` or `ok`
permits validated bindings. The manager does not print reason text. Legacy
flat projections and the original roster shape remain supported.

Keep all `trial-*` service windows intact. Update only ownership-tagged UI panes.
An authenticated native worker endpoint needs an operator-owned launcher with
its private UI grant. Publish or adopt its public binding after readiness;
never put credentials in the public roster or viewer arguments.
