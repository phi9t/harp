from __future__ import annotations

import argparse
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from meta_harness_trae import build_command


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run_dir", type=Path)
    parser.add_argument("--print-command", action="store_true")
    args = parser.parse_args()
    run_dir = args.run_dir.resolve()
    workspace = run_dir / "workspace"
    schema = run_dir / "final_response.schema.json"
    final = run_dir / "final_response.json"
    command = build_command(workspace, schema, final)
    metadata = {
        "schema_version": "harp-meta-harness-trae-command/v1",
        "command": command,
        "cwd": str(workspace),
        "model": "gpt-5.4",
        "approval_policy": "never",
        "permission_compatibility": (
            "TRAE CLI 0.200.19 rejects simultaneous legacy permission-mode "
            "and sandbox overrides; approval_policy=never plus "
            "sandbox=workspace-write preserves the required headless boundary"
        ),
        "sandbox": "workspace-write",
        "allowed_tools": ["Read", "Glob", "Grep", "Bash", "Write", "Edit"],
        "network_tools": [],
        "subagents": False,
        "benchmark_requested": False,
        "environment_overrides": {
            "PYTHONDONTWRITEBYTECODE": "1",
        },
    }
    (run_dir / "command.json").write_text(
        json.dumps(metadata, indent=2, sort_keys=True) + "\n"
    )
    if args.print_command:
        print(json.dumps(metadata, indent=2))
        return
    if (run_dir / "events.jsonl").exists():
        raise SystemExit("refusing to rerun an existing proposer attempt")
    metadata["started_at"] = datetime.now(timezone.utc).isoformat()
    prompt = (run_dir / "prompt.md").read_text()
    environment = os.environ.copy()
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    with (run_dir / "events.jsonl").open("w") as stdout, (
        run_dir / "stderr.txt"
    ).open("w") as stderr:
        completed = subprocess.run(
            command,
            input=prompt,
            text=True,
            stdout=stdout,
            stderr=stderr,
            check=False,
            env=environment,
        )
    metadata["completed_at"] = datetime.now(timezone.utc).isoformat()
    metadata["returncode"] = completed.returncode
    (run_dir / "command.json").write_text(
        json.dumps(metadata, indent=2, sort_keys=True) + "\n"
    )
    raise SystemExit(completed.returncode)


if __name__ == "__main__":
    main()
