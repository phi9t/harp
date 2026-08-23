from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

try:
    from . import route_validation
except ImportError:  # direct script execution
    import route_validation


PINNED_TOOLCHAIN = "leanprover/lean4:v4.32.1"
SCHEMA_VERSION = "crouzeix-proof-preflight/v1"
ROUTE_ORDER = ("jin", "lorist-schwenninger", "harp")
MANIFEST_CONTRACT_SHA256 = "b349bbab303db9a554fafd3f0eafa8a2422d3b843b5e16c6f796debc4d8f4381"
DISPLAY_CACHE_ROOT = "formalization/lean/.lake"
MANAGED_LOCAL_PREFIXES = ("Crouzeix", "CrouzeixConjecture")
MANAGED_LOCAL_AGGREGATES = frozenset(
    {"CrouzeixJin", "CrouzeixLoristSchwenninger", "CrouzeixHarp"}
)
MATHLIB_GIT_URL = "https://github.com/leanprover-community/mathlib4.git"
MATHLIB_MANIFEST_REV = "520045ab14e26149ee970e2e617ca04b09bde5d6"
MATHLIB_LAKEFILE_REV = "v4.32.1"
EXPECTED_DEFAULT_TARGETS = (
    "TrainingDynamics",
    "MathematicalFoundations",
    "NNG4Intro",
    "AutodiffGeometry",
    "Crouzeix",
)
EXPECTED_LEAN_LIBS = (
    "TrainingDynamics",
    "MathematicalFoundations",
    "NNG4Intro",
    "AutodiffGeometry",
    "Crouzeix",
    "CrouzeixJin",
    "CrouzeixLoristSchwenninger",
    "CrouzeixHarp",
    "CrouzeixConjecture",
)
EXPECTED_REQUIRE_ENTRY = {
    "name": "mathlib",
    "git": MATHLIB_GIT_URL,
    "rev": MATHLIB_LAKEFILE_REV,
}
EXPECTED_LAKEFILE_CONTRACT = {
    "name": "harp_formalization",
    "defaultTargets": list(EXPECTED_DEFAULT_TARGETS),
    "lean_lib": [{"name": name} for name in EXPECTED_LEAN_LIBS],
    "require": [dict(EXPECTED_REQUIRE_ENTRY)],
}


@dataclass(frozen=True)
class RoutePolicy:
    route_id: str
    aggregate_module: str
    allowed_exact: tuple[str, ...]
    allowed_prefixes: tuple[str, ...]
    rejected_prefixes: tuple[str, ...]
    rejected_exact: tuple[str, ...] = ()


@dataclass(frozen=True)
class PreflightResult:
    route_id: str
    status: str
    aggregate_module: str
    toolchain: str | None
    expected_toolchain: str
    cache_root: str
    cache_identity: str
    aggregate_root: str
    expected_commands: tuple[str, ...]
    missing_artifacts: tuple[str, ...]
    local_modules: tuple[str, ...]
    reason: str | None = None

    def to_json(self) -> dict[str, object]:
        payload: dict[str, object] = {
            "schema_version": SCHEMA_VERSION,
            "route_id": self.route_id,
            "status": self.status,
            "aggregate_module": self.aggregate_module,
            "toolchain": self.toolchain,
            "expected_toolchain": self.expected_toolchain,
            "cache_root": self.cache_root,
            "cache_identity": self.cache_identity,
            "aggregate_root": self.aggregate_root,
            "expected_commands": list(self.expected_commands),
            "missing_artifacts": list(self.missing_artifacts),
            "local_modules": list(self.local_modules),
        }
        if self.reason is not None:
            payload["reason"] = self.reason
        return payload


