# Lean Hermetic Proof-Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local hermetic Lean proof-suite gate that can validate a pinned proof-suite manifest, execute fixture Lean checks through a controlled runner, and emit typed receipts before Crouzeix/Jin validation or training data depends on Lean.

**Architecture:** Add a new `lean_suite` lab module with strict lock, manifest, runner, inventory, and receipt validation. Keep default verification provider-free and network-free by using fake Lean/Lake tools and Harp-owned fixture suites; optional real Lean runs are explicit CLI actions that return typed `blocked` when the local toolchain or artifacts are missing.

**Tech Stack:** Python 3 standard library, existing `protocol` validation helpers, `unittest`, fixture shell scripts, Harp repository verifier.

---

## Invariant Map

- **No proof-progress authority:** this gate does not claim Crouzeix proof progress, Jin route progress, strict expert results, admissions, mathematical nodes, independent evaluations, or formal theorem status.
- **No live providers:** no TRAE/Codex/provider invocation, no proof-agent call, no CPFR-032 root execution, and no CPFR-033 action.
- **No ignored run mutation:** no creation, recovery, validation, or hand editing under `labs/crouzeix_proof_reproduction/.runs`.
- **Closed local authority:** runtime lock, suite manifest, source files, fake tool scripts, outputs, and receipts must be regular files under declared roots; symlinks, traversal, absolute manifest paths, unknown files, and digest drift fail closed.
- **Typed outcomes:** missing toolchain/artifact/resource is `blocked`; Lean executed and rejected proof code or policy is `failed`; exact compile plus exact inventory validation is `passed`.
- **No ambient tool trust:** runner execution uses explicit `argv`, explicit `cwd`, explicit environment allowlist, no shell interpolation, and no mutable `PATH` lookup except where a lock explicitly points to a resolved executable.
- **Training boundary:** no dataset rows, prompts, model-evaluation inputs, or generated candidate corpora are created in this plan.

## File Map

- Create `labs/crouzeix_proof_reproduction/lean_suite.py`: strict JSON loading, runtime lock validation, suite manifest validation, source/artifact inventory, controlled command runner, receipt validation, and CLI.
- Create `labs/crouzeix_proof_reproduction/lean_suite_fixtures/`: tiny Harp-owned Lean/Lake fixture files plus fake Lean/Lake scripts used by tests.
- Create `labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json`: JSON Schema mirror for receipt shape.
- Create `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`: focused unit and integration tests for the new gate.
- Modify `labs/crouzeix_proof_reproduction/tests/test_verify_registration.py`: assert `test_lean_suite.py` is discovered and does not invoke live providers.
- Modify `labs/crouzeix_proof_reproduction/README.md`: document the proof-suite gate and optional local command.
- Modify `docs/workstream/crouzeix-proof-reproduction/tracker.org`: add CPFR-L001 through CPFR-L006 planning/verification entries.
- Modify `docs/import-receipt.md`: refresh final tracked payload digest after all tracked changes settle.

## Task 1: Add Lean Suite Lock And Manifest Validators

**Files:**
- Create: `labs/crouzeix_proof_reproduction/lean_suite.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Claim CPFR-L001 in the tracker**

Add this item near the other repair/follow-up tickets:

```org
** IMPLEMENTING Add Lean Proof-Suite Lock And Manifest Validators :code:tests:ticket:
:PROPERTIES:
:ID: CPFR-L001
:DESIGN_SECTION: Local Hermetic Lean Proof-Suite Gate
:DEPENDS_ON: CPFR-058
:BRANCH: feat/crouzeix-proof-reproduction
:WORKTREE: .worktrees/crouzeix-proof-reproduction
:OWNER: CPFR-L001 implementation subagent
:AGENT_RUN: current worker thread
:MODEL: GPT-5.5
:END:

*** Scope

Add strict Python validators for Lean runtime locks and proof-suite manifests.

*** Non-goals

- No Lean execution.
- No live provider call.
- No CPFR-R018, CPFR-032, or CPFR-033 action.
- No ignored =.runs= mutation.
- No training data.

*** Owned Files

- =labs/crouzeix_proof_reproduction/lean_suite.py=
- =labs/crouzeix_proof_reproduction/tests/test_lean_suite.py=
- this tracker subtree

*** Acceptance Criteria

- Runtime lock validation rejects unknown fields, duplicate JSON keys,
  unsupported schema versions, unsafe executable paths, invalid digests,
  invalid environment allowlists, and invalid command profiles.
- Suite manifest validation rejects unknown fields, duplicate module IDs,
  absolute/traversal paths, undeclared tiers, undeclared command profiles,
  invalid digests, duplicate paths, and malformed module names.
- Canonical digest helpers are deterministic and use existing lab SHA-256
  conventions.

*** Implementation Steps

1. Add red tests for lock validation.
2. Add red tests for manifest validation.
3. Implement minimal validators and canonical digest helpers.
4. Run focused tests.
5. Record verification evidence.

*** Verification Plan

- =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v=
- =python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org=
- =git diff --check=

*** Verification Evidence

- Pending implementation.
```

- [ ] **Step 2: Write red lock tests**

Create `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`:

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import sys


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import lean_suite
import protocol


def write_json(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def digest(data: bytes) -> str:
    return protocol.sha256_bytes(data)


def valid_lock(root: Path) -> dict[str, object]:
    fake_lean = root / "tools" / "fake-lean"
    fake_lean.parent.mkdir(parents=True, exist_ok=True)
    fake_lean.write_text("#!/bin/sh\nprintf 'Lean fake 4.0.0\\n'\n", encoding="utf-8")
    fake_lean.chmod(0o755)
    return {
        "schema_version": "crouzeix-lean-runtime-lock/v1",
        "runtime_id": "fake-lean-runtime",
        "platform": "test-darwin-arm64",
        "lean": {
            "path": "tools/fake-lean",
            "version": "Lean fake 4.0.0",
            "bytes": fake_lean.stat().st_size,
            "sha256": digest(fake_lean.read_bytes()),
        },
        "lake": None,
        "packages": [],
        "command_profiles": [
            {
                "profile_id": "lean-check",
                "argv": ["{lean}", "{module_path}"],
                "timeout_seconds": 10,
                "env": {},
            }
        ],
        "allowed_env": [],
        "created_at_utc": "2026-08-16T12:00:00Z",
    }


class LeanSuiteValidationTests(unittest.TestCase):
    def test_runtime_lock_rejects_unknown_fields_and_ambient_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            lock["unknown"] = True
            with self.assertRaisesRegex(protocol.ValidationError, "fields"):
                lean_suite.validate_runtime_lock(lock, root)

            bad = valid_lock(root)
            bad["lean"] = dict(bad["lean"])
            bad["lean"]["path"] = "/usr/bin/lean"
            with self.assertRaisesRegex(protocol.ValidationError, "relative"):
                lean_suite.validate_runtime_lock(bad, root)

    def test_runtime_lock_rejects_digest_drift_and_duplicate_json_keys(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock = valid_lock(root)
            lock["lean"] = dict(lock["lean"])
            lock["lean"]["sha256"] = "0" * 64
            with self.assertRaisesRegex(protocol.ValidationError, "lean.sha256"):
                lean_suite.validate_runtime_lock(lock, root)

            duplicate = '{"schema_version":"x","schema_version":"y"}'
            with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
                lean_suite.loads_json_object(duplicate.encode("utf-8"), "runtime lock")
```

