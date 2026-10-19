#!/usr/bin/env python3
"""Aggregate Emacs gym benchmark summaries for the selected Insula rootfs."""

from __future__ import annotations

import argparse
import json
import pathlib
from datetime import datetime, timezone
from typing import Any


BENCHMARKS = (
    "magit",
    "comint",
    "jupyter",
    "lsp",
    "org",
    "dired",
    "bridge",
    "bridge-daemon",
)
SOMA_INSULA_AUTHORITY = "scripts/insula/soma_insula.py"
MODULE_AUTHORITY_BENCHMARKS = frozenset(BENCHMARKS)


def load_json(path: pathlib.Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def resolve_selected_rootfs(store_root: pathlib.Path) -> tuple[str, pathlib.Path, pathlib.Path, dict[str, Any]]:
    store = store_root.resolve()
    selected_path = store / "selected.json"
    if not selected_path.is_file():
        raise SystemExit(f"missing rootfs selection: {selected_path}")
    if store_root.is_symlink():
        raise SystemExit(f"rootfs store must not be a symlink: {store_root}")

    selected = load_json(selected_path)
    store_id = str(selected.get("store_id", ""))
    if not store_id or "/" in store_id or store_id in {".", ".."}:
        raise SystemExit(f"invalid selected rootfs store id: {store_id}")

    content_path = store / "content"
    if content_path.is_symlink():
        raise SystemExit(f"rootfs content directory must not be a symlink: {content_path}")
    candidate = content_path / store_id
    if candidate.is_symlink():
        raise SystemExit(f"selected rootfs must not be a symlink: {candidate}")
    rootfs = candidate.resolve()
    if rootfs.parent != content_path.resolve():
        raise SystemExit(f"selected rootfs escapes store: {rootfs}")
    if not rootfs.is_dir():
        raise SystemExit(f"selected rootfs is not a directory: {rootfs}")

    contract_path = rootfs / "etc" / "soma-insula-rootfs-contract.json"
    if not contract_path.is_file():
        raise SystemExit(f"selected rootfs lacks contract: {contract_path}")
    return store_id, rootfs, contract_path, load_json(contract_path)


def expected_identity(
    store_id: str, rootfs: pathlib.Path, contract_path: pathlib.Path, contract: dict[str, Any]
) -> dict[str, Any]:
    return {
        "store_id": store_id,
        "rootfs": str(rootfs),
        "rootfs_contract_host_path": str(contract_path),
        "rootfs_recipe_sha256": contract.get("rootfs_recipe_sha256", ""),
        "base_image": contract.get("base_image", ""),
        "image_tag": contract.get("image_tag", ""),
        "image_id": contract.get("image_id", ""),
        "materialization_kind": contract.get("materialization_kind", ""),
        "rootfs_tree_manifest": contract.get("rootfs_tree_manifest", {}),
    }


def check_plan(source: str, plan: dict[str, Any] | None, expected: dict[str, Any], run_id: str) -> list[str]:
    if not isinstance(plan, dict):
        return [f"{source}: missing object"]

    checks: list[tuple[str, Any]] = [
        ("rootfs", expected["rootfs"]),
        ("rootfs_contract_host_path", expected["rootfs_contract_host_path"]),
        ("rootfs_recipe_sha256", expected["rootfs_recipe_sha256"]),
        ("base_image", expected["base_image"]),
        ("image_tag", expected["image_tag"]),
        ("image_id", expected["image_id"]),
        ("rootfs_tree_manifest", expected["rootfs_tree_manifest"]),
        ("mode", "selected-container-rootfs-insula"),
        ("dependency_closure", "rootfs_owned"),
        ("glibc_owner", "container-image-rootfs"),
        ("nix_allowed", False),
        ("run_id", run_id),
    ]
    errors: list[str] = []
    for field, wanted in checks:
        got = plan.get(field)
        if got != wanted:
            errors.append(f"{source}:{field}: expected {wanted!r}, got {got!r}")

    rootfs = str(plan.get("rootfs", ""))
    if expected["store_id"] and expected["store_id"] not in rootfs:
        errors.append(
            f"{source}:rootfs: selected store id {expected['store_id']!r} not present in {rootfs!r}"
        )
    return errors


def check_authority(source: str, name: str, plan: dict[str, Any] | None) -> list[str]:
    if name not in MODULE_AUTHORITY_BENCHMARKS:
        return []
    if not isinstance(plan, dict):
        return []
    got = plan.get("soma_insula_authority")
    if got != SOMA_INSULA_AUTHORITY:
        return [f"{source}:soma_insula_authority: expected {SOMA_INSULA_AUTHORITY!r}, got {got!r}"]
    return []


def aggregate(out_dir: pathlib.Path, run_id: str, store_root: pathlib.Path) -> dict[str, Any]:
    store_id, rootfs, contract_path, contract = resolve_selected_rootfs(store_root)
    current_rootfs = expected_identity(store_id, rootfs, contract_path, contract)

    records = []
    all_passed = True
    total_scenarios = 0
    scenarios_as_expected = 0

    for name in BENCHMARKS:
        summary_path = out_dir / name / "summary.json"
        plan_path = out_dir / name / "insula-plan.json"
        summary = load_json(summary_path)
        plan = load_json(plan_path)
        total = int(summary.get("total_scenarios", 0))
        expected = int(summary.get("scenarios_as_expected", 0))
        insula = bool(summary.get("insula_checks_passed"))
        current_selection_errors = []
        current_selection_errors.extend(check_plan("insula-plan.json", plan, current_rootfs, run_id))
        current_selection_errors.extend(check_authority("insula-plan.json", name, plan))
        if "plan" in summary:
            current_selection_errors.extend(
                check_plan("summary.plan", summary.get("plan"), current_rootfs, run_id)
            )
            current_selection_errors.extend(check_authority("summary.plan", name, summary.get("plan")))
        current_selection_ok = not current_selection_errors
        passed = total == expected and insula and current_selection_ok
        all_passed = all_passed and passed
        total_scenarios += total
        scenarios_as_expected += expected
        records.append(
            {
                "name": name,
                "benchmark": summary.get("benchmark", name),
                "summary": str(summary_path),
                "insula_plan": str(plan_path),
                "total_scenarios": total,
                "scenarios_as_expected": expected,
                "insula_checks_passed": insula,
                "current_selection_checks_passed": current_selection_ok,
                "current_selection_errors": current_selection_errors,
                "passed": passed,
            }
        )

    return {
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "benchmark": "emacs-gym-selected-rootfs",
        "run_id": run_id,
        "current_rootfs": current_rootfs,
        "total_benchmarks": len(records),
        "benchmarks_passed": sum(1 for record in records if record["passed"]),
        "total_scenarios": total_scenarios,
        "scenarios_as_expected": scenarios_as_expected,
        "passed": all_passed,
        "records": records,
    }


def write_markdown(out_dir: pathlib.Path, aggregate_result: dict[str, Any]) -> None:
    lines = [
        "# Emacs Gym Selected-Rootfs Benchmark Summary",
        "",
        f"- Generated: {aggregate_result['generated_at']}",
        "- Benchmark: `emacs-gym-selected-rootfs`",
        f"- Run id: `{aggregate_result['run_id']}`",
        f"- Selected store id: `{aggregate_result['current_rootfs']['store_id']}`",
        f"- Selected image id: `{aggregate_result['current_rootfs']['image_id']}`",
        f"- Selected recipe sha256: `{aggregate_result['current_rootfs']['rootfs_recipe_sha256']}`",
        f"- Benchmarks passed: {aggregate_result['benchmarks_passed']} / {aggregate_result['total_benchmarks']}",
        f"- Total scenarios: {aggregate_result['total_scenarios']}",
        f"- Scenarios as expected: {aggregate_result['scenarios_as_expected']}",
        f"- Passed: `{aggregate_result['passed']}`",
        "",
    ]
    for record in aggregate_result["records"]:
        lines.extend(
            [
                f"## {record['name']}",
                "",
                f"- Benchmark: `{record['benchmark']}`",
                f"- Total scenarios: {record['total_scenarios']}",
                f"- Scenarios as expected: {record['scenarios_as_expected']}",
                f"- Insula checks passed: `{record['insula_checks_passed']}`",
                f"- Current selection checks passed: `{record['current_selection_checks_passed']}`",
                f"- Passed: `{record['passed']}`",
                f"- Summary: `{record['summary']}`",
                f"- Insula plan: `{record['insula_plan']}`",
                "",
            ]
        )
        if record["current_selection_errors"]:
            lines.append("Current selection errors:")
            lines.append("")
            for error in record["current_selection_errors"]:
                lines.append(f"- `{error}`")
            lines.append("")
    (out_dir / "summary.md").write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("out_dir", type=pathlib.Path)
    parser.add_argument("run_id")
    parser.add_argument(
        "--store-root",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[2]
        / ".scratch"
        / "emacs-gym"
        / "insula"
        / "rootfs-store",
    )
    args = parser.parse_args()

    aggregate_result = aggregate(args.out_dir, args.run_id, args.store_root)
    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "summary.json").write_text(
        json.dumps(aggregate_result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    write_markdown(args.out_dir, aggregate_result)

    if not aggregate_result["passed"]:
        print(json.dumps(aggregate_result, indent=2, sort_keys=True))
        return 1
    print(f"emacs-gym-benchmarks: wrote aggregate summary to {args.out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
