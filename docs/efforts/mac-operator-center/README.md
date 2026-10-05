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
