#!/usr/bin/env python3
"""Build public task inventories for agent benchmarks.

The script intentionally uses only the Python standard library so it can run in
this checkout without adding dependencies.
"""

from __future__ import annotations

import csv
import json
import re
import subprocess
import sys
import time
import tomllib
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "coding_terminal_swe_program"
CACHE = Path(".scratch/public_agent_benchmarks/cache/coding_terminal_swe_program")
REPOS = CACHE / "repos"
USER_AGENT = "corenius-benchmark-inventory/1.0"


SOURCE_REPOS = {
    "harbor-framework/terminal-bench-1": "d28711d0da2675d0bb1d56de45ae5df6082438a3",
    "datacurve-ai/deep-swe": "0b9fabbb63b9104d678fe965e1632f2dd9eaa2ea",
    "facebookresearch/ProgramBench": "27f02157c785f8da3647aa6dbbe6b9137f99f10e",
}


def run(command: list[str]) -> None:
    subprocess.run(command, check=True)


def ensure_clone(owner_repo: str, dest: Path) -> None:
    commit = SOURCE_REPOS[owner_repo]
    if not (dest / ".git").exists():
        dest.parent.mkdir(parents=True, exist_ok=True)
        run(["git", "clone", "--no-checkout", f"https://github.com/{owner_repo}", str(dest)])
    run(["git", "-C", str(dest), "fetch", "--depth", "1", "origin", commit])
    run(["git", "-C", str(dest), "checkout", "--force", commit])


def fetch_text(url: str, retries: int = 4) -> str:
    last_error: Exception | None = None
    for attempt in range(retries):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(req, timeout=45) as response:
                return response.read().decode("utf-8")
        except Exception as exc:  # pragma: no cover - this is a scratch script
            last_error = exc
            time.sleep(1.5 * (attempt + 1))
    raise RuntimeError(f"failed to fetch {url}: {last_error}")


def fetch_json(url: str) -> Any:
    return json.loads(fetch_text(url))


def write_csv(path: Path, rows: list[dict[str, Any]], fields: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fields, extrasaction="ignore")
        writer.writeheader()
        for row in rows:
            writer.writerow({field: row.get(field, "") for field in fields})


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")


def first_line(text: str, limit: int = 220) -> str:
    line = " ".join(text.strip().splitlines()).strip()
    if len(line) > limit:
        return line[: limit - 1].rstrip() + "..."
    return line


def parse_simple_yaml_scalar(text: str, key: str) -> str:
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if not line.startswith(f"{key}:"):
            continue
        value = line.split(":", 1)[1].strip()
        if value in {"|", "|-", ">", ">-"}:
            block: list[str] = []
            for child in lines[i + 1 :]:
                if child and not child.startswith((" ", "\t")) and re.match(r"^[A-Za-z0-9_-]+:", child):
                    break
                block.append(child[2:] if child.startswith("  ") else child)
            return "\n".join(block).strip()
        return value.strip('"').strip("'")
    return ""


def parse_simple_yaml_list(text: str, key: str) -> list[str]:
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if not line.startswith(f"{key}:"):
            continue
        out: list[str] = []
        for child in lines[i + 1 :]:
            if child.startswith("  - "):
                out.append(child[4:].strip().strip('"').strip("'"))
                continue
            if child and not child.startswith((" ", "\t")):
                break
        return out
    return []


def github_tree(owner_repo: str, ref: str = "main") -> list[dict[str, Any]]:
    url = f"https://api.github.com/repos/{owner_repo}/git/trees/{ref}?recursive=1"
    return fetch_json(url)["tree"]


def raw_github(owner_repo: str, path: str, ref: str = "main") -> str:
    return f"https://raw.githubusercontent.com/{owner_repo}/{ref}/{path}"


def terminal_bench() -> list[dict[str, Any]]:
    owner_repo = "harbor-framework/terminal-bench-1"
    repo_root = REPOS / "terminal-bench-1"
    ensure_clone(owner_repo, repo_root)
    paths = sorted(
        path.relative_to(repo_root).as_posix()
        for path in (repo_root / "original-tasks").glob("*/task.yaml")
    )
    rows: list[dict[str, Any]] = []
    for path in paths:
        task_id = path.split("/")[1]
        url = raw_github(owner_repo, path)
        text = (repo_root / path).read_text(encoding="utf-8")
        rows.append(
            {
                "benchmark": "terminal-bench-public-original-tasks",
                "task_id": task_id,
                "path": path,
                "instruction": first_line(parse_simple_yaml_scalar(text, "instruction"), 320),
                "difficulty": parse_simple_yaml_scalar(text, "difficulty"),
                "category": parse_simple_yaml_scalar(text, "category"),
                "tags": ";".join(parse_simple_yaml_list(text, "tags")),
                "parser_name": parse_simple_yaml_scalar(text, "parser_name"),
                "max_agent_timeout_sec": parse_simple_yaml_scalar(text, "max_agent_timeout_sec"),
                "max_test_timeout_sec": parse_simple_yaml_scalar(text, "max_test_timeout_sec"),
                "source_url": f"https://github.com/{owner_repo}/tree/main/original-tasks/{task_id}",
                "task_yaml_url": url,
            }
        )
    return rows


