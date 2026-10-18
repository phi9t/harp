# Public Agent Benchmark Inventories

This directory contains tracked inventories of public external benchmarks that
are useful when comparing agent task surfaces. These inventories are not runnable
Corenius benchmark tasks. Runnable tasks live under `bench/tasks/`; this catalog
is normalized source metadata for external benchmarks.

## Layout

| Path | Contents |
|---|---|
| `coding_terminal_swe_program/` | Terminal-Bench original tasks, DeepSWE, ProgramBench, and SWE-bench public splits |
| `retrieval_professional/` | Finance Agent v2 public questions, GDP.pdf public samples/mirror/examples, and Harvey LAB public tasks |
| `scripts/` | Stdlib-only builders and the inventory validator |
| `sources_lock.json` | Source identities, access caveats, row counts, and artifact SHA-256 hashes |

Each inventory group contains `counts.json` plus one CSV and one JSONL file per
normalized artifact stem. Use a CSV parser when counting CSV rows; several
fields contain embedded newlines.

## Regenerate

From the repository root:

```bash
python3 tools/temp_root.py -- python3 bench/inventories/public_agent_benchmarks/scripts/build_coding_inventory.py
python3 tools/temp_root.py -- python3 bench/inventories/public_agent_benchmarks/scripts/build_retrieval_inventory.py
python3 tools/temp_root.py -- python3 bench/inventories/public_agent_benchmarks/scripts/validate_inventory.py
```

The builders write generated CSV/JSONL/count snapshots back into the group
directories and keep clones, downloaded HTML, and other source caches under
`.scratch/public_agent_benchmarks/cache/`.

## Validate

```bash
python3 tools/temp_root.py -- python3 bench/inventories/public_agent_benchmarks/scripts/validate_inventory.py
```

Validation checks CSV row counts with `csv.DictReader`, counts JSONL records
line by line, compares both against each group's `counts.json`, and verifies the
artifact SHA-256 hashes recorded in `sources_lock.json`.
