from __future__ import annotations

import hashlib
import json
import re
import stat
from pathlib import Path, PurePosixPath
from typing import Any, Mapping


HISTORICAL_PROMPT_BYTES = 4106
HISTORICAL_PROMPT_SHA256 = (
    "0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc"
)
HISTORICAL_OUTPUT_PATH = "/Users/shanmujin/Documents/CrouzeixConjecture/LaTeX"
MAX_JSON_BYTES = 1024 * 1024

SHA256 = re.compile(r"^[0-9a-f]{64}$")
PORTABLE_ID = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
UTC_TIMESTAMP = re.compile(
    r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z$"
)

RUN_SPEC_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "arm",
        "leakage",
        "model",
        "cli",
        "historical_prompt",
        "sandbox",
        "approval_policy",
        "allowed_tools",
        "network_access",
        "timeout_seconds",
        "max_calls",
        "token_accounting",
        "generation_visible_files",
        "generation_excluded_classes",
        "created_at_utc",
    }
)
FRONTIER_RUN_SPEC_FIELDS = RUN_SPEC_FIELDS | frozenset({"frontier", "digests"})
GUIDED_RUN_SPEC_FIELDS = RUN_SPEC_FIELDS | frozenset({"guided", "digests"})
CLI_FIELDS = frozenset({"path", "version", "sha256"})
PROMPT_FIELDS = frozenset(
    {
        "source_bytes",
        "source_sha256",
        "execution_bytes",
        "execution_sha256",
        "normalization",
    }
)
TOKEN_ACCOUNTING_FIELDS = frozenset({"boundary"})
GUIDED_FIELDS = frozenset(
    {
        "mechanism_card_sha256",
        "theorem_sha256",
        "prompt_sha256",
        "schema_sha256",
        "resource_preflight_passed",
        "max_calls",
    }
)
FRONTIER_FIELDS = frozenset(
    {
        "selection_seed",
        "expert_roles",
        "proof_progress_probe_ids",
        "root_expert_count",
        "child_generations",
        "draws_per_generation",
        "admitted_mathematical_node_budget",
        "total_call_budget",
        "per_call_preflight",
    }
)
PREFLIGHT_FIELDS = frozenset({"memory_mib"})
ROUTE_EVENT_FIELDS = frozenset(
    {
        "schema_version",
        "sequence",
        "run_id",
        "route_id",
        "family",
        "from_state",
        "to_state",
        "parent_route_ids",
        "prompt_sha256",
        "output_sha256",
        "candidate_sha256",
        "reason",
        "mechanism",
        "concrete_artifacts",
        "unproved_obligations",
        "unresolved_critical_findings",
        "occurred_at_utc",
    }
)
OBLIGATION_FIELDS = frozenset({"statement", "strength"})

ARMS = frozenset({"historical", "orchestrated", "guided", "expert_frontier"})
LEAKAGE_TIERS = frozenset({"L0", "L1", "L2", "L3", "L4"})
SANDBOXES = frozenset({"read-only", "workspace-write"})
ALLOWED_TOOL_NAMES = frozenset(
    {"Read", "Glob", "Grep", "Bash", "Write", "Edit", "spawn_agent"}
)
ROUTE_STATES = frozenset(
    {"independent", "blocked", "viable", "audited", "promoted", "rejected"}
)
ALLOWED_TRANSITIONS = frozenset(
    {
        (None, "independent"),
        ("independent", "blocked"),
        ("independent", "viable"),
        ("blocked", "independent"),
        ("viable", "audited"),
        ("audited", "promoted"),
        ("audited", "rejected"),
    }
)


class ValidationError(ValueError):
    pass


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def verify_historical_prompt(
    data: bytes,
    *,
    expected_bytes: int = HISTORICAL_PROMPT_BYTES,
    expected_sha256: str = HISTORICAL_PROMPT_SHA256,
) -> dict[str, object]:
    if len(data) != expected_bytes:
        raise ValidationError(
            f"historical prompt byte count must be {expected_bytes}, got {len(data)}"
        )
    observed = sha256_bytes(data)
    if observed != expected_sha256:
        raise ValidationError(
            f"historical prompt SHA-256 must be {expected_sha256}, got {observed}"
        )
    return {"bytes": len(data), "sha256": observed}