ROUTE_POLICIES = {
    "jin": RoutePolicy(
        route_id="jin",
        aggregate_module="CrouzeixJin",
        allowed_exact=("CrouzeixJin",),
        allowed_prefixes=("Crouzeix.Jin", "CrouzeixConjecture"),
        rejected_prefixes=("Crouzeix.LoristSchwenninger", "Crouzeix.Harp"),
        rejected_exact=("CrouzeixLoristSchwenninger", "CrouzeixHarp", "Crouzeix"),
    ),
    "lorist-schwenninger": RoutePolicy(
        route_id="lorist-schwenninger",
        aggregate_module="CrouzeixLoristSchwenninger",
        allowed_exact=("CrouzeixLoristSchwenninger",),
        allowed_prefixes=("Crouzeix.LoristSchwenninger", "CrouzeixConjecture"),
        rejected_prefixes=("Crouzeix.Jin", "Crouzeix.Harp"),
        rejected_exact=("CrouzeixJin", "CrouzeixHarp", "Crouzeix"),
    ),
    "harp": RoutePolicy(
        route_id="harp",
        aggregate_module="CrouzeixHarp",
        allowed_exact=("CrouzeixHarp",),
        allowed_prefixes=("Crouzeix.Harp", "CrouzeixConjecture"),
        rejected_prefixes=("Crouzeix.Jin",),
        rejected_exact=(
            "Crouzeix",
            "CrouzeixJin",
            "CrouzeixLoristSchwenninger",
            "Crouzeix.LoristSchwenninger.Consequences",
            "Crouzeix.LoristSchwenninger.MainTheorem",
        ),
    ),
}

HARP_ALLOWED_LS_SUPPORT = frozenset(
    {
        "Crouzeix.LoristSchwenninger.BoundaryEmbedding",
        "Crouzeix.LoristSchwenninger.BoundaryMultiplier",
        "Crouzeix.LoristSchwenninger.BoundarySquareRoot",
        "Crouzeix.LoristSchwenninger.CompanionAlgebra",
        "Crouzeix.LoristSchwenninger.CompletedSquare",
        "Crouzeix.LoristSchwenninger.CompressionMoments",
        "Crouzeix.LoristSchwenninger.Dilation",
        "Crouzeix.LoristSchwenninger.NormAttainment",
        "Crouzeix.LoristSchwenninger.PolynomialPowerCauchy",
        "Crouzeix.LoristSchwenninger.Recurrence",
        "Crouzeix.LoristSchwenninger.Scalar",
    }
)


class PreflightError(RuntimeError):
    def __init__(
        self,
        route_id: str,
        aggregate_module: str,
        *,
        toolchain: str | None,
        cache_identity: str,
        aggregate_root: str,
        expected_commands: Iterable[str],
        missing_artifacts: Iterable[str] = (),
        reason: str,
        local_modules: Iterable[str] = (),
    ) -> None:
        super().__init__(reason)
        self.route_id = route_id
        self.aggregate_module = aggregate_module
        self.toolchain = toolchain
        self.cache_identity = cache_identity
        self.aggregate_root = aggregate_root
        self.expected_commands = tuple(expected_commands)
        self.missing_artifacts = tuple(sorted(set(missing_artifacts)))
        self.reason = reason
        self.local_modules = tuple(sorted(set(local_modules)))


class ToolchainError(RuntimeError):
    def __init__(self, reason: str, observed_toolchain: str | None) -> None:
        super().__init__(reason)
        self.observed_toolchain = observed_toolchain


def canonical_json(value: object) -> str:
    return (
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        )
        + "\n"
    )


def canonical_json_sha256(value: object) -> str:
    return hashlib.sha256(canonical_json(value).encode("utf-8")).hexdigest()


def module_relative_path(module: str) -> Path:
    return Path(*module.split(".")).with_suffix(".lean")


def mathlib_source_relative_path(module: str) -> Path:
    if module == "Mathlib":
        return Path("Mathlib.lean")
    return module_relative_path(module)


def artifact_relative_path(module: str) -> Path:
    return Path(*module.split(".")).with_suffix(".olean")