- [ ] **Step 3: Run red lock tests**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
```

Expected: import failure for `lean_suite`.

- [ ] **Step 4: Write red manifest tests**

Append to `test_lean_suite.py`:

```python
def valid_manifest() -> dict[str, object]:
    return {
        "schema_version": "crouzeix-lean-suite-manifest/v1",
        "suite_id": "local-smoke-suite",
        "runtime_id": "fake-lean-runtime",
        "modules": [
            {
                "module_id": "tiny-smoke",
                "tier": "tiny_smoke",
                "path": "TinySmoke.lean",
                "module_name": "TinySmoke",
                "command_profile": "lean-check",
                "expected_declarations": ["tiny_smoke"],
                "allowed_axioms": [],
                "source_sha256": "a" * 64,
                "source_bytes": 37,
            }
        ],
        "created_at_utc": "2026-08-16T12:00:00Z",
    }


class LeanSuiteManifestTests(unittest.TestCase):
    def test_manifest_rejects_duplicate_module_ids_and_unsafe_paths(self) -> None:
        manifest = valid_manifest()
        duplicate = dict(manifest)
        duplicate["modules"] = list(manifest["modules"]) + [dict(manifest["modules"][0])]
        with self.assertRaisesRegex(protocol.ValidationError, "duplicate module_id"):
            lean_suite.validate_suite_manifest(duplicate, {"lean-check"})

        unsafe = valid_manifest()
        unsafe["modules"] = [dict(unsafe["modules"][0])]
        unsafe["modules"][0]["path"] = "../Escape.lean"
        with self.assertRaisesRegex(protocol.ValidationError, "module path"):
            lean_suite.validate_suite_manifest(unsafe, {"lean-check"})

    def test_manifest_rejects_unknown_profile_and_bad_tier(self) -> None:
        manifest = valid_manifest()
        with self.assertRaisesRegex(protocol.ValidationError, "command_profile"):
            lean_suite.validate_suite_manifest(manifest, {"other-profile"})

        bad = valid_manifest()
        bad["modules"] = [dict(bad["modules"][0])]
        bad["modules"][0]["tier"] = "jin_target"
        with self.assertRaisesRegex(protocol.ValidationError, "tier"):
            lean_suite.validate_suite_manifest(bad, {"lean-check"})
```

- [ ] **Step 5: Implement minimal validators**

Create `labs/crouzeix_proof_reproduction/lean_suite.py`:

```python
from __future__ import annotations

import json
import re
import stat
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Mapping

import protocol


LOCK_FIELDS = frozenset(
    {
        "schema_version",
        "runtime_id",
        "platform",
        "lean",
        "lake",
        "packages",
        "command_profiles",
        "allowed_env",
        "created_at_utc",
    }
)
TOOL_FIELDS = frozenset({"path", "version", "bytes", "sha256"})
COMMAND_PROFILE_FIELDS = frozenset({"profile_id", "argv", "timeout_seconds", "env"})
MANIFEST_FIELDS = frozenset(
    {"schema_version", "suite_id", "runtime_id", "modules", "created_at_utc"}
)
MODULE_FIELDS = frozenset(
    {
        "module_id",
        "tier",
        "path",
        "module_name",
        "command_profile",
        "expected_declarations",
        "allowed_axioms",
        "source_sha256",
        "source_bytes",
    }
)
TIERS = frozenset({"tiny_smoke", "local_lake", "reference_suite", "target_route"})
SAFE_ID = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
LEAN_MODULE = re.compile(r"^[A-Z][A-Za-z0-9_]*(?:\\.[A-Z][A-Za-z0-9_]*)*$")
MAX_JSON_BYTES = 1024 * 1024


@dataclass(frozen=True)
class ToolIdentity:
    path: str
    version: str
    bytes: int
    sha256: str
    absolute_path: Path


@dataclass(frozen=True)
class CommandProfile:
    profile_id: str
    argv: tuple[str, ...]
    timeout_seconds: int
    env: dict[str, str]


@dataclass(frozen=True)
class RuntimeLock:
    runtime_id: str
    platform: str
    lean: ToolIdentity
    lake: ToolIdentity | None
    command_profiles: dict[str, CommandProfile]


@dataclass(frozen=True)
class SuiteModule:
    module_id: str
    tier: str
    path: str
    module_name: str
    command_profile: str
    expected_declarations: tuple[str, ...]
    allowed_axioms: tuple[str, ...]
    source_sha256: str
    source_bytes: int


@dataclass(frozen=True)
class SuiteManifest:
    suite_id: str
    runtime_id: str
    modules: tuple[SuiteModule, ...]


def loads_json_object(data: bytes, label: str) -> dict[str, object]:
    if len(data) > MAX_JSON_BYTES:
        raise protocol.ValidationError(f"{label} exceeds size limit")
    def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise protocol.ValidationError(f"{label} has duplicate key {key}")
            result[key] = value
        return result
    value = json.loads(data.decode("utf-8"), object_pairs_hook=reject_duplicates)
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must be an object")
    return value


def validate_runtime_lock(value: Mapping[str, Any], root: Path) -> RuntimeLock:
    _require_fields(value, LOCK_FIELDS, "runtime lock")
    _require_equal(value["schema_version"], "crouzeix-lean-runtime-lock/v1", "schema_version")
    runtime_id = _safe_id(value["runtime_id"], "runtime_id")
    platform = _bounded_string(value["platform"], "platform", 1, 128)
    lean = _validate_tool(value["lean"], root, "lean")
    lake_value = value["lake"]
    lake = None if lake_value is None else _validate_tool(lake_value, root, "lake")
    profiles = _validate_command_profiles(value["command_profiles"])
    _string_list(value["allowed_env"], "allowed_env", maximum=64)
    _timestamp(value["created_at_utc"], "created_at_utc")
    packages = value["packages"]
    if not isinstance(packages, list):
        raise protocol.ValidationError("packages must be a list")
    return RuntimeLock(runtime_id=runtime_id, platform=platform, lean=lean, lake=lake, command_profiles=profiles)


def validate_suite_manifest(value: Mapping[str, Any], command_profiles: set[str]) -> SuiteManifest:
    _require_fields(value, MANIFEST_FIELDS, "suite manifest")
    _require_equal(value["schema_version"], "crouzeix-lean-suite-manifest/v1", "schema_version")
    suite_id = _safe_id(value["suite_id"], "suite_id")
    runtime_id = _safe_id(value["runtime_id"], "runtime_id")
    _timestamp(value["created_at_utc"], "created_at_utc")
    raw_modules = value["modules"]
    if not isinstance(raw_modules, list) or not raw_modules:
        raise protocol.ValidationError("modules must be a non-empty list")
    modules: list[SuiteModule] = []
    seen_ids: set[str] = set()
    seen_paths: set[str] = set()
    for raw in raw_modules:
        module = _validate_module(raw, command_profiles)
        if module.module_id in seen_ids:
            raise protocol.ValidationError(f"duplicate module_id {module.module_id}")
        if module.path in seen_paths:
            raise protocol.ValidationError(f"duplicate module path {module.path}")
        seen_ids.add(module.module_id)
        seen_paths.add(module.path)
        modules.append(module)
    return SuiteManifest(suite_id=suite_id, runtime_id=runtime_id, modules=tuple(modules))


def canonical_sha256(value: Mapping[str, Any]) -> str:
    return protocol.sha256_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8"))
```

Also add helpers used above:

```python
def _validate_tool(value: Any, root: Path, label: str) -> ToolIdentity:
    mapping = _mapping(value, label)
    _require_fields(mapping, TOOL_FIELDS, label)
    path = _safe_relative_path(mapping["path"], f"{label}.path")
    absolute = root / path
    _reject_symlink_ancestors(absolute, f"{label}.path")
    if not absolute.is_file():
        raise protocol.ValidationError(f"{label}.path must be a regular file")
    mode = absolute.stat().st_mode
    if not (mode & stat.S_IXUSR):
        raise protocol.ValidationError(f"{label}.path must be executable")
    expected_bytes = _integer(mapping["bytes"], f"{label}.bytes", 1, 1 << 40)
    data = absolute.read_bytes()
    if len(data) != expected_bytes:
        raise protocol.ValidationError(f"{label}.bytes mismatch")
    expected_sha = _digest(mapping["sha256"], f"{label}.sha256")
    if protocol.sha256_bytes(data) != expected_sha:
        raise protocol.ValidationError(f"{label}.sha256 mismatch")
    return ToolIdentity(path=path, version=_bounded_string(mapping["version"], f"{label}.version", 1, 256), bytes=expected_bytes, sha256=expected_sha, absolute_path=absolute)


