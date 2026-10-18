# Coding, Terminal, SWE, and ProgramBench Inventory

This tracked directory contains public task inventories for coding-terminal
benchmark families: Terminal-Bench, DeepSWE, ProgramBench, and SWE-bench.
Each inventory is emitted as both CSV and JSONL so it is easy to skim in a
spreadsheet-like viewer or consume from scripts. These are catalog snapshots,
not runnable Corenius benchmark tasks.

## Regenerate

From the repository root:

```bash
python3 tools/temp_root.py -- python3 bench/inventories/public_agent_benchmarks/scripts/build_coding_inventory.py
```

The script is intentionally stdlib-only. It writes generated artifacts back to
this directory, uses `.scratch/public_agent_benchmarks/cache/` for local clones,
and uses the HuggingFace dataset-server API for SWE-bench.

## Counts

| Inventory | Rows | Source scope |
|---|---:|---|
| `terminal_bench_original_tasks` | 241 | `harbor-framework/terminal-bench-1`, `original-tasks/*/task.yaml` only |
| `deepswe_tasks` | 113 | `datacurve-ai/deep-swe`, `tasks/*/task.toml` plus `instruction.md` |
| `programbench_tasks` | 201 | `facebookresearch/ProgramBench`, `src/programbench/data/tasks/*/task.yaml` |
| `swe_bench_full_test` | 2294 | `SWE-bench/SWE-bench`, `test` split |
| `swe_bench_full_dev` | 225 | `SWE-bench/SWE-bench`, `dev` split |
| `swe_bench_verified_test` | 500 | `SWE-bench/SWE-bench_Verified`, `test` split |
| `swe_bench_lite_test` | 300 | `SWE-bench/SWE-bench_Lite`, `test` split |
| `swe_bench_lite_dev` | 23 | `SWE-bench/SWE-bench_Lite`, `dev` split |

The same numbers are recorded in `counts.json`.

## Files

| File stem | Format | What is included |
|---|---|---|
| `terminal_bench_original_tasks` | CSV, JSONL | Task id, path, first-line instruction preview, difficulty, category, tags, parser, timeouts, source URLs |
| `deepswe_tasks` | CSV, JSONL | Task id, display/original titles, category, language, repository, base commit, instruction preview, timeouts, source URLs |
| `programbench_tasks` | CSV, JSONL | Task id, target repository, commit, language, difficulty, source URLs, tests JSON URL |
| `swe_bench_*` | CSV, JSONL | Instance id, repository, base commit, problem statement preview, version, fail-to-pass/pass-to-pass counts, issue/PR metadata, dataset URL |

## Source Notes

- Terminal-Bench has additional `adapters/*` task templates in the repository.
  These are not counted here because the user asked for benchmark tasks, and
  the public original task set lives under `original-tasks/*`.
- SWE-bench rows are pulled from the current public `SWE-bench/*` HuggingFace
  dataset IDs. Legacy `princeton-nlp/*` dataset IDs expose the same dev/test
  sizes for SWE-bench and SWE-bench Lite, and `princeton-nlp/SWE-bench` also has
  a larger train split. Historical registry subsets and train rows are left as
  future inventory add-ons.
- DeepSWE and ProgramBench inventories are limited to the task metadata shipped
  in their public GitHub repositories. They do not include hidden evaluator
  internals beyond the public task definitions and links emitted here.