def expected_commands(aggregate_module: str) -> tuple[str, ...]:
    return (f"scripts/check_lean_library.sh {aggregate_module}",)


def aggregate_root_hint(lean_root: Path, aggregate_module: str) -> str:
    return str((lean_root.resolve() / module_relative_path(aggregate_module)).resolve(strict=False))


def ensure_safe_descendant(
    root: Path,
    relative_path: Path,
    *,
    label: str,
    require_exists: bool,
    expect_directory: bool = False,
) -> Path:
    canonical_root = root.resolve()
    candidate = canonical_root / relative_path
    probe = canonical_root
    for part in relative_path.parts:
        probe = probe / part
        if probe.is_symlink():
            raise RuntimeError(f"{label} is a symlink: {probe}")
    if not candidate.exists():
        if require_exists:
            raise FileNotFoundError(candidate)
        return candidate
    resolved = candidate.resolve(strict=True)
    try:
        resolved.relative_to(canonical_root)
    except ValueError as error:
        raise RuntimeError(f"{label} is a symlink escape: {candidate}") from error
    if expect_directory:
        if not resolved.is_dir():
            raise RuntimeError(f"{label} is not a directory: {resolved}")
    elif not resolved.is_file():
        raise RuntimeError(f"{label} is not a file: {resolved}")
    return resolved


def resolve_checked_directory(root: Path, relative_path: Path, *, label: str) -> Path:
    try:
        return ensure_safe_descendant(
            root,
            relative_path,
            label=label,
            require_exists=True,
            expect_directory=True,
        )
    except FileNotFoundError as error:
        raise RuntimeError(f"{label} is missing: {error.filename}") from error


def read_text_file(root: Path, relative_path: Path, *, label: str) -> str:
    try:
        path = ensure_safe_descendant(root, relative_path, label=label, require_exists=True)
    except FileNotFoundError as error:
        raise RuntimeError(f"{label} is missing: {error.filename}") from error
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError as error:
        raise RuntimeError(f"{label} is not valid UTF-8: {path}") from error


def read_json_object(root: Path, relative_path: Path, label: str) -> dict[str, object]:
    source = read_text_file(root, relative_path, label=label)
    try:
        value = json.loads(source)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"{label} is invalid JSON: {root.resolve() / relative_path}") from error
    if not isinstance(value, dict):
        raise RuntimeError(f"{label} must be a JSON object: {root.resolve() / relative_path}")
    return value


def parse_active_imports(source: str, module: str) -> tuple[str, ...]:
    imports: list[str] = []
    comment_depth = 0
    for raw_line in source.splitlines():
        line = raw_line
        clean: list[str] = []
        cursor = 0
        while cursor < len(line):
            pair = line[cursor : cursor + 2]
            if comment_depth > 0:
                if pair == "/-":
                    comment_depth += 1
                    cursor += 2
                elif pair == "-/":
                    comment_depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            elif pair == "/-":
                comment_depth += 1
                cursor += 2
            elif pair == "--":
                break
            else:
                clean.append(line[cursor])
                cursor += 1
        stripped = "".join(clean).strip()
        if not stripped:
            continue
        fields = stripped.split()
        if fields[0] == "import":
            modules = fields[1:]
        elif len(fields) > 1 and fields[0] == "public" and fields[1] == "import":
            modules = fields[2:]
        elif stripped in {"module", "prelude"}:
            continue
        else:
            break
        if not modules:
            raise RuntimeError(f"malformed import in {module}")
        imports.extend(modules)
    if comment_depth != 0:
        raise RuntimeError(f"unterminated block comment in {module}")
    return tuple(imports)


