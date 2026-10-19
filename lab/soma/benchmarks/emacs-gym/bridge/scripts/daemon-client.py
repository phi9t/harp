#!/usr/bin/env python3
"""External JSONL client for the live Emacs daemon bridge benchmark."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shlex
import socket
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from typing import Any


def read_json_file(path: pathlib.Path | None) -> Any:
    if path is None or not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: pathlib.Path, payload: Any) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def check(name: str, passed: bool, detail: Any) -> dict[str, Any]:
    return {"name": name, "passed": bool(passed), "detail": detail}


def all_checks_pass(checks: list[dict[str, Any]]) -> bool:
    return bool(checks) and all(item.get("passed") is True for item in checks)


def find_manifest_operation(manifest: dict[str, Any], operation: str) -> dict[str, Any] | None:
    for entry in manifest.get("operations") or []:
        if entry.get("operation") == operation:
            return entry
    return None


def manifest_operation_is_stable(manifest: dict[str, Any], operation: str) -> bool:
    entry = find_manifest_operation(manifest, operation) or {}
    return (
        entry.get("maturity") == "stable"
        and entry.get("implemented") is True
        and entry.get("enabled") is True
    )


def response_like(value: Any) -> bool:
    return (
        isinstance(value, dict)
        and "request_id" in value
        and "operation" in value
        and "state" in value
        and "state_sequence" in value
    )


def response_slot(key: str | None) -> bool:
    return (
        isinstance(key, str)
        and (
            key == "response"
            or key.endswith("_response")
            or key in {"status", "target_status"}
        )
    )


def response_contract_issues(
    manifest: dict[str, Any],
    path: str,
    response: dict[str, Any],
) -> list[dict[str, Any]]:
    if response.get("client_side") is True:
        return []

    operation = response.get("operation")
    state = response.get("state")
    operation_spec = (
        find_manifest_operation(manifest, operation)
        if isinstance(operation, str)
        else None
    )
    result_contract = (operation_spec or {}).get("result_contract") or {}
    required_fields = result_contract.get("required_fields") or []
    allowed_states = set(result_contract.get("states") or [])
    allowed_states.update(result_contract.get("terminal_states") or [])
    allowed_sequence_states = set(allowed_states)
    allowed_sequence_states.update({"accepted", "running", "waiting_for_input"})
    issues: list[dict[str, Any]] = []

    def issue(code: str, message: str, detail: Any = None) -> None:
        issues.append(
            {
                "path": path,
                "request_id": response.get("request_id"),
                "operation": operation,
                "state": state,
                "code": code,
                "message": message,
                "detail": detail,
            }
        )

    if response.get("api_version") != "soma-emacs-agent-v0":
        issue("invalid_api_version", "response must declare soma-emacs-agent-v0")
    if not isinstance(response.get("request_id"), str):
        issue("invalid_request_id", "response request_id must be a string")
    if not isinstance(operation, str):
        issue("invalid_operation", "response operation must be a string")
    if operation_spec is None:
        issue(
            "unknown_operation",
            "response operation must be present in the operation manifest",
        )
    for field in required_fields:
        if field not in response:
            issue(
                "missing_required_field",
                "response is missing a manifest-required result-envelope field",
                field,
            )
    if state not in allowed_states:
        issue(
            "invalid_state",
            "response state must be declared by the operation result contract",
            {"allowed_states": sorted(allowed_states)},
        )

    state_sequence = response.get("state_sequence")
    if not isinstance(state_sequence, list) or not state_sequence:
        issue(
            "invalid_state_sequence",
            "response state_sequence must be a non-empty array",
        )
    else:
        if state_sequence[-1] != state:
            issue(
                "invalid_state_sequence",
                "response state_sequence must end in the current response state",
                state_sequence,
            )
        for sequence_state in state_sequence:
            if sequence_state not in allowed_sequence_states:
                issue(
                    "invalid_state_sequence",
                    "response state_sequence contains a state outside the lifecycle contract",
                    {
                        "sequence_state": sequence_state,
                        "allowed_states": sorted(allowed_sequence_states),
                    },
                )

    for field in ("evidence", "observations", "effects"):
        if not isinstance(response.get(field), dict):
            issue(f"invalid_{field}", f"response {field} must be an object")

    diagnostics = response.get("diagnostics")
    if not isinstance(diagnostics, list):
        issue(
            "invalid_diagnostics",
            "response diagnostics must be an array of code/message objects",
        )
    else:
        for index, diagnostic in enumerate(diagnostics):
            if (
                not isinstance(diagnostic, dict)
                or not isinstance(diagnostic.get("code"), str)
                or not isinstance(diagnostic.get("message"), str)
            ):
                issue(
                    "invalid_diagnostics",
                    "response diagnostics must be an array of code/message objects",
                    {"index": index, "diagnostic": diagnostic},
                )

    if "continuation" not in response:
        issue("missing_required_field", "response is missing continuation", "continuation")

    return issues


def collect_response_validations(
    manifest: dict[str, Any],
    records: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    validations: list[dict[str, Any]] = []

    def walk(value: Any, path: str, key: str | None = None) -> None:
        if response_like(value) and (response_slot(key) or "api_version" in value):
            skipped = value.get("client_side") is True
            validations.append(
                {
                    "path": path,
                    "request_id": value.get("request_id"),
                    "operation": value.get("operation"),
                    "state": value.get("state"),
                    "client_side": bool(skipped),
                    "issues": []
                    if skipped
                    else response_contract_issues(manifest, path, value),
                }
            )
        if isinstance(value, dict):
            for key, child in value.items():
                walk(child, f"{path}.{key}", key)
        elif isinstance(value, list):
            for index, child in enumerate(value):
                walk(child, f"{path}[{index}]", None)

    for index, record in enumerate(records):
        walk(record, f"records[{index}]")
    return validations


def rootfs_owned_process_path(path: str) -> bool:
    return (
        path.startswith("/")
        and not path.startswith("/nix/")
        and not path.startswith("/data02/home/")
        and not path.startswith("/workspace/")
        and not path.startswith("/project/")
    )


def rootfs_owned_runtime_path(path: str) -> bool:
    return (
        path.startswith("/")
        and not path.startswith("/nix/")
        and not path.startswith("/data02/home/")
        and not path.startswith("/workspace/")
    )


def run_command(argv: list[str]) -> dict[str, Any]:
    result = subprocess.run(
        argv,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    return {
        "argv": argv,
        "returncode": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }


def run_git(repo: str, *args: str) -> dict[str, Any]:
    return run_command(["git", "-C", repo, *args])


def git_output(repo: str, *args: str) -> str:
    result = run_git(repo, *args)
    if result["returncode"] != 0:
        return ""
    return result["stdout"]


def git_lines(repo: str, *args: str) -> list[str]:
    return sorted(line for line in git_output(repo, *args).splitlines() if line)


def read_text(path: str) -> str:
    if not path:
        return ""
    try:
        return pathlib.Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def file_lines(path: str) -> list[str]:
    text = read_text(path)
    if not text:
        return []
    return text.splitlines()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def file_sha256(path: str) -> str | bool:
    try:
        return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
    except OSError:
        return False


def file_signature(path: str) -> dict[str, Any]:
    if not path:
        return {
            "exists": False,
            "path": path,
            "truename": False,
            "type": "missing",
            "sha256": False,
        }
    file_path = pathlib.Path(path)
    if not file_path.exists():
        return {
            "exists": False,
            "path": path,
            "truename": False,
            "type": "missing",
            "sha256": False,
        }
    try:
        stat_result = file_path.stat()
        truename = str(file_path.resolve())
    except OSError:
        return {
            "exists": False,
            "path": path,
            "truename": False,
            "type": "missing",
            "sha256": False,
        }
    return {
        "exists": True,
        "path": path,
        "truename": truename,
        "type": "directory"
        if file_path.is_dir()
        else ("regular" if file_path.is_file() else "special"),
        "size": stat_result.st_size,
        "sha256": file_sha256(path) if file_path.is_file() else False,
    }


def org_preconditions(
    observation: dict[str, Any],
    heading: dict[str, Any],
    source_block: dict[str, Any] | None = None,
) -> dict[str, Any]:
    preconditions = {
        "buffer_name": observation.get("buffer_name"),
        "text_owner_name": observation.get("text_owner_name"),
        "buffer_file_name": observation.get("buffer_file_name"),
        "chars_tick": observation.get("chars_tick"),
        "point": observation.get("point"),
        "modified": observation.get("modified"),
        "read_only": observation.get("read_only"),
        "minibuffer_active": observation.get("minibuffer_active"),
        "mark_active": observation.get("mark_active"),
        "narrowed": observation.get("narrowed"),
        "org_node": heading,
        "structure_fingerprint": observation.get("structure_fingerprint"),
    }
    if source_block is not None:
        preconditions["source_block"] = source_block
    return preconditions


def first_dict(items: Any) -> dict[str, Any]:
    if isinstance(items, list) and items and isinstance(items[0], dict):
        return items[0]
    return {}


def magit_fixture_repo_state(repo: str, hook_log: str = "") -> dict[str, Any]:
    return {
        "cached_names": git_lines(repo, "diff", "--cached", "--name-only") if repo else [],
        "unstaged_names": git_lines(repo, "diff", "--name-only") if repo else [],
        "target_index": git_output(repo, "show", ":target.txt") if repo else "",
        "target_worktree": read_text(str(pathlib.Path(repo) / "target.txt")) if repo else "",
        "debug_index": git_output(repo, "show", ":debug.txt") if repo else "",
        "debug_worktree": read_text(str(pathlib.Path(repo) / "debug.txt")) if repo else "",
        "staged_index": git_output(repo, "show", ":staged.txt") if repo else "",
        "hook_log": read_text(hook_log),
    }


def target_content(selected: bool = False, other: bool = False, external: bool = False) -> str:
    lines: list[str] = []
    for line in range(1, 31):
        if line == 5:
            lines.append("target-05-selected" if selected else "target-05-base")
        elif line == 25:
            if external:
                lines.append("target-25-external")
            elif other:
                lines.append("target-25-other")
            else:
                lines.append("target-25-base")
        else:
            lines.append(f"target-{line:02d}-base")
    return "\n".join(lines) + "\n"


def find_magit_section(
    observation: dict[str, Any],
    *,
    section_type: str,
    needle: str = "",
    ancestor_file: str = "",
) -> dict[str, Any]:
    for section in observation.get("sections") or []:
        if section.get("type") != section_type:
            continue
        if needle and needle not in str(section.get("text_excerpt") or ""):
            continue
        if ancestor_file and section.get("ancestor_file") != ancestor_file:
            continue
        return section
    return {}


def magit_section_point(section: dict[str, Any]) -> int | None:
    for key in ("content", "start"):
        value = section.get(key)
        if isinstance(value, int):
            return value
    return None


def magit_repo_fingerprint(observation: dict[str, Any]) -> dict[str, Any]:
    return {
        key: observation.get(key)
        for key in (
            "magit_repo",
            "magit_head",
            "magit_status_hash",
            "magit_worktree_diff_hash",
            "magit_index_diff_hash",
        )
    }


def magit_section_identity(section: dict[str, Any]) -> dict[str, Any]:
    explicit = section.get("section_identity")
    if isinstance(explicit, dict):
        return explicit
    return {
        key: section.get(key)
        for key in (
            "type",
            "value",
            "ancestor_file",
            "start",
            "content",
            "end",
            "parent_types",
            "text_hash",
        )
    }


def magit_stage_preconditions(
    observation: dict[str, Any],
    section: dict[str, Any],
) -> dict[str, Any]:
    preconditions = {"buffer_name": observation.get("buffer_name")}
    preconditions["repo_fingerprint"] = magit_repo_fingerprint(observation)
    preconditions["section_identity"] = magit_section_identity(section)
    preconditions["this_command"] = "magit-stage"
    return preconditions


def stream_json_payload(stream_texts: list[Any], prefix: str) -> dict[str, Any]:
    for text in stream_texts:
        for line in str(text).splitlines():
            if line.startswith(prefix):
                try:
                    payload = json.loads(line[len(prefix) :])
                except json.JSONDecodeError:
                    return {}
                return payload if isinstance(payload, dict) else {}
    return {}


def display_payload(event: dict[str, Any]) -> dict[str, Any]:
    payload = event.get("application_json")
    return payload if isinstance(payload, dict) else {}


def contains(text: str, needle: str) -> bool:
    return needle in text


def has_exact_line(text: str, needle: str) -> bool:
    return any(line.strip() == needle for line in str(text).splitlines())


def find_transient_suffix(observation: dict[str, Any], command: str) -> dict[str, Any]:
    for suffix in observation.get("transient_suffixes") or []:
        if isinstance(suffix, dict) and suffix.get("command") == command:
            return suffix
    return {}


def transient_value_contains(value: Any, needle: str) -> bool:
    if isinstance(value, str):
        return needle in value
    if isinstance(value, list):
        return any(transient_value_contains(item, needle) for item in value)
    return False


def transient_active_argument_present(observation: dict[str, Any], argument: str) -> bool:
    return transient_value_contains(observation.get("transient_active_arguments"), argument)


def terminal_capture_shows_magit_diff_transient(capture: dict[str, Any]) -> bool:
    text = str(capture.get("stdout") or "")
    return (
        contains(text, "Limit arguments")
        and contains(text, "Context arguments")
        and contains(text, "Tune arguments")
        and (
            contains(text, "Show stats")
            or contains(text, "Diff algorithm")
            or contains(text, "Detect renames")
        )
    )


def noop_command(argv: list[str]) -> dict[str, Any]:
    return {"argv": argv, "returncode": 0, "stdout": "", "stderr": ""}


def precondition_check_passed(response: dict[str, Any], name: str, expected: bool = True) -> bool:
    result = response.get("result") or {}
    for item in result.get("precondition_checks") or []:
        if item.get("name") == name:
            return item.get("passed") is expected
    return False


def jupyter_preconditions_from_observation(observation: dict[str, Any]) -> dict[str, Any]:
    selected_range = observation.get("selected_range") or {}
    region_text = (
        selected_range.get("text_excerpt")
        if isinstance(selected_range, dict)
        else ""
    )
    preconditions = dict(observation)
    preconditions.update(
        {
            "region_text": region_text,
            "expected_text": region_text,
            "client": observation.get("jupyter_repl_buffer"),
            "kernel_token": observation.get("jupyter_kernel_token"),
            "jupyter_connection_file": observation.get("jupyter_connection_file"),
            "jupyter_connection_file_rootfs_owned": observation.get(
                "jupyter_connection_file_rootfs_owned"
            )
            if "jupyter_connection_file_rootfs_owned" in observation
            else rootfs_owned_runtime_path(
                str(observation.get("jupyter_connection_file") or "")
            ),
            "jupyter_kernel_token": observation.get("jupyter_kernel_token"),
            "jupyter_kernel_exe": observation.get("jupyter_kernel_exe"),
            "jupyter_kernel_exe_rootfs_owned": observation.get(
                "jupyter_kernel_exe_rootfs_owned"
            )
            if "jupyter_kernel_exe_rootfs_owned" in observation
            else rootfs_owned_process_path(
                str(observation.get("jupyter_kernel_exe") or "")
            ),
            "jupyter_client_session_id": observation.get(
                "jupyter_client_session_id"
            ),
            "jupyter_repl_buffer": observation.get("jupyter_repl_buffer"),
        }
    )
    return preconditions


def response_result(response: dict[str, Any]) -> dict[str, Any]:
    result = response.get("result") or {}
    return result if isinstance(result, dict) else {}


def v0_request(
    request_id: str,
    operation: str,
    capability: str,
    params: dict[str, Any] | None = None,
    preconditions: dict[str, Any] | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": request_id,
        "operation": operation,
        "capability": capability,
        "params": params or {},
    }
    if preconditions is not None:
        payload["preconditions"] = preconditions
    return payload


def full_buffer_text_v0(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    timeout: float = 2.0,
) -> dict[str, Any]:
    observe_request = v0_request(
        f"{request_prefix}-observe",
        "emacs.observe_buffer",
        "passive_observation",
        {"buffer_name": buffer_name},
    )
    observe_response = request_or_outcome_unknown(
        socket_path,
        observe_request,
        timeout=timeout,
    )
    observation = response_result(observe_response)
    point_max = observation.get("point_max")
    read_request: dict[str, Any] = {}
    read_response: dict[str, Any] = {}
    text = ""
    if isinstance(point_max, int):
        read_request = v0_request(
            f"{request_prefix}-read",
            "buffer.read",
            "passive_observation",
            {
                "buffer_name": buffer_name,
                "start": 1,
                "end": point_max,
            },
        )
        read_response = request_or_outcome_unknown(
            socket_path,
            read_request,
            timeout=timeout,
        )
        text = str(response_result(read_response).get("text") or "")
    return {
        "buffer_name": buffer_name,
        "observe_request": observe_request,
        "observe_response": observe_response,
        "observation": observation,
        "read_request": read_request,
        "read_response": read_response,
        "text": text,
    }


def lsp_diagnostic_count(observation: dict[str, Any]) -> int | None:
    diagnostics = observation.get("diagnostics")
    if not isinstance(diagnostics, dict):
        return None
    count = diagnostics.get("count")
    return count if isinstance(count, int) else None


def lsp_diagnostic_raw(observation: dict[str, Any]) -> str:
    diagnostics = observation.get("diagnostics")
    if not isinstance(diagnostics, dict):
        return ""
    return str(diagnostics.get("raw") or "")


def lsp_diagnostic_contains(observation: dict[str, Any], needle: str) -> bool:
    return bool(needle) and needle in lsp_diagnostic_raw(observation)


def wait_lsp_diagnostic_count(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    predicate: Any,
    timeout_seconds: float = 10.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    attempts: list[dict[str, Any]] = []
    request_payload: dict[str, Any] = {}
    response: dict[str, Any] = {}
    observation: dict[str, Any] = {}
    attempt = 0
    while time.time() < deadline:
        attempt += 1
        request_payload = {
            "request_id": f"{request_prefix}-{attempt}",
            "operation": "lsp.observe_diagnostics",
            "params": {
                "buffer_name": buffer_name,
                "settle_seconds": 0.2,
            },
        }
        response = request_or_outcome_unknown(socket_path, request_payload, timeout=2.0)
        observation = response_result(response)
        diagnostics = observation.get("diagnostics") if isinstance(observation, dict) else {}
        count = lsp_diagnostic_count(observation)
        raw = str((diagnostics or {}).get("raw") or "") if isinstance(diagnostics, dict) else ""
        attempts.append(
            {
                "attempt": attempt,
                "response_state": response.get("state"),
                "diagnostic_count": count,
                "diagnostic_raw_excerpt": raw[:1200],
                "document_version": observation.get("document_version"),
                "chars_tick": observation.get("chars_tick"),
            }
        )
        if response.get("state") == "succeeded" and predicate(count, observation):
            break
        time.sleep(0.2)
    return {
        "buffer_name": buffer_name,
        "attempts": attempts,
        "request": request_payload,
        "response": response,
        "observation": observation,
        "diagnostic_count": lsp_diagnostic_count(observation),
    }


def buffer_text_via_bridge(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_id: str,
    end: int,
    timeout: float = 2.0,
) -> tuple[dict[str, Any], str]:
    request_payload = {
        "request_id": request_id,
        "operation": "buffer.read",
        "params": {
            "buffer_name": buffer_name,
            "start": 1,
            "end": end,
        },
    }
    response = request_or_outcome_unknown(socket_path, request_payload, timeout=timeout)
    result = response_result(response)
    return response, str(result.get("text") or "")


def full_buffer_text_via_bridge(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    timeout: float = 2.0,
) -> dict[str, Any]:
    observe_request = {
        "request_id": f"{request_prefix}-observe",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": buffer_name},
    }
    observe_response = request_or_outcome_unknown(
        socket_path,
        observe_request,
        timeout=timeout,
    )
    observation = response_result(observe_response)
    point_max = observation.get("point_max")
    read_response: dict[str, Any] = {}
    text = ""
    if isinstance(point_max, int):
        read_response, text = buffer_text_via_bridge(
            socket_path,
            buffer_name,
            f"{request_prefix}-read",
            point_max,
            timeout=timeout,
        )
    return {
        "buffer_name": buffer_name,
        "observe_request": observe_request,
        "observe_response": observe_response,
        "observation": observation,
        "read_response": read_response,
        "text": text,
    }


def wait_buffer_text_via_bridge(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    needle: str,
    timeout_seconds: float = 5.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    attempts: list[dict[str, Any]] = []
    last_observe_request: dict[str, Any] = {}
    last_observe_response: dict[str, Any] = {}
    last_observation: dict[str, Any] = {}
    last_read_response: dict[str, Any] = {}
    last_text = ""
    attempt = 0
    while time.time() < deadline:
        attempt += 1
        last_observe_request = {
            "request_id": f"{request_prefix}-observe-{attempt}",
            "operation": "emacs.observe_buffer",
            "params": {"buffer_name": buffer_name},
        }
        last_observe_response = request_or_outcome_unknown(
            socket_path,
            last_observe_request,
            timeout=0.5,
        )
        last_observation = response_result(last_observe_response)
        point_max = last_observation.get("point_max")
        if isinstance(point_max, int):
            last_read_response, last_text = buffer_text_via_bridge(
                socket_path,
                buffer_name,
                f"{request_prefix}-read-{attempt}",
                point_max,
                timeout=0.5,
            )
        attempts.append(
            {
                "attempt": attempt,
                "observe_state": last_observe_response.get("state"),
                "point_max": point_max,
                "read_state": last_read_response.get("state"),
                "found_needle": bool(needle and needle in last_text),
                "text_tail": last_text[-400:],
            }
        )
        if needle and needle in last_text:
            break
        time.sleep(0.1)
    return {
        "buffer_name": buffer_name,
        "needle": needle,
        "attempts": attempts,
        "observe_request": last_observe_request,
        "observe_response": last_observe_response,
        "observation": last_observation,
        "read_response": last_read_response,
        "text": last_text,
        "found": bool(needle and needle in last_text),
    }


def wait_jupyter_display_observation(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    display_id: str,
    expected_text: str,
    timeout_seconds: float = 5.0,
    capability: str | None = None,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    attempts: list[dict[str, Any]] = []
    request_payload: dict[str, Any] = {}
    response: dict[str, Any] = {}
    result: dict[str, Any] = {}
    matching_range_texts: list[str] = []
    matching_range_contains_expected = False
    attempt = 0
    while time.time() < deadline:
        attempt += 1
        params = {
            "buffer_name": buffer_name,
            "display_id": display_id,
            "max_text": 4000,
        }
        if capability:
            request_payload = v0_request(
                f"{request_prefix}-{attempt}",
                "jupyter.observe_display",
                capability,
                params,
            )
        else:
            request_payload = {
                "request_id": f"{request_prefix}-{attempt}",
                "operation": "jupyter.observe_display",
                "params": params,
            }
        response = request_or_outcome_unknown(socket_path, request_payload, timeout=0.5)
        result = response_result(response)
        text = str(result.get("text_excerpt") or "")
        display_ranges = result.get("display_ranges") or []
        matching_ranges = [
            item
            for item in display_ranges
            if isinstance(item, dict) and item.get("display_id_matches") is True
        ]
        matching_range_texts = [
            str(item.get("text_excerpt") or "")
            for item in matching_ranges
        ]
        matching_range_contains_expected = bool(expected_text) and any(
            expected_text in text
            for text in matching_range_texts
        )
        attempts.append(
            {
                "attempt": attempt,
                "response_state": response.get("state"),
                "display_id_found": result.get("display_id_found"),
                "matching_range_count": len(matching_ranges),
                "matching_range_contains_expected": matching_range_contains_expected,
                "point_max": result.get("point_max"),
                "text_contains_expected": bool(expected_text and expected_text in text),
                "matching_range_text_excerpt": matching_range_texts[0][-800:] if matching_range_texts else "",
                "text_excerpt": text[-800:],
            }
        )
        if (
            response.get("state") == "succeeded"
            and result.get("display_id_found") is True
            and matching_range_contains_expected
        ):
            break
        time.sleep(0.1)
    return {
        "buffer_name": buffer_name,
        "display_id": display_id,
        "expected_text": expected_text,
        "attempts": attempts,
        "request": request_payload,
        "response": response,
        "observation": result,
        "found": (
            response.get("state") == "succeeded"
            and result.get("display_id_found") is True
            and matching_range_contains_expected
        ),
    }


def save_buffer_via_bridge(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    timeout: float = 2.0,
) -> dict[str, Any]:
    observe_request = {
        "request_id": f"{request_prefix}-observe",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": buffer_name},
    }
    observe_response = request_or_outcome_unknown(socket_path, observe_request, timeout=timeout)
    observation = response_result(observe_response)
    preconditions = dict(observation)
    preconditions.pop("selected_window_buffer", None)
    save_request = {
        "request_id": f"{request_prefix}-save",
        "operation": "buffer.save",
        "params": {
            "buffer_name": buffer_name,
            "preconditions": preconditions,
        },
    }
    save_response = request_or_outcome_unknown(socket_path, save_request, timeout=timeout)
    return {
        "buffer_name": buffer_name,
        "observe_request": observe_request,
        "observe_response": observe_response,
        "save_request": save_request,
        "save_response": save_response,
    }


def replace_range_via_bridge(
    socket_path: pathlib.Path,
    buffer_name: str,
    start: int | None,
    end: int | None,
    text: str,
    expected_text: str,
    request_prefix: str,
    timeout: float = 2.0,
) -> dict[str, Any]:
    observe_request = {
        "request_id": f"{request_prefix}-observe",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": buffer_name},
    }
    observe_response = request_or_outcome_unknown(socket_path, observe_request, timeout=timeout)
    observation = response_result(observe_response)
    preconditions = dict(observation)
    preconditions.pop("selected_window_buffer", None)
    preconditions["expected_text"] = expected_text
    replace_request = {
        "request_id": f"{request_prefix}-replace",
        "operation": "buffer.replace_range",
        "params": {
            "buffer_name": buffer_name,
            "start": start,
            "end": end,
            "text": text,
            "preconditions": preconditions,
        },
    }
    replace_response = request_or_outcome_unknown(socket_path, replace_request, timeout=timeout)
    return {
        "buffer_name": buffer_name,
        "start": start,
        "end": end,
        "expected_text": expected_text,
        "replacement_text": text,
        "observe_request": observe_request,
        "observe_response": observe_response,
        "replace_request": replace_request,
        "replace_response": replace_response,
    }


def request(socket_path: pathlib.Path, payload: dict[str, Any], timeout: float = 5.0) -> dict[str, Any]:
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
        client.settimeout(timeout)
        client.connect(str(socket_path))
        client.sendall(json.dumps(payload, sort_keys=True).encode("utf-8") + b"\n")
        chunks: list[bytes] = []
        while True:
            chunk = client.recv(65536)
            if not chunk:
                break
            chunks.append(chunk)
            if b"\n" in chunk:
                break
    data = b"".join(chunks)
    if b"\n" not in data:
        raise TimeoutError(f"bridge socket response had no newline: {data!r}")
    line = data.split(b"\n", 1)[0]
    return json.loads(line.decode("utf-8"))


def request_or_outcome_unknown(
    socket_path: pathlib.Path,
    payload: dict[str, Any],
    timeout: float = 5.0,
) -> dict[str, Any]:
    try:
        return request(socket_path, payload, timeout=timeout)
    except TimeoutError as exc:
        return {
            "request_id": payload.get("request_id"),
            "operation": payload.get("operation"),
            "state": "outcome_unknown",
            "state_sequence": ["accepted", "running", "outcome_unknown"],
            "result": None,
            "error_code": "client_timeout",
            "error": str(exc),
            "client_side": True,
        }


def switch_terminal_frame_to_buffer(
    socket_path: pathlib.Path,
    tmux_socket: str,
    tmux_target: str,
    buffer_name: str,
    request_prefix: str,
) -> dict[str, Any]:
    def failed_key_result(argv: list[str], reason: str) -> dict[str, Any]:
        return {"argv": argv, "returncode": 1, "stdout": "", "stderr": reason}

    def observe_minibuffer(
        suffix: str,
        attempts: list[dict[str, Any]],
        prompt_substring: str | None = None,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        response: dict[str, Any] = {}
        result: dict[str, Any] = {}
        for attempt in range(1, 31):
            request_payload = {
                "request_id": f"{request_prefix}-{suffix}-{attempt}",
                "operation": "emacs.observe_interaction",
                "params": {},
            }
            response = request_or_outcome_unknown(
                socket_path,
                request_payload,
                timeout=0.5,
            )
            result = response.get("result") or {}
            prompt = result.get("minibuffer_prompt")
            attempts.append(
                {
                    "attempt": attempt,
                    "response_state": response.get("state"),
                    "minibuffer_active": result.get("minibuffer_active"),
                    "minibuffer_prompt": prompt,
                    "selected_window_buffer": result.get("selected_window_buffer"),
                }
            )
            if (
                response.get("state") == "succeeded"
                and result.get("minibuffer_active") is True
                and (
                    prompt_substring is None
                    or (
                        isinstance(prompt, str)
                        and prompt_substring in prompt
                    )
                )
            ):
                break
            time.sleep(0.1)
        return response, result

    escape_result: dict[str, Any] = noop_command(
        ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "Escape"]
    )
    cancel_attempts: list[dict[str, Any]] = []
    cancel_result: dict[str, Any] = noop_command(
        ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "C-g"]
    )
    cancel_interaction_response: dict[str, Any] = {}
    cancel_interaction_result: dict[str, Any] = {}
    for attempt in range(1, 7):
        escape_result = run_command(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "Escape"]
        )
        time.sleep(0.05)
        cancel_result = run_command(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "C-g"]
        )
        time.sleep(0.1)
        cancel_interaction_request = {
            "request_id": f"{request_prefix}-cancel-interaction-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        cancel_interaction_response = request_or_outcome_unknown(
            socket_path,
            cancel_interaction_request,
            timeout=0.5,
        )
        cancel_interaction_result = cancel_interaction_response.get("result") or {}
        cancel_attempts.append(
            {
                "attempt": attempt,
                "cancel": cancel_result,
                "escape": escape_result,
                "response_state": cancel_interaction_response.get("state"),
                "minibuffer_active": cancel_interaction_result.get("minibuffer_active"),
                "transient_active": cancel_interaction_result.get("transient_active"),
                "transient_map_installed": cancel_interaction_result.get("transient_map_installed"),
                "selected_window_buffer": cancel_interaction_result.get("selected_window_buffer"),
            }
        )
        if (
            cancel_result["returncode"] == 0
            and cancel_interaction_response.get("state") == "succeeded"
            and cancel_interaction_result.get("minibuffer_active") is not True
            and cancel_interaction_result.get("transient_active") is not True
            and cancel_interaction_result.get("transient_map_installed") is not True
        ):
            break

    direct_prefix_result = run_command(
        ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "C-x", "b"]
    )
    minibuffer_attempts: list[dict[str, Any]] = []
    minibuffer_response, minibuffer_result = observe_minibuffer(
        "switch-minibuffer",
        minibuffer_attempts,
        "Switch to buffer",
    )
    fallback: dict[str, Any] = {"used": False}
    prefix_result: dict[str, Any] = dict(direct_prefix_result)
    if not (
        minibuffer_response.get("state") == "succeeded"
        and minibuffer_result.get("minibuffer_active") is True
    ):
        fallback_cancel = run_command(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "C-g"]
        )
        fallback_mx = run_command(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "M-x"]
        )
        fallback_mx_attempts: list[dict[str, Any]] = []
        fallback_mx_response, fallback_mx_result = observe_minibuffer(
            "mx-minibuffer",
            fallback_mx_attempts,
            "M-x",
        )
        fallback_command_keys = failed_key_result(
            ["tmux", "-S", tmux_socket, "send-keys", "-l", "-t", tmux_target, "switch-to-buffer"],
            "M-x minibuffer did not become active",
        )
        fallback_command_enter = failed_key_result(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "Enter"],
            "M-x command was not submitted",
        )
        fallback_switch_attempts: list[dict[str, Any]] = []
        if (
            fallback_mx_response.get("state") == "succeeded"
            and fallback_mx_result.get("minibuffer_active") is True
        ):
            fallback_command_keys = run_command(
                [
                    "tmux",
                    "-S",
                    tmux_socket,
                    "send-keys",
                    "-l",
                    "-t",
                    tmux_target,
                    "switch-to-buffer",
                ]
            )
            fallback_command_enter = run_command(
                ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "Enter"]
            )
            minibuffer_response, minibuffer_result = observe_minibuffer(
                "mx-switch-minibuffer",
                fallback_switch_attempts,
                "Switch to buffer",
            )
        fallback = {
            "used": True,
            "cancel": fallback_cancel,
            "mx": fallback_mx,
            "mx_attempts": fallback_mx_attempts,
            "command_keys": fallback_command_keys,
            "command_enter": fallback_command_enter,
            "switch_attempts": fallback_switch_attempts,
            "switch_response": minibuffer_response,
        }
        prefix_result = {
            "argv": direct_prefix_result.get("argv"),
            "returncode": 0
            if (
                direct_prefix_result.get("returncode") == 0
                and fallback_mx.get("returncode") == 0
                and fallback_command_keys.get("returncode") == 0
                and fallback_command_enter.get("returncode") == 0
            )
            else 1,
            "stdout": "",
            "stderr": "",
            "primary": direct_prefix_result,
            "fallback": fallback,
        }

    if (
        minibuffer_response.get("state") == "succeeded"
        and minibuffer_result.get("minibuffer_active") is True
    ):
        buffer_keys_result = run_command(
            ["tmux", "-S", tmux_socket, "send-keys", "-l", "-t", tmux_target, buffer_name]
        )
        enter_result = run_command(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "Enter"]
        )
    else:
        buffer_keys_result = failed_key_result(
            ["tmux", "-S", tmux_socket, "send-keys", "-l", "-t", tmux_target, buffer_name],
            "switch-to-buffer minibuffer did not become active",
        )
        enter_result = failed_key_result(
            ["tmux", "-S", tmux_socket, "send-keys", "-t", tmux_target, "Enter"],
            "target buffer name was not submitted",
        )
    frame_attempts: list[dict[str, Any]] = []
    frames_response: dict[str, Any] = {}
    frames_result: dict[str, Any] = {}
    terminal_frames: list[dict[str, Any]] = []
    for attempt in range(1, 31):
        frames_request = {
            "request_id": f"{request_prefix}-frame-{attempt}",
            "operation": "emacs.observe_frames",
            "params": {},
        }
        frames_response = request(socket_path, frames_request)
        frames_result = frames_response.get("result") or {}
        frames = frames_result.get("frames") or []
        terminal_frames = [
            frame
            for frame in frames
            if frame.get("client_frame") is True
            and frame.get("graphic") is False
            and frame.get("terminal_live") is True
            and frame.get("selected_window_buffer") == buffer_name
        ]
        frame_attempts.append(
            {
                "attempt": attempt,
                "response_state": frames_response.get("state"),
                "client_terminal_buffers": [
                    frame.get("selected_window_buffer")
                    for frame in frames
                    if frame.get("client_frame") is True and frame.get("graphic") is False
                ],
            }
        )
        if terminal_frames:
            break
        time.sleep(0.1)

    return {
        "escape": escape_result,
        "cancel": cancel_result,
        "cancel_attempts": cancel_attempts,
        "cancel_interaction_response": cancel_interaction_response,
        "prefix": prefix_result,
        "fallback": fallback,
        "minibuffer_attempts": minibuffer_attempts,
        "minibuffer_response": minibuffer_response,
        "buffer_keys": buffer_keys_result,
        "enter": enter_result,
        "observation_attempts": frame_attempts,
        "response": frames_response,
        "result": frames_result,
        "terminal_frames": terminal_frames,
    }


def wait_comint_observation(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    done_needle: str,
    counter_path: str,
    timeout_seconds: float = 5.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    attempts: list[dict[str, Any]] = []
    last_response: dict[str, Any] = {}
    last_result: dict[str, Any] = {}
    attempt = 0
    while time.time() < deadline:
        attempt += 1
        observe_request = {
            "request_id": f"{request_prefix}-{attempt}",
            "operation": "comint.observe",
            "params": {"buffer_name": buffer_name},
        }
        last_response = request_or_outcome_unknown(
            socket_path,
            observe_request,
            timeout=0.5,
        )
        last_result = last_response.get("result") or {}
        counter_lines = file_lines(counter_path)
        buffer_tail = str(last_result.get("buffer_tail") or "")
        attempts.append(
            {
                "attempt": attempt,
                "response_state": last_response.get("state"),
                "process_live": last_result.get("process_live"),
                "process_id": last_result.get("process_id"),
                "last_input_markers_live": last_result.get("last_input_markers_live"),
                "last_input_text": last_result.get("last_input_text"),
                "counter_lines": counter_lines,
                "buffer_tail": buffer_tail[-600:],
            }
        )
        if done_needle in buffer_tail and counter_lines:
            break
        time.sleep(0.1)
    return {
        "attempts": attempts,
        "response": last_response,
        "result": last_result,
        "counter_lines": file_lines(counter_path),
    }


def wait_comint_tail_needle(
    socket_path: pathlib.Path,
    buffer_name: str,
    request_prefix: str,
    needle: str,
    counter_path: str,
    timeout_seconds: float = 5.0,
    exact_line: bool = False,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    attempts: list[dict[str, Any]] = []
    last_response: dict[str, Any] = {}
    last_result: dict[str, Any] = {}
    attempt = 0
    while time.time() < deadline:
        attempt += 1
        observe_request = {
            "request_id": f"{request_prefix}-{attempt}",
            "operation": "comint.observe",
            "params": {"buffer_name": buffer_name},
        }
        last_response = request_or_outcome_unknown(
            socket_path,
            observe_request,
            timeout=0.5,
        )
        last_result = last_response.get("result") or {}
        counter_lines = file_lines(counter_path)
        buffer_tail = str(last_result.get("buffer_tail") or "")
        found = has_exact_line(buffer_tail, needle) if exact_line else needle in buffer_tail
        attempts.append(
            {
                "attempt": attempt,
                "response_state": last_response.get("state"),
                "process_live": last_result.get("process_live"),
                "process_id": last_result.get("process_id"),
                "last_input_markers_live": last_result.get("last_input_markers_live"),
                "last_input_text": last_result.get("last_input_text"),
                "counter_lines": counter_lines,
                "found": found,
                "buffer_tail": buffer_tail[-600:],
            }
        )
        if found:
            break
        time.sleep(0.1)
    return {
        "attempts": attempts,
        "response": last_response,
        "result": last_result,
        "counter_lines": file_lines(counter_path),
    }


TERMINAL_REQUEST_STATES = {"succeeded", "failed", "outcome_unknown"}


def wait_request_terminal_status(
    socket_path: pathlib.Path,
    target_request_id: str,
    request_prefix: str,
    timeout_seconds: float = 10.0,
    per_request_timeout: float = 0.5,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    attempts: list[dict[str, Any]] = []
    last_request: dict[str, Any] = {}
    last_response: dict[str, Any] = {}
    last_status: dict[str, Any] = {}
    attempt = 0
    while time.time() < deadline:
        attempt += 1
        last_request = {
            "request_id": f"{request_prefix}-{attempt}",
            "operation": "request.status",
            "params": {"request_id": target_request_id},
        }
        last_response = request_or_outcome_unknown(
            socket_path,
            last_request,
            timeout=per_request_timeout,
        )
        result = last_response.get("result") or {}
        last_status = result.get("status") or {}
        attempts.append(
            {
                "attempt": attempt,
                "response_state": last_response.get("state"),
                "status_request_id": last_status.get("request_id"),
                "status_state": last_status.get("state"),
                "status_operation": last_status.get("operation"),
                "status_error_code": last_status.get("error_code"),
            }
        )
        if last_status.get("state") in TERMINAL_REQUEST_STATES:
            break
        time.sleep(0.1)
    return {
        "attempts": attempts,
        "request": last_request,
        "response": last_response,
        "status": last_status,
    }


def scenario_record(
    index: int,
    scenario_id: str,
    title: str,
    expected_policy_pass: bool,
    out_dir: pathlib.Path,
    **payload: Any,
) -> dict[str, Any]:
    checks = payload.get("checks", [])
    policy_passed = all_checks_pass(checks)
    as_expected = policy_passed if expected_policy_pass else not policy_passed
    record = {
        "id": scenario_id,
        "title": title,
        "expected_policy_pass": expected_policy_pass,
        "policy_passed_grader": policy_passed,
        "as_expected": as_expected,
        **payload,
    }
    write_json(out_dir / f"{index:02d}-{scenario_id}.json", record)
    return record


def insula_checks(
    plan: dict[str, Any] | None,
    ready: dict[str, Any] | None,
    socket_path: pathlib.Path,
    daemon_name: str,
) -> list[dict[str, Any]]:
    plan = plan or {}
    ready = ready or {}
    return [
        check("SOMA_INSULA_ACTIVE is set", os.environ.get("SOMA_INSULA_ACTIVE") == "1", os.environ.get("SOMA_INSULA_ACTIVE", "")),
        check(
            "runtime mode is selected rootfs",
            os.environ.get("SOMA_INSULA_MODE") == "selected-container-rootfs"
            and plan.get("mode") == "selected-container-rootfs-insula",
            {"env": os.environ.get("SOMA_INSULA_MODE", ""), "plan": plan.get("mode", "")},
        ),
        check("benchmark plan is for daemon bridge transport", plan.get("benchmark") == "bridge-daemon-transport", plan.get("benchmark", "")),
        check("dependency closure is rootfs-owned", plan.get("dependency_closure") == "rootfs_owned", plan.get("dependency_closure", "")),
        check("glibc owner is container image rootfs", plan.get("glibc_owner") == "container-image-rootfs", plan.get("glibc_owner", "")),
        check("Emacs executable is source-built rootfs path", plan.get("emacs") == "/opt/soma/emacs/bin/emacs", plan.get("emacs", "")),
        check("bridge socket is private run state", str(socket_path).startswith("/project/run/"), str(socket_path)),
        check("daemon bootstrap reported daemonp", ready.get("daemonp") is True, ready),
        check("daemon server name matches client target", ready.get("server_name") == daemon_name, ready.get("server_name", "")),
        check("daemon run id matches Insula run id", ready.get("soma_insula_run_id") == os.environ.get("SOMA_INSULA_RUN_ID", ""), ready.get("soma_insula_run_id", "")),
        check("daemon bootstrap socket matches client socket", ready.get("socket") == str(socket_path), ready.get("socket", "")),
        check("client Python executable is rootfs-owned", rootfs_owned_process_path(sys.executable), sys.executable),
    ]


def run_profile_stable_smoke(
    args: argparse.Namespace,
    ready: dict[str, Any] | None,
    plan: dict[str, Any] | None,
) -> dict[str, Any]:
    fixture = (ready or {}).get("profile_smoke_fixture") or {}
    code_fixture = fixture.get("code") or {}
    comint_fixture = fixture.get("comint") or {}
    comint_interrupt_fixture = fixture.get("comint_interrupt") or {}
    jupyter_fixture = fixture.get("jupyter") or {}
    jupyter_display_fixture = fixture.get("jupyter_display") or {}
    org_fixture = fixture.get("org_report") or {}
    dired_fixture = fixture.get("dired_rename") or {}
    dired_delete_fixture = fixture.get("dired_delete") or {}
    magit_fixture = fixture.get("magit") or {}
    magit_file_fixture = fixture.get("magit_file") or {}
    lsp_fixture = fixture.get("lsp") or {}

    all_requests: list[dict[str, Any]] = []

    def send(
        request_id: str,
        operation: str,
        capability: str,
        params: dict[str, Any] | None = None,
        preconditions: dict[str, Any] | None = None,
        timeout: float = 5.0,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        payload = v0_request(request_id, operation, capability, params, preconditions)
        all_requests.append(payload)
        return payload, request_or_outcome_unknown(args.socket, payload, timeout=timeout)

    def select_buffer(
        request_id: str,
        buffer_name: str,
        point: Any = None,
        timeout: float = 5.0,
    ) -> tuple[dict[str, Any], dict[str, Any]]:
        params: dict[str, Any] = {"buffer_name": buffer_name}
        if isinstance(point, int):
            params["point"] = point
        return send(
            request_id,
            "buffer.select",
            "editor_state_change",
            params,
            timeout=timeout,
        )

    def jupyter_eval_preconditions(observation: dict[str, Any]) -> dict[str, Any]:
        return jupyter_preconditions_from_observation(observation)

    manifest_request, manifest_response = send(
        "daemon-profile-smoke-api-list-1",
        "api.list_capabilities",
        "passive_observation",
        {},
    )
    manifest = response_result(manifest_response)

    describe_request, describe_response = send(
        "daemon-profile-smoke-describe-1",
        "api.describe_operation",
        "passive_observation",
        {"operation": "emacs.observe_interaction"},
    )
    description = response_result(describe_response)
    description_request_contract = description.get("request_contract") or {}
    description_result_contract = description.get("result_contract") or {}

    session_request, session_response = send(
        "daemon-profile-smoke-session-1",
        "emacs.observe_session",
        "passive_observation",
        {},
    )
    session_result = response_result(session_response)
    status_request, status_response = send(
        "daemon-profile-smoke-session-status-1",
        "request.status",
        "passive_observation",
        {"request_id": "daemon-profile-smoke-session-1"},
    )
    status_result = response_result(status_response)
    stored_session_status = status_result.get("status") or {}
    frames_request, frames_response = send(
        "daemon-profile-smoke-frames-1",
        "emacs.observe_frames",
        "passive_observation",
        {},
    )
    frames_result = response_result(frames_response)
    frames_evidence = frames_response.get("evidence") or {}
    frames_observations = frames_response.get("observations") or {}
    frames_effects = frames_response.get("effects") or {}
    interaction_request, interaction_response = send(
        "daemon-profile-smoke-interaction-1",
        "emacs.observe_interaction",
        "passive_observation",
        {},
    )
    interaction_result = response_result(interaction_response)
    interaction_evidence = interaction_response.get("evidence") or {}
    interaction_observations = interaction_response.get("observations") or {}
    interaction_effects = interaction_response.get("effects") or {}

    code_root = str(code_fixture.get("root") or "")
    code_buffer = str(code_fixture.get("buffer_name") or "")
    code_file = str(code_fixture.get("source_file") or "")
    code_completion = str(code_fixture.get("completion_text") or "")
    code_edit_old = str(code_fixture.get("edit_old") or "")
    code_edit_new = str(code_fixture.get("edit_new") or "")
    completion_point = code_fixture.get("completion_point")
    search_needle = str(code_fixture.get("search_needle") or "")

    project_observe_request, project_observe_response = send(
        "daemon-profile-smoke-project-observe-1",
        "project.observe",
        "passive_observation",
        {"root": code_root},
    )
    project_search_request, project_search_response = send(
        "daemon-profile-smoke-project-search-1",
        "project.search",
        "passive_observation",
        {
            "root": code_root,
            "query": search_needle,
            "max_results": 10,
            "max_text": 200,
        },
    )

    code_select_request, code_select_response = select_buffer(
        "daemon-profile-smoke-code-select-1",
        code_buffer,
        completion_point,
    )
    code_observe_request, code_observe_response = send(
        "daemon-profile-smoke-code-observe-1",
        "emacs.observe_buffer",
        "passive_observation",
        {"buffer_name": code_buffer},
    )
    code_observation = response_result(code_observe_response)

    completion_observe_request, completion_observe_response = send(
        "daemon-profile-smoke-completion-observe-1",
        "completion.observe",
        "passive_observation",
        {
            "buffer_name": code_buffer,
            "max_candidates": 8,
            "prefix_chars": 80,
            "suffix_chars": 80,
        },
    )
    completion_observation = response_result(completion_observe_response)
    completion_range = completion_observation.get("replacement_range") or {}
    completion_preconditions = completion_observation.get("acceptance_preconditions") or {}
    completion_accept_request, completion_accept_response = send(
        "daemon-profile-smoke-completion-accept-1",
        "completion.accept",
        "reversible_editor_edit",
        {
            "buffer_name": code_buffer,
            "start": completion_range.get("start"),
            "end": completion_range.get("end"),
            "text": code_completion,
        },
        completion_preconditions,
    )

    code_after_completion_select_request, code_after_completion_select_response = (
        select_buffer(
            "daemon-profile-smoke-code-select-after-completion-1",
            code_buffer,
        )
    )
    code_after_completion = full_buffer_text_v0(
        args.socket,
        code_buffer,
        "daemon-profile-smoke-code-after-completion",
    )
    all_requests.extend(
        [
            code_after_completion.get("observe_request") or {},
            code_after_completion.get("read_request") or {},
        ]
    )
    code_after_completion_text = str(code_after_completion.get("text") or "")
    edit_index = code_after_completion_text.find(code_edit_old)
    if edit_index >= 0:
        edit_start = edit_index + 1
        edit_end = edit_start + len(code_edit_old)
        code_edit_preconditions = dict(code_after_completion.get("observation") or {})
        code_edit_preconditions["expected_text"] = code_edit_old
        code_edit_request, code_edit_response = send(
            "daemon-profile-smoke-code-edit-1",
            "buffer.replace_range",
            "reversible_editor_edit",
            {
                "buffer_name": code_buffer,
                "start": edit_start,
                "end": edit_end,
                "text": code_edit_new,
            },
            code_edit_preconditions,
        )
    else:
        edit_start = None
        edit_end = None
        code_edit_request = {}
        code_edit_response = {
            "request_id": "daemon-profile-smoke-code-edit-1",
            "operation": "buffer.replace_range",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "error_code": "target_text_missing",
            "error": "profile smoke code edit target was not found",
            "client_side": True,
        }

    code_after_edit_select_request, code_after_edit_select_response = select_buffer(
        "daemon-profile-smoke-code-select-after-edit-1",
        code_buffer,
    )
    code_after_edit = full_buffer_text_v0(
        args.socket,
        code_buffer,
        "daemon-profile-smoke-code-after-edit",
    )
    all_requests.extend(
        [
            code_after_edit.get("observe_request") or {},
            code_after_edit.get("read_request") or {},
        ]
    )
    code_patch_old = "    return note, answer\n"
    code_patch_new = '    return {"note": note, "answer": answer}\n'
    code_after_edit_text = str(code_after_edit.get("text") or "")
    if code_patch_old in code_after_edit_text:
        code_patch_preconditions = dict(code_after_edit.get("observation") or {})
        code_patch_request, code_patch_response = send(
            "daemon-profile-smoke-code-apply-patch-1",
            "buffer.apply_patch",
            "reversible_editor_edit",
            {
                "buffer_name": code_buffer,
                "patch": {
                    "format": "soma-context-patch-v0",
                    "hunks": [
                        {
                            "old_text": code_patch_old,
                            "new_text": code_patch_new,
                        }
                    ],
                },
            },
            code_patch_preconditions,
        )
    else:
        code_patch_request = {}
        code_patch_response = {
            "request_id": "daemon-profile-smoke-code-apply-patch-1",
            "operation": "buffer.apply_patch",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "error_code": "target_text_missing",
            "error": "profile smoke code patch target was not found",
            "client_side": True,
        }

    code_after_patch_select_request, code_after_patch_select_response = select_buffer(
        "daemon-profile-smoke-code-select-after-patch-1",
        code_buffer,
    )
    code_after_patch = full_buffer_text_v0(
        args.socket,
        code_buffer,
        "daemon-profile-smoke-code-after-patch",
    )
    all_requests.extend(
        [
            code_after_patch.get("observe_request") or {},
            code_after_patch.get("read_request") or {},
        ]
    )
    code_save_preconditions = dict(code_after_patch.get("observation") or {})
    code_save_request, code_save_response = send(
        "daemon-profile-smoke-code-save-1",
        "buffer.save",
        "filesystem_write",
        {"buffer_name": code_buffer},
        code_save_preconditions,
    )
    code_final_text = read_text(code_file)

    lsp_buffer = str(lsp_fixture.get("buffer_name") or "")
    lsp_root = str(lsp_fixture.get("root") or "")
    lsp_symbol = str(lsp_fixture.get("symbol") or "")
    lsp_new_name = str(lsp_fixture.get("new_name") or "")
    lsp_definition_file = str(lsp_fixture.get("definition_file") or "")
    lsp_main_file = str(lsp_fixture.get("main_file") or "")
    lsp_test_file = str(lsp_fixture.get("test_file") or "")
    lsp_test_dir = str(lsp_fixture.get("test_dir") or "")
    lsp_definition_point = lsp_fixture.get("definition_point")
    lsp_select_request, lsp_select_response = select_buffer(
        "daemon-profile-smoke-lsp-select-1",
        lsp_buffer,
        lsp_definition_point,
        timeout=5.0,
    )
    lsp_observe_request, lsp_observe_response = send(
        "daemon-profile-smoke-lsp-observe-1",
        "lsp.observe_source",
        "passive_observation",
        {"buffer_name": lsp_buffer, "max_text": 1600},
        timeout=10.0,
    )
    lsp_observation = response_result(lsp_observe_response)
    lsp_diagnostics_request, lsp_diagnostics_response = send(
        "daemon-profile-smoke-lsp-diagnostics-1",
        "lsp.observe_diagnostics",
        "passive_observation",
        {
            "buffer_name": lsp_buffer,
            "settle_seconds": 0.2,
        },
        lsp_observation,
        timeout=10.0,
    )
    lsp_diagnostics_observation = response_result(lsp_diagnostics_response)
    lsp_diagnostics_evidence = lsp_diagnostics_response.get("evidence") or {}
    lsp_diagnostics_observations = (
        lsp_diagnostics_response.get("observations") or {}
    )
    lsp_diagnostics_effects = lsp_diagnostics_response.get("effects") or {}
    lsp_rename_request, lsp_rename_response = send(
        "daemon-profile-smoke-lsp-rename-1",
        "lsp.rename_symbol",
        "multi_buffer_editor_edit",
        {
            "buffer_name": lsp_buffer,
            "new_name": lsp_new_name,
            "timeout_seconds": 30.0,
        },
        lsp_observation,
        timeout=2.0,
    )
    lsp_await_request, lsp_await_response = send(
        "daemon-profile-smoke-lsp-await-1",
        "request.await",
        "passive_observation",
        {
            "request_id": "daemon-profile-smoke-lsp-rename-1",
            "timeout_seconds": 40.0,
        },
        timeout=45.0,
    )
    lsp_await_result = response_result(lsp_await_response)
    lsp_final_status = lsp_await_result.get("status") or lsp_rename_response
    lsp_rename_result = lsp_final_status.get("result") or {}
    lsp_test = run_command(
        [
            "python3",
            "-c",
            (
                "import sys, unittest; "
                f"sys.path.insert(0, {json.dumps(lsp_root)}); "
                f"suite = unittest.defaultTestLoader.discover({json.dumps(lsp_test_dir)}); "
                "result = unittest.TextTestRunner(verbosity=2).run(suite); "
                "raise SystemExit(0 if result.wasSuccessful() else 1)"
            ),
        ]
    )
    lsp_definition_text = read_text(lsp_definition_file)
    lsp_main_text = read_text(lsp_main_file)
    lsp_test_text = read_text(lsp_test_file)

    comint_buffer = str(comint_fixture.get("buffer_name") or "")
    comint_counter = str(comint_fixture.get("counter") or "")
    comint_label = "profile"
    comint_done_needle = f"SOMA_PROFILE_COMINT_DONE:{comint_label}"
    comint_command = (
        f"printf 'SOMA_PROFILE_COMINT_SENT:{comint_label}\\n'; "
        f"printf 'SOMA_PROFILE_COMINT_RUN:%s\\n' \"$SOMA_INSULA_RUN_ID\"; "
        f"printf '{comint_label}\\n' >> {shlex.quote(comint_counter)}; "
        f"printf 'SOMA_PROFILE_COMINT_DONE:{comint_label}\\n'"
    )
    comint_observe_request, comint_observe_response = send(
        "daemon-profile-smoke-comint-observe-1",
        "comint.observe",
        "passive_observation",
        {"buffer_name": comint_buffer},
    )
    comint_observation = response_result(comint_observe_response)
    comint_submit_request, comint_submit_response = send(
        "daemon-profile-smoke-comint-submit-1",
        "comint.submit_input",
        "external_process_input",
        {
            "buffer_name": comint_buffer,
            "input": comint_command,
        },
        comint_observation,
        timeout=3.0,
    )
    comint_submit_result = response_result(comint_submit_response)
    comint_after_submit = (
        comint_submit_result.get("after")
        if isinstance(comint_submit_result.get("after"), dict)
        else comint_observation
    )
    comint_await_request, comint_await_response = send(
        "daemon-profile-smoke-comint-await-1",
        "comint.await_output",
        "passive_observation",
        {
            "buffer_name": comint_buffer,
            "needle": comint_done_needle,
            "timeout_seconds": 5.0,
        },
        comint_after_submit,
        timeout=7.0,
    )
    comint_await_result = response_result(comint_await_response)
    comint_counter_lines = file_lines(comint_counter)

    comint_interrupt_buffer = str(
        comint_interrupt_fixture.get("buffer_name") or ""
    )
    comint_interrupt_counter = str(comint_interrupt_fixture.get("counter") or "")
    comint_interrupt_command = str(comint_interrupt_fixture.get("command") or "")
    comint_interrupt_start_needle = str(
        comint_interrupt_fixture.get("start_needle")
        or "SOMA_DAEMON_COMINT_INTERRUPT_START"
    )
    comint_interrupt_done_needle = str(
        comint_interrupt_fixture.get("done_needle")
        or "SOMA_DAEMON_COMINT_INTERRUPT_DONE"
    )
    comint_interrupt_observe_request, comint_interrupt_observe_response = send(
        "daemon-profile-smoke-comint-interrupt-observe-1",
        "comint.observe",
        "passive_observation",
        {"buffer_name": comint_interrupt_buffer},
    )
    comint_interrupt_observation = response_result(
        comint_interrupt_observe_response
    )
    comint_interrupt_submit_request, comint_interrupt_submit_response = send(
        "daemon-profile-smoke-comint-interrupt-submit-1",
        "comint.submit_input",
        "external_process_input",
        {
            "buffer_name": comint_interrupt_buffer,
            "input": comint_interrupt_command,
        },
        comint_interrupt_observation,
        timeout=3.0,
    )
    comint_interrupt_submit_result = response_result(
        comint_interrupt_submit_response
    )
    comint_interrupt_after_submit = (
        comint_interrupt_submit_result.get("after")
        if isinstance(comint_interrupt_submit_result.get("after"), dict)
        else comint_interrupt_observation
    )
    comint_interrupt_await_start_request, comint_interrupt_await_start_response = (
        send(
            "daemon-profile-smoke-comint-interrupt-await-start-1",
            "comint.await_output",
            "passive_observation",
            {
                "buffer_name": comint_interrupt_buffer,
                "needle": comint_interrupt_start_needle,
                "timeout_seconds": 3.0,
            },
            comint_interrupt_after_submit,
            timeout=5.0,
        )
    )
    comint_interrupt_await_start_result = response_result(
        comint_interrupt_await_start_response
    )
    comint_interrupt_before_request, comint_interrupt_before_response = send(
        "daemon-profile-smoke-comint-interrupt-before-1",
        "comint.observe",
        "passive_observation",
        {"buffer_name": comint_interrupt_buffer},
    )
    comint_interrupt_before = response_result(comint_interrupt_before_response)
    comint_interrupt_request, comint_interrupt_response = send(
        "daemon-profile-smoke-comint-interrupt-1",
        "comint.interrupt",
        "external_process_control",
        {"buffer_name": comint_interrupt_buffer},
        comint_interrupt_before,
        timeout=3.0,
    )
    comint_interrupt_result = response_result(comint_interrupt_response)
    time.sleep(2.5)
    comint_interrupt_final_request, comint_interrupt_final_response = send(
        "daemon-profile-smoke-comint-interrupt-final-1",
        "comint.observe",
        "passive_observation",
        {"buffer_name": comint_interrupt_buffer},
    )
    comint_interrupt_final = response_result(comint_interrupt_final_response)
    comint_interrupt_tail = str(comint_interrupt_final.get("buffer_tail") or "")
    comint_interrupt_counter_lines = file_lines(comint_interrupt_counter)
    comint_interrupt_evidence = comint_interrupt_response.get("evidence") or {}
    comint_interrupt_effects = comint_interrupt_response.get("effects") or {}
    comint_interrupt_process_identity = (
        comint_interrupt_evidence.get("process_identity")
        if isinstance(comint_interrupt_evidence, dict)
        else {}
    )
    comint_interrupt_process_survival = (
        comint_interrupt_evidence.get("process_survival")
        if isinstance(comint_interrupt_evidence, dict)
        else {}
    )
    comint_interrupt_process_effect = (
        comint_interrupt_effects.get("process_interrupt")
        if isinstance(comint_interrupt_effects, dict)
        else {}
    )

    jupyter_buffer = str(jupyter_fixture.get("source_buffer") or "")
    jupyter_repl = str(jupyter_fixture.get("repl_buffer") or "")
    jupyter_beg = jupyter_fixture.get("beg")
    jupyter_end = jupyter_fixture.get("end")
    jupyter_selected_code = str(jupyter_fixture.get("selected_code") or "")
    jupyter_training_code = str(jupyter_fixture.get("training_code") or "")
    jupyter_attach_preconditions = {
        "kernel_id": jupyter_fixture.get("kernel_token"),
        "kernel_session": jupyter_fixture.get("client_session_id"),
        "jupyter_connection_file": jupyter_fixture.get("connection_file"),
        "jupyter_connection_file_rootfs_owned": rootfs_owned_runtime_path(
            str(jupyter_fixture.get("connection_file") or "")
        ),
        "jupyter_kernel_token": jupyter_fixture.get("kernel_token"),
        "jupyter_kernel_exe": jupyter_fixture.get("kernel_exe"),
        "jupyter_kernel_exe_rootfs_owned": rootfs_owned_process_path(
            str(jupyter_fixture.get("kernel_exe") or "")
        ),
        "jupyter_client_session_id": jupyter_fixture.get("client_session_id"),
        "jupyter_repl_buffer": jupyter_repl,
    }
    jupyter_attach_request, jupyter_attach_response = send(
        "daemon-profile-smoke-jupyter-attach-1",
        "jupyter.attach_existing",
        "editor_state_change",
        {
            "buffer_name": jupyter_buffer,
            "client": jupyter_repl,
        },
        jupyter_attach_preconditions,
        timeout=5.0,
    )
    jupyter_select_request, jupyter_select_response = select_buffer(
        "daemon-profile-smoke-jupyter-select-1",
        jupyter_buffer,
        jupyter_beg,
    )
    jupyter_observe_request, jupyter_observe_response = send(
        "daemon-profile-smoke-jupyter-observe-1",
        "jupyter.observe_source",
        "passive_observation",
        {
            "buffer_name": jupyter_buffer,
            "start": jupyter_beg,
            "end": jupyter_end,
            "max_text": 1600,
        },
        timeout=5.0,
    )
    jupyter_observation = response_result(jupyter_observe_response)
    jupyter_eval_preconditions_payload = jupyter_eval_preconditions(
        jupyter_observation
    )
    jupyter_eval_request, jupyter_eval_response = send(
        "daemon-profile-smoke-jupyter-eval-1",
        "jupyter.eval_region",
        "kernel_execution",
        {
            "buffer_name": jupyter_buffer,
            "start": jupyter_beg,
            "end": jupyter_end,
            "timeout_seconds": 30.0,
        },
        jupyter_eval_preconditions_payload,
        timeout=2.0,
    )
    jupyter_await_request, jupyter_await_response = send(
        "daemon-profile-smoke-jupyter-await-1",
        "jupyter.await_request",
        "passive_observation",
        {
            "request_id": "daemon-profile-smoke-jupyter-eval-1",
            "timeout_seconds": 40.0,
        },
        timeout=45.0,
    )
    jupyter_await_result = response_result(jupyter_await_response)
    jupyter_final_status = jupyter_await_result.get("status") or jupyter_eval_response
    jupyter_eval_result = jupyter_final_status.get("result") or {}
    jupyter_stream_payload = stream_json_payload(
        jupyter_eval_result.get("stream_texts") or [],
        "SOMA_SELECTED_JSON:",
    )
    jupyter_display_id = str(jupyter_display_fixture.get("display_id") or "")
    jupyter_display_preparation = (
        jupyter_display_fixture.get("display_preparation") or {}
    )
    jupyter_display_preparation_evidence = (
        jupyter_display_preparation.get("evidence") or {}
    )
    jupyter_display_repl = str(
        jupyter_display_preparation.get("repl_buffer")
        or jupyter_display_fixture.get("repl_buffer")
        or ""
    )
    jupyter_display_update_events = (
        jupyter_display_preparation_evidence.get("display_update_events") or []
    )
    jupyter_display_update_event = (
        jupyter_display_update_events[0]
        if len(jupyter_display_update_events) == 1
        and isinstance(jupyter_display_update_events[0], dict)
        else {}
    )
    jupyter_display_expected_text = str(
        jupyter_display_update_event.get("text_plain")
        or "SOMA_DISPLAY_STAGE:updated"
    )
    jupyter_display_wait = wait_jupyter_display_observation(
        args.socket,
        jupyter_display_repl,
        "daemon-profile-smoke-jupyter-display-observe",
        jupyter_display_id,
        jupyter_display_expected_text,
        timeout_seconds=8.0,
        capability="passive_observation",
    )
    jupyter_display_request = jupyter_display_wait.get("request") or {}
    jupyter_display_response = jupyter_display_wait.get("response") or {}
    jupyter_display_observation = (
        jupyter_display_wait.get("observation") or {}
    )
    jupyter_display_evidence = jupyter_display_response.get("evidence") or {}
    jupyter_display_effects = jupyter_display_response.get("effects") or {}
    jupyter_display_matching_ranges = [
        item
        for item in jupyter_display_observation.get("display_ranges") or []
        if isinstance(item, dict) and item.get("display_id_matches") is True
    ]
    jupyter_display_matching_range_texts = [
        str(item.get("text_excerpt") or "")
        for item in jupyter_display_matching_ranges
    ]
    all_requests.append(jupyter_display_request)

    org_buffer = str(org_fixture.get("buffer_name") or "")
    org_tmp = str(org_fixture.get("tmp") or "")
    org_publish_dir = str(org_fixture.get("publish_dir") or "")
    org_export_file = str(org_fixture.get("export_file") or "")
    org_observe_request, org_observe_response = send(
        "daemon-profile-smoke-org-observe-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 800,
        },
        timeout=4.0,
    )
    org_observation = response_result(org_observe_response)
    org_heading = first_dict(org_observation.get("headings"))
    org_source = first_dict(org_observation.get("source_blocks"))
    org_update_preconditions = org_preconditions(org_observation, org_heading)
    org_update_request, org_update_response = send(
        "daemon-profile-smoke-org-update-1",
        "org.update_heading",
        "reversible_editor_edit",
        {
            "buffer_name": org_buffer,
            "heading": org_heading,
            "changes": {
                "todo": "DONE",
                "title": "Profile Smoke Summary",
            },
        },
        org_update_preconditions,
        timeout=4.0,
    )
    org_after_update_request, org_after_update_response = send(
        "daemon-profile-smoke-org-after-update-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 800,
        },
        timeout=4.0,
    )
    org_after_update = response_result(org_after_update_response)
    org_after_heading = first_dict(org_after_update.get("headings"))
    org_after_source = first_dict(org_after_update.get("source_blocks")) or org_source
    org_edit_preconditions = org_preconditions(
        org_after_update,
        org_after_heading,
        org_after_source,
    )
    org_edit_src_request, org_edit_src_response = send(
        "daemon-profile-smoke-org-edit-src-1",
        "org.edit_src_block",
        "editor_state_change",
        {
            "buffer_name": org_buffer,
            "source_block": org_after_source,
        },
        org_edit_preconditions,
        timeout=4.0,
    )
    org_edit_src_result = response_result(org_edit_src_response)
    org_edit_src_effect = (org_edit_src_response.get("effects") or {}).get(
        "source_edit_buffer",
        {},
    )
    org_edit_buffer = str(org_edit_src_result.get("edit_buffer_name") or "")
    org_edit_buffer_text_read = full_buffer_text_v0(
        args.socket,
        org_edit_buffer,
        "daemon-profile-smoke-org-edit-buffer",
        timeout=4.0,
    )
    all_requests.extend(
        [
            org_edit_buffer_text_read.get("observe_request") or {},
            org_edit_buffer_text_read.get("read_request") or {},
        ]
    )
    org_edit_buffer_observation = org_edit_buffer_text_read.get("observation") or {}
    org_edit_buffer_text = str(org_edit_buffer_text_read.get("text") or "")
    org_source_edit_text = '(princ "profile-source-edit\\n")\n'
    org_edit_replace_preconditions = dict(org_edit_buffer_observation)
    org_edit_replace_preconditions["expected_text"] = org_edit_buffer_text
    org_edit_replace_request, org_edit_replace_response = send(
        "daemon-profile-smoke-org-edit-buffer-replace-1",
        "buffer.replace_range",
        "reversible_editor_edit",
        {
            "buffer_name": org_edit_buffer,
            "start": org_edit_buffer_observation.get("point_min") or 1,
            "end": org_edit_buffer_observation.get("point_max") or 1,
            "text": org_source_edit_text,
        },
        org_edit_replace_preconditions,
        timeout=4.0,
    )
    org_edit_replace_result = response_result(org_edit_replace_response)
    org_exit_preconditions = {
        "edit_buffer_name": org_edit_buffer,
        "edit_buffer_chars_tick": (
            (org_edit_replace_result.get("after") or {}).get("chars_tick")
        ),
        "edit_text_sha256": sha256_text(org_source_edit_text),
        "parent_buffer_name": org_buffer,
        "parent_text_owner_name": org_after_update.get("text_owner_name"),
        "parent_file_name": org_after_update.get("buffer_file_name"),
        "parent_chars_tick": org_after_update.get("chars_tick"),
        "parent_structure_fingerprint": org_after_update.get(
            "structure_fingerprint"
        ),
        "parent_markers": org_edit_src_result.get("parent_markers"),
        "source_block": org_after_source,
    }
    org_exit_src_request, org_exit_src_response = send(
        "daemon-profile-smoke-org-exit-src-1",
        "org.exit_src_edit",
        "reversible_editor_edit",
        {"edit_buffer_name": org_edit_buffer},
        org_exit_preconditions,
        timeout=4.0,
    )
    org_exit_src_effect = (org_exit_src_response.get("effects") or {}).get(
        "parent_buffer_writeback",
        {},
    )
    org_after_exit_request, org_after_exit_response = send(
        "daemon-profile-smoke-org-after-exit-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
        timeout=4.0,
    )
    org_after_exit = response_result(org_after_exit_response)
    org_after_exit_heading = first_dict(org_after_exit.get("headings"))
    org_after_exit_source = (
        first_dict(org_after_exit.get("source_blocks")) or org_after_source
    )
    org_after_exit_text = full_buffer_text_v0(
        args.socket,
        org_buffer,
        "daemon-profile-smoke-org-after-exit-text",
        timeout=4.0,
    )
    all_requests.extend(
        [
            org_after_exit_text.get("observe_request") or {},
            org_after_exit_text.get("read_request") or {},
        ]
    )
    org_babel_preconditions = org_preconditions(
        org_after_exit,
        org_after_exit_heading,
        org_after_exit_source,
    )
    org_babel_preconditions.update(
        {
            "language_allowlist": ["emacs-lisp"],
            "default_directory": str(pathlib.Path(org_tmp).resolve()) + "/"
            if org_tmp
            else "",
            "session": False,
        }
    )
    org_babel_request, org_babel_response = send(
        "daemon-profile-smoke-org-babel-1",
        "org.execute_babel_block",
        "external_process_or_kernel_execution",
        {
            "buffer_name": org_buffer,
            "source_block": org_after_exit_source,
        },
        org_babel_preconditions,
        timeout=8.0,
    )
    org_after_babel_request, org_after_babel_response = send(
        "daemon-profile-smoke-org-after-babel-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
        timeout=4.0,
    )
    org_after_babel = response_result(org_after_babel_response)
    org_export_preconditions = {
        "buffer_name": org_buffer,
        "chars_tick": org_after_babel.get("chars_tick"),
        "publish_directory": org_publish_dir,
        "export_options": {},
    }
    org_export_request, org_export_response = send(
        "daemon-profile-smoke-org-export-1",
        "org.export_html",
        "filesystem_write",
        {
            "buffer_name": org_buffer,
            "output_path": org_export_file,
        },
        org_export_preconditions,
        timeout=8.0,
    )
    org_export_text = read_text(org_export_file)

    dired_buffer = str(dired_fixture.get("buffer_name") or "")
    dired_tmp = str(dired_fixture.get("tmp") or "")
    dired_source = str(dired_fixture.get("source") or "")
    dired_target = str(dired_fixture.get("target") or "")
    dired_source_path = str(dired_fixture.get("source_path") or "")
    dired_target_path = str(dired_fixture.get("target_path") or "")
    dired_decoy_path = str(dired_fixture.get("decoy_path") or "")
    dired_observe_request, dired_observe_response = send(
        "daemon-profile-smoke-dired-observe-1",
        "dired.observe",
        "passive_observation",
        {"buffer_name": dired_buffer, "max_files": 50},
        timeout=4.0,
    )
    dired_rename_preconditions = {
        "buffer_name": dired_buffer,
        "directory": str(pathlib.Path(dired_tmp).resolve()) + "/"
        if dired_tmp
        else "",
        "file_signature": file_signature(dired_source_path),
        "target_absent": True,
        "collision_policy": "no_overwrite",
    }
    dired_rename_request, dired_rename_response = send(
        "daemon-profile-smoke-dired-rename-1",
        "dired.rename_no_overwrite",
        "filesystem_write",
        {
            "buffer_name": dired_buffer,
            "source": dired_source,
            "target": dired_target,
        },
        dired_rename_preconditions,
        timeout=6.0,
    )
    dired_delete_buffer = str(dired_delete_fixture.get("buffer_name") or "")
    dired_delete_tmp = str(dired_delete_fixture.get("tmp") or "")
    dired_delete_marked = dired_delete_fixture.get("marked_files") or []
    dired_delete_a_path = str(dired_delete_fixture.get("delete_a_path") or "")
    dired_delete_b_path = str(dired_delete_fixture.get("delete_b_path") or "")
    dired_delete_flagged_decoy_path = str(
        dired_delete_fixture.get("flagged_decoy_path") or ""
    )
    dired_delete_observe_request, dired_delete_observe_response = send(
        "daemon-profile-smoke-dired-delete-observe-1",
        "dired.observe",
        "passive_observation",
        {"buffer_name": dired_delete_buffer, "max_files": 50},
        timeout=4.0,
    )
    dired_delete_preconditions = {
        "buffer_name": dired_delete_buffer,
        "directory": str(pathlib.Path(dired_delete_tmp).resolve()) + "/"
        if dired_delete_tmp
        else "",
        "marked_files": dired_delete_marked,
        "file_signatures": [
            {
                "name": "delete-a.txt",
                "signature": file_signature(dired_delete_a_path),
            },
            {
                "name": "delete-b.txt",
                "signature": file_signature(dired_delete_b_path),
            },
        ],
        "delete_policy": {
            "selection": "marked",
            "delete_by_moving_to_trash": False,
            "recursive_deletes": False,
        },
    }
    dired_delete_request, dired_delete_response = send(
        "daemon-profile-smoke-dired-delete-marked-1",
        "dired.delete_marked",
        "filesystem_write",
        {"buffer_name": dired_delete_buffer},
        dired_delete_preconditions,
        timeout=6.0,
    )

    magit_repo = str(magit_fixture.get("repo") or "")
    magit_buffer = str(magit_fixture.get("status_buffer") or "")
    magit_hook_log = str(magit_fixture.get("hook_log") or "")
    magit_initial_state = magit_fixture_repo_state(magit_repo, magit_hook_log)
    magit_observe_request, magit_observe_response = send(
        "daemon-profile-smoke-magit-observe-1",
        "magit.observe_sections",
        "passive_observation",
        {
            "buffer_name": magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
        timeout=3.0,
    )
    magit_observation = response_result(magit_observe_response)
    magit_section = find_magit_section(
        magit_observation,
        section_type="hunk",
        needle="target-05-selected",
        ancestor_file="target.txt",
    )
    magit_stage_preconditions_payload = magit_stage_preconditions(
        magit_observation,
        magit_section,
    )
    magit_stage_request, magit_stage_response = send(
        "daemon-profile-smoke-magit-stage-hunk-1",
        "magit.stage_hunk",
        "git_index_write",
        {
            "buffer_name": magit_buffer,
            "section": magit_section,
        },
        magit_stage_preconditions_payload,
        timeout=6.0,
    )
    magit_final_state = magit_fixture_repo_state(magit_repo, magit_hook_log)
    magit_file_repo = str(magit_file_fixture.get("repo") or "")
    magit_file_buffer = str(magit_file_fixture.get("status_buffer") or "")
    magit_file_hook_log = str(magit_file_fixture.get("hook_log") or "")
    magit_file_initial_state = magit_fixture_repo_state(
        magit_file_repo,
        magit_file_hook_log,
    )
    magit_file_observe_request, magit_file_observe_response = send(
        "daemon-profile-smoke-magit-file-observe-1",
        "magit.observe_sections",
        "passive_observation",
        {
            "buffer_name": magit_file_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
        timeout=3.0,
    )
    magit_file_observation = response_result(magit_file_observe_response)
    magit_file_section = find_magit_section(
        magit_file_observation,
        section_type="file",
        needle="target.txt",
    )
    magit_file_preconditions = magit_stage_preconditions(
        magit_file_observation,
        magit_file_section,
    )
    magit_file_stage_request, magit_file_stage_response = send(
        "daemon-profile-smoke-magit-stage-file-1",
        "magit.stage_file",
        "git_index_write",
        {
            "buffer_name": magit_file_buffer,
            "section": magit_file_section,
        },
        magit_file_preconditions,
        timeout=6.0,
    )
    magit_file_result = response_result(magit_file_stage_response)
    magit_file_effects = magit_file_stage_response.get("effects") or {}
    magit_file_git_effect = (
        magit_file_effects.get("git_index_update")
        if isinstance(magit_file_effects, dict)
        else {}
    ) or {}
    magit_file_final_state = magit_fixture_repo_state(
        magit_file_repo,
        magit_file_hook_log,
    )

    used_operations = [
        str(item.get("operation") or "")
        for item in all_requests
        if isinstance(item, dict) and item.get("operation")
    ]
    used_operation_set = sorted(set(used_operations))
    forbidden_operations = {
        "command.dispatch",
        "emacs.goto_char",
        "episode.finish",
        "direct-call",
        "elisp.eval",
        "process.write",
        "git.index_write",
        "filesystem.rename",
        "jupyter.restart_kernel",
        "jupyter.query_state",
        "jupyter.eval_region_via_repl",
        "bridge.defer_echo",
        "bridge.prompt_echo",
        "request.respond",
    }
    operation_entries = {
        operation: find_manifest_operation(manifest, operation) or {}
        for operation in used_operation_set
    }
    covered_profiles = sorted(
        {
            str(entry.get("profile"))
            for entry in operation_entries.values()
            if entry.get("profile")
        }
    )
    non_stable_operations = {
        operation: entry
        for operation, entry in operation_entries.items()
        if not manifest_operation_is_stable(manifest, operation)
    }
    first_stable_profiles = {
        "core-v0",
        "coding-v0",
        "terminal-v0",
        "analysis-v0",
        "reports-v0",
        "vcs-files-v0",
    }
    first_profile_stable_operations = sorted(
        str(entry.get("operation"))
        for entry in manifest.get("operations") or []
        if isinstance(entry, dict)
        and entry.get("profile") in first_stable_profiles
        and entry.get("maturity") == "stable"
        and entry.get("implemented") is True
        and entry.get("enabled") is True
        and entry.get("operation")
    )
    missing_first_profile_stable_operations = sorted(
        set(first_profile_stable_operations) - set(used_operation_set)
    )
    requests_missing_v0 = [
        item
        for item in all_requests
        if isinstance(item, dict)
        and item.get("operation")
        and item.get("api_version") != "soma-emacs-agent-v0"
    ]
    project_search_result = response_result(project_search_response)
    project_search_results = project_search_result.get("results") or []
    lsp_workspace_effect = (lsp_final_status.get("effects") or {}).get(
        "workspace_edit",
        {},
    )
    jupyter_await_status = jupyter_await_result.get("status") or {}
    org_update_effect = (org_update_response.get("effects") or {}).get(
        "org_heading_update",
        {},
    )
    org_babel_effect = (org_babel_response.get("effects") or {}).get(
        "org_babel_execution",
        {},
    )
    org_export_effect = (org_export_response.get("effects") or {}).get(
        "html_export",
        {},
    )
    dired_rename_effect = (dired_rename_response.get("effects") or {}).get(
        "filesystem_rename",
        {},
    )
    dired_delete_effect = (dired_delete_response.get("effects") or {}).get(
        "filesystem_delete",
        {},
    )
    magit_stage_effect = (magit_stage_response.get("effects") or {}).get(
        "git_index_update",
        {},
    )

    return scenario_record(
        39,
        "daemon-profile-stable-smoke",
        "The live daemon composes only stable profile operations across coding, terminal, analysis, reports, and VCS/file work",
        True,
        args.out_dir,
        fixture=fixture,
        used_operations=used_operations,
        used_operation_set=used_operation_set,
        operation_entries=operation_entries,
        covered_profiles=covered_profiles,
        first_profile_stable_operations=first_profile_stable_operations,
        missing_first_profile_stable_operations=missing_first_profile_stable_operations,
        forbidden_operations=sorted(forbidden_operations),
        manifest_request=manifest_request,
        manifest_response=manifest_response,
        describe_request=describe_request,
        describe_response=describe_response,
        session_request=session_request,
        session_response=session_response,
        status_request=status_request,
        status_response=status_response,
        frames_request=frames_request,
        frames_response=frames_response,
        interaction_request=interaction_request,
        interaction_response=interaction_response,
        project_observe_request=project_observe_request,
        project_observe_response=project_observe_response,
        project_search_request=project_search_request,
        project_search_response=project_search_response,
        code_select_request=code_select_request,
        code_select_response=code_select_response,
        code_observe_request=code_observe_request,
        code_observe_response=code_observe_response,
        completion_observe_request=completion_observe_request,
        completion_observe_response=completion_observe_response,
        completion_accept_request=completion_accept_request,
        completion_accept_response=completion_accept_response,
        code_after_completion_select_request=code_after_completion_select_request,
        code_after_completion_select_response=code_after_completion_select_response,
        code_after_completion=code_after_completion,
        code_edit_request=code_edit_request,
        code_edit_response=code_edit_response,
        code_after_edit_select_request=code_after_edit_select_request,
        code_after_edit_select_response=code_after_edit_select_response,
        code_after_edit=code_after_edit,
        code_patch_request=code_patch_request,
        code_patch_response=code_patch_response,
        code_after_patch_select_request=code_after_patch_select_request,
        code_after_patch_select_response=code_after_patch_select_response,
        code_after_patch=code_after_patch,
        code_save_request=code_save_request,
        code_save_response=code_save_response,
        code_final_text=code_final_text,
        lsp_select_request=lsp_select_request,
        lsp_select_response=lsp_select_response,
        lsp_observe_request=lsp_observe_request,
        lsp_observe_response=lsp_observe_response,
        lsp_diagnostics_request=lsp_diagnostics_request,
        lsp_diagnostics_response=lsp_diagnostics_response,
        lsp_rename_request=lsp_rename_request,
        lsp_rename_response=lsp_rename_response,
        lsp_await_request=lsp_await_request,
        lsp_await_response=lsp_await_response,
        lsp_final_status=lsp_final_status,
        lsp_build_test=lsp_test,
        lsp_final_files={
            "definition": lsp_definition_text,
            "main": lsp_main_text,
            "test": lsp_test_text,
        },
        comint_observe_request=comint_observe_request,
        comint_observe_response=comint_observe_response,
        comint_submit_request=comint_submit_request,
        comint_submit_response=comint_submit_response,
        comint_await_request=comint_await_request,
        comint_await_response=comint_await_response,
        comint_counter_lines=comint_counter_lines,
        comint_interrupt_fixture=comint_interrupt_fixture,
        comint_interrupt_observe_request=comint_interrupt_observe_request,
        comint_interrupt_observe_response=comint_interrupt_observe_response,
        comint_interrupt_submit_request=comint_interrupt_submit_request,
        comint_interrupt_submit_response=comint_interrupt_submit_response,
        comint_interrupt_await_start_request=comint_interrupt_await_start_request,
        comint_interrupt_await_start_response=comint_interrupt_await_start_response,
        comint_interrupt_before_request=comint_interrupt_before_request,
        comint_interrupt_before_response=comint_interrupt_before_response,
        comint_interrupt_request=comint_interrupt_request,
        comint_interrupt_response=comint_interrupt_response,
        comint_interrupt_final_request=comint_interrupt_final_request,
        comint_interrupt_final_response=comint_interrupt_final_response,
        comint_interrupt_counter_lines=comint_interrupt_counter_lines,
        jupyter_attach_request=jupyter_attach_request,
        jupyter_attach_response=jupyter_attach_response,
        jupyter_select_request=jupyter_select_request,
        jupyter_select_response=jupyter_select_response,
        jupyter_observe_request=jupyter_observe_request,
        jupyter_observe_response=jupyter_observe_response,
        jupyter_eval_preconditions=jupyter_eval_preconditions_payload,
        jupyter_eval_request=jupyter_eval_request,
        jupyter_eval_response=jupyter_eval_response,
        jupyter_await_request=jupyter_await_request,
        jupyter_await_response=jupyter_await_response,
        jupyter_final_status=jupyter_final_status,
        jupyter_stream_payload=jupyter_stream_payload,
        jupyter_display_fixture=jupyter_display_fixture,
        jupyter_display_preparation=jupyter_display_preparation,
        jupyter_display_request=jupyter_display_request,
        jupyter_display_response=jupyter_display_response,
        jupyter_display_wait=jupyter_display_wait,
        jupyter_display_observation=jupyter_display_observation,
        jupyter_display_matching_ranges=jupyter_display_matching_ranges,
        org_observe_request=org_observe_request,
        org_observe_response=org_observe_response,
        org_update_request=org_update_request,
        org_update_response=org_update_response,
        org_after_update_request=org_after_update_request,
        org_after_update_response=org_after_update_response,
        org_edit_src_request=org_edit_src_request,
        org_edit_src_response=org_edit_src_response,
        org_edit_buffer_text_read=org_edit_buffer_text_read,
        org_edit_replace_request=org_edit_replace_request,
        org_edit_replace_response=org_edit_replace_response,
        org_exit_src_request=org_exit_src_request,
        org_exit_src_response=org_exit_src_response,
        org_after_exit_request=org_after_exit_request,
        org_after_exit_response=org_after_exit_response,
        org_after_exit_text=org_after_exit_text,
        org_babel_request=org_babel_request,
        org_babel_response=org_babel_response,
        org_after_babel_request=org_after_babel_request,
        org_after_babel_response=org_after_babel_response,
        org_export_request=org_export_request,
        org_export_response=org_export_response,
        org_export_text_excerpt=org_export_text[:1200],
        dired_observe_request=dired_observe_request,
        dired_observe_response=dired_observe_response,
        dired_rename_request=dired_rename_request,
        dired_rename_response=dired_rename_response,
        dired_delete_fixture=dired_delete_fixture,
        dired_delete_observe_request=dired_delete_observe_request,
        dired_delete_observe_response=dired_delete_observe_response,
        dired_delete_request=dired_delete_request,
        dired_delete_response=dired_delete_response,
        magit_initial_state=magit_initial_state,
        magit_observe_request=magit_observe_request,
        magit_observe_response=magit_observe_response,
        magit_stage_request=magit_stage_request,
        magit_stage_response=magit_stage_response,
        magit_final_state=magit_final_state,
        magit_file_fixture=magit_file_fixture,
        magit_file_initial_state=magit_file_initial_state,
        magit_file_observe_request=magit_file_observe_request,
        magit_file_observe_response=magit_file_observe_response,
        magit_file_stage_request=magit_file_stage_request,
        magit_file_stage_response=magit_file_stage_response,
        magit_file_final_state=magit_file_final_state,
        checks=[
            check("profile smoke fixture is present", bool(fixture), fixture),
            check("profile smoke covers all first stable profiles", all(profile in covered_profiles for profile in ["core-v0", "coding-v0", "terminal-v0", "analysis-v0", "reports-v0", "vcs-files-v0"]), covered_profiles),
            check("profile smoke covers every stable operation in the first six profiles", missing_first_profile_stable_operations == [] and len(first_profile_stable_operations) == 41, {"stable_operation_count": len(first_profile_stable_operations), "missing": missing_first_profile_stable_operations}),
            check("profile smoke uses only manifest-stable operations", non_stable_operations == {}, non_stable_operations),
            check("profile smoke avoids benchmark-only and command escape operations", set(used_operation_set).isdisjoint(forbidden_operations), used_operation_set),
            check("profile smoke requests all use v0 request envelopes", requests_missing_v0 == [], requests_missing_v0),
            check("profile smoke describes a stable operation through the v0 envelope", describe_response.get("state") == "succeeded" and description.get("operation") == "emacs.observe_interaction" and description.get("profile") == "core-v0" and description.get("maturity") == "stable" and description_request_contract.get("envelope") == "soma-emacs-agent-v0.request" and description_result_contract.get("envelope") == "soma-emacs-agent-v0.result", {"response": describe_response, "description": description}),
            check("profile smoke session observation reports selected rootfs", session_response.get("state") == "succeeded" and (session_result.get("rootfs") or {}).get("mode") == "selected-container-rootfs", session_result),
            check("profile smoke request.status observes a stable completed request", status_response.get("state") == "succeeded" and status_result.get("operation") == "request.status" and stored_session_status.get("request_id") == "daemon-profile-smoke-session-1" and stored_session_status.get("state") == "succeeded", {"response": status_response, "status": stored_session_status}),
            check("profile smoke frame observation publishes passive typed frame evidence", frames_response.get("state") == "succeeded" and (frames_evidence.get("frame_identity") or {}).get("emacs_pid") == (ready or {}).get("emacs_pid") and isinstance(frames_observations.get("frames"), list) and frames_observations.get("frame_summary", {}).get("frame_count") == frames_result.get("frame_count") and frames_effects == {}, {"response": frames_response, "result": frames_result, "evidence": frames_evidence, "observations": frames_observations, "effects": frames_effects}),
            check("profile smoke interaction observation publishes passive command-loop evidence", interaction_response.get("state") == "succeeded" and interaction_result.get("operation") == "emacs.observe_interaction" and isinstance(interaction_result.get("selected_window_buffer"), str) and interaction_evidence.get("minibuffer", {}).get("active") is False and interaction_evidence.get("transient", {}).get("active") is False and interaction_observations.get("interaction", {}).get("selected_window_buffer") == interaction_result.get("selected_window_buffer") and interaction_effects == {}, {"response": interaction_response, "result": interaction_result, "evidence": interaction_evidence, "observations": interaction_observations, "effects": interaction_effects}),
            check("profile smoke stable buffer.select establishes owner buffers", all(response.get("state") == "succeeded" for response in [code_select_response, code_after_completion_select_response, code_after_edit_select_response, code_after_patch_select_response, lsp_select_response, jupyter_select_response]), {"code": code_select_response, "code_after_completion": code_after_completion_select_response, "code_after_edit": code_after_edit_select_response, "code_after_patch": code_after_patch_select_response, "lsp": lsp_select_response, "jupyter": jupyter_select_response}),
            check("profile smoke project inspection succeeds", project_observe_response.get("state") == "succeeded" and (response_result(project_observe_response).get("vcs") or {}).get("type") == "git", project_observe_response),
            check("profile smoke project search finds the source-context needle", project_search_response.get("state") == "succeeded" and any(search_needle in str(item.get("text") or "") for item in project_search_results), project_search_result),
            check("profile smoke completion observes checked candidate", completion_observe_response.get("state") == "succeeded" and code_completion in (completion_observation.get("candidates") or []), completion_observation),
            check("profile smoke checked completion accept succeeds", completion_accept_response.get("state") == "succeeded", completion_accept_response),
            check("profile smoke guarded buffer edit succeeds", code_edit_response.get("state") == "succeeded" and isinstance(edit_start, int) and isinstance(edit_end, int), {"response": code_edit_response, "edit_start": edit_start, "edit_end": edit_end}),
            check("profile smoke checked buffer patch succeeds", code_patch_response.get("state") == "succeeded" and ((code_patch_response.get("effects") or {}).get("changed_ranges") or []), code_patch_response),
            check("profile smoke edited and patched source buffer is saved to its visited file", code_save_response.get("state") == "succeeded" and "answer = 42" in code_final_text and code_edit_new in code_final_text and code_edit_old not in code_final_text and code_patch_new in code_final_text and code_patch_old not in code_final_text, code_final_text),
            check("profile smoke LSP source observation uses rootfs-owned server", lsp_observe_response.get("state") == "succeeded" and rootfs_owned_process_path(str(lsp_observation.get("server_executable") or "")), lsp_observation),
            check("profile smoke LSP diagnostics observation uses rootfs-owned server", lsp_diagnostics_response.get("state") == "succeeded" and lsp_diagnostics_observation.get("operation") == "lsp.observe_diagnostics" and rootfs_owned_process_path(str(lsp_diagnostics_observation.get("server_executable") or "")) and isinstance(lsp_diagnostic_count(lsp_diagnostics_observation), int), lsp_diagnostics_observation),
            check("profile smoke LSP diagnostics publishes typed passive evidence", isinstance(lsp_diagnostics_evidence, dict) and isinstance(lsp_diagnostics_evidence.get("workspace"), dict) and isinstance(lsp_diagnostics_evidence.get("server"), dict) and isinstance(lsp_diagnostics_evidence.get("document"), dict) and isinstance(lsp_diagnostics_evidence.get("diagnostics"), dict) and isinstance(lsp_diagnostics_evidence.get("diagnostic_cache"), dict) and isinstance(lsp_diagnostics_evidence.get("lsp_sync"), dict) and isinstance(lsp_diagnostics_evidence.get("rootfs_runtime"), dict) and isinstance(lsp_diagnostics_observations.get("source_buffer"), dict) and lsp_diagnostics_observations.get("source_buffer", {}).get("buffer_name") == lsp_buffer and isinstance(lsp_diagnostics_observations.get("document"), dict) and isinstance(lsp_diagnostics_observations.get("diagnostics"), dict) and lsp_diagnostics_effects == {}, {"response": lsp_diagnostics_response, "evidence": lsp_diagnostics_evidence, "observations": lsp_diagnostics_observations, "effects": lsp_diagnostics_effects}),
            check("profile smoke LSP rename reaches terminal success", lsp_await_response.get("state") == "succeeded" and lsp_final_status.get("state") == "succeeded", {"rename": lsp_rename_response, "await": lsp_await_response, "final": lsp_final_status}),
            check("profile smoke LSP rename applies workspace edit", lsp_rename_result.get("route") == "lsp-rename" and lsp_workspace_effect.get("applied") is True and lsp_new_name in lsp_definition_text and lsp_symbol not in lsp_definition_text and lsp_test.get("returncode") == 0, {"result": lsp_rename_result, "effect": lsp_workspace_effect, "test": lsp_test}),
            check("profile smoke Comint observe sees rootfs-owned process", comint_observe_response.get("state") == "succeeded" and rootfs_owned_process_path(str(comint_observation.get("process_exe") or "")), comint_observation),
            check("profile smoke Comint submit and await succeed exactly once", comint_submit_response.get("state") == "succeeded" and comint_await_response.get("state") == "succeeded" and comint_await_result.get("matched") is True and comint_counter_lines == [comint_label], {"submit": comint_submit_response, "await": comint_await_response, "counter": comint_counter_lines}),
            check("profile smoke Comint interrupt fixture is present", bool(comint_interrupt_buffer and comint_interrupt_counter and comint_interrupt_command), comint_interrupt_fixture),
            check("profile smoke Comint interrupt observes rootfs-owned process", comint_interrupt_observe_response.get("state") == "succeeded" and rootfs_owned_process_path(str(comint_interrupt_observation.get("process_exe") or "")), comint_interrupt_observation),
            check("profile smoke Comint interrupt command starts through Comint", comint_interrupt_submit_response.get("state") == "succeeded" and comint_interrupt_submit_result.get("route") == "comint-send-input" and comint_interrupt_await_start_response.get("state") == "succeeded" and comint_interrupt_await_start_result.get("matched") is True and has_exact_line(str(comint_interrupt_await_start_result.get("matched_output") or ""), comint_interrupt_start_needle), {"submit": comint_interrupt_submit_response, "await": comint_interrupt_await_start_response}),
            check("profile smoke comint.interrupt uses semantic process control", comint_interrupt_response.get("state") == "succeeded" and comint_interrupt_result.get("route") == "comint-interrupt-subjob" and comint_interrupt_result.get("interrupt_dispatched") is True and comint_interrupt_result.get("used_raw_process_write") is False and comint_interrupt_result.get("bridge_used_raw_process_control") is False, comint_interrupt_response),
            check("profile smoke comint.interrupt publishes typed evidence and effect", isinstance(comint_interrupt_evidence, dict) and comint_interrupt_evidence.get("interrupt_command") == "comint-interrupt-subjob" and isinstance(comint_interrupt_process_identity, dict) and comint_interrupt_process_identity.get("process_id") == comint_interrupt_before.get("process_id") and isinstance(comint_interrupt_process_survival, dict) and comint_interrupt_process_survival.get("process_live_after") is True and comint_interrupt_evidence.get("side_effect_reconciliation") == "external_verifier_required" and isinstance(comint_interrupt_process_effect, dict) and comint_interrupt_process_effect.get("command") == "comint-interrupt-subjob" and comint_interrupt_process_effect.get("dispatched") is True and comint_interrupt_process_effect.get("used_raw_process_write") is False and comint_interrupt_process_effect.get("bridge_used_raw_process_control") is False, {"evidence": comint_interrupt_evidence, "effects": comint_interrupt_effects}),
            check("profile smoke Comint interrupt prevents delayed side effect", comint_interrupt_final_response.get("state") == "succeeded" and comint_interrupt_final.get("process_id") == comint_interrupt_observation.get("process_id") and comint_interrupt_final.get("process_live") is True and not has_exact_line(comint_interrupt_tail, comint_interrupt_done_needle) and comint_interrupt_counter_lines == [], {"final": comint_interrupt_final_response, "counter": comint_interrupt_counter_lines, "tail": comint_interrupt_tail}),
            check("profile smoke Jupyter attach uses existing rootfs-owned kernel", jupyter_attach_response.get("state") == "succeeded" and rootfs_owned_process_path(str(jupyter_fixture.get("kernel_exe") or "")), {"fixture": jupyter_fixture, "response": jupyter_attach_response}),
            check("profile smoke Jupyter selected-region eval completes without training cell", jupyter_await_response.get("state") == "succeeded" and jupyter_await_status.get("state") == "succeeded" and jupyter_stream_payload.get("kernel_token") == jupyter_fixture.get("kernel_token") and jupyter_stream_payload.get("training_executions") == 0 and jupyter_training_code not in str(jupyter_eval_result.get("submitted_code") or "") and jupyter_eval_result.get("submitted_code") == jupyter_selected_code, {"await": jupyter_await_response, "payload": jupyter_stream_payload, "eval": jupyter_eval_result}),
            check("profile smoke Jupyter display fixture is prepared by a real REPL display/update", bool(jupyter_display_repl and jupyter_display_id) and jupyter_display_preparation_evidence.get("state") == "succeeded" and jupyter_display_preparation_evidence.get("route") == "jupyter-repl-execute-cell" and len(jupyter_display_preparation_evidence.get("display_events") or []) == 1 and len(jupyter_display_update_events) == 1, {"fixture": jupyter_display_fixture, "preparation": jupyter_display_preparation_evidence}),
            check("profile smoke jupyter.observe_display reads prepared display state through stable API", jupyter_display_response.get("state") == "succeeded" and jupyter_display_wait.get("found") is True and jupyter_display_observation.get("display_id_found") is True and bool(jupyter_display_matching_ranges) and any(jupyter_display_expected_text in text for text in jupyter_display_matching_range_texts), {"response": jupyter_display_response, "observation": jupyter_display_observation, "expected_text": jupyter_display_expected_text, "matching_ranges": jupyter_display_matching_ranges}),
            check("profile smoke jupyter.observe_display publishes typed passive evidence without effects", isinstance(jupyter_display_evidence, dict) and jupyter_display_evidence.get("display_id") == jupyter_display_id and isinstance(jupyter_display_evidence.get("display_data"), dict) and jupyter_display_evidence["display_data"].get("display_id_found") is True and isinstance(jupyter_display_evidence.get("repl_range"), dict) and jupyter_display_evidence["repl_range"].get("buffer_name") == jupyter_display_repl and jupyter_display_effects == {}, {"evidence": jupyter_display_evidence, "effects": jupyter_display_effects}),
            check("profile smoke Org source edit opens a separate edit buffer", org_edit_src_response.get("state") == "succeeded" and org_edit_src_effect.get("route") == "org-edit-src-code" and org_edit_src_effect.get("source_edit_buffer_separate") is True and isinstance(org_edit_buffer, str) and bool(org_edit_buffer), {"response": org_edit_src_response, "effect": org_edit_src_effect}),
            check("profile smoke Org source edit buffer is replaced through checked buffer editing", org_edit_replace_response.get("state") == "succeeded" and contains(org_source_edit_text, "profile-source-edit") and ((org_edit_replace_response.get("effects") or {}).get("changed_range") or {}), org_edit_replace_response),
            check("profile smoke Org source edit exits through Org and writes back to parent", org_exit_src_response.get("state") == "succeeded" and org_exit_src_effect.get("route") == "org-edit-src-exit" and org_exit_src_effect.get("edit_buffer_live_after") is False and contains(str(org_after_exit_text.get("text") or ""), "profile-source-edit"), {"response": org_exit_src_response, "effect": org_exit_src_effect, "after": org_after_exit_response, "text": org_after_exit_text}),
            check("profile smoke Org update executes Babel and exports HTML", org_update_response.get("state") == "succeeded" and org_update_effect.get("route") == "org-update-heading" and org_babel_response.get("state") == "succeeded" and org_babel_effect.get("route") == "org-babel-execute-src-block" and org_export_response.get("state") == "succeeded" and org_export_effect.get("route") == "org-export" and pathlib.Path(org_export_file).exists() and "Profile Smoke Summary" in org_export_text, {"update": org_update_response, "babel": org_babel_response, "export": org_export_response, "html": org_export_text[:400]}),
            check("profile smoke Dired rename mutates only requested filesystem target", dired_observe_response.get("state") == "succeeded" and dired_rename_response.get("state") == "succeeded" and dired_rename_effect.get("route") == "dired-rename-file" and not pathlib.Path(dired_source_path).exists() and pathlib.Path(dired_target_path).exists() and read_text(dired_target_path) == "source\n" and read_text(dired_decoy_path) == "decoy\n", {"observe": dired_observe_response, "rename": dired_rename_response, "effect": dired_rename_effect}),
            check("profile smoke Dired delete mutates only ordinary marked files", dired_delete_observe_response.get("state") == "succeeded" and dired_delete_response.get("state") == "succeeded" and dired_delete_effect.get("route") == "dired-do-delete" and dired_delete_effect.get("marked_files") == ["delete-a.txt", "delete-b.txt"] and not pathlib.Path(dired_delete_a_path).exists() and not pathlib.Path(dired_delete_b_path).exists() and pathlib.Path(dired_delete_flagged_decoy_path).exists() and read_text(dired_delete_flagged_decoy_path) == "flagged-decoy\n", {"observe": dired_delete_observe_response, "delete": dired_delete_response, "effect": dired_delete_effect}),
            check("profile smoke Magit stages only the intended hunk", magit_observe_response.get("state") == "succeeded" and magit_stage_response.get("state") == "succeeded" and magit_stage_effect.get("route") == "magit-stage" and magit_stage_effect.get("section_type") == "hunk" and magit_final_state.get("cached_names") == ["staged.txt", "target.txt"] and magit_final_state.get("unstaged_names") == ["debug.txt", "target.txt"] and contains(magit_final_state.get("target_index") or "", "target-05-selected") and contains(magit_final_state.get("target_worktree") or "", "target-25-other"), {"initial": magit_initial_state, "final": magit_final_state, "effect": magit_stage_effect}),
            check("profile smoke Magit stages a full file section through Magit", magit_file_observe_response.get("state") == "succeeded" and magit_file_stage_response.get("state") == "succeeded" and magit_file_result.get("route") == "magit-stage" and magit_file_result.get("this_command") == "magit-stage" and magit_file_git_effect.get("route") == "magit-stage" and magit_file_git_effect.get("section_type") == "file" and magit_file_final_state.get("cached_names") == ["staged.txt", "target.txt"] and magit_file_final_state.get("unstaged_names") == ["debug.txt"] and contains(magit_file_final_state.get("target_index") or "", "target-05-selected") and not contains(magit_file_final_state.get("target_worktree") or "", "target-25-other"), {"initial": magit_file_initial_state, "final": magit_file_final_state, "effect": magit_file_git_effect, "response": magit_file_stage_response}),
        ],
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--socket", required=True, type=pathlib.Path)
    parser.add_argument("--out-dir", required=True, type=pathlib.Path)
    parser.add_argument("--plan", required=True, type=pathlib.Path)
    parser.add_argument("--ready", required=True, type=pathlib.Path)
    parser.add_argument("--daemon-name", required=True)
    parser.add_argument("--terminal-frame-baseline", type=int)
    parser.add_argument("--terminal-tmux-socket", default="")
    parser.add_argument("--terminal-tmux-session", default="")
    parser.add_argument("--terminal-pane-capture", type=pathlib.Path)
    parser.add_argument("--terminal-key-pane-capture", type=pathlib.Path)
    parser.add_argument("--buffer-name", default="soma-bridge-daemon-target")
    args = parser.parse_args()

    args.out_dir.mkdir(parents=True, exist_ok=True)
    plan = read_json_file(args.plan)
    ready = read_json_file(args.ready)
    records: list[dict[str, Any]] = []

    observe_request = {
        "request_id": "daemon-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": args.buffer_name},
    }
    observe_response = request(args.socket, observe_request)
    observe_result = observe_response.get("result") or {}
    observe_evidence = observe_response.get("evidence") or {}
    observe_observations = observe_response.get("observations") or {}
    observe_effects = observe_response.get("effects") or {}
    observe_buffer_identity = observe_evidence.get("buffer_identity") or {}
    observe_text_owner = observe_evidence.get("text_owner") or {}
    observe_point_region = observe_evidence.get("point_region_narrowing") or {}
    observe_buffer_state = observe_observations.get("buffer_state") or {}
    records.append(
        scenario_record(
            1,
            "daemon-external-observe",
            "A process outside Emacs observes a buffer through the live daemon JSONL bridge",
            True,
            args.out_dir,
            request=observe_request,
            response=observe_response,
            client_process={
                "pid": os.getpid(),
                "executable": sys.executable,
                "transport": "AF_UNIX JSONL",
                "emacs_daemon_name": args.daemon_name,
            },
            checks=[
                check("request id is echoed", observe_response.get("request_id") == "daemon-observe-1", observe_response),
                check("request succeeded", observe_response.get("state") == "succeeded", observe_response),
                check("observe result names daemon buffer", observe_result.get("buffer_name") == args.buffer_name, observe_result),
                check("daemon emacs.observe_buffer publishes typed buffer identity and state sections", observe_buffer_identity.get("buffer_name") == args.buffer_name and observe_text_owner.get("text_owner_name") == args.buffer_name and isinstance(observe_point_region.get("point_min"), int) and isinstance(observe_point_region.get("point_max"), int) and isinstance(observe_buffer_state.get("major_mode"), str) and observe_effects == {}, {"evidence": observe_evidence, "observations": observe_observations, "effects": observe_effects}),
                check("observe ran in daemon transport", ready and ready.get("daemonp") is True, ready),
            ],
        )
    )

    wrong_owner_select_request = v0_request(
        "daemon-buffer-wrong-text-owner-select-1",
        "buffer.select",
        "editor_state_change",
        {"buffer_name": args.buffer_name},
    )
    wrong_owner_select_response = request_or_outcome_unknown(
        args.socket,
        wrong_owner_select_request,
    )
    wrong_owner_before = full_buffer_text_v0(
        args.socket,
        args.buffer_name,
        "daemon-buffer-wrong-text-owner-before",
    )
    wrong_owner_before_text = str(wrong_owner_before.get("text") or "")
    wrong_owner_preconditions = dict(wrong_owner_before.get("observation") or {})
    wrong_owner_actual_owner = str(
        wrong_owner_preconditions.get("text_owner_name") or ""
    )
    wrong_owner_preconditions["text_owner_name"] = (
        f"{wrong_owner_actual_owner}-wrong-owner"
    )
    wrong_owner_preconditions["expected_text"] = ""
    wrong_owner_request = v0_request(
        "daemon-buffer-wrong-text-owner-replace-1",
        "buffer.replace_range",
        "reversible_editor_edit",
        {
            "buffer_name": args.buffer_name,
            "start": 1,
            "end": 1,
            "text": "SOMA_DAEMON_WRONG_TEXT_OWNER_MUTATED\n",
        },
        wrong_owner_preconditions,
    )
    wrong_owner_response = request_or_outcome_unknown(
        args.socket,
        wrong_owner_request,
    )
    wrong_owner_after = full_buffer_text_v0(
        args.socket,
        args.buffer_name,
        "daemon-buffer-wrong-text-owner-after",
    )
    wrong_owner_after_text = str(wrong_owner_after.get("text") or "")
    records.append(
        scenario_record(
            51,
            "daemon-buffer-wrong-text-owner-rejected",
            "buffer.replace_range rejects wrong text-owner preconditions before mutating a live daemon buffer",
            True,
            args.out_dir,
            select_request=wrong_owner_select_request,
            select_response=wrong_owner_select_response,
            before=wrong_owner_before,
            replace_request=wrong_owner_request,
            replace_response=wrong_owner_response,
            after=wrong_owner_after,
            checks=[
                check(
                    "wrong text-owner fixture observes a concrete owner",
                    wrong_owner_select_response.get("state") == "succeeded"
                    and wrong_owner_before.get("observe_response", {}).get("state")
                    == "succeeded"
                    and bool(wrong_owner_actual_owner),
                    {
                        "select": wrong_owner_select_response,
                        "before": wrong_owner_before.get("observation"),
                    },
                ),
                check(
                    "buffer.replace_range rejects the wrong text owner",
                    wrong_owner_response.get("state") == "failed"
                    and wrong_owner_response.get("error_code") == "conflict"
                    and precondition_check_passed(
                        wrong_owner_response,
                        "text owner matches precondition",
                        False,
                    ),
                    wrong_owner_response,
                ),
                check(
                    "wrong text-owner rejection publishes no text mutation effect",
                    not ((wrong_owner_response.get("effects") or {}).get("changed_range")),
                    wrong_owner_response,
                ),
                check(
                    "wrong text-owner rejection preserves buffer text",
                    wrong_owner_before_text == wrong_owner_after_text
                    and "SOMA_DAEMON_WRONG_TEXT_OWNER_MUTATED"
                    not in wrong_owner_after_text,
                    {
                        "before": wrong_owner_before_text,
                        "after": wrong_owner_after_text,
                    },
                ),
            ],
        )
    )

    duplicate_before = full_buffer_text_v0(
        args.socket,
        args.buffer_name,
        "daemon-duplicate-mutating-before",
    )
    duplicate_before_text = str(duplicate_before.get("text") or "")
    duplicate_observation = duplicate_before.get("observation") or {}
    duplicate_insert_at = (
        duplicate_observation.get("point_max")
        if isinstance(duplicate_observation.get("point_max"), int)
        else 1
    )
    duplicate_preconditions = dict(duplicate_observation)
    duplicate_preconditions["expected_text"] = ""
    duplicate_request_id = "daemon-duplicate-mutating-replace-1"
    duplicate_first_marker = "SOMA_DAEMON_DUPLICATE_FIRST\n"
    duplicate_second_marker = "SOMA_DAEMON_DUPLICATE_SECOND\n"
    duplicate_first_request = v0_request(
        duplicate_request_id,
        "buffer.replace_range",
        "reversible_editor_edit",
        {
            "buffer_name": args.buffer_name,
            "start": duplicate_insert_at,
            "end": duplicate_insert_at,
            "text": duplicate_first_marker,
        },
        duplicate_preconditions,
    )
    duplicate_first_response = request_or_outcome_unknown(
        args.socket,
        duplicate_first_request,
    )
    duplicate_second_request = v0_request(
        duplicate_request_id,
        "buffer.replace_range",
        "reversible_editor_edit",
        {
            "buffer_name": args.buffer_name,
            "start": duplicate_insert_at,
            "end": duplicate_insert_at,
            "text": duplicate_second_marker,
        },
        duplicate_preconditions,
    )
    duplicate_second_response = request_or_outcome_unknown(
        args.socket,
        duplicate_second_request,
    )
    duplicate_status_request = v0_request(
        "daemon-duplicate-mutating-status-1",
        "request.status",
        "passive_observation",
        {"request_id": duplicate_request_id},
    )
    duplicate_status_response = request_or_outcome_unknown(
        args.socket,
        duplicate_status_request,
    )
    duplicate_status = (
        response_result(duplicate_status_response).get("status") or {}
    )
    duplicate_after = full_buffer_text_v0(
        args.socket,
        args.buffer_name,
        "daemon-duplicate-mutating-after",
    )
    duplicate_after_text = str(duplicate_after.get("text") or "")
    records.append(
        scenario_record(
            53,
            "daemon-duplicate-mutating-request-id-rejected",
            "Duplicate request IDs for live daemon mutating operations are rejected without resubmitting editor edits",
            True,
            args.out_dir,
            before=duplicate_before,
            first_request=duplicate_first_request,
            first_response=duplicate_first_response,
            second_request=duplicate_second_request,
            second_response=duplicate_second_response,
            status_request=duplicate_status_request,
            status_response=duplicate_status_response,
            after=duplicate_after,
            checks=[
                check(
                    "duplicate request fixture observes an insertion point",
                    duplicate_before.get("observe_response", {}).get("state")
                    == "succeeded"
                    and isinstance(duplicate_insert_at, int),
                    {
                        "before": duplicate_observation,
                        "insert_at": duplicate_insert_at,
                    },
                ),
                check(
                    "first mutating request with unique id succeeds",
                    duplicate_first_response.get("state") == "succeeded"
                    and bool(
                        (duplicate_first_response.get("effects") or {}).get(
                            "changed_range"
                        )
                    ),
                    duplicate_first_response,
                ),
                check(
                    "second mutating request with duplicate id is rejected before execution",
                    duplicate_second_response.get("state") == "failed"
                    and duplicate_second_response.get("error_code")
                    == "duplicate_request_id"
                    and not (
                        (duplicate_second_response.get("effects") or {}).get(
                            "changed_range"
                        )
                    ),
                    duplicate_second_response,
                ),
                check(
                    "duplicate rejection does not replace the stored successful response",
                    duplicate_status_response.get("state") == "succeeded"
                    and duplicate_status.get("request_id") == duplicate_request_id
                    and duplicate_status.get("operation") == "buffer.replace_range"
                    and duplicate_status.get("state") == "succeeded",
                    {
                        "status_response": duplicate_status_response,
                        "stored_status": duplicate_status,
                    },
                ),
                check(
                    "duplicate request does not replay or apply conflicting mutation",
                    duplicate_after_text.count(duplicate_first_marker)
                    == duplicate_before_text.count(duplicate_first_marker) + 1
                    and duplicate_after_text.count(duplicate_second_marker)
                    == duplicate_before_text.count(duplicate_second_marker),
                    {
                        "before": duplicate_before_text,
                        "after": duplicate_after_text,
                    },
                ),
            ],
        )
    )

    frames_request = {
        "request_id": "daemon-terminal-frames-1",
        "operation": "emacs.observe_frames",
        "params": {},
    }
    frames_response = request(args.socket, frames_request)
    frames_result = frames_response.get("result") or {}
    frames = frames_result.get("frames") or []
    terminal_frames = [
        frame
        for frame in frames
        if frame.get("graphic") is False and frame.get("terminal_live") is True
    ]
    pane_text = ""
    if args.terminal_pane_capture and args.terminal_pane_capture.exists():
        pane_text = args.terminal_pane_capture.read_text(encoding="utf-8", errors="replace")
    rootfs_emacsclient = ((plan.get("tool_inventory") or {}).get("emacsclient") or {})
    records.append(
        scenario_record(
            2,
            "daemon-terminal-frame",
            "A rootfs-owned emacsclient --tty creates a terminal frame attached to the live daemon",
            True,
            args.out_dir,
            request=frames_request,
            response=frames_response,
            terminal={
                "tmux_socket": args.terminal_tmux_socket,
                "tmux_session": args.terminal_tmux_session,
                "terminal_frame_baseline": args.terminal_frame_baseline,
                "pane_capture": str(args.terminal_pane_capture or ""),
                "pane_capture_sample": pane_text[:400],
            },
            checks=[
                check("frame observation request succeeded", frames_response.get("state") == "succeeded", frames_response),
                check("frame observation ran in daemon", frames_result.get("daemonp") is True, frames_result),
                check("frame observation server name matches daemon", frames_result.get("server_name") == args.daemon_name, frames_result),
                check("frame observation pid matches daemon bootstrap", frames_result.get("emacs_pid") == ready.get("emacs_pid"), {"frames": frames_result.get("emacs_pid"), "ready": ready.get("emacs_pid")}),
                check("emacsclient path is rootfs-owned in plan", plan.get("emacsclient") == "/opt/soma/emacs/bin/emacsclient", plan.get("emacsclient", "")),
                check("tool inventory includes rootfs emacsclient", rootfs_emacsclient.get("path") == "/opt/soma/emacs/bin/emacsclient", rootfs_emacsclient),
                check("terminal frame count increased after emacsclient --tty", args.terminal_frame_baseline is not None and frames_result.get("terminal_frame_count", 0) > args.terminal_frame_baseline, {"baseline": args.terminal_frame_baseline, "observed": frames_result.get("terminal_frame_count")}),
                check("at least one live nongraphic terminal frame exists", len(terminal_frames) >= 1, frames),
                check("at least one frame is owned by an emacsclient", frames_result.get("client_frame_count", 0) >= 1, frames_result.get("client_frame_count")),
                check("tmux session name was recorded", bool(args.terminal_tmux_session), args.terminal_tmux_session),
                check("terminal pane capture is non-empty", bool(pane_text.strip()), str(args.terminal_pane_capture or "")),
            ],
        )
    )

    records.append(run_profile_stable_smoke(args, ready, plan))

    key_target = args.buffer_name
    tmux_target = f"{args.terminal_tmux_session}:0.0"
    key_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        tmux_target,
        key_target,
        "daemon-terminal-key",
    )
    target_result = {
        "literal": key_switch["buffer_keys"],
        "enter": key_switch["enter"],
        "returncode": 0
        if key_switch["buffer_keys"]["returncode"] == 0
        and key_switch["enter"]["returncode"] == 0
        else 1,
    }
    key_frames_response = key_switch["response"]
    key_frames_result = key_switch["result"]
    key_terminal_frames = key_switch["terminal_frames"]

    key_capture_result = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", tmux_target, "-p"]
    )
    key_pane_text = key_capture_result["stdout"]
    key_capture_path = args.terminal_key_pane_capture or (args.out_dir / "terminal-key-dispatch-pane.txt")
    key_capture_path.write_text(key_pane_text, encoding="utf-8", errors="replace")
    records.append(
        scenario_record(
            3,
            "daemon-terminal-key-dispatch",
            "Real terminal key input switches the daemon-owned emacsclient frame to the target buffer",
            True,
            args.out_dir,
            terminal_key_sequence=["Escape", "C-g", "C-x", "b", key_target, "Enter"],
            tmux_target=tmux_target,
            tmux_escape=key_switch["escape"],
            tmux_cancel=key_switch["cancel"],
            cancel_attempts=key_switch["cancel_attempts"],
            cancel_interaction_response=key_switch["cancel_interaction_response"],
            tmux_prefix=key_switch["prefix"],
            minibuffer_attempts=key_switch["minibuffer_attempts"],
            minibuffer_response=key_switch["minibuffer_response"],
            tmux_target_submit=target_result,
            observation_attempts=key_switch["observation_attempts"],
            response=key_frames_response,
            terminal={
                "pane_capture": str(key_capture_path),
                "pane_capture_sample": key_pane_text[:400],
            },
            checks=[
                check("tmux escape key send succeeded", key_switch["escape"]["returncode"] == 0, key_switch["escape"]),
                check("tmux cancel key send succeeded", key_switch["cancel"]["returncode"] == 0, key_switch["cancel"]),
                check("tmux switch-buffer prefix send succeeded", key_switch["prefix"]["returncode"] == 0, key_switch["prefix"]),
                check("switch-buffer minibuffer became active before target text", key_switch["minibuffer_response"].get("state") == "succeeded" and (key_switch["minibuffer_response"].get("result") or {}).get("minibuffer_active") is True, key_switch["minibuffer_attempts"]),
                check("tmux target buffer submit succeeded", target_result["returncode"] == 0, target_result),
                check("frame observation after key dispatch succeeded", key_frames_response.get("state") == "succeeded", key_frames_response),
                check("client terminal frame displays target buffer after keys", len(key_terminal_frames) >= 1, key_frames_result.get("frames")),
                check("terminal pane capture includes target buffer", key_target in key_pane_text, str(key_capture_path)),
            ],
        )
    )

    status_request = {
        "request_id": "daemon-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-observe-1"},
    }
    status_response = request(args.socket, status_request)
    status_result = status_response.get("result") or {}
    stored_status = status_result.get("status") or {}
    records.append(
        scenario_record(
            4,
            "daemon-request-status",
            "A process outside Emacs can retrieve stored terminal request status from the live daemon",
            True,
            args.out_dir,
            request=status_request,
            response=status_response,
            checks=[
                check("status request succeeded", status_response.get("state") == "succeeded", status_response),
                check("status result identifies status operation", status_result.get("operation") == "request.status", status_result),
                check("stored request id matches observed request", stored_status.get("request_id") == "daemon-observe-1", stored_status),
                check("stored request is terminal succeeded", stored_status.get("state") == "succeeded", stored_status),
            ],
        )
    )

    direct_request = {
        "request_id": "daemon-direct-1",
        "operation": "direct-call",
        "params": {"requested_operation": "magit-stage-1"},
    }
    elisp_eval_request = {
        "request_id": "daemon-elisp-eval-1",
        "operation": "elisp.eval",
        "params": {"form": "(write-region \"bad\" nil \"/project/tmp/soma-elisp-eval-ran\")"},
    }
    raw_process_fixture = (ready or {}).get("comint_fixture") or {}
    raw_process_buffer = str(raw_process_fixture.get("buffer_name") or "")
    raw_process_counter = str(raw_process_fixture.get("counter") or "")
    raw_process_label = "raw-process-write"
    raw_process_input = (
        f"printf '{raw_process_label}\\n' >> {shlex.quote(raw_process_counter)}\n"
    )
    raw_process_before_lines = file_lines(raw_process_counter)
    raw_process_write_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-raw-process-write-1",
        "operation": "process.write",
        "capability": "unauthorized_operation",
        "params": {
            "buffer_name": raw_process_buffer,
            "input": raw_process_input,
        },
    }
    command_request = {
        "request_id": "daemon-command-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": args.buffer_name,
            "command": "keyboard-quit",
            "preconditions": {"buffer_name": args.buffer_name},
        },
    }
    command_response = request(args.socket, command_request)
    direct_response = request(args.socket, direct_request)
    elisp_eval_response = request(args.socket, elisp_eval_request)
    raw_process_write_response = request(args.socket, raw_process_write_request)
    time.sleep(0.1)
    raw_process_after_lines = file_lines(raw_process_counter)
    records.append(
        scenario_record(
            5,
            "daemon-command-dispatch-rejected",
            "The live daemon transport rejects command.dispatch for a non-allowlisted command",
            True,
            args.out_dir,
            request=command_request,
            response=command_response,
            checks=[
                check("command dispatch is rejected", command_response.get("state") == "failed", command_response),
                check("command rejection reports operation failure", command_response.get("error_code") == "operation_failed", command_response),
                check("command precondition rejected allowlist", precondition_check_passed(command_response, "command is allowlisted", False), command_response),
                check("non-allowlisted command was interactive", precondition_check_passed(command_response, "command is interactive", True), command_response),
            ],
        )
    )

    records.append(
        scenario_record(
            6,
            "daemon-raw-process-write-rejected",
            "The live daemon transport rejects raw process writes before subprocess side effects",
            True,
            args.out_dir,
            fixture=raw_process_fixture,
            request=raw_process_write_request,
            response=raw_process_write_response,
            counter_before=raw_process_before_lines,
            counter_after=raw_process_after_lines,
            checks=[
                check("raw-process rejection fixture is present", bool(raw_process_buffer and raw_process_counter), raw_process_fixture),
                check("raw process write operation is rejected", raw_process_write_response.get("state") == "failed", raw_process_write_response),
                check("raw process write rejection uses unauthorized code", raw_process_write_response.get("error_code") == "unauthorized_operation", raw_process_write_response),
                check("raw process write result reports no raw write", ((raw_process_write_response.get("result") or {}).get("used_raw_process_write") is False) and ((raw_process_write_response.get("result") or {}).get("sent_bytes") == 0), raw_process_write_response),
                check("raw process write side effect did not occur", raw_process_after_lines == raw_process_before_lines and raw_process_label not in raw_process_after_lines, {"before": raw_process_before_lines, "after": raw_process_after_lines}),
            ],
        )
    )

    records.append(
        scenario_record(
            6,
            "daemon-direct-call-rejected",
            "The live daemon transport rejects unchecked direct-call operations",
            True,
            args.out_dir,
            request=direct_request,
            response=direct_response,
            checks=[
                check("direct operation is rejected", direct_response.get("state") == "failed", direct_response),
                check("direct rejection uses unauthorized code", direct_response.get("error_code") == "unauthorized_operation", direct_response),
            ],
        )
    )

    records.append(
        scenario_record(
            7,
            "daemon-elisp-eval-rejected",
            "The live daemon transport rejects arbitrary Elisp evaluation",
            True,
            args.out_dir,
            request=elisp_eval_request,
            response=elisp_eval_response,
            checks=[
                check("elisp eval operation is rejected", elisp_eval_response.get("state") == "failed", elisp_eval_response),
                check("elisp eval rejection uses unauthorized code", elisp_eval_response.get("error_code") == "unauthorized_operation", elisp_eval_response),
                check("elisp eval result reports no evaluation", ((elisp_eval_response.get("result") or {}).get("evaluated") is False) and (elisp_eval_response.get("effects") or {}) == {}, elisp_eval_response),
            ],
        )
    )

    semantic_bypass_text = "daemon bridge buffer\nsemantic rename result\n"
    semantic_bypass_edit = replace_range_via_bridge(
        args.socket,
        args.buffer_name,
        1,
        len("daemon bridge buffer\n") + 1,
        semantic_bypass_text,
        "daemon bridge buffer\n",
        "daemon-semantic-bypass-text-edit",
    )
    semantic_bypass_finish_request = {
        "request_id": "daemon-semantic-bypass-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-semantic-bypass-lsp-rename-1",
                    "state": "succeeded",
                }
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": args.buffer_name,
                    "start": 1,
                    "end": len(semantic_bypass_text) + 1,
                    "preconditions": {
                        "expected_text": semantic_bypass_text,
                    },
                }
            ],
        },
    }
    semantic_bypass_finish_response = request(
        args.socket,
        semantic_bypass_finish_request,
    )
    semantic_bypass_finish_result = (
        semantic_bypass_finish_response.get("result") or {}
    )
    semantic_bypass_request_checks = (
        semantic_bypass_finish_result.get("request_checks") or []
    )
    semantic_bypass_buffer_checks = (
        semantic_bypass_finish_result.get("buffer_checks") or []
    )
    records.append(
        scenario_record(
            40,
            "daemon-semantic-bypass-rejected",
            "episode.finish rejects direct text substitution when a semantic LSP request is required",
            True,
            args.out_dir,
            text_edit=semantic_bypass_edit,
            finish_request=semantic_bypass_finish_request,
            finish_response=semantic_bypass_finish_response,
            checks=[
                check("semantic-bypass checked text edit succeeds", (semantic_bypass_edit.get("replace_response") or {}).get("state") == "succeeded", semantic_bypass_edit),
                check("semantic-bypass episode finish fails without semantic request evidence", semantic_bypass_finish_response.get("state") == "failed" and semantic_bypass_finish_result.get("request_checks_passed") is False, semantic_bypass_finish_response),
                check("semantic-bypass missing request id is reported", any(item.get("request_id") == "daemon-semantic-bypass-lsp-rename-1" and item.get("passed") is False and item.get("actual_state") is False and item.get("detail") == "request id is not known" for item in semantic_bypass_request_checks), semantic_bypass_request_checks),
                check("semantic-bypass final buffer text alone matches", semantic_bypass_finish_result.get("buffer_checks_passed") is True and any(item.get("buffer_name") == args.buffer_name and item.get("passed") is True for item in semantic_bypass_buffer_checks), semantic_bypass_buffer_checks),
            ],
        )
    )

    async_request = {
        "request_id": "daemon-async-1",
        "operation": "bridge.defer_echo",
        "params": {
            "delay_seconds": 0.5,
            "value": "daemon async ok",
        },
    }
    async_response = request(args.socket, async_request)
    async_status_request = {
        "request_id": "daemon-async-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-async-1"},
    }
    async_status_response = request(args.socket, async_status_request)
    async_status_result = async_status_response.get("result") or {}
    async_running_status = async_status_result.get("status") or {}
    await_request = {
        "request_id": "daemon-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-async-1",
            "timeout_seconds": 2.0,
        },
    }
    await_response = request(args.socket, await_request, timeout=3.0)
    await_result = await_response.get("result") or {}
    awaited_status = await_result.get("status") or {}
    final_status_request = {
        "request_id": "daemon-async-final-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-async-1"},
    }
    final_status_response = request(args.socket, final_status_request)
    final_status_result = final_status_response.get("result") or {}
    final_status = final_status_result.get("status") or {}
    records.append(
        scenario_record(
            7,
            "daemon-async-status-await",
            "The live daemon transport exposes an in-flight request, then awaits its terminal status without resubmission",
            True,
            args.out_dir,
            request=async_request,
            response=async_response,
            running_status_request=async_status_request,
            running_status_response=async_status_response,
            await_request=await_request,
            await_response=await_response,
            final_status_request=final_status_request,
            final_status_response=final_status_response,
            checks=[
                check("async request is accepted as running", async_response.get("state") == "running", async_response),
                check("async request records accepted running sequence", async_response.get("state_sequence") == ["accepted", "running"], async_response),
                check("status observes in-flight request", async_running_status.get("state") == "running", async_running_status),
                check("await request succeeds", await_response.get("state") == "succeeded", await_response),
                check("awaited status is original request", awaited_status.get("request_id") == "daemon-async-1", awaited_status),
                check("awaited status is terminal succeeded", awaited_status.get("state") == "succeeded", awaited_status),
                check("awaited result preserves echoed value", (awaited_status.get("result") or {}).get("value") == "daemon async ok", awaited_status),
                check("final status remains terminal succeeded", final_status.get("state") == "succeeded", final_status),
            ],
        )
    )

    timeout_async_request = {
        "request_id": "daemon-timeout-async-1",
        "operation": "bridge.defer_echo",
        "params": {
            "delay_seconds": 0.7,
            "value": "timeout later ok",
        },
    }
    timeout_async_response = request(args.socket, timeout_async_request)
    timeout_await_request = {
        "request_id": "daemon-timeout-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-timeout-async-1",
            "timeout_seconds": 0.05,
        },
    }
    timeout_await_response = request(args.socket, timeout_await_request, timeout=1.0)
    timeout_await_result = timeout_await_response.get("result") or {}
    timeout_status = timeout_await_result.get("status") or {}
    timeout_reconcile_request = {
        "request_id": "daemon-timeout-reconcile-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-timeout-async-1",
            "timeout_seconds": 2.0,
        },
    }
    timeout_reconcile_response = request(args.socket, timeout_reconcile_request, timeout=3.0)
    timeout_reconcile_result = timeout_reconcile_response.get("result") or {}
    reconciled_status = timeout_reconcile_result.get("status") or {}
    records.append(
        scenario_record(
            8,
            "daemon-await-timeout-outcome-unknown",
            "request.await reports outcome_unknown on timeout, then reconciles the original request without resubmission",
            True,
            args.out_dir,
            request=timeout_async_request,
            response=timeout_async_response,
            timeout_await_request=timeout_await_request,
            timeout_await_response=timeout_await_response,
            reconcile_request=timeout_reconcile_request,
            reconcile_response=timeout_reconcile_response,
            checks=[
                check("timeout target request is accepted as running", timeout_async_response.get("state") == "running", timeout_async_response),
                check("short await reports outcome_unknown", timeout_await_response.get("state") == "outcome_unknown", timeout_await_response),
                check("timeout response uses timeout error code", timeout_await_response.get("error_code") == "timeout", timeout_await_response),
                check("timeout result references original request", timeout_status.get("request_id") == "daemon-timeout-async-1", timeout_status),
                check("timeout result leaves original request non-terminal", timeout_status.get("state") == "running", timeout_status),
                check("reconcile await succeeds later", timeout_reconcile_response.get("state") == "succeeded", timeout_reconcile_response),
                check("reconciled status is original request", reconciled_status.get("request_id") == "daemon-timeout-async-1", reconciled_status),
                check("reconciled original request is terminal succeeded", reconciled_status.get("state") == "succeeded", reconciled_status),
                check("reconciled result preserves original value", (reconciled_status.get("result") or {}).get("value") == "timeout later ok", reconciled_status),
            ],
        )
    )

    prompt_request = {
        "request_id": "daemon-prompt-1",
        "operation": "bridge.prompt_echo",
        "params": {
            "prompt": "confirm daemon prompt",
            "value_prefix": "confirmed:",
        },
    }
    prompt_response = request(args.socket, prompt_request)
    prompt_status_request = {
        "request_id": "daemon-prompt-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-prompt-1"},
    }
    prompt_status_response = request(args.socket, prompt_status_request)
    prompt_status_result = prompt_status_response.get("result") or {}
    waiting_status = prompt_status_result.get("status") or {}
    respond_request = {
        "request_id": "daemon-prompt-respond-1",
        "operation": "request.respond",
        "params": {
            "request_id": "daemon-prompt-1",
            "response": "yes",
        },
    }
    respond_response = request(args.socket, respond_request)
    respond_result = respond_response.get("result") or {}
    responded_status = respond_result.get("target_status") or {}
    prompt_await_request = {
        "request_id": "daemon-prompt-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-prompt-1",
            "timeout_seconds": 0.0,
        },
    }
    prompt_await_response = request(args.socket, prompt_await_request)
    prompt_await_result = prompt_await_response.get("result") or {}
    awaited_prompt_status = prompt_await_result.get("status") or {}
    records.append(
        scenario_record(
            9,
            "daemon-waiting-for-input-response",
            "A daemon request enters waiting_for_input and completes after an explicit external response",
            True,
            args.out_dir,
            request=prompt_request,
            response=prompt_response,
            status_request=prompt_status_request,
            status_response=prompt_status_response,
            respond_request=respond_request,
            respond_response=respond_response,
            await_request=prompt_await_request,
            await_response=prompt_await_response,
            checks=[
                check("prompt request enters waiting_for_input", prompt_response.get("state") == "waiting_for_input", prompt_response),
                check("prompt request records waiting sequence", prompt_response.get("state_sequence") == ["accepted", "running", "waiting_for_input"], prompt_response),
                check("status observes waiting_for_input", waiting_status.get("state") == "waiting_for_input", waiting_status),
                check("status preserves prompt text", (waiting_status.get("result") or {}).get("prompt") == "confirm daemon prompt", waiting_status),
                check("respond request succeeds", respond_response.get("state") == "succeeded", respond_response),
                check("response updates original request", responded_status.get("request_id") == "daemon-prompt-1", responded_status),
                check("responded original request is terminal succeeded", responded_status.get("state") == "succeeded", responded_status),
                check("responded result includes supplied response", (responded_status.get("result") or {}).get("prompt_response") == "yes", responded_status),
                check("responded result preserves value", (responded_status.get("result") or {}).get("value") == "confirmed:yes", responded_status),
                check("await sees original prompt request succeeded", awaited_prompt_status.get("state") == "succeeded", awaited_prompt_status),
                check("awaited prompt status is original request", awaited_prompt_status.get("request_id") == "daemon-prompt-1", awaited_prompt_status),
            ],
        )
    )

    terminal_prompt_target = f"{args.terminal_tmux_session}:0.0"
    terminal_prompt_cancel = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", terminal_prompt_target, "C-g"]
    )
    terminal_prompt_prefix = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", terminal_prompt_target, "C-x", "b"]
    )
    time.sleep(0.1)
    terminal_prompt_buffer_keys = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", terminal_prompt_target, args.buffer_name]
    )
    terminal_prompt_enter = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", terminal_prompt_target, "Enter"]
    )
    terminal_prompt_frame_attempts: list[dict[str, Any]] = []
    terminal_prompt_frames_response: dict[str, Any] = {}
    terminal_prompt_frames_result: dict[str, Any] = {}
    terminal_prompt_frames: list[dict[str, Any]] = []
    for attempt in range(1, 31):
        terminal_prompt_frames_request = {
            "request_id": f"daemon-terminal-prompt-frame-{attempt}",
            "operation": "emacs.observe_frames",
            "params": {},
        }
        terminal_prompt_frames_response = request(args.socket, terminal_prompt_frames_request)
        terminal_prompt_frames_result = terminal_prompt_frames_response.get("result") or {}
        frames = terminal_prompt_frames_result.get("frames") or []
        terminal_prompt_frames = [
            frame
            for frame in frames
            if frame.get("client_frame") is True
            and frame.get("graphic") is False
            and frame.get("terminal_live") is True
            and frame.get("selected_window_buffer") == args.buffer_name
        ]
        terminal_prompt_frame_attempts.append(
            {
                "attempt": attempt,
                "response_state": terminal_prompt_frames_response.get("state"),
                "client_terminal_buffers": [
                    frame.get("selected_window_buffer")
                    for frame in frames
                    if frame.get("client_frame") is True and frame.get("graphic") is False
                ],
            }
        )
        if terminal_prompt_frames:
            break
        time.sleep(0.1)

    terminal_prompt_observe_request = {
        "request_id": "daemon-terminal-prompt-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": args.buffer_name},
    }
    terminal_prompt_observe_response = request(args.socket, terminal_prompt_observe_request)
    terminal_prompt_observation = terminal_prompt_observe_response.get("result") or {}
    terminal_prompt_request = {
        "request_id": "daemon-terminal-prompt-command-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": args.buffer_name,
            "command": "soma-bridge-daemon-terminal-prompt-insert",
            "preconditions": terminal_prompt_observation,
        },
    }
    terminal_prompt_response = request_or_outcome_unknown(args.socket, terminal_prompt_request, timeout=2.0)
    terminal_prompt_status_request = {
        "request_id": "daemon-terminal-prompt-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-terminal-prompt-command-1"},
    }
    terminal_prompt_status_response: dict[str, Any] = {}
    terminal_prompt_running_status: dict[str, Any] = {}
    terminal_prompt_interaction_attempts: list[dict[str, Any]] = []
    terminal_prompt_interaction_response: dict[str, Any] = {}
    terminal_prompt_interaction_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        terminal_prompt_interaction_request = {
            "request_id": f"daemon-terminal-prompt-interaction-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        terminal_prompt_interaction_response = request_or_outcome_unknown(
            args.socket,
            terminal_prompt_interaction_request,
            timeout=0.5,
        )
        terminal_prompt_interaction_result = terminal_prompt_interaction_response.get("result") or {}
        terminal_prompt_interaction_attempts.append(
            {
                "attempt": attempt,
                "response_state": terminal_prompt_interaction_response.get("state"),
                "minibuffer_active": terminal_prompt_interaction_result.get("minibuffer_active"),
                "minibuffer_prompt": terminal_prompt_interaction_result.get("minibuffer_prompt"),
                "selected_window_buffer": terminal_prompt_interaction_result.get("selected_window_buffer"),
            }
        )
        if (
            terminal_prompt_interaction_response.get("state") == "succeeded"
            and terminal_prompt_interaction_result.get("minibuffer_active") is True
        ):
            break
        time.sleep(0.1)
    terminal_prompt_status_response = request_or_outcome_unknown(
        args.socket,
        terminal_prompt_status_request,
        timeout=0.5,
    )
    terminal_prompt_status_result = terminal_prompt_status_response.get("result") or {}
    terminal_prompt_running_status = terminal_prompt_status_result.get("status") or {}
    terminal_prompt_capture_before = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", terminal_prompt_target, "-p"]
    )
    terminal_prompt_input = "minibuffer-ok"
    terminal_prompt_input_keys = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", terminal_prompt_target, terminal_prompt_input]
    )
    terminal_prompt_submit = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", terminal_prompt_target, "Enter"]
    )
    terminal_prompt_await_request = {
        "request_id": "daemon-terminal-prompt-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-terminal-prompt-command-1",
            "timeout_seconds": 5.0,
        },
    }
    terminal_prompt_await_response = request_or_outcome_unknown(
        args.socket,
        terminal_prompt_await_request,
        timeout=6.0,
    )
    terminal_prompt_await_result = terminal_prompt_await_response.get("result") or {}
    terminal_prompt_final_status = (
        terminal_prompt_await_result.get("status")
        or terminal_prompt_running_status
        or terminal_prompt_response
    )
    terminal_prompt_result = terminal_prompt_final_status.get("result") or {}
    terminal_prompt_final_observe_request = {
        "request_id": "daemon-terminal-prompt-final-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": args.buffer_name},
    }
    terminal_prompt_final_observe_response = request_or_outcome_unknown(
        args.socket,
        terminal_prompt_final_observe_request,
        timeout=2.0,
    )
    terminal_prompt_final_observation = terminal_prompt_final_observe_response.get("result") or {}
    final_text = ""
    if terminal_prompt_final_observe_response.get("state") == "succeeded":
        final_read_request = {
            "request_id": "daemon-terminal-prompt-final-read-1",
            "operation": "buffer.read",
            "params": {
                "buffer_name": args.buffer_name,
                "start": 1,
                "end": max(1, int(terminal_prompt_final_observation.get("point") or 1)),
            },
        }
        final_read_response = request_or_outcome_unknown(args.socket, final_read_request, timeout=2.0)
        final_text = (final_read_response.get("result") or {}).get("text") or ""
    else:
        final_read_request = {
            "request_id": "daemon-terminal-prompt-final-read-1",
            "operation": "buffer.read",
            "params": {"buffer_name": args.buffer_name, "start": 1, "end": 1},
        }
        final_read_response = {
            "request_id": "daemon-terminal-prompt-final-read-1",
            "operation": "buffer.read",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_final_observation",
            "error": "final observation did not succeed",
        }
    final_read_evidence = final_read_response.get("evidence") or {}
    final_read_observations = final_read_response.get("observations") or {}
    final_read_effects = final_read_response.get("effects") or {}
    final_read_buffer_identity = final_read_evidence.get("buffer_identity") or {}
    final_read_range = final_read_evidence.get("text_range") or {}
    final_read_snapshot = final_read_observations.get("text_snapshot") or {}
    records.append(
        scenario_record(
            10,
            "daemon-terminal-minibuffer-command",
            "A queued daemon command opens a real terminal minibuffer and completes from tmux input",
            True,
            args.out_dir,
            terminal_switch={
                "tmux_target": terminal_prompt_target,
                "cancel": terminal_prompt_cancel,
                "prefix": terminal_prompt_prefix,
                "buffer_keys": terminal_prompt_buffer_keys,
                "enter": terminal_prompt_enter,
                "observation_attempts": terminal_prompt_frame_attempts,
                "response": terminal_prompt_frames_response,
            },
            observe_request=terminal_prompt_observe_request,
            observe_response=terminal_prompt_observe_response,
            request=terminal_prompt_request,
            response=terminal_prompt_response,
            interaction_attempts=terminal_prompt_interaction_attempts,
            interaction_response=terminal_prompt_interaction_response,
            status_request=terminal_prompt_status_request,
            status_response=terminal_prompt_status_response,
            terminal_before_input=terminal_prompt_capture_before,
            terminal_input=terminal_prompt_input,
            terminal_input_keys=terminal_prompt_input_keys,
            terminal_submit=terminal_prompt_submit,
            await_request=terminal_prompt_await_request,
            await_response=terminal_prompt_await_response,
            final_status=terminal_prompt_final_status,
            final_observe_request=terminal_prompt_final_observe_request,
            final_observe_response=terminal_prompt_final_observe_response,
            final_read_request=final_read_request,
            final_read_response=final_read_response,
            final_text=final_text,
            checks=[
                check("terminal prompt switch cancel succeeded", terminal_prompt_cancel["returncode"] == 0, terminal_prompt_cancel),
                check("terminal prompt switch prefix succeeded", terminal_prompt_prefix["returncode"] == 0, terminal_prompt_prefix),
                check("terminal prompt buffer text sent", terminal_prompt_buffer_keys["returncode"] == 0, terminal_prompt_buffer_keys),
                check("terminal prompt switch enter succeeded", terminal_prompt_enter["returncode"] == 0, terminal_prompt_enter),
                check("client terminal frame displays prompt target buffer", len(terminal_prompt_frames) >= 1, terminal_prompt_frames_result.get("frames")),
                check("terminal prompt observe succeeded", terminal_prompt_observe_response.get("state") == "succeeded", terminal_prompt_observe_response),
                check("terminal prompt command accepted as running", terminal_prompt_response.get("state") == "running", terminal_prompt_response),
                check("terminal prompt command uses queued command route", (terminal_prompt_response.get("result") or {}).get("route") == "queued-command-execute", terminal_prompt_response),
                check("interaction observation remains responsive during minibuffer", terminal_prompt_interaction_response.get("state") == "succeeded", terminal_prompt_interaction_response),
                check("real minibuffer is active", terminal_prompt_interaction_result.get("minibuffer_active") is True, terminal_prompt_interaction_result),
                check("minibuffer prompt is the command prompt", terminal_prompt_interaction_result.get("minibuffer_prompt") == "Soma terminal prompt: ", terminal_prompt_interaction_result),
                check("running command status remains observable during minibuffer", terminal_prompt_status_response.get("state") == "succeeded" and terminal_prompt_running_status.get("state") == "running", terminal_prompt_status_response),
                check("terminal pane shows minibuffer prompt", "Soma terminal prompt:" in terminal_prompt_capture_before["stdout"], terminal_prompt_capture_before),
                check("terminal minibuffer input was sent", terminal_prompt_input_keys["returncode"] == 0, terminal_prompt_input_keys),
                check("terminal minibuffer submit succeeded", terminal_prompt_submit["returncode"] == 0, terminal_prompt_submit),
                check("terminal prompt await reaches success", terminal_prompt_await_response.get("state") == "succeeded" and terminal_prompt_final_status.get("state") == "succeeded", terminal_prompt_await_response),
                check("terminal prompt ran through command-execute", terminal_prompt_result.get("route") == "command-execute", terminal_prompt_result),
                check("terminal prompt command identity is preserved", terminal_prompt_result.get("this_command") == "soma-bridge-daemon-terminal-prompt-insert", terminal_prompt_result),
                check("terminal prompt final observation succeeded", terminal_prompt_final_observe_response.get("state") == "succeeded", terminal_prompt_final_observe_response),
                check("terminal prompt final read succeeded", final_read_response.get("state") == "succeeded", final_read_response),
                check("daemon buffer.read publishes typed range and text snapshot sections", final_read_buffer_identity.get("buffer_name") == args.buffer_name and final_read_range.get("start") == 1 and isinstance(final_read_range.get("end"), int) and final_read_snapshot.get("text") == final_text and isinstance(final_read_snapshot.get("chars_tick"), int) and final_read_effects == {}, {"evidence": final_read_evidence, "observations": final_read_observations, "effects": final_read_effects}),
                check("terminal prompt input mutated target buffer", f"terminal-prompt:{terminal_prompt_input}" in final_text, final_text),
            ],
        )
    )

    terminal_magit_fixture = (ready or {}).get("terminal_magit_fixture") or {}
    terminal_magit_repo = terminal_magit_fixture.get("repo") or ""
    terminal_magit_buffer = terminal_magit_fixture.get("status_buffer") or ""
    terminal_magit_hook_log = terminal_magit_fixture.get("hook_log") or ""
    terminal_magit_preconditions = terminal_magit_fixture.get("preconditions") or {}
    terminal_magit_target_section = terminal_magit_fixture.get("target_section") or {}
    terminal_magit_target = f"{args.terminal_tmux_session}:0.0"
    terminal_magit_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        terminal_magit_target,
        terminal_magit_buffer,
        "daemon-terminal-magit",
    )
    terminal_magit_cancel = terminal_magit_switch["cancel"]
    terminal_magit_prefix = terminal_magit_switch["prefix"]
    terminal_magit_buffer_keys = terminal_magit_switch["buffer_keys"]
    terminal_magit_enter = terminal_magit_switch["enter"]
    terminal_magit_frame_attempts = terminal_magit_switch["observation_attempts"]
    terminal_magit_frames_response = terminal_magit_switch["response"]
    terminal_magit_frames_result = terminal_magit_switch["result"]
    terminal_magit_frames = terminal_magit_switch["terminal_frames"]

    terminal_magit_observe_request = {
        "request_id": "daemon-terminal-magit-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": terminal_magit_buffer},
    }
    terminal_magit_observe_response = request_or_outcome_unknown(
        args.socket,
        terminal_magit_observe_request,
        timeout=2.0,
    )
    terminal_magit_initial_observation = terminal_magit_observe_response.get("result") or {}
    terminal_magit_target_point = (
        terminal_magit_target_section.get("target_point")
        or terminal_magit_preconditions.get("point")
    )
    terminal_magit_section_start = terminal_magit_target_section.get("start")
    terminal_magit_section_end = terminal_magit_target_section.get("end")
    if isinstance(terminal_magit_target_point, int):
        terminal_magit_goto_request = {
            "request_id": "daemon-terminal-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": terminal_magit_buffer,
                "point": terminal_magit_target_point,
                "preconditions": terminal_magit_initial_observation,
            },
        }
        terminal_magit_goto_response = request_or_outcome_unknown(
            args.socket,
            terminal_magit_goto_request,
            timeout=2.0,
        )
    else:
        terminal_magit_goto_request = {
            "request_id": "daemon-terminal-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": terminal_magit_buffer,
                "point": None,
                "preconditions": terminal_magit_initial_observation,
            },
        }
        terminal_magit_goto_response = {
            "request_id": "daemon-terminal-magit-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_target_point",
            "error": "terminal Magit fixture did not provide a target point",
        }
    terminal_magit_ready_observe_request = {
        "request_id": "daemon-terminal-magit-ready-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": terminal_magit_buffer},
    }
    terminal_magit_ready_observe_response = request_or_outcome_unknown(
        args.socket,
        terminal_magit_ready_observe_request,
        timeout=2.0,
    )
    terminal_magit_ready_observation = terminal_magit_ready_observe_response.get("result") or {}
    terminal_magit_ready_point = terminal_magit_ready_observation.get("point")
    terminal_magit_point_in_target_section = (
        isinstance(terminal_magit_ready_point, int)
        and isinstance(terminal_magit_section_start, int)
        and isinstance(terminal_magit_section_end, int)
        and terminal_magit_section_start <= terminal_magit_ready_point < terminal_magit_section_end
    )
    terminal_magit_key = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", terminal_magit_target, "s"]
    )
    terminal_magit_poll: list[dict[str, Any]] = []
    terminal_magit_cached_names: list[str] = []
    terminal_magit_unstaged_names: list[str] = []
    terminal_magit_target_index = ""
    terminal_magit_target_worktree = ""
    terminal_magit_debug_index = ""
    terminal_magit_debug_worktree = ""
    terminal_magit_staged_index = ""
    terminal_magit_hook_log_text = ""
    for attempt in range(1, 51):
        time.sleep(0.1)
        terminal_magit_cached_names = git_lines(terminal_magit_repo, "diff", "--cached", "--name-only") if terminal_magit_repo else []
        terminal_magit_unstaged_names = git_lines(terminal_magit_repo, "diff", "--name-only") if terminal_magit_repo else []
        terminal_magit_target_index = git_output(terminal_magit_repo, "show", ":target.txt") if terminal_magit_repo else ""
        terminal_magit_target_worktree = read_text(str(pathlib.Path(terminal_magit_repo) / "target.txt")) if terminal_magit_repo else ""
        terminal_magit_debug_index = git_output(terminal_magit_repo, "show", ":debug.txt") if terminal_magit_repo else ""
        terminal_magit_debug_worktree = read_text(str(pathlib.Path(terminal_magit_repo) / "debug.txt")) if terminal_magit_repo else ""
        terminal_magit_staged_index = git_output(terminal_magit_repo, "show", ":staged.txt") if terminal_magit_repo else ""
        terminal_magit_hook_log_text = read_text(terminal_magit_hook_log)
        terminal_magit_poll.append(
            {
                "attempt": attempt,
                "cached_names": terminal_magit_cached_names,
                "unstaged_names": terminal_magit_unstaged_names,
                "hook_log": terminal_magit_hook_log_text,
            }
        )
        if (
            terminal_magit_cached_names == ["staged.txt", "target.txt"]
            and terminal_magit_unstaged_names == ["debug.txt", "target.txt"]
            and contains(terminal_magit_target_index, "target-05-selected")
            and contains(terminal_magit_hook_log_text, "post-stage:magit-stage")
        ):
            break
    terminal_magit_capture = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", terminal_magit_target, "-p"]
    )
    records.append(
        scenario_record(
            11,
            "daemon-terminal-magit-key-dispatch",
            "Real terminal key input stages one Magit hunk in the daemon terminal frame",
            True,
            args.out_dir,
            fixture=terminal_magit_fixture,
            terminal_switch={
                "tmux_target": terminal_magit_target,
                "cancel": terminal_magit_cancel,
                "cancel_attempts": terminal_magit_switch["cancel_attempts"],
                "prefix": terminal_magit_prefix,
                "minibuffer_attempts": terminal_magit_switch["minibuffer_attempts"],
                "buffer_keys": terminal_magit_buffer_keys,
                "enter": terminal_magit_enter,
                "observation_attempts": terminal_magit_frame_attempts,
                "response": terminal_magit_frames_response,
            },
            observe_request=terminal_magit_observe_request,
            observe_response=terminal_magit_observe_response,
            point_adjustment={
                "target_point": terminal_magit_target_point,
                "request": terminal_magit_goto_request,
                "response": terminal_magit_goto_response,
                "target_section": terminal_magit_target_section,
            },
            ready_observe_request=terminal_magit_ready_observe_request,
            ready_observe_response=terminal_magit_ready_observe_response,
            terminal_key_sequence=["s"],
            terminal_stage_key=terminal_magit_key,
            poll=terminal_magit_poll,
            terminal_capture=terminal_magit_capture,
            repo_state={
                "cached_names": terminal_magit_cached_names,
                "unstaged_names": terminal_magit_unstaged_names,
                "target_index": terminal_magit_target_index,
                "target_worktree": terminal_magit_target_worktree,
                "debug_index": terminal_magit_debug_index,
                "debug_worktree": terminal_magit_debug_worktree,
                "staged_index": terminal_magit_staged_index,
                "hook_log": terminal_magit_hook_log_text,
            },
            checks=[
                check("terminal Magit fixture is present", bool(terminal_magit_repo and terminal_magit_buffer), terminal_magit_fixture),
                check("terminal Magit switch cancel succeeded", terminal_magit_cancel["returncode"] == 0, terminal_magit_cancel),
                check("terminal Magit switch prefix succeeded", terminal_magit_prefix["returncode"] == 0, terminal_magit_prefix),
                check("terminal Magit buffer text sent", terminal_magit_buffer_keys["returncode"] == 0, terminal_magit_buffer_keys),
                check("terminal Magit switch enter succeeded", terminal_magit_enter["returncode"] == 0, terminal_magit_enter),
                check("client terminal frame displays terminal Magit buffer", len(terminal_magit_frames) >= 1, terminal_magit_frames_result.get("frames")),
                check("terminal Magit observation succeeded", terminal_magit_observe_response.get("state") == "succeeded", terminal_magit_observe_response),
                check("checked terminal Magit point navigation succeeded", terminal_magit_goto_response.get("state") == "succeeded", terminal_magit_goto_response),
                check("ready terminal Magit observation succeeded", terminal_magit_ready_observe_response.get("state") == "succeeded", terminal_magit_ready_observe_response),
                check("ready terminal Magit point remains inside target hunk", terminal_magit_point_in_target_section, {"target_section": terminal_magit_target_section, "observed": terminal_magit_ready_point}),
                check("terminal Magit stage key send succeeded", terminal_magit_key["returncode"] == 0, terminal_magit_key),
                check("terminal Magit cached names are exact", terminal_magit_cached_names == ["staged.txt", "target.txt"], terminal_magit_cached_names),
                check("terminal Magit unstaged names preserve unrelated work", terminal_magit_unstaged_names == ["debug.txt", "target.txt"], terminal_magit_unstaged_names),
                check("terminal Magit selected change is staged", contains(terminal_magit_target_index, "target-05-selected"), terminal_magit_target_index),
                check("terminal Magit unrelated hunk remains unstaged", contains(terminal_magit_target_index, "target-25-base") and contains(terminal_magit_target_worktree, "target-25-other"), {"index": terminal_magit_target_index, "worktree": terminal_magit_target_worktree}),
                check("terminal Magit debug change remains unstaged", terminal_magit_debug_index == "debug-base\n" and contains(terminal_magit_debug_worktree, "debug-worktree-only"), {"index": terminal_magit_debug_index, "worktree": terminal_magit_debug_worktree}),
                check("terminal Magit unrelated staged file is preserved", terminal_magit_staged_index == "staged-expected\n", terminal_magit_staged_index),
                check("terminal Magit post-stage hook observed command identity", contains(terminal_magit_hook_log_text, "post-stage:magit-stage"), terminal_magit_hook_log_text),
                check("terminal Magit refresh hook ran", contains(terminal_magit_hook_log_text, "refresh:"), terminal_magit_hook_log_text),
                check("terminal pane capture is from Magit buffer", terminal_magit_buffer in terminal_magit_capture["stdout"], terminal_magit_capture),
            ],
        )
    )

    magit_fixture = (ready or {}).get("magit_fixture") or {}
    magit_repo = magit_fixture.get("repo") or ""
    magit_buffer = magit_fixture.get("status_buffer") or ""
    magit_hook_log = magit_fixture.get("hook_log") or ""
    magit_preconditions = magit_fixture.get("preconditions") or {}
    magit_target_section = magit_fixture.get("target_section") or {}
    magit_target = f"{args.terminal_tmux_session}:0.0"
    magit_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        magit_target,
        magit_buffer,
        "daemon-magit",
    )
    magit_cancel = magit_switch["cancel"]
    magit_prefix = magit_switch["prefix"]
    magit_buffer_keys = magit_switch["buffer_keys"]
    magit_enter = magit_switch["enter"]
    magit_frame_attempts = magit_switch["observation_attempts"]
    magit_frames_response = magit_switch["response"]
    magit_frames_result = magit_switch["result"]
    magit_terminal_frames = magit_switch["terminal_frames"]

    magit_observe_request = {
        "request_id": "daemon-magit-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": magit_buffer},
    }
    magit_observe_response = request(args.socket, magit_observe_request)
    magit_initial_observation = magit_observe_response.get("result") or {}
    magit_observed_point = magit_initial_observation.get("point")
    magit_target_point = magit_target_section.get("target_point") or magit_preconditions.get("point")
    target_section_start = magit_target_section.get("start")
    target_section_end = magit_target_section.get("end")
    if isinstance(magit_target_point, int):
        magit_goto_request = {
            "request_id": "daemon-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": magit_buffer,
                "point": magit_target_point,
                "preconditions": magit_initial_observation,
            },
        }
        magit_goto_response = request_or_outcome_unknown(args.socket, magit_goto_request, timeout=2.0)
    else:
        magit_goto_request = {
            "request_id": "daemon-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": magit_buffer,
                "point": None,
                "preconditions": magit_initial_observation,
            },
        }
        magit_goto_response = {
            "request_id": "daemon-magit-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_target_point",
            "error": "Magit fixture did not provide a target point",
        }
    magit_ready_observe_request = {
        "request_id": "daemon-magit-ready-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": magit_buffer},
    }
    magit_ready_observe_response = request(args.socket, magit_ready_observe_request)
    magit_observation = magit_ready_observe_response.get("result") or {}
    magit_ready_point = magit_observation.get("point")
    magit_point_in_target_section = (
        isinstance(magit_ready_point, int)
        and isinstance(target_section_start, int)
        and isinstance(target_section_end, int)
        and target_section_start <= magit_ready_point < target_section_end
    )
    magit_current_preconditions = magit_observation or magit_preconditions
    magit_request = {
        "request_id": "daemon-magit-stage-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": magit_buffer,
            "command": "magit-stage",
            "preconditions": magit_current_preconditions,
        },
    }
    magit_response = request_or_outcome_unknown(args.socket, magit_request, timeout=5.0)
    magit_status_request = {
        "request_id": "daemon-magit-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-magit-stage-1"},
    }
    magit_status_response = request_or_outcome_unknown(args.socket, magit_status_request, timeout=2.0)
    magit_status_result = magit_status_response.get("result") or {}
    magit_running_status = magit_status_result.get("status") or {}
    magit_await_request = {
        "request_id": "daemon-magit-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-magit-stage-1",
            "timeout_seconds": 10.0,
        },
    }
    magit_await_response = request_or_outcome_unknown(args.socket, magit_await_request, timeout=12.0)
    magit_await_result = magit_await_response.get("result") or {}
    magit_final_status = magit_await_result.get("status") or magit_running_status or magit_response
    magit_result = magit_final_status.get("result") or {}
    cached_names = git_lines(magit_repo, "diff", "--cached", "--name-only") if magit_repo else []
    unstaged_names = git_lines(magit_repo, "diff", "--name-only") if magit_repo else []
    target_index = git_output(magit_repo, "show", ":target.txt") if magit_repo else ""
    target_worktree = read_text(str(pathlib.Path(magit_repo) / "target.txt")) if magit_repo else ""
    debug_index = git_output(magit_repo, "show", ":debug.txt") if magit_repo else ""
    debug_worktree = read_text(str(pathlib.Path(magit_repo) / "debug.txt")) if magit_repo else ""
    staged_index = git_output(magit_repo, "show", ":staged.txt") if magit_repo else ""
    hook_log_text = read_text(magit_hook_log)
    records.append(
        scenario_record(
            12,
            "daemon-magit-command-dispatch",
            "A live daemon JSONL request stages one Magit hunk through checked command.dispatch",
            True,
            args.out_dir,
            fixture=magit_fixture,
            terminal_switch={
                "tmux_target": magit_target,
                "cancel": magit_cancel,
                "cancel_attempts": magit_switch["cancel_attempts"],
                "prefix": magit_prefix,
                "minibuffer_attempts": magit_switch["minibuffer_attempts"],
                "buffer_keys": magit_buffer_keys,
                "enter": magit_enter,
                "observation_attempts": magit_frame_attempts,
                "response": magit_frames_response,
            },
            observe_request=magit_observe_request,
            observe_response=magit_observe_response,
            point_adjustment={
                "target_point": magit_target_point,
                "observed_point": magit_observed_point,
                "request": magit_goto_request,
                "response": magit_goto_response,
                "target_section": magit_target_section,
            },
            ready_observe_request=magit_ready_observe_request,
            ready_observe_response=magit_ready_observe_response,
            request=magit_request,
            response=magit_response,
            status_request=magit_status_request,
            status_response=magit_status_response,
            await_request=magit_await_request,
            await_response=magit_await_response,
            final_status=magit_final_status,
            repo_state={
                "cached_names": cached_names,
                "unstaged_names": unstaged_names,
                "target_index": target_index,
                "target_worktree": target_worktree,
                "debug_index": debug_index,
                "debug_worktree": debug_worktree,
                "staged_index": staged_index,
                "hook_log": hook_log_text,
            },
            checks=[
                check("Magit fixture is present", bool(magit_repo and magit_buffer), magit_fixture),
                check("terminal switch cancel succeeded", magit_cancel["returncode"] == 0, magit_cancel),
                check("terminal switch prefix succeeded", magit_prefix["returncode"] == 0, magit_prefix),
                check("terminal switch buffer text sent", magit_buffer_keys["returncode"] == 0, magit_buffer_keys),
                check("terminal switch enter succeeded", magit_enter["returncode"] == 0, magit_enter),
                check("client terminal frame displays Magit buffer", len(magit_terminal_frames) >= 1, magit_frames_result.get("frames")),
                check("post-terminal Magit observation succeeded", magit_observe_response.get("state") == "succeeded", magit_observe_response),
                check("post-terminal Magit observation names status buffer", magit_initial_observation.get("buffer_name") == magit_buffer, magit_initial_observation),
                check("checked Magit point navigation succeeded", magit_goto_response.get("state") == "succeeded", magit_goto_response),
                check("ready Magit observation succeeded", magit_ready_observe_response.get("state") == "succeeded", magit_ready_observe_response),
                check("ready Magit point remains inside target hunk", magit_point_in_target_section, {"target_section": magit_target_section, "observed": magit_ready_point}),
                check("Magit command request is accepted as running", magit_response.get("state") == "running", magit_response),
                check("Magit command request uses queued command route", (magit_response.get("result") or {}).get("route") == "queued-command-execute", magit_response),
                check("Magit queued request status is observable", magit_status_response.get("state") == "succeeded" and magit_running_status.get("request_id") == "daemon-magit-stage-1", magit_status_response),
                check("Magit queued request reaches terminal success", magit_await_response.get("state") == "succeeded" and magit_final_status.get("state") == "succeeded", {"await": magit_await_response, "final": magit_final_status}),
                check("Magit command dispatch succeeded", magit_final_status.get("state") == "succeeded", magit_final_status),
                check("Magit command ran through command-execute", magit_result.get("route") == "command-execute", magit_result),
                check("Magit command identity is preserved", magit_result.get("this_command") == "magit-stage", magit_result),
                check("Magit preconditions passed", all(item.get("passed") is True for item in magit_result.get("precondition_checks") or []), magit_result.get("precondition_checks")),
                check("cached names are exact after daemon staging", cached_names == ["staged.txt", "target.txt"], cached_names),
                check("unstaged names preserve unrelated work", unstaged_names == ["debug.txt", "target.txt"], unstaged_names),
                check("target selected change is staged", contains(target_index, "target-05-selected"), target_index),
                check("target unrelated hunk remains unstaged", contains(target_index, "target-25-base") and contains(target_worktree, "target-25-other"), {"index": target_index, "worktree": target_worktree}),
                check("debug worktree change remains unstaged", debug_index == "debug-base\n" and contains(debug_worktree, "debug-worktree-only"), {"index": debug_index, "worktree": debug_worktree}),
                check("unrelated staged file is preserved", staged_index == "staged-expected\n", staged_index),
                check("post-stage hook observed Magit command identity", contains(hook_log_text, "post-stage:magit-stage"), hook_log_text),
                check("Magit refresh hook ran in daemon", contains(hook_log_text, "refresh:"), hook_log_text),
            ],
        )
    )

    transient_fixture = (ready or {}).get("transient_magit_fixture") or {}
    transient_repo = transient_fixture.get("repo") or ""
    transient_buffer = transient_fixture.get("status_buffer") or ""
    transient_target = f"{args.terminal_tmux_session}:0.0"
    transient_initial_cached_names = git_lines(transient_repo, "diff", "--cached", "--name-only") if transient_repo else []
    transient_initial_unstaged_names = git_lines(transient_repo, "diff", "--name-only") if transient_repo else []
    transient_initial_target_index = git_output(transient_repo, "show", ":target.txt") if transient_repo else ""
    transient_initial_target_worktree = read_text(str(pathlib.Path(transient_repo) / "target.txt")) if transient_repo else ""
    transient_initial_debug_index = git_output(transient_repo, "show", ":debug.txt") if transient_repo else ""
    transient_initial_debug_worktree = read_text(str(pathlib.Path(transient_repo) / "debug.txt")) if transient_repo else ""
    transient_initial_staged_index = git_output(transient_repo, "show", ":staged.txt") if transient_repo else ""
    transient_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        transient_target,
        transient_buffer,
        "daemon-transient-magit",
    )
    transient_pre_key_request = {
        "request_id": "daemon-transient-magit-pre-key-1",
        "operation": "emacs.observe_interaction",
        "params": {},
    }
    transient_pre_key_response = request_or_outcome_unknown(
        args.socket,
        transient_pre_key_request,
        timeout=0.5,
    )
    transient_pre_key_result = transient_pre_key_response.get("result") or {}
    transient_key = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", transient_target, "?"]
    )
    transient_poll: list[dict[str, Any]] = []
    transient_interaction_response: dict[str, Any] = {}
    transient_interaction_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_interaction_request = {
            "request_id": f"daemon-transient-magit-interaction-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_interaction_response = request_or_outcome_unknown(
            args.socket,
            transient_interaction_request,
            timeout=0.5,
        )
        transient_interaction_result = transient_interaction_response.get("result") or {}
        transient_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_interaction_response.get("state"),
                "selected_window_buffer": transient_interaction_result.get("selected_window_buffer"),
                "transient_active": transient_interaction_result.get("transient_active"),
                "transient_prefix_command": transient_interaction_result.get("transient_prefix_command"),
                "transient_map_installed": transient_interaction_result.get("transient_map_installed"),
                "transient_buffer": transient_interaction_result.get("transient_buffer"),
                "transient_window_live": transient_interaction_result.get("transient_window_live"),
                "transient_window_buffer": transient_interaction_result.get("transient_window_buffer"),
            }
        )
        if (
            transient_interaction_response.get("state") == "succeeded"
            and transient_interaction_result.get("transient_active") is True
            and transient_interaction_result.get("transient_prefix_command") == "magit-dispatch"
            and transient_interaction_result.get("transient_map_installed") is True
            and transient_interaction_result.get("transient_window_live") is True
            and bool(transient_interaction_result.get("transient_buffer"))
            and bool(transient_interaction_result.get("transient_window_buffer"))
        ):
            break

    transient_capture_active = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", transient_target, "-p"]
    )
    transient_cancel = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", transient_target, "C-g"]
    )
    transient_clear_poll: list[dict[str, Any]] = []
    transient_clear_response: dict[str, Any] = {}
    transient_clear_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_clear_request = {
            "request_id": f"daemon-transient-magit-clear-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_clear_response = request_or_outcome_unknown(
            args.socket,
            transient_clear_request,
            timeout=0.5,
        )
        transient_clear_result = transient_clear_response.get("result") or {}
        transient_clear_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_clear_response.get("state"),
                "selected_window_buffer": transient_clear_result.get("selected_window_buffer"),
                "transient_active": transient_clear_result.get("transient_active"),
                "transient_prefix_command": transient_clear_result.get("transient_prefix_command"),
                "transient_map_installed": transient_clear_result.get("transient_map_installed"),
                "transient_buffer": transient_clear_result.get("transient_buffer"),
                "transient_window_live": transient_clear_result.get("transient_window_live"),
                "transient_window_buffer": transient_clear_result.get("transient_window_buffer"),
            }
        )
        if (
            transient_clear_response.get("state") == "succeeded"
            and transient_clear_result.get("transient_active") is not True
            and transient_clear_result.get("transient_map_installed") is not True
        ):
            break

    transient_final_cached_names = git_lines(transient_repo, "diff", "--cached", "--name-only") if transient_repo else []
    transient_final_unstaged_names = git_lines(transient_repo, "diff", "--name-only") if transient_repo else []
    transient_final_target_index = git_output(transient_repo, "show", ":target.txt") if transient_repo else ""
    transient_final_target_worktree = read_text(str(pathlib.Path(transient_repo) / "target.txt")) if transient_repo else ""
    transient_final_debug_index = git_output(transient_repo, "show", ":debug.txt") if transient_repo else ""
    transient_final_debug_worktree = read_text(str(pathlib.Path(transient_repo) / "debug.txt")) if transient_repo else ""
    transient_final_staged_index = git_output(transient_repo, "show", ":staged.txt") if transient_repo else ""
    records.append(
        scenario_record(
            13,
            "daemon-terminal-transient-magit-dispatch",
            "Real terminal input opens and cancels Magit's Transient dispatch without changing fixture Git state",
            True,
            args.out_dir,
            fixture=transient_fixture,
            initial_repo_state={
                "cached_names": transient_initial_cached_names,
                "unstaged_names": transient_initial_unstaged_names,
                "target_index": transient_initial_target_index,
                "target_worktree": transient_initial_target_worktree,
                "debug_index": transient_initial_debug_index,
                "debug_worktree": transient_initial_debug_worktree,
                "staged_index": transient_initial_staged_index,
            },
            terminal_switch={
                "tmux_target": transient_target,
                "cancel": transient_switch["cancel"],
                "cancel_attempts": transient_switch["cancel_attempts"],
                "cancel_interaction_response": transient_switch["cancel_interaction_response"],
                "prefix": transient_switch["prefix"],
                "minibuffer_attempts": transient_switch["minibuffer_attempts"],
                "buffer_keys": transient_switch["buffer_keys"],
                "enter": transient_switch["enter"],
                "observation_attempts": transient_switch["observation_attempts"],
                "response": transient_switch["response"],
            },
            pre_key_request=transient_pre_key_request,
            pre_key_response=transient_pre_key_response,
            terminal_key_sequence=["?"],
            terminal_dispatch_key=transient_key,
            interaction_poll=transient_poll,
            interaction_response=transient_interaction_response,
            active_terminal_capture=transient_capture_active,
            terminal_cancel=transient_cancel,
            clear_poll=transient_clear_poll,
            clear_response=transient_clear_response,
            final_repo_state={
                "cached_names": transient_final_cached_names,
                "unstaged_names": transient_final_unstaged_names,
                "target_index": transient_final_target_index,
                "target_worktree": transient_final_target_worktree,
                "debug_index": transient_final_debug_index,
                "debug_worktree": transient_final_debug_worktree,
                "staged_index": transient_final_staged_index,
            },
            checks=[
                check("Transient Magit fixture is present", bool(transient_repo and transient_buffer), transient_fixture),
                check("Transient Magit terminal switch cancel succeeded", transient_switch["cancel"]["returncode"] == 0, transient_switch["cancel"]),
                check("Transient Magit terminal switch prefix succeeded", transient_switch["prefix"]["returncode"] == 0, transient_switch["prefix"]),
                check("Transient Magit terminal switch buffer text sent", transient_switch["buffer_keys"]["returncode"] == 0, transient_switch["buffer_keys"]),
                check("Transient Magit terminal switch enter succeeded", transient_switch["enter"]["returncode"] == 0, transient_switch["enter"]),
                check("client terminal frame displays Transient Magit buffer", len(transient_switch["terminal_frames"]) >= 1, transient_switch["result"].get("frames")),
                check("Transient pre-key interaction observation succeeded", transient_pre_key_response.get("state") == "succeeded", transient_pre_key_response),
                check("Transient is inactive before dispatch key", transient_pre_key_result.get("transient_active") is not True and transient_pre_key_result.get("transient_map_installed") is not True, transient_pre_key_result),
                check("pre-key selected window is Transient Magit buffer", transient_pre_key_result.get("selected_window_buffer") == transient_buffer, transient_pre_key_result),
                check("Transient dispatch key send succeeded", transient_key["returncode"] == 0, transient_key),
                check("Transient interaction observation succeeded", transient_interaction_response.get("state") == "succeeded", transient_interaction_response),
                check("Transient selected window is target Magit buffer", transient_interaction_result.get("selected_window_buffer") == transient_buffer, transient_interaction_result),
                check("Transient is active after terminal key", transient_interaction_result.get("transient_active") is True, transient_interaction_result),
                check("Transient prefix is Magit dispatch", transient_interaction_result.get("transient_prefix_command") == "magit-dispatch", transient_interaction_result),
                check("Transient keymap is installed", transient_interaction_result.get("transient_map_installed") is True, transient_interaction_result),
                check("Transient buffer is live", bool(transient_interaction_result.get("transient_buffer")), transient_interaction_result),
                check("Transient window is live", transient_interaction_result.get("transient_window_live") is True, transient_interaction_result),
                check("Transient window displays a buffer", bool(transient_interaction_result.get("transient_window_buffer")), transient_interaction_result),
                check("terminal pane shows Magit dispatch transient", "Transient and dwim commands" in transient_capture_active["stdout"], transient_capture_active),
                check("Transient cancel key send succeeded", transient_cancel["returncode"] == 0, transient_cancel),
                check("Transient clear observation succeeded", transient_clear_response.get("state") == "succeeded", transient_clear_response),
                check("Transient is inactive after cancel", transient_clear_result.get("transient_active") is not True, transient_clear_result),
                check("Transient keymap clears after cancel", transient_clear_result.get("transient_map_installed") is not True, transient_clear_result),
                check("Transient cached names are unchanged", transient_final_cached_names == transient_initial_cached_names == ["staged.txt"], {"initial": transient_initial_cached_names, "final": transient_final_cached_names}),
                check("Transient unstaged names are unchanged", transient_final_unstaged_names == transient_initial_unstaged_names == ["debug.txt", "target.txt"], {"initial": transient_initial_unstaged_names, "final": transient_final_unstaged_names}),
                check("Transient target index is unchanged", transient_final_target_index == transient_initial_target_index and contains(transient_final_target_index, "target-05-base"), {"initial": transient_initial_target_index, "final": transient_final_target_index}),
                check("Transient target worktree is unchanged", transient_final_target_worktree == transient_initial_target_worktree and contains(transient_final_target_worktree, "target-05-selected"), {"initial": transient_initial_target_worktree, "final": transient_final_target_worktree}),
                check("Transient debug file is unchanged", transient_final_debug_index == transient_initial_debug_index == "debug-base\n" and transient_final_debug_worktree == transient_initial_debug_worktree, {"initial_index": transient_initial_debug_index, "final_index": transient_final_debug_index, "initial_worktree": transient_initial_debug_worktree, "final_worktree": transient_final_debug_worktree}),
                check("Transient staged file is unchanged", transient_final_staged_index == transient_initial_staged_index == "staged-expected\n", {"initial": transient_initial_staged_index, "final": transient_final_staged_index}),
            ],
        )
    )

    transient_stage_fixture = (ready or {}).get("transient_stage_magit_fixture") or {}
    transient_stage_repo = transient_stage_fixture.get("repo") or ""
    transient_stage_buffer = transient_stage_fixture.get("status_buffer") or ""
    transient_stage_hook_log = transient_stage_fixture.get("hook_log") or ""
    transient_stage_preconditions = transient_stage_fixture.get("preconditions") or {}
    transient_stage_target_section = transient_stage_fixture.get("target_section") or {}
    transient_stage_target = f"{args.terminal_tmux_session}:0.0"
    transient_stage_initial_state = magit_fixture_repo_state(
        transient_stage_repo,
        transient_stage_hook_log,
    )
    transient_stage_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        transient_stage_target,
        transient_stage_buffer,
        "daemon-transient-magit-stage",
    )
    transient_stage_observe_request = {
        "request_id": "daemon-transient-magit-stage-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": transient_stage_buffer},
    }
    transient_stage_observe_response = request_or_outcome_unknown(
        args.socket,
        transient_stage_observe_request,
        timeout=2.0,
    )
    transient_stage_initial_observation = transient_stage_observe_response.get("result") or {}
    transient_stage_target_point = (
        transient_stage_target_section.get("target_point")
        or transient_stage_preconditions.get("point")
    )
    transient_stage_section_start = transient_stage_target_section.get("start")
    transient_stage_section_end = transient_stage_target_section.get("end")
    if isinstance(transient_stage_target_point, int):
        transient_stage_goto_request = {
            "request_id": "daemon-transient-magit-stage-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": transient_stage_buffer,
                "point": transient_stage_target_point,
                "preconditions": transient_stage_initial_observation,
            },
        }
        transient_stage_goto_response = request_or_outcome_unknown(
            args.socket,
            transient_stage_goto_request,
            timeout=2.0,
        )
    else:
        transient_stage_goto_request = {
            "request_id": "daemon-transient-magit-stage-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": transient_stage_buffer,
                "point": None,
                "preconditions": transient_stage_initial_observation,
            },
        }
        transient_stage_goto_response = {
            "request_id": "daemon-transient-magit-stage-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_target_point",
            "error": "Transient suffix Magit fixture did not provide a target point",
        }
    transient_stage_ready_observe_request = {
        "request_id": "daemon-transient-magit-stage-ready-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": transient_stage_buffer},
    }
    transient_stage_ready_observe_response = request_or_outcome_unknown(
        args.socket,
        transient_stage_ready_observe_request,
        timeout=2.0,
    )
    transient_stage_ready_observation = transient_stage_ready_observe_response.get("result") or {}
    transient_stage_ready_point = transient_stage_ready_observation.get("point")
    transient_stage_point_in_target_section = (
        isinstance(transient_stage_ready_point, int)
        and isinstance(transient_stage_section_start, int)
        and isinstance(transient_stage_section_end, int)
        and transient_stage_section_start <= transient_stage_ready_point < transient_stage_section_end
    )
    transient_stage_pre_key_request = {
        "request_id": "daemon-transient-magit-stage-pre-key-1",
        "operation": "emacs.observe_interaction",
        "params": {},
    }
    transient_stage_pre_key_response = request_or_outcome_unknown(
        args.socket,
        transient_stage_pre_key_request,
        timeout=0.5,
    )
    transient_stage_pre_key_result = transient_stage_pre_key_response.get("result") or {}
    transient_stage_dispatch_key = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", transient_stage_target, "?"]
    )
    transient_stage_active_poll: list[dict[str, Any]] = []
    transient_stage_active_response: dict[str, Any] = {}
    transient_stage_active_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_stage_active_request = {
            "request_id": f"daemon-transient-magit-stage-active-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_stage_active_response = request_or_outcome_unknown(
            args.socket,
            transient_stage_active_request,
            timeout=0.5,
        )
        transient_stage_active_result = transient_stage_active_response.get("result") or {}
        transient_stage_active_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_stage_active_response.get("state"),
                "selected_window_buffer": transient_stage_active_result.get("selected_window_buffer"),
                "transient_active": transient_stage_active_result.get("transient_active"),
                "transient_prefix_command": transient_stage_active_result.get("transient_prefix_command"),
                "transient_map_installed": transient_stage_active_result.get("transient_map_installed"),
                "transient_buffer": transient_stage_active_result.get("transient_buffer"),
                "transient_window_live": transient_stage_active_result.get("transient_window_live"),
                "transient_window_buffer": transient_stage_active_result.get("transient_window_buffer"),
            }
        )
        if (
            transient_stage_active_response.get("state") == "succeeded"
            and transient_stage_active_result.get("selected_window_buffer") == transient_stage_buffer
            and transient_stage_active_result.get("transient_active") is True
            and transient_stage_active_result.get("transient_prefix_command") == "magit-dispatch"
            and transient_stage_active_result.get("transient_map_installed") is True
            and transient_stage_active_result.get("transient_window_live") is True
            and bool(transient_stage_active_result.get("transient_buffer"))
            and bool(transient_stage_active_result.get("transient_window_buffer"))
        ):
            break

    transient_stage_active_frames_request = {
        "request_id": "daemon-transient-magit-stage-active-frames-1",
        "operation": "emacs.observe_frames",
        "params": {},
    }
    transient_stage_active_frames_response = request_or_outcome_unknown(
        args.socket,
        transient_stage_active_frames_request,
        timeout=2.0,
    )
    transient_stage_active_frames_result = transient_stage_active_frames_response.get("result") or {}
    transient_stage_active_terminal_frames = [
        frame
        for frame in transient_stage_active_frames_result.get("frames") or []
        if frame.get("client_frame") is True
        and frame.get("graphic") is False
        and frame.get("terminal_live") is True
        and frame.get("selected_window_buffer") == transient_stage_buffer
    ]
    transient_stage_capture_active = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", transient_stage_target, "-p"]
    )
    transient_stage_suffix_key = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", transient_stage_target, "s"]
    )
    transient_stage_final_poll: list[dict[str, Any]] = []
    transient_stage_final_state = magit_fixture_repo_state(
        transient_stage_repo,
        transient_stage_hook_log,
    )
    transient_stage_clear_response: dict[str, Any] = {}
    transient_stage_clear_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_stage_final_state = magit_fixture_repo_state(
            transient_stage_repo,
            transient_stage_hook_log,
        )
        transient_stage_clear_request = {
            "request_id": f"daemon-transient-magit-stage-clear-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_stage_clear_response = request_or_outcome_unknown(
            args.socket,
            transient_stage_clear_request,
            timeout=0.5,
        )
        transient_stage_clear_result = transient_stage_clear_response.get("result") or {}
        transient_stage_final_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_stage_clear_response.get("state"),
                "cached_names": transient_stage_final_state.get("cached_names"),
                "unstaged_names": transient_stage_final_state.get("unstaged_names"),
                "hook_log": transient_stage_final_state.get("hook_log"),
                "selected_window_buffer": transient_stage_clear_result.get("selected_window_buffer"),
                "last_command": transient_stage_clear_result.get("last_command"),
                "transient_active": transient_stage_clear_result.get("transient_active"),
                "transient_map_installed": transient_stage_clear_result.get("transient_map_installed"),
            }
        )
        if (
            transient_stage_clear_response.get("state") == "succeeded"
            and transient_stage_clear_result.get("transient_active") is not True
            and transient_stage_clear_result.get("transient_map_installed") is not True
            and transient_stage_final_state.get("cached_names") == ["staged.txt", "target.txt"]
            and transient_stage_final_state.get("unstaged_names") == ["debug.txt"]
            and contains(transient_stage_final_state.get("target_index") or "", "target-05-selected")
            and contains(transient_stage_final_state.get("target_index") or "", "target-25-other")
            and contains(transient_stage_final_state.get("hook_log") or "", "post-stage:magit-stage")
        ):
            break
    transient_stage_capture_final = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", transient_stage_target, "-p"]
    )
    records.append(
        scenario_record(
            14,
            "daemon-terminal-transient-magit-stage-suffix",
            "Real terminal input opens Magit's Transient dispatch and executes its Stage suffix with the observed target-file scope",
            True,
            args.out_dir,
            fixture=transient_stage_fixture,
            initial_repo_state=transient_stage_initial_state,
            terminal_switch={
                "tmux_target": transient_stage_target,
                "cancel": transient_stage_switch["cancel"],
                "cancel_attempts": transient_stage_switch["cancel_attempts"],
                "cancel_interaction_response": transient_stage_switch["cancel_interaction_response"],
                "prefix": transient_stage_switch["prefix"],
                "minibuffer_attempts": transient_stage_switch["minibuffer_attempts"],
                "buffer_keys": transient_stage_switch["buffer_keys"],
                "enter": transient_stage_switch["enter"],
                "observation_attempts": transient_stage_switch["observation_attempts"],
                "response": transient_stage_switch["response"],
            },
            observe_request=transient_stage_observe_request,
            observe_response=transient_stage_observe_response,
            point_adjustment={
                "target_point": transient_stage_target_point,
                "request": transient_stage_goto_request,
                "response": transient_stage_goto_response,
                "target_section": transient_stage_target_section,
            },
            ready_observe_request=transient_stage_ready_observe_request,
            ready_observe_response=transient_stage_ready_observe_response,
            pre_key_request=transient_stage_pre_key_request,
            pre_key_response=transient_stage_pre_key_response,
            terminal_key_sequence=["?", "s"],
            terminal_dispatch_key=transient_stage_dispatch_key,
            active_poll=transient_stage_active_poll,
            active_interaction_response=transient_stage_active_response,
            active_frames_request=transient_stage_active_frames_request,
            active_frames_response=transient_stage_active_frames_response,
            active_terminal_capture=transient_stage_capture_active,
            terminal_suffix_key=transient_stage_suffix_key,
            final_poll=transient_stage_final_poll,
            final_interaction_response=transient_stage_clear_response,
            final_terminal_capture=transient_stage_capture_final,
            final_repo_state=transient_stage_final_state,
            checks=[
                check("Transient suffix Magit fixture is present", bool(transient_stage_repo and transient_stage_buffer), transient_stage_fixture),
                check("Transient suffix terminal switch cancel succeeded", transient_stage_switch["cancel"]["returncode"] == 0, transient_stage_switch["cancel"]),
                check("Transient suffix terminal switch prefix succeeded", transient_stage_switch["prefix"]["returncode"] == 0, transient_stage_switch["prefix"]),
                check("Transient suffix terminal switch buffer text sent", transient_stage_switch["buffer_keys"]["returncode"] == 0, transient_stage_switch["buffer_keys"]),
                check("Transient suffix terminal switch enter succeeded", transient_stage_switch["enter"]["returncode"] == 0, transient_stage_switch["enter"]),
                check("client terminal frame displays Transient suffix Magit buffer", len(transient_stage_switch["terminal_frames"]) >= 1, transient_stage_switch["result"].get("frames")),
                check("Transient suffix observation succeeded", transient_stage_observe_response.get("state") == "succeeded", transient_stage_observe_response),
                check("checked Transient suffix point navigation succeeded", transient_stage_goto_response.get("state") == "succeeded", transient_stage_goto_response),
                check("ready Transient suffix observation succeeded", transient_stage_ready_observe_response.get("state") == "succeeded", transient_stage_ready_observe_response),
                check("ready Transient suffix point remains inside target hunk", transient_stage_point_in_target_section, {"target_section": transient_stage_target_section, "observed": transient_stage_ready_point}),
                check("Transient suffix pre-key interaction observation succeeded", transient_stage_pre_key_response.get("state") == "succeeded", transient_stage_pre_key_response),
                check("Transient suffix inactive before dispatch key", transient_stage_pre_key_result.get("transient_active") is not True and transient_stage_pre_key_result.get("transient_map_installed") is not True, transient_stage_pre_key_result),
                check("Transient suffix pre-key selected window is target buffer", transient_stage_pre_key_result.get("selected_window_buffer") == transient_stage_buffer, transient_stage_pre_key_result),
                check("Transient suffix dispatch key send succeeded", transient_stage_dispatch_key["returncode"] == 0, transient_stage_dispatch_key),
                check("Transient suffix interaction observation succeeded", transient_stage_active_response.get("state") == "succeeded", transient_stage_active_response),
                check("Transient suffix selected window is target Magit buffer", transient_stage_active_result.get("selected_window_buffer") == transient_stage_buffer, transient_stage_active_result),
                check("Transient suffix prefix is Magit dispatch", transient_stage_active_result.get("transient_prefix_command") == "magit-dispatch", transient_stage_active_result),
                check("Transient suffix keymap is installed before suffix", transient_stage_active_result.get("transient_map_installed") is True, transient_stage_active_result),
                check("Transient suffix active terminal frame displays target buffer", len(transient_stage_active_terminal_frames) >= 1, transient_stage_active_frames_result.get("frames")),
                check("terminal pane shows Transient suffix menu", "Transient and dwim commands" in transient_stage_capture_active["stdout"], transient_stage_capture_active),
                check("Transient Stage suffix key send succeeded", transient_stage_suffix_key["returncode"] == 0, transient_stage_suffix_key),
                check("Transient clears after suffix execution", transient_stage_clear_response.get("state") == "succeeded" and transient_stage_clear_result.get("transient_active") is not True and transient_stage_clear_result.get("transient_map_installed") is not True, transient_stage_clear_response),
                check("Transient suffix cached names are exact", transient_stage_final_state.get("cached_names") == ["staged.txt", "target.txt"], transient_stage_final_state),
                check("Transient suffix unstaged names preserve unrelated file work", transient_stage_final_state.get("unstaged_names") == ["debug.txt"], transient_stage_final_state),
                check("Transient suffix selected change is staged", contains(transient_stage_final_state.get("target_index") or "", "target-05-selected"), transient_stage_final_state.get("target_index")),
                check("Transient suffix stages target file scope", contains(transient_stage_final_state.get("target_index") or "", "target-25-other") and transient_stage_final_state.get("target_index") == transient_stage_final_state.get("target_worktree"), {"index": transient_stage_final_state.get("target_index"), "worktree": transient_stage_final_state.get("target_worktree")}),
                check("Transient suffix debug change remains unstaged", transient_stage_final_state.get("debug_index") == "debug-base\n" and contains(transient_stage_final_state.get("debug_worktree") or "", "debug-worktree-only"), {"index": transient_stage_final_state.get("debug_index"), "worktree": transient_stage_final_state.get("debug_worktree")}),
                check("Transient suffix unrelated staged file is preserved", transient_stage_final_state.get("staged_index") == "staged-expected\n", transient_stage_final_state.get("staged_index")),
                check("Transient suffix post-stage hook observed Magit command identity", contains(transient_stage_final_state.get("hook_log") or "", "post-stage:magit-stage"), transient_stage_final_state.get("hook_log")),
                check("Transient suffix Magit refresh hook ran", contains(transient_stage_final_state.get("hook_log") or "", "refresh:"), transient_stage_final_state.get("hook_log")),
            ],
        )
    )

    transient_diff_fixture = (ready or {}).get("transient_diff_magit_fixture") or {}
    transient_diff_repo = transient_diff_fixture.get("repo") or ""
    transient_diff_buffer = transient_diff_fixture.get("status_buffer") or ""
    transient_diff_target = f"{args.terminal_tmux_session}:0.0"
    transient_diff_initial_state = magit_fixture_repo_state(transient_diff_repo)
    transient_diff_expected_stat = (
        git_output(transient_diff_repo, "diff", "--stat") if transient_diff_repo else ""
    )
    transient_diff_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        transient_diff_target,
        transient_diff_buffer,
        "daemon-transient-magit-diff",
    )
    transient_diff_observe_request = {
        "request_id": "daemon-transient-magit-diff-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": transient_diff_buffer},
    }
    transient_diff_observe_response = request_or_outcome_unknown(
        args.socket,
        transient_diff_observe_request,
        timeout=2.0,
    )
    transient_diff_observation = response_result(transient_diff_observe_response)
    transient_diff_request = {
        "request_id": "daemon-transient-magit-diff-command-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": transient_diff_buffer,
            "command": "magit-diff",
            "preconditions": transient_diff_observation,
        },
    }
    transient_diff_response = request_or_outcome_unknown(
        args.socket,
        transient_diff_request,
        timeout=2.0,
    )
    transient_diff_status_request = {
        "request_id": "daemon-transient-magit-diff-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-transient-magit-diff-command-1"},
    }
    transient_diff_status_response = request_or_outcome_unknown(
        args.socket,
        transient_diff_status_request,
        timeout=0.5,
    )
    transient_diff_status_result = transient_diff_status_response.get("result") or {}
    transient_diff_running_status = transient_diff_status_result.get("status") or {}
    transient_diff_await_request = {
        "request_id": "daemon-transient-magit-diff-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-transient-magit-diff-command-1",
            "timeout_seconds": 5.0,
        },
    }
    transient_diff_await_response = request_or_outcome_unknown(
        args.socket,
        transient_diff_await_request,
        timeout=6.0,
    )
    transient_diff_await_result = transient_diff_await_response.get("result") or {}
    transient_diff_final_status = (
        transient_diff_await_result.get("status")
        or transient_diff_running_status
        or transient_diff_response
    )
    transient_diff_result = transient_diff_final_status.get("result") or {}
    transient_diff_active_poll: list[dict[str, Any]] = []
    transient_diff_active_response: dict[str, Any] = {}
    transient_diff_active_result: dict[str, Any] = {}
    transient_diff_initial_stat_suffix: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_diff_active_request = {
            "request_id": f"daemon-transient-magit-diff-active-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_diff_active_response = request_or_outcome_unknown(
            args.socket,
            transient_diff_active_request,
            timeout=0.5,
        )
        transient_diff_active_result = transient_diff_active_response.get("result") or {}
        transient_diff_initial_stat_suffix = find_transient_suffix(
            transient_diff_active_result,
            "magit-diff:--stat",
        )
        transient_diff_active_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_diff_active_response.get("state"),
                "selected_window_buffer": transient_diff_active_result.get("selected_window_buffer"),
                "transient_active": transient_diff_active_result.get("transient_active"),
                "transient_prefix_command": transient_diff_active_result.get("transient_prefix_command"),
                "transient_map_installed": transient_diff_active_result.get("transient_map_installed"),
                "stat_suffix": transient_diff_initial_stat_suffix,
                "active_arguments": transient_diff_active_result.get("transient_active_arguments"),
            }
        )
        if (
            transient_diff_active_response.get("state") == "succeeded"
            and transient_diff_active_result.get("selected_window_buffer") == transient_diff_buffer
            and transient_diff_active_result.get("transient_active") is True
            and transient_diff_active_result.get("transient_prefix_command") == "magit-diff"
            and transient_diff_active_result.get("transient_map_installed") is True
            and transient_diff_initial_stat_suffix.get("argument") == "--stat"
        ):
            break

    transient_diff_capture_active = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", transient_diff_target, "-p"]
    )
    transient_diff_stat_key = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", transient_diff_target, "--", "-", "s"]
    )
    transient_diff_toggle_poll: list[dict[str, Any]] = []
    transient_diff_toggle_response: dict[str, Any] = {}
    transient_diff_toggle_result: dict[str, Any] = {}
    transient_diff_toggled_stat_suffix: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_diff_toggle_request = {
            "request_id": f"daemon-transient-magit-diff-toggle-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_diff_toggle_response = request_or_outcome_unknown(
            args.socket,
            transient_diff_toggle_request,
            timeout=0.5,
        )
        transient_diff_toggle_result = transient_diff_toggle_response.get("result") or {}
        transient_diff_toggled_stat_suffix = find_transient_suffix(
            transient_diff_toggle_result,
            "magit-diff:--stat",
        )
        transient_diff_toggle_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_diff_toggle_response.get("state"),
                "selected_window_buffer": transient_diff_toggle_result.get("selected_window_buffer"),
                "transient_active": transient_diff_toggle_result.get("transient_active"),
                "transient_prefix_command": transient_diff_toggle_result.get("transient_prefix_command"),
                "transient_map_installed": transient_diff_toggle_result.get("transient_map_installed"),
                "stat_suffix": transient_diff_toggled_stat_suffix,
                "active_arguments": transient_diff_toggle_result.get("transient_active_arguments"),
            }
        )
        if (
            transient_diff_toggle_response.get("state") == "succeeded"
            and transient_diff_toggle_result.get("transient_active") is True
            and transient_diff_toggle_result.get("transient_prefix_command") == "magit-diff"
            and transient_diff_toggled_stat_suffix.get("command") == "magit-diff:--stat"
            and (
                transient_value_contains(
                    transient_diff_toggled_stat_suffix.get("value"),
                    "--stat",
                )
                != transient_value_contains(
                    transient_diff_initial_stat_suffix.get("value"),
                    "--stat",
                )
            )
        ):
            break

    transient_diff_initial_stat_enabled = (
        transient_value_contains(
            transient_diff_initial_stat_suffix.get("value"),
            "--stat",
        )
        or transient_active_argument_present(transient_diff_active_result, "--stat")
    )
    transient_diff_toggled_stat_enabled = (
        transient_value_contains(
            transient_diff_toggled_stat_suffix.get("value"),
            "--stat",
        )
        or transient_active_argument_present(transient_diff_toggle_result, "--stat")
    )
    transient_diff_capture_toggled = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", transient_diff_target, "-p"]
    )
    transient_diff_suffix_key = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", transient_diff_target, "u"]
    )
    transient_diff_clear_poll: list[dict[str, Any]] = []
    transient_diff_clear_response: dict[str, Any] = {}
    transient_diff_clear_result: dict[str, Any] = {}
    transient_diff_result_buffer = ""
    for attempt in range(1, 51):
        time.sleep(0.1)
        transient_diff_clear_request = {
            "request_id": f"daemon-transient-magit-diff-clear-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        transient_diff_clear_response = request_or_outcome_unknown(
            args.socket,
            transient_diff_clear_request,
            timeout=0.5,
        )
        transient_diff_clear_result = transient_diff_clear_response.get("result") or {}
        selected_after = str(transient_diff_clear_result.get("selected_window_buffer") or "")
        if selected_after and selected_after != transient_diff_buffer:
            transient_diff_result_buffer = selected_after
        transient_diff_clear_poll.append(
            {
                "attempt": attempt,
                "response_state": transient_diff_clear_response.get("state"),
                "selected_window_buffer": selected_after,
                "transient_active": transient_diff_clear_result.get("transient_active"),
                "transient_map_installed": transient_diff_clear_result.get("transient_map_installed"),
                "last_command": transient_diff_clear_result.get("last_command"),
            }
        )
        if (
            transient_diff_clear_response.get("state") == "succeeded"
            and transient_diff_clear_result.get("transient_active") is not True
            and transient_diff_clear_result.get("transient_map_installed") is not True
            and transient_diff_result_buffer
        ):
            break

    transient_diff_capture_final = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-S", "-200", "-t", transient_diff_target, "-p"]
    )
    transient_diff_buffer_observe_request = {
        "request_id": "daemon-transient-magit-diff-buffer-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": transient_diff_result_buffer},
    }
    transient_diff_buffer_observe_response = (
        request_or_outcome_unknown(
            args.socket,
            transient_diff_buffer_observe_request,
            timeout=2.0,
        )
        if transient_diff_result_buffer
        else {
            "request_id": "daemon-transient-magit-diff-buffer-observe-1",
            "operation": "emacs.observe_buffer",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_diff_buffer",
            "error": "Magit diff suffix did not select a result buffer",
        }
    )
    transient_diff_buffer_observation = response_result(
        transient_diff_buffer_observe_response
    )
    transient_diff_buffer_read_response: dict[str, Any] = {}
    transient_diff_buffer_text = ""
    transient_diff_buffer_point_max = transient_diff_buffer_observation.get("point_max")
    if transient_diff_result_buffer and isinstance(transient_diff_buffer_point_max, int):
        transient_diff_buffer_read_response, transient_diff_buffer_text = buffer_text_via_bridge(
            args.socket,
            transient_diff_result_buffer,
            "daemon-transient-magit-diff-read-1",
            transient_diff_buffer_point_max,
        )
    else:
        transient_diff_buffer_read_response = {
            "request_id": "daemon-transient-magit-diff-read-1",
            "operation": "buffer.read",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_diff_buffer_bounds",
            "error": "Magit diff buffer observation did not provide point_max",
        }
    transient_diff_final_state = magit_fixture_repo_state(transient_diff_repo)
    transient_diff_result_reflects_toggled_mode = (
        (
            transient_diff_toggled_stat_enabled
            and contains(transient_diff_buffer_text, "target.txt")
            and contains(transient_diff_buffer_text, "debug.txt")
            and (
                contains(transient_diff_buffer_text, "files changed")
                or contains(transient_diff_buffer_text, "file changed")
            )
        )
        or (
            not transient_diff_toggled_stat_enabled
            and contains(transient_diff_buffer_text, "target-05-selected")
            and contains(transient_diff_buffer_text, "target-25-other")
            and contains(transient_diff_buffer_text, "debug-worktree-only")
        )
    )
    records.append(
        scenario_record(
            15,
            "daemon-terminal-transient-magit-diff-infix",
            "A queued daemon Magit diff Transient observes, toggles, and applies the --stat infix state from terminal input without mutating Git state",
            True,
            args.out_dir,
            fixture=transient_diff_fixture,
            initial_repo_state=transient_diff_initial_state,
            expected_git_stat=transient_diff_expected_stat,
            terminal_switch={
                "tmux_target": transient_diff_target,
                "cancel": transient_diff_switch["cancel"],
                "cancel_attempts": transient_diff_switch["cancel_attempts"],
                "cancel_interaction_response": transient_diff_switch["cancel_interaction_response"],
                "prefix": transient_diff_switch["prefix"],
                "minibuffer_attempts": transient_diff_switch["minibuffer_attempts"],
                "buffer_keys": transient_diff_switch["buffer_keys"],
                "enter": transient_diff_switch["enter"],
                "observation_attempts": transient_diff_switch["observation_attempts"],
                "response": transient_diff_switch["response"],
            },
            observe_request=transient_diff_observe_request,
            observe_response=transient_diff_observe_response,
            dispatch_request=transient_diff_request,
            dispatch_response=transient_diff_response,
            status_request=transient_diff_status_request,
            status_response=transient_diff_status_response,
            await_request=transient_diff_await_request,
            await_response=transient_diff_await_response,
            final_status=transient_diff_final_status,
            active_poll=transient_diff_active_poll,
            active_interaction_response=transient_diff_active_response,
            active_terminal_capture=transient_diff_capture_active,
            terminal_infix_keys=transient_diff_stat_key,
            toggle_poll=transient_diff_toggle_poll,
            toggle_interaction_response=transient_diff_toggle_response,
            toggled_terminal_capture=transient_diff_capture_toggled,
            terminal_suffix_key=transient_diff_suffix_key,
            clear_poll=transient_diff_clear_poll,
            clear_interaction_response=transient_diff_clear_response,
            final_terminal_capture=transient_diff_capture_final,
            diff_buffer=transient_diff_result_buffer,
            diff_buffer_observe_request=transient_diff_buffer_observe_request,
            diff_buffer_observe_response=transient_diff_buffer_observe_response,
            diff_buffer_read_response=transient_diff_buffer_read_response,
            diff_buffer_text=transient_diff_buffer_text,
            final_repo_state=transient_diff_final_state,
            checks=[
                check("Transient diff Magit fixture is present", bool(transient_diff_repo and transient_diff_buffer), transient_diff_fixture),
                check("Transient diff terminal switch cancel succeeded", transient_diff_switch["cancel"]["returncode"] == 0, transient_diff_switch["cancel"]),
                check("Transient diff terminal switch prefix succeeded", transient_diff_switch["prefix"]["returncode"] == 0, transient_diff_switch["prefix"]),
                check("Transient diff terminal switch buffer text sent", transient_diff_switch["buffer_keys"]["returncode"] == 0, transient_diff_switch["buffer_keys"]),
                check("Transient diff terminal switch enter succeeded", transient_diff_switch["enter"]["returncode"] == 0, transient_diff_switch["enter"]),
                check("client terminal frame displays Transient diff Magit buffer", len(transient_diff_switch["terminal_frames"]) >= 1, transient_diff_switch["result"].get("frames")),
                check("Transient diff observation succeeded", transient_diff_observe_response.get("state") == "succeeded", transient_diff_observe_response),
                check("Transient diff command accepted as running", transient_diff_response.get("state") == "running", transient_diff_response),
                check("Transient diff command status is observable", transient_diff_status_response.get("state") == "succeeded" and transient_diff_running_status.get("request_id") == "daemon-transient-magit-diff-command-1", transient_diff_status_response),
                check("Transient diff command reaches terminal success", transient_diff_await_response.get("state") == "succeeded" and transient_diff_final_status.get("state") == "succeeded", {"await": transient_diff_await_response, "final": transient_diff_final_status}),
                check("Transient diff ran through command-execute", transient_diff_result.get("route") == "command-execute", transient_diff_result),
                check("Transient diff command identity is preserved", transient_diff_result.get("this_command") == "magit-diff", transient_diff_result),
                check("Transient diff preconditions passed", all(item.get("passed") is True for item in transient_diff_result.get("precondition_checks") or []), transient_diff_result.get("precondition_checks")),
                check("Transient diff interaction observation succeeded", transient_diff_active_response.get("state") == "succeeded", transient_diff_active_response),
                check("Transient diff selected window is target Magit buffer", transient_diff_active_result.get("selected_window_buffer") == transient_diff_buffer, transient_diff_active_result),
                check("Transient diff prefix is active", transient_diff_active_result.get("transient_prefix_command") == "magit-diff" and transient_diff_active_result.get("transient_map_installed") is True, transient_diff_active_result),
                check("Transient diff --stat infix is observed before input", transient_diff_initial_stat_suffix.get("command") == "magit-diff:--stat" and transient_diff_initial_stat_suffix.get("argument") == "--stat", transient_diff_initial_stat_suffix),
                check("Transient diff initial --stat state is observable", transient_diff_initial_stat_enabled == transient_active_argument_present(transient_diff_active_result, "--stat"), {"suffix": transient_diff_initial_stat_suffix, "arguments": transient_diff_active_result.get("transient_active_arguments")}),
                check("terminal pane shows Magit diff Transient", terminal_capture_shows_magit_diff_transient(transient_diff_capture_active), transient_diff_capture_active),
                check("Transient diff --stat key send succeeded", transient_diff_stat_key["returncode"] == 0, transient_diff_stat_key),
                check("Transient diff --stat value toggles", transient_diff_toggled_stat_enabled != transient_diff_initial_stat_enabled, {"before": transient_diff_active_result, "after": transient_diff_toggle_result}),
                check("Transient diff active arguments match toggled --stat state", transient_active_argument_present(transient_diff_toggle_result, "--stat") == transient_diff_toggled_stat_enabled, transient_diff_toggle_result),
                check("terminal pane still shows diff Transient after infix", terminal_capture_shows_magit_diff_transient(transient_diff_capture_toggled), transient_diff_capture_toggled),
                check("Transient diff unstaged suffix key send succeeded", transient_diff_suffix_key["returncode"] == 0, transient_diff_suffix_key),
                check("Transient diff clears after suffix execution", transient_diff_clear_response.get("state") == "succeeded" and transient_diff_clear_result.get("transient_active") is not True and transient_diff_clear_result.get("transient_map_installed") is not True, transient_diff_clear_response),
                check("Transient diff selected a result buffer", bool(transient_diff_result_buffer) and transient_diff_result_buffer != transient_diff_buffer, {"status_buffer": transient_diff_buffer, "result_buffer": transient_diff_result_buffer}),
                check("Transient diff result buffer observation succeeded", transient_diff_buffer_observe_response.get("state") == "succeeded", transient_diff_buffer_observe_response),
                check("Transient diff result buffer is Magit diff mode", transient_diff_buffer_observation.get("major_mode") == "magit-diff-mode", transient_diff_buffer_observation),
                check("Transient diff result buffer read succeeded", transient_diff_buffer_read_response.get("state") == "succeeded", transient_diff_buffer_read_response),
                check("Transient diff result reflects toggled --stat mode", transient_diff_result_reflects_toggled_mode, {"toggled_stat_enabled": transient_diff_toggled_stat_enabled, "text": transient_diff_buffer_text}),
                check("Transient diff expected git stat is non-empty", contains(transient_diff_expected_stat, "target.txt") and contains(transient_diff_expected_stat, "debug.txt"), transient_diff_expected_stat),
                check("Transient diff cached names are unchanged", transient_diff_final_state.get("cached_names") == transient_diff_initial_state.get("cached_names") == ["staged.txt"], {"initial": transient_diff_initial_state, "final": transient_diff_final_state}),
                check("Transient diff unstaged names are unchanged", transient_diff_final_state.get("unstaged_names") == transient_diff_initial_state.get("unstaged_names") == ["debug.txt", "target.txt"], {"initial": transient_diff_initial_state, "final": transient_diff_final_state}),
                check("Transient diff target file is unchanged", transient_diff_final_state.get("target_index") == transient_diff_initial_state.get("target_index") and transient_diff_final_state.get("target_worktree") == transient_diff_initial_state.get("target_worktree"), {"initial": transient_diff_initial_state, "final": transient_diff_final_state}),
                check("Transient diff debug file is unchanged", transient_diff_final_state.get("debug_index") == transient_diff_initial_state.get("debug_index") and transient_diff_final_state.get("debug_worktree") == transient_diff_initial_state.get("debug_worktree"), {"initial": transient_diff_initial_state, "final": transient_diff_final_state}),
                check("Transient diff staged file is unchanged", transient_diff_final_state.get("staged_index") == transient_diff_initial_state.get("staged_index") == "staged-expected\n", {"initial": transient_diff_initial_state.get("staged_index"), "final": transient_diff_final_state.get("staged_index")}),
                check("Transient diff did not run stage hook", "post-stage:" not in str(transient_diff_final_state.get("hook_log") or ""), transient_diff_final_state.get("hook_log")),
            ],
        )
    )

    prompt_magit_fixture = (ready or {}).get("prompt_magit_fixture") or {}
    prompt_magit_repo = prompt_magit_fixture.get("repo") or ""
    prompt_magit_buffer = prompt_magit_fixture.get("status_buffer") or ""
    prompt_magit_hook_log = prompt_magit_fixture.get("hook_log") or ""
    prompt_magit_preconditions = prompt_magit_fixture.get("preconditions") or {}
    prompt_magit_target_section = prompt_magit_fixture.get("target_section") or {}
    prompt_magit_target = f"{args.terminal_tmux_session}:0.0"
    prompt_magit_initial_state = magit_fixture_repo_state(
        prompt_magit_repo,
        prompt_magit_hook_log,
    )
    prompt_magit_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        prompt_magit_target,
        prompt_magit_buffer,
        "daemon-package-prompt-magit",
    )
    prompt_magit_observe_request = {
        "request_id": "daemon-package-prompt-magit-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": prompt_magit_buffer},
    }
    prompt_magit_observe_response = request_or_outcome_unknown(
        args.socket,
        prompt_magit_observe_request,
        timeout=2.0,
    )
    prompt_magit_initial_observation = prompt_magit_observe_response.get("result") or {}
    prompt_magit_target_point = (
        prompt_magit_target_section.get("target_point")
        or prompt_magit_preconditions.get("point")
    )
    prompt_magit_section_start = prompt_magit_target_section.get("start")
    prompt_magit_section_end = prompt_magit_target_section.get("end")
    if isinstance(prompt_magit_target_point, int):
        prompt_magit_goto_request = {
            "request_id": "daemon-package-prompt-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": prompt_magit_buffer,
                "point": prompt_magit_target_point,
                "preconditions": prompt_magit_initial_observation,
            },
        }
        prompt_magit_goto_response = request_or_outcome_unknown(
            args.socket,
            prompt_magit_goto_request,
            timeout=2.0,
        )
    else:
        prompt_magit_goto_request = {
            "request_id": "daemon-package-prompt-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": prompt_magit_buffer,
                "point": None,
                "preconditions": prompt_magit_initial_observation,
            },
        }
        prompt_magit_goto_response = {
            "request_id": "daemon-package-prompt-magit-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_target_point",
            "error": "Package prompt Magit fixture did not provide a target point",
        }
    prompt_magit_ready_observe_request = {
        "request_id": "daemon-package-prompt-magit-ready-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": prompt_magit_buffer},
    }
    prompt_magit_ready_observe_response = request_or_outcome_unknown(
        args.socket,
        prompt_magit_ready_observe_request,
        timeout=2.0,
    )
    prompt_magit_ready_observation = prompt_magit_ready_observe_response.get("result") or {}
    prompt_magit_ready_point = prompt_magit_ready_observation.get("point")
    prompt_magit_point_in_target_section = (
        isinstance(prompt_magit_ready_point, int)
        and isinstance(prompt_magit_section_start, int)
        and isinstance(prompt_magit_section_end, int)
        and prompt_magit_section_start <= prompt_magit_ready_point < prompt_magit_section_end
    )
    prompt_magit_request = {
        "request_id": "daemon-package-prompt-magit-stage-files-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": prompt_magit_buffer,
            "command": "magit-stage-files",
            "preconditions": prompt_magit_ready_observation,
        },
    }
    prompt_magit_response = request_or_outcome_unknown(
        args.socket,
        prompt_magit_request,
        timeout=2.0,
    )
    prompt_magit_interaction_attempts: list[dict[str, Any]] = []
    prompt_magit_interaction_response: dict[str, Any] = {}
    prompt_magit_interaction_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        time.sleep(0.1)
        prompt_magit_interaction_request = {
            "request_id": f"daemon-package-prompt-magit-interaction-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        prompt_magit_interaction_response = request_or_outcome_unknown(
            args.socket,
            prompt_magit_interaction_request,
            timeout=0.5,
        )
        prompt_magit_interaction_result = prompt_magit_interaction_response.get("result") or {}
        prompt_magit_interaction_attempts.append(
            {
                "attempt": attempt,
                "response_state": prompt_magit_interaction_response.get("state"),
                "minibuffer_active": prompt_magit_interaction_result.get("minibuffer_active"),
                "minibuffer_prompt": prompt_magit_interaction_result.get("minibuffer_prompt"),
                "minibuffer_contents": prompt_magit_interaction_result.get("minibuffer_contents"),
                "selected_window_buffer": prompt_magit_interaction_result.get("selected_window_buffer"),
                "transient_active": prompt_magit_interaction_result.get("transient_active"),
            }
        )
        if (
            prompt_magit_interaction_response.get("state") == "succeeded"
            and prompt_magit_interaction_result.get("minibuffer_active") is True
            and isinstance(prompt_magit_interaction_result.get("minibuffer_prompt"), str)
            and "Stage" in prompt_magit_interaction_result.get("minibuffer_prompt", "")
            and "file" in prompt_magit_interaction_result.get("minibuffer_prompt", "")
        ):
            break
    prompt_magit_status_request = {
        "request_id": "daemon-package-prompt-magit-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-package-prompt-magit-stage-files-1"},
    }
    prompt_magit_status_response = request_or_outcome_unknown(
        args.socket,
        prompt_magit_status_request,
        timeout=0.5,
    )
    prompt_magit_status_result = prompt_magit_status_response.get("result") or {}
    prompt_magit_running_status = prompt_magit_status_result.get("status") or {}
    prompt_magit_capture_before = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-t", prompt_magit_target, "-p"]
    )
    prompt_magit_input = "target.txt"
    prompt_magit_input_keys = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", prompt_magit_target, prompt_magit_input]
    )
    prompt_magit_submit = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", prompt_magit_target, "Enter"]
    )
    prompt_magit_await_request = {
        "request_id": "daemon-package-prompt-magit-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-package-prompt-magit-stage-files-1",
            "timeout_seconds": 10.0,
        },
    }
    prompt_magit_await_response = request_or_outcome_unknown(
        args.socket,
        prompt_magit_await_request,
        timeout=12.0,
    )
    prompt_magit_await_result = prompt_magit_await_response.get("result") or {}
    prompt_magit_final_status = (
        prompt_magit_await_result.get("status")
        or prompt_magit_running_status
        or prompt_magit_response
    )
    prompt_magit_result = prompt_magit_final_status.get("result") or {}
    prompt_magit_final_state = magit_fixture_repo_state(
        prompt_magit_repo,
        prompt_magit_hook_log,
    )
    prompt_magit_final_interaction_request = {
        "request_id": "daemon-package-prompt-magit-final-interaction-1",
        "operation": "emacs.observe_interaction",
        "params": {},
    }
    prompt_magit_final_interaction_response = request_or_outcome_unknown(
        args.socket,
        prompt_magit_final_interaction_request,
        timeout=2.0,
    )
    prompt_magit_final_interaction_result = prompt_magit_final_interaction_response.get("result") or {}
    records.append(
        scenario_record(
            16,
            "daemon-package-prompt-magit-stage-files",
            "A queued daemon Magit command opens a package minibuffer prompt and stages the selected file from tmux input",
            True,
            args.out_dir,
            fixture=prompt_magit_fixture,
            initial_repo_state=prompt_magit_initial_state,
            terminal_switch={
                "tmux_target": prompt_magit_target,
                "cancel": prompt_magit_switch["cancel"],
                "cancel_attempts": prompt_magit_switch["cancel_attempts"],
                "cancel_interaction_response": prompt_magit_switch["cancel_interaction_response"],
                "prefix": prompt_magit_switch["prefix"],
                "minibuffer_attempts": prompt_magit_switch["minibuffer_attempts"],
                "buffer_keys": prompt_magit_switch["buffer_keys"],
                "enter": prompt_magit_switch["enter"],
                "observation_attempts": prompt_magit_switch["observation_attempts"],
                "response": prompt_magit_switch["response"],
            },
            observe_request=prompt_magit_observe_request,
            observe_response=prompt_magit_observe_response,
            point_adjustment={
                "target_point": prompt_magit_target_point,
                "request": prompt_magit_goto_request,
                "response": prompt_magit_goto_response,
                "target_section": prompt_magit_target_section,
            },
            ready_observe_request=prompt_magit_ready_observe_request,
            ready_observe_response=prompt_magit_ready_observe_response,
            request=prompt_magit_request,
            response=prompt_magit_response,
            interaction_attempts=prompt_magit_interaction_attempts,
            interaction_response=prompt_magit_interaction_response,
            status_request=prompt_magit_status_request,
            status_response=prompt_magit_status_response,
            terminal_before_input=prompt_magit_capture_before,
            terminal_input=prompt_magit_input,
            terminal_input_keys=prompt_magit_input_keys,
            terminal_submit=prompt_magit_submit,
            await_request=prompt_magit_await_request,
            await_response=prompt_magit_await_response,
            final_status=prompt_magit_final_status,
            final_interaction_request=prompt_magit_final_interaction_request,
            final_interaction_response=prompt_magit_final_interaction_response,
            final_repo_state=prompt_magit_final_state,
            checks=[
                check("package prompt Magit fixture is present", bool(prompt_magit_repo and prompt_magit_buffer), prompt_magit_fixture),
                check("package prompt terminal switch cancel succeeded", prompt_magit_switch["cancel"]["returncode"] == 0, prompt_magit_switch["cancel"]),
                check("package prompt terminal switch prefix succeeded", prompt_magit_switch["prefix"]["returncode"] == 0, prompt_magit_switch["prefix"]),
                check("package prompt terminal switch buffer text sent", prompt_magit_switch["buffer_keys"]["returncode"] == 0, prompt_magit_switch["buffer_keys"]),
                check("package prompt terminal switch enter succeeded", prompt_magit_switch["enter"]["returncode"] == 0, prompt_magit_switch["enter"]),
                check("client terminal frame displays package prompt Magit buffer", len(prompt_magit_switch["terminal_frames"]) >= 1, prompt_magit_switch["result"].get("frames")),
                check("package prompt Magit observation succeeded", prompt_magit_observe_response.get("state") == "succeeded", prompt_magit_observe_response),
                check("checked package prompt point navigation succeeded", prompt_magit_goto_response.get("state") == "succeeded", prompt_magit_goto_response),
                check("ready package prompt observation succeeded", prompt_magit_ready_observe_response.get("state") == "succeeded", prompt_magit_ready_observe_response),
                check("ready package prompt point remains inside target hunk", prompt_magit_point_in_target_section, {"target_section": prompt_magit_target_section, "observed": prompt_magit_ready_point}),
                check("package prompt command accepted as running", prompt_magit_response.get("state") == "running", prompt_magit_response),
                check("package prompt command uses queued command route", (prompt_magit_response.get("result") or {}).get("route") == "queued-command-execute", prompt_magit_response),
                check("package prompt interaction observation succeeded", prompt_magit_interaction_response.get("state") == "succeeded", prompt_magit_interaction_response),
                check("package prompt minibuffer is active", prompt_magit_interaction_result.get("minibuffer_active") is True, prompt_magit_interaction_result),
                check("package prompt is Magit stage files", "Stage" in str(prompt_magit_interaction_result.get("minibuffer_prompt")) and "file" in str(prompt_magit_interaction_result.get("minibuffer_prompt")), prompt_magit_interaction_result),
                check("package prompt queued status is observable", prompt_magit_status_response.get("state") == "succeeded" and prompt_magit_running_status.get("state") == "running", prompt_magit_status_response),
                check("terminal pane shows package prompt", "Stage" in prompt_magit_capture_before["stdout"] and "file" in prompt_magit_capture_before["stdout"], prompt_magit_capture_before),
                check("package prompt input was sent", prompt_magit_input_keys["returncode"] == 0, prompt_magit_input_keys),
                check("package prompt submit succeeded", prompt_magit_submit["returncode"] == 0, prompt_magit_submit),
                check("package prompt await reaches success", prompt_magit_await_response.get("state") == "succeeded" and prompt_magit_final_status.get("state") == "succeeded", prompt_magit_await_response),
                check("package prompt ran through command-execute", prompt_magit_result.get("route") == "command-execute", prompt_magit_result),
                check("package prompt command identity is preserved", prompt_magit_result.get("this_command") == "magit-stage-files", prompt_magit_result),
                check("package prompt preconditions passed", all(item.get("passed") is True for item in prompt_magit_result.get("precondition_checks") or []), prompt_magit_result.get("precondition_checks")),
                check("package prompt minibuffer clears after submit", prompt_magit_final_interaction_response.get("state") == "succeeded" and prompt_magit_final_interaction_result.get("minibuffer_active") is not True, prompt_magit_final_interaction_response),
                check("package prompt cached names are exact", prompt_magit_final_state.get("cached_names") == ["staged.txt", "target.txt"], prompt_magit_final_state),
                check("package prompt unstaged names preserve unrelated file work", prompt_magit_final_state.get("unstaged_names") == ["debug.txt"], prompt_magit_final_state),
                check("package prompt stages target file scope", prompt_magit_final_state.get("target_index") == prompt_magit_final_state.get("target_worktree") and contains(prompt_magit_final_state.get("target_index") or "", "target-25-other"), {"index": prompt_magit_final_state.get("target_index"), "worktree": prompt_magit_final_state.get("target_worktree")}),
                check("package prompt debug change remains unstaged", prompt_magit_final_state.get("debug_index") == "debug-base\n" and contains(prompt_magit_final_state.get("debug_worktree") or "", "debug-worktree-only"), {"index": prompt_magit_final_state.get("debug_index"), "worktree": prompt_magit_final_state.get("debug_worktree")}),
                check("package prompt unrelated staged file is preserved", prompt_magit_final_state.get("staged_index") == "staged-expected\n", prompt_magit_final_state.get("staged_index")),
                check("package prompt post-stage hook observed command identity", contains(prompt_magit_final_state.get("hook_log") or "", "post-stage:magit-stage-files"), prompt_magit_final_state.get("hook_log")),
                check("package prompt Magit refresh hook ran", contains(prompt_magit_final_state.get("hook_log") or "", "refresh:"), prompt_magit_final_state.get("hook_log")),
            ],
        )
    )

    finish_async_request = {
        "request_id": "daemon-finish-async-1",
        "operation": "bridge.defer_echo",
        "params": {
            "delay_seconds": 0.5,
            "value": "finish ok",
        },
    }
    finish_async_response = request(args.socket, finish_async_request)
    premature_finish_request = {
        "request_id": "daemon-finish-premature-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {"request_id": "daemon-finish-async-1", "state": "succeeded"},
            ],
        },
    }
    premature_finish_response = request(args.socket, premature_finish_request)
    finish_await_request = {
        "request_id": "daemon-finish-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-finish-async-1",
            "timeout_seconds": 2.0,
        },
    }
    finish_await_response = request(args.socket, finish_await_request, timeout=3.0)
    final_finish_request = {
        "request_id": "daemon-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {"request_id": "daemon-finish-async-1", "state": "succeeded"},
                {"request_id": "daemon-finish-await-1", "state": "succeeded"},
            ],
        },
    }
    final_finish_response = request(args.socket, final_finish_request)
    premature_finish_result = premature_finish_response.get("result") or {}
    final_finish_result = final_finish_response.get("result") or {}
    records.append(
        scenario_record(
            17,
            "daemon-episode-finish-lifecycle",
            "episode.finish rejects a running dependency and succeeds after required daemon requests are terminal",
            True,
            args.out_dir,
            async_request=finish_async_request,
            async_response=finish_async_response,
            premature_finish_request=premature_finish_request,
            premature_finish_response=premature_finish_response,
            await_request=finish_await_request,
            await_response=finish_await_response,
            final_finish_request=final_finish_request,
            final_finish_response=final_finish_response,
            checks=[
                check("finish async request starts running", finish_async_response.get("state") == "running", finish_async_response),
                check("premature finish is rejected", premature_finish_response.get("state") == "failed", premature_finish_response),
                check("premature finish reports incomplete episode", premature_finish_response.get("error_code") == "incomplete_episode", premature_finish_response),
                check("premature finish identifies running dependency", any(
                    item.get("request_id") == "daemon-finish-async-1"
                    and item.get("actual_state") == "running"
                    and item.get("passed") is False
                    for item in premature_finish_result.get("request_checks", [])
                ), premature_finish_result),
                check("await before final finish succeeds", finish_await_response.get("state") == "succeeded", finish_await_response),
                check("final finish request succeeds", final_finish_response.get("state") == "succeeded", final_finish_response),
                check("final finish records episode completion", final_finish_result.get("state") == "succeeded", final_finish_result),
                check("final finish validated required requests", all(
                    item.get("passed") is True
                    for item in final_finish_result.get("request_checks", [])
                ), final_finish_result),
            ],
        )
    )

    comint_fixture = (ready or {}).get("comint_fixture") or {}
    comint_buffer = comint_fixture.get("buffer_name") or ""
    comint_counter = comint_fixture.get("counter") or ""
    comint_command_log = comint_fixture.get("command_log") or ""
    comint_command = comint_fixture.get("command") or ""
    comint_expected_label = comint_fixture.get("expected_label") or "live"
    comint_done_needle = f"SOMA_DAEMON_COMINT_DONE:{comint_expected_label}"
    comint_sent_needle = f"SOMA_DAEMON_COMINT_SENT:{comint_expected_label}"
    comint_target = f"{args.terminal_tmux_session}:0.0"
    comint_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        comint_target,
        comint_buffer,
        "daemon-comint-terminal",
    )
    comint_initial_request = {
        "request_id": "daemon-comint-terminal-observe-1",
        "operation": "comint.observe",
        "params": {"buffer_name": comint_buffer},
    }
    comint_initial_response = request_or_outcome_unknown(
        args.socket,
        comint_initial_request,
        timeout=2.0,
    )
    comint_initial_result = comint_initial_response.get("result") or {}
    comint_process_mark = comint_initial_result.get("process_mark")
    if isinstance(comint_process_mark, int):
        comint_goto_request = {
            "request_id": "daemon-comint-terminal-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": comint_buffer,
                "point": comint_process_mark,
                "preconditions": comint_initial_result,
            },
        }
        comint_goto_response = request_or_outcome_unknown(
            args.socket,
            comint_goto_request,
            timeout=2.0,
        )
    else:
        comint_goto_request = {
            "request_id": "daemon-comint-terminal-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": comint_buffer,
                "point": None,
                "preconditions": comint_initial_result,
            },
        }
        comint_goto_response = {
            "request_id": "daemon-comint-terminal-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_process_mark",
            "error": "Comint fixture did not provide a process mark",
        }
    comint_ready_request = {
        "request_id": "daemon-comint-terminal-ready-1",
        "operation": "comint.observe",
        "params": {"buffer_name": comint_buffer},
    }
    comint_ready_response = request_or_outcome_unknown(
        args.socket,
        comint_ready_request,
        timeout=2.0,
    )
    comint_ready_result = comint_ready_response.get("result") or {}
    comint_capture_before = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-S", "-200", "-t", comint_target, "-p"]
    )
    comint_input_keys = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", comint_target, comint_command]
    )
    comint_submit = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", comint_target, "Enter"]
    )
    comint_wait = wait_comint_observation(
        args.socket,
        comint_buffer,
        "daemon-comint-terminal-wait",
        comint_done_needle,
        comint_counter,
        timeout_seconds=5.0,
    )
    comint_final_response = comint_wait.get("response") or {}
    comint_final_result = comint_wait.get("result") or {}
    comint_tail = str(comint_final_result.get("buffer_tail") or "")
    comint_counter_lines = comint_wait.get("counter_lines") or []
    comint_command_log_text = read_text(comint_command_log)
    comint_capture_after = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-S", "-200", "-t", comint_target, "-p"]
    )
    records.append(
        scenario_record(
            18,
            "daemon-comint-terminal-send-input",
            "A live daemon terminal frame submits input through Comint to a rootfs-owned subprocess exactly once",
            True,
            args.out_dir,
            fixture=comint_fixture,
            terminal_switch={
                "tmux_target": comint_target,
                "cancel": comint_switch["cancel"],
                "cancel_attempts": comint_switch["cancel_attempts"],
                "cancel_interaction_response": comint_switch["cancel_interaction_response"],
                "prefix": comint_switch["prefix"],
                "minibuffer_attempts": comint_switch["minibuffer_attempts"],
                "buffer_keys": comint_switch["buffer_keys"],
                "enter": comint_switch["enter"],
                "observation_attempts": comint_switch["observation_attempts"],
                "response": comint_switch["response"],
            },
            initial_request=comint_initial_request,
            initial_response=comint_initial_response,
            point_adjustment={
                "process_mark": comint_process_mark,
                "request": comint_goto_request,
                "response": comint_goto_response,
            },
            ready_request=comint_ready_request,
            ready_response=comint_ready_response,
            terminal_before_input=comint_capture_before,
            terminal_input=comint_command,
            terminal_input_keys=comint_input_keys,
            terminal_submit=comint_submit,
            wait=comint_wait,
            command_log=comint_command_log_text,
            terminal_after_input=comint_capture_after,
            checks=[
                check("Comint fixture is present", bool(comint_buffer and comint_counter and comint_command), comint_fixture),
                check("Comint terminal switch cancel succeeded", comint_switch["cancel"]["returncode"] == 0, comint_switch["cancel"]),
                check("Comint terminal switch prefix succeeded", comint_switch["prefix"]["returncode"] == 0, comint_switch["prefix"]),
                check("Comint terminal switch buffer text sent", comint_switch["buffer_keys"]["returncode"] == 0, comint_switch["buffer_keys"]),
                check("Comint terminal switch enter succeeded", comint_switch["enter"]["returncode"] == 0, comint_switch["enter"]),
                check("client terminal frame displays Comint buffer", len(comint_switch["terminal_frames"]) >= 1, comint_switch["result"].get("frames")),
                check("initial Comint observation succeeded", comint_initial_response.get("state") == "succeeded", comint_initial_response),
                check("initial Comint process is live", comint_initial_result.get("process_live") is True, comint_initial_result),
                check("initial Comint process executable is rootfs-owned", rootfs_owned_process_path(str(comint_initial_result.get("process_exe") or "")), comint_initial_result.get("process_exe")),
                check("initial Comint selected window is target buffer", comint_initial_result.get("selected_window_buffer") == comint_buffer, comint_initial_result),
                check("checked Comint point navigation succeeded", comint_goto_response.get("state") == "succeeded", comint_goto_response),
                check("ready Comint observation succeeded", comint_ready_response.get("state") == "succeeded", comint_ready_response),
                check("ready Comint point is at or after process mark", comint_ready_result.get("point_after_process_mark") is True, comint_ready_result),
                check("Comint terminal input was sent", comint_input_keys["returncode"] == 0, comint_input_keys),
                check("Comint terminal Enter succeeded", comint_submit["returncode"] == 0, comint_submit),
                check("final Comint observation succeeded", comint_final_response.get("state") == "succeeded", comint_final_response),
                check("Comint process identity is stable", comint_final_result.get("process_id") == comint_initial_result.get("process_id"), {"initial": comint_initial_result, "final": comint_final_result}),
                check("final Comint process remains live", comint_final_result.get("process_live") is True, comint_final_result),
                check("final Comint process executable is rootfs-owned", rootfs_owned_process_path(str(comint_final_result.get("process_exe") or "")), comint_final_result.get("process_exe")),
                check("Comint last-input markers updated", comint_final_result.get("last_input_markers_live") is True, comint_final_result),
                check("Comint last-input text includes submitted command", comint_command in str(comint_final_result.get("last_input_text") or ""), comint_final_result.get("last_input_text")),
                check("Comint output contains submitted label", comint_sent_needle in comint_tail and comint_done_needle in comint_tail, comint_tail),
                check("Comint subprocess received run id", os.environ.get("SOMA_INSULA_RUN_ID", "") in comint_tail and bool(os.environ.get("SOMA_INSULA_RUN_ID", "")), comint_tail),
                check("Comint side effect occurred exactly once", comint_counter_lines == [comint_expected_label], comint_counter_lines),
                check("Comint post-command hook saw comint-send-input", "post-command:comint-send-input" in comint_command_log_text, comint_command_log_text),
                check("terminal pane shows Comint result", comint_done_needle in comint_capture_after["stdout"], comint_capture_after),
            ],
        )
    )

    semantic_label = "semantic"
    semantic_done_needle = f"SOMA_BRIDGE_COMINT_DONE:{semantic_label}"
    semantic_stale_label = "semantic-stale"
    semantic_stale_done_needle = f"SOMA_BRIDGE_COMINT_DONE:{semantic_stale_label}"
    semantic_command = (
        f"printf 'SOMA_BRIDGE_COMINT_SENT:{semantic_label}\\n'; "
        f"printf 'SOMA_BRIDGE_COMINT_RUN:%s\\n' \"$SOMA_INSULA_RUN_ID\"; "
        f"printf '{semantic_label}\\n' >> {shlex.quote(comint_counter)}; "
        f"printf 'SOMA_BRIDGE_COMINT_DONE:{semantic_label}\\n'"
    )
    semantic_stale_command = (
        f"printf 'SOMA_BRIDGE_COMINT_SENT:{semantic_stale_label}\\n'; "
        f"printf '{semantic_stale_label}\\n' >> {shlex.quote(comint_counter)}; "
        f"printf 'SOMA_BRIDGE_COMINT_DONE:{semantic_stale_label}\\n'"
    )
    semantic_initial_request = {
        "request_id": "daemon-comint-api-observe-1",
        "operation": "comint.observe",
        "params": {"buffer_name": comint_buffer},
    }
    semantic_initial_response = request_or_outcome_unknown(
        args.socket,
        semantic_initial_request,
        timeout=2.0,
    )
    semantic_initial_result = semantic_initial_response.get("result") or {}
    semantic_initial_evidence = semantic_initial_response.get("evidence") or {}
    semantic_initial_observations = semantic_initial_response.get("observations") or {}
    semantic_initial_effects = semantic_initial_response.get("effects") or {}
    semantic_initial_process_identity = semantic_initial_evidence.get("process_identity") or {}
    semantic_initial_input_boundary = semantic_initial_evidence.get("input_boundary") or {}
    semantic_initial_runtime = semantic_initial_evidence.get("rootfs_runtime") or {}
    semantic_initial_state = semantic_initial_observations.get("comint_state") or {}
    semantic_submit_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-comint-api-submit-1",
        "operation": "comint.submit_input",
        "capability": "external_process_input",
        "params": {
            "buffer_name": comint_buffer,
            "input": semantic_command,
            "preconditions": semantic_initial_result,
        },
    }
    semantic_submit_response = request_or_outcome_unknown(
        args.socket,
        semantic_submit_request,
        timeout=3.0,
    )
    semantic_submit_result = semantic_submit_response.get("result") or {}
    semantic_submit_after = (
        semantic_submit_result.get("after")
        if isinstance(semantic_submit_result.get("after"), dict)
        else semantic_initial_result
    )
    semantic_await_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-comint-api-await-1",
        "operation": "comint.await_output",
        "capability": "passive_observation",
        "params": {
            "buffer_name": comint_buffer,
            "needle": semantic_done_needle,
            "timeout_seconds": 5.0,
            "preconditions": semantic_submit_after,
        },
    }
    semantic_await_response = request_or_outcome_unknown(
        args.socket,
        semantic_await_request,
        timeout=7.0,
    )
    semantic_await_result = semantic_await_response.get("result") or {}
    semantic_stale_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-comint-api-stale-submit-1",
        "operation": "comint.submit_input",
        "capability": "external_process_input",
        "params": {
            "buffer_name": comint_buffer,
            "input": semantic_stale_command,
            "preconditions": semantic_initial_result,
        },
    }
    semantic_stale_response = request_or_outcome_unknown(
        args.socket,
        semantic_stale_request,
        timeout=3.0,
    )
    semantic_stale_result = semantic_stale_response.get("result") or {}
    semantic_final_request = {
        "request_id": "daemon-comint-api-final-observe-1",
        "operation": "comint.observe",
        "params": {"buffer_name": comint_buffer},
    }
    semantic_final_response = request_or_outcome_unknown(
        args.socket,
        semantic_final_request,
        timeout=2.0,
    )
    semantic_final_result = semantic_final_response.get("result") or {}
    semantic_counter_lines = file_lines(comint_counter)
    semantic_tail = str(semantic_final_result.get("buffer_tail") or "")
    semantic_submit_evidence = semantic_submit_response.get("evidence") or {}
    semantic_submit_effects = semantic_submit_response.get("effects") or {}
    semantic_submit_process_input = (
        semantic_submit_effects.get("process_input")
        if isinstance(semantic_submit_effects, dict)
        else {}
    )
    semantic_await_evidence = semantic_await_response.get("evidence") or {}
    semantic_await_effects = semantic_await_response.get("effects") or {}
    records.append(
        scenario_record(
            19,
            "daemon-comint-semantic-api",
            "The live daemon semantic Comint API submits through Comint, awaits output, and rejects stale process-mark ownership",
            True,
            args.out_dir,
            fixture=comint_fixture,
            initial_request=semantic_initial_request,
            initial_response=semantic_initial_response,
            submit_request=semantic_submit_request,
            submit_response=semantic_submit_response,
            await_request=semantic_await_request,
            await_response=semantic_await_response,
            stale_request=semantic_stale_request,
            stale_response=semantic_stale_response,
            final_request=semantic_final_request,
            final_response=semantic_final_response,
            counter_lines=semantic_counter_lines,
            checks=[
                check("semantic Comint fixture is present", bool(comint_buffer and comint_counter), comint_fixture),
                check("semantic initial Comint observation succeeded", semantic_initial_response.get("state") == "succeeded", semantic_initial_response),
                check("semantic initial Comint process is live", semantic_initial_result.get("process_live") is True, semantic_initial_result),
                check("semantic initial Comint process executable is rootfs-owned", rootfs_owned_process_path(str(semantic_initial_result.get("process_exe") or "")), semantic_initial_result.get("process_exe")),
                check("daemon comint.observe publishes typed process and input-boundary sections", semantic_initial_process_identity.get("process_id") == semantic_initial_result.get("process_id") and semantic_initial_process_identity.get("process_live") is True and semantic_initial_input_boundary.get("process_mark") == semantic_initial_result.get("process_mark") and semantic_initial_runtime.get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "") and semantic_initial_state.get("buffer_name") == comint_buffer and isinstance(semantic_initial_state.get("buffer_tail"), str) and semantic_initial_effects == {}, {"evidence": semantic_initial_evidence, "observations": semantic_initial_observations, "effects": semantic_initial_effects}),
                check("comint.submit_input succeeds through the v0 envelope", semantic_submit_response.get("api_version") == "soma-emacs-agent-v0" and semantic_submit_response.get("state") == "succeeded" and semantic_submit_response.get("state_sequence") == ["accepted", "running", "succeeded"], semantic_submit_response),
                check("comint.submit_input uses comint-send-input and not raw process writes", semantic_submit_result.get("route") == "comint-send-input" and semantic_submit_result.get("used_raw_process_write") is False and isinstance(semantic_submit_result.get("submitted_bytes"), int) and semantic_submit_result.get("submitted_bytes", 0) > 0, semantic_submit_result),
                check("comint.submit_input records before and after Comint ownership state", isinstance(semantic_submit_result.get("before"), dict) and isinstance(semantic_submit_result.get("after"), dict) and semantic_submit_result.get("after", {}).get("process_id") == semantic_initial_result.get("process_id"), semantic_submit_result),
                check("comint.submit_input publishes typed terminal evidence and process-input effect", isinstance(semantic_submit_evidence, dict) and semantic_submit_evidence.get("submitted_bytes") == semantic_submit_result.get("submitted_bytes") and isinstance(semantic_submit_evidence.get("comint_input_markers"), dict) and isinstance(semantic_submit_evidence.get("process_output"), str) and isinstance(semantic_submit_process_input, dict) and semantic_submit_process_input.get("route") == "comint-send-input" and semantic_submit_process_input.get("submitted_bytes") == semantic_submit_result.get("submitted_bytes") and semantic_submit_process_input.get("used_raw_process_write") is False, {"evidence": semantic_submit_evidence, "effects": semantic_submit_effects}),
                check("comint.await_output succeeds and matches the intended output", semantic_await_response.get("state") == "succeeded" and semantic_await_result.get("matched") is True and semantic_done_needle in str(semantic_await_result.get("matched_output") or ""), semantic_await_response),
                check("comint.await_output publishes typed output evidence without process effects", isinstance(semantic_await_evidence, dict) and semantic_await_evidence.get("matched") is True and semantic_await_evidence.get("needle") == semantic_done_needle and isinstance(semantic_await_evidence.get("matched_output"), str) and isinstance(semantic_await_evidence.get("partial_output"), str) and isinstance(semantic_await_evidence.get("timeout_seconds"), (int, float)) and isinstance(semantic_await_effects, dict) and "process_input" not in semantic_await_effects and "process_interrupt" not in semantic_await_effects, {"evidence": semantic_await_evidence, "effects": semantic_await_effects}),
                check("semantic Comint side effect occurred exactly once while preserving earlier terminal side effect", semantic_counter_lines == [comint_expected_label, semantic_label], semantic_counter_lines),
                check("semantic Comint process identity stayed stable", semantic_final_result.get("process_id") == semantic_initial_result.get("process_id") and semantic_final_result.get("process_live") is True, {"initial": semantic_initial_result, "final": semantic_final_result}),
                check("stale semantic submit is rejected before side effects", semantic_stale_response.get("state") == "failed" and semantic_stale_label not in semantic_counter_lines and semantic_stale_done_needle not in semantic_tail, {"response": semantic_stale_response, "counter_lines": semantic_counter_lines, "tail": semantic_tail[-800:]}),
                check("stale semantic submit reports failed Comint precondition checks", semantic_stale_result.get("route") == "precondition-check" and any(item.get("passed") is False and item.get("name") == "Comint process mark matches precondition" for item in semantic_stale_result.get("precondition_checks") or []), semantic_stale_result),
            ],
        )
    )

    api_interrupt_fixture = (ready or {}).get("comint_api_interrupt_fixture") or {}
    api_interrupt_buffer = api_interrupt_fixture.get("buffer_name") or ""
    api_interrupt_counter = api_interrupt_fixture.get("counter") or ""
    api_interrupt_command = api_interrupt_fixture.get("command") or ""
    api_interrupt_start_needle = str(api_interrupt_fixture.get("start_needle") or "SOMA_DAEMON_COMINT_INTERRUPT_START")
    api_interrupt_done_needle = str(api_interrupt_fixture.get("done_needle") or "SOMA_DAEMON_COMINT_INTERRUPT_DONE")
    if api_interrupt_buffer and api_interrupt_counter and api_interrupt_command:
        api_interrupt_initial_request = {
            "request_id": "daemon-comint-api-interrupt-observe-1",
            "operation": "comint.observe",
            "params": {"buffer_name": api_interrupt_buffer},
        }
        api_interrupt_initial_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_initial_request,
            timeout=2.0,
        )
        api_interrupt_initial_result = api_interrupt_initial_response.get("result") or {}
        api_interrupt_submit_request = {
            "api_version": "soma-emacs-agent-v0",
            "request_id": "daemon-comint-api-interrupt-submit-1",
            "operation": "comint.submit_input",
            "capability": "external_process_input",
            "params": {
                "buffer_name": api_interrupt_buffer,
                "input": api_interrupt_command,
                "preconditions": api_interrupt_initial_result,
            },
        }
        api_interrupt_submit_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_submit_request,
            timeout=3.0,
        )
        api_interrupt_submit_result = api_interrupt_submit_response.get("result") or {}
        api_interrupt_submit_after = (
            api_interrupt_submit_result.get("after")
            if isinstance(api_interrupt_submit_result.get("after"), dict)
            else api_interrupt_initial_result
        )
        api_interrupt_await_start_request = {
            "api_version": "soma-emacs-agent-v0",
            "request_id": "daemon-comint-api-interrupt-await-start-1",
            "operation": "comint.await_output",
            "capability": "passive_observation",
            "params": {
                "buffer_name": api_interrupt_buffer,
                "needle": api_interrupt_start_needle,
                "timeout_seconds": 3.0,
                "preconditions": api_interrupt_submit_after,
            },
        }
        api_interrupt_await_start_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_await_start_request,
            timeout=5.0,
        )
        api_interrupt_await_start_result = api_interrupt_await_start_response.get("result") or {}
        api_interrupt_stale_request = {
            "api_version": "soma-emacs-agent-v0",
            "request_id": "daemon-comint-api-interrupt-stale-1",
            "operation": "comint.interrupt",
            "capability": "external_process_control",
            "params": {
                "buffer_name": api_interrupt_buffer,
                "preconditions": api_interrupt_initial_result,
            },
        }
        api_interrupt_stale_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_stale_request,
            timeout=3.0,
        )
        api_interrupt_stale_result = api_interrupt_stale_response.get("result") or {}
        api_interrupt_before_request = {
            "request_id": "daemon-comint-api-interrupt-before-1",
            "operation": "comint.observe",
            "params": {"buffer_name": api_interrupt_buffer},
        }
        api_interrupt_before_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_before_request,
            timeout=2.0,
        )
        api_interrupt_before_result = api_interrupt_before_response.get("result") or {}
        api_interrupt_request = {
            "api_version": "soma-emacs-agent-v0",
            "request_id": "daemon-comint-api-interrupt-1",
            "operation": "comint.interrupt",
            "capability": "external_process_control",
            "params": {
                "buffer_name": api_interrupt_buffer,
                "preconditions": api_interrupt_before_result,
            },
        }
        api_interrupt_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_request,
            timeout=3.0,
        )
        api_interrupt_result = api_interrupt_response.get("result") or {}
        time.sleep(2.5)
        api_interrupt_final_request = {
            "request_id": "daemon-comint-api-interrupt-final-1",
            "operation": "comint.observe",
            "params": {"buffer_name": api_interrupt_buffer},
        }
        api_interrupt_final_response = request_or_outcome_unknown(
            args.socket,
            api_interrupt_final_request,
            timeout=2.0,
        )
        api_interrupt_final_result = api_interrupt_final_response.get("result") or {}
        api_interrupt_tail = str(api_interrupt_final_result.get("buffer_tail") or "")
        api_interrupt_counter_lines = file_lines(api_interrupt_counter)
        api_interrupt_evidence = api_interrupt_response.get("evidence") or {}
        api_interrupt_effects = api_interrupt_response.get("effects") or {}
        api_interrupt_process_identity = (
            api_interrupt_evidence.get("process_identity")
            if isinstance(api_interrupt_evidence, dict)
            else {}
        )
        api_interrupt_process_survival = (
            api_interrupt_evidence.get("process_survival")
            if isinstance(api_interrupt_evidence, dict)
            else {}
        )
        api_interrupt_process_effect = (
            api_interrupt_effects.get("process_interrupt")
            if isinstance(api_interrupt_effects, dict)
            else {}
        )
        api_interrupt_payload = {
            "fixture": api_interrupt_fixture,
            "initial_request": api_interrupt_initial_request,
            "initial_response": api_interrupt_initial_response,
            "submit_request": api_interrupt_submit_request,
            "submit_response": api_interrupt_submit_response,
            "await_start_request": api_interrupt_await_start_request,
            "await_start_response": api_interrupt_await_start_response,
            "stale_interrupt_request": api_interrupt_stale_request,
            "stale_interrupt_response": api_interrupt_stale_response,
            "before_interrupt_request": api_interrupt_before_request,
            "before_interrupt_response": api_interrupt_before_response,
            "interrupt_request": api_interrupt_request,
            "interrupt_response": api_interrupt_response,
            "final_request": api_interrupt_final_request,
            "final_response": api_interrupt_final_response,
            "counter_lines": api_interrupt_counter_lines,
        }
        api_interrupt_checks = [
            check("Comint API interrupt fixture is present", True, api_interrupt_fixture),
            check("initial API interrupt Comint observation succeeded", api_interrupt_initial_response.get("state") == "succeeded", api_interrupt_initial_response),
            check("initial API interrupt Comint process is live", api_interrupt_initial_result.get("process_live") is True, api_interrupt_initial_result),
            check("initial API interrupt Comint process executable is rootfs-owned", rootfs_owned_process_path(str(api_interrupt_initial_result.get("process_exe") or "")), api_interrupt_initial_result.get("process_exe")),
            check("API interrupt command was submitted through Comint", api_interrupt_submit_response.get("state") == "succeeded" and api_interrupt_submit_result.get("route") == "comint-send-input", api_interrupt_submit_response),
            check("API interrupt fixture reached delayed command before interrupt", api_interrupt_await_start_response.get("state") == "succeeded" and api_interrupt_await_start_result.get("matched") is True and has_exact_line(str(api_interrupt_await_start_result.get("matched_output") or ""), api_interrupt_start_needle), api_interrupt_await_start_response),
            check("stale API interrupt is rejected before side effects", api_interrupt_stale_response.get("state") == "failed" and precondition_check_passed(api_interrupt_stale_response, "Comint process mark matches precondition", False), {"response": api_interrupt_stale_response, "result": api_interrupt_stale_result}),
            check("fresh API interrupt observation succeeded", api_interrupt_before_response.get("state") == "succeeded" and api_interrupt_before_result.get("process_live") is True, api_interrupt_before_response),
            check("comint.interrupt succeeds through the v0 envelope", api_interrupt_response.get("api_version") == "soma-emacs-agent-v0" and api_interrupt_response.get("state") == "succeeded" and api_interrupt_response.get("state_sequence") == ["accepted", "running", "succeeded"], api_interrupt_response),
            check("comint.interrupt uses comint-interrupt-subjob and not raw process control", api_interrupt_result.get("route") == "comint-interrupt-subjob" and api_interrupt_result.get("interrupt_dispatched") is True and api_interrupt_result.get("used_raw_process_write") is False and api_interrupt_result.get("bridge_used_raw_process_control") is False, api_interrupt_result),
            check("comint.interrupt records before and after Comint ownership state", isinstance(api_interrupt_result.get("before"), dict) and isinstance(api_interrupt_result.get("after"), dict) and api_interrupt_result.get("after", {}).get("process_id") == api_interrupt_before_result.get("process_id"), api_interrupt_result),
            check("comint.interrupt publishes typed terminal evidence and process-control effect", isinstance(api_interrupt_evidence, dict) and api_interrupt_evidence.get("interrupt_command") == "comint-interrupt-subjob" and isinstance(api_interrupt_process_identity, dict) and api_interrupt_process_identity.get("process_id") == api_interrupt_before_result.get("process_id") and api_interrupt_process_identity.get("status_before") == api_interrupt_result.get("process_status_before") and isinstance(api_interrupt_process_survival, dict) and api_interrupt_process_survival.get("process_live_after") is True and api_interrupt_evidence.get("side_effect_reconciliation") == "external_verifier_required" and isinstance(api_interrupt_process_effect, dict) and api_interrupt_process_effect.get("command") == "comint-interrupt-subjob" and api_interrupt_process_effect.get("dispatched") is True and api_interrupt_process_effect.get("used_raw_process_write") is False and api_interrupt_process_effect.get("bridge_used_raw_process_control") is False, {"evidence": api_interrupt_evidence, "effects": api_interrupt_effects}),
            check("API interrupted Comint shell process survives", api_interrupt_final_result.get("process_id") == api_interrupt_initial_result.get("process_id") and api_interrupt_final_result.get("process_live") is True, {"initial": api_interrupt_initial_result, "final": api_interrupt_final_result}),
            check("API interrupt command did not reach completion marker", not has_exact_line(api_interrupt_tail, api_interrupt_done_needle), api_interrupt_tail),
            check("API interrupt side effect did not occur", api_interrupt_counter_lines == [], api_interrupt_counter_lines),
        ]
    else:
        api_interrupt_payload = {
            "fixture": api_interrupt_fixture,
            "initial_request": None,
            "initial_response": None,
            "submit_request": None,
            "submit_response": None,
            "await_start_request": None,
            "await_start_response": None,
            "stale_interrupt_request": None,
            "stale_interrupt_response": None,
            "before_interrupt_request": None,
            "before_interrupt_response": None,
            "interrupt_request": None,
            "interrupt_response": None,
            "final_request": None,
            "final_response": None,
            "counter_lines": [],
        }
        api_interrupt_checks = [
            check("Comint API interrupt fixture is present", False, api_interrupt_fixture)
        ]
    records.append(
        scenario_record(
            36,
            "daemon-comint-interrupt-api",
            "The live daemon semantic Comint API interrupts the intended subprocess and reconciles side effects",
            True,
            args.out_dir,
            **api_interrupt_payload,
            checks=api_interrupt_checks,
        )
    )

    stale_magit_fixture = (ready or {}).get("stale_magit_fixture") or {}
    stale_magit_repo = stale_magit_fixture.get("repo") or ""
    stale_magit_buffer = stale_magit_fixture.get("status_buffer") or ""
    stale_magit_hook_log = stale_magit_fixture.get("hook_log") or ""
    stale_magit_target = f"{args.terminal_tmux_session}:0.0"
    stale_magit_initial_state = magit_fixture_repo_state(
        stale_magit_repo,
        stale_magit_hook_log,
    )
    stale_magit_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        stale_magit_target,
        stale_magit_buffer,
        "daemon-magit-stale",
    )
    stale_initial_sections_request = {
        "request_id": "daemon-magit-stale-initial-sections-1",
        "operation": "magit.observe_sections",
        "params": {
            "buffer_name": stale_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    stale_initial_sections_response = request_or_outcome_unknown(
        args.socket,
        stale_initial_sections_request,
        timeout=3.0,
    )
    stale_initial_sections = stale_initial_sections_response.get("result") or {}
    stale_initial_target_section = find_magit_section(
        stale_initial_sections,
        section_type="hunk",
        needle="target-05-selected",
        ancestor_file="target.txt",
    )
    stale_initial_target_point = magit_section_point(stale_initial_target_section)
    if isinstance(stale_initial_target_point, int):
        stale_initial_goto_request = {
            "request_id": "daemon-magit-stale-initial-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": stale_magit_buffer,
                "point": stale_initial_target_point,
                "preconditions": stale_initial_sections,
            },
        }
        stale_initial_goto_response = request_or_outcome_unknown(
            args.socket,
            stale_initial_goto_request,
            timeout=2.0,
        )
    else:
        stale_initial_goto_request = {
            "request_id": "daemon-magit-stale-initial-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": stale_magit_buffer,
                "point": None,
                "preconditions": stale_initial_sections,
            },
        }
        stale_initial_goto_response = {
            "request_id": "daemon-magit-stale-initial-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_target_point",
            "error": "stale Magit fixture did not expose an initial target hunk point",
        }
    stale_ready_sections_request = {
        "request_id": "daemon-magit-stale-ready-sections-1",
        "operation": "magit.observe_sections",
        "params": {
            "buffer_name": stale_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    stale_ready_sections_response = request_or_outcome_unknown(
        args.socket,
        stale_ready_sections_request,
        timeout=3.0,
    )
    stale_ready_sections = stale_ready_sections_response.get("result") or {}
    stale_ready_selected = stale_ready_sections.get("selected_section") or {}

    stale_external_write: dict[str, Any]
    if stale_magit_repo:
        stale_external_path = pathlib.Path(stale_magit_repo) / "target.txt"
        stale_external_path.write_text(
            target_content(selected=True, other=False, external=True),
            encoding="utf-8",
        )
        stale_external_write = {
            "path": str(stale_external_path),
            "content": target_content(selected=True, other=False, external=True),
        }
    else:
        stale_external_write = {
            "path": "",
            "content": "",
            "error": "missing stale Magit repository path",
        }
    stale_after_external_state = magit_fixture_repo_state(
        stale_magit_repo,
        stale_magit_hook_log,
    )

    stale_refresh_request = {
        "request_id": "daemon-magit-stale-refresh-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": stale_magit_buffer,
            "command": "magit-refresh",
            "preconditions": {
                "buffer_name": stale_ready_sections.get("buffer_name"),
                "selected_window_buffer": stale_ready_sections.get("selected_window_buffer"),
                "chars_tick": stale_ready_sections.get("chars_tick"),
                "point": stale_ready_sections.get("point"),
                "mark_active": stale_ready_sections.get("mark_active"),
                "narrowed": stale_ready_sections.get("narrowed"),
                "read_only": stale_ready_sections.get("read_only"),
                "minibuffer_active": stale_ready_sections.get("minibuffer_active"),
            },
        },
    }
    stale_refresh_response = request_or_outcome_unknown(
        args.socket,
        stale_refresh_request,
        timeout=2.0,
    )
    stale_refresh_await_request = {
        "request_id": "daemon-magit-stale-refresh-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-magit-stale-refresh-1",
            "timeout_seconds": 10.0,
        },
    }
    stale_refresh_await_response = request_or_outcome_unknown(
        args.socket,
        stale_refresh_await_request,
        timeout=12.0,
    )
    stale_refresh_await_result = stale_refresh_await_response.get("result") or {}
    stale_refresh_status = (
        stale_refresh_await_result.get("status")
        or stale_refresh_response
    )
    stale_refreshed_sections_request = {
        "request_id": "daemon-magit-stale-refreshed-sections-1",
        "operation": "magit.observe_sections",
        "params": {
            "buffer_name": stale_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    stale_refreshed_sections_response = request_or_outcome_unknown(
        args.socket,
        stale_refreshed_sections_request,
        timeout=3.0,
    )
    stale_refreshed_sections = stale_refreshed_sections_response.get("result") or {}
    stale_refreshed_initial_section = find_magit_section(
        stale_refreshed_sections,
        section_type="hunk",
        needle="target-05-selected",
        ancestor_file="target.txt",
    )
    stale_refreshed_external_section = find_magit_section(
        stale_refreshed_sections,
        section_type="hunk",
        needle="target-25-external",
        ancestor_file="target.txt",
    )

    stale_replay_request = {
        "request_id": "daemon-magit-stale-replay-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": stale_magit_buffer,
            "command": "magit-stage",
            "preconditions": stale_ready_sections,
        },
    }
    stale_replay_response = request_or_outcome_unknown(
        args.socket,
        stale_replay_request,
        timeout=2.0,
    )
    stale_after_replay_state = magit_fixture_repo_state(
        stale_magit_repo,
        stale_magit_hook_log,
    )

    stale_refreshed_target_point = magit_section_point(stale_refreshed_initial_section)
    if isinstance(stale_refreshed_target_point, int):
        stale_refreshed_goto_request = {
            "request_id": "daemon-magit-stale-refreshed-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": stale_magit_buffer,
                "point": stale_refreshed_target_point,
                "preconditions": stale_refreshed_sections,
            },
        }
        stale_refreshed_goto_response = request_or_outcome_unknown(
            args.socket,
            stale_refreshed_goto_request,
            timeout=2.0,
        )
    else:
        stale_refreshed_goto_request = {
            "request_id": "daemon-magit-stale-refreshed-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": stale_magit_buffer,
                "point": None,
                "preconditions": stale_refreshed_sections,
            },
        }
        stale_refreshed_goto_response = {
            "request_id": "daemon-magit-stale-refreshed-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_refreshed_target_point",
            "error": "refreshed Magit observation did not expose target hunk point",
        }
    stale_final_ready_sections_request = {
        "request_id": "daemon-magit-stale-final-ready-sections-1",
        "operation": "magit.observe_sections",
        "params": {
            "buffer_name": stale_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    stale_final_ready_sections_response = request_or_outcome_unknown(
        args.socket,
        stale_final_ready_sections_request,
        timeout=3.0,
    )
    stale_final_ready_sections = stale_final_ready_sections_response.get("result") or {}
    stale_final_selected_section = stale_final_ready_sections.get("selected_section") or {}
    stale_recovery_request = {
        "request_id": "daemon-magit-stale-recovery-stage-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": stale_magit_buffer,
            "command": "magit-stage",
            "preconditions": stale_final_ready_sections,
        },
    }
    stale_recovery_response = request_or_outcome_unknown(
        args.socket,
        stale_recovery_request,
        timeout=5.0,
    )
    stale_recovery_status_request = {
        "request_id": "daemon-magit-stale-recovery-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-magit-stale-recovery-stage-1"},
    }
    stale_recovery_status_response = request_or_outcome_unknown(
        args.socket,
        stale_recovery_status_request,
        timeout=2.0,
    )
    stale_recovery_status_result = stale_recovery_status_response.get("result") or {}
    stale_recovery_running_status = stale_recovery_status_result.get("status") or {}
    stale_recovery_await_request = {
        "request_id": "daemon-magit-stale-recovery-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-magit-stale-recovery-stage-1",
            "timeout_seconds": 10.0,
        },
    }
    stale_recovery_await_response = request_or_outcome_unknown(
        args.socket,
        stale_recovery_await_request,
        timeout=12.0,
    )
    stale_recovery_await_result = stale_recovery_await_response.get("result") or {}
    stale_recovery_final_status = (
        stale_recovery_await_result.get("status")
        or stale_recovery_running_status
        or stale_recovery_response
    )
    stale_recovery_result = stale_recovery_final_status.get("result") or {}
    stale_final_state = magit_fixture_repo_state(
        stale_magit_repo,
        stale_magit_hook_log,
    )
    records.append(
        scenario_record(
            20,
            "daemon-magit-stale-view-recovery",
            "The daemon bridge rejects stale Magit view preconditions, refreshes, re-observes sections, and stages only the refreshed intended hunk",
            True,
            args.out_dir,
            fixture=stale_magit_fixture,
            initial_repo_state=stale_magit_initial_state,
            terminal_switch={
                "tmux_target": stale_magit_target,
                "cancel": stale_magit_switch["cancel"],
                "cancel_attempts": stale_magit_switch["cancel_attempts"],
                "cancel_interaction_response": stale_magit_switch["cancel_interaction_response"],
                "prefix": stale_magit_switch["prefix"],
                "minibuffer_attempts": stale_magit_switch["minibuffer_attempts"],
                "buffer_keys": stale_magit_switch["buffer_keys"],
                "enter": stale_magit_switch["enter"],
                "observation_attempts": stale_magit_switch["observation_attempts"],
                "response": stale_magit_switch["response"],
            },
            initial_sections_request=stale_initial_sections_request,
            initial_sections_response=stale_initial_sections_response,
            initial_target_section=stale_initial_target_section,
            initial_goto_request=stale_initial_goto_request,
            initial_goto_response=stale_initial_goto_response,
            ready_sections_request=stale_ready_sections_request,
            ready_sections_response=stale_ready_sections_response,
            ready_selected_section=stale_ready_selected,
            external_write=stale_external_write,
            after_external_state=stale_after_external_state,
            refresh_request=stale_refresh_request,
            refresh_response=stale_refresh_response,
            refresh_await_request=stale_refresh_await_request,
            refresh_await_response=stale_refresh_await_response,
            refresh_status=stale_refresh_status,
            refreshed_sections_request=stale_refreshed_sections_request,
            refreshed_sections_response=stale_refreshed_sections_response,
            refreshed_initial_section=stale_refreshed_initial_section,
            refreshed_external_section=stale_refreshed_external_section,
            stale_replay_request=stale_replay_request,
            stale_replay_response=stale_replay_response,
            after_stale_replay_state=stale_after_replay_state,
            refreshed_goto_request=stale_refreshed_goto_request,
            refreshed_goto_response=stale_refreshed_goto_response,
            final_ready_sections_request=stale_final_ready_sections_request,
            final_ready_sections_response=stale_final_ready_sections_response,
            final_selected_section=stale_final_selected_section,
            recovery_request=stale_recovery_request,
            recovery_response=stale_recovery_response,
            recovery_status_request=stale_recovery_status_request,
            recovery_status_response=stale_recovery_status_response,
            recovery_await_request=stale_recovery_await_request,
            recovery_await_response=stale_recovery_await_response,
            recovery_final_status=stale_recovery_final_status,
            final_repo_state=stale_final_state,
            checks=[
                check("stale Magit fixture is present", bool(stale_magit_repo and stale_magit_buffer), stale_magit_fixture),
                check("stale Magit terminal switch cancel succeeded", stale_magit_switch["cancel"]["returncode"] == 0, stale_magit_switch["cancel"]),
                check("stale Magit terminal switch prefix succeeded", stale_magit_switch["prefix"]["returncode"] == 0, stale_magit_switch["prefix"]),
                check("stale Magit terminal switch buffer text sent", stale_magit_switch["buffer_keys"]["returncode"] == 0, stale_magit_switch["buffer_keys"]),
                check("stale Magit terminal switch enter succeeded", stale_magit_switch["enter"]["returncode"] == 0, stale_magit_switch["enter"]),
                check("client terminal frame displays stale Magit buffer", len(stale_magit_switch["terminal_frames"]) >= 1, stale_magit_switch["result"].get("frames")),
                check("initial Magit section observation succeeded", stale_initial_sections_response.get("state") == "succeeded", stale_initial_sections_response),
                check("initial target hunk observed through bridge", bool(stale_initial_target_section) and contains(stale_initial_target_section.get("text_excerpt") or "", "target-05-selected"), stale_initial_target_section),
                check("initial observation excludes later external hunk", not contains(str(stale_initial_sections.get("sections") or ""), "target-25-external"), stale_initial_sections),
                check("checked initial target navigation succeeded", stale_initial_goto_response.get("state") == "succeeded", stale_initial_goto_response),
                check("ready section observation succeeded", stale_ready_sections_response.get("state") == "succeeded", stale_ready_sections_response),
                check("ready selected section is initial intended hunk", contains(stale_ready_selected.get("text_excerpt") or "", "target-05-selected"), stale_ready_selected),
                check("external rootfs-side mutation changed worktree only", contains(stale_after_external_state.get("target_worktree") or "", "target-25-external") and contains(stale_after_external_state.get("target_index") or "", "target-25-base") and stale_after_external_state.get("cached_names") == ["staged.txt"], stale_after_external_state),
                check("Magit refresh request accepted as running", stale_refresh_response.get("state") == "running", stale_refresh_response),
                check("Magit refresh request reaches terminal success", stale_refresh_await_response.get("state") == "succeeded" and stale_refresh_status.get("state") == "succeeded", {"await": stale_refresh_await_response, "status": stale_refresh_status}),
                check("Magit refresh ran through command-execute", (stale_refresh_status.get("result") or {}).get("route") == "command-execute", stale_refresh_status),
                check("Magit refresh command identity is preserved", (stale_refresh_status.get("result") or {}).get("this_command") == "magit-refresh", stale_refresh_status),
                check("refreshed Magit section observation succeeded", stale_refreshed_sections_response.get("state") == "succeeded", stale_refreshed_sections_response),
                check("refreshed observation detects changed worktree fingerprint", stale_refreshed_sections.get("magit_worktree_diff_hash") != stale_ready_sections.get("magit_worktree_diff_hash"), {"before": stale_ready_sections.get("magit_worktree_diff_hash"), "after": stale_refreshed_sections.get("magit_worktree_diff_hash")}),
                check("refreshed observation includes intended hunk", bool(stale_refreshed_initial_section) and contains(stale_refreshed_initial_section.get("text_excerpt") or "", "target-05-selected"), stale_refreshed_initial_section),
                check("refreshed observation includes external hunk separately", bool(stale_refreshed_external_section) and contains(stale_refreshed_external_section.get("text_excerpt") or "", "target-25-external"), stale_refreshed_external_section),
                check("stale replay is rejected before command mutation", stale_replay_response.get("state") == "failed", stale_replay_response),
                check("stale replay reports Magit worktree conflict", precondition_check_passed(stale_replay_response, "Magit worktree diff hash matches precondition", False), stale_replay_response),
                check("stale replay left target index untouched", stale_after_replay_state.get("cached_names") == ["staged.txt"] and contains(stale_after_replay_state.get("target_index") or "", "target-05-base") and contains(stale_after_replay_state.get("target_worktree") or "", "target-05-selected") and contains(stale_after_replay_state.get("target_worktree") or "", "target-25-external"), stale_after_replay_state),
                check("refreshed target navigation succeeded", stale_refreshed_goto_response.get("state") == "succeeded", stale_refreshed_goto_response),
                check("final ready section observation succeeded", stale_final_ready_sections_response.get("state") == "succeeded", stale_final_ready_sections_response),
                check("final selected section is refreshed intended hunk", contains(stale_final_selected_section.get("text_excerpt") or "", "target-05-selected"), stale_final_selected_section),
                check("recovery Magit command accepted as running", stale_recovery_response.get("state") == "running", stale_recovery_response),
                check("recovery queued status is observable", stale_recovery_status_response.get("state") == "succeeded" and stale_recovery_running_status.get("request_id") == "daemon-magit-stale-recovery-stage-1", stale_recovery_status_response),
                check("recovery Magit command reaches terminal success", stale_recovery_await_response.get("state") == "succeeded" and stale_recovery_final_status.get("state") == "succeeded", {"await": stale_recovery_await_response, "final": stale_recovery_final_status}),
                check("recovery command ran through command-execute", stale_recovery_result.get("route") == "command-execute", stale_recovery_result),
                check("recovery command identity is preserved", stale_recovery_result.get("this_command") == "magit-stage", stale_recovery_result),
                check("recovery preconditions passed", all(item.get("passed") is True for item in stale_recovery_result.get("precondition_checks") or []), stale_recovery_result.get("precondition_checks")),
                check("final stale-recovery cached names are exact", stale_final_state.get("cached_names") == ["staged.txt", "target.txt"], stale_final_state),
                check("final stale-recovery unstaged names preserve unrelated work", stale_final_state.get("unstaged_names") == ["debug.txt", "target.txt"], stale_final_state),
                check("final selected change is staged", contains(stale_final_state.get("target_index") or "", "target-05-selected"), stale_final_state.get("target_index")),
                check("final external hunk remains unstaged", contains(stale_final_state.get("target_index") or "", "target-25-base") and contains(stale_final_state.get("target_worktree") or "", "target-25-external"), {"index": stale_final_state.get("target_index"), "worktree": stale_final_state.get("target_worktree")}),
                check("final debug change remains unstaged", stale_final_state.get("debug_index") == "debug-base\n" and contains(stale_final_state.get("debug_worktree") or "", "debug-worktree-only"), {"index": stale_final_state.get("debug_index"), "worktree": stale_final_state.get("debug_worktree")}),
                check("final unrelated staged file is preserved", stale_final_state.get("staged_index") == "staged-expected\n", stale_final_state.get("staged_index")),
                check("stale-recovery post-stage hook observed command identity", contains(stale_final_state.get("hook_log") or "", "post-stage:magit-stage"), stale_final_state.get("hook_log")),
                check("stale-recovery Magit refresh hook ran", contains(stale_final_state.get("hook_log") or "", "refresh:"), stale_final_state.get("hook_log")),
            ],
        )
    )

    jupyter_fixture = (ready or {}).get("jupyter_fixture") or {}
    jupyter_buffer = jupyter_fixture.get("source_buffer") or ""
    jupyter_beg = jupyter_fixture.get("beg")
    jupyter_end = jupyter_fixture.get("end")
    jupyter_selected_code = jupyter_fixture.get("selected_code") or ""
    jupyter_training_code = jupyter_fixture.get("training_code") or ""
    jupyter_target = f"{args.terminal_tmux_session}:0.0"
    jupyter_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        jupyter_target,
        jupyter_buffer,
        "daemon-jupyter-existing",
    )
    jupyter_observe_request = {
        "request_id": "daemon-jupyter-existing-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_buffer,
            "start": jupyter_beg,
            "end": jupyter_end,
            "max_text": 1600,
        },
    }
    jupyter_observe_response = request_or_outcome_unknown(
        args.socket,
        jupyter_observe_request,
        timeout=5.0,
    )
    jupyter_observation = jupyter_observe_response.get("result") or {}
    jupyter_observe_evidence = jupyter_observe_response.get("evidence") or {}
    jupyter_observe_effects = jupyter_observe_response.get("effects") or {}
    jupyter_selected_range = jupyter_observation.get("selected_range") or {}
    jupyter_eval_request = {
        "request_id": "daemon-jupyter-existing-eval-1",
        "operation": "jupyter.eval_region",
        "params": {
            "buffer_name": jupyter_buffer,
            "start": jupyter_beg,
            "end": jupyter_end,
            "timeout_seconds": 30.0,
            "preconditions": jupyter_observation,
        },
    }
    jupyter_eval_response = request_or_outcome_unknown(
        args.socket,
        jupyter_eval_request,
        timeout=2.0,
    )
    jupyter_status_request = {
        "request_id": "daemon-jupyter-existing-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-jupyter-existing-eval-1"},
    }
    jupyter_status_response = request_or_outcome_unknown(
        args.socket,
        jupyter_status_request,
        timeout=2.0,
    )
    jupyter_status_result = jupyter_status_response.get("result") or {}
    jupyter_observed_status = jupyter_status_result.get("status") or {}
    jupyter_await_request = {
        "request_id": "daemon-jupyter-existing-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-jupyter-existing-eval-1",
            "timeout_seconds": 40.0,
        },
    }
    jupyter_await_response = request_or_outcome_unknown(
        args.socket,
        jupyter_await_request,
        timeout=45.0,
    )
    jupyter_await_result = jupyter_await_response.get("result") or {}
    jupyter_await_status = jupyter_await_result.get("status") or {}
    jupyter_semantic_await_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-jupyter-existing-semantic-await-1",
        "operation": "jupyter.await_request",
        "capability": "passive_observation",
        "params": {
            "request_id": "daemon-jupyter-existing-eval-1",
            "timeout_seconds": 40.0,
        },
    }
    jupyter_semantic_await_response = request_or_outcome_unknown(
        args.socket,
        jupyter_semantic_await_request,
        timeout=45.0,
    )
    jupyter_semantic_await_result = (
        jupyter_semantic_await_response.get("result") or {}
    )
    jupyter_semantic_await_evidence = (
        jupyter_semantic_await_response.get("evidence") or {}
    )
    jupyter_semantic_await_effects = (
        jupyter_semantic_await_response.get("effects") or {}
    )
    jupyter_semantic_await_status = (
        jupyter_semantic_await_result.get("status") or {}
    )
    jupyter_semantic_await_eval_result = (
        jupyter_semantic_await_status.get("result") or {}
    )
    if jupyter_await_status.get("state") in TERMINAL_REQUEST_STATES:
        jupyter_final_status = jupyter_await_status
        jupyter_final_status_source = "request.await"
    elif jupyter_semantic_await_status.get("state") in TERMINAL_REQUEST_STATES:
        jupyter_final_status = jupyter_semantic_await_status
        jupyter_final_status_source = "jupyter.await_request"
    else:
        jupyter_final_status = jupyter_observed_status or jupyter_eval_response
        jupyter_final_status_source = "request.status"
    jupyter_eval_result = jupyter_final_status.get("result") or {}
    jupyter_eval_evidence = jupyter_final_status.get("evidence") or {}
    jupyter_eval_effects = jupyter_final_status.get("effects") or {}
    jupyter_kernel_execution_effect = (
        jupyter_eval_effects.get("kernel_execution")
        if isinstance(jupyter_eval_effects, dict)
        else {}
    )
    jupyter_stream_payload = stream_json_payload(
        jupyter_eval_result.get("stream_texts") or [],
        "SOMA_SELECTED_JSON:",
    )
    jupyter_query_request = {
        "request_id": "daemon-jupyter-existing-query-1",
        "operation": "jupyter.query_state",
        "params": {
            "buffer_name": jupyter_buffer,
            "timeout_seconds": 30.0,
        },
    }
    jupyter_query_response = request_or_outcome_unknown(
        args.socket,
        jupyter_query_request,
        timeout=35.0,
    )
    jupyter_query_result = jupyter_query_response.get("result") or {}
    jupyter_payload = jupyter_query_result.get("payload") or {}
    jupyter_finish_request = {
        "request_id": "daemon-jupyter-existing-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-jupyter-existing-eval-1",
                    "state": "succeeded",
                },
                {
                    "request_id": "daemon-jupyter-existing-query-1",
                    "state": "succeeded",
                },
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": jupyter_buffer,
                    "start": jupyter_beg,
                    "end": jupyter_end,
                    "preconditions": {
                        "buffer_name": jupyter_observation.get("buffer_name"),
                        "text_owner_name": jupyter_observation.get("text_owner_name"),
                        "chars_tick": jupyter_observation.get("chars_tick"),
                        "expected_text": jupyter_selected_code,
                    },
                }
            ],
        },
    }
    jupyter_finish_response = request_or_outcome_unknown(
        args.socket,
        jupyter_finish_request,
        timeout=5.0,
    )
    records.append(
        scenario_record(
                21,
            "daemon-jupyter-existing-kernel-eval-region",
            "The live daemon bridge evaluates only the selected region on an existing rootfs-owned Jupyter kernel",
            True,
            args.out_dir,
            fixture=jupyter_fixture,
            terminal_switch={
                "tmux_target": jupyter_target,
                "cancel": jupyter_switch["cancel"],
                "cancel_attempts": jupyter_switch["cancel_attempts"],
                "cancel_interaction_response": jupyter_switch["cancel_interaction_response"],
                "prefix": jupyter_switch["prefix"],
                "minibuffer_attempts": jupyter_switch["minibuffer_attempts"],
                "buffer_keys": jupyter_switch["buffer_keys"],
                "enter": jupyter_switch["enter"],
                "observation_attempts": jupyter_switch["observation_attempts"],
                "response": jupyter_switch["response"],
            },
            observe_request=jupyter_observe_request,
            observe_response=jupyter_observe_response,
            eval_request=jupyter_eval_request,
            eval_response=jupyter_eval_response,
            status_request=jupyter_status_request,
            status_response=jupyter_status_response,
            await_request=jupyter_await_request,
            await_response=jupyter_await_response,
            semantic_await_request=jupyter_semantic_await_request,
            semantic_await_response=jupyter_semantic_await_response,
            final_status=jupyter_final_status,
            final_status_source=jupyter_final_status_source,
            stream_payload=jupyter_stream_payload,
            query_request=jupyter_query_request,
            query_response=jupyter_query_response,
            finish_request=jupyter_finish_request,
            finish_response=jupyter_finish_response,
            checks=[
                check("Jupyter fixture is present", bool(jupyter_buffer and isinstance(jupyter_beg, int) and isinstance(jupyter_end, int)), jupyter_fixture),
                check("Jupyter terminal switch cancel succeeded", jupyter_switch["cancel"]["returncode"] == 0, jupyter_switch["cancel"]),
                check("Jupyter terminal switch prefix succeeded", jupyter_switch["prefix"]["returncode"] == 0, jupyter_switch["prefix"]),
                check("Jupyter terminal switch buffer text sent", jupyter_switch["buffer_keys"]["returncode"] == 0, jupyter_switch["buffer_keys"]),
                check("Jupyter terminal switch enter succeeded", jupyter_switch["enter"]["returncode"] == 0, jupyter_switch["enter"]),
                check("client terminal frame displays Jupyter source buffer", len(jupyter_switch["terminal_frames"]) >= 1, jupyter_switch["result"].get("frames")),
                check("Jupyter helper executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_fixture.get("helper_executable") or "")), jupyter_fixture.get("helper_executable")),
                check("Jupyter kernel executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_fixture.get("kernel_exe") or "")), jupyter_fixture.get("kernel_exe")),
                check("Jupyter runtime directory is private project run state", str(jupyter_fixture.get("jupyter_runtime_dir") or "").startswith("/project/run/jupyter"), jupyter_fixture.get("jupyter_runtime_dir")),
                check("Jupyter source observation succeeded", jupyter_observe_response.get("state") == "succeeded", jupyter_observe_response),
                check("jupyter.observe_source publishes typed source/client/kernel evidence without effects", isinstance(jupyter_observe_evidence, dict) and isinstance(jupyter_observe_evidence.get("source_buffer"), dict) and jupyter_observe_evidence["source_buffer"].get("buffer_name") == jupyter_buffer and isinstance(jupyter_observe_evidence.get("client"), dict) and jupyter_observe_evidence["client"].get("associated") is True and jupyter_observe_evidence["client"].get("session_id") == jupyter_fixture.get("client_session_id") and isinstance(jupyter_observe_evidence.get("kernel_incarnation"), dict) and jupyter_observe_evidence["kernel_incarnation"].get("kernel_token") == jupyter_fixture.get("kernel_token") and isinstance(jupyter_observe_evidence.get("selected_region"), dict) and jupyter_observe_evidence["selected_region"].get("text_hash") == jupyter_fixture.get("selected_sha256") and isinstance(jupyter_observe_evidence.get("rootfs_runtime"), dict) and jupyter_observe_evidence["rootfs_runtime"].get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "") and not jupyter_observe_effects, {"evidence": jupyter_observe_evidence, "effects": jupyter_observe_effects}),
                check("Jupyter source buffer is associated", jupyter_observation.get("jupyter_client_associated") is True, jupyter_observation),
                check("Jupyter connection file matches existing fixture", jupyter_observation.get("jupyter_connection_file") == jupyter_fixture.get("connection_file"), {"observation": jupyter_observation, "fixture": jupyter_fixture}),
                check("Jupyter kernel token matches existing fixture", jupyter_observation.get("jupyter_kernel_token") == jupyter_fixture.get("kernel_token"), {"observation": jupyter_observation, "fixture": jupyter_fixture}),
                check("Jupyter client session matches existing fixture", jupyter_observation.get("jupyter_client_session_id") == jupyter_fixture.get("client_session_id"), {"observation": jupyter_observation, "fixture": jupyter_fixture}),
                check("selected range hash matches fixture", jupyter_selected_range.get("text_hash") == jupyter_fixture.get("selected_sha256"), {"selected_range": jupyter_selected_range, "fixture_sha": jupyter_fixture.get("selected_sha256")}),
                check("Jupyter eval request is accepted as running", jupyter_eval_response.get("state") == "running", jupyter_eval_response),
                check("Jupyter eval status is observable", jupyter_status_response.get("state") == "succeeded" and jupyter_observed_status.get("request_id") == "daemon-jupyter-existing-eval-1", jupyter_status_response),
                check("Jupyter generic await either succeeds or leaves an explicit unknown outcome for reconciliation", jupyter_await_response.get("state") == "succeeded" or (jupyter_await_response.get("state") == "outcome_unknown" and jupyter_await_response.get("error_code") in {"timeout", "client_timeout"}), jupyter_await_response),
                check("Jupyter eval reaches terminal success", jupyter_final_status.get("state") == "succeeded" and jupyter_final_status.get("request_id") == "daemon-jupyter-existing-eval-1" and jupyter_final_status_source in {"request.await", "jupyter.await_request"}, {"generic_await": jupyter_await_response, "semantic_await": jupyter_semantic_await_response, "final": jupyter_final_status, "source": jupyter_final_status_source}),
                check("semantic Jupyter await reaches terminal success", jupyter_semantic_await_response.get("state") == "succeeded" and jupyter_semantic_await_result.get("route") == "jupyter-request-await" and jupyter_semantic_await_status.get("state") == "succeeded" and jupyter_semantic_await_result.get("target_operation") == "jupyter.eval_region", jupyter_semantic_await_response),
                check("semantic Jupyter await exposes original request evidence without resubmission", jupyter_semantic_await_eval_result.get("request_id") == jupyter_eval_result.get("request_id") and jupyter_semantic_await_eval_result.get("reply_parent") == jupyter_eval_result.get("reply_parent") and jupyter_semantic_await_eval_result.get("stream_texts") == jupyter_eval_result.get("stream_texts"), {"semantic": jupyter_semantic_await_eval_result, "generic": jupyter_eval_result}),
                check("jupyter.await_request publishes typed await evidence without kernel effects", isinstance(jupyter_semantic_await_evidence, dict) and isinstance(jupyter_semantic_await_evidence.get("target_status"), dict) and jupyter_semantic_await_evidence["target_status"].get("state") == "succeeded" and isinstance(jupyter_semantic_await_evidence.get("reply"), dict) and jupyter_semantic_await_evidence["reply"].get("status") == "ok" and jupyter_semantic_await_evidence["reply"].get("parent") == jupyter_eval_result.get("reply_parent") and isinstance(jupyter_semantic_await_evidence.get("iopub"), dict) and jupyter_semantic_await_evidence["iopub"].get("stream_texts") == jupyter_eval_result.get("stream_texts") and isinstance(jupyter_semantic_await_evidence.get("timeout"), dict) and jupyter_semantic_await_evidence["timeout"].get("seconds") == 40.0 and not jupyter_semantic_await_effects, {"evidence": jupyter_semantic_await_evidence, "effects": jupyter_semantic_await_effects}),
                check("Jupyter eval uses jupyter-eval-region route", jupyter_eval_result.get("route") == "jupyter-eval-region", jupyter_eval_result),
                check("jupyter.eval_region publishes typed kernel-execution evidence and no editor text effect", isinstance(jupyter_eval_evidence, dict) and jupyter_eval_evidence.get("request_id") == jupyter_eval_result.get("request_id") and isinstance(jupyter_eval_evidence.get("reply"), dict) and jupyter_eval_evidence["reply"].get("status") == "ok" and jupyter_eval_evidence["reply"].get("parent") == jupyter_eval_result.get("reply_parent") and isinstance(jupyter_eval_evidence.get("iopub"), dict) and jupyter_eval_evidence["iopub"].get("stream_texts") == jupyter_eval_result.get("stream_texts") and isinstance(jupyter_eval_evidence.get("source_region"), dict) and jupyter_eval_evidence["source_region"].get("start") == jupyter_beg and jupyter_eval_evidence["source_region"].get("end") == jupyter_end and isinstance(jupyter_eval_evidence.get("rootfs_runtime"), dict) and jupyter_eval_evidence["rootfs_runtime"].get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "") and isinstance(jupyter_kernel_execution_effect, dict) and jupyter_kernel_execution_effect.get("route") == "jupyter-eval-region" and jupyter_kernel_execution_effect.get("request_id") == jupyter_eval_result.get("request_id") and jupyter_kernel_execution_effect.get("submitted_hash") == jupyter_eval_result.get("submitted_hash") and "changed_range" not in jupyter_eval_effects and "changed_ranges" not in jupyter_eval_effects, {"evidence": jupyter_eval_evidence, "effects": jupyter_eval_effects}),
                check("Jupyter eval preconditions passed", all(item.get("passed") is True for item in jupyter_eval_result.get("precondition_checks") or []), jupyter_eval_result.get("precondition_checks")),
                check("Jupyter submitted exact selected region", jupyter_eval_result.get("submitted_code") == jupyter_selected_code, {"submitted": jupyter_eval_result.get("submitted_code"), "expected": jupyter_selected_code}),
                check("Jupyter submitted code excludes training cell", jupyter_training_code not in str(jupyter_eval_result.get("submitted_code") or ""), jupyter_eval_result.get("submitted_code")),
                check("Jupyter execute reply is ok", jupyter_eval_result.get("reply_status") == "ok", jupyter_eval_result),
                check("Jupyter reply parent matches request id", jupyter_eval_result.get("reply_parent") == jupyter_eval_result.get("request_id") or jupyter_eval_result.get("request_id") in (jupyter_eval_result.get("parent_ids") or []), jupyter_eval_result),
                check("Jupyter stream payload reports intended kernel", jupyter_stream_payload.get("kernel_token") == jupyter_fixture.get("kernel_token"), jupyter_stream_payload),
                check("Jupyter stream payload reports selected side effect once", jupyter_stream_payload.get("counter") == 1, jupyter_stream_payload),
                check("Jupyter stream payload reports training cell untouched", jupyter_stream_payload.get("training_executions") == 0, jupyter_stream_payload),
                check("Jupyter state query succeeded", jupyter_query_response.get("state") == "succeeded", jupyter_query_response),
                check("Jupyter state query sees same kernel token", jupyter_payload.get("kernel_token") == jupyter_fixture.get("kernel_token"), jupyter_payload),
                check("Jupyter state query sees stable run id", bool(os.environ.get("SOMA_INSULA_RUN_ID", "")) and jupyter_payload.get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", ""), jupyter_payload),
                check("Jupyter state query sees selected side effect exactly once", jupyter_payload.get("counter") == 1, jupyter_payload),
                check("Jupyter state query sees training cell unexecuted", jupyter_payload.get("training_executions") == 0, jupyter_payload),
                check("Jupyter state query executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_payload.get("executable") or "")), jupyter_payload.get("executable")),
                check("Jupyter state query cwd is repo mount", jupyter_payload.get("cwd") == "/workspace/soma", jupyter_payload),
                check("Jupyter state query plan is repo-mounted", str(jupyter_payload.get("plan") or "").startswith("/workspace/soma/"), jupyter_payload),
                check("Jupyter episode finish succeeds with eval/query evidence", jupyter_finish_response.get("state") == "succeeded", jupyter_finish_response),
            ],
        )
    )

    jupyter_display_fixture = (ready or {}).get("jupyter_display_fixture") or {}
    jupyter_display_buffer = jupyter_display_fixture.get("source_buffer") or ""
    jupyter_display_beg = jupyter_display_fixture.get("beg")
    jupyter_display_end = jupyter_display_fixture.get("end")
    jupyter_display_selected_code = jupyter_display_fixture.get("selected_code") or ""
    jupyter_display_training_code = jupyter_display_fixture.get("training_code") or ""
    jupyter_display_id = jupyter_display_fixture.get("display_id") or ""
    jupyter_display_target = f"{args.terminal_tmux_session}:0.0"
    jupyter_display_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        jupyter_display_target,
        jupyter_display_buffer,
        "daemon-jupyter-display-update",
    )
    jupyter_display_observe_request = {
        "request_id": "daemon-jupyter-display-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_display_buffer,
            "start": jupyter_display_beg,
            "end": jupyter_display_end,
            "max_text": 2400,
        },
    }
    jupyter_display_observe_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_observe_request,
        timeout=5.0,
    )
    jupyter_display_observation = jupyter_display_observe_response.get("result") or {}
    jupyter_display_observe_evidence = (
        jupyter_display_observe_response.get("evidence") or {}
    )
    jupyter_display_observe_effects = (
        jupyter_display_observe_response.get("effects") or {}
    )
    jupyter_display_selected_range = (
        jupyter_display_observation.get("selected_range") or {}
    )
    jupyter_display_eval_request = {
        "request_id": "daemon-jupyter-display-eval-1",
        "operation": "jupyter.eval_region_via_repl",
        "params": {
            "buffer_name": jupyter_display_buffer,
            "start": jupyter_display_beg,
            "end": jupyter_display_end,
            "timeout_seconds": 30.0,
            "preconditions": jupyter_display_observation,
        },
    }
    jupyter_display_eval_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_eval_request,
        timeout=2.0,
    )
    jupyter_display_status_request = {
        "request_id": "daemon-jupyter-display-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-jupyter-display-eval-1"},
    }
    jupyter_display_status_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_status_request,
        timeout=2.0,
    )
    jupyter_display_status_result = (
        jupyter_display_status_response.get("result") or {}
    )
    jupyter_display_observed_status = (
        jupyter_display_status_result.get("status") or {}
    )
    jupyter_display_await_request = {
        "request_id": "daemon-jupyter-display-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-jupyter-display-eval-1",
            "timeout_seconds": 40.0,
        },
    }
    jupyter_display_await_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_await_request,
        timeout=45.0,
    )
    jupyter_display_await_result = (
        jupyter_display_await_response.get("result") or {}
    )
    jupyter_display_await_status = (
        jupyter_display_await_result.get("status") or {}
    )
    jupyter_display_semantic_await_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-jupyter-display-semantic-await-1",
        "operation": "jupyter.await_request",
        "capability": "passive_observation",
        "params": {
            "request_id": "daemon-jupyter-display-eval-1",
            "timeout_seconds": 40.0,
        },
    }
    jupyter_display_semantic_await_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_semantic_await_request,
        timeout=45.0,
    )
    jupyter_display_semantic_await_result = (
        jupyter_display_semantic_await_response.get("result") or {}
    )
    jupyter_display_semantic_await_status = (
        jupyter_display_semantic_await_result.get("status") or {}
    )
    if jupyter_display_await_status.get("state") in TERMINAL_REQUEST_STATES:
        jupyter_display_final_status = jupyter_display_await_status
        jupyter_display_final_status_source = "request.await"
    elif (
        jupyter_display_semantic_await_status.get("state")
        in TERMINAL_REQUEST_STATES
    ):
        jupyter_display_final_status = jupyter_display_semantic_await_status
        jupyter_display_final_status_source = "jupyter.await_request"
    else:
        jupyter_display_final_status = (
            jupyter_display_observed_status
            or jupyter_display_eval_response
        )
        jupyter_display_final_status_source = "request.status"
    jupyter_display_eval_result = (
        jupyter_display_final_status.get("result") or {}
    )
    jupyter_display_eval_evidence = (
        jupyter_display_final_status.get("evidence") or {}
    )
    jupyter_display_eval_effects = (
        jupyter_display_final_status.get("effects") or {}
    )
    jupyter_display_kernel_execution_effect = (
        jupyter_display_eval_effects.get("kernel_execution")
        if isinstance(jupyter_display_eval_effects, dict)
        else {}
    )
    jupyter_display_stream_payload = stream_json_payload(
        jupyter_display_eval_result.get("stream_texts") or [],
        "SOMA_SELECTED_JSON:",
    )
    jupyter_display_events = (
        jupyter_display_eval_result.get("display_events") or []
    )
    jupyter_display_update_events = (
        jupyter_display_eval_result.get("display_update_events") or []
    )
    jupyter_display_event = (
        jupyter_display_events[0]
        if len(jupyter_display_events) == 1 and isinstance(jupyter_display_events[0], dict)
        else {}
    )
    jupyter_display_update_event = (
        jupyter_display_update_events[0]
        if len(jupyter_display_update_events) == 1
        and isinstance(jupyter_display_update_events[0], dict)
        else {}
    )
    jupyter_display_payload = display_payload(jupyter_display_event)
    jupyter_display_update_payload = display_payload(jupyter_display_update_event)
    jupyter_display_repl_buffer = str(jupyter_display_fixture.get("repl_buffer") or "")
    jupyter_display_repl_expected_text = str(
        jupyter_display_update_event.get("text_plain") or ""
    )
    jupyter_display_output_buffer = jupyter_display_repl_buffer
    jupyter_display_output_wait: dict[str, Any] = {}
    jupyter_display_output_observation: dict[str, Any] = {}
    jupyter_display_output_switch: dict[str, Any] = {}
    jupyter_display_output_goto_request: dict[str, Any] = {}
    jupyter_display_output_goto_response: dict[str, Any] = {}
    jupyter_display_output_terminal_capture: dict[str, Any] = {}
    jupyter_display_output_matching_ranges: list[dict[str, Any]] = []
    jupyter_display_output_matching_range_texts: list[str] = []
    jupyter_display_repl_switch: dict[str, Any] = {}
    jupyter_display_repl_wait: dict[str, Any] = {}
    jupyter_display_repl_observe_request: dict[str, Any] = {}
    jupyter_display_repl_observe_response: dict[str, Any] = {}
    jupyter_display_repl_observation: dict[str, Any] = {}
    jupyter_display_repl_read_response: dict[str, Any] = {}
    jupyter_display_repl_text = ""
    jupyter_display_repl_terminal_capture: dict[str, Any] = {}
    jupyter_display_output_observe_response: dict[str, Any] = {}
    jupyter_display_output_observe_evidence: dict[str, Any] = {}
    jupyter_display_output_observe_effects: dict[str, Any] = {}
    if jupyter_display_repl_buffer:
        jupyter_display_repl_wait = wait_buffer_text_via_bridge(
            args.socket,
            jupyter_display_repl_buffer,
            "daemon-jupyter-display-repl",
            "",
            timeout_seconds=0.2,
        )
        jupyter_display_repl_observe_request = (
            jupyter_display_repl_wait.get("observe_request") or {}
        )
        jupyter_display_repl_observe_response = (
            jupyter_display_repl_wait.get("observe_response") or {}
        )
        jupyter_display_repl_observation = (
            jupyter_display_repl_wait.get("observation") or {}
        )
        jupyter_display_repl_read_response = (
            jupyter_display_repl_wait.get("read_response") or {}
        )
        jupyter_display_repl_text = str(jupyter_display_repl_wait.get("text") or "")
        jupyter_display_repl_switch = switch_terminal_frame_to_buffer(
            args.socket,
            args.terminal_tmux_socket,
            jupyter_display_target,
            jupyter_display_repl_buffer,
            "daemon-jupyter-display-repl",
        )
        time.sleep(0.2)
        jupyter_display_repl_terminal_capture = run_command(
            [
                "tmux",
                "-S",
                args.terminal_tmux_socket,
                "capture-pane",
                "-S",
                "-200",
                "-t",
                jupyter_display_target,
                "-p",
            ]
        )
    if jupyter_display_id and jupyter_display_repl_expected_text:
        jupyter_display_output_wait = wait_jupyter_display_observation(
            args.socket,
            jupyter_display_output_buffer,
            "daemon-jupyter-display-output",
            jupyter_display_id,
            jupyter_display_repl_expected_text,
            timeout_seconds=5.0,
        )
        jupyter_display_output_observation = (
            jupyter_display_output_wait.get("observation") or {}
        )
        jupyter_display_output_observe_response = (
            jupyter_display_output_wait.get("response") or {}
        )
        jupyter_display_output_observe_evidence = (
            jupyter_display_output_observe_response.get("evidence") or {}
        )
        jupyter_display_output_observe_effects = (
            jupyter_display_output_observe_response.get("effects") or {}
        )
        jupyter_display_output_switch = switch_terminal_frame_to_buffer(
            args.socket,
            args.terminal_tmux_socket,
            jupyter_display_target,
            jupyter_display_output_buffer,
            "daemon-jupyter-display-output",
        )
        jupyter_display_output_matching_ranges = [
            item
            for item in (jupyter_display_output_observation.get("display_ranges") or [])
            if isinstance(item, dict) and item.get("display_id_matches") is True
        ]
        jupyter_display_output_matching_range_texts = [
            str(item.get("text_excerpt") or "")
            for item in jupyter_display_output_matching_ranges
        ]
        display_output_start = (
            jupyter_display_output_matching_ranges[0].get("start")
            if jupyter_display_output_matching_ranges
            else None
        )
        if isinstance(display_output_start, int):
            jupyter_display_output_goto_request = {
                "request_id": "daemon-jupyter-display-output-goto-1",
                "operation": "emacs.goto_char",
                "params": {
                    "buffer_name": jupyter_display_output_buffer,
                    "point": display_output_start,
                    "preconditions": {
                        "buffer_name": jupyter_display_output_observation.get("buffer_name"),
                        "selected_window_buffer": jupyter_display_output_buffer,
                        "chars_tick": jupyter_display_output_observation.get("chars_tick"),
                        "minibuffer_active": False,
                        "narrowed": jupyter_display_output_observation.get("narrowed", False),
                    },
                },
            }
            jupyter_display_output_goto_response = request_or_outcome_unknown(
                args.socket,
                jupyter_display_output_goto_request,
                timeout=2.0,
            )
        time.sleep(0.2)
        jupyter_display_output_terminal_capture = run_command(
            [
                "tmux",
                "-S",
                args.terminal_tmux_socket,
                "capture-pane",
                "-S",
                "-200",
                "-t",
                jupyter_display_target,
                "-p",
            ]
        )
    jupyter_display_query_request = {
        "request_id": "daemon-jupyter-display-query-1",
        "operation": "jupyter.query_state",
        "params": {
            "buffer_name": jupyter_display_buffer,
            "timeout_seconds": 30.0,
        },
    }
    jupyter_display_query_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_query_request,
        timeout=35.0,
    )
    jupyter_display_query_result = (
        jupyter_display_query_response.get("result") or {}
    )
    jupyter_display_state_payload = (
        jupyter_display_query_result.get("payload") or {}
    )
    jupyter_display_finish_request = {
        "request_id": "daemon-jupyter-display-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-jupyter-display-eval-1",
                    "state": "succeeded",
                },
                {
                    "request_id": "daemon-jupyter-display-query-1",
                    "state": "succeeded",
                },
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": jupyter_display_buffer,
                    "start": jupyter_display_beg,
                    "end": jupyter_display_end,
                    "preconditions": {
                        "buffer_name": jupyter_display_observation.get("buffer_name"),
                        "text_owner_name": jupyter_display_observation.get("text_owner_name"),
                        "chars_tick": jupyter_display_observation.get("chars_tick"),
                        "expected_text": jupyter_display_selected_code,
                    },
                }
            ],
        },
    }
    jupyter_display_finish_response = request_or_outcome_unknown(
        args.socket,
        jupyter_display_finish_request,
        timeout=5.0,
    )
    records.append(
        scenario_record(
            22,
            "daemon-jupyter-display-update",
            "The live daemon bridge captures display_data and update_display_data from an existing rootfs-owned Jupyter kernel",
            True,
            args.out_dir,
            fixture=jupyter_display_fixture,
            terminal_switch={
                "tmux_target": jupyter_display_target,
                "cancel": jupyter_display_switch["cancel"],
                "cancel_attempts": jupyter_display_switch["cancel_attempts"],
                "cancel_interaction_response": jupyter_display_switch["cancel_interaction_response"],
                "prefix": jupyter_display_switch["prefix"],
                "minibuffer_attempts": jupyter_display_switch["minibuffer_attempts"],
                "buffer_keys": jupyter_display_switch["buffer_keys"],
                "enter": jupyter_display_switch["enter"],
                "observation_attempts": jupyter_display_switch["observation_attempts"],
                "response": jupyter_display_switch["response"],
            },
            observe_request=jupyter_display_observe_request,
            observe_response=jupyter_display_observe_response,
            eval_request=jupyter_display_eval_request,
            eval_response=jupyter_display_eval_response,
            status_request=jupyter_display_status_request,
            status_response=jupyter_display_status_response,
            await_request=jupyter_display_await_request,
            await_response=jupyter_display_await_response,
            semantic_await_request=jupyter_display_semantic_await_request,
            semantic_await_response=jupyter_display_semantic_await_response,
            final_status=jupyter_display_final_status,
            final_status_source=jupyter_display_final_status_source,
            stream_payload=jupyter_display_stream_payload,
            repl_buffer=jupyter_display_repl_buffer,
            repl_expected_text=jupyter_display_repl_expected_text,
            repl_wait=jupyter_display_repl_wait,
            repl_terminal_switch=jupyter_display_repl_switch,
            repl_observe_request=jupyter_display_repl_observe_request,
            repl_observe_response=jupyter_display_repl_observe_response,
            repl_observation=jupyter_display_repl_observation,
            repl_read_response=jupyter_display_repl_read_response,
            repl_text_sample=jupyter_display_repl_text[-2400:],
            repl_terminal_capture=jupyter_display_repl_terminal_capture,
            display_output_buffer=jupyter_display_output_buffer,
            display_output_wait=jupyter_display_output_wait,
            display_output_observation=jupyter_display_output_observation,
            display_output_terminal_switch=jupyter_display_output_switch,
            display_output_goto_request=jupyter_display_output_goto_request,
            display_output_goto_response=jupyter_display_output_goto_response,
            display_output_terminal_capture=jupyter_display_output_terminal_capture,
            query_request=jupyter_display_query_request,
            query_response=jupyter_display_query_response,
            finish_request=jupyter_display_finish_request,
            finish_response=jupyter_display_finish_response,
            checks=[
                check("Jupyter display fixture is present", bool(jupyter_display_buffer and isinstance(jupyter_display_beg, int) and isinstance(jupyter_display_end, int) and jupyter_display_id), jupyter_display_fixture),
                check("Jupyter display terminal switch cancel succeeded", jupyter_display_switch["cancel"]["returncode"] == 0, jupyter_display_switch["cancel"]),
                check("Jupyter display terminal switch prefix succeeded", jupyter_display_switch["prefix"]["returncode"] == 0, jupyter_display_switch["prefix"]),
                check("Jupyter display terminal switch buffer text sent", jupyter_display_switch["buffer_keys"]["returncode"] == 0, jupyter_display_switch["buffer_keys"]),
                check("Jupyter display terminal switch enter succeeded", jupyter_display_switch["enter"]["returncode"] == 0, jupyter_display_switch["enter"]),
                check("client terminal frame displays Jupyter display source buffer", len(jupyter_display_switch["terminal_frames"]) >= 1, jupyter_display_switch["result"].get("frames")),
                check("Jupyter display helper executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_display_fixture.get("helper_executable") or "")), jupyter_display_fixture.get("helper_executable")),
                check("Jupyter display kernel executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_display_fixture.get("kernel_exe") or "")), jupyter_display_fixture.get("kernel_exe")),
                check("Jupyter display runtime directory is private project run state", str(jupyter_display_fixture.get("jupyter_runtime_dir") or "").startswith("/project/run/jupyter"), jupyter_display_fixture.get("jupyter_runtime_dir")),
                check("Jupyter display source observation succeeded", jupyter_display_observe_response.get("state") == "succeeded", jupyter_display_observe_response),
                check("jupyter.observe_source publishes typed display-source evidence without effects", isinstance(jupyter_display_observe_evidence, dict) and isinstance(jupyter_display_observe_evidence.get("source_buffer"), dict) and jupyter_display_observe_evidence["source_buffer"].get("buffer_name") == jupyter_display_buffer and isinstance(jupyter_display_observe_evidence.get("client"), dict) and jupyter_display_observe_evidence["client"].get("associated") is True and jupyter_display_observe_evidence["client"].get("session_id") == jupyter_display_fixture.get("client_session_id") and isinstance(jupyter_display_observe_evidence.get("kernel_incarnation"), dict) and jupyter_display_observe_evidence["kernel_incarnation"].get("kernel_token") == jupyter_display_fixture.get("kernel_token") and isinstance(jupyter_display_observe_evidence.get("selected_region"), dict) and jupyter_display_observe_evidence["selected_region"].get("text_hash") == jupyter_display_fixture.get("selected_sha256") and not jupyter_display_observe_effects, {"evidence": jupyter_display_observe_evidence, "effects": jupyter_display_observe_effects}),
                check("Jupyter display source buffer is associated", jupyter_display_observation.get("jupyter_client_associated") is True, jupyter_display_observation),
                check("Jupyter display connection file matches existing fixture", jupyter_display_observation.get("jupyter_connection_file") == jupyter_display_fixture.get("connection_file"), {"observation": jupyter_display_observation, "fixture": jupyter_display_fixture}),
                check("Jupyter display kernel token matches existing fixture", jupyter_display_observation.get("jupyter_kernel_token") == jupyter_display_fixture.get("kernel_token"), {"observation": jupyter_display_observation, "fixture": jupyter_display_fixture}),
                check("Jupyter display client session matches existing fixture", jupyter_display_observation.get("jupyter_client_session_id") == jupyter_display_fixture.get("client_session_id"), {"observation": jupyter_display_observation, "fixture": jupyter_display_fixture}),
                check("Jupyter display selected range hash matches fixture", jupyter_display_selected_range.get("text_hash") == jupyter_display_fixture.get("selected_sha256"), {"selected_range": jupyter_display_selected_range, "fixture_sha": jupyter_display_fixture.get("selected_sha256")}),
                check("Jupyter display eval request is accepted as running", jupyter_display_eval_response.get("state") == "running", jupyter_display_eval_response),
                check("Jupyter display eval status is observable", jupyter_display_status_response.get("state") == "succeeded" and jupyter_display_observed_status.get("request_id") == "daemon-jupyter-display-eval-1", jupyter_display_status_response),
                check("Jupyter display generic await either succeeds or leaves an explicit unknown outcome for reconciliation", jupyter_display_await_response.get("state") == "succeeded" or (jupyter_display_await_response.get("state") == "outcome_unknown" and jupyter_display_await_response.get("error_code") in {"timeout", "client_timeout"}), jupyter_display_await_response),
                check("Jupyter display eval reaches terminal success", jupyter_display_final_status.get("state") == "succeeded" and jupyter_display_final_status.get("request_id") == "daemon-jupyter-display-eval-1" and jupyter_display_final_status_source in {"request.await", "jupyter.await_request"}, {"generic_await": jupyter_display_await_response, "semantic_await": jupyter_display_semantic_await_response, "final": jupyter_display_final_status, "source": jupyter_display_final_status_source}),
                check("semantic Jupyter display await reaches terminal success", jupyter_display_semantic_await_response.get("state") == "succeeded" and jupyter_display_semantic_await_result.get("route") == "jupyter-request-await" and jupyter_display_semantic_await_status.get("state") == "succeeded" and jupyter_display_semantic_await_result.get("target_operation") == "jupyter.eval_region_via_repl", jupyter_display_semantic_await_response),
                check("Jupyter display eval uses REPL route", jupyter_display_eval_result.get("route") == "jupyter-repl-execute-cell", jupyter_display_eval_result),
                check("jupyter.eval_region_via_repl publishes typed kernel/display evidence and no editor text effect", isinstance(jupyter_display_eval_evidence, dict) and jupyter_display_eval_evidence.get("request_id") == jupyter_display_eval_result.get("request_id") and isinstance(jupyter_display_eval_evidence.get("reply"), dict) and jupyter_display_eval_evidence["reply"].get("status") == "ok" and isinstance(jupyter_display_eval_evidence.get("iopub"), dict) and len(jupyter_display_eval_evidence["iopub"].get("display_events") or []) == 1 and len(jupyter_display_eval_evidence["iopub"].get("display_update_events") or []) == 1 and isinstance(jupyter_display_kernel_execution_effect, dict) and jupyter_display_kernel_execution_effect.get("route") == "jupyter-repl-execute-cell" and jupyter_display_kernel_execution_effect.get("request_id") == jupyter_display_eval_result.get("request_id") and jupyter_display_kernel_execution_effect.get("submitted_hash") == jupyter_display_eval_result.get("submitted_hash") and "changed_range" not in jupyter_display_eval_effects and "changed_ranges" not in jupyter_display_eval_effects, {"evidence": jupyter_display_eval_evidence, "effects": jupyter_display_eval_effects}),
                check("Jupyter display eval preconditions passed", all(item.get("passed") is True for item in jupyter_display_eval_result.get("precondition_checks") or []), jupyter_display_eval_result.get("precondition_checks")),
                check("Jupyter display sourced exact selected region", jupyter_display_eval_result.get("selected_code") == jupyter_display_selected_code, {"selected": jupyter_display_eval_result.get("selected_code"), "expected": jupyter_display_selected_code}),
                check("Jupyter display submitted REPL-trimmed selected region", jupyter_display_eval_result.get("submitted_code") == jupyter_display_selected_code.strip(), {"submitted": jupyter_display_eval_result.get("submitted_code"), "expected": jupyter_display_selected_code.strip()}),
                check("Jupyter display submitted code excludes training cell", jupyter_display_training_code not in str(jupyter_display_eval_result.get("submitted_code") or ""), jupyter_display_eval_result.get("submitted_code")),
                check("Jupyter display execute reply is ok", jupyter_display_eval_result.get("reply_status") == "ok", jupyter_display_eval_result),
                check("Jupyter display reply parent matches request id", jupyter_display_eval_result.get("reply_parent") == jupyter_display_eval_result.get("request_id") or jupyter_display_eval_result.get("request_id") in (jupyter_display_eval_result.get("parent_ids") or []), jupyter_display_eval_result),
                check("Jupyter display/update messages are captured separately", len(jupyter_display_events) == 1 and len(jupyter_display_update_events) == 1, {"display": jupyter_display_events, "update": jupyter_display_update_events}),
                check("Jupyter display/update message types are ordered", jupyter_display_event.get("index", -1) < jupyter_display_update_event.get("index", -1), {"display": jupyter_display_event, "update": jupyter_display_update_event}),
                check("Jupyter display parent id matches request id", jupyter_display_event.get("parent_id") == jupyter_display_eval_result.get("request_id"), jupyter_display_event),
                check("Jupyter update parent id matches request id", jupyter_display_update_event.get("parent_id") == jupyter_display_eval_result.get("request_id"), jupyter_display_update_event),
                check("Jupyter display id is stable across update", jupyter_display_event.get("display_id") == jupyter_display_id and jupyter_display_update_event.get("display_id") == jupyter_display_id, {"display": jupyter_display_event, "update": jupyter_display_update_event}),
                check("Jupyter display payload carries initial state", jupyter_display_payload.get("stage") == "initial" and jupyter_display_payload.get("kernel_token") == jupyter_display_fixture.get("kernel_token") and jupyter_display_payload.get("display_id") == jupyter_display_id, jupyter_display_payload),
                check("Jupyter update payload carries final state", jupyter_display_update_payload.get("stage") == "updated" and jupyter_display_update_payload.get("kernel_token") == jupyter_display_fixture.get("kernel_token") and jupyter_display_update_payload.get("display_id") == jupyter_display_id, jupyter_display_update_payload),
                check("Jupyter display REPL buffer fixture is present", bool(jupyter_display_repl_buffer), jupyter_display_fixture),
                check("Jupyter display REPL buffer observation succeeded", jupyter_display_repl_observe_response.get("state") == "succeeded", jupyter_display_repl_observe_response),
                check("client terminal frame displays Jupyter display REPL buffer", len(jupyter_display_repl_switch.get("terminal_frames", [])) >= 1, jupyter_display_repl_switch.get("result", {}).get("frames")),
                check("terminal pane capture shows Jupyter display REPL buffer", jupyter_display_repl_terminal_capture.get("returncode") == 0 and jupyter_display_repl_buffer in jupyter_display_repl_terminal_capture.get("stdout", ""), jupyter_display_repl_terminal_capture),
                check("Jupyter display output buffer observation succeeded", (jupyter_display_output_wait.get("response") or {}).get("state") == "succeeded", jupyter_display_output_wait.get("response")),
                check("jupyter.observe_display publishes typed display evidence without effects", isinstance(jupyter_display_output_observe_evidence, dict) and jupyter_display_output_observe_evidence.get("display_id") == jupyter_display_id and isinstance(jupyter_display_output_observe_evidence.get("display_data"), dict) and jupyter_display_output_observe_evidence["display_data"].get("display_id_found") is True and isinstance(jupyter_display_output_observe_evidence.get("repl_range"), dict) and jupyter_display_output_observe_evidence["repl_range"].get("buffer_name") == jupyter_display_output_buffer and not jupyter_display_output_observe_effects, {"evidence": jupyter_display_output_observe_evidence, "effects": jupyter_display_output_observe_effects}),
                check("Jupyter display output buffer keeps stable display-id property", jupyter_display_output_observation.get("display_id_found") is True and bool(jupyter_display_output_matching_ranges), jupyter_display_output_observation.get("display_ranges")),
                check("Jupyter display-id range renders updated text/plain payload", bool(jupyter_display_repl_expected_text) and any(text.startswith(jupyter_display_repl_expected_text) for text in jupyter_display_output_matching_range_texts), {"expected_text_plain": jupyter_display_repl_expected_text, "display_ranges": jupyter_display_output_matching_ranges}),
                check("client terminal frame displays Jupyter display output buffer", len(jupyter_display_output_switch.get("terminal_frames", [])) >= 1, jupyter_display_output_switch.get("result", {}).get("frames")),
                check("checked Jupyter display output navigation succeeded", jupyter_display_output_goto_response.get("state") == "succeeded", jupyter_display_output_goto_response),
                check("terminal pane capture shows Jupyter display output buffer", jupyter_display_output_terminal_capture.get("returncode") == 0 and jupyter_display_output_buffer in jupyter_display_output_terminal_capture.get("stdout", ""), jupyter_display_output_terminal_capture),
                check("terminal pane capture shows updated Jupyter display payload", jupyter_display_output_terminal_capture.get("returncode") == 0 and "SOMA_DISPLAY_STAGE:updated" in jupyter_display_output_terminal_capture.get("stdout", ""), jupyter_display_output_terminal_capture),
                check("Jupyter display stream payload reports intended kernel", jupyter_display_stream_payload.get("kernel_token") == jupyter_display_fixture.get("kernel_token"), jupyter_display_stream_payload),
                check("Jupyter display stream payload reports selected side effect once", jupyter_display_stream_payload.get("counter") == 1, jupyter_display_stream_payload),
                check("Jupyter display stream payload reports training cell untouched", jupyter_display_stream_payload.get("training_executions") == 0, jupyter_display_stream_payload),
                check("Jupyter display state query succeeded", jupyter_display_query_response.get("state") == "succeeded", jupyter_display_query_response),
                check("Jupyter display state query sees same kernel token", jupyter_display_state_payload.get("kernel_token") == jupyter_display_fixture.get("kernel_token"), jupyter_display_state_payload),
                check("Jupyter display state query sees stable run id", bool(os.environ.get("SOMA_INSULA_RUN_ID", "")) and jupyter_display_state_payload.get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", ""), jupyter_display_state_payload),
                check("Jupyter display state query sees selected side effect exactly once", jupyter_display_state_payload.get("counter") == 1, jupyter_display_state_payload),
                check("Jupyter display state query sees training cell unexecuted", jupyter_display_state_payload.get("training_executions") == 0, jupyter_display_state_payload),
                check("Jupyter display state query executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_display_state_payload.get("executable") or "")), jupyter_display_state_payload.get("executable")),
                check("Jupyter display state query cwd is repo mount", jupyter_display_state_payload.get("cwd") == "/workspace/soma", jupyter_display_state_payload),
                check("Jupyter display state query plan is repo-mounted", str(jupyter_display_state_payload.get("plan") or "").startswith("/workspace/soma/"), jupyter_display_state_payload),
                check("Jupyter display episode finish succeeds with eval/query evidence", jupyter_display_finish_response.get("state") == "succeeded", jupyter_display_finish_response),
            ],
        )
    )

    completion_fixture = (ready or {}).get("completion_fixture") or {}
    completion_buffer = str(completion_fixture.get("buffer_name") or "")
    completion_point = completion_fixture.get("point")
    completion_proposal = str(completion_fixture.get("proposal") or "")
    completion_human_text = str(completion_fixture.get("human_text") or "")
    completion_initial_text = str(completion_fixture.get("initial_text") or "")
    completion_after_human_text = str(completion_fixture.get("after_human_text") or "")
    completion_read_end = completion_fixture.get("post_human_read_end")
    if not isinstance(completion_read_end, int):
        completion_read_end = max(len(completion_initial_text), len(completion_after_human_text)) + 1
    if not isinstance(completion_point, int):
        completion_point = 1
    completion_target = f"{args.terminal_tmux_session}:0.0"
    if completion_buffer:
        completion_switch = switch_terminal_frame_to_buffer(
            args.socket,
            args.terminal_tmux_socket,
            completion_target,
            completion_buffer,
            "daemon-stale-completion",
        )
    else:
        completion_switch = {
            "cancel": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", completion_target, "C-g"]),
            "cancel_attempts": [],
            "cancel_interaction_response": {},
            "prefix": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", completion_target, "C-x", "b"]),
            "minibuffer_attempts": [],
            "buffer_keys": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", completion_target, ""]),
            "enter": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", completion_target, "Enter"]),
            "observation_attempts": [],
            "response": {},
            "result": {},
            "terminal_frames": [],
        }
    completion_initial_request = {
        "request_id": "daemon-stale-completion-observe-initial-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": completion_buffer},
    }
    completion_initial_response = request_or_outcome_unknown(
        args.socket,
        completion_initial_request,
        timeout=2.0,
    )
    completion_initial_observation = response_result(completion_initial_response)
    completion_goto_request = {
        "request_id": "daemon-stale-completion-goto-1",
        "operation": "emacs.goto_char",
        "params": {
            "buffer_name": completion_buffer,
            "point": completion_point,
            "preconditions": completion_initial_observation,
        },
    }
    completion_goto_response = request_or_outcome_unknown(
        args.socket,
        completion_goto_request,
        timeout=2.0,
    )
    completion_observe_request = {
        "request_id": "daemon-stale-completion-observe-context-1",
        "operation": "completion.observe",
        "params": {
            "buffer_name": completion_buffer,
            "max_candidates": 8,
            "prefix_chars": 80,
            "suffix_chars": 80,
        },
    }
    completion_observe_response = request_or_outcome_unknown(
        args.socket,
        completion_observe_request,
        timeout=2.0,
    )
    completion_observation = response_result(completion_observe_response)
    completion_replacement_range = completion_observation.get("replacement_range") or {}
    completion_accept_preconditions = completion_observation.get("acceptance_preconditions") or {}
    if completion_human_text:
        completion_human_input = run_command(
            [
                "tmux",
                "-S",
                args.terminal_tmux_socket,
                "send-keys",
                "-l",
                "-t",
                completion_target,
                completion_human_text,
            ]
        )
        time.sleep(0.2)
    else:
        completion_human_input = noop_command(
            ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", completion_target, ""]
        )
    completion_after_human_request = {
        "request_id": "daemon-stale-completion-observe-after-human-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": completion_buffer},
    }
    completion_after_human_response = request_or_outcome_unknown(
        args.socket,
        completion_after_human_request,
        timeout=2.0,
    )
    completion_after_human_observation = response_result(completion_after_human_response)
    completion_after_human_read_response, completion_after_human_bridge_text = buffer_text_via_bridge(
        args.socket,
        completion_buffer,
        "daemon-stale-completion-read-after-human-1",
        completion_read_end,
    )
    completion_accept_request = {
        "request_id": "daemon-stale-completion-accept-1",
        "operation": "completion.accept",
        "params": {
            "buffer_name": completion_buffer,
            "start": completion_replacement_range.get("start"),
            "end": completion_replacement_range.get("end"),
            "text": completion_proposal,
            "preconditions": completion_accept_preconditions,
        },
    }
    completion_accept_response = request_or_outcome_unknown(
        args.socket,
        completion_accept_request,
        timeout=2.0,
    )
    completion_final_request = {
        "request_id": "daemon-stale-completion-observe-final-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": completion_buffer},
    }
    completion_final_response = request_or_outcome_unknown(
        args.socket,
        completion_final_request,
        timeout=2.0,
    )
    completion_final_observation = response_result(completion_final_response)
    completion_final_read_response, completion_final_text = buffer_text_via_bridge(
        args.socket,
        completion_buffer,
        "daemon-stale-completion-read-final-1",
        completion_read_end,
    )
    records.append(
        scenario_record(
            23,
            "daemon-stale-completion-rejected",
            "The live daemon bridge rejects stale completion.accept after intervening terminal editing",
            True,
            args.out_dir,
            fixture=completion_fixture,
            terminal_switch={
                "tmux_target": completion_target,
                "cancel": completion_switch["cancel"],
                "cancel_attempts": completion_switch["cancel_attempts"],
                "cancel_interaction_response": completion_switch["cancel_interaction_response"],
                "prefix": completion_switch["prefix"],
                "minibuffer_attempts": completion_switch["minibuffer_attempts"],
                "buffer_keys": completion_switch["buffer_keys"],
                "enter": completion_switch["enter"],
                "observation_attempts": completion_switch["observation_attempts"],
                "response": completion_switch["response"],
            },
            initial_observe_request=completion_initial_request,
            initial_observe_response=completion_initial_response,
            goto_request=completion_goto_request,
            goto_response=completion_goto_response,
            completion_observe_request=completion_observe_request,
            completion_observe_response=completion_observe_response,
            terminal_human_input=completion_human_input,
            after_human_observe_request=completion_after_human_request,
            after_human_observe_response=completion_after_human_response,
            after_human_read_response=completion_after_human_read_response,
            after_human_text=completion_after_human_bridge_text,
            accept_request=completion_accept_request,
            accept_response=completion_accept_response,
            final_observe_request=completion_final_request,
            final_observe_response=completion_final_response,
            final_read_response=completion_final_read_response,
            final_text=completion_final_text,
            checks=[
                check("completion fixture is present", bool(completion_buffer and completion_fixture), completion_fixture),
                check("completion terminal switch cancel succeeded", completion_switch["cancel"]["returncode"] == 0, completion_switch["cancel"]),
                check("completion terminal switch prefix succeeded", completion_switch["prefix"]["returncode"] == 0, completion_switch["prefix"]),
                check("completion terminal switch buffer text sent", completion_switch["buffer_keys"]["returncode"] == 0, completion_switch["buffer_keys"]),
                check("completion terminal switch enter succeeded", completion_switch["enter"]["returncode"] == 0, completion_switch["enter"]),
                check("client terminal frame displays completion buffer", len(completion_switch["terminal_frames"]) >= 1, completion_switch["result"].get("frames")),
                check("initial completion observation succeeded", completion_initial_response.get("state") == "succeeded", completion_initial_response),
                check("initial completion point is insertion point", completion_initial_observation.get("point") == completion_point, completion_initial_observation),
                check("checked completion point navigation succeeded", completion_goto_response.get("state") == "succeeded", completion_goto_response),
                check("completion.observe captures provider context", completion_observe_response.get("state") == "succeeded" and completion_observation.get("operation") == "completion.observe" and completion_replacement_range.get("start") == completion_point and completion_replacement_range.get("end") == completion_point and completion_proposal in (completion_observation.get("candidates") or []), completion_observation),
                check("completion.observe supplies accept preconditions", completion_accept_preconditions.get("chars_tick") == completion_observation.get("chars_tick") and completion_accept_preconditions.get("expected_prefix") == completion_fixture.get("prefix") and completion_accept_preconditions.get("expected_suffix") == completion_fixture.get("suffix"), completion_accept_preconditions),
                check("terminal human edit was sent", completion_human_input["returncode"] == 0, completion_human_input),
                check("after-human completion observation succeeded", completion_after_human_response.get("state") == "succeeded", completion_after_human_response),
                check("human edit changed completion revision", isinstance(completion_initial_observation.get("chars_tick"), int) and isinstance(completion_after_human_observation.get("chars_tick"), int) and completion_initial_observation.get("chars_tick") < completion_after_human_observation.get("chars_tick"), {"before": completion_initial_observation, "after": completion_after_human_observation}),
                check("human edit changed completion context", completion_after_human_bridge_text == completion_after_human_text, completion_after_human_bridge_text),
                check("stale completion accept is rejected as conflict", completion_accept_response.get("state") == "failed" and completion_accept_response.get("error_code") == "conflict", completion_accept_response),
                check("stale completion rejection reports context failure", precondition_check_passed(completion_accept_response, "character revision matches precondition", False) or precondition_check_passed(completion_accept_response, "completion suffix matches precondition", False), completion_accept_response),
                check("stale completion did not insert proposed text", completion_final_text == completion_after_human_text and completion_proposal not in completion_final_text, completion_final_text),
                check("failed stale completion did not advance revision further", completion_final_observation.get("chars_tick") == completion_after_human_observation.get("chars_tick"), {"after_human": completion_after_human_observation, "final": completion_final_observation}),
            ],
        )
    )

    lsp_fixture = (ready or {}).get("lsp_fixture") or {}
    lsp_buffer = str(lsp_fixture.get("buffer_name") or "")
    lsp_main_buffer = str(lsp_fixture.get("main_buffer_name") or "main.py")
    lsp_test_buffer = str(lsp_fixture.get("test_buffer_name") or "test_core.py")
    lsp_root = str(lsp_fixture.get("root") or "")
    lsp_symbol = str(lsp_fixture.get("symbol") or "")
    lsp_new_name = str(lsp_fixture.get("new_name") or "")
    lsp_definition_file = str(lsp_fixture.get("definition_file") or "")
    lsp_main_file = str(lsp_fixture.get("main_file") or "")
    lsp_test_file = str(lsp_fixture.get("test_file") or "")
    lsp_test_dir = str(lsp_fixture.get("test_dir") or "")
    lsp_diagnostics_fixture = (ready or {}).get("lsp_diagnostics_fixture") or {}
    lsp_diag_buffer = str(lsp_diagnostics_fixture.get("buffer_name") or "")
    lsp_diag_root = str(lsp_diagnostics_fixture.get("root") or "")
    lsp_diag_definition_file = str(lsp_diagnostics_fixture.get("definition_file") or "")
    lsp_diag_test_dir = str(lsp_diagnostics_fixture.get("test_dir") or "")
    lsp_diag_good_return = str(lsp_diagnostics_fixture.get("good_return") or "return x + 22")
    lsp_diag_broken_return = str(lsp_diagnostics_fixture.get("broken_return") or "return x +")
    lsp_diag_expected_diagnostic_needle = str(
        lsp_diagnostics_fixture.get("expected_diagnostic_needle") or "Expected expression"
    )
    lsp_diag_pyright_path = str(lsp_diagnostics_fixture.get("pyright_path") or "")
    lsp_diag_node_path = str(lsp_diagnostics_fixture.get("node_path") or "")
    lsp_diag_python_path = str(lsp_diagnostics_fixture.get("python_path") or "")
    lsp_target = f"{args.terminal_tmux_session}:0.0"
    if lsp_buffer:
        lsp_switch = switch_terminal_frame_to_buffer(
            args.socket,
            args.terminal_tmux_socket,
            lsp_target,
            lsp_buffer,
            "daemon-lsp-rename",
        )
    else:
        lsp_switch = {
            "cancel": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", lsp_target, "C-g"]),
            "cancel_attempts": [],
            "cancel_interaction_response": {},
            "prefix": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", lsp_target, "C-x", "b"]),
            "minibuffer_attempts": [],
            "buffer_keys": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", lsp_target, ""]),
            "enter": noop_command(["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", lsp_target, "Enter"]),
            "observation_attempts": [],
            "response": {},
            "result": {},
            "terminal_frames": [],
        }
    lsp_observe_request = {
        "request_id": "daemon-lsp-rename-observe-1",
        "operation": "lsp.observe_source",
        "params": {
            "buffer_name": lsp_buffer,
            "max_text": 1600,
        },
    }
    lsp_observe_response = request_or_outcome_unknown(
        args.socket,
        lsp_observe_request,
        timeout=10.0,
    )
    lsp_observation = response_result(lsp_observe_response)
    lsp_observe_evidence = lsp_observe_response.get("evidence") or {}
    lsp_observe_observations = lsp_observe_response.get("observations") or {}
    lsp_observe_effects = lsp_observe_response.get("effects") or {}
    lsp_rename_request = {
        "request_id": "daemon-lsp-rename-1",
        "operation": "lsp.rename_symbol",
        "params": {
            "buffer_name": lsp_buffer,
            "new_name": lsp_new_name,
            "timeout_seconds": 30.0,
            "preconditions": lsp_observation,
        },
    }
    lsp_rename_response = request_or_outcome_unknown(
        args.socket,
        lsp_rename_request,
        timeout=2.0,
    )
    lsp_status_request = {
        "request_id": "daemon-lsp-rename-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-lsp-rename-1"},
    }
    lsp_status_response = request_or_outcome_unknown(
        args.socket,
        lsp_status_request,
        timeout=2.0,
    )
    lsp_status_result = lsp_status_response.get("result") or {}
    lsp_observed_status = lsp_status_result.get("status") or {}
    lsp_await_request = {
        "request_id": "daemon-lsp-rename-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-lsp-rename-1",
            "timeout_seconds": 40.0,
        },
    }
    lsp_await_response = request_or_outcome_unknown(
        args.socket,
        lsp_await_request,
        timeout=45.0,
    )
    lsp_await_result = lsp_await_response.get("result") or {}
    lsp_final_status = (
        lsp_await_result.get("status")
        or lsp_observed_status
        or lsp_rename_response
    )
    lsp_rename_result = lsp_final_status.get("result") or {}
    lsp_rename_evidence = lsp_final_status.get("evidence") or {}
    lsp_rename_observations = lsp_final_status.get("observations") or {}
    lsp_rename_effects = lsp_final_status.get("effects") or {}
    lsp_workspace_edit_effect = (
        lsp_rename_effects.get("workspace_edit")
        if isinstance(lsp_rename_effects, dict)
        else {}
    )
    lsp_build_test = run_command(
        [
            "python3",
            "-c",
            (
                "import pathlib, sys, unittest; "
                f"sys.path.insert(0, {json.dumps(lsp_root)}); "
                f"suite = unittest.defaultTestLoader.discover({json.dumps(lsp_test_dir)}); "
                "result = unittest.TextTestRunner(verbosity=2).run(suite); "
                "raise SystemExit(0 if result.wasSuccessful() else 1)"
            ),
        ]
    )
    lsp_definition_text = read_text(lsp_definition_file)
    lsp_main_text = read_text(lsp_main_file)
    lsp_test_text = read_text(lsp_test_file)
    records.append(
        scenario_record(
            24,
            "daemon-lsp-python-rename-symbol",
            "The live daemon bridge renames a Python symbol through lsp-mode and verifies the workspace edit",
            True,
            args.out_dir,
            fixture=lsp_fixture,
            terminal_switch={
                "tmux_target": lsp_target,
                "cancel": lsp_switch["cancel"],
                "cancel_attempts": lsp_switch["cancel_attempts"],
                "cancel_interaction_response": lsp_switch["cancel_interaction_response"],
                "prefix": lsp_switch["prefix"],
                "minibuffer_attempts": lsp_switch["minibuffer_attempts"],
                "buffer_keys": lsp_switch["buffer_keys"],
                "enter": lsp_switch["enter"],
                "observation_attempts": lsp_switch["observation_attempts"],
                "response": lsp_switch["response"],
            },
            observe_request=lsp_observe_request,
            observe_response=lsp_observe_response,
            rename_request=lsp_rename_request,
            rename_response=lsp_rename_response,
            status_request=lsp_status_request,
            status_response=lsp_status_response,
            await_request=lsp_await_request,
            await_response=lsp_await_response,
            final_status=lsp_final_status,
            build_test=lsp_build_test,
            final_files={
                "definition": lsp_definition_text,
                "main": lsp_main_text,
                "test": lsp_test_text,
            },
            checks=[
                check("LSP fixture is present", bool(lsp_buffer and lsp_root and lsp_symbol and lsp_new_name), lsp_fixture),
                check("LSP terminal switch cancel succeeded", lsp_switch["cancel"]["returncode"] == 0, lsp_switch["cancel"]),
                check("LSP terminal switch prefix succeeded", lsp_switch["prefix"]["returncode"] == 0, lsp_switch["prefix"]),
                check("LSP terminal switch buffer text sent", lsp_switch["buffer_keys"]["returncode"] == 0, lsp_switch["buffer_keys"]),
                check("LSP terminal switch enter succeeded", lsp_switch["enter"]["returncode"] == 0, lsp_switch["enter"]),
                check("client terminal frame displays LSP source buffer", len(lsp_switch["terminal_frames"]) >= 1, lsp_switch["result"].get("frames")),
                check("LSP source observation succeeded", lsp_observe_response.get("state") == "succeeded", lsp_observe_response),
                check("LSP workspace is initialized", lsp_observation.get("workspace_status") == "initialized", lsp_observation),
                check("LSP server executable is rootfs-owned", rootfs_owned_process_path(str(lsp_observation.get("server_executable") or "")), lsp_observation.get("server_executable")),
                check("LSP rename capability is advertised", ((lsp_observation.get("capabilities") or {}).get("textDocument/rename") is True), lsp_observation.get("capabilities")),
                check("LSP observed symbol matches fixture", lsp_observation.get("symbol") == lsp_symbol, lsp_observation),
                check("LSP source observation publishes typed workspace/server/document evidence", isinstance(lsp_observe_evidence, dict) and isinstance(lsp_observe_evidence.get("workspace"), dict) and lsp_observe_evidence.get("workspace", {}).get("status") == "initialized" and isinstance(lsp_observe_evidence.get("server"), dict) and rootfs_owned_process_path(str(lsp_observe_evidence.get("server", {}).get("executable") or "")) and isinstance(lsp_observe_evidence.get("document"), dict) and isinstance(lsp_observe_evidence.get("diagnostics"), dict) and isinstance(lsp_observe_evidence.get("diagnostic_cache"), dict) and isinstance(lsp_observe_evidence.get("lsp_sync"), dict) and isinstance(lsp_observe_evidence.get("rootfs_runtime"), dict), lsp_observe_evidence),
                check("LSP source observation publishes typed observations without effects", isinstance(lsp_observe_observations, dict) and isinstance(lsp_observe_observations.get("source_buffer"), dict) and lsp_observe_observations.get("source_buffer", {}).get("buffer_name") == lsp_buffer and isinstance(lsp_observe_observations.get("document"), dict) and isinstance(lsp_observe_observations.get("symbol"), dict) and lsp_observe_observations.get("symbol", {}).get("name") == lsp_symbol and isinstance(lsp_observe_effects, dict) and not lsp_observe_effects, {"observations": lsp_observe_observations, "effects": lsp_observe_effects}),
                check("LSP rename request is accepted as running", lsp_rename_response.get("state") == "running", lsp_rename_response),
                check("LSP rename status is observable", lsp_status_response.get("state") == "succeeded" and lsp_observed_status.get("request_id") == "daemon-lsp-rename-1", lsp_status_response),
                check("LSP rename reaches terminal success", lsp_await_response.get("state") == "succeeded" and lsp_final_status.get("state") == "succeeded", {"await": lsp_await_response, "final": lsp_final_status}),
                check("LSP rename route is lsp-rename", lsp_rename_result.get("route") == "lsp-rename", lsp_rename_result),
                check("LSP rename sent textDocument/rename", "textDocument/rename" in (lsp_rename_result.get("request_methods") or []), lsp_rename_result),
                check("LSP workspace edit was applied by lsp-mode", lsp_rename_result.get("workspace_edit_applied") is True, lsp_rename_result),
                check("LSP rename publishes typed request and workspace-edit evidence", isinstance(lsp_rename_evidence, dict) and isinstance(lsp_rename_evidence.get("lsp_request"), dict) and "textDocument/rename" in (lsp_rename_evidence.get("lsp_request", {}).get("methods") or []) and isinstance(lsp_rename_evidence.get("workspace_edit"), dict) and lsp_rename_evidence.get("workspace_edit", {}).get("applied") is True and isinstance(lsp_rename_evidence.get("saved_buffers"), list) and isinstance(lsp_rename_evidence.get("rootfs_runtime"), dict), lsp_rename_evidence),
                check("LSP rename publishes typed multi-buffer workspace-edit effect", isinstance(lsp_workspace_edit_effect, dict) and lsp_workspace_edit_effect.get("route") == "lsp-rename" and lsp_workspace_edit_effect.get("new_name") == lsp_new_name and lsp_workspace_edit_effect.get("applied") is True and "textDocument/rename" in (lsp_workspace_edit_effect.get("request_methods") or []) and isinstance(lsp_workspace_edit_effect.get("saved_buffers"), list) and len(lsp_workspace_edit_effect.get("saved_buffers") or []) >= 1 and isinstance(lsp_workspace_edit_effect.get("affected_files"), list) and len(lsp_workspace_edit_effect.get("affected_files") or []) >= 1 and "changed_range" not in lsp_rename_effects, lsp_rename_effects),
                check("LSP rename publishes before and after LSP observations", isinstance(lsp_rename_observations, dict) and isinstance(lsp_rename_observations.get("before"), dict) and isinstance(lsp_rename_observations.get("after"), dict) and lsp_rename_observations.get("before", {}).get("symbol") == lsp_symbol and lsp_rename_observations.get("after", {}).get("symbol") == lsp_new_name, lsp_rename_observations),
                check("LSP rename updated definition file", lsp_new_name in lsp_definition_text and lsp_symbol not in lsp_definition_text, lsp_definition_text),
                check("LSP rename updated main file", lsp_new_name in lsp_main_text and lsp_symbol not in lsp_main_text, lsp_main_text),
                check("LSP rename updated test file", lsp_new_name in lsp_test_text and lsp_symbol not in lsp_test_text, lsp_test_text),
                check("rootfs Python tests pass after LSP rename", lsp_build_test.get("returncode") == 0, lsp_build_test),
            ],
        )
    )

    lsp_stale_before_request = {
        "request_id": "daemon-lsp-stale-document-observe-before-1",
        "operation": "lsp.observe_source",
        "params": {
            "buffer_name": lsp_buffer,
            "max_text": 1600,
        },
    }
    lsp_stale_before_response = request_or_outcome_unknown(
        args.socket,
        lsp_stale_before_request,
        timeout=10.0,
    )
    lsp_stale_before = response_result(lsp_stale_before_response)
    lsp_stale_buffer_text = full_buffer_text_v0(
        args.socket,
        lsp_buffer,
        "daemon-lsp-stale-document-buffer-before",
        timeout=4.0,
    )
    lsp_stale_buffer_observation = (
        lsp_stale_buffer_text.get("observation") or {}
    )
    lsp_stale_edit_point = lsp_stale_buffer_observation.get("point_max")
    lsp_stale_edit = (
        replace_range_via_bridge(
            args.socket,
            lsp_buffer,
            lsp_stale_edit_point,
            lsp_stale_edit_point,
            "\n# stale-lsp-document-version-guard\n",
            "",
            "daemon-lsp-stale-document-advance",
            timeout=4.0,
        )
        if isinstance(lsp_stale_edit_point, int)
        else {
            "replace_response": {
                "request_id": "daemon-lsp-stale-document-advance-replace",
                "operation": "buffer.replace_range",
                "state": "failed",
                "state_sequence": ["accepted", "failed"],
                "error_code": "missing_point_max",
                "error": "could not observe point_max for LSP stale document setup",
                "client_side": True,
            }
        }
    )
    lsp_stale_restore_point_request = {
        "request_id": "daemon-lsp-stale-document-restore-point-1",
        "operation": "buffer.select",
        "params": {
            "buffer_name": lsp_buffer,
            "point": lsp_stale_before.get("point"),
        },
    }
    lsp_stale_restore_point_response = request_or_outcome_unknown(
        args.socket,
        lsp_stale_restore_point_request,
        timeout=4.0,
    )
    lsp_stale_after_attempts: list[dict[str, Any]] = []
    lsp_stale_after_request: dict[str, Any] = {}
    lsp_stale_after_response: dict[str, Any] = {}
    lsp_stale_after: dict[str, Any] = {}
    lsp_stale_old_version = lsp_stale_before.get("document_version")
    for attempt in range(1, 16):
        lsp_stale_after_request = {
            "request_id": f"daemon-lsp-stale-document-observe-after-{attempt}",
            "operation": "lsp.observe_source",
            "params": {
                "buffer_name": lsp_buffer,
                "max_text": 1600,
            },
        }
        lsp_stale_after_response = request_or_outcome_unknown(
            args.socket,
            lsp_stale_after_request,
            timeout=5.0,
        )
        lsp_stale_after = response_result(lsp_stale_after_response)
        lsp_stale_after_attempts.append(
            {
                "attempt": attempt,
                "response_state": lsp_stale_after_response.get("state"),
                "document_version": lsp_stale_after.get("document_version"),
                "chars_tick": lsp_stale_after.get("chars_tick"),
                "point": lsp_stale_after.get("point"),
            }
        )
        if (
            lsp_stale_after_response.get("state") == "succeeded"
            and lsp_stale_old_version != lsp_stale_after.get("document_version")
        ):
            break
        time.sleep(0.2)
    lsp_stale_rename_request = {
        "request_id": "daemon-lsp-stale-document-rename-1",
        "operation": "lsp.rename_symbol",
        "params": {
            "buffer_name": lsp_buffer,
            "new_name": "compute_stale_rejected",
            "timeout_seconds": 5.0,
            "preconditions": lsp_stale_before,
        },
    }
    lsp_stale_rename_response = request_or_outcome_unknown(
        args.socket,
        lsp_stale_rename_request,
        timeout=2.0,
    )
    lsp_stale_await_request = {
        "request_id": "daemon-lsp-stale-document-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-lsp-stale-document-rename-1",
            "timeout_seconds": 10.0,
        },
    }
    lsp_stale_await_response = request_or_outcome_unknown(
        args.socket,
        lsp_stale_await_request,
        timeout=12.0,
    )
    lsp_stale_await_result = lsp_stale_await_response.get("result") or {}
    lsp_stale_final_status = (
        lsp_stale_await_result.get("status")
        or lsp_stale_rename_response
    )
    lsp_stale_final_result = lsp_stale_final_status.get("result") or {}
    lsp_stale_final_effects = lsp_stale_final_status.get("effects") or {}
    lsp_stale_after_reject_request = {
        "request_id": "daemon-lsp-stale-document-after-reject-1",
        "operation": "lsp.observe_source",
        "params": {
            "buffer_name": lsp_buffer,
            "max_text": 1600,
        },
    }
    lsp_stale_after_reject_response = request_or_outcome_unknown(
        args.socket,
        lsp_stale_after_reject_request,
        timeout=5.0,
    )
    lsp_stale_after_reject = response_result(lsp_stale_after_reject_response)
    lsp_stale_definition_text = read_text(lsp_definition_file)
    records.append(
        scenario_record(
            58,
            "daemon-lsp-stale-document-version-rejected",
            "lsp.rename_symbol rejects stale LSP document-version preconditions before applying a workspace edit",
            True,
            args.out_dir,
            fixture=lsp_fixture,
            stale_before_request=lsp_stale_before_request,
            stale_before_response=lsp_stale_before_response,
            buffer_before=lsp_stale_buffer_text,
            edit_setup=lsp_stale_edit,
            restore_point_request=lsp_stale_restore_point_request,
            restore_point_response=lsp_stale_restore_point_response,
            after_edit_attempts=lsp_stale_after_attempts,
            stale_rename_request=lsp_stale_rename_request,
            stale_rename_response=lsp_stale_rename_response,
            stale_await_request=lsp_stale_await_request,
            stale_await_response=lsp_stale_await_response,
            stale_final_status=lsp_stale_final_status,
            after_reject_request=lsp_stale_after_reject_request,
            after_reject_response=lsp_stale_after_reject_response,
            final_definition_text=lsp_stale_definition_text,
            checks=[
                check("LSP stale-document fixture is present", bool(lsp_buffer and lsp_definition_file and lsp_stale_before_response.get("state") == "succeeded"), {"fixture": lsp_fixture, "before": lsp_stale_before}),
                check("checked buffer edit advances LSP document version", (lsp_stale_edit.get("replace_response") or {}).get("state") == "succeeded" and lsp_stale_after_response.get("state") == "succeeded" and lsp_stale_old_version != lsp_stale_after.get("document_version"), {"edit": lsp_stale_edit, "old_version": lsp_stale_old_version, "after_attempts": lsp_stale_after_attempts}),
                check("stale LSP rename request is accepted for daemon lifecycle", lsp_stale_rename_response.get("state") == "running", lsp_stale_rename_response),
                check("stale LSP rename reaches failed terminal status", lsp_stale_await_response.get("state") == "succeeded" and lsp_stale_final_status.get("state") == "failed", {"await": lsp_stale_await_response, "final": lsp_stale_final_status}),
                check("stale LSP rename reports document-version precondition failure", precondition_check_passed(lsp_stale_final_status, "LSP document version matches precondition", False), lsp_stale_final_status),
                check("stale LSP rename publishes no workspace-edit effect", not (lsp_stale_final_effects.get("workspace_edit") if isinstance(lsp_stale_final_effects, dict) else None) and lsp_stale_final_result.get("workspace_edit_applied") is not True, {"result": lsp_stale_final_result, "effects": lsp_stale_final_effects}),
                check("stale LSP rename does not apply rejected symbol name", "compute_stale_rejected" not in lsp_stale_definition_text and lsp_stale_after_reject.get("symbol") == lsp_stale_after.get("symbol"), {"text": lsp_stale_definition_text, "after_edit": lsp_stale_after, "after_reject": lsp_stale_after_reject}),
            ],
        )
    )

    lsp_terminal_new_name = "compute_total"
    lsp_terminal_observe_request = {
        "request_id": "daemon-lsp-terminal-rename-observe-1",
        "operation": "lsp.observe_source",
        "params": {
            "buffer_name": lsp_buffer,
            "max_text": 1600,
        },
    }
    lsp_terminal_observe_response = request_or_outcome_unknown(
        args.socket,
        lsp_terminal_observe_request,
        timeout=10.0,
    )
    lsp_terminal_observation = response_result(lsp_terminal_observe_response)
    lsp_terminal_request = {
        "request_id": "daemon-lsp-terminal-rename-command-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": lsp_buffer,
            "command": "lsp-rename",
            "preconditions": lsp_terminal_observation,
        },
    }
    lsp_terminal_response = request_or_outcome_unknown(
        args.socket,
        lsp_terminal_request,
        timeout=2.0,
    )
    lsp_terminal_interaction_attempts: list[dict[str, Any]] = []
    lsp_terminal_interaction_response: dict[str, Any] = {}
    lsp_terminal_interaction_result: dict[str, Any] = {}
    for attempt in range(1, 51):
        lsp_terminal_interaction_request = {
            "request_id": f"daemon-lsp-terminal-rename-interaction-{attempt}",
            "operation": "emacs.observe_interaction",
            "params": {},
        }
        lsp_terminal_interaction_response = request_or_outcome_unknown(
            args.socket,
            lsp_terminal_interaction_request,
            timeout=0.5,
        )
        lsp_terminal_interaction_result = lsp_terminal_interaction_response.get("result") or {}
        lsp_terminal_interaction_attempts.append(
            {
                "attempt": attempt,
                "response_state": lsp_terminal_interaction_response.get("state"),
                "minibuffer_active": lsp_terminal_interaction_result.get("minibuffer_active"),
                "minibuffer_prompt": lsp_terminal_interaction_result.get("minibuffer_prompt"),
                "minibuffer_contents": lsp_terminal_interaction_result.get("minibuffer_contents"),
                "selected_window_buffer": lsp_terminal_interaction_result.get("selected_window_buffer"),
            }
        )
        if (
            lsp_terminal_interaction_response.get("state") == "succeeded"
            and lsp_terminal_interaction_result.get("minibuffer_active") is True
        ):
            break
        time.sleep(0.1)
    lsp_terminal_status_request = {
        "request_id": "daemon-lsp-terminal-rename-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-lsp-terminal-rename-command-1"},
    }
    lsp_terminal_status_response = request_or_outcome_unknown(
        args.socket,
        lsp_terminal_status_request,
        timeout=0.5,
    )
    lsp_terminal_status_result = lsp_terminal_status_response.get("result") or {}
    lsp_terminal_observed_status = lsp_terminal_status_result.get("status") or {}
    lsp_terminal_clear = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", lsp_target, "C-a", "C-k"]
    )
    lsp_terminal_input = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", lsp_target, lsp_terminal_new_name]
    )
    lsp_terminal_submit = run_command(
        ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", lsp_target, "Enter"]
    )
    lsp_terminal_terminal_status = wait_request_terminal_status(
        args.socket,
        "daemon-lsp-terminal-rename-command-1",
        "daemon-lsp-terminal-rename-terminal-status",
        timeout_seconds=45.0,
        per_request_timeout=0.5,
    )
    lsp_terminal_await_request = {
        "request_id": "daemon-lsp-terminal-rename-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-lsp-terminal-rename-command-1",
            "timeout_seconds": 0.0,
        },
    }
    lsp_terminal_await_response = request_or_outcome_unknown(
        args.socket,
        lsp_terminal_await_request,
        timeout=2.0,
    )
    lsp_terminal_await_result = lsp_terminal_await_response.get("result") or {}
    lsp_terminal_final_status = (
        lsp_terminal_await_result.get("status")
        or lsp_terminal_terminal_status.get("status")
        or lsp_terminal_observed_status
        or lsp_terminal_response
    )
    lsp_terminal_result = lsp_terminal_final_status.get("result") or {}
    lsp_terminal_after = lsp_terminal_result.get("lsp_after") or {}
    lsp_terminal_saves = [
        save_buffer_via_bridge(args.socket, name, f"daemon-lsp-terminal-rename-{index}")
        for index, name in enumerate(
            [lsp_buffer, lsp_main_buffer, lsp_test_buffer],
            start=1,
        )
        if name
    ]
    lsp_terminal_build_test = run_command(
        [
            "python3",
            "-c",
            (
                "import pathlib, sys, unittest; "
                f"sys.path.insert(0, {json.dumps(lsp_root)}); "
                f"suite = unittest.defaultTestLoader.discover({json.dumps(lsp_test_dir)}); "
                "result = unittest.TextTestRunner(verbosity=2).run(suite); "
                "raise SystemExit(0 if result.wasSuccessful() else 1)"
            ),
        ]
    )
    lsp_terminal_definition_text = read_text(lsp_definition_file)
    lsp_terminal_main_text = read_text(lsp_main_file)
    lsp_terminal_test_text = read_text(lsp_test_file)
    records.append(
        scenario_record(
            25,
            "daemon-lsp-terminal-rename-symbol",
            "A queued daemon command runs interactive lsp-rename and receives the new name from the terminal minibuffer",
            True,
            args.out_dir,
            fixture=lsp_fixture,
            terminal_switch={
                "tmux_target": lsp_target,
                "response": lsp_switch["response"],
                "terminal_frames": lsp_switch["terminal_frames"],
            },
            observe_request=lsp_terminal_observe_request,
            observe_response=lsp_terminal_observe_response,
            dispatch_request=lsp_terminal_request,
            dispatch_response=lsp_terminal_response,
            interaction_attempts=lsp_terminal_interaction_attempts,
            interaction_response=lsp_terminal_interaction_response,
            status_request=lsp_terminal_status_request,
            status_response=lsp_terminal_status_response,
            terminal_clear_default=lsp_terminal_clear,
            terminal_input=lsp_terminal_input,
            terminal_submit=lsp_terminal_submit,
            terminal_status=lsp_terminal_terminal_status,
            await_request=lsp_terminal_await_request,
            await_response=lsp_terminal_await_response,
            final_status=lsp_terminal_final_status,
            save_results=lsp_terminal_saves,
            build_test=lsp_terminal_build_test,
            final_files={
                "definition": lsp_terminal_definition_text,
                "main": lsp_terminal_main_text,
                "test": lsp_terminal_test_text,
            },
            checks=[
                check("interactive LSP fixture is present", bool(lsp_buffer and lsp_root and lsp_new_name), lsp_fixture),
                check("interactive LSP source observation succeeded", lsp_terminal_observe_response.get("state") == "succeeded", lsp_terminal_observe_response),
                check("interactive LSP starts from semantic rename output", lsp_terminal_observation.get("symbol") == lsp_new_name, lsp_terminal_observation),
                check("interactive LSP command dispatch is accepted as running", lsp_terminal_response.get("state") == "running", lsp_terminal_response),
                check("interactive LSP rename minibuffer opens", lsp_terminal_interaction_response.get("state") == "succeeded" and lsp_terminal_interaction_result.get("minibuffer_active") is True, lsp_terminal_interaction_response),
                check("interactive LSP rename prompt names current symbol", contains(str(lsp_terminal_interaction_result.get("minibuffer_prompt") or ""), lsp_new_name), lsp_terminal_interaction_result),
                check("interactive LSP default name is visible in minibuffer contents", contains(str(lsp_terminal_interaction_result.get("minibuffer_contents") or ""), lsp_new_name), lsp_terminal_interaction_result),
                check("interactive LSP rename status is observable while prompted", lsp_terminal_status_response.get("state") == "succeeded" and lsp_terminal_observed_status.get("request_id") == "daemon-lsp-terminal-rename-command-1", lsp_terminal_status_response),
                check("interactive LSP terminal clear succeeded", lsp_terminal_clear.get("returncode") == 0, lsp_terminal_clear),
                check("interactive LSP terminal input succeeded", lsp_terminal_input.get("returncode") == 0, lsp_terminal_input),
                check("interactive LSP terminal submit succeeded", lsp_terminal_submit.get("returncode") == 0, lsp_terminal_submit),
                check("interactive LSP terminal status polling observes completion", lsp_terminal_terminal_status.get("status", {}).get("state") == "succeeded", lsp_terminal_terminal_status),
                check("interactive LSP rename reaches terminal success", lsp_terminal_await_response.get("state") == "succeeded" and lsp_terminal_final_status.get("state") == "succeeded", {"await": lsp_terminal_await_response, "final": lsp_terminal_final_status}),
                check("interactive LSP route is command-execute", lsp_terminal_result.get("route") == "command-execute", lsp_terminal_result),
                check("interactive LSP this-command is lsp-rename", lsp_terminal_result.get("this_command") == "lsp-rename", lsp_terminal_result),
                check("interactive LSP sent textDocument/rename", "textDocument/rename" in (lsp_terminal_result.get("lsp_request_methods") or []), lsp_terminal_result),
                check("interactive LSP workspace edit was applied", lsp_terminal_result.get("lsp_workspace_edit_applied") is True, lsp_terminal_result),
                check("interactive LSP after-observation sees new symbol", lsp_terminal_after.get("symbol") == lsp_terminal_new_name, lsp_terminal_after),
                check("interactive LSP save operations succeeded", all((item.get("save_response") or {}).get("state") == "succeeded" for item in lsp_terminal_saves) and len(lsp_terminal_saves) == 3, lsp_terminal_saves),
                check("interactive LSP rename updated definition file", lsp_terminal_new_name in lsp_terminal_definition_text and lsp_new_name not in lsp_terminal_definition_text and lsp_symbol not in lsp_terminal_definition_text, lsp_terminal_definition_text),
                check("interactive LSP rename updated main file", lsp_terminal_new_name in lsp_terminal_main_text and lsp_new_name not in lsp_terminal_main_text and lsp_symbol not in lsp_terminal_main_text, lsp_terminal_main_text),
                check("interactive LSP rename updated test file", lsp_terminal_new_name in lsp_terminal_test_text and lsp_new_name not in lsp_terminal_test_text and lsp_symbol not in lsp_terminal_test_text, lsp_terminal_test_text),
                check("rootfs Python tests pass after interactive LSP rename", lsp_terminal_build_test.get("returncode") == 0, lsp_terminal_build_test),
            ],
        )
    )

    lsp_diag_initial_request = {
        "request_id": "daemon-lsp-diagnostics-initial-1",
        "operation": "lsp.observe_diagnostics",
        "params": {
            "buffer_name": lsp_diag_buffer,
            "settle_seconds": 0.2,
        },
    }
    lsp_diag_initial_response = request_or_outcome_unknown(
        args.socket,
        lsp_diag_initial_request,
        timeout=10.0,
    )
    lsp_diag_initial_observation = response_result(lsp_diag_initial_response)
    lsp_diag_initial_evidence = lsp_diag_initial_response.get("evidence") or {}
    lsp_diag_initial_observations = lsp_diag_initial_response.get("observations") or {}
    lsp_diag_initial_effects = lsp_diag_initial_response.get("effects") or {}
    lsp_diag_initial_text_read = full_buffer_text_via_bridge(
        args.socket,
        lsp_diag_buffer,
        "daemon-lsp-diagnostics-initial-text",
    )
    lsp_diag_initial_text = str(lsp_diag_initial_text_read.get("text") or "")
    lsp_diag_break_index = lsp_diag_initial_text.find(lsp_diag_good_return)
    if lsp_diag_break_index >= 0:
        lsp_diag_break_start = lsp_diag_break_index + 1
        lsp_diag_break_end = lsp_diag_break_start + len(lsp_diag_good_return)
        lsp_diag_break_edit = replace_range_via_bridge(
            args.socket,
            lsp_diag_buffer,
            lsp_diag_break_start,
            lsp_diag_break_end,
            lsp_diag_broken_return,
            lsp_diag_good_return,
            "daemon-lsp-diagnostics-break",
        )
    else:
        lsp_diag_break_start = None
        lsp_diag_break_end = None
        lsp_diag_break_edit = {
            "buffer_name": lsp_diag_buffer,
            "start": None,
            "end": None,
            "expected_text": lsp_diag_good_return,
            "replacement_text": lsp_diag_broken_return,
            "observe_request": None,
            "observe_response": lsp_diag_initial_response,
            "replace_request": None,
            "replace_response": {
                "state": "failed",
                "error_code": "target_text_missing",
                "error": "diagnostic fixture target text was not found",
            },
        }
    lsp_diag_break_save = save_buffer_via_bridge(
        args.socket,
        lsp_diag_buffer,
        "daemon-lsp-diagnostics-break-save",
    )
    lsp_diag_error_wait = wait_lsp_diagnostic_count(
        args.socket,
        lsp_diag_buffer,
        "daemon-lsp-diagnostics-error",
        lambda count, observation: (
            isinstance(count, int)
            and count > 0
            and lsp_diagnostic_contains(observation, lsp_diag_expected_diagnostic_needle)
        ),
        timeout_seconds=15.0,
    )
    lsp_diag_error_observation = lsp_diag_error_wait.get("observation") or {}
    lsp_diag_error_response = lsp_diag_error_wait.get("response") or {}
    lsp_diag_error_evidence = lsp_diag_error_response.get("evidence") or {}
    lsp_diag_error_text_read = full_buffer_text_via_bridge(
        args.socket,
        lsp_diag_buffer,
        "daemon-lsp-diagnostics-error-text",
    )
    lsp_diag_error_text = str(lsp_diag_error_text_read.get("text") or "")
    lsp_diag_fix_index = lsp_diag_error_text.find(lsp_diag_broken_return)
    if lsp_diag_fix_index >= 0:
        lsp_diag_fix_start = lsp_diag_fix_index + 1
        lsp_diag_fix_end = lsp_diag_fix_start + len(lsp_diag_broken_return)
        lsp_diag_fix_edit = replace_range_via_bridge(
            args.socket,
            lsp_diag_buffer,
            lsp_diag_fix_start,
            lsp_diag_fix_end,
            lsp_diag_good_return,
            lsp_diag_broken_return,
            "daemon-lsp-diagnostics-fix",
        )
    else:
        lsp_diag_fix_start = None
        lsp_diag_fix_end = None
        lsp_diag_fix_edit = {
            "buffer_name": lsp_diag_buffer,
            "start": None,
            "end": None,
            "expected_text": lsp_diag_broken_return,
            "replacement_text": lsp_diag_good_return,
            "observe_request": None,
            "observe_response": lsp_diag_error_wait.get("response"),
            "replace_request": None,
            "replace_response": {
                "state": "failed",
                "error_code": "target_text_missing",
                "error": "diagnostic repair target text was not found",
            },
        }
    lsp_diag_save = save_buffer_via_bridge(
        args.socket,
        lsp_diag_buffer,
        "daemon-lsp-diagnostics-repair",
    )
    lsp_diag_clear_wait = wait_lsp_diagnostic_count(
        args.socket,
        lsp_diag_buffer,
        "daemon-lsp-diagnostics-clear",
        lambda count, _observation: count == 0,
        timeout_seconds=15.0,
    )
    lsp_diag_clear_response = lsp_diag_clear_wait.get("response") or {}
    lsp_diag_clear_evidence = lsp_diag_clear_response.get("evidence") or {}
    lsp_diag_build_test = run_command(
        [
            "python3",
            "-c",
            (
                "import pathlib, sys, unittest; "
                f"sys.path.insert(0, {json.dumps(lsp_diag_root)}); "
                f"suite = unittest.defaultTestLoader.discover({json.dumps(lsp_diag_test_dir)}); "
                "result = unittest.TextTestRunner(verbosity=2).run(suite); "
                "raise SystemExit(0 if result.wasSuccessful() else 1)"
            ),
        ]
    )
    lsp_diag_final_text = read_text(lsp_diag_definition_file)
    records.append(
        scenario_record(
            26,
            "daemon-lsp-diagnostics-after-edit",
            "A guarded daemon buffer edit creates live pyright diagnostics, repairs the source, saves it, and observes diagnostics clearing",
            True,
            args.out_dir,
            fixture=lsp_diagnostics_fixture,
            initial_request=lsp_diag_initial_request,
            initial_response=lsp_diag_initial_response,
            break_edit=lsp_diag_break_edit,
            break_save=lsp_diag_break_save,
            error_wait=lsp_diag_error_wait,
            initial_text_read=lsp_diag_initial_text_read,
            error_text_read=lsp_diag_error_text_read,
            fix_edit=lsp_diag_fix_edit,
            clear_wait=lsp_diag_clear_wait,
            save_result=lsp_diag_save,
            build_test=lsp_diag_build_test,
            final_file=lsp_diag_final_text,
            checks=[
                check("diagnostic LSP fixture is present", bool(lsp_diag_buffer and lsp_diag_root and lsp_diag_test_dir), lsp_diagnostics_fixture),
                check("diagnostic fixture uses full-document LSP sync", lsp_diagnostics_fixture.get("diagnostic_sync_method") == "full" and lsp_diagnostics_fixture.get("diagnostic_debounce_full_sync") is False, lsp_diagnostics_fixture),
                check("diagnostic fixture clears stale diagnostics on buffer change", lsp_diagnostics_fixture.get("diagnostic_clean_after_change") is True, lsp_diagnostics_fixture),
                check("diagnostic fixture observes protocol cache without UI backend", lsp_diagnostics_fixture.get("diagnostics_provider") == "none", lsp_diagnostics_fixture),
                check("diagnostic pyright executable is rootfs-owned", rootfs_owned_process_path(lsp_diag_pyright_path), lsp_diag_pyright_path),
                check("diagnostic node executable is rootfs-owned", rootfs_owned_process_path(lsp_diag_node_path), lsp_diag_node_path),
                check("diagnostic Python executable is rootfs-owned", rootfs_owned_process_path(lsp_diag_python_path), lsp_diag_python_path),
                check("diagnostic initial LSP observation succeeded", lsp_diag_initial_response.get("state") == "succeeded" and lsp_diag_initial_response.get("operation") == "lsp.observe_diagnostics", lsp_diag_initial_response),
                check("diagnostic initial observation includes diagnostic evidence", isinstance(lsp_diag_initial_observation.get("diagnostics"), dict), lsp_diag_initial_observation),
                check("diagnostic initial response publishes typed LSP evidence", isinstance(lsp_diag_initial_evidence, dict) and isinstance(lsp_diag_initial_evidence.get("workspace"), dict) and isinstance(lsp_diag_initial_evidence.get("server"), dict) and lsp_diag_initial_evidence.get("server", {}).get("id") == lsp_diagnostics_fixture.get("server_id") and isinstance(lsp_diag_initial_evidence.get("document"), dict) and isinstance(lsp_diag_initial_evidence.get("diagnostics"), dict) and isinstance(lsp_diag_initial_evidence.get("diagnostic_cache"), dict) and isinstance(lsp_diag_initial_evidence.get("lsp_sync"), dict) and isinstance(lsp_diag_initial_evidence.get("rootfs_runtime"), dict), lsp_diag_initial_evidence),
                check("diagnostic initial response publishes typed observations without effects", isinstance(lsp_diag_initial_observations, dict) and isinstance(lsp_diag_initial_observations.get("source_buffer"), dict) and lsp_diag_initial_observations.get("source_buffer", {}).get("buffer_name") == lsp_diag_buffer and isinstance(lsp_diag_initial_observations.get("document"), dict) and isinstance(lsp_diag_initial_observations.get("diagnostics"), dict) and isinstance(lsp_diag_initial_effects, dict) and not lsp_diag_initial_effects, {"observations": lsp_diag_initial_observations, "effects": lsp_diag_initial_effects}),
                check("diagnostic LSP server is pyright fixture", lsp_diag_initial_observation.get("server_id") == lsp_diagnostics_fixture.get("server_id"), lsp_diag_initial_observation),
                check("diagnostic initial count is zero", lsp_diagnostic_count(lsp_diag_initial_observation) == 0, lsp_diag_initial_observation.get("diagnostics")),
                check("diagnostic edit target text was read through buffer API", (lsp_diag_initial_text_read.get("read_response") or {}).get("state") == "succeeded" and lsp_diag_good_return in lsp_diag_initial_text, lsp_diag_initial_text_read),
                check("diagnostic break target range was found", isinstance(lsp_diag_break_start, int) and isinstance(lsp_diag_break_end, int), {"text": lsp_diag_initial_text, "needle": lsp_diag_good_return}),
                check("diagnostic break edit used guarded buffer replacement", (lsp_diag_break_edit.get("replace_response") or {}).get("state") == "succeeded", lsp_diag_break_edit),
                check("diagnostic broken buffer save succeeded", (lsp_diag_break_save.get("save_response") or {}).get("state") == "succeeded", lsp_diag_break_save),
                check("diagnostic expected pyright syntax diagnostic appears after broken save", isinstance(lsp_diag_error_wait.get("diagnostic_count"), int) and lsp_diag_error_wait.get("diagnostic_count", 0) > 0 and lsp_diagnostic_contains(lsp_diag_error_observation, lsp_diag_expected_diagnostic_needle), lsp_diag_error_wait),
                check("diagnostic error response publishes typed diagnostic evidence", isinstance(lsp_diag_error_evidence, dict) and isinstance(lsp_diag_error_evidence.get("diagnostics"), dict) and lsp_diag_error_evidence.get("diagnostics", {}).get("count") == lsp_diag_error_wait.get("diagnostic_count") and isinstance(lsp_diag_error_evidence.get("diagnostic_cache"), dict), lsp_diag_error_evidence),
                check("diagnostic error observation still uses rootfs LSP server", rootfs_owned_process_path(str(lsp_diag_error_observation.get("server_executable") or "")), lsp_diag_error_observation.get("server_executable")),
                check("diagnostic repair target text was read through buffer API", (lsp_diag_error_text_read.get("read_response") or {}).get("state") == "succeeded" and lsp_diag_broken_return in lsp_diag_error_text, lsp_diag_error_text_read),
                check("diagnostic repair target range was found", isinstance(lsp_diag_fix_start, int) and isinstance(lsp_diag_fix_end, int), {"text": lsp_diag_error_text, "needle": lsp_diag_broken_return}),
                check("diagnostic repair edit used guarded buffer replacement", (lsp_diag_fix_edit.get("replace_response") or {}).get("state") == "succeeded", lsp_diag_fix_edit),
                check("diagnostics clear after repair", lsp_diag_clear_wait.get("diagnostic_count") == 0, lsp_diag_clear_wait),
                check("diagnostic clear response publishes typed zero-diagnostic evidence", isinstance(lsp_diag_clear_evidence, dict) and isinstance(lsp_diag_clear_evidence.get("diagnostics"), dict) and lsp_diag_clear_evidence.get("diagnostics", {}).get("count") == 0 and isinstance(lsp_diag_clear_evidence.get("diagnostic_cache"), dict), lsp_diag_clear_evidence),
                check("diagnostic repaired buffer save succeeded", (lsp_diag_save.get("save_response") or {}).get("state") == "succeeded", lsp_diag_save),
                check("diagnostic final source is restored", lsp_diag_good_return in lsp_diag_final_text and lsp_diag_broken_return + "\n" not in lsp_diag_final_text, lsp_diag_final_text),
                check("rootfs Python tests pass after diagnostic repair", lsp_diag_build_test.get("returncode") == 0, lsp_diag_build_test),
            ],
        )
    )

    lsp_bypass_original_text = lsp_diag_final_text
    lsp_bypass_symbol = str(lsp_diagnostics_fixture.get("symbol") or "")
    lsp_bypass_new_name = "compute_total_direct_text"
    lsp_bypass_mutated_text = (
        lsp_bypass_original_text.replace(lsp_bypass_symbol, lsp_bypass_new_name, 1)
        if lsp_bypass_symbol
        else lsp_bypass_original_text
    )
    lsp_bypass_symbol_index = (
        lsp_bypass_original_text.find(lsp_bypass_symbol)
        if lsp_bypass_symbol
        else -1
    )
    if lsp_bypass_symbol_index >= 0:
        lsp_bypass_start = lsp_bypass_symbol_index + 1
        lsp_bypass_end = lsp_bypass_start + len(lsp_bypass_symbol)
        lsp_bypass_edit = replace_range_via_bridge(
            args.socket,
            lsp_diag_buffer,
            lsp_bypass_start,
            lsp_bypass_end,
            lsp_bypass_new_name,
            lsp_bypass_symbol,
            "daemon-lsp-semantic-bypass-text-edit",
        )
    else:
        lsp_bypass_start = None
        lsp_bypass_end = None
        lsp_bypass_edit = {
            "buffer_name": lsp_diag_buffer,
            "start": None,
            "end": None,
            "expected_text": lsp_bypass_symbol,
            "replacement_text": lsp_bypass_new_name,
            "observe_request": None,
            "observe_response": {},
            "replace_request": None,
            "replace_response": {
                "state": "failed",
                "error_code": "target_text_missing",
                "error": "LSP bypass source symbol was not found",
            },
        }
    lsp_bypass_finish_request = {
        "request_id": "daemon-lsp-semantic-bypass-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-lsp-semantic-bypass-rename-1",
                    "state": "succeeded",
                }
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": lsp_diag_buffer,
                    "start": 1,
                    "end": len(lsp_bypass_mutated_text) + 1,
                    "preconditions": {
                        "expected_text": lsp_bypass_mutated_text,
                    },
                }
            ],
        },
    }
    lsp_bypass_finish_response = request_or_outcome_unknown(
        args.socket,
        lsp_bypass_finish_request,
        timeout=2.0,
    )
    lsp_bypass_finish_result = lsp_bypass_finish_response.get("result") or {}
    lsp_bypass_request_checks = lsp_bypass_finish_result.get("request_checks") or []
    lsp_bypass_buffer_checks = lsp_bypass_finish_result.get("buffer_checks") or []
    records.append(
        scenario_record(
            41,
            "daemon-lsp-text-bypass-rejected",
            "episode.finish rejects direct LSP source text substitution when semantic rename evidence is required",
            True,
            args.out_dir,
            fixture=lsp_diagnostics_fixture,
            original_text=lsp_bypass_original_text,
            mutated_text=lsp_bypass_mutated_text,
            text_edit=lsp_bypass_edit,
            finish_request=lsp_bypass_finish_request,
            finish_response=lsp_bypass_finish_response,
            checks=[
                check("LSP semantic-bypass target symbol is present", lsp_bypass_symbol_index >= 0 and lsp_bypass_symbol in lsp_bypass_original_text, {"symbol": lsp_bypass_symbol, "index": lsp_bypass_symbol_index, "text": lsp_bypass_original_text}),
                check("LSP semantic-bypass checked text edit succeeds", (lsp_bypass_edit.get("replace_response") or {}).get("state") == "succeeded", lsp_bypass_edit),
                check("LSP semantic-bypass final source text matches direct substitution", f"def {lsp_bypass_new_name}(" in lsp_bypass_mutated_text and f"def {lsp_bypass_symbol}(" not in lsp_bypass_mutated_text, lsp_bypass_mutated_text),
                check("LSP semantic-bypass episode finish fails without rename request evidence", lsp_bypass_finish_response.get("state") == "failed" and lsp_bypass_finish_result.get("request_checks_passed") is False, lsp_bypass_finish_response),
                check("LSP semantic-bypass missing rename request id is reported", any(item.get("request_id") == "daemon-lsp-semantic-bypass-rename-1" and item.get("passed") is False and item.get("detail") == "request id is not known" for item in lsp_bypass_request_checks), lsp_bypass_request_checks),
                check("LSP semantic-bypass buffer checks pass independently", lsp_bypass_finish_result.get("buffer_checks_passed") is True and any(item.get("buffer_name") == lsp_diag_buffer and item.get("passed") is True for item in lsp_bypass_buffer_checks), lsp_bypass_buffer_checks),
            ],
        )
    )

    multi_async_a_request = {
        "request_id": "daemon-multi-async-a",
        "operation": "bridge.defer_echo",
        "params": {
            "delay_seconds": 2.0,
            "value": "multi async A",
        },
    }
    multi_async_b_request = {
        "request_id": "daemon-multi-async-b",
        "operation": "bridge.defer_echo",
        "params": {
            "delay_seconds": 0.4,
            "value": "multi async B",
        },
    }
    multi_async_a_response = request(args.socket, multi_async_a_request)
    multi_async_b_response = request(args.socket, multi_async_b_request)
    multi_async_a_status_request = {
        "request_id": "daemon-multi-async-a-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-multi-async-a"},
    }
    multi_async_b_status_request = {
        "request_id": "daemon-multi-async-b-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-multi-async-b"},
    }
    multi_async_a_status_response = request(args.socket, multi_async_a_status_request)
    multi_async_b_status_response = request(args.socket, multi_async_b_status_request)
    multi_async_a_initial_status = (
        (multi_async_a_status_response.get("result") or {}).get("status") or {}
    )
    multi_async_b_initial_status = (
        (multi_async_b_status_response.get("result") or {}).get("status") or {}
    )
    multi_async_b_await_request = {
        "request_id": "daemon-multi-async-b-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-multi-async-b",
            "timeout_seconds": 2.0,
        },
    }
    multi_async_b_await_response = request(args.socket, multi_async_b_await_request, timeout=3.0)
    multi_async_b_await_status = (
        (multi_async_b_await_response.get("result") or {}).get("status") or {}
    )
    multi_async_a_mid_status_request = {
        "request_id": "daemon-multi-async-a-status-2",
        "operation": "request.status",
        "params": {"request_id": "daemon-multi-async-a"},
    }
    multi_async_a_mid_status_response = request(args.socket, multi_async_a_mid_status_request)
    multi_async_a_mid_status = (
        (multi_async_a_mid_status_response.get("result") or {}).get("status") or {}
    )
    multi_async_a_await_request = {
        "request_id": "daemon-multi-async-a-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-multi-async-a",
            "timeout_seconds": 3.0,
        },
    }
    multi_async_a_await_response = request(args.socket, multi_async_a_await_request, timeout=4.0)
    multi_async_a_await_status = (
        (multi_async_a_await_response.get("result") or {}).get("status") or {}
    )
    multi_async_a_final_status_request = {
        "request_id": "daemon-multi-async-a-status-3",
        "operation": "request.status",
        "params": {"request_id": "daemon-multi-async-a"},
    }
    multi_async_b_final_status_request = {
        "request_id": "daemon-multi-async-b-status-2",
        "operation": "request.status",
        "params": {"request_id": "daemon-multi-async-b"},
    }
    multi_async_a_final_status_response = request(args.socket, multi_async_a_final_status_request)
    multi_async_b_final_status_response = request(args.socket, multi_async_b_final_status_request)
    multi_async_a_final_status = (
        (multi_async_a_final_status_response.get("result") or {}).get("status") or {}
    )
    multi_async_b_final_status = (
        (multi_async_b_final_status_response.get("result") or {}).get("status") or {}
    )
    records.append(
        scenario_record(
            27,
            "daemon-multi-async-status-await",
            "Two daemon-owned async requests keep independent status and can be awaited out of submission order",
            True,
            args.out_dir,
            request_a=multi_async_a_request,
            response_a=multi_async_a_response,
            request_b=multi_async_b_request,
            response_b=multi_async_b_response,
            initial_status_a_request=multi_async_a_status_request,
            initial_status_a_response=multi_async_a_status_response,
            initial_status_b_request=multi_async_b_status_request,
            initial_status_b_response=multi_async_b_status_response,
            await_b_request=multi_async_b_await_request,
            await_b_response=multi_async_b_await_response,
            mid_status_a_request=multi_async_a_mid_status_request,
            mid_status_a_response=multi_async_a_mid_status_response,
            await_a_request=multi_async_a_await_request,
            await_a_response=multi_async_a_await_response,
            final_status_a_request=multi_async_a_final_status_request,
            final_status_a_response=multi_async_a_final_status_response,
            final_status_b_request=multi_async_b_final_status_request,
            final_status_b_response=multi_async_b_final_status_response,
            checks=[
                check("async A request is accepted as running", multi_async_a_response.get("state") == "running", multi_async_a_response),
                check("async B request is accepted as running", multi_async_b_response.get("state") == "running", multi_async_b_response),
                check("async A initial status is running", multi_async_a_initial_status.get("request_id") == "daemon-multi-async-a" and multi_async_a_initial_status.get("state") == "running", multi_async_a_initial_status),
                check("async B initial status is running", multi_async_b_initial_status.get("request_id") == "daemon-multi-async-b" and multi_async_b_initial_status.get("state") == "running", multi_async_b_initial_status),
                check("shorter async B can be awaited before A", multi_async_b_await_response.get("state") == "succeeded" and multi_async_b_await_status.get("request_id") == "daemon-multi-async-b" and multi_async_b_await_status.get("state") == "succeeded", multi_async_b_await_response),
                check("async B preserves its own result value", (multi_async_b_await_status.get("result") or {}).get("value") == "multi async B", multi_async_b_await_status),
                check("awaiting B does not swap A status", multi_async_a_mid_status.get("request_id") == "daemon-multi-async-a" and (multi_async_a_mid_status.get("result") or {}).get("value") != "multi async B", multi_async_a_mid_status),
                check("longer async A can be awaited after B", multi_async_a_await_response.get("state") == "succeeded" and multi_async_a_await_status.get("request_id") == "daemon-multi-async-a" and multi_async_a_await_status.get("state") == "succeeded", multi_async_a_await_response),
                check("async A preserves its own result value", (multi_async_a_await_status.get("result") or {}).get("value") == "multi async A", multi_async_a_await_status),
                check("final A status remains independent", multi_async_a_final_status.get("request_id") == "daemon-multi-async-a" and (multi_async_a_final_status.get("result") or {}).get("value") == "multi async A", multi_async_a_final_status),
                check("final B status remains independent", multi_async_b_final_status.get("request_id") == "daemon-multi-async-b" and (multi_async_b_final_status.get("result") or {}).get("value") == "multi async B", multi_async_b_final_status),
            ],
        )
    )

    async_jupyter_fixture = (ready or {}).get("async_jupyter_fixture") or {}
    async_jupyter_buffer = async_jupyter_fixture.get("source_buffer") or ""
    async_jupyter_beg = async_jupyter_fixture.get("beg")
    async_jupyter_end = async_jupyter_fixture.get("end")
    async_jupyter_selected_code = async_jupyter_fixture.get("selected_code") or ""
    async_jupyter_training_code = async_jupyter_fixture.get("training_code") or ""
    async_jupyter_target = f"{args.terminal_tmux_session}:0.0"
    async_jupyter_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        async_jupyter_target,
        async_jupyter_buffer,
        "daemon-package-async-jupyter",
    )
    async_jupyter_observe_request = {
        "request_id": "daemon-package-async-jupyter-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": async_jupyter_buffer,
            "start": async_jupyter_beg,
            "end": async_jupyter_end,
            "max_text": 2400,
        },
    }
    async_jupyter_observe_response = request_or_outcome_unknown(
        args.socket,
        async_jupyter_observe_request,
        timeout=5.0,
    )
    async_jupyter_observation = async_jupyter_observe_response.get("result") or {}
    async_jupyter_selected_range = async_jupyter_observation.get("selected_range") or {}
    async_jupyter_eval_preconditions = jupyter_preconditions_from_observation(
        async_jupyter_observation
    )
    async_jupyter_eval_preconditions.pop("selected_window_buffer", None)
    async_jupyter_eval_preconditions["expected_text"] = async_jupyter_selected_code
    async_jupyter_eval_request = {
        "request_id": "daemon-package-async-jupyter-eval-1",
        "operation": "jupyter.eval_region",
        "params": {
            "buffer_name": async_jupyter_buffer,
            "start": async_jupyter_beg,
            "end": async_jupyter_end,
            "timeout_seconds": 30.0,
            "queue_delay_seconds": 4.0,
            "preconditions": async_jupyter_eval_preconditions,
        },
    }
    async_jupyter_eval_response = request_or_outcome_unknown(
        args.socket,
        async_jupyter_eval_request,
        timeout=2.0,
    )
    async_jupyter_initial_status_request = {
        "request_id": "daemon-package-async-jupyter-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-package-async-jupyter-eval-1"},
    }
    async_jupyter_initial_status_response = request_or_outcome_unknown(
        args.socket,
        async_jupyter_initial_status_request,
        timeout=0.5,
    )
    async_jupyter_initial_status = (
        (async_jupyter_initial_status_response.get("result") or {}).get("status") or {}
    )

    async_magit_fixture = (ready or {}).get("async_magit_fixture") or {}
    async_magit_repo = async_magit_fixture.get("repo") or ""
    async_magit_buffer = async_magit_fixture.get("status_buffer") or ""
    async_magit_hook_log = async_magit_fixture.get("hook_log") or ""
    async_magit_target = f"{args.terminal_tmux_session}:0.0"
    async_magit_initial_state = magit_fixture_repo_state(
        async_magit_repo,
        async_magit_hook_log,
    )
    async_magit_switch = switch_terminal_frame_to_buffer(
        args.socket,
        args.terminal_tmux_socket,
        async_magit_target,
        async_magit_buffer,
        "daemon-package-async-magit",
    )
    async_magit_sections_request = {
        "request_id": "daemon-package-async-magit-sections-1",
        "operation": "magit.observe_sections",
        "params": {
            "buffer_name": async_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    async_magit_sections_response = request_or_outcome_unknown(
        args.socket,
        async_magit_sections_request,
        timeout=3.0,
    )
    async_magit_sections = async_magit_sections_response.get("result") or {}
    async_magit_target_section = find_magit_section(
        async_magit_sections,
        section_type="hunk",
        needle="target-05-selected",
        ancestor_file="target.txt",
    )
    async_magit_target_point = magit_section_point(async_magit_target_section)
    if isinstance(async_magit_target_point, int):
        async_magit_goto_request = {
            "request_id": "daemon-package-async-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": async_magit_buffer,
                "point": async_magit_target_point,
                "preconditions": async_magit_sections,
            },
        }
        async_magit_goto_response = request_or_outcome_unknown(
            args.socket,
            async_magit_goto_request,
            timeout=2.0,
        )
    else:
        async_magit_goto_request = {
            "request_id": "daemon-package-async-magit-goto-1",
            "operation": "emacs.goto_char",
            "params": {
                "buffer_name": async_magit_buffer,
                "point": None,
                "preconditions": async_magit_sections,
            },
        }
        async_magit_goto_response = {
            "request_id": "daemon-package-async-magit-goto-1",
            "operation": "emacs.goto_char",
            "state": "failed",
            "state_sequence": ["accepted", "failed"],
            "result": None,
            "error_code": "missing_target_point",
            "error": "async Magit fixture did not expose a target hunk point",
        }
    async_magit_ready_sections_request = {
        "request_id": "daemon-package-async-magit-ready-sections-1",
        "operation": "magit.observe_sections",
        "params": {
            "buffer_name": async_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    async_magit_ready_sections_response = request_or_outcome_unknown(
        args.socket,
        async_magit_ready_sections_request,
        timeout=3.0,
    )
    async_magit_ready_sections = async_magit_ready_sections_response.get("result") or {}
    async_magit_ready_selected = async_magit_ready_sections.get("selected_section") or {}

    async_magit_dispatch_request = {
        "request_id": "daemon-package-async-magit-stage-1",
        "operation": "command.dispatch",
        "params": {
            "buffer_name": async_magit_buffer,
            "command": "magit-stage",
            "preconditions": async_magit_ready_sections,
        },
    }
    async_magit_dispatch_response = request_or_outcome_unknown(
        args.socket,
        async_magit_dispatch_request,
        timeout=2.0,
    )
    async_magit_status_request = {
        "request_id": "daemon-package-async-magit-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-package-async-magit-stage-1"},
    }
    async_magit_status_response = request_or_outcome_unknown(
        args.socket,
        async_magit_status_request,
        timeout=0.5,
    )
    async_magit_running_status = (
        (async_magit_status_response.get("result") or {}).get("status") or {}
    )
    async_magit_await_request = {
        "request_id": "daemon-package-async-magit-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-package-async-magit-stage-1",
            "timeout_seconds": 4.0,
        },
    }
    async_magit_await_response = request_or_outcome_unknown(
        args.socket,
        async_magit_await_request,
        timeout=6.0,
    )
    async_magit_await_status = (
        (async_magit_await_response.get("result") or {}).get("status")
        or async_magit_running_status
        or async_magit_dispatch_response
    )
    async_magit_result = async_magit_await_status.get("result") or {}
    async_after_magit_state = magit_fixture_repo_state(
        async_magit_repo,
        async_magit_hook_log,
    )
    async_jupyter_mid_status_request = {
        "request_id": "daemon-package-async-jupyter-status-2",
        "operation": "request.status",
        "params": {"request_id": "daemon-package-async-jupyter-eval-1"},
    }
    async_jupyter_mid_status_response = request_or_outcome_unknown(
        args.socket,
        async_jupyter_mid_status_request,
        timeout=0.5,
    )
    async_jupyter_mid_status = (
        (async_jupyter_mid_status_response.get("result") or {}).get("status") or {}
    )
    async_jupyter_await_request = {
        "request_id": "daemon-package-async-jupyter-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-package-async-jupyter-eval-1",
            "timeout_seconds": 10.0,
        },
    }
    async_jupyter_await_response = request_or_outcome_unknown(
        args.socket,
        async_jupyter_await_request,
        timeout=12.0,
    )
    async_jupyter_await_status = (
        (async_jupyter_await_response.get("result") or {}).get("status")
        or async_jupyter_mid_status
        or async_jupyter_initial_status
        or async_jupyter_eval_response
    )
    async_jupyter_result = async_jupyter_await_status.get("result") or {}
    async_jupyter_initial_effects = async_jupyter_eval_response.get("effects") or {}
    async_jupyter_final_effects = async_jupyter_await_status.get("effects") or {}
    async_jupyter_kernel_execution_effect = (
        async_jupyter_final_effects.get("kernel_execution")
        if isinstance(async_jupyter_final_effects, dict)
        else {}
    )
    async_jupyter_stream_payload = stream_json_payload(
        async_jupyter_result.get("stream_texts") or [],
        "SOMA_ASYNC_PACKAGE_JSON:",
    )
    async_jupyter_query_request = {
        "request_id": "daemon-package-async-jupyter-query-1",
        "operation": "jupyter.query_state",
        "params": {
            "buffer_name": async_jupyter_buffer,
            "timeout_seconds": 30.0,
        },
    }
    async_jupyter_query_response = request_or_outcome_unknown(
        args.socket,
        async_jupyter_query_request,
        timeout=35.0,
    )
    async_jupyter_query_result = async_jupyter_query_response.get("result") or {}
    async_jupyter_payload = async_jupyter_query_result.get("payload") or {}
    async_finish_request = {
        "request_id": "daemon-package-async-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-package-async-jupyter-eval-1",
                    "state": "succeeded",
                },
                {
                    "request_id": "daemon-package-async-magit-stage-1",
                    "state": "succeeded",
                },
                {
                    "request_id": "daemon-package-async-jupyter-query-1",
                    "state": "succeeded",
                },
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": async_jupyter_buffer,
                    "start": async_jupyter_beg,
                    "end": async_jupyter_end,
                    "preconditions": {
                        "buffer_name": async_jupyter_observation.get("buffer_name"),
                        "text_owner_name": async_jupyter_observation.get("text_owner_name"),
                        "chars_tick": async_jupyter_observation.get("chars_tick"),
                        "expected_text": async_jupyter_selected_code,
                    },
                }
            ],
        },
    }
    async_finish_response = request_or_outcome_unknown(
        args.socket,
        async_finish_request,
        timeout=5.0,
    )
    records.append(
        scenario_record(
            28,
            "daemon-package-async-jupyter-magit",
            "A daemon-owned Jupyter eval request keeps independent status while a queued Magit package command completes first",
            True,
            args.out_dir,
            jupyter_fixture=async_jupyter_fixture,
            jupyter_terminal_switch={
                "tmux_target": async_jupyter_target,
                "cancel": async_jupyter_switch["cancel"],
                "cancel_attempts": async_jupyter_switch["cancel_attempts"],
                "cancel_interaction_response": async_jupyter_switch["cancel_interaction_response"],
                "prefix": async_jupyter_switch["prefix"],
                "minibuffer_attempts": async_jupyter_switch["minibuffer_attempts"],
                "buffer_keys": async_jupyter_switch["buffer_keys"],
                "enter": async_jupyter_switch["enter"],
                "observation_attempts": async_jupyter_switch["observation_attempts"],
                "response": async_jupyter_switch["response"],
            },
            jupyter_observe_request=async_jupyter_observe_request,
            jupyter_observe_response=async_jupyter_observe_response,
            jupyter_eval_preconditions=async_jupyter_eval_preconditions,
            magit_fixture=async_magit_fixture,
            magit_initial_state=async_magit_initial_state,
            magit_terminal_switch={
                "tmux_target": async_magit_target,
                "response": async_magit_switch["response"],
                "terminal_frames": async_magit_switch["terminal_frames"],
            },
            magit_sections_request=async_magit_sections_request,
            magit_sections_response=async_magit_sections_response,
            magit_goto_request=async_magit_goto_request,
            magit_goto_response=async_magit_goto_response,
            magit_ready_sections_request=async_magit_ready_sections_request,
            magit_ready_sections_response=async_magit_ready_sections_response,
            jupyter_eval_request=async_jupyter_eval_request,
            jupyter_eval_response=async_jupyter_eval_response,
            jupyter_initial_status_request=async_jupyter_initial_status_request,
            jupyter_initial_status_response=async_jupyter_initial_status_response,
            magit_dispatch_request=async_magit_dispatch_request,
            magit_dispatch_response=async_magit_dispatch_response,
            magit_status_request=async_magit_status_request,
            magit_status_response=async_magit_status_response,
            magit_await_request=async_magit_await_request,
            magit_await_response=async_magit_await_response,
            magit_final_status=async_magit_await_status,
            after_magit_state=async_after_magit_state,
            jupyter_mid_status_request=async_jupyter_mid_status_request,
            jupyter_mid_status_response=async_jupyter_mid_status_response,
            jupyter_await_request=async_jupyter_await_request,
            jupyter_await_response=async_jupyter_await_response,
            jupyter_final_status=async_jupyter_await_status,
            jupyter_stream_payload=async_jupyter_stream_payload,
            jupyter_query_request=async_jupyter_query_request,
            jupyter_query_response=async_jupyter_query_response,
            finish_request=async_finish_request,
            finish_response=async_finish_response,
            checks=[
                check("async Jupyter fixture is present", bool(async_jupyter_buffer and isinstance(async_jupyter_beg, int) and isinstance(async_jupyter_end, int)), async_jupyter_fixture),
                check("async Magit fixture is present", bool(async_magit_repo and async_magit_buffer), async_magit_fixture),
                check("async Jupyter source terminal switch succeeded", len(async_jupyter_switch["terminal_frames"]) >= 1, async_jupyter_switch.get("result", {}).get("frames")),
                check("async Jupyter source observation succeeded", async_jupyter_observe_response.get("state") == "succeeded", async_jupyter_observe_response),
                check("async Jupyter source was selected at submit observation", async_jupyter_observation.get("selected_window_buffer") == async_jupyter_buffer, async_jupyter_observation),
                check("async Jupyter source is associated with existing kernel", async_jupyter_observation.get("jupyter_client_associated") is True, async_jupyter_observation),
                check("async Jupyter selected range hash matches fixture", async_jupyter_selected_range.get("text_hash") == async_jupyter_fixture.get("selected_sha256"), {"selected_range": async_jupyter_selected_range, "fixture_sha": async_jupyter_fixture.get("selected_sha256")}),
                check("async Magit terminal switch displays status buffer", len(async_magit_switch["terminal_frames"]) >= 1, async_magit_switch.get("result", {}).get("frames")),
                check("async Magit section observation succeeded", async_magit_sections_response.get("state") == "succeeded", async_magit_sections_response),
                check("async Magit target hunk is visible", bool(async_magit_target_section) and contains(async_magit_target_section.get("text_excerpt") or "", "target-05-selected"), async_magit_target_section),
                check("async Magit target navigation succeeded", async_magit_goto_response.get("state") == "succeeded", async_magit_goto_response),
                check("async Magit selected section is intended hunk", contains(async_magit_ready_selected.get("text_excerpt") or "", "target-05-selected"), async_magit_ready_selected),
                check("async package Jupyter request is accepted as running", async_jupyter_eval_response.get("state") == "running", async_jupyter_eval_response),
                check("async package Jupyter initial status is running", async_jupyter_initial_status.get("request_id") == "daemon-package-async-jupyter-eval-1" and async_jupyter_initial_status.get("state") == "running", async_jupyter_initial_status),
                check("async package Magit request is accepted as running", async_magit_dispatch_response.get("state") == "running", async_magit_dispatch_response),
                check("async package Magit status is observable", async_magit_status_response.get("state") == "succeeded" and async_magit_running_status.get("request_id") == "daemon-package-async-magit-stage-1", async_magit_status_response),
                check("async package Magit completes before Jupyter await", async_magit_await_response.get("state") == "succeeded" and async_magit_await_status.get("state") == "succeeded", {"await": async_magit_await_response, "status": async_magit_await_status}),
                check("async package Magit ran through command-execute", async_magit_result.get("route") == "command-execute", async_magit_result),
                check("async package Magit this-command is preserved", async_magit_result.get("this_command") == "magit-stage", async_magit_result),
                check("async package Magit preconditions passed", all(item.get("passed") is True for item in async_magit_result.get("precondition_checks") or []), async_magit_result.get("precondition_checks")),
                check("async package Magit stages only intended target change", async_after_magit_state.get("cached_names") == ["staged.txt", "target.txt"] and contains(async_after_magit_state.get("target_index") or "", "target-05-selected"), async_after_magit_state),
                check("async package Magit preserves unrelated unstaged work", async_after_magit_state.get("unstaged_names") == ["debug.txt", "target.txt"] and contains(async_after_magit_state.get("target_worktree") or "", "target-25-other"), async_after_magit_state),
                check("async package Magit hook observes command identity", contains(async_after_magit_state.get("hook_log") or "", "post-stage:magit-stage"), async_after_magit_state.get("hook_log")),
                check("async package Jupyter status remains its own request after Magit await", async_jupyter_mid_status.get("request_id") == "daemon-package-async-jupyter-eval-1" and (async_jupyter_mid_status.get("result") or {}).get("command") != "magit-stage", async_jupyter_mid_status),
                check("async package Jupyter reaches terminal success after Magit", async_jupyter_await_response.get("state") == "succeeded" and async_jupyter_await_status.get("state") == "succeeded", {"await": async_jupyter_await_response, "status": async_jupyter_await_status}),
                check("async package Jupyter response reports only kernel execution effects, not editor text effects", not async_jupyter_initial_effects and isinstance(async_jupyter_final_effects, dict) and "changed_range" not in async_jupyter_final_effects and "changed_ranges" not in async_jupyter_final_effects and isinstance(async_jupyter_kernel_execution_effect, dict) and async_jupyter_kernel_execution_effect.get("request_id") == async_jupyter_result.get("request_id"), {"initial_effects": async_jupyter_initial_effects, "final_effects": async_jupyter_final_effects}),
                check("async package Jupyter uses jupyter-eval-region route", async_jupyter_result.get("route") == "jupyter-eval-region", async_jupyter_result),
                check("async package Jupyter submitted exact selected region", async_jupyter_result.get("submitted_code") == async_jupyter_selected_code, {"submitted": async_jupyter_result.get("submitted_code"), "expected": async_jupyter_selected_code}),
                check("async package Jupyter submitted code excludes training cell", async_jupyter_training_code not in str(async_jupyter_result.get("submitted_code") or ""), async_jupyter_result.get("submitted_code")),
                check("async package Jupyter stream payload reports intended kernel", async_jupyter_stream_payload.get("kernel_token") == async_jupyter_fixture.get("kernel_token"), async_jupyter_stream_payload),
                check("async package Jupyter stream payload reports selected side effect once", async_jupyter_stream_payload.get("counter") == 1, async_jupyter_stream_payload),
                check("async package Jupyter stream payload reports training cell untouched", async_jupyter_stream_payload.get("training_executions") == 0, async_jupyter_stream_payload),
                check("async package Jupyter state query succeeded", async_jupyter_query_response.get("state") == "succeeded", async_jupyter_query_response),
                check("async package Jupyter state query sees selected side effect exactly once", async_jupyter_payload.get("counter") == 1, async_jupyter_payload),
                check("async package Jupyter state query sees training cell unexecuted", async_jupyter_payload.get("training_executions") == 0, async_jupyter_payload),
                check("async package Jupyter state query executable is rootfs-owned", rootfs_owned_process_path(str(async_jupyter_payload.get("executable") or "")), async_jupyter_payload.get("executable")),
                check("async package episode finish succeeds with cross-package evidence", async_finish_response.get("state") == "succeeded", async_finish_response),
            ],
        )
    )

    comint_interrupt_fixture = (ready or {}).get("comint_interrupt_fixture") or {}
    comint_interrupt_buffer = comint_interrupt_fixture.get("buffer_name") or ""
    comint_interrupt_counter = comint_interrupt_fixture.get("counter") or ""
    comint_interrupt_command_log = comint_interrupt_fixture.get("command_log") or ""
    comint_interrupt_command = comint_interrupt_fixture.get("command") or ""
    comint_interrupt_start_needle = str(comint_interrupt_fixture.get("start_needle") or "SOMA_DAEMON_COMINT_INTERRUPT_START")
    comint_interrupt_done_needle = str(comint_interrupt_fixture.get("done_needle") or "SOMA_DAEMON_COMINT_INTERRUPT_DONE")
    comint_interrupt_target = f"{args.terminal_tmux_session}:0.0"
    if comint_interrupt_buffer and comint_interrupt_counter and comint_interrupt_command:
        comint_interrupt_switch = switch_terminal_frame_to_buffer(
            args.socket,
            args.terminal_tmux_socket,
            comint_interrupt_target,
            comint_interrupt_buffer,
            "daemon-comint-interrupt",
        )
        comint_interrupt_initial_request = {
            "request_id": "daemon-comint-interrupt-observe-1",
            "operation": "comint.observe",
            "params": {"buffer_name": comint_interrupt_buffer},
        }
        comint_interrupt_initial_response = request_or_outcome_unknown(
            args.socket,
            comint_interrupt_initial_request,
            timeout=2.0,
        )
        comint_interrupt_initial_result = comint_interrupt_initial_response.get("result") or {}
        comint_interrupt_process_mark = comint_interrupt_initial_result.get("process_mark")
        if isinstance(comint_interrupt_process_mark, int):
            comint_interrupt_goto_request = {
                "request_id": "daemon-comint-interrupt-goto-1",
                "operation": "emacs.goto_char",
                "params": {
                    "buffer_name": comint_interrupt_buffer,
                    "point": comint_interrupt_process_mark,
                    "preconditions": comint_interrupt_initial_result,
                },
            }
            comint_interrupt_goto_response = request_or_outcome_unknown(
                args.socket,
                comint_interrupt_goto_request,
                timeout=2.0,
            )
        else:
            comint_interrupt_goto_request = {
                "request_id": "daemon-comint-interrupt-goto-1",
                "operation": "emacs.goto_char",
                "params": {
                    "buffer_name": comint_interrupt_buffer,
                    "point": None,
                    "preconditions": comint_interrupt_initial_result,
                },
            }
            comint_interrupt_goto_response = {
                "request_id": "daemon-comint-interrupt-goto-1",
                "operation": "emacs.goto_char",
                "state": "failed",
                "state_sequence": ["accepted", "failed"],
                "result": None,
                "error_code": "missing_process_mark",
                "error": "Comint interrupt fixture did not provide a process mark",
            }
        comint_interrupt_ready_request = {
            "request_id": "daemon-comint-interrupt-ready-1",
            "operation": "comint.observe",
            "params": {"buffer_name": comint_interrupt_buffer},
        }
        comint_interrupt_ready_response = request_or_outcome_unknown(
            args.socket,
            comint_interrupt_ready_request,
            timeout=2.0,
        )
        comint_interrupt_ready_result = comint_interrupt_ready_response.get("result") or {}
        comint_interrupt_input_keys = run_command(
            ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-l", "-t", comint_interrupt_target, comint_interrupt_command]
        )
        comint_interrupt_submit = run_command(
            ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", comint_interrupt_target, "Enter"]
        )
        comint_interrupt_partial_wait = wait_comint_tail_needle(
            args.socket,
            comint_interrupt_buffer,
            "daemon-comint-interrupt-partial",
            comint_interrupt_start_needle,
            comint_interrupt_counter,
            timeout_seconds=3.0,
            exact_line=True,
        )
        comint_interrupt_keys = run_command(
            ["tmux", "-S", args.terminal_tmux_socket, "send-keys", "-t", comint_interrupt_target, "C-c", "C-c"]
        )
        time.sleep(0.5)
        comint_interrupt_final_request = {
            "request_id": "daemon-comint-interrupt-final-1",
            "operation": "comint.observe",
            "params": {"buffer_name": comint_interrupt_buffer},
        }
        comint_interrupt_final_response = request_or_outcome_unknown(
            args.socket,
            comint_interrupt_final_request,
            timeout=2.0,
        )
        comint_interrupt_final_result = comint_interrupt_final_response.get("result") or {}
        comint_interrupt_tail = str(comint_interrupt_final_result.get("buffer_tail") or "")
        comint_interrupt_counter_lines = file_lines(comint_interrupt_counter)
        comint_interrupt_command_log_text = read_text(comint_interrupt_command_log)
        comint_interrupt_capture_after = run_command(
            ["tmux", "-S", args.terminal_tmux_socket, "capture-pane", "-S", "-200", "-t", comint_interrupt_target, "-p"]
        )
        comint_interrupt_payload = {
            "fixture": comint_interrupt_fixture,
            "terminal_switch": {
                "tmux_target": comint_interrupt_target,
                "cancel": comint_interrupt_switch["cancel"],
                "cancel_attempts": comint_interrupt_switch["cancel_attempts"],
                "cancel_interaction_response": comint_interrupt_switch["cancel_interaction_response"],
                "prefix": comint_interrupt_switch["prefix"],
                "minibuffer_attempts": comint_interrupt_switch["minibuffer_attempts"],
                "buffer_keys": comint_interrupt_switch["buffer_keys"],
                "enter": comint_interrupt_switch["enter"],
                "observation_attempts": comint_interrupt_switch["observation_attempts"],
                "response": comint_interrupt_switch["response"],
            },
            "initial_request": comint_interrupt_initial_request,
            "initial_response": comint_interrupt_initial_response,
            "point_adjustment": {
                "process_mark": comint_interrupt_process_mark,
                "request": comint_interrupt_goto_request,
                "response": comint_interrupt_goto_response,
            },
            "ready_request": comint_interrupt_ready_request,
            "ready_response": comint_interrupt_ready_response,
            "terminal_input": comint_interrupt_command,
            "terminal_input_keys": comint_interrupt_input_keys,
            "terminal_submit": comint_interrupt_submit,
            "partial_wait": comint_interrupt_partial_wait,
            "terminal_interrupt_keys": comint_interrupt_keys,
            "final_request": comint_interrupt_final_request,
            "final_response": comint_interrupt_final_response,
            "command_log": comint_interrupt_command_log_text,
            "terminal_after_interrupt": comint_interrupt_capture_after,
        }
        comint_interrupt_checks = [
            check("Comint interrupt fixture is present", True, comint_interrupt_fixture),
            check("Comint interrupt terminal switch succeeded", len(comint_interrupt_switch["terminal_frames"]) >= 1, comint_interrupt_switch["result"].get("frames")),
            check("initial interrupt Comint observation succeeded", comint_interrupt_initial_response.get("state") == "succeeded", comint_interrupt_initial_response),
            check("initial interrupt Comint process is live", comint_interrupt_initial_result.get("process_live") is True, comint_interrupt_initial_result),
            check("initial interrupt Comint process executable is rootfs-owned", rootfs_owned_process_path(str(comint_interrupt_initial_result.get("process_exe") or "")), comint_interrupt_initial_result.get("process_exe")),
            check("checked interrupt Comint point navigation succeeded", comint_interrupt_goto_response.get("state") == "succeeded", comint_interrupt_goto_response),
            check("ready interrupt Comint observation succeeded", comint_interrupt_ready_response.get("state") == "succeeded", comint_interrupt_ready_response),
            check("ready interrupt Comint point is at or after process mark", comint_interrupt_ready_result.get("point_after_process_mark") is True, comint_interrupt_ready_result),
            check("interrupt Comint terminal input was sent", comint_interrupt_input_keys["returncode"] == 0, comint_interrupt_input_keys),
            check("interrupt Comint terminal Enter succeeded", comint_interrupt_submit["returncode"] == 0, comint_interrupt_submit),
            check("partial interrupt output was observed", has_exact_line(str((comint_interrupt_partial_wait.get("result") or {}).get("buffer_tail") or ""), comint_interrupt_start_needle), comint_interrupt_partial_wait),
            check("terminal interrupt key sequence was sent", comint_interrupt_keys["returncode"] == 0, comint_interrupt_keys),
            check("final interrupt Comint observation succeeded", comint_interrupt_final_response.get("state") == "succeeded", comint_interrupt_final_response),
            check("interrupt Comint process identity is stable", comint_interrupt_final_result.get("process_id") == comint_interrupt_initial_result.get("process_id"), {"initial": comint_interrupt_initial_result, "final": comint_interrupt_final_result}),
            check("interrupt Comint shell process survived", comint_interrupt_final_result.get("process_live") is True, comint_interrupt_final_result),
            check("final interrupt Comint process executable is rootfs-owned", rootfs_owned_process_path(str(comint_interrupt_final_result.get("process_exe") or "")), comint_interrupt_final_result.get("process_exe")),
            check("interrupt command did not reach completion marker", not has_exact_line(comint_interrupt_tail, comint_interrupt_done_needle), comint_interrupt_tail),
            check("interrupt side effect did not occur", comint_interrupt_counter_lines == [], comint_interrupt_counter_lines),
            check("Comint post-command hook saw comint-send-input", "post-command:comint-send-input" in comint_interrupt_command_log_text, comint_interrupt_command_log_text),
            check("Comint post-command hook saw comint-interrupt-subjob", "post-command:comint-interrupt-subjob" in comint_interrupt_command_log_text, comint_interrupt_command_log_text),
            check("terminal pane shows interrupt partial output", has_exact_line(comint_interrupt_capture_after["stdout"], comint_interrupt_start_needle), comint_interrupt_capture_after),
            check("terminal pane omits interrupt completion marker", not has_exact_line(comint_interrupt_capture_after["stdout"], comint_interrupt_done_needle), comint_interrupt_capture_after),
        ]
    else:
        comint_interrupt_payload = {
            "fixture": comint_interrupt_fixture,
            "terminal_switch": None,
            "initial_request": None,
            "initial_response": None,
            "point_adjustment": None,
            "ready_request": None,
            "ready_response": None,
            "terminal_input": comint_interrupt_command,
            "terminal_input_keys": None,
            "terminal_submit": None,
            "partial_wait": None,
            "terminal_interrupt_keys": None,
            "final_request": None,
            "final_response": None,
            "command_log": "",
            "terminal_after_interrupt": None,
        }
        comint_interrupt_checks = [
            check("Comint interrupt fixture is present", False, comint_interrupt_fixture)
        ]
    records.append(
        scenario_record(
            29,
            "daemon-comint-terminal-interrupt-reconcile",
            "A live daemon terminal frame interrupts a partial Comint command and reconciles process state without the side effect",
            True,
            args.out_dir,
            **comint_interrupt_payload,
            checks=comint_interrupt_checks,
        )
    )

    manifest_request = {
        "request_id": "daemon-api-list-1",
        "operation": "api.list_capabilities",
        "params": {},
    }
    manifest_response = request(args.socket, manifest_request)
    manifest = manifest_response.get("result") or {}
    manifest_profiles = manifest.get("profiles") or []
    session_entry = find_manifest_operation(manifest, "emacs.observe_session") or {}
    frames_entry = find_manifest_operation(manifest, "emacs.observe_frames") or {}
    replace_entry = find_manifest_operation(manifest, "buffer.replace_range") or {}
    apply_patch_entry = find_manifest_operation(manifest, "buffer.apply_patch") or {}
    project_observe_entry = find_manifest_operation(manifest, "project.observe") or {}
    project_search_entry = find_manifest_operation(manifest, "project.search") or {}
    completion_observe_entry = find_manifest_operation(manifest, "completion.observe") or {}
    completion_accept_entry = find_manifest_operation(manifest, "completion.accept") or {}
    comint_submit_entry = find_manifest_operation(manifest, "comint.submit_input") or {}
    comint_await_entry = find_manifest_operation(manifest, "comint.await_output") or {}
    comint_interrupt_entry = find_manifest_operation(manifest, "comint.interrupt") or {}
    magit_stage_hunk_entry = find_manifest_operation(manifest, "magit.stage_hunk") or {}
    magit_stage_file_entry = find_manifest_operation(manifest, "magit.stage_file") or {}
    dired_delete_entry = find_manifest_operation(manifest, "dired.delete_marked") or {}
    jupyter_await_entry = find_manifest_operation(manifest, "jupyter.await_request") or {}
    org_update_entry = find_manifest_operation(manifest, "org.update_heading") or {}
    org_edit_entry = find_manifest_operation(manifest, "org.edit_src_block") or {}
    org_exit_entry = find_manifest_operation(manifest, "org.exit_src_edit") or {}
    org_babel_entry = find_manifest_operation(manifest, "org.execute_babel_block") or {}
    org_parent_text_write_entry = find_manifest_operation(manifest, "org.parent_text_write") or {}
    request_respond_entry = find_manifest_operation(manifest, "request.respond") or {}
    direct_entry = find_manifest_operation(manifest, "direct-call") or {}
    elisp_eval_entry = find_manifest_operation(manifest, "elisp.eval") or {}
    process_write_entry = find_manifest_operation(manifest, "process.write") or {}
    git_index_write_entry = find_manifest_operation(manifest, "git.index_write") or {}
    filesystem_rename_entry = (
        find_manifest_operation(manifest, "filesystem.rename") or {}
    )
    jupyter_restart_entry = find_manifest_operation(manifest, "jupyter.restart_kernel") or {}
    lsp_entry = find_manifest_operation(manifest, "lsp.observe_source") or {}
    lsp_diagnostics_entry = find_manifest_operation(manifest, "lsp.observe_diagnostics") or {}
    lsp_rename_entry = find_manifest_operation(manifest, "lsp.rename_symbol") or {}
    describe_request = {
        "request_id": "daemon-api-describe-1",
        "operation": "api.describe_operation",
        "params": {"operation": "buffer.replace_range"},
    }
    describe_response = request(args.socket, describe_request)
    description = describe_response.get("result") or {}
    request_contract = description.get("request_contract") or {}
    precondition_contract = description.get("precondition_contract") or {}
    result_contract = description.get("result_contract") or {}
    records.append(
        scenario_record(
            30,
            "daemon-api-manifest-introspection",
            "The live daemon exposes the stable semantic API manifest over JSONL",
            True,
            args.out_dir,
            list_request=manifest_request,
            list_response=manifest_response,
            describe_request=describe_request,
            describe_response=describe_response,
            checks=[
                check("daemon api.list_capabilities succeeds", manifest_response.get("state") == "succeeded", manifest_response),
                check("daemon manifest uses soma-emacs-agent-v0", manifest.get("api_version") == "soma-emacs-agent-v0", manifest),
                check("daemon manifest publishes all first stable profiles", all(profile in manifest_profiles for profile in ["core-v0", "coding-v0", "terminal-v0", "analysis-v0", "reports-v0", "vcs-files-v0"]), manifest_profiles),
                check("daemon manifest exposes stable session observation", session_entry.get("profile") == "core-v0" and session_entry.get("maturity") == "stable" and session_entry.get("implemented") is True and session_entry.get("enabled") is True and "rootfs_identity" in (session_entry.get("evidence") or []) and "doom_identity" in (session_entry.get("evidence") or []), session_entry),
                check("daemon manifest exposes stable frame observation", frames_entry.get("profile") == "core-v0" and frames_entry.get("maturity") == "stable" and frames_entry.get("implemented") is True and frames_entry.get("enabled") is True and frames_entry.get("route") == "frame-read" and "frame_identity" in (frames_entry.get("evidence") or []) and "terminal_client_identity" in (frames_entry.get("evidence") or []), frames_entry),
                check("daemon manifest exposes stable project observation and search", project_observe_entry.get("profile") == "coding-v0" and project_observe_entry.get("maturity") == "stable" and project_observe_entry.get("implemented") is True and project_observe_entry.get("enabled") is True and "search_tool_identity" in (project_observe_entry.get("evidence") or []) and "rootfs_runtime" in (project_observe_entry.get("evidence") or []) and project_search_entry.get("profile") == "coding-v0" and project_search_entry.get("maturity") == "stable" and project_search_entry.get("implemented") is True and project_search_entry.get("enabled") is True and "rootfs_runtime" in (project_search_entry.get("evidence") or []), {"project.observe": project_observe_entry, "project.search": project_search_entry}),
                check("daemon manifest exposes stable completion observe and accept operations", completion_observe_entry.get("profile") == "coding-v0" and completion_observe_entry.get("maturity") == "stable" and completion_observe_entry.get("implemented") is True and completion_observe_entry.get("enabled") is True and "acceptance_preconditions" in (completion_observe_entry.get("evidence") or []) and completion_accept_entry.get("profile") == "coding-v0" and completion_accept_entry.get("maturity") == "stable" and completion_accept_entry.get("implemented") is True and completion_accept_entry.get("enabled") is True, {"completion.observe": completion_observe_entry, "completion.accept": completion_accept_entry}),
                check("daemon manifest exposes stable Comint submit, await, and interrupt operations", comint_submit_entry.get("profile") == "terminal-v0" and comint_submit_entry.get("maturity") == "stable" and comint_submit_entry.get("implemented") is True and comint_submit_entry.get("enabled") is True and comint_submit_entry.get("route") == "comint-send-input" and comint_await_entry.get("profile") == "terminal-v0" and comint_await_entry.get("maturity") == "stable" and comint_await_entry.get("implemented") is True and comint_await_entry.get("enabled") is True and comint_interrupt_entry.get("profile") == "terminal-v0" and comint_interrupt_entry.get("maturity") == "stable" and comint_interrupt_entry.get("implemented") is True and comint_interrupt_entry.get("enabled") is True and comint_interrupt_entry.get("route") == "comint-interrupt-subjob", {"comint.submit_input": comint_submit_entry, "comint.await_output": comint_await_entry, "comint.interrupt": comint_interrupt_entry}),
                check("daemon manifest exposes stable semantic Magit staging operations", magit_stage_hunk_entry.get("profile") == "vcs-files-v0" and magit_stage_hunk_entry.get("maturity") == "stable" and magit_stage_hunk_entry.get("implemented") is True and magit_stage_hunk_entry.get("enabled") is True and magit_stage_hunk_entry.get("route") == "magit-stage" and magit_stage_file_entry.get("profile") == "vcs-files-v0" and magit_stage_file_entry.get("maturity") == "stable" and magit_stage_file_entry.get("implemented") is True and magit_stage_file_entry.get("enabled") is True and magit_stage_file_entry.get("route") == "magit-stage", {"magit.stage_hunk": magit_stage_hunk_entry, "magit.stage_file": magit_stage_file_entry}),
                check("daemon manifest exposes stable Dired marked-delete operation", dired_delete_entry.get("profile") == "vcs-files-v0" and dired_delete_entry.get("maturity") == "stable" and dired_delete_entry.get("implemented") is True and dired_delete_entry.get("enabled") is True and dired_delete_entry.get("route") == "dired-do-delete" and "pre_filesystem_state" in (dired_delete_entry.get("evidence") or []) and "post_filesystem_state" in (dired_delete_entry.get("evidence") or []), dired_delete_entry),
                check("daemon manifest exposes stable semantic Jupyter await", jupyter_await_entry.get("profile") == "analysis-v0" and jupyter_await_entry.get("maturity") == "stable" and jupyter_await_entry.get("implemented") is True and jupyter_await_entry.get("enabled") is True and jupyter_await_entry.get("route") == "jupyter-request-await", jupyter_await_entry),
                check("daemon manifest exposes stable Org heading update", org_update_entry.get("profile") == "reports-v0" and org_update_entry.get("maturity") == "stable" and org_update_entry.get("implemented") is True and org_update_entry.get("enabled") is True and org_update_entry.get("route") == "org-update-heading", org_update_entry),
                check("daemon manifest exposes stable Org source-edit operations", org_edit_entry.get("profile") == "reports-v0" and org_edit_entry.get("maturity") == "stable" and org_edit_entry.get("implemented") is True and org_edit_entry.get("enabled") is True and org_edit_entry.get("route") == "org-edit-src-code" and org_exit_entry.get("profile") == "reports-v0" and org_exit_entry.get("maturity") == "stable" and org_exit_entry.get("implemented") is True and org_exit_entry.get("enabled") is True and org_exit_entry.get("route") == "org-edit-src-exit", {"org.edit_src_block": org_edit_entry, "org.exit_src_edit": org_exit_entry}),
                check("daemon manifest exposes stable Org Babel execution", org_babel_entry.get("profile") == "reports-v0" and org_babel_entry.get("maturity") == "stable" and org_babel_entry.get("implemented") is True and org_babel_entry.get("enabled") is True and org_babel_entry.get("route") == "org-babel-execute-src-block" and "runtime_identity" in (org_babel_entry.get("evidence") or []), org_babel_entry),
                check("daemon manifest keeps direct Org parent-buffer text writes out of stable API", bool(org_parent_text_write_entry) and org_parent_text_write_entry.get("maturity") != "stable" and org_parent_text_write_entry.get("route") == "rejection-only", org_parent_text_write_entry),
                check("daemon manifest keeps request.respond experimental until prompt-owner semantics are proven", bool(request_respond_entry) and request_respond_entry.get("profile") == "core-v0" and request_respond_entry.get("maturity") == "experimental" and request_respond_entry.get("route") == "request-continuation", request_respond_entry),
                check("daemon manifest classifies buffer.replace_range as stable coding edit", replace_entry.get("profile") == "coding-v0" and replace_entry.get("maturity") == "stable" and replace_entry.get("side_effect_class") == "reversible_editor_edit", replace_entry),
                check("daemon manifest exposes stable buffer.apply_patch", apply_patch_entry.get("profile") == "coding-v0" and apply_patch_entry.get("maturity") == "stable" and apply_patch_entry.get("implemented") is True and apply_patch_entry.get("enabled") is True and apply_patch_entry.get("route") == "checked-buffer-patch" and "conflict_preview" in (apply_patch_entry.get("evidence") or []) and "changed_ranges" in (apply_patch_entry.get("evidence") or []), apply_patch_entry),
                check("daemon manifest keeps direct-call out of stable API", bool(direct_entry) and direct_entry.get("maturity") != "stable", direct_entry),
                check("daemon manifest keeps arbitrary Elisp eval out of stable API", bool(elisp_eval_entry) and elisp_eval_entry.get("maturity") != "stable" and elisp_eval_entry.get("route") == "rejection-only", elisp_eval_entry),
                check("daemon manifest keeps raw process writes out of stable API", bool(process_write_entry) and process_write_entry.get("maturity") != "stable" and process_write_entry.get("route") == "rejection-only", process_write_entry),
                check("daemon manifest keeps raw Git index writes out of stable API", bool(git_index_write_entry) and git_index_write_entry.get("maturity") != "stable" and git_index_write_entry.get("route") == "rejection-only", git_index_write_entry),
                check("daemon manifest keeps raw filesystem renames out of stable API", bool(filesystem_rename_entry) and filesystem_rename_entry.get("maturity") != "stable" and filesystem_rename_entry.get("route") == "rejection-only", filesystem_rename_entry),
                check("daemon manifest keeps direct Jupyter restarts out of stable API", bool(jupyter_restart_entry) and jupyter_restart_entry.get("maturity") != "stable" and jupyter_restart_entry.get("route") == "rejection-only", jupyter_restart_entry),
                check("daemon manifest classifies lsp.observe_source as active diagnostic", lsp_entry.get("observation_class") == "active-diagnostic", lsp_entry),
                check("daemon manifest exposes stable lsp.observe_diagnostics", lsp_diagnostics_entry.get("profile") == "coding-v0" and lsp_diagnostics_entry.get("maturity") == "stable" and lsp_diagnostics_entry.get("implemented") is True and lsp_diagnostics_entry.get("enabled") is True and "diagnostic_cache" in (lsp_diagnostics_entry.get("evidence") or []), lsp_diagnostics_entry),
                check("daemon manifest exposes stable lsp.rename_symbol workspace-edit operation", lsp_rename_entry.get("profile") == "coding-v0" and lsp_rename_entry.get("maturity") == "stable" and lsp_rename_entry.get("implemented") is True and lsp_rename_entry.get("enabled") is True and lsp_rename_entry.get("route") == "lsp-rename" and "lsp_request" in (lsp_rename_entry.get("evidence") or []) and "workspace_edit" in (lsp_rename_entry.get("evidence") or []), lsp_rename_entry),
                check("daemon api.describe_operation succeeds", describe_response.get("state") == "succeeded", describe_response),
                check("daemon operation description exposes owner, preconditions, states, and evidence", description.get("operation") == "buffer.replace_range" and description.get("owner") == "core-emacs-buffer" and "buffer_name" in (description.get("required_preconditions") or []) and "chars_tick" in (description.get("required_preconditions") or []) and "failed" in (description.get("possible_states") or []) and "before_after_observations" in (description.get("evidence") or []), description),
                check("daemon operation description exposes machine-readable envelope contracts", request_contract.get("envelope") == "soma-emacs-agent-v0.request" and request_contract.get("capability") == "reversible_editor_edit" and "buffer_name" in (request_contract.get("required_params") or []) and precondition_contract.get("mode") == "typed-object" and "chars_tick" in (precondition_contract.get("required_fields") or []) and result_contract.get("envelope") == "soma-emacs-agent-v0.result" and "state_sequence" in (result_contract.get("required_fields") or []) and "failed" in (result_contract.get("states") or []) and "diagnostics" in (result_contract.get("required_fields") or []) and "before_after_observations" in (result_contract.get("evidence_channels") or []), description),
            ],
        )
    )

    session_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-session-observe-1",
        "operation": "emacs.observe_session",
        "capability": "passive_observation",
        "params": {},
    }
    session_observe_response = request(args.socket, session_observe_request)
    session_observe_result = session_observe_response.get("result") or {}
    session_rootfs = session_observe_result.get("rootfs") or {}
    session_editor = session_observe_result.get("editor") or {}
    session_doom = session_observe_result.get("doom") or {}
    session_selected = session_observe_result.get("selected_frame") or {}
    session_info = session_observe_result.get("session") or {}
    session_evidence = session_observe_response.get("evidence") or {}
    session_observations = session_observe_response.get("observations") or {}
    session_effects = session_observe_response.get("effects") or {}
    session_editor_identity = session_evidence.get("editor_session") or {}
    session_rootfs_identity = session_evidence.get("rootfs_identity") or {}
    session_doom_identity = session_evidence.get("doom_identity") or {}
    session_selected_frame = session_observations.get("selected_frame") or {}
    expected_rootfs_store_id = os.path.basename(os.path.normpath(str((plan or {}).get("rootfs") or "")))
    records.append(
        scenario_record(
            31,
            "daemon-observe-session",
            "The live daemon reports stable core-v0 session, rootfs, Doom, editor, and frame identity",
            True,
            args.out_dir,
            request=session_observe_request,
            response=session_observe_response,
            checks=[
                check("daemon emacs.observe_session succeeds through the v0 envelope", session_observe_response.get("api_version") == "soma-emacs-agent-v0" and session_observe_response.get("state") == "succeeded" and session_observe_response.get("state_sequence") == ["accepted", "running", "succeeded"], session_observe_response),
                check("daemon emacs.observe_session publishes typed session, rootfs, Doom, and frame sections", session_editor_identity.get("api_version") == "soma-emacs-agent-v0" and "GNU Emacs 30.2" in session_editor_identity.get("version", "") and session_rootfs_identity.get("mode") == "selected-container-rootfs" and session_doom_identity.get("doom_dir") == "/project/home/.doom.d/" and isinstance(session_selected_frame.get("buffer_name"), str) and session_effects == {}, {"evidence": session_evidence, "observations": session_observations, "effects": session_effects}),
                check("daemon session observation echoes run id and selected rootfs mode", session_observe_result.get("operation") == "emacs.observe_session" and session_rootfs.get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "") and session_rootfs.get("mode") == "selected-container-rootfs" and session_rootfs.get("dependency_closure") == "rootfs_owned" and session_rootfs.get("glibc_owner") == "container-image-rootfs", session_rootfs),
                check("daemon session observation reports selected Insula plan identity", session_rootfs.get("plan_path") == os.environ.get("SOMA_INSULA_PLAN", "") and session_rootfs.get("rootfs_path") == (plan or {}).get("rootfs") and session_rootfs.get("rootfs_store_id") == expected_rootfs_store_id, session_rootfs),
                check("daemon session observation reports source-built Emacs identity", isinstance(session_editor.get("version"), str) and "GNU Emacs 30.2" in session_editor.get("version", "") and session_editor.get("executable") == "/opt/soma/emacs/bin/emacs" and session_editor.get("source_directory") == "/opt/soma/build/emacs-30.2/", session_editor),
                check("daemon session observation reports rootfs-owned Doom identity", session_doom.get("emacs_dir") == "/project/home/.emacs.d/" and session_doom.get("doom_dir") == "/project/home/.doom.d/" and isinstance(session_doom.get("commit"), str) and len(session_doom.get("commit", "")) == 40 and session_doom.get("emacs_dir_exists") is True and session_doom.get("doom_dir_exists") is True, session_doom),
                check("daemon session observation reports selected frame and buffer context", isinstance(session_selected.get("frame_id"), int) and isinstance(session_selected.get("window_id"), int) and isinstance(session_selected.get("buffer_name"), str) and session_info.get("api_version") == "soma-emacs-agent-v0", {"session": session_info, "selected": session_selected}),
            ],
        )
    )

    plan_tool_inventory = (plan or {}).get("tool_inventory") or {}
    plan_rg = plan_tool_inventory.get("rg") or {}
    with tempfile.TemporaryDirectory(prefix="soma-daemon-project-api-") as project_tmp:
        project_root = pathlib.Path(project_tmp)
        source_dir = project_root / "src"
        source_dir.mkdir(parents=True, exist_ok=True)
        subject_file = source_dir / "subject.py"
        readme_file = project_root / "README.md"
        subject_file.write_text(
            "def daemon_project_probe():\n"
            "    return 'soma-daemon-project-needle'\n",
            encoding="utf-8",
        )
        readme_file.write_text("# Soma daemon project API fixture\n", encoding="utf-8")
        git_init = run_git(str(project_root), "init", "-q")
        git_add = run_git(str(project_root), "add", "src/subject.py", "README.md")
        project_observe_request = {
            "api_version": "soma-emacs-agent-v0",
            "request_id": "daemon-project-observe-1",
            "operation": "project.observe",
            "capability": "passive_observation",
            "params": {"root": str(project_root)},
        }
        project_observe_response = request(args.socket, project_observe_request)
        project_observe_result = project_observe_response.get("result") or {}
        project_observe_evidence = project_observe_response.get("evidence") or {}
        project_observe_observations = project_observe_response.get("observations") or {}
        project_observe_effects = project_observe_response.get("effects") or {}
        project_observe_root = project_observe_evidence.get("project_root") or {}
        project_observe_vcs = project_observe_evidence.get("vcs_identity") or {}
        project_observe_search_tool = project_observe_evidence.get("search_tool_identity") or {}
        project_observe_runtime = project_observe_evidence.get("rootfs_runtime") or {}
        project_observe_context = project_observe_observations.get("project_context") or {}
        project_search_request = {
            "api_version": "soma-emacs-agent-v0",
            "request_id": "daemon-project-search-1",
            "operation": "project.search",
            "capability": "passive_observation",
            "params": {
                "root": str(project_root),
                "query": "soma-daemon-project-needle",
                "max_results": 10,
                "max_text": 200,
            },
        }
        project_search_response = request(args.socket, project_search_request)
        project_search_result = project_search_response.get("result") or {}
        project_search_evidence = project_search_response.get("evidence") or {}
        project_search_observations = project_search_response.get("observations") or {}
        project_search_effects = project_search_response.get("effects") or {}
        project_search_tool = project_search_evidence.get("search_tool_identity") or {}
        project_search_bounds = project_search_evidence.get("bounded_results") or {}
        project_search_runtime = project_search_evidence.get("rootfs_runtime") or {}
        project_search_matches = project_search_observations.get("matches") or {}
        project_search_results = project_search_result.get("results") or []
        project_search_typed_results = project_search_matches.get("results") or []
        first_project_match = (
            project_search_results[0]
            if isinstance(project_search_results, list) and project_search_results
            else {}
        )
        daemon_project_submatches = (
            first_project_match.get("submatches")
            if isinstance(first_project_match, dict)
            else None
        )
        daemon_project_first_submatch = (
            daemon_project_submatches[0]
            if isinstance(daemon_project_submatches, list)
            and daemon_project_submatches
            else {}
        )
        daemon_project_submatch_text = (
            (daemon_project_first_submatch.get("match") or {}).get("text")
            if isinstance(daemon_project_first_submatch, dict)
            else ""
        )
        records.append(
            scenario_record(
                32,
                "daemon-project-api",
                "The live daemon observes Git project identity and searches through rootfs-owned ripgrep",
                True,
                args.out_dir,
                git_init=git_init,
                git_add=git_add,
                observe_request=project_observe_request,
                observe_response=project_observe_response,
                search_request=project_search_request,
                search_response=project_search_response,
                checks=[
                    check("daemon project Git fixture initialized", git_init.get("returncode") == 0 and git_add.get("returncode") == 0, {"git_init": git_init, "git_add": git_add}),
                    check("daemon project.observe succeeds through the v0 envelope", project_observe_response.get("state") == "succeeded", project_observe_response),
                    check("daemon project.observe reports Git project identity", project_observe_result.get("project_root") == str(project_root.resolve()) + "/" and (project_observe_result.get("vcs") or {}).get("type") == "git", project_observe_result),
                    check("daemon project.observe reports selected-rootfs search tool", (project_observe_result.get("search_tool") or {}).get("path") == plan_rg.get("path") and (project_observe_result.get("search_tool") or {}).get("rootfs_owned") is True, {"result": project_observe_result.get("search_tool"), "plan": plan_rg}),
                    check("daemon project.observe publishes typed project context sections", project_observe_root.get("path") == str(project_root.resolve()) + "/" and project_observe_vcs.get("type") == "git" and project_observe_search_tool.get("path") == plan_rg.get("path") and project_observe_runtime.get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "") and project_observe_context.get("project_root") == str(project_root.resolve()) + "/" and project_observe_effects == {}, {"evidence": project_observe_evidence, "observations": project_observe_observations, "effects": project_observe_effects}),
                    check("daemon project.search succeeds through rootfs-owned ripgrep", project_search_response.get("state") == "succeeded" and (project_search_result.get("search_tool") or {}).get("path") == plan_rg.get("path") and (project_search_result.get("search_tool") or {}).get("rootfs_owned") is True, project_search_result),
                    check("daemon project.search publishes typed search evidence and match observations", project_search_tool.get("path") == plan_rg.get("path") and project_search_bounds.get("result_count") == 1 and project_search_bounds.get("results_returned") == 1 and project_search_bounds.get("results_truncated") is False and project_search_runtime.get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "") and project_search_matches.get("query") == "soma-daemon-project-needle" and isinstance(project_search_typed_results, list) and len(project_search_typed_results) == 1 and project_search_typed_results[0].get("path") == str(subject_file.resolve()) and project_search_effects == {}, {"evidence": project_search_evidence, "observations": project_search_observations, "effects": project_search_effects}),
                    check("daemon project.search returns bounded source match evidence", project_search_result.get("result_count") == 1 and first_project_match.get("path") == str(subject_file.resolve()) and first_project_match.get("line") == 2 and contains(str(first_project_match.get("text") or ""), "soma-daemon-project-needle"), project_search_result),
                    check("daemon project.search returns structured submatch evidence", isinstance(daemon_project_submatches, list) and daemon_project_submatch_text == "soma-daemon-project-needle" and daemon_project_first_submatch.get("start") == 12 and daemon_project_first_submatch.get("end") == 38, project_search_result),
                ],
            )
        )

    project_host_fixture = (ready or {}).get("project_host_shadow_fixture") or {}
    project_host_rg_buffer = str(project_host_fixture.get("rg_buffer_name") or "")
    project_host_git_buffer = str(project_host_fixture.get("git_buffer_name") or "")
    project_host_rg_marker = pathlib.Path(str(project_host_fixture.get("host_rg_marker") or ""))
    project_host_git_marker = pathlib.Path(str(project_host_fixture.get("host_git_marker") or ""))
    project_host_rg_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-project-host-rg-rejected-1",
        "operation": "project.search",
        "capability": "passive_observation",
        "params": {
            "buffer_name": project_host_rg_buffer,
            "query": "daemon-host-rg-needle",
            "max_results": 10,
            "max_text": 200,
        },
    }
    project_host_rg_response = request_or_outcome_unknown(
        args.socket,
        project_host_rg_request,
        timeout=3.0,
    )
    project_host_rg_result = project_host_rg_response.get("result") or {}
    project_host_rg_tool = project_host_rg_result.get("search_tool") or {}
    project_host_git_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-project-host-git-rejected-1",
        "operation": "project.observe",
        "capability": "passive_observation",
        "params": {"buffer_name": project_host_git_buffer},
    }
    project_host_git_response = request_or_outcome_unknown(
        args.socket,
        project_host_git_request,
        timeout=3.0,
    )
    project_host_git_result = project_host_git_response.get("result") or {}
    project_host_git_vcs = project_host_git_result.get("vcs") or {}
    project_host_git_tool = project_host_git_vcs.get("tool") or {}
    records.append(
        scenario_record(
            42,
            "daemon-project-host-tool-rejected",
            "The live daemon project APIs reject host-shadow rg and git before probing or execution",
            True,
            args.out_dir,
            fixture=project_host_fixture,
            host_rg_request=project_host_rg_request,
            host_rg_response=project_host_rg_response,
            host_git_request=project_host_git_request,
            host_git_response=project_host_git_response,
            checks=[
                check("daemon project host-shadow fixture is present", bool(project_host_rg_buffer and project_host_git_buffer and project_host_rg_marker and project_host_git_marker), project_host_fixture),
                check("daemon project.search rejects executable rg outside the selected rootfs", project_host_rg_response.get("state") == "failed" and project_host_rg_result.get("error") == "rootfs-owned ripgrep is not available" and project_host_rg_tool.get("rootfs_owned") is False, project_host_rg_response),
                check("daemon project.search does not execute rejected rg for version probing", not project_host_rg_marker.exists() and not str(project_host_rg_tool.get("version") or ""), {"marker": str(project_host_rg_marker), "tool": project_host_rg_tool}),
                check("daemon project.observe does not execute rejected git for VCS probing", project_host_git_response.get("state") == "succeeded" and not project_host_git_marker.exists() and project_host_git_vcs.get("type") is False and project_host_git_tool.get("rootfs_owned") is False and not str(project_host_git_tool.get("version") or ""), project_host_git_response),
            ],
        )
    )

    semantic_magit_fixture = (ready or {}).get("semantic_magit_fixture") or {}
    semantic_magit_repo = semantic_magit_fixture.get("repo") or ""
    semantic_magit_repo_dir = (
        str(pathlib.Path(semantic_magit_repo).resolve()) + "/"
        if semantic_magit_repo
        else ""
    )
    semantic_magit_buffer = semantic_magit_fixture.get("status_buffer") or ""
    semantic_magit_hook_log = semantic_magit_fixture.get("hook_log") or ""
    semantic_hunk_initial_state = magit_fixture_repo_state(
        semantic_magit_repo,
        semantic_magit_hook_log,
    )
    semantic_hunk_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-hunk-observe-1",
        "operation": "magit.observe_sections",
        "capability": "passive_observation",
        "params": {
            "buffer_name": semantic_magit_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    semantic_hunk_observe_response = request_or_outcome_unknown(
        args.socket,
        semantic_hunk_observe_request,
        timeout=3.0,
    )
    semantic_hunk_observation = semantic_hunk_observe_response.get("result") or {}
    semantic_hunk_observe_evidence = semantic_hunk_observe_response.get("evidence") or {}
    semantic_hunk_observe_observations = semantic_hunk_observe_response.get("observations") or {}
    semantic_hunk_observe_effects = semantic_hunk_observe_response.get("effects") or {}
    semantic_hunk_repo_evidence = semantic_hunk_observe_evidence.get("repo_fingerprint") or {}
    semantic_hunk_generation = semantic_hunk_observe_evidence.get("status_generation") or {}
    semantic_hunk_selected_section = semantic_hunk_observe_evidence.get("selected_section") or {}
    semantic_hunk_sections_observation = semantic_hunk_observe_observations.get("sections") or {}
    semantic_hunk_section = find_magit_section(
        semantic_hunk_observation,
        section_type="hunk",
        needle="target-05-selected",
        ancestor_file="target.txt",
    )
    semantic_hunk_preconditions = magit_stage_preconditions(
        semantic_hunk_observation,
        semantic_hunk_section,
    )
    semantic_hunk_stage_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-stage-hunk-1",
        "operation": "magit.stage_hunk",
        "capability": "git_index_write",
        "params": {
            "buffer_name": semantic_magit_buffer,
            "section": semantic_hunk_section,
            "preconditions": semantic_hunk_preconditions,
        },
    }
    semantic_hunk_stage_response = request_or_outcome_unknown(
        args.socket,
        semantic_hunk_stage_request,
        timeout=8.0,
    )
    semantic_hunk_status_request = {
        "request_id": "daemon-semantic-magit-hunk-status-1",
        "operation": "request.status",
        "params": {"request_id": "daemon-semantic-magit-stage-hunk-1"},
    }
    semantic_hunk_status_response = request_or_outcome_unknown(
        args.socket,
        semantic_hunk_status_request,
        timeout=2.0,
    )
    semantic_hunk_status_result = semantic_hunk_status_response.get("result") or {}
    semantic_hunk_status = (
        semantic_hunk_status_result.get("status")
        or semantic_hunk_stage_response
    )
    semantic_hunk_result = semantic_hunk_status.get("result") or {}
    semantic_hunk_command_result = (
        semantic_hunk_result.get("command_dispatch")
        if isinstance(semantic_hunk_result, dict)
        else {}
    ) or {}
    semantic_hunk_final_evidence = semantic_hunk_status.get("evidence") or {}
    semantic_hunk_final_effects = semantic_hunk_status.get("effects") or {}
    semantic_hunk_dispatch_evidence = semantic_hunk_command_result
    semantic_hunk_git_effect = (
        semantic_hunk_final_effects.get("git_index_update")
        if isinstance(semantic_hunk_final_effects, dict)
        else {}
    ) or {}
    semantic_hunk_state = magit_fixture_repo_state(
        semantic_magit_repo,
        semantic_magit_hook_log,
    )

    semantic_file_fixture = (ready or {}).get("semantic_file_magit_fixture") or {}
    semantic_file_repo = semantic_file_fixture.get("repo") or ""
    semantic_file_buffer = semantic_file_fixture.get("status_buffer") or ""
    semantic_file_hook_log = semantic_file_fixture.get("hook_log") or ""
    semantic_file_initial_state = magit_fixture_repo_state(
        semantic_file_repo,
        semantic_file_hook_log,
    )
    semantic_file_initial_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-file-initial-observe-1",
        "operation": "magit.observe_sections",
        "capability": "passive_observation",
        "params": {
            "buffer_name": semantic_file_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    semantic_file_initial_observe_response = request_or_outcome_unknown(
        args.socket,
        semantic_file_initial_observe_request,
        timeout=3.0,
    )
    semantic_file_initial_observation = (
        semantic_file_initial_observe_response.get("result") or {}
    )
    semantic_file_initial_section = find_magit_section(
        semantic_file_initial_observation,
        section_type="file",
        needle="target.txt",
    )
    semantic_file_point = semantic_file_initial_section.get("start")
    if not isinstance(semantic_file_point, int):
        semantic_file_point = magit_section_point(semantic_file_initial_section)
    semantic_file_goto_request = {
        "operation": "not-required",
        "reason": "magit.stage_file resolves the live section by identity",
        "target_point": semantic_file_point,
    }
    semantic_file_goto_response = {
        "client_side": True,
        "operation": "not-required",
        "request_id": "daemon-semantic-magit-file-goto-skipped",
        "state": "succeeded",
        "state_sequence": ["succeeded"],
        "result": semantic_file_goto_request,
    }
    semantic_file_ready_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-file-ready-observe-1",
        "operation": "magit.observe_sections",
        "capability": "passive_observation",
        "params": {
            "buffer_name": semantic_file_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    semantic_file_ready_observe_response = request_or_outcome_unknown(
        args.socket,
        semantic_file_ready_observe_request,
        timeout=3.0,
    )
    semantic_file_observation = semantic_file_ready_observe_response.get("result") or {}
    semantic_file_section = find_magit_section(
        semantic_file_observation,
        section_type="file",
        needle="target.txt",
    )
    semantic_file_preconditions = magit_stage_preconditions(
        semantic_file_observation,
        semantic_file_section,
    )
    semantic_file_stage_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-stage-file-1",
        "operation": "magit.stage_file",
        "capability": "git_index_write",
        "params": {
            "buffer_name": semantic_file_buffer,
            "section": semantic_file_section,
            "preconditions": semantic_file_preconditions,
        },
    }
    semantic_file_stage_response = request_or_outcome_unknown(
        args.socket,
        semantic_file_stage_request,
        timeout=8.0,
    )
    semantic_file_result = semantic_file_stage_response.get("result") or {}
    semantic_file_command_result = (
        semantic_file_result.get("command_dispatch")
        if isinstance(semantic_file_result, dict)
        else {}
    ) or {}
    semantic_file_evidence = semantic_file_stage_response.get("evidence") or {}
    semantic_file_effects = semantic_file_stage_response.get("effects") or {}
    semantic_file_dispatch_evidence = semantic_file_command_result
    semantic_file_git_effect = (
        semantic_file_effects.get("git_index_update")
        if isinstance(semantic_file_effects, dict)
        else {}
    ) or {}
    semantic_file_state = magit_fixture_repo_state(
        semantic_file_repo,
        semantic_file_hook_log,
    )

    semantic_stale_fixture = (ready or {}).get("semantic_stale_magit_fixture") or {}
    semantic_stale_repo = semantic_stale_fixture.get("repo") or ""
    semantic_stale_buffer = semantic_stale_fixture.get("status_buffer") or ""
    semantic_stale_hook_log = semantic_stale_fixture.get("hook_log") or ""
    semantic_stale_initial_state = magit_fixture_repo_state(
        semantic_stale_repo,
        semantic_stale_hook_log,
    )
    semantic_stale_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-stale-observe-1",
        "operation": "magit.observe_sections",
        "capability": "passive_observation",
        "params": {
            "buffer_name": semantic_stale_buffer,
            "max_sections": 300,
            "max_text": 1200,
        },
    }
    semantic_stale_observe_response = request_or_outcome_unknown(
        args.socket,
        semantic_stale_observe_request,
        timeout=3.0,
    )
    semantic_stale_observation = semantic_stale_observe_response.get("result") or {}
    semantic_stale_section = find_magit_section(
        semantic_stale_observation,
        section_type="hunk",
        needle="target-05-selected",
        ancestor_file="target.txt",
    )
    semantic_stale_preconditions = magit_stage_preconditions(
        semantic_stale_observation,
        semantic_stale_section,
    )
    semantic_stale_external_content = target_content(
        selected=True,
        other=False,
        external=False,
    ).replace("target-05-selected", "target-05-external")
    if semantic_stale_repo:
        semantic_stale_external_path = pathlib.Path(semantic_stale_repo) / "target.txt"
        semantic_stale_external_path.write_text(
            semantic_stale_external_content,
            encoding="utf-8",
        )
        semantic_stale_external_write = {
            "path": str(semantic_stale_external_path),
            "content": semantic_stale_external_content,
        }
    else:
        semantic_stale_external_write = {
            "path": "",
            "content": "",
            "error": "missing semantic stale Magit repository path",
        }
    semantic_stale_after_external_state = magit_fixture_repo_state(
        semantic_stale_repo,
        semantic_stale_hook_log,
    )
    semantic_stale_stage_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-semantic-magit-stage-stale-hunk-1",
        "operation": "magit.stage_hunk",
        "capability": "git_index_write",
        "params": {
            "buffer_name": semantic_stale_buffer,
            "section": semantic_stale_section,
            "preconditions": semantic_stale_preconditions,
        },
    }
    semantic_stale_stage_response = request_or_outcome_unknown(
        args.socket,
        semantic_stale_stage_request,
        timeout=5.0,
    )
    semantic_stale_result = semantic_stale_stage_response.get("result") or {}
    semantic_stale_effects = semantic_stale_stage_response.get("effects") or {}
    semantic_stale_state = magit_fixture_repo_state(
        semantic_stale_repo,
        semantic_stale_hook_log,
    )
    records.append(
        scenario_record(
            33,
            "daemon-magit-semantic-api",
            "The live daemon exposes semantic Magit staging with checked hunk/file staging and stale-section rejection",
            True,
            args.out_dir,
            hunk_fixture=semantic_magit_fixture,
            hunk_initial_state=semantic_hunk_initial_state,
            hunk_observe_request=semantic_hunk_observe_request,
            hunk_observe_response=semantic_hunk_observe_response,
            hunk_stage_request=semantic_hunk_stage_request,
            hunk_stage_response=semantic_hunk_stage_response,
            hunk_status_request=semantic_hunk_status_request,
            hunk_status_response=semantic_hunk_status_response,
            hunk_final_status=semantic_hunk_status,
            hunk_final_evidence=semantic_hunk_final_evidence,
            hunk_final_effects=semantic_hunk_final_effects,
            hunk_repo_state=semantic_hunk_state,
            file_fixture=semantic_file_fixture,
            file_initial_state=semantic_file_initial_state,
            file_initial_observe_request=semantic_file_initial_observe_request,
            file_initial_observe_response=semantic_file_initial_observe_response,
            file_goto_request=semantic_file_goto_request,
            file_goto_response=semantic_file_goto_response,
            file_ready_observe_request=semantic_file_ready_observe_request,
            file_ready_observe_response=semantic_file_ready_observe_response,
            file_stage_request=semantic_file_stage_request,
            file_stage_response=semantic_file_stage_response,
            file_evidence=semantic_file_evidence,
            file_effects=semantic_file_effects,
            file_repo_state=semantic_file_state,
            stale_fixture=semantic_stale_fixture,
            stale_initial_state=semantic_stale_initial_state,
            stale_observe_request=semantic_stale_observe_request,
            stale_observe_response=semantic_stale_observe_response,
            stale_external_write=semantic_stale_external_write,
            stale_after_external_state=semantic_stale_after_external_state,
            stale_stage_request=semantic_stale_stage_request,
            stale_stage_response=semantic_stale_stage_response,
            stale_effects=semantic_stale_effects,
            stale_repo_state=semantic_stale_state,
            checks=[
                check("semantic Magit hunk fixture is present", bool(semantic_magit_repo and semantic_magit_buffer), semantic_magit_fixture),
                check("semantic Magit hunk observation succeeds through the v0 envelope", semantic_hunk_observe_response.get("state") == "succeeded", semantic_hunk_observe_response),
                check("semantic Magit target hunk is observed with section identity", bool(semantic_hunk_section.get("section_identity")) and semantic_hunk_section.get("type") == "hunk", semantic_hunk_section),
                check("daemon magit.observe_sections publishes typed repo, generation, selected-section, and section-list state", semantic_hunk_repo_evidence.get("magit_repo") == semantic_magit_repo_dir and semantic_hunk_generation.get("magit_status_hash") == semantic_hunk_observation.get("magit_status_hash") and semantic_hunk_selected_section.get("type") == "hunk" and semantic_hunk_sections_observation.get("sections_returned") == len(semantic_hunk_observation.get("sections") or []) and semantic_hunk_observe_effects == {}, {"expected_repo": semantic_magit_repo_dir, "evidence": semantic_hunk_observe_evidence, "observations": semantic_hunk_observe_observations, "effects": semantic_hunk_observe_effects}),
                check("magit.stage_hunk succeeds through the v0 envelope", semantic_hunk_stage_response.get("state") == "succeeded" and semantic_hunk_status.get("state") == "succeeded", {"response": semantic_hunk_stage_response, "status": semantic_hunk_status}),
                check("magit.stage_hunk uses Magit command context", semantic_hunk_result.get("route") == "magit-stage" and semantic_hunk_result.get("this_command") == "magit-stage" and semantic_hunk_command_result.get("route") == "command-execute" and semantic_hunk_command_result.get("this_command") == "magit-stage", semantic_hunk_result),
                check("magit.stage_hunk publishes typed command evidence and git-index effect", semantic_hunk_dispatch_evidence.get("route") == "command-execute" and semantic_hunk_dispatch_evidence.get("this_command") == "magit-stage" and semantic_hunk_git_effect.get("route") == "magit-stage" and semantic_hunk_git_effect.get("this_command") == "magit-stage" and semantic_hunk_git_effect.get("section_type") == "hunk" and isinstance(semantic_hunk_git_effect.get("after_repo_fingerprint"), dict), {"evidence": semantic_hunk_final_evidence, "effects": semantic_hunk_final_effects}),
                check("magit.stage_hunk preconditions pass", all(item.get("passed") is True for item in semantic_hunk_result.get("precondition_checks") or []), semantic_hunk_result.get("precondition_checks")),
                check("semantic hunk staging updates only the intended target hunk", semantic_hunk_state.get("cached_names") == ["staged.txt", "target.txt"] and semantic_hunk_state.get("unstaged_names") == ["debug.txt", "target.txt"] and contains(semantic_hunk_state.get("target_index") or "", "target-05-selected") and contains(semantic_hunk_state.get("target_index") or "", "target-25-base") and contains(semantic_hunk_state.get("target_worktree") or "", "target-25-other"), semantic_hunk_state),
                check("semantic hunk staging preserves unrelated Git state", semantic_hunk_state.get("debug_index") == "debug-base\n" and contains(semantic_hunk_state.get("debug_worktree") or "", "debug-worktree-only") and semantic_hunk_state.get("staged_index") == "staged-expected\n", semantic_hunk_state),
                check("semantic hunk post-stage hook observes magit-stage", contains(semantic_hunk_state.get("hook_log") or "", "post-stage:magit-stage"), semantic_hunk_state.get("hook_log")),
                check("semantic Magit file fixture is present", bool(semantic_file_repo and semantic_file_buffer), semantic_file_fixture),
                check("semantic Magit file observation and point selection succeed", semantic_file_initial_observe_response.get("state") == "succeeded" and semantic_file_goto_response.get("state") == "succeeded" and semantic_file_ready_observe_response.get("state") == "succeeded", {"initial": semantic_file_initial_observe_response, "goto": semantic_file_goto_response, "ready": semantic_file_ready_observe_response}),
                check("semantic Magit target file section is observed with section identity", bool(semantic_file_section.get("section_identity")) and semantic_file_section.get("type") == "file", semantic_file_section),
                check("magit.stage_file succeeds through the v0 envelope", semantic_file_stage_response.get("state") == "succeeded", semantic_file_stage_response),
                check("magit.stage_file uses Magit command context", semantic_file_result.get("route") == "magit-stage" and semantic_file_result.get("this_command") == "magit-stage" and semantic_file_command_result.get("route") == "command-execute" and semantic_file_command_result.get("this_command") == "magit-stage", semantic_file_result),
                check("magit.stage_file publishes typed command evidence and git-index effect", semantic_file_dispatch_evidence.get("route") == "command-execute" and semantic_file_dispatch_evidence.get("this_command") == "magit-stage" and semantic_file_git_effect.get("route") == "magit-stage" and semantic_file_git_effect.get("this_command") == "magit-stage" and semantic_file_git_effect.get("section_type") == "file" and isinstance(semantic_file_git_effect.get("after_repo_fingerprint"), dict), {"evidence": semantic_file_evidence, "effects": semantic_file_effects}),
                check("semantic file staging stages the target file while preserving unrelated state", semantic_file_state.get("cached_names") == ["staged.txt", "target.txt"] and semantic_file_state.get("unstaged_names") == ["debug.txt"] and semantic_file_state.get("target_index") == semantic_file_state.get("target_worktree") and contains(semantic_file_state.get("target_index") or "", "target-05-selected") and semantic_file_state.get("debug_index") == "debug-base\n" and contains(semantic_file_state.get("debug_worktree") or "", "debug-worktree-only") and semantic_file_state.get("staged_index") == "staged-expected\n", semantic_file_state),
                check("semantic file post-stage hook observes magit-stage", contains(semantic_file_state.get("hook_log") or "", "post-stage:magit-stage"), semantic_file_state.get("hook_log")),
                check("semantic stale fixture is present", bool(semantic_stale_repo and semantic_stale_buffer), semantic_stale_fixture),
                check("semantic stale hunk observation succeeds through the v0 envelope", semantic_stale_observe_response.get("state") == "succeeded" and bool(semantic_stale_section.get("section_identity")), {"response": semantic_stale_observe_response, "section": semantic_stale_section}),
                check("magit.stage_hunk rejects stale repository fingerprint before staging", semantic_stale_stage_response.get("state") == "failed" and precondition_check_passed(semantic_stale_stage_response, "Magit repository fingerprint field magit_worktree_diff_hash matches", expected=False), semantic_stale_stage_response),
                check("stale magit.stage_hunk publishes no git-index effect", semantic_stale_effects == {}, semantic_stale_effects),
                check("semantic stale rejection does not mutate the Git index", semantic_stale_state.get("cached_names") == ["staged.txt"] and semantic_stale_state.get("unstaged_names") == ["debug.txt", "target.txt"] and contains(semantic_stale_state.get("target_index") or "", "target-05-base") and contains(semantic_stale_state.get("target_worktree") or "", "target-05-external") and not contains(semantic_stale_state.get("hook_log") or "", "post-stage:magit-stage"), semantic_stale_state),
            ],
        )
    )

    git_bypass_fixture = (ready or {}).get("bypass_magit_fixture") or {}
    git_bypass_repo = git_bypass_fixture.get("repo") or ""
    git_bypass_hook_log = git_bypass_fixture.get("hook_log") or ""
    git_bypass_initial_state = magit_fixture_repo_state(
        git_bypass_repo,
        git_bypass_hook_log,
    )
    git_index_write_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-git-index-write-1",
        "operation": "git.index_write",
        "capability": "unauthorized_operation",
        "params": {
            "repo": git_bypass_repo,
            "pathspec": "target.txt",
            "mode": "stage-file",
        },
    }
    git_index_write_response = request_or_outcome_unknown(
        args.socket,
        git_index_write_request,
        timeout=5.0,
    )
    git_bypass_final_state = magit_fixture_repo_state(
        git_bypass_repo,
        git_bypass_hook_log,
    )
    git_index_write_result = git_index_write_response.get("result") or {}
    records.append(
        scenario_record(
            43,
            "daemon-git-index-write-rejected",
            "The live daemon bridge rejects raw Git index writes so Magit-owned staging cannot be bypassed",
            True,
            args.out_dir,
            fixture=git_bypass_fixture,
            initial_state=git_bypass_initial_state,
            request=git_index_write_request,
            response=git_index_write_response,
            final_state=git_bypass_final_state,
            checks=[
                check("raw Git index bypass fixture is present", bool(git_bypass_repo and git_bypass_hook_log), git_bypass_fixture),
                check("raw Git index write operation is rejected", git_index_write_response.get("state") == "failed", git_index_write_response),
                check("raw Git index write rejection uses unauthorized code", git_index_write_response.get("error_code") == "unauthorized_operation", git_index_write_response),
                check("raw Git index write result reports no Git mutation", git_index_write_result.get("used_raw_git_index_write") is False and git_index_write_result.get("staged_path_count") == 0, git_index_write_response),
                check("raw Git index write side effect did not occur", git_bypass_final_state.get("cached_names") == git_bypass_initial_state.get("cached_names") and git_bypass_final_state.get("unstaged_names") == git_bypass_initial_state.get("unstaged_names") and git_bypass_final_state.get("target_index") == git_bypass_initial_state.get("target_index") and git_bypass_final_state.get("target_worktree") == git_bypass_initial_state.get("target_worktree"), {"initial": git_bypass_initial_state, "final": git_bypass_final_state}),
                check("raw Git index write does not run Magit post-stage hooks", "post-stage:" not in str(git_bypass_final_state.get("hook_log") or ""), git_bypass_final_state.get("hook_log")),
            ],
        )
    )

    org_report_fixture = (ready or {}).get("org_report_fixture") or {}
    org_report_buffer = org_report_fixture.get("buffer_name") or ""
    org_report_tmp = org_report_fixture.get("tmp") or ""
    org_report_publish_dir = org_report_fixture.get("publish_dir") or ""
    org_report_export_file = org_report_fixture.get("export_file") or ""
    org_heading_bypass_fixture = (ready or {}).get("org_heading_bypass_fixture") or {}
    org_heading_bypass_buffer = org_heading_bypass_fixture.get("buffer_name") or ""
    org_duplicate_id_fixture = (ready or {}).get("org_duplicate_id_fixture") or {}
    org_duplicate_id_buffer = org_duplicate_id_fixture.get("buffer_name") or ""
    org_duplicate_id_value = org_duplicate_id_fixture.get("duplicate_id") or ""
    org_subtree_bypass_fixture = (ready or {}).get("org_subtree_bypass_fixture") or {}
    org_subtree_bypass_buffer = org_subtree_bypass_fixture.get("buffer_name") or ""
    org_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-observe-1",
        "operation": "org.observe_tree",
        "capability": "passive_observation",
        "params": {
            "buffer_name": org_report_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 800,
        },
    }
    org_observe_response = request_or_outcome_unknown(
        args.socket,
        org_observe_request,
        timeout=4.0,
    )
    org_observe_result = response_result(org_observe_response)
    org_heading = first_dict(org_observe_result.get("headings"))
    org_source_block = first_dict(org_observe_result.get("source_blocks"))
    org_direct_before_text = full_buffer_text_via_bridge(
        args.socket,
        org_report_buffer,
        "daemon-org-parent-text-write-before",
        timeout=4.0,
    )
    org_parent_text_write_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-org-parent-text-write-1",
        "operation": "org.parent_text_write",
        "capability": "unauthorized_operation",
        "params": {
            "buffer_name": org_report_buffer,
            "source_block": org_source_block,
            "mode": "replace-source-block-body",
            "text": '(princ "org-parent-text-write-bypass\\n")\n',
        },
    }
    org_parent_text_write_response = request_or_outcome_unknown(
        args.socket,
        org_parent_text_write_request,
        timeout=4.0,
    )
    org_direct_after_text = full_buffer_text_via_bridge(
        args.socket,
        org_report_buffer,
        "daemon-org-parent-text-write-after",
        timeout=4.0,
    )
    org_parent_text_write_result = org_parent_text_write_response.get("result") or {}
    records.append(
        scenario_record(
            44,
            "daemon-org-parent-text-write-rejected",
            "The live daemon bridge rejects direct Org parent-buffer source-block text writes so Org source-edit semantics cannot be bypassed",
            True,
            args.out_dir,
            fixture=org_report_fixture,
            observe_request=org_observe_request,
            observe_response=org_observe_response,
            before_text=org_direct_before_text,
            request=org_parent_text_write_request,
            response=org_parent_text_write_response,
            after_text=org_direct_after_text,
            checks=[
                check("Org parent-text bypass fixture is present", bool(org_report_buffer and org_source_block), {"fixture": org_report_fixture, "source_block": org_source_block}),
                check("Org parent-text write operation is rejected", org_parent_text_write_response.get("state") == "failed", org_parent_text_write_response),
                check("Org parent-text write rejection uses unauthorized code", org_parent_text_write_response.get("error_code") == "unauthorized_operation", org_parent_text_write_response),
                check("Org parent-text write result reports no parent mutation", org_parent_text_write_result.get("direct_parent_text_write_attempted") is False and org_parent_text_write_result.get("source_edit_bypassed") is False and org_parent_text_write_result.get("written_bytes") == 0, org_parent_text_write_response),
                check("Org parent-text write rejection preserves parent buffer text", org_direct_before_text.get("text") == org_direct_after_text.get("text") and not contains(org_direct_after_text.get("text", ""), "org-parent-text-write-bypass"), {"before": org_direct_before_text, "after": org_direct_after_text}),
            ],
        )
    )

    org_bypass_fixture = (ready or {}).get("org_bypass_fixture") or {}
    org_bypass_buffer = org_bypass_fixture.get("buffer_name") or ""
    org_bypass_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-org-source-block-buffer-edit-observe-1",
        "operation": "org.observe_tree",
        "capability": "passive_observation",
        "params": {
            "buffer_name": org_bypass_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    }
    org_bypass_observe_response = request_or_outcome_unknown(
        args.socket,
        org_bypass_observe_request,
        timeout=4.0,
    )
    org_bypass_observe_result = response_result(org_bypass_observe_response)
    org_bypass_source_block = first_dict(
        org_bypass_observe_result.get("source_blocks")
    )
    org_bypass_start = org_bypass_source_block.get("begin")
    org_bypass_end = org_bypass_source_block.get("end")
    org_bypass_select_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-org-source-block-buffer-edit-select-1",
        "operation": "buffer.select",
        "capability": "editor_state_change",
        "params": {
            "buffer_name": org_bypass_buffer,
            "point": org_bypass_start,
        },
    }
    org_bypass_select_response = request_or_outcome_unknown(
        args.socket,
        org_bypass_select_request,
        timeout=4.0,
    )
    org_bypass_before_text = full_buffer_text_via_bridge(
        args.socket,
        org_bypass_buffer,
        "daemon-org-source-block-buffer-edit-before",
        timeout=4.0,
    )
    org_bypass_full_text = str(org_bypass_before_text.get("text") or "")
    org_bypass_expected_text = (
        org_bypass_full_text[org_bypass_start - 1 : org_bypass_end - 1]
        if isinstance(org_bypass_start, int)
        and isinstance(org_bypass_end, int)
        and 1 <= org_bypass_start <= org_bypass_end <= len(org_bypass_full_text) + 1
        else ""
    )
    org_bypass_replacement_text = (
        "#+begin_src emacs-lisp :results output replace\n"
        '(princ "org-buffer-replace-source-bypass\\n")\n'
        "#+end_src\n"
    )
    org_bypass_replace_preconditions = dict(
        org_bypass_before_text.get("observation") or {}
    )
    org_bypass_replace_preconditions["expected_text"] = org_bypass_expected_text
    org_bypass_replace_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-org-source-block-buffer-edit-1",
        "operation": "buffer.replace_range",
        "capability": "reversible_editor_edit",
        "preconditions": org_bypass_replace_preconditions,
        "params": {
            "buffer_name": org_bypass_buffer,
            "start": org_bypass_start,
            "end": org_bypass_end,
            "text": org_bypass_replacement_text,
        },
    }
    org_bypass_replace_response = request_or_outcome_unknown(
        args.socket,
        org_bypass_replace_request,
        timeout=4.0,
    )
    org_bypass_after_text = full_buffer_text_via_bridge(
        args.socket,
        org_bypass_buffer,
        "daemon-org-source-block-buffer-edit-after",
        timeout=4.0,
    )
    records.append(
        scenario_record(
            45,
            "daemon-org-buffer-replace-source-block-rejected",
            "The live daemon bridge rejects generic buffer.replace_range edits that overlap an Org parent source block",
            True,
            args.out_dir,
            fixture=org_bypass_fixture,
            observe_request=org_bypass_observe_request,
            observe_response=org_bypass_observe_response,
            select_request=org_bypass_select_request,
            select_response=org_bypass_select_response,
            before_text=org_bypass_before_text,
            replace_request=org_bypass_replace_request,
            replace_response=org_bypass_replace_response,
            after_text=org_bypass_after_text,
            checks=[
                check("Org source-block buffer-edit bypass fixture is present", bool(org_bypass_buffer and org_bypass_source_block), {"fixture": org_bypass_fixture, "source_block": org_bypass_source_block}),
                check("Org source-block buffer-edit bypass observes a concrete parent block range", org_bypass_observe_response.get("state") == "succeeded" and isinstance(org_bypass_start, int) and isinstance(org_bypass_end, int) and contains(org_bypass_expected_text, "#+begin_src"), {"source_block": org_bypass_source_block, "expected_text": org_bypass_expected_text}),
                check("Org source-block buffer-edit bypass selects target buffer for v0 preconditions", org_bypass_select_response.get("state") == "succeeded", org_bypass_select_response),
                check("generic buffer.replace_range rejects Org parent source-block overlap", org_bypass_replace_response.get("state") == "failed" and org_bypass_replace_response.get("error_code") == "conflict" and precondition_check_passed(org_bypass_replace_response, "target range does not overlap Org source block in parent buffer", False), org_bypass_replace_response),
                check("generic buffer.replace_range publishes no changed_range effect for rejected Org source-block bypass", not ((org_bypass_replace_response.get("effects") or {}).get("changed_range")), org_bypass_replace_response),
                check("generic buffer.replace_range rejection preserves Org parent buffer text", org_bypass_before_text.get("text") == org_bypass_after_text.get("text") and not contains(org_bypass_after_text.get("text", ""), "org-buffer-replace-source-bypass"), {"before": org_bypass_before_text, "after": org_bypass_after_text}),
            ],
        )
    )
    org_duplicate_id_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-org-duplicate-id-observe-1",
        "operation": "org.observe_tree",
        "capability": "passive_observation",
        "params": {
            "buffer_name": org_duplicate_id_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    }
    org_duplicate_id_observe_response = request_or_outcome_unknown(
        args.socket,
        org_duplicate_id_observe_request,
        timeout=4.0,
    )
    org_duplicate_id_observe_result = response_result(
        org_duplicate_id_observe_response
    )
    org_duplicate_id_headings = [
        item
        for item in org_duplicate_id_observe_result.get("headings") or []
        if isinstance(item, dict)
        and item.get("id") == org_duplicate_id_value
    ]
    org_duplicate_id_heading = (
        org_duplicate_id_headings[0] if org_duplicate_id_headings else {}
    )
    org_duplicate_id_before_text = full_buffer_text_via_bridge(
        args.socket,
        org_duplicate_id_buffer,
        "daemon-org-duplicate-id-before-text",
        timeout=4.0,
    )
    org_duplicate_id_preconditions = org_preconditions(
        org_duplicate_id_observe_result,
        org_duplicate_id_heading,
    )
    org_duplicate_id_update_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-org-duplicate-id-update-heading-1",
        "operation": "org.update_heading",
        "capability": "reversible_editor_edit",
        "preconditions": org_duplicate_id_preconditions,
        "params": {
            "buffer_name": org_duplicate_id_buffer,
            "heading": org_duplicate_id_heading,
            "changes": {
                "todo": "DONE",
                "title": "Should Not Mutate Duplicate",
            },
        },
    }
    org_duplicate_id_update_response = request_or_outcome_unknown(
        args.socket,
        org_duplicate_id_update_request,
        timeout=4.0,
    )
    org_duplicate_id_effect = (
        org_duplicate_id_update_response.get("effects") or {}
    ).get("org_heading_update", {})
    org_duplicate_id_after_text = full_buffer_text_via_bridge(
        args.socket,
        org_duplicate_id_buffer,
        "daemon-org-duplicate-id-after-text",
        timeout=4.0,
    )
    records.append(
        scenario_record(
            61,
            "daemon-org-duplicate-id-rejected",
            "org.update_heading rejects duplicated Org IDs instead of choosing an ambiguous owner",
            True,
            args.out_dir,
            fixture=org_duplicate_id_fixture,
            observe_request=org_duplicate_id_observe_request,
            observe_response=org_duplicate_id_observe_response,
            request=org_duplicate_id_update_request,
            response=org_duplicate_id_update_response,
            before_text=org_duplicate_id_before_text,
            after_text=org_duplicate_id_after_text,
            checks=[
                check("Org duplicate-ID fixture exposes the same ID on multiple headings", bool(org_duplicate_id_buffer and org_duplicate_id_value and len(org_duplicate_id_headings) >= 2), {"fixture": org_duplicate_id_fixture, "headings": org_duplicate_id_headings}),
                check("org.update_heading rejects duplicate Org ID ownership", org_duplicate_id_update_response.get("state") == "failed" and org_duplicate_id_update_response.get("error_code") == "conflict" and precondition_check_passed(org_duplicate_id_update_response, "Org heading ID is unique in buffer", False), org_duplicate_id_update_response),
                check("rejected duplicate-ID update publishes no heading effect", not org_duplicate_id_effect, org_duplicate_id_update_response),
                check("rejected duplicate-ID update preserves Org buffer text", org_duplicate_id_before_text.get("text") == org_duplicate_id_after_text.get("text") and not contains(org_duplicate_id_after_text.get("text", ""), "Should Not Mutate Duplicate"), {"before": org_duplicate_id_before_text, "after": org_duplicate_id_after_text}),
            ],
        )
    )
    org_heading_bypass_observe_request = v0_request(
        "daemon-org-heading-text-bypass-observe-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_heading_bypass_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    )
    org_heading_bypass_observe_response = request_or_outcome_unknown(
        args.socket,
        org_heading_bypass_observe_request,
        timeout=4.0,
    )
    org_heading_bypass_observe_result = response_result(
        org_heading_bypass_observe_response
    )
    org_heading_bypass_heading = first_dict(
        org_heading_bypass_observe_result.get("headings")
    )
    org_heading_bypass_before_text = full_buffer_text_via_bridge(
        args.socket,
        org_heading_bypass_buffer,
        "daemon-org-heading-text-bypass-before",
        timeout=4.0,
    )
    org_heading_bypass_original_text = str(
        org_heading_bypass_before_text.get("text") or ""
    )
    org_heading_bypass_old_line = "* TODO Summary\n"
    org_heading_bypass_new_line = "* DONE Daemon Updated Summary\n"
    org_heading_bypass_index = org_heading_bypass_original_text.find(
        org_heading_bypass_old_line
    )
    if org_heading_bypass_index >= 0:
        org_heading_bypass_start = org_heading_bypass_index + 1
        org_heading_bypass_end = org_heading_bypass_start + len(
            org_heading_bypass_old_line
        )
        org_heading_bypass_mutated_text = (
            org_heading_bypass_original_text[:org_heading_bypass_index]
            + org_heading_bypass_new_line
            + org_heading_bypass_original_text[
                org_heading_bypass_index + len(org_heading_bypass_old_line) :
            ]
        )
        org_heading_bypass_edit = replace_range_via_bridge(
            args.socket,
            org_heading_bypass_buffer,
            org_heading_bypass_start,
            org_heading_bypass_end,
            org_heading_bypass_new_line,
            org_heading_bypass_old_line,
            "daemon-org-heading-text-bypass",
            timeout=4.0,
        )
    else:
        org_heading_bypass_start = None
        org_heading_bypass_end = None
        org_heading_bypass_mutated_text = org_heading_bypass_original_text
        org_heading_bypass_edit = {
            "buffer_name": org_heading_bypass_buffer,
            "start": None,
            "end": None,
            "expected_text": org_heading_bypass_old_line,
            "replacement_text": org_heading_bypass_new_line,
            "observe_request": None,
            "observe_response": {},
            "replace_request": None,
            "replace_response": {
                "state": "failed",
                "error_code": "target_text_missing",
                "error": "Org heading text-bypass source line was not found",
            },
        }
    org_heading_bypass_after_text = full_buffer_text_via_bridge(
        args.socket,
        org_heading_bypass_buffer,
        "daemon-org-heading-text-bypass-after",
        timeout=4.0,
    )
    org_heading_bypass_final_text = str(
        org_heading_bypass_after_text.get("text") or ""
    )
    org_heading_bypass_finish_request = {
        "request_id": "daemon-org-heading-text-bypass-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-org-heading-text-bypass-update-heading-1",
                    "state": "succeeded",
                }
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": org_heading_bypass_buffer,
                    "start": 1,
                    "end": len(org_heading_bypass_mutated_text) + 1,
                    "preconditions": {
                        "expected_text": org_heading_bypass_mutated_text,
                    },
                }
            ],
        },
    }
    org_heading_bypass_finish_response = request_or_outcome_unknown(
        args.socket,
        org_heading_bypass_finish_request,
        timeout=4.0,
    )
    org_heading_bypass_finish_result = response_result(
        org_heading_bypass_finish_response
    )
    org_heading_bypass_request_checks = (
        org_heading_bypass_finish_result.get("request_checks") or []
    )
    org_heading_bypass_buffer_checks = (
        org_heading_bypass_finish_result.get("buffer_checks") or []
    )
    org_heading_bypass_replace_response = (
        org_heading_bypass_edit.get("replace_response") or {}
    )
    org_heading_bypass_replace_effects = (
        org_heading_bypass_replace_response.get("effects") or {}
    )
    records.append(
        scenario_record(
            62,
            "daemon-org-heading-text-bypass-rejected",
            "episode.finish rejects direct Org heading text edits when semantic org.update_heading evidence is required",
            True,
            args.out_dir,
            fixture=org_heading_bypass_fixture,
            observe_request=org_heading_bypass_observe_request,
            observe_response=org_heading_bypass_observe_response,
            before_text=org_heading_bypass_before_text,
            text_edit=org_heading_bypass_edit,
            after_text=org_heading_bypass_after_text,
            finish_request=org_heading_bypass_finish_request,
            finish_response=org_heading_bypass_finish_response,
            checks=[
                check("Org heading text-bypass fixture is present", bool(org_heading_bypass_buffer and org_heading_bypass_heading and org_heading_bypass_index >= 0), {"fixture": org_heading_bypass_fixture, "heading": org_heading_bypass_heading, "index": org_heading_bypass_index}),
                check("Org heading text-bypass checked buffer edit succeeds", org_heading_bypass_replace_response.get("state") == "succeeded" and bool(org_heading_bypass_replace_effects.get("changed_range")), org_heading_bypass_replace_response),
                check("Org heading text-bypass final text matches direct visible edit", org_heading_bypass_final_text == org_heading_bypass_mutated_text and contains(org_heading_bypass_final_text, org_heading_bypass_new_line.strip()), {"expected": org_heading_bypass_mutated_text, "actual": org_heading_bypass_final_text}),
                check("Org heading text-bypass edit publishes no semantic heading effect", not org_heading_bypass_replace_effects.get("org_heading_update"), org_heading_bypass_replace_response),
                check("Org heading text-bypass episode finish fails without org.update_heading evidence", org_heading_bypass_finish_response.get("state") == "failed" and org_heading_bypass_finish_result.get("request_checks_passed") is False, org_heading_bypass_finish_response),
                check("Org heading text-bypass missing org.update_heading request id is reported", any(item.get("request_id") == "daemon-org-heading-text-bypass-update-heading-1" and item.get("passed") is False and item.get("detail") == "request id is not known" for item in org_heading_bypass_request_checks), org_heading_bypass_request_checks),
                check("Org heading text-bypass final buffer checks pass independently", org_heading_bypass_finish_result.get("buffer_checks_passed") is True and any(item.get("buffer_name") == org_heading_bypass_buffer and item.get("passed") is True for item in org_heading_bypass_buffer_checks), org_heading_bypass_buffer_checks),
            ],
        )
    )
    org_subtree_bypass_observe_request = v0_request(
        "daemon-org-subtree-text-bypass-observe-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_subtree_bypass_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1600,
        },
    )
    org_subtree_bypass_observe_response = request_or_outcome_unknown(
        args.socket,
        org_subtree_bypass_observe_request,
        timeout=4.0,
    )
    org_subtree_bypass_observe_result = response_result(
        org_subtree_bypass_observe_response
    )
    org_subtree_bypass_before_text = full_buffer_text_via_bridge(
        args.socket,
        org_subtree_bypass_buffer,
        "daemon-org-subtree-text-bypass-before",
        timeout=4.0,
    )
    org_subtree_bypass_original_text = str(
        org_subtree_bypass_before_text.get("text") or ""
    )
    org_subtree_bypass_moved_subtree = (
        "** TODO Draft Section\n"
        ":PROPERTIES:\n"
        ":ID: daemon-subtree-draft\n"
        ":END:\n"
        "Draft body.\n\n"
    )
    org_subtree_bypass_target_parent = (
        "* TODO Published\n"
        ":PROPERTIES:\n"
        ":ID: daemon-subtree-published\n"
        ":END:\n"
        "Published body.\n"
    )
    org_subtree_bypass_has_plan = (
        org_subtree_bypass_moved_subtree in org_subtree_bypass_original_text
        and org_subtree_bypass_target_parent in org_subtree_bypass_original_text
    )
    if org_subtree_bypass_has_plan:
        org_subtree_bypass_mutated_text = org_subtree_bypass_original_text.replace(
            org_subtree_bypass_moved_subtree,
            "",
            1,
        ).replace(
            org_subtree_bypass_target_parent,
            org_subtree_bypass_target_parent + org_subtree_bypass_moved_subtree,
            1,
        )
        org_subtree_bypass_edit = replace_range_via_bridge(
            args.socket,
            org_subtree_bypass_buffer,
            1,
            len(org_subtree_bypass_original_text) + 1,
            org_subtree_bypass_mutated_text,
            org_subtree_bypass_original_text,
            "daemon-org-subtree-text-bypass",
            timeout=4.0,
        )
    else:
        org_subtree_bypass_mutated_text = org_subtree_bypass_original_text
        org_subtree_bypass_edit = {
            "buffer_name": org_subtree_bypass_buffer,
            "start": None,
            "end": None,
            "expected_text": org_subtree_bypass_original_text,
            "replacement_text": org_subtree_bypass_mutated_text,
            "observe_request": None,
            "observe_response": {},
            "replace_request": None,
            "replace_response": {
                "state": "failed",
                "error_code": "target_text_missing",
                "error": "Org subtree text-bypass source blocks were not found",
            },
        }
    org_subtree_bypass_after_text = full_buffer_text_via_bridge(
        args.socket,
        org_subtree_bypass_buffer,
        "daemon-org-subtree-text-bypass-after",
        timeout=4.0,
    )
    org_subtree_bypass_final_text = str(
        org_subtree_bypass_after_text.get("text") or ""
    )
    org_subtree_bypass_finish_request = {
        "request_id": "daemon-org-subtree-text-bypass-finish-1",
        "operation": "episode.finish",
        "params": {
            "required_requests": [
                {
                    "request_id": "daemon-org-subtree-reparent-semantic-1",
                    "state": "succeeded",
                }
            ],
            "final_buffer_checks": [
                {
                    "buffer_name": org_subtree_bypass_buffer,
                    "start": 1,
                    "end": len(org_subtree_bypass_mutated_text) + 1,
                    "preconditions": {
                        "expected_text": org_subtree_bypass_mutated_text,
                    },
                }
            ],
        },
    }
    org_subtree_bypass_finish_response = request_or_outcome_unknown(
        args.socket,
        org_subtree_bypass_finish_request,
        timeout=4.0,
    )
    org_subtree_bypass_finish_result = response_result(
        org_subtree_bypass_finish_response
    )
    org_subtree_bypass_request_checks = (
        org_subtree_bypass_finish_result.get("request_checks") or []
    )
    org_subtree_bypass_buffer_checks = (
        org_subtree_bypass_finish_result.get("buffer_checks") or []
    )
    org_subtree_bypass_replace_response = (
        org_subtree_bypass_edit.get("replace_response") or {}
    )
    org_subtree_bypass_replace_effects = (
        org_subtree_bypass_replace_response.get("effects") or {}
    )
    org_subtree_bypass_published_index = org_subtree_bypass_final_text.find(
        "* TODO Published"
    )
    org_subtree_bypass_draft_index = org_subtree_bypass_final_text.find(
        "** TODO Draft Section"
    )
    records.append(
        scenario_record(
            63,
            "daemon-org-subtree-text-bypass-rejected",
            "episode.finish rejects direct Org subtree text reparenting when semantic subtree evidence is required",
            True,
            args.out_dir,
            fixture=org_subtree_bypass_fixture,
            observe_request=org_subtree_bypass_observe_request,
            observe_response=org_subtree_bypass_observe_response,
            before_text=org_subtree_bypass_before_text,
            text_edit=org_subtree_bypass_edit,
            after_text=org_subtree_bypass_after_text,
            finish_request=org_subtree_bypass_finish_request,
            finish_response=org_subtree_bypass_finish_response,
            checks=[
                check("Org subtree text-bypass fixture is present", bool(org_subtree_bypass_buffer and org_subtree_bypass_observe_response.get("state") == "succeeded" and len(org_subtree_bypass_observe_result.get("headings") or []) >= 3), {"fixture": org_subtree_bypass_fixture, "observation": org_subtree_bypass_observe_response}),
                check("Org subtree text-bypass has a valid direct reparent plan", org_subtree_bypass_has_plan and org_subtree_bypass_mutated_text != org_subtree_bypass_original_text, {"original": org_subtree_bypass_original_text, "mutated": org_subtree_bypass_mutated_text}),
                check("Org subtree text-bypass checked buffer edit succeeds", org_subtree_bypass_replace_response.get("state") == "succeeded" and bool(org_subtree_bypass_replace_effects.get("changed_range")), org_subtree_bypass_replace_response),
                check("Org subtree text-bypass final text matches direct reparenting", org_subtree_bypass_final_text == org_subtree_bypass_mutated_text and 0 <= org_subtree_bypass_published_index < org_subtree_bypass_draft_index, {"expected": org_subtree_bypass_mutated_text, "actual": org_subtree_bypass_final_text, "published_index": org_subtree_bypass_published_index, "draft_index": org_subtree_bypass_draft_index}),
                check("Org subtree text-bypass edit publishes no semantic Org structure effect", not org_subtree_bypass_replace_effects.get("org_heading_update") and not org_subtree_bypass_replace_effects.get("org_subtree_reparent") and not org_subtree_bypass_replace_effects.get("parent_buffer_writeback"), org_subtree_bypass_replace_response),
                check("Org subtree text-bypass episode finish fails without semantic subtree evidence", org_subtree_bypass_finish_response.get("state") == "failed" and org_subtree_bypass_finish_result.get("request_checks_passed") is False, org_subtree_bypass_finish_response),
                check("Org subtree text-bypass missing semantic subtree request id is reported", any(item.get("request_id") == "daemon-org-subtree-reparent-semantic-1" and item.get("passed") is False and item.get("detail") == "request id is not known" for item in org_subtree_bypass_request_checks), org_subtree_bypass_request_checks),
                check("Org subtree text-bypass final buffer checks pass independently", org_subtree_bypass_finish_result.get("buffer_checks_passed") is True and any(item.get("buffer_name") == org_subtree_bypass_buffer and item.get("passed") is True for item in org_subtree_bypass_buffer_checks), org_subtree_bypass_buffer_checks),
            ],
        )
    )
    org_update_preconditions = org_preconditions(org_observe_result, org_heading)
    org_update_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-update-heading-1",
        "operation": "org.update_heading",
        "capability": "reversible_editor_edit",
        "preconditions": org_update_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "heading": org_heading,
            "changes": {
                "todo": "DONE",
                "title": "Daemon Updated Summary",
            },
        },
    }
    org_update_response = request_or_outcome_unknown(
        args.socket,
        org_update_request,
        timeout=4.0,
    )
    org_update_result = response_result(org_update_response)
    org_update_effect = (org_update_response.get("effects") or {}).get(
        "org_heading_update",
        {},
    )
    org_after_update_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-after-update-observe-1",
        "operation": "org.observe_tree",
        "capability": "passive_observation",
        "params": {
            "buffer_name": org_report_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 800,
        },
    }
    org_after_update_response = request_or_outcome_unknown(
        args.socket,
        org_after_update_request,
        timeout=4.0,
    )
    org_after_update_result = response_result(org_after_update_response)
    org_after_heading = first_dict(org_after_update_result.get("headings"))
    org_after_source_block = first_dict(org_after_update_result.get("source_blocks"))
    org_edit_preconditions = org_preconditions(
        org_after_update_result,
        org_after_heading,
        org_after_source_block,
    )
    org_edit_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-edit-src-block-1",
        "operation": "org.edit_src_block",
        "capability": "editor_state_change",
        "preconditions": org_edit_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "source_block": org_after_source_block,
        },
    }
    org_edit_response = request_or_outcome_unknown(
        args.socket,
        org_edit_request,
        timeout=4.0,
    )
    org_edit_result = response_result(org_edit_response)
    org_edit_effect = (org_edit_response.get("effects") or {}).get(
        "source_edit_buffer",
        {},
    )
    org_edit_buffer = str(org_edit_result.get("edit_buffer_name") or "")
    org_edit_buffer_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-edit-buffer-observe-1",
        "operation": "emacs.observe_buffer",
        "capability": "passive_observation",
        "params": {"buffer_name": org_edit_buffer},
    }
    org_edit_buffer_observe_response = request_or_outcome_unknown(
        args.socket,
        org_edit_buffer_observe_request,
        timeout=4.0,
    )
    org_edit_buffer_observation = response_result(org_edit_buffer_observe_response)
    org_edit_buffer_text_response, org_edit_buffer_text = buffer_text_via_bridge(
        args.socket,
        org_edit_buffer,
        "daemon-reports-org-edit-buffer-read-1",
        int(org_edit_buffer_observation.get("point_max") or 1),
        timeout=4.0,
    )
    org_source_edit_text = '(princ "daemon-source-edit\\n")\n'
    org_edit_buffer_replace_preconditions = dict(org_edit_buffer_observation)
    org_edit_buffer_replace_preconditions["expected_text"] = org_edit_buffer_text
    org_edit_replace_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-edit-buffer-replace-1",
        "operation": "buffer.replace_range",
        "capability": "reversible_editor_edit",
        "preconditions": org_edit_buffer_replace_preconditions,
        "params": {
            "buffer_name": org_edit_buffer,
            "start": org_edit_buffer_observation.get("point_min") or 1,
            "end": org_edit_buffer_observation.get("point_max") or 1,
            "text": org_source_edit_text,
        },
    }
    org_edit_replace_response = request_or_outcome_unknown(
        args.socket,
        org_edit_replace_request,
        timeout=4.0,
    )
    org_edit_replace_result = response_result(org_edit_replace_response)
    org_exit_preconditions = {
        "edit_buffer_name": org_edit_buffer,
        "edit_buffer_chars_tick": (
            (org_edit_replace_result.get("after") or {}).get("chars_tick")
        ),
        "edit_text_sha256": sha256_text(org_source_edit_text),
        "parent_buffer_name": org_report_buffer,
        "parent_text_owner_name": org_after_update_result.get("text_owner_name"),
        "parent_file_name": org_after_update_result.get("buffer_file_name"),
        "parent_chars_tick": org_after_update_result.get("chars_tick"),
        "parent_structure_fingerprint": org_after_update_result.get(
            "structure_fingerprint"
        ),
        "parent_markers": org_edit_result.get("parent_markers"),
        "source_block": org_after_source_block,
    }
    org_exit_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-exit-src-edit-1",
        "operation": "org.exit_src_edit",
        "capability": "reversible_editor_edit",
        "preconditions": org_exit_preconditions,
        "params": {"edit_buffer_name": org_edit_buffer},
    }
    org_exit_response = request_or_outcome_unknown(
        args.socket,
        org_exit_request,
        timeout=4.0,
    )
    org_exit_effect = (org_exit_response.get("effects") or {}).get(
        "parent_buffer_writeback",
        {},
    )
    org_after_exit_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-after-exit-observe-1",
        "operation": "org.observe_tree",
        "capability": "passive_observation",
        "params": {
            "buffer_name": org_report_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    }
    org_after_exit_response = request_or_outcome_unknown(
        args.socket,
        org_after_exit_request,
        timeout=4.0,
    )
    org_after_exit_result = response_result(org_after_exit_response)
    org_after_exit_heading = first_dict(org_after_exit_result.get("headings"))
    org_after_exit_source_block = first_dict(org_after_exit_result.get("source_blocks"))
    org_after_exit_text = full_buffer_text_via_bridge(
        args.socket,
        org_report_buffer,
        "daemon-reports-org-after-exit-text",
        timeout=4.0,
    )
    org_babel_preconditions = org_preconditions(
        org_after_exit_result,
        org_after_exit_heading,
        org_after_exit_source_block,
    )
    org_babel_preconditions.update(
        {
            "language_allowlist": ["emacs-lisp"],
            "default_directory": str(pathlib.Path(org_report_tmp).resolve()) + "/"
            if org_report_tmp
            else "",
            "session": False,
        }
    )
    org_babel_wrong_session_preconditions = dict(org_babel_preconditions)
    org_babel_wrong_session_preconditions["session"] = "wrong-session"
    org_babel_wrong_session_before_text = full_buffer_text_via_bridge(
        args.socket,
        org_report_buffer,
        "daemon-reports-org-wrong-session-before-text",
        timeout=4.0,
    )
    org_babel_wrong_session_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-execute-babel-wrong-session-1",
        "operation": "org.execute_babel_block",
        "capability": "external_process_or_kernel_execution",
        "preconditions": org_babel_wrong_session_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "source_block": org_after_exit_source_block,
        },
    }
    org_babel_wrong_session_response = request_or_outcome_unknown(
        args.socket,
        org_babel_wrong_session_request,
        timeout=8.0,
    )
    org_babel_wrong_session_effect = (
        org_babel_wrong_session_response.get("effects") or {}
    ).get("org_babel_execution", {})
    org_babel_wrong_session_after_text = full_buffer_text_via_bridge(
        args.socket,
        org_report_buffer,
        "daemon-reports-org-wrong-session-after-text",
        timeout=4.0,
    )
    org_babel_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-execute-babel-1",
        "operation": "org.execute_babel_block",
        "capability": "external_process_or_kernel_execution",
        "preconditions": org_babel_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "source_block": org_after_exit_source_block,
        },
    }
    org_babel_response = request_or_outcome_unknown(
        args.socket,
        org_babel_request,
        timeout=8.0,
    )
    org_babel_effect = (org_babel_response.get("effects") or {}).get(
        "org_babel_execution",
        {},
    )
    org_after_babel_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-after-babel-observe-1",
        "operation": "org.observe_tree",
        "capability": "passive_observation",
        "params": {
            "buffer_name": org_report_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    }
    org_after_babel_response = request_or_outcome_unknown(
        args.socket,
        org_after_babel_request,
        timeout=4.0,
    )
    org_after_babel_result = response_result(org_after_babel_response)
    org_after_babel_text = full_buffer_text_via_bridge(
        args.socket,
        org_report_buffer,
        "daemon-reports-org-after-babel-text",
        timeout=4.0,
    )
    org_export_preconditions = {
        "buffer_name": org_report_buffer,
        "chars_tick": org_after_babel_result.get("chars_tick"),
        "publish_directory": org_report_publish_dir,
        "export_options": {},
    }
    org_report_preview_file = str(
        pathlib.Path(org_report_publish_dir) / "report-preview.html"
    )
    org_export_preview_before = file_signature(org_report_preview_file)
    org_export_preview_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-export-preview-1",
        "operation": "org.export_html",
        "capability": "filesystem_write",
        "preconditions": org_export_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "output_path": org_report_preview_file,
            "mode": "preview",
        },
    }
    org_export_preview_response = request_or_outcome_unknown(
        args.socket,
        org_export_preview_request,
        timeout=8.0,
    )
    org_export_preview_result = response_result(org_export_preview_response)
    org_export_preview_effect = (
        org_export_preview_response.get("effects") or {}
    ).get("html_export", {})
    org_export_preview_after = file_signature(org_report_preview_file)
    org_export_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-export-html-1",
        "operation": "org.export_html",
        "capability": "filesystem_write",
        "preconditions": org_export_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "output_path": org_report_export_file,
            "mode": "apply",
        },
    }
    org_export_response = request_or_outcome_unknown(
        args.socket,
        org_export_request,
        timeout=8.0,
    )
    org_export_effect = (org_export_response.get("effects") or {}).get(
        "html_export",
        {},
    )
    org_export_text = read_text(org_report_export_file)
    org_export_escape_dir = pathlib.Path(org_report_tmp) / "outside-export"
    org_export_escape_dir.mkdir(parents=True, exist_ok=True)
    org_export_escape_link = pathlib.Path(org_report_publish_dir) / "escape-link"
    org_export_escape_output = org_export_escape_link / "escaped.html"
    org_export_escape_real_output = org_export_escape_dir / "escaped.html"
    if org_export_escape_link.exists() or org_export_escape_link.is_symlink():
        org_export_escape_link.unlink()
    org_export_escape_link.symlink_to(org_export_escape_dir, target_is_directory=True)
    org_export_escape_preconditions = {
        "buffer_name": org_report_buffer,
        "chars_tick": org_after_babel_result.get("chars_tick"),
        "publish_directory": org_report_publish_dir,
        "export_options": {},
    }
    org_export_escape_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-export-symlink-escape-1",
        "operation": "org.export_html",
        "capability": "filesystem_write",
        "preconditions": org_export_escape_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "output_path": str(org_export_escape_output),
        },
    }
    org_export_escape_response = request_or_outcome_unknown(
        args.socket,
        org_export_escape_request,
        timeout=8.0,
    )
    org_export_escape_effect = (
        org_export_escape_response.get("effects") or {}
    ).get("html_export", {})
    org_export_escape_signature = file_signature(str(org_export_escape_output))
    org_export_escape_real_signature = file_signature(str(org_export_escape_real_output))
    org_export_direct_escape_output = (
        pathlib.Path(org_report_publish_dir) / ".." / "direct-escaped.html"
    )
    org_export_direct_escape_real_output = (
        pathlib.Path(org_report_publish_dir).resolve().parent / "direct-escaped.html"
    )
    org_export_direct_escape_preconditions = {
        "buffer_name": org_report_buffer,
        "chars_tick": org_after_babel_result.get("chars_tick"),
        "publish_directory": org_report_publish_dir,
        "export_options": {},
    }
    org_export_direct_escape_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-reports-org-export-direct-escape-1",
        "operation": "org.export_html",
        "capability": "filesystem_write",
        "preconditions": org_export_direct_escape_preconditions,
        "params": {
            "buffer_name": org_report_buffer,
            "output_path": str(org_export_direct_escape_output),
        },
    }
    org_export_direct_escape_response = request_or_outcome_unknown(
        args.socket,
        org_export_direct_escape_request,
        timeout=8.0,
    )
    org_export_direct_escape_effect = (
        org_export_direct_escape_response.get("effects") or {}
    ).get("html_export", {})
    org_export_direct_escape_signature = file_signature(
        str(org_export_direct_escape_output)
    )
    org_export_direct_escape_real_signature = file_signature(
        str(org_export_direct_escape_real_output)
    )

    dired_rename_fixture = (ready or {}).get("dired_rename_fixture") or {}
    dired_rename_buffer = dired_rename_fixture.get("buffer_name") or ""
    dired_rename_tmp = dired_rename_fixture.get("tmp") or ""
    dired_rename_source = dired_rename_fixture.get("source") or ""
    dired_rename_target = dired_rename_fixture.get("target") or ""
    dired_rename_source_path = dired_rename_fixture.get("source_path") or ""
    dired_rename_target_path = dired_rename_fixture.get("target_path") or ""
    dired_rename_decoy_path = dired_rename_fixture.get("decoy_path") or ""
    dired_rename_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-rename-observe-1",
        "operation": "dired.observe",
        "capability": "passive_observation",
        "params": {"buffer_name": dired_rename_buffer, "max_files": 50},
    }
    dired_rename_observe_response = request_or_outcome_unknown(
        args.socket,
        dired_rename_observe_request,
        timeout=4.0,
    )
    dired_rename_preconditions = {
        "buffer_name": dired_rename_buffer,
        "directory": str(pathlib.Path(dired_rename_tmp).resolve()) + "/"
        if dired_rename_tmp
        else "",
        "file_signature": file_signature(dired_rename_source_path),
        "target_absent": True,
        "collision_policy": "no_overwrite",
    }
    dired_rename_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-rename-no-overwrite-1",
        "operation": "dired.rename_no_overwrite",
        "capability": "filesystem_write",
        "preconditions": dired_rename_preconditions,
        "params": {
            "buffer_name": dired_rename_buffer,
            "source": dired_rename_source,
            "target": dired_rename_target,
        },
    }
    dired_rename_response = request_or_outcome_unknown(
        args.socket,
        dired_rename_request,
        timeout=6.0,
    )
    dired_rename_effect = (dired_rename_response.get("effects") or {}).get(
        "filesystem_rename",
        {},
    )
    dired_collision_source = "collision-source.txt"
    dired_collision_target = "collision.txt"
    dired_collision_source_path = (
        str(pathlib.Path(dired_rename_tmp) / dired_collision_source)
        if dired_rename_tmp
        else ""
    )
    dired_collision_target_path = (
        str(pathlib.Path(dired_rename_tmp) / dired_collision_target)
        if dired_rename_tmp
        else ""
    )
    if dired_rename_tmp:
        pathlib.Path(dired_collision_source_path).write_text(
            "collision-source\n",
            encoding="utf-8",
        )
        pathlib.Path(dired_collision_target_path).write_text(
            "collision-original\n",
            encoding="utf-8",
        )
    dired_collision_source_before = file_signature(dired_collision_source_path)
    dired_collision_target_before = file_signature(dired_collision_target_path)
    dired_collision_preconditions = {
        "buffer_name": dired_rename_buffer,
        "directory": str(pathlib.Path(dired_rename_tmp).resolve()) + "/"
        if dired_rename_tmp
        else "",
        "file_signature": dired_collision_source_before,
        "target_absent": True,
        "collision_policy": "no_overwrite",
    }
    dired_collision_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-rename-collision-1",
        "operation": "dired.rename_no_overwrite",
        "capability": "filesystem_write",
        "preconditions": dired_collision_preconditions,
        "params": {
            "buffer_name": dired_rename_buffer,
            "source": dired_collision_source,
            "target": dired_collision_target,
        },
    }
    dired_collision_response = request_or_outcome_unknown(
        args.socket,
        dired_collision_request,
        timeout=6.0,
    )
    dired_collision_effect = (
        dired_collision_response.get("effects") or {}
    ).get("filesystem_rename", {})
    dired_collision_source_after = file_signature(dired_collision_source_path)
    dired_collision_target_after = file_signature(dired_collision_target_path)
    dired_unsafe_source = "unsafe-source.txt"
    dired_unsafe_target_basename = (
        f"soma-daemon-dired-unsafe-target-{os.getpid()}.txt"
    )
    dired_unsafe_target = f"../{dired_unsafe_target_basename}"
    dired_unsafe_source_path = (
        str(pathlib.Path(dired_rename_tmp) / dired_unsafe_source)
        if dired_rename_tmp
        else ""
    )
    dired_unsafe_escape_path = (
        str(pathlib.Path(dired_rename_tmp).parent / dired_unsafe_target_basename)
        if dired_rename_tmp
        else ""
    )
    if dired_rename_tmp:
        pathlib.Path(dired_unsafe_source_path).write_text(
            "unsafe-source\n",
            encoding="utf-8",
        )
    dired_unsafe_source_before = file_signature(dired_unsafe_source_path)
    dired_unsafe_escape_before = file_signature(dired_unsafe_escape_path)
    dired_unsafe_preconditions = {
        "buffer_name": dired_rename_buffer,
        "directory": str(pathlib.Path(dired_rename_tmp).resolve()) + "/"
        if dired_rename_tmp
        else "",
        "file_signature": dired_unsafe_source_before,
        "target_absent": True,
        "collision_policy": "no_overwrite",
    }
    dired_unsafe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-rename-unsafe-target-1",
        "operation": "dired.rename_no_overwrite",
        "capability": "filesystem_write",
        "preconditions": dired_unsafe_preconditions,
        "params": {
            "buffer_name": dired_rename_buffer,
            "source": dired_unsafe_source,
            "target": dired_unsafe_target,
        },
    }
    dired_unsafe_response = request_or_outcome_unknown(
        args.socket,
        dired_unsafe_request,
        timeout=6.0,
    )
    dired_unsafe_effect = (
        dired_unsafe_response.get("effects") or {}
    ).get("filesystem_rename", {})
    dired_unsafe_source_after = file_signature(dired_unsafe_source_path)
    dired_unsafe_escape_after = file_signature(dired_unsafe_escape_path)

    raw_filesystem_source = "raw-filesystem-source.txt"
    raw_filesystem_target = "raw-filesystem-target.txt"
    raw_filesystem_source_path = (
        str(pathlib.Path(dired_rename_tmp) / raw_filesystem_source)
        if dired_rename_tmp
        else ""
    )
    raw_filesystem_target_path = (
        str(pathlib.Path(dired_rename_tmp) / raw_filesystem_target)
        if dired_rename_tmp
        else ""
    )
    if dired_rename_tmp:
        pathlib.Path(raw_filesystem_source_path).write_text(
            "raw-filesystem-source-preserve\n",
            encoding="utf-8",
        )
    raw_filesystem_source_before = file_signature(raw_filesystem_source_path)
    raw_filesystem_target_before = file_signature(raw_filesystem_target_path)
    raw_filesystem_rename_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-filesystem-rename-rejected-1",
        "operation": "filesystem.rename",
        "capability": "unauthorized_operation",
        "params": {
            "source_path": raw_filesystem_source_path,
            "target_path": raw_filesystem_target_path,
        },
    }
    raw_filesystem_rename_response = request_or_outcome_unknown(
        args.socket,
        raw_filesystem_rename_request,
        timeout=4.0,
    )
    raw_filesystem_rename_result = raw_filesystem_rename_response.get("result") or {}
    raw_filesystem_rename_effect = (
        raw_filesystem_rename_response.get("effects") or {}
    ).get("filesystem_rename", {})
    raw_filesystem_source_after = file_signature(raw_filesystem_source_path)
    raw_filesystem_target_after = file_signature(raw_filesystem_target_path)
    raw_filesystem_source_after_text = read_text(raw_filesystem_source_path)

    dired_delete_fixture = (ready or {}).get("dired_delete_fixture") or {}
    dired_delete_buffer = dired_delete_fixture.get("buffer_name") or ""
    dired_delete_tmp = dired_delete_fixture.get("tmp") or ""
    dired_delete_marked = dired_delete_fixture.get("marked_files") or []
    dired_delete_a_path = dired_delete_fixture.get("delete_a_path") or ""
    dired_delete_b_path = dired_delete_fixture.get("delete_b_path") or ""
    dired_delete_flagged_decoy_path = (
        dired_delete_fixture.get("flagged_decoy_path") or ""
    )
    dired_delete_observe_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-delete-observe-1",
        "operation": "dired.observe",
        "capability": "passive_observation",
        "params": {"buffer_name": dired_delete_buffer, "max_files": 50},
    }
    dired_delete_observe_response = request_or_outcome_unknown(
        args.socket,
        dired_delete_observe_request,
        timeout=4.0,
    )
    dired_stale_delete_a_before = file_signature(dired_delete_a_path)
    dired_stale_delete_b_before = file_signature(dired_delete_b_path)
    if dired_delete_a_path:
        pathlib.Path(dired_delete_a_path).write_text(
            "delete-a-replaced-before-submit\n",
            encoding="utf-8",
        )
    dired_stale_delete_preconditions = {
        "buffer_name": dired_delete_buffer,
        "directory": str(pathlib.Path(dired_delete_tmp).resolve()) + "/"
        if dired_delete_tmp
        else "",
        "marked_files": dired_delete_marked,
        "file_signatures": [
            {
                "name": "delete-a.txt",
                "signature": dired_stale_delete_a_before,
            },
            {
                "name": "delete-b.txt",
                "signature": dired_stale_delete_b_before,
            },
        ],
        "delete_policy": {
            "selection": "marked",
            "delete_by_moving_to_trash": False,
            "recursive_deletes": False,
        },
    }
    dired_stale_delete_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-delete-stale-signature-1",
        "operation": "dired.delete_marked",
        "capability": "filesystem_write",
        "preconditions": dired_stale_delete_preconditions,
        "params": {"buffer_name": dired_delete_buffer},
    }
    dired_stale_delete_response = request_or_outcome_unknown(
        args.socket,
        dired_stale_delete_request,
        timeout=6.0,
    )
    dired_stale_delete_effect = (
        dired_stale_delete_response.get("effects") or {}
    ).get("filesystem_delete", {})
    dired_stale_delete_a_after = file_signature(dired_delete_a_path)
    dired_stale_delete_b_after = file_signature(dired_delete_b_path)
    dired_stale_delete_a_after_text = read_text(dired_delete_a_path)
    dired_unsafe_delete_marked = ["../delete-escaped.txt"]
    dired_unsafe_delete_escape_path = ""
    if dired_delete_tmp:
        dired_unsafe_delete_escape_path = str(
            pathlib.Path(dired_delete_tmp).resolve().parent / "delete-escaped.txt"
        )
        pathlib.Path(dired_unsafe_delete_escape_path).write_text(
            "delete-escaped-preserve\n",
            encoding="utf-8",
        )
    dired_unsafe_delete_escape_before = file_signature(
        dired_unsafe_delete_escape_path
    )
    dired_unsafe_delete_preconditions = {
        "buffer_name": dired_delete_buffer,
        "directory": str(pathlib.Path(dired_delete_tmp).resolve()) + "/"
        if dired_delete_tmp
        else "",
        "marked_files": dired_unsafe_delete_marked,
        "file_signatures": [
            {
                "name": "../delete-escaped.txt",
                "signature": dired_unsafe_delete_escape_before,
            },
        ],
        "delete_policy": {
            "selection": "marked",
            "delete_by_moving_to_trash": False,
            "recursive_deletes": False,
        },
    }
    dired_unsafe_delete_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-delete-unsafe-marked-1",
        "operation": "dired.delete_marked",
        "capability": "filesystem_write",
        "preconditions": dired_unsafe_delete_preconditions,
        "params": {"buffer_name": dired_delete_buffer},
    }
    dired_unsafe_delete_response = request_or_outcome_unknown(
        args.socket,
        dired_unsafe_delete_request,
        timeout=6.0,
    )
    dired_unsafe_delete_effect = (
        dired_unsafe_delete_response.get("effects") or {}
    ).get("filesystem_delete", {})
    dired_unsafe_delete_escape_after = file_signature(
        dired_unsafe_delete_escape_path
    )
    dired_unsafe_delete_escape_after_text = read_text(
        dired_unsafe_delete_escape_path
    )
    dired_delete_preconditions = {
        "buffer_name": dired_delete_buffer,
        "directory": str(pathlib.Path(dired_delete_tmp).resolve()) + "/"
        if dired_delete_tmp
        else "",
        "marked_files": dired_delete_marked,
        "file_signatures": [
            {
                "name": "delete-a.txt",
                "signature": file_signature(dired_delete_a_path),
            },
            {
                "name": "delete-b.txt",
                "signature": file_signature(dired_delete_b_path),
            },
        ],
        "delete_policy": {
            "selection": "marked",
            "delete_by_moving_to_trash": False,
            "recursive_deletes": False,
        },
    }
    dired_delete_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-files-dired-delete-marked-1",
        "operation": "dired.delete_marked",
        "capability": "filesystem_write",
        "preconditions": dired_delete_preconditions,
        "params": {"buffer_name": dired_delete_buffer},
    }
    dired_delete_response = request_or_outcome_unknown(
        args.socket,
        dired_delete_request,
        timeout=6.0,
    )
    dired_delete_effect = (dired_delete_response.get("effects") or {}).get(
        "filesystem_delete",
        {},
    )
    records.append(
        scenario_record(
            38,
            "daemon-reports-vcs-files-semantic-api",
            "The live daemon executes reports-v0 Org mutators and vcs-files-v0 Dired mutators through stable JSONL operations with typed effects",
            True,
            args.out_dir,
            org_fixture=org_report_fixture,
            org_observe_request=org_observe_request,
            org_observe_response=org_observe_response,
            org_update_request=org_update_request,
            org_update_response=org_update_response,
            org_after_update_request=org_after_update_request,
            org_after_update_response=org_after_update_response,
            org_edit_request=org_edit_request,
            org_edit_response=org_edit_response,
            org_edit_buffer_observe_request=org_edit_buffer_observe_request,
            org_edit_buffer_observe_response=org_edit_buffer_observe_response,
            org_edit_buffer_read_response=org_edit_buffer_text_response,
            org_edit_replace_request=org_edit_replace_request,
            org_edit_replace_response=org_edit_replace_response,
            org_exit_request=org_exit_request,
            org_exit_response=org_exit_response,
            org_after_exit_request=org_after_exit_request,
            org_after_exit_response=org_after_exit_response,
            org_after_exit_text=org_after_exit_text,
            org_babel_wrong_session_request=org_babel_wrong_session_request,
            org_babel_wrong_session_response=org_babel_wrong_session_response,
            org_babel_wrong_session_before_text=org_babel_wrong_session_before_text,
            org_babel_wrong_session_after_text=org_babel_wrong_session_after_text,
            org_babel_request=org_babel_request,
            org_babel_response=org_babel_response,
            org_after_babel_request=org_after_babel_request,
            org_after_babel_response=org_after_babel_response,
            org_after_babel_text=org_after_babel_text,
            org_export_preview_request=org_export_preview_request,
            org_export_preview_response=org_export_preview_response,
            org_export_preview_before=org_export_preview_before,
            org_export_preview_after=org_export_preview_after,
            org_export_request=org_export_request,
            org_export_response=org_export_response,
            org_export_text_excerpt=org_export_text[:1200],
            dired_rename_fixture=dired_rename_fixture,
            dired_rename_observe_request=dired_rename_observe_request,
            dired_rename_observe_response=dired_rename_observe_response,
            dired_rename_request=dired_rename_request,
            dired_rename_response=dired_rename_response,
            dired_delete_fixture=dired_delete_fixture,
            dired_delete_observe_request=dired_delete_observe_request,
            dired_delete_observe_response=dired_delete_observe_response,
            dired_delete_request=dired_delete_request,
            dired_delete_response=dired_delete_response,
            checks=[
                check("daemon Org report fixture is present", bool(org_report_buffer and org_report_tmp and org_report_export_file), org_report_fixture),
                check("daemon Org report observation succeeds", org_observe_response.get("state") == "succeeded" and org_heading.get("title") == "Summary" and org_source_block.get("language") == "emacs-lisp", org_observe_response),
                check("org.update_heading succeeds and publishes typed heading effect", org_update_response.get("state") == "succeeded" and org_update_effect.get("route") == "org-update-heading" and (org_update_effect.get("after_heading") or {}).get("todo") == "DONE" and (org_update_effect.get("after_heading") or {}).get("title") == "Daemon Updated Summary" and (org_update_effect.get("after_heading") or {}).get("id") == "daemon-report-summary", {"response": org_update_response, "effect": org_update_effect}),
                check("org.edit_src_block opens a typed separate source-edit buffer", org_edit_response.get("state") == "succeeded" and org_edit_effect.get("route") == "org-edit-src-code" and org_edit_effect.get("source_edit_buffer_separate") is True and isinstance(org_edit_effect.get("edit_buffer_name"), str), {"response": org_edit_response, "effect": org_edit_effect}),
                check("daemon source-edit replacement uses checked buffer editing", org_edit_replace_response.get("state") == "succeeded" and contains(org_source_edit_text, "daemon-source-edit") and (org_edit_replace_response.get("effects") or {}).get("changed_range"), org_edit_replace_response),
                check("org.exit_src_edit writes source edit back to the parent Org buffer", org_exit_response.get("state") == "succeeded" and org_exit_effect.get("route") == "org-edit-src-exit" and org_exit_effect.get("edit_buffer_live_after") is False and org_after_exit_heading.get("title") == "Daemon Updated Summary" and org_after_exit_heading.get("id") == "daemon-report-summary" and contains(org_after_exit_text.get("text", ""), "daemon-source-edit"), {"response": org_exit_response, "effect": org_exit_effect, "after": org_after_exit_response}),
                check("org.execute_babel_block rejects wrong Babel session before execution", org_babel_wrong_session_response.get("state") == "failed" and org_babel_wrong_session_response.get("error_code") == "conflict" and precondition_check_passed(org_babel_wrong_session_response, "Babel session matches precondition", False) and not org_babel_wrong_session_effect and org_babel_wrong_session_before_text.get("text") == org_babel_wrong_session_after_text.get("text") and not contains(org_babel_wrong_session_after_text.get("text", ""), "#+RESULTS:"), {"response": org_babel_wrong_session_response, "before": org_babel_wrong_session_before_text, "after": org_babel_wrong_session_after_text}),
                check("org.execute_babel_block runs selected emacs-lisp through Org Babel with rootfs runtime evidence", org_babel_response.get("state") == "succeeded" and org_babel_effect.get("route") == "org-babel-execute-src-block" and (((org_babel_effect.get("runtime_identity") or {}).get("rootfs_runtime") or {}).get("run_id") == os.environ.get("SOMA_INSULA_RUN_ID", "")) and contains(((org_babel_effect.get("parent_result_block") or {}).get("text_excerpt") or ""), "daemon-source-edit") and contains(org_after_babel_text.get("text", ""), "#+RESULTS:"), {"response": org_babel_response, "effect": org_babel_effect}),
                check("org.export_html preview validates without writing an artifact", org_export_preview_response.get("state") == "succeeded" and org_export_preview_result.get("mode") == "preview" and org_export_preview_result.get("route") == "org-export-preview" and org_export_preview_result.get("generated_file") is False and (org_export_preview_response.get("evidence") or {}).get("export_preview", {}).get("planned_output_path") == org_report_preview_file and not org_export_preview_effect and org_export_preview_before.get("exists") is False and org_export_preview_after.get("exists") is False, {"response": org_export_preview_response, "before": org_export_preview_before, "after": org_export_preview_after}),
                check("org.export_html writes the requested report artifact and publishes typed HTML effect", org_export_response.get("state") == "succeeded" and org_export_effect.get("route") == "org-export" and org_export_effect.get("generated_file") == org_report_export_file and (org_export_effect.get("output_signature") or {}).get("exists") is True and pathlib.Path(org_report_export_file).exists() and contains(org_export_text, "Daemon Updated Summary"), {"response": org_export_response, "effect": org_export_effect, "html_excerpt": org_export_text[:400]}),
                check("daemon Dired rename fixture is present", bool(dired_rename_buffer and dired_rename_tmp and dired_rename_source_path and dired_rename_target_path), dired_rename_fixture),
                check("dired.rename_no_overwrite succeeds and publishes typed filesystem rename effect", dired_rename_response.get("state") == "succeeded" and dired_rename_effect.get("route") == "dired-rename-file" and dired_rename_effect.get("source") == dired_rename_source and dired_rename_effect.get("target") == dired_rename_target and (dired_rename_effect.get("source_after") or {}).get("exists") is False and (dired_rename_effect.get("target_after") or {}).get("exists") is True and not pathlib.Path(dired_rename_source_path).exists() and pathlib.Path(dired_rename_target_path).exists() and read_text(dired_rename_target_path) == "source\n" and read_text(dired_rename_decoy_path) == "decoy\n", {"response": dired_rename_response, "effect": dired_rename_effect}),
                check("daemon Dired delete fixture is present", bool(dired_delete_buffer and dired_delete_tmp and dired_delete_marked and dired_delete_a_path and dired_delete_b_path), dired_delete_fixture),
                check("dired.delete_marked deletes checked ordinary marks and preserves deletion-flagged decoy", dired_delete_response.get("state") == "succeeded" and dired_delete_effect.get("route") == "dired-do-delete" and dired_delete_effect.get("marked_files") == ["delete-a.txt", "delete-b.txt"] and not pathlib.Path(dired_delete_a_path).exists() and not pathlib.Path(dired_delete_b_path).exists() and pathlib.Path(dired_delete_flagged_decoy_path).exists() and read_text(dired_delete_flagged_decoy_path) == "flagged-decoy\n", {"response": dired_delete_response, "effect": dired_delete_effect}),
            ],
        )
    )
    records.append(
        scenario_record(
            55,
            "daemon-dired-delete-stale-signature-rejected",
            "dired.delete_marked rejects stale marked-file signatures in the live daemon without deleting replacement files",
            True,
            args.out_dir,
            fixture=dired_delete_fixture,
            observe_request=dired_delete_observe_request,
            observe_response=dired_delete_observe_response,
            request=dired_stale_delete_request,
            response=dired_stale_delete_response,
            delete_a_before=dired_stale_delete_a_before,
            delete_b_before=dired_stale_delete_b_before,
            delete_a_after=dired_stale_delete_a_after,
            delete_b_after=dired_stale_delete_b_after,
            delete_a_after_text=dired_stale_delete_a_after_text,
            checks=[
                check("Dired stale-delete fixture is present", bool(dired_delete_buffer and dired_delete_tmp and dired_delete_marked and dired_stale_delete_a_before.get("exists") is True and dired_stale_delete_b_before.get("exists") is True), {"buffer": dired_delete_buffer, "directory": dired_delete_tmp, "marked": dired_delete_marked, "delete_a": dired_stale_delete_a_before, "delete_b": dired_stale_delete_b_before}),
                check("dired.delete_marked rejects a stale replacement file signature", dired_stale_delete_response.get("state") == "failed" and dired_stale_delete_response.get("error_code") == "conflict" and precondition_check_passed(dired_stale_delete_response, "file signature for marked file delete-a.txt matches", False), dired_stale_delete_response),
                check("rejected dired.delete_marked stale signature publishes no filesystem delete effect", not dired_stale_delete_effect, dired_stale_delete_response),
                check("rejected dired.delete_marked stale signature preserves marked files", dired_stale_delete_a_after.get("exists") is True and dired_stale_delete_b_after.get("exists") is True and dired_stale_delete_a_after.get("sha256") != dired_stale_delete_a_before.get("sha256") and dired_stale_delete_b_after.get("sha256") == dired_stale_delete_b_before.get("sha256") and dired_stale_delete_a_after_text == "delete-a-replaced-before-submit\n", {"delete_a_before": dired_stale_delete_a_before, "delete_b_before": dired_stale_delete_b_before, "delete_a_after": dired_stale_delete_a_after, "delete_b_after": dired_stale_delete_b_after, "delete_a_after_text": dired_stale_delete_a_after_text}),
            ],
        )
    )
    records.append(
        scenario_record(
            56,
            "daemon-dired-delete-unsafe-marked-rejected",
            "dired.delete_marked rejects unsafe marked-file names in the live daemon without escaping the Dired directory",
            True,
            args.out_dir,
            fixture=dired_delete_fixture,
            request=dired_unsafe_delete_request,
            response=dired_unsafe_delete_response,
            escape_before=dired_unsafe_delete_escape_before,
            escape_after=dired_unsafe_delete_escape_after,
            escape_after_text=dired_unsafe_delete_escape_after_text,
            checks=[
                check("Dired unsafe-delete fixture is present", bool(dired_delete_buffer and dired_delete_tmp and dired_unsafe_delete_escape_before.get("exists") is True), {"buffer": dired_delete_buffer, "directory": dired_delete_tmp, "escape": dired_unsafe_delete_escape_before}),
                check("dired.delete_marked rejects path traversal through basename-safety precondition", dired_unsafe_delete_response.get("state") == "failed" and dired_unsafe_delete_response.get("error_code") == "conflict" and precondition_check_passed(dired_unsafe_delete_response, "marked files are explicit safe relative basenames", False), dired_unsafe_delete_response),
                check("rejected dired.delete_marked unsafe marked name publishes no filesystem delete effect", not dired_unsafe_delete_effect, dired_unsafe_delete_response),
                check("rejected dired.delete_marked unsafe marked name preserves escaped file", dired_unsafe_delete_escape_after.get("exists") is True and dired_unsafe_delete_escape_after.get("sha256") == dired_unsafe_delete_escape_before.get("sha256") and dired_unsafe_delete_escape_after_text == "delete-escaped-preserve\n", {"escape_before": dired_unsafe_delete_escape_before, "escape_after": dired_unsafe_delete_escape_after, "escape_after_text": dired_unsafe_delete_escape_after_text}),
            ],
        )
    )
    records.append(
        scenario_record(
            52,
            "daemon-dired-rename-collision-rejected",
            "dired.rename_no_overwrite rejects an existing target in the live daemon without mutating either file",
            True,
            args.out_dir,
            fixture=dired_rename_fixture,
            request=dired_collision_request,
            response=dired_collision_response,
            source_before=dired_collision_source_before,
            target_before=dired_collision_target_before,
            source_after=dired_collision_source_after,
            target_after=dired_collision_target_after,
            checks=[
                check("Dired collision fixture is present", bool(dired_rename_buffer and dired_rename_tmp and dired_collision_source_before.get("exists") is True and dired_collision_target_before.get("exists") is True), {"buffer": dired_rename_buffer, "directory": dired_rename_tmp, "source": dired_collision_source_before, "target": dired_collision_target_before}),
                check("dired.rename_no_overwrite rejects existing target through the target_absent precondition", dired_collision_response.get("state") == "failed" and dired_collision_response.get("error_code") == "conflict" and precondition_check_passed(dired_collision_response, "target is absent for no-overwrite rename", False), dired_collision_response),
                check("rejected dired.rename_no_overwrite collision publishes no filesystem rename effect", not dired_collision_effect, dired_collision_response),
                check("rejected dired.rename_no_overwrite collision preserves both files", dired_collision_source_after.get("exists") is True and dired_collision_target_after.get("exists") is True and dired_collision_source_after.get("sha256") == dired_collision_source_before.get("sha256") and dired_collision_target_after.get("sha256") == dired_collision_target_before.get("sha256") and read_text(dired_collision_source_path) == "collision-source\n" and read_text(dired_collision_target_path) == "collision-original\n", {"source_before": dired_collision_source_before, "target_before": dired_collision_target_before, "source_after": dired_collision_source_after, "target_after": dired_collision_target_after}),
            ],
        )
    )
    records.append(
        scenario_record(
            54,
            "daemon-dired-rename-unsafe-target-rejected",
            "dired.rename_no_overwrite rejects unsafe target paths in the live daemon without escaping the Dired directory",
            True,
            args.out_dir,
            fixture=dired_rename_fixture,
            request=dired_unsafe_request,
            response=dired_unsafe_response,
            source_before=dired_unsafe_source_before,
            escape_before=dired_unsafe_escape_before,
            source_after=dired_unsafe_source_after,
            escape_after=dired_unsafe_escape_after,
            checks=[
                check("Dired unsafe-target fixture is present", bool(dired_rename_buffer and dired_rename_tmp and dired_unsafe_source_before.get("exists") is True and dired_unsafe_escape_before.get("exists") is False), {"buffer": dired_rename_buffer, "directory": dired_rename_tmp, "source": dired_unsafe_source_before, "escape": dired_unsafe_escape_before}),
                check("dired.rename_no_overwrite rejects target path traversal through basename-safety precondition", dired_unsafe_response.get("state") == "failed" and dired_unsafe_response.get("error_code") == "conflict" and precondition_check_passed(dired_unsafe_response, "source target names are safe relative basenames", False), dired_unsafe_response),
                check("rejected dired.rename_no_overwrite unsafe target publishes no filesystem rename effect", not dired_unsafe_effect, dired_unsafe_response),
                check("rejected dired.rename_no_overwrite unsafe target preserves source and creates no escaped file", dired_unsafe_source_after.get("exists") is True and dired_unsafe_source_after.get("sha256") == dired_unsafe_source_before.get("sha256") and dired_unsafe_escape_after.get("exists") is False and read_text(dired_unsafe_source_path) == "unsafe-source\n", {"source_before": dired_unsafe_source_before, "escape_before": dired_unsafe_escape_before, "source_after": dired_unsafe_source_after, "escape_after": dired_unsafe_escape_after}),
            ],
        )
    )
    records.append(
        scenario_record(
            60,
            "daemon-filesystem-rename-rejected",
            "filesystem.rename is rejected as a raw filesystem bypass and does not mutate Dired-owned files",
            True,
            args.out_dir,
            fixture=dired_rename_fixture,
            request=raw_filesystem_rename_request,
            response=raw_filesystem_rename_response,
            source_before=raw_filesystem_source_before,
            target_before=raw_filesystem_target_before,
            source_after=raw_filesystem_source_after,
            target_after=raw_filesystem_target_after,
            source_after_text=raw_filesystem_source_after_text,
            checks=[
                check("Raw filesystem rename fixture is present", bool(dired_rename_tmp and raw_filesystem_source_before.get("exists") is True and raw_filesystem_target_before.get("exists") is False), {"directory": dired_rename_tmp, "source": raw_filesystem_source_before, "target": raw_filesystem_target_before}),
                check("filesystem.rename operation is rejected", raw_filesystem_rename_response.get("state") == "failed" and raw_filesystem_rename_response.get("error_code") == "unauthorized_operation", raw_filesystem_rename_response),
                check("filesystem.rename rejection reports no raw rename", raw_filesystem_rename_result.get("route") == "rejection-only" and raw_filesystem_rename_result.get("used_raw_filesystem_rename") is False and raw_filesystem_rename_result.get("renamed_paths") == 0, raw_filesystem_rename_response),
                check("rejected filesystem.rename publishes no filesystem rename effect", not raw_filesystem_rename_effect, raw_filesystem_rename_response),
                check("rejected filesystem.rename preserves source and creates no target", raw_filesystem_source_after.get("exists") is True and raw_filesystem_source_after.get("sha256") == raw_filesystem_source_before.get("sha256") and raw_filesystem_target_after.get("exists") is False and raw_filesystem_source_after_text == "raw-filesystem-source-preserve\n", {"source_before": raw_filesystem_source_before, "target_before": raw_filesystem_target_before, "source_after": raw_filesystem_source_after, "target_after": raw_filesystem_target_after, "source_after_text": raw_filesystem_source_after_text}),
            ],
        )
    )
    records.append(
        scenario_record(
            50,
            "daemon-org-export-symlink-escape-rejected",
            "org.export_html rejects symlink-resolved output paths that escape the declared publish directory",
            True,
            args.out_dir,
            fixture=org_report_fixture,
            symlink=str(org_export_escape_link),
            symlink_target=str(org_export_escape_dir),
            request=org_export_escape_request,
            response=org_export_escape_response,
            output_signature=org_export_escape_signature,
            real_output_signature=org_export_escape_real_signature,
            checks=[
                check("Org export symlink-escape fixture is present", org_export_escape_link.is_symlink() and org_export_escape_link.resolve() == org_export_escape_dir.resolve(), {"link": str(org_export_escape_link), "target": str(org_export_escape_dir)}),
                check("org.export_html rejects symlink-resolved publish-directory escape", org_export_escape_response.get("state") == "failed" and org_export_escape_response.get("error_code") == "conflict" and precondition_check_passed(org_export_escape_response, "output path resolves inside publish directory", False), org_export_escape_response),
                check("rejected org.export_html symlink escape publishes no html_export effect", not org_export_escape_effect, org_export_escape_response),
                check("rejected org.export_html symlink escape creates no outside artifact", org_export_escape_signature.get("exists") is False and org_export_escape_real_signature.get("exists") is False, {"requested": org_export_escape_signature, "real": org_export_escape_real_signature}),
            ],
        )
    )
    records.append(
        scenario_record(
            57,
            "daemon-org-export-direct-escape-rejected",
            "org.export_html rejects direct parent-directory output paths that escape the declared publish directory",
            True,
            args.out_dir,
            fixture=org_report_fixture,
            requested_output=str(org_export_direct_escape_output),
            real_output=str(org_export_direct_escape_real_output),
            request=org_export_direct_escape_request,
            response=org_export_direct_escape_response,
            output_signature=org_export_direct_escape_signature,
            real_output_signature=org_export_direct_escape_real_signature,
            checks=[
                check("Org export direct-escape fixture is present", bool(org_report_buffer and org_report_publish_dir), {"buffer": org_report_buffer, "publish_dir": org_report_publish_dir, "requested": str(org_export_direct_escape_output), "real": str(org_export_direct_escape_real_output)}),
                check("org.export_html rejects direct publish-directory escape", org_export_direct_escape_response.get("state") == "failed" and org_export_direct_escape_response.get("error_code") == "conflict" and precondition_check_passed(org_export_direct_escape_response, "output path is inside publish directory", False), org_export_direct_escape_response),
                check("rejected org.export_html direct escape publishes no html_export effect", not org_export_direct_escape_effect, org_export_direct_escape_response),
                check("rejected org.export_html direct escape creates no outside artifact", org_export_direct_escape_signature.get("exists") is False and org_export_direct_escape_real_signature.get("exists") is False, {"requested": org_export_direct_escape_signature, "real": org_export_direct_escape_real_signature}),
            ],
        )
    )

    org_stale_initial_observe_request = v0_request(
        "daemon-org-stale-parent-initial-observe-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_bypass_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    )
    org_stale_initial_observe_response = request_or_outcome_unknown(
        args.socket,
        org_stale_initial_observe_request,
        timeout=4.0,
    )
    org_stale_initial_observe_result = response_result(
        org_stale_initial_observe_response
    )
    org_stale_initial_heading = first_dict(
        org_stale_initial_observe_result.get("headings")
    )
    org_stale_initial_source_block = first_dict(
        org_stale_initial_observe_result.get("source_blocks")
    )
    org_stale_edit_preconditions = org_preconditions(
        org_stale_initial_observe_result,
        org_stale_initial_heading,
        org_stale_initial_source_block,
    )
    org_stale_edit_request = v0_request(
        "daemon-org-stale-parent-edit-src-block-1",
        "org.edit_src_block",
        "editor_state_change",
        {
            "buffer_name": org_bypass_buffer,
            "source_block": org_stale_initial_source_block,
        },
        org_stale_edit_preconditions,
    )
    org_stale_edit_response = request_or_outcome_unknown(
        args.socket,
        org_stale_edit_request,
        timeout=4.0,
    )
    org_stale_edit_result = response_result(org_stale_edit_response)
    org_stale_edit_effect = (org_stale_edit_response.get("effects") or {}).get(
        "source_edit_buffer",
        {},
    )
    org_stale_edit_buffer = str(org_stale_edit_result.get("edit_buffer_name") or "")
    org_stale_edit_buffer_before = full_buffer_text_via_bridge(
        args.socket,
        org_stale_edit_buffer,
        "daemon-org-stale-parent-edit-buffer-before",
        timeout=4.0,
    )
    org_stale_edit_buffer_observation = (
        org_stale_edit_buffer_before.get("observation") or {}
    )
    org_stale_edit_buffer_text = str(org_stale_edit_buffer_before.get("text") or "")
    org_stale_source_edit_text = '(princ "stale-parent-should-not-write\\n")\n'
    org_stale_edit_replace_preconditions = dict(org_stale_edit_buffer_observation)
    org_stale_edit_replace_preconditions["expected_text"] = org_stale_edit_buffer_text
    org_stale_edit_replace_request = v0_request(
        "daemon-org-stale-parent-edit-buffer-replace-1",
        "buffer.replace_range",
        "reversible_editor_edit",
        {
            "buffer_name": org_stale_edit_buffer,
            "start": org_stale_edit_buffer_observation.get("point_min") or 1,
            "end": org_stale_edit_buffer_observation.get("point_max") or 1,
            "text": org_stale_source_edit_text,
        },
        org_stale_edit_replace_preconditions,
    )
    org_stale_edit_replace_response = request_or_outcome_unknown(
        args.socket,
        org_stale_edit_replace_request,
        timeout=4.0,
    )
    org_stale_edit_replace_result = response_result(org_stale_edit_replace_response)
    org_stale_parent_before_request = v0_request(
        "daemon-org-stale-parent-before-observe-1",
        "org.observe_tree",
        "passive_observation",
        {
            "buffer_name": org_bypass_buffer,
            "max_headings": 20,
            "max_source_blocks": 20,
            "max_text": 1200,
        },
    )
    org_stale_parent_before_response = request_or_outcome_unknown(
        args.socket,
        org_stale_parent_before_request,
        timeout=4.0,
    )
    org_stale_parent_before_result = response_result(
        org_stale_parent_before_response
    )
    org_stale_parent_before_heading = first_dict(
        org_stale_parent_before_result.get("headings")
    )
    org_stale_parent_before_source_block = first_dict(
        org_stale_parent_before_result.get("source_blocks")
    )
    org_stale_exit_preconditions = {
        "edit_buffer_name": org_stale_edit_buffer,
        "edit_buffer_chars_tick": (
            (org_stale_edit_replace_result.get("after") or {}).get("chars_tick")
        ),
        "edit_text_sha256": sha256_text(org_stale_source_edit_text),
        "parent_buffer_name": org_bypass_buffer,
        "parent_text_owner_name": org_stale_parent_before_result.get(
            "text_owner_name"
        ),
        "parent_file_name": org_stale_parent_before_result.get("buffer_file_name"),
        "parent_chars_tick": org_stale_parent_before_result.get("chars_tick"),
        "parent_structure_fingerprint": org_stale_parent_before_result.get(
            "structure_fingerprint"
        ),
        "parent_markers": org_stale_edit_result.get("parent_markers"),
        "source_block": org_stale_parent_before_source_block,
    }
    org_stale_parent_update_preconditions = org_preconditions(
        org_stale_parent_before_result,
        org_stale_parent_before_heading,
    )
    org_stale_parent_update_request = v0_request(
        "daemon-org-stale-parent-update-heading-1",
        "org.update_heading",
        "reversible_editor_edit",
        {
            "buffer_name": org_bypass_buffer,
            "heading": org_stale_parent_before_heading,
            "changes": {"title": "Stale Parent Changed"},
        },
        org_stale_parent_update_preconditions,
    )
    org_stale_parent_update_response = request_or_outcome_unknown(
        args.socket,
        org_stale_parent_update_request,
        timeout=4.0,
    )
    org_stale_parent_update_effect = (
        org_stale_parent_update_response.get("effects") or {}
    ).get("org_heading_update", {})
    org_stale_exit_request = v0_request(
        "daemon-org-stale-parent-exit-src-edit-1",
        "org.exit_src_edit",
        "reversible_editor_edit",
        {"edit_buffer_name": org_stale_edit_buffer},
        org_stale_exit_preconditions,
    )
    org_stale_exit_response = request_or_outcome_unknown(
        args.socket,
        org_stale_exit_request,
        timeout=4.0,
    )
    org_stale_exit_effect = (org_stale_exit_response.get("effects") or {}).get(
        "parent_buffer_writeback",
        {},
    )
    org_stale_parent_after = full_buffer_text_via_bridge(
        args.socket,
        org_bypass_buffer,
        "daemon-org-stale-parent-after",
        timeout=4.0,
    )
    org_stale_edit_buffer_after = full_buffer_text_via_bridge(
        args.socket,
        org_stale_edit_buffer,
        "daemon-org-stale-parent-edit-buffer-after",
        timeout=4.0,
    )
    org_stale_restore_selection_request = v0_request(
        "daemon-org-stale-parent-restore-selection-1",
        "buffer.select",
        "editor_state_change",
        {"buffer_name": args.buffer_name},
    )
    org_stale_restore_selection_response = request_or_outcome_unknown(
        args.socket,
        org_stale_restore_selection_request,
        timeout=4.0,
    )
    records.append(
        scenario_record(
            59,
            "daemon-org-source-edit-stale-parent-rejected",
            "org.exit_src_edit rejects stale parent Org state in the live daemon before writing source-edit text back",
            True,
            args.out_dir,
            fixture=org_bypass_fixture,
            initial_observe_request=org_stale_initial_observe_request,
            initial_observe_response=org_stale_initial_observe_response,
            edit_request=org_stale_edit_request,
            edit_response=org_stale_edit_response,
            edit_buffer_before=org_stale_edit_buffer_before,
            edit_replace_request=org_stale_edit_replace_request,
            edit_replace_response=org_stale_edit_replace_response,
            parent_before_request=org_stale_parent_before_request,
            parent_before_response=org_stale_parent_before_response,
            parent_update_request=org_stale_parent_update_request,
            parent_update_response=org_stale_parent_update_response,
            exit_request=org_stale_exit_request,
            exit_response=org_stale_exit_response,
            parent_after=org_stale_parent_after,
            edit_buffer_after=org_stale_edit_buffer_after,
            restore_selection_request=org_stale_restore_selection_request,
            restore_selection_response=org_stale_restore_selection_response,
            checks=[
                check("Org stale-parent source-edit fixture is present", org_stale_initial_observe_response.get("state") == "succeeded" and bool(org_bypass_buffer and org_stale_initial_heading and org_stale_initial_source_block), {"fixture": org_bypass_fixture, "heading": org_stale_initial_heading, "source_block": org_stale_initial_source_block, "observation": org_stale_initial_observe_response}),
                check("org.edit_src_block opens a live source-edit buffer for stale-parent coverage", org_stale_edit_response.get("state") == "succeeded" and org_stale_edit_effect.get("route") == "org-edit-src-code" and isinstance(org_stale_edit_buffer, str) and bool(org_stale_edit_buffer), {"response": org_stale_edit_response, "effect": org_stale_edit_effect}),
                check("stale-parent source-edit buffer replacement succeeds before parent mutation", org_stale_edit_replace_response.get("state") == "succeeded" and contains(org_stale_source_edit_text, "stale-parent-should-not-write") and (org_stale_edit_replace_response.get("effects") or {}).get("changed_range"), org_stale_edit_replace_response),
                check("stale-parent setup mutates the parent Org buffer after source-edit observation", org_stale_parent_update_response.get("state") == "succeeded" and (org_stale_parent_update_effect.get("after_heading") or {}).get("title") == "Stale Parent Changed", {"response": org_stale_parent_update_response, "effect": org_stale_parent_update_effect}),
                check("org.exit_src_edit rejects stale parent structure before writeback", org_stale_exit_response.get("state") == "failed" and org_stale_exit_response.get("error_code") == "conflict" and precondition_check_passed(org_stale_exit_response, "parent structure fingerprint matches precondition", False), org_stale_exit_response),
                check("rejected org.exit_src_edit publishes no parent_buffer_writeback effect", not org_stale_exit_effect, org_stale_exit_response),
                check("rejected org.exit_src_edit preserves parent mutation without copying stale source-edit text", contains(org_stale_parent_after.get("text", ""), "Stale Parent Changed") and not contains(org_stale_parent_after.get("text", ""), "stale-parent-should-not-write"), org_stale_parent_after),
                check("rejected org.exit_src_edit leaves source-edit text outside the parent document", contains(org_stale_edit_buffer_after.get("text", ""), "stale-parent-should-not-write"), org_stale_edit_buffer_after),
                check("benchmark restores primary selected buffer after stale Org rejection", org_stale_restore_selection_response.get("state") == "succeeded", org_stale_restore_selection_response),
            ],
        )
    )

    envelope_valid_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-valid-1",
        "operation": "emacs.observe_buffer",
        "capability": "passive_observation",
        "params": {"buffer_name": args.buffer_name},
    }
    envelope_valid_response = request(args.socket, envelope_valid_request)
    envelope_invalid_version_request = {
        "api_version": "soma-emacs-agent-v999",
        "request_id": "daemon-envelope-invalid-version-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": args.buffer_name},
    }
    envelope_invalid_version_response = request(args.socket, envelope_invalid_version_request)
    envelope_capability_mismatch_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-capability-mismatch-1",
        "operation": "emacs.observe_buffer",
        "capability": "kernel_execution",
        "params": {"buffer_name": args.buffer_name},
    }
    envelope_capability_mismatch_response = request(
        args.socket, envelope_capability_mismatch_request
    )
    envelope_before_observe_request = {
        "request_id": "daemon-envelope-before-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": args.buffer_name},
    }
    envelope_before_observe_response = request(args.socket, envelope_before_observe_request)
    envelope_before_observation = envelope_before_observe_response.get("result") or {}
    envelope_before_read_request = {
        "request_id": "daemon-envelope-before-read-1",
        "operation": "buffer.read",
        "params": {
            "buffer_name": args.buffer_name,
            "start": 1,
            "end": max(1, int(envelope_before_observation.get("point_max") or 1)),
        },
    }
    envelope_before_read_response = request(args.socket, envelope_before_read_request)
    envelope_before_text = (envelope_before_read_response.get("result") or {}).get("text")
    envelope_full_preconditions = {
        "buffer_name": envelope_before_observation.get("buffer_name"),
        "text_owner_name": envelope_before_observation.get("text_owner_name"),
        "buffer_file_name": envelope_before_observation.get("buffer_file_name"),
        "chars_tick": envelope_before_observation.get("chars_tick"),
        "expected_text": "",
        "point": envelope_before_observation.get("point"),
        "selected_window_buffer": envelope_before_observation.get("selected_window_buffer"),
        "modified": envelope_before_observation.get("modified"),
        "read_only": envelope_before_observation.get("read_only"),
        "minibuffer_active": envelope_before_observation.get("minibuffer_active"),
    }
    envelope_invalid_preconditions_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-invalid-preconditions-1",
        "operation": "buffer.replace_range",
        "preconditions": "not-an-object",
        "params": {
            "buffer_name": args.buffer_name,
            "start": 1,
            "end": 1,
            "text": "SOMA_DAEMON_ENVELOPE_MUTATED\n",
        },
    }
    envelope_invalid_preconditions_response = request(
        args.socket, envelope_invalid_preconditions_request
    )
    envelope_missing_param_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-missing-param-1",
        "operation": "buffer.replace_range",
        "capability": "reversible_editor_edit",
        "preconditions": envelope_full_preconditions,
        "params": {
            "buffer_name": args.buffer_name,
            "start": 1,
            "end": 1,
        },
    }
    envelope_missing_param_response = request(args.socket, envelope_missing_param_request)
    envelope_missing_precondition_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-missing-precondition-1",
        "operation": "buffer.replace_range",
        "capability": "reversible_editor_edit",
        "preconditions": {
            key: value
            for key, value in envelope_full_preconditions.items()
            if key != "chars_tick"
        },
        "params": {
            "buffer_name": args.buffer_name,
            "start": 1,
            "end": 1,
            "text": "SOMA_DAEMON_ENVELOPE_MUTATED\n",
        },
    }
    envelope_missing_precondition_response = request(
        args.socket, envelope_missing_precondition_request
    )
    envelope_benchmark_only_defer_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-benchmark-only-defer-1",
        "operation": "bridge.defer_echo",
        "capability": "async_fixture",
        "params": {"delay_seconds": 0.02, "value": "must-not-run-as-v0"},
    }
    envelope_benchmark_only_defer_response = request(
        args.socket, envelope_benchmark_only_defer_request
    )
    envelope_benchmark_only_finish_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-benchmark-only-finish-1",
        "operation": "episode.finish",
        "capability": "private_grader",
        "params": {
            "required_requests": [],
            "final_buffer_checks": [],
        },
    }
    envelope_benchmark_only_finish_response = request(
        args.socket, envelope_benchmark_only_finish_request
    )
    envelope_experimental_respond_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-experimental-respond-1",
        "operation": "request.respond",
        "capability": "continuation_response",
        "params": {
            "request_id": "daemon-prompt-1",
            "response": "must-not-run-as-v0",
        },
    }
    envelope_experimental_respond_response = request(
        args.socket, envelope_experimental_respond_request
    )
    envelope_after_observe_request = {
        "request_id": "daemon-envelope-after-observe-1",
        "operation": "emacs.observe_buffer",
        "params": {"buffer_name": args.buffer_name},
    }
    envelope_after_observe_response = request(args.socket, envelope_after_observe_request)
    envelope_after_observation = envelope_after_observe_response.get("result") or {}
    envelope_after_read_request = {
        "request_id": "daemon-envelope-after-read-1",
        "operation": "buffer.read",
        "params": {
            "buffer_name": args.buffer_name,
            "start": 1,
            "end": max(1, int(envelope_after_observation.get("point_max") or 1)),
        },
    }
    envelope_after_read_response = request(args.socket, envelope_after_read_request)
    envelope_after_text = (envelope_after_read_response.get("result") or {}).get("text")
    envelope_defer_request = {
        "request_id": "daemon-envelope-defer-1",
        "operation": "bridge.defer_echo",
        "params": {"delay_seconds": 0.02, "value": "daemon-timeout-ms-ok"},
    }
    envelope_defer_response = request(args.socket, envelope_defer_request)
    envelope_await_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-await-1",
        "operation": "request.await",
        "capability": "passive_observation",
        "timeout_ms": 1000,
        "params": {"request_id": "daemon-envelope-defer-1"},
    }
    envelope_await_response = request(args.socket, envelope_await_request)
    envelope_await_status = (
        (envelope_await_response.get("result") or {}).get("status") or {}
    )
    envelope_invalid_timeout_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-envelope-invalid-timeout-1",
        "operation": "request.await",
        "timeout_ms": -1,
        "params": {"request_id": "daemon-envelope-defer-1"},
    }
    envelope_invalid_timeout_response = request(args.socket, envelope_invalid_timeout_request)
    records.append(
        scenario_record(
            34,
            "daemon-request-envelope-validation",
            "The live daemon JSONL bridge validates the v0 request envelope before executing operations",
            True,
            args.out_dir,
            valid_request=envelope_valid_request,
            valid_response=envelope_valid_response,
            invalid_version_request=envelope_invalid_version_request,
            invalid_version_response=envelope_invalid_version_response,
            capability_mismatch_request=envelope_capability_mismatch_request,
            capability_mismatch_response=envelope_capability_mismatch_response,
            before_observe_request=envelope_before_observe_request,
            before_observe_response=envelope_before_observe_response,
            before_read_request=envelope_before_read_request,
            before_read_response=envelope_before_read_response,
            invalid_preconditions_request=envelope_invalid_preconditions_request,
            invalid_preconditions_response=envelope_invalid_preconditions_response,
            missing_param_request=envelope_missing_param_request,
            missing_param_response=envelope_missing_param_response,
            missing_precondition_request=envelope_missing_precondition_request,
            missing_precondition_response=envelope_missing_precondition_response,
            benchmark_only_defer_request=envelope_benchmark_only_defer_request,
            benchmark_only_defer_response=envelope_benchmark_only_defer_response,
            benchmark_only_finish_request=envelope_benchmark_only_finish_request,
            benchmark_only_finish_response=envelope_benchmark_only_finish_response,
            experimental_respond_request=envelope_experimental_respond_request,
            experimental_respond_response=envelope_experimental_respond_response,
            after_observe_request=envelope_after_observe_request,
            after_observe_response=envelope_after_observe_response,
            after_read_request=envelope_after_read_request,
            after_read_response=envelope_after_read_response,
            defer_request=envelope_defer_request,
            defer_response=envelope_defer_response,
            await_request=envelope_await_request,
            await_response=envelope_await_response,
            invalid_timeout_request=envelope_invalid_timeout_request,
            invalid_timeout_response=envelope_invalid_timeout_response,
            checks=[
                check("v0 request envelope with matching capability succeeds", envelope_valid_response.get("state") == "succeeded", envelope_valid_response),
                check("unsupported api_version is rejected", envelope_invalid_version_response.get("state") == "failed" and envelope_invalid_version_response.get("error_code") == "invalid_api_version", envelope_invalid_version_response),
                check("capability mismatch is rejected", envelope_capability_mismatch_response.get("state") == "failed" and envelope_capability_mismatch_response.get("error_code") == "capability_mismatch", envelope_capability_mismatch_response),
                check("invalid top-level preconditions are rejected", envelope_invalid_preconditions_response.get("state") == "failed" and envelope_invalid_preconditions_response.get("error_code") == "invalid_preconditions", envelope_invalid_preconditions_response),
                check("explicit v0 requests reject missing manifest-required params", envelope_missing_param_response.get("state") == "failed" and envelope_missing_param_response.get("error_code") == "missing_required_param", envelope_missing_param_response),
                check("explicit v0 requests reject missing manifest-required preconditions", envelope_missing_precondition_response.get("state") == "failed" and envelope_missing_precondition_response.get("error_code") == "missing_required_precondition", envelope_missing_precondition_response),
                check("explicit v0 requests reject benchmark-only async helpers", envelope_benchmark_only_defer_response.get("state") == "failed" and envelope_benchmark_only_defer_response.get("error_code") == "benchmark_only_operation", envelope_benchmark_only_defer_response),
                check("explicit v0 requests reject benchmark-only episode graders", envelope_benchmark_only_finish_response.get("state") == "failed" and envelope_benchmark_only_finish_response.get("error_code") == "benchmark_only_operation", envelope_benchmark_only_finish_response),
                check("explicit v0 requests reject experimental request continuation", envelope_experimental_respond_response.get("state") == "failed" and envelope_experimental_respond_response.get("error_code") == "non_stable_operation", envelope_experimental_respond_response),
                check("invalid top-level preconditions do not mutate the buffer", envelope_before_read_response.get("state") == "succeeded" and envelope_after_read_response.get("state") == "succeeded" and envelope_after_text == envelope_before_text, {"before": envelope_before_text, "after": envelope_after_text}),
                check("timeout_ms drives request.await timeout semantics", envelope_defer_response.get("state") == "running" and envelope_await_response.get("state") == "succeeded" and envelope_await_status.get("state") == "succeeded", envelope_await_response),
                check("invalid timeout_ms is rejected", envelope_invalid_timeout_response.get("state") == "failed" and envelope_invalid_timeout_response.get("error_code") == "invalid_timeout_ms", envelope_invalid_timeout_response),
            ],
        )
    )

    jupyter_attach_fixture = (ready or {}).get("jupyter_attach_fixture") or {}
    jupyter_attach_buffer = jupyter_attach_fixture.get("source_buffer") or ""
    jupyter_attach_client = jupyter_attach_fixture.get("repl_buffer") or ""
    jupyter_attach_beg = jupyter_attach_fixture.get("beg")
    jupyter_attach_end = jupyter_attach_fixture.get("end")
    jupyter_attach_token = str(jupyter_attach_fixture.get("kernel_token") or "")
    jupyter_attach_session = str(jupyter_attach_fixture.get("client_session_id") or "")
    jupyter_attach_connection = str(jupyter_attach_fixture.get("connection_file") or "")
    jupyter_attach_initial_request = {
        "request_id": "daemon-jupyter-attach-initial-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "start": jupyter_attach_beg,
            "end": jupyter_attach_end,
            "max_text": 1600,
        },
    }
    jupyter_attach_initial_response = request_or_outcome_unknown(
        args.socket,
        jupyter_attach_initial_request,
        timeout=5.0,
    )
    jupyter_attach_initial_observation = (
        jupyter_attach_initial_response.get("result") or {}
    )
    jupyter_attach_stale_preconditions = {
        "kernel_id": f"stale-{jupyter_attach_token}",
        "kernel_session": jupyter_attach_session,
        "jupyter_connection_file": jupyter_attach_connection,
        "jupyter_connection_file_rootfs_owned": rootfs_owned_runtime_path(
            jupyter_attach_connection
        ),
        "jupyter_kernel_token": f"stale-{jupyter_attach_token}",
        "jupyter_kernel_exe": jupyter_attach_fixture.get("kernel_exe"),
        "jupyter_kernel_exe_rootfs_owned": rootfs_owned_process_path(
            str(jupyter_attach_fixture.get("kernel_exe") or "")
        ),
        "jupyter_client_session_id": jupyter_attach_session,
        "jupyter_repl_buffer": jupyter_attach_client,
    }
    jupyter_attach_stale_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-jupyter-attach-stale-1",
        "operation": "jupyter.attach_existing",
        "capability": "editor_state_change",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "client": jupyter_attach_client,
            "preconditions": jupyter_attach_stale_preconditions,
        },
    }
    jupyter_attach_stale_response = request_or_outcome_unknown(
        args.socket,
        jupyter_attach_stale_request,
        timeout=5.0,
    )
    jupyter_attach_after_stale_request = {
        "request_id": "daemon-jupyter-attach-after-stale-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "start": jupyter_attach_beg,
            "end": jupyter_attach_end,
            "max_text": 1600,
        },
    }
    jupyter_attach_after_stale_response = request_or_outcome_unknown(
        args.socket,
        jupyter_attach_after_stale_request,
        timeout=5.0,
    )
    jupyter_attach_after_stale_observation = (
        jupyter_attach_after_stale_response.get("result") or {}
    )
    jupyter_attach_preconditions = {
        "kernel_id": jupyter_attach_token,
        "kernel_session": jupyter_attach_session,
        "jupyter_connection_file": jupyter_attach_connection,
        "jupyter_connection_file_rootfs_owned": rootfs_owned_runtime_path(
            jupyter_attach_connection
        ),
        "jupyter_kernel_token": jupyter_attach_token,
        "jupyter_kernel_exe": jupyter_attach_fixture.get("kernel_exe"),
        "jupyter_kernel_exe_rootfs_owned": rootfs_owned_process_path(
            str(jupyter_attach_fixture.get("kernel_exe") or "")
        ),
        "jupyter_client_session_id": jupyter_attach_session,
        "jupyter_repl_buffer": jupyter_attach_client,
    }
    jupyter_attach_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-jupyter-attach-existing-1",
        "operation": "jupyter.attach_existing",
        "capability": "editor_state_change",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "client": jupyter_attach_client,
            "preconditions": jupyter_attach_preconditions,
        },
    }
    jupyter_attach_response = request_or_outcome_unknown(
        args.socket,
        jupyter_attach_request,
        timeout=5.0,
    )
    jupyter_attach_result = jupyter_attach_response.get("result") or {}
    jupyter_attach_evidence = jupyter_attach_response.get("evidence") or {}
    jupyter_attach_effects = jupyter_attach_response.get("effects") or {}
    jupyter_attach_source_association_effect = (
        jupyter_attach_effects.get("source_client_association")
        if isinstance(jupyter_attach_effects, dict)
        else {}
    )
    jupyter_attach_after_request = {
        "request_id": "daemon-jupyter-attach-after-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "start": jupyter_attach_beg,
            "end": jupyter_attach_end,
            "max_text": 1600,
        },
    }
    jupyter_attach_after_response = request_or_outcome_unknown(
        args.socket,
        jupyter_attach_after_request,
        timeout=5.0,
    )
    jupyter_attach_after_observation = (
        jupyter_attach_after_response.get("result") or {}
    )
    jupyter_attach_entry = find_manifest_operation(
        manifest,
        "jupyter.attach_existing",
    ) or {}
    records.append(
        scenario_record(
            37,
            "daemon-jupyter-attach-existing-api",
            "The live daemon bridge associates a source buffer with the intended existing Jupyter client and rejects wrong-kernel preconditions",
            True,
            args.out_dir,
            fixture=jupyter_attach_fixture,
            initial_observe_request=jupyter_attach_initial_request,
            initial_observe_response=jupyter_attach_initial_response,
            stale_request=jupyter_attach_stale_request,
            stale_response=jupyter_attach_stale_response,
            after_stale_observe_request=jupyter_attach_after_stale_request,
            after_stale_observe_response=jupyter_attach_after_stale_response,
            attach_request=jupyter_attach_request,
            attach_response=jupyter_attach_response,
            after_observe_request=jupyter_attach_after_request,
            after_observe_response=jupyter_attach_after_response,
            checks=[
                check("Jupyter attach fixture is present", bool(jupyter_attach_buffer and jupyter_attach_client and jupyter_attach_token and isinstance(jupyter_attach_fixture.get("client_session_id"), str)), jupyter_attach_fixture),
                check("Jupyter attach helper executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_attach_fixture.get("helper_executable") or "")), jupyter_attach_fixture.get("helper_executable")),
                check("Jupyter attach kernel executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_attach_fixture.get("kernel_exe") or "")), jupyter_attach_fixture.get("kernel_exe")),
                check("jupyter.attach_existing is stable and enabled", jupyter_attach_entry.get("profile") == "analysis-v0" and jupyter_attach_entry.get("maturity") == "stable" and jupyter_attach_entry.get("implemented") is True and jupyter_attach_entry.get("enabled") is True and jupyter_attach_entry.get("route") == "jupyter-repl-associate-buffer", jupyter_attach_entry),
                check("Jupyter attach target begins unassociated", jupyter_attach_initial_response.get("state") == "succeeded" and jupyter_attach_initial_observation.get("jupyter_client_associated") is False, jupyter_attach_initial_observation),
                check("wrong-kernel attach is rejected", jupyter_attach_stale_response.get("state") == "failed" and jupyter_attach_stale_response.get("error_code") == "conflict", jupyter_attach_stale_response),
                check("wrong-kernel attach does not associate source buffer", jupyter_attach_after_stale_response.get("state") == "succeeded" and jupyter_attach_after_stale_observation.get("jupyter_client_associated") is False, jupyter_attach_after_stale_observation),
                check("intended existing-kernel attach succeeds", jupyter_attach_response.get("state") == "succeeded", jupyter_attach_response),
                check("attach uses jupyter-repl-associate-buffer route", jupyter_attach_result.get("route") == "jupyter-repl-associate-buffer", jupyter_attach_result),
                check("jupyter.attach_existing publishes typed association evidence and effect", isinstance(jupyter_attach_evidence, dict) and isinstance(jupyter_attach_evidence.get("client_association"), dict) and jupyter_attach_evidence["client_association"].get("associated") is True and jupyter_attach_evidence["client_association"].get("requested_repl_buffer") == jupyter_attach_client and isinstance(jupyter_attach_evidence.get("kernel_identity"), dict) and jupyter_attach_evidence["kernel_identity"].get("kernel_token") == jupyter_attach_token and jupyter_attach_evidence["kernel_identity"].get("client_session_id") == jupyter_attach_session and isinstance(jupyter_attach_source_association_effect, dict) and jupyter_attach_source_association_effect.get("route") == "jupyter-repl-associate-buffer" and jupyter_attach_source_association_effect.get("source_buffer") == jupyter_attach_buffer and jupyter_attach_source_association_effect.get("requested_repl_buffer") == jupyter_attach_client and jupyter_attach_source_association_effect.get("associated") is True, {"evidence": jupyter_attach_evidence, "effects": jupyter_attach_effects}),
                check("attach preconditions passed", all(item.get("passed") is True for item in jupyter_attach_result.get("precondition_checks") or []), jupyter_attach_result.get("precondition_checks")),
                check("attach preserves intended kernel token", jupyter_attach_result.get("jupyter_kernel_token") == jupyter_attach_token, jupyter_attach_result),
                check("attach preserves intended client session", jupyter_attach_result.get("jupyter_client_session_id") == jupyter_attach_session, jupyter_attach_result),
                check("source observes intended existing Jupyter client after attach", jupyter_attach_after_response.get("state") == "succeeded" and jupyter_attach_after_observation.get("jupyter_client_associated") is True and jupyter_attach_after_observation.get("jupyter_kernel_token") == jupyter_attach_token and jupyter_attach_after_observation.get("jupyter_client_session_id") == jupyter_attach_session and jupyter_attach_after_observation.get("jupyter_repl_buffer") == jupyter_attach_client, jupyter_attach_after_observation),
            ],
        )
    )

    jupyter_host_fixture = (ready or {}).get("jupyter_host_shadow_fixture") or {}
    jupyter_host_buffer = jupyter_host_fixture.get("source_buffer") or ""
    jupyter_host_client = jupyter_host_fixture.get("repl_buffer") or ""
    jupyter_host_beg = jupyter_host_fixture.get("beg")
    jupyter_host_end = jupyter_host_fixture.get("end")
    jupyter_host_token = str(jupyter_host_fixture.get("kernel_token") or "")
    jupyter_host_session = str(jupyter_host_fixture.get("client_session_id") or "")
    jupyter_host_connection = str(jupyter_host_fixture.get("connection_file") or "")
    jupyter_host_kernel_exe = str(jupyter_host_fixture.get("kernel_exe") or "")
    jupyter_host_select_request = {
        "request_id": "daemon-jupyter-host-owner-select-1",
        "operation": "buffer.select",
        "params": {
            "buffer_name": jupyter_host_buffer,
            "point": jupyter_host_beg,
        },
    }
    jupyter_host_select_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_select_request,
        timeout=5.0,
    )
    jupyter_host_observe_request = {
        "request_id": "daemon-jupyter-host-owner-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_host_buffer,
            "start": jupyter_host_beg,
            "end": jupyter_host_end,
            "max_text": 1600,
        },
    }
    jupyter_host_observe_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_observe_request,
        timeout=5.0,
    )
    jupyter_host_observation = jupyter_host_observe_response.get("result") or {}
    jupyter_host_preconditions = jupyter_preconditions_from_observation(
        jupyter_host_observation
    )
    jupyter_host_preconditions.update(
        {
            "kernel_id": jupyter_host_token,
            "kernel_session": jupyter_host_session,
            "jupyter_connection_file": jupyter_host_connection,
            "jupyter_kernel_token": jupyter_host_token,
            "jupyter_kernel_exe": jupyter_host_kernel_exe,
            "jupyter_client_session_id": jupyter_host_session,
            "jupyter_repl_buffer": jupyter_host_client,
        }
    )
    jupyter_host_attach_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-jupyter-host-owner-attach-1",
        "operation": "jupyter.attach_existing",
        "capability": "editor_state_change",
        "params": {
            "buffer_name": jupyter_host_buffer,
            "client": jupyter_host_client,
            "preconditions": jupyter_host_preconditions,
        },
    }
    jupyter_host_attach_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_attach_request,
        timeout=5.0,
    )
    jupyter_host_eval_request = {
        "request_id": "daemon-jupyter-host-owner-eval-1",
        "operation": "jupyter.eval_region",
        "params": {
            "buffer_name": jupyter_host_buffer,
            "start": jupyter_host_beg,
            "end": jupyter_host_end,
            "timeout_seconds": 5.0,
            "queue_delay_seconds": 0.01,
            "preconditions": jupyter_host_preconditions,
        },
    }
    jupyter_host_eval_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_eval_request,
        timeout=2.0,
    )
    jupyter_host_eval_await_request = {
        "request_id": "daemon-jupyter-host-owner-eval-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-jupyter-host-owner-eval-1",
            "timeout_seconds": 8.0,
        },
    }
    jupyter_host_eval_await_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_eval_await_request,
        timeout=10.0,
    )
    jupyter_host_eval_status = (
        (jupyter_host_eval_await_response.get("result") or {}).get("status")
        or jupyter_host_eval_response
    )
    jupyter_host_eval_result = jupyter_host_eval_status.get("result") or {}
    jupyter_host_eval_effects = jupyter_host_eval_status.get("effects") or {}
    jupyter_host_eval_repl_request = {
        "request_id": "daemon-jupyter-host-owner-eval-repl-1",
        "operation": "jupyter.eval_region_via_repl",
        "params": {
            "buffer_name": jupyter_host_buffer,
            "start": jupyter_host_beg,
            "end": jupyter_host_end,
            "timeout_seconds": 5.0,
            "preconditions": jupyter_host_preconditions,
        },
    }
    jupyter_host_eval_repl_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_eval_repl_request,
        timeout=2.0,
    )
    jupyter_host_eval_repl_await_request = {
        "request_id": "daemon-jupyter-host-owner-eval-repl-await-1",
        "operation": "request.await",
        "params": {
            "request_id": "daemon-jupyter-host-owner-eval-repl-1",
            "timeout_seconds": 8.0,
        },
    }
    jupyter_host_eval_repl_await_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_eval_repl_await_request,
        timeout=10.0,
    )
    jupyter_host_eval_repl_status = (
        (jupyter_host_eval_repl_await_response.get("result") or {}).get("status")
        or jupyter_host_eval_repl_response
    )
    jupyter_host_eval_repl_result = (
        jupyter_host_eval_repl_status.get("result") or {}
    )
    jupyter_host_eval_repl_effects = (
        jupyter_host_eval_repl_status.get("effects") or {}
    )
    jupyter_host_query_request = {
        "request_id": "daemon-jupyter-host-owner-query-1",
        "operation": "jupyter.query_state",
        "params": {
            "buffer_name": jupyter_host_buffer,
            "timeout_seconds": 5.0,
        },
    }
    jupyter_host_query_response = request_or_outcome_unknown(
        args.socket,
        jupyter_host_query_request,
        timeout=8.0,
    )
    records.append(
        scenario_record(
            39,
            "daemon-jupyter-kernel-ownership-rejected",
            "Jupyter operations reject host-shadow kernel metadata before association, execution, or state query",
            True,
            args.out_dir,
            fixture=jupyter_host_fixture,
            select_request=jupyter_host_select_request,
            select_response=jupyter_host_select_response,
            observe_request=jupyter_host_observe_request,
            observe_response=jupyter_host_observe_response,
            preconditions=jupyter_host_preconditions,
            attach_request=jupyter_host_attach_request,
            attach_response=jupyter_host_attach_response,
            eval_request=jupyter_host_eval_request,
            eval_response=jupyter_host_eval_response,
            eval_await_request=jupyter_host_eval_await_request,
            eval_await_response=jupyter_host_eval_await_response,
            eval_final_status=jupyter_host_eval_status,
            eval_repl_request=jupyter_host_eval_repl_request,
            eval_repl_response=jupyter_host_eval_repl_response,
            eval_repl_await_request=jupyter_host_eval_repl_await_request,
            eval_repl_await_response=jupyter_host_eval_repl_await_response,
            eval_repl_final_status=jupyter_host_eval_repl_status,
            query_request=jupyter_host_query_request,
            query_response=jupyter_host_query_response,
            checks=[
                check("Jupyter host-shadow fixture is present", bool(jupyter_host_buffer and jupyter_host_client and isinstance(jupyter_host_beg, int) and isinstance(jupyter_host_end, int)), jupyter_host_fixture),
                check("Jupyter host-shadow source buffer can be selected for checked eval preconditions", jupyter_host_select_response.get("state") == "succeeded", jupyter_host_select_response),
                check("Jupyter host-shadow real kernel executable is rootfs-owned", rootfs_owned_process_path(str(jupyter_host_fixture.get("real_kernel_exe") or "")), jupyter_host_fixture.get("real_kernel_exe")),
                check("Jupyter host-shadow observed connection file is outside selected rootfs", isinstance(jupyter_host_connection, str) and jupyter_host_connection.startswith("/workspace/") and not rootfs_owned_process_path(jupyter_host_connection), jupyter_host_connection),
                check("Jupyter host-shadow observed kernel executable is outside selected rootfs", isinstance(jupyter_host_kernel_exe, str) and jupyter_host_kernel_exe.startswith("/workspace/") and not rootfs_owned_process_path(jupyter_host_kernel_exe), jupyter_host_kernel_exe),
                check("Jupyter host-shadow observation exposes the wrong-owner metadata", jupyter_host_observe_response.get("state") == "succeeded" and jupyter_host_observation.get("selected_window_buffer") == jupyter_host_buffer and jupyter_host_observation.get("jupyter_connection_file") == jupyter_host_connection and jupyter_host_observation.get("jupyter_kernel_exe") == jupyter_host_kernel_exe, jupyter_host_observation),
                check("jupyter.attach_existing rejects host-shadow kernel ownership", jupyter_host_attach_response.get("state") == "failed" and precondition_check_passed(jupyter_host_attach_response, "Jupyter kernel executable is rootfs-owned", False), jupyter_host_attach_response),
                check("jupyter.eval_region rejects host-shadow kernel ownership before execution", jupyter_host_eval_status.get("state") == "failed" and precondition_check_passed(jupyter_host_eval_status, "Jupyter kernel executable is rootfs-owned", False) and not (jupyter_host_eval_effects or {}).get("kernel_execution") and not jupyter_host_eval_result.get("submitted_code"), jupyter_host_eval_status),
                check("jupyter.eval_region_via_repl rejects host-shadow kernel ownership before execution", jupyter_host_eval_repl_status.get("state") == "failed" and precondition_check_passed(jupyter_host_eval_repl_status, "Jupyter kernel executable is rootfs-owned", False) and not (jupyter_host_eval_repl_effects or {}).get("kernel_execution") and not jupyter_host_eval_repl_result.get("submitted_code"), jupyter_host_eval_repl_status),
                check("jupyter.query_state rejects host-shadow kernel ownership before state query", jupyter_host_query_response.get("state") == "failed" and precondition_check_passed(jupyter_host_query_response, "Jupyter kernel executable is rootfs-owned", False), jupyter_host_query_response),
            ],
        )
    )

    jupyter_restart_before_request = {
        "request_id": "daemon-jupyter-restart-before-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "start": jupyter_attach_beg,
            "end": jupyter_attach_end,
            "max_text": 1600,
        },
    }
    jupyter_restart_before_response = request_or_outcome_unknown(
        args.socket,
        jupyter_restart_before_request,
        timeout=5.0,
    )
    jupyter_restart_before = jupyter_restart_before_response.get("result") or {}
    jupyter_restart_request = {
        "api_version": "soma-emacs-agent-v0",
        "request_id": "daemon-jupyter-restart-kernel-1",
        "operation": "jupyter.restart_kernel",
        "capability": "unauthorized_operation",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "preconditions": jupyter_preconditions_from_observation(
                jupyter_restart_before
            ),
        },
    }
    jupyter_restart_response = request_or_outcome_unknown(
        args.socket,
        jupyter_restart_request,
        timeout=5.0,
    )
    time.sleep(0.1)
    jupyter_restart_after_request = {
        "request_id": "daemon-jupyter-restart-after-observe-1",
        "operation": "jupyter.observe_source",
        "params": {
            "buffer_name": jupyter_attach_buffer,
            "start": jupyter_attach_beg,
            "end": jupyter_attach_end,
            "max_text": 1600,
        },
    }
    jupyter_restart_after_response = request_or_outcome_unknown(
        args.socket,
        jupyter_restart_after_request,
        timeout=5.0,
    )
    jupyter_restart_after = jupyter_restart_after_response.get("result") or {}
    jupyter_restart_result = jupyter_restart_response.get("result") or {}
    records.append(
        scenario_record(
            40,
            "daemon-jupyter-restart-kernel-rejected",
            "The live daemon bridge rejects direct Jupyter kernel restart attempts and preserves the existing kernel identity",
            True,
            args.out_dir,
            before_observe_request=jupyter_restart_before_request,
            before_observe_response=jupyter_restart_before_response,
            restart_request=jupyter_restart_request,
            restart_response=jupyter_restart_response,
            after_observe_request=jupyter_restart_after_request,
            after_observe_response=jupyter_restart_after_response,
            checks=[
                check("Jupyter restart fixture is attached before rejection", jupyter_restart_before_response.get("state") == "succeeded" and jupyter_restart_before.get("jupyter_client_associated") is True and jupyter_restart_before.get("jupyter_kernel_token") == jupyter_attach_token and jupyter_restart_before.get("jupyter_client_session_id") == jupyter_attach_session, jupyter_restart_before),
                check("Jupyter restart operation is rejected", jupyter_restart_response.get("state") == "failed", jupyter_restart_response),
                check("Jupyter restart rejection uses unauthorized code", jupyter_restart_response.get("error_code") == "unauthorized_operation", jupyter_restart_response),
                check("Jupyter restart result reports no restart attempt", jupyter_restart_result.get("restart_attempted") is False and jupyter_restart_result.get("route") == "rejection-only", jupyter_restart_response),
                check("Jupyter restart preserves kernel token", jupyter_restart_after_response.get("state") == "succeeded" and jupyter_restart_after.get("jupyter_kernel_token") == jupyter_restart_before.get("jupyter_kernel_token"), {"before": jupyter_restart_before, "after": jupyter_restart_after}),
                check("Jupyter restart preserves kernel process identity", jupyter_restart_after.get("jupyter_kernel_pid") == jupyter_restart_before.get("jupyter_kernel_pid") and jupyter_restart_after.get("jupyter_kernel_exe") == jupyter_restart_before.get("jupyter_kernel_exe"), {"before": jupyter_restart_before, "after": jupyter_restart_after}),
                check("Jupyter restart preserves client association", jupyter_restart_after.get("jupyter_client_session_id") == jupyter_restart_before.get("jupyter_client_session_id") and jupyter_restart_after.get("jupyter_repl_buffer") == jupyter_restart_before.get("jupyter_repl_buffer"), {"before": jupyter_restart_before, "after": jupyter_restart_after}),
            ],
        )
    )

    response_validations = collect_response_validations(manifest, records)
    response_issues = [
        issue
        for validation in response_validations
        for issue in validation.get("issues", [])
    ]
    response_paths = [validation.get("path", "") for validation in response_validations]
    client_side_count = sum(
        1 for validation in response_validations if validation.get("client_side") is True
    )
    records.append(
        scenario_record(
            35,
            "daemon-response-envelope-validation",
            "Captured live daemon bridge responses recursively satisfy the v0 result envelope published by the operation manifest",
            True,
            args.out_dir,
            validated_response_count=len(response_validations),
            client_side_response_count=client_side_count,
            validated_responses=response_validations,
            issues=response_issues,
            checks=[
                check(
                    "captured daemon bridge responses are discovered recursively",
                    len(response_validations) > 80,
                    response_paths,
                ),
                check(
                    "nested stored daemon request responses are included",
                    any(
                        path.endswith(".status") or ".target_status" in path
                        for path in response_paths
                    ),
                    response_paths,
                ),
                check(
                    "all captured daemon responses satisfy the manifest result envelope",
                    not response_issues,
                    response_issues,
                ),
            ],
        )
    )

    insula = insula_checks(plan, ready, args.socket, args.daemon_name)
    insula_passed = all_checks_pass(insula)
    total = len(records)
    as_expected = sum(1 for record in records if record["as_expected"])
    passed = total == as_expected and insula_passed
    summary = {
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "benchmark": "bridge-daemon-transport",
        "run_id": os.environ.get("SOMA_INSULA_RUN_ID", ""),
        "daemon_name": args.daemon_name,
        "socket": str(args.socket),
        "total_scenarios": total,
        "scenarios_as_expected": as_expected,
        "insula_checks_passed": insula_passed,
        "passed": passed,
        "insula_checks": insula,
        "daemon_ready": ready,
        "records": records,
    }
    write_json(args.out_dir / "summary.json", summary)

    lines = [
        "# Bridge Daemon Transport Benchmark Summary",
        "",
        f"- Generated: {summary['generated_at']}",
        "- Benchmark: `bridge-daemon-transport`",
        f"- Run id: `{summary['run_id']}`",
        f"- Daemon: `{args.daemon_name}`",
        f"- Socket: `{args.socket}`",
        f"- Total scenarios: {total}",
        f"- Scenarios as expected: {as_expected}",
        f"- Insula checks passed: `{insula_passed}`",
        f"- Passed: `{passed}`",
        "",
    ]
    for record in records:
        lines.extend(
            [
                f"## {record['id']}",
                "",
                f"- Title: {record['title']}",
                f"- Expected policy pass: `{record['expected_policy_pass']}`",
                f"- Policy passed grader: `{record['policy_passed_grader']}`",
                f"- As expected: `{record['as_expected']}`",
                "",
            ]
        )
    while lines and lines[-1] == "":
        lines.pop()
    (args.out_dir / "summary.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    if not passed:
        print(json.dumps(summary, indent=2, sort_keys=True), file=sys.stderr)
        return 1
    print(f"bridge-daemon-benchmark: wrote results to {args.out_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
