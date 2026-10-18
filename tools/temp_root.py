#!/usr/bin/env python3
"""Run Corenius commands with an owned, checked temporary root."""
import argparse
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Linux sockaddr_un permits 107 pathname bytes plus its trailing NUL. Reserve
# 48 bytes for nested temporary directories and socket names (e.g. pymp/listener).
UNIX_PATH_BYTES = 107
SOCKET_SUFFIX_BYTES = 48
MAX_ROOT_BYTES = UNIX_PATH_BYTES - SOCKET_SUFFIX_BYTES


def temporary_root(value=None, *, socket_suffix_bytes=None):
    """Validate a data root, optionally reserving a complete socket suffix budget.

    Callers handling regular files (including host setup of a private sandbox)
    need no AF_UNIX restriction. The command wrapper requests the standard
    nested socket budget explicitly; socket owners can request their own.
    """
    selected = value if value is not None else os.environ.get("TMPDIR")
    root = Path(selected).expanduser() if selected else ROOT / ".scratch" / "tmp"
    if not root.is_absolute():
        raise ValueError("TMPDIR must be absolute")
    root = root.resolve()
    if not selected and len(os.fsencode(root)) > MAX_ROOT_BYTES:
        root = (Path.home() / '.ctmp').resolve()
    forbidden = Path("/") / "tmp"
    if root == forbidden or forbidden in root.parents:
        raise ValueError("TMPDIR must be outside the system temporary directory")
    if root == Path("/"):
        raise ValueError("TMPDIR must be an owned subdirectory")
    if socket_suffix_bytes is not None:
        if not 0 <= socket_suffix_bytes < UNIX_PATH_BYTES:
            raise ValueError("invalid Unix socket suffix budget")
        budget = UNIX_PATH_BYTES - socket_suffix_bytes
        if len(os.fsencode(root)) > budget:
            raise ValueError(f"TMPDIR exceeds Unix socket budget: root must be at most {budget} bytes "
                             f"to reserve {socket_suffix_bytes} bytes for socket paths; select a shorter owned root")
    root.mkdir(mode=0o700, parents=True, exist_ok=True)
    if root.stat().st_uid != os.getuid():
        raise ValueError("TMPDIR must be owned by the current user")
    if root.stat().st_mode & 0o022:
        raise ValueError("TMPDIR must not be writable by other users")
    return root


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        root = temporary_root(socket_suffix_bytes=SOCKET_SUFFIX_BYTES)
    except (ValueError, OSError) as error:
        parser.exit(2, f"temporary root: {error}\n")
    if not args.command:
        print(root)
        return 0
    command = args.command[1:] if args.command[0] == "--" else args.command
    environment = dict(os.environ, TMPDIR=str(root), TMP=str(root), TEMP=str(root))
    # Preserve signal ownership: cancellation must reach the supervised command,
    # not stop at a waiting wrapper that could kill it before cleanup finishes.
    os.execvpe(command[0], command, environment)


if __name__ == "__main__":
    raise SystemExit(main())