def blocked_error(
    policy: RoutePolicy,
    lean_root: Path,
    cache_identity: str,
    *,
    toolchain: str | None,
    reason: str,
    missing_artifacts: Iterable[str] = (),
    local_modules: Iterable[str] = (),
) -> PreflightError:
    return PreflightError(
        policy.route_id,
        policy.aggregate_module,
        toolchain=toolchain,
        cache_identity=cache_identity,
        aggregate_root=aggregate_root_hint(lean_root, policy.aggregate_module),
        expected_commands=expected_commands(policy.aggregate_module),
        missing_artifacts=missing_artifacts,
        reason=reason,
        local_modules=local_modules,
    )


def is_managed_local_module(module: str) -> bool:
    if module in MANAGED_LOCAL_AGGREGATES:
        return True
    return any(module == root or module.startswith(root + ".") for root in MANAGED_LOCAL_PREFIXES)


def module_matches(module: str, names: Iterable[str]) -> bool:
    for name in names:
        if module == name or module.startswith(name + "."):
            return True
    return False


def is_allowed_local_import(policy: RoutePolicy, module: str) -> bool:
    if module in policy.allowed_exact:
        return True
    if policy.route_id == "harp" and module in HARP_ALLOWED_LS_SUPPORT:
        return True
    return module_matches(module, policy.allowed_prefixes)


def is_rejected_local_import(policy: RoutePolicy, module: str) -> bool:
    if module in policy.rejected_exact:
        return True
    if policy.route_id == "harp" and module.startswith("Crouzeix.LoristSchwenninger"):
        return module not in HARP_ALLOWED_LS_SUPPORT
    return module_matches(module, policy.rejected_prefixes)


def read_local_module(lean_root: Path, module: str) -> str:
    return read_text_file(
        lean_root,
        module_relative_path(module),
        label=f"local module {module}",
    )


def read_mathlib_module(mathlib_root: Path, module: str) -> str:
    return read_text_file(
        mathlib_root,
        mathlib_source_relative_path(module),
        label=f"mathlib source {module}",
    )


def gather_route_closure(
    lean_root: Path,
    policy: RoutePolicy,
    *,
    cache_identity: str,
    toolchain: str | None,
) -> tuple[str, ...]:
    pending = [policy.aggregate_module]
    visited: set[str] = set()
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        try:
            source = read_local_module(lean_root, module)
        except RuntimeError as error:
            reason = str(error)
            if reason.startswith(f"local module {module} is missing"):
                reason = f"missing local import {module}"
            raise blocked_error(
                policy,
                lean_root,
                cache_identity,
                toolchain=toolchain,
                reason=reason,
                local_modules=visited,
            ) from error
        visited.add(module)
        for imported in reversed(parse_active_imports(source, module)):
            if not is_managed_local_module(imported):
                continue
            if is_rejected_local_import(policy, imported) or not is_allowed_local_import(policy, imported):
                raise blocked_error(
                    policy,
                    lean_root,
                    cache_identity,
                    toolchain=toolchain,
                    reason=f"rejected provider import {imported}",
                    local_modules=(*visited, imported),
                )
            if imported not in visited:
                pending.append(imported)
    return tuple(sorted(visited))


def gather_initial_mathlib_modules(lean_root: Path, local_modules: Iterable[str]) -> tuple[str, ...]:
    required: set[str] = set()
    for module in local_modules:
        source = read_local_module(lean_root, module)
        for imported in parse_active_imports(source, module):
            if imported == "Mathlib" or imported.startswith("Mathlib."):
                required.add(imported)
    return tuple(sorted(required))


def gather_mathlib_closure(
    mathlib_root: Path,
    initial_modules: Iterable[str],
) -> tuple[str, ...]:
    pending = list(initial_modules)
    visited: set[str] = set()
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        source = read_mathlib_module(mathlib_root, module)
        visited.add(module)
        for imported in reversed(parse_active_imports(source, module)):
            if imported == "Mathlib" or imported.startswith("Mathlib."):
                if imported not in visited:
                    pending.append(imported)
    return tuple(sorted(visited))


