from __future__ import annotations

import ast
import gzip
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Iterable, Mapping, Sequence


FINAL_SCHEMA_VERSION = "harp-meta-harness-trae-final/v1"
ALLOWED_CANDIDATE_FIELDS = frozenset(
    {"name", "path", "axis", "hypothesis", "components", "base_system"}
)
REQUIRED_CANDIDATE_FIELDS = frozenset({"name", "path", "axis", "hypothesis"})
ALLOWED_TOOLS = ("Read", "Glob", "Grep", "Bash", "Write", "Edit")
SNAKE_CASE = re.compile(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")
SECRET_PATTERNS = (
    re.compile(
        rb"""(?ix)
        ["']?authorization["']?\s*[:=]\s*["']?bearer\s+
        [A-Za-z0-9._~+/=-]{8,}
        """
    ),
    re.compile(rb"(?i)(?:api[_-]?key|access[_-]?token|secret[_-]?key)\s*[:=]\s*[\"'][^\"'\r\n]{8,}"),
    re.compile(rb"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
    re.compile(rb"\bsk-[A-Za-z0-9_-]{16,}\b"),
    re.compile(rb"\bAKIA[0-9A-Z]{16}\b"),
)


class ValidationError(ValueError):
    pass


def build_command(
    workspace: Path,
    schema: Path,
    final_message: Path,
    prompt: str = "-",
) -> list[str]:
    command = [
        "traecli",
        "exec",
        "--ignore-user-config",
        "--ignore-rules",
        "--ephemeral",
        "--sandbox",
        "workspace-write",
        "--config",
        'approval_policy="never"',
        "--model",
        "gpt-5.4",
        "--cd",
        str(workspace),
        "--output-schema",
        str(schema),
        "--output-last-message",
        str(final_message),
        "--json",
    ]
    for tool in ALLOWED_TOOLS:
        command.extend(["--allowed-tool", tool])
    command.append(prompt)
    return command


def run_proposer(
    workspace: Path,
    schema: Path,
    final_message: Path,
    *,
    prompt: str = "-",
    prompt_text: str | None = None,
    timeout: float = 2400,
    executor: Callable[..., Any] = subprocess.run,
) -> Any:
    command = build_command(workspace, schema, final_message, prompt)
    completed = executor(
        command,
        input=prompt_text,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
        env=_minimal_environment(),
    )
    completed.args = command
    return completed


def validate_proposal(
    final_response: Mapping[str, Any],
    pending_eval: Mapping[str, Any],
) -> list[dict[str, Any]]:
    if final_response.get("schema_version") != FINAL_SCHEMA_VERSION:
        raise ValidationError(f"schema_version must be {FINAL_SCHEMA_VERSION}")
    if set(final_response) != {"schema_version", "candidates"}:
        raise ValidationError("final response contains fields outside the strict schema")
    if set(pending_eval) != {"candidates"}:
        raise ValidationError("pending_eval contains fields outside the strict schema")
    final_candidates = _validate_candidate_list(final_response.get("candidates"))
    pending_candidates = _validate_candidate_list(pending_eval.get("candidates"))
    if final_candidates != pending_candidates:
        raise ValidationError("final response and pending_eval candidates must match exactly")
    return final_candidates


def validate_workspace(
    workspace: Path,
    candidates: Sequence[Mapping[str, Any]],
    expected_reference_digests: Mapping[str, str] | None = None,
    *,
    interface_timeout: float = 5.0,
) -> dict[str, Any]:
    report = assess_workspace(
        workspace,
        candidates,
        expected_reference_digests,
        interface_timeout=interface_timeout,
    )
    invalid = [
        candidate for candidate in report["candidates"] if not candidate["valid"]
    ]
    if invalid:
        first = invalid[0]
        raise ValidationError(
            f"candidate {first['name']} {first['error']}"
        )
    return report


def assess_workspace(
    workspace: Path,
    candidates: Sequence[Mapping[str, Any]],
    expected_reference_digests: Mapping[str, str] | None = None,
    *,
    interface_timeout: float = 5.0,
) -> dict[str, Any]:
    workspace = workspace.resolve()
    validated_candidates = _validate_candidate_list(list(candidates))
    expected_writes = {
        PurePosixPath(candidate["path"]).as_posix() for candidate in validated_candidates
    }
    expected_writes.add("logs/pending_eval.json")
    actual_writes = set()
    for relative_root in ("agents", "logs", "reference"):
        if not (workspace / relative_root).is_dir():
            raise ValidationError(f"missing required workspace directory {relative_root}")
    immutable_inputs = set((expected_reference_digests or {}).keys())
    for path in workspace.rglob("*"):
        relative = path.relative_to(workspace).as_posix()
        metadata = path.lstat()
        if stat_is_symlink(metadata.st_mode):
            raise ValidationError(f"workspace cannot contain symlink {relative}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise ValidationError(f"workspace entry is not a regular file {relative}")
        if relative in expected_writes:
            actual_writes.add(relative)
        elif relative.startswith("reference/") or relative in immutable_inputs:
            continue
        else:
            raise ValidationError(f"undeclared workspace write: {relative}")
    undeclared = sorted(actual_writes - expected_writes)
    missing = sorted(expected_writes - actual_writes)
    if undeclared:
        raise ValidationError(f"undeclared workspace writes: {undeclared}")
    if missing:
        raise ValidationError(f"declared workspace writes are missing: {missing}")
    _verify_reference_digests(workspace, expected_reference_digests or {})

    pending_eval = _read_json(workspace / "logs/pending_eval.json", "pending_eval")
    validate_proposal(
        {"schema_version": FINAL_SCHEMA_VERSION, "candidates": validated_candidates},
        pending_eval,
    )
    reports: list[dict[str, Any]] = []
    for candidate in validated_candidates:
        try:
            candidate_report = _validate_candidate_interface(
                workspace,
                candidate,
                timeout=interface_timeout,
            )
        except ValidationError as error:
            reports.append(
                {
                    "name": candidate["name"],
                    "path": candidate["path"],
                    "sha256": _sha256_bytes(
                        (workspace / candidate["path"]).read_bytes()
                    ),
                    "valid": False,
                    "error": str(error),
                }
            )
        else:
            candidate_report["valid"] = True
            candidate_report["error"] = None
            reports.append(candidate_report)
    workspace_files = [
        path.relative_to(workspace).as_posix()
        for path in sorted(workspace.rglob("*"))
        if path.is_file()
    ]
    valid_count = sum(1 for candidate in reports if candidate["valid"])
    return {
        "schema_version": "harp-meta-harness-trae-validation/v1",
        "candidate_count": len(reports),
        "valid_candidate_count": valid_count,
        "candidates": reports,
        "interface_checks_passed": valid_count == len(reports),
        "benchmark_invoked": False,
        "held_out_test_invoked": False,
        "workspace_boundary_passed": True,
        "workspace_files": workspace_files,
    }


def create_deterministic_archive(
    source: Path,
    archive_path: Path,
    inventory_path: Path,
) -> None:
    source = source.resolve()
    files = _regular_tree_files(source)
    inventory_rows = []
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    with archive_path.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.PAX_FORMAT) as archive:
                for path in files:
                    relative = path.relative_to(source).as_posix()
                    data = path.read_bytes()
                    info = tarfile.TarInfo(relative)
                    info.size = len(data)
                    info.mode = 0o644
                    info.mtime = 0
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    info.pax_headers = {}
                    archive.addfile(info, fileobj=BytesReader(data))
                    inventory_rows.append((relative, len(data), _sha256_bytes(data)))
    inventory_path.parent.mkdir(parents=True, exist_ok=True)
    inventory_path.write_text(
        "path\tbytes\tsha256\n"
        + "".join(
            f"{path}\t{size}\t{digest}\n"
            for path, size, digest in inventory_rows
        )
    )


def safe_extract_archive(
    archive_path: Path,
    inventory_path: Path,
    output: Path,
    *,
    max_decompressed_bytes: int = 64 * 1024 * 1024,
) -> None:
    expected = _read_member_inventory(inventory_path)
    total = 0
    seen: dict[str, tuple[int, str]] = {}
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="harp-trae-unpack-", dir=output.parent) as temporary:
        staging = Path(temporary)
        with tarfile.open(archive_path, mode="r:gz") as archive:
            for member in archive:
                relative = _safe_posix_path(member.name, "archive member")
                if not member.isfile():
                    raise ValidationError("archive may contain regular files only")
                total += member.size
                if total > max_decompressed_bytes:
                    raise ValidationError("archive decompressed size exceeds limit")
                extracted = archive.extractfile(member)
                if extracted is None:
                    raise ValidationError(f"archive member could not be read: {relative}")
                data = extracted.read(max_decompressed_bytes + 1)
                if len(data) != member.size:
                    raise ValidationError(f"archive member size mismatch: {relative}")
                digest = _sha256_bytes(data)
                seen[relative] = (len(data), digest)
                destination = staging / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(data)
        if seen != expected:
            raise ValidationError("archive member digest inventory mismatch")
        if output.exists():
            if output.is_symlink() or not output.is_dir():
                raise ValidationError("archive output must be a real directory")
            shutil.rmtree(output)
        shutil.copytree(staging, output)


def scan_for_secrets(paths: Iterable[Path]) -> None:
    for path in paths:
        metadata = path.lstat()
        if stat_is_symlink(metadata.st_mode) or not path.is_file():
            raise ValidationError(f"secret-scan input must be a regular file: {path}")
        data = path.read_bytes()
        if any(pattern.search(data) for pattern in SECRET_PATTERNS):
            raise ValidationError(f"sensitive material found in {path}")


def audit_event_log(path: Path) -> None:
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValidationError(f"event log line {line_number} is not JSON") from error
        for command in _tool_commands(event):
            lowered = command.lower()
            if (
                "benchmark.py" in lowered
                or re.search(r"(?:^|\s)--test(?:\s|$)", lowered)
                or "held-out" in lowered
            ):
                raise ValidationError(
                    f"event log contains a benchmark or held-out-test command on line {line_number}"
                )
            if re.search(
                r"(?:^|[;&|]\s*|\s)(?:curl|wget|nc|ncat|telnet|ssh|scp|rsync)\s",
                lowered,
            ) or re.search(r"\bgit\s+(?:clone|fetch|pull|ls-remote)\b", lowered):
                raise ValidationError(
                    f"event log contains a network shell command on line {line_number}"
                )


def _validate_candidate_list(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list) or len(value) != 3:
        raise ValidationError("proposal must contain exactly three candidates")
    candidates = []
    names = set()
    for index, raw in enumerate(value):
        if not isinstance(raw, dict):
            raise ValidationError(f"candidate {index + 1} must be an object")
        fields = set(raw)
        if not REQUIRED_CANDIDATE_FIELDS.issubset(fields):
            raise ValidationError(f"candidate {index + 1} is missing required fields")
        if not fields.issubset(ALLOWED_CANDIDATE_FIELDS):
            raise ValidationError(f"candidate {index + 1} has fields outside allowed fields")
        if not all(isinstance(raw[field], str) for field in REQUIRED_CANDIDATE_FIELDS):
            raise ValidationError(f"candidate {index + 1} fields must be strings")
        name = raw["name"]
        if not SNAKE_CASE.fullmatch(name):
            raise ValidationError(f"candidate {index + 1} name must be snake_case")
        if name in names:
            raise ValidationError("candidate names must be unique")
        names.add(name)
        candidate_path = _safe_posix_path(raw["path"], "candidate path")
        if candidate_path != f"agents/{name}.py":
            raise ValidationError("candidate name requires a matching path")
        if not raw["axis"].strip() or not raw["hypothesis"].strip():
            raise ValidationError("candidate axis and hypothesis must be nonempty")
        if "components" in raw and (
            not isinstance(raw["components"], list)
            or not raw["components"]
            or not all(isinstance(item, str) and item.strip() for item in raw["components"])
        ):
            raise ValidationError("candidate components must be nonempty strings")
        if "base_system" in raw and (
            not isinstance(raw["base_system"], str) or not raw["base_system"].strip()
        ):
            raise ValidationError("candidate base_system must be a nonempty string")
        candidates.append(dict(raw))
    return candidates


def _validate_candidate_interface(
    workspace: Path,
    candidate: Mapping[str, Any],
    *,
    timeout: float,
) -> dict[str, Any]:
    path = workspace / candidate["path"]
    source = path.read_text()
    try:
        tree = ast.parse(source, filename=str(path))
    except SyntaxError as error:
        raise ValidationError(f"candidate syntax failure for {candidate['name']}: {error}") from error
    classes = [
        node
        for node in tree.body
        if isinstance(node, ast.ClassDef)
        and any(
            (isinstance(base, ast.Name) and base.id == "MemorySystem")
            or (isinstance(base, ast.Attribute) and base.attr == "MemorySystem")
            for base in node.bases
        )
    ]
    if len(classes) != 1:
        raise ValidationError(
            f"candidate {candidate['name']} must define one MemorySystem subclass"
        )
    script = r"""
import importlib.util
import json
import pathlib
import sys
import types
from typing import Protocol

workspace = pathlib.Path(sys.argv[1])
candidate = pathlib.Path(sys.argv[2])
package_root = workspace / "reference" / "text_classification"
package = types.ModuleType("text_classification")
package.__path__ = [str(package_root)]
sys.modules["text_classification"] = package
llm_module = types.ModuleType("text_classification.llm")
class LLMCallable(Protocol):
    def __call__(self, prompt: str) -> str: ...
llm_module.LLMCallable = LLMCallable
sys.modules["text_classification.llm"] = llm_module
memory_spec = importlib.util.spec_from_file_location(
    "text_classification.memory_system",
    package_root / "memory_system.py",
)
memory_module = importlib.util.module_from_spec(memory_spec)
sys.modules["text_classification.memory_system"] = memory_module
memory_spec.loader.exec_module(memory_module)
spec = importlib.util.spec_from_file_location("candidate_module", candidate)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
from text_classification.memory_system import MemorySystem
classes = [
    value for value in vars(module).values()
    if isinstance(value, type) and issubclass(value, MemorySystem) and value is not MemorySystem
]
if len(classes) != 1:
    raise RuntimeError(f"expected one MemorySystem subclass, found {len(classes)}")
calls = []
def llm(prompt):
    calls.append(prompt)
    return '{"final_answer":"ok"}'
memory = classes[0](llm=llm)
prediction, metadata = memory.predict("synthetic input")
assert isinstance(prediction, str)
assert isinstance(metadata, dict)
batch = [{
    "input": "synthetic input",
    "prediction": prediction,
    "ground_truth": "synthetic truth",
    "was_correct": True,
    "metadata": metadata,
}]
memory.learn_from_batch(batch)
state = memory.get_state()
assert isinstance(state, str)
restored = classes[0](llm=llm)
restored.set_state(state)
assert restored.get_state() == state
print(json.dumps({"calls": len(calls), "state_sha256": __import__("hashlib").sha256(state.encode()).hexdigest()}))
"""
    try:
        environment = _minimal_environment()
        environment["PYTHONDONTWRITEBYTECODE"] = "1"
        completed = subprocess.run(
            [sys.executable, "-I", "-B", "-c", script, str(workspace), str(path)],
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
            env=environment,
        )
    except subprocess.TimeoutExpired as error:
        raise ValidationError(f"candidate {candidate['name']} interface check timed out") from error
    if completed.returncode != 0:
        error_text = completed.stderr.strip().replace(str(workspace), "<workspace>")
        raise ValidationError(
            f"candidate {candidate['name']} import or interface failure: "
            f"{error_text[:500]}"
        )
    try:
        report = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise ValidationError(f"candidate {candidate['name']} returned malformed validation output") from error
    return {
        "name": candidate["name"],
        "path": candidate["path"],
        "sha256": _sha256_bytes(path.read_bytes()),
        "interface": report,
    }


def _verify_reference_digests(workspace: Path, expected: Mapping[str, str]) -> None:
    if expected:
        actual = {
            path.relative_to(workspace).as_posix()
            for path in (workspace / "reference").rglob("*")
            if path.is_file()
        }
        if actual != set(expected):
            raise ValidationError("reference file set differs from the declared immutable inputs")
    for raw_path, digest in expected.items():
        relative = _safe_posix_path(raw_path, "reference digest path")
        if not SHA256.fullmatch(digest):
            raise ValidationError(f"invalid reference digest for {relative}")
        path = workspace / relative
        if path.is_symlink() or not path.is_file():
            raise ValidationError(f"reference digest target is not a regular file: {relative}")
        if _sha256_bytes(path.read_bytes()) != digest:
            raise ValidationError(f"reference digest mismatch: {relative}")


def _regular_tree_files(root: Path) -> list[Path]:
    if root.is_symlink() or not root.is_dir():
        raise ValidationError("archive source must be a real directory")
    files = []
    for path in root.rglob("*"):
        metadata = path.lstat()
        if stat_is_symlink(metadata.st_mode):
            raise ValidationError(f"archive source cannot contain symlink {path}")
        if path.is_file():
            files.append(path)
        elif not path.is_dir():
            raise ValidationError(f"archive source contains non-regular entry {path}")
    return sorted(files, key=lambda path: path.relative_to(root).as_posix())


def _read_member_inventory(path: Path) -> dict[str, tuple[int, str]]:
    lines = path.read_text().splitlines()
    if not lines or lines[0] != "path\tbytes\tsha256":
        raise ValidationError("archive member inventory has an invalid header")
    rows = {}
    for line_number, line in enumerate(lines[1:], 2):
        cells = line.split("\t")
        if len(cells) != 3:
            raise ValidationError(f"archive member inventory line {line_number} is malformed")
        relative = _safe_posix_path(cells[0], "archive inventory path")
        try:
            size = int(cells[1])
        except ValueError as error:
            raise ValidationError(f"archive member inventory line {line_number} has invalid size") from error
        if size < 0 or not SHA256.fullmatch(cells[2]):
            raise ValidationError(f"archive member inventory line {line_number} is invalid")
        if relative in rows:
            raise ValidationError(f"duplicate archive inventory path {relative}")
        rows[relative] = (size, cells[2])
    return rows


def _read_json(path: Path, label: str) -> Mapping[str, Any]:
    try:
        value = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise ValidationError(f"{label} is not readable JSON: {error}") from error
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be a JSON object")
    return value


def _safe_posix_path(value: str, label: str) -> str:
    if not isinstance(value, str):
        raise ValidationError(f"{label} must be a string")
    path = PurePosixPath(value)
    if (
        not value
        or path.is_absolute()
        or "\\" in value
        or any(part in ("", ".", "..") for part in path.parts)
    ):
        raise ValidationError(f"{label} must be a normal relative path")
    return path.as_posix()


def _tool_commands(value: Any, *, in_tool_call: bool = False) -> list[str]:
    commands = []
    if isinstance(value, dict):
        names = [
            item
            for key in ("tool_name", "name", "function", "type")
            if isinstance((item := value.get(key)), str)
        ]
        is_tool_call = in_tool_call or any(
            token in name.lower()
            for name in names
            for token in (
                "tool",
                "bash",
                "shell",
                "exec_command",
                "command_execution",
                "function_call",
            )
        )
        if is_tool_call:
            for key in ("command", "cmd"):
                if isinstance(value.get(key), str):
                    commands.append(value[key])
        for child in value.values():
            commands.extend(_tool_commands(child, in_tool_call=is_tool_call))
    elif isinstance(value, list):
        for child in value:
            commands.extend(_tool_commands(child, in_tool_call=in_tool_call))
    elif in_tool_call and isinstance(value, str):
        try:
            decoded = json.loads(value)
        except json.JSONDecodeError:
            pass
        else:
            commands.extend(_tool_commands(decoded, in_tool_call=True))
    return commands


def _minimal_environment() -> dict[str, str]:
    allowed = ("HOME", "PATH", "TMPDIR", "LANG", "LC_ALL", "SSL_CERT_FILE", "SSL_CERT_DIR")
    return {key: os.environ[key] for key in allowed if key in os.environ}


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def stat_is_symlink(mode: int) -> bool:
    import stat

    return stat.S_ISLNK(mode)


class BytesReader:
    def __init__(self, data: bytes):
        self._data = data
        self._offset = 0

    def read(self, size: int = -1) -> bytes:
        if size < 0:
            size = len(self._data) - self._offset
        chunk = self._data[self._offset : self._offset + size]
        self._offset += len(chunk)
        return chunk