def deepswe() -> list[dict[str, Any]]:
    owner_repo = "datacurve-ai/deep-swe"
    repo_root = REPOS / "deep-swe"
    ensure_clone(owner_repo, repo_root)
    paths = sorted(
        path.relative_to(repo_root).as_posix()
        for path in (repo_root / "tasks").glob("*/task.toml")
    )
    rows: list[dict[str, Any]] = []
    for path in paths:
        task_dir = "/".join(path.split("/")[:-1])
        task_id = path.split("/")[1]
        toml_url = raw_github(owner_repo, path)
        instruction_url = raw_github(owner_repo, f"{task_dir}/instruction.md")
        doc = tomllib.loads((repo_root / path).read_text(encoding="utf-8"))
        metadata = doc.get("metadata", {})
        instruction = (repo_root / task_dir / "instruction.md").read_text(encoding="utf-8")
        rows.append(
            {
                "benchmark": "deepswe-v1.1-public-repo",
                "task_id": metadata.get("task_id", task_id),
                "path": task_dir,
                "display_title": metadata.get("display_title", ""),
                "display_description": metadata.get("display_description", ""),
                "original_title": metadata.get("original_title", ""),
                "category": metadata.get("category", ""),
                "language": metadata.get("language", ""),
                "repository_url": metadata.get("repository_url", ""),
                "base_commit_hash": metadata.get("base_commit_hash", ""),
                "instruction": first_line(instruction, 420),
                "agent_timeout_sec": doc.get("agent", {}).get("timeout_sec", ""),
                "verifier_timeout_sec": doc.get("verifier", {}).get("timeout_sec", ""),
                "source_url": f"https://github.com/{owner_repo}/tree/main/{task_dir}",
                "task_toml_url": toml_url,
            }
        )
    return rows


def programbench() -> list[dict[str, Any]]:
    owner_repo = "facebookresearch/ProgramBench"
    prefix = "src/programbench/data/tasks"
    repo_root = REPOS / "ProgramBench"
    ensure_clone(owner_repo, repo_root)
    paths = sorted(
        path.relative_to(repo_root).as_posix()
        for path in (repo_root / prefix).glob("*/task.yaml")
    )
    rows: list[dict[str, Any]] = []
    for path in paths:
        task_id = path.split("/")[-2]
        url = raw_github(owner_repo, path)
        text = (repo_root / path).read_text(encoding="utf-8")
        rows.append(
            {
                "benchmark": "programbench-public-repo",
                "task_id": task_id,
                "repository": parse_simple_yaml_scalar(text, "repository"),
                "commit": parse_simple_yaml_scalar(text, "commit"),
                "language": parse_simple_yaml_scalar(text, "language"),
                "difficulty": parse_simple_yaml_scalar(text, "difficulty"),
                "source_url": f"https://github.com/{owner_repo}/tree/main/{prefix}/{task_id}",
                "task_yaml_url": url,
                "tests_json_url": raw_github(owner_repo, f"{prefix}/{task_id}/tests.json"),
            }
        )
    return rows


def hf_dataset_rows(dataset: str, split: str) -> list[dict[str, Any]]:
    encoded = urllib.parse.quote(dataset, safe="")
    size_url = f"https://datasets-server.huggingface.co/size?dataset={encoded}"
    size_doc = fetch_json(size_url)
    split_doc = [
        item
        for item in size_doc["size"]["splits"]
        if item["config"] == "default" and item["split"] == split
    ][0]
    total = int(split_doc["num_rows"])

    rows: list[dict[str, Any]] = []
    page = 100
    for offset in range(0, total, page):
        length = min(page, total - offset)
        url = (
            "https://datasets-server.huggingface.co/rows?"
            f"dataset={encoded}&config=default&split={split}&offset={offset}&length={length}"
        )
        data = fetch_json(url)
        rows.extend(row["row"] for row in data["rows"])
    if len(rows) != total:
        raise RuntimeError(f"{dataset}/{split}: expected {total}, got {len(rows)}")
    return rows