def normalize_historical_prompt(data: bytes) -> tuple[bytes, dict[str, object]]:
    source = HISTORICAL_OUTPUT_PATH.encode("utf-8")
    replacement = b"candidate.tex"
    count = data.count(source)
    if count != 1:
        raise ValidationError(
            f"historical output path must occur exactly once, got {count}"
        )
    normalized = data.replace(source, replacement)
    return normalized, {
        "schema_version": "crouzeix-prompt-normalization/v1",
        "replacement_count": count,
        "source_text": HISTORICAL_OUTPUT_PATH,
        "replacement_text": replacement.decode("ascii"),
        "source_bytes": len(data),
        "source_sha256": sha256_bytes(data),
        "execution_bytes": len(normalized),
        "execution_sha256": sha256_bytes(normalized),
    }


def validate_run_spec(value: Mapping[str, Any]) -> dict[str, object]:
    if not isinstance(value, Mapping):
        raise ValidationError("run specification must be an object")
    if value.get("arm") == "expert_frontier":
        expected_fields = FRONTIER_RUN_SPEC_FIELDS
    elif value.get("arm") == "guided" and "guided" in value:
        expected_fields = GUIDED_RUN_SPEC_FIELDS
    else:
        expected_fields = RUN_SPEC_FIELDS
    _require_fields(value, expected_fields, "run specification")
    _require_equal(value["schema_version"], "crouzeix-run-spec/v1", "schema_version")
    _portable_id(value["run_id"], "run_id")
    _enum(value["arm"], ARMS, "arm")
    _enum(value["leakage"], LEAKAGE_TIERS, "leakage")
    _bounded_string(value["model"], "model", 1, 128)

    cli = _mapping(value["cli"], "cli")
    _require_fields(cli, CLI_FIELDS, "cli")
    cli_path = _bounded_string(cli["path"], "cli.path", 1, 4096)
    if not Path(cli_path).is_absolute() or "\0" in cli_path:
        raise ValidationError("cli.path must be an absolute non-NUL path")
    _bounded_string(cli["version"], "cli.version", 1, 256)
    _digest(cli["sha256"], "cli.sha256")

    prompt = _mapping(value["historical_prompt"], "historical_prompt")
    _require_fields(prompt, PROMPT_FIELDS, "historical_prompt")
    _bounded_integer(prompt["source_bytes"], "historical_prompt.source_bytes", 1, 1_000_000)
    _digest(prompt["source_sha256"], "historical_prompt.source_sha256")
    _bounded_integer(
        prompt["execution_bytes"],
        "historical_prompt.execution_bytes",
        1,
        1_000_000,
    )
    _digest(prompt["execution_sha256"], "historical_prompt.execution_sha256")
    _bounded_string(prompt["normalization"], "historical_prompt.normalization", 1, 512)

    _enum(value["sandbox"], SANDBOXES, "sandbox")
    _require_equal(value["approval_policy"], "never", "approval_policy")
    tools = _string_list(value["allowed_tools"], "allowed_tools", minimum=1, maximum=8)
    if len(set(tools)) != len(tools) or not set(tools).issubset(ALLOWED_TOOL_NAMES):
        raise ValidationError("allowed_tools must be unique approved tool names")
    if value["network_access"] is not False:
        raise ValidationError("network_access must be false")
    _bounded_integer(value["timeout_seconds"], "timeout_seconds", 30, 14_400)
    max_calls = _bounded_integer(value["max_calls"], "max_calls", 1, 128)
    if value["arm"] == "historical" and max_calls != 1:
        raise ValidationError("historical arm max_calls must be 1")
    if value["arm"] == "expert_frontier" and max_calls != 33:
        raise ValidationError("expert_frontier max_calls must be 33")
    if "guided" in value:
        if value["arm"] != "guided":
            raise ValidationError("guided run specification requires arm guided")
        if value["leakage"] != "L3":
            raise ValidationError("guided run specification leakage must be L3")
        if max_calls != 1:
            raise ValidationError("guided arm max_calls must be 1")

    accounting = _mapping(value["token_accounting"], "token_accounting")
    _require_fields(accounting, TOKEN_ACCOUNTING_FIELDS, "token_accounting")
    _bounded_string(accounting["boundary"], "token_accounting.boundary", 1, 512)

    visible = _string_list(
        value["generation_visible_files"],
        "generation_visible_files",
        minimum=1,
        maximum=64,
    )
    if len(set(visible)) != len(visible):
        raise ValidationError("generation_visible_files must be unique")
    for path in visible:
        _safe_relative_path(path, "generation_visible_files")

    excluded = _string_list(
        value["generation_excluded_classes"],
        "generation_excluded_classes",
        minimum=1,
        maximum=64,
    )
    if len(set(excluded)) != len(excluded):
        raise ValidationError("generation_excluded_classes must be unique")
    if value["arm"] == "expert_frontier":
        _validate_frontier_config(value["frontier"], "frontier")
        digests = _mapping(value["digests"], "digests")
        if len(digests) == 0 or len(digests) > 128:
            raise ValidationError("digests must be a nonempty bounded object")
        for key, digest in digests.items():
            _bounded_string(key, "digests key", 1, 256)
            _digest(digest, f"digests.{key}")
    if "guided" in value:
        _validate_guided_config(value["guided"], "guided")
        digests = _mapping(value["digests"], "digests")
        if len(digests) == 0 or len(digests) > 128:
            raise ValidationError("digests must be a nonempty bounded object")
        for key, digest in digests.items():
            _bounded_string(key, "digests key", 1, 256)
            _digest(digest, f"digests.{key}")
    _timestamp(value["created_at_utc"], "created_at_utc")
    return dict(value)