def find_missing_mathlib_artifacts(
    artifact_root: Path,
    mathlib_modules: Iterable[str],
) -> tuple[str, ...]:
    missing: list[str] = []
    canonical_root = artifact_root.resolve()
    for module in mathlib_modules:
        relative = artifact_relative_path(module)
        candidate = ensure_safe_descendant(
            canonical_root,
            relative,
            label=f"mathlib artifact {module}",
            require_exists=False,
        )
        if not candidate.exists():
            missing.append(relative.as_posix())
            continue
        if candidate.read_bytes()[:5] != b"olean":
            missing.append(relative.as_posix())
    return tuple(sorted(missing))


def validate_toolchain(lean_root: Path) -> str:
    try:
        toolchain = read_text_file(lean_root, Path("lean-toolchain"), label="lean-toolchain").strip()
    except RuntimeError as error:
        if str(error).startswith("lean-toolchain is missing"):
            raise ToolchainError(str(error), None) from error
        raise
    if toolchain != PINNED_TOOLCHAIN:
        raise ToolchainError(
            f"lean-toolchain is not pinned to {PINNED_TOOLCHAIN}: {toolchain}",
            toolchain,
        )
    return toolchain


def validate_lake_manifest(lean_root: Path) -> None:
    manifest = read_json_object(lean_root, Path("lake-manifest.json"), "lake-manifest.json")
    if canonical_json_sha256(manifest) != MANIFEST_CONTRACT_SHA256:
        raise RuntimeError("lake-manifest.json contract hash is invalid")


def validate_lakefile(lean_root: Path) -> None:
    try:
        content = read_text_file(lean_root, Path("lakefile.toml"), label="lakefile.toml")
    except RuntimeError as error:
        raise error
    try:
        lakefile = parse_lakefile_contract(content)
    except RuntimeError as error:
        if str(error) == "invalid-lakefile-toml":
            raise RuntimeError(
                f"lakefile.toml is invalid TOML: {lean_root.resolve() / 'lakefile.toml'}"
            ) from error
        raise
    if not isinstance(lakefile, dict):
        raise RuntimeError("lakefile.toml contract is invalid")
    if tuple(lakefile.keys()) != tuple(EXPECTED_LAKEFILE_CONTRACT.keys()):
        raise RuntimeError("lakefile.toml contract is invalid")
    if lakefile.get("name") != EXPECTED_LAKEFILE_CONTRACT["name"]:
        raise RuntimeError("lakefile.toml contract is invalid")
    if lakefile.get("defaultTargets") != EXPECTED_LAKEFILE_CONTRACT["defaultTargets"]:
        raise RuntimeError("lakefile.toml defaultTargets are invalid")
    lean_lib_entries = lakefile.get("lean_lib")
    if not isinstance(lean_lib_entries, list) or any(
        not isinstance(entry, dict) or tuple(entry.keys()) != ("name",) for entry in lean_lib_entries
    ):
        raise RuntimeError("lakefile.toml lean_lib names are invalid")
    if lean_lib_entries != EXPECTED_LAKEFILE_CONTRACT["lean_lib"]:
        raise RuntimeError("lakefile.toml lean_lib names are invalid")
    require_entries = lakefile.get("require")
    if not isinstance(require_entries, list) or any(
        not isinstance(entry, dict) or tuple(entry.keys()) != ("name", "git", "rev")
        for entry in require_entries
    ):
        raise RuntimeError("lakefile.toml require entries are invalid")
    if require_entries != EXPECTED_LAKEFILE_CONTRACT["require"]:
        raise RuntimeError("lakefile.toml require entries are invalid")


