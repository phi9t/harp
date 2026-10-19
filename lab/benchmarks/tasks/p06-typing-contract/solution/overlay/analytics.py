from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class RunSummary:
    count: int
    total_ms: int
    average_ms: float


def summarize(rows: list[dict[str, Any]]) -> RunSummary:
    total_ms = sum(int(row["duration_ms"]) for row in rows)
    count = len(rows)
    average_ms = float(total_ms / count) if count else 0.0
    return RunSummary(count=count, total_ms=total_ms, average_ms=average_ms)