def read_run_spec(path: Path) -> dict[str, object]:
    return validate_run_spec(read_strict_json_object(path, "run specification"))


def read_strict_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise ValidationError(f"{label} must be a regular file")
    if metadata.st_size > MAX_JSON_BYTES:
        raise ValidationError(f"{label} exceeds byte cap")
    try:
        data = path.read_bytes()
        value = json.loads(data)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must contain one JSON object")
    return value


class RouteRegistry:
    def __init__(self, run_id: str) -> None:
        self.run_id = _portable_id(run_id, "run_id")
        self.events: list[dict[str, object]] = []
        self.states: dict[str, str] = {}
        self._mechanisms: dict[str, str] = {}
        self._audits: dict[str, dict[str, object]] = {}

    def apply(self, raw_event: Mapping[str, Any]) -> None:
        event = validate_route_event(raw_event)
        if event["run_id"] != self.run_id:
            raise ValidationError("route event run_id does not match registry")
        expected_sequence = len(self.events) + 1
        if event["sequence"] != expected_sequence:
            raise ValidationError(
                f"route event sequence must be {expected_sequence}, "
                f"got {event['sequence']}"
            )

        route_id = str(event["route_id"])
        previous = self.states.get(route_id)
        if event["from_state"] != previous:
            raise ValidationError(
                f"route {route_id} from_state does not match current state"
            )
        transition = (previous, str(event["to_state"]))
        if transition not in ALLOWED_TRANSITIONS:
            raise ValidationError(f"invalid route transition {transition}")

        if transition == ("blocked", "independent"):
            prior_mechanism = self._mechanisms[route_id]
            if event["mechanism"].strip() == prior_mechanism.strip():
                raise ValidationError("reopening a blocked route requires a new mechanism")
        if event["to_state"] == "viable" and not event["concrete_artifacts"]:
            raise ValidationError("viability requires concrete mathematical artifacts")
        if event["to_state"] == "audited":
            if event["candidate_sha256"] is None:
                raise ValidationError("audit requires a frozen candidate SHA-256")
            self._audits[route_id] = {
                "candidate_sha256": event["candidate_sha256"],
                "unresolved_critical_findings": event[
                    "unresolved_critical_findings"
                ],
                "unproved_obligations": list(event["unproved_obligations"]),
            }
        if event["to_state"] == "promoted":
            audit = self._audits.get(route_id)
            if audit is None:
                raise ValidationError("promotion requires a prior audit")
            if event["candidate_sha256"] != audit["candidate_sha256"]:
                raise ValidationError("promotion candidate does not match audited candidate")
            if int(audit["unresolved_critical_findings"]) != 0:
                raise ValidationError(
                    "promotion cannot retain unresolved critical findings"
                )
            obligations = list(audit["unproved_obligations"])
            if any(item["strength"] == "theorem_strength" for item in obligations):
                raise ValidationError(
                    "promotion cannot retain a theorem-strength obligation"
                )

        self.states[route_id] = str(event["to_state"])
        self._mechanisms[route_id] = str(event["mechanism"])
        self.events.append(event)


