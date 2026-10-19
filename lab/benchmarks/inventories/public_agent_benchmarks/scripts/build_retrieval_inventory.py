#!/usr/bin/env python3
"""Build public inventories for retrieval/professional-reasoning benchmarks.

The script is intentionally stdlib-only. It records the public task rows that
are directly fetchable without a platform credential, and labels mirror/sample
sources where the official full task source is not directly row-fetchable.
"""

from __future__ import annotations

import csv
import html
import json
import re
import subprocess
import time
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "retrieval_professional"
CACHE = Path(".scratch/public_agent_benchmarks/cache/retrieval_professional")
REPOS = CACHE / "repos"
USER_AGENT = "corenius-retrieval-benchmark-inventory/1.0"


SOURCE_REPOS = {
    "vals-ai/finance-agent-v2": "502aab6fdaa3fb9294905c7453f89882baa8d39b",
    "surge-ai/gdp-pdf": "7a72a514a6ab19c90babb00adc817e4ae86b9c1b",
    "harveyai/harvey-labs": "a15650fe74bb20b86f160aaabdad75b87a91b6e8",
}


def run(command: list[str]) -> None:
    subprocess.run(command, check=True)


def ensure_clone(url: str, dest: Path) -> None:
    owner_repo = url.removeprefix("https://github.com/").removesuffix(".git")
    commit = SOURCE_REPOS[owner_repo]
    if not (dest / ".git").exists():
        dest.parent.mkdir(parents=True, exist_ok=True)
        run(["git", "clone", "--no-checkout", url, str(dest)])
    run(["git", "-C", str(dest), "fetch", "--depth", "1", "origin", commit])
    run(["git", "-C", str(dest), "checkout", "--force", commit])


def fetch_text(url: str, retries: int = 4) -> str:
    last_error: Exception | None = None
    for attempt in range(retries):
        try:
            request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(request, timeout=60) as response:
                return response.read().decode("utf-8", "replace")
        except Exception as exc:  # pragma: no cover - scratch network helper
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


def first_line(text: str, limit: int = 360) -> str:
    line = " ".join(str(text or "").strip().split())
    if len(line) > limit:
        return line[: limit - 1].rstrip() + "..."
    return line


