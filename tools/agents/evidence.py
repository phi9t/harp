#!/usr/bin/env python3
"""Content-addressed evidence records for worker runs.

Records live under ``<main worktree>/.corenius/evidence`` by default, or under
``CORENIUS_EVIDENCE`` / an explicit ``out_dir``. The record digest is the sha256
of canonical JSON for the record without its ``digest`` field. Claim outcomes
are appended to a sidecar JSONL file and linked by a simple hash chain.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import secrets
import stat
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
from worker import WorkerError, git, main_worktree, now  # noqa: E402

SCHEMA = "corenius.evidence.v1"
KIND = "worker-run"
PREFIX = "sha256:"
REQUIRED_RUN_FILES = ("run.json", "prompt.md", "events.jsonl", "verdict.json", "last-message.md", "stderr.log")
RECOMPUTABLE_FIELDS = (
    "run_json_digest",
    "prompt_digest",
    "event_stream_digest",
    "verdict_json_digest",
    "last_message_digest",
    "stderr_digest",
    "base_sha",
    "worktree_diff_digest",
    "verdict",
    "timings",
)
CLAIM_KEYS = {"claim_id", "passed", "command", "output_digest", "recorded_at", "prev"}


def _canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def _sha256_bytes(data: bytes) -> str:
    return PREFIX + hashlib.sha256(data).hexdigest()


def _chain_hash(data: bytes | str) -> str:
    if isinstance(data, str):
        data = data.encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def _read_bytes(path: Path) -> bytes:
    try:
        return path.read_bytes()
    except OSError as error:
        raise WorkerError(f"{path} cannot be read: {error}") from error


def _read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text())
    except OSError as error:
        raise WorkerError(f"{path} cannot be read: {error}") from error
    except ValueError as error:
        raise WorkerError(f"{path} is not valid JSON: {error}") from error


def _file_digest(path: Path) -> str:
    return _sha256_bytes(_read_bytes(path))


def _field(value: Any, provenance: str, source: str) -> dict[str, Any]:
    return {"value": value, "provenance": provenance, "source": source}


def _normalize_digest(digest: str) -> tuple[str, str]:
    if digest.startswith(PREFIX):
        hex_part = digest[len(PREFIX):]
    else:
        hex_part = digest
        digest = PREFIX + digest
    if len(hex_part) != 64 or any(c not in "0123456789abcdef" for c in hex_part):
        raise WorkerError(f"bad digest {digest!r}: expected sha256:<64 lowercase hex>")
    return digest, hex_part


def _resolve_out_dir(repo: str | Path | None = None, out_dir: str | Path | None = None) -> Path:
    if out_dir is not None:
        return Path(out_dir)
    override = os.environ.get("CORENIUS_EVIDENCE")
    if override:
        return Path(override)
    target = Path(repo or ".")
    try:
        root = main_worktree(target)
    except WorkerError:
        root = target.resolve()
    return root / ".corenius" / "evidence"


def _record_path(digest: str, out_dir: str | Path | None = None) -> Path:
    _, hex_part = _normalize_digest(digest)
    return _resolve_out_dir(out_dir=out_dir) / f"{hex_part}.json"


def _claims_path(digest: str, out_dir: str | Path | None = None) -> Path:
    _, hex_part = _normalize_digest(digest)
    return _resolve_out_dir(out_dir=out_dir) / f"{hex_part}.claims.jsonl"


def _repo_for_run(run: dict[str, Any], repo: str | Path | None) -> Path:
    if repo is not None:
        return Path(repo)
    worktree = run.get("worktree")
    if worktree:
        return Path(worktree)
    return Path(".")


def _git_output(repo: Path, *args: str) -> str:
    try:
        return git(repo, *args)
    except WorkerError as error:
        raise WorkerError(str(error)) from error


def _git_bytes(repo: Path, *args: str) -> bytes:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True)
    if result.returncode != 0:
        command = " ".join(args)
        raise WorkerError(f"git {command} failed in {repo}: {result.stderr.decode('utf-8', 'replace').strip()}")
    return result.stdout


def _untracked_paths(repo: Path) -> list[bytes]:
    raw = _git_bytes(repo, "ls-files", "--others", "--exclude-standard", "-z")
    return sorted(path for path in raw.split(b"\0") if path)


def _update_digest_part(hasher: Any, label: bytes, data: bytes) -> None:
    hasher.update(label)
    hasher.update(len(data).to_bytes(8, "big"))
    hasher.update(data)


def _untracked_file_digest(repo: Path, path: bytes) -> tuple[bytes, str]:
    full_path = repo / os.fsdecode(path)
    try:
        mode = os.lstat(full_path).st_mode
    except OSError as error:
        raise WorkerError(f"untracked file {full_path} cannot be inspected: {error}") from error
    if stat.S_ISLNK(mode):
        try:
            return b"symlink", _sha256_bytes(os.fsencode(os.readlink(full_path)))
        except OSError as error:
            raise WorkerError(f"untracked symlink {full_path} cannot be read: {error}") from error
    if not stat.S_ISREG(mode):
        raise WorkerError(f"untracked path {full_path} is not a regular file or symlink")
    return b"file", _file_digest(full_path)


def _git_diff_digest(repo: Path, base_sha: str) -> str:
    diff = _git_bytes(repo, "diff", "--binary", base_sha)
    untracked = _untracked_paths(repo)
    if not untracked:
        return _sha256_bytes(diff)
    hasher = hashlib.sha256()
    hasher.update(diff)
    hasher.update(b"\0corenius.evidence.untracked.v1\0")
    for path in untracked:
        kind, content_digest = _untracked_file_digest(repo, path)
        _update_digest_part(hasher, b"path", path)
        _update_digest_part(hasher, b"kind", kind)
        _update_digest_part(hasher, b"sha256", content_digest.encode("ascii"))
    return PREFIX + hasher.hexdigest()


def _timings(run: dict[str, Any], verdict: dict[str, Any]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key in ("started", "started_epoch"):
        if key in run:
            result[key] = run[key]
    for key in ("finished", "elapsed_seconds"):
        if key in verdict:
            result[key] = verdict[key]
    return result


def _require_run_files(run_dir: Path) -> None:
    missing = [name for name in REQUIRED_RUN_FILES if not (run_dir / name).exists()]
    if missing:
        raise WorkerError(f"{run_dir} is not a complete worker run; missing {', '.join(missing)}")


def _run_inputs(run_dir: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    _require_run_files(run_dir)
    return _read_json(run_dir / "run.json"), _read_json(run_dir / "verdict.json")


def _declared_fields(run_dir: Path, repo: Path, run: dict[str, Any]) -> dict[str, dict[str, Any]]:
    fields = {
        "run_dir": _field(str(run_dir), "declared", "record_run RUN_DIR argument"),
        "repo": _field(str(repo), "declared", "record_run --repo or run.json worktree"),
    }
    for name in ("name", "model", "effort", "sandbox", "allowed", "branch", "worktree", "cap_seconds", "stall_seconds"):
        if name in run:
            field_name = "worker_name" if name == "name" else name
            fields[field_name] = _field(run[name], "declared", f"run.json {name}")
    if "base_sha" in run:
        fields["run_json_base_sha"] = _field(run["base_sha"], "declared", "run.json base_sha")
    return fields


def _measured_fields(run_dir: Path, repo: Path, run: dict[str, Any], verdict: dict[str, Any]) -> dict[str, dict[str, Any]]:
    base_sha = _git_output(repo, "rev-parse", "HEAD").strip()
    run_base = run.get("base_sha")
    diff_base = run_base if isinstance(run_base, str) and run_base else base_sha
    return {
        "run_json_digest": _field(_file_digest(run_dir / "run.json"), "measured", "sha256 of run.json"),
        "prompt_digest": _field(_file_digest(run_dir / "prompt.md"), "measured", "sha256 of prompt.md"),
        "event_stream_digest": _field(_file_digest(run_dir / "events.jsonl"), "measured", "sha256 of events.jsonl"),
        "verdict_json_digest": _field(_file_digest(run_dir / "verdict.json"), "measured", "sha256 of verdict.json"),
        "last_message_digest": _field(_file_digest(run_dir / "last-message.md"), "measured", "sha256 of last-message.md"),
        "stderr_digest": _field(_file_digest(run_dir / "stderr.log"), "measured", "sha256 of stderr.log"),
        "base_sha": _field(base_sha, "measured", f"git -C {repo} rev-parse HEAD"),
        "worktree_diff_digest": _field(
            _git_diff_digest(repo, diff_base),
            "measured",
            f"git -C {repo} diff --binary {diff_base}; git -C {repo} ls-files --others --exclude-standard",
        ),
        "verdict": _field(verdict.get("verdict"), "measured", "verdict.json verdict"),
        "timings": _field(_timings(run, verdict), "measured", "run.json and verdict.json timing fields"),
    }


def _build_fields(run_dir: Path, repo: Path) -> dict[str, dict[str, Any]]:
    run, verdict = _run_inputs(run_dir)
    fields = _declared_fields(run_dir, repo, run)
    fields.update(_measured_fields(run_dir, repo, run, verdict))
    fields["nonce"] = _field(secrets.token_hex(16), "measured", "record_run nonce")
    return fields


def _record_digest(record: dict[str, Any]) -> str:
    without_digest = dict(record)
    without_digest.pop("digest", None)
    return _sha256_bytes(_canonical_bytes(without_digest))


def record_run(run_dir: str | Path, repo: str | Path | None = None, out_dir: str | Path | None = None) -> str:
    run_dir = Path(run_dir).resolve()
    run = _read_json(run_dir / "run.json")
    repo_path = _repo_for_run(run, repo).resolve()
    record = {
        "schema": SCHEMA,
        "kind": KIND,
        "recorded_at": now(),
        "fields": _build_fields(run_dir, repo_path),
    }
    digest = _record_digest(record)
    record["digest"] = digest
    evidence_dir = _resolve_out_dir(repo_path, out_dir)
    evidence_dir.mkdir(parents=True, exist_ok=True)
    _, hex_part = _normalize_digest(digest)
    path = evidence_dir / f"{hex_part}.json"
    if path.exists():
        raise WorkerError(f"evidence record already exists: {path}")
    path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
    return digest


def load(digest: str, out_dir: str | Path | None = None) -> dict[str, Any]:
    path = _record_path(digest, out_dir)
    try:
        return json.loads(path.read_text())
    except OSError as error:
        raise WorkerError(f"evidence record {path} cannot be read: {error}") from error
    except ValueError as error:
        raise WorkerError(f"evidence record {path} is not valid JSON: {error}") from error


def _value(record: dict[str, Any], field: str) -> Any:
    try:
        return record["fields"][field]["value"]
    except KeyError as error:
        raise WorkerError(f"record has no field {field}") from error


def _verify_record_digest(record: dict[str, Any], requested: str) -> list[str]:
    problems = []
    computed = _record_digest(record)
    stored = record.get("digest")
    if stored != computed:
        problems.append(f"record digest mismatch: stored {stored!r}, computed {computed}")
    if stored != requested:
        problems.append(f"record digest mismatch: requested {requested}, record says {stored!r}")
    return problems


def _verify_field_shape(record: dict[str, Any]) -> list[str]:
    problems = []
    fields = record.get("fields")
    if not isinstance(fields, dict):
        return ["record fields missing or not an object"]
    for name, field in fields.items():
        if not isinstance(field, dict):
            problems.append(f"field {name} is not an object")
            continue
        extra = set(field) - {"value", "provenance", "source"}
        if extra:
            problems.append(f"field {name} has unexpected keys: {', '.join(sorted(extra))}")
        provenance = field.get("provenance")
        if provenance not in ("measured", "declared"):
            problems.append(f"field {name} has invalid provenance {provenance!r}")
        for key in ("value", "source"):
            if key not in field:
                problems.append(f"field {name} is missing {key}")
    return problems


def _verify_recomputable(record: dict[str, Any]) -> list[str]:
    problems: list[str] = []
    try:
        run_dir = Path(_value(record, "run_dir"))
        repo = Path(_value(record, "repo"))
        run, verdict = _run_inputs(run_dir)
        recomputed = _measured_fields(run_dir, repo, run, verdict)
    except WorkerError as error:
        return [f"recomputable input failed: {error}"]
    fields = record.get("fields", {})
    for name in RECOMPUTABLE_FIELDS:
        if name not in fields:
            problems.append(f"recomputable field missing: {name}")
            continue
        current = recomputed[name]["value"]
        recorded = fields[name].get("value") if isinstance(fields[name], dict) else None
        if recorded != current:
            problems.append(f"recomputable field mismatch: {name}")
    return problems


def _claim_lines(digest: str, out_dir: str | Path | None = None) -> list[bytes]:
    path = _claims_path(digest, out_dir)
    if not path.exists():
        return []
    return path.read_bytes().splitlines()


def _verify_claims(digest: str, out_dir: str | Path | None = None) -> list[str]:
    problems = []
    expected_prev = _chain_hash(digest)
    previous_raw: bytes | None = None
    for index, raw in enumerate(_claim_lines(digest, out_dir), start=1):
        try:
            item = json.loads(raw.decode("utf-8"))
        except ValueError as error:
            problems.append(f"claims chain broken at line {index}: invalid JSON: {error}")
            previous_raw = raw
            expected_prev = _chain_hash(raw)
            continue
        missing = CLAIM_KEYS - set(item)
        if missing:
            problems.append(f"claims chain broken at line {index}: missing {', '.join(sorted(missing))}")
        if item.get("prev") != expected_prev:
            problems.append(f"claims chain broken at line {index}: prev does not match")
        previous_raw = raw
        expected_prev = _chain_hash(previous_raw)
    return problems


def verify(digest: str, out_dir: str | Path | None = None) -> list[str]:
    requested, _ = _normalize_digest(digest)
    path = _record_path(requested, out_dir)
    problems: list[str] = []
    try:
        record = json.loads(path.read_text())
    except OSError as error:
        return [f"record missing or unreadable: {path}: {error}"]
    except ValueError as error:
        return [f"record JSON invalid: {path}: {error}"]
    problems.extend(_verify_field_shape(record))
    problems.extend(_verify_record_digest(record, requested))
    if record.get("schema") != SCHEMA:
        problems.append(f"record schema mismatch: {record.get('schema')!r}")
    if record.get("kind") != KIND:
        problems.append(f"record kind mismatch: {record.get('kind')!r}")
    problems.extend(_verify_recomputable(record))
    problems.extend(_verify_claims(requested, out_dir))
    return problems


def append_claim_outcome(
    digest: str,
    claim_id: str,
    passed: bool,
    command: str,
    output_digest: str,
    out_dir: str | Path | None = None,
) -> None:
    normalized, _ = _normalize_digest(digest)
    load(normalized, out_dir)
    claims = _claim_lines(normalized, out_dir)
    prev = _chain_hash(claims[-1]) if claims else _chain_hash(normalized)
    item = {
        "claim_id": claim_id,
        "passed": bool(passed),
        "command": command,
        "output_digest": output_digest,
        "recorded_at": now(),
        "prev": prev,
    }
    path = _claims_path(normalized, out_dir)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("ab") as handle:
        handle.write(_canonical_bytes(item) + b"\n")


def _claims(digest: str, out_dir: str | Path | None = None) -> list[Any]:
    claims = []
    for raw in _claim_lines(digest, out_dir):
        try:
            claims.append(json.loads(raw.decode("utf-8")))
        except ValueError:
            claims.append({"invalid": raw.decode("utf-8", "replace")})
    return claims


def _cmd_record(args: argparse.Namespace) -> int:
    print(record_run(args.run_dir, repo=args.repo))
    return 0


def _cmd_verify(args: argparse.Namespace) -> int:
    problems = verify(args.digest)
    for problem in problems:
        print(problem)
    return 1 if problems else 0


def _cmd_show(args: argparse.Namespace) -> int:
    record = load(args.digest)
    print(json.dumps({"record": record, "claims": _claims(args.digest)}, indent=2, sort_keys=True))
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("record")
    p.add_argument("run_dir")
    p.add_argument("--repo")
    p.set_defaults(func=_cmd_record)
    p = sub.add_parser("verify")
    p.add_argument("digest")
    p.set_defaults(func=_cmd_verify)
    p = sub.add_parser("show")
    p.add_argument("digest")
    p.set_defaults(func=_cmd_show)
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except WorkerError as error:
        print(f"evidence: {error}", file=sys.stderr)
        return 64


if __name__ == "__main__":
    raise SystemExit(main())