def parse_lakefile_contract(source: str) -> dict[str, object]:
    result: dict[str, object] = {}
    current_table: tuple[str, dict[str, object]] | None = None
    lines = source.splitlines()

    def flush_table() -> None:
        nonlocal current_table
        if current_table is None:
            return
        table_name, table_value = current_table
        result.setdefault(table_name, [])
        if not isinstance(result[table_name], list):
            raise RuntimeError("invalid-lakefile-toml")
        result[table_name].append(dict(table_value))
        current_table = None

    index = 0
    while index < len(lines):
        raw_line = lines[index]
        line = raw_line.split("#", 1)[0].strip()
        index += 1
        if not line:
            continue
        if line == "[[lean_lib]]":
            flush_table()
            current_table = ("lean_lib", {})
            continue
        if line == "[[require]]":
            flush_table()
            current_table = ("require", {})
            continue
        if "=" not in line:
            raise RuntimeError("invalid-lakefile-toml")
        key, raw_value = (part.strip() for part in line.split("=", 1))
        if not key:
            raise RuntimeError("invalid-lakefile-toml")
        if raw_value.startswith("["):
            value_lines = [raw_value]
            if not raw_value.endswith("]"):
                while index < len(lines):
                    continuation = lines[index].split("#", 1)[0].strip()
                    index += 1
                    if continuation:
                        value_lines.append(continuation)
                    if continuation.endswith("]"):
                        break
                else:
                    raise RuntimeError("invalid-lakefile-toml")
            normalized = " ".join(part for part in value_lines if part)
            value = parse_string_list_literal(normalized)
        else:
            value = parse_value_literal(raw_value)
        if current_table is None:
            if key in result:
                raise RuntimeError("invalid-lakefile-toml")
            result[key] = value
        else:
            _, table = current_table
            if key in table:
                raise RuntimeError("invalid-lakefile-toml")
            table[key] = value
    flush_table()
    return result


def parse_string_literal(raw_value: str) -> str:
    raw_value = raw_value.strip()
    if len(raw_value) < 2 or not raw_value.startswith('"') or not raw_value.endswith('"'):
        raise RuntimeError("invalid-lakefile-toml")
    inner = raw_value[1:-1]
    if '"' in inner:
        raise RuntimeError("invalid-lakefile-toml")
    return inner


def parse_value_literal(raw_value: str) -> object:
    raw_value = raw_value.strip()
    if raw_value in {"true", "false"}:
        return raw_value == "true"
    return parse_string_literal(raw_value)


def parse_string_list_literal(raw_value: str) -> list[str]:
    raw_value = raw_value.strip()
    if not raw_value.startswith("[") or not raw_value.endswith("]"):
        raise RuntimeError("invalid-lakefile-toml")
    inner = raw_value[1:-1].strip()
    if not inner:
        return []
    values: list[str] = []
    cursor = 0
    while cursor < len(inner):
        while cursor < len(inner) and inner[cursor] in " \t\r\n,":
            cursor += 1
        if cursor >= len(inner):
            break
        if inner[cursor] != '"':
            raise RuntimeError("invalid-lakefile-toml")
        end = cursor + 1
        while end < len(inner) and inner[end] != '"':
            end += 1
        if end >= len(inner):
            raise RuntimeError("invalid-lakefile-toml")
        values.append(inner[cursor + 1 : end])
        cursor = end + 1
        while cursor < len(inner) and inner[cursor].isspace():
            cursor += 1
        if cursor < len(inner):
            if inner[cursor] != ",":
                raise RuntimeError("invalid-lakefile-toml")
            cursor += 1
    return values


def validate_cache_root(
    repository_root: Path,
    approved_cache_root: Path,
) -> tuple[Path, str]:
    lake_root = repository_root / "formalization/lean/.lake"
    if not lake_root.exists():
        raise RuntimeError(f"Lean cache root is missing: {lake_root}")
    actual = lake_root.resolve()
    approved = approved_cache_root.resolve()
    if actual != approved:
        raise RuntimeError(f"Lean cache root does not resolve to approved primary cache: {actual}")
    return actual, DISPLAY_CACHE_ROOT