def validate_route_event(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, ROUTE_EVENT_FIELDS, "route event")
    _require_equal(value["schema_version"], "crouzeix-route-event/v1", "schema_version")
    _bounded_integer(value["sequence"], "sequence", 1, 10_000)
    _portable_id(value["run_id"], "run_id")
    _portable_id(value["route_id"], "route_id")
    _bounded_string(value["family"], "family", 1, 256)
    if value["from_state"] is not None:
        _enum(value["from_state"], ROUTE_STATES, "from_state")
    _enum(value["to_state"], ROUTE_STATES, "to_state")
    parents = _string_list(
        value["parent_route_ids"], "parent_route_ids", minimum=0, maximum=32
    )
    if len(set(parents)) != len(parents):
        raise ValidationError("parent_route_ids must be unique")
    for parent in parents:
        _portable_id(parent, "parent_route_ids")
    _digest(value["prompt_sha256"], "prompt_sha256")
    _digest(value["output_sha256"], "output_sha256")
    if value["candidate_sha256"] is not None:
        _digest(value["candidate_sha256"], "candidate_sha256")
    _bounded_string(value["reason"], "reason", 1, 4096)
    _bounded_string(value["mechanism"], "mechanism", 1, 4096)
    artifacts = _string_list(
        value["concrete_artifacts"],
        "concrete_artifacts",
        minimum=0,
        maximum=64,
    )
    obligations = value["unproved_obligations"]
    if not isinstance(obligations, list) or len(obligations) > 64:
        raise ValidationError("unproved_obligations must be a bounded list")
    normalized_obligations = []
    for obligation in obligations:
        item = _mapping(obligation, "unproved obligation")
        _require_fields(item, OBLIGATION_FIELDS, "unproved obligation")
        normalized_obligations.append(
            {
                "statement": _bounded_string(
                    item["statement"], "obligation.statement", 1, 4096
                ),
                "strength": _enum(
                    item["strength"],
                    frozenset({"local", "major", "theorem_strength"}),
                    "obligation.strength",
                ),
            }
        )
    _bounded_integer(
        value["unresolved_critical_findings"],
        "unresolved_critical_findings",
        0,
        10_000,
    )
    _timestamp(value["occurred_at_utc"], "occurred_at_utc")
    result = dict(value)
    result["concrete_artifacts"] = artifacts
    result["unproved_obligations"] = normalized_obligations
    return result


def _require_fields(
    value: Mapping[str, Any], expected: frozenset[str], label: str
) -> None:
    actual = frozenset(value)
    if actual != expected:
        raise ValidationError(
            f"{label} fields must be {sorted(expected)}, got {sorted(actual)}"
        )


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be an object")
    return value


def _require_equal(value: Any, expected: Any, label: str) -> Any:
    if value != expected:
        raise ValidationError(f"{label} must be {expected!r}")
    return value


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise ValidationError(f"{label} must be one of {sorted(allowed)}")
    return value


def _bounded_string(
    value: Any, label: str, minimum: int, maximum: int
) -> str:
    if (
        not isinstance(value, str)
        or len(value) < minimum
        or len(value) > maximum
        or "\0" in value
    ):
        raise ValidationError(
            f"{label} must be a non-NUL string of length {minimum}..{maximum}"
        )
    return value


