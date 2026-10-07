#!/bin/sh
# The gate runs each checks.d script from the repository root.
exec python3 -c '
from __future__ import annotations

import ast
import pathlib
import re
import sys

ROOTS = [
    pathlib.Path("tools/agents"),
    pathlib.Path("tools/insula"),
    pathlib.Path("bench/harness"),
]
SELF = pathlib.Path("tools/agents/checks.d/90-no-host-tmp.sh")

# Allowed in-capsule mount destinations and their tests:
# - /tmp/run: private run directory mounted inside an insula worker capsule.
# - /tmp/run/tmp: private temp directory inside the mounted run directory.
# - /tmp/solution: private solution mount inside benchmark verification capsules.
# - /tmp/ticket: private ticket mount inside benchmark task capsules.
# - /tmp/task: private prompt/instruction mount inside benchmark task capsules.
# - /tmp: private tmpfs mounted by capsule.py, not the host /tmp.
ALLOW = {
    "/tmp/run",
    "/tmp/run/tmp",
    "/tmp/run/tmp/tmux",
    "/tmp/run/tmp/zsh",
    "/tmp/solution",
    "/tmp/ticket",
    "/tmp/task",
}

SH_TMP_RE = re.compile("/tmp[^\\s" + chr(39) + chr(34) + ";:]*")
TMP_RE = re.compile("(?<![A-Za-z0-9_.-])/tmp(?:/[^\\s" + chr(39) + chr(34) + ";:),}]*)?")


def literal_values(path: pathlib.Path):
    try:
        tree = ast.parse(path.read_text(), filename=str(path))
    except SyntaxError:
        if path.suffix == ".sh":
            for line_no, line in enumerate(path.read_text().splitlines(), 1):
                for match in SH_TMP_RE.finditer(line):
                    yield line_no, match.group(0)
        return
    for node in ast.walk(tree):
        value = getattr(node, "value", None)
        if isinstance(value, str):
            yield node.lineno, value


def allowed(value: str, path: pathlib.Path, literal: str) -> bool:
    # Only the capsule builder defines the root tmpfs and its TMPDIR.
    if path == pathlib.Path("tools/insula/capsule.py") and value in {"/tmp", "/tmp/"}:
        return True
    # These are instructional/captured text, never filesystem destinations.
    if path == pathlib.Path("tools/agents/worker.py") and value == "/tmp" and "temp directory is {temp_dir}" in literal:
        return True
    if path == pathlib.Path("tools/agents/test_lead_launch.py") and value == "/tmp/r-worker-demo-01-hello" and "Folder access" in literal:
        return True
    return any(value == entry or value.startswith(entry + "/") for entry in ALLOW)


def host_tmp_paths(value: str) -> list[str]:
    return TMP_RE.findall(value)


bad = []
for root in ROOTS:
    for path in sorted(root.rglob("*")):
        if path.is_dir() or path.suffix not in {".py", ".sh"}:
            continue
        if path == SELF:
            continue
        for line, value in literal_values(path) or []:
            for found in host_tmp_paths(value):
                if not allowed(found, path, value):
                    bad.append(f"{path}:{line}: {found!r}")

if bad:
    print("host /tmp path literal found outside the allowlist:", file=sys.stderr)
    for item in bad:
        print(item, file=sys.stderr)
    raise SystemExit(1)
'