def validate_mathlib_roots(cache_root: Path) -> tuple[Path, Path]:
    mathlib_root = resolve_checked_directory(
        cache_root,
        Path("packages/mathlib"),
        label="mathlib source root",
    )
    artifact_root = resolve_checked_directory(
        cache_root,
        Path("packages/mathlib/.lake/build/lib/lean"),
        label="mathlib artifact root",
    )
    return mathlib_root, artifact_root


def evaluate_route(repository_root: Path, approved_cache_root: Path, route_id: str) -> PreflightResult:
    policy = ROUTE_POLICIES[route_id]
    lean_root = repository_root / "formalization/lean"
    cache_identity = str(approved_cache_root.resolve())
    display_cache_root = DISPLAY_CACHE_ROOT
    observed_toolchain: str | None = None
    try:
        observed_toolchain = validate_toolchain(lean_root)
        validate_lake_manifest(lean_root)
        validate_lakefile(lean_root)
        actual_cache_root, display_cache_root = validate_cache_root(
            repository_root, approved_cache_root
        )
        cache_identity = str(actual_cache_root)
        mathlib_root, artifact_root = validate_mathlib_roots(actual_cache_root)
        local_modules = gather_route_closure(
            lean_root,
            policy,
            cache_identity=cache_identity,
            toolchain=observed_toolchain,
        )
        required_mathlib_modules = gather_mathlib_closure(
            mathlib_root,
            gather_initial_mathlib_modules(lean_root, local_modules),
        )
        missing_artifacts = find_missing_mathlib_artifacts(
            artifact_root,
            required_mathlib_modules,
        )
    except PreflightError:
        raise
    except ToolchainError as error:
        raise blocked_error(
            policy,
            lean_root,
            cache_identity,
            toolchain=error.observed_toolchain,
            reason=str(error),
        ) from error
    except RuntimeError as error:
        raise blocked_error(
            policy,
            lean_root,
            cache_identity,
            toolchain=observed_toolchain,
            reason=str(error),
        ) from error
    if missing_artifacts:
        raise blocked_error(
            policy,
            lean_root,
            cache_identity,
            toolchain=observed_toolchain,
            missing_artifacts=missing_artifacts,
            reason="missing-mathlib-artifacts",
            local_modules=local_modules,
        )
    aggregate_root = ensure_safe_descendant(
        lean_root,
        module_relative_path(policy.aggregate_module),
        label=f"local module {policy.aggregate_module}",
        require_exists=True,
    )
    return PreflightResult(
        route_id=route_id,
        status="ready",
        aggregate_module=policy.aggregate_module,
        toolchain=observed_toolchain,
        expected_toolchain=PINNED_TOOLCHAIN,
        cache_root=display_cache_root,
        cache_identity=cache_identity,
        aggregate_root=str(aggregate_root),
        expected_commands=expected_commands(policy.aggregate_module),
        missing_artifacts=(),
        local_modules=local_modules,
    )


def evaluate_all_routes(
    repository_root: Path,
    approved_cache_root: Path,
) -> tuple[PreflightResult, ...]:
    return tuple(
        evaluate_route(repository_root, approved_cache_root, route_id)
        for route_id in ROUTE_ORDER
    )


def blocked_result(error: PreflightError) -> PreflightResult:
    return PreflightResult(
        route_id=error.route_id,
        status="blocked",
        aggregate_module=error.aggregate_module,
        toolchain=error.toolchain,
        expected_toolchain=PINNED_TOOLCHAIN,
        cache_root=DISPLAY_CACHE_ROOT,
        cache_identity=error.cache_identity,
        aggregate_root=error.aggregate_root,
        expected_commands=error.expected_commands,
        missing_artifacts=error.missing_artifacts,
        local_modules=error.local_modules,
        reason=error.reason,
    )


