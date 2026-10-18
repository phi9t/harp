#!/usr/bin/env python3
"""Live dashboard over the state files: a curses table of every run and agent pane, plus aggregates.

  dashboard.py --root RUN_ROOT [--once] [--socket S --session NAME]

Keys: j/k or arrows select, Enter jumps to that run's tmux window, l toggles the log tail of the selected agent,
x kills the selected agent's pane, r toggles auto-refresh pause, q quits. `--once` prints the table as plain text and exits.
The render functions are pure (state snapshot in, text out) so they are testable without a terminal. Standard library only.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import state  # noqa: E402

SPIN = "|/-\\"


def fmt_secs(value) -> str:
    if value is None:
        return "-"
    value = int(value)
    return f"{value // 3600}:{value % 3600 // 60:02d}:{value % 60:02d}" if value >= 3600 else f"{value // 60}:{value % 60:02d}"


def clip(text, n: int) -> str:
    text = " ".join(str(text or "").split())
    return text if len(text) <= n else text[: max(0, n - 1)] + "…"


def unit_elapsed(u: dict, now: float) -> float | None:
    if u.get("phase") in state.FINAL:
        return u.get("elapsed")
    if u.get("started"):
        return now - u["started"]
    return None


def run_elapsed(r: dict, now: float) -> float | None:
    if r.get("phase") in state.FINAL and r.get("wall") is not None:
        return r["wall"]
    return now - r["created"] if r.get("created") else None


def rows(snap: dict) -> list[dict]:
    """Flattened display rows: each run followed by its agent units."""
    by_run: dict[str, list] = {}
    for u in snap["units"]:
        by_run.setdefault(u.get("run_id", ""), []).append(u)
    out = []
    for r in snap["runs"]:
        out.append({"kind": "run", "obj": r})
        for u in by_run.get(r.get("run_id"), []):
            out.append({"kind": "unit", "obj": u})
    for rid in by_run:  # units whose run state is missing (a stray or crashed scheduler)
        if not any(r.get("run_id") == rid for r in snap["runs"]):
            out.extend({"kind": "unit", "obj": u} for u in by_run[rid])
    return out


def summary(snap: dict, now: float, slots: int | None = None) -> list[str]:
    runs, units = snap["runs"], snap["units"]
    count = lambda ph: sum(1 for r in runs if r.get("phase") == ph)
    done = [r for r in runs if r.get("phase") in state.FINAL]
    passed = sum(1 for r in done if r.get("passed") is True)
    failed = len(done) - passed
    active = [u for u in units if u.get("phase") == "agent"]
    queued = [u for u in units if u.get("phase") == "queued"]
    tin = sum(int(u.get("tokens_in") or 0) for u in units)
    tout = sum(int(u.get("tokens_out") or 0) for u in units)
    start = min((r["created"] for r in runs if r.get("created")), default=None)
    line1 = (f"runs {len(runs)}  active {sum(1 for r in runs if r.get('phase') not in state.FINAL and r.get('phase') != 'queued')}  "
             f"queued {count('queued')}  done {len(done)}  pass {passed}  fail {failed}"
             + (f"  pass-rate {passed / len(done):.0%}" if done else ""))
    line2 = (f"agent panes: {len(active)} running" + (f" / {slots} slots" if slots else "") + f", {len(queued)} waiting   "
             f"tokens in {tin:,} out {tout:,}   elapsed {fmt_secs(now - start) if start else '-'}")
    return [line1, line2]


def render(snap: dict, width: int = 140, selected: int | None = None, now: float | None = None, slots: int | None = None,
           log_tail: list[str] | None = None, spinner: int = 0) -> list[tuple[str, str]]:
    """[(style, text)] lines. Styles: head, run, unit, pass, fail, warn, dim, sel."""
    now = now if now is not None else time.time()
    lines: list[tuple[str, str]] = [("head", f" corenius-bench dashboard {SPIN[spinner % 4]}")]
    for text in summary(snap, now, slots):
        lines.append(("dim", " " + text))
    header = f" {'RUN / AGENT':44s} {'PHASE':9s} {'TIME':>7s} {'TOK-OUT':>8s} {'CMD':>4s} {'C/R':>5s}  LAST"
    lines.append(("head", header[:width].ljust(min(width, 200))))
    display = rows(snap)
    if not display:
        lines.append(("dim", " (no runs yet)"))
    last_w = max(10, width - 44 - 9 - 7 - 8 - 4 - 5 - 12)
    for i, row in enumerate(display):
        o = row["obj"]
        if row["kind"] == "run":
            label = f"{o.get('task', '?')} {o.get('condition', '')}"
            phase = o.get("phase", "?")
            verdict = {True: "PASS", False: "FAIL"}.get(o.get("passed"))
            shown = verdict or phase
            cr = f"{o.get('conflicts', 0)}/{o.get('redos', 0)}" if o.get("condition", "solo") != "solo" else "-"
            tokens = o.get("tokens_out")
            style = "pass" if verdict == "PASS" else "fail" if verdict == "FAIL" or phase == "crashed" else "run"
            text = (f" {clip(label, 44):44s} {shown:9s} {fmt_secs(run_elapsed(o, now)):>7s} "
                    f"{(str(tokens) if tokens is not None else '-'):>8s} {'':>4s} {cr:>5s}  {clip(o.get('error') or '', last_w)}")
        else:
            label = "  └ " + (o.get("ticket") or "agent")
            phase = o.get("phase", "?")
            style = "fail" if phase == "crashed" or (phase == "done" and o.get("rc") not in (0, None)) else "unit"
            text = (f" {clip(label, 44):44s} {phase:9s} {fmt_secs(unit_elapsed(o, now)):>7s} {str(o.get('tokens_out', 0)):>8s} "
                    f"{str(o.get('commands', 0)):>4s} {'':>5s}  {clip(o.get('last'), last_w)}")
        if selected is not None and i == selected:
            style = "sel"
        lines.append((style, text[:width]))
    if log_tail:
        lines.append(("head", " log tail (selected agent)"))
        lines.extend(("dim", "  " + clip(t, width - 3)) for t in log_tail)
    return lines


def plain(snap: dict, width: int = 140, now: float | None = None, slots: int | None = None) -> str:
    return "\n".join(text for _, text in render(snap, width, None, now, slots))


# -- tmux glue ----------------------------------------------------------------------------------------------------------------


def tmux(socket: str, *args: str) -> str:
    out = subprocess.run(["tmux", "-L", socket, *args], capture_output=True, text=True)
    return out.stdout.strip()


def jump(socket: str, pane_id: str | None) -> None:
    if not pane_id:
        return
    window_id = tmux(socket, "display-message", "-p", "-t", pane_id, "#{window_id}")
    if window_id:
        tmux(socket, "select-window", "-t", window_id)
        tmux(socket, "select-pane", "-t", pane_id)


def read_tail(root: Path, unit: str, n: int = 8) -> list[str]:
    try:
        return (Path(root) / "units" / unit / "pane.log").read_text().splitlines()[-n:]
    except OSError:
        return []


def run_curses(root: Path, socket: str | None, session: str | None) -> int:
    import curses

    meta = state.read(Path(root) / "session.json") or {}
    slots = meta.get("slots")

    def loop(stdscr) -> int:
        curses.curs_set(0)
        curses.use_default_colors()
        for i, color in enumerate((curses.COLOR_GREEN, curses.COLOR_RED, curses.COLOR_YELLOW, curses.COLOR_CYAN, curses.COLOR_WHITE), 1):
            curses.init_pair(i, color, -1)
        styles = {"pass": curses.color_pair(1), "fail": curses.color_pair(2), "warn": curses.color_pair(3),
                  "run": curses.color_pair(4) | curses.A_BOLD, "unit": curses.color_pair(5), "dim": curses.A_DIM,
                  "head": curses.A_REVERSE, "sel": curses.A_REVERSE | curses.A_BOLD}
        stdscr.timeout(500)
        selected, show_log, paused, spin = 0, False, False, 0
        snap = state.collect(root)
        while True:
            if not paused:
                snap = state.collect(root)
            display = rows(snap)
            selected = max(0, min(selected, len(display) - 1)) if display else 0
            tail = None
            if show_log and display and display[selected]["kind"] == "unit":
                tail = read_tail(root, display[selected]["obj"].get("unit", ""), 8)
            height, width = stdscr.getmaxyx()
            stdscr.erase()
            for y, (style, text) in enumerate(render(snap, width - 1, selected if display else None, slots=slots, log_tail=tail,
                                                     spinner=spin)[: height - 1]):
                try:
                    stdscr.addstr(y, 0, text.ljust(width - 1)[: width - 1], styles.get(style, 0))
                except curses.error:
                    pass
            try:
                stdscr.addstr(height - 1, 0, " j/k select  Enter jump  l log  x kill pane  r pause  q quit"[: width - 1], curses.A_DIM)
            except curses.error:
                pass
            stdscr.refresh()
            spin += 1
            key = stdscr.getch()
            if key in (ord("q"), 27):
                return 0
            if key in (ord("j"), curses.KEY_DOWN):
                selected = min(selected + 1, max(0, len(display) - 1))
            elif key in (ord("k"), curses.KEY_UP):
                selected = max(selected - 1, 0)
            elif key in (10, 13, curses.KEY_ENTER) and display and socket:
                o = display[selected]["obj"]
                pane = o.get("pane_id") if display[selected]["kind"] == "unit" else next(
                    (u.get("pane_id") for u in snap["units"] if u.get("run_id") == o.get("run_id") and u.get("pane_id")), None)
                jump(socket, pane)
            elif key == ord("l"):
                show_log = not show_log
            elif key == ord("r"):
                paused = not paused
            elif key == ord("x") and display and socket and display[selected]["kind"] == "unit":
                pane = display[selected]["obj"].get("pane_id")
                if pane:
                    tmux(socket, "kill-pane", "-t", pane)

    return curses.wrapper(loop)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--root", required=True)
    p.add_argument("--once", action="store_true")
    p.add_argument("--socket")
    p.add_argument("--session")
    p.add_argument("--width", type=int, default=150)
    args = p.parse_args(argv)
    if args.once or not sys.stdout.isatty():
        print(plain(state.collect(Path(args.root)), args.width, slots=(state.read(Path(args.root) / "session.json") or {}).get("slots")))
        return 0
    return run_curses(Path(args.root), args.socket, args.session)


if __name__ == "__main__":
    sys.exit(main())
