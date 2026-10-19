`analytics.summarize(rows)` is a public API consumed by typed callers. It must
return a `RunSummary` dataclass with these fields:

- `count: int`
- `total_ms: int`
- `average_ms: float`

Each row is a mapping with a `duration_ms` value. Sum durations as integers.
For an empty input, return `count=0`, `total_ms=0`, and `average_ms=0.0`. The
existing caller in `caller.py` must work without changes. The verifier uses
runtime behavior and standard-library type-shape checks only; there are no
third-party type checkers or dependencies.
