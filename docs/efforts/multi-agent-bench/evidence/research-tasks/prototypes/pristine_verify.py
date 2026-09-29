#!/usr/bin/env python3
import argparse
import os, shutil, subprocess
from pathlib import Path
# Restore verifier-owned tests before invoking the task-local command.
def replace_tests(workspace: Path, pristine: Path) -> None:
    tests = workspace / "tests"
    if tests.is_symlink():
        tests.unlink()
    elif tests.is_dir():
        shutil.rmtree(tests)
    elif tests.exists():
        tests.unlink()
    shutil.copytree(pristine, tests, symlinks=False)
    for path in tests.rglob("*"):
        if path.is_symlink():
            raise SystemExit(f"refusing symlink in tests: {path}")
        if path.is_file():
            path.chmod(path.stat().st_mode | 0o400)
def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--workspace", required=True)
    ap.add_argument("--pristine-tests", required=True)
    ap.add_argument("command", nargs=argparse.REMAINDER)
    ns = ap.parse_args()
    workspace = Path(ns.workspace).resolve()
    pristine = Path(ns.pristine_tests).resolve()
    command = ns.command[1:] if ns.command[:1] == ["--"] else ns.command
    if not command:
        raise SystemExit("missing verifier command")
    if not workspace.is_dir():
        raise SystemExit(f"workspace not found: {workspace}")
    if not pristine.is_dir():
        raise SystemExit(f"pristine tests not found: {pristine}")
    replace_tests(workspace, pristine)
    env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin")}
    proc = subprocess.run(command, cwd=workspace, env=env, text=True)
    return proc.returncode
if __name__ == "__main__":
    raise SystemExit(main())
