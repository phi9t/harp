from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
LAB = Path(__file__).resolve().parent
SNAPSHOT = ROOT / "evidence/implementations/meta_harness/snapshot"
SITE = ROOT / "evidence/meta_harness/site"
REFERENCE_FILES = {
    "reference/text_classification/__init__.py": (
        SNAPSHOT / "reference_examples/text_classification/__init__.py"
    ),
    "reference/text_classification/memory_system.py": (
        SNAPSHOT / "reference_examples/text_classification/memory_system.py"
    ),
    "reference/text_classification/llm.py": (
        SNAPSHOT / "reference_examples/text_classification/llm.py"
    ),
    "reference/text_classification/agents/__init__.py": (
        SNAPSHOT / "reference_examples/text_classification/agents/__init__.py"
    ),
    "reference/text_classification/agents/no_memory.py": (
        SNAPSHOT / "reference_examples/text_classification/agents/no_memory.py"
    ),
    "reference/text_classification/agents/fewshot_memory.py": (
        SNAPSHOT / "reference_examples/text_classification/agents/fewshot_memory.py"
    ),
    "reference/text_classification/agents/fewshot_all.py": (
        SNAPSHOT / "reference_examples/text_classification/agents/fewshot_all.py"
    ),
    "reference/site-pareto.js": SITE / "static/data/pareto.js",
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--run-dir",
        type=Path,
        default=LAB / ".runs/live",
    )
    args = parser.parse_args()
    run_dir = args.run_dir.resolve()
    if run_dir.exists():
        raise SystemExit(f"refusing to replace existing run directory: {run_dir}")
    workspace = run_dir / "workspace"
    (workspace / "agents").mkdir(parents=True)
    (workspace / "logs").mkdir()
    for relative, source in REFERENCE_FILES.items():
        destination = workspace / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    state = {
        "schema_version": "harp-meta-harness-reference-state/v1",
        "paper_source_id": "META-HARNESS",
        "repository_source_id": "META-HARNESS-REPO",
        "repository_revision": "44b9942127847f7421db70d8c7e48407f09a3c70",
        "site_source_id": "META-HARNESS-SITE",
        "site_last_modified": "Tue, 04 Aug 2026 03:07:36 GMT",
        "reported_site_state": {
            "measurement_status": "first-party-reported-not-local",
            "best_accuracy": 48.6,
            "ace_accuracy": 40.9,
            "best_context_characters": 45500,
            "ace_context_characters": 203000,
            "reported_improvement_points": 7.7,
        },
        "available_inputs": [
            "pinned MemorySystem interface",
            "pinned no_memory baseline",
            "pinned fewshot_memory baseline",
            "pinned fewshot_all baseline",
            "dated first-party Pareto data",
        ],
        "missing_inputs": [
            "per-dataset validation history",
            "frontier_val.json",
            "validation traces",
            "raw upstream proposer traces",
            "benchmark history",
            "held-out results",
        ],
        "local_measurements": [],
    }
    (workspace / "reference/reference_state.json").write_text(
        json.dumps(state, indent=2, sort_keys=True) + "\n"
    )
    shutil.copyfile(LAB / "prompt.md", run_dir / "prompt.md")
    shutil.copyfile(
        LAB / "final_response.schema.json",
        run_dir / "final_response.schema.json",
    )
    digests = {
        path.relative_to(workspace).as_posix(): sha256(path)
        for path in sorted((workspace / "reference").rglob("*"))
        if path.is_file()
    }
    (run_dir / "reference-digests.json").write_text(
        json.dumps(digests, indent=2, sort_keys=True) + "\n"
    )
    print(run_dir)


if __name__ == "__main__":
    main()
