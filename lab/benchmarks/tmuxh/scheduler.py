"""libtmux orchestration: a window per run, a pane per agent, on a dedicated tmux server.

The scheduler is the lead: it runs the same run_solo / run_fleet_condition code as the direct runner, with PaneAgent standing in for
Agent. PaneAgent.run() writes a job, opens a pane that runs pane.py, waits for the pane's state file and returns the same dict Agent.run
returns, so landing, verification and result records are unchanged. tmux only hosts and shows the work.
"""

from __future__ import annotations

import json
import os
import shlex
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
for extra in (HERE, ROOT / "lab" / "benchmarks" / "harness", ROOT):
    if str(extra) not in sys.path:
        sys.path.insert(0, str(extra))

import libtmux  # noqa: E402
import fleet  # noqa: E402
import results  # noqa: E402
import run  # noqa: E402
import state  # noqa: E402
import tasklib  # noqa: E402

SOCKET = "corenius-bench"
MARK = {"running": "▶", "pass": "✓", "fail": "✗", "queued": "…"}
TMUX_CONF = """\
set -g history-limit 50000
set -g remain-on-exit on
set -g mouse on
set -g status-interval 2
set -g default-terminal "screen-256color"
set -g automatic-rename off
set -g allow-rename off
set -g pane-border-status top
set -g pane-border-format " #{pane_title} "
set -g status-left "#[bold] corenius-bench "
set -g status-left-length 24
set -g window-status-current-style "reverse"
"""


@dataclass
class RunCtx:
    run_id: str
    window_name: str
    task: str
    condition: str
    agent: str
    fleet: bool


@dataclass
class _Window:
    window: object
    lead_pane: object | None = None
    panes: dict = field(default_factory=dict)


