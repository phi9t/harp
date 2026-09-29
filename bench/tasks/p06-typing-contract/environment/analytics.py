from __future__ import annotations

from typing import Any


def summarize(rows: list[dict[str, Any]]) -> dict[str, Any]:
    total_ms = sum(int(row["duration_ms"]) for row in rows)
    count = len(rows)
    average_ms = total_ms / count if count else 0.0
    return {"count": count, "total_ms": total_ms, "average_ms": average_ms}
