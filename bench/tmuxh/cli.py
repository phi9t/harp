#!/usr/bin/env python3
"""bench-tmux: run benchmark tasks with one tmux window per run and one pane per agent, with a live dashboard.

  bench-tmux run TASK... [--all] --agent reference|noop|cmd:SHELL|traecli[:MODEL] [--condition solo|fleet:N] [--repeat K]
                         [--slots K] [--attach] [--down] [--authorize-model-spend]
  bench-tmux dash [--root RUN_ROOT] [--once]     dashboard for a run root (default: the latest)
  bench-tmux ls | attach | down                  inspect, attach to, or kill the dedicated tmux server

Everything runs on a dedicated tmux server (socket `corenius-bench`); your own tmux sessions are never touched.
Run through ./bench-tmux (it uses the uv project that installs libtmux).
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import threading
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
for extra in (HERE, ROOT / "bench" / "harness", ROOT / "tools" / "insula"):
    if str(extra) not in sys.path:
        sys.path.insert(0, str(extra))

try:
    import libtmux  # noqa: F401
except ImportError:
    sys.exit("libtmux is not installed. Run through bench/tmuxh/bench-tmux (uses `uv run --project bench/tmuxh`), or: "
             "cd bench/tmuxh && uv sync && .venv/bin/python cli.py ...")

import dashboard  # noqa: E402
import results  # noqa: E402
import scheduler  # noqa: E402
import state  # noqa: E402
import tasklib  # noqa: E402

RESULTS = ROOT / "bench" / "results"


def latest_root() -> Path | None:
    roots = sorted(RESULTS.glob("tmux-*"))
    return roots[-1] if roots else None


def cmd_run(args) -> int:
    condition = args.condition
    if not (condition == "solo" or re.fullmatch(r"fleet:\d+", condition)):
        raise SystemExit("condition must be solo or fleet:N")
    tasks = tasklib.load_tasks(only=None if args.all else (args.task or None))
    if not args.all and not args.task:
        raise SystemExit("name at least one task, or pass --all")
    skipped = [t.id for t in tasks if condition != "solo" and "multi" not in t.tags]
    tasks = [t for t in tasks if condition == "solo" or "multi" in t.tags]
    if skipped:
        print(f"skipping single-ticket tasks for {condition}: {', '.join(skipped)}", file=sys.stderr)
    if not tasks:
        raise SystemExit("nothing to run")
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    root = Path(args.root) if args.root else RESULTS / f"tmux-{stamp}"
    out = root / "results"
    work = [(t, condition, args.agent, i) for i in range(args.repeat) for t in tasks]
    h = scheduler.Harness(root, socket=args.socket, slots=args.slots, dashboard=not args.no_dashboard).start()
    print(f"run root : {root}\nsession  : {h.session_name} on tmux socket '{args.socket}'\n"
          f"watch    : tmux -L {args.socket} attach -t {h.session_name}   (window 0 is the dashboard)\n"
          f"plain    : python3 bench/tmuxh/dashboard.py --root {root} --once\nwork     : {len(work)} run(s), {args.slots} agent slot(s)",
          flush=True)
    records: list = []
    thread = threading.Thread(target=lambda: records.extend(
        h.run_batch(work, out, authorized=args.authorize_model_spend, timeout=args.timeout)))
    thread.start()
    if args.attach:
        env = {k: v for k, v in os.environ.items() if k != "TMUX"}
        subprocess.call(h.attach_argv(), env=env)
    thread.join()
    rows = results.summarize(records)
    print(results.table(rows) if rows else "no completed runs")
    failed = sum(1 for r in records if not r["outcome"]["passed"])
    print(f"{len(records)}/{len(work)} runs completed, {failed} failed the verifier; results: {out}")
    if args.down:
        h.stop()
    else:
        print(f"session left running for inspection; stop it with: bench/tmuxh/bench-tmux down --socket {args.socket}")
    return 0 if len(records) == len(work) else 1


def cmd_dash(args) -> int:
    root = Path(args.root) if args.root else latest_root()
    if root is None:
        raise SystemExit("no run root found; pass --root")
    return dashboard.main(["--root", str(root)] + (["--once"] if args.once else []))


def _server(socket: str):
    return libtmux.Server(socket_name=socket)


def cmd_ls(args) -> int:
    server = _server(args.socket)
    if not server.is_alive():
        print(f"no tmux server on socket '{args.socket}'")
        return 0
    for s in server.sessions:
        print(f"session {s.session_name}")
        for w in s.windows:
            print(f"  window {w.window_index} {w.window_name}  ({len(w.panes)} pane(s))")
    return 0


def cmd_attach(args) -> int:
    env = {k: v for k, v in os.environ.items() if k != "TMUX"}
    os.execvpe("tmux", ["tmux", "-L", args.socket, "attach"], env)


def cmd_down(args) -> int:
    server = _server(args.socket)
    if server.is_alive():
        server.kill()
        print(f"killed the tmux server on socket '{args.socket}'")
    else:
        print(f"no tmux server on socket '{args.socket}'")
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("task", nargs="*")
    r.add_argument("--all", action="store_true")
    r.add_argument("--agent", default="reference")
    r.add_argument("--condition", default="solo")
    r.add_argument("--repeat", type=int, default=1)
    r.add_argument("--slots", type=int, default=4)
    r.add_argument("--root")
    r.add_argument("--timeout", type=float)
    r.add_argument("--attach", action="store_true", help="attach this terminal to the tmux session while the runs execute")
    r.add_argument("--down", action="store_true", help="kill the tmux server when the runs finish")
    r.add_argument("--no-dashboard", action="store_true")
    r.add_argument("--authorize-model-spend", action="store_true")
    d = sub.add_parser("dash")
    d.add_argument("--root")
    d.add_argument("--once", action="store_true")
    for name in ("ls", "attach", "down"):
        sub.add_parser(name)
    for sp in sub.choices.values():
        sp.add_argument("--socket", default=scheduler.SOCKET)
    args = p.parse_args(argv)
    return {"run": cmd_run, "dash": cmd_dash, "ls": cmd_ls, "attach": cmd_attach, "down": cmd_down}[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main())