def _validate_command_profiles(value: Any) -> dict[str, CommandProfile]:
    if not isinstance(value, list) or not value:
        raise protocol.ValidationError("command_profiles must be a non-empty list")
    profiles: dict[str, CommandProfile] = {}
    for raw in value:
        mapping = _mapping(raw, "command profile")
        _require_fields(mapping, COMMAND_PROFILE_FIELDS, "command profile")
        profile_id = _safe_id(mapping["profile_id"], "profile_id")
        if profile_id in profiles:
            raise protocol.ValidationError(f"duplicate profile_id {profile_id}")
        argv = tuple(_string_list(mapping["argv"], "argv", maximum=32))
        if not argv:
            raise protocol.ValidationError("argv must be non-empty")
        env = mapping["env"]
        if not isinstance(env, dict) or any(not isinstance(k, str) or not isinstance(v, str) for k, v in env.items()):
            raise protocol.ValidationError("env must be an object with string values")
        profiles[profile_id] = CommandProfile(profile_id=profile_id, argv=argv, timeout_seconds=_integer(mapping["timeout_seconds"], "timeout_seconds", 1, 3600), env=dict(env))
    return profiles


def _validate_module(value: Any, command_profiles: set[str]) -> SuiteModule:
    mapping = _mapping(value, "module")
    _require_fields(mapping, MODULE_FIELDS, "module")
    module_id = _safe_id(mapping["module_id"], "module_id")
    tier = _enum(mapping["tier"], TIERS, "tier")
    path = _safe_relative_path(mapping["path"], "module path")
    if not path.endswith(".lean"):
        raise protocol.ValidationError("module path must end with .lean")
    module_name = _bounded_string(mapping["module_name"], "module_name", 1, 256)
    if not LEAN_MODULE.fullmatch(module_name):
        raise protocol.ValidationError("module_name is invalid")
    command_profile = _safe_id(mapping["command_profile"], "command_profile")
    if command_profile not in command_profiles:
        raise protocol.ValidationError("command_profile is not declared")
    return SuiteModule(
        module_id=module_id,
        tier=tier,
        path=path,
        module_name=module_name,
        command_profile=command_profile,
        expected_declarations=tuple(_string_list(mapping["expected_declarations"], "expected_declarations", maximum=64)),
        allowed_axioms=tuple(_string_list(mapping["allowed_axioms"], "allowed_axioms", maximum=64)),
        source_sha256=_digest(mapping["source_sha256"], "source_sha256"),
        source_bytes=_integer(mapping["source_bytes"], "source_bytes", 0, 1 << 30),
    )
```

And the primitive helpers:

```python
def _require_fields(value: Mapping[str, Any], fields: frozenset[str], label: str) -> None:
    actual = set(value)
    if actual != fields:
        raise protocol.ValidationError(f"{label} fields mismatch: expected {sorted(fields)}, got {sorted(actual)}")


def _require_equal(value: Any, expected: str, label: str) -> None:
    if value != expected:
        raise protocol.ValidationError(f"{label} must be {expected}")


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise protocol.ValidationError(f"{label} must be an object")
    return value


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    if not isinstance(value, str) or len(value) < minimum or len(value) > maximum or "\0" in value:
        raise protocol.ValidationError(f"{label} must be a bounded non-NUL string")
    return value


