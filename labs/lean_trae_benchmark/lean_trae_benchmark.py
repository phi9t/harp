from __future__ import annotations

import json
import os
import platform
import re
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from model import ValidationError, parse_json_object, sha256_bytes


ALLOWED_TOOLS = (
    "Read",
    "Glob",
    "Grep",
    "Write",
    "Edit",
    "Bash(lake env lean:*)",
    "Bash(lean:*)",
)
TRAE_VERSION = "0.200.19"
TRAE_DARWIN_ARM64_SHA256 = "2de3b4a458c219953a9eb044a4e07dcae18f150637d5d83874f38a4ae57bed57"
TRAE_VERSION_OUTPUT = re.compile(r"^traecli (0\.\d+\.\d+)(?:\([^\r\n]*\))?$")
CORE_DECLARATION = re.compile(
    r"^theorem core_and_left \(p q : Prop\) : p ∧ q → p := by\n",
)
AUTH_ENVIRONMENT_KEYS = (
    "HOME",
    "USER",
    "LOGNAME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "SSL_CERT_FILE",
    "CODEX_HOME",
)


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes()).hex


def inspect_cli(traecli: Path, *, expected_sha256: str | None = None) -> dict[str, str]:
    """Verify the immutable runner points at the expected TRAE executable."""
    try:
        metadata = traecli.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect TRAE executable: {error}") from error
    if not traecli.is_file() or traecli.is_symlink():
        raise ValidationError("TRAE executable must be a regular non-symlink file")
    if not metadata.st_mode & 0o111:
        raise ValidationError("TRAE executable is not executable")
    completed = subprocess.run(
        [str(traecli), "--version"],
        text=True,
        capture_output=True,
        check=False,
        timeout=30,
    )
    if completed.returncode != 0:
        raise ValidationError("TRAE --version command failed")
    match = TRAE_VERSION_OUTPUT.fullmatch(completed.stdout.strip())
    if match is None:
        raise ValidationError("TRAE --version output is not recognized")
    if match.group(1) != TRAE_VERSION:
        raise ValidationError(f"TRAE version must be {TRAE_VERSION}")
    digest = sha256_bytes(traecli.read_bytes()).hex
    if expected_sha256 is not None:
        if not re.fullmatch(r"[0-9a-f]{64}", expected_sha256):
            raise ValidationError("expected TRAE SHA-256 is invalid")
        if digest != expected_sha256:
            raise ValidationError("TRAE executable SHA-256 does not match its pin")
    return {
        "path": str(traecli),
        "version": TRAE_VERSION,
        "sha256": digest,
    }


def build_command(
    *,
    traecli: Path,
    workspace: Path,
    prompt_path: Path,
    final_message: Path,
    model: str = "gpt-5.6-sol",
) -> list[str]:
    """Build the isolated real-TRAE invocation for one sealed Lean task."""
    command = [
        str(traecli),
        "exec",
        "--ephemeral",
        "--ignore-user-config",
        "--ignore-rules",
        "--skip-git-repo-check",
        "--sandbox",
        "workspace-write",
        "--config",
        'approval_policy="never"',
        "--model",
        model,
        "--cd",
        str(workspace),
        "--output-last-message",
        str(final_message),
        "--json",
        "--color",
        "never",
    ]
    for tool in ALLOWED_TOOLS:
        command.extend(("--allowed-tool", tool))
    command.extend(("-",))
    return command