def _portable_id(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if PORTABLE_ID.fullmatch(text) is None:
        raise ValidationError(f"{label} must be a portable lowercase ID")
    return text


def _validate_frontier_config(value: Any, label: str) -> dict[str, object]:
    item = _mapping(value, label)
    _require_fields(item, FRONTIER_FIELDS, label)
    if _bounded_integer(item["selection_seed"], "selection_seed", 1, 10_000_000_000) != 20260814:
        raise ValidationError("selection_seed must be 20260814")
    roles = _string_list(item["expert_roles"], "expert_roles", minimum=5, maximum=5)
    expected_roles = [
        "function_theory",
        "operator_dilation",
        "matrix_extremal",
        "completion_positivity",
        "approximation_audit",
    ]
    if roles != expected_roles:
        raise ValidationError("expert_roles must match the fixed E-arm roster")
    probes = _string_list(
        item["proof_progress_probe_ids"],
        "proof_progress_probe_ids",
        minimum=10,
        maximum=10,
    )
    if len(set(probes)) != 10:
        raise ValidationError("proof_progress_probe_ids must be unique")
    if _bounded_integer(item["root_expert_count"], "root_expert_count", 5, 5) != 5:
        raise ValidationError("root_expert_count must be 5")
    if _bounded_integer(item["child_generations"], "child_generations", 3, 3) != 3:
        raise ValidationError("child_generations must be 3")
    if _bounded_integer(item["draws_per_generation"], "draws_per_generation", 2, 2) != 2:
        raise ValidationError("draws_per_generation must be 2")
    if (
        _bounded_integer(
            item["admitted_mathematical_node_budget"],
            "admitted_mathematical_node_budget",
            11,
            11,
        )
        != 11
    ):
        raise ValidationError("admitted_mathematical_node_budget must be 11")
    if _bounded_integer(item["total_call_budget"], "total_call_budget", 33, 33) != 33:
        raise ValidationError("total_call_budget must be 33")
    preflight = _mapping(item["per_call_preflight"], "per_call_preflight")
    _require_fields(preflight, PREFLIGHT_FIELDS, "per_call_preflight")
    if _bounded_integer(preflight["memory_mib"], "memory_mib", 4096, 4096) != 4096:
        raise ValidationError("memory_mib must be 4096")
    return dict(item)


def _validate_guided_config(value: Any, label: str) -> dict[str, object]:
    item = _mapping(value, label)
    _require_fields(item, GUIDED_FIELDS, label)
    _digest(item["mechanism_card_sha256"], "guided.mechanism_card_sha256")
    _digest(item["theorem_sha256"], "guided.theorem_sha256")
    _digest(item["prompt_sha256"], "guided.prompt_sha256")
    _digest(item["schema_sha256"], "guided.schema_sha256")
    if item["resource_preflight_passed"] is not True:
        raise ValidationError("guided resource_preflight_passed must be true")
    if _bounded_integer(item["max_calls"], "guided.max_calls", 1, 1) != 1:
        raise ValidationError("guided.max_calls must be 1")
    return dict(item)


def _digest(value: Any, label: str) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise ValidationError(f"{label} must be a lowercase SHA-256")
    return value


def _bounded_integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or value < minimum
        or value > maximum
    ):
        raise ValidationError(f"{label} must be an integer in {minimum}..{maximum}")
    return value


def _string_list(
    value: Any, label: str, *, minimum: int, maximum: int
) -> list[str]:
    if (
        not isinstance(value, list)
        or len(value) < minimum
        or len(value) > maximum
    ):
        raise ValidationError(
            f"{label} must be a list with {minimum}..{maximum} entries"
        )
    return [
        _bounded_string(item, f"{label} item", 1, 4096)
        for item in value
    ]


def _safe_relative_path(value: str, label: str) -> str:
    path = PurePosixPath(value)
    if (
        path.is_absolute()
        or value in {"", "."}
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise ValidationError(f"{label} must contain normalized relative paths")
    return value


def _timestamp(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 20, 40)
    if UTC_TIMESTAMP.fullmatch(text) is None:
        raise ValidationError(f"{label} must be an RFC 3339 UTC timestamp")
    return text