class Harness:
    def __init__(self, root: Path, socket: str = SOCKET, slots: int = 4, python: str | None = None,
                 session_name: str | None = None, dashboard: bool = True):
        self.root = Path(root).resolve()
        self.socket = socket
        self.slots = threading.BoundedSemaphore(max(1, slots))
        self.n_slots = max(1, slots)
        self.python = python or sys.executable
        self.session_name = session_name or "bench-" + datetime.now(timezone.utc).strftime("%H%M%S")
        self.with_dashboard = dashboard
        self.lock = threading.RLock()
        self.windows: dict[str, _Window] = {}
        self.server = self.session = None

    # -- tmux lifecycle ---------------------------------------------------------------------------------------------------------

    def start(self) -> "Harness":
        self.root.mkdir(parents=True, exist_ok=True)
        conf = self.root / "tmux.conf"
        conf.write_text(TMUX_CONF)
        self.server = libtmux.Server(socket_name=self.socket, config_file=str(conf))
        dash = (f"{shlex.quote(self.python)} {shlex.quote(str(HERE / 'dashboard.py'))} --root {shlex.quote(str(self.root))} "
                f"--socket {shlex.quote(self.socket)} --session {shlex.quote(self.session_name)}") if self.with_dashboard else None
        self.session = self.server.new_session(session_name=self.session_name, window_name="dashboard", attach=False,
                                               window_command=dash, x=200, y=50)
        (self.root / "session.json").write_text(json.dumps({"socket": self.socket, "session": self.session_name,
                                                            "root": str(self.root), "started": state.now(), "slots": self.n_slots}) + "\n")
        return self

    def attach_argv(self) -> list[str]:
        return ["tmux", "-L", self.socket, "attach", "-t", self.session_name]

    def stop(self) -> None:
        """Kill the dedicated tmux server (never the user's default server)."""
        if self.server is not None and self.server.is_alive():
            self.server.kill()
        # tmux can leave the socket file behind; remove only our own, and only when no server answers on it.
        sock = Path(f"/run/insula-tmp/tmux-{os.getuid()}") / self.socket
        if self.server is not None and sock.exists() and not self.server.is_alive():
            try:
                sock.unlink()
            except OSError:
                pass

    # -- panes ------------------------------------------------------------------------------------------------------------------

    def _pane_cmd(self, job_path: Path) -> str:
        return f"{shlex.quote(self.python)} {shlex.quote(str(HERE / 'pane.py'))} --job {shlex.quote(str(job_path))}"

    def open_pane(self, ctx: RunCtx, unit: str, title: str, job_path: Path):
        with self.lock:
            win = self.windows.get(ctx.run_id)
            cmd = self._pane_cmd(job_path)
            if win is None:
                if ctx.fleet:
                    lead_log = state.unit_dir(self.root, ctx.run_id + "--lead") / "lead.log"
                    lead_log.parent.mkdir(parents=True, exist_ok=True)
                    lead_log.touch()
                    window = self.session.new_window(window_name=f"{MARK['running']} {ctx.window_name}", attach=False,
                                                     window_shell=f"tail -n +1 -F {shlex.quote(str(lead_log))}")
                    win = _Window(window, lead_pane=window.panes[0])
                    win.lead_pane.cmd("select-pane", "-T", f"lead · {ctx.task} · {ctx.condition}")
                    pane = window.split(shell=cmd, attach=False)
                else:
                    window = self.session.new_window(window_name=f"{MARK['running']} {ctx.window_name}", attach=False,
                                                     window_shell=cmd)
                    win = _Window(window)
                    pane = window.panes[0]
                self.windows[ctx.run_id] = win
            else:
                pane = win.window.split(shell=cmd, attach=False)
            pane.cmd("select-pane", "-T", title)
            win.panes[unit] = pane
            try:
                win.window.select_layout("tiled" if ctx.fleet else "even-horizontal")
            except Exception:
                pass
            return pane

    def mark_window(self, run_id: str, marker: str, window_name: str) -> None:
        with self.lock:
            win = self.windows.get(run_id)
            if win is not None:
                try:
                    win.window.rename_window(f"{marker} {window_name}")
                except Exception:
                    pass

    def pane_dead(self, pane) -> bool:
        try:
            pane.refresh()
            return str(getattr(pane, "pane_dead", "0")) == "1"
        except Exception:
            return True  # the pane is gone (killed from the dashboard or by the user)

    # -- runs -------------------------------------------------------------------------------------------------------------------

    def observer_for(self, ctx: RunCtx):
        path = state.run_state_path(self.root, ctx.run_id)

        def observe(event: str, **fields) -> None:
            phase = {"agents": "agents", "landing": "landing", "verify": "verify", "done": "done"}.get(event, event)
            update = {"phase": phase, **fields}
            state.update(path, **update)

        return observe

    def lead_log_for(self, ctx: RunCtx):
        if not ctx.fleet:
            return None
        path = state.unit_dir(self.root, ctx.run_id + "--lead") / "lead.log"
        path.parent.mkdir(parents=True, exist_ok=True)
        lock = threading.Lock()

        def log(message: str) -> None:
            with lock, path.open("a") as handle:
                handle.write(time.strftime("%H:%M:%S ") + message + "\n")

        return log

    def run_one(self, task: tasklib.Task, condition: str, agent_spec: str, index: int, out: Path, authorized: bool = False,
                timeout: float | None = None) -> dict:
        rid = run.run_id(task.id, condition, agent_spec, index)
        is_fleet = condition != "solo"
        ctx = RunCtx(rid, f"{task.id}-{condition.replace(':', '')}-{index}", task.id, condition, agent_spec, is_fleet)
        spath = state.run_state_path(self.root, rid)
        state.update(spath, run_id=rid, task=task.id, condition=condition, agent=agent_spec, phase="queued",
                     created=state.now(), window=ctx.window_name, conflicts=0, redos=0, passed=None)
        agent = PaneAgent(agent_spec, self, ctx, authorized, timeout)
        observer = self.observer_for(ctx)
        try:
            if is_fleet:
                n = int(condition.split(":")[1])
                rec = run.run_fleet_condition(task, agent, out, index, n, rid=rid, observer=observer, log=self.lead_log_for(ctx))
            else:
                rec = run.run_solo(task, agent, out, index, rid=rid, observer=observer)
            run.persist(rec, out)
            state.update(spath, phase="done", passed=rec["outcome"]["passed"], wall=rec["timing"]["wall_seconds"],
                         tokens_out=rec["cost"]["output_tokens"], conflicts=rec["coordination"]["conflicts"],
                         redos=rec["coordination"]["redos"])
            self.mark_window(rid, MARK["pass" if rec["outcome"]["passed"] else "fail"], ctx.window_name)
            return rec
        except BaseException as exc:
            state.update(spath, phase="crashed", error=f"{type(exc).__name__}: {exc}"[:200])
            self.mark_window(rid, MARK["fail"], ctx.window_name)
            raise

    def run_batch(self, work: list[tuple], out: Path, authorized: bool = False, timeout: float | None = None,
                  max_runs: int | None = None) -> list[dict]:
        """work: [(task, condition, agent_spec, index)]. Runs concurrently (panes are bounded by --slots)."""
        out = Path(out)
        records: list[dict | None] = [None] * len(work)

        def one(i: int) -> None:
            task, condition, spec, index = work[i]
            try:
                records[i] = self.run_one(task, condition, spec, index, out, authorized, timeout)
            except Exception as exc:
                print(f"run failed: {task.id} {condition}: {exc}", file=sys.stderr)

        with ThreadPoolExecutor(max_workers=max_runs or min(len(work), max(self.n_slots, 8))) as pool:
            list(pool.map(one, range(len(work))))
        return [r for r in records if r]


