#!/usr/bin/env python3
"""Executable entry point for the collab CLI."""
from __future__ import annotations

from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cli import main


if __name__ == "__main__":
    raise SystemExit(main())
