#!/usr/bin/env python3
"""Scripted worker for the lead-driven access-profile readiness scenario."""

from __future__ import annotations

import http.server
import json
import os
import subprocess
import sys
import threading
import urllib.request
from pathlib import Path


def emit(event: dict) -> None:
    print(json.dumps(event), flush=True)


def git(root: Path, *args: str) -> None:
    subprocess.run(["git", "-C", str(root), *args], check=True)


def git_commit(root: Path, message: str) -> None:
    git(root, "-c", "user.email=bench@corenius", "-c", "user.name=bench", "commit", "-q", "-m", message)


def write_last_message(args: list[str], text: str) -> None:
    if "-o" in args:
        Path(args[args.index("-o") + 1]).write_text(text)


def commit_result(root: Path, text: str) -> None:
    (root / "result.txt").write_text(text)
    git(root, "add", "result.txt")
    git_commit(root, "result")


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b"ok")

    def log_message(self, format, *args):
        return


def fetch_loopback() -> str:
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    url = f"http://127.0.0.1:{server.server_port}/"
    try:
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": f"curl {url}"}})
        body = urllib.request.urlopen(url, timeout=2).read().decode()
        return body
    finally:
        server.shutdown()


def shell_timeout(args: list[str]) -> str:
    return args[args.index("--shell-tool-timeout") + 1] if "--shell-tool-timeout" in args else ""


def external_add_paths() -> list[Path]:
    raw = os.environ.get("CORENIUS_READINESS_EXTERNAL_ADD_PATHS", "")
    return [Path(part) for part in raw.split(os.pathsep) if part]


def main() -> int:
    args = sys.argv[1:]
    root = Path(args[args.index("-C") + 1])
    mode = os.environ.get("CORENIUS_READINESS_PARTICIPANT", "default-network")
    emit({"type": "thread.started"})
    if mode == "default-network":
        commit_result(root, f"loopback {fetch_loopback()}\n")
    elif mode == "network-off-download":
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "curl http://example.com/archive.tgz"}})
        commit_result(root, "download attempted\n")
    elif mode == "long-command":
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": "sleep 121"}})
        if shell_timeout(args) != "125s":
            print(f"single-command timeout too short: {shell_timeout(args)}", file=sys.stderr)
            return 124
        subprocess.run(["sleep", "121"], check=True)
        commit_result(root, f"timeout {shell_timeout(args)}\n")
    elif mode == "external-add-path":
        targets = external_add_paths()
        if not targets:
            print("no external add path provided", file=sys.stderr)
            return 66
        targets[0].mkdir(parents=True, exist_ok=True)
        target = targets[0] / "allowed-output.txt"
        target.write_text("created outside repo\n")
        emit({"type": "item.completed", "item": {"type": "command_execution", "command": f"python write {target}"}})
        commit_result(root, "external add path recorded\n")
    else:
        print(f"unknown participant mode {mode}", file=sys.stderr)
        return 64
    write_last_message(args, "ready for review\n")
    emit({"type": "turn.completed"})
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