class PaneAgent(run.Agent):
    """Agent whose units run in tmux panes. One instance per run (it carries the run's window context)."""

    def __init__(self, spec: str, harness: Harness, ctx: RunCtx, authorized: bool = False, timeout: float | None = None):
        super().__init__(spec, authorized, timeout)
        self.harness, self.ctx = harness, ctx
        self._seen: dict[str, int] = {}
        self._lock = threading.Lock()

    def _unit_name(self, ticket_path: Path | None) -> tuple[str, str]:
        ticket = ticket_path.name if ticket_path is not None and ticket_path.parent.name == "tickets" else ""
        base = ticket or "agent"
        with self._lock:
            n = self._seen.get(base, 0)
            self._seen[base] = n + 1
        label = base if n == 0 else f"{base}-redo{n}"
        return f"{self.ctx.run_id}--{label}", label

    def run(self, instruction: str, ws: Path, record: dict, solution_or_ticket: Path | None, task: tasklib.Task) -> dict:
        h, ctx = self.harness, self.ctx
        unit, label = self._unit_name(solution_or_ticket)
        udir = state.unit_dir(h.root, unit)
        udir.mkdir(parents=True, exist_ok=True)
        plan = self.plan(instruction, task, solution_or_ticket, udir)
        meta = {"task": task.id, "condition": ctx.condition, "ticket": "" if label == "agent" else label,
                "agent": self.spec, "created": state.now()}
        job = {"unit": unit, "run_id": ctx.run_id, "unit_dir": str(udir), "workspace": str(ws), "meta": meta,
               "plan": plan.to_json()}
        job_path = udir / "job.json"
        job_path.write_text(json.dumps(job, indent=1) + "\n")
        spath = state.unit_state_path(h.root, unit)
        state.update(spath, unit=unit, run_id=ctx.run_id, phase="queued", last="waiting for a free slot", **meta)
        with h.slots:
            pane = h.open_pane(ctx, unit, f"{label} · {task.id}" if label != "agent" else f"{task.id} · {self.spec}", job_path)
            state.update(spath, pane_id=pane.pane_id, window=ctx.window_name, phase="agent", last="pane opened")
            return self._wait(pane, spath, udir, plan.timeout)

    def _wait(self, pane, spath: Path, udir: Path, timeout: float) -> dict:
        h = self.harness
        deadline = time.time() + timeout + 120  # the pane enforces the real timeout; this only catches a stuck pane
        while True:
            st = state.read(spath) or {}
            if st.get("phase") in state.FINAL:
                break
            if h.pane_dead(pane):
                time.sleep(1.5)  # let the pane's final flush land
                st = state.read(spath) or {}
                if st.get("phase") not in state.FINAL:
                    state.update(spath, phase="crashed", rc=255, last="pane died without a result")
                    (udir / "result.json").write_text(json.dumps({"rc": 255, "timed_out": False, "seconds": 0.0,
                                                                  "tail": "pane died without a result", "crashed": True}) + "\n")
                break
            if time.time() > deadline:
                state.update(spath, phase="crashed", rc=255, last="scheduler gave up waiting")
                try:
                    pane.kill()
                except Exception:
                    pass
                break
            time.sleep(0.4)
        result = state.read(udir / "result.json") or {"rc": 255, "timed_out": False, "seconds": 0.0, "tail": "no result"}
        return {"rc": result.get("rc", 255), "timed_out": bool(result.get("timed_out")), "seconds": result.get("seconds", 0.0),
                "tail": result.get("tail", "")}