def run_attempt(
    *,
    runtime_root: Path,
    workspace: Path,
    prompt_path: Path,
    run_dir: Path,
    model: str = "gpt-5.6-sol",
    timeout: float = 2_400,
    lean_bin: Path | None = None,
    elan_home: Path | None = None,
) -> dict[str, object]:
    """Execute one real, isolated TRAE attempt and preserve its raw telemetry."""
    _require_regular_file(prompt_path, "prompt")
    _require_directory(workspace, "workspace")
    if run_dir.exists() or run_dir.is_symlink():
        raise ValidationError("refusing to overwrite an existing run directory")
    traecli = resolve_pinned_cli(runtime_root)
    cli_identity = inspect_cli(traecli, expected_sha256=_platform_cli_pin())
    _prepare_new_directory(run_dir)
    final_message = run_dir / "candidate.lean"
    events_path = run_dir / "events.jsonl"
    stderr_path = run_dir / "stderr.txt"
    command = build_command(
        traecli=traecli,
        workspace=workspace,
        prompt_path=prompt_path,
        final_message=final_message,
        model=model,
    )
    started_at = _utc_now()
    timed_out = False
    with _prepare_new_file(events_path).open("x") as events, _prepare_new_file(stderr_path).open("x") as stderr:
        try:
            completed = subprocess.run(
                command,
                input=prompt_path.read_text(encoding="utf-8"),
                text=True,
                stdout=events,
                stderr=stderr,
                check=False,
                timeout=timeout,
                env=_execution_environment(lean_bin, elan_home),
            )
        except subprocess.TimeoutExpired:
            completed = None
            timed_out = True
    try:
        validate_trace_policy(events_path)
        policy_error: str | None = None
    except ValidationError as error:
        policy_error = str(error)
    if timed_out:
        terminal_state = "timed_out"
    elif policy_error is not None:
        terminal_state = "policy_violation"
    else:
        terminal_state = "completed" if completed.returncode == 0 else "agent_failure"
    metadata: dict[str, object] = {
        "schema_version": "harp-lean-trae-attempt/v1",
        "started_at_utc": started_at,
        "completed_at_utc": _utc_now(),
        "command": command,
        "cli": cli_identity,
        "model": model,
        "returncode": None if timed_out else completed.returncode,
        "terminal_state": terminal_state,
        "policy_error": policy_error,
        "prompt_sha256": sha256_file(prompt_path),
        "events_sha256": sha256_file(events_path),
        "stderr_sha256": sha256_file(stderr_path),
        "candidate_sha256": sha256_file(final_message) if final_message.exists() else None,
    }
    _write_json_new(run_dir / "attempt.json", metadata)
    return metadata


def export_receipt(run_dir: Path, receipt_path: Path) -> dict[str, object]:
    """Export only hash-linked public facts; the raw trace stays in ``run_dir``."""
    _require_directory(run_dir, "run directory")
    attempt = _read_json_object(run_dir / "attempt.json", "attempt metadata")
    events_path = run_dir / "events.jsonl"
    stderr_path = run_dir / "stderr.txt"
    candidate_path = run_dir / "candidate.lean"
    _require_regular_file(events_path, "events trace")
    _require_regular_file(stderr_path, "stderr trace")
    _require_regular_file(candidate_path, "candidate")
    if attempt.get("terminal_state") == "policy_violation":
        raise ValidationError("cannot export a receipt for a policy-violating attempt")
    validate_trace_policy(events_path)
    _verify_attempt_digests(attempt, events_path, stderr_path, candidate_path)
    receipt: dict[str, object] = {
        "schema_version": "harp-lean-trae-receipt/v1",
        "attempt_sha256": sha256_file(run_dir / "attempt.json"),
        "events_sha256": sha256_file(events_path),
        "stderr_sha256": sha256_file(stderr_path),
        "candidate_sha256": sha256_file(candidate_path),
        "returncode": attempt["returncode"],
        "tool_events": _trace_tool_names(events_path),
    }
    _write_json_new(receipt_path, receipt)
    return receipt


def verify_receipt(run_dir: Path, receipt_path: Path) -> dict[str, object]:
    _require_directory(run_dir, "run directory")
    receipt = _read_json_object(receipt_path, "receipt")
    expected_fields = {
        "schema_version",
        "attempt_sha256",
        "events_sha256",
        "stderr_sha256",
        "candidate_sha256",
        "returncode",
        "tool_events",
    }
    if set(receipt) != expected_fields:
        raise ValidationError("receipt fields are not exact")
    if receipt["schema_version"] != "harp-lean-trae-receipt/v1":
        raise ValidationError("receipt schema version is not recognized")
    attempt_path = run_dir / "attempt.json"
    events_path = run_dir / "events.jsonl"
    stderr_path = run_dir / "stderr.txt"
    candidate_path = run_dir / "candidate.lean"
    attempt = _read_json_object(attempt_path, "attempt metadata")
    if attempt.get("terminal_state") == "policy_violation":
        raise ValidationError("cannot verify a receipt for a policy-violating attempt")
    _verify_attempt_digests(attempt, events_path, stderr_path, candidate_path)
    actual = {
        "attempt_sha256": sha256_file(attempt_path),
        "events_sha256": sha256_file(events_path),
        "stderr_sha256": sha256_file(stderr_path),
        "candidate_sha256": sha256_file(candidate_path),
    }
    for field, digest in actual.items():
        if receipt[field] != digest:
            raise ValidationError(f"receipt {field} does not match local trace")
    if receipt["returncode"] != attempt["returncode"]:
        raise ValidationError("receipt return code does not match attempt")
    if receipt["tool_events"] != _trace_tool_names(events_path):
        raise ValidationError("receipt tool events do not match trace")
    return receipt