def canonical_repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_primary_checkout_cache(repository_root: Path) -> tuple[Path, Path]:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(repository_root),
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        stderr = result.stderr.strip() or "unknown git error"
        raise RuntimeError(f"git common-dir lookup failed: {stderr}")
    common_dir_output = result.stdout.strip()
    if not common_dir_output:
        raise RuntimeError("git common-dir lookup failed: empty stdout")
    common_dir = Path(common_dir_output)
    if common_dir.name != ".git":
        raise RuntimeError(f"git common-dir is not a .git directory: {common_dir}")
    if not common_dir.is_dir():
        raise RuntimeError(f"git common-dir is missing or not a directory: {common_dir}")
    primary_root = common_dir.parent.resolve()
    expected_paths = (
        primary_root / "formalization/lean/lake-manifest.json",
        primary_root / "formalization/lean/lakefile.toml",
        primary_root / "scripts/check_lean_library.sh",
        primary_root / "crates/harp",
    )
    for expected in expected_paths:
        if not expected.exists():
            raise RuntimeError(f"git primary checkout is malformed: missing {expected}")
    return primary_root, primary_root / "formalization/lean/.lake"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    preflight = subparsers.add_parser("preflight")
    preflight.add_argument("--route", choices=(*ROUTE_ORDER, "all"), required=True)
    validate = subparsers.add_parser("validate")
    validate.add_argument("--route", choices=(*ROUTE_ORDER, "all"), required=True)
    validate.add_argument("--allow-unpublished", action="store_true")
    args = parser.parse_args(argv)

    repository_root = canonical_repository_root()
    if args.command == "validate":
        selected = ROUTE_ORDER if args.route == "all" else (args.route,)
        routes: list[dict[str, object]] = []
        exit_code = 0
        for route_id in selected:
            try:
                result = route_validation.inspect_route(
                    repository_root,
                    route_id,
                    allow_unpublished=args.allow_unpublished,
                )
                payload = {
                    "schema_version": "crouzeix-route-validation/v1",
                    "route_id": result.route_id,
                    "status": result.status,
                    "claim_level": result.claim_level,
                    "manifest_path": result.manifest_path,
                }
                if result.reason is not None:
                    payload["reason"] = result.reason
                if result.status != "complete":
                    exit_code = 1
            except route_validation.RouteValidationError as error:
                payload = {
                    "schema_version": "crouzeix-route-validation/v1",
                    "route_id": route_id,
                    "status": "invalid",
                    "claim_level": "authored",
                    "manifest_path": route_validation.ROUTE_MANIFEST_PATHS[route_id].as_posix(),
                    "reason": str(error),
                }
                exit_code = 1
            routes.append(payload)
        if args.route == "all":
            sys.stdout.write(canonical_json({
                "schema_version": "crouzeix-route-validation/v1",
                "route_id": "all",
                "routes": routes,
            }))
        else:
            sys.stdout.write(canonical_json(routes[0]))
        return exit_code

    if args.command != "preflight":
        parser.error("unknown command")

    try:
        _, approved_cache_root = resolve_primary_checkout_cache(repository_root)
    except RuntimeError as error:
        payload = {
            "schema_version": SCHEMA_VERSION,
            "route_id": args.route,
            "status": "blocked",
            "reason": str(error),
        }
        sys.stdout.write(canonical_json(payload))
        return 1

    if args.route == "all":
        routes: list[dict[str, object]] = []
        exit_code = 0
        for route_id in ROUTE_ORDER:
            try:
                result = evaluate_route(repository_root, approved_cache_root, route_id)
            except PreflightError as error:
                result = blocked_result(error)
                exit_code = 1
            routes.append(result.to_json())
        sys.stdout.write(
            canonical_json(
                {
                    "schema_version": SCHEMA_VERSION,
                    "route_id": "all",
                    "routes": routes,
                }
            )
        )
        return exit_code

    try:
        result = evaluate_route(repository_root, approved_cache_root, args.route)
    except PreflightError as error:
        sys.stdout.write(canonical_json(blocked_result(error).to_json()))
        return 1
    sys.stdout.write(canonical_json(result.to_json()))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
