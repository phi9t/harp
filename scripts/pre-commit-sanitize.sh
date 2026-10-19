#!/bin/sh
set -eu
# Optional hook entry point; checks index blobs without changing the worktree.
exec python3 - "$@" <<'PY'
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def git(*args):
    return subprocess.check_output(["git", *args])


root = Path(os.fsdecode(git("rev-parse", "--show-toplevel").rstrip(b"\n")))
os.chdir(root)
paths = [path for path in git("diff", "--cached", "--name-only", "-z", "--diff-filter=ACMR").split(b"\0") if path]
# The implementation deliberately contains detection fixtures and rule literals.
paths = [path for path in paths if not path.startswith(b"cli/src/sanitize/")]
if not paths:
    raise SystemExit(0)
configured = os.environ.get("HARP_BIN")
candidates = [Path(configured)] if configured else [root / ".build/harp-target/size/harp", root / "target/debug/harp"]
binary = next((path.resolve() for path in candidates if path.is_file() and os.access(path, os.X_OK)), None)
if binary is None:
    print("Sanitizer binary unavailable. Build Harp explicitly and set HARP_BIN to its path.", file=sys.stderr)
    raise SystemExit(1)
failed = False
with tempfile.TemporaryDirectory(prefix="harp-staged-sanitize-") as temporary:
    for index, path in enumerate(paths):
        staged = git("cat-file", "blob", b":" + path)
        snapshot = Path(temporary) / str(index)
        snapshot.write_bytes(staged)
        result = subprocess.run([str(binary), "sanitize", "check", str(snapshot.resolve())], capture_output=True)
        if result.returncode:
            print("Sanitization check failed for staged file: " + repr(os.fsdecode(path)), file=sys.stderr)
            sys.stderr.buffer.write(result.stdout)
            sys.stderr.buffer.write(result.stderr)
            failed = True
if failed:
    print("Review and sanitize the staged content, then stage the repaired file again.", file=sys.stderr)
    raise SystemExit(1)
PY