def compile_core_candidate(
    lean: Path,
    candidate: Path,
    *,
    timeout: float = 120,
    elan_home: Path | None = None,
) -> dict[str, object]:
    """Ask a newly created Lean process to check one Core-only candidate."""
    _require_executable_file(lean, "Lean compiler")
    validate_core_candidate(candidate)
    try:
        completed = subprocess.run(
            [str(lean), str(candidate)],
            text=True,
            capture_output=True,
            check=False,
            timeout=timeout,
            env=_lean_environment(elan_home),
        )
    except subprocess.TimeoutExpired as error:
        raise ValidationError(f"Lean compiler timed out after {timeout} seconds") from error
    return {
        "schema_version": "harp-lean-core-compile/v1",
        "command": [str(lean), str(candidate)],
        "candidate_sha256": sha256_file(candidate),
        "returncode": completed.returncode,
        "passed": completed.returncode == 0,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
    }


def validate_core_candidate(candidate: Path) -> None:
    """Bind Core acceptance to the one sealed declaration and no imported theory."""
    _require_regular_file(candidate, "candidate")
    source = candidate.read_text(encoding="utf-8")
    if "\nimport " in f"\n{source}" or source.startswith("import "):
        raise ValidationError("Core task candidates cannot import libraries")
    if not CORE_DECLARATION.match(source):
        raise ValidationError("Core task candidate must define the sealed core_and_left declaration")
    header = CORE_DECLARATION.match(source)
    assert header is not None
    body = source[header.end():]
    if any(line and not line[0].isspace() for line in body.splitlines()):
        raise ValidationError("Core task candidate must not contain another top-level declaration")
    if re.search(
        r"(?m)^\s*(?:private|protected|noncomputable|theorem|def|opaque|axiom|example|"
        r"inductive|structure|class|abbrev|instance|namespace|section|end|open|export|"
        r"macro|syntax|elab|scoped|attribute|set_option)\b",
        body,
    ):
        raise ValidationError("Core task candidate contains a prohibited Lean command")
    if re.search(r"\b(?:sorry|admit|axiom|unsafe)\b", source):
        raise ValidationError("Core task candidate contains a prohibited proof escape")


def provision_pinned_cli(source: Path, runtime_root: Path) -> Path:
    """Copy a verified TRAE artifact into the Harp-owned isolated runtime root."""
    inspect_cli(source, expected_sha256=_platform_cli_pin())
    _prepare_new_directory(runtime_root)
    target = _prepare_new_file(runtime_root / "traecli")
    try:
        shutil.copyfile(source, target)
        target.chmod(source.stat().st_mode & 0o777)
    except OSError as error:
        raise ValidationError(f"cannot provision pinned TRAE executable: {error}") from error
    inspect_cli(target, expected_sha256=_platform_cli_pin())
    return target


def resolve_pinned_cli(runtime_root: Path) -> Path:
    _require_directory(runtime_root, "TRAE runtime directory")
    target = runtime_root / "traecli"
    inspect_cli(target, expected_sha256=_platform_cli_pin())
    return target


def validate_trace_policy(events_path: Path) -> list[str]:
    """Reject shell activity outside the fixed Lean compiler allowlist."""
    _require_regular_file(events_path, "events trace")
    commands: list[str] = []
    for number, event in _trace_events(events_path):
        item = event.get("item")
        if not isinstance(item, dict) or item.get("type") != "command_execution":
            continue
        command = item.get("command")
        if not isinstance(command, str):
            raise ValidationError(f"event trace line {number} has an invalid command")
        if not re.fullmatch(r"(?:lean|lake env lean) [A-Za-z0-9_./-]+\.lean", command):
            raise ValidationError(f"event trace command is not allowlisted: {command!r}")
        commands.append(command)
    return commands


