#!/usr/bin/env python3
"""Validate public benchmark inventory snapshots."""

from __future__ import annotations

import csv
import hashlib
import json
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
LOCK_PATH = ROOT / "sources_lock.json"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def count_csv(path: Path) -> int:
    with path.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            raise ValueError("missing CSV header")
        return sum(1 for _ in reader)


def count_jsonl(path: Path) -> int:
    count = 0
    with path.open(encoding="utf-8") as f:
        for line_number, line in enumerate(f, start=1):
            if not line.strip():
                raise ValueError(f"blank JSONL line {line_number}")
            json.loads(line)
            count += 1
    return count


def validate_group(group: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    group_id = str(group.get("group_id", "<unknown>"))
    group_path = ROOT / str(group.get("path", ""))
    counts_path = group_path / "counts.json"

    if not group_path.is_dir():
        return [f"{group_id}: missing group directory {group_path.relative_to(ROOT)}"]
    if not counts_path.is_file():
        return [f"{group_id}: missing counts.json"]

    counts = load_json(counts_path)
    locked_counts = group.get("generated_row_counts", {})
    if counts != locked_counts:
        errors.append(f"{group_id}/counts.json: counts differ from sources_lock.json")

    if not isinstance(counts, dict):
        return [f"{group_id}/counts.json: expected object"]

    for stem, expected in sorted(counts.items()):
        if not isinstance(expected, int):
            errors.append(f"{group_id}/{stem}: count is not an integer")
            continue

        csv_path = group_path / f"{stem}.csv"
        jsonl_path = group_path / f"{stem}.jsonl"
        if not csv_path.is_file():
            errors.append(f"{group_id}/{stem}.csv: missing")
        else:
            try:
                actual_csv = count_csv(csv_path)
            except Exception as exc:
                errors.append(f"{group_id}/{stem}.csv: {exc}")
            else:
                if actual_csv != expected:
                    errors.append(f"{group_id}/{stem}.csv: expected {expected}, got {actual_csv}")

        if not jsonl_path.is_file():
            errors.append(f"{group_id}/{stem}.jsonl: missing")
        else:
            try:
                actual_jsonl = count_jsonl(jsonl_path)
            except Exception as exc:
                errors.append(f"{group_id}/{stem}.jsonl: {exc}")
            else:
                if actual_jsonl != expected:
                    errors.append(f"{group_id}/{stem}.jsonl: expected {expected}, got {actual_jsonl}")

    artifacts = group.get("artifacts_sha256", {})
    if not isinstance(artifacts, dict):
        errors.append(f"{group_id}: artifacts_sha256 is not an object")
        return errors

    for rel_path, expected_hash in sorted(artifacts.items()):
        artifact_path = group_path / rel_path
        if not artifact_path.is_file():
            errors.append(f"{group_id}/{rel_path}: missing hashed artifact")
            continue
        actual_hash = sha256(artifact_path)
        if actual_hash != expected_hash:
            errors.append(f"{group_id}/{rel_path}: sha256 mismatch")

    return errors


def main() -> int:
    lock = load_json(LOCK_PATH)
    errors: list[str] = []
    if lock.get("schema_version") != 1:
        errors.append("sources_lock.json: expected schema_version 1")

    root_artifacts = lock.get("root_artifacts_sha256", {})
    if not isinstance(root_artifacts, dict):
        errors.append("sources_lock.json: root_artifacts_sha256 is not an object")
    else:
        for rel_path, expected_hash in sorted(root_artifacts.items()):
            artifact_path = ROOT / rel_path
            if not artifact_path.is_file():
                errors.append(f"{rel_path}: missing hashed artifact")
                continue
            actual_hash = sha256(artifact_path)
            if actual_hash != expected_hash:
                errors.append(f"{rel_path}: sha256 mismatch")

    groups = lock.get("groups", [])
    if not isinstance(groups, list) or not groups:
        errors.append("sources_lock.json: groups must be a nonempty list")
    else:
        for group in groups:
            if not isinstance(group, dict):
                errors.append("sources_lock.json: group entry is not an object")
                continue
            errors.extend(validate_group(group))

    if errors:
        print("inventory validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"validated {len(groups)} inventory groups")
    return 0


if __name__ == "__main__":
    sys.exit(main())
