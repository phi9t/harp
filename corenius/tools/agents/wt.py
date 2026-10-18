#!/usr/bin/env python3
"""Deprecated worktree compatibility shim; use tools/collab/collab.py for new workflows."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "collab"))
from worktree_compat import *  # noqa: E402,F403 - retain the public Python API for one release
from worktree_compat import _load_plugins  # noqa: E402,F401

if __name__ == "__main__":
    raise SystemExit(main())
