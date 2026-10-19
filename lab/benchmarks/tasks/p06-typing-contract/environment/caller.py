from __future__ import annotations

from typing import Any

from analytics import summarize


def headline(rows: list[dict[str, Any]]) -> str:
    summary = summarize(rows)
    return f"{summary.count} runs, average {summary.average_ms:.1f} ms"