def _require_regular_file(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if path.is_symlink() or not path.is_file():
        raise ValidationError(f"{label} must be a regular non-symlink file")
    if metadata.st_size > 4 * 1024 * 1024:
        raise ValidationError(f"{label} exceeds the 4 MiB byte limit")


def _require_executable_file(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if path.is_symlink() or not path.is_file():
        raise ValidationError(f"{label} must be a regular non-symlink file")
    if not metadata.st_mode & 0o111:
        raise ValidationError(f"{label} must be executable")


def _require_directory(path: Path, label: str) -> None:
    if path.is_symlink() or not path.is_dir():
        raise ValidationError(f"{label} must be a real directory")


def _prepare_new_directory(path: Path) -> None:
    if path.exists() or path.is_symlink():
        raise ValidationError("refusing to overwrite an existing run directory")
    _ensure_real_directory(path.parent)
    path.mkdir()


def _prepare_new_file(path: Path) -> Path:
    if path.exists() or path.is_symlink():
        raise ValidationError(f"refusing to overwrite existing file {path}")
    _ensure_real_directory(path.parent)
    return path


def _ensure_real_directory(path: Path) -> None:
    if path.exists() or path.is_symlink():
        if path.is_symlink():
            raise ValidationError(f"refusing symlinked parent directory {path}")
        if not path.is_dir():
            raise ValidationError(f"parent path is not a directory: {path}")
        return
    _ensure_real_directory(path.parent)
    path.mkdir()


def _write_json_new(path: Path, value: dict[str, object]) -> None:
    target = _prepare_new_file(path)
    with target.open("x", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True)
        handle.write("\n")


def _utc_now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _execution_environment(
    lean_bin: Path | None,
    elan_home: Path | None,
) -> dict[str, str]:
    environment = {
        key: os.environ[key]
        for key in AUTH_ENVIRONMENT_KEYS
        if key in os.environ
    }
    path_entries = [str(lean_bin)] if lean_bin is not None else []
    path_entries.append(os.defpath)
    environment["PATH"] = ":".join(path_entries)
    if elan_home is not None:
        environment["ELAN_HOME"] = str(elan_home)
    return environment


def _platform_cli_pin() -> str:
    if platform.system() == "Darwin" and platform.machine() == "arm64":
        return TRAE_DARWIN_ARM64_SHA256
    raise ValidationError("no sealed TRAE executable pin is available for this platform")


def _lean_environment(elan_home: Path | None) -> dict[str, str]:
    environment = {"PATH": os.defpath}
    if "TMPDIR" in os.environ:
        environment["TMPDIR"] = os.environ["TMPDIR"]
    if elan_home is not None:
        environment["ELAN_HOME"] = str(elan_home)
    return environment


def _read_json_object(path: Path, label: str) -> dict[str, object]:
    _require_regular_file(path, label)
    try:
        return parse_json_object(path.read_bytes(), label)
    except (OSError, ValidationError) as error:
        raise ValidationError(f"cannot parse {label}: {error}") from error


def _verify_attempt_digests(
    attempt: dict[str, object],
    events_path: Path,
    stderr_path: Path,
    candidate_path: Path,
) -> None:
    for key, path in (
        ("events_sha256", events_path),
        ("stderr_sha256", stderr_path),
        ("candidate_sha256", candidate_path),
    ):
        if attempt.get(key) != sha256_file(path):
            raise ValidationError(f"{key.removesuffix('_sha256')} digest does not match attempt")


def _trace_tool_names(events_path: Path) -> list[str]:
    names: list[str] = []
    for number, event in _trace_events(events_path):
        tool = event.get("tool")
        if tool is not None:
            if not isinstance(tool, str) or not re.fullmatch(r"[A-Za-z][A-Za-z0-9_().:* -]{0,127}", tool):
                raise ValidationError(f"event trace line {number} has an invalid tool name")
            names.append(tool)
        item = event.get("item")
        if isinstance(item, dict) and item.get("type") == "command_execution":
            names.append("Bash")
    return names


def _trace_events(events_path: Path) -> list[tuple[int, dict[str, object]]]:
    events: list[tuple[int, dict[str, object]]] = []
    for number, line in enumerate(events_path.read_text(encoding="utf-8").splitlines(), 1):
        try:
            event = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValidationError(f"event trace line {number} is not JSON") from error
        if not isinstance(event, dict):
            raise ValidationError(f"event trace line {number} is not an object")
        events.append((number, event))
    return events