def swe_bench(dataset: str, split: str) -> list[dict[str, Any]]:
    raw_rows = hf_dataset_rows(dataset, split)
    out: list[dict[str, Any]] = []
    for row in raw_rows:
        def count_cases(value: Any) -> int | str:
            if isinstance(value, list):
                return len(value)
            if isinstance(value, str):
                try:
                    loaded = json.loads(value)
                except Exception:
                    return ""
                return len(loaded) if isinstance(loaded, list) else ""
            return ""

        f2p_count = count_cases(row.get("FAIL_TO_PASS"))
        p2p_count = count_cases(row.get("PASS_TO_PASS"))
        out.append(
            {
                "benchmark": dataset,
                "split": split,
                "instance_id": row.get("instance_id", ""),
                "repo": row.get("repo", ""),
                "version": row.get("version", ""),
                "base_commit": row.get("base_commit", ""),
                "difficulty": row.get("difficulty", ""),
                "problem_title": first_line(row.get("problem_statement", ""), 260),
                "created_at": row.get("created_at", ""),
                "eval_type": row.get("eval_type", ""),
                "log_parser": row.get("log_parser", ""),
                "fail_to_pass_count": f2p_count,
                "pass_to_pass_count": p2p_count,
                "image": row.get("image", ""),
                "dataset_url": f"https://huggingface.co/datasets/{dataset}",
            }
        )
    return out


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    CACHE.mkdir(parents=True, exist_ok=True)

    terminal_rows = terminal_bench()
    write_csv(
        OUT / "terminal_bench_original_tasks.csv",
        terminal_rows,
        [
            "benchmark",
            "task_id",
            "path",
            "instruction",
            "difficulty",
            "category",
            "tags",
            "parser_name",
            "max_agent_timeout_sec",
            "max_test_timeout_sec",
            "source_url",
            "task_yaml_url",
        ],
    )
    write_jsonl(OUT / "terminal_bench_original_tasks.jsonl", terminal_rows)

    deepswe_rows = deepswe()
    write_csv(
        OUT / "deepswe_tasks.csv",
        deepswe_rows,
        [
            "benchmark",
            "task_id",
            "path",
            "display_title",
            "display_description",
            "original_title",
            "category",
            "language",
            "repository_url",
            "base_commit_hash",
            "instruction",
            "agent_timeout_sec",
            "verifier_timeout_sec",
            "source_url",
            "task_toml_url",
        ],
    )
    write_jsonl(OUT / "deepswe_tasks.jsonl", deepswe_rows)

    programbench_rows = programbench()
    write_csv(
        OUT / "programbench_tasks.csv",
        programbench_rows,
        [
            "benchmark",
            "task_id",
            "repository",
            "commit",
            "language",
            "difficulty",
            "source_url",
            "task_yaml_url",
            "tests_json_url",
        ],
    )
    write_jsonl(OUT / "programbench_tasks.jsonl", programbench_rows)

    swe_sets = [
        ("SWE-bench/SWE-bench", "test", "swe_bench_full_test"),
        ("SWE-bench/SWE-bench", "dev", "swe_bench_full_dev"),
        ("SWE-bench/SWE-bench_Verified", "test", "swe_bench_verified_test"),
        ("SWE-bench/SWE-bench_Lite", "test", "swe_bench_lite_test"),
        ("SWE-bench/SWE-bench_Lite", "dev", "swe_bench_lite_dev"),
    ]
    swe_fields = [
        "benchmark",
        "split",
        "instance_id",
        "repo",
        "version",
        "base_commit",
        "difficulty",
        "problem_title",
        "created_at",
        "eval_type",
        "log_parser",
        "fail_to_pass_count",
        "pass_to_pass_count",
        "image",
        "dataset_url",
    ]
    for dataset, split, stem in swe_sets:
        rows = swe_bench(dataset, split)
        write_csv(OUT / f"{stem}.csv", rows, swe_fields)
        write_jsonl(OUT / f"{stem}.jsonl", rows)

    summary = {
        "terminal_bench_original_tasks": len(terminal_rows),
        "deepswe_tasks": len(deepswe_rows),
        "programbench_tasks": len(programbench_rows),
    }
    for _, _, stem in swe_sets:
        path = OUT / f"{stem}.jsonl"
        summary[stem] = sum(1 for _ in path.open(encoding="utf-8"))
    (OUT / "counts.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