def _safe_id(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if not SAFE_ID.fullmatch(text):
        raise protocol.ValidationError(f"{label} is invalid")
    return text


def _safe_relative_path(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 512)
    pure = PurePosixPath(text)
    if pure.is_absolute() or any(part in {"", ".", ".."} for part in pure.parts):
        raise protocol.ValidationError(f"{label} must be a safe relative path")
    return text


def _reject_symlink_ancestors(path: Path, label: str) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    for part in path.parts[1:] if path.is_absolute() else path.parts:
        current = current / part
        if current.exists() and current.is_symlink():
            raise protocol.ValidationError(f"{label} contains symlink component")


def _digest(value: Any, label: str) -> str:
    if not isinstance(value, str) or len(value) != 64 or any(char not in "0123456789abcdef" for char in value):
        raise protocol.ValidationError(f"{label} must be a SHA-256 hex digest")
    return value


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < minimum or value > maximum:
        raise protocol.ValidationError(f"{label} is out of range")
    return value


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if text not in allowed:
        raise protocol.ValidationError(f"{label} is invalid")
    return text


def _string_list(value: Any, label: str, *, maximum: int, minimum: int = 0) -> list[str]:
    if not isinstance(value, list) or len(value) < minimum or len(value) > maximum:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [_bounded_string(item, f"{label} item", 0, 512) for item in value]


def _timestamp(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 64)
    if not text.endswith("Z") or "T" not in text:
        raise protocol.ValidationError(f"{label} must be an UTC timestamp")
    return text
```

- [ ] **Step 6: Run focused tests green**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
```

Expected: all `test_lean_suite` tests pass.

- [ ] **Step 7: Validate tracker and commit**

Run:

```bash
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
git diff --check
```

Expected: both commands exit 0.

Stage and commit:

```bash
git add labs/crouzeix_proof_reproduction/lean_suite.py \
  labs/crouzeix_proof_reproduction/tests/test_lean_suite.py \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "feat(crouzeix): validate lean proof suite contracts" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

## Task 2: Add Harp-Owned Fixture Suite And Inventory Checks

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/lean_suite.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`
- Create: `labs/crouzeix_proof_reproduction/lean_suite_fixtures/smoke/TinySmoke.lean`
- Create: `labs/crouzeix_proof_reproduction/lean_suite_fixtures/lake/LeanSuiteLake.lean`
- Create: `labs/crouzeix_proof_reproduction/lean_suite_fixtures/lake/lakefile.toml`
- Create: `labs/crouzeix_proof_reproduction/lean_suite_fixtures/lake/lean-toolchain`
- Create: `labs/crouzeix_proof_reproduction/lean_suite_fixtures/tools/fake-lean.py`
- Create: `labs/crouzeix_proof_reproduction/lean_suite_fixtures/tools/fake-lake.py`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Claim CPFR-L002 in the tracker**

Add:

```org
** IMPLEMENTING Add Lean Proof-Suite Fixtures And Inventory Checks :code:tests:ticket:
:PROPERTIES:
:ID: CPFR-L002
:DESIGN_SECTION: Local Hermetic Lean Proof-Suite Gate
:DEPENDS_ON: CPFR-L001
:BRANCH: feat/crouzeix-proof-reproduction
:WORKTREE: .worktrees/crouzeix-proof-reproduction
:OWNER: CPFR-L002 implementation subagent
:AGENT_RUN: current worker thread
:MODEL: GPT-5.5
:END:

*** Scope

Add Harp-owned Lean fixture sources and deterministic source inventory checks.

*** Non-goals

- No real Lean dependency.
- No sibling-worktree import.
- No Crouzeix/Jin target.
- No training data.

*** Owned Files

- =labs/crouzeix_proof_reproduction/lean_suite.py=
- =labs/crouzeix_proof_reproduction/tests/test_lean_suite.py=
- =labs/crouzeix_proof_reproduction/lean_suite_fixtures/=
- this tracker subtree

*** Acceptance Criteria

- Fixture files are Harp-owned and tiny.
- Inventory computation rejects missing, extra, symlinked, and digest-drifted
  source files before execution.
- Fake tools are regular executable files and never call real Lean or Lake.

*** Implementation Steps

1. Add fixture files.
2. Add red inventory tests.
3. Implement inventory checks.
4. Run focused tests.
5. Record verification evidence.

*** Verification Plan

- =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v=
- =git diff --check=

*** Verification Evidence

- Pending implementation.
```

- [ ] **Step 2: Create fixture files**

Create `lean_suite_fixtures/smoke/TinySmoke.lean`:

```lean
theorem tiny_smoke : True := by
  trivial
```

Create `lean_suite_fixtures/lake/LeanSuiteLake.lean`:

```lean
theorem lean_suite_lake_smoke : True := by
  trivial
```

Create `lean_suite_fixtures/lake/lakefile.toml`:

```toml
name = "lean_suite_lake"
version = "0.1.0"
defaultTargets = ["LeanSuiteLake"]
```

Create `lean_suite_fixtures/lake/lean-toolchain`:

```text
leanprover/lean4:v4.0.0
```

Create `lean_suite_fixtures/tools/fake-lean.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import sys


def main() -> int:
    if "--version" in sys.argv:
        print("Lean fake 4.0.0")
        return 0
    if len(sys.argv) != 2:
        print("usage: fake-lean.py <module.lean>", file=sys.stderr)
        return 2
    path = pathlib.Path(sys.argv[1])
    text = path.read_text(encoding="utf-8")
    if "LEAN_SUITE_FAIL" in text:
        print("fake lean: type mismatch", file=sys.stderr)
        return 1
    print(f"fake lean checked {path.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

Create `lean_suite_fixtures/tools/fake-lake.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import sys


def main() -> int:
    if "--version" in sys.argv:
        print("Lake fake 4.0.0")
        return 0
    if sys.argv[1:] != ["build"]:
        print("usage: fake-lake.py build", file=sys.stderr)
        return 2
    if not pathlib.Path("lakefile.toml").is_file():
        print("missing lakefile.toml", file=sys.stderr)
        return 1
    print("fake lake build ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

Mark both fake tools executable:

```bash
chmod +x labs/crouzeix_proof_reproduction/lean_suite_fixtures/tools/fake-lean.py
chmod +x labs/crouzeix_proof_reproduction/lean_suite_fixtures/tools/fake-lake.py
```

- [ ] **Step 3: Add red inventory tests**

Append to `test_lean_suite.py`:

```python
class LeanSuiteInventoryTests(unittest.TestCase):
    def test_source_inventory_rejects_missing_extra_symlink_and_digest_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            manifest = valid_manifest()
            manifest["modules"] = [dict(manifest["modules"][0])]
            manifest["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest["modules"][0]["source_bytes"] = source.stat().st_size
            parsed = lean_suite.validate_suite_manifest(manifest, {"lean-check"})

            inventory = lean_suite.compute_source_inventory(suite, parsed)
            self.assertEqual(inventory["file_count"], 1)

            source.write_text("LEAN_SUITE_FAIL\n", encoding="utf-8")
            with self.assertRaisesRegex(protocol.ValidationError, "source_sha256"):
                lean_suite.compute_source_inventory(suite, parsed)

            source.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            (suite / "Extra.lean").write_text("theorem extra : True := by\n  trivial\n", encoding="utf-8")
            with self.assertRaisesRegex(protocol.ValidationError, "unknown source"):
                lean_suite.compute_source_inventory(suite, parsed)

    def test_source_inventory_rejects_symlinked_source(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            target = root / "target.lean"
            target.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            (suite / "TinySmoke.lean").symlink_to(target)
            manifest = valid_manifest()
            manifest["modules"] = [dict(manifest["modules"][0])]
            manifest["modules"][0]["source_sha256"] = digest(target.read_bytes())
            manifest["modules"][0]["source_bytes"] = target.stat().st_size
            parsed = lean_suite.validate_suite_manifest(manifest, {"lean-check"})

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                lean_suite.compute_source_inventory(suite, parsed)
```

- [ ] **Step 4: Implement source inventory**

Add to `lean_suite.py`:

```python
def compute_source_inventory(suite_root: Path, manifest: SuiteManifest) -> dict[str, object]:
    root = suite_root.expanduser().absolute()
    _reject_symlink_ancestors(root, "suite root")
    if not root.is_dir():
        raise protocol.ValidationError("suite root must be a directory")
    expected = {module.path: module for module in manifest.modules}
    observed: dict[str, dict[str, object]] = {}
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            raise protocol.ValidationError(f"source contains symlink: {relative}")
        if path.is_dir():
            continue
        if relative not in expected:
            raise protocol.ValidationError(f"unknown source file: {relative}")
        module = expected[relative]
        data = path.read_bytes()
        if len(data) != module.source_bytes:
            raise protocol.ValidationError(f"source_bytes mismatch: {relative}")
        sha256 = protocol.sha256_bytes(data)
        if sha256 != module.source_sha256:
            raise protocol.ValidationError(f"source_sha256 mismatch: {relative}")
        observed[relative] = {"path": relative, "bytes": len(data), "sha256": sha256}
    missing = sorted(set(expected) - set(observed))
    if missing:
        raise protocol.ValidationError(f"missing source file: {missing[0]}")
    return {
        "schema_version": "crouzeix-lean-source-inventory/v1",
        "suite_id": manifest.suite_id,
        "file_count": len(observed),
        "files": [observed[key] for key in sorted(observed)],
    }
```

- [ ] **Step 5: Run focused tests green**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

Run:

```bash
git diff --check
git add labs/crouzeix_proof_reproduction/lean_suite.py \
  labs/crouzeix_proof_reproduction/tests/test_lean_suite.py \
  labs/crouzeix_proof_reproduction/lean_suite_fixtures \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "feat(crouzeix): add lean proof suite fixtures" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

## Task 3: Implement Controlled Runner And Typed Receipts

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/lean_suite.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`
- Create: `labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Claim CPFR-L003 in the tracker**

Add:

```org
** IMPLEMENTING Add Lean Proof-Suite Runner And Receipts :code:tests:ticket:
:PROPERTIES:
:ID: CPFR-L003
:DESIGN_SECTION: Local Hermetic Lean Proof-Suite Gate
:DEPENDS_ON: CPFR-L002
:BRANCH: feat/crouzeix-proof-reproduction
:WORKTREE: .worktrees/crouzeix-proof-reproduction
:OWNER: CPFR-L003 implementation subagent
:AGENT_RUN: current worker thread
:MODEL: GPT-5.5
:END:

*** Scope

Execute manifest-declared fixture modules through lock-derived commands and
write strict typed receipts.

*** Non-goals

- No real Lean requirement in default tests.
- No global PATH lookup.
- No shell invocation.
- No Crouzeix/Jin target status.

*** Owned Files

- =labs/crouzeix_proof_reproduction/lean_suite.py=
- =labs/crouzeix_proof_reproduction/tests/test_lean_suite.py=
- =labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json=
- this tracker subtree

*** Acceptance Criteria

- Runner uses explicit argv/cwd/env and rejects output path escapes.
- Receipts distinguish =passed=, =failed=, and =blocked=.
- Receipts bind lock, manifest, source inventory, command, stdout/stderr, and
  module outcomes with recomputable digests.
- Existing receipt destinations are create-only.

*** Implementation Steps

1. Add red runner tests.
2. Add red receipt validation tests.
3. Implement runner and receipt validation.
4. Add schema mirror.
5. Run focused tests.

*** Verification Plan

- =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v=
- =python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json=
- =git diff --check=

*** Verification Evidence

- Pending implementation.
```

- [ ] **Step 2: Add red runner/receipt tests**

Append to `test_lean_suite.py`:

```python
class LeanSuiteRunnerTests(unittest.TestCase):
    def test_runner_records_passed_failed_and_blocked_outcomes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            lock_value = valid_lock(root)
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            lock = lean_suite.validate_runtime_lock(lock_value, root)
            manifest = lean_suite.validate_suite_manifest(manifest_value, set(lock.command_profiles))

            receipt_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-pass",
            )
            receipt = lean_suite.validate_receipt_json(receipt_path)
            self.assertEqual(receipt["outcome"], "passed")
            self.assertEqual(receipt["module_outcomes"][0]["outcome"], "passed")

            source.write_text("LEAN_SUITE_FAIL\n", encoding="utf-8")
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            manifest = lean_suite.validate_suite_manifest(manifest_value, set(lock.command_profiles))
            failed_path = lean_suite.run_suite(
                runtime_lock=lock,
                manifest=manifest,
                suite_root=suite,
                receipt_root=root / "receipts",
                run_id="lean-suite-fail",
            )
            failed = lean_suite.validate_receipt_json(failed_path)
            self.assertEqual(failed["outcome"], "failed")

            missing_lock = valid_lock(root)
            missing_lock["lean"] = dict(missing_lock["lean"])
            missing_lock["lean"]["path"] = "tools/missing-lean"
            with self.assertRaisesRegex(protocol.ValidationError, "regular file"):
                lean_suite.validate_runtime_lock(missing_lock, root)

    def test_runner_rejects_existing_receipt_and_receipt_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            lock = lean_suite.validate_runtime_lock(valid_lock(root), root)
            manifest_value = valid_manifest()
            manifest_value["modules"] = [dict(manifest_value["modules"][0])]
            manifest_value["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest_value["modules"][0]["source_bytes"] = source.stat().st_size
            manifest = lean_suite.validate_suite_manifest(manifest_value, set(lock.command_profiles))
            receipt_root = root / "receipts"
            lean_suite.run_suite(lock, manifest, suite, receipt_root, "lean-suite-pass")
            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                lean_suite.run_suite(lock, manifest, suite, receipt_root, "lean-suite-pass")
            receipt_path = receipt_root / "lean-suite-pass" / "receipt.json"
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            receipt["outcome"] = "blocked"
            receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
            with self.assertRaisesRegex(protocol.ValidationError, "lean_suite_receipt_sha256"):
                lean_suite.validate_receipt_json(receipt_path)
```

- [ ] **Step 3: Implement runner and receipt validation**

Add imports:

```python
import os
import shutil
import subprocess
import tempfile
```

Add constants:

```python
RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "runtime_id",
        "suite_id",
        "runtime_lock_sha256",
        "suite_manifest_sha256",
        "source_inventory_sha256",
        "command_receipts",
        "module_outcomes",
        "outcome",
        "reason",
        "lean_suite_receipt_sha256",
    }
)
COMMAND_RECEIPT_FIELDS = frozenset(
    {"module_id", "argv", "cwd", "env", "exit_code", "stdout_path", "stdout_sha256", "stderr_path", "stderr_sha256"}
)
MODULE_OUTCOME_FIELDS = frozenset({"module_id", "outcome", "reason"})
OUTCOMES = frozenset({"passed", "failed", "blocked"})
```

Add:

```python
def run_suite(
    runtime_lock: RuntimeLock,
    manifest: SuiteManifest,
    suite_root: Path,
    receipt_root: Path,
    run_id: str,
) -> Path:
    safe_run_id = _safe_id(run_id, "run_id")
    destination = receipt_root.expanduser().absolute() / safe_run_id
    _reject_symlink_ancestors(destination, "receipt destination")
    if destination.exists():
        raise protocol.ValidationError(f"receipt destination already exists: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    source_inventory = compute_source_inventory(suite_root, manifest)
    destination.mkdir(mode=0o700)
    command_receipts: list[dict[str, object]] = []
    module_outcomes: list[dict[str, object]] = []
    try:
        for module in manifest.modules:
            profile = runtime_lock.command_profiles[module.command_profile]
            command_receipt, module_outcome = _run_module(runtime_lock, profile, suite_root, destination, module)
            command_receipts.append(command_receipt)
            module_outcomes.append(module_outcome)
        final = _final_outcome(module_outcomes)
        receipt: dict[str, object] = {
            "schema_version": "crouzeix-lean-suite-receipt/v1",
            "run_id": safe_run_id,
            "runtime_id": runtime_lock.runtime_id,
            "suite_id": manifest.suite_id,
            "runtime_lock_sha256": canonical_sha256(_runtime_lock_wire(runtime_lock)),
            "suite_manifest_sha256": canonical_sha256(_suite_manifest_wire(manifest)),
            "source_inventory_sha256": canonical_sha256(source_inventory),
            "command_receipts": command_receipts,
            "module_outcomes": module_outcomes,
            "outcome": final["outcome"],
            "reason": final["reason"],
            "lean_suite_receipt_sha256": "",
        }
        receipt["lean_suite_receipt_sha256"] = canonical_sha256_without_receipt_self(receipt)
        _write_json_create_only(destination / "receipt.json", receipt)
    except BaseException:
        _remove_new_tree(destination)
        raise
    return destination / "receipt.json"
```

Add:

```python
def _run_module(
    runtime_lock: RuntimeLock,
    profile: CommandProfile,
    suite_root: Path,
    receipt_root: Path,
    module: SuiteModule,
) -> tuple[dict[str, object], dict[str, object]]:
    module_dir = receipt_root / "modules" / module.module_id
    module_dir.mkdir(parents=True, mode=0o700)
    stdout_path = module_dir / "stdout.txt"
    stderr_path = module_dir / "stderr.txt"
    argv = [_expand_arg(arg, runtime_lock, module, suite_root) for arg in profile.argv]
    env = {key: value for key, value in profile.env.items()}
    completed = subprocess.run(
        argv,
        cwd=suite_root,
        env=env,
        text=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=profile.timeout_seconds,
        check=False,
    )
    stdout_path.write_bytes(completed.stdout)
    stderr_path.write_bytes(completed.stderr)
    outcome = "passed" if completed.returncode == 0 else "failed"
    reason = "module compiled under locked command" if outcome == "passed" else "lean command returned nonzero exit"
    command_receipt = {
        "module_id": module.module_id,
        "argv": argv,
        "cwd": suite_root.as_posix(),
        "env": env,
        "exit_code": completed.returncode,
        "stdout_path": stdout_path.relative_to(receipt_root).as_posix(),
        "stdout_sha256": protocol.sha256_bytes(completed.stdout),
        "stderr_path": stderr_path.relative_to(receipt_root).as_posix(),
        "stderr_sha256": protocol.sha256_bytes(completed.stderr),
    }
    return command_receipt, {"module_id": module.module_id, "outcome": outcome, "reason": reason}
```

Add helper functions:

```python
def _expand_arg(arg: str, runtime_lock: RuntimeLock, module: SuiteModule, suite_root: Path) -> str:
    if arg == "{lean}":
        return runtime_lock.lean.absolute_path.as_posix()
    if arg == "{lake}":
        if runtime_lock.lake is None:
            raise protocol.ValidationError("command profile references lake but lake is null")
        return runtime_lock.lake.absolute_path.as_posix()
    if arg == "{module_path}":
        return (suite_root / module.path).as_posix()
    if "{" in arg or "}" in arg:
        raise protocol.ValidationError("unknown command template variable")
    return arg


def _final_outcome(module_outcomes: list[dict[str, object]]) -> dict[str, str]:
    for outcome in module_outcomes:
        if outcome["outcome"] == "blocked":
            return {"outcome": "blocked", "reason": str(outcome["reason"])}
    for outcome in module_outcomes:
        if outcome["outcome"] == "failed":
            return {"outcome": "failed", "reason": str(outcome["reason"])}
    return {"outcome": "passed", "reason": "all modules compiled under locked commands"}


def canonical_sha256_without_receipt_self(value: Mapping[str, Any]) -> str:
    payload = dict(value)
    payload.pop("lean_suite_receipt_sha256", None)
    return canonical_sha256(payload)


def validate_receipt_json(path: Path) -> dict[str, object]:
    root = path.expanduser().absolute()
    _reject_symlink_ancestors(root, "receipt")
    receipt = loads_json_object(root.read_bytes(), "lean suite receipt")
    _require_fields(receipt, RECEIPT_FIELDS, "lean suite receipt")
    _require_equal(receipt["schema_version"], "crouzeix-lean-suite-receipt/v1", "schema_version")
    result = dict(receipt)
    result["run_id"] = _safe_id(result["run_id"], "run_id")
    result["runtime_id"] = _safe_id(result["runtime_id"], "runtime_id")
    result["suite_id"] = _safe_id(result["suite_id"], "suite_id")
    result["outcome"] = _enum(result["outcome"], OUTCOMES, "outcome")
    result["reason"] = _bounded_string(result["reason"], "reason", 1, 4096)
    result["runtime_lock_sha256"] = _digest(result["runtime_lock_sha256"], "runtime_lock_sha256")
    result["suite_manifest_sha256"] = _digest(result["suite_manifest_sha256"], "suite_manifest_sha256")
    result["source_inventory_sha256"] = _digest(result["source_inventory_sha256"], "source_inventory_sha256")
    result["lean_suite_receipt_sha256"] = _digest(result["lean_suite_receipt_sha256"], "lean_suite_receipt_sha256")
    _validate_receipt_lists(result)
    if result["lean_suite_receipt_sha256"] != canonical_sha256_without_receipt_self(result):
        raise protocol.ValidationError("lean_suite_receipt_sha256 mismatch")
    return result
```

Add:

```python
def _validate_receipt_lists(receipt: dict[str, object]) -> None:
    commands = receipt["command_receipts"]
    outcomes = receipt["module_outcomes"]
    if not isinstance(commands, list) or not commands:
        raise protocol.ValidationError("command_receipts must be a non-empty list")
    if not isinstance(outcomes, list) or not outcomes:
        raise protocol.ValidationError("module_outcomes must be a non-empty list")
    for command in commands:
        mapping = _mapping(command, "command receipt")
        _require_fields(mapping, COMMAND_RECEIPT_FIELDS, "command receipt")
        _safe_id(mapping["module_id"], "module_id")
        _string_list(mapping["argv"], "argv", minimum=1, maximum=64)
        _bounded_string(mapping["cwd"], "cwd", 1, 4096)
        _mapping(mapping["env"], "env")
        _integer(mapping["exit_code"], "exit_code", -255, 255)
        _safe_relative_path(mapping["stdout_path"], "stdout_path")
        _digest(mapping["stdout_sha256"], "stdout_sha256")
        _safe_relative_path(mapping["stderr_path"], "stderr_path")
        _digest(mapping["stderr_sha256"], "stderr_sha256")
    for outcome in outcomes:
        mapping = _mapping(outcome, "module outcome")
        _require_fields(mapping, MODULE_OUTCOME_FIELDS, "module outcome")
        _safe_id(mapping["module_id"], "module_id")
        _enum(mapping["outcome"], OUTCOMES, "module outcome")
        _bounded_string(mapping["reason"], "reason", 1, 4096)


def _write_json_create_only(path: Path, value: Mapping[str, object]) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    fd = os.open(path, flags, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True)
        handle.write("\n")


def _remove_new_tree(path: Path) -> None:
    if path.exists() and not path.is_symlink():
        shutil.rmtree(path)


def _runtime_lock_wire(lock: RuntimeLock) -> dict[str, object]:
    return {
        "runtime_id": lock.runtime_id,
        "platform": lock.platform,
        "lean": {"path": lock.lean.path, "version": lock.lean.version, "bytes": lock.lean.bytes, "sha256": lock.lean.sha256},
        "lake": None if lock.lake is None else {"path": lock.lake.path, "version": lock.lake.version, "bytes": lock.lake.bytes, "sha256": lock.lake.sha256},
        "command_profiles": {key: {"argv": list(value.argv), "timeout_seconds": value.timeout_seconds, "env": value.env} for key, value in sorted(lock.command_profiles.items())},
    }


def _suite_manifest_wire(manifest: SuiteManifest) -> dict[str, object]:
    return {
        "suite_id": manifest.suite_id,
        "runtime_id": manifest.runtime_id,
        "modules": [
            {
                "module_id": module.module_id,
                "tier": module.tier,
                "path": module.path,
                "module_name": module.module_name,
                "command_profile": module.command_profile,
                "expected_declarations": list(module.expected_declarations),
                "allowed_axioms": list(module.allowed_axioms),
                "source_sha256": module.source_sha256,
                "source_bytes": module.source_bytes,
            }
            for module in manifest.modules
        ],
    }
```

- [ ] **Step 4: Add receipt JSON Schema**

Create `schemas/lean_suite_receipt.schema.json`:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "lean_suite_receipt.schema.json",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "schema_version",
    "run_id",
    "runtime_id",
    "suite_id",
    "runtime_lock_sha256",
    "suite_manifest_sha256",
    "source_inventory_sha256",
    "command_receipts",
    "module_outcomes",
    "outcome",
    "reason",
    "lean_suite_receipt_sha256"
  ],
  "properties": {
    "schema_version": {"const": "crouzeix-lean-suite-receipt/v1"},
    "run_id": {"type": "string"},
    "runtime_id": {"type": "string"},
    "suite_id": {"type": "string"},
    "runtime_lock_sha256": {"$ref": "#/$defs/sha256"},
    "suite_manifest_sha256": {"$ref": "#/$defs/sha256"},
    "source_inventory_sha256": {"$ref": "#/$defs/sha256"},
    "command_receipts": {"type": "array", "minItems": 1},
    "module_outcomes": {"type": "array", "minItems": 1},
    "outcome": {"enum": ["passed", "failed", "blocked"]},
    "reason": {"type": "string", "minLength": 1, "maxLength": 4096},
    "lean_suite_receipt_sha256": {"$ref": "#/$defs/sha256"}
  },
  "$defs": {
    "sha256": {"type": "string", "pattern": "^[0-9a-f]{64}$"}
  }
}
```

- [ ] **Step 5: Run focused tests**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json
```

Expected: tests pass and schema prints valid JSON.

- [ ] **Step 6: Commit**

Run:

```bash
git diff --check
git add labs/crouzeix_proof_reproduction/lean_suite.py \
  labs/crouzeix_proof_reproduction/tests/test_lean_suite.py \
  labs/crouzeix_proof_reproduction/schemas/lean_suite_receipt.schema.json \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "feat(crouzeix): run lean proof suite fixtures" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

## Task 4: Add CLI, README, And Verify Registration

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/lean_suite.py`
- Modify: `labs/crouzeix_proof_reproduction/README.md`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_verify_registration.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Claim CPFR-L004 in the tracker**

Add:

```org
** IMPLEMENTING Add Lean Proof-Suite CLI And Documentation :docs:code:tests:ticket:
:PROPERTIES:
:ID: CPFR-L004
:DESIGN_SECTION: Local Hermetic Lean Proof-Suite Gate
:DEPENDS_ON: CPFR-L003
:BRANCH: feat/crouzeix-proof-reproduction
:WORKTREE: .worktrees/crouzeix-proof-reproduction
:OWNER: CPFR-L004 implementation subagent
:AGENT_RUN: current worker thread
:MODEL: GPT-5.5
:END:

*** Scope

Expose the proof-suite gate through a local CLI and document its safe use.

*** Non-goals

- No global Lean installation.
- No real Crouzeix/Jin run.
- No live providers.
- No ignored =.runs= mutation.

*** Owned Files

- =labs/crouzeix_proof_reproduction/lean_suite.py=
- =labs/crouzeix_proof_reproduction/README.md=
- =labs/crouzeix_proof_reproduction/tests/test_verify_registration.py=
- =labs/crouzeix_proof_reproduction/tests/test_lean_suite.py=
- this tracker subtree

*** Acceptance Criteria

- CLI can validate a lock and manifest.
- CLI can run fixture suites into an explicit receipt root.
- README documents that default tests use fake tools and optional real Lean
  setup returns typed =blocked= when unavailable.
- Verify-registration test includes =test_lean_suite.py=.

*** Implementation Steps

1. Add CLI tests.
2. Implement CLI.
3. Update README.
4. Update verification registration test.
5. Run focused tests.

*** Verification Plan

- =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite labs.crouzeix_proof_reproduction.tests.test_verify_registration -v=
- =git diff --check=

*** Verification Evidence

- Pending implementation.
```

- [ ] **Step 2: Add CLI tests**

Append to `test_lean_suite.py`:

```python
class LeanSuiteCliTests(unittest.TestCase):
    def test_cli_validate_and_run_fixture_suite(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            suite = root / "suite"
            suite.mkdir()
            source = suite / "TinySmoke.lean"
            source.write_text("theorem tiny_smoke : True := by\n  trivial\n", encoding="utf-8")
            lock = valid_lock(root)
            manifest = valid_manifest()
            manifest["modules"] = [dict(manifest["modules"][0])]
            manifest["modules"][0]["source_sha256"] = digest(source.read_bytes())
            manifest["modules"][0]["source_bytes"] = source.stat().st_size
            lock_path = root / "lock.json"
            manifest_path = root / "manifest.json"
            write_json(lock_path, lock)
            write_json(manifest_path, manifest)

            validate = subprocess.run(
                [
                    sys.executable,
                    str(LAB / "lean_suite.py"),
                    "validate",
                    "--lock",
                    str(lock_path),
                    "--manifest",
                    str(manifest_path),
                    "--root",
                    str(root),
                ],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(validate.returncode, 0, validate.stderr)
            self.assertIn("lean suite manifest ok", validate.stdout)

            run = subprocess.run(
                [
                    sys.executable,
                    str(LAB / "lean_suite.py"),
                    "run",
                    "--lock",
                    str(lock_path),
                    "--manifest",
                    str(manifest_path),
                    "--root",
                    str(root),
                    "--suite-root",
                    str(suite),
                    "--receipt-root",
                    str(root / "receipts"),
                    "--run-id",
                    "lean-suite-cli",
                ],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(run.returncode, 0, run.stderr)
            self.assertIn("receipt.json", run.stdout)
```

Also add imports at the top of `test_lean_suite.py`:

```python
import subprocess
```

- [ ] **Step 3: Implement CLI**

Append to `lean_suite.py`:

```python
def read_json_file(path: Path, label: str) -> dict[str, object]:
    _reject_symlink_ancestors(path.expanduser().absolute(), label)
    return loads_json_object(path.read_bytes(), label)


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    validate = sub.add_parser("validate")
    validate.add_argument("--lock", required=True)
    validate.add_argument("--manifest", required=True)
    validate.add_argument("--root", required=True)
    run = sub.add_parser("run")
    run.add_argument("--lock", required=True)
    run.add_argument("--manifest", required=True)
    run.add_argument("--root", required=True)
    run.add_argument("--suite-root", required=True)
    run.add_argument("--receipt-root", required=True)
    run.add_argument("--run-id", required=True)
    args = parser.parse_args()

    root = Path(args.root).expanduser().absolute()
    lock = validate_runtime_lock(read_json_file(Path(args.lock), "runtime lock"), root)
    manifest = validate_suite_manifest(read_json_file(Path(args.manifest), "suite manifest"), set(lock.command_profiles))

    if args.command == "validate":
        print(f"lean suite manifest ok: {manifest.suite_id}")
        return
    receipt = run_suite(
        runtime_lock=lock,
        manifest=manifest,
        suite_root=Path(args.suite_root),
        receipt_root=Path(args.receipt_root),
        run_id=args.run_id,
    )
    print(receipt)


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Update README**

Add this section to `labs/crouzeix_proof_reproduction/README.md` before `## Execute`:

````markdown
## Lean Proof-Suite Gate

The Lean proof-suite gate is a local formal-infrastructure check. It validates
a closed runtime lock and suite manifest, then runs declared Lean modules
through explicit commands into create-only receipts. Default repository tests
use Harp-owned fake Lean/Lake fixtures; they do not require a global Lean
installation and do not claim Crouzeix or Jin proof progress.

Use it only with explicit local paths:

```sh
python3 labs/crouzeix_proof_reproduction/lean_suite.py validate \
  --lock /path/to/lock.json \
  --manifest /path/to/manifest.json \
  --root /path/to/runtime/root
python3 labs/crouzeix_proof_reproduction/lean_suite.py run \
  --lock /path/to/lock.json \
  --manifest /path/to/manifest.json \
  --root /path/to/runtime/root \
  --suite-root /path/to/suite \
  --receipt-root /path/to/receipts \
  --run-id lean-suite-local
```

Missing local toolchain bytes are recorded as a blocked setup condition by the
owning runner; ill-typed Lean code is a failed proof-suite outcome. This gate
does not prepare expert-frontier runs, consume runtime tickets, or run model
providers.
````

- [ ] **Step 5: Update verify registration**

Modify `test_verify_registration.py` to add `"test_lean_suite.py"` to the required test set:

```python
                "test_lean_suite.py",
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite labs.crouzeix_proof_reproduction.tests.test_verify_registration -v
```

Expected: tests pass.

- [ ] **Step 7: Commit**

Run:

```bash
git diff --check
git add labs/crouzeix_proof_reproduction/lean_suite.py \
  labs/crouzeix_proof_reproduction/README.md \
  labs/crouzeix_proof_reproduction/tests/test_verify_registration.py \
  labs/crouzeix_proof_reproduction/tests/test_lean_suite.py \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "feat(crouzeix): expose lean proof suite gate" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

## Task 5: Add Optional Real-Lean Blocked/Ready Probe

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/lean_suite.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_lean_suite.py`
- Modify: `labs/crouzeix_proof_reproduction/README.md`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Claim CPFR-L005 in the tracker**

Add:

```org
** IMPLEMENTING Add Optional Real-Lean Readiness Probe :code:tests:ticket:
:PROPERTIES:
:ID: CPFR-L005
:DESIGN_SECTION: Local Hermetic Lean Proof-Suite Gate
:DEPENDS_ON: CPFR-L004
:BRANCH: feat/crouzeix-proof-reproduction
:WORKTREE: .worktrees/crouzeix-proof-reproduction
:OWNER: CPFR-L005 implementation subagent
:AGENT_RUN: current worker thread
:MODEL: GPT-5.5
:END:

*** Scope

Add an explicit non-default readiness probe that reports whether a caller-
supplied local Lean/Lake toolchain can be locked for future suite runs.

*** Non-goals

- No installer.
- No network fetch.
- No global profile mutation.
- No default verification dependency on real Lean.

*** Owned Files

- =labs/crouzeix_proof_reproduction/lean_suite.py=
- =labs/crouzeix_proof_reproduction/tests/test_lean_suite.py=
- =labs/crouzeix_proof_reproduction/README.md=
- this tracker subtree

*** Acceptance Criteria

- Probe accepts explicit executable paths only.
- Missing executable is =blocked=.
- Version command timeout or nonzero exit is =blocked=.
- Successful probe prints a lock fragment with bytes and SHA-256.

*** Implementation Steps

1. Add red probe tests with fake executables.
2. Implement probe CLI.
3. Document probe as optional.
4. Run focused tests.

*** Verification Plan

- =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v=
- =git diff --check=

*** Verification Evidence

- Pending implementation.
```

- [ ] **Step 2: Add red probe tests**

Append to `test_lean_suite.py`:

```python
class LeanSuiteProbeTests(unittest.TestCase):
    def test_probe_reports_blocked_for_missing_explicit_tool(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            result = lean_suite.probe_tool(root / "missing-lean", "lean", timeout_seconds=1)
            self.assertEqual(result["status"], "blocked")
            self.assertIn("missing", result["reason"])

    def test_probe_reports_ready_for_fake_tool(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            tool = root / "fake-lean"
            tool.write_text("#!/bin/sh\nprintf 'Lean fake 4.0.0\\n'\n", encoding="utf-8")
            tool.chmod(0o755)
            result = lean_suite.probe_tool(tool, "lean", timeout_seconds=1)
            self.assertEqual(result["status"], "ready")
            self.assertEqual(result["version"], "Lean fake 4.0.0")
            self.assertEqual(result["sha256"], digest(tool.read_bytes()))
```

- [ ] **Step 3: Implement probe helper and CLI command**

Add:

```python
def probe_tool(path: Path, tool_name: str, timeout_seconds: int) -> dict[str, object]:
    absolute = path.expanduser().absolute()
    try:
        _reject_symlink_ancestors(absolute, f"{tool_name}.path")
        if not absolute.is_file():
            return {"status": "blocked", "reason": f"{tool_name} missing", "path": absolute.as_posix()}
        if not (absolute.stat().st_mode & stat.S_IXUSR):
            return {"status": "blocked", "reason": f"{tool_name} is not executable", "path": absolute.as_posix()}
        completed = subprocess.run(
            [absolute.as_posix(), "--version"],
            cwd=absolute.parent,
            env={},
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout_seconds,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return {"status": "blocked", "reason": f"{tool_name} version timed out", "path": absolute.as_posix()}
    if completed.returncode != 0:
        return {"status": "blocked", "reason": f"{tool_name} version failed", "path": absolute.as_posix()}
    data = absolute.read_bytes()
    return {
        "status": "ready",
        "path": absolute.as_posix(),
        "version": completed.stdout.strip(),
        "bytes": len(data),
        "sha256": protocol.sha256_bytes(data),
    }
```

Extend `main()`:

```python
    probe = sub.add_parser("probe-tool")
    probe.add_argument("--lean", required=True)
    probe.add_argument("--timeout-seconds", type=int, default=5)
```

Then before lock parsing in `main()`:

```python
    if args.command == "probe-tool":
        print(json.dumps(probe_tool(Path(args.lean), "lean", args.timeout_seconds), indent=2, sort_keys=True))
        return
```

- [ ] **Step 4: Document optional probe**

Add to README after the Lean proof-suite gate command block:

```markdown
To inspect an explicit local Lean binary without mutating global setup:

```sh
python3 labs/crouzeix_proof_reproduction/lean_suite.py probe-tool \
  --lean /absolute/path/to/lean \
  --timeout-seconds 5
```

The probe prints `ready` with version, byte count, and SHA-256, or `blocked`
with the first local setup blocker. It does not install Lean.
```

- [ ] **Step 5: Run focused tests and commit**

Run:

```bash
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
git diff --check
```

Expected: pass.

Commit:

```bash
git add labs/crouzeix_proof_reproduction/lean_suite.py \
  labs/crouzeix_proof_reproduction/tests/test_lean_suite.py \
  labs/crouzeix_proof_reproduction/README.md \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "feat(crouzeix): probe local lean readiness" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

## Task 6: Integrate Release Gate And Close The Lean Suite Milestone

**Files:**
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Claim CPFR-L006 in the tracker**

Add:

```org
** IMPLEMENTING Verify Lean Proof-Suite Gate Milestone :review:tests:ticket:
:PROPERTIES:
:ID: CPFR-L006
:DESIGN_SECTION: Local Hermetic Lean Proof-Suite Gate
:DEPENDS_ON: CPFR-L005
:BRANCH: feat/crouzeix-proof-reproduction
:WORKTREE: .worktrees/crouzeix-proof-reproduction
:OWNER: primary orchestrator
:AGENT_RUN: current thread
:MODEL: GPT-5.5
:END:

*** Scope

Verify and close the local hermetic Lean proof-suite gate milestone.

*** Non-goals

- No Crouzeix/Jin target module.
- No real Lean requirement.
- No training data.
- No live provider call.
- No CPFR-R018, CPFR-032, or CPFR-033 action.

*** Owned Files

- this tracker subtree
- =docs/import-receipt.md=

*** Acceptance Criteria

- Full Crouzeix lab discovery includes the Lean suite tests.
- Repository verifier passes after receipt refresh.
- =mise run verify= passes.
- Tracker records that the next implementation milestone may integrate a
  pinned reference suite or Jin target only after owner approval.

*** Implementation Steps

1. Run focused lab tests.
2. Run tracker validation.
3. Refresh repository receipt.
4. Run full verification gate.
5. Mark CPFR-L001 through CPFR-L006 terminal with evidence.

*** Verification Plan

- =python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v=
- =python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org=
- =cargo run -q -p harp -- repository verify=
- =mise run verify=

*** Verification Evidence

- Pending implementation.
```

- [ ] **Step 2: Run focused lab discovery**

Run:

```bash
python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v
```

Expected: all Crouzeix lab tests pass, including `test_lean_suite.py`.

- [ ] **Step 3: Validate tracker**

Run:

```bash
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
```

Expected: exits 0.

- [ ] **Step 4: Refresh receipt**

Run:

```bash
cargo run -q -p harp -- repository verify
```

If it reports `import receipt payload digest is stale; expected ...`, copy the expected digest into `docs/import-receipt.md`, then rerun:

```bash
cargo run -q -p harp -- repository verify
```

Expected: `repository verified: 521 import rows`.

- [ ] **Step 5: Run full gate**

Run:

```bash
mise run verify
```

Expected: exits 0. Non-fatal cache warnings from `mise` may be recorded, but test/build/repository stages must pass.

- [ ] **Step 6: Mark tickets DONE**

Update CPFR-L001 through CPFR-L006:

- state becomes `DONE`;
- `:COMMIT:` records the corresponding commit for each implementation ticket;
- verification evidence records the command, exit status, and result summary;
- CPFR-L006 follow-up states that adding pinned reference-suite modules or a Jin/Crouzeix target requires a separate approved plan.

- [ ] **Step 7: Commit milestone closeout**

Run:

```bash
git diff --check
git add docs/workstream/crouzeix-proof-reproduction/tracker.org docs/import-receipt.md
git commit -m "docs(crouzeix): verify lean proof suite gate" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected: commit created. No push.

## Plan Self-Review

- **Spec coverage:** Task 1 covers lock/manifest validation; Task 2 covers source inventory and Harp-owned fixtures; Task 3 covers hermetic runner and typed receipts; Task 4 covers CLI and documentation; Task 5 covers optional explicit local Lean readiness; Task 6 covers release-gate verification and milestone closure.
- **Scope calibration:** No task runs providers, mutates ignored `.runs`, starts CPFR-R018/032/033, claims Crouzeix/Jin proof progress, installs global Lean, vendors sibling-worktree bytes, or creates training data.
- **Type consistency:** The plan consistently uses `RuntimeLock`, `SuiteManifest`, `SuiteModule`, `CommandProfile`, `run_suite`, `compute_source_inventory`, `probe_tool`, and `validate_receipt_json`.
- **Execution split:** Each implementation task has a disjoint enough concern for fresh subagents, but all touch `lean_suite.py`; integrate serially in task order.
- **Verification:** Focused tests are required at every task; full lab discovery, repository verification, and `mise run verify` are required only at the closeout gate.
