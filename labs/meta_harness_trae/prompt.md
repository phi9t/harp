# Meta-Harness proposer iteration

Propose exactly three text-classification `MemorySystem` candidates.

## Evidence boundary

- Read `reference/reference_state.json` first.
- The only code authorities are the pinned interface and three shipped
  baselines under `reference/text_classification/`.
- `reference/site-pareto.js` is dated first-party reported state, not a local
  measurement.
- No per-dataset validation history, validation trace, raw upstream proposer
  trace, benchmark output, or held-out result has been reconstructed.
- Do not invent `frontier_val.json`, validation traces, benchmark history,
  scores, or measurements.

## Required writes

Write exactly:

1. `agents/<name>.py` for each of three unique snake-case candidate names;
2. `logs/pending_eval.json`.

Do not modify or add any other file. Each candidate file must:

- define exactly one concrete subclass of
  `text_classification.memory_system.MemorySystem`;
- support cold-start construction with `llm=`;
- implement `predict`, `learn_from_batch`, `get_state`, and `set_state`;
- return a string and metadata dictionary from `predict`;
- serialize state to a string and restore it exactly;
- use only the Python standard library plus the pinned interface;
- avoid network access, subprocesses, filesystem reads, benchmark calls, and
  held-out-test logic.

Validation commands must not create files. If you invoke Python, use `python
-B` or `PYTHONDONTWRITEBYTECODE=1`; do not create `__pycache__` or `.pyc`
files.

Use the upstream-compatible shape:

```json
{
  "candidates": [
    {
      "name": "snake_case_name",
      "path": "agents/snake_case_name.py",
      "axis": "short search-axis label",
      "hypothesis": "specific mechanism hypothesis"
    }
  ]
}
```

After the final tool call, immediately return only the schema object: no
preamble, progress update, Markdown fence, or trailing prose. It must contain
the same three candidate objects and schema version
`harp-meta-harness-trae-final/v1`. Candidate quality is not being benchmarked;
this iteration tests proposer and candidate-interface compatibility only.