def repo_commit(repo_root: Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(repo_root), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def finance_agent_v2_public_questions() -> list[dict[str, Any]]:
    owner_repo = "vals-ai/finance-agent-v2"
    repo_root = REPOS / "finance-agent-v2"
    ensure_clone(f"https://github.com/{owner_repo}", repo_root)
    commit = repo_commit(repo_root)
    public_csv = repo_root / "data" / "public.csv"
    rows: list[dict[str, Any]] = []
    with public_csv.open(newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for index, row in enumerate(reader, start=1):
            rubric = json.loads(row.get("Rubric", "[]") or "[]")
            must_pass_count = 0
            severity_sum = 0.0
            for item in rubric:
                modifiers = item.get("modifiers", {}) if isinstance(item, dict) else {}
                if modifiers.get("category") == "must_pass":
                    must_pass_count += 1
                try:
                    severity_sum += float(modifiers.get("severity", 0))
                except (TypeError, ValueError):
                    pass
            rows.append(
                {
                    "benchmark": "finance-agent-v2-public",
                    "question_id": f"public-{index:03d}",
                    "question": row.get("Question", ""),
                    "question_preview": first_line(row.get("Question", ""), 420),
                    "question_type": row.get("Question Type", ""),
                    "expert_time_mins": row.get("Expert time (mins)", ""),
                    "rubric_count": len(rubric),
                    "must_pass_count": must_pass_count,
                    "severity_sum": severity_sum,
                    "rubric_json": json.dumps(rubric, ensure_ascii=False, sort_keys=True),
                    "source_url": f"https://github.com/{owner_repo}/blob/main/data/public.csv",
                    "questions_txt_url": f"https://github.com/{owner_repo}/blob/main/data/public.txt",
                    "source_commit": commit,
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


def gdp_rubric(row: dict[str, Any]) -> list[dict[str, Any]]:
    criteria: list[dict[str, Any]] = []
    for index in range(1, 40):
        criterion = row.get(f"rubric - {index}. criterion")
        if not criterion:
            continue
        criteria.append(
            {
                "index": index,
                "criterion": criterion,
                "criterion_type": row.get(f"rubric - {index}. criterion_type", ""),
                "criterion_severity": row.get(f"rubric - {index}. criterion_severity", ""),
                "criterion_implicitness": row.get(f"rubric - {index}. criterion_implicitness", ""),
                "criterion_subjectiveness": row.get(f"rubric - {index}. criterion_subjectiveness", ""),
                "criterion_failure_mode": row.get(f"rubric - {index}. criterion_failure_mode", ""),
            }
        )
    return criteria


def gdp_pdf_public_mirror_tasks() -> list[dict[str, Any]]:
    dataset = "cloudybean/GDP.pdf"
    raw_rows = hf_dataset_rows(dataset, "test")
    rows: list[dict[str, Any]] = []
    for index, row in enumerate(raw_rows, start=1):
        rubric = gdp_rubric(row)
        rows.append(
            {
                "benchmark": "gdp-pdf-public-huggingface-mirror",
                "row_index": index - 1,
                "task_id": row.get("task_id", ""),
                "task_response_id": row.get("task_response_id", ""),
                "worker_id": row.get("worker_id", ""),
                "domain": row.get("domain", ""),
                "prompt": row.get("prompt", ""),
                "prompt_preview": first_line(row.get("prompt", ""), 420),
                "pdf_path": row.get("pdf_path", ""),
                "rubric_count": len(rubric),
                "rubric_json": json.dumps(rubric, ensure_ascii=False, sort_keys=True),
                "dataset_url": f"https://huggingface.co/datasets/{dataset}",
                "source_note": (
                    "Public HuggingFace mirror. Artificial Analysis links an official "
                    "surgeai/GDP.pdf dataset, but unauthenticated row access returned 401."
                ),
            }
        )
    return rows


def parse_gdp_sample_yaml(text: str) -> dict[str, Any]:
    lines = text.splitlines()
    doc: dict[str, Any] = {"attachments": [], "rubrics": [], "metadata": {}}
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("id: "):
            doc["id"] = line.split(":", 1)[1].strip()
        elif line == "prompt: |":
            block: list[str] = []
            i += 1
            while i < len(lines) and lines[i].startswith("  "):
                block.append(lines[i][2:])
                i += 1
            doc["prompt"] = "\n".join(block).strip()
            continue
        elif line == "attachments:":
            values: list[str] = []
            i += 1
            while i < len(lines) and lines[i].startswith("  - "):
                values.append(lines[i][4:].strip())
                i += 1
            doc["attachments"] = values
            continue
        elif line == "rubrics:":
            rubrics: list[dict[str, str]] = []
            current: dict[str, str] | None = None
            i += 1
            while i < len(lines) and lines[i].startswith("  "):
                item = lines[i]
                if item.startswith("  - id: "):
                    if current is not None:
                        rubrics.append(current)
                    current = {"id": item.split(":", 1)[1].strip()}
                elif item.startswith("    criterion: ") and current is not None:
                    current["criterion"] = item.split(":", 1)[1].strip()
                i += 1
            if current is not None:
                rubrics.append(current)
            doc["rubrics"] = rubrics
            continue
        elif line == "metadata:":
            metadata: dict[str, str] = {}
            i += 1
            while i < len(lines) and lines[i].startswith("  "):
                if ":" in lines[i]:
                    key, value = lines[i].split(":", 1)
                    metadata[key.strip()] = value.strip()
                i += 1
            doc["metadata"] = metadata
            continue
        i += 1
    return doc


def gdp_pdf_official_sample_tasks() -> list[dict[str, Any]]:
    owner_repo = "surge-ai/gdp-pdf"
    repo_root = REPOS / "gdp-pdf"
    ensure_clone(f"https://github.com/{owner_repo}", repo_root)
    commit = repo_commit(repo_root)
    rows: list[dict[str, Any]] = []
    for task_file in sorted((repo_root / "task_packs" / "sample_pdf" / "tasks").glob("*.yaml")):
        task = parse_gdp_sample_yaml(task_file.read_text(encoding="utf-8"))
        rubrics = task.get("rubrics", [])
        attachments = task.get("attachments", [])
        metadata = task.get("metadata", {})
        rel = task_file.relative_to(repo_root).as_posix()
        rows.append(
            {
                "benchmark": "gdp-pdf-official-public-samples",
                "task_id": task.get("id", task_file.stem),
                "path": rel,
                "prompt": task.get("prompt", ""),
                "prompt_preview": first_line(task.get("prompt", ""), 420),
                "attachments": ";".join(attachments),
                "attachment_count": len(attachments),
                "rubric_count": len(rubrics),
                "rubric_json": json.dumps(rubrics, ensure_ascii=False, sort_keys=True),
                "metadata_json": json.dumps(metadata, ensure_ascii=False, sort_keys=True),
                "source_url": f"https://github.com/{owner_repo}/blob/main/{rel}",
                "source_commit": commit,
            }
        )
    return rows


def find_examples_payload(value: Any) -> list[dict[str, Any]] | None:
    if isinstance(value, dict):
        examples = value.get("examples")
        if isinstance(examples, list):
            return examples
        for child in value.values():
            found = find_examples_payload(child)
            if found is not None:
                return found
    elif isinstance(value, list):
        for child in value:
            found = find_examples_payload(child)
            if found is not None:
                return found
    return None


def artificial_analysis_gdp_examples() -> list[dict[str, Any]]:
    url = "https://artificialanalysis.ai/evaluations/gdp-pdf"
    cache_path = CACHE / "gdp-pdf.html"
    if cache_path.exists():
        page = cache_path.read_text(encoding="utf-8")
    else:
        page = fetch_text(url)
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        cache_path.write_text(page, encoding="utf-8")

    examples: list[dict[str, Any]] | None = None
    for match in re.finditer(r"self\.__next_f\.push\((\[.*?\])\)</script>", page, re.S):
        try:
            pushed = json.loads(match.group(1))
        except json.JSONDecodeError:
            continue
        if len(pushed) < 2 or not isinstance(pushed[1], str) or '"examples"' not in pushed[1]:
            continue
        payload_text = pushed[1].split(":", 1)[1]
        try:
            payload = json.loads(payload_text)
        except json.JSONDecodeError:
            continue
        examples = find_examples_payload(payload)
        if examples is not None:
            break
    if examples is None:
        raise RuntimeError("could not find GDP.pdf examples in Artificial Analysis page")

    rows: list[dict[str, Any]] = []
    for index, item in enumerate(examples, start=1):
        task = item.get("task", {})
        submission = item.get("submission", {})
        criteria = submission.get("criteria", [])
        passed_count = sum(1 for criterion in criteria if criterion.get("passed") is True)
        rows.append(
            {
                "benchmark": "gdp-pdf-artificial-analysis-public-examples",
                "example_number": index,
                "public_task_id": task.get("publicTaskId", ""),
                "title": task.get("title", ""),
                "domain": task.get("domain", ""),
                "prompt": task.get("prompt", ""),
                "prompt_preview": first_line(task.get("prompt", ""), 420),
                "pdf_url": task.get("pdfUrl", ""),
                "example_model_name": submission.get("modelName", ""),
                "example_all_pass": submission.get("allPass", ""),
                "example_mean_pass": submission.get("meanPass", ""),
                "criteria_count": len(criteria),
                "criteria_passed_count": passed_count,
                "criteria_json": json.dumps(criteria, ensure_ascii=False, sort_keys=True),
                "source_url": url,
            }
        )
    return rows


def list_task_documents(task_dir: Path, config: dict[str, Any]) -> tuple[str, list[Path]]:
    docs_dir = config.get("docs_dir")
    if isinstance(docs_dir, str) and docs_dir.strip():
        candidate = (task_dir / docs_dir).resolve()
        label = docs_dir
    else:
        candidate = task_dir / "documents"
        label = "documents"
    if not candidate.exists():
        return label, []
    return label, sorted(path for path in candidate.rglob("*") if path.is_file())


def deliverable_names(value: Any) -> list[str]:
    if isinstance(value, dict):
        return sorted(str(key) for key in value)
    if isinstance(value, list):
        return sorted(str(item) for item in value)
    return []


def harvey_lab_public_tasks() -> list[dict[str, Any]]:
    owner_repo = "harveyai/harvey-labs"
    repo_root = REPOS / "harvey-labs"
    ensure_clone(f"https://github.com/{owner_repo}", repo_root)
    commit = repo_commit(repo_root)
    rows: list[dict[str, Any]] = []
    for task_json in sorted((repo_root / "tasks").rglob("task.json")):
        task_dir = task_json.parent
        config = json.loads(task_json.read_text(encoding="utf-8"))
        relative_dir = task_dir.relative_to(repo_root).as_posix()
        task_id = relative_dir.removeprefix("tasks/")
        parts = task_id.split("/")
        practice_area = parts[0] if parts else ""
        scenario = parts[-2] if len(parts) >= 2 and parts[-1] == "task" else ""
        deliverables = deliverable_names(config.get("deliverables", {}))
        criteria = config.get("criteria", [])
        criterion_deliverables = sorted(
            {
                str(deliverable)
                for criterion in criteria
                for deliverable in criterion.get("deliverables", [])
                if isinstance(criterion, dict)
            }
        )
        docs_label, documents = list_task_documents(task_dir, config)
        extensions = sorted({path.suffix.lower() or "[no extension]" for path in documents})
        rows.append(
            {
                "benchmark": "harvey-lab-public-repo",
                "task_id": task_id,
                "declared_id": config.get("id", ""),
                "path": relative_dir,
                "practice_area": practice_area,
                "title": config.get("title", ""),
                "work_type": config.get("work_type", ""),
                "tags": ";".join(str(tag) for tag in config.get("tags", [])),
                "instructions": config.get("instructions", ""),
                "instruction_preview": first_line(config.get("instructions", ""), 420),
                "deliverable_count": len(deliverables),
                "deliverables": ";".join(deliverables),
                "criterion_count": len(criteria),
                "criterion_deliverable_count": len(criterion_deliverables),
                "document_count": len(documents),
                "documents_dir": docs_label,
                "document_extensions": ";".join(extensions),
                "schema_keys": ";".join(sorted(config.keys())),
                "source_url": f"https://github.com/{owner_repo}/tree/main/{relative_dir}",
                "task_json_url": f"https://github.com/{owner_repo}/blob/main/{relative_dir}/task.json",
                "source_commit": commit,
            }
        )
    return rows


def write_inventory(stem: str, rows: list[dict[str, Any]], fields: list[str]) -> None:
    write_csv(OUT / f"{stem}.csv", rows, fields)
    write_jsonl(OUT / f"{stem}.jsonl", rows)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)

    finance_rows = finance_agent_v2_public_questions()
    gdp_sample_rows = gdp_pdf_official_sample_tasks()
    gdp_mirror_rows = gdp_pdf_public_mirror_tasks()
    gdp_example_rows = artificial_analysis_gdp_examples()
    harvey_rows = harvey_lab_public_tasks()

    write_inventory(
        "finance_agent_v2_public_questions",
        finance_rows,
        [
            "benchmark",
            "question_id",
            "question",
            "question_preview",
            "question_type",
            "expert_time_mins",
            "rubric_count",
            "must_pass_count",
            "severity_sum",
            "rubric_json",
            "source_url",
            "questions_txt_url",
            "source_commit",
        ],
    )
    write_inventory(
        "gdp_pdf_official_public_samples",
        gdp_sample_rows,
        [
            "benchmark",
            "task_id",
            "path",
            "prompt",
            "prompt_preview",
            "attachments",
            "attachment_count",
            "rubric_count",
            "rubric_json",
            "metadata_json",
            "source_url",
            "source_commit",
        ],
    )
    write_inventory(
        "gdp_pdf_public_mirror_tasks",
        gdp_mirror_rows,
        [
            "benchmark",
            "row_index",
            "task_id",
            "task_response_id",
            "worker_id",
            "domain",
            "prompt",
            "prompt_preview",
            "pdf_path",
            "rubric_count",
            "rubric_json",
            "dataset_url",
            "source_note",
        ],
    )
    write_inventory(
        "gdp_pdf_artificial_analysis_examples",
        gdp_example_rows,
        [
            "benchmark",
            "example_number",
            "public_task_id",
            "title",
            "domain",
            "prompt",
            "prompt_preview",
            "pdf_url",
            "example_model_name",
            "example_all_pass",
            "example_mean_pass",
            "criteria_count",
            "criteria_passed_count",
            "criteria_json",
            "source_url",
        ],
    )
    write_inventory(
        "harvey_lab_public_tasks",
        harvey_rows,
        [
            "benchmark",
            "task_id",
            "declared_id",
            "path",
            "practice_area",
            "title",
            "work_type",
            "tags",
            "instructions",
            "instruction_preview",
            "deliverable_count",
            "deliverables",
            "criterion_count",
            "criterion_deliverable_count",
            "document_count",
            "documents_dir",
            "document_extensions",
            "schema_keys",
            "source_url",
            "task_json_url",
            "source_commit",
        ],
    )

    counts = {
        "finance_agent_v2_public_questions": len(finance_rows),
        "gdp_pdf_artificial_analysis_examples": len(gdp_example_rows),
        "gdp_pdf_official_public_samples": len(gdp_sample_rows),
        "gdp_pdf_public_mirror_tasks": len(gdp_mirror_rows),
        "harvey_lab_public_tasks": len(harvey_rows),
    }
    (OUT / "counts.json").write_text(json.dumps(counts, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
