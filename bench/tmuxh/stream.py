"""Turn a traecli `--json` event line into a readable pane line plus a state update.

format_event(line) -> (text | None, update)
  text    what to print in the pane (None: nothing worth showing)
  update  fields to merge into the unit state (last, commands, files, tokens_in/out, errors)
Non-JSON lines (stderr, other tools) are shown verbatim. The raw stream is always kept in events.jsonl by the caller.
"""

from __future__ import annotations

import json

RESET, DIM, BOLD, RED, GREEN, YELLOW, CYAN = "\033[0m", "\033[2m", "\033[1m", "\033[31m", "\033[32m", "\033[33m", "\033[36m"


def clip(text: str, n: int) -> str:
    text = " ".join(str(text).split())
    return text if len(text) <= n else text[: n - 1] + "…"


def format_event(line: str, color: bool = True, width: int = 110) -> tuple[str | None, dict]:
    def c(code: str, s: str) -> str:
        return f"{code}{s}{RESET}" if color else s

    line = line.rstrip("\n")
    if not line.strip():
        return None, {}
    try:
        event = json.loads(line)
    except ValueError:
        return line, {"last": clip(line, 100)}
    if not isinstance(event, dict):
        return line, {"last": clip(line, 100)}
    kind = event.get("type")
    item = event.get("item") or {}
    itype = item.get("type")
    if kind == "thread.started":
        return c(DIM, "· session started"), {"last": "session started"}
    if kind == "turn.started":
        return c(DIM, "· turn started"), {}
    if kind == "turn.completed":
        usage = event.get("usage") or {}
        tin, tout = int(usage.get("input_tokens", 0)), int(usage.get("output_tokens", 0))
        return c(DIM, f"· turn done  in={tin} out={tout}"), {"tokens_in_add": tin, "tokens_out_add": tout,
                                                            "last": f"turn done out={tout}"}
    if kind == "item.started" and itype == "command_execution":
        cmd = clip(item.get("command", ""), width - 4)
        return c(CYAN, "$ ") + cmd, {"last": "$ " + clip(cmd, 90)}
    if kind == "item.completed" and itype == "command_execution":
        rc = item.get("exit_code")
        ok = rc in (0, None)
        out = clip((item.get("aggregated_output") or "").strip().splitlines()[-1] if (item.get("aggregated_output") or "").strip() else "", width - 12)
        text = c(GREEN if ok else RED, f"  → rc={rc}") + (c(DIM, "  " + out) if out else "")
        return text, {"commands_add": 1, "last": f"rc={rc} " + out[:70]}
    if kind == "item.started" and itype == "file_change":
        return None, {}
    if kind == "item.completed" and itype == "file_change":
        paths = ", ".join(clip(ch.get("path", ""), 50) for ch in item.get("changes", [])[:3])
        return c(YELLOW, "✎ ") + paths, {"files_add": len(item.get("changes", [])), "last": "edit " + clip(paths, 80)}
    if kind == "item.completed" and itype == "agent_message":
        text = clip(item.get("text", ""), width - 2)
        return c(BOLD, "» ") + text, {"last": clip(item.get("text", ""), 100)}
    if kind == "item.completed" and itype == "reasoning":
        first = (item.get("text", "").strip().splitlines() or [""])[0].strip("* ")
        return c(DIM, "… " + clip(first, width - 4)), {"last": "thinking: " + clip(first, 80)}
    if kind == "item.completed" and itype == "error":
        return c(RED, "! ") + clip(item.get("message", ""), width - 3), {"errors_add": 1}
    return None, {}


def merge_update(state: dict, update: dict) -> dict:
    """Apply a format_event update to a unit state dict (adds the *_add counters)."""
    out = dict(state)
    for key, value in update.items():
        if key.endswith("_add"):
            base = key[:-4]
            out[base] = int(out.get(base, 0)) + int(value)
        else:
            out[key] = value
    return out
